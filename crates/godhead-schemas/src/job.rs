use crate::envelope::Envelope;
use crate::error::SchemaError;
use crate::macros::closed_enum;
use semver::Version;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

closed_enum! {
    /// A.2 — the closed roster of ephemeral agent kinds.
    AgentType {
        Slave => "SLAVE",
        Aggregator => "AGGREGATOR",
        Notary => "NOTARY",
        Auditor => "AUDITOR",
        Reconciler => "RECONCILER",
        Student => "STUDENT",
        Teacher => "TEACHER",
    }
}

closed_enum! {
    /// A.2 — Gabriel and Lucy are named, not titled (Law X.6).
    AuditorName {
        Gabriel => "GABRIEL",
        Lucy => "LUCY",
    }
}

closed_enum! {
    /// A.2 — the specificity axis (Holy Standard §4, Handbook §1).
    Tier {
        Regular => "REGULAR",
        Devout => "DEVOUT",
        Canon => "CANON",
    }
}

closed_enum! {
    /// A.2 / Law I.1 — the canonical lifecycle, forward-only.
    JobStatus {
        Pending => "PENDING",
        Leased => "LEASED",
        Running => "RUNNING",
        Written => "WRITTEN",
        Flagged => "FLAGGED",
        Terminated => "TERMINATED",
        Refused => "REFUSED",
    }
}

impl JobStatus {
    /// The single forward successor along Law I.1's chain.
    pub fn successor(self) -> Option<JobStatus> {
        match self {
            JobStatus::Pending => Some(JobStatus::Leased),
            JobStatus::Leased => Some(JobStatus::Running),
            JobStatus::Running => Some(JobStatus::Written),
            JobStatus::Written => Some(JobStatus::Flagged),
            JobStatus::Flagged => Some(JobStatus::Terminated),
            JobStatus::Terminated | JobStatus::Refused => None,
        }
    }

    /// Live = the agent still labors. Law I.1: REFUSED is reachable from any
    /// live state. FLAGGED is not live — after FLAG the labor is surrendered
    /// (I.4) and only TERMINATE remains.
    pub fn is_live(self) -> bool {
        matches!(
            self,
            JobStatus::Pending | JobStatus::Leased | JobStatus::Running | JobStatus::Written
        )
    }

    /// Terminal: nothing of the agent survives (I.4).
    pub fn is_terminal(self) -> bool {
        matches!(self, JobStatus::Terminated | JobStatus::Refused)
    }

    /// Law I.1's full legality relation: strictly one step forward, plus
    /// REFUSED from any live state. Any other pair is rejected by the store.
    pub fn may_transition_to(self, next: JobStatus) -> bool {
        self.successor() == Some(next) || (next == JobStatus::Refused && self.is_live())
    }
}

/// Law XIV — every invocation carries budgets; no unbounded agent exists.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Budgets {
    pub max_wall_ms: i64,
    pub max_tool_calls: i32,
    pub max_tokens: i64,
}

impl Budgets {
    /// A budget of zero or less is no budget: the record fails validation
    /// (SC-H05) rather than admitting an unbounded or unstartable labor.
    pub fn validate(&self) -> Result<(), SchemaError> {
        if self.max_wall_ms <= 0 || self.max_tool_calls <= 0 || self.max_tokens <= 0 {
            return Err(SchemaError::ValidationFailed(format!(
                "Law XIV: budgets must all be positive, got wall_ms={}, tool_calls={}, tokens={}",
                self.max_wall_ms, self.max_tool_calls, self.max_tokens
            )));
        }
        Ok(())
    }
}

/// What the dispatcher supplies at spawn. No status, no timestamps, no
/// revision — those are the store's to issue.
#[derive(Debug, Clone)]
pub struct JobDraft {
    pub agent_type: AgentType,
    pub auditor_name: Option<AuditorName>,
    pub tier: Option<Tier>,
    pub input_refs: Vec<Uuid>,
    pub env_ref: Option<Uuid>,
    pub brief_ref: Option<Uuid>,
    /// Endpoint alias only, never a key (Law XV); None for modelless labor.
    pub endpoint_alias: Option<String>,
    pub manual_version: Version,
    pub budgets: Budgets,
}

/// A.2 — the persisted JobRecord.
#[derive(Debug, Clone)]
pub struct JobRecord {
    pub job_id: Uuid,
    pub agent_type: AgentType,
    pub auditor_name: Option<AuditorName>,
    pub tier: Option<Tier>,
    pub status: JobStatus,
    pub attempt: i32,
    pub input_refs: Vec<Uuid>,
    pub env_ref: Option<Uuid>,
    pub brief_ref: Option<Uuid>,
    pub endpoint_alias: Option<String>,
    pub manual_version: Version,
    pub budgets: Budgets,
    pub started_at: Option<OffsetDateTime>,
    pub heartbeat_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
    pub revision: i32,
    pub envelope: Envelope,
}
