use crate::envelope::Envelope;
use crate::macros::closed_enum;
use uuid::Uuid;

closed_enum! {
    /// The Fifteen Laws, cited by RefusalRecords (A.4).
    Law {
        I => "I",
        II => "II",
        III => "III",
        IV => "IV",
        V => "V",
        VI => "VI",
        VII => "VII",
        VIII => "VIII",
        IX => "IX",
        X => "X",
        XI => "XI",
        XII => "XII",
        XIII => "XIII",
        XIV => "XIV",
        XV => "XV",
    }
}

closed_enum! {
    /// A.4 — the closed refusal reason enum.
    RefusalReason {
        SchemaMismatch => "SCHEMA_MISMATCH",
        ValidationFailed => "VALIDATION_FAILED",
        FlagUntrusted => "FLAG_UNTRUSTED",
        ToolMalformed => "TOOL_MALFORMED",
        ToolOutputInvalid => "TOOL_OUTPUT_INVALID",
        ProvenanceIncomplete => "PROVENANCE_INCOMPLETE",
        OverrideConflict => "OVERRIDE_CONFLICT",
        GateBypassAttempt => "GATE_BYPASS_ATTEMPT",
        EnvInvalid => "ENV_INVALID",
        LeaseConflict => "LEASE_CONFLICT",
        BudgetExceeded => "BUDGET_EXCEEDED",
        LawConflict => "LAW_CONFLICT",
    }
}

/// What a refusing agent supplies (Law VII.2). Ids and timestamps are the
/// store's to issue.
#[derive(Debug, Clone)]
pub struct RefusalDraft {
    pub law: Law,
    pub reason: RefusalReason,
    /// Refs to the offending state, as text refs (uuid or namespaced id).
    pub subject_refs: Vec<String>,
    pub detail: String,
    /// State the refusal quarantine-marks and preserves (VII.3).
    pub preserved_refs: Vec<String>,
}

/// A.4 — the persisted RefusalRecord. Refusal is compliance (VII.4).
#[derive(Debug, Clone)]
pub struct RefusalRecord {
    pub refusal_id: Uuid,
    pub job_id: Uuid,
    pub law: Law,
    pub reason: RefusalReason,
    pub subject_refs: Vec<String>,
    pub detail: String,
    pub preserved_refs: Vec<String>,
    pub envelope: Envelope,
}
