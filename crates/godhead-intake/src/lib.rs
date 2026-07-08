//! godhead-intake — the deterministic onboard pipe (doc 2): commit → raw
//! copy → normalization → floor classification → at rest.
//!
//! The whole path runs without a reasoner (doc 2 §1.2). Every stage is a
//! full Book I lifecycle over the slice-1 store — job, lease, work,
//! artifact, flag, terminate — and stages correspond only through
//! readiness flags. The dispatcher's successor map ends at CLASSIFY: the
//! seam (doc 2 §4) is the absence of any rule beyond at-rest.

pub mod classify;
pub mod dispatch;
pub mod normalize;
pub mod pipe;
pub mod supervise;

use godhead_schemas::SchemaRegistry;
use semver::VersionReq;
use thiserror::Error;

pub use dispatch::{DispatchedStage, Dispatcher};
pub use pipe::IntakePipe;
pub use supervise::{NodeProgress, Supervisor};

/// Stage names as they appear in readiness flags. Closed set for the
/// intake pipe; the dispatcher maps RAW_COPY → NORMALIZE → CLASSIFY and
/// nothing beyond.
pub const STAGE_RAW_COPY: &str = "intake:raw_copy";
pub const STAGE_NORMALIZE: &str = "intake:normalize";
pub const STAGE_CLASSIFY: &str = "intake:classify";
pub const STAGE_RENORMALIZE: &str = "intake:renormalize";

/// The schema of every intake stage's output artifact.
pub const STAGE_RESULT_SCHEMA: &str = "intake.stage_result";

#[derive(Debug, Error)]
pub enum IntakeError {
    #[error(transparent)]
    Store(#[from] godhead_store::StoreError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("NOT_FOUND: {0}")]
    NotFound(String),
    /// The labor refused per Law VII; the RefusalRecord is already written.
    #[error("REFUSED: {0}")]
    Refused(String),
}

/// The intake build's declared schema support (Law II.4).
pub fn registry() -> SchemaRegistry {
    let mut reg = SchemaRegistry::new();
    register_into(&mut reg);
    reg
}

/// Adds the intake schemas to a build registry (composition surface).
pub fn register_into(reg: &mut SchemaRegistry) {
    reg.register(
        STAGE_RESULT_SCHEMA,
        VersionReq::parse("^1.0").expect("valid req"),
        |payload| {
            let obj = payload.as_object().ok_or("payload must be an object")?;
            let node_id = obj
                .get("node_id")
                .and_then(|v| v.as_str())
                .ok_or("field 'node_id' (string) is required")?;
            uuid::Uuid::parse_str(node_id).map_err(|_| "node_id must be a uuid".to_string())?;
            obj.get("stage")
                .and_then(|v| v.as_str())
                .ok_or("field 'stage' (string) is required")?;
            obj.get("outcome")
                .and_then(|v| v.as_str())
                .ok_or("field 'outcome' (string) is required")?;
            Ok(())
        },
    );
}

/// SHA-256 as lowercase hex — the checksum of record throughout intake.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
