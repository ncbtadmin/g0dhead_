use crate::envelope::Envelope;
use time::OffsetDateTime;
use uuid::Uuid;

/// Doc 3 §2.2 — one persisted vector per node. Never recomputed when it
/// can be read.
#[derive(Debug, Clone)]
pub struct EmbeddingRecord {
    pub node_id: Uuid,
    pub vector: Vec<f32>,
    pub embedder_alias: String,
    pub dims: i32,
    pub revision: i32,
    pub envelope: Envelope,
}

/// Doc 3 §2.3 — a bond: first-class, mutable, quarantined from the atoms.
/// Canonical ordering: source_ref < target_ref, one record per pair.
#[derive(Debug, Clone)]
pub struct LinkRecord {
    pub link_id: Uuid,
    pub source_ref: Uuid,
    pub target_ref: Uuid,
    pub similarity: f32,
    pub weight: f32,
    pub category: String,
    pub user_overridden: bool,
    pub revision: i32,
    pub envelope: Envelope,
}

/// Doc 4 §5.2 — recalculation eligibility per category: ingestion marks,
/// only a user-chosen trigger executes.
#[derive(Debug, Clone)]
pub struct RebalanceState {
    pub category: String,
    pub eligible: bool,
    pub marked_at: Option<OffsetDateTime>,
    pub last_recalc_at: Option<OffsetDateTime>,
    pub config_rev: Option<i32>,
    pub revision: i32,
    pub envelope: Envelope,
}

/// The consumer-facing weight surface (doc 4 §5.4): below the coherence
/// threshold weights are inert — present in the record, absent in force.
#[derive(Debug, Clone)]
pub struct LiveWeights {
    pub category: String,
    pub live: bool,
    pub density: f32,
    /// Law VI.1: every density evaluation cites the config revision it read.
    pub config_rev: i32,
    /// Empty when not live: inert weights exert no force in any consumer.
    pub weights: Vec<(Uuid, f32)>,
}
