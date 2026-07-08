//! Section F — Tool-Calling (Law VIII), adversarial-heavy by directive.
//! SC-F01 … SC-F10.

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
            max_wall_ms: 120_000,
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

/// A tool that records how many times its gloves came off — the side
/// effect the property test audits.
struct CountingTool {
    executions: Arc<AtomicUsize>,
    idempotent: bool,
    /// A secret to smuggle into the output, if any (SC-F10 fixture).
    leak: Option<String>,
}

#[async_trait::async_trait]
impl Tool for CountingTool {
    fn name(&self) -> &str {
        "echo"
    }
    fn description(&self) -> &str {
        "echo {text: string} -> {echoed: string}"
    }
    fn idempotent(&self) -> bool {
        self.idempotent
    }
    fn validate_input(&self, args: &serde_json::Value) -> Result<(), String> {
        let obj = args.as_object().ok_or("arguments must be an object")?;
        for key in obj.keys() {
            if key != "text" {
                return Err(format!("unknown argument '{key}'"));
            }
        }
        obj.get("text")
            .and_then(|v| v.as_str())
            .ok_or("argument 'text' (string) is required")?;
        Ok(())
    }
    fn validate_output(&self, output: &serde_json::Value) -> Result<(), String> {
        output
            .get("echoed")
            .and_then(|v| v.as_str())
            .ok_or("output must carry 'echoed' (string)")?;
        Ok(())
    }
    async fn execute(&self, args: &serde_json::Value) -> serde_json::Value {
        self.executions.fetch_add(1, Ordering::SeqCst);
        let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
        if let Some(secret) = &self.leak {
            return serde_json::json!({ "echoed": text, "note": secret });
        }
        serde_json::json!({ "echoed": text })
    }
}

fn counting_registry() -> (ToolRegistry, Arc<AtomicUsize>) {
    let executions = Arc::new(AtomicUsize::new(0));
    let mut reg = ToolRegistry::new();
    reg.add(Arc::new(CountingTool {
        executions: Arc::clone(&executions),
        idempotent: true,
        leak: None,
    }));
    (reg, executions)
}

/// A caller that emits a fixed script of raw strings, one per proposal,
/// and records the feedback the ladder hands back (SC-F01/F02 verbatim).
struct ScriptedCaller {
    script: Vec<String>,
    next: AtomicUsize,
    constrained: bool,
    /// Records the `constrained` flag of each call (SC-F06).
    saw_constrained: Arc<AtomicUsize>,
    /// Every non-empty feedback string the ladder passed in.
    feedbacks: Arc<std::sync::Mutex<Vec<String>>>,
}

impl ScriptedCaller {
    fn new(script: Vec<&str>, constrained: bool) -> Self {
        Self {
            script: script.into_iter().map(str::to_string).collect(),
            next: AtomicUsize::new(0),
            constrained,
            saw_constrained: Arc::new(AtomicUsize::new(0)),
            feedbacks: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl ToolCaller for ScriptedCaller {
    fn supports_constrained(&self) -> bool {
        self.constrained
    }
    async fn propose_call(
        &self,
        _context: &str,
        feedback: Option<&str>,
        constrained: bool,
    ) -> String {
        if constrained {
            self.saw_constrained.fetch_add(1, Ordering::SeqCst);
        }
        if let Some(fb) = feedback {
            self.feedbacks.lock().unwrap().push(fb.to_string());
        }
        let i = self.next.fetch_add(1, Ordering::SeqCst);
        self.script.get(i).cloned().unwrap_or_default()
    }
}

const VALID: &str = r#"{"tool": "echo", "arguments": {"text": "hello"}}"#;

/// SC-F01 — schema-failing arguments are never executed; the ladder
/// engages, then a corrected call succeeds.
#[tokio::test]
async fn sc_f01_ladder_engages() {
    let Some(store) = store().await else { return };
    let (registry, executions) = counting_registry();
    let job = running_job(&store).await;
    // First emission: valid tool, bad arguments. Second: corrected.
    let caller = ScriptedCaller::new(
        vec![r#"{"tool": "echo", "arguments": {"wrong": 1}}"#, VALID],
        false,
    );
    let feedbacks = Arc::clone(&caller.feedbacks);
    let outcome = run_tool_call(&store, job.job_id, &caller, &registry, "ctx")
        .await
        .expect("ladder");
    match outcome {
        ToolCallOutcome::Executed {
            repair_attempts, ..
        } => {
            assert_eq!(repair_attempts, 1, "one repair engaged");
            assert_eq!(
                executions.load(Ordering::SeqCst),
                1,
                "executed once, after validation"
            );
        }
        ToolCallOutcome::Refused { reason } => panic!("should have recovered: {reason}"),
    }
    // VIII.3: the validator's errors AND the roster were fed back verbatim.
    let fb = feedbacks.lock().unwrap();
    assert_eq!(fb.len(), 1, "exactly one repair round fed back");
    assert!(
        fb[0].contains("failed validation"),
        "the validator error, verbatim: {}",
        fb[0]
    );
    assert!(
        fb[0].contains("unknown argument 'wrong'"),
        "the specific defect: {}",
        fb[0]
    );
    assert!(fb[0].contains("echo"), "the valid-tool roster: {}", fb[0]);
}

/// SC-F02 — a hallucinated tool name is invalid; the roster is fed back;
/// after the cap the call refuses TOOL_MALFORMED.
#[tokio::test]
async fn sc_f02_hallucinated_tool() {
    let Some(store) = store().await else { return };
    let (registry, executions) = counting_registry();
    let job = running_job(&store).await;
    let phantom = r#"{"tool": "rm_rf", "arguments": {}}"#;
    let caller = ScriptedCaller::new(vec![phantom, phantom, phantom], false);
    let outcome = run_tool_call(&store, job.job_id, &caller, &registry, "ctx")
        .await
        .expect("ladder");
    match outcome {
        ToolCallOutcome::Refused { reason } => assert!(reason.contains("exhausted"), "{reason}"),
        other => panic!("a phantom tool must never execute: {other:?}"),
    }
    assert_eq!(executions.load(Ordering::SeqCst), 0, "zero executions");
    // The refusal is on the record with the closed reason code.
    let reason: String = sqlx::query_scalar(
        "SELECT reason FROM refusal_records WHERE job_id = $1 ORDER BY produced_at DESC LIMIT 1",
    )
    .bind(job.job_id)
    .fetch_one(store.raw_pool())
    .await
    .expect("refusal record");
    assert_eq!(reason, "TOOL_MALFORMED");
}

/// SC-F03 — missing field, wrong type, and prose-where-structure each
/// independently render a call invalid and unexecuted.
#[tokio::test]
async fn sc_f03_three_fixtures() {
    let Some(store) = store().await else { return };
    let fixtures = [
        r#"{"tool": "echo", "arguments": {}}"#, // missing 'text'
        r#"{"tool": "echo", "arguments": {"text": 42}}"#, // wrong type
        r#"echo the text please"#,              // prose, no structure
    ];
    for fixture in fixtures {
        let (registry, executions) = counting_registry();
        let job = running_job(&store).await;
        // Repeat the same bad fixture past the cap so it always refuses.
        let caller = ScriptedCaller::new(vec![fixture, fixture, fixture], false);
        let outcome = run_tool_call(&store, job.job_id, &caller, &registry, "ctx")
            .await
            .expect("ladder");
        assert!(
            matches!(outcome, ToolCallOutcome::Refused { .. }),
            "fixture must be invalid: {fixture}"
        );
        assert_eq!(
            executions.load(Ordering::SeqCst),
            0,
            "unexecuted: {fixture}"
        );
    }
}

/// SC-F04 — repair attempts are capped at the config value and counted; no
/// unbounded loop is possible.
#[tokio::test]
async fn sc_f04_repair_cap() {
    let Some(store) = store().await else { return };
    let cap = store
        .get_config("tool_repair_attempts")
        .await
        .expect("config")
        .value
        .as_u64()
        .expect("int");
    assert_eq!(cap, 2, "day-one operational default");
    let (registry, _executions) = counting_registry();
    let job = running_job(&store).await;
    // A caller that never produces a valid call, offered far more than cap+1.
    let junk = "not a call";
    let caller = ScriptedCaller::new(vec![junk; 100], false);
    let outcome = run_tool_call(&store, job.job_id, &caller, &registry, "ctx")
        .await
        .expect("ladder");
    match outcome {
        ToolCallOutcome::Refused { reason } => {
            assert!(
                reason.contains(&format!("cap {cap}")),
                "cap surfaced: {reason}"
            );
        }
        other => panic!("junk never validates: {other:?}"),
    }
    // The caller was asked exactly cap+1 times (initial + cap repairs), no more.
    assert_eq!(
        caller.next.load(Ordering::SeqCst),
        usize::try_from(cap).unwrap() + 1,
        "bounded: initial proposal + cap repairs, and not one more"
    );
    // "Counted in the job record": the attempt count is on the PERSISTED
    // refusal, not merely the in-memory reason (SC-F04, read from the DB).
    let detail: String = sqlx::query_scalar(
        "SELECT detail FROM refusal_records WHERE job_id = $1 ORDER BY produced_at DESC LIMIT 1",
    )
    .bind(job.job_id)
    .fetch_one(store.raw_pool())
    .await
    .expect("persisted refusal");
    assert!(
        detail.contains(&format!("{cap} repair(s)")) && detail.contains(&format!("cap {cap}")),
        "the persisted record carries the count: {detail}"
    );
}

/// SC-F05 — invalid output is never consumed; an idempotent tool is
/// re-executed exactly once; a non-idempotent tool refuses immediately.
#[tokio::test]
async fn sc_f05_output_validation() {
    let Some(store) = store().await else { return };

    // A tool whose output is always bad, and idempotent: two executions,
    // then refusal.
    let executions = Arc::new(AtomicUsize::new(0));
    struct AlwaysBad {
        executions: Arc<AtomicUsize>,
        idempotent: bool,
    }
    #[async_trait::async_trait]
    impl Tool for AlwaysBad {
        fn name(&self) -> &str {
            "echo"
        }
        fn description(&self) -> &str {
            "always emits bad output"
        }
        fn idempotent(&self) -> bool {
            self.idempotent
        }
        fn validate_input(&self, _a: &serde_json::Value) -> Result<(), String> {
            Ok(())
        }
        fn validate_output(&self, _o: &serde_json::Value) -> Result<(), String> {
            Err("output is never valid".into())
        }
        async fn execute(&self, _a: &serde_json::Value) -> serde_json::Value {
            self.executions.fetch_add(1, Ordering::SeqCst);
            serde_json::json!({ "wrong": true })
        }
    }

    // Idempotent: exactly two executions (one + one re-execution).
    let mut reg = ToolRegistry::new();
    reg.add(Arc::new(AlwaysBad {
        executions: Arc::clone(&executions),
        idempotent: true,
    }));
    let job = running_job(&store).await;
    let caller = ScriptedCaller::new(vec![VALID], false);
    let outcome = run_tool_call(&store, job.job_id, &caller, &reg, "ctx")
        .await
        .expect("ladder");
    assert!(matches!(outcome, ToolCallOutcome::Refused { .. }));
    assert_eq!(
        executions.load(Ordering::SeqCst),
        2,
        "idempotent: one re-execution, no more"
    );

    // Non-idempotent: exactly one execution, immediate refusal.
    let executions = Arc::new(AtomicUsize::new(0));
    let mut reg = ToolRegistry::new();
    reg.add(Arc::new(AlwaysBad {
        executions: Arc::clone(&executions),
        idempotent: false,
    }));
    let job = running_job(&store).await;
    let caller = ScriptedCaller::new(vec![VALID], false);
    let outcome = run_tool_call(&store, job.job_id, &caller, &reg, "ctx")
        .await
        .expect("ladder");
    assert!(matches!(outcome, ToolCallOutcome::Refused { .. }));
    assert_eq!(
        executions.load(Ordering::SeqCst),
        1,
        "non-idempotent: never re-run"
    );
    let reason: String = sqlx::query_scalar(
        "SELECT reason FROM refusal_records WHERE job_id = $1 ORDER BY produced_at DESC LIMIT 1",
    )
    .bind(job.job_id)
    .fetch_one(store.raw_pool())
    .await
    .expect("refusal");
    assert_eq!(reason, "TOOL_OUTPUT_INVALID");
}

/// SC-F06 — constrained generation is passed whenever the endpoint
/// declares support, and not when it doesn't.
#[tokio::test]
async fn sc_f06_constrained_generation() {
    let Some(store) = store().await else { return };
    let (registry, _e) = counting_registry();

    // Supports constrained: the ladder passes constrained=true.
    let job = running_job(&store).await;
    let caller = ScriptedCaller::new(vec![VALID], true);
    let saw = Arc::clone(&caller.saw_constrained);
    run_tool_call(&store, job.job_id, &caller, &registry, "ctx")
        .await
        .expect("ladder");
    assert_eq!(
        saw.load(Ordering::SeqCst),
        1,
        "constrained enabled when supported"
    );

    // Does not support: constrained is never asserted.
    let (registry, _e) = counting_registry();
    let job = running_job(&store).await;
    let caller = ScriptedCaller::new(vec![VALID], false);
    let saw = Arc::clone(&caller.saw_constrained);
    run_tool_call(&store, job.job_id, &caller, &registry, "ctx")
        .await
        .expect("ladder");
    assert_eq!(saw.load(Ordering::SeqCst), 0, "not forced when unsupported");
    // NOTE: the integration half of SC-F06 (a real local endpoint honoring
    // the grammar) re-arms when Ollama lands — pinned in SLICE_06 §3.
}

/// SC-F07 — the gauntlet: a valid call plus seven malformations, shuffled;
/// zero malformed executions across the suite; every case ends lawfully.
#[tokio::test]
async fn sc_f07_gauntlet() {
    let Some(store) = store().await else { return };
    let cases = [
        (r#"{"tool": "echo", "arguments": {"text": "ok"}}"#, true), // valid
        ("{not json at all", false),                                // malformed JSON
        (r#"{"tool": "echo"}"#, false),                             // schema-invalid (no arguments)
        ("just prose here", false),                                 // pure prose
        (
            r#"here you go: {"tool":"echo","arguments":{"text":"x"}}"#,
            false,
        ), // prose wrapping JSON
        (r#"{"tool": "delete_everything", "arguments": {}}"#, false), // hallucinated tool
        (
            r#"{"tool": "echo", "arguments": {"text": "x"}, "extra": 1}"#,
            false,
        ), // extra field
        (r#"{"tool": "echo", "arguments": {"text": null}}"#, false), // null for required
    ];
    // A fixed non-identity permutation — shuffled without a RNG (the valid
    // case is not first), emitted one after another by a single suite.
    let order = [5usize, 0, 3, 7, 1, 6, 2, 4];
    let total_executions = Arc::new(AtomicUsize::new(0));

    for &idx in &order {
        let (emission, should_execute) = cases[idx];
        let executions = Arc::clone(&total_executions);
        let mut registry = ToolRegistry::new();
        registry.add(Arc::new(CountingTool {
            executions: Arc::clone(&executions),
            idempotent: true,
            leak: None,
        }));
        let job = running_job(&store).await;
        let before = executions.load(Ordering::SeqCst);
        // Offer the case, then only junk repairs — an invalid case refuses
        // rather than accidentally recovering.
        let caller = ScriptedCaller::new(vec![emission, "junk", "junk"], false);
        let outcome = run_tool_call(&store, job.job_id, &caller, &registry, "ctx")
            .await
            .expect("ladder");
        let ran = executions.load(Ordering::SeqCst) - before;
        if should_execute {
            assert!(
                matches!(outcome, ToolCallOutcome::Executed { .. }),
                "the valid case executes: {emission}"
            );
            assert_eq!(ran, 1, "the valid case ran once");
        } else {
            assert!(
                matches!(outcome, ToolCallOutcome::Refused { .. }),
                "malformed case must refuse: {emission}"
            );
            assert_eq!(ran, 0, "ZERO malformed executions: {emission}");
        }
    }
    // The headline ledger: exactly one execution across the shuffled suite.
    assert_eq!(
        total_executions.load(Ordering::SeqCst),
        1,
        "exactly one execution across the suite — the sole valid case"
    );
}

/// A deterministic malformation generator — no external dependency; the
/// index is the seed.
fn malformed(seed: usize) -> String {
    let mutations: &[&str] = &[
        "",
        "null",
        "[]",
        "42",
        "\"a string\"",
        "{}",
        r#"{"tool": "echo"}"#,
        r#"{"arguments": {"text": "x"}}"#,
        r#"{"tool": 1, "arguments": {"text": "x"}}"#,
        r#"{"tool": "echo", "arguments": "not an object"}"#,
        r#"{"tool": "echo", "arguments": {"text": 1}}"#,
        r#"{"tool": "echo", "arguments": {"text": "x"}, "junk": true}"#,
        r#"{"tool": "ghost", "arguments": {"text": "x"}}"#,
        r#"prefix {"tool": "echo", "arguments": {"text": "x"}}"#,
        r#"{"tool": "echo", "arguments": {"text": "x"}} trailing"#,
        r#"{"tool": "echo", "arguments": {}}"#,
        r#"{"tool": "", "arguments": {}}"#,
        "\u{0000}\u{0001}garbage",
    ];
    let base = mutations[seed % mutations.len()];
    // Vary length: repeat the fragment to make long malformed sequences.
    let repeats = seed % 5;
    let mut s = base.to_string();
    for _ in 0..repeats {
        s.push_str(base);
    }
    s
}

/// SC-F08 — the property test: no malformed sequence of any length or
/// shape causes a tool execution, and a rejected call has zero observable
/// side effects (no execution, no artifact).
#[tokio::test]
async fn sc_f08_no_execution_property() {
    let Some(store) = store().await else { return };
    // 150 seeds over 18 mutation shapes × length variation — every shape
    // exercised many times; the headline invariant (SC-F08) proven by
    // exhaustion, not by example. (Each seed is a full job + ladder run
    // against the live store, so the count is bounded for gate wall-time.)
    for seed in 0..150usize {
        let (registry, executions) = counting_registry();
        let job = running_job(&store).await;
        // A whole sequence of malformations — cap+1 of them, all different.
        let script: Vec<String> = (0..3).map(|k| malformed(seed + k * 7)).collect();
        let refs: Vec<&str> = script.iter().map(String::as_str).collect();
        let caller = ScriptedCaller::new(refs, seed % 2 == 0);
        let outcome = run_tool_call(&store, job.job_id, &caller, &registry, "ctx")
            .await
            .expect("ladder never panics");
        assert!(
            matches!(outcome, ToolCallOutcome::Refused { .. }),
            "seed {seed}: malformed sequences never execute"
        );
        assert_eq!(
            executions.load(Ordering::SeqCst),
            0,
            "seed {seed}: zero executions"
        );
        // Zero side effects: no tool_call artifact persisted.
        let artifacts: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM artifacts WHERE job_id = $1 AND output_slot = $2",
        )
        .bind(job.job_id)
        .bind(TOOL_CALL_SLOT)
        .fetch_one(store.raw_pool())
        .await
        .expect("count");
        assert_eq!(artifacts, 0, "seed {seed}: a rejected call writes nothing");
    }
}

/// SC-F09 — an executed call is attributable: the tool_call artifact
/// carries the job's identity and the job resolves its endpoint alias.
#[tokio::test]
async fn sc_f09_provenance() {
    let Some(store) = store().await else { return };
    let (registry, _e) = counting_registry();
    let job = running_job(&store).await;
    let caller = ScriptedCaller::new(vec![VALID], false);
    run_tool_call(&store, job.job_id, &caller, &registry, "ctx")
        .await
        .expect("ladder");
    let artifact = store
        .read_artifact(job.job_id, TOOL_CALL_SLOT)
        .await
        .expect("the call is on the record");
    assert_eq!(artifact.envelope.produced_by, job.job_id.to_string());
    let reread = store.get_job(job.job_id).await.expect("job");
    assert_eq!(
        reread.endpoint_alias.as_deref(),
        Some("mock-reasoner"),
        "which mind did the work"
    );
    assert_eq!(artifact.payload["tool"], "echo");
}

/// SC-F10 — a tool whose output smuggles a secret is refused at
/// consumption; no secret reaches the store.
#[tokio::test]
async fn sc_f10_secret_scan() {
    let Some(store) = store().await else { return };
    let executions = Arc::new(AtomicUsize::new(0));
    let mut reg = ToolRegistry::new();
    reg.add(Arc::new(CountingTool {
        executions: Arc::clone(&executions),
        idempotent: true,
        leak: Some("postgres://svc:hunter2@db.internal:5432/prod".to_string()),
    }));
    let job = running_job(&store).await;
    let caller = ScriptedCaller::new(vec![VALID], false);
    let outcome = run_tool_call(&store, job.job_id, &caller, &reg, "ctx")
        .await
        .expect("ladder");
    assert!(
        matches!(outcome, ToolCallOutcome::Refused { .. }),
        "a secret-bearing output is not consumed"
    );
    // Nothing persisted, and the credential is nowhere in the job's records.
    let hits: i64 = sqlx::query_scalar(
        r#"SELECT
             (SELECT count(*) FROM artifacts WHERE job_id = $1)
           + (SELECT count(*) FROM refusal_records
              WHERE job_id = $1 AND detail ~ '://[^/\s:@]+:[^/\s@]+@')"#,
    )
    .bind(job.job_id)
    .fetch_one(store.raw_pool())
    .await
    .expect("sweep");
    assert_eq!(
        hits, 0,
        "no secret-shaped string reaches artifact or refusal record"
    );
}

/// Regression (slice-6 review, HIGH): an adversarial model cannot suppress
/// its own refusal record. A secret-shaped tool name must NOT poison the
/// persisted refusal — the record is written, the job ends REFUSED, and no
/// secret reaches the store.
#[tokio::test]
async fn secret_shaped_tool_name_still_refuses_on_record() {
    let Some(store) = store().await else { return };
    let (registry, executions) = counting_registry();
    let job = running_job(&store).await;
    // The tool name is a live connection string — the exact class the old
    // code echoed into the refusal detail, tripping the Law XV scan.
    let evil = r#"{"tool": "postgres://svc:hunter2@db.internal:5432/prod", "arguments": {}}"#;
    let caller = ScriptedCaller::new(vec![evil, evil, evil], false);
    let outcome = run_tool_call(&store, job.job_id, &caller, &registry, "ctx")
        .await
        .expect("the ladder does not error");
    assert!(
        matches!(outcome, ToolCallOutcome::Refused { .. }),
        "refused"
    );
    assert_eq!(executions.load(Ordering::SeqCst), 0, "never executed");
    // The refusal IS on the record — the model did not get to choose.
    let (count, detail): (i64, String) = sqlx::query_as(
        "SELECT count(*)::bigint, coalesce(max(detail), '') FROM refusal_records WHERE job_id = $1",
    )
    .bind(job.job_id)
    .fetch_one(store.raw_pool())
    .await
    .expect("query");
    assert_eq!(
        count, 1,
        "exactly one RefusalRecord, written despite the secret-shaped emission"
    );
    assert!(
        !detail.contains("hunter2"),
        "no model secret in the persisted detail: {detail}"
    );
    // And the job reached REFUSED, not stranded RUNNING.
    let reread = store.get_job(job.job_id).await.expect("job");
    assert_eq!(reread.status, JobStatus::Refused);
    // Neither does the returned reason leak the secret.
    let clean = run_tool_call(
        &store,
        running_job(&store).await.job_id,
        &ScriptedCaller::new(vec![evil, evil, evil], false),
        &registry,
        "ctx",
    )
    .await
    .expect("ladder");
    if let ToolCallOutcome::Refused { reason } = clean {
        assert!(
            !reason.contains("hunter2"),
            "the returned reason is clean: {reason}"
        );
    }
}

/// Regression (slice-6 review, MEDIUM): a tool whose input validator panics
/// is caught as invalid — never an execution, and the job ends in a clean
/// refusal rather than a stranded RUNNING job (SLICE_06 §5).
#[tokio::test]
async fn panicking_input_validator_caught_as_invalid() {
    let Some(store) = store().await else { return };
    let executions = Arc::new(AtomicUsize::new(0));
    struct PanicValidator {
        executions: Arc<AtomicUsize>,
    }
    #[async_trait::async_trait]
    impl Tool for PanicValidator {
        fn name(&self) -> &str {
            "echo"
        }
        fn description(&self) -> &str {
            "its input validator panics"
        }
        fn idempotent(&self) -> bool {
            true
        }
        fn validate_input(&self, args: &serde_json::Value) -> Result<(), String> {
            // The classic defect: unwrap a key the model didn't supply.
            let _ = args["text"].as_str().unwrap();
            Ok(())
        }
        fn validate_output(&self, _o: &serde_json::Value) -> Result<(), String> {
            Ok(())
        }
        async fn execute(&self, _a: &serde_json::Value) -> serde_json::Value {
            self.executions.fetch_add(1, Ordering::SeqCst);
            serde_json::json!({ "echoed": "x" })
        }
    }
    let mut registry = ToolRegistry::new();
    registry.add(Arc::new(PanicValidator {
        executions: Arc::clone(&executions),
    }));
    let job = running_job(&store).await;
    // A well-formed call whose arguments make the validator panic.
    let bad = r#"{"tool": "echo", "arguments": {"count": 1}}"#;
    let caller = ScriptedCaller::new(vec![bad, bad, bad], false);
    let outcome = run_tool_call(&store, job.job_id, &caller, &registry, "ctx")
        .await
        .expect("the ladder survives a panicking validator");
    assert!(
        matches!(outcome, ToolCallOutcome::Refused { .. }),
        "caught as invalid"
    );
    assert_eq!(
        executions.load(Ordering::SeqCst),
        0,
        "a validator defect never becomes an execution"
    );
    let reread = store.get_job(job.job_id).await.expect("job");
    assert_eq!(
        reread.status,
        JobStatus::Refused,
        "clean refusal, not stranded RUNNING"
    );
}

/// Regression (slice-6 review, MEDIUM): repair that succeeds on the LAST
/// permitted attempt recovers (the boundary case SLICE_06 §5 pins).
#[tokio::test]
async fn repair_succeeds_on_last_attempt() {
    let Some(store) = store().await else { return };
    let (registry, executions) = counting_registry();
    let job = running_job(&store).await;
    // cap is 2; junk, junk, then valid on the second (last) repair.
    let caller = ScriptedCaller::new(vec!["junk", "junk", VALID], false);
    let outcome = run_tool_call(&store, job.job_id, &caller, &registry, "ctx")
        .await
        .expect("ladder");
    match outcome {
        ToolCallOutcome::Executed {
            repair_attempts, ..
        } => {
            assert_eq!(
                repair_attempts, 2,
                "recovered on the last permitted attempt"
            );
            assert_eq!(executions.load(Ordering::SeqCst), 1);
        }
        ToolCallOutcome::Refused { reason } => {
            panic!("should have recovered on the boundary: {reason}")
        }
    }
}

/// Regression (slice-6 review, MEDIUM): an empty roster refuses every call
/// cleanly — never a crash (SLICE_06 §5).
#[tokio::test]
async fn empty_roster_refuses_never_crashes() {
    let Some(store) = store().await else { return };
    let registry = ToolRegistry::new(); // empty
    assert!(
        registry.roster_text().contains("empty"),
        "the empty roster is legible"
    );
    let job = running_job(&store).await;
    // Even a perfectly-formed call names a tool that cannot exist.
    let caller = ScriptedCaller::new(vec![VALID, VALID, VALID], false);
    let outcome = run_tool_call(&store, job.job_id, &caller, &registry, "ctx")
        .await
        .expect("no crash on an empty roster");
    assert!(
        matches!(outcome, ToolCallOutcome::Refused { .. }),
        "every call is a hallucination"
    );
    let reread = store.get_job(job.job_id).await.expect("job");
    assert_eq!(reread.status, JobStatus::Refused);
}
