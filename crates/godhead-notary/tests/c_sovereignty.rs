//! Section C — Human Sovereignty (Law IV). SC-C01 … SC-C07.

use godhead_intake::{Dispatcher, IntakePipe};
use godhead_schemas::{
    AgentType, Budgets, ConsentDecision, JobDraft, JobRecord, JobStatus, LogEvent, OverrideBasis,
    OverrideKind, PetitionDraft, PetitionStatus, SchemaRegistry, Severity,
};
use godhead_store::{PgStore, Store, StoreError};
use semver::Version;
use serde_json::json;
use std::path::PathBuf;
use uuid::Uuid;

fn database_url() -> Option<String> {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        return Some(url);
    }
    let env_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../.env");
    let text = std::fs::read_to_string(env_path).ok()?;
    text.lines().find_map(|line| {
        line.trim()
            .strip_prefix("DATABASE_URL=")
            .map(|rest| rest.trim().to_string())
    })
}

fn full_registry() -> SchemaRegistry {
    let mut reg = godhead_intake::registry();
    godhead_notary::register_into(&mut reg);
    reg
}

async fn store() -> Option<PgStore> {
    let Some(url) = database_url() else {
        eprintln!("SKIP: DATABASE_URL unset — database-backed criterion NOT exercised");
        return None;
    };
    Some(
        PgStore::connect(&url, full_registry())
            .await
            .expect("store connect + migrate"),
    )
}

fn temp_root() -> PathBuf {
    std::env::temp_dir().join(format!("godhead_test_{}", Uuid::now_v7()))
}

/// A node committed and at rest — the subject sovereignty acts on.
async fn resting_node(store: &PgStore) -> Uuid {
    let pipe = IntakePipe::new(store, temp_root()).expect("pipe");
    let node_id = pipe
        .commit_file("subject.md", b"a datum the sovereign cares about\n")
        .await
        .expect("commit");
    let dispatcher = Dispatcher::new(&pipe);
    let scope = [node_id];
    dispatcher.tick_scoped(Some(&scope)).await.expect("tick 1");
    dispatcher.tick_scoped(Some(&scope)).await.expect("tick 2");
    node_id
}

/// A live agent job to act (and petition) under.
async fn agent_job(store: &PgStore) -> JobRecord {
    let draft = JobDraft {
        agent_type: AgentType::Aggregator,
        auditor_name: None,
        tier: None,
        input_refs: vec![],
        env_ref: None,
        brief_ref: None,
        endpoint_alias: None,
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 120_000,
            max_tool_calls: 10,
            max_tokens: 100_000,
        },
    };
    let job = store.create_job(&draft).await.expect("create");
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await
        .expect("lease");
    store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await
        .expect("run")
}

fn scripture() -> serde_json::Value {
    json!([{ "category": "scripture", "weight": 0.5, "low_trust": false, "source": "sovereign_hand" }])
}

fn heresy_petition(node_id: Uuid) -> PetitionDraft {
    PetitionDraft {
        subject_ref: node_id,
        change_kind: OverrideKind::CategoryReassigned,
        reason: "the embedding geometry places this node among the programming corpus".to_string(),
        evidence_refs: vec!["vector-distance-report".to_string()],
        proposed_change: json!([{ "category": "programming", "weight": 0.5, "low_trust": false, "source": "granted_petition" }]),
    }
}

/// SC-C01 — a human-held datum is not agent-writable: mutation without a
/// resolving consent is rejected at the store, regardless of writer.
#[tokio::test]
async fn sc_c01_override_protection() {
    let Some(store) = store().await else { return };
    let node_id = resting_node(&store).await;
    let laid = store
        .lay_category_override("sovereign", node_id, &scripture())
        .await
        .expect("the sovereign's hand");
    assert_eq!(laid.basis, OverrideBasis::SovereignHand);
    assert!(laid.user_overridden);

    let agent = agent_job(&store).await;
    let node = store.get_node(node_id).await.expect("node");
    let err = store
        .set_node_classification(
            agent.job_id,
            node_id,
            node.revision,
            &json!([{ "category": "spam" }]),
        )
        .await
        .expect_err("the hand, once laid, is not lifted by ours");
    assert!(matches!(err, StoreError::OverrideConflict(_)), "got {err}");
    let node = store.get_node(node_id).await.expect("node");
    assert_eq!(node.classification[0]["category"], "scripture", "untouched");
}

/// SC-C02 — recurrence on the same (subject, kind) increments
/// occurrence_count and escalates OPEN → ESCALATED.
#[tokio::test]
async fn sc_c02_petition_escalation() {
    let Some(store) = store().await else { return };
    let node_id = resting_node(&store).await;
    store
        .lay_category_override("sovereign", node_id, &scripture())
        .await
        .expect("override");
    let agent = agent_job(&store).await;

    let p1 = store
        .open_petition(agent.job_id, &heresy_petition(node_id))
        .await
        .expect("first petition");
    assert_eq!(p1.status, PetitionStatus::Open);
    assert_eq!(p1.occurrence_count, 1);

    let p2 = store
        .open_petition(agent.job_id, &heresy_petition(node_id))
        .await
        .expect("second petition");
    assert_eq!(
        p2.petition_id, p1.petition_id,
        "one lineage per (subject, kind)"
    );
    assert_eq!(p2.status, PetitionStatus::Escalated);
    assert_eq!(p2.occurrence_count, 2);

    let p3 = store
        .open_petition(agent.job_id, &heresy_petition(node_id))
        .await
        .expect("third petition");
    assert_eq!(p3.status, PetitionStatus::Escalated);
    assert_eq!(p3.occurrence_count, 3);
}

/// SC-C03 — SILENCED suppresses future matching petitions; suppressed
/// attempts are still logged severity: suppressed and never purged.
#[tokio::test]
async fn sc_c03_silenced_suppression() {
    let Some(store) = store().await else { return };
    let node_id = resting_node(&store).await;
    store
        .lay_category_override("sovereign", node_id, &scripture())
        .await
        .expect("override");
    let agent = agent_job(&store).await;
    let petition = store
        .open_petition(agent.job_id, &heresy_petition(node_id))
        .await
        .expect("petition");
    let silenced = store
        .resolve_petition("sovereign", petition.petition_id, ConsentDecision::Silenced)
        .await
        .expect("don't ask me again");
    assert_eq!(silenced.status, PetitionStatus::Silenced);

    // The next matching petition is auto-suppressed, but counted and logged.
    let again = store
        .open_petition(agent.job_id, &heresy_petition(node_id))
        .await
        .expect("suppressed, not an error");
    assert_eq!(again.status, PetitionStatus::Silenced, "still silenced");
    assert_eq!(again.occurrence_count, 2, "still counted");
    let logs = store.read_logs(&node_id.to_string()).await.expect("logs");
    assert!(
        logs.iter()
            .any(|l| { l.event == LogEvent::PetitionOpened && l.severity == Severity::Suppressed }),
        "the suppressed attempt is on the record"
    );
    // Never purged: the lineage remains queryable.
    let kept = store
        .get_petition(petition.petition_id)
        .await
        .expect("retained");
    assert_eq!(kept.status, PetitionStatus::Silenced);
    // And the substrate refuses deletion outright.
    let purge = sqlx::query("DELETE FROM petition_records WHERE petition_id = $1")
        .bind(petition.petition_id)
        .execute(store.raw_pool())
        .await;
    assert!(purge.is_err(), "no cleanup purges petitions (IV.3)");
}

/// SC-C04 — executing a grant lays a successor override: user_overridden
/// true, basis GRANTED_PETITION, prior_ref and consent_ref resolving.
#[tokio::test]
async fn sc_c04_grant_lays_successor() {
    let Some(store) = store().await else { return };
    let node_id = resting_node(&store).await;
    let first = store
        .lay_category_override("sovereign", node_id, &scripture())
        .await
        .expect("override");
    let agent = agent_job(&store).await;
    let petition = store
        .open_petition(agent.job_id, &heresy_petition(node_id))
        .await
        .expect("petition");
    let granted = store
        .resolve_petition("sovereign", petition.petition_id, ConsentDecision::Granted)
        .await
        .expect("grant");
    let consent_id = granted.consent_ref.expect("consent minted");

    let successor = godhead_notary::run_grant(&store, petition.petition_id)
        .await
        .expect("the Notary makes it so");
    assert_eq!(successor.basis, OverrideBasis::GrantedPetition);
    assert!(successor.user_overridden, "the datum stays human-held");
    assert_eq!(
        successor.prior_ref,
        Some(first.override_id),
        "chained to the prior hand"
    );
    assert_eq!(
        successor.consent_ref,
        Some(consent_id),
        "chained to the consent"
    );

    // Exactly the granted change was applied.
    let node = store.get_node(node_id).await.expect("node");
    assert_eq!(node.classification[0]["category"], "programming");

    // The loop closed mechanically.
    let closed = store
        .get_petition(petition.petition_id)
        .await
        .expect("petition");
    assert!(
        closed.execution_job_ref.is_some(),
        "execution on the record"
    );

    // Retry converges: a second Notary finds the grant executed and
    // returns the same successor, applying nothing twice.
    let node_rev_before = node.revision;
    let retried = godhead_notary::run_grant(&store, petition.petition_id)
        .await
        .expect("idempotent retry");
    assert_eq!(retried.override_id, successor.override_id);
    let node = store.get_node(node_id).await.expect("node");
    assert_eq!(node.revision, node_rev_before, "no double-apply");

    // A bogus petition never executes.
    let err = godhead_notary::run_grant(&store, Uuid::now_v7()).await;
    assert!(err.is_err(), "no chain, no act");
}

/// SC-C05 — post-grant, ordinary agent mutation is still rejected:
/// protection persisted through the grant.
#[tokio::test]
async fn sc_c05_post_grant_protection() {
    let Some(store) = store().await else { return };
    let node_id = resting_node(&store).await;
    store
        .lay_category_override("sovereign", node_id, &scripture())
        .await
        .expect("override");
    let agent = agent_job(&store).await;
    let petition = store
        .open_petition(agent.job_id, &heresy_petition(node_id))
        .await
        .expect("petition");
    store
        .resolve_petition("sovereign", petition.petition_id, ConsentDecision::Granted)
        .await
        .expect("grant");
    godhead_notary::run_grant(&store, petition.petition_id)
        .await
        .expect("execute");

    // The next rebalance cannot undo what was just granted (IV.5).
    let node = store.get_node(node_id).await.expect("node");
    let err = store
        .set_node_classification(
            agent.job_id,
            node_id,
            node.revision,
            &json!([{ "category": "spam" }]),
        )
        .await
        .expect_err("what the sovereign has touched stays human-held");
    assert!(matches!(err, StoreError::OverrideConflict(_)), "got {err}");
}

/// SC-C06 — a GRANTED petition with no completed execution inside the
/// stall window is surfaced; nothing the sovereign grants may quietly
/// fail to happen.
#[tokio::test]
async fn sc_c06_stall_surfaced() {
    let Some(store) = store().await else { return };
    let node_id = resting_node(&store).await;
    store
        .lay_category_override("sovereign", node_id, &scripture())
        .await
        .expect("override");
    let agent = agent_job(&store).await;
    let petition = store
        .open_petition(agent.job_id, &heresy_petition(node_id))
        .await
        .expect("petition");
    store
        .resolve_petition("sovereign", petition.petition_id, ConsentDecision::Granted)
        .await
        .expect("grant");

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let stalled = store.stalled_grants(100).await.expect("stall query");
    assert!(
        stalled
            .iter()
            .any(|p| p.petition_id == petition.petition_id),
        "the unexecuted grant is surfaced"
    );

    // The dispatcher rule closes the loop; the surface clears.
    let executed = godhead_notary::grants_tick(&store, Some(&[petition.petition_id]))
        .await
        .expect("tick");
    assert_eq!(executed, vec![petition.petition_id]);
    let stalled = store.stalled_grants(100).await.expect("stall query");
    assert!(
        !stalled
            .iter()
            .any(|p| p.petition_id == petition.petition_id),
        "executed grants no longer surface"
    );
}

/// SC-C07 — the human-reserved actions with live surfaces have no
/// agent-callable path: an agent-authored write to a reserved table is
/// rejected at the substrate with GATE_BYPASS_ATTEMPT. (Sovereign-act
/// store methods additionally take no job identity — uncallable by
/// signature. Entries whose surfaces don't exist yet are pinned to their
/// slices in SLICE_03 §3.)
#[tokio::test]
async fn sc_c07_gate_bypass() {
    let Some(store) = store().await else { return };
    let node_id = resting_node(&store).await;
    let agent = agent_job(&store).await;
    let agent_author = agent.job_id.to_string();

    // Laying an override under an agent identity.
    let attempt = sqlx::query(
        r#"INSERT INTO override_records
             (override_id, subject_ref, kind, basis, protected_state,
              schema_name, schema_version, produced_by)
           VALUES ($1, $2, 'CATEGORY_REASSIGNED', 'SOVEREIGN_HAND', '[]',
                   'OverrideRecord', '1.0.0', $3)"#,
    )
    .bind(Uuid::now_v7())
    .bind(node_id)
    .bind(&agent_author)
    .execute(store.raw_pool())
    .await;
    let message = format!("{}", attempt.expect_err("agent-laid override rejected"));
    assert!(message.contains("GATE_BYPASS_ATTEMPT"), "got: {message}");

    // Minting a consent under an agent identity.
    let attempt = sqlx::query(
        r#"INSERT INTO consent_records
             (consent_id, subject_ref, decision, decided_by,
              schema_name, schema_version, produced_by)
           VALUES ($1, $2, 'GRANTED', 'forged', 'ConsentRecord', '1.0.0', $3)"#,
    )
    .bind(Uuid::now_v7())
    .bind(Uuid::now_v7())
    .bind(&agent_author)
    .execute(store.raw_pool())
    .await;
    let message = format!("{}", attempt.expect_err("agent-minted consent rejected"));
    assert!(message.contains("GATE_BYPASS_ATTEMPT"), "got: {message}");

    // Tuning config under an agent identity — an order that tunes its own
    // law is no longer bound by it (IV.4).
    let attempt = sqlx::query(
        r#"INSERT INTO config_constants
             (key, tier, value, changed_by, schema_name, schema_version, produced_by)
           VALUES ($1, 'OPERATIONAL', '9999', 'forged', 'ConfigConstant', '1.0.0', $2)"#,
    )
    .bind(format!("forged_{}", Uuid::now_v7()))
    .bind(&agent_author)
    .execute(store.raw_pool())
    .await;
    let message = format!("{}", attempt.expect_err("agent-tuned config rejected"));
    assert!(message.contains("GATE_BYPASS_ATTEMPT"), "got: {message}");
}
