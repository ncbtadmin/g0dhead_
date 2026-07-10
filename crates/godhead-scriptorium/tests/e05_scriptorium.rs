//! Slice 10 — SC-E05 rider, the mount's halt site (SLICE_10.md §3; Law
//! IX.3's labor-rule debt, ordered paid by ruling G1 — ENV_INVALID's first
//! construction site). A mount failing floor validation after RUNNING ends
//! REFUSED with a persisted ENV_INVALID RefusalRecord: the agent refuses
//! rather than work atop an invalid room, and the job never strands live.

use godhead_schemas::{AgentType, Budgets, EnvKind, JobDraft, JobStatus, SchemaRegistry, Tier};
use godhead_scriptorium::{establish, mount, ScriptoriumError};
use godhead_store::{PgStore, Store, StoreError};
use semver::Version;
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
        PgStore::connect(&url, SchemaRegistry::new())
            .await
            .expect("store connect + migrate"),
    )
}

/// A running job to plant the matrix fixture under (emerged_by must
/// resolve — SC-A08 hygiene: the stamp is a live job's identity).
async fn planter(store: &PgStore) -> Uuid {
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
            max_tokens: 1,
        },
    };
    let job = store.create_job(&draft).await.expect("create");
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await
        .expect("lease");
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await
        .expect("run");
    job.job_id
}

/// SC-E05 (mount site) / Law IX.3 — `mount` on a room that fails floor
/// validation (ORPHANED: a read-only archive, not a workplace) returns
/// ENV_INVALID, AND the mounting job stands REFUSED with a persisted
/// ENV_INVALID RefusalRecord citing Law IX — the agent-side record IX.3's
/// own text demands, not merely a returned error.
#[tokio::test]
async fn mount_failure_refuses_not_strands() {
    let Some(store) = store().await else { return };
    // A matrix planted raw (born POSTULANT — the substrate permits no other
    // birth) and a room established around it, then orphaned.
    let planter_job = planter(&store).await;
    let matrix = Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO matrices
             (matrix_id, category, node_refs, link_refs, emerged_by, config_rev,
              schema_name, schema_version, produced_by)
           VALUES ($1, $2, '[]', '[]', $3, 1, 'MatrixRecord', '1.0.0', $3::text)"#,
    )
    .bind(matrix)
    .bind(format!("e05_mount_{}", Uuid::now_v7()))
    .bind(planter_job)
    .execute(store.raw_pool())
    .await
    .expect("planted matrix");
    let (_establisher, env) = establish(&store, EnvKind::Teacher, Tier::Devout, matrix)
        .await
        .expect("establish");
    store
        .orphan_environment(env.env_id)
        .await
        .expect("the dependency is lost; the room orphans");

    // The mount fails ENV_INVALID...
    let err = mount(&store, EnvKind::Teacher, Tier::Devout, matrix, env.env_id).await;
    assert!(
        matches!(err, Err(ScriptoriumError::Store(StoreError::EnvInvalid(_)))),
        "an ORPHANED room is not mountable for work: {err:?}"
    );

    // ...AND the mounting job stands REFUSED on the record (IX.3: the
    // AGENT refuses on a failed mount), never stranded RUNNING.
    let jobs = store
        .list_jobs_by_input_ref(matrix)
        .await
        .expect("jobs by input");
    let mounter = jobs
        .iter()
        .find(|j| j.status == JobStatus::Refused)
        .expect("the mounting job exists and is REFUSED (SC-E05)");
    let (law, reason, detail): (String, String, String) =
        sqlx::query_as("SELECT law, reason, detail FROM refusal_records WHERE job_id = $1")
            .bind(mounter.job_id)
            .fetch_one(store.raw_pool())
            .await
            .expect("the RefusalRecord is persisted");
    assert_eq!(law, "IX", "the mount's law");
    assert_eq!(reason, "ENV_INVALID", "IX.3's code, first constructed here");
    assert!(
        detail.contains("failed floor validation at mount"),
        "the record narrates the refusal: {detail}"
    );
}
