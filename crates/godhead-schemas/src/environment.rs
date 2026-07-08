use crate::envelope::Envelope;
use crate::macros::closed_enum;
use time::OffsetDateTime;
use uuid::Uuid;

closed_enum! {
    /// A.8 — the two kinds of environment; the floor binds both.
    EnvKind {
        Teacher => "TEACHER",
        Student => "STUDENT",
    }
}

closed_enum! {
    /// A.8 — LIVE is a workplace; ORPHANED is a read-only archive;
    /// DISSOLVED is gone. Forward-only.
    EnvStatus {
        Live => "LIVE",
        Orphaned => "ORPHANED",
        Dissolved => "DISSOLVED",
    }
}

closed_enum! {
    /// A.10 — the two pairing types (X.5). Regulars do not pair.
    PairingKind {
        DevoutAssignment => "DEVOUT_ASSIGNMENT",
        CanonicalInstruction => "CANONICAL_INSTRUCTION",
    }
}

/// A.8 — the EnvironmentRecord: a persistent, matrix-bound working profile.
/// `title`/`name` are the conferral laid at establishment (X.1),
/// immutable for the life of the record.
#[derive(Debug, Clone)]
pub struct EnvironmentRecord {
    pub env_id: Uuid,
    pub kind: EnvKind,
    pub matrix_ref: Uuid,
    /// DEVOUT or CANON — Regulars establish no environment (X.1).
    pub tier: crate::job::Tier,
    pub title: String,
    pub name: String,
    pub established_by: Uuid,
    pub established_at: OffsetDateTime,
    pub status: EnvStatus,
    pub revision: i32,
    pub envelope: Envelope,
}

/// A contents-index row (the floor's view over A.8's contents_index_ref).
/// `provenance` is a ProvenanceChain (C.2 shape); `flagged` marks a
/// certified handoff artifact — the only thing the Pairing Exception
/// (IX.5) grants across the bridge.
#[derive(Debug, Clone)]
pub struct EnvItem {
    pub env_id: Uuid,
    pub item_ref: Uuid,
    pub provenance: serde_json::Value,
    pub flagged: bool,
    pub envelope: Envelope,
}

/// A.10 — the PairingRecord.
#[derive(Debug, Clone)]
pub struct PairingRecord {
    pub pairing_id: Uuid,
    pub kind: PairingKind,
    pub teacher_env_ref: Uuid,
    pub student_env_ref: Uuid,
    pub matrix_ref: Uuid,
    pub formed_at: OffsetDateTime,
    pub envelope: Envelope,
}

/// X.4 — the deterministic roster index: reproducible from the record
/// alone. A stable FNV-1a over the env_id's bytes, mod the roster length.
/// The determinism of conferral must not hinge on a hasher's internals.
#[must_use]
pub fn roster_index(env_id: Uuid, roster_len: usize) -> usize {
    if roster_len == 0 {
        return 0;
    }
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in env_id.as_bytes() {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    usize::try_from(hash % roster_len as u64).expect("mod fits usize")
}

/// X.4 — a living-collision ordinal in Roman numerals: the first bearer is
/// unadorned, the second is "II", and so on. Bounded to what any real
/// order would ever produce.
#[must_use]
pub fn roman_ordinal(n: u32) -> String {
    if n <= 1 {
        return String::new();
    }
    let table = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut remaining = n;
    let mut out = String::new();
    for (value, sym) in table {
        while remaining >= value {
            out.push_str(sym);
            remaining -= value;
        }
    }
    out
}
