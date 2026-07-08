//! godhead-schemas — the rigidly standardized record types of the order.
//!
//! Slice 1 scope (docs/dev/SLICE_01.md): the Appendix A schemas the Book I
//! enforcement layer requires — Envelope (A.1), JobRecord (A.2),
//! ReadinessFlag (A.3), RefusalRecord (A.4), LogSnapshot (A.5),
//! LeaseRecord (A.13), ConfigConstant (A.14) — with closed enums and the
//! Law II schema registry.
//!
//! Design invariants carried in the types themselves:
//! - Draft types carry no ids, timestamps, statuses, or revisions — those
//!   are store-issued (Laws XII, XI.3), so an agent structurally cannot
//!   supply them.
//! - Enums are closed (Book I conventions); parsing admits listed values
//!   only.

mod macros;

pub mod config;
pub mod envelope;
pub mod environment;
pub mod error;
pub mod flag;
pub mod job;
pub mod lease;
pub mod log;
pub mod matrix;
pub mod ml;
pub mod node;
pub mod refusal;
pub mod registry;
pub mod sovereignty;

pub use config::{ConfigConstant, ConfigTier};
pub use envelope::Envelope;
pub use environment::{
    roman_ordinal, roster_index, EnvItem, EnvKind, EnvStatus, EnvironmentRecord, PairingKind,
    PairingRecord,
};
pub use error::SchemaError;
pub use flag::{Certifies, FlagDraft, FlagStatus, ReadinessFlag, Validator};
pub use job::{AgentType, AuditorName, Budgets, JobDraft, JobRecord, JobStatus, Tier};
pub use lease::LeaseRecord;
pub use log::{LogEvent, LogSnapshot, Severity};
pub use matrix::{
    Amendment, AmendmentKind, AuditReport, AuditReportDraft, AuditorKind, Claim, ClaimSeverity,
    JointProposal, MatrixRecord, MatrixStatus, ProposalDraft, ReportKind, Verdict,
};
pub use ml::{EmbeddingRecord, LinkRecord, LiveWeights, RebalanceState};
pub use node::{IntakeStatus, NodeDraft, NodeRecord, NormalizeOutcome};
pub use refusal::{Law, RefusalDraft, RefusalReason, RefusalRecord};
pub use registry::{SchemaRegistry, SchemaSpec, ValidatorFn};
pub use sovereignty::{
    ConsentDecision, ConsentRecord, ConsentScope, OverrideBasis, OverrideKind, OverrideRecord,
    PetitionDraft, PetitionRecord, PetitionStatus,
};

/// The schema version of the slice-1 record types themselves.
pub const RECORD_SCHEMA_VERSION: &str = "1.0.0";
