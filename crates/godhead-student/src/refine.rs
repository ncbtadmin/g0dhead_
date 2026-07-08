//! Deterministic refinement and its derivation record (Handbook §1.2b),
//! and the three mechanical properties of redundant consistency (§1.2):
//! (a) conformance, (b) re-derivability, (c) closure. An ephemeral steward
//! offers no memory as warranty — only a derivation anyone can re-run.

use crate::StudentError;
use godhead_schemas::RefinedArtifact;
use godhead_store::{Store, StoreError};
use uuid::Uuid;

/// The one refinement method of v1: SHA-256 over the method name and the
/// sorted source derivative checksums. Content is a stable function of
/// store state — no disk, no session, no memory.
pub const REFINE_METHOD: &str = "sha-fold@1.0";

/// SHA-256 as lowercase hex — the checksum idiom of record (intake's).
fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Folds the sources' derivative checksums under the named method. Order
/// of `source_refs` must not matter — the fold sorts — so the same
/// derivation re-run by a stranger lands on the same digest.
async fn fold_sources<S: Store>(store: &S, source_refs: &[Uuid]) -> Result<String, StudentError> {
    let mut shas = Vec::with_capacity(source_refs.len());
    for &id in source_refs {
        // A source that is not a node (a link, a room, another artifact —
        // things the store admits by design) is NOT_REFINABLE debris, not
        // an infrastructure fault: the walk must name it, never crash on it.
        let node = match store.get_node(id).await {
            Ok(node) => node,
            Err(StoreError::NotFound(_)) => {
                return Err(StudentError::NotRefinable(format!(
                    "source {id} is not a re-derivable node; only nodes fold (§1.2b)"
                )))
            }
            Err(e) => return Err(e.into()),
        };
        let dsha = node.derivative_sha256.ok_or_else(|| {
            StudentError::NotRefinable(format!(
                "source {id} has no derivative checksum; refinement is a stable function of normalized sources (§1.2b)"
            ))
        })?;
        shas.push(dsha);
    }
    shas.sort();
    let mut material = String::from(REFINE_METHOD);
    for sha in &shas {
        material.push('\n');
        material.push_str(sha);
    }
    Ok(sha256_hex(material.as_bytes()))
}

/// A deterministic refinement over source nodes: computes the content_sha
/// under REFINE_METHOD and persists the artifact with its derivation. The
/// job is the Student's running labor (the store proves it is one).
pub async fn refine<S: Store>(
    store: &S,
    job_id: Uuid,
    env_ref: Uuid,
    source_refs: &[Uuid],
) -> Result<RefinedArtifact, StudentError> {
    let content_sha = fold_sources(store, source_refs).await?;
    Ok(store
        .persist_refined_artifact(job_id, env_ref, source_refs, REFINE_METHOD, &content_sha)
        .await?)
}

/// Re-runs a stored derivation and returns the recomputed digest — the
/// strongest honest re-derivability an ephemeral steward can offer.
pub async fn re_derive<S: Store>(store: &S, artifact_id: Uuid) -> Result<String, StudentError> {
    let artifact = store.get_refined_artifact(artifact_id).await?;
    if artifact.method != REFINE_METHOD {
        return Err(StudentError::NotRefinable(format!(
            "the stored derivation names method '{}', which this Student cannot re-run",
            artifact.method
        )));
    }
    fold_sources(store, &artifact.source_refs).await
}

/// What the consistency walk found and where — property (a) conformance,
/// (b) re-derivability, or (c) closure (Handbook §1.2). Debris is found,
/// named, and reported; it is never silently repaired.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsistencyDebris {
    pub property: char,
    pub subject: Uuid,
    pub detail: String,
}

impl std::fmt::Display for ConsistencyDebris {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "property ({}) on {}: {}",
            self.property, self.subject, self.detail
        )
    }
}

/// The redundant-consistency walk over one scriptorium (Handbook §1.2):
/// every refined artifact's shape validates (a), its derivation reproduces
/// its content_sha (b), and no ref inside the room dangles (c) — the
/// artifacts' source_refs and the room's elected items alike.
pub async fn redundant_consistency<S: Store>(
    store: &S,
    env_ref: Uuid,
) -> Result<Result<(), ConsistencyDebris>, StudentError> {
    // The walk certifies a Student's scriptorium. A room that does not
    // resolve, or is not a Student's, fails loudly — a walk aimed at
    // nothing must never hand back a clean bill of consistency.
    let env = store.get_environment(env_ref).await?;
    if env.kind != godhead_schemas::EnvKind::Student {
        return Err(StoreError::ValidationFailed(format!(
            "the consistency walk covers a Student's scriptorium; {env_ref} is a {} room (§1.2)",
            env.kind
        ))
        .into());
    }
    for artifact in store.refined_artifacts_in(env_ref).await? {
        // (a) Conformance: the record's shape validates.
        if let Err(detail) = conforms(&artifact, env_ref) {
            return Ok(Err(ConsistencyDebris {
                property: 'a',
                subject: artifact.artifact_id,
                detail,
            }));
        }
        // (c) Closure of the derivation: every source resolves. Proven
        // before (b) — a dangling ref is debris in its own right, not a
        // re-derivation crash.
        for &src in &artifact.source_refs {
            if !resolves(store, src).await? {
                return Ok(Err(ConsistencyDebris {
                    property: 'c',
                    subject: artifact.artifact_id,
                    detail: format!("source_ref {src} dangles (§1.2c)"),
                }));
            }
        }
        // (b) Re-derivability: the recorded derivation reproduces the
        // recorded digest.
        let recomputed = match re_derive(store, artifact.artifact_id).await {
            Ok(sha) => sha,
            Err(StudentError::NotRefinable(detail)) => {
                return Ok(Err(ConsistencyDebris {
                    property: 'b',
                    subject: artifact.artifact_id,
                    detail,
                }))
            }
            Err(e) => return Err(e),
        };
        if recomputed != artifact.content_sha {
            return Ok(Err(ConsistencyDebris {
                property: 'b',
                subject: artifact.artifact_id,
                detail: format!(
                    "re-derivation yields {recomputed}; the record claims {} (§1.2b)",
                    artifact.content_sha
                ),
            }));
        }
    }
    // (c) Closure of the room: every elected item resolves.
    for item in store.env_items(env_ref).await? {
        if !resolves(store, item.item_ref).await? {
            return Ok(Err(ConsistencyDebris {
                property: 'c',
                subject: item.item_ref,
                detail: "an elected item of the room dangles (§1.2c)".into(),
            }));
        }
    }
    Ok(Ok(()))
}

/// Property (a): the shape checks the store also enforces at persist,
/// proven again at the reading end (the double-validation covenant).
fn conforms(artifact: &RefinedArtifact, env_ref: Uuid) -> Result<(), String> {
    if artifact.env_ref != env_ref {
        return Err(format!(
            "the artifact belongs to {}, not this scriptorium",
            artifact.env_ref
        ));
    }
    if artifact.source_refs.is_empty() {
        return Err("the derivation names no sources (§1.2b)".into());
    }
    if artifact.method.trim().is_empty() {
        return Err("the derivation names no method (§1.2b)".into());
    }
    if artifact.content_sha.len() != 64
        || !artifact
            .content_sha
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err("content_sha is not 64 lowercase hex chars".into());
    }
    Ok(())
}

/// A ref resolves if it names any first-class store object: node, link,
/// matrix, environment, refined artifact, Return, or job (the mount's
/// item-resolution set accepts job artifacts — the walk must not brand
/// what the mount blesses as debris). "Not this kind" (NotFound) is
/// distinguished from a real store fault, which surfaces.
async fn resolves<S: Store>(store: &S, id: Uuid) -> Result<bool, StudentError> {
    // Probed one at a time so a hit short-circuits: an array literal
    // would await all seven round-trips per ref, and the walk pays this
    // per source of every artifact in the room.
    if exists(store.get_node(id).await)? {
        return Ok(true);
    }
    if exists(store.get_link(id).await)? {
        return Ok(true);
    }
    if exists(store.get_matrix(id).await)? {
        return Ok(true);
    }
    if exists(store.get_environment(id).await)? {
        return Ok(true);
    }
    if exists(store.get_refined_artifact(id).await)? {
        return Ok(true);
    }
    if exists(store.get_return(id).await)? {
        return Ok(true);
    }
    exists(store.get_job(id).await)
}

fn exists<T>(result: Result<T, StoreError>) -> Result<bool, StudentError> {
    match result {
        Ok(_) => Ok(true),
        Err(StoreError::NotFound(_)) => Ok(false),
        Err(e) => Err(e.into()),
    }
}
