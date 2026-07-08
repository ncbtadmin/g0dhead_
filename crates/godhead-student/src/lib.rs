//! godhead-student — the Student's core function (Student Handbook §1, §3):
//! the ReturnManifest and its completion contract (everything a Student is
//! *for* ends in a Return, B.2), deterministic refinement with redundant
//! consistency (§1.2), and stewardship that treats the sovereign's hand as
//! fixed stars (§4.5).
//!
//! A Return is not a report of effort; it is an answer to an Instruction,
//! criterion by criterion, evidence in hand. A Return that does not
//! validate never flags, never poisons (Law VII).

pub mod refine;
pub mod returns;
pub mod steward;

use godhead_schemas::SchemaRegistry;
use godhead_store::StoreError;
use semver::VersionReq;
use thiserror::Error;

pub use refine::{re_derive, redundant_consistency, refine, ConsistencyDebris, REFINE_METHOD};
pub use returns::{validate_return, write_return, ReturnFailure};
pub use steward::{steward_consolidate, ConsolidationReport};

/// The Student's declared Concordat range (§2.4): skew in either direction
/// is refused at this end, never best-effort.
pub const SUPPORTED_CONCORDAT: &str = "^1.0";

/// The schema of the flagged return-pointer artifact.
pub const RETURN_POINTER_SCHEMA: &str = "student.return_pointer";

/// Adds the student schemas to a build registry (Law II.4).
pub fn register_into(reg: &mut SchemaRegistry) {
    reg.register(
        RETURN_POINTER_SCHEMA,
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
pub enum StudentError {
    #[error(transparent)]
    Store(#[from] StoreError),
    /// The Return failed its VALIDATE_OUT; nothing was written, nothing
    /// flags (Law VII refusal already recorded).
    #[error("RETURN_INVALID: {0}")]
    ReturnInvalid(String),
    /// A refinement source is not a stable function of store state — no
    /// derivative checksum to fold, or a derivation method this Student
    /// cannot re-run (§1.2b).
    #[error("NOT_REFINABLE: {0}")]
    NotRefinable(String),
}
