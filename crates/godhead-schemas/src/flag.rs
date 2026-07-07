use crate::envelope::Envelope;
use crate::macros::closed_enum;
use semver::Version;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

closed_enum! {
    /// A.3 / Law III.4 — flags are never deleted, only superseded by status.
    FlagStatus {
        Active => "ACTIVE",
        Consumed => "CONSUMED",
        Distrusted => "DISTRUSTED",
        Superseded => "SUPERSEDED",
    }
}

/// A.3 — what the flag certifies: named output slots of the flagging job and
/// the exact revisions that passed VALIDATE_OUT. A reader re-validates these
/// (Law III.3); a revision drift means the state moved after certification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certifies {
    pub output_slots: Vec<String>,
    pub revisions: Vec<i32>,
}

/// A.3 — the validator's identity and version, recorded in the flag (II.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub id: String,
    pub version: String,
}

/// What an agent supplies at FLAG. Status, id, and timestamps are the
/// store's to issue.
#[derive(Debug, Clone)]
pub struct FlagDraft {
    pub stage: String,
    pub certifies: Certifies,
    pub validator: Validator,
}

/// A.3 — the persisted ReadinessFlag: a certification of integrity, not a
/// done-marker (Law III.2).
#[derive(Debug, Clone)]
pub struct ReadinessFlag {
    pub flag_id: Uuid,
    pub job_id: Uuid,
    pub stage: String,
    pub certifies: Certifies,
    pub validator: Validator,
    pub status: FlagStatus,
    pub revision: i32,
    pub envelope: Envelope,
}

impl ReadinessFlag {
    /// Schema version of the ReadinessFlag record itself.
    pub fn record_version() -> Version {
        Version::new(1, 0, 0)
    }
}
