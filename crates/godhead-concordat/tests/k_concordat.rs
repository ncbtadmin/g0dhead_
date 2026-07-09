//! Section K â€” The Teacher's Lint & the Concordat. SC-K01 â€¦ SC-K07.

use godhead_concordat::{
    compute_skew, disclose_regular_output, lint_instruction, read_instruction, write_instruction,
    ConcordatError, BIAS_SCOPE,
};
use godhead_intake::{Dispatcher, IntakePipe};
use godhead_ml::{aggregate, slave, LexicalEmbedder, Roster};
use godhead_schemas::{
    AcceptanceCriterion, AgentType, Budgets, CapabilityAction, ConfigTier, EnvKind,
    InstructionDraft, JobDraft, JobRecord, JobStatus, SourceDraw, Step, TestableAs, Tier,
};
use godhead_scriptorium::establish;
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

async fn store() -> Option<PgStore> {
    let Some(url) = database_url() else {
        eprintln!("SKIP: DATABASE_URL unset â€” database-backed criterion NOT exercised");
        return None;
    };
    let mut reg = godhead_intake::registry();
    godhead_ml::register_into(&mut reg);
    godhead_concordat::register_into(&mut reg);
    Some(
        PgStore::connect(&url, reg)
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

async fn ensure_threshold(store: &PgStore) {
    loop {
        match store.get_config("coherence_threshold").await {
            Ok(c) => {
                if c.value == json!(0.01) {
                    return;
                }
                if store
                    .set_config(
                        "sovereign",
                        "coherence_threshold",
                        ConfigTier::Sovereign,
                        &json!(0.01),
                        Some(c.revision),
                    )
                    .await
                    .is_ok()
                {
                    return;
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

async fn commit_to_rest(pipe: &IntakePipe<'_, PgStore>, filename: &str, bytes: &[u8]) -> Uuid {
    let node_id = pipe.commit_file(filename, bytes).await.expect("commit");
    let dispatcher = Dispatcher::new(pipe);
    let scope = [node_id];
    dispatcher.tick_scoped(Some(&scope)).await.expect("tick 1");
    dispatcher.tick_scoped(Some(&scope)).await.expect("tick 2");
    node_id
}

/// Grows a matrix and a Devout Teacher environment bound to it.
async fn devout_teacher(store: &PgStore) -> (Uuid, Uuid, Uuid) {
    ensure_threshold(store).await;
    let pipe = IntakePipe::new(store, temp_root()).expect("pipe");
    let text = b"the joins are true and the cathedral stands\n";
    let a = commit_to_rest(&pipe, "a.md", text).await;
    let b = commit_to_rest(&pipe, "b.md", text).await;
    let scope = [a, b];
    let category = format!("concordat_{}", Uuid::now_v7());
    slave::backfill_tick(store, &lexical_roster(), pipe.data_root(), Some(&scope))
        .await
        .expect("backfill");
    let summary = aggregate::consolidate(store, &lexical_roster(), &category, &scope)
        .await
        .expect("consolidate");
    let matrix = summary.emerged.expect("emergence");
    let (_j, env) = establish(store, EnvKind::Teacher, Tier::Devout, matrix)
        .await
        .expect("teacher env");
    (matrix, env.env_id, a)
}

async fn student_job(store: &PgStore) -> JobRecord {
    let draft = JobDraft {
        agent_type: AgentType::Student,
        auditor_name: None,
        tier: Some(Tier::Devout),
        input_refs: vec![],
        env_ref: None,
        brief_ref: None,
        endpoint_alias: None,
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 120_000,
            max_tool_calls: 10,
            max_tokens: 1,
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

/// A conforming Devout-targeted Instruction from a Devout Teacher.
fn conforming(env: Uuid, node: Uuid) -> InstructionDraft {
    InstructionDraft {
        teacher_env_ref: Some(env),
        teacher_tier: Tier::Devout,
        target_tier: Tier::Devout,
        concordat_version: Version::new(1, 0, 0),
        objective: "refine the elected corpus toward completeness".to_string(),
        steps: vec![Step {
            step_id: 1,
            action: CapabilityAction::Refine,
            params: json!({ "refs": [node.to_string()] }),
            expected_output: "refined.doc@1.0".to_string(),
            budget_hint_tokens: 1000,
        }],
        acceptance_criteria: vec![AcceptanceCriterion {
            criterion: "every refined artifact validates against its schema".to_string(),
            testable_as: TestableAs::Validation("schema_conformance".to_string()),
        }],
        sources_drawn: vec![],
        supersedes_ref: None,
    }
}

/// SC-K01 â€” each lint clause (a)â€“(f) has a violating fixture that blocks
/// the write; a conforming Instruction passes (seven checks).
#[tokio::test]
async fn sc_k01_lint_clauses() {
    let Some(store) = store().await else { return };
    let (matrix, env, node) = devout_teacher(&store).await;
    let _ = matrix;

    // Conforming â†’ passes.
    assert!(
        lint_instruction(&store, &conforming(env, node))
            .await
            .expect("lint runs")
            .is_ok(),
        "the conforming Instruction lints clean"
    );

    // (a) Resolution: a step ref that does not resolve.
    let mut a = conforming(env, node);
    a.steps[0].params = json!({ "refs": [Uuid::now_v7().to_string()] });
    assert_clause(&store, &a, 'a').await;

    // (b) Capability: target a Regular Student (empty table).
    let mut b = conforming(env, node);
    b.target_tier = Tier::Regular;
    assert_clause(&store, &b, 'b').await;

    // (c) Closure: a step with no expected_output.
    let mut c = conforming(env, node);
    c.steps[0].expected_output = String::new();
    assert_clause(&store, &c, 'c').await;

    // (d) Checkability: every criterion is SOVEREIGN_JUDGMENT.
    let mut d = conforming(env, node);
    d.acceptance_criteria[0].testable_as = TestableAs::SovereignJudgment;
    assert_clause(&store, &d, 'd').await;

    // (e) Budget: a step over the Devout ceiling (200000).
    let mut e = conforming(env, node);
    e.steps[0].budget_hint_tokens = 9_999_999;
    assert_clause(&store, &e, 'e').await;

    // (f) Sovereignty: a fetch step (barred in v1).
    let mut f = conforming(env, node);
    f.steps[0].action = CapabilityAction::FetchPerWrit;
    assert_clause(&store, &f, 'f').await;

    // End to end: a violation refuses through write_instruction and writes
    // nothing; the conforming one flags.
    let err = write_instruction(&store, &b).await;
    assert!(
        matches!(err, Err(ConcordatError::LintFailed(_))),
        "the write refuses"
    );
    let flagged = write_instruction(&store, &conforming(env, node))
        .await
        .expect("conforming writes and flags");
    assert!(flagged.flagged, "the Instruction is flagged");
}

async fn assert_clause(store: &PgStore, draft: &InstructionDraft, clause: char) {
    let result = lint_instruction(store, draft).await.expect("lint runs");
    match result {
        Err(failure) => assert_eq!(failure.clause, clause, "wrong clause: {failure}"),
        Ok(()) => panic!("clause ({clause}) should have failed"),
    }
}

/// SC-K02 â€” SOVEREIGN_JUDGMENT is excluded from the machine-checkable
/// floor; an all-sovereign Instruction fails lint; a mix passes.
#[tokio::test]
async fn sc_k02_sovereign_judgment() {
    let Some(store) = store().await else { return };
    let (_m, env, node) = devout_teacher(&store).await;

    // All SOVEREIGN_JUDGMENT â†’ fails clause (d).
    let mut all = conforming(env, node);
    all.acceptance_criteria = vec![AcceptanceCriterion {
        criterion: "the writing reads well".to_string(),
        testable_as: TestableAs::SovereignJudgment,
    }];
    assert_clause(&store, &all, 'd').await;

    // A mix (one machine-checkable floor + one sovereign) passes.
    let mut mix = conforming(env, node);
    mix.acceptance_criteria = vec![
        AcceptanceCriterion {
            criterion: "form validates".to_string(),
            testable_as: TestableAs::Validation("form".to_string()),
        },
        AcceptanceCriterion {
            criterion: "the prose is graceful".to_string(),
            testable_as: TestableAs::SovereignJudgment,
        },
    ];
    assert!(
        lint_instruction(&store, &mix).await.expect("lint").is_ok(),
        "a machine-checkable floor plus sovereign judgment lints clean"
    );
}

/// SC-K03 â€” Concordat version skew in either direction refuses
/// SCHEMA_MISMATCH at the Student's end; every cited version is retained.
#[tokio::test]
async fn sc_k03_version_skew() {
    let Some(store) = store().await else { return };
    let (_m, env, node) = devout_teacher(&store).await;
    let tables = json!({
        "REGULAR": [], "DEVOUT": ["REFINE","VERIFY"], "CANON": ["VERIFY"]
    });

    // Adopt two flanking versions (idempotent across parallel runs).
    for v in ["2.0.0", "0.9.0"] {
        let version = Version::parse(v).unwrap();
        if store.get_concordat(&version).await.is_err() {
            let _ = store
                .adopt_concordat("sovereign", &version, &tables, &json!({}))
                .await;
        }
    }

    // An Instruction citing 2.0.0 (above the Student's ^1.0) refuses.
    let mut newer = conforming(env, node);
    newer.concordat_version = Version::new(2, 0, 0);
    let flagged = write_instruction(&store, &newer)
        .await
        .expect("writes against 2.0.0");
    let reader = student_job(&store).await;
    let err = read_instruction(&store, reader.job_id, flagged.instruction_id).await;
    assert!(
        matches!(err, Err(ConcordatError::SchemaMismatch(_))),
        "newer skew refuses"
    );

    // An Instruction citing 0.9.0 (below ^1.0) also refuses.
    let mut older = conforming(env, node);
    older.concordat_version = Version::new(0, 9, 0);
    let flagged = write_instruction(&store, &older)
        .await
        .expect("writes against 0.9.0");
    let err = read_instruction(&store, reader.job_id, flagged.instruction_id).await;
    assert!(
        matches!(err, Err(ConcordatError::SchemaMismatch(_))),
        "older skew refuses"
    );

    // Every cited version remains retrievable forever (Â§3.3).
    for v in ["1.0.0", "2.0.0", "0.9.0"] {
        store
            .get_concordat(&Version::parse(v).unwrap())
            .await
            .unwrap_or_else(|_| panic!("Concordat {v} retained"));
    }
}

/// SC-K04 â€” double-validation: an Instruction corrupted between flag and
/// read is caught by the Student's VALIDATE_IN, not the flag.
#[tokio::test]
async fn sc_k04_double_validation() {
    let Some(store) = store().await else { return };
    let (_m, env, node) = devout_teacher(&store).await;
    let flagged = write_instruction(&store, &conforming(env, node))
        .await
        .expect("write");
    // A clean read succeeds.
    let reader = student_job(&store).await;
    read_instruction(&store, reader.job_id, flagged.instruction_id)
        .await
        .expect("a valid Instruction reads");

    // Corrupt the body out-of-band â€” bypassing the immutability trigger to
    // simulate a corruption the trigger did not catch (defense in depth):
    // rewrite the step to a fetch action, which the re-lint rejects.
    sqlx::query("ALTER TABLE instructions DISABLE TRIGGER instruction_immutable")
        .execute(store.raw_pool())
        .await
        .expect("disable trigger");
    sqlx::query(
        r#"UPDATE instructions
           SET steps = jsonb_build_array(jsonb_build_object(
               'step_id', 1, 'action', 'FETCH_PER_WRIT', 'params', '{}'::jsonb,
               'expected_output', 'x@1.0', 'budget_hint_tokens', 1))
           WHERE instruction_id = $1"#,
    )
    .bind(flagged.instruction_id)
    .execute(store.raw_pool())
    .await
    .expect("corrupt");
    sqlx::query("ALTER TABLE instructions ENABLE TRIGGER instruction_immutable")
        .execute(store.raw_pool())
        .await
        .expect("re-enable trigger");

    let err = read_instruction(&store, reader.job_id, flagged.instruction_id).await;
    assert!(
        matches!(err, Err(ConcordatError::SchemaMismatch(_))),
        "the Student's VALIDATE_IN catches the corruption"
    );
}

/// SC-K05 â€” a flagged Instruction is immutable; correction flows through
/// supersedes_ref and the chain resolves.
#[tokio::test]
async fn sc_k05_immutable_supersede() {
    let Some(store) = store().await else { return };
    let (_m, env, node) = devout_teacher(&store).await;
    let first = write_instruction(&store, &conforming(env, node))
        .await
        .expect("write");

    // A direct edit of the flagged body is rejected at the substrate.
    let edit =
        sqlx::query("UPDATE instructions SET objective = 'sneaky edit' WHERE instruction_id = $1")
            .bind(first.instruction_id)
            .execute(store.raw_pool())
            .await;
    assert!(edit.is_err(), "a flagged Instruction is immutable (Â§1.4)");

    // Correction is a new Instruction superseding the old.
    let mut correction = conforming(env, node);
    correction.objective = "the corrected objective".to_string();
    correction.supersedes_ref = Some(first.instruction_id);
    let second = write_instruction(&store, &correction)
        .await
        .expect("supersede");
    assert_eq!(
        second.supersedes_ref,
        Some(first.instruction_id),
        "the chain resolves"
    );
    // The superseded Instruction stays readable â€” what version one read
    // must remain provable.
    let reader = student_job(&store).await;
    read_instruction(&store, reader.job_id, first.instruction_id)
        .await
        .expect("the superseded Instruction remains readable");
}

/// SC-K06 â€” a Regular Teacher output missing sources_drawn fails; skew
/// computes against draws and bias_skew_threshold.
#[tokio::test]
async fn sc_k06_bias_disclosure() {
    let Some(store) = store().await else { return };
    let (matrix, _env, node) = devout_teacher(&store).await;

    // A Regular Teacher instruction (no environment) with no sources_drawn
    // is refused.
    let mut regular = InstructionDraft {
        teacher_env_ref: None,
        teacher_tier: Tier::Regular,
        target_tier: Tier::Devout,
        concordat_version: Version::new(1, 0, 0),
        objective: "leverage material across the store".to_string(),
        steps: vec![Step {
            step_id: 1,
            action: CapabilityAction::Refine,
            params: json!({ "refs": [node.to_string()] }),
            expected_output: "refined@1.0".to_string(),
            budget_hint_tokens: 500,
        }],
        acceptance_criteria: vec![AcceptanceCriterion {
            criterion: "form validates".to_string(),
            testable_as: TestableAs::Validation("form".to_string()),
        }],
        sources_drawn: vec![],
        supersedes_ref: None,
    };
    let err = write_instruction(&store, &regular).await;
    // The lint passes (it is a valid instruction); the store's persist
    // refuses the missing disclosure â€” surfaced as a Store error.
    assert!(
        err.is_err(),
        "a Regular output without sources_drawn is refused"
    );

    // skew computes: 2 of 3 draws canon-associated, threshold 0.50 â†’ skew.
    let sources = vec![
        SourceDraw {
            matrix_ref: matrix,
            draw_count: 2,
            canon_associated: true,
        },
        SourceDraw {
            matrix_ref: Uuid::now_v7(),
            draw_count: 1,
            canon_associated: false,
        },
    ];
    assert!(
        compute_skew(&sources, 0.50),
        "2/3 canon draws exceeds 0.50 â†’ skew"
    );
    assert!(!compute_skew(&sources, 0.80), "2/3 does not exceed 0.80");

    // With disclosure, the Regular instruction writes.
    regular.sources_drawn = sources;
    let flagged = write_instruction(&store, &regular)
        .await
        .expect("a disclosed Regular output writes");
    assert!(flagged.skew, "the skew mark is recorded");
}

/// SC-K07 â€” pattern escalation: crossing the threshold raises one standing
/// warning; acknowledge keeps it; silence suppresses it, not re-raised.
#[tokio::test]
async fn sc_k07_pattern_escalation() {
    let Some(store) = store().await else { return };
    let (_m, _env, node) = devout_teacher(&store).await;
    // Anchor an instruction the disclosures reference.
    let mut regular = InstructionDraft {
        teacher_env_ref: None,
        teacher_tier: Tier::Regular,
        target_tier: Tier::Devout,
        concordat_version: Version::new(1, 0, 0),
        objective: "regular leverage".to_string(),
        steps: vec![Step {
            step_id: 1,
            action: CapabilityAction::Verify,
            params: json!({}),
            expected_output: "v@1.0".to_string(),
            budget_hint_tokens: 100,
        }],
        acceptance_criteria: vec![AcceptanceCriterion {
            criterion: "form".to_string(),
            testable_as: TestableAs::Validation("form".to_string()),
        }],
        sources_drawn: vec![SourceDraw {
            matrix_ref: Uuid::now_v7(),
            draw_count: 1,
            canon_associated: true,
        }],
        supersedes_ref: None,
    };
    let _ = node;
    let anchor = write_instruction(&store, &regular).await.expect("anchor");
    regular.supersedes_ref = None;

    // Reset the warning to a raisable state (the sovereign lifting a prior
    // run's warning) so the raise is deterministic across runs.
    sqlx::query("DELETE FROM bias_warnings WHERE scope = $1")
        .bind(BIAS_SCOPE)
        .execute(store.raw_pool())
        .await
        .expect("reset warning");

    // A burst of fully-canon (skewed) disclosures dominates the trailing
    // window and crosses bias_pattern_threshold.
    let skewed = vec![SourceDraw {
        matrix_ref: Uuid::now_v7(),
        draw_count: 1,
        canon_associated: true,
    }];
    let mut stood = false;
    for _ in 0..30 {
        let (was_skew, stands) = disclose_regular_output(&store, anchor.instruction_id, &skewed)
            .await
            .expect("disclose");
        assert!(was_skew, "a fully-canon output is skewed");
        stood = stands;
    }
    assert!(
        stood,
        "a sustained skewed pattern raises a standing warning"
    );
    // Exactly one warning (idempotent raise).
    assert_eq!(
        store
            .bias_warning_state(BIAS_SCOPE)
            .await
            .expect("state")
            .as_deref(),
        Some("STANDING")
    );

    // Acknowledge â†’ keeps counting (still stands, status ACKNOWLEDGED).
    store
        .resolve_bias_warning("sovereign", BIAS_SCOPE, true)
        .await
        .expect("acknowledge");
    assert_eq!(
        store
            .bias_warning_state(BIAS_SCOPE)
            .await
            .expect("state")
            .as_deref(),
        Some("ACKNOWLEDGED")
    );

    // Silence â†’ suppressed; a further crossing does not re-raise it.
    store
        .resolve_bias_warning("sovereign", BIAS_SCOPE, false)
        .await
        .expect("silence");
    let (_s, stands) = disclose_regular_output(&store, anchor.instruction_id, &skewed)
        .await
        .expect("disclose after silence");
    assert!(!stands, "a silenced scope is not re-raised until lifted");
    assert_eq!(
        store
            .bias_warning_state(BIAS_SCOPE)
            .await
            .expect("state")
            .as_deref(),
        Some("SILENCED")
    );
}

/// Regression (slice-8 review, MEDIUM): a non-array `refs` or `consumes`
/// does not silently skip its clause — it fails the clause.
#[tokio::test]
async fn non_array_refs_fails_the_clause() {
    let Some(store) = store().await else { return };
    let (_m, env, node) = devout_teacher(&store).await;

    // refs as a bare string (a dangling uuid) — must fail clause (a),
    // not skip resolution.
    let mut a = conforming(env, node);
    a.steps[0].params = json!({ "refs": Uuid::now_v7().to_string() });
    assert_clause(&store, &a, 'a').await;

    // consumes as a non-array — must fail clause (c).
    let mut c = conforming(env, node);
    c.steps[0].params = json!({ "refs": [node.to_string()], "consumes": 1 });
    assert_clause(&store, &c, 'c').await;
}

/// Regression (slice-8 review, MEDIUM): a step referencing a real link is
/// resolvable — links are first-class store objects (clause a must not
/// over-refuse them).
#[tokio::test]
async fn link_ref_resolves() {
    let Some(store) = store().await else { return };
    let (matrix, env, _node) = devout_teacher(&store).await;
    let link_id = store.get_matrix(matrix).await.expect("matrix").link_refs[0];
    let mut d = conforming(env, link_id);
    d.steps[0].params = json!({ "refs": [link_id.to_string()] });
    assert!(
        lint_instruction(&store, &d).await.expect("lint").is_ok(),
        "a step referencing a live link resolves"
    );
}

/// Regression (slice-8 review, MEDIUM): an empty validation id is not a
/// machine-checkable floor — clause (d) rejects it.
#[tokio::test]
async fn empty_validation_id_fails_clause_d() {
    let Some(store) = store().await else { return };
    let (_m, env, node) = devout_teacher(&store).await;
    let mut d = conforming(env, node);
    d.acceptance_criteria = vec![AcceptanceCriterion {
        criterion: "some criterion".to_string(),
        testable_as: TestableAs::Validation(String::new()),
    }];
    assert_clause(&store, &d, 'd').await;
}

/// Regression (slice-9 review, HIGH — the write_instruction mirror of
/// `mid_labor_halt_refuses_never_strands`): a store wall firing after
/// VALIDATE_OUT passes ends the Teacher's job REFUSED with the stage on
/// record, never stranded RUNNING (Law VII).
#[tokio::test]
async fn teacher_mid_labor_halt_refuses_never_strands() {
    let Some(store) = store().await else { return };
    let (_m, env, node) = devout_teacher(&store).await;

    // A Devout draft carrying sources_drawn: the lint does not police
    // disclosure, so VALIDATE_OUT passes — and persist_instruction's B.1
    // wall (a conferred Teacher carries none) halts the labor mid-flight.
    let mut d = conforming(env, node);
    d.sources_drawn = vec![SourceDraw {
        matrix_ref: Uuid::now_v7(),
        draw_count: 1,
        canon_associated: true,
    }];
    assert!(
        lint_instruction(&store, &d)
            .await
            .expect("lint runs")
            .is_ok(),
        "disclosure is not the lint's clause; the wall is the store's"
    );
    let err = write_instruction(&store, &d).await;
    assert!(
        matches!(
            err,
            Err(ConcordatError::Store(StoreError::ValidationFailed(_)))
        ),
        "the wall at persist holds: {err:?}"
    );

    // The laborer did not strand: REFUSED, the stage on record. The
    // Teacher job carries its room in input_refs (unique per test).
    let jobs: Vec<(Uuid, String)> = sqlx::query_as(
        r#"SELECT job_id, status FROM job_records
           WHERE input_refs @> jsonb_build_array($1::text)"#,
    )
    .bind(env.to_string())
    .fetch_all(store.raw_pool())
    .await
    .expect("jobs");
    assert!(!jobs.is_empty(), "the labor got its laborer");
    let halted: Vec<&(Uuid, String)> = jobs.iter().filter(|(_, s)| s != "TERMINATED").collect();
    assert!(!halted.is_empty(), "the halted labor is visible");
    for (job_id, status) in &halted {
        assert_eq!(status, "REFUSED", "job {job_id} must not strand live");
    }
    let details: Vec<(String,)> =
        sqlx::query_as("SELECT detail FROM refusal_records WHERE job_id = $1")
            .bind(halted[0].0)
            .fetch_all(store.raw_pool())
            .await
            .expect("refusals");
    assert_eq!(details.len(), 1, "the refusal is on record");
    assert!(
        details[0].0.contains("stage 'persist'"),
        "the stage is named: {}",
        details[0].0
    );
}

/// Regression (slice-9 review, HIGH — the write_instruction mirror of
/// `refusal_never_echoes_the_draft`): a hostile draft cannot suppress its
/// own refusal. A secret-shaped semver prerelease rides the version string
/// into the ephemeral lint detail — the PERSISTED detail names the clause
/// only, so the Law XV scan passes and the record lands.
#[tokio::test]
async fn teacher_refusal_never_echoes_the_draft() {
    let Some(store) = store().await else { return };
    let (_m, env, node) = devout_teacher(&store).await;

    let mut hostile = conforming(env, node);
    hostile.concordat_version = Version::parse("1.0.0-AKIA0123456789ABCDEF").expect("valid semver");
    let err = write_instruction(&store, &hostile).await;
    assert!(
        matches!(err, Err(ConcordatError::LintFailed(_))),
        "the hostile draft refuses at the lint: {err:?}"
    );

    // The refusal record LANDED — clause-referencing, echo-free.
    let details: Vec<(String,)> = sqlx::query_as(
        r#"SELECT r.detail FROM refusal_records r
             JOIN job_records j ON j.job_id = r.job_id
           WHERE j.input_refs @> jsonb_build_array($1::text)"#,
    )
    .bind(env.to_string())
    .fetch_all(store.raw_pool())
    .await
    .expect("refusals");
    assert_eq!(details.len(), 1, "the refusal is on record");
    assert!(
        details[0].0.contains("clause 'b'"),
        "the clause is named: {}",
        details[0].0
    );
    assert!(
        !details[0].0.contains("AKIA"),
        "the emission is never echoed: {}",
        details[0].0
    );
}

/// Regression (slice-8 review, MEDIUM/spec): `skew` is derived from the
/// disclosed draws, never trusted; and a conferred Teacher may not carry
/// sources_drawn.
#[tokio::test]
async fn skew_is_derived_and_sources_are_regular_only() {
    let Some(store) = store().await else { return };
    let (_m, _env, node) = devout_teacher(&store).await;

    fn regular(node: Uuid, sources: Vec<SourceDraw>) -> InstructionDraft {
        InstructionDraft {
            teacher_env_ref: None,
            teacher_tier: Tier::Regular,
            target_tier: Tier::Devout,
            concordat_version: Version::new(1, 0, 0),
            objective: "regular leverage".to_string(),
            steps: vec![Step {
                step_id: 1,
                action: CapabilityAction::Refine,
                params: json!({ "refs": [node.to_string()] }),
                expected_output: "r@1.0".to_string(),
                budget_hint_tokens: 100,
            }],
            acceptance_criteria: vec![AcceptanceCriterion {
                criterion: "form".to_string(),
                testable_as: TestableAs::Validation("form".to_string()),
            }],
            sources_drawn: sources,
            supersedes_ref: None,
        }
    }

    // Mostly non-canon draws → derived skew is false, whatever a caller
    // might have wished.
    let low = regular(
        node,
        vec![
            SourceDraw {
                matrix_ref: Uuid::now_v7(),
                draw_count: 1,
                canon_associated: true,
            },
            SourceDraw {
                matrix_ref: Uuid::now_v7(),
                draw_count: 9,
                canon_associated: false,
            },
        ],
    );
    let flagged = write_instruction(&store, &low).await.expect("low writes");
    assert!(
        !flagged.skew,
        "1/10 canon draws is not skew — derived, not trusted"
    );

    // A Devout Teacher carrying sources_drawn is refused (B.1).
    let (matrix2, env2, node2) = devout_teacher(&store).await;
    let _ = matrix2;
    let mut devout = conforming(env2, node2);
    devout.sources_drawn = vec![SourceDraw {
        matrix_ref: Uuid::now_v7(),
        draw_count: 1,
        canon_associated: true,
    }];
    let err = write_instruction(&store, &devout).await;
    assert!(err.is_err(), "a conferred Teacher carries no sources_drawn");
}

/// Hardening (external review [3]): a field that no longer parses fails
/// reconstruction loudly — a corrupted budget must not read back as 0 and
/// trivially pass clause (e).
#[tokio::test]
async fn corrupted_budget_fails_reconstruction_loudly() {
    let Some(store) = store().await else { return };
    let (_m, env, node) = devout_teacher(&store).await;
    let flagged = write_instruction(&store, &conforming(env, node))
        .await
        .expect("write");

    // Corrupt budget_hint_tokens to a string out-of-band, bypassing the
    // immutability trigger (defense in depth, the SC-K04 pattern).
    sqlx::query("ALTER TABLE instructions DISABLE TRIGGER instruction_immutable")
        .execute(store.raw_pool())
        .await
        .expect("disable trigger");
    sqlx::query(
        r#"UPDATE instructions
           SET steps = jsonb_build_array(jsonb_build_object(
               'step_id', 1, 'action', 'REFINE', 'params', '{}'::jsonb,
               'expected_output', 'x@1.0', 'budget_hint_tokens', 'a great many'))
           WHERE instruction_id = $1"#,
    )
    .bind(flagged.instruction_id)
    .execute(store.raw_pool())
    .await
    .expect("corrupt");
    sqlx::query("ALTER TABLE instructions ENABLE TRIGGER instruction_immutable")
        .execute(store.raw_pool())
        .await
        .expect("re-enable trigger");

    let reader = student_job(&store).await;
    let err = read_instruction(&store, reader.job_id, flagged.instruction_id).await;
    assert!(
        matches!(err, Err(ConcordatError::SchemaMismatch(_))),
        "an unparseable budget fails reconstruction, never reads as 0"
    );
}

/// Hardening (external review [2]): a present-but-malformed sovereign
/// constant refuses — a fabricated default threshold would be a decision
/// the sovereign never made (Law II.2).
#[tokio::test]
async fn malformed_bias_config_refuses_never_guesses() {
    let Some(store) = store().await else { return };
    // Malform the constant out-of-band and restore it immediately below —
    // the window is kept minimal because the parallel bias tests read the
    // same row (the sc_k07 caveat).
    sqlx::query(
        r#"UPDATE config_constants SET value = '"half"'::jsonb
           WHERE key = 'bias_skew_threshold'"#,
    )
    .execute(store.raw_pool())
    .await
    .expect("malform");
    let sources = vec![SourceDraw {
        matrix_ref: Uuid::now_v7(),
        draw_count: 1,
        canon_associated: true,
    }];
    let err = disclose_regular_output(&store, Uuid::now_v7(), &sources).await;
    sqlx::query(
        r#"UPDATE config_constants SET value = '0.50'::jsonb
           WHERE key = 'bias_skew_threshold'"#,
    )
    .execute(store.raw_pool())
    .await
    .expect("restore");
    assert!(
        matches!(
            err,
            Err(ConcordatError::Store(StoreError::ValidationFailed(_)))
        ),
        "a malformed threshold refuses; it is never guessed"
    );
}
