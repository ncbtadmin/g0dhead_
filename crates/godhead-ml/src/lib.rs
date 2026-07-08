//! godhead-ml — the model layer beneath the agents (doc 4): the uniform
//! endpoint interface, the deterministic floor embedder, the Vectoring
//! Slave, link consolidation, and the dial-able weight system.
//!
//! The handleless-hammer principle is realized here as routing (doc 4
//! §2.4): "no model" is just an empty roster, handled the same way
//! everywhere — stages complete or degrade to their floor, they never
//! crash for want of a mind.

pub mod aggregate;
pub mod lexical;
pub mod rebalance;
pub mod roster;
pub mod slave;

use godhead_schemas::SchemaRegistry;
use semver::VersionReq;
use thiserror::Error;

pub use lexical::LexicalEmbedder;
pub use rebalance::{rebalance_now, rebalance_tick, RebalanceOutcome};
pub use roster::{Embedder, EndpointError, Reasoner, Roster};

/// Adds the ML-layer schemas to a build registry (Law II.4).
pub fn register_into(reg: &mut SchemaRegistry) {
    fn require_str(
        obj: &serde_json::Map<String, serde_json::Value>,
        f: &str,
    ) -> Result<(), String> {
        obj.get(f)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|_| ())
            .ok_or_else(|| format!("field '{f}' (non-empty string) is required"))
    }
    reg.register(
        slave::EMBED_RESULT_SCHEMA,
        VersionReq::parse("^1.0").expect("valid req"),
        |payload| {
            let obj = payload.as_object().ok_or("payload must be an object")?;
            require_str(obj, "node_id")?;
            require_str(obj, "embedder_alias")?;
            require_str(obj, "outcome")
        },
    );
    reg.register(
        aggregate::CONSOLIDATE_RESULT_SCHEMA,
        VersionReq::parse("^1.0").expect("valid req"),
        |payload| {
            let obj = payload.as_object().ok_or("payload must be an object")?;
            require_str(obj, "category")?;
            require_str(obj, "mode")?;
            for f in ["links_touched", "weights_set", "reasoner_calls"] {
                if !obj.get(f).is_some_and(serde_json::Value::is_u64) {
                    return Err(format!("field '{f}' (unsigned integer) is required"));
                }
            }
            Ok(())
        },
    );
    reg.register(
        aggregate::EMERGENCE_SCHEMA,
        VersionReq::parse("^1.0").expect("valid req"),
        |payload| {
            let obj = payload.as_object().ok_or("payload must be an object")?;
            require_str(obj, "matrix_id")?;
            require_str(obj, "category")?;
            if !obj.get("config_rev").is_some_and(serde_json::Value::is_i64) {
                return Err("field 'config_rev' (integer) is required".into());
            }
            Ok(())
        },
    );
}

/// The floor embedder's roster alias.
pub const LEXICAL_ALIAS: &str = "lexical-floor";
/// Embedding dimensionality, fixed by the migration's vector(256) column.
pub const EMBED_DIMS: usize = 256;

#[derive(Debug, Error)]
pub enum MlError {
    #[error(transparent)]
    Store(#[from] godhead_store::StoreError),
    #[error("endpoint error: {0}")]
    Endpoint(#[from] roster::EndpointError),
}
