use crate::error::SchemaError;
use semver::{Version, VersionReq};
use std::collections::HashMap;

/// A payload validator for one schema name. Returns a human-readable defect
/// description on failure; the caller wraps it in the Law II reason code.
pub type ValidatorFn = fn(&serde_json::Value) -> Result<(), String>;

/// What one build supports for one schema (Law II.4: every agent build
/// declares the schema version ranges it supports).
#[derive(Debug, Clone)]
pub struct SchemaSpec {
    pub supported: VersionReq,
    pub validate: ValidatorFn,
}

/// The registry a store (or agent build) is constructed with. Law II made
/// callable: every artifact read or written validates against its declared
/// `schema_name@schema_version`, out-of-range versions are refused before
/// any processing, and unknown names are never best-effort parsed.
#[derive(Debug, Clone, Default)]
pub struct SchemaRegistry {
    specs: HashMap<String, SchemaSpec>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, schema_name: &str, supported: VersionReq, validate: ValidatorFn) {
        self.specs.insert(
            schema_name.to_owned(),
            SchemaSpec {
                supported,
                validate,
            },
        );
    }

    /// The Law II gate, in check order: known name (II.1), in-range version
    /// (II.4 — checked before any payload processing), conforming payload
    /// (II.2 — malformed input is refused, never repaired).
    pub fn check(
        &self,
        schema_name: &str,
        schema_version: &Version,
        payload: &serde_json::Value,
    ) -> Result<(), SchemaError> {
        let spec = self.specs.get(schema_name).ok_or_else(|| {
            SchemaError::SchemaMismatch(format!("undeclared schema '{schema_name}'"))
        })?;
        if !spec.supported.matches(schema_version) {
            return Err(SchemaError::SchemaMismatch(format!(
                "schema '{schema_name}' version {schema_version} outside supported range {}",
                spec.supported
            )));
        }
        (spec.validate)(payload).map_err(|defect| {
            SchemaError::ValidationFailed(format!("schema '{schema_name}': {defect}"))
        })
    }
}
