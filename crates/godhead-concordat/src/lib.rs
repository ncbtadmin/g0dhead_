//! godhead-concordat — the Teacher's core function (Holy Standard §1, §3):
//! the Instruction, the Executability Lint (the Teacher's mandatory
//! VALIDATE_OUT), the Concordat contract, and the Bias Doctrine.
//!
//! An Instruction is not intent; it is text, received by a stranger who
//! cannot ask. The lint is the gate between a thought and a record: an
//! unexecutable instruction is a malformed handoff, refused before it
//! flags.

pub mod bias;
pub mod lint;

use godhead_schemas::SchemaRegistry;
use godhead_store::StoreError;
use semver::VersionReq;
use thiserror::Error;

pub use bias::{compute_skew, disclose_regular_output, BIAS_SCOPE};
pub use lint::{lint_instruction, read_instruction, write_instruction, LintFailure};

/// The v1 supported Concordat range every agent declares (§2.4): additive
/// changes bump minor, breaking changes bump major; skew is refused.
pub const SUPPORTED_CONCORDAT: &str = "^1.0";

/// The schema of the flagged instruction-pointer artifact.
pub const INSTRUCTION_POINTER_SCHEMA: &str = "concordat.instruction_pointer";

/// Adds the concordat schemas to a build registry (Law II.4).
pub fn register_into(reg: &mut SchemaRegistry) {
    reg.register(
        INSTRUCTION_POINTER_SCHEMA,
        VersionReq::parse("^1.0").expect("valid req"),
        |payload| {
            let id = payload
                .get("ref")
                .and_then(|v| v.as_str())
                .ok_or("field 'ref' (string) is required")?;
            uuid::Uuid::parse_str(id).map_err(|_| "ref must be a uuid".to_string())?;
            Ok(())
        },
    );
}

#[derive(Debug, Error)]
pub enum ConcordatError {
    #[error(transparent)]
    Store(#[from] StoreError),
    /// The Executability Lint failed; the Instruction is not written
    /// (Law VII refusal already recorded).
    #[error("LINT_FAILED: {0}")]
    LintFailed(String),
    #[error("SCHEMA_MISMATCH: {0}")]
    SchemaMismatch(String),
}
