//! Slice 10 — SC-E05 rider, the tool's halt site (SLICE_10.md §3; the
//! F4-new finding). A panicking `execute()` unwinds into a refusal, never
//! a strand: the call refuses TOOL_OUTPUT_INVALID on the record, the job
//! ends REFUSED, nothing the tool produced is consumed — and a tool that
//! panicked is never re-executed, idempotent or not.

use godhead_schemas::{AgentType, Budgets, JobDraft, JobRecord, JobStatus};
use godhead_store::{PgStore, Store};
use godhead_toolcall::{
    register_into, run_tool_call, Tool, ToolCallOutcome, ToolCaller, ToolRegistry, TOOL_CALL_SLOT,
};
use semver::Version;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

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
    let mut reg = godhead_schemas::SchemaRegistry::new();
    register_into(&mut reg);
    Some(
        PgStore::connect(&url, reg)
            .await
            .expect("store connect + migrate"),
    )
}

async fn running_job(store: &PgStore) -> JobRecord {
    let draft = JobDraft {
        agent_type: AgentType::Student,
        auditor_name: None,
        tier: None,
        input_refs: vec![],
        env_ref: None,
        brief_ref: None,
        endpoint_alias: Some("mock-reasoner".to_string()),
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 3_600_000,
            max_tool_calls: 100,
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

/// A tool that counts its executions, then panics — the defect class F4
/// found: a mind that dies mid-labor instead of answering.
struct PanickingTool {
    executions: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl Tool for PanickingTool {
    fn name(&self) -> &str {
        "echo"
    }
    fn description(&self) -> &str {
        "its execute() panics"
    }
    fn idempotent(&self) -> bool {
        // Deliberately idempotent: VIII.4 would permit ONE re-execution on
        // invalid output — but a panic forfeits even that (SC-E05's class).
        true
    }
    fn validate_input(&self, _args: &serde_json::Value) -> Result<(), String> {
        Ok(())
    }
    fn validate_output(&self, _output: &serde_json::Value) -> Result<(), String> {
        Ok(())
    }
    async fn execute(&self, _args: &serde_json::Value) -> serde_json::Value {
        self.executions.fetch_add(1, Ordering::SeqCst);
        panic!("the tool dies mid-labor (e05 fixture)");
    }
}

/// A caller that emits one fixed valid call.
struct FixedCaller;

#[async_trait::async_trait]
impl ToolCaller for FixedCaller {
    fn supports_constrained(&self) -> bool {
        false
    }
    async fn propose_call(
        &self,
        _context: &str,
        _feedback: Option<&str>,
        _constrained: bool,
    ) -> String {
        r#"{"tool": "echo", "arguments": {}}"#.to_string()
    }
}

/// SC-E05 (tool site) — a panicking `execute()` never strands the labor:
/// the panic is caught, the call refuses TOOL_OUTPUT_INVALID with the
/// RefusalRecord persisted, the job ends REFUSED, no output is consumed —
/// and the panicking tool is NOT re-executed despite swearing idempotence.
#[tokio::test]
async fn panicking_tool_refuses() {
    let Some(store) = store().await else { return };
    let executions = Arc::new(AtomicUsize::new(0));
    let mut registry = ToolRegistry::new();
    registry.add(Arc::new(PanickingTool {
        executions: Arc::clone(&executions),
    }));
    let job = running_job(&store).await;

    let outcome = run_tool_call(&store, job.job_id, &FixedCaller, &registry, "ctx")
        .await
        .expect("the ladder survives a panicking tool");
    match outcome {
        ToolCallOutcome::Refused { reason } => {
            assert!(
                reason.contains("panicked"),
                "the refusal names the panic: {reason}"
            );
        }
        other => panic!("a panicking tool never executes to consumption: {other:?}"),
    }

    // Exactly one execution: a panic forfeits the idempotent re-run.
    assert_eq!(
        executions.load(Ordering::SeqCst),
        1,
        "a panicking idempotent tool is NOT re-executed (SC-E05's class)"
    );

    // The refusal is persisted with the closed reason code (Law VIII.4),
    // and the job ends REFUSED — never stranded RUNNING.
    let (reason, law): (String, String) = sqlx::query_as(
        "SELECT reason, law FROM refusal_records WHERE job_id = $1 ORDER BY produced_at DESC LIMIT 1",
    )
    .bind(job.job_id)
    .fetch_one(store.raw_pool())
    .await
    .expect("the RefusalRecord is persisted");
    assert_eq!(reason, "TOOL_OUTPUT_INVALID");
    assert_eq!(law, "VIII");
    let reread = store.get_job(job.job_id).await.expect("job");
    assert_eq!(reread.status, JobStatus::Refused, "refused, not stranded");

    // No strand: nothing the dead tool touched was consumed.
    let artifacts: i64 =
        sqlx::query_scalar("SELECT count(*) FROM artifacts WHERE job_id = $1 AND output_slot = $2")
            .bind(job.job_id)
            .bind(TOOL_CALL_SLOT)
            .fetch_one(store.raw_pool())
            .await
            .expect("count");
    assert_eq!(artifacts, 0, "no tool_call artifact from a panicked call");
}
