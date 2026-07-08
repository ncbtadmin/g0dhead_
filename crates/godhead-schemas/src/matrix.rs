use crate::envelope::Envelope;
use crate::macros::closed_enum;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

closed_enum! {
    /// A.9 — the matrix arc: grown, tried, professed — or dissolved.
    MatrixStatus {
        Postulant => "POSTULANT",
        Cardinal => "CARDINAL",
        Dissolved => "DISSOLVED",
    }
}

closed_enum! {
    /// A.11 — who audited.
    AuditorKind {
        Gabriel => "GABRIEL",
        Lucy => "LUCY",
    }
}

closed_enum! {
    /// A.11 — what a report is.
    ReportKind {
        Affirmation => "AFFIRMATION",
        Indictment => "INDICTMENT",
    }
}

closed_enum! {
    /// A.11 — the Joint Proposal's verdict.
    Verdict {
        Commit => "COMMIT",
        Amend => "AMEND",
        Reject => "REJECT",
    }
}

closed_enum! {
    /// The closed v1 amendment vocabulary: membership edits only. The
    /// records referenced are never destroyed — bonds outlive structures
    /// (Law VI.5).
    AmendmentKind {
        RemoveLink => "REMOVE_LINK",
        RemoveNode => "REMOVE_NODE",
    }
}

closed_enum! {
    /// A.11 — claim severity, where a claim carries one.
    ClaimSeverity {
        High => "high",
        Medium => "medium",
        Low => "low",
    }
}

/// One enumerated change inside an AMEND verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Amendment {
    pub kind: String,
    pub subject_ref: Uuid,
}

/// A.11 — one claim in an audit report. Every claim MUST carry evidence
/// that resolves to live store records (the truth-binding): an unsupported
/// word does not validate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub claim: String,
    pub evidence_refs: Vec<Uuid>,
    pub severity: Option<String>,
}

/// A.9 — the MatrixRecord: the authoritative state of a matrix, and the
/// only one (there is no registry-keeper).
#[derive(Debug, Clone)]
pub struct MatrixRecord {
    pub matrix_id: Uuid,
    pub status: MatrixStatus,
    pub category: String,
    pub revision: i32,
    pub audit_depth: i32,
    pub node_refs: Vec<Uuid>,
    pub link_refs: Vec<Uuid>,
    /// The Aggregator job whose evaluation recorded the emergence.
    pub emerged_by: Uuid,
    /// Law VI.1 — the coherence-threshold revision the evaluation cited.
    pub config_rev: i32,
    pub committed_proposal_ref: Option<Uuid>,
    pub committed_consent_ref: Option<Uuid>,
    pub committed_at: Option<OffsetDateTime>,
    pub envelope: Envelope,
}

/// What an auditor files (A.11). Ids and timestamps are the store's.
#[derive(Debug, Clone)]
pub struct AuditReportDraft {
    pub matrix_ref: Uuid,
    pub matrix_revision: i32,
    pub auditor: AuditorKind,
    pub kind: ReportKind,
    pub claims: Vec<Claim>,
}

/// A.11 — the persisted AuditReport.
#[derive(Debug, Clone)]
pub struct AuditReport {
    pub report_id: Uuid,
    pub job_id: Uuid,
    pub matrix_ref: Uuid,
    pub matrix_revision: i32,
    pub auditor: AuditorKind,
    pub kind: ReportKind,
    pub claims: Vec<Claim>,
    pub envelope: Envelope,
}

/// What Reconciliation files (A.11).
#[derive(Debug, Clone)]
pub struct ProposalDraft {
    pub matrix_ref: Uuid,
    pub matrix_revision: i32,
    pub report_refs: [Uuid; 2],
    pub verdict: Verdict,
    /// Required iff AMEND.
    pub changes: Vec<Amendment>,
    /// Required iff REJECT.
    pub reasons: Vec<String>,
}

/// A.11 — the persisted JointProposal.
#[derive(Debug, Clone)]
pub struct JointProposal {
    pub proposal_id: Uuid,
    pub job_id: Uuid,
    pub matrix_ref: Uuid,
    pub matrix_revision: i32,
    pub report_refs: Vec<Uuid>,
    pub verdict: Verdict,
    pub changes: Vec<Amendment>,
    pub reasons: Vec<String>,
    pub consent_ref: Option<Uuid>,
    pub envelope: Envelope,
}
