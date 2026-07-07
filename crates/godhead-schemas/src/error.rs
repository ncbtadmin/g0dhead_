use thiserror::Error;

/// Validation failures at the schema layer. The two codes mirror Law II.2:
/// malformed input is refused, never guessed at or repaired.
#[derive(Debug, Error)]
pub enum SchemaError {
    /// The value does not conform to its declared schema.
    #[error("VALIDATION_FAILED: {0}")]
    ValidationFailed(String),
    /// The declared schema name/version is unknown or outside the supported
    /// range (Law II.4 — there is no compatibility mode).
    #[error("SCHEMA_MISMATCH: {0}")]
    SchemaMismatch(String),
}
