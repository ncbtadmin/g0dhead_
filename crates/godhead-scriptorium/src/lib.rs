//! godhead-scriptorium — the environment floor's establishment and mount
//! orchestration (Laws IX–X). Thin over the store, which holds the
//! enforcement; this is what a Teacher (K) or Student (L) job will call to
//! build and mount its room. In-world: the Scriptorium.

use godhead_schemas::{AgentType, Budgets, EnvKind, EnvironmentRecord, JobRecord, JobStatus, Tier};
use godhead_store::{Store, StoreError};
use semver::Version;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ScriptoriumError {
    #[error(transparent)]
    Store(#[from] StoreError),
}

fn establisher_draft(
    agent_type: AgentType,
    tier: Tier,
    matrix_ref: Uuid,
) -> godhead_schemas::JobDraft {
    godhead_schemas::JobDraft {
        agent_type,
        auditor_name: None,
        tier: Some(tier),
        input_refs: vec![matrix_ref],
        env_ref: None,
        brief_ref: None,
        endpoint_alias: None,
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 300_000,
            max_tool_calls: 100,
            max_tokens: 1,
        },
    }
}

/// Spawns the establishing job (running) and establishes the environment —
/// the conferral is laid in the same act (X.1). Returns the running job
/// (still LIVE, for further curation) and the environment record.
pub async fn establish<S: Store>(
    store: &S,
    kind: EnvKind,
    tier: Tier,
    matrix_ref: Uuid,
) -> Result<(JobRecord, EnvironmentRecord), ScriptoriumError> {
    let agent_type = match kind {
        EnvKind::Teacher => AgentType::Teacher,
        EnvKind::Student => AgentType::Student,
    };
    let job = store
        .create_job(&establisher_draft(agent_type, tier, matrix_ref))
        .await?;
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await?;
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await?;
    let env = store
        .establish_environment(job.job_id, kind, matrix_ref, tier)
        .await?;
    Ok((job, env))
}

/// A fresh agent spun up into an environment mounts it before working
/// (Law IX.3). Spawns a mounting job and returns it with the validated
/// record. Law IX.3's own text says the AGENT refuses on a failed mount —
/// so a mount failure after RUNNING ends in a persisted `ENV_INVALID`
/// refusal (the labor-rule debt ruling G1 ordered paid; ENV_INVALID's
/// first construction site), and a failed refusal write propagates, never
/// swallowed (SC-E05).
pub async fn mount<S: Store>(
    store: &S,
    kind: EnvKind,
    tier: Tier,
    matrix_ref: Uuid,
    env_id: Uuid,
) -> Result<(JobRecord, EnvironmentRecord), ScriptoriumError> {
    let agent_type = match kind {
        EnvKind::Teacher => AgentType::Teacher,
        EnvKind::Student => AgentType::Student,
    };
    let job = store
        .create_job(&establisher_draft(agent_type, tier, matrix_ref))
        .await?;
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await?;
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await?;
    match store.mount_environment(job.job_id, env_id).await {
        Ok(env) => Ok((job, env)),
        Err(err) => {
            // BudgetExceeded is the one lawful skip: the store already
            // enacted that refusal itself (already-recorded — G5).
            if !matches!(err, StoreError::BudgetExceeded(_)) {
                store
                    .refuse(
                        job.job_id,
                        &godhead_schemas::RefusalDraft {
                            law: godhead_schemas::Law::IX,
                            reason: godhead_schemas::RefusalReason::EnvInvalid,
                            subject_refs: vec![env_id.to_string()],
                            detail: format!(
                                "the room {env_id} failed floor validation at mount; the \
                                 mounting agent refuses rather than work atop an invalid \
                                 room (Law IX.3)"
                            ),
                            preserved_refs: vec![],
                        },
                    )
                    .await?;
            }
            Err(err.into())
        }
    }
}
