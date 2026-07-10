//! C.2 — the ProvenanceChain, as a persisted table (ruling G8): for
//! external-origin data the chain is the whole point, so it is A.6-as-table
//! here — append-in-flight (Handbook §4.2: each fetch or follow-up writes
//! its entry BEFORE its results may be written; an item write validates its
//! producing entry already exists, else `PROVENANCE_INCOMPLETE`), rooted in
//! a human hand (`CANON | WRIT | BRIEF`), append-only, walked root-to-leaf.
//! Internal-origin provenance remains the declared view (Envelope →
//! JobRecord → logs), swept by SC-A08 — no registry-keeper.

use crate::envelope::Envelope;
use crate::macros::closed_enum;
use time::OffsetDateTime;
use uuid::Uuid;

closed_enum! {
    /// C.2 — the closed entry kinds. Roots are `CANON | WRIT | BRIEF`:
    /// every chain begins in a human hand.
    ChainEntryKind {
        Canon => "CANON",
        Writ => "WRIT",
        Brief => "BRIEF",
        Fetch => "FETCH",
        FollowUp => "FOLLOW_UP",
        Refinement => "REFINEMENT",
        Admission => "ADMISSION",
    }
}

impl ChainEntryKind {
    /// May this kind stand at a chain's root?
    pub fn is_human_root(self) -> bool {
        matches!(
            self,
            ChainEntryKind::Canon | ChainEntryKind::Writ | ChainEntryKind::Brief
        )
    }
}

/// What an appending labor supplies. `link_seq` is store-issued (the next
/// in the chain — append means append); ids and timestamps likewise.
#[derive(Debug, Clone)]
pub struct ChainEntryDraft {
    /// The subject whose arrival story this chain narrates (for external
    /// material: the quarantine item).
    pub chain_ref: Uuid,
    pub kind: ChainEntryKind,
    /// The human mandate or brief a root entry cites; None past the root.
    pub mandate_ref: Option<Uuid>,
    pub prompt_or_reason: String,
    /// Refs this entry produced; each must resolve when written.
    pub produced: Vec<Uuid>,
}

/// C.2 — one persisted chain entry.
#[derive(Debug, Clone)]
pub struct ChainEntry {
    pub chain_ref: Uuid,
    pub link_seq: i32,
    pub kind: ChainEntryKind,
    pub actor_job_ref: Uuid,
    pub mandate_ref: Option<Uuid>,
    pub prompt_or_reason: String,
    pub produced: Vec<Uuid>,
    pub at: OffsetDateTime,
    pub envelope: Envelope,
}
