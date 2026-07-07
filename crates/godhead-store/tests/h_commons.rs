//! Section H — The Commons (Laws XI–XV). SC-H01 … SC-H06.

mod common;

use godhead_schemas::{AgentType, ConfigTier, JobStatus, LogEvent, Severity};
use godhead_store::{Store, StoreError};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

/// SC-H01 — a second acquisition on a leased subject refuses LEASE_CONFLICT
/// immediately: acquire-or-refuse, no waiting, no spinning.
#[tokio::test]
async fn second_lease_refused_immediately() {
    let Some(store) = common::store().await else {
        return;
    };
    let holder = common::running_job(&store).await;
    let contender = common::running_job(&store).await;
    let subject = uuid::Uuid::now_v7();

    store
        .acquire_lease(holder.job_id, subject, 60_000)
        .await
        .expect("first acquisition");
    let err = store
        .acquire_lease(contender.job_id, subject, 60_000)
        .await
        .expect_err("second acquisition");
    assert!(matches!(err, StoreError::LeaseConflict(_)), "got {err}");
}

/// SC-H02 (part 1) — an expired lease does not block: the subject routes to
/// recovery and a fresh job converges on it.
#[tokio::test]
async fn expired_lease_recovery() {
    let Some(store) = common::store().await else {
        return;
    };
    let dead = common::running_job(&store).await;
    let subject = uuid::Uuid::now_v7();
    store
        .acquire_lease(dead.job_id, subject, 1_200)
        .await
        .expect("doomed acquisition");
    tokio::time::sleep(Duration::from_millis(2_000)).await;

    // The dead job never heartbeated; a recovery job takes the subject.
    let recovery = common::running_job(&store).await;
    let lease = store
        .acquire_lease(recovery.job_id, subject, 60_000)
        .await
        .expect("expired lease must not block recovery (Law XI.2)");
    assert_eq!(lease.job_id, recovery.job_id);
}

/// Law XI.2 — heartbeat extends expiry by the original TTL.
#[tokio::test]
async fn lease_heartbeat_extends() {
    let Some(store) = common::store().await else {
        return;
    };
    let job = common::running_job(&store).await;
    let lease = store
        .acquire_lease(job.job_id, uuid::Uuid::now_v7(), 60_000)
        .await
        .expect("acquire");
    let beaten = store
        .heartbeat_lease(job.job_id, lease.lease_id)
        .await
        .expect("heartbeat");
    assert!(
        beaten.expires_at >= lease.expires_at,
        "expiry never moves backward"
    );
    assert!(beaten.heartbeat_at >= lease.heartbeat_at);
}

/// SC-H02 (part 2) — the CAS race harness: concurrent writers under
/// compare-and-swap; every stale revision loses and re-reads; no update is
/// lost and none overwrites another.
#[tokio::test]
async fn cas_race_harness() {
    let Some(store) = common::store().await else {
        return;
    };
    let store = Arc::new(store);
    let key = format!("race_{}", uuid::Uuid::now_v7());
    store
        .set_config(
            "test-harness",
            &key,
            ConfigTier::Operational,
            &json!(0),
            None,
        )
        .await
        .expect("seed key");

    let writers = 8;
    let mut handles = Vec::new();
    for i in 0..writers {
        let store = Arc::clone(&store);
        let key = key.clone();
        handles.push(tokio::spawn(async move {
            let mut retries = 0u32;
            loop {
                let current = store.get_config(&key).await.expect("read");
                match store
                    .set_config(
                        "test-harness",
                        &key,
                        ConfigTier::Operational,
                        &json!(i),
                        Some(current.revision),
                    )
                    .await
                {
                    Ok(_) => return retries,
                    Err(StoreError::StaleRevision { .. }) => retries += 1, // lose → re-read
                    Err(e) => panic!("only staleness may fail the race: {e}"),
                }
            }
        }));
    }
    for handle in handles {
        handle.await.expect("writer task");
    }
    let end = store.get_config(&key).await.expect("final read");
    // Seed = revision 1; each of the 8 writers landed exactly one CAS win.
    assert_eq!(
        end.revision,
        1 + writers,
        "no lost updates, no blind overwrites"
    );
}

/// SC-H03 — timestamps are store-issued; an agent-supplied timestamp is
/// rejected; ordering assertions use store sequence, not wall-clock.
#[tokio::test]
async fn agent_timestamp_rejected_and_store_sequence_orders() {
    let Some(store) = common::store().await else {
        return;
    };
    let job = common::running_job(&store).await;
    let mut smuggled = common::widget("x");
    smuggled.payload = json!({ "name": "x", "produced_at": "2001-01-01T00:00:00Z" });
    let err = store
        .write_artifact(job.job_id, "out", &smuggled)
        .await
        .expect_err("agent-supplied timestamp");
    assert!(matches!(err, StoreError::ValidationFailed(_)), "got {err}");

    // Order is established by store sequence (Law XII.2).
    let subject = format!("ordering_{}", uuid::Uuid::now_v7());
    for event in [
        LogEvent::AuditOpened,
        LogEvent::ReportFiled,
        LogEvent::ProposalFiled,
    ] {
        store
            .write_log(&subject, event, &json!({}), Severity::Info)
            .await
            .expect("log");
    }
    let logs = store.read_logs(&subject).await.expect("read");
    assert_eq!(logs.len(), 3);
    assert!(
        logs.windows(2).all(|w| w[0].seq < w[1].seq),
        "store sequence is the order"
    );
    assert_eq!(
        logs.iter().map(|l| l.event).collect::<Vec<_>>(),
        vec![
            LogEvent::AuditOpened,
            LogEvent::ReportFiled,
            LogEvent::ProposalFiled
        ]
    );
    // The rotation chain: each snapshot names its prior; the first has none.
    assert!(logs[0].prior_ref.is_none());
    assert_eq!(logs[1].prior_ref, Some(logs[0].log_id));
    assert_eq!(logs[2].prior_ref, Some(logs[1].log_id));
}

/// SC-H04 — an anonymous write (unknown job identity) is rejected at the store.
#[tokio::test]
async fn anonymous_write_rejected() {
    let Some(store) = common::store().await else {
        return;
    };
    let ghost = uuid::Uuid::now_v7();
    let err = store
        .write_artifact(ghost, "out", &common::widget("x"))
        .await
        .expect_err("anonymous write");
    assert!(matches!(err, StoreError::NotFound(_)), "got {err}");
    let err = store
        .acquire_lease(ghost, uuid::Uuid::now_v7(), 5_000)
        .await
        .expect_err("anonymous lease");
    assert!(matches!(err, StoreError::NotFound(_)));
}

/// SC-H05 — a job without (positive) budgets fails validation; exhaustion
/// mid-labor refuses BUDGET_EXCEEDED, releases leases, marks partials
/// non-authoritative, terminates the labor.
#[tokio::test]
async fn missing_budgets_rejected_and_exhaustion_refuses() {
    let Some(store) = common::store().await else {
        return;
    };
    // No budget is no job (Law XIV.1).
    let mut unbounded = common::job_draft(AgentType::Slave);
    unbounded.budgets.max_tokens = 0;
    let err = store
        .create_job(&unbounded)
        .await
        .expect_err("budget-less spawn");
    assert!(matches!(err, StoreError::ValidationFailed(_)), "got {err}");

    // Exhaustion mid-labor (Law XIV.2).
    let mut brief = common::job_draft(AgentType::Slave);
    brief.budgets.max_wall_ms = 6_000;
    let job = store.create_job(&brief).await.expect("create");
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await
        .expect("lease");
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await
        .expect("run");
    let subject = uuid::Uuid::now_v7();
    store
        .acquire_lease(job.job_id, subject, 60_000)
        .await
        .expect("lease within budget");
    store
        .write_artifact(job.job_id, "early", &common::widget("within-budget"))
        .await
        .expect("write within budget");

    tokio::time::sleep(Duration::from_millis(7_000)).await;

    let err = store
        .write_artifact(job.job_id, "late", &common::widget("over-budget"))
        .await
        .expect_err("the wall budget is spent");
    assert!(matches!(err, StoreError::BudgetExceeded(_)), "got {err}");

    let job_after = store.get_job(job.job_id).await.expect("re-read");
    assert_eq!(
        job_after.status,
        JobStatus::Refused,
        "graceful refusal, not a crash"
    );
    // Partials preserved non-authoritative.
    let err = store
        .read_artifact(job.job_id, "early")
        .await
        .expect_err("partial invisible");
    assert!(matches!(err, StoreError::NotFound(_)));
    // Leases released.
    let active: i64 =
        sqlx::query_scalar("SELECT count(*) FROM lease_records WHERE job_id = $1 AND active")
            .bind(job.job_id)
            .fetch_one(store.raw_pool())
            .await
            .expect("count leases");
    assert_eq!(active, 0, "leases released at refusal");
    // The refusal is on the record, with the closed reason code.
    let reason: String = sqlx::query_scalar(
        "SELECT reason FROM refusal_records WHERE job_id = $1 ORDER BY produced_at DESC LIMIT 1",
    )
    .bind(job.job_id)
    .fetch_one(store.raw_pool())
    .await
    .expect("refusal record");
    assert_eq!(reason, "BUDGET_EXCEEDED");
}

/// SC-H06 — secret-shaped strings are refused on every write path and the
/// violation is logged; no secret reaches store, log, or provenance.
#[tokio::test]
async fn secret_scan_blocks_write() {
    let Some(store) = common::store().await else {
        return;
    };
    let job = common::running_job(&store).await;

    // Artifact path: a connection string with an inline password.
    let mut leaky = common::widget("ok");
    leaky.payload = json!({ "name": "ok", "note": "postgres://svc:hunter2@db.internal:5432/prod" });
    let err = store
        .write_artifact(job.job_id, "out", &leaky)
        .await
        .expect_err("secret in artifact");
    assert!(matches!(err, StoreError::SecretDetected(_)), "got {err}");
    assert_eq!(
        common::artifact_count(&store, job.job_id).await,
        0,
        "nothing persisted"
    );
    let logs = store
        .read_logs(&job.job_id.to_string())
        .await
        .expect("logs");
    assert!(
        logs.iter()
            .any(|l| l.event == LogEvent::Violation && l.severity == Severity::Violation),
        "the secret hit is logged severity: violation"
    );

    // Config path: an AWS access key id.
    let err = store
        .set_config(
            "test-harness",
            &format!("leak_{}", uuid::Uuid::now_v7()),
            ConfigTier::Operational,
            &json!("AKIAIOSFODNN7EXAMPLE"),
            None,
        )
        .await
        .expect_err("secret in config value");
    assert!(matches!(err, StoreError::SecretDetected(_)));

    // Log path: a GitHub token.
    let err = store
        .write_log(
            "leak-subject",
            LogEvent::Violation,
            &json!({ "token": "ghp_0123456789abcdefghijABCDEFGHIJ" }),
            Severity::Info,
        )
        .await
        .expect_err("secret in log payload");
    assert!(matches!(err, StoreError::SecretDetected(_)));

    // And the store's own credential never appears in any record it keeps:
    // sweep every text/jsonb surface written by this suite for url-with-
    // password shapes (the store's own DATABASE_URL included).
    let hits: i64 = sqlx::query_scalar(
        r#"SELECT
             (SELECT count(*) FROM artifacts WHERE payload::text ~ '://[^/\s:@]+:[^/\s@]+@')
           + (SELECT count(*) FROM log_snapshots WHERE payload::text ~ '://[^/\s:@]+:[^/\s@]+@')
           + (SELECT count(*) FROM refusal_records WHERE detail ~ '://[^/\s:@]+:[^/\s@]+@')"#,
    )
    .fetch_one(store.raw_pool())
    .await
    .expect("sweep");
    assert_eq!(hits, 0, "no credential-shaped string persisted anywhere");
}
