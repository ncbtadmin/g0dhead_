//! Slice 10 — SC-E05 rider, the trial's halt site (SLICE_10.md §3;
//! the repaired swallow at godhead-audit/src/lib.rs `refuse_labor`).
//! An auditor labor halting after RUNNING ends REFUSED with a persisted
//! RefusalRecord — never a job stranded live, never a swallowed error.

use godhead_audit::run_auditor;
use godhead_intake::IntakePipe;
use godhead_schemas::{AgentType, AuditorKind, JobStatus, SchemaRegistry};
use godhead_store::{PgStore, Store};
use std::path::PathBuf;
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

fn full_registry() -> SchemaRegistry {
    let mut reg = godhead_intake::registry();
    godhead_audit::register_into(&mut reg);
    reg
}

async fn store() -> Option<PgStore> {
    let Some(url) = database_url() else {
        eprintln!("SKIP: DATABASE_URL unset — database-backed criterion NOT exercised");
        return None;
    };
    Some(
        PgStore::connect(&url, full_registry())
            .await
            .expect("store connect + migrate"),
    )
}

fn temp_root() -> PathBuf {
    std::env::temp_dir().join(format!("godhead_test_{}", Uuid::now_v7()))
}

/// SC-E05 (audit site) — an auditor whose matrix fixture fails mid-labor
/// (the cited ref resolves as a record, but no matrix stands there) halts
/// after RUNNING: the job ends REFUSED with a persisted RefusalRecord
/// carrying the stage code (VII, VALIDATION_FAILED — the labor could not
/// complete), and the error propagates to the caller. The fixture cites a
/// live NODE so the job's input_refs still resolve (SC-A08 hygiene): the
/// failure is the trial's, not the provenance view's.
#[tokio::test]
async fn sc_e05_no_labor_strands() {
    let Some(store) = store().await else { return };
    // A real record that is not a matrix: the spawn succeeds (identical
    // input, Book II §2), the labor's first read fails.
    let pipe = IntakePipe::new(&store, temp_root()).expect("pipe");
    let not_a_matrix = pipe
        .commit_file("decoy.md", b"a node standing where a matrix is cited\n")
        .await
        .expect("commit");

    let err = run_auditor(&store, not_a_matrix, AuditorKind::Gabriel).await;
    assert!(
        err.is_err(),
        "the halt propagates as an error, never swallowed"
    );

    // The spawned auditor is on the record and ends REFUSED — not RUNNING.
    let jobs = store
        .list_jobs_by_input_ref(not_a_matrix)
        .await
        .expect("jobs by input");
    let auditor = jobs
        .iter()
        .find(|j| j.agent_type == AgentType::Auditor)
        .expect("the spawned auditor's job exists");
    assert_eq!(
        auditor.status,
        JobStatus::Refused,
        "the job ends REFUSED, never stranded live (SC-E05)"
    );
    let (law, reason, detail): (String, String, String) =
        sqlx::query_as("SELECT law, reason, detail FROM refusal_records WHERE job_id = $1")
            .bind(auditor.job_id)
            .fetch_one(store.raw_pool())
            .await
            .expect("the RefusalRecord is persisted");
    assert_eq!(law, "VII", "the labor rule's law");
    assert_eq!(reason, "VALIDATION_FAILED", "the stage code (ruling G1)");
    assert!(
        detail.contains("halted after RUNNING"),
        "the record narrates the halt: {detail}"
    );
}
