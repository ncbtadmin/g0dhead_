use godhead_schemas::SchemaError;
use thiserror::Error;

/// Store-layer failures. Codes mirror the closed refusal reason enum (A.4)
/// where one applies; the remainder are substrate conditions.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("VALIDATION_FAILED: {0}")]
    ValidationFailed(String),
    #[error("SCHEMA_MISMATCH: {0}")]
    SchemaMismatch(String),
    #[error("FLAG_UNTRUSTED: {0}")]
    FlagUntrusted(String),
    #[error("LEASE_CONFLICT: {0}")]
    LeaseConflict(String),
    #[error("BUDGET_EXCEEDED: {0}")]
    BudgetExceeded(String),
    /// Law I.4 — store access after FLAG or REFUSED; logged severity: violation.
    #[error("TERMINAL_ACCESS: {0}")]
    TerminalAccess(String),
    /// Law XI.3 — compare-and-swap lost; the writer re-reads, never overwrites.
    #[error("STALE_REVISION: expected revision {expected}, subject {subject}")]
    StaleRevision { expected: i32, subject: String },
    /// Law IV.1 — the subject is human-held; mutation requires a resolving
    /// consent. The sovereign's hand, once laid, is not lifted by ours.
    #[error("OVERRIDE_CONFLICT: {0}")]
    OverrideConflict(String),
    /// Law XV.2 — a secret-shaped string in an outbound write.
    #[error("SECRET_DETECTED: {0}")]
    SecretDetected(String),
    #[error("NOT_FOUND: {0}")]
    NotFound(String),
    /// A path that exists only to be refused (e.g. deleting a flag).
    #[error("FORBIDDEN: {0}")]
    Forbidden(String),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

impl StoreError {
    /// Lifts a SchemaError into the store's vocabulary without flattening
    /// the two Law II codes into one.
    pub fn from_schema(err: SchemaError) -> Self {
        match err {
            SchemaError::ValidationFailed(d) => StoreError::ValidationFailed(d),
            SchemaError::SchemaMismatch(d) => StoreError::SchemaMismatch(d),
        }
    }
}
