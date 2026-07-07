use crate::envelope::Envelope;
use time::OffsetDateTime;
use uuid::Uuid;

/// A.13 — a lease over one mutable subject (Law XI). Acquire-or-refuse:
/// there is no waiting and no spinning. All timestamps are store-issued.
#[derive(Debug, Clone)]
pub struct LeaseRecord {
    pub lease_id: Uuid,
    pub subject_ref: Uuid,
    pub job_id: Uuid,
    /// Retained so heartbeats can re-extend expiry by the original TTL.
    pub ttl_ms: i64,
    pub active: bool,
    pub acquired_at: OffsetDateTime,
    pub heartbeat_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
    pub envelope: Envelope,
}
