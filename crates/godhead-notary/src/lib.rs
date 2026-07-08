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
    AgentType, Budgets, Certifies, FlagDraft, JobStatus, Law, MatrixRecord, OverrideRecord,
    RefusalDraft, RefusalReason, SchemaRegistry, Validator,
};
use godhead_store::{ArtifactDraft, Store, StoreError};
use semver::{Version, VersionReq};
use thiserror::Error;
use uuid::Uuid;

/// The stage a grant-execution Notary flags.
pub const STAGE_GRANT: &str = "notary:grant";
/// The schema of its output artifact.
pub const GRANT_RESULT_SCHEMA: &str = "notary.grant_result";
/// The stage a commitment/amendment/decommission Notary flags.
pub const STAGE_MATRIX: &str = "notary:matrix";
/// The schema of its output artifact.
pub const MATRIX_RESULT_SCHEMA: &str = "notary.matrix_result";

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
    registry.register(
        MATRIX_RESULT_SCHEMA,
        VersionReq::parse("^1.0").expect("valid req"),
        |payload| {
            let obj = payload.as_object().ok_or("payload must be an object")?;
            for field in ["matrix_id", "act", "outcome"] {
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

/// The Law VII close for a Notary whose labor failed mid-flight: refuse
/// on the record (best-effort — budget exhaustion was already refused by
/// the store itself); the job never strands live.
async fn refuse_notary<S: Store>(store: &S, job_id: Uuid, subject: &str, err: &StoreError) {
    if matches!(err, StoreError::BudgetExceeded(_)) {
        return;
    }
    let (law, reason) = match err {
        StoreError::LeaseConflict(_) => (Law::XI, RefusalReason::LeaseConflict),
        _ => (Law::IV, RefusalReason::ValidationFailed),
    };
    let _ = store
        .refuse(
            job_id,
            &RefusalDraft {
                law,
                reason,
                subject_refs: vec![subject.to_string()],
                detail: format!("notary labor could not complete: {err}"),
                preserved_refs: vec![],
            },
        )
        .await;
}

/// One summoning: spawn, validate the chain, apply exactly the granted
/// change, flag, die — or refuse (Law VII); the job never strands live and
/// the petition stands GRANTED-unexecuted for the next summons.
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
    match grant_labor(store, job.job_id, petition_id, petition.subject_ref).await {
        Ok(successor) => Ok(successor),
        Err(err) => {
            refuse_notary(store, job.job_id, &petition_id.to_string(), &err).await;
            Err(NotaryError::Refused(format!(
                "petition {petition_id}: {err}"
            )))
        }
    }
}

async fn grant_labor<S: Store>(
    store: &S,
    job_id: Uuid,
    petition_id: Uuid,
    subject_ref: Uuid,
) -> Result<OverrideRecord, StoreError> {
    let lease = store.acquire_lease(job_id, subject_ref, 60_000).await?;
    let successor = store.execute_grant(job_id, petition_id).await?;
    let artifact = store
        .write_artifact(
            job_id,
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
    let job = store.get_job(job_id).await?;
    store
        .transition_job(job_id, job.revision, JobStatus::Written)
        .await?;
    store
        .write_flag(
            job_id,
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
    store.release_lease(job_id, lease.lease_id).await?;
    let job = store.get_job(job_id).await?;
    store
        .transition_job(job_id, job.revision, JobStatus::Terminated)
        .await?;
    Ok(successor)
}

/// The shared shape of a matrix-act Notary: spawn on the consent's
/// summons, lease the matrix, execute the chain-validated store act,
/// flag, die — any mid-labor failure ends in a Law VII refusal; the job
/// never strands live.
async fn run_matrix_act<S, F, Fut>(
    store: &S,
    matrix_id: Uuid,
    act: &str,
    input_ref: Uuid,
    execute: F,
) -> Result<MatrixRecord, NotaryError>
where
    S: Store,
    F: FnOnce(Uuid) -> Fut,
    Fut: std::future::Future<Output = Result<MatrixRecord, StoreError>>,
{
    let job = store.create_job(&notary_draft(input_ref)).await?;
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await?;
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await?;
    match matrix_act_labor(store, job.job_id, matrix_id, act, execute).await {
        Ok(matrix) => Ok(matrix),
        Err(err) => {
            refuse_notary(store, job.job_id, &input_ref.to_string(), &err).await;
            Err(NotaryError::Refused(format!("{act} {input_ref}: {err}")))
        }
    }
}

async fn matrix_act_labor<S, F, Fut>(
    store: &S,
    job_id: Uuid,
    matrix_id: Uuid,
    act: &str,
    execute: F,
) -> Result<MatrixRecord, StoreError>
where
    S: Store,
    F: FnOnce(Uuid) -> Fut,
    Fut: std::future::Future<Output = Result<MatrixRecord, StoreError>>,
{
    let lease = store.acquire_lease(job_id, matrix_id, 60_000).await?;
    let matrix = execute(job_id).await?;
    let artifact = store
        .write_artifact(
            job_id,
            "result",
            &ArtifactDraft {
                schema_name: MATRIX_RESULT_SCHEMA.to_string(),
                schema_version: Version::new(1, 0, 0),
                payload: serde_json::json!({
                    "matrix_id": matrix_id.to_string(),
                    "act": act,
                    "outcome": matrix.status.as_str(),
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
                stage: STAGE_MATRIX.to_string(),
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
    store.release_lease(job_id, lease.lease_id).await?;
    let job = store.get_job(job_id).await?;
    store
        .transition_job(job_id, job.revision, JobStatus::Terminated)
        .await?;
    Ok(matrix)
}

/// Executes a consented Joint Proposal (Law VI.3–VI.5): COMMIT professes
/// the Cardinal, AMEND yields Postulant revision N+1 with exactly the
/// enumerated changes, REJECT dissolves. Idempotent under retry.
pub async fn run_matrix_proposal<S: Store>(
    store: &S,
    proposal_id: Uuid,
) -> Result<MatrixRecord, NotaryError> {
    let proposal = store.get_proposal(proposal_id).await?;
    run_matrix_act(
        store,
        proposal.matrix_ref,
        "execute_proposal",
        proposal_id,
        |job_id| store.execute_matrix_proposal(job_id, proposal_id),
    )
    .await
}

/// Executes a consented decommission (Law VI.5): CARDINAL → DISSOLVED;
/// the dissolved matrix's links persist — bonds outlive the structure.
pub async fn run_decommission<S: Store>(
    store: &S,
    matrix_id: Uuid,
    consent_id: Uuid,
) -> Result<MatrixRecord, NotaryError> {
    run_matrix_act(store, matrix_id, "decommission", consent_id, |job_id| {
        store.execute_decommission(job_id, matrix_id, consent_id)
    })
    .await
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
