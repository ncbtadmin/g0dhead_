//! A.12 — the Deacon's threshold records (Dogma Book II §1): ScanVerdict,
//! QuarantineItem, and the Manifest. The ConsentRecord the protocol shares
//! with Law IV lives in `sovereignty`.
//!
//! Drafts carry no ids, timestamps, or statuses — store-issued (Laws XI,
//! XII). External-origin content rests in the quarantine namespace and
//! nowhere else (Law V.4); only `CLEAN + ADMITTED` leaves it, and only into
//! the onboard pipe at its beginning (Book II §1 step 6).

use crate::envelope::Envelope;
use crate::macros::closed_enum;
use time::OffsetDateTime;
use uuid::Uuid;

closed_enum! {
    /// A.12 — the closed verdict set. `INFECTED | SUSPECT | ERROR` items are
    /// held and are never presented as admissible (Book II §1 step 3).
    ScanVerdictKind {
        Clean => "CLEAN",
        Infected => "INFECTED",
        Suspect => "SUSPECT",
        Error => "ERROR",
    }
}

/// A.12 — the scanning engine's identity: abstracted like all endpoints,
/// referenced by alias only (Law XV.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanEngine {
    pub alias: String,
    pub version: String,
    pub signature_rev: Option<String>,
}

/// A.12 — one scan's verdict over one quarantined item. Office-authored:
/// only the Deacon's path writes these (ruling G10).
#[derive(Debug, Clone)]
pub struct ScanVerdict {
    pub scan_id: Uuid,
    pub item_ref: Uuid,
    pub verdict: ScanVerdictKind,
    pub engine: ScanEngine,
    pub scanned_at: OffsetDateTime,
    pub envelope: Envelope,
}

/// What lands in quarantine (A.12): the external bytes at rest, their
/// origin (the fetching job and the human mandate or brief that sent it —
/// every outward act begins in a human hand), and the item's standing
/// against the protocol. `admitted_node_ref` is the admission's convergence
/// witness: set exactly once when the item enters the onboard pipe, so a
/// retried admission reads it back instead of copying twice (Law I.3).
#[derive(Debug, Clone)]
pub struct QuarantineItem {
    pub item_ref: Uuid,
    pub origin_job_ref: Uuid,
    /// The human mandate that drove the trip (C.4) — or None for a
    /// BRIEF-rooted arrival, which carries `brief_ref` instead.
    pub mandate_ref: Option<Uuid>,
    pub brief_ref: Option<Uuid>,
    pub filename: String,
    pub declared_type: String,
    pub content: Vec<u8>,
    /// The latest verdict on this item, if any scan has run.
    pub scan_ref: Option<Uuid>,
    /// The consent that admitted or rejected it, if the sovereign has ruled.
    pub consent_ref: Option<Uuid>,
    /// Set exactly once at admission (the node the onboard pipe minted).
    pub admitted_node_ref: Option<Uuid>,
    pub held_since: OffsetDateTime,
    pub revision: i32,
    pub envelope: Envelope,
}

/// What a fetching labor (or, in this slice, a fixture standing where the
/// fetch layer will) hands the quarantine wall. The store issues ids and
/// timestamps; the writer never chooses them.
#[derive(Debug, Clone)]
pub struct QuarantineDraft {
    pub mandate_ref: Option<Uuid>,
    pub brief_ref: Option<Uuid>,
    pub filename: String,
    pub declared_type: String,
    pub content: Vec<u8>,
}

/// Book II §1 step 4 — the Manifest: items, full provenance chains,
/// verdicts, presented whole to the sovereign. One Manifest serves one
/// mandate-trip, never pooled across trips (the doctrine of the deliberate
/// gate, ruling G11); `standing_notice` carries SC-I07b's graduated
/// legibility when admission volume or rate crosses the operational
/// constants — never blocking, never silent.
#[derive(Debug, Clone)]
pub struct Manifest {
    pub manifest_id: Uuid,
    pub mandate_ref: Uuid,
    /// The trip's execution job — the uniqueness key of "one Manifest per
    /// mandate-trip".
    pub trip_job_ref: Uuid,
    /// `[{item_ref, verdict, chain}]` — what the sovereign consents over.
    pub items: serde_json::Value,
    pub standing_notice: Option<String>,
    pub presented_at: OffsetDateTime,
    pub envelope: Envelope,
}
