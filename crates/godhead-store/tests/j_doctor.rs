//! Slice 11b — the Doctor & the dissolution cascade (SLICE_11B.md §1).
//! SC-J08 comes home: deploy-requires-LIVE, dissolve-orphans-through-the-
//! pairing (both levers — decommission→ORPHANED and retire→DISSOLVED), and
//! no-silent-revival. Plus retire_environment's human-reserved wall (IV.4).
//!
//! Closes SLICE_07's deferred ORPHANED cascade on the record (§0.6): the
//! decommission leg drives execute_decommission end-to-end and proves the
//! matrix's rooms — Student and Doctor alike — go ORPHANED.

mod common;

use godhead_schemas::{
    AgentType, Budgets, EnvKind, EnvStatus, JobDraft, JobRecord, JobStatus, Tier,
};
use godhead_store::{PgStore, Store, StoreError};
use semver::Version;
use uuid::Uuid;

// ---- fixtures ----

/// A fixture job draft with an hour of wall budget.
fn wide_draft(agent_type: AgentType, tier: Option<Tier>) -> JobDraft {
    JobDraft {
        agent_type,
        auditor_name: None,
        tier,
        input_refs: vec![],
        env_ref: None,
        brief_ref: None,
        endpoint_alias: None,
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 3_600_000,
            max_tool_calls: 10,
            max_tokens: 100_000,
        },
    }
}

/// PENDING → LEASED → RUNNING, the lawful spawn (Law I.1).
async fn spawn_running(store: &PgStore, draft: &JobDraft) -> JobRecord {
    let job = store.create_job(draft).await.expect("create");
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await
        .expect("to LEASED");
    store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await
        .expect("to RUNNING")
}

/// Plants a POSTULANT matrix raw (matrices are born POSTULANT), stamped with
/// a live job's identity so the provenance view resolves it.
async fn planted_matrix(store: &PgStore, job: &JobRecord) -> Uuid {
    let matrix_id = Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO matrices
             (matrix_id, category, node_refs, link_refs, emerged_by, config_rev,
              schema_name, schema_version, produced_by)
           VALUES ($1, $2, '[]', '[]', $3, 1, 'MatrixRecord', '1.0.0', $3::text)"#,
    )
    .bind(matrix_id)
    .bind(format!("j_doctor_{}", Uuid::now_v7()))
    .bind(job.job_id)
    .execute(store.raw_pool())
    .await
    .expect("planted matrix");
    matrix_id
}

/// A LIVE Canon Student on a fresh matrix — the Doctor's charge.
async fn canon_student(store: &PgStore, job: &JobRecord, matrix: Uuid) -> Uuid {
    store
        .establish_environment(job.job_id, EnvKind::Student, matrix, Tier::Canon)
        .await
        .expect("canon student")
        .env_id
}

/// Plants a GRANTED consent for a matrix's decommission. The full minting
/// path (audit → proposal → Notary) is exercised by the audit and notary
/// suites; here the granted consent is a fixture, not the object under test.
/// consent_records is sovereign-reserved (ruling G10), so the fixture stands
/// on the store's own lawful surface — the sovereign class SET LOCAL on this
/// transaction, exactly as set_config/resolve_proposal do — rather than
/// forging a raw row the wall (correctly) refuses.
async fn plant_granted_consent(store: &PgStore, matrix_id: Uuid) -> Uuid {
    let consent_id = Uuid::now_v7();
    let mut tx = store.raw_pool().begin().await.expect("consent tx");
    sqlx::query("SET LOCAL godhead.actor_class = 'sovereign'")
        .execute(&mut *tx)
        .await
        .expect("authenticate the sovereign class");
    sqlx::query(
        r#"INSERT INTO consent_records
             (consent_id, subject_ref, decision, scope, decided_by,
              schema_name, schema_version, produced_by)
           VALUES ($1, $2, 'GRANTED', 'ITEM', 'sovereign',
                   'ConsentRecord', '1.0.0', 'sovereign')"#,
    )
    .bind(consent_id)
    .bind(matrix_id)
    .execute(&mut *tx)
    .await
    .expect("planted granted consent");
    tx.commit().await.expect("commit consent");
    consent_id
}

async fn env_status(store: &PgStore, env_id: Uuid) -> EnvStatus {
    store.get_environment(env_id).await.expect("env").status
}

// ---- SC-J08 leg (a): deploy requires a LIVE Canon Student ----

/// The Doctor deploys over a LIVE Canon Student and no other. It shares the
/// Student's matrix (the pairing binds one shared node, X.5/IX.5); a Devout
/// student, an orphaned student, and a Teacher room each refuse ENV_INVALID.
#[tokio::test]
async fn sc_j08_deploy_requires_live() {
    let Some(store) = common::store().await else {
        return;
    };
    let job = spawn_running(&store, &wide_draft(AgentType::Teacher, Some(Tier::Canon))).await;
    let matrix = planted_matrix(&store, &job).await;

    // The lawful deploy over a LIVE Canon Student.
    let student = canon_student(&store, &job, matrix).await;
    let dep = store
        .deploy_doctor(job.job_id, student)
        .await
        .expect("deploy over a LIVE Canon Student");
    assert_eq!(dep.student_env_ref, student);
    let doctor = store
        .get_environment(dep.doctor_env_ref)
        .await
        .expect("doctor");
    assert_eq!(doctor.kind, EnvKind::Teacher, "a Doctor is a Canon Teacher");
    assert_eq!(doctor.tier, Tier::Canon);
    assert_eq!(doctor.status, EnvStatus::Live);
    assert_eq!(
        doctor.matrix_ref, matrix,
        "the Doctor stands on the Student's matrix (X.5)"
    );
    // Both instruments recorded: a CANONICAL_INSTRUCTION pairing over the node.
    let pairing_kind: String =
        sqlx::query_scalar("SELECT kind FROM pairings WHERE pairing_id = $1")
            .bind(dep.pairing_id)
            .fetch_one(store.raw_pool())
            .await
            .expect("the pairing exists");
    assert_eq!(pairing_kind, "CANONICAL_INSTRUCTION");

    // A Devout student is not a Doctor's charge.
    let devout = store
        .establish_environment(job.job_id, EnvKind::Student, matrix, Tier::Devout)
        .await
        .expect("devout student");
    let err = store
        .deploy_doctor(job.job_id, devout.env_id)
        .await
        .expect_err("a Devout student is not Canon");
    assert!(matches!(err, StoreError::EnvInvalid(_)), "got {err}");

    // An orphaned Canon Student is not LIVE.
    let orphaned = canon_student(&store, &job, matrix).await;
    store.orphan_environment(orphaned).await.expect("orphan");
    let err = store
        .deploy_doctor(job.job_id, orphaned)
        .await
        .expect_err("an orphaned student is not a workplace");
    assert!(matches!(err, StoreError::EnvInvalid(_)), "got {err}");

    // A Teacher room is not a Student.
    let teacher = store
        .establish_environment(job.job_id, EnvKind::Teacher, matrix, Tier::Canon)
        .await
        .expect("canon teacher");
    let err = store
        .deploy_doctor(job.job_id, teacher.env_id)
        .await
        .expect_err("a Teacher is not a Doctor's charge");
    assert!(matches!(err, StoreError::EnvInvalid(_)), "got {err}");
}

// ---- SC-J08 leg (b): a Student leaving LIVE orphans its Doctor ----

/// One substrate wall, both levers. When the Canon Student's room leaves LIVE
/// — ORPHANED (its matrix decommissioned) or DISSOLVED (retired) — the Doctor
/// bound to it goes ORPHANED. The decommission lever drives
/// execute_decommission end-to-end, closing SLICE_07's deferred cascade.
#[tokio::test]
async fn sc_j08_leaving_live_orphans_doctor() {
    let Some(store) = common::store().await else {
        return;
    };

    // Lever 1 — decommission → ORPHANED (the store cascade + the trigger).
    let job = spawn_running(&store, &wide_draft(AgentType::Teacher, Some(Tier::Canon))).await;
    let m1 = planted_matrix(&store, &job).await;
    let s1 = canon_student(&store, &job, m1).await;
    let d1 = store.deploy_doctor(job.job_id, s1).await.expect("deploy 1");

    let notary = spawn_running(&store, &wide_draft(AgentType::Notary, None)).await;
    let consent = plant_granted_consent(&store, m1).await;
    store
        .execute_decommission(notary.job_id, m1, consent)
        .await
        .expect("the Notary executes the decommission");
    assert_eq!(
        env_status(&store, s1).await,
        EnvStatus::Orphaned,
        "the matrix's Student room orphaned (SLICE_07 cascade, closed)"
    );
    assert_eq!(
        env_status(&store, d1.doctor_env_ref).await,
        EnvStatus::Orphaned,
        "the Doctor orphaned with its Student — decommission lever"
    );

    // Lever 2 — retire → DISSOLVED (the trigger, same wall).
    let m2 = planted_matrix(&store, &job).await;
    let s2 = canon_student(&store, &job, m2).await;
    let d2 = store.deploy_doctor(job.job_id, s2).await.expect("deploy 2");
    let retired = store
        .retire_environment("sovereign", s2)
        .await
        .expect("the sovereign retires the Student");
    assert_eq!(retired.status, EnvStatus::Dissolved);
    assert_eq!(
        env_status(&store, d2.doctor_env_ref).await,
        EnvStatus::Orphaned,
        "the Doctor orphaned when its Student dissolved — retire lever"
    );
}

// ---- SC-J08 leg (c): no silent revival ----

/// The status arc only descends, and no fresh deployment adopts an orphaned
/// Doctor. Both halves the criterion pins as behavior (§0.5): (1) an ORPHANED
/// room never flips back to LIVE and a DISSOLVED room never returns — refused
/// beneath the store's API, at the substrate, so no path (store or raw)
/// revives a retired dependent; (2) deploying against a NEW Canon Student
/// mints a NEW Doctor (new env, new reference, new pairing) and leaves the old
/// one orphaned — doc 06:90's "pairs anew," never a status flip on the old.
#[tokio::test]
async fn sc_j08_no_silent_revival() {
    let Some(store) = common::store().await else {
        return;
    };
    let job = spawn_running(&store, &wide_draft(AgentType::Teacher, Some(Tier::Canon))).await;
    let matrix = planted_matrix(&store, &job).await;
    let student = canon_student(&store, &job, matrix).await;
    let dep = store
        .deploy_doctor(job.job_id, student)
        .await
        .expect("deploy");

    // The Doctor orphans with its Student.
    store
        .orphan_environment(student)
        .await
        .expect("orphan student");
    let doctor = dep.doctor_env_ref;
    assert_eq!(env_status(&store, doctor).await, EnvStatus::Orphaned);

    // ORPHANED → LIVE is refused at the substrate — a raw flip cannot revive it.
    let err = sqlx::query("UPDATE environments SET status = 'LIVE' WHERE env_id = $1")
        .bind(doctor)
        .execute(store.raw_pool())
        .await
        .expect_err("no silent revival");
    assert!(
        err.to_string().contains("no silent revival"),
        "the substrate refuses ORPHANED → LIVE; got {err}"
    );
    assert_eq!(env_status(&store, doctor).await, EnvStatus::Orphaned);

    // The fresh Student does not adopt the old Doctor (§0.5(2); doc 06:90's
    // "pairs anew"). A new deployment mints a NEW Doctor — new env, new
    // reference, new pairing — and the orphaned one is untouched.
    let fresh_matrix = planted_matrix(&store, &job).await;
    let fresh_student = canon_student(&store, &job, fresh_matrix).await;
    let fresh = store
        .deploy_doctor(job.job_id, fresh_student)
        .await
        .expect("deploy over a fresh Canon Student");
    assert_ne!(
        fresh.doctor_env_ref, doctor,
        "a fresh deployment mints a new Doctor env, never the old"
    );
    assert_ne!(
        fresh.deployment_id, dep.deployment_id,
        "a new deployment reference"
    );
    assert_ne!(fresh.pairing_id, dep.pairing_id, "a new pairing");
    assert_eq!(
        env_status(&store, doctor).await,
        EnvStatus::Orphaned,
        "the fresh deployment left the old Doctor orphaned — no adoption, no revival"
    );
    assert_eq!(
        env_status(&store, fresh.doctor_env_ref).await,
        EnvStatus::Live,
        "the fresh Doctor stands LIVE, its own room"
    );

    // Retire the orphaned Doctor (ORPHANED → DISSOLVED, a lawful descent).
    let dissolved = store
        .retire_environment("sovereign", doctor)
        .await
        .expect("retire the orphaned Doctor");
    assert_eq!(dissolved.status, EnvStatus::Dissolved);

    // And DISSOLVED never returns — not even to ORPHANED.
    let err = sqlx::query("UPDATE environments SET status = 'ORPHANED' WHERE env_id = $1")
        .bind(doctor)
        .execute(store.raw_pool())
        .await
        .expect_err("a struck room does not return");
    assert!(
        err.to_string().contains("does not return"),
        "the substrate refuses any exit from DISSOLVED; got {err}"
    );
}

// ---- retire_environment is human-reserved (IV.4) ----

/// DISSOLVED's one lever is a HUMAN act. A job-uuid actor is a gate bypass,
/// refused at the substrate exactly as the agent-author wall refuses one on
/// the other reserved tables; the room is untouched. A human hand retires it,
/// and the act is idempotent.
#[tokio::test]
async fn sc_j08_retire_is_human_reserved() {
    let Some(store) = common::store().await else {
        return;
    };
    let job = spawn_running(&store, &wide_draft(AgentType::Teacher, Some(Tier::Canon))).await;
    let matrix = planted_matrix(&store, &job).await;
    let env = canon_student(&store, &job, matrix).await;

    // A job-uuid actor — a gate bypass (IV.4). Refused; the room stays LIVE.
    let err = store
        .retire_environment(&job.job_id.to_string(), env)
        .await
        .expect_err("an agent-shaped retire is a gate bypass");
    assert!(
        err.to_string().contains("GATE_BYPASS"),
        "a job-uuid retire is refused at the substrate; got {err}"
    );
    assert_eq!(
        env_status(&store, env).await,
        EnvStatus::Live,
        "the refused retire changed nothing"
    );

    // A human hand retires it: LIVE → DISSOLVED.
    let dissolved = store
        .retire_environment("sovereign", env)
        .await
        .expect("the sovereign retires the room");
    assert_eq!(dissolved.status, EnvStatus::Dissolved);
    let retired_by: Option<String> =
        sqlx::query_scalar("SELECT retired_by FROM environments WHERE env_id = $1")
            .bind(env)
            .fetch_one(store.raw_pool())
            .await
            .expect("retired_by");
    assert_eq!(retired_by.as_deref(), Some("sovereign"));

    // Idempotent — a struck room is already gone (A.8).
    let again = store
        .retire_environment("sovereign", env)
        .await
        .expect("idempotent retire");
    assert_eq!(again.status, EnvStatus::Dissolved);
}
