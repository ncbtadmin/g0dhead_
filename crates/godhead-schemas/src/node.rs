use crate::envelope::Envelope;
use crate::macros::closed_enum;
use uuid::Uuid;

closed_enum! {
    /// Where a node stands on the intake path (doc 2). RAW: copied, not yet
    /// normalized. DECODE_FAILED / UNSUPPORTED are surfaced states, never
    /// silent ones — the raw atom is stored and preserved in all four.
    IntakeStatus {
        Raw => "RAW",
        Normalized => "NORMALIZED",
        DecodeFailed => "DECODE_FAILED",
        Unsupported => "UNSUPPORTED",
    }
}

/// What the raw-copy labor supplies at node creation. Raw reference fields
/// exist ONLY here: no later store method accepts them, and the substrate
/// rejects their mutation — raw-copied-once, structurally (doc 3 §4.2).
#[derive(Debug, Clone)]
pub struct NodeDraft {
    pub filename: String,
    /// Lowercased extension without the dot; "" when the file has none.
    pub filetype: String,
    pub size_bytes: i64,
    /// Path relative to the deployment's data root.
    pub raw_path: String,
    pub raw_sha256: String,
}

/// The outcome of normalization (doc 2 §2.3), applied by the store to the
/// node. Failures are recorded states, not errors — flag, don't bury.
#[derive(Debug, Clone)]
pub enum NormalizeOutcome {
    Normalized {
        derivative_path: String,
        derivative_sha256: String,
    },
    DecodeFailed {
        reason: String,
    },
    Unsupported {
        notice: String,
    },
}

/// Doc 3 §2.1 — the atom. Raw content by reference, derivative by
/// reference, metadata; content immutable once committed.
#[derive(Debug, Clone)]
pub struct NodeRecord {
    pub node_id: Uuid,
    pub filename: String,
    pub filetype: String,
    pub size_bytes: i64,
    pub raw_path: String,
    pub raw_sha256: String,
    pub derivative_path: Option<String>,
    pub derivative_sha256: Option<String>,
    pub normalized: bool,
    pub intake_status: IntakeStatus,
    /// Floor classification (doc 2 §2.5): low-trust bucket entries the AI
    /// layer knows not to over-weight.
    pub classification: serde_json::Value,
    /// Incompatibility notice / decode-failure reason, surfaced not buried.
    pub notice: Option<String>,
    pub revision: i32,
    pub envelope: Envelope,
}
