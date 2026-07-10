//! Section M — Weights & the ML Floor. SC-M01 … SC-M06.

use godhead_intake::{Dispatcher, IntakePipe};
use godhead_ml::{
    aggregate, rebalance_now, rebalance_tick, slave, Embedder, EndpointError, LexicalEmbedder,
    Reasoner, RebalanceOutcome, Roster,
};
use godhead_schemas::ConfigTier;
use godhead_store::{PgStore, Store, StoreError};
use serde_json::json;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use uuid::Uuid;

fn database_url() -> Option<String> {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        return Some(url);
    }
    let env_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../.env");
    let text = std::fs::read_to_string(env_path).ok()?;
    text.lines().find_map(|line| {
        line.trim()
            .strip_prefix("DATABASE_URL=")
            .map(|rest| rest.trim().to_string())
    })
}

async fn store() -> Option<PgStore> {
    let Some(url) = database_url() else {
        eprintln!("SKIP: DATABASE_URL unset — database-backed criterion NOT exercised");
        return None;
    };
    let mut registry = godhead_intake::registry();
    godhead_ml::register_into(&mut registry);
    Some(
        PgStore::connect(&url, registry)
            .await
            .expect("store connect + migrate"),
    )
}

fn temp_root() -> PathBuf {
    std::env::temp_dir().join(format!("godhead_test_{}", Uuid::now_v7()))
}

/// Commits a file and drives it to at-rest.
async fn commit_to_rest(pipe: &IntakePipe<'_, PgStore>, filename: &str, bytes: &[u8]) -> Uuid {
    let node_id = pipe.commit_file(filename, bytes).await.expect("commit");
    let dispatcher = Dispatcher::new(pipe);
    let scope = [node_id];
    dispatcher.tick_scoped(Some(&scope)).await.expect("tick 1");
    dispatcher.tick_scoped(Some(&scope)).await.expect("tick 2");
    node_id
}

/// CAS-retrying config write — the test database is shared, revisions race.
async fn set_config_retry(store: &PgStore, key: &str, tier: ConfigTier, value: &serde_json::Value) {
    loop {
        match store.get_config(key).await {
            Ok(current) => {
                match store
                    .set_config("test-harness", key, tier, value, Some(current.revision))
                    .await
                {
                    Ok(_) => return,
                    Err(StoreError::StaleRevision { .. }) => {}
                    Err(e) => panic!("config write: {e}"),
                }
            }
            Err(_) => {
                if store
                    .set_config("test-harness", key, tier, value, None)
                    .await
                    .is_ok()
                {
                    return;
                }
                // lost the create race; loop retries as an update
            }
        }
    }
}

/// A counting wrapper over the floor embedder — SC-M05's call assertion.
struct CountingEmbedder {
    calls: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl Embedder for CountingEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EndpointError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        LexicalEmbedder.embed(text).await
    }
}

/// A counting mock reasoner — SC-M03's call assertion. Doubles the floor.
struct CountingReasoner {
    calls: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl Reasoner for CountingReasoner {
    async fn weigh(&self, _context: &str) -> Result<f32, EndpointError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(2.0)
    }
}

fn counting_roster() -> (Roster, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut roster = Roster::new();
    roster.add_embedder(
        godhead_ml::LEXICAL_ALIAS,
        Arc::new(CountingEmbedder {
            calls: Arc::clone(&calls),
        }),
    );
    (roster, calls)
}

/// SC-M01 — ingestion marks recalculation-eligibility and performs zero
/// recalculation; each configured trigger kind executes it. Uses .json
/// nodes (floor bucket "database") — the only test embedding that bucket,
/// so eligibility assertions cannot race the other tests.
/// Serialized (H3(5)): the rebalance_state row for the "database" bucket
/// and the rebalance_trigger constant are global singletons; a
/// nondeterministic gate cannot keep doc 00 §4's commitment.
#[tokio::test]
#[serial_test::serial(rebalance_state)]
async fn sc_m01_triggers() {
    let Some(store) = store().await else { return };
    let root = temp_root();
    let pipe = IntakePipe::new(&store, &root).expect("pipe");
    let (roster, _calls) = counting_roster();
    let category = "database";

    let before = store.rebalance_state(category).await.expect("state");
    let last_recalc_before = before.as_ref().and_then(|s| s.last_recalc_at);

    // Ingestion: embed marks eligibility — and executes nothing.
    let n1 = commit_to_rest(&pipe, "data1.json", b"{\"alpha\": 1, \"beta\": 2}\n").await;
    slave::backfill_tick(&store, &roster, pipe.data_root(), Some(&[n1]))
        .await
        .expect("backfill");
    let state = store
        .rebalance_state(category)
        .await
        .expect("state")
        .expect("marked");
    assert!(state.eligible, "ingestion marks eligibility");
    assert_eq!(
        state.last_recalc_at, last_recalc_before,
        "ingestion performs zero recalculation"
    );

    // Manual: the standing tick never fires; only the human act executes.
    set_config_retry(
        &store,
        "rebalance_trigger",
        ConfigTier::Operational,
        &json!({"kind": "manual"}),
    )
    .await;
    let outcome = rebalance_tick(&store, &roster, category, &[n1])
        .await
        .expect("tick");
    assert_eq!(
        outcome,
        RebalanceOutcome::NotTriggered,
        "manual never auto-fires"
    );
    rebalance_now(&store, &roster, "sovereign", category, &[n1])
        .await
        .expect("rebalance now");
    let state = store
        .rebalance_state(category)
        .await
        .expect("state")
        .expect("exists");
    assert!(!state.eligible, "execution clears eligibility");

    // On-add: eligible ⇒ the tick fires.
    let n2 = commit_to_rest(&pipe, "data2.json", b"{\"alpha\": 1, \"gamma\": 3}\n").await;
    slave::backfill_tick(&store, &roster, pipe.data_root(), Some(&[n2]))
        .await
        .expect("backfill");
    set_config_retry(
        &store,
        "rebalance_trigger",
        ConfigTier::Operational,
        &json!({"kind": "on_add"}),
    )
    .await;
    let outcome = rebalance_tick(&store, &roster, category, &[n1, n2])
        .await
        .expect("tick");
    assert!(
        matches!(outcome, RebalanceOutcome::Executed(_)),
        "on-add fires when eligible"
    );

    // Interval: not yet elapsed ⇒ holds; elapsed ⇒ fires.
    let n3 = commit_to_rest(&pipe, "data3.json", b"{\"delta\": 4}\n").await;
    slave::backfill_tick(&store, &roster, pipe.data_root(), Some(&[n3]))
        .await
        .expect("backfill");
    set_config_retry(
        &store,
        "rebalance_trigger",
        ConfigTier::Operational,
        &json!({"kind": "interval", "ms": 900_000}),
    )
    .await;
    let outcome = rebalance_tick(&store, &roster, category, &[n1, n2, n3])
        .await
        .expect("tick");
    assert_eq!(
        outcome,
        RebalanceOutcome::NotTriggered,
        "interval holds until elapsed"
    );
    set_config_retry(
        &store,
        "rebalance_trigger",
        ConfigTier::Operational,
        &json!({"kind": "interval", "ms": 1}),
    )
    .await;
    let outcome = rebalance_tick(&store, &roster, category, &[n1, n2, n3])
        .await
        .expect("tick");
    assert!(
        matches!(outcome, RebalanceOutcome::Executed(_)),
        "interval fires once elapsed"
    );

    // Restore the default for other runs.
    set_config_retry(
        &store,
        "rebalance_trigger",
        ConfigTier::Operational,
        &json!({"kind": "manual"}),
    )
    .await;
}

/// SC-M02 — below the coherence threshold, weights are inert: present in
/// the records, absent from every consumer query.
#[tokio::test]
async fn sc_m02_weights_inert_below_threshold() {
    let Some(store) = store().await else { return };
    let root = temp_root();
    let pipe = IntakePipe::new(&store, &root).expect("pipe");
    let (roster, _calls) = counting_roster();
    let category = format!("markup_{}", Uuid::now_v7());

    // Law VI.1: with no sovereign threshold ever set, a density evaluation
    // refuses rather than guesses. (Only observable on a fresh database —
    // the sovereign constant persists once set.)
    if store.get_config("coherence_threshold").await.is_err() {
        let err = store.live_weights(&category, None).await;
        assert!(err.is_err(), "no threshold, no evaluation — never a guess");
    }

    let text = b"the cathedral of references stands because every join was true\n";
    let a = commit_to_rest(&pipe, "creed_a.md", text).await;
    let b = commit_to_rest(&pipe, "creed_b.md", text).await;
    let scope = [a, b];
    slave::backfill_tick(&store, &roster, pipe.data_root(), Some(&scope))
        .await
        .expect("backfill");
    let summary = aggregate::consolidate(&store, &roster, &category, &scope)
        .await
        .expect("consolidate");
    assert!(summary.weights_set >= 1, "identical texts must link");

    // Threshold above the cluster's density (two nodes, one link: 0.5):
    // inert. 1.0 is the contract ceiling — the write-side contract (H3(2))
    // holds thresholds to [0, 1], so the old 1000.0 fixture is unwritable
    // by design.
    set_config_retry(
        &store,
        "coherence_threshold",
        ConfigTier::Sovereign,
        &json!(1.0),
    )
    .await;
    let weights = store
        .live_weights(&category, Some(&scope))
        .await
        .expect("evaluation");
    assert!(!weights.live, "below the line");
    assert!(weights.density > 0.0, "density measured");
    assert!(
        weights.weights.is_empty(),
        "inert: no force in any consumer"
    );
    assert!(
        weights.config_rev > 0,
        "the citation is mandatory (Law VI.1)"
    );

    // Threshold below the density: live.
    set_config_retry(
        &store,
        "coherence_threshold",
        ConfigTier::Sovereign,
        &json!(0.01),
    )
    .await;
    let weights = store
        .live_weights(&category, Some(&scope))
        .await
        .expect("evaluation");
    assert!(weights.live, "above the line");
    assert!(!weights.weights.is_empty(), "weights now exert force");
}

/// SC-M03 — the mode dial: the same ingestion completes under assisted and
/// floor modes; floor mode makes zero reasoner calls.
#[tokio::test]
async fn sc_m03_mode_dial() {
    let Some(store) = store().await else { return };
    let root = temp_root();
    let pipe = IntakePipe::new(&store, &root).expect("pipe");
    let (embed_roster, _e) = counting_roster();
    let category = format!("markup_{}", Uuid::now_v7());

    let text = b"grey procedural work, exact and unglamorous, kept true\n";
    let a = commit_to_rest(&pipe, "labor_a.md", text).await;
    let b = commit_to_rest(&pipe, "labor_b.md", text).await;
    let scope = [a, b];
    slave::backfill_tick(&store, &embed_roster, pipe.data_root(), Some(&scope))
        .await
        .expect("backfill");

    // A roster carrying a counting reasoner, for both phases.
    let reasoner_calls = Arc::new(AtomicUsize::new(0));
    let mut reasoner_roster = Roster::new();
    reasoner_roster.add_reasoner(
        "mock-reasoner",
        Arc::new(CountingReasoner {
            calls: Arc::clone(&reasoner_calls),
        }),
    );

    // Floor mode: completes, zero reasoner calls even with one rostered.
    set_config_retry(
        &store,
        "weight_mode",
        ConfigTier::Operational,
        &json!("floor"),
    )
    .await;
    let floor_summary = aggregate::consolidate(&store, &reasoner_roster, &category, &scope)
        .await
        .expect("floor consolidation");
    assert!(floor_summary.weights_set >= 1);
    assert_eq!(
        floor_summary.links_touched, 1,
        "one pair, one draw — the undirected pair is never double-counted"
    );
    assert_eq!(
        floor_summary.reasoner_calls, 0,
        "the floor thinks with no one"
    );
    assert_eq!(reasoner_calls.load(Ordering::SeqCst), 0);
    let floor_weight = store
        .links_by_category(&category, Some(&scope))
        .await
        .expect("links")[0]
        .weight;

    // Assisted mode: same ingestion completes; the reasoner is consulted.
    set_config_retry(
        &store,
        "weight_mode",
        ConfigTier::Operational,
        &json!("assisted"),
    )
    .await;
    let assisted_summary = aggregate::consolidate(&store, &reasoner_roster, &category, &scope)
        .await
        .expect("assisted consolidation");
    assert!(
        assisted_summary.reasoner_calls >= 1,
        "assisted consults the reasoner"
    );
    assert!(reasoner_calls.load(Ordering::SeqCst) >= 1);
    let assisted_weight = store
        .links_by_category(&category, Some(&scope))
        .await
        .expect("links")[0]
        .weight;
    assert!(
        (assisted_weight - 2.0 * floor_weight).abs() < 1e-5,
        "assisted = floor × the reasoner's multiplier (got {assisted_weight} vs floor {floor_weight})"
    );

    // Assisted with an empty roster degrades to the floor (doc 4 §2.4).
    let degraded = aggregate::consolidate(&store, &Roster::new(), &category, &scope)
        .await
        .expect("degraded consolidation completes");
    assert_eq!(degraded.reasoner_calls, 0, "no reasoner rostered ⇒ floor");

    set_config_retry(
        &store,
        "weight_mode",
        ConfigTier::Operational,
        &json!("floor"),
    )
    .await;
}

/// SC-M04 — the empty roster: every stage completes or degrades to its
/// floor with zero crashes. "No model" is routing, not error.
#[tokio::test]
async fn sc_m04_empty_roster() {
    let Some(store) = store().await else { return };
    let root = temp_root();
    let pipe = IntakePipe::new(&store, &root).expect("pipe");
    let empty = Roster::new();
    let category = format!("markup_{}", Uuid::now_v7());

    // Intake is modelless by construction and completes.
    let node = commit_to_rest(&pipe, "alone.md", b"# a file with no minds around\n").await;
    let record = store.get_node(node).await.expect("node");
    assert!(record.normalized, "intake completed");

    // The Slave pass degrades: nothing embedded, nothing crashed.
    let summary = slave::backfill_tick(&store, &empty, pipe.data_root(), Some(&[node]))
        .await
        .expect("backfill completes on an empty roster");
    assert_eq!(
        summary.embedded, 0,
        "no embedder, no embeddings — and no error"
    );
    assert!(summary.failures.is_empty(), "absence is not failure");

    // Consolidation degrades: zero links (nothing embedded), zero calls.
    let summary = aggregate::consolidate(&store, &empty, &category, &[node])
        .await
        .expect("consolidation completes on an empty roster");
    assert_eq!(summary.links_touched, 0);
    assert_eq!(summary.reasoner_calls, 0);

    // The human act still works — floor all the way down.
    rebalance_now(&store, &empty, "sovereign", &category, &[node])
        .await
        .expect("rebalance completes on an empty roster");
}

/// SC-M05 — one persisted embedding per node: a repeat request reads,
/// never recomputes.
#[tokio::test]
async fn sc_m05_embed_once() {
    let Some(store) = store().await else { return };
    let root = temp_root();
    let pipe = IntakePipe::new(&store, &root).expect("pipe");
    let (roster, calls) = counting_roster();

    let node = commit_to_rest(&pipe, "once.md", b"embed me exactly once\n").await;
    let scope = [node];
    let first = slave::backfill_tick(&store, &roster, pipe.data_root(), Some(&scope))
        .await
        .expect("first backfill");
    assert_eq!(first.embedded, 1);
    assert!(first.failures.is_empty());
    assert_eq!(calls.load(Ordering::SeqCst), 1, "one embedder call");
    let embedding = store
        .get_embedding(node)
        .await
        .expect("read")
        .expect("persisted");
    assert_eq!(embedding.dims, 256);
    assert_eq!(embedding.revision, 1);

    // The repeat pass finds no backlog: read, never recomputed.
    let second = slave::backfill_tick(&store, &roster, pipe.data_root(), Some(&scope))
        .await
        .expect("second backfill");
    assert_eq!(second.embedded, 0, "nothing to do");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "embedder call count unchanged"
    );
    let again = store
        .get_embedding(node)
        .await
        .expect("read")
        .expect("still there");
    assert_eq!(again.revision, 1, "the persisted vector is untouched");
    assert_eq!(again.vector, embedding.vector);
}

/// SC-M06 — embedder-down intake: the file rests normalized and linkless,
/// flagged for backfill; later embedding backfills without touching the
/// raw atom.
#[tokio::test]
async fn sc_m06_backfill() {
    let Some(store) = store().await else { return };
    let root = temp_root();
    let pipe = IntakePipe::new(&store, &root).expect("pipe");
    let category = format!("markup_{}", Uuid::now_v7());

    // Intake with the embedder down (absent): completes, rests linkless.
    let node = commit_to_rest(&pipe, "patient.md", b"i can wait for a mind\n").await;
    let record = store.get_node(node).await.expect("node");
    assert!(record.normalized, "rests normalized");
    let sha_before = record.raw_sha256.clone();
    let derivative_before = record.derivative_sha256.clone();
    let links = store
        .links_by_category(&category, Some(&[node]))
        .await
        .expect("links");
    assert!(links.is_empty(), "rests linkless");

    // Flagged for backfill: present in the backlog surface.
    let backlog = store
        .embedding_backlog(Some(&[node]))
        .await
        .expect("backlog");
    assert_eq!(backlog.len(), 1, "the wait is visible, not buried");

    // The embedder returns; backfill embeds without touching the atom.
    let (roster, _calls) = counting_roster();
    let summary = slave::backfill_tick(&store, &roster, pipe.data_root(), Some(&[node]))
        .await
        .expect("backfill");
    assert_eq!(summary.embedded, 1);
    assert!(
        store.get_embedding(node).await.expect("read").is_some(),
        "vector persisted"
    );
    let backlog = store
        .embedding_backlog(Some(&[node]))
        .await
        .expect("backlog");
    assert!(backlog.is_empty(), "backlog cleared");
    let record = store.get_node(node).await.expect("node");
    assert_eq!(record.raw_sha256, sha_before, "the atom untouched");
    assert_eq!(
        record.derivative_sha256, derivative_before,
        "the derivative untouched"
    );
}

/// Regression (slice-4 review): a node whose derivative is unreadable is a
/// contained, per-node failure — the pass continues to younger nodes, and
/// no Slave job is ever stranded live (every failure ends REFUSED or never
/// spawns, and refusal is compliance).
#[tokio::test]
async fn backfill_contains_per_node_failure() {
    let Some(store) = store().await else { return };
    let root = temp_root();
    let pipe = IntakePipe::new(&store, &root).expect("pipe");
    let (roster, _calls) = counting_roster();

    let broken = commit_to_rest(&pipe, "older_broken.md", b"i will lose my derivative\n").await;
    let healthy = commit_to_rest(&pipe, "younger_fine.md", b"i am fine\n").await;
    // Break the older node: delete its derivative from disk out-of-band.
    let broken_record = store.get_node(broken).await.expect("node");
    std::fs::remove_file(root.join(broken_record.derivative_path.expect("derivative")))
        .expect("delete derivative");

    let scope = [broken, healthy];
    let summary = slave::backfill_tick(&store, &roster, pipe.data_root(), Some(&scope))
        .await
        .expect("the pass itself completes");
    assert_eq!(
        summary.embedded, 1,
        "the younger node embedded despite the older failing"
    );
    assert_eq!(
        summary.failures.len(),
        1,
        "the failure is reported, not swallowed"
    );
    assert_eq!(summary.failures[0].0, broken);
    assert!(store.get_embedding(healthy).await.expect("read").is_some());
    assert!(store.get_embedding(broken).await.expect("read").is_none());

    // No stranded jobs: every job this pass touched for either node is
    // terminal (TERMINATED or REFUSED) or never existed.
    for node in [broken, healthy] {
        for job in store.list_jobs_by_input_ref(node).await.expect("jobs") {
            assert!(
                job.status.is_terminal(),
                "job {} for node {node} left {} — stranded",
                job.job_id,
                job.status
            );
        }
    }
}
