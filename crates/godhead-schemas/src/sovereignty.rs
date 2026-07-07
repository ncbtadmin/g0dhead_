use crate::envelope::Envelope;
use crate::macros::closed_enum;
use time::OffsetDateTime;
use uuid::Uuid;

closed_enum! {
    /// A.7 — what kind of change a hand (or petition) concerns. In slice 3
    /// only CATEGORY_REASSIGNED has a live surface (node classification);
    /// the others gain enforcement with the link and weight slices.
    OverrideKind {
        LinkSevered => "LINK_SEVERED",
        LinkForced => "LINK_FORCED",
        CategoryReassigned => "CATEGORY_REASSIGNED",
        WeightCorrected => "WEIGHT_CORRECTED",
    }
}

closed_enum! {
    /// A.7 — how the protection came to be: the sovereign's own hand, or a
    /// successor override laid under a granted petition (IV.5).
    OverrideBasis {
        SovereignHand => "SOVEREIGN_HAND",
        GrantedPetition => "GRANTED_PETITION",
    }
}

closed_enum! {
    /// A.7 — the petition lineage. One record per (subject, kind):
    /// recurrence escalates, it never duplicates.
    PetitionStatus {
        Open => "OPEN",
        Declined => "DECLINED",
        Escalated => "ESCALATED",
        Granted => "GRANTED",
        Silenced => "SILENCED",
    }
}

closed_enum! {
    /// A.12 — every decision a consent record can carry.
    ConsentDecision {
        Admitted => "ADMITTED",
        Rejected => "REJECTED",
        Granted => "GRANTED",
        Declined => "DECLINED",
        Silenced => "SILENCED",
    }
}

closed_enum! {
    /// A.12 — consent scope.
    ConsentScope {
        Item => "ITEM",
        Batch => "BATCH",
    }
}

/// A.7 — OverrideRecord: the sovereign's hand, laid on a datum. The latest
/// record for a subject is the active protection; prior_ref walks the
/// successor chain backward through every grant.
#[derive(Debug, Clone)]
pub struct OverrideRecord {
    pub override_id: Uuid,
    pub subject_ref: Uuid,
    pub kind: OverrideKind,
    pub basis: OverrideBasis,
    pub prior_ref: Option<Uuid>,
    /// Resolves iff basis is GRANTED_PETITION.
    pub consent_ref: Option<Uuid>,
    /// The state the hand protects, as laid.
    pub protected_state: serde_json::Value,
    pub user_overridden: bool,
    pub laid_at: OffsetDateTime,
    pub envelope: Envelope,
}

/// What a petitioning agent supplies (IV.2). `proposed_change` is v1
/// mechanical necessity: the Notary applies exactly what was petitioned,
/// so the petition must carry it.
#[derive(Debug, Clone)]
pub struct PetitionDraft {
    pub subject_ref: Uuid,
    pub change_kind: OverrideKind,
    pub reason: String,
    pub evidence_refs: Vec<String>,
    pub proposed_change: serde_json::Value,
}

/// A.7 — PetitionRecord.
#[derive(Debug, Clone)]
pub struct PetitionRecord {
    pub petition_id: Uuid,
    pub subject_ref: Uuid,
    pub change_kind: OverrideKind,
    pub reason: String,
    pub evidence_refs: Vec<String>,
    pub proposed_change: serde_json::Value,
    pub status: PetitionStatus,
    pub occurrence_count: i32,
    pub consent_ref: Option<Uuid>,
    /// The Notary job that executed the grant (IV.5); null until executed.
    pub execution_job_ref: Option<Uuid>,
    pub resolved_at: Option<OffsetDateTime>,
    pub envelope: Envelope,
}

/// A.12 — ConsentRecord. `decided_by` is always the sovereign today; the
/// field exists for the multi-tenant future.
#[derive(Debug, Clone)]
pub struct ConsentRecord {
    pub consent_id: Uuid,
    pub subject_ref: Uuid,
    pub decision: ConsentDecision,
    pub scope: ConsentScope,
    pub decided_by: String,
    pub decided_at: OffsetDateTime,
    pub envelope: Envelope,
}
