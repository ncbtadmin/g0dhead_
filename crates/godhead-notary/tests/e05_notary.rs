//! Slice 10 — SC-E05 rider, the Notary's halt site (SLICE_10.md §3;
//! the repaired swallow at godhead-notary/src/lib.rs `refuse_notary`).
//! A grant labor halting after RUNNING ends REFUSED with a persisted
//! RefusalRecord — never a job stranded live, never a swallowed error.

use godhead_intake::{Dispatcher, IntakePipe};
use godhead_schemas::{
    AgentType, Budgets, JobDraft, JobRecord, JobStatus, OverrideKind, PetitionDraft, SchemaRegistry,
};
use godhead_store::{PgStore, Store};
use semver::Version;
use serde_json::json;
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
    godhead_notary::register_into(&mut reg);
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

/// A node committed and at rest — the subject the petition names.
async fn resting_node(store: &PgStore) -> Uuid {
    let pipe = IntakePipe::new(store, temp_root()).expect("pipe");
    let node_id = pipe
        .commit_file("subject.md", b"a datum the sovereign cares about\n")
        .await
        .expect("commit");
    let dispatcher = Dispatcher::new(&pipe);
    let scope = [node_id];
    dispatcher.tick_scoped(Some(&scope)).await.expect("tick 1");
    dispatcher.tick_scoped(Some(&scope)).await.expect("tick 2");
    node_id
}

async fn agent_job(store: &PgStore) -> JobRecord {
    let draft = JobDraft {
        agent_type: AgentType::Aggregator,
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
            max_tokens: 100_000,
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

/// SC-E05 (notary site) — a grant labor whose consent chain does not
/// validate halts after RUNNING: the summoned Notary's job ends REFUSED
/// with a persisted RefusalRecord carrying the stage code (VII,
/// VALIDATION_FAILED — the labor could not complete), and the error
/// propagates to the caller. The petition here stands OPEN — a chain with
/// no consent is exactly a subject that no longer validates for execution.
#[tokio::test]
async fn sc_e05_no_labor_strands() {
    let Some(store) = store().await else { return };
    let node_id = resting_node(&store).await;
    store
        .lay_category_override(
            "sovereign",
            node_id,
            &json!([{ "category": "scripture", "weight": 0.5, "low_trust": false, "source": "sovereign_hand" }]),
        )
        .await
        .expect("the sovereign's hand");
    let agent = agent_job(&store).await;
    let petition = store
        .open_petition(
            agent.job_id,
            &PetitionDraft {
                subject_ref: node_id,
                change_kind: OverrideKind::CategoryReassigned,
                reason: "the embedding geometry places this node elsewhere".to_string(),
                evidence_refs: vec!["vector-distance-report".to_string()],
                proposed_change: json!([{ "category": "programming", "weight": 0.5, "low_trust": false, "source": "granted_petition" }]),
            },
        )
        .await
        .expect("petition");
    // Deliberately NOT granted: the chain override → petition → consent
    // cannot resolve, so the labor halts mid-grant, after RUNNING.

    let err = godhead_notary::run_grant(&store, petition.petition_id).await;
    assert!(
        matches!(err, Err(godhead_notary::NotaryError::Refused(_))),
        "the halt propagates as a refusal, never swallowed: {err:?}"
    );

    // The summoned Notary is on the record and ends REFUSED — not RUNNING.
    let jobs = store
        .list_jobs_by_input_ref(petition.petition_id)
        .await
        .expect("jobs by input");
    let notary = jobs
        .iter()
        .find(|j| j.agent_type == AgentType::Notary)
        .expect("the summoned Notary's job exists");
    assert_eq!(
        notary.status,
        JobStatus::Refused,
        "the job ends REFUSED, never stranded live (SC-E05)"
    );
    let (law, reason, detail): (String, String, String) =
        sqlx::query_as("SELECT law, reason, detail FROM refusal_records WHERE job_id = $1")
            .bind(notary.job_id)
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
