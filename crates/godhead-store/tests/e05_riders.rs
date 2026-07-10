//! Slice 10 — the hardening riders, store side (SLICE_10.md §3).
//! SC-E05's store-level halt + suite-end sweep, ruling G7's content-hash
//! certification, H3(2) write-side config contracts, H3(3) identity fixes,
//! H3(4) env-lease strictness, and the SC-A08 provenance-view sweep.
//!
//! The per-crate halt-after-RUNNING riders (notary, audit, ml, toolcall,
//! scriptorium) live in those crates' own `e05_*` test binaries —
//! dependency direction forbids them here (they depend on the store).

mod common;

use godhead_schemas::{
    AcceptanceCriterion, AgentType, Budgets, CapabilityAction, ConfigTier, EnvKind,
    InstructionDraft, JobDraft, JobRecord, JobStatus, SourceDraw, Step, TestableAs, Tier,
};
use godhead_store::{PgStore, Store, StoreError};
use semver::Version;
use serde_json::json;
use sqlx::{Connection, PgConnection};
use time::OffsetDateTime;
use uuid::Uuid;

// ---- fixtures ----

/// The suite watermark (Law XII: the store's clock, never the caller's).
/// The first test to ask pins it; every scoped sweep in this binary reads
/// the pinned value, so no fixture row escapes its own suite's sweep.
static WATERMARK: std::sync::Mutex<Option<OffsetDateTime>> = std::sync::Mutex::new(None);

async fn watermark(store: &PgStore) -> OffsetDateTime {
    let now = store.store_now().await.expect("the store's clock");
    let mut pin = WATERMARK.lock().expect("watermark mutex");
    *pin.get_or_insert(now)
}

/// A fixture job draft with an hour of wall budget: fixture jobs left
/// RUNNING at test end must never drift over budget mid-suite, or they
/// would trip the SC-E05 sweep this very binary runs.
fn wide_draft(agent_type: AgentType, tier: Option<Tier>, input_refs: Vec<Uuid>) -> JobDraft {
    JobDraft {
        agent_type,
        auditor_name: None,
        tier,
        input_refs,
        env_ref: None,
        brief_ref: None,
        endpoint_alias: None,
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 3_600_000,
            max_tool_calls: 10,
            max_tokens: 100_000,
        },
    }
}

/// PENDING → LEASED → RUNNING, the lawful spawn (Law I.1).
async fn spawn_running(store: &PgStore, draft: &JobDraft) -> JobRecord {
    let job = store.create_job(draft).await.expect("create");
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await
        .expect("to LEASED");
    store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await
        .expect("to RUNNING")
}

/// Plants a POSTULANT matrix raw (matrices are born POSTULANT — the
/// substrate rejects any other birth), stamped with a live job's identity
/// so the SC-A08 sweep resolves it.
async fn planted_matrix(store: &PgStore, job: &JobRecord) -> Uuid {
    let matrix_id = Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO matrices
             (matrix_id, category, node_refs, link_refs, emerged_by, config_rev,
              schema_name, schema_version, produced_by)
           VALUES ($1, $2, '[]', '[]', $3, 1, 'MatrixRecord', '1.0.0', $3::text)"#,
    )
    .bind(matrix_id)
    .bind(format!("e05_riders_{}", Uuid::now_v7()))
    .bind(job.job_id)
    .execute(store.raw_pool())
    .await
    .expect("planted matrix");
    matrix_id
}

/// A conforming Regular-Teacher Instruction (B.1): sources disclosed,
/// no environment (X.1) — the lightest lawful body the store persists.
fn regular_instruction(matrix_ref: Uuid) -> InstructionDraft {
    InstructionDraft {
        teacher_env_ref: None,
        teacher_tier: Tier::Regular,
        target_tier: Tier::Devout,
        concordat_version: Version::new(1, 0, 0),
        objective: "refine the elected corpus toward completeness".to_string(),
        steps: vec![Step {
            step_id: 1,
            action: CapabilityAction::Refine,
            params: json!({}),
            expected_output: "refined.doc@1.0".to_string(),
            budget_hint_tokens: 100,
        }],
        acceptance_criteria: vec![AcceptanceCriterion {
            criterion: "every refined artifact validates against its schema".to_string(),
            testable_as: TestableAs::Validation("schema_conformance".to_string()),
        }],
        sources_drawn: vec![SourceDraw {
            matrix_ref,
            draw_count: 1,
            canon_associated: false,
        }],
        supersedes_ref: None,
    }
}

/// The persisted refusal for a job — SC-E05's witness.
async fn refusal_of(store: &PgStore, job_id: Uuid) -> (String, String, String) {
    sqlx::query_as("SELECT law, reason, detail FROM refusal_records WHERE job_id = $1")
        .bind(job_id)
        .fetch_one(store.raw_pool())
        .await
        .expect("the RefusalRecord is on the record (SC-E05)")
}

// ---- SC-E05, the store-level halt ----

/// SC-E05 (store half) — a labor halting after RUNNING on the store's own
/// wall (Law XIV.2) ends REFUSED with a persisted RefusalRecord, enacted by
/// the store itself; the BudgetExceeded return is the already-recorded arm
/// of G5's distinction, and a retry meets TERMINAL_ACCESS, never a second
/// record. The agent-side halt sites are exercised in the per-crate `e05_*`
/// binaries (notary, audit, ml slave/aggregate, toolcall, scriptorium).
#[tokio::test]
async fn sc_e05_no_labor_strands() {
    let Some(store) = common::store().await else {
        return;
    };
    let _wm = watermark(&store).await;
    // One millisecond of wall: the labor is over budget the moment it runs.
    let mut draft = wide_draft(AgentType::Slave, None, vec![]);
    draft.budgets.max_wall_ms = 1;
    let job = spawn_running(&store, &draft).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // The next store touch finds the wall crossed: the store enacts the
    // refusal in the same act (already-recorded, not failed-to-record — G5).
    let err = store
        .write_artifact(job.job_id, "out", &common::widget("late"))
        .await
        .expect_err("the wall is crossed");
    assert!(matches!(err, StoreError::BudgetExceeded(_)), "got {err}");

    // The job ends REFUSED — never stranded RUNNING (VII.1).
    let job_now = store.get_job(job.job_id).await.expect("re-read");
    assert_eq!(job_now.status, JobStatus::Refused, "refused, not stranded");
    let (law, reason, detail) = refusal_of(&store, job.job_id).await;
    assert_eq!(law, "XIV");
    assert_eq!(reason, "BUDGET_EXCEEDED");
    assert!(
        detail.contains("max_wall_ms"),
        "the record names the wall: {detail}"
    );

    // A retry poking the refused job meets Law I.4, and the store does NOT
    // write a second refusal — already-recorded means exactly once.
    let err = store
        .write_artifact(job.job_id, "out", &common::widget("again"))
        .await
        .expect_err("access ends at REFUSED");
    assert!(matches!(err, StoreError::TerminalAccess(_)), "got {err}");
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM refusal_records WHERE job_id = $1")
        .bind(job.job_id)
        .fetch_one(store.raw_pool())
        .await
        .expect("count");
    assert_eq!(count, 1, "one halt, one record");
}

// ---- Ruling G7 / SC-K04's class: content-hash certification ----

/// Content-hash certification (ruling G7, S3): FLAG persists the SHA-256 of
/// the canonical body; every read of the flagged record re-proves it, and a
/// byte-corrupt body refuses SCHEMA_MISMATCH naming the hash — no
/// best-effort read path. The corruption fixture is H6(d)'s pinned
/// connection (`session_replication_role='replica'`), never a trigger
/// disabled on the shared pool.
///
/// G13: this test proves the Instruction half. The flagged-Return half is
/// the same store path (`prove_content_sha` at `get_return`) and re-arms in
/// the k/l suites, where the Return fixture (instruction + student
/// environment + completion contract) already stands — deferred here on
/// fixture cost, said so per the convention.
#[tokio::test]
async fn content_sha_at_flag_reproves_at_read() {
    let Some(store) = common::store().await else {
        return;
    };
    let _wm = watermark(&store).await;
    let teacher = spawn_running(
        &store,
        &wide_draft(AgentType::Teacher, Some(Tier::Regular), vec![]),
    )
    .await;
    let matrix_holder =
        spawn_running(&store, &wide_draft(AgentType::Aggregator, None, vec![])).await;
    let matrix = planted_matrix(&store, &matrix_holder).await;

    let record = store
        .persist_instruction(teacher.job_id, &regular_instruction(matrix))
        .await
        .expect("persist");
    assert!(record.content_sha.is_none(), "no hash before FLAG");

    // FLAG persists the certification hash in the same act (G7).
    let flagged = store
        .flag_instruction(teacher.job_id, record.instruction_id)
        .await
        .expect("flag");
    let sha = flagged.content_sha.as_deref().expect("hash at flag");
    assert_eq!(sha.len(), 64, "SHA-256 hex");
    assert!(
        sha.chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "lowercase hex: {sha}"
    );

    // A clean read re-proves and passes.
    store
        .get_instruction(record.instruction_id)
        .await
        .expect("a clean flagged Instruction reads");

    // Corrupt the flagged body OUT-OF-BAND on a dedicated non-pool
    // connection under replica role (H6(d)) — the immutability trigger is
    // bypassed for this session only; the shared pool never loses a wall.
    let url = common::database_url().expect("url");
    let mut pinned = PgConnection::connect(&url)
        .await
        .expect("pinned connection");
    sqlx::query("SET session_replication_role = 'replica'")
        .execute(&mut pinned)
        .await
        .expect("replica role on the pinned session");
    sqlx::query("UPDATE instructions SET objective = 'corrupted between flag and read' WHERE instruction_id = $1")
        .bind(record.instruction_id)
        .execute(&mut pinned)
        .await
        .expect("out-of-band corruption");
    pinned.close().await.ok();

    // The read refuses SCHEMA_MISMATCH, naming the broken content hash.
    let err = store
        .get_instruction(record.instruction_id)
        .await
        .expect_err("byte-integrity is broken");
    assert!(matches!(err, StoreError::SchemaMismatch(_)), "got {err}");
    assert!(
        err.to_string().contains("content hash"),
        "the refusal names the hash: {err}"
    );
}

// ---- H3(2): write-side config contracts ----

/// Write-side config contracts (H3(2)): per-key type + semantic floor
/// enforced at write, so a malformed constant never lands — window=0
/// becomes unrepresentable, thresholds live in [0,1], the roster is never
/// empty. A key without a contract still writes (its contract arrives with
/// its consumer). No standing constant is mutated: every contract fixture
/// fails before any write, and the contract-less probe is a fresh key.
#[tokio::test]
async fn config_write_contracts() {
    let Some(store) = common::store().await else {
        return;
    };
    let _wm = watermark(&store).await;

    // window = 0 silently disables escalation — unrepresentable at write.
    let err = store
        .set_config(
            "e05-harness",
            "bias_pattern_window",
            ConfigTier::Operational,
            &json!(0),
            None,
        )
        .await
        .expect_err("window 0 never lands");
    assert!(matches!(err, StoreError::ValidationFailed(_)), "got {err}");
    assert!(
        err.to_string().contains("write-side contract"),
        "the contract speaks: {err}"
    );

    // Thresholds outside [0, 1], both directions.
    for (key, value) in [
        ("bias_skew_threshold", json!(1.5)),
        ("coherence_threshold", json!(-0.25)),
        ("bias_pattern_threshold", json!(2)),
    ] {
        let err = store
            .set_config("e05-harness", key, ConfigTier::Sovereign, &value, None)
            .await
            .expect_err("a threshold lives in [0, 1]");
        assert!(
            matches!(err, StoreError::ValidationFailed(_)),
            "{key}: got {err}"
        );
    }

    // The name roster is never empty (X.4 needs a name to confer).
    let err = store
        .set_config(
            "e05-harness",
            "name_roster",
            ConfigTier::Operational,
            &json!([]),
            None,
        )
        .await
        .expect_err("an empty roster never lands");
    assert!(matches!(err, StoreError::ValidationFailed(_)), "got {err}");

    // honorific_set is a NESTED object, not a flat array (the Slice 11
    // opening round caught the wrong contract). A flat array is now rejected...
    let err = store
        .set_config(
            "e05-harness",
            "honorific_set",
            ConfigTier::Operational,
            &json!(["Br.", "Sr."]),
            None,
        )
        .await
        .expect_err("a flat-array honorific_set never lands");
    assert!(
        matches!(err, StoreError::ValidationFailed(_))
            && err.to_string().contains("write-side contract"),
        "the honorific_set contract rejects the flat shape: {err}"
    );
    // ...and the correct nested shape CLEARS the contract — the only barrier
    // it then meets is the already-exists guard (proving the contract passed
    // it; before the fix this shape was unwritable through the store).
    let err = store
        .set_config(
            "e05-harness",
            "honorific_set",
            ConfigTier::Operational,
            &json!({ "teacher": { "DEVOUT": "Professor" }, "student": ["Br."] }),
            None,
        )
        .await
        .expect_err("None-insert of the seeded key hits the already-exists guard");
    assert!(
        !err.to_string().contains("write-side contract"),
        "the nested honorific_set shape must clear the contract: {err}"
    );

    // A contract-less unknown key still writes: prevention is per-contract,
    // not a closed-world guess about keys the order has not yet met.
    let free_key = format!("e05_uncontracted_{}", Uuid::now_v7());
    let written = store
        .set_config(
            "e05-harness",
            &free_key,
            ConfigTier::Operational,
            &json!({ "free": "form" }),
            None,
        )
        .await
        .expect("a contract-less key writes as-is");
    assert_eq!(written.value, json!({ "free": "form" }));
}

// ---- H3(3): identity fixes ----

/// Bias disclosure surfaces carry the disclosing job's identity (H3(3);
/// XIII.1 means what it says): an unknown job is an anonymous write and is
/// rejected NOT_FOUND; a lawful disclosure row is stamped with the job's
/// uuid, never a role string.
#[tokio::test]
async fn bias_surfaces_identified() {
    let Some(store) = common::store().await else {
        return;
    };
    let _wm = watermark(&store).await;
    let ghost = Uuid::now_v7();

    // Anonymous disclosure: rejected at the identity wall.
    let err = store
        .record_regular_output(ghost, Uuid::now_v7(), &[], false, 5)
        .await
        .expect_err("an unknown job is anonymity");
    assert!(matches!(err, StoreError::NotFound(_)), "got {err}");
    assert!(
        err.to_string().contains("anonymous writes are rejected"),
        "XIII.1 named: {err}"
    );
    let err = store
        .raise_bias_warning(ghost, "e05_ghost_scope")
        .await
        .expect_err("an unknown job raises nothing");
    assert!(matches!(err, StoreError::NotFound(_)), "got {err}");

    // A lawful disclosure is stamped with the disclosing job's uuid.
    let teacher = spawn_running(
        &store,
        &wide_draft(AgentType::Teacher, Some(Tier::Regular), vec![]),
    )
    .await;
    let holder = spawn_running(&store, &wide_draft(AgentType::Aggregator, None, vec![])).await;
    let matrix = planted_matrix(&store, &holder).await;
    let instruction = store
        .persist_instruction(teacher.job_id, &regular_instruction(matrix))
        .await
        .expect("instruction");
    let draws = [SourceDraw {
        matrix_ref: matrix,
        draw_count: 1,
        canon_associated: false,
    }];
    store
        .record_regular_output(teacher.job_id, instruction.instruction_id, &draws, false, 5)
        .await
        .expect("disclose");
    let produced_by: String = sqlx::query_scalar(
        "SELECT produced_by FROM regular_outputs WHERE instruction_ref = $1
         ORDER BY produced_at DESC LIMIT 1",
    )
    .bind(instruction.instruction_id)
    .fetch_one(store.raw_pool())
    .await
    .expect("disclosure row");
    assert_eq!(
        produced_by,
        teacher.job_id.to_string(),
        "the disclosure bears the disclosing job's identity (XIII.1)"
    );

    // The raised warning bears it too (a fresh scope: no global singleton
    // is touched — the 'regular_teacher' scope belongs to the k-suite).
    let scope = format!("e05_scope_{}", Uuid::now_v7());
    store
        .raise_bias_warning(teacher.job_id, &scope)
        .await
        .expect("raise");
    let produced_by: String =
        sqlx::query_scalar("SELECT produced_by FROM bias_warnings WHERE scope = $1")
            .bind(&scope)
            .fetch_one(store.raw_pool())
            .await
            .expect("warning row");
    assert_eq!(produced_by, teacher.job_id.to_string());
}

/// `release_lease` gains its guard actor (H3(3)): an unknown job is
/// anonymity; a job that does not hold the lease releases nothing — only
/// the holder's identity opens the hand (XIII.1).
#[tokio::test]
async fn lease_release_guarded() {
    let Some(store) = common::store().await else {
        return;
    };
    let _wm = watermark(&store).await;

    // Anonymous release: rejected before any lease is even consulted.
    let err = store
        .release_lease(Uuid::now_v7(), Uuid::now_v7())
        .await
        .expect_err("an unknown job releases nothing");
    assert!(matches!(err, StoreError::NotFound(_)), "got {err}");
    assert!(
        err.to_string().contains("anonymous writes are rejected"),
        "XIII.1 named: {err}"
    );

    // A live job that is not the holder releases nothing either.
    let holder = spawn_running(&store, &wide_draft(AgentType::Slave, None, vec![])).await;
    let stranger = spawn_running(&store, &wide_draft(AgentType::Slave, None, vec![])).await;
    let subject = Uuid::now_v7();
    let lease = store
        .acquire_lease(holder.job_id, subject, 60_000)
        .await
        .expect("acquire");
    let err = store
        .release_lease(stranger.job_id, lease.lease_id)
        .await
        .expect_err("not the holder");
    assert!(matches!(err, StoreError::NotFound(_)), "got {err}");
    // The lease survived the stranger's attempt: the holder still beats.
    store
        .heartbeat_lease(holder.job_id, lease.lease_id)
        .await
        .expect("the lease still stands under its holder");
    // The holder's own hand opens.
    store
        .release_lease(holder.job_id, lease.lease_id)
        .await
        .expect("the holder releases");
}

// ---- H3(4): env mutation under lease ----

/// Env-lease rule, strict XI.1 (H3(4)): `add_env_item` writes under the
/// environment's lease, acquire-or-refuse — while ANOTHER job holds the
/// room's lease the curation refuses LEASE_CONFLICT; after release it
/// succeeds, and the curation lease is released in the same breath.
///
/// G13: office/system environment-status transitions (`orphan_environment`)
/// remain CAS-guarded rather than lease-guarded, and lease identity for
/// offices re-arms when offices gain lease surfaces — the Deacon holds no
/// leases in this slice.
#[tokio::test]
async fn env_mutation_under_lease() {
    let Some(store) = common::store().await else {
        return;
    };
    let _wm = watermark(&store).await;
    let establisher = spawn_running(
        &store,
        &wide_draft(AgentType::Teacher, Some(Tier::Devout), vec![]),
    )
    .await;
    let matrix = planted_matrix(&store, &establisher).await;
    let env = store
        .establish_environment(establisher.job_id, EnvKind::Teacher, matrix, Tier::Devout)
        .await
        .expect("establish");

    // Another job holds the room's lease.
    let holder = spawn_running(&store, &wide_draft(AgentType::Slave, None, vec![])).await;
    let held = store
        .acquire_lease(holder.job_id, env.env_id, 60_000)
        .await
        .expect("the holder leases the room");

    let item = Uuid::now_v7();
    let provenance = json!([{
        "link_seq": 0, "kind": "BRIEF", "actor": establisher.job_id.to_string(),
        "prompt_or_reason": "e05 fixture: the sovereign's charge",
        "produced": [item.to_string()],
    }]);
    let err = store
        .add_env_item(establisher.job_id, env.env_id, item, &provenance, false)
        .await
        .expect_err("curation under another's lease refuses (XI.1)");
    assert!(matches!(err, StoreError::LeaseConflict(_)), "got {err}");
    let items = store.env_items(env.env_id).await.expect("items");
    assert!(items.is_empty(), "the refused curation wrote nothing");

    // The holder releases; curation now acquires, writes, and releases in
    // the same breath.
    store
        .release_lease(holder.job_id, held.lease_id)
        .await
        .expect("release");
    let written = store
        .add_env_item(establisher.job_id, env.env_id, item, &provenance, false)
        .await
        .expect("curation under its own lease");
    assert_eq!(written.item_ref, item);
    // Released in the same breath: the room's lease is free again.
    let reacquired = store
        .acquire_lease(holder.job_id, env.env_id, 60_000)
        .await
        .expect("the curation lease did not linger");
    store
        .release_lease(holder.job_id, reacquired.lease_id)
        .await
        .expect("cleanup release");
}

// ---- SC-A08: the provenance-view integrity sweep ----

/// SC-A08 (minted by ruling G8) — the declared provenance view holds:
/// every `produced_by` written during this suite resolves to a live
/// JobRecord (when uuid-shaped) or a registered office identity
/// (`office:deacon`; all other non-uuid strings are human/deployment
/// actors), and every such job's `input_refs` resolve to persisted
/// records. Envelope-bearing tables are discovered from
/// `information_schema`, not a hand-kept list.
///
/// G13: scoped to rows produced at/after this binary's watermark — the
/// one-time archaeology pass over the historical store (H4 NEW-2) is run
/// at delivery and recorded in the slice ledger, not in-gate; and
/// human/deployment actor strings are accepted by shape (non-uuid), there
/// being no actor roster to resolve them against yet.
#[tokio::test]
async fn sc_a08_provenance_view_integrity() {
    let Some(store) = common::store().await else {
        return;
    };
    let wm = watermark(&store).await;

    // Non-vacuity: at least one job in the window carries a resolving
    // input_ref (the planted matrix) — the sweep must have something to walk.
    let holder = spawn_running(&store, &wide_draft(AgentType::Aggregator, None, vec![])).await;
    let matrix = planted_matrix(&store, &holder).await;
    let _walker = spawn_running(&store, &wide_draft(AgentType::Slave, None, vec![matrix])).await;

    // Every envelope-bearing table, discovered — not enumerated by hand.
    let tables: Vec<String> = sqlx::query_scalar(
        r#"SELECT c.table_name::text FROM information_schema.columns c
           WHERE c.table_schema = 'public' AND c.column_name = 'produced_by'
             AND EXISTS (SELECT 1 FROM information_schema.columns c2
                         WHERE c2.table_schema = 'public'
                           AND c2.table_name = c.table_name
                           AND c2.column_name = 'produced_at')
           ORDER BY c.table_name"#,
    )
    .fetch_all(store.raw_pool())
    .await
    .expect("schema walk");
    assert!(
        tables.len() >= 20,
        "the envelope convention is workspace-wide; found only {tables:?}"
    );

    let mut violations: Vec<String> = Vec::new();
    for table in &tables {
        let stamps: Vec<String> = sqlx::query_scalar(&format!(
            "SELECT DISTINCT produced_by FROM {table} WHERE produced_at >= $1"
        ))
        .bind(wm)
        .fetch_all(store.raw_pool())
        .await
        .expect("stamp scan");
        for stamp in stamps {
            if let Ok(job_id) = Uuid::parse_str(&stamp) {
                let resolves: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM job_records WHERE job_id = $1)",
                )
                .bind(job_id)
                .fetch_one(store.raw_pool())
                .await
                .expect("job lookup");
                if !resolves {
                    violations.push(format!("{table}: produced_by {stamp} resolves to no job"));
                }
            } else if stamp.starts_with("office:") && stamp != "office:deacon" {
                violations.push(format!("{table}: unregistered office identity '{stamp}'"));
            }
            // Any other non-uuid stamp is a human/deployment actor string
            // (A.1) — accepted by shape; see the G13 note above.
        }
    }
    assert!(violations.is_empty(), "SC-A08 violations: {violations:#?}");

    // ... and that JobRecord's input_refs resolve (V.2 as dispersed): every
    // uuid an in-window job cites must exist somewhere in the store.
    let jobs: Vec<(Uuid, serde_json::Value)> =
        sqlx::query_as("SELECT job_id, input_refs FROM job_records WHERE produced_at >= $1")
            .bind(wm)
            .fetch_all(store.raw_pool())
            .await
            .expect("jobs in window");
    assert!(!jobs.is_empty(), "the window saw this suite's own jobs");
    let mut dangling: Vec<String> = Vec::new();
    let mut walked = 0usize;
    for (job_id, input_refs) in &jobs {
        let refs = input_refs.as_array().cloned().unwrap_or_default();
        for r in refs {
            let Some(id) = r.as_str().and_then(|s| Uuid::parse_str(s).ok()) else {
                dangling.push(format!("job {job_id}: non-uuid input_ref {r}"));
                continue;
            };
            walked += 1;
            let resolves: bool = sqlx::query_scalar(
                r#"SELECT EXISTS(SELECT 1 FROM job_records WHERE job_id = $1)
                    OR EXISTS(SELECT 1 FROM nodes WHERE node_id = $1)
                    OR EXISTS(SELECT 1 FROM links WHERE link_id = $1)
                    OR EXISTS(SELECT 1 FROM matrices WHERE matrix_id = $1)
                    OR EXISTS(SELECT 1 FROM petition_records WHERE petition_id = $1)
                    OR EXISTS(SELECT 1 FROM readiness_flags WHERE flag_id = $1)
                    OR EXISTS(SELECT 1 FROM override_records WHERE override_id = $1)
                    OR EXISTS(SELECT 1 FROM consent_records WHERE consent_id = $1)
                    OR EXISTS(SELECT 1 FROM audit_reports WHERE report_id = $1)
                    OR EXISTS(SELECT 1 FROM joint_proposals WHERE proposal_id = $1)
                    OR EXISTS(SELECT 1 FROM environments WHERE env_id = $1)
                    OR EXISTS(SELECT 1 FROM pairings WHERE pairing_id = $1)
                    OR EXISTS(SELECT 1 FROM instructions WHERE instruction_id = $1)
                    OR EXISTS(SELECT 1 FROM regular_outputs WHERE output_id = $1)
                    OR EXISTS(SELECT 1 FROM returns WHERE return_id = $1)
                    OR EXISTS(SELECT 1 FROM refined_artifacts WHERE artifact_id = $1)
                    OR EXISTS(SELECT 1 FROM mandates WHERE mandate_id = $1)
                    OR EXISTS(SELECT 1 FROM provenance_chains WHERE chain_ref = $1)
                    OR EXISTS(SELECT 1 FROM quarantine_items WHERE item_ref = $1)
                    OR EXISTS(SELECT 1 FROM scan_verdicts WHERE scan_id = $1)
                    OR EXISTS(SELECT 1 FROM manifests WHERE manifest_id = $1)
                    OR EXISTS(SELECT 1 FROM lease_records WHERE lease_id = $1)
                    OR EXISTS(SELECT 1 FROM refusal_records WHERE refusal_id = $1)
                    OR EXISTS(SELECT 1 FROM log_snapshots WHERE log_id = $1)"#,
            )
            .bind(id)
            .fetch_one(store.raw_pool())
            .await
            .expect("resolution walk");
            if !resolves {
                dangling.push(format!("job {job_id}: input_ref {id} resolves to nothing"));
            }
        }
    }
    assert!(dangling.is_empty(), "dangling input_refs: {dangling:#?}");
    assert!(walked >= 1, "the walker fixture made the sweep non-vacuous");
}

// ---- SC-E05, the suite-end sweep ----

/// SC-E05 (sweep half) — at suite end, zero jobs stand RUNNING beyond
/// their wall budget: every halt in this suite ended in a refusal on the
/// record, and every fixture job left RUNNING holds unexhausted budget.
/// Named `zz_` so an ordered (single-threaded) run places it last; under
/// the default parallel runner the bounded re-check below absorbs
/// neighbors mid-halt (a job between its wall crossing and its refusal is
/// in flight, not stranded — the assertion is about where jobs STAND).
///
/// G13: scoped to jobs created at/after this binary's watermark. The
/// historical store carries pre-slice-10 debt (suites through slice 9
/// lawfully abandoned RUNNING fixture jobs, per SC-E05's late minting by
/// ruling G5); that archaeology is run once at delivery and recorded in
/// the slice ledger — the criterion's unscoped words re-arm there.
#[tokio::test]
async fn zz_sc_e05_suite_end_sweep() {
    let Some(store) = common::store().await else {
        return;
    };
    let wm = watermark(&store).await;
    let mut stranded: Vec<(Uuid, String)> = Vec::new();
    for _attempt in 0..20 {
        stranded = sqlx::query_as(
            r#"SELECT job_id, agent_type FROM job_records
               WHERE status = 'RUNNING'
                 AND produced_at >= $1
                 AND started_at IS NOT NULL
                 AND started_at + (max_wall_ms::double precision * interval '1 millisecond')
                     < now()"#,
        )
        .bind(wm)
        .fetch_all(store.raw_pool())
        .await
        .expect("sweep");
        if stranded.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    assert!(
        stranded.is_empty(),
        "jobs stand RUNNING beyond their wall budget at suite end (SC-E05): {stranded:?}"
    );
}
