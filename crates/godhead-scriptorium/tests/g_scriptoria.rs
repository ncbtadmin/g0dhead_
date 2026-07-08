//! Section G — Scriptoria & Titles (Laws IX–X). SC-G01 … SC-G07.

use godhead_intake::{Dispatcher, IntakePipe};
use godhead_ml::{aggregate, slave, LexicalEmbedder, Roster};
use godhead_schemas::{
    roster_index, AgentType, Budgets, ConfigTier, EnvKind, EnvStatus, JobDraft, JobRecord,
    JobStatus, PairingKind, Tier,
};
use godhead_scriptorium::{establish, mount};
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
        eprintln!("SKIP: DATABASE_URL unset — database-backed criterion NOT exercised");
        return None;
    };
    let mut reg = godhead_intake::registry();
    godhead_ml::register_into(&mut reg);
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

/// Grows a Postulant the ordinary way and returns its id and a node id in
/// it (for use as a resolvable item_ref).
async fn grown_matrix(store: &PgStore) -> (Uuid, [Uuid; 2]) {
    ensure_threshold(store).await;
    let pipe = IntakePipe::new(store, temp_root()).expect("pipe");
    let text = b"the joins are true and the cathedral stands\n";
    let a = commit_to_rest(&pipe, "a.md", text).await;
    let b = commit_to_rest(&pipe, "b.md", text).await;
    let scope = [a, b];
    let category = format!("scriptorium_{}", Uuid::now_v7());
    slave::backfill_tick(store, &lexical_roster(), pipe.data_root(), Some(&scope))
        .await
        .expect("backfill");
    let summary = aggregate::consolidate(store, &lexical_roster(), &category, &scope)
        .await
        .expect("consolidate");
    (summary.emerged.expect("emergence"), scope)
}

async fn commit_to_rest(pipe: &IntakePipe<'_, PgStore>, filename: &str, bytes: &[u8]) -> Uuid {
    let node_id = pipe.commit_file(filename, bytes).await.expect("commit");
    let dispatcher = Dispatcher::new(pipe);
    let scope = [node_id];
    dispatcher.tick_scoped(Some(&scope)).await.expect("tick 1");
    dispatcher.tick_scoped(Some(&scope)).await.expect("tick 2");
    node_id
}

/// A live agent job to read/curate under. When `binding` is Some
/// `(env, matrix)`, the job is born bound to that environment — the store
/// authenticates the binding, so the tier must match the env and the
/// matrix rides in input_refs (IX.4).
async fn agent_job(store: &PgStore, tier: Tier, binding: Option<(Uuid, Uuid)>) -> JobRecord {
    let (env_ref, input_refs) = match binding {
        Some((env, matrix)) => (Some(env), vec![matrix]),
        None => (None, vec![]),
    };
    let draft = JobDraft {
        agent_type: AgentType::Student,
        auditor_name: None,
        tier: Some(tier),
        input_refs,
        env_ref,
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

/// The Roman-ordinal suffix of a conferred name as an integer (1 if bare).
fn ordinal_of(name: &str) -> u32 {
    let Some((_, suffix)) = name.rsplit_once(' ') else {
        return 1;
    };
    if !suffix.chars().all(|c| "IVXLCDM".contains(c)) || suffix.is_empty() {
        return 1;
    }
    let mut total = 0u32;
    let mut prev = 0u32;
    for c in suffix.chars().rev() {
        let v = match c {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            'C' => 100,
            'D' => 500,
            'M' => 1000,
            _ => 0,
        };
        if v < prev {
            total -= v;
        } else {
            total += v;
            prev = v;
        }
    }
    total
}

/// A valid provenance chain rooted in a BRIEF, producing a live node.
fn valid_chain(seq_root_produces: Uuid) -> serde_json::Value {
    json!([
        { "link_seq": 0, "kind": "BRIEF", "actor": Uuid::now_v7().to_string(),
          "prompt_or_reason": "the sovereign's charge", "produced": [seq_root_produces.to_string()] },
        { "link_seq": 1, "kind": "REFINEMENT", "actor": Uuid::now_v7().to_string(),
          "prompt_or_reason": "refined into the room", "produced": [seq_root_produces.to_string()] },
    ])
}

/// SC-G04 — conferral is deterministic and reproducible; a living collision
/// takes the ordinal.
#[tokio::test]
async fn sc_g04_conferral() {
    let Some(store) = store().await else { return };
    let (matrix, _scope) = grown_matrix(&store).await;
    let (_job, env) = establish(&store, EnvKind::Teacher, Tier::Devout, matrix)
        .await
        .expect("establish");
    assert_eq!(env.title, "Professor", "Devout Teacher → Professor (X.2)");

    // Reproduce the base name from the record alone (X.4).
    let roster = store.get_config("name_roster").await.expect("roster");
    let names = roster.value.as_array().expect("array");
    let base = names[roster_index(env.env_id, names.len())]
        .as_str()
        .expect("str");
    assert!(
        env.name.starts_with(base),
        "the name reproduces from hash(env_id): {} starts with {base}",
        env.name
    );

    // A second Devout Teacher on the same matrix cannot re-emerge a matrix,
    // so use a fresh matrix; force a name collision by seeding the roster
    // to a single entry, guaranteeing both envs draw the same base.
    // (Deterministic: two living envs sharing a base — the second ordinals.)
    // Simpler: assert the ordinal machinery directly via two Student envs
    // in the same matrix category are disallowed; instead rely on the
    // living-collision count. We construct two environments whose base
    // collides by using the same roster index is not guaranteed; so we
    // check the recorded name is either bare or carries a Roman ordinal.
    let bare_or_ordinal = env.name == base
        || env
            .name
            .strip_prefix(&format!("{base} "))
            .is_some_and(|o| o.chars().all(|c| "IVXLCDM".contains(c)));
    assert!(
        bare_or_ordinal,
        "name is the base or base + Roman ordinal: {}",
        env.name
    );
}

/// SC-G04 (ordinal) — two living environments hashing to the same roster
/// base: the second takes the ordinal, recorded immutably.
#[tokio::test]
async fn sc_g04_ordinal_on_collision() {
    let Some(store) = store().await else { return };
    // Seed a single-entry roster so every env draws the same base name.
    let single = json!(["Solo"]);
    loop {
        let c = store.get_config("name_roster").await.expect("roster");
        match store
            .set_config(
                "test-harness",
                "name_roster",
                ConfigTier::Operational,
                &single,
                Some(c.revision),
            )
            .await
        {
            Ok(_) => break,
            Err(StoreError::StaleRevision { .. }) => continue,
            Err(e) => panic!("seed roster: {e}"),
        }
    }
    let (m1, _) = grown_matrix(&store).await;
    let (m2, _) = grown_matrix(&store).await;
    let (_j1, e1) = establish(&store, EnvKind::Teacher, Tier::Devout, m1)
        .await
        .expect("first");
    let (_j2, e2) = establish(&store, EnvKind::Teacher, Tier::Devout, m2)
        .await
        .expect("second");
    // Both draw the base "Solo"; a later living bearer takes a strictly
    // higher ordinal. (The absolute number floats — the test DB is shared
    // and other tests establish envs in parallel — so we assert the
    // monotonic collision rule, not a fixed value.)
    assert!(
        e1.name.starts_with("Solo"),
        "first draws the base: {}",
        e1.name
    );
    assert!(
        e2.name.starts_with("Solo"),
        "second draws the base: {}",
        e2.name
    );
    assert!(
        ordinal_of(&e2.name) > ordinal_of(&e1.name),
        "the second living bearer takes a higher ordinal: {} then {}",
        e1.name,
        e2.name
    );
}

/// SC-G01 — mounting a floor-invalid environment refuses ENV_INVALID; and
/// a well-formed room mounts.
#[tokio::test]
async fn sc_g01_mount_invalid() {
    let Some(store) = store().await else { return };
    let (matrix, scope) = grown_matrix(&store).await;
    let (job, env) = establish(&store, EnvKind::Teacher, Tier::Devout, matrix)
        .await
        .expect("establish");

    // A well-formed room (one resolvable item, valid chain) mounts.
    store
        .add_env_item(
            job.job_id,
            env.env_id,
            scope[0],
            &valid_chain(scope[0]),
            false,
        )
        .await
        .expect("election");
    mount(&store, EnvKind::Teacher, Tier::Devout, matrix, env.env_id)
        .await
        .expect("a well-formed room mounts");

    // A dangling item ref → ENV_INVALID (whole room).
    let (matrix2, scope2) = grown_matrix(&store).await;
    let (job2, env2) = establish(&store, EnvKind::Teacher, Tier::Devout, matrix2)
        .await
        .expect("establish");
    let ghost = Uuid::now_v7();
    store
        .add_env_item(
            job2.job_id,
            env2.env_id,
            ghost,
            &valid_chain(scope2[0]),
            false,
        )
        .await
        .expect("add dangling");
    let err = mount(&store, EnvKind::Teacher, Tier::Devout, matrix2, env2.env_id).await;
    assert!(
        matches!(
            err,
            Err(godhead_scriptorium::ScriptoriumError::Store(
                StoreError::EnvInvalid(_)
            ))
        ),
        "a dangling item invalidates the whole room"
    );
}

/// SC-G06 — the mount walks provenance: a chain that does not root in a
/// human hand renders the whole room ENV_INVALID.
#[tokio::test]
async fn sc_g06_provenance_walk() {
    let Some(store) = store().await else { return };
    let (matrix, scope) = grown_matrix(&store).await;
    let (job, env) = establish(&store, EnvKind::Teacher, Tier::Devout, matrix)
        .await
        .expect("establish");
    // A chain rooted in FETCH, not CANON|WRIT|BRIEF — no human hand.
    let rootless = json!([
        { "link_seq": 0, "kind": "FETCH", "actor": Uuid::now_v7().to_string(),
          "prompt_or_reason": "no human root", "produced": [scope[0].to_string()] },
    ]);
    store
        .add_env_item(job.job_id, env.env_id, scope[0], &rootless, false)
        .await
        .expect("add rootless");
    let err = mount(&store, EnvKind::Teacher, Tier::Devout, matrix, env.env_id).await;
    assert!(
        matches!(
            err,
            Err(godhead_scriptorium::ScriptoriumError::Store(
                StoreError::EnvInvalid(_)
            ))
        ),
        "a chain without a human root is an invalid room"
    );
}

/// SC-G05 — a Teacher env whose tier/title disagree fails; a pairing naming
/// REGULAR fails.
#[tokio::test]
async fn sc_g05_tier_title() {
    let Some(store) = store().await else { return };
    let (matrix, _scope) = grown_matrix(&store).await;
    let establisher = agent_job(&store, Tier::Devout, None).await;
    // Forge a disagreeing record directly: a Devout Teacher env titled
    // "Doctor". The conferral is immutable (X.1), so an established row
    // cannot be UPDATEd into disagreement — we insert a forged row and
    // mount it, which is the path SC-G05 tests.
    let forged = Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO environments
             (env_id, kind, matrix_ref, tier, title, name, established_by,
              schema_name, schema_version, produced_by)
           VALUES ($1, 'TEACHER', $2, 'DEVOUT', 'Doctor', 'Forged', $3,
                   'EnvironmentRecord', '1.0.0', $3::text)"#,
    )
    .bind(forged)
    .bind(matrix)
    .bind(establisher.job_id)
    .execute(store.raw_pool())
    .await
    .expect("forge");
    let err = mount(&store, EnvKind::Teacher, Tier::Devout, matrix, forged).await;
    assert!(
        matches!(
            err,
            Err(godhead_scriptorium::ScriptoriumError::Store(
                StoreError::EnvInvalid(_)
            ))
        ),
        "tier and title must agree (X.2)"
    );

    // A pairing needs a matching Student; REGULAR establishes none, so the
    // establishment path itself refuses REGULAR (the mechanical guard).
    let job = agent_job(&store, Tier::Regular, None).await;
    let err = store
        .establish_environment(job.job_id, EnvKind::Teacher, matrix, Tier::Regular)
        .await;
    assert!(
        matches!(err, Err(StoreError::ValidationFailed(_))),
        "Regulars establish no environment (X.1)"
    );
}

/// SC-G02 — an out-of-index read is rejected and logged; the allowlist
/// (own job, own lease) is excepted.
#[tokio::test]
async fn sc_g02_scoping_wall() {
    let Some(store) = store().await else { return };
    let (matrix, scope) = grown_matrix(&store).await;
    let (job, env) = establish(&store, EnvKind::Student, Tier::Devout, matrix)
        .await
        .expect("establish");
    store
        .add_env_item(
            job.job_id,
            env.env_id,
            scope[0],
            &valid_chain(scope[0]),
            false,
        )
        .await
        .expect("election");
    // A reader mounted into this room (bound: env_ref = env, matrix in inputs).
    let reader = agent_job(&store, Tier::Devout, Some((env.env_id, matrix))).await;

    // In-index → permitted.
    store
        .env_scoped_read(reader.job_id, env.env_id, scope[0])
        .await
        .expect("in-index read permitted");
    // Out-of-index (the other node, never elected) → rejected + logged.
    let err = store
        .env_scoped_read(reader.job_id, env.env_id, scope[1])
        .await
        .expect_err("out-of-index read rejected");
    assert!(matches!(err, StoreError::Forbidden(_)), "got {err}");
    let logs = store
        .read_logs(&env.env_id.to_string())
        .await
        .expect("logs");
    assert!(
        logs.iter()
            .any(|l| l.event == godhead_schemas::LogEvent::Violation),
        "the out-of-scope read is logged"
    );
    // Allowlist: the reader's own job record.
    store
        .env_scoped_read(reader.job_id, env.env_id, reader.job_id)
        .await
        .expect("own job is allowlisted");
    // And its own lease.
    let lease = store
        .acquire_lease(reader.job_id, Uuid::now_v7(), 60_000)
        .await
        .expect("lease");
    store
        .env_scoped_read(reader.job_id, env.env_id, lease.lease_id)
        .await
        .expect("own lease is allowlisted");
}

/// SC-G03 — the Pairing Exception grants exactly flagged handoff artifacts:
/// a paired agent reads the counterpart's flagged artifact, not its
/// unflagged draft; an unpaired agent is rejected for both.
#[tokio::test]
async fn sc_g03_pairing_exception() {
    let Some(store) = store().await else { return };
    let (matrix, scope) = grown_matrix(&store).await;
    let (t_job, teacher_env) = establish(&store, EnvKind::Teacher, Tier::Devout, matrix)
        .await
        .expect("teacher env");
    let (_s_job, student_env) = establish(&store, EnvKind::Student, Tier::Devout, matrix)
        .await
        .expect("student env");
    // The Student that reads is a fresh agent mounted into its room.
    let student = agent_job(&store, Tier::Devout, Some((student_env.env_id, matrix))).await;

    // The Teacher publishes one flagged Instruction and keeps one unflagged
    // draft (distinct item refs — reuse the two scope nodes as stand-ins).
    let published = scope[0];
    let draft = scope[1];
    store
        .add_env_item(
            t_job.job_id,
            teacher_env.env_id,
            published,
            &valid_chain(published),
            true,
        )
        .await
        .expect("published");
    store
        .add_env_item(
            t_job.job_id,
            teacher_env.env_id,
            draft,
            &valid_chain(draft),
            false,
        )
        .await
        .expect("draft");

    // Before pairing: the Student (reading from its own env) is rejected
    // for the flagged item — no pairing, no exception.
    let err = store
        .env_scoped_read(student.job_id, student_env.env_id, published)
        .await
        .expect_err("unpaired: rejected");
    assert!(matches!(err, StoreError::Forbidden(_)));

    // Form the pairing.
    store
        .form_pairing(
            teacher_env.env_id,
            student_env.env_id,
            matrix,
            PairingKind::DevoutAssignment,
        )
        .await
        .expect("pairing");

    // Now the flagged artifact is readable across the bridge…
    store
        .env_scoped_read(student.job_id, student_env.env_id, published)
        .await
        .expect("the pairing grants the flagged artifact (IX.5)");
    // …but the unflagged draft is not.
    let err = store
        .env_scoped_read(student.job_id, student_env.env_id, draft)
        .await
        .expect_err("unflagged working state stays out of scope");
    assert!(matches!(err, StoreError::Forbidden(_)), "got {err}");

    // An unpaired third party — bound to its OWN room, not paired with the
    // teacher — reads neither of the teacher's items.
    let (_o_job, other_env) = establish(&store, EnvKind::Student, Tier::Devout, matrix)
        .await
        .expect("other student env");
    let stranger = agent_job(&store, Tier::Devout, Some((other_env.env_id, matrix))).await;
    for target in [published, draft] {
        let err = store
            .env_scoped_read(stranger.job_id, other_env.env_id, target)
            .await
            .expect_err("no pairing, no read");
        assert!(matches!(err, StoreError::Forbidden(_)));
    }
}

/// SC-G05 (pairing) — a pairing whose tiers disagree with its kind fails.
#[tokio::test]
async fn sc_g05_pairing_tier_mismatch() {
    let Some(store) = store().await else { return };
    let (matrix, _scope) = grown_matrix(&store).await;
    let (_t, teacher_env) = establish(&store, EnvKind::Teacher, Tier::Devout, matrix)
        .await
        .expect("teacher");
    let (_s, student_env) = establish(&store, EnvKind::Student, Tier::Canon, matrix)
        .await
        .expect("student");
    // Devout teacher + Canon student is neither pairing kind.
    let err = store
        .form_pairing(
            teacher_env.env_id,
            student_env.env_id,
            matrix,
            PairingKind::DevoutAssignment,
        )
        .await
        .expect_err("tiers must match kind");
    assert!(matches!(err, StoreError::ValidationFailed(_)), "got {err}");
}

/// SC-G07 — an ORPHANED environment is a read-only archive: unmountable for
/// work, no write against it succeeds; but its contents remain readable.
#[tokio::test]
async fn sc_g07_orphaned() {
    let Some(store) = store().await else { return };
    let (matrix, scope) = grown_matrix(&store).await;
    let (job, env) = establish(&store, EnvKind::Teacher, Tier::Devout, matrix)
        .await
        .expect("establish");
    store
        .add_env_item(
            job.job_id,
            env.env_id,
            scope[0],
            &valid_chain(scope[0]),
            true,
        )
        .await
        .expect("election");

    // Orphan it.
    let orphaned = store.orphan_environment(env.env_id).await.expect("orphan");
    assert_eq!(orphaned.status, EnvStatus::Orphaned);

    // Unmountable for work.
    let err = mount(&store, EnvKind::Teacher, Tier::Devout, matrix, env.env_id).await;
    assert!(
        matches!(
            err,
            Err(godhead_scriptorium::ScriptoriumError::Store(
                StoreError::EnvInvalid(_)
            ))
        ),
        "an ORPHANED room is not a workplace"
    );
    // No write succeeds.
    let err = store
        .add_env_item(
            job.job_id,
            env.env_id,
            scope[1],
            &valid_chain(scope[1]),
            false,
        )
        .await
        .expect_err("no write against an archive");
    assert!(matches!(err, StoreError::ValidationFailed(_)), "got {err}");
    // But the archive is still readable.
    let items = store.env_items(env.env_id).await.expect("archive readable");
    assert_eq!(items.len(), 1, "the history remains consultable");
}

/// Regression (slice-7 review, HIGH): a binding is not self-declared. A job
/// cannot be born bound to an environment of the wrong tier, an
/// environment whose matrix it is not working, or a non-LIVE room — so the
/// env_ref that env_scoped_read trusts cannot be forged.
#[tokio::test]
async fn env_ref_binding_is_authenticated() {
    let Some(store) = store().await else { return };
    let (matrix, _scope) = grown_matrix(&store).await;
    let (_job, env) = establish(&store, EnvKind::Student, Tier::Devout, matrix)
        .await
        .expect("victim env");

    fn bind_draft(tier: Tier, env_id: Uuid, inputs: Vec<Uuid>) -> JobDraft {
        JobDraft {
            agent_type: AgentType::Student,
            auditor_name: None,
            tier: Some(tier),
            input_refs: inputs,
            env_ref: Some(env_id),
            brief_ref: None,
            endpoint_alias: None,
            manual_version: Version::new(1, 0, 0),
            budgets: Budgets {
                max_wall_ms: 120_000,
                max_tool_calls: 10,
                max_tokens: 1,
            },
        }
    }

    // Wrong tier (env is Devout; attacker claims Canon).
    let err = store
        .create_job(&bind_draft(Tier::Canon, env.env_id, vec![matrix]))
        .await
        .expect_err("tier must match the room");
    assert!(matches!(err, StoreError::ValidationFailed(_)), "got {err}");

    // Right tier but not working the env's matrix.
    let err = store
        .create_job(&bind_draft(Tier::Devout, env.env_id, vec![Uuid::now_v7()]))
        .await
        .expect_err("the matrix must be among the job's inputs");
    assert!(matches!(err, StoreError::ValidationFailed(_)), "got {err}");

    // A legitimate binding (right tier, working the matrix) is accepted.
    store
        .create_job(&bind_draft(Tier::Devout, env.env_id, vec![matrix]))
        .await
        .expect("a legitimate binding is accepted");

    // Binding to a non-LIVE room is refused.
    store.orphan_environment(env.env_id).await.expect("orphan");
    let err = store
        .create_job(&bind_draft(Tier::Devout, env.env_id, vec![matrix]))
        .await
        .expect_err("cannot bind to an archive");
    assert!(matches!(err, StoreError::ValidationFailed(_)), "got {err}");
}

/// Regression (slice-7 review, MEDIUM): a pairing binds two rooms over one
/// shared matrix; a cross-matrix pairing is refused (IX.5, X.5).
#[tokio::test]
async fn pairing_requires_shared_matrix() {
    let Some(store) = store().await else { return };
    let (m1, _) = grown_matrix(&store).await;
    let (m2, _) = grown_matrix(&store).await;
    let (_t, teacher_env) = establish(&store, EnvKind::Teacher, Tier::Devout, m1)
        .await
        .expect("teacher on m1");
    let (_s, student_env) = establish(&store, EnvKind::Student, Tier::Devout, m2)
        .await
        .expect("student on m2");
    let err = store
        .form_pairing(
            teacher_env.env_id,
            student_env.env_id,
            m1,
            PairingKind::DevoutAssignment,
        )
        .await
        .expect_err("the rooms do not share a matrix");
    assert!(matches!(err, StoreError::ValidationFailed(_)), "got {err}");
}
