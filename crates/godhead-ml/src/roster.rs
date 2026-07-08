//! The uniform model-endpoint interface (doc 4 §2). The core never talks
//! to a *model*; it talks to an endpoint — local and remote are the same
//! kind of thing. The roster holds what this deployment has; an empty
//! roster is not an error, it is the floor.

use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EndpointError {
    #[error("endpoint unavailable: {0}")]
    Unavailable(String),
    #[error("endpoint returned malformed output: {0}")]
    Malformed(String),
}

/// An embedder locates: text in, position in meaning-space out (doc 4
/// §3.1). It does not reason; call it geometry, not judgment.
#[async_trait::async_trait]
pub trait Embedder: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EndpointError>;
}

/// A reasoner judges (doc 4 §3.2). Slice 4 uses it only for the weight
/// system's assisted mode: given link context, a relevance multiplier.
#[async_trait::async_trait]
pub trait Reasoner: Send + Sync {
    async fn weigh(&self, context: &str) -> Result<f32, EndpointError>;
}

/// The provider abstraction's roster of endpoints, alias-keyed; routing is
/// per-invocation (doc 4 §2.3). Graceful degradation is routing (doc 4
/// §2.4): the `Option` returns ARE the fallback path.
#[derive(Default, Clone)]
pub struct Roster {
    embedders: HashMap<String, Arc<dyn Embedder>>,
    reasoners: HashMap<String, Arc<dyn Reasoner>>,
}

impl Roster {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_embedder(&mut self, alias: &str, endpoint: Arc<dyn Embedder>) {
        self.embedders.insert(alias.to_string(), endpoint);
    }

    pub fn add_reasoner(&mut self, alias: &str, endpoint: Arc<dyn Reasoner>) {
        self.reasoners.insert(alias.to_string(), endpoint);
    }

    /// A specific embedder, or any if `alias` is None (single-embedder
    /// deployments are the norm). None ⇒ the caller takes its floor.
    pub fn embedder(&self, alias: Option<&str>) -> Option<(&str, &Arc<dyn Embedder>)> {
        match alias {
            Some(a) => self.embedders.get_key_value(a),
            None => self.embedders.iter().next(),
        }
        .map(|(k, v)| (k.as_str(), v))
    }

    pub fn reasoner(&self, alias: Option<&str>) -> Option<(&str, &Arc<dyn Reasoner>)> {
        match alias {
            Some(a) => self.reasoners.get_key_value(a),
            None => self.reasoners.iter().next(),
        }
        .map(|(k, v)| (k.as_str(), v))
    }

    pub fn is_empty(&self) -> bool {
        self.embedders.is_empty() && self.reasoners.is_empty()
    }
}
