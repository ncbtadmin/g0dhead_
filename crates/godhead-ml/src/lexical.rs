//! The built-in floor embedder: hashed bag-of-words into 256 dimensions,
//! L2-normalized. Deterministic, offline, free — a second deterministic
//! floor far smarter than filetype-bucketing but spending no reasoning
//! tokens (doc 4 §3.1). A neural embedder (Ollama et al.) later replaces
//! it behind the same trait via configuration, not rewrite.

use crate::roster::{Embedder, EndpointError};
use crate::EMBED_DIMS;

pub struct LexicalEmbedder;

/// FNV-1a, dependency-free and stable across platforms — the determinism
/// of the floor must not hinge on a hasher implementation detail.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    hash
}

/// Lowercased alphanumeric tokens, unigrams + bigrams: bigrams give the
/// bag-of-words floor a whisper of word order.
fn tokens(text: &str) -> Vec<String> {
    let words: Vec<String> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect();
    let mut out = words.clone();
    out.extend(words.windows(2).map(|pair| pair.join(" ")));
    out
}

pub fn embed_text(text: &str) -> Vec<f32> {
    let mut vector = vec![0.0f32; EMBED_DIMS];
    for token in tokens(text) {
        let hash = fnv1a(token.as_bytes());
        let dim = usize::try_from(hash % EMBED_DIMS as u64).expect("dim fits usize");
        // A second hash bit decides sign, spreading tokens across the
        // sphere instead of piling onto the positive orthant.
        let sign = if (hash >> 63) == 0 { 1.0 } else { -1.0 };
        vector[dim] += sign;
    }
    let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut vector {
            *v /= norm;
        }
    }
    vector
}

#[async_trait::async_trait]
impl Embedder for LexicalEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EndpointError> {
        Ok(embed_text(text))
    }
}
