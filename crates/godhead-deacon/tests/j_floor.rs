//! The J-floor — Section J's substrate at the Deacon's threshold
//! (docs/dev/SLICE_10.md §1; docs/08 as amended 2026-07-09).
//!
//! SC-J01 (mandates human-authored, C.4), SC-J02 (writ concreteness, the
//! sovereign criterion), SC-J09's substrate half (chain grammar +
//! append-in-flight, Handbook §4.2 / C.2), the frozen-record walls, the
//! SC-C07 threshold entry (IV.4, narrowed by ruling G6), and SC-L01's
//! items half (Return item resolution — SLICE_09 §6 finding 7's pin).
//!
//! Fixtures that must stand where the fetch layer will (no fetch exists —
//! the no-HTTP wall) deposit through the lawful store surfaces; rows the
//! lawful surfaces refuse to mint are planted out-of-band on the raw pool,
//! authenticating with `SET LOCAL godhead.actor_class` inside their own
//! transaction where a class-guarded table demands it (ruling G10) — never
//! by disabling a trigger.

use godhead_intake::IntakePipe;
use godhead_schemas::{
    AcceptanceCriterion, AgentType, Budgets, CapabilityAction, ChainEntryDraft, ChainEntryKind,
    CompletionEntry, ConsentDecision, ConsentScope, EnvKind, InstructionDraft, JobDraft, JobRecord,
    JobStatus, Locator, MandateDemands, MandateDraft, MandateKind, PairingKind, QuarantineDraft,
    ReturnDraft, ReturnItem, ReturnItemKind, Step, TestableAs, Tier, WritTarget,
};
use godhead_store::{PgStore, Store, StoreError};
use semver::Version;
use serde_json::json;
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
    Some(
        PgStore::connect(&url, godhead_intake::registry())
            .await
            .expect("store connect + migrate"),
    )
}

fn job_draft(agent_type: AgentType) -> JobDraft {
    JobDraft {
        agent_type,
        auditor_name: None,
        tier: Some(Tier::Devout),
        input_refs: vec![],
        env_ref: None,
        brief_ref: None,
        endpoint_alias: None,
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 120_000,
            max_tool_calls: 10,
            max_tokens: 1,
        },
    }
}

/// A job advanced PENDING → LEASED → RUNNING.
async fn run(store: &PgStore, draft: &JobDraft) -> JobRecord {
    let job = store.create_job(draft).await.expect("create job");
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await
        .expect("to LEASED");
    store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await
        .expect("to RUNNING")
}

async fn running_job(store: &PgStore, agent_type: AgentType) -> JobRecord {
    run(store, &job_draft(agent_type)).await
}

/// A mandate-trip job: the fetching labor under the mandate's own human
/// charter (§1.4). `create_job` does not validate `brief_ref` — the chain
/// and deposit walls do, which is exactly what these tests prove.
async fn trip_job(store: &PgStore, mandate: Uuid) -> JobRecord {
    let mut draft = job_draft(AgentType::Student);
    draft.brief_ref = Some(mandate);
    draft.input_refs = vec![mandate];
    run(store, &draft).await
}

/// Plants a CARDINAL matrix out-of-band. Birth must be POSTULANT (the
/// substrate permits no other), so the fixture lays a raw COMMIT proposal
/// and a GRANTED consent for the commitment-chain trigger (Law VI.3) to
/// verify before the status advances. The consent row is class-guarded
/// (ruling G10): the fixture authenticates as 'sovereign' inside its own
/// transaction — the planted-row pattern, never a disabled trigger.
async fn plant_cardinal_matrix(store: &PgStore) -> Uuid {
    let job = running_job(store, AgentType::Slave).await;
    let matrix = Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO matrices
             (matrix_id, category, node_refs, link_refs, emerged_by, config_rev,
              schema_name, schema_version, produced_by)
           VALUES ($1, $2, '[]', '[]', $3, 1, 'MatrixRecord', '1.0.0', $3::text)"#,
    )
    .bind(matrix)
    .bind(format!("jfloor_{}", Uuid::now_v7()))
    .bind(job.job_id)
    .execute(store.raw_pool())
    .await
    .expect("plant postulant");
    let proposal = Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO joint_proposals
             (proposal_id, job_id, matrix_ref, matrix_revision, report_refs, verdict,
              schema_name, schema_version, produced_by)
           VALUES ($1, $2, $3, 1, '[]', 'COMMIT', 'JointProposal', '1.0.0', $2::text)"#,
    )
    .bind(proposal)
    .bind(job.job_id)
    .bind(matrix)
    .execute(store.raw_pool())
    .await
    .expect("plant proposal");
    let consent = Uuid::now_v7();
    let mut tx = store.raw_pool().begin().await.expect("tx");
    sqlx::query("SET LOCAL godhead.actor_class = 'sovereign'")
        .execute(&mut *tx)
        .await
        .expect("authenticate fixture");
    sqlx::query(
        r#"INSERT INTO consent_records
             (consent_id, subject_ref, decision, scope, decided_by,
              schema_name, schema_version, produced_by)
           VALUES ($1, $2, 'GRANTED', 'ITEM', 'sovereign', 'ConsentRecord', '1.0.0', 'sovereign')"#,
    )
    .bind(consent)
    .bind(proposal)
    .execute(&mut *tx)
    .await
    .expect("plant consent");
    tx.commit().await.expect("commit consent");
    sqlx::query(
        r#"UPDATE matrices
           SET status = 'CARDINAL', committed_proposal_ref = $2, committed_consent_ref = $3
           WHERE matrix_id = $1"#,
    )
    .bind(matrix)
    .bind(proposal)
    .bind(consent)
    .execute(store.raw_pool())
    .await
    .expect("advance to CARDINAL");
    matrix
}

fn uri(value: &str) -> WritTarget {
    WritTarget {
        locator: Locator::Uri(value.to_string()),
        note: None,
    }
}

/// A writ over the given matrix (C.4: a writ names a matrix and typed
/// locator demands).
fn writ(matrix: Uuid, targets: Vec<WritTarget>) -> MandateDraft {
    MandateDraft {
        kind: MandateKind::Writ,
        teacher_env_ref: None,
        matrix_ref: Some(matrix),
        demands: MandateDemands::WritTargets(targets),
        trip_budget: json!({ "max_trips": 1 }),
    }
}

/// One lawfully authored writ (the human hand, SC-J01's positive half).
async fn authored_writ(store: &PgStore, matrix: Uuid) -> Uuid {
    store
        .author_mandate(
            "sovereign",
            &writ(matrix, vec![uri("https://example.org/spec")]),
        )
        .await
        .expect("human authorship")
        .mandate_id
}

fn quarantine_draft(mandate: Uuid) -> QuarantineDraft {
    QuarantineDraft {
        mandate_ref: Some(mandate),
        brief_ref: None,
        filename: "arrival.txt".to_string(),
        declared_type: "text/plain".to_string(),
        content: b"external bytes, held at the threshold".to_vec(),
    }
}

/// Lands one item at the threshold the lawful way: the trip appends its
/// WRIT root and a FETCH entry PROMISING the item — append-in-flight,
/// §4.2 — then deposits. Returns the item_ref, which is also the
/// chain_ref: the chain narrates this item's arrival (C.2).
async fn lawful_deposit(store: &PgStore, mandate: Uuid, trip: &JobRecord) -> Uuid {
    let item_ref = Uuid::now_v7();
    store
        .append_chain_entry(
            trip.job_id,
            &ChainEntryDraft {
                chain_ref: item_ref,
                kind: ChainEntryKind::Writ,
                mandate_ref: Some(mandate),
                prompt_or_reason: "the writ that sent the trip (C.4)".to_string(),
                produced: vec![],
            },
        )
        .await
        .expect("WRIT root");
    store
        .append_chain_entry(
            trip.job_id,
            &ChainEntryDraft {
                chain_ref: item_ref,
                kind: ChainEntryKind::Fetch,
                mandate_ref: None,
                prompt_or_reason: "fetched the named locator (SC-J09)".to_string(),
                produced: vec![item_ref],
            },
        )
        .await
        .expect("FETCH entry");
    store
        .quarantine_deposit(trip.job_id, item_ref, &quarantine_draft(mandate))
        .await
        .expect("deposit behind a standing chain");
    item_ref
}

async fn mandates_for(store: &PgStore, matrix: Uuid) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM mandates WHERE matrix_ref = $1")
        .bind(matrix)
        .fetch_one(store.raw_pool())
        .await
        .expect("count mandates")
}

/// A raw mandate INSERT: what an agent (or anything below the API) would
/// have to write. `$3` is the produced_by stamp under test.
const RAW_MANDATE_INSERT: &str = r#"INSERT INTO mandates
     (mandate_id, kind, matrix_ref, demands, trip_budget,
      schema_name, schema_version, produced_by)
   VALUES ($1, 'WRIT', $2, '[]', '{}', 'MandateRecord', '1.0.0', $3)"#;

/// A raw admission-consent INSERT (`$3` stamps both decided_by and
/// produced_by — the strings under test).
const RAW_CONSENT_INSERT: &str = r#"INSERT INTO consent_records
     (consent_id, subject_ref, decision, scope, decided_by,
      schema_name, schema_version, produced_by)
   VALUES ($1, $2, 'ADMITTED', 'ITEM', $3, 'ConsentRecord', '1.0.0', $3)"#;

/// SC-J01 — a MandateRecord written under any agent identity is rejected
/// at the store: mandates are human-authored by construction (C.4; IV.4).
///
/// Above the API the impossibility is the signature's:
/// `author_mandate(&self, actor: &str, draft: &MandateDraft)` takes a
/// human actor string and NO job identity, and it is the ONLY mandate
/// write the Store trait exposes — a job-identified mandate write is not
/// rejected at runtime, it is unspellable (SC-C07's claims-by-signature
/// doctrine). Below the API, two independent walls hold: the agent-author
/// trigger (a UUID-shaped produced_by IS an agent, Law IV.4) and the
/// actor-class trigger (no class credential means no entry whatever the
/// stamp — ruling G10).
#[tokio::test]
async fn sc_j01_mandates_human_authored() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;

    // The human hand authors; the charter stands and reads back.
    let record = store
        .author_mandate(
            "sovereign",
            &writ(matrix, vec![uri("https://example.org/spec")]),
        )
        .await
        .expect("the sovereign authors");
    assert_eq!(
        record.envelope.produced_by, "sovereign",
        "the record bears the human hand"
    );
    let read = store
        .get_mandate(record.mandate_id)
        .await
        .expect("read back");
    assert_eq!(read.mandate_id, record.mandate_id);

    // The agent-author wall: a job-UUID produced_by is a gate bypass even
    // on a path that authenticated as sovereign — the class credential
    // does not launder an agent identity (Law IV.4; F1's office-shaped
    // hole, closed).
    let job = running_job(&store, AgentType::Slave).await;
    let mut tx = store.raw_pool().begin().await.expect("tx");
    sqlx::query("SET LOCAL godhead.actor_class = 'sovereign'")
        .execute(&mut *tx)
        .await
        .expect("class set");
    let err = sqlx::query(RAW_MANDATE_INSERT)
        .bind(Uuid::now_v7())
        .bind(matrix)
        .bind(job.job_id.to_string())
        .execute(&mut *tx)
        .await
        .expect_err("an agent identity never writes a mandate");
    assert!(
        err.to_string().contains("GATE_BYPASS_ATTEMPT"),
        "the wall names the bypass: {err}"
    );
    tx.rollback().await.expect("rollback");

    // Ruling G10: even a human-SHAPED author string is rejected when the
    // path did not authenticate as any class — below the API, 'sovereign'
    // is just a string.
    let err = sqlx::query(RAW_MANDATE_INSERT)
        .bind(Uuid::now_v7())
        .bind(matrix)
        .bind("sovereign")
        .execute(store.raw_pool())
        .await
        .expect_err("an unauthenticated path never writes a mandate");
    assert!(
        err.to_string().contains("GATE_BYPASS_ATTEMPT"),
        "G10 names it: {err}"
    );

    // Exactly one mandate stands for this matrix — the human-authored
    // one; neither rejected write left a row.
    assert_eq!(mandates_for(&store, matrix).await, 1);
}

/// SC-J02 — writ concreteness (the sovereign criterion; C.4): a demand
/// that is query-shaped or unresolvable fails AT AUTHORSHIP, before any
/// trip. The six adversarial fixtures from the criterion's text are each
/// rejected; a writ of named, resolvable locators validates; and a mixed
/// writ is rejected WHOLE — no partial mandate persists. This criterion IS
/// the enforced boundary between the writ system and the deferred breadth
/// system.
#[tokio::test]
async fn sc_j02_writ_concreteness() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;

    // Fixtures 1–4 and 6, shape-rejected: a writ says fetch THESE
    // sources, never find-things-about.
    let adversarial: Vec<(&str, WritTarget)> = vec![
        // 1. Query-shaped prose.
        (
            "a query wearing a locator's field",
            uri("find things about X"),
        ),
        // 2. The bare topic, as the criterion writes it (its space trips
        //    the query-shape rule) …
        ("a bare topic string, as written", uri("quantum computing")),
        //    … and space-free, proving bare topics are rejected on their
        //    own terms (no scheme), not only as prose.
        ("a bare topic string, space-free", uri("quantum-computing")),
        // 3. An empty locator names nothing.
        ("an empty locator", uri("")),
        // 4. A malformed URI is a query wearing a locator's field.
        ("a malformed URI", uri("http:/half")),
        // 6. Search operators, both spellings from the criterion text.
        ("boolean search operators", uri("https://ex.com/a OR b")),
        ("a wildcard", uri("https://ex.com/a*b")),
    ];
    for (what, target) in adversarial {
        let err = store
            .author_mandate("sovereign", &writ(matrix, vec![target]))
            .await;
        assert!(
            matches!(err, Err(StoreError::ValidationFailed(_))),
            "{what} must fail at authorship: {err:?}"
        );
    }

    // 5. An unknown source_id: a well-formed token the known-source
    // registry does not carry (seeded EMPTY in v1 — migration 0013; a
    // unique token keeps the fixture honest on the shared database).
    // Resolution fails at authorship, never on a trip.
    let unknown = format!("src-{}", Uuid::now_v7().simple());
    let err = store
        .author_mandate(
            "sovereign",
            &writ(
                matrix,
                vec![WritTarget {
                    locator: Locator::SourceId(unknown),
                    note: None,
                }],
            ),
        )
        .await;
    match err {
        Err(StoreError::ValidationFailed(detail)) => assert!(
            detail.contains("unknown source_id"),
            "the failure names the unresolved token: {detail}"
        ),
        other => panic!("an unknown source_id must fail at authorship: {other:?}"),
    }

    // Nothing persisted from any rejection.
    assert_eq!(mandates_for(&store, matrix).await, 0);

    // A writ of named, resolvable locators validates.
    store
        .author_mandate(
            "sovereign",
            &writ(
                matrix,
                vec![uri("https://example.org/a"), uri("https://example.org/b")],
            ),
        )
        .await
        .expect("named resolvable locators validate");
    assert_eq!(mandates_for(&store, matrix).await, 1);

    // The mixed writ: five good demands and one query-shaped — rejected
    // whole. A mandate is a charter, not a sieve.
    let err = store
        .author_mandate(
            "sovereign",
            &writ(
                matrix,
                vec![
                    uri("https://example.org/1"),
                    uri("https://example.org/2"),
                    uri("https://example.org/3"),
                    uri("https://example.org/4"),
                    uri("https://example.org/5"),
                    uri("find things about X"),
                ],
            ),
        )
        .await;
    assert!(
        matches!(err, Err(StoreError::ValidationFailed(_))),
        "the mixed writ fails whole: {err:?}"
    );
    assert_eq!(
        mandates_for(&store, matrix).await,
        1,
        "no partial mandate persists (SC-J02)"
    );
}

/// SC-J09, the substrate half — the ProvenanceChain's grammar and the
/// append-in-flight wall (Handbook §4.2; C.2): roots are CANON|WRIT|BRIEF
/// (a chain begins in a human hand), a CANON/WRIT root cites its mandate,
/// gaps are impossible (the store issues seqs; the trigger holds the rule
/// below every writer), and an item write whose producing entry is absent
/// refuses PROVENANCE_INCOMPLETE.
///
/// G13 — satisfied below the criterion's words, deliberately: this proves
/// the SUBSTRATE half only. The in-flight *fetch* half — a real outward
/// trip appending FETCH entries as it walks — re-arms with Slice 11's
/// trips, which cannot exist behind the no-HTTP wall this slice stands on
/// (SLICE_10 §1, §4).
#[tokio::test]
async fn sc_j09_chain_append_substrate() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let mandate = authored_writ(&store, matrix).await;
    let trip = trip_job(&store, mandate).await;

    // Root grammar: a FETCH root has no human hand — refused.
    let err = store
        .append_chain_entry(
            trip.job_id,
            &ChainEntryDraft {
                chain_ref: Uuid::now_v7(),
                kind: ChainEntryKind::Fetch,
                mandate_ref: None,
                prompt_or_reason: "a fetch with no root".to_string(),
                produced: vec![],
            },
        )
        .await;
    assert!(
        matches!(err, Err(StoreError::ProvenanceIncomplete(_))),
        "a FETCH root refuses PROVENANCE_INCOMPLETE: {err:?}"
    );

    // A WRIT root that cites no mandate is no root either (C.4).
    let err = store
        .append_chain_entry(
            trip.job_id,
            &ChainEntryDraft {
                chain_ref: Uuid::now_v7(),
                kind: ChainEntryKind::Writ,
                mandate_ref: None,
                prompt_or_reason: "a writ root citing nothing".to_string(),
                produced: vec![],
            },
        )
        .await;
    assert!(
        matches!(err, Err(StoreError::ProvenanceIncomplete(_))),
        "a WRIT root without its mandate refuses: {err:?}"
    );

    // Gaps are impossible from above: ChainEntryDraft carries no link_seq
    // field at all — the store issues the next seq (append means append),
    // so a caller cannot even SPELL a gap. Two lawful appends: 0, then 1.
    let item = Uuid::now_v7();
    let root = store
        .append_chain_entry(
            trip.job_id,
            &ChainEntryDraft {
                chain_ref: item,
                kind: ChainEntryKind::Writ,
                mandate_ref: Some(mandate),
                prompt_or_reason: "the writ that sent the trip (C.4)".to_string(),
                produced: vec![],
            },
        )
        .await
        .expect("root");
    assert_eq!(root.link_seq, 0, "the store issued the root seq");
    let fetch = store
        .append_chain_entry(
            trip.job_id,
            &ChainEntryDraft {
                chain_ref: item,
                kind: ChainEntryKind::Fetch,
                mandate_ref: None,
                prompt_or_reason: "fetched the named locator, promising the item".to_string(),
                produced: vec![item],
            },
        )
        .await
        .expect("fetch entry");
    assert_eq!(fetch.link_seq, 1, "the store issued the next seq");

    // … and from below: a raw append at seq 5 (a forged gap) is refused
    // by the grammar trigger — the rule lives beneath every writer.
    let err = sqlx::query(
        r#"INSERT INTO provenance_chains
             (chain_ref, link_seq, kind, actor_job_ref, prompt_or_reason, produced,
              schema_name, schema_version, produced_by)
           VALUES ($1, 5, 'FETCH', $2, 'a gap', '[]', 'ChainEntry', '1.0.0', 'test')"#,
    )
    .bind(item)
    .bind(trip.job_id)
    .execute(store.raw_pool())
    .await
    .expect_err("a gap never lands");
    assert!(
        err.to_string().contains("PROVENANCE_INCOMPLETE"),
        "the trigger names the law: {err}"
    );

    // A raced append converges or errors — never a gap. Two racers, one
    // chain; whatever each saw, the chain reads back contiguous.
    let raced = |why: &str| ChainEntryDraft {
        chain_ref: item,
        kind: ChainEntryKind::FollowUp,
        mandate_ref: None,
        prompt_or_reason: why.to_string(),
        produced: vec![],
    };
    let (draft_a, draft_b) = (raced("raced append A"), raced("raced append B"));
    let (a, b) = tokio::join!(
        store.append_chain_entry(trip.job_id, &draft_a),
        store.append_chain_entry(trip.job_id, &draft_b),
    );
    assert!(
        a.is_ok() || b.is_ok(),
        "at least one racer lands: {a:?} / {b:?}"
    );
    let chain = store.chain_for(item).await.expect("chain reads");
    for (i, entry) in chain.iter().enumerate() {
        assert_eq!(
            entry.link_seq,
            i32::try_from(i).expect("small chain"),
            "the chain is gapless, whatever the race did"
        );
    }

    // Append-in-flight (§4.2): a deposit whose producing chain entry is
    // absent refuses PROVENANCE_INCOMPLETE — memory is not trusted to
    // survive until homecoming.
    let orphan = Uuid::now_v7();
    let err = store
        .quarantine_deposit(trip.job_id, orphan, &quarantine_draft(mandate))
        .await;
    assert!(
        matches!(err, Err(StoreError::ProvenanceIncomplete(_))),
        "no producing entry, no item: {err:?}"
    );
    let held: i64 = sqlx::query_scalar("SELECT count(*) FROM quarantine_items WHERE item_ref = $1")
        .bind(orphan)
        .fetch_one(store.raw_pool())
        .await
        .expect("count");
    assert_eq!(held, 0, "the refused deposit left nothing");

    // With root + FETCH(produced=[item]) standing, the deposit lands.
    let deposited = store
        .quarantine_deposit(trip.job_id, item, &quarantine_draft(mandate))
        .await
        .expect("the promised item lands behind its chain");
    assert_eq!(deposited.item_ref, item);
    assert_eq!(deposited.mandate_ref, Some(mandate));
}

/// The frozen records beneath the J-floor: a chain entry is immutable and
/// never deleted (C.2 — the story extends, it never revises), and a
/// mandate is a charter frozen at birth (C.4 — a correction is a new
/// mandate). UPDATE and DELETE are refused at the substrate, below every
/// API.
#[tokio::test]
async fn chains_frozen() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let mandate = authored_writ(&store, matrix).await;
    let trip = trip_job(&store, mandate).await;
    let item = lawful_deposit(&store, mandate, &trip).await;

    // The chain: UPDATE refused …
    let err = sqlx::query(
        "UPDATE provenance_chains SET prompt_or_reason = 'revised' WHERE chain_ref = $1 AND link_seq = 0",
    )
    .bind(item)
    .execute(store.raw_pool())
    .await
    .expect_err("a chain entry never revises");
    assert!(err.to_string().contains("immutable"), "named: {err}");

    // … and DELETE refused.
    let err = sqlx::query("DELETE FROM provenance_chains WHERE chain_ref = $1")
        .bind(item)
        .execute(store.raw_pool())
        .await
        .expect_err("an arrival story is never destroyed");
    assert!(
        err.to_string().contains("deletion forbidden"),
        "named: {err}"
    );

    // The mandate: UPDATE refused even on a lawfully-authenticated path.
    // The fixture authenticates as 'sovereign' precisely so the
    // IMMUTABILITY trigger is the wall that fires, not the class wall in
    // front of it (ruling G10 guards the door; C.4 freezes the room).
    let mut tx = store.raw_pool().begin().await.expect("tx");
    sqlx::query("SET LOCAL godhead.actor_class = 'sovereign'")
        .execute(&mut *tx)
        .await
        .expect("class set");
    let err = sqlx::query("UPDATE mandates SET trip_budget = '{}'::jsonb WHERE mandate_id = $1")
        .bind(mandate)
        .execute(&mut *tx)
        .await
        .expect_err("a mandate is frozen at birth");
    assert!(err.to_string().contains("immutable"), "named: {err}");
    tx.rollback().await.expect("rollback");

    // And a mandate is never deleted: the charter outlives its errand.
    let err = sqlx::query("DELETE FROM mandates WHERE mandate_id = $1")
        .bind(mandate)
        .execute(store.raw_pool())
        .await
        .expect_err("a charter is never destroyed");
    assert!(
        err.to_string().contains("deletion forbidden"),
        "named: {err}"
    );
}

/// SC-C07, the "admitting external material at the threshold" entry
/// (IV.4; narrowed by ruling G6; Book II §1 step 5).
///
/// Above the API the entry is human by SIGNATURE:
/// `consent_admission(&self, actor: &str, subject_ref, scope, decision,
/// scan_ref)` takes a human actor string and NO job identity — and the
/// Store trait offers no job-identified consent path at all, so admission
/// by any agent surface is not a runtime case but an unspellable one
/// (claims-by-signature become claims-by-test in the sibling arch pins).
/// Below the API this test proves the runtime wall: an unauthenticated
/// write and an agent-authored write are each rejected
/// GATE_BYPASS_ATTEMPT.
///
/// One G13 note for the CRITERION (not this entry): SC-C07 is
/// one-test-per-entry-surface; the mandate-AUTHORING entry's test is
/// claimed by Slice 11, where mandates first move in behavior (SLICE_10
/// §5, ruling G9) — recorded there so no entry silently drops.
#[tokio::test]
async fn sc_c07_threshold_entry() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let mandate = authored_writ(&store, matrix).await;
    let trip = trip_job(&store, mandate).await;
    let item = lawful_deposit(&store, mandate, &trip).await;

    // The human surface answers: REJECTED is a threshold answer and needs
    // no scan citation (only an ADMITTING consent names the scan it saw).
    let consent = store
        .consent_admission(
            "sovereign",
            item,
            ConsentScope::Item,
            ConsentDecision::Rejected,
            None,
        )
        .await
        .expect("the sovereign's answer at the threshold");
    let held = store
        .get_quarantine_item(item)
        .await
        .expect("the item stands");
    assert_eq!(held.consent_ref, Some(consent), "the ruling binds the item");

    // GRANTED (a petition answer) is not a threshold answer.
    let err = store
        .consent_admission(
            "sovereign",
            item,
            ConsentScope::Item,
            ConsentDecision::Granted,
            None,
        )
        .await;
    assert!(
        matches!(err, Err(StoreError::ValidationFailed(_))),
        "only ADMITTED and REJECTED answer at the threshold: {err:?}"
    );

    // The wall (ruling G10): a raw INSERT that never authenticated as any
    // class is rejected GATE_BYPASS_ATTEMPT — whatever string it stamps.
    let err = sqlx::query(RAW_CONSENT_INSERT)
        .bind(Uuid::now_v7())
        .bind(item)
        .bind("sovereign")
        .execute(store.raw_pool())
        .await
        .expect_err("no class, no consent");
    assert!(
        err.to_string().contains("GATE_BYPASS_ATTEMPT"),
        "G10 names it: {err}"
    );

    // And an agent identity is rejected even on an authenticated path
    // (Law IV.4): the wall beneath the signature. Together with the
    // signature itself, admission by any agent surface is impossible —
    // unspellable above the API, rejected below it.
    let mut tx = store.raw_pool().begin().await.expect("tx");
    sqlx::query("SET LOCAL godhead.actor_class = 'sovereign'")
        .execute(&mut *tx)
        .await
        .expect("class set");
    let err = sqlx::query(RAW_CONSENT_INSERT)
        .bind(Uuid::now_v7())
        .bind(item)
        .bind(trip.job_id.to_string())
        .execute(&mut *tx)
        .await
        .expect_err("an agent never consents");
    assert!(
        err.to_string().contains("GATE_BYPASS_ATTEMPT"),
        "Law IV.4 names it: {err}"
    );
    tx.rollback().await.expect("rollback");
}

/// SC-L01, the items half — Return items resolve at the threshold
/// (SLICE_09 §6 finding 7's pin: "full item resolution is pinned as the
/// Deacon's threshold"). A ReturnManifest whose `item_ref`s and
/// `provenance_ref`s RESOLVE — against the quarantine and chain tables,
/// and the rest of the generous-but-real set persist_return documents
/// (node | refined artifact | link; elected-item provenance) — passes the
/// item checks; a ref that names nothing refuses with B.2 on its lips,
/// and nothing persists. (The completion-contract half of SC-L01 stands
/// in godhead-student's sc_l01_completion_contract, since slice 9.)
#[tokio::test]
async fn return_items_resolve_at_threshold() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;

    // The rooms and the bridge (X.5).
    let establisher = running_job(&store, AgentType::Slave).await;
    let teacher_env = store
        .establish_environment(establisher.job_id, EnvKind::Teacher, matrix, Tier::Devout)
        .await
        .expect("teacher room");
    let student_env = store
        .establish_environment(establisher.job_id, EnvKind::Student, matrix, Tier::Devout)
        .await
        .expect("student room");
    store
        .form_pairing(
            teacher_env.env_id,
            student_env.env_id,
            matrix,
            PairingKind::DevoutAssignment,
        )
        .await
        .expect("pairing");

    // A flagged Instruction with one machine-checkable criterion (B.1).
    let teacher = running_job(&store, AgentType::Teacher).await;
    let instruction = store
        .persist_instruction(
            teacher.job_id,
            &InstructionDraft {
                teacher_env_ref: Some(teacher_env.env_id),
                teacher_tier: Tier::Devout,
                target_tier: Tier::Devout,
                concordat_version: Version::new(1, 0, 0),
                objective: "hand back the held arrival under stewardship".to_string(),
                steps: vec![Step {
                    step_id: 1,
                    action: CapabilityAction::Verify,
                    params: json!({}),
                    expected_output: "return.pointer@1.0".to_string(),
                    budget_hint_tokens: 100,
                }],
                acceptance_criteria: vec![AcceptanceCriterion {
                    criterion: "the returned refs resolve at the threshold".to_string(),
                    testable_as: TestableAs::Validation("resolution".to_string()),
                }],
                sources_drawn: vec![],
                supersedes_ref: None,
            },
        )
        .await
        .expect("instruction");
    store
        .flag_instruction(teacher.job_id, instruction.instruction_id)
        .await
        .expect("certified");

    // External material, lawfully at the threshold: the quarantine item
    // whose chain narrates its arrival (chain_ref = item_ref).
    let mandate = authored_writ(&store, matrix).await;
    let trip = trip_job(&store, mandate).await;
    let item = lawful_deposit(&store, mandate, &trip).await;

    // The bound Student (IX.4: the binding is authenticated at create).
    let mut sdraft = job_draft(AgentType::Student);
    sdraft.env_ref = Some(student_env.env_id);
    sdraft.input_refs = vec![matrix];
    let student = run(&store, &sdraft).await;

    let draft_with = |items: Vec<ReturnItem>| ReturnDraft {
        instruction_ref: instruction.instruction_id,
        student_env_ref: student_env.env_id,
        concordat_version: Version::new(1, 0, 0),
        items,
        completion: vec![CompletionEntry {
            criterion_index: 0,
            passed: Some(true),
            evidence_ref: item,
        }],
    };

    // Direction 1 — refs that RESOLVE pass persist_return's item checks:
    // the item is a quarantine item; its provenance is the chain that
    // produced it.
    let good = draft_with(vec![ReturnItem {
        item_ref: item,
        kind: ReturnItemKind::CorpusItem,
        provenance_ref: item,
    }]);
    let persisted = store
        .persist_return(student.job_id, &good)
        .await
        .expect("resolvable refs pass the threshold");
    assert!(!persisted.flagged, "persisted unflagged, per §3.1");

    // The generous set is real end to end: a NODE resolves as an item,
    // and an ELECTED item's recorded provenance resolves as provenance
    // (env-item provenance, A.8's contents index).
    let pipe = IntakePipe::new(
        &store,
        std::env::temp_dir().join(format!("godhead_jfloor_{}", Uuid::now_v7())),
    )
    .expect("pipe");
    let node = pipe
        .commit_file("stewarded.md", b"an internal atom handed back\n")
        .await
        .expect("node");
    store
        .add_env_item(
            student.job_id,
            student_env.env_id,
            node,
            &json!([{
                "link_seq": 0,
                "kind": "BRIEF",
                "actor": student.job_id.to_string(),
                "prompt_or_reason": "elected under the brief",
                "produced": [node.to_string()],
            }]),
            false,
        )
        .await
        .expect("election");
    let node_return = draft_with(vec![ReturnItem {
        item_ref: node,
        kind: ReturnItemKind::RefinedDoc,
        provenance_ref: node,
    }]);
    store
        .persist_return(student.job_id, &node_return)
        .await
        .expect("a node with elected provenance resolves");

    // Direction 2 — an item_ref that names NOTHING refuses, named B.2.
    let ghost_item = draft_with(vec![ReturnItem {
        item_ref: Uuid::now_v7(),
        kind: ReturnItemKind::CorpusItem,
        provenance_ref: item,
    }]);
    match store.persist_return(student.job_id, &ghost_item).await {
        Err(StoreError::ValidationFailed(detail)) => assert!(
            detail.contains("B.2") && detail.contains("resolves to no"),
            "the refusal names B.2 and the hole: {detail}"
        ),
        other => panic!("an unresolvable item_ref must refuse: {other:?}"),
    }

    // … and so does a provenance_ref that names nothing.
    let ghost_prov = draft_with(vec![ReturnItem {
        item_ref: item,
        kind: ReturnItemKind::CorpusItem,
        provenance_ref: Uuid::now_v7(),
    }]);
    match store.persist_return(student.job_id, &ghost_prov).await {
        Err(StoreError::ValidationFailed(detail)) => assert!(
            detail.contains("B.2") && detail.contains("resolves to no"),
            "the refusal names B.2 and the hole: {detail}"
        ),
        other => panic!("an unresolvable provenance_ref must refuse: {other:?}"),
    }

    // The refusals persisted nothing: exactly the two resolvable Returns
    // stand.
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM returns WHERE instruction_ref = $1")
        .bind(instruction.instruction_id)
        .fetch_one(store.raw_pool())
        .await
        .expect("count returns");
    assert_eq!(count, 2, "refused Returns never persist");
}
