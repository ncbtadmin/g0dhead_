use crate::envelope::Envelope;
use crate::macros::closed_enum;
use uuid::Uuid;

closed_enum! {
    /// A.5 — the closed event taxonomy, v1. Extended only by version bump.
    /// JOB_TRANSITION is a v1 addition: Law I.1 requires each lifecycle
    /// transition to write a snapshot, and the taxonomy is closed *per
    /// schema version* — this is v1's entry for that event.
    LogEvent {
        IntakeRawCopied => "INTAKE_RAW_COPIED",
        Normalized => "NORMALIZED",
        Embedded => "EMBEDDED",
        LinkDrawn => "LINK_DRAWN",
        LinkSevered => "LINK_SEVERED",
        WeightRecalc => "WEIGHT_RECALC",
        PostulantEmerged => "POSTULANT_EMERGED",
        AuditOpened => "AUDIT_OPENED",
        ReportFiled => "REPORT_FILED",
        ProposalFiled => "PROPOSAL_FILED",
        Committed => "COMMITTED",
        Decommissioned => "DECOMMISSIONED",
        OverrideLaid => "OVERRIDE_LAID",
        PetitionOpened => "PETITION_OPENED",
        PetitionResolved => "PETITION_RESOLVED",
        Admitted => "ADMITTED",
        Rejected => "REJECTED",
        Refusal => "REFUSAL",
        Violation => "VIOLATION",
        JobTransition => "JOB_TRANSITION",
        Classified => "CLASSIFIED",
        Amended => "AMENDED",
        ProposalResolved => "PROPOSAL_RESOLVED",
        EnvEstablished => "ENV_ESTABLISHED",
        EnvOrphaned => "ENV_ORPHANED",
        PairingFormed => "PAIRING_FORMED",
        InstructionFlagged => "INSTRUCTION_FLAGGED",
        ConcordatAdopted => "CONCORDAT_ADOPTED",
        BiasWarning => "BIAS_WARNING",
        ReturnFlagged => "RETURN_FLAGGED",
        Refined => "REFINED",
    }
}

closed_enum! {
    /// A.5 — log severity.
    Severity {
        Info => "info",
        Warning => "warning",
        Violation => "violation",
        Suppressed => "suppressed",
    }
}

/// A.5 — an append-only log snapshot. `prior_ref` chains to the snapshot it
/// rotates (V.1: nothing overwrites); `seq` is store-issued and establishes
/// order (Law XII.2 — never wall-clock comparison).
#[derive(Debug, Clone)]
pub struct LogSnapshot {
    pub log_id: Uuid,
    pub seq: i64,
    pub subject_ref: String,
    pub event: LogEvent,
    pub payload: serde_json::Value,
    pub prior_ref: Option<Uuid>,
    pub severity: Severity,
    pub envelope: Envelope,
}
