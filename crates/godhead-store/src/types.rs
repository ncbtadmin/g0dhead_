use godhead_schemas::Envelope;
use semver::Version;
use uuid::Uuid;

/// A persisted output artifact — the generic write surface, keyed
/// `(job_id, output_slot)` per Law I.3.
#[derive(Debug, Clone)]
pub struct ArtifactRecord {
    pub job_id: Uuid,
    pub output_slot: String,
    pub payload: serde_json::Value,
    pub authoritative: bool,
    pub quarantine_marked: bool,
    pub revision: i32,
    pub envelope: Envelope,
}

/// What an agent supplies when writing an output. No flags, timestamps, or
/// revisions — the store issues those.
#[derive(Debug, Clone)]
pub struct ArtifactDraft {
    pub schema_name: String,
    pub schema_version: Version,
    pub payload: serde_json::Value,
}

/// Law VII.4 — the reference metrics shape: refusal is compliance, and no
/// derived metric may score it as error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComplianceMetrics {
    pub total: usize,
    /// Labors concluded lawfully: TERMINATED, FLAGGED, or REFUSED.
    pub compliant: usize,
    /// Of the compliant, how many were refusals (distinct, never an error).
    pub refused: usize,
    /// Still live (or abandoned mid-flight — recovery's concern, not error's).
    pub in_flight: usize,
}
