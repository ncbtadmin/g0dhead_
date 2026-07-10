//! Slice 10 — SC-E05 riders, the ML floor's halt sites (SLICE_10.md §3;
//! the repaired swallows at godhead-ml/src/slave.rs and
//! godhead-ml/src/aggregate.rs). A labor halting after RUNNING ends
//! REFUSED with a persisted RefusalRecord — never a job stranded live,
//! never a swallowed error; a failed refusal write would propagate hard.

use godhead_intake::{Dispatcher, IntakePipe};
use godhead_ml::{aggregate, slave, EndpointError, LexicalEmbedder, MlError, Reasoner, Roster};
use godhead_schemas::{AgentType, Budgets, ConfigTier, JobDraft, JobRecord, JobStatus};
use godhead_store::{PgStore, Store, StoreError};
use semver::Version;
use serde_json::json;
use std::path::PathBuf;
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

fn lexical_roster() -> Roster {
    let mut roster = Roster::new();
    roster.add_embedder(godhead_ml::LEXICAL_ALIAS, Arc::new(LexicalEmbedder));
    roster
}

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
            }
        }
    }
}

/// A live job to hold the fixture's planted lease conflict.
async fn holder_job(store: &PgStore) -> JobRecord {
    let draft = JobDraft {
        agent_type: AgentType::Slave,
        auditor_name: None,
        tier: None,
        input_refs: vec![],
        env_ref: None,
        brief_ref: None,
        endpoint_alias: None,
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 3_600_000,
            max_tool_calls: 10,
            max_tokens: 1,
        },
    };
    let job = store.create_job(&draft).await.expect("create");
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await
        .expect("lease");
    store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await
        .expect("run")
}

/// The persisted refusal for a job — SC-E05's witness.
async fn refusal_of(store: &PgStore, job_id: Uuid) -> (String, String, String) {
    sqlx::query_as("SELECT law, reason, detail FROM refusal_records WHERE job_id = $1")
        .bind(job_id)
        .fetch_one(store.raw_pool())
        .await
        .expect("the RefusalRecord is persisted (SC-E05)")
}

/// SC-E05 (slave site) — an embedding labor that meets a planted lease
/// conflict mid-pass halts after RUNNING: the Slave's job ends REFUSED
/// with a persisted RefusalRecord (XI, LEASE_CONFLICT), the failure is
/// contained per node (reported in the summary, the pass survives), and
/// the node stays in the backlog for the next tick.
#[tokio::test]
async fn sc_e05_no_labor_strands_slave() {
    let Some(store) = store().await else { return };
    let pipe = IntakePipe::new(&store, temp_root()).expect("pipe");
    let node = commit_to_rest(&pipe, "held.md", b"a derivative another hand holds\n").await;

    // The planted conflict: another live job holds the node's lease.
    let holder = holder_job(&store).await;
    store
        .acquire_lease(holder.job_id, node, 60_000)
        .await
        .expect("the holder leases the node");

    let summary = slave::backfill_tick(&store, &lexical_roster(), pipe.data_root(), Some(&[node]))
        .await
        .expect("the pass survives a contained failure");
    assert_eq!(summary.embedded, 0, "nothing embedded under the conflict");
    assert_eq!(summary.failures.len(), 1, "the failure is reported");
    assert_eq!(summary.failures[0].0, node);

    // The Slave's job ends REFUSED on the record — not stranded RUNNING.
    let jobs = store
        .list_jobs_by_input_ref(node)
        .await
        .expect("jobs by input");
    let slave_job = jobs
        .iter()
        .find(|j| j.agent_type == AgentType::Slave && j.status == JobStatus::Refused)
        .expect("the refused Slave job exists (SC-E05)");
    let (law, reason, detail) = refusal_of(&store, slave_job.job_id).await;
    assert_eq!(law, "XI", "a lease conflict cites Law XI");
    assert_eq!(reason, "LEASE_CONFLICT");
    assert!(
        detail.contains("halted after RUNNING"),
        "the record narrates the halt: {detail}"
    );

    // The node remains in the backlog: refusal is compliance, and the next
    // pass will find the labor still owed (SC-M06's surface).
    let backlog = store
        .embedding_backlog(Some(&[node]))
        .await
        .expect("backlog");
    assert!(
        backlog.iter().any(|n| n.node_id == node),
        "the held node stays in the backlog"
    );
}

/// A reasoner that fails on consultation — the endpoint fault fixture.
struct DownReasoner;

#[async_trait::async_trait]
impl Reasoner for DownReasoner {
    async fn weigh(&self, _context: &str) -> Result<f32, EndpointError> {
        Err(EndpointError::Unavailable(
            "the rostered reasoner is down (e05 fixture)".to_string(),
        ))
    }
}

/// SC-E05 (aggregate site) — a consolidation pass whose rostered endpoint
/// fails mid-labor halts after RUNNING: the Aggregator's job ends REFUSED
/// with a persisted RefusalRecord carrying the stage code (VII,
/// VALIDATION_FAILED — ruling G1: an endpoint invocation is not a Law VIII
/// tool call, so the ladder's codes are not borrowed), the detail names
/// ENDPOINT_FAULT, and the error propagates to the caller.
#[tokio::test]
async fn sc_e05_no_labor_strands_aggregate() {
    let Some(store) = store().await else { return };
    let pipe = IntakePipe::new(&store, temp_root()).expect("pipe");
    let text = b"the joins are true and the cathedral stands\n";
    let a = commit_to_rest(&pipe, "creed_a.md", text).await;
    let b = commit_to_rest(&pipe, "creed_b.md", text).await;
    let scope = [a, b];
    slave::backfill_tick(&store, &lexical_roster(), pipe.data_root(), Some(&scope))
        .await
        .expect("backfill");

    // Assisted mode consults the rostered reasoner; the fixture's is down.
    // The prior mode is read first and restored after — the dial is a
    // shared constant (same discipline as SC-M03's own toggling).
    let prior_mode = store
        .get_config("weight_mode")
        .await
        .map(|c| c.value)
        .unwrap_or_else(|_| json!("floor"));
    set_config_retry(
        &store,
        "weight_mode",
        ConfigTier::Operational,
        &json!("assisted"),
    )
    .await;
    let mut roster = lexical_roster();
    roster.add_reasoner("down-reasoner", Arc::new(DownReasoner));
    let category = format!("e05_endpoint_{}", Uuid::now_v7());
    let err = aggregate::consolidate(&store, &roster, &category, &scope).await;
    set_config_retry(&store, "weight_mode", ConfigTier::Operational, &prior_mode).await;
    assert!(
        matches!(err, Err(MlError::Endpoint(_))),
        "the endpoint fault propagates, never swallowed: {err:?}"
    );

    // The Aggregator's job ends REFUSED on the record — not RUNNING.
    let jobs = store
        .list_jobs_by_input_ref(a)
        .await
        .expect("jobs by input");
    let aggregator = jobs
        .iter()
        .find(|j| j.agent_type == AgentType::Aggregator && j.status == JobStatus::Refused)
        .expect("the refused Aggregator job exists (SC-E05)");
    let (law, reason, detail) = refusal_of(&store, aggregator.job_id).await;
    assert_eq!(law, "VII", "the labor rule's law");
    assert_eq!(
        reason, "VALIDATION_FAILED",
        "the stage code — never Law VIII's ladder codes (ruling G1)"
    );
    assert!(
        detail.contains("ENDPOINT_FAULT"),
        "the record names the fault class: {detail}"
    );
}
