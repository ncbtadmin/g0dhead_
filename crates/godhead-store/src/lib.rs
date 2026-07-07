//! godhead-store — the single abstracted store interface (doc 3 §1.1) and
//! its Postgres substrate.
//!
//! Every Book I invariant that is "store-enforced" is enforced HERE, at the
//! write layer — never left to agent etiquette. Slice 1
//! (docs/dev/SLICE_01.md) implements: forward-only job lifecycle, idempotent
//! keyed writes, envelope/schema validation at write, flag-after-output
//! certification, no-delete flag supersession, refusal-with-preservation,
//! leases with CAS revisions, store-issued UTC clock, attributable writes,
//! mandatory budgets, and the outbound secret scan.
//!
//! Law III.1 (SC-B04): the [`Store`] trait is the sole inter-agent surface.
//! This crate deliberately exposes no messaging, socket, or channel API of
//! any kind — agents correspond only through the eternal record.

pub mod error;
pub mod interface;
pub mod postgres;
pub mod secrets;
pub mod types;

pub use error::StoreError;
pub use interface::Store;
pub use postgres::PgStore;
pub use types::{ArtifactDraft, ArtifactRecord, ComplianceMetrics};
