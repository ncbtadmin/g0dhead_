//! Section L — Student Returns & Stewardship. SC-L01 … SC-L04.

use godhead_concordat::write_instruction;
use godhead_intake::{Dispatcher, IntakePipe};
use godhead_ml::{aggregate, slave, LexicalEmbedder, Roster};
use godhead_schemas::{
    AcceptanceCriterion, AgentType, Budgets, CapabilityAction, CompletionEntry, ConfigTier,
    EnvKind, InstructionDraft, InstructionRecord, JobDraft, JobRecord, JobStatus, LogEvent,
    PairingKind, PetitionStatus, ReturnDraft, ReturnItem, ReturnItemKind, Step, TestableAs, Tier,
};
use godhead_scriptorium::{establish, mount, ScriptoriumError};
use godhead_store::{PgStore, Store, StoreError};
use godhead_student::{
    re_derive, redundant_consistency, refine, steward_consolidate, validate_return, write_return,
    StudentError, REFINE_METHOD,
};
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
        eprintln!("SKIP: DATABASE_URL unset — database-backed criterion NOT exercised");
        return None;
    };
    let mut reg = godhead_intake::registry();
    godhead_ml::register_into(&mut reg);
    godhead_concordat::register_into(&mut reg);
    godhead_student::register_into(&mut reg);
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

struct DevoutPair {
    matrix: Uuid,
    teacher_env: Uuid,
    student_env: Uuid,
    node_a: Uuid,
    node_b: Uuid,
}

/// Grows a matrix and the paired Teacher/Student rooms bound to it — the
/// manual pair section L completes. The two source files carry DIFFERENT
/// bytes so their derivative checksums differ: the canonical-fold and
/// source-identity assertions must be able to fail.
async fn devout_pair(store: &PgStore) -> DevoutPair {
    ensure_threshold(store).await;
    let pipe = IntakePipe::new(store, temp_root()).expect("pipe");
    let a = commit_to_rest(
        &pipe,
        "a.md",
        b"the joins are true and the cathedral stands\n",
    )
    .await;
    let b = commit_to_rest(
        &pipe,
        "b.md",
        b"the joins are true and the cathedral endures\n",
    )
    .await;
    let scope = [a, b];
    let category = format!("student_{}", Uuid::now_v7());
    slave::backfill_tick(store, &lexical_roster(), pipe.data_root(), Some(&scope))
        .await
        .expect("backfill");
    let summary = aggregate::consolidate(store, &lexical_roster(), &category, &scope)
        .await
        .expect("consolidate");
    let matrix = summary.emerged.expect("emergence");
    let (_j, teacher) = establish(store, EnvKind::Teacher, Tier::Devout, matrix)
        .await
        .expect("teacher env");
    let (_j, student) = establish(store, EnvKind::Student, Tier::Devout, matrix)
        .await
        .expect("student env");
    // X.5: the bridge a Return answers across.
    store
        .form_pairing(
            teacher.env_id,
            student.env_id,
            matrix,
            PairingKind::DevoutAssignment,
        )
        .await
        .expect("pairing");
    DevoutPair {
        matrix,
        teacher_env: teacher.env_id,
        student_env: student.env_id,
        node_a: a,
        node_b: b,
    }
}

async fn running_job(store: &PgStore, agent_type: AgentType) -> JobRecord {
    let draft = JobDraft {
        agent_type,
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

async fn student_job(store: &PgStore) -> JobRecord {
    running_job(store, AgentType::Student).await
}

/// A running Student job born BOUND to its room (env_ref + matrix in
/// inputs — the Law IX.4 binding create_job authenticates).
async fn bound_student_job(store: &PgStore, env_id: Uuid, matrix: Uuid) -> JobRecord {
    let draft = JobDraft {
        agent_type: AgentType::Student,
        auditor_name: None,
        tier: Some(Tier::Devout),
        input_refs: vec![matrix],
        env_ref: Some(env_id),
        brief_ref: None,
        endpoint_alias: None,
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 120_000,
            max_tool_calls: 10,
            max_tokens: 1,
        },
    };
    let job = store.create_job(&draft).await.expect("create bound");
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await
        .expect("lease");
    store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await
        .expect("run")
}

/// A C.2-shaped provenance chain rooted in a human act, producing `re`.
fn valid_chain(re: Uuid) -> serde_json::Value {
    json!([
        { "link_seq": 0, "kind": "BRIEF", "actor": Uuid::now_v7().to_string(),
          "prompt_or_reason": "the sovereign's charge", "produced": [re.to_string()] },
        { "link_seq": 1, "kind": "REFINEMENT", "actor": Uuid::now_v7().to_string(),
          "prompt_or_reason": "refined into the room", "produced": [re.to_string()] },
    ])
}

/// A conforming Instruction with a mixed contract: one machine-checkable
/// floor plus one sovereign judgment (§1.3d).
fn instruction_draft(env: Uuid, node: Uuid) -> InstructionDraft {
    InstructionDraft {
        teacher_env_ref: Some(env),
        teacher_tier: Tier::Devout,
        target_tier: Tier::Devout,
        concordat_version: Version::new(1, 0, 0),
        objective: "refine the elected corpus and hand back a Return".to_string(),
        steps: vec![Step {
            step_id: 1,
            action: CapabilityAction::Refine,
            params: json!({ "refs": [node.to_string()] }),
            expected_output: "refined.doc@1.0".to_string(),
            budget_hint_tokens: 1000,
        }],
        acceptance_criteria: vec![
            AcceptanceCriterion {
                criterion: "every refined artifact validates against its schema".to_string(),
                testable_as: TestableAs::Validation("schema_conformance".to_string()),
            },
            AcceptanceCriterion {
                criterion: "the refinement reads faithful to its sources".to_string(),
                testable_as: TestableAs::SovereignJudgment,
            },
        ],
        sources_drawn: vec![],
        supersedes_ref: None,
    }
}

/// A conforming Return: one entry per criterion, evidence everywhere, a
/// verdict on the machine-checkable floor and none on the sovereign's.
fn conforming_return(instruction: &InstructionRecord, student_env: Uuid) -> ReturnDraft {
    ReturnDraft {
        instruction_ref: instruction.instruction_id,
        student_env_ref: student_env,
        concordat_version: instruction.concordat_version.clone(),
        items: vec![ReturnItem {
            item_ref: Uuid::now_v7(),
            kind: ReturnItemKind::RefinedDoc,
            provenance_ref: Uuid::now_v7(),
        }],
        completion: vec![
            CompletionEntry {
                criterion_index: 0,
                passed: Some(true),
                evidence_ref: Uuid::now_v7(),
            },
            CompletionEntry {
                criterion_index: 1,
                passed: None,
                evidence_ref: Uuid::now_v7(),
            },
        ],
    }
}

/// The invalid Return fails VALIDATE_OUT and refuses through write_return —
/// it never flags, never poisons.
async fn assert_invalid(store: &PgStore, draft: &ReturnDraft, needle: &str) {
    let result = validate_return(store, draft).await.expect("validate runs");
    match result {
        Err(failure) => assert!(
            failure.detail.contains(needle),
            "unexpected failure: {failure}"
        ),
        Ok(()) => panic!("the Return should have failed ({needle})"),
    }
    let err = write_return(store, draft).await;
    assert!(
        matches!(err, Err(StudentError::ReturnInvalid(_))),
        "the write refuses"
    );
}

/// SC-L01 — the completion contract: exactly one entry per criterion
/// (missing/extra/duplicate invalidate); evidence mandatory in every case;
/// `passed` is None iff the criterion is SOVEREIGN_JUDGMENT.
#[tokio::test]
async fn sc_l01_completion_contract() {
    let Some(store) = store().await else { return };
    let pair = devout_pair(&store).await;
    let instruction = write_instruction(&store, &instruction_draft(pair.teacher_env, pair.node_a))
        .await
        .expect("instruction");

    // Conforming → validates, writes, flags.
    let ok = conforming_return(&instruction, pair.student_env);
    assert!(
        validate_return(&store, &ok)
            .await
            .expect("validate runs")
            .is_ok(),
        "the conforming Return validates"
    );
    let flagged = write_return(&store, &ok)
        .await
        .expect("conforming writes and flags");
    assert!(flagged.flagged, "the Return is flagged");
    let read_back = store.get_return(flagged.return_id).await.expect("read");
    assert_eq!(read_back.instruction_ref, instruction.instruction_id);

    // A missing criterion entry.
    let mut missing = conforming_return(&instruction, pair.student_env);
    missing.completion.pop();
    assert_invalid(&store, &missing, "exactly one each").await;

    // An extra entry.
    let mut extra = conforming_return(&instruction, pair.student_env);
    extra.completion.push(CompletionEntry {
        criterion_index: 1,
        passed: None,
        evidence_ref: Uuid::now_v7(),
    });
    assert_invalid(&store, &extra, "exactly one each").await;

    // An entry beyond the Instruction's criteria.
    let mut beyond = conforming_return(&instruction, pair.student_env);
    beyond.completion[1].criterion_index = 2;
    assert_invalid(&store, &beyond, "beyond the Instruction").await;

    // A duplicate index.
    let mut dup = conforming_return(&instruction, pair.student_env);
    dup.completion[1] = CompletionEntry {
        criterion_index: 0,
        passed: Some(true),
        evidence_ref: Uuid::now_v7(),
    };
    assert_invalid(&store, &dup, "answered twice").await;

    // A machine-checkable criterion with no verdict.
    let mut unjudged = conforming_return(&instruction, pair.student_env);
    unjudged.completion[0].passed = None;
    assert_invalid(&store, &unjudged, "renders a verdict").await;

    // A sovereign criterion with a Student verdict.
    let mut judged = conforming_return(&instruction, pair.student_env);
    judged.completion[1].passed = Some(false);
    assert_invalid(&store, &judged, "sovereign's to render").await;

    // A nil evidence_ref — evidence is mandatory in every case.
    let mut nil = conforming_return(&instruction, pair.student_env);
    nil.completion[0].evidence_ref = Uuid::nil();
    assert_invalid(&store, &nil, "evidence is mandatory").await;

    // A nil item ref — the manifest's payload half carries the same floor.
    let mut nil_item = conforming_return(&instruction, pair.student_env);
    nil_item.items[0].item_ref = Uuid::nil();
    assert_invalid(&store, &nil_item, "carries a nil ref").await;

    // The store is the wall beneath the crate: a direct persist of an
    // invalid Return refuses (the double-validation covenant).
    let job = bound_student_job(&store, pair.student_env, pair.matrix).await;
    let err = store.persist_return(job.job_id, &missing).await;
    assert!(
        matches!(err, Err(StoreError::ValidationFailed(_))),
        "the wall holds: {err:?}"
    );

    // And only a Student writes Returns.
    let teacher = running_job(&store, AgentType::Teacher).await;
    let err = store.persist_return(teacher.job_id, &ok).await;
    assert!(
        matches!(err, Err(StoreError::ValidationFailed(_))),
        "only a Student writes Returns: {err:?}"
    );

    // §3.1 at the substrate: a flagged Return's body is frozen, and the
    // record is never deleted — correction is a fresh Return.
    let edit = sqlx::query("UPDATE returns SET completion = '[]'::jsonb WHERE return_id = $1")
        .bind(flagged.return_id)
        .execute(store.raw_pool())
        .await;
    assert!(edit.is_err(), "a flagged Return is immutable (§3.1)");
    let del = sqlx::query("DELETE FROM returns WHERE return_id = $1")
        .bind(flagged.return_id)
        .execute(store.raw_pool())
        .await;
    assert!(del.is_err(), "a Return is never deleted");
}

/// SC-L02 — Devout consolidation over human-held data leaves it untouched
/// (structural diff); it may petition, never write.
#[tokio::test]
async fn sc_l02_human_held_untouched() {
    let Some(store) = store().await else { return };
    let pair = devout_pair(&store).await;

    // The sovereign's hand on node A.
    let scripture = json!([{ "category": "scripture" }]);
    let laid = store
        .lay_category_override("sovereign", pair.node_a, &scripture)
        .await
        .expect("the sovereign's hand");
    assert!(laid.user_overridden);

    let job = student_job(&store).await;
    let target = format!("consolidated_{}", Uuid::now_v7());
    let report = steward_consolidate(&store, job.job_id, &[pair.node_a, pair.node_b], &target)
        .await
        .expect("consolidation runs");

    // The hand-held node is untouched — structural diff, not summary.
    let a = store.get_node(pair.node_a).await.expect("node a");
    assert_eq!(a.classification, scripture, "human-held data is untouched");
    // The free node consolidated.
    let b = store.get_node(pair.node_b).await.expect("node b");
    assert_eq!(
        b.classification[0]["category"],
        json!(target),
        "the free node consolidates"
    );
    assert_eq!(report.consolidated, vec![pair.node_b]);

    // A petition was opened — the steward's only voice on the star.
    assert_eq!(report.petitioned.len(), 1, "one petition, for node A");
    let (subject, petition_id) = report.petitioned[0];
    assert_eq!(subject, pair.node_a);
    let petition = store.get_petition(petition_id).await.expect("petition");
    assert_eq!(petition.subject_ref, pair.node_a);
    assert_eq!(petition.status, PetitionStatus::Open);

    // Recurrence escalates the same lineage; the star still stands.
    let job2 = student_job(&store).await;
    let report2 = steward_consolidate(&store, job2.job_id, &[pair.node_a], &target)
        .await
        .expect("second pass");
    assert_eq!(report2.petitioned.len(), 1);
    let petition = store
        .get_petition(report2.petitioned[0].1)
        .await
        .expect("petition");
    assert_eq!(petition.status, PetitionStatus::Escalated);
    assert_eq!(petition.occurrence_count, 2);
    let a = store.get_node(pair.node_a).await.expect("node a");
    assert_eq!(a.classification, scripture, "still untouched");
}

/// SC-L03 — redundant consistency: re-running a refinement from its
/// derivation reproduces the artifact; a dangling intra-scriptorium ref
/// fails closure.
#[tokio::test]
async fn sc_l03_redundant_consistency() {
    let Some(store) = store().await else { return };
    let pair = devout_pair(&store).await;
    let job = bound_student_job(&store, pair.student_env, pair.matrix).await;

    // A deterministic refinement, persisted with its derivation.
    let artifact = refine(
        &store,
        job.job_id,
        pair.student_env,
        &[pair.node_a, pair.node_b],
    )
    .await
    .expect("refine");
    assert_eq!(artifact.method, REFINE_METHOD);

    // Unchanged sources → the derivation reproduces the digest.
    let recomputed = re_derive(&store, artifact.artifact_id)
        .await
        .expect("re-derive");
    assert_eq!(recomputed, artifact.content_sha, "re-derivation reproduces");

    // Source order does not matter: the derivation is canonical.
    let mirrored = refine(
        &store,
        job.job_id,
        pair.student_env,
        &[pair.node_b, pair.node_a],
    )
    .await
    .expect("mirrored refine");
    assert_eq!(mirrored.content_sha, artifact.content_sha);

    // The clean scriptorium walks consistent.
    assert!(
        redundant_consistency(&store, pair.student_env)
            .await
            .expect("walk runs")
            .is_ok(),
        "a clean room is consistent"
    );

    // A dangling derivation ref fails closure (c). The store lets the
    // record in — the walk is the detector, debris is found not hidden.
    let (_j, room2) = establish(&store, EnvKind::Student, Tier::Devout, pair.matrix)
        .await
        .expect("second room");
    let job2 = bound_student_job(&store, room2.env_id, pair.matrix).await;
    store
        .persist_refined_artifact(
            job2.job_id,
            room2.env_id,
            &[Uuid::now_v7()],
            REFINE_METHOD,
            &artifact.content_sha,
        )
        .await
        .expect("debris persists");
    let debris = redundant_consistency(&store, room2.env_id)
        .await
        .expect("walk runs")
        .expect_err("closure must fail");
    assert_eq!(debris.property, 'c', "wrong property: {debris}");

    // An out-of-band forged derivation fails re-derivability (b). The
    // record is frozen at birth (round-2 hardening), so the forgery
    // arrives as a planted row, not a rewrite — the walk detects either.
    let (_j, room3) = establish(&store, EnvKind::Student, Tier::Devout, pair.matrix)
        .await
        .expect("third room");
    let job3 = bound_student_job(&store, room3.env_id, pair.matrix).await;
    refine(&store, job3.job_id, room3.env_id, &[pair.node_a])
        .await
        .expect("honest refine");
    plant_artifact(
        &store,
        room3.env_id,
        pair.node_a,
        REFINE_METHOD,
        &"f".repeat(64),
    )
    .await;
    let debris = redundant_consistency(&store, room3.env_id)
        .await
        .expect("walk runs")
        .expect_err("re-derivability must fail");
    assert_eq!(debris.property, 'b', "wrong property: {debris}");
}

/// Plants a derivation record out-of-band (below the store's walls) — the
/// walk, not the persist gate, is the detector the planted row exercises.
async fn plant_artifact(
    store: &PgStore,
    env_ref: Uuid,
    source: Uuid,
    method: &str,
    content_sha: &str,
) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO refined_artifacts
             (artifact_id, env_ref, source_refs, method, content_sha,
              schema_name, schema_version, produced_by)
           VALUES ($1, $2, $3, $4, $5, 'RefinedArtifact', '1.0.0', 'test')"#,
    )
    .bind(id)
    .bind(env_ref)
    .bind(json!([source.to_string()]))
    .bind(method)
    .bind(content_sha)
    .execute(store.raw_pool())
    .await
    .expect("planted row");
    id
}

/// SC-L04 — the unbound are unnamed: a conferral (environment) for a
/// REGULAR tier fails validation, at the API and at the substrate.
#[tokio::test]
async fn sc_l04_unbound_unnamed() {
    let Some(store) = store().await else { return };
    let pair = devout_pair(&store).await;

    // The API wall (X.1), exercised from the Student path.
    let err = establish(&store, EnvKind::Student, Tier::Regular, pair.matrix).await;
    assert!(
        matches!(
            err,
            Err(ScriptoriumError::Store(StoreError::ValidationFailed(_)))
        ),
        "Regulars establish no environment"
    );

    // The substrate wall beneath it: the tier CHECK rejects REGULAR.
    let job = student_job(&store).await;
    let insert = sqlx::query(
        r#"INSERT INTO environments
             (env_id, kind, matrix_ref, tier, title, name, established_by,
              schema_name, schema_version, produced_by)
           VALUES ($1, 'STUDENT', $2, 'REGULAR', 'Br.', 'Unnamed', $3,
                   'EnvironmentRecord', '1.0.0', 'test')"#,
    )
    .bind(Uuid::now_v7())
    .bind(pair.matrix)
    .bind(job.job_id)
    .execute(store.raw_pool())
    .await;
    assert!(insert.is_err(), "the tier CHECK holds at the substrate");
}

/// Regression (slice-9 review, HIGH): a Return rises only from a LIVE room
/// (A.8), only from the job's own room (IX.4), only from a room of the
/// Instruction's target tier (B.1), and only across a pairing (X.5) — and
/// a bound job cannot drop refinement debris into another's room.
#[tokio::test]
async fn return_walls_live_bound_paired_tiered() {
    let Some(store) = store().await else { return };
    let pair = devout_pair(&store).await;
    let instruction = write_instruction(&store, &instruction_draft(pair.teacher_env, pair.node_a))
        .await
        .expect("instruction");

    // An archived room takes no new Return — refused on record.
    let (_j, doomed) = establish(&store, EnvKind::Student, Tier::Devout, pair.matrix)
        .await
        .expect("doomed room");
    store
        .form_pairing(
            pair.teacher_env,
            doomed.env_id,
            pair.matrix,
            PairingKind::DevoutAssignment,
        )
        .await
        .expect("pair doomed");
    store
        .orphan_environment(doomed.env_id)
        .await
        .expect("orphan");
    assert_invalid(
        &store,
        &conforming_return(&instruction, doomed.env_id),
        "archived room",
    )
    .await;

    // An unbound job cannot answer from a room by naming it.
    let unbound = student_job(&store).await;
    let err = store
        .persist_return(
            unbound.job_id,
            &conforming_return(&instruction, pair.student_env),
        )
        .await
        .expect_err("the binding wall");
    assert!(
        err.to_string().contains("not bound"),
        "IX.4 names it: {err}"
    );

    // Nor can a bound job drop refinement debris into another's room.
    let (_j, other) = establish(&store, EnvKind::Student, Tier::Devout, pair.matrix)
        .await
        .expect("other room");
    let bound = bound_student_job(&store, pair.student_env, pair.matrix).await;
    let err = store
        .persist_refined_artifact(
            bound.job_id,
            other.env_id,
            &[pair.node_a],
            REFINE_METHOD,
            &"0".repeat(64),
        )
        .await
        .expect_err("cross-room refinement");
    assert!(
        err.to_string().contains("not bound"),
        "IX.4 names it: {err}"
    );

    // A room of another tier answers a contract that never bound it.
    let (_j, canon) = establish(&store, EnvKind::Student, Tier::Canon, pair.matrix)
        .await
        .expect("canon room");
    assert_invalid(
        &store,
        &conforming_return(&instruction, canon.env_id),
        "the Instruction binds DEVOUT",
    )
    .await;

    // An unpaired room has no standing to answer — the store's wall.
    let bound_other = bound_student_job(&store, other.env_id, pair.matrix).await;
    let err = store
        .persist_return(
            bound_other.job_id,
            &conforming_return(&instruction, other.env_id),
        )
        .await
        .expect_err("the pairing wall");
    assert!(
        err.to_string().contains("no pairing binds"),
        "X.5 names it: {err}"
    );
}

/// Regression (slice-9 review, MEDIUM): the VALIDATE_OUT gates beyond the
/// completion contract — instruction resolution and certification, the
/// §2.4 version range (a compatible minor is lawful; equality was
/// over-refusal), and the Student-room rule.
#[tokio::test]
async fn validate_gates_beyond_the_contract() {
    let Some(store) = store().await else { return };
    let pair = devout_pair(&store).await;
    let instruction = write_instruction(&store, &instruction_draft(pair.teacher_env, pair.node_a))
        .await
        .expect("instruction");

    // The answered Instruction must resolve …
    let mut ghost = conforming_return(&instruction, pair.student_env);
    ghost.instruction_ref = Uuid::now_v7();
    assert_invalid(&store, &ghost, "does not resolve").await;

    // … and be certified: an unflagged Instruction is invisible (§5.1).
    let tjob = running_job(&store, AgentType::Teacher).await;
    let unflagged = store
        .persist_instruction(
            tjob.job_id,
            &instruction_draft(pair.teacher_env, pair.node_a),
        )
        .await
        .expect("unflagged instruction");
    assert_invalid(
        &store,
        &conforming_return(&unflagged, pair.student_env),
        "not flagged",
    )
    .await;

    // Version skew outside the declared range refuses (§2.4) …
    let mut skewed = conforming_return(&instruction, pair.student_env);
    skewed.concordat_version = Version::new(2, 0, 0);
    assert_invalid(&store, &skewed, "outside the Student's supported range").await;

    // … but an additive minor within ^1.0 is lawful, even when the
    // Instruction was written under an earlier one.
    let v110 = Version::new(1, 1, 0);
    if store.get_concordat(&v110).await.is_err() {
        let tables = json!({ "REGULAR": [], "DEVOUT": ["REFINE","VERIFY"], "CANON": ["VERIFY"] });
        let _ = store
            .adopt_concordat("sovereign", &v110, &tables, &json!({}))
            .await;
    }
    let mut minor = conforming_return(&instruction, pair.student_env);
    minor.concordat_version = v110;
    assert!(
        validate_return(&store, &minor)
            .await
            .expect("validate runs")
            .is_ok(),
        "a compatible citation is lawful"
    );

    // A Return rises from a Student's room, not a Teacher's.
    let mut wrong_room = conforming_return(&instruction, pair.student_env);
    wrong_room.student_env_ref = pair.teacher_env;
    assert_invalid(&store, &wrong_room, "names a TEACHER environment").await;
}

/// Regression (slice-9 review, HIGH): the walk names debris, it never
/// crashes on it — a source the store admits by design but cannot re-fold
/// (a link, another artifact) is property (b) debris, not a store fault.
#[tokio::test]
async fn walk_names_debris_never_faults() {
    let Some(store) = store().await else { return };
    let pair = devout_pair(&store).await;
    let link_id = store
        .get_matrix(pair.matrix)
        .await
        .expect("matrix")
        .link_refs[0];
    let job = bound_student_job(&store, pair.student_env, pair.matrix).await;
    let admitted = store
        .persist_refined_artifact(
            job.job_id,
            pair.student_env,
            &[link_id],
            REFINE_METHOD,
            &"a".repeat(64),
        )
        .await
        .expect("the store admits; the walk detects");

    let debris = redundant_consistency(&store, pair.student_env)
        .await
        .expect("walk runs, never crashes")
        .expect_err("named debris");
    assert_eq!(debris.property, 'b', "wrong property: {debris}");
    assert!(
        debris.detail.contains("not a re-derivable node"),
        "named precisely: {debris}"
    );

    // re_derive called directly names it the same way.
    let err = re_derive(&store, admitted.artifact_id).await;
    assert!(
        matches!(err, Err(StudentError::NotRefinable(_))),
        "re-derive names, never faults: {err:?}"
    );
}

/// Regression (slice-9 review, MEDIUM): the walk proves its room before
/// certifying anything; property (a) conformance and the room-closure loop
/// each fire; the room's own products are lawful elections for the walk
/// AND the mount — one closure definition, not two.
#[tokio::test]
async fn walk_covers_conformance_and_the_room() {
    let Some(store) = store().await else { return };
    let pair = devout_pair(&store).await;

    // A walk aimed at nothing is never a clean bill.
    let err = redundant_consistency(&store, Uuid::now_v7()).await;
    assert!(
        matches!(err, Err(StudentError::Store(StoreError::NotFound(_)))),
        "a room that does not resolve: {err:?}"
    );
    let err = redundant_consistency(&store, pair.teacher_env).await;
    assert!(
        matches!(
            err,
            Err(StudentError::Store(StoreError::ValidationFailed(_)))
        ),
        "a Teacher room is no scriptorium: {err:?}"
    );

    // The room's own products are lawful elections.
    let instruction = write_instruction(&store, &instruction_draft(pair.teacher_env, pair.node_a))
        .await
        .expect("instruction");
    let job = bound_student_job(&store, pair.student_env, pair.matrix).await;
    let refined = refine(&store, job.job_id, pair.student_env, &[pair.node_a])
        .await
        .expect("refine");
    let answered = write_return(&store, &conforming_return(&instruction, pair.student_env))
        .await
        .expect("return");
    let writer_job: Uuid = answered
        .envelope
        .produced_by
        .parse()
        .expect("writer job id");
    for item in [refined.artifact_id, answered.return_id, writer_job] {
        store
            .add_env_item(
                job.job_id,
                pair.student_env,
                item,
                &valid_chain(pair.node_a),
                true,
            )
            .await
            .expect("election");
    }
    assert!(
        redundant_consistency(&store, pair.student_env)
            .await
            .expect("walk runs")
            .is_ok(),
        "the room's own products are not debris"
    );
    mount(
        &store,
        EnvKind::Student,
        Tier::Devout,
        pair.matrix,
        pair.student_env,
    )
    .await
    .expect("the room mounts with its products elected (IX.3)");

    // Property (a): a malformed derivation record is conformance debris.
    // Records are frozen at birth (round-2 hardening), so the malformed
    // one is planted in its own room, out-of-band.
    let (_j, room_a) = establish(&store, EnvKind::Student, Tier::Devout, pair.matrix)
        .await
        .expect("conformance room");
    plant_artifact(&store, room_a.env_id, pair.node_a, "", &"0".repeat(64)).await;
    let debris = redundant_consistency(&store, room_a.env_id)
        .await
        .expect("walk runs")
        .expect_err("conformance must fail");
    assert_eq!(debris.property, 'a', "wrong property: {debris}");

    // Property (c) over the room: a dangling election is named debris.
    store
        .add_env_item(
            job.job_id,
            pair.student_env,
            Uuid::now_v7(),
            &valid_chain(pair.node_a),
            false,
        )
        .await
        .expect("dangling election");
    let debris = redundant_consistency(&store, pair.student_env)
        .await
        .expect("walk runs")
        .expect_err("room closure must fail");
    assert_eq!(debris.property, 'c', "wrong property: {debris}");
    assert!(
        debris.detail.contains("elected item"),
        "named precisely: {debris}"
    );
}

/// Regression (review round 2, HIGH): certification is the trust boundary.
/// Only the RUNNING Student job bound to the Return's own room flags it; a
/// replay mints no second RETURN_FLAGGED event; and the substrate freezes
/// the flagged record whole — provenance envelope and derivation records
/// included.
#[tokio::test]
async fn certification_walls_and_single_flag_event() {
    let Some(store) = store().await else { return };
    let pair = devout_pair(&store).await;
    let instruction = write_instruction(&store, &instruction_draft(pair.teacher_env, pair.node_a))
        .await
        .expect("instruction");
    let draft = conforming_return(&instruction, pair.student_env);
    let job = bound_student_job(&store, pair.student_env, pair.matrix).await;
    let manifest = store
        .persist_return(job.job_id, &draft)
        .await
        .expect("persist");

    // A Teacher's job cannot certify.
    let teacher = running_job(&store, AgentType::Teacher).await;
    let err = store.flag_return(teacher.job_id, manifest.return_id).await;
    assert!(
        matches!(err, Err(StoreError::ValidationFailed(_))),
        "only a Student certifies: {err:?}"
    );

    // An unbound Student job cannot certify another room's Return.
    let unbound = student_job(&store).await;
    let err = store.flag_return(unbound.job_id, manifest.return_id).await;
    assert!(
        matches!(err, Err(StoreError::ValidationFailed(_))),
        "certification rises from the bound room: {err:?}"
    );

    // The bound job certifies; the log testifies exactly once.
    let flag_events = |logs: &[godhead_schemas::LogSnapshot]| {
        logs.iter()
            .filter(|l| l.event == LogEvent::ReturnFlagged)
            .count()
    };
    let flagged = store
        .flag_return(job.job_id, manifest.return_id)
        .await
        .expect("flag");
    assert!(flagged.flagged);
    let logs = store
        .read_logs(&manifest.return_id.to_string())
        .await
        .expect("logs");
    assert_eq!(flag_events(&logs), 1);

    // Replay: an idempotent read-back, never a second certification event.
    let again = store
        .flag_return(job.job_id, manifest.return_id)
        .await
        .expect("replay reads back");
    assert!(again.flagged);
    let logs = store
        .read_logs(&manifest.return_id.to_string())
        .await
        .expect("logs");
    assert_eq!(flag_events(&logs), 1, "one certification, one event");

    // The envelope is frozen too: who certified, and when, cannot be
    // falsified after the flag (round-2 hardening, migration 0012).
    let edit = sqlx::query("UPDATE returns SET produced_by = 'forged' WHERE return_id = $1")
        .bind(manifest.return_id)
        .execute(store.raw_pool())
        .await;
    assert!(edit.is_err(), "the envelope of a flagged Return is frozen");

    // A derivation record is frozen at birth.
    let artifact = refine(&store, job.job_id, pair.student_env, &[pair.node_a])
        .await
        .expect("refine");
    let edit = sqlx::query("UPDATE refined_artifacts SET content_sha = $2 WHERE artifact_id = $1")
        .bind(artifact.artifact_id)
        .bind("0".repeat(64))
        .execute(store.raw_pool())
        .await;
    assert!(edit.is_err(), "a derivation record is immutable");
}

/// Regression (review round 2, HIGH): no job strands live. A Return from
/// an unpaired room passes VALIDATE_OUT (the bridge is the store's wall to
/// prove) and halts at persist — the job ends REFUSED with a stage-naming
/// detail, never stranded RUNNING.
#[tokio::test]
async fn mid_labor_halt_refuses_never_strands() {
    let Some(store) = store().await else { return };
    let pair = devout_pair(&store).await;
    let instruction = write_instruction(&store, &instruction_draft(pair.teacher_env, pair.node_a))
        .await
        .expect("instruction");

    // A live, correctly-tiered Student room on the same matrix — unpaired.
    let (_j, lone) = establish(&store, EnvKind::Student, Tier::Devout, pair.matrix)
        .await
        .expect("lone room");
    let draft = conforming_return(&instruction, lone.env_id);
    assert!(
        validate_return(&store, &draft)
            .await
            .expect("validate runs")
            .is_ok(),
        "the bridge is not this end's to prove"
    );
    let err = write_return(&store, &draft).await;
    assert!(
        matches!(
            err,
            Err(StudentError::Store(StoreError::ValidationFailed(_)))
        ),
        "the wall at persist holds: {err:?}"
    );

    // The laborer did not strand: REFUSED, the stage on record.
    let jobs: Vec<(Uuid, String)> =
        sqlx::query_as("SELECT job_id, status FROM job_records WHERE env_ref = $1")
            .bind(lone.env_id)
            .fetch_all(store.raw_pool())
            .await
            .expect("jobs");
    assert!(!jobs.is_empty(), "the labor got its laborer");
    for (job_id, status) in &jobs {
        assert_eq!(status, "REFUSED", "job {job_id} must not strand live");
    }
    let details: Vec<(String,)> =
        sqlx::query_as("SELECT detail FROM refusal_records WHERE job_id = $1")
            .bind(jobs[0].0)
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

/// Regression (review round 2, HIGH): a hostile draft cannot suppress its
/// own refusal. A secret-shaped semver prerelease rides the version string
/// into the ephemeral failure detail — the PERSISTED detail names the
/// clause only, so the Law XV scan passes and the record lands.
#[tokio::test]
async fn refusal_never_echoes_the_draft() {
    let Some(store) = store().await else { return };
    let pair = devout_pair(&store).await;
    let instruction = write_instruction(&store, &instruction_draft(pair.teacher_env, pair.node_a))
        .await
        .expect("instruction");

    let mut hostile = conforming_return(&instruction, pair.student_env);
    hostile.concordat_version = Version::parse("1.0.0-AKIA0123456789ABCDEF").expect("valid semver");
    let err = write_return(&store, &hostile).await;
    assert!(
        matches!(err, Err(StudentError::ReturnInvalid(_))),
        "the hostile draft refuses: {err:?}"
    );

    // The refusal record LANDED — clause-referencing, echo-free.
    let details: Vec<(String,)> = sqlx::query_as(
        r#"SELECT r.detail FROM refusal_records r
             JOIN job_records j ON j.job_id = r.job_id
           WHERE j.env_ref = $1"#,
    )
    .bind(pair.student_env)
    .fetch_all(store.raw_pool())
    .await
    .expect("refusals");
    assert_eq!(details.len(), 1, "the refusal is on record");
    assert!(
        details[0].0.contains("concordat-skew"),
        "the clause is named: {}",
        details[0].0
    );
    assert!(
        !details[0].0.contains("AKIA"),
        "the emission is never echoed: {}",
        details[0].0
    );

    // A version inside the range but never adopted is refused too — a
    // certified record must cite a retrievable Concordat (SC-K03).
    let mut unadopted = conforming_return(&instruction, pair.student_env);
    unadopted.concordat_version = Version::new(1, 987_654, 0);
    assert_invalid(&store, &unadopted, "never adopted").await;
}
