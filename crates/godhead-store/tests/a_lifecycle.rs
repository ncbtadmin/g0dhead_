//! Section A — Lifecycle & Contract (Laws I–II). SC-A01 … SC-A07.

mod common;

use godhead_schemas::{AgentType, JobStatus, Law, LogEvent, RefusalDraft, RefusalReason, Severity};
use godhead_store::{Store, StoreError};
use semver::Version;

/// SC-A01 — the canonical chain advances forward step by step.
#[tokio::test]
async fn job_status_forward_only() {
    let Some(store) = common::store().await else {
        return;
    };
    let (job, _flag) = common::flagged_job(&store).await;
    // FLAGGED → TERMINATED is the one act remaining after FLAG.
    let job = store.get_job(job.job_id).await.expect("re-read");
    assert_eq!(job.status, JobStatus::Flagged);
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Terminated)
        .await
        .expect("to TERMINATED");
    assert_eq!(job.status, JobStatus::Terminated);
    assert!(job.finished_at.is_some(), "termination stamps finished_at");
}

/// SC-A01 — every non-forward transition is rejected by the store.
#[tokio::test]
async fn invalid_transitions_rejected() {
    let Some(store) = common::store().await else {
        return;
    };
    let job = store
        .create_job(&common::job_draft(AgentType::Slave))
        .await
        .expect("create");
    // Skipping forward, and the protocol-only targets, are all rejected.
    for target in [
        JobStatus::Running,
        JobStatus::Written,
        JobStatus::Terminated,
        JobStatus::Flagged,
        JobStatus::Refused,
    ] {
        let err = store
            .transition_job(job.job_id, job.revision, target)
            .await
            .expect_err("PENDING may only step to LEASED");
        assert!(
            matches!(err, StoreError::ValidationFailed(_)),
            "PENDING -> {target}: got {err}"
        );
    }
    // Backward from RUNNING is rejected.
    let job = common::running_job(&store).await;
    for target in [JobStatus::Pending, JobStatus::Leased] {
        let err = store
            .transition_job(job.job_id, job.revision, target)
            .await
            .expect_err("backward transition");
        assert!(matches!(err, StoreError::ValidationFailed(_)));
    }
}

/// SC-A01 — REFUSED is reachable from every live state, and only live states.
#[tokio::test]
async fn refused_reachable_from_any_live_state() {
    let Some(store) = common::store().await else {
        return;
    };
    let refusal = RefusalDraft {
        law: Law::VII,
        reason: RefusalReason::ValidationFailed,
        subject_refs: vec![],
        detail: "fixture refusal".to_string(),
        preserved_refs: vec![],
    };
    // PENDING
    let job = store
        .create_job(&common::job_draft(AgentType::Slave))
        .await
        .expect("create");
    store
        .refuse(job.job_id, &refusal)
        .await
        .expect("refuse from PENDING");
    // LEASED
    let job = store
        .create_job(&common::job_draft(AgentType::Slave))
        .await
        .expect("create");
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await
        .expect("to LEASED");
    store
        .refuse(job.job_id, &refusal)
        .await
        .expect("refuse from LEASED");
    // RUNNING
    let job = common::running_job(&store).await;
    store
        .refuse(job.job_id, &refusal)
        .await
        .expect("refuse from RUNNING");
    // WRITTEN
    let job = common::running_job(&store).await;
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Written)
        .await
        .expect("to WRITTEN");
    store
        .refuse(job.job_id, &refusal)
        .await
        .expect("refuse from WRITTEN");
    // FLAGGED is not live: the refusal window has closed (Law I.4).
    let (job, _flag) = common::flagged_job(&store).await;
    let err = store
        .refuse(job.job_id, &refusal)
        .await
        .expect_err("no refusal after FLAG");
    assert!(matches!(err, StoreError::TerminalAccess(_)));
}

/// SC-A02 — a retry finding FLAGGED performs zero writes and terminates.
#[tokio::test]
async fn retry_flagged_job_writes_nothing() {
    let Some(store) = common::store().await else {
        return;
    };
    let (job, _flag) = common::flagged_job(&store).await;
    let before = common::artifact_count(&store, job.job_id).await;

    // The retried invocation, replaying its labor under the same job_id:
    let err = store
        .write_artifact(job.job_id, "out", &common::widget("alpha"))
        .await
        .expect_err("no writes after FLAG");
    assert!(matches!(err, StoreError::TerminalAccess(_)));
    let err = store
        .write_flag(job.job_id, &common::flag_draft(vec!["out".into()], vec![1]))
        .await
        .expect_err("no re-flag after FLAG");
    assert!(matches!(err, StoreError::TerminalAccess(_)));

    assert_eq!(
        common::artifact_count(&store, job.job_id).await,
        before,
        "zero writes"
    );
    // ... and terminates.
    let job = store.get_job(job.job_id).await.expect("re-read");
    store
        .transition_job(job.job_id, job.revision, JobStatus::Terminated)
        .await
        .expect("retry terminates cleanly");
}

/// SC-A03 — a retry over partial outputs converges to the single-clean-run
/// state: writes are keyed (job_id, output_slot), never duplicated.
#[tokio::test]
async fn retry_partial_job_converges() {
    let Some(store) = common::store().await else {
        return;
    };
    let job = common::running_job(&store).await;
    // First attempt: writes slot "a", then dies before "b".
    store
        .write_artifact(job.job_id, "a", &common::widget("first-attempt"))
        .await
        .expect("write a");
    // Retry replays the whole labor under the same job_id.
    store
        .write_artifact(job.job_id, "a", &common::widget("retry"))
        .await
        .expect("rewrite own key");
    store
        .write_artifact(job.job_id, "b", &common::widget("retry"))
        .await
        .expect("write b");

    assert_eq!(
        common::artifact_count(&store, job.job_id).await,
        2,
        "no duplicates"
    );
    let a = store.read_artifact(job.job_id, "a").await.expect("read a");
    assert_eq!(a.payload["name"], "retry", "converged on the retry's write");
    assert_eq!(a.revision, 2, "own-key overwrite, revision advanced");
}

/// SC-A04 — a schema-invalid write is rejected atomically.
#[tokio::test]
async fn invalid_write_rejected_no_partial() {
    let Some(store) = common::store().await else {
        return;
    };
    let job = common::running_job(&store).await;
    let mut bad = common::widget("x");
    bad.payload = serde_json::json!({ "wrong_field": 1 });
    let err = store
        .write_artifact(job.job_id, "out", &bad)
        .await
        .expect_err("invalid payload");
    assert!(matches!(err, StoreError::ValidationFailed(_)), "got {err}");
    assert_eq!(
        common::artifact_count(&store, job.job_id).await,
        0,
        "no partial persists"
    );
}

/// SC-A05 — out-of-range schema_version refuses before any processing;
/// there is no best-effort parse path (Law II.4).
#[tokio::test]
async fn out_of_range_schema_version_refused() {
    let Some(store) = common::store().await else {
        return;
    };
    let job = common::running_job(&store).await;
    let mut draft = common::widget("x");
    draft.schema_version = Version::new(2, 0, 0);
    let err = store
        .write_artifact(job.job_id, "out", &draft)
        .await
        .expect_err("version 2.0.0 outside ^1.0");
    assert!(matches!(err, StoreError::SchemaMismatch(_)), "got {err}");

    let mut unknown = common::widget("x");
    unknown.schema_name = "test.unregistered".to_string();
    let err = store
        .write_artifact(job.job_id, "out", &unknown)
        .await
        .expect_err("undeclared schema");
    assert!(matches!(err, StoreError::SchemaMismatch(_)), "got {err}");
    assert_eq!(common::artifact_count(&store, job.job_id).await, 0);
}

/// SC-A06 — the store rejects records missing any envelope field, enforced
/// at the substrate (NOT NULL), below even the store's own API.
#[tokio::test]
async fn envelope_completeness_enforced() {
    let Some(store) = common::store().await else {
        return;
    };
    let job = common::running_job(&store).await;
    // Missing schema_name.
    let err = sqlx::query(
        "INSERT INTO artifacts (job_id, output_slot, payload, schema_version, produced_by)
         VALUES ($1, 'raw', '{}', '1.0.0', 'nobody')",
    )
    .bind(job.job_id)
    .execute(store.raw_pool())
    .await;
    assert!(err.is_err(), "missing schema_name must be rejected");
    // Missing produced_by.
    let err = sqlx::query(
        "INSERT INTO log_snapshots (log_id, subject_ref, event, severity, schema_name, schema_version)
         VALUES ($1, 'x', 'VIOLATION', 'info', 'LogSnapshot', '1.0.0')",
    )
    .bind(uuid::Uuid::now_v7())
    .execute(store.raw_pool())
    .await;
    assert!(err.is_err(), "missing produced_by must be rejected");
}

/// SC-A07 — store access after FLAG or REFUSED is rejected and logged
/// severity: violation.
#[tokio::test]
async fn post_terminal_access_rejected_and_logged() {
    let Some(store) = common::store().await else {
        return;
    };
    // After REFUSED.
    let job = common::running_job(&store).await;
    store
        .refuse(
            job.job_id,
            &RefusalDraft {
                law: Law::VII,
                reason: RefusalReason::ValidationFailed,
                subject_refs: vec![],
                detail: "fixture".to_string(),
                preserved_refs: vec![],
            },
        )
        .await
        .expect("refuse");
    let err = store
        .write_artifact(job.job_id, "late", &common::widget("x"))
        .await
        .expect_err("access after REFUSED");
    assert!(matches!(err, StoreError::TerminalAccess(_)));

    let logs = store
        .read_logs(&job.job_id.to_string())
        .await
        .expect("logs");
    assert!(
        logs.iter()
            .any(|l| l.event == LogEvent::Violation && l.severity == Severity::Violation),
        "the rejected access is logged severity: violation"
    );

    // After FLAGGED: leases, artifacts, and flags are all out of reach.
    let (job, _flag) = common::flagged_job(&store).await;
    let err = store
        .acquire_lease(job.job_id, uuid::Uuid::now_v7(), 5_000)
        .await
        .expect_err("no leases after FLAG");
    assert!(matches!(err, StoreError::TerminalAccess(_)));
}
