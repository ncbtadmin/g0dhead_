//! The Vectoring Slave (Dogma Book II §3): one labor — take a normalized
//! derivative, obtain its vector from the embedding endpoint, persist it,
//! flag. It does not link, judge, or weigh. Volume is its virtue.

use crate::roster::Roster;
use crate::MlError;
use godhead_schemas::{
    AgentType, Budgets, Certifies, FlagDraft, JobStatus, Law, NodeRecord, RefusalDraft,
    RefusalReason, Validator,
};
use godhead_store::{ArtifactDraft, Store, StoreError};
use semver::Version;
use std::path::Path;
use uuid::Uuid;

/// The stage a Slave flags per embedded node.
pub const STAGE_EMBED: &str = "slave:embed";
/// The schema of its output artifact.
pub const EMBED_RESULT_SCHEMA: &str = "slave.embed_result";

/// What one backlog pass did. Failures are contained per node — one bad
/// node never blocks the rest of the backlog — and each is either a
/// lawfully refused job (on the record) or a pre-job fault reported here.
#[derive(Debug, Default)]
pub struct BackfillSummary {
    pub embedded: usize,
    pub failures: Vec<(Uuid, String)>,
}

fn slave_draft(node_id: Uuid, endpoint_alias: &str) -> godhead_schemas::JobDraft {
    godhead_schemas::JobDraft {
        agent_type: AgentType::Slave,
        auditor_name: None,
        tier: None,
        input_refs: vec![node_id],
        env_ref: None,
        brief_ref: None,
        // The embedder is floor machinery, but which one did the work is
        // provenance always (Law XIII.2).
        endpoint_alias: Some(endpoint_alias.to_string()),
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 120_000,
            max_tool_calls: 10,
            max_tokens: 1,
        },
    }
}

/// Maps a mid-labor store error to the law and reason its refusal cites.
fn refusal_of(err: &StoreError) -> (Law, RefusalReason) {
    match err {
        StoreError::LeaseConflict(_) => (Law::XI, RefusalReason::LeaseConflict),
        StoreError::SchemaMismatch(_) => (Law::II, RefusalReason::SchemaMismatch),
        _ => (Law::II, RefusalReason::ValidationFailed),
    }
}

/// The store-labor half of one embedding: everything after the job exists.
async fn persist_one<S: Store>(
    store: &S,
    job_id: Uuid,
    node_id: Uuid,
    alias: &str,
    vector: &[f32],
) -> Result<(), StoreError> {
    let lease = store.acquire_lease(job_id, node_id, 60_000).await?;
    store.put_embedding(job_id, node_id, alias, vector).await?;
    let artifact = store
        .write_artifact(
            job_id,
            "result",
            &ArtifactDraft {
                schema_name: EMBED_RESULT_SCHEMA.to_string(),
                schema_version: Version::new(1, 0, 0),
                payload: serde_json::json!({
                    "node_id": node_id.to_string(),
                    "embedder_alias": alias,
                    "outcome": "EMBEDDED",
                }),
            },
        )
        .await?;
    let job = store.get_job(job_id).await?;
    store
        .transition_job(job_id, job.revision, JobStatus::Written)
        .await?;
    store
        .write_flag(
            job_id,
            &FlagDraft {
                stage: STAGE_EMBED.to_string(),
                certifies: Certifies {
                    output_slots: vec!["result".to_string()],
                    revisions: vec![artifact.revision],
                },
                validator: Validator {
                    id: "godhead-ml/registry".to_string(),
                    version: "1.0.0".to_string(),
                },
            },
        )
        .await?;
    store.release_lease(job_id, lease.lease_id).await?;
    let job = store.get_job(job_id).await?;
    store
        .transition_job(job_id, job.revision, JobStatus::Terminated)
        .await?;
    Ok(())
}

/// Embeds one node: read derivative, embed, then a full Book I lifecycle.
/// Any failure after the job exists ends in a Law VII refusal — the job is
/// never stranded live; refusal is compliance, and the node stays in the
/// backlog for the next pass.
async fn embed_one<S: Store>(
    store: &S,
    roster: &Roster,
    data_root: &Path,
    node: &NodeRecord,
) -> Result<bool, String> {
    let Some((alias, embedder)) = roster.embedder(None) else {
        // No embedder is not an error; the backlog simply remains (SC-M06).
        return Ok(false);
    };
    let Some(derivative_path) = &node.derivative_path else {
        return Ok(false); // not normalized; not this Slave's labor
    };
    // Pre-job work: no identity yet, so a fault here is the caller's to
    // report — nothing exists in the store to strand.
    let text = std::fs::read_to_string(data_root.join(derivative_path))
        .map_err(|e| format!("derivative unreadable: {e}"))?;
    let vector = embedder.embed(&text).await.map_err(|e| e.to_string())?;

    let spawn = async {
        let job = store.create_job(&slave_draft(node.node_id, alias)).await?;
        let job = store
            .transition_job(job.job_id, job.revision, JobStatus::Leased)
            .await?;
        store
            .transition_job(job.job_id, job.revision, JobStatus::Running)
            .await
    };
    let job = spawn.await.map_err(|e: StoreError| e.to_string())?;

    match persist_one(store, job.job_id, node.node_id, alias, &vector).await {
        Ok(()) => Ok(true),
        Err(err) => {
            // The store may already have refused the job itself (budget
            // exhaustion); a second refusal would be terminal access.
            // Best-effort: a refusal that cannot be written is still
            // reported to the caller.
            if !matches!(err, StoreError::BudgetExceeded(_)) {
                let (law, reason) = refusal_of(&err);
                let _ = store
                    .refuse(
                        job.job_id,
                        &RefusalDraft {
                            law,
                            reason,
                            subject_refs: vec![node.node_id.to_string()],
                            detail: format!("embedding labor could not complete: {err}"),
                            preserved_refs: vec![],
                        },
                    )
                    .await;
            }
            Err(err.to_string())
        }
    }
}

/// One pass over the backlog: every normalized, unembedded node in scope
/// gets one Slave. Existing embeddings are read, never recomputed — the
/// backlog query itself guarantees it (SC-M05). Failures are contained
/// per node: the pass continues, and the failed node remains in the
/// backlog for the next tick.
pub async fn backfill_tick<S: Store>(
    store: &S,
    roster: &Roster,
    data_root: &Path,
    scope: Option<&[Uuid]>,
) -> Result<BackfillSummary, MlError> {
    let backlog = store.embedding_backlog(scope).await?;
    let mut summary = BackfillSummary::default();
    if roster.embedder(None).is_none() {
        // Embedder down/absent: the file rests normalized and linkless,
        // flagged for backfill by its presence in this very backlog.
        return Ok(summary);
    }
    for node in &backlog {
        match embed_one(store, roster, data_root, node).await {
            Ok(true) => summary.embedded += 1,
            Ok(false) => {}
            Err(detail) => summary.failures.push((node.node_id, detail)),
        }
    }
    Ok(summary)
}
