use semver::Version;
use time::OffsetDateTime;

/// A.1 — carried by every persisted record.
///
/// `produced_at` is always store-issued (Law XII); no draft type in this
/// workspace carries a timestamp field, so an agent cannot supply one.
/// `produced_by` is a JobRecord reference (as text) or an office id
/// (e.g. "deacon", "sovereign", "dispatcher").
#[derive(Debug, Clone)]
pub struct Envelope {
    pub schema_name: String,
    pub schema_version: Version,
    pub produced_by: String,
    pub produced_at: OffsetDateTime,
}
