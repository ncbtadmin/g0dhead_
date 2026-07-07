//! godhead-notary — the hands of consent (Dogma Book II §3).
//!
//! Where the sovereign consents, a Notary is summoned to make it so — and
//! then it dies. It validates the full reference chain before writing,
//! applies exactly what was consented to, and holds no judgment with which
//! to apply anything else. It uses no model; there is nothing to think
//! about. Slice 3 implements its first labor: granted-petition execution
//! (Law IV.5). Commitment, amendment, and decommission arrive with
//! section D.

use godhead_schemas::{
    AgentType, Budgets, Certifies, FlagDraft, JobStatus, Law, OverrideRecord, RefusalDraft,
    RefusalReason, SchemaRegistry, Validator,
};
use godhead_store::{ArtifactDraft, Store, StoreError};
use semver::{Version, VersionReq};
use thiserror::Error;
use uuid::Uuid;

/// The stage a grant-execution Notary flags.
pub const STAGE_GRANT: &str = "notary:grant";
/// The schema of its output artifact.
pub const GRANT_RESULT_SCHEMA: &str = "notary.grant_result";

#[derive(Debug, Error)]
pub enum NotaryError {
    #[error(transparent)]
    Store(#[from] StoreError),
    /// The labor refused per Law VII; the RefusalRecord is already written
    /// and the petition stands GRANTED-unexecuted (surfaced by SC-C06).
    #[error("REFUSED: {0}")]
    Refused(String),
}

/// Adds the Notary's declared schemas to a build registry (Law II.4).
pub fn register_into(registry: &mut SchemaRegistry) {
    registry.register(
        GRANT_RESULT_SCHEMA,
        VersionReq::parse("^1.0").expect("valid req"),
        |payload| {
            let obj = payload.as_object().ok_or("payload must be an object")?;
            for field in ["petition_id", "override_id", "outcome"] {
                let value = obj
                    .get(field)
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| format!("field '{field}' (string) is required"))?;
                if value.is_empty() {
                    return Err(format!("field '{field}' must be non-empty"));
                }
            }
            Ok(())
        },
    );
}

pub fn registry() -> SchemaRegistry {
    let mut reg = SchemaRegistry::new();
    register_into(&mut reg);
    reg
}

fn notary_draft(petition_id: Uuid) -> godhead_schemas::JobDraft {
    godhead_schemas::JobDraft {
        agent_type: AgentType::Notary,
        auditor_name: None,
        tier: None,
        input_refs: vec![petition_id],
        env_ref: None,
        brief_ref: None,
        // Notaries are modelless (A.2): there is nothing to think about.
        endpoint_alias: None,
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 120_000,
            max_tool_calls: 10,
            max_tokens: 1,
        },
    }
}

/// One summoning: spawn, validate the chain, apply exactly the granted
/// change, flag, die. On a chain that does not resolve or a subject that
/// no longer validates, the Notary refuses per Law VII and the petition
/// stands GRANTED-unexecuted.
pub async fn run_grant<S: Store>(
    store: &S,
    petition_id: Uuid,
) -> Result<OverrideRecord, NotaryError> {
    let petition = store.get_petition(petition_id).await?;
    let job = store.create_job(&notary_draft(petition_id)).await?;
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await?;
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await?;
    let lease = store
        .acquire_lease(job.job_id, petition.subject_ref, 60_000)
        .await?;

    let successor = match store.execute_grant(job.job_id, petition_id).await {
        Ok(successor) => successor,
        Err(defect) => {
            // Refuse, flag, preserve (Law VII): the refusal releases the
            // lease and the grant loop stays mechanically open (IV.5).
            store
                .refuse(
                    job.job_id,
                    &RefusalDraft {
                        law: Law::IV,
                        reason: RefusalReason::ValidationFailed,
                        subject_refs: vec![petition_id.to_string()],
                        detail: format!("grant execution chain did not validate: {defect}"),
                        preserved_refs: vec![petition.subject_ref.to_string()],
                    },
                )
                .await?;
            return Err(NotaryError::Refused(format!(
                "petition {petition_id}: {defect}"
            )));
        }
    };

    let artifact = store
        .write_artifact(
            job.job_id,
            "result",
            &ArtifactDraft {
                schema_name: GRANT_RESULT_SCHEMA.to_string(),
                schema_version: Version::new(1, 0, 0),
                payload: serde_json::json!({
                    "petition_id": petition_id.to_string(),
                    "override_id": successor.override_id.to_string(),
                    "outcome": "APPLIED",
                }),
            },
        )
        .await?;
    let job = store.get_job(job.job_id).await?;
    store
        .transition_job(job.job_id, job.revision, JobStatus::Written)
        .await?;
    store
        .write_flag(
            job.job_id,
            &FlagDraft {
                stage: STAGE_GRANT.to_string(),
                certifies: Certifies {
                    output_slots: vec!["result".to_string()],
                    revisions: vec![artifact.revision],
                },
                validator: Validator {
                    id: "godhead-notary/registry".to_string(),
                    version: "1.0.0".to_string(),
                },
            },
        )
        .await?;
    store.release_lease(job.job_id, lease.lease_id).await?;
    let job = store.get_job(job.job_id).await?;
    store
        .transition_job(job.job_id, job.revision, JobStatus::Terminated)
        .await?;
    Ok(successor)
}

/// The dispatcher rule for consent (doc 3 §3.2 applied to IV.5): every
/// GRANTED petition with no completed execution is an executable consent;
/// each summons one Notary. Scope limits to a petition set (test isolation
/// on a shared store); production runs unscoped.
pub async fn grants_tick<S: Store>(
    store: &S,
    scope: Option<&[Uuid]>,
) -> Result<Vec<Uuid>, NotaryError> {
    let pending = store.stalled_grants(0).await?;
    let mut executed = Vec::new();
    for petition in pending {
        if let Some(ids) = scope {
            if !ids.contains(&petition.petition_id) {
                continue;
            }
        }
        run_grant(store, petition.petition_id).await?;
        executed.push(petition.petition_id);
    }
    Ok(executed)
}
