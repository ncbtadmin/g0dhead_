//! The clause→code map (ruling G1; PROMPT_H NEW-1) — ONE place where a
//! halt-clause token becomes a persisted `(Law, RefusalReason)` pair, so the
//! Teacher's and the Student's halt handlers cannot drift apart. Same
//! disease, same cure as `SUPPORTED_CONCORDAT`.
//!
//! The Dogma names `SCHEMA_MISMATCH` for Concordat version skew three times
//! (II.4, SC-A05, SC-K03); every other VALIDATE_OUT halt is the labor's
//! contract failing — `VALIDATION_FAILED`. A store-stage halt after RUNNING
//! is the labor rule's territory: the labor could not complete, `(VII,
//! VALIDATION_FAILED)` — endpoint faults land here too, never borrowing
//! Law VIII's ladder codes (ruling G1: the ladder deliberately does not run
//! there, so its codes may not be borrowed).

use crate::refusal::{Law, RefusalReason};

/// The persisted code for a VALIDATE_OUT halt, derived from the halt's
/// stable clause token. Skew-shaped clauses — a citation outside the
/// supported range, or of a version never adopted (unretrievable, SC-K03) —
/// carry `SCHEMA_MISMATCH`; the rest carry `VALIDATION_FAILED`. Unknown
/// tokens default to the broad code: a new clause must OPT IN to the sharp
/// one, never receive it by accident.
pub fn halt_code(clause: &str) -> (Law, RefusalReason) {
    match clause {
        // The Student's end (returns.rs) names these string tokens; the
        // Teacher's end (lint.rs) names its unadopted-citation precondition
        // 'v' — distinct from the six lint clauses (a)–(f), whose failures
        // are contract failures, not skew.
        "concordat-skew" | "concordat-unadopted" | "v" => (Law::II, RefusalReason::SchemaMismatch),
        _ => (Law::II, RefusalReason::ValidationFailed),
    }
}

/// The persisted code for a store-stage halt after RUNNING (the labor
/// rule): the labor could not complete. One code for every stage — a wall's
/// own error already named what the wall rejected; the refusal record
/// carries the stage token, never the emission (Law XV).
pub fn stage_code() -> (Law, RefusalReason) {
    (Law::VII, RefusalReason::ValidationFailed)
}
