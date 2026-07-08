//! godhead-toolcall — Law VIII: a tool call is a handoff wearing gloves.
//!
//! Constrain where the endpoint supports it; validate before execution;
//! feed failures back verbatim and regenerate, boundedly; validate after
//! execution; refuse when the ladder is exhausted. The law is
//! provider-blind: weak local models receive no leniency — the ladder IS
//! the leniency (VIII.5). An invalid call is never executed, on any
//! guess, ever.

pub mod ladder;

use godhead_schemas::SchemaRegistry;
use semver::VersionReq;
use std::collections::BTreeMap;
use std::sync::Arc;
use thiserror::Error;

pub use ladder::{run_tool_call, ToolCallOutcome};

/// The schema of the persisted tool_call artifact.
pub const TOOL_CALL_SCHEMA: &str = "toolcall.result";
/// The artifact slot an executed call is recorded under.
pub const TOOL_CALL_SLOT: &str = "tool_call";

#[derive(Debug, Error)]
pub enum ToolCallError {
    #[error(transparent)]
    Store(#[from] godhead_store::StoreError),
    /// The ladder exhausted or the output could not be trusted; the
    /// refusal is already on the record (Law VII).
    #[error("REFUSED: {0}")]
    Refused(String),
}

/// One tool: its declared contract and its labor. Validators return a
/// human-readable defect — that text is what the ladder feeds back
/// verbatim (VIII.3).
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    /// One line for the roster feedback.
    fn description(&self) -> &str;
    /// VIII.4: an idempotent tool may be re-executed exactly once on
    /// invalid output; a non-idempotent tool may not.
    fn idempotent(&self) -> bool;
    fn validate_input(&self, args: &serde_json::Value) -> Result<(), String>;
    fn validate_output(&self, output: &serde_json::Value) -> Result<(), String>;
    async fn execute(&self, args: &serde_json::Value) -> serde_json::Value;
}

/// The closed roster of tools an invocation may call.
#[derive(Default, Clone)]
pub struct ToolRegistry {
    tools: BTreeMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    /// The roster text fed back on an invalid call (VIII.3).
    pub fn roster_text(&self) -> String {
        if self.tools.is_empty() {
            return "(the roster is empty: no tool may be called)".to_string();
        }
        self.tools
            .values()
            .map(|t| format!("- {}: {}", t.name(), t.description()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// The serving-endpoint seam: whatever produces tool calls implements
/// this. `constrained` MUST be honored when `supports_constrained` is
/// true (VIII.1) — the ladder passes it accordingly.
#[async_trait::async_trait]
pub trait ToolCaller: Send + Sync {
    fn supports_constrained(&self) -> bool;
    /// Produce one raw emission. `feedback` carries the previous attempt's
    /// validator errors and the roster, verbatim, when repairing.
    async fn propose_call(
        &self,
        context: &str,
        feedback: Option<&str>,
        constrained: bool,
    ) -> String;
}

/// Adds the tool-call schemas to a build registry (Law II.4).
pub fn register_into(reg: &mut SchemaRegistry) {
    reg.register(
        TOOL_CALL_SCHEMA,
        VersionReq::parse("^1.0").expect("valid req"),
        |payload| {
            let obj = payload.as_object().ok_or("payload must be an object")?;
            let tool = obj
                .get("tool")
                .and_then(|v| v.as_str())
                .ok_or("field 'tool' (string) is required")?;
            if tool.is_empty() {
                return Err("field 'tool' must be non-empty".into());
            }
            if !obj.get("attempts").is_some_and(serde_json::Value::is_u64) {
                return Err("field 'attempts' (unsigned integer) is required".into());
            }
            if !obj.contains_key("arguments") || !obj.contains_key("output") {
                return Err("fields 'arguments' and 'output' are required".into());
            }
            Ok(())
        },
    );
}
