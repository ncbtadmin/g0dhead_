//! The validate → feed-back → regenerate → refuse ladder (Law VIII.2–4).

use crate::{Tool, ToolCallError, ToolCaller, ToolRegistry, TOOL_CALL_SCHEMA, TOOL_CALL_SLOT};
use godhead_schemas::{Law, RefusalDraft, RefusalReason};
use godhead_store::{ArtifactDraft, Store, StoreError};
use semver::Version;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use uuid::Uuid;

/// A sane ceiling on repair attempts, whatever the operator configured —
/// a misconfigured `tool_repair_attempts` can never make the ladder
/// unbounded (the loop is bounded by cap; this bounds the cap).
const REPAIR_CAP_CEILING: u64 = 1000;

/// How one tool call ended. Both arms are lawful; there is no third.
#[derive(Debug)]
pub enum ToolCallOutcome {
    /// Validated in, executed, validated out, persisted. Carries the
    /// consumed output and the repair attempts spent.
    Executed {
        tool: String,
        output: serde_json::Value,
        repair_attempts: u64,
    },
    /// The ladder exhausted or the output could not be trusted; a
    /// RefusalRecord is on the record and the job is REFUSED. The reason is
    /// a law-referencing description that carries no model-controlled text.
    Refused { reason: String },
}

/// A validated call, ready for the gloves to come off.
struct ValidCall {
    tool: Arc<dyn Tool>,
    arguments: serde_json::Value,
}

/// A tool's input validator, run under panic isolation: a validator that
/// panics is a validator defect, and a defect must never become an
/// execution — it is caught here and treated as an invalid call
/// (SLICE_06 §5). Panics cannot cross an await; validators are sync.
fn guarded_validate_input(tool: &Arc<dyn Tool>, args: &serde_json::Value) -> Result<(), String> {
    std::panic::catch_unwind(AssertUnwindSafe(|| tool.validate_input(args))).unwrap_or_else(|_| {
        Err("the tool's input validator panicked — a validator defect is treated as an invalid call, never an execution".to_string())
    })
}

fn guarded_validate_output(tool: &Arc<dyn Tool>, out: &serde_json::Value) -> Result<(), String> {
    std::panic::catch_unwind(AssertUnwindSafe(|| tool.validate_output(out))).unwrap_or_else(|_| {
        Err("the tool's output validator panicked — treated as invalid output".to_string())
    })
}

/// VIII.2 made strict (and II.2 honored): the emission's trimmed entirety
/// must be a JSON object with exactly the keys `tool` (string) and
/// `arguments` (object). Prose around perfect JSON is as invalid as
/// noise — there is no extraction, no repair, no best-effort parse path.
/// The returned defect string may echo model-controlled text; it is used
/// ONLY as feedback to the model (in-memory, never persisted — no model
/// text ever reaches a store-scanned field).
fn validate_raw(registry: &ToolRegistry, raw: &str) -> Result<ValidCall, String> {
    let value: serde_json::Value = serde_json::from_str(raw.trim())
        .map_err(|e| format!("the emission is not a JSON object: {e}"))?;
    let obj = value
        .as_object()
        .ok_or("the emission is JSON but not an object")?;
    for key in obj.keys() {
        if key != "tool" && key != "arguments" {
            return Err(format!(
                "unknown top-level field '{key}': a call carries exactly 'tool' and 'arguments'"
            ));
        }
    }
    let tool_name = obj
        .get("tool")
        .and_then(|v| v.as_str())
        .ok_or("field 'tool' (string) is required")?;
    let arguments = obj
        .get("arguments")
        .ok_or("field 'arguments' (object) is required")?;
    if !arguments.is_object() {
        return Err("field 'arguments' must be an object".to_string());
    }
    let tool = registry
        .get(tool_name)
        .ok_or_else(|| format!("unknown tool '{tool_name}'"))?;
    guarded_validate_input(tool, arguments)
        .map_err(|e| format!("arguments for '{tool_name}' failed validation: {e}"))?;
    Ok(ValidCall {
        tool: Arc::clone(tool),
        arguments: arguments.clone(),
    })
}

/// Law VII for the calling job — refuse, flag, preserve. The detail is
/// caller-supplied and MUST NOT contain model-controlled text (it is
/// persisted and Law XV-scanned; a secret-shaped detail would make the
/// write fail). A failed refusal write is a hard error, never swallowed:
/// "a RefusalRecord is on the record" is a promise the outcome keeps.
async fn refuse<S: Store>(
    store: &S,
    job_id: Uuid,
    reason: RefusalReason,
    detail: String,
) -> Result<ToolCallOutcome, ToolCallError> {
    store
        .refuse(
            job_id,
            &RefusalDraft {
                law: Law::VIII,
                reason,
                subject_refs: vec![job_id.to_string()],
                detail: detail.clone(),
                preserved_refs: vec![],
            },
        )
        .await?;
    Ok(ToolCallOutcome::Refused { reason: detail })
}

/// One complete tool call under the ladder, for the given RUNNING job:
///
/// 1. propose (constrained iff the caller supports it — VIII.1);
/// 2. validate before execution (VIII.2) — invalid ⇒ errors + roster fed
///    back verbatim, regenerate, at most `tool_repair_attempts` times
///    (VIII.3); exhausted ⇒ refuse TOOL_MALFORMED;
/// 3. execute;
/// 4. validate after execution (VIII.4) — invalid ⇒ one re-execution iff
///    idempotent; otherwise, and on second failure, refuse
///    TOOL_OUTPUT_INVALID;
/// 5. persist the consumed call as the job's `tool_call` artifact (which
///    passes the store's Law XV scan or the call refuses).
///
/// Zero side effects on any rejected call: execution happens only after
/// step 2 passes, ever. No model-controlled text ever reaches a persisted
/// field — validator errors are fed back to the model only, never stored.
pub async fn run_tool_call<S: Store, M: ToolCaller>(
    store: &S,
    job_id: Uuid,
    caller: &M,
    registry: &ToolRegistry,
    context: &str,
) -> Result<ToolCallOutcome, ToolCallError> {
    let repair_cap = store
        .get_config("tool_repair_attempts")
        .await?
        .value
        .as_u64()
        .ok_or_else(|| {
            StoreError::ValidationFailed("tool_repair_attempts must be an unsigned integer".into())
        })?
        .min(REPAIR_CAP_CEILING);
    // VIII.1: constrained generation is enabled whenever the endpoint
    // supports it — not a preference, an obligation.
    let constrained = caller.supports_constrained();

    let mut feedback: Option<String> = None;
    let mut repair_attempts: u64 = 0;
    let call = loop {
        let raw = caller
            .propose_call(context, feedback.as_deref(), constrained)
            .await;
        match validate_raw(registry, &raw) {
            Ok(call) => break call,
            Err(defect) => {
                if repair_attempts >= repair_cap {
                    // The persisted detail references the law and the count
                    // only — never the model's emission (which `defect`
                    // carries and which would poison the Law XV scan).
                    let detail = format!(
                        "the tool-call ladder is exhausted after {repair_attempts} repair(s) (cap {repair_cap}); the emission did not validate against Law VIII.2"
                    );
                    return refuse(store, job_id, RefusalReason::ToolMalformed, detail).await;
                }
                repair_attempts += 1;
                // VIII.3: the validator's errors and the roster of valid
                // tools are fed back verbatim (in-memory feedback only).
                feedback = Some(format!(
                    "INVALID CALL: {defect}\nValid tools:\n{}\nRespond with exactly one JSON object: {{\"tool\": <name>, \"arguments\": {{...}}}}",
                    registry.roster_text()
                ));
            }
        }
    };

    // The gloves come off: execute, then validate the hands' work. The
    // tool name here is from OUR registry (it resolved), never the raw
    // emission — safe to name in a persisted refusal.
    let tool_name = call.tool.name().to_string();
    let mut output = call.tool.execute(&call.arguments).await;
    if guarded_validate_output(&call.tool, &output).is_err() {
        if call.tool.idempotent() {
            // VIII.4: exactly one re-execution, and only because the tool
            // swears a second run is the same run.
            output = call.tool.execute(&call.arguments).await;
            if guarded_validate_output(&call.tool, &output).is_err() {
                let detail = format!(
                    "tool '{tool_name}' output failed validation twice after {repair_attempts} repair(s); refused (Law VIII.4)"
                );
                return refuse(store, job_id, RefusalReason::ToolOutputInvalid, detail).await;
            }
        } else {
            let detail = format!(
                "tool '{tool_name}' output failed validation and the tool is not idempotent, after {repair_attempts} repair(s); refused (Law VIII.4)"
            );
            return refuse(store, job_id, RefusalReason::ToolOutputInvalid, detail).await;
        }
    }

    // Attributable consumption (VIII; XIII.2): the call, its arguments,
    // its output, and the attempts spent — on the job's record. The
    // store's outbound secret scan (XV.2) guards this write; a hit
    // refuses the call rather than persisting the secret.
    let artifact = store
        .write_artifact(
            job_id,
            TOOL_CALL_SLOT,
            &ArtifactDraft {
                schema_name: TOOL_CALL_SCHEMA.to_string(),
                schema_version: Version::new(1, 0, 0),
                payload: serde_json::json!({
                    "tool": tool_name,
                    "arguments": call.arguments,
                    "output": output,
                    "attempts": repair_attempts,
                }),
            },
        )
        .await;
    if let Err(err) = artifact {
        // A secret in the output (or any write defect) refuses the call —
        // the tool ran, but nothing it produced is consumed. The error
        // message names the pattern, never the secret itself (Law XV.1),
        // so it is safe to persist in the refusal detail.
        let detail = format!(
            "tool '{tool_name}' executed but its record could not be consumed: {err} (Law XV outbound scan included)"
        );
        return refuse(store, job_id, RefusalReason::ValidationFailed, detail).await;
    }

    Ok(ToolCallOutcome::Executed {
        tool: tool_name,
        output,
        repair_attempts,
    })
}
