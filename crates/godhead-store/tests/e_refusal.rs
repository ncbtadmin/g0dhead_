//! Section E — Refusal, the keystone (Law VII). SC-E01a … SC-E04.

mod common;

use godhead_schemas::{JobStatus, Law, RefusalDraft, RefusalReason};
use godhead_store::{Store, StoreError};

fn fixture_refusal() -> RefusalDraft {
    RefusalDraft {
        law: Law::II,
        reason: RefusalReason::SchemaMismatch,
        subject_refs: vec!["subject-a".to_string(), "subject-b".to_string()],
        detail: "the input did not validate against its declared schema".to_string(),
        preserved_refs: vec!["preserved-a".to_string()],
    }
}

/// SC-E01a — every refusal produces a RefusalRecord naming the law, a closed
/// reason code, subject refs, and preserved refs.
#[tokio::test]
async fn refusal_record_complete() {
    let Some(store) = common::store().await else {
        return;
    };
    let job = common::running_job(&store).await;
    let record = store
        .refuse(job.job_id, &fixture_refusal())
        .await
        .expect("refuse");

    assert_eq!(record.job_id, job.job_id);
    assert_eq!(record.law, Law::II);
    assert_eq!(record.reason, RefusalReason::SchemaMismatch);
    assert_eq!(record.subject_refs, vec!["subject-a", "subject-b"]);
    assert_eq!(record.preserved_refs, vec!["preserved-a"]);
    assert!(!record.detail.is_empty());
    assert_eq!(record.envelope.schema_name, "RefusalRecord");
    // Attribution (Law XIII): the refusing job signed its refusal.
    assert_eq!(record.envelope.produced_by, job.job_id.to_string());
}

/// SC-E02 — a refusing agent mutates nothing beyond quarantine marks:
/// pre/post state is identical except authoritative/quarantine flags, and
/// unrelated jobs' state is untouched.
#[tokio::test]
async fn refusal_preserves_state() {
    let Some(store) = common::store().await else {
        return;
    };
    let victim = common::running_job(&store).await;
    let bystander = common::running_job(&store).await;
    store
        .write_artifact(victim.job_id, "work", &common::widget("victim-output"))
        .await
        .expect("victim writes");
    store
        .write_artifact(
            bystander.job_id,
            "work",
            &common::widget("bystander-output"),
        )
        .await
        .expect("bystander writes");

    store
        .refuse(victim.job_id, &fixture_refusal())
        .await
        .expect("refuse");

    // The offending state is preserved — marked, never deleted or altered.
    let row: (serde_json::Value, bool, bool) = sqlx::query_as(
        "SELECT payload, authoritative, quarantine_marked FROM artifacts
         WHERE job_id = $1 AND output_slot = 'work'",
    )
    .bind(victim.job_id)
    .fetch_one(store.raw_pool())
    .await
    .expect("preserved row");
    assert_eq!(row.0["name"], "victim-output", "payload untouched");
    assert!(!row.1, "non-authoritative");
    assert!(row.2, "quarantine-marked");

    // The bystander's state is exactly as it was.
    let bystander_artifact = store
        .read_artifact(bystander.job_id, "work")
        .await
        .expect("bystander unaffected");
    assert!(bystander_artifact.authoritative);
    assert_eq!(bystander_artifact.payload["name"], "bystander-output");
}

/// SC-E03 — REFUSED is distinct from failure; the reference metrics query
/// scores refusals as compliance.
#[tokio::test]
async fn refused_scored_as_compliance() {
    let Some(store) = common::store().await else {
        return;
    };
    // One full lawful run to TERMINATED.
    let (terminated, _flag) = common::flagged_job(&store).await;
    let terminated = store.get_job(terminated.job_id).await.expect("re-read");
    store
        .transition_job(
            terminated.job_id,
            terminated.revision,
            JobStatus::Terminated,
        )
        .await
        .expect("terminate");
    // One refusal.
    let refused = common::running_job(&store).await;
    store
        .refuse(refused.job_id, &fixture_refusal())
        .await
        .expect("refuse");
    // One still in flight.
    let in_flight = common::running_job(&store).await;

    let metrics = store
        .compliance_metrics(&[terminated.job_id, refused.job_id, in_flight.job_id])
        .await
        .expect("metrics");
    assert_eq!(metrics.total, 3);
    assert_eq!(
        metrics.compliant, 2,
        "terminated + refused are both compliance"
    );
    assert_eq!(
        metrics.refused, 1,
        "refusal is distinct — and never an error"
    );
    assert_eq!(metrics.in_flight, 1);

    let job = store.get_job(refused.job_id).await.expect("re-read");
    assert_eq!(
        job.status,
        JobStatus::Refused,
        "REFUSED is its own status, not failure"
    );
}

/// SC-E04 — partial outputs of a refused job are non-authoritative and
/// invisible to downstream readers.
#[tokio::test]
async fn refused_partials_invisible() {
    let Some(store) = common::store().await else {
        return;
    };
    let job = common::running_job(&store).await;
    store
        .write_artifact(job.job_id, "partial", &common::widget("half-done"))
        .await
        .expect("write partial");
    store
        .refuse(job.job_id, &fixture_refusal())
        .await
        .expect("refuse");

    let err = store
        .read_artifact(job.job_id, "partial")
        .await
        .expect_err("refused partials are invisible");
    assert!(matches!(err, StoreError::NotFound(_)), "got {err}");
}
