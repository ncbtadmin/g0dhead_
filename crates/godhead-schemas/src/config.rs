use crate::envelope::Envelope;
use crate::macros::closed_enum;
use time::OffsetDateTime;

closed_enum! {
    /// A.14 — the two-tier config split. SOVEREIGN changes are sovereign acts
    /// (Law IV.4); OPERATIONAL changes are ordinary administration. Neither
    /// tier is agent-writable, ever.
    ConfigTier {
        Sovereign => "SOVEREIGN",
        Operational => "OPERATIONAL",
    }
}

/// A.14 — a revisioned config constant. Any job citing config MUST cite the
/// revision it read (Law VI.1).
#[derive(Debug, Clone)]
pub struct ConfigConstant {
    pub key: String,
    pub tier: ConfigTier,
    pub value: serde_json::Value,
    pub revision: i32,
    pub changed_at: OffsetDateTime,
    pub changed_by: String,
    pub envelope: Envelope,
}
