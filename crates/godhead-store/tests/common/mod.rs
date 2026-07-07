// Shared fixtures for the slice-1 criteria suites. Each integration-test
// crate compiles this module independently and uses a subset of it, so
// dead_code is allowed here (and only here).
#![allow(dead_code)]

use godhead_schemas::{
    AgentType, Budgets, Certifies, FlagDraft, JobDraft, JobRecord, JobStatus, ReadinessFlag,
    SchemaRegistry, Validator,
};
use godhead_store::{ArtifactDraft, PgStore, Store};
use semver::{Version, VersionReq};
use serde_json::json;

/// DATABASE_URL from the environment, falling back to the workspace .env
/// (untracked; see .env.example).
pub fn database_url() -> Option<String> {
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

/// The test build's declared schema support (Law II.4): one test schema,
/// `test.widget@^1.0`, requiring a non-empty string field `name`.
pub fn registry() -> SchemaRegistry {
    let mut reg = SchemaRegistry::new();
    reg.register(
        "test.widget",
        VersionReq::parse("^1.0").expect("valid req"),
        |payload| {
            let obj = payload.as_object().ok_or("payload must be an object")?;
            let name = obj
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("field 'name' (string) is required")?;
            if name.is_empty() {
                return Err("field 'name' must be non-empty".to_string());
            }
            Ok(())
        },
    );
    reg
}

/// Connects (running migrations) or skips loudly. A run without
/// DATABASE_URL is not a full gate pass.
pub async fn store() -> Option<PgStore> {
    let Some(url) = database_url() else {
        eprintln!(
            "SKIP: DATABASE_URL unset and no workspace .env — database-backed criterion NOT exercised"
        );
        return None;
    };
    Some(
        PgStore::connect(&url, registry())
            .await
            .expect("store connect + migrate"),
    )
}

pub fn job_draft(agent_type: AgentType) -> JobDraft {
    JobDraft {
        agent_type,
        auditor_name: None,
        tier: None,
        input_refs: vec![],
        env_ref: None,
        brief_ref: None,
        endpoint_alias: None,
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 60_000,
            max_tool_calls: 10,
            max_tokens: 100_000,
        },
    }
}

/// A job advanced PENDING → LEASED → RUNNING.
pub async fn running_job(store: &PgStore) -> JobRecord {
    let job = store
        .create_job(&job_draft(AgentType::Slave))
        .await
        .expect("create");
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await
        .expect("to LEASED");
    store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await
        .expect("to RUNNING")
}

pub fn widget(name: &str) -> ArtifactDraft {
    ArtifactDraft {
        schema_name: "test.widget".to_string(),
        schema_version: Version::new(1, 0, 0),
        payload: json!({ "name": name }),
    }
}

pub fn flag_draft(slots: Vec<String>, revisions: Vec<i32>) -> FlagDraft {
    FlagDraft {
        stage: "slice1_test".to_string(),
        certifies: Certifies {
            output_slots: slots,
            revisions,
        },
        validator: Validator {
            id: "godhead-store/registry".to_string(),
            version: "1.0.0".to_string(),
        },
    }
}

/// A job carried through the full lawful path to FLAGGED, with one output
/// in slot "out". Returns the job (stale revision) and its flag.
pub async fn flagged_job(store: &PgStore) -> (JobRecord, ReadinessFlag) {
    let job = running_job(store).await;
    let artifact = store
        .write_artifact(job.job_id, "out", &widget("alpha"))
        .await
        .expect("write out");
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Written)
        .await
        .expect("to WRITTEN");
    let flag = store
        .write_flag(
            job.job_id,
            &flag_draft(vec!["out".to_string()], vec![artifact.revision]),
        )
        .await
        .expect("flag");
    (job, flag)
}

/// Rows in `artifacts` for a job, authoritative or not — the raw census
/// used by convergence and preservation diffs.
pub async fn artifact_count(store: &PgStore, job_id: uuid::Uuid) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM artifacts WHERE job_id = $1")
        .bind(job_id)
        .fetch_one(store.raw_pool())
        .await
        .expect("count artifacts")
}
