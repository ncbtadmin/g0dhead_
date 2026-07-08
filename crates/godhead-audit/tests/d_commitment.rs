//! Section D — The Commitment Chain (Law VI). SC-D01 … SC-D10.

use godhead_audit::{invoke_audit, reconcile, run_auditor};
use godhead_intake::{Dispatcher, IntakePipe};
use godhead_ml::{aggregate, slave, LexicalEmbedder, Roster};
use godhead_schemas::{
    AgentType, AuditorKind, Budgets, ConfigTier, ConsentDecision, JobDraft, JobRecord, JobStatus,
    LogEvent, MatrixRecord, MatrixStatus, SchemaRegistry, Verdict,
};
use godhead_store::{PgStore, Store, StoreError};
use semver::Version;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
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
    godhead_ml::register_into(&mut reg);
    godhead_notary::register_into(&mut reg);
    godhead_audit::register_into(&mut reg);
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

fn lexical_roster() -> Roster {
    let mut roster = Roster::new();
    roster.add_embedder(godhead_ml::LEXICAL_ALIAS, Arc::new(LexicalEmbedder));
    roster
}

async fn commit_to_rest(pipe: &IntakePipe<'_, PgStore>, filename: &str, bytes: &[u8]) -> Uuid {
    let node_id = pipe.commit_file(filename, bytes).await.expect("commit");
    let dispatcher = Dispatcher::new(pipe);
    let scope = [node_id];
    dispatcher.tick_scoped(Some(&scope)).await.expect("tick 1");
    dispatcher.tick_scoped(Some(&scope)).await.expect("tick 2");
    node_id
}

/// Ensures the sovereign threshold stands at 0.01 (every d-test wants the
/// same value; concurrent identical writes are benign).
async fn ensure_threshold(store: &PgStore) {
    loop {
        match store.get_config("coherence_threshold").await {
            Ok(c) => {
                if c.value == json!(0.01) {
                    return;
                }
                match store
                    .set_config(
                        "sovereign",
                        "coherence_threshold",
                        ConfigTier::Sovereign,
                        &json!(0.01),
                        Some(c.revision),
                    )
                    .await
                {
                    Ok(_) => return,
                    Err(StoreError::StaleRevision { .. }) => {}
                    Err(e) => panic!("threshold write: {e}"),
                }
            }
            Err(_) => {
                if store
                    .set_config(
                        "sovereign",
                        "coherence_threshold",
                        ConfigTier::Sovereign,
                        &json!(0.01),
                        None,
                    )
                    .await
                    .is_ok()
                {
                    return;
                }
            }
        }
    }
}

async fn agent_job(store: &PgStore, agent_type: AgentType) -> JobRecord {
    let draft = JobDraft {
        agent_type,
        auditor_name: None,
        tier: None,
        input_refs: vec![],
        env_ref: None,
        brief_ref: None,
        endpoint_alias: None,
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 300_000,
            max_tool_calls: 100,
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

/// A Postulant grown the ordinary way: two identical files, embedded,
/// consolidated — one weighted link, emergence at density 0.5.
async fn grown_postulant(store: &PgStore) -> (MatrixRecord, String, [Uuid; 2]) {
    ensure_threshold(store).await;
    let pipe = IntakePipe::new(store, temp_root()).expect("pipe");
    let text = b"the joins are true and the cathedral stands\n";
    let a = commit_to_rest(&pipe, "creed_a.md", text).await;
    let b = commit_to_rest(&pipe, "creed_b.md", text).await;
    let scope = [a, b];
    let category = format!("trial_{}", Uuid::now_v7());
    slave::backfill_tick(store, &lexical_roster(), pipe.data_root(), Some(&scope))
        .await
        .expect("backfill");
    let summary = aggregate::consolidate(store, &lexical_roster(), &category, &scope)
        .await
        .expect("consolidate");
    let matrix_id = summary.emerged.expect("density 0.5 crosses 0.01");
    (
        store.get_matrix(matrix_id).await.expect("matrix"),
        category,
        scope,
    )
}

/// A Postulant carrying one weighted and one weightless bond — the fixture
/// whose trial must AMEND. Built by hand under an Aggregator job.
async fn flawed_postulant(store: &PgStore) -> (MatrixRecord, String) {
    ensure_threshold(store).await;
    let pipe = IntakePipe::new(store, temp_root()).expect("pipe");
    let text = b"the joins are true and the cathedral stands\n";
    let a = commit_to_rest(&pipe, "creed_a.md", text).await;
    let b = commit_to_rest(&pipe, "creed_b.md", text).await;
    let c = commit_to_rest(&pipe, "stray.md", b"unrelated marginalia\n").await;
    let scope = [a, b, c];
    slave::backfill_tick(store, &lexical_roster(), pipe.data_root(), Some(&scope))
        .await
        .expect("backfill");
    let category = format!("flawed_{}", Uuid::now_v7());
    let job = agent_job(store, AgentType::Aggregator).await;
    let strong = store
        .draw_link(job.job_id, a, b, 0.95, &category)
        .await
        .expect("strong bond");
    store
        .set_link_weight(job.job_id, strong.link_id, strong.revision, 0.9)
        .await
        .expect("weight the strong bond");
    // The flaw: a bond drawn but never weighed.
    store
        .draw_link(job.job_id, a, c, 0.4, &category)
        .await
        .expect("weightless bond");
    let matrix = store
        .emerge_postulant(job.job_id, &category, Some(&scope))
        .await
        .expect("evaluation")
        .expect("density 2/3 crosses 0.01");
    (matrix, category)
}

/// Walks one full trial cycle: audit → barrier → reconcile → consent →
/// Notary. Returns the applied matrix.
async fn run_cycle(store: &PgStore, matrix_id: Uuid, decision: ConsentDecision) -> MatrixRecord {
    invoke_audit(store, "sovereign", matrix_id)
        .await
        .expect("audit");
    store
        .certify_audit_barrier(matrix_id)
        .await
        .expect("barrier");
    let proposal = reconcile(store, matrix_id).await.expect("reconcile");
    store
        .resolve_proposal("sovereign", proposal.proposal_id, decision)
        .await
        .expect("sovereign answer");
    if decision == ConsentDecision::Granted {
        godhead_notary::run_matrix_proposal(store, proposal.proposal_id)
            .await
            .expect("the Notary makes it so");
    }
    store.get_matrix(matrix_id).await.expect("matrix")
}

/// SC-D01 — a density evaluation lacking its config citation fails; no
/// hardcoded threshold exists in agent code.
#[tokio::test]
async fn sc_d01_config_citation() {
    // Architectural half: in agent crates, every mention of the coherence
    // threshold is a config read — never a literal.
    for crate_dir in ["godhead-ml", "godhead-audit"] {
        let src = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
            .join(crate_dir)
            .join("src");
        for entry in std::fs::read_dir(&src).expect("read src") {
            let path = entry.expect("entry").path();
            let text = std::fs::read_to_string(&path).expect("read");
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.contains("coherence_threshold")
                    && !trimmed.starts_with("//")
                    && !trimmed.starts_with("//!")
                    && !trimmed.starts_with("///")
                {
                    assert!(
                        trimmed.contains("get_config"),
                        "{}: a threshold mention outside a config read: {trimmed}",
                        path.display()
                    );
                }
            }
        }
    }
    // Runtime half: the citation column is NOT NULL at the substrate.
    let Some(store) = store().await else { return };
    let job = agent_job(&store, AgentType::Aggregator).await;
    let uncited = sqlx::query(
        r#"INSERT INTO matrices
             (matrix_id, category, node_refs, link_refs, emerged_by,
              schema_name, schema_version, produced_by)
           VALUES ($1, $2, '[]', '[]', $3, 'MatrixRecord', '1.0.0', $3::text)"#,
    )
    .bind(Uuid::now_v7())
    .bind(format!("uncited_{}", Uuid::now_v7()))
    .bind(job.job_id)
    .execute(store.raw_pool())
    .await;
    assert!(
        uncited.is_err(),
        "an evaluation without citation fails validation"
    );
}

/// SC-D02 — crossing creates exactly one Postulant and one audit-eligibility
/// flag; below the threshold, neither; weights go live at the same event.
#[tokio::test]
async fn sc_d02_emergence() {
    let Some(store) = store().await else { return };
    ensure_threshold(&store).await;
    let pipe = IntakePipe::new(&store, temp_root()).expect("pipe");

    // Below: two dissimilar files draw no links; density 0 < 0.01.
    let a = commit_to_rest(&pipe, "one.md", b"entirely about beekeeping\n").await;
    let b = commit_to_rest(&pipe, "two.md", b"strictly regarding submarines\n").await;
    let below = [a, b];
    let below_category = format!("below_{}", Uuid::now_v7());
    slave::backfill_tick(&store, &lexical_roster(), pipe.data_root(), Some(&below))
        .await
        .expect("backfill");
    let summary = aggregate::consolidate(&store, &lexical_roster(), &below_category, &below)
        .await
        .expect("consolidate");
    assert_eq!(summary.emerged, None, "below the line: no Postulant");
    assert!(
        store
            .live_matrix_for_category(&below_category)
            .await
            .expect("query")
            .is_none(),
        "no matrix record"
    );
    let weights = store
        .live_weights(&below_category, Some(&below))
        .await
        .expect("evaluation");
    assert!(!weights.live, "weights inert below the same line");

    // Above: the grown fixture — one Postulant, one eligibility flag,
    // weights live at the same event.
    let (matrix, category, scope) = grown_postulant(&store).await;
    assert_eq!(matrix.status, MatrixStatus::Postulant);
    assert!(
        matrix.config_rev > 0,
        "the citation is on the record (VI.1)"
    );
    let eligibility = store
        .list_active_flags(godhead_ml::aggregate::STAGE_AUDIT_ELIGIBLE)
        .await
        .expect("flags");
    let for_this: Vec<_> = {
        let mut hits = Vec::new();
        for flag in &eligibility {
            if let Some(job_id) = flag.job_id {
                let job = store.get_job(job_id).await.expect("job");
                if job.input_refs.contains(&scope[0]) {
                    hits.push(flag.flag_id);
                }
            }
        }
        hits
    };
    assert_eq!(for_this.len(), 1, "exactly one audit-eligibility flag");
    let weights = store
        .live_weights(&category, Some(&scope))
        .await
        .expect("evaluation");
    assert!(weights.live, "weights live at the same event (VI.2)");

    // Exactly one: a second pass over the same ground emerges nothing new.
    let again = aggregate::consolidate(&store, &lexical_roster(), &category, &scope)
        .await
        .expect("second pass");
    assert_eq!(again.emerged, None, "emergence is idempotent");
}

/// SC-D03 — fiat-impossibility: no write path sets CARDINAL without a
/// resolving, cross-referencing chain — regardless of writer.
#[tokio::test]
async fn sc_d03_fiat_impossible() {
    let Some(store) = store().await else { return };
    let (matrix, _category, _scope) = grown_postulant(&store).await;

    // Bare fiat, below every API: rejected.
    let fiat = sqlx::query("UPDATE matrices SET status = 'CARDINAL' WHERE matrix_id = $1")
        .bind(matrix.matrix_id)
        .execute(store.raw_pool())
        .await;
    let message = format!("{}", fiat.expect_err("fiat must be impossible"));
    assert!(message.contains("fiat is impossible"), "got: {message}");

    // Fabricated refs that resolve to nothing: rejected.
    let forged = sqlx::query(
        r#"UPDATE matrices SET status = 'CARDINAL',
               committed_proposal_ref = $2, committed_consent_ref = $3
           WHERE matrix_id = $1"#,
    )
    .bind(matrix.matrix_id)
    .bind(Uuid::now_v7())
    .bind(Uuid::now_v7())
    .execute(store.raw_pool())
    .await;
    let message = format!("{}", forged.expect_err("a forged chain must not resolve"));
    assert!(message.contains("does not resolve"), "got: {message}");

    // Birth is always Postulant.
    let job = agent_job(&store, AgentType::Aggregator).await;
    let born_cardinal = sqlx::query(
        r#"INSERT INTO matrices
             (matrix_id, status, category, node_refs, link_refs, emerged_by, config_rev,
              schema_name, schema_version, produced_by)
           VALUES ($1, 'CARDINAL', $2, '[]', '[]', $3, 1, 'MatrixRecord', '1.0.0', $3::text)"#,
    )
    .bind(Uuid::now_v7())
    .bind(format!("forged_{}", Uuid::now_v7()))
    .bind(job.job_id)
    .execute(store.raw_pool())
    .await;
    assert!(born_cardinal.is_err(), "a matrix is born POSTULANT");

    let matrix = store.get_matrix(matrix.matrix_id).await.expect("matrix");
    assert_eq!(
        matrix.status,
        MatrixStatus::Postulant,
        "untouched by every attempt"
    );
}

/// SC-D04 — Gabriel and Lucy spawn with identical input; a pre-barrier
/// cross-read is rejected; the barrier opens the reports.
#[tokio::test]
async fn sc_d04_auditor_isolation() {
    let Some(store) = store().await else { return };
    let (matrix, _category, _scope) = grown_postulant(&store).await;
    let (gabriel, lucy) = invoke_audit(&store, "sovereign", matrix.matrix_id)
        .await
        .expect("audit");

    // Identical input: the matrix, nothing else.
    let gabriel_job = store.get_job(gabriel.job_id).await.expect("job");
    let lucy_job = store.get_job(lucy.job_id).await.expect("job");
    assert_eq!(gabriel_job.input_refs, vec![matrix.matrix_id]);
    assert_eq!(gabriel_job.input_refs, lucy_job.input_refs);

    // Pre-barrier: a third party's cross-read is rejected and logged.
    let reader = agent_job(&store, AgentType::Auditor).await;
    let err = store
        .read_audit_report(reader.job_id, gabriel.report_id)
        .await
        .expect_err("sealed until the barrier");
    assert!(matches!(err, StoreError::Forbidden(_)), "got {err}");

    // Post-barrier: the same read opens.
    store
        .certify_audit_barrier(matrix.matrix_id)
        .await
        .expect("barrier");
    let opened = store
        .read_audit_report(reader.job_id, gabriel.report_id)
        .await
        .expect("the barrier opens the record");
    assert_eq!(opened.auditor, AuditorKind::Gabriel);
}

/// SC-D05 — the AND-barrier: one report missing holds it; an invalid
/// report holds it; both present and valid release it.
#[tokio::test]
async fn sc_d05_and_barrier() {
    let Some(store) = store().await else { return };
    // One report only: held.
    let (matrix, _category, _scope) = grown_postulant(&store).await;
    run_auditor(&store, matrix.matrix_id, AuditorKind::Gabriel)
        .await
        .expect("gabriel alone");
    let err = store
        .certify_audit_barrier(matrix.matrix_id)
        .await
        .expect_err("one of two");
    assert!(format!("{err}").contains("1 of 2"), "got {err}");

    // Both filed: released.
    run_auditor(&store, matrix.matrix_id, AuditorKind::Lucy)
        .await
        .expect("lucy");
    store
        .certify_audit_barrier(matrix.matrix_id)
        .await
        .expect("both present and valid");

    // A corrupted report holds a fresh barrier (Law III.3 at the barrier).
    let (matrix2, _category2, _scope2) = grown_postulant(&store).await;
    let (gabriel2, _lucy2) = invoke_audit(&store, "sovereign", matrix2.matrix_id)
        .await
        .expect("audit");
    sqlx::query(
        r#"UPDATE audit_reports
           SET claims = jsonb_build_array(jsonb_build_object(
               'claim', 'forged', 'evidence_refs', jsonb_build_array($2::text), 'severity', null))
           WHERE report_id = $1"#,
    )
    .bind(gabriel2.report_id)
    .bind(Uuid::now_v7().to_string())
    .execute(store.raw_pool())
    .await
    .expect("out-of-band corruption");
    let err = store
        .certify_audit_barrier(matrix2.matrix_id)
        .await
        .expect_err("the state is the witness");
    assert!(matches!(err, StoreError::FlagUntrusted(_)), "got {err}");
}

/// SC-D06 — the truth-binding: a claim whose evidence does not resolve
/// fails VALIDATE_OUT; the report never exists and never flags.
#[tokio::test]
async fn sc_d06_truth_binding() {
    let Some(store) = store().await else { return };
    let (matrix, _category, _scope) = grown_postulant(&store).await;
    let liar = agent_job(&store, AgentType::Auditor).await;
    let err = store
        .file_audit_report(
            liar.job_id,
            &godhead_schemas::AuditReportDraft {
                matrix_ref: matrix.matrix_id,
                matrix_revision: matrix.revision,
                auditor: AuditorKind::Lucy,
                kind: godhead_schemas::ReportKind::Indictment,
                claims: vec![godhead_schemas::Claim {
                    claim: "this bond is corrupt".to_string(),
                    evidence_refs: vec![Uuid::now_v7()],
                    severity: Some("high".to_string()),
                }],
            },
        )
        .await
        .expect_err("an unsupported word does not validate");
    assert!(matches!(err, StoreError::ValidationFailed(_)), "got {err}");
    assert!(
        store
            .audit_reports_for(matrix.matrix_id, matrix.revision)
            .await
            .expect("query")
            .is_empty(),
        "the report never exists"
    );
    // And a claim with no evidence at all is equally dead.
    let err = store
        .file_audit_report(
            liar.job_id,
            &godhead_schemas::AuditReportDraft {
                matrix_ref: matrix.matrix_id,
                matrix_revision: matrix.revision,
                auditor: AuditorKind::Lucy,
                kind: godhead_schemas::ReportKind::Indictment,
                claims: vec![godhead_schemas::Claim {
                    claim: "trust me".to_string(),
                    evidence_refs: vec![],
                    severity: Some("high".to_string()),
                }],
            },
        )
        .await
        .expect_err("no evidence, no claim");
    assert!(matches!(err, StoreError::ValidationFailed(_)));
}

/// SC-D07 — AMEND applies exactly the enumerated changes: the structural
/// diff between revision N and N+1 is those changes and nothing else.
#[tokio::test]
async fn sc_d07_amend_exact() {
    let Some(store) = store().await else { return };
    let (matrix, _category) = flawed_postulant(&store).await;
    let before = store.get_matrix(matrix.matrix_id).await.expect("rev 1");
    assert_eq!(before.link_refs.len(), 2);

    let after = run_cycle(&store, matrix.matrix_id, ConsentDecision::Granted).await;
    assert_eq!(
        after.status,
        MatrixStatus::Postulant,
        "amended, not committed"
    );
    assert_eq!(after.revision, before.revision + 1);
    assert_eq!(after.audit_depth, before.audit_depth + 1);

    // The diff: exactly the weightless bond gone; nodes untouched.
    let removed: Vec<_> = before
        .link_refs
        .iter()
        .filter(|r| !after.link_refs.contains(r))
        .collect();
    assert_eq!(removed.len(), 1, "one link removed and no other");
    let proposal_changes = {
        let proposals: Vec<(serde_json::Value,)> = sqlx::query_as(
            "SELECT changes FROM joint_proposals WHERE matrix_ref = $1 AND matrix_revision = $2",
        )
        .bind(matrix.matrix_id)
        .bind(before.revision)
        .fetch_all(store.raw_pool())
        .await
        .expect("proposal");
        proposals[0].0.clone()
    };
    assert_eq!(
        proposal_changes[0]["subject_ref"],
        removed[0].to_string(),
        "the diff is the enumerated change"
    );
    assert_eq!(after.node_refs, before.node_refs, "and nothing else");
}

/// SC-D08 — recursion reaches fixpoint (a zero-amendment COMMIT); every
/// cycle logs its depth; the sovereign halt exits cleanly at any depth.
#[tokio::test]
async fn sc_d08_fixpoint_and_halt() {
    let Some(store) = store().await else { return };
    // Fixpoint: cycle 1 amends (depth 0 → 1), cycle 2 commits at depth 1.
    let (matrix, _category) = flawed_postulant(&store).await;
    let amended = run_cycle(&store, matrix.matrix_id, ConsentDecision::Granted).await;
    assert_eq!(amended.status, MatrixStatus::Postulant);
    assert_eq!(amended.audit_depth, 1);
    let committed = run_cycle(&store, matrix.matrix_id, ConsentDecision::Granted).await;
    assert_eq!(
        committed.status,
        MatrixStatus::Cardinal,
        "the fixpoint professes"
    );
    assert!(committed.committed_proposal_ref.is_some());
    assert!(committed.committed_consent_ref.is_some());

    // Depth is on the record for every cycle.
    let logs = store
        .read_logs(&matrix.matrix_id.to_string())
        .await
        .expect("logs");
    let depths: Vec<i64> = logs
        .iter()
        .filter(|l| l.event == LogEvent::AuditOpened)
        .map(|l| l.payload["audit_depth"].as_i64().expect("depth"))
        .collect();
    assert_eq!(depths, vec![0, 1], "every cycle logs its depth");

    // The halt: a fresh flawed Postulant, the sovereign declines the AMEND.
    let (halted, _category) = flawed_postulant(&store).await;
    let standing = run_cycle(&store, halted.matrix_id, ConsentDecision::Declined).await;
    assert_eq!(
        standing.status,
        MatrixStatus::Postulant,
        "the Postulant stands"
    );
    assert_eq!(standing.revision, halted.revision, "nothing applied");
    // The unconsented proposal is a dead letter to the Notary.
    let proposal_id: Uuid =
        sqlx::query_scalar("SELECT proposal_id FROM joint_proposals WHERE matrix_ref = $1")
            .bind(halted.matrix_id)
            .fetch_one(store.raw_pool())
            .await
            .expect("proposal");
    let err = godhead_notary::run_matrix_proposal(&store, proposal_id).await;
    assert!(err.is_err(), "a declined proposal executes nothing");
}

/// SC-D09 — decommission requires human consent and a Notary; the
/// dissolved matrix's links persist.
#[tokio::test]
async fn sc_d09_decommission() {
    let Some(store) = store().await else { return };
    let (matrix, category, scope) = grown_postulant(&store).await;
    let cardinal = run_cycle(&store, matrix.matrix_id, ConsentDecision::Granted).await;
    assert_eq!(cardinal.status, MatrixStatus::Cardinal);

    let links_before = store
        .links_by_category(&category, Some(&scope))
        .await
        .expect("links");
    assert!(!links_before.is_empty());

    let consent_id = store
        .consent_decommission("sovereign", matrix.matrix_id)
        .await
        .expect("human consent");
    let dissolved = godhead_notary::run_decommission(&store, matrix.matrix_id, consent_id)
        .await
        .expect("the Notary executes");
    assert_eq!(dissolved.status, MatrixStatus::Dissolved);

    // Bonds outlive the structure (VI.5).
    let links_after = store
        .links_by_category(&category, Some(&scope))
        .await
        .expect("links");
    assert_eq!(links_after.len(), links_before.len(), "links persist");

    // A dissolved matrix does not rise.
    let raise = sqlx::query("UPDATE matrices SET status = 'POSTULANT' WHERE matrix_id = $1")
        .bind(matrix.matrix_id)
        .execute(store.raw_pool())
        .await;
    assert!(raise.is_err(), "dissolution is forward-only");
}

/// SC-D10 — the Notary refuses unresolvable chains; a valid chain executes
/// exactly once, idempotent under retry, provenance linking every ref.
#[tokio::test]
async fn sc_d10_notary_chain() {
    let Some(store) = store().await else { return };
    let (matrix, _category, _scope) = grown_postulant(&store).await;
    invoke_audit(&store, "sovereign", matrix.matrix_id)
        .await
        .expect("audit");
    store
        .certify_audit_barrier(matrix.matrix_id)
        .await
        .expect("barrier");
    let proposal = reconcile(&store, matrix.matrix_id)
        .await
        .expect("reconcile");
    assert_eq!(proposal.verdict, Verdict::Commit, "a clean matrix commits");

    // Unconsented: the chain does not resolve; the Notary refuses.
    let err = godhead_notary::run_matrix_proposal(&store, proposal.proposal_id).await;
    assert!(err.is_err(), "no consent, no act");

    // Consented: executes, with provenance linking every reference.
    store
        .resolve_proposal("sovereign", proposal.proposal_id, ConsentDecision::Granted)
        .await
        .expect("grant");
    let first = godhead_notary::run_matrix_proposal(&store, proposal.proposal_id)
        .await
        .expect("executes");
    assert_eq!(first.status, MatrixStatus::Cardinal);
    let logs = store
        .read_logs(&matrix.matrix_id.to_string())
        .await
        .expect("logs");
    let committed_log = logs
        .iter()
        .find(|l| l.event == LogEvent::Committed)
        .expect("COMMITTED on the record");
    assert_eq!(
        committed_log.payload["proposal"],
        proposal.proposal_id.to_string()
    );
    assert!(committed_log.payload["consent"].is_string());
    assert!(committed_log.payload["executed_by_job"].is_string());

    // Retry converges: same Cardinal, no double-apply.
    let retry = godhead_notary::run_matrix_proposal(&store, proposal.proposal_id)
        .await
        .expect("idempotent retry");
    assert_eq!(retry.revision, first.revision, "nothing applied twice");
}
