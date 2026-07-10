//! Section I — the Deacon's Threshold (Dogma Book II §1; Law V.4; rulings
//! G10/G11). SC-I01 … SC-I07b, plus the manifest-per-trip doctrine and the
//! §6 edge cases pinned to this slice: re-scan defeats stale consent, and
//! the Deacon's retry converges.
//!
//! No fetch layer exists (the no-HTTP wall stands), so a fixture labors
//! where the fetch layer will: a mandate-trip job — `brief_ref` IS a
//! MandateRecord — appends its chain entry and deposits at the wall,
//! exactly the shape Slice 11's trips will take.

use godhead_deacon::{Deacon, DeaconError, MockScanner};
use godhead_intake::{sha256_hex, Dispatcher, IntakePipe, STAGE_RESULT_SCHEMA};
use godhead_schemas::{
    AgentType, Budgets, ChainEntryDraft, ChainEntryKind, ConfigTier, ConsentDecision, ConsentScope,
    EnvKind, JobDraft, JobRecord, JobStatus, Locator, LogEvent, MandateDemands, MandateDraft,
    MandateKind, MandateRecord, NodeDraft, QuarantineDraft, ScanEngine, ScanVerdictKind, Tier,
    WritTarget,
};
use godhead_scriptorium::establish;
use godhead_store::{ArtifactDraft, PgStore, Store, StoreError};
use semver::Version;
use serde_json::json;
use std::path::{Path, PathBuf};
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
    Some(
        PgStore::connect(&url, godhead_intake::registry())
            .await
            .expect("store connect + migrate"),
    )
}

fn temp_root() -> PathBuf {
    std::env::temp_dir().join(format!("godhead_deacon_test_{}", Uuid::now_v7()))
}

fn mock_engine() -> ScanEngine {
    ScanEngine {
        alias: "mock-scan".into(),
        version: "1.0.0".into(),
        signature_rev: Some("mock-r1".into()),
    }
}

/// A CARDINAL matrix, planted raw — but never by fiat: the fixture walks
/// VI.3's chain (proposal → consent → act) inside one transaction, because
/// the commitment trigger admits nothing less even from a fixture, and the
/// consent row rides `SET LOCAL godhead.actor_class = 'sovereign'` because
/// consent_records is class-guarded (ruling G10).
async fn plant_cardinal_matrix(store: &PgStore) -> Uuid {
    let job = store
        .create_job(&JobDraft {
            agent_type: AgentType::Slave,
            auditor_name: None,
            tier: None,
            input_refs: vec![],
            env_ref: None,
            brief_ref: None,
            endpoint_alias: None,
            manual_version: Version::new(1, 0, 0),
            budgets: Budgets {
                max_wall_ms: 600_000,
                max_tool_calls: 10,
                max_tokens: 1,
            },
        })
        .await
        .expect("fixture job");
    let matrix_id = Uuid::now_v7();
    let proposal_id = Uuid::now_v7();
    let consent_id = Uuid::now_v7();
    let category = format!("deacon_threshold_{}", Uuid::now_v7());
    let mut tx = store.raw_pool().begin().await.expect("fixture tx");
    sqlx::query("SET LOCAL godhead.actor_class = 'sovereign'")
        .execute(&mut *tx)
        .await
        .expect("fixture class");
    sqlx::query(
        r#"INSERT INTO matrices
             (matrix_id, category, node_refs, link_refs, emerged_by, config_rev,
              schema_name, schema_version, produced_by)
           VALUES ($1, $2, '[]', '[]', $3, 1, 'MatrixRecord', '1.0.0', $3::text)"#,
    )
    .bind(matrix_id)
    .bind(&category)
    .bind(job.job_id)
    .execute(&mut *tx)
    .await
    .expect("born POSTULANT");
    sqlx::query(
        r#"INSERT INTO joint_proposals
             (proposal_id, job_id, matrix_ref, matrix_revision, report_refs, verdict,
              schema_name, schema_version, produced_by)
           VALUES ($1, $2, $3, 1, '[]', 'COMMIT', 'JointProposal', '1.0.0', $2::text)"#,
    )
    .bind(proposal_id)
    .bind(job.job_id)
    .bind(matrix_id)
    .execute(&mut *tx)
    .await
    .expect("COMMIT proposal");
    sqlx::query(
        r#"INSERT INTO consent_records
             (consent_id, subject_ref, decision, scope, decided_by,
              schema_name, schema_version, produced_by)
           VALUES ($1, $2, 'GRANTED', 'ITEM', 'sovereign', 'ConsentRecord', '1.0.0', 'sovereign')"#,
    )
    .bind(consent_id)
    .bind(proposal_id)
    .execute(&mut *tx)
    .await
    .expect("granted consent");
    sqlx::query(
        r#"UPDATE matrices
           SET status = 'CARDINAL', committed_proposal_ref = $2,
               committed_consent_ref = $3, committed_at = now()
           WHERE matrix_id = $1"#,
    )
    .bind(matrix_id)
    .bind(proposal_id)
    .bind(consent_id)
    .execute(&mut *tx)
    .await
    .expect("the chain resolves; the profession lands");
    tx.commit().await.expect("fixture commit");
    matrix_id
}

/// The sovereign authors a writ against a Cardinal matrix (C.4; SC-J02's
/// happy path — a named, resolvable locator).
async fn writ_mandate(store: &PgStore, matrix_ref: Uuid) -> MandateRecord {
    store
        .author_mandate(
            "sovereign",
            &MandateDraft {
                kind: MandateKind::Writ,
                teacher_env_ref: None,
                matrix_ref: Some(matrix_ref),
                demands: MandateDemands::WritTargets(vec![WritTarget {
                    locator: Locator::Uri("https://example.org/atlas.txt".into()),
                    note: None,
                }]),
                trip_budget: json!({ "max_items": 16 }),
            },
        )
        .await
        .expect("writ authored")
}

/// The sovereign authors a canon for a live Teacher room (C.4).
async fn canon_mandate(store: &PgStore, matrix_ref: Uuid) -> MandateRecord {
    let (_teacher_job, teacher_env) = establish(store, EnvKind::Teacher, Tier::Canon, matrix_ref)
        .await
        .expect("a live Teacher room");
    store
        .author_mandate(
            "sovereign",
            &MandateDraft {
                kind: MandateKind::Canon,
                teacher_env_ref: Some(teacher_env.env_id),
                matrix_ref: None,
                demands: MandateDemands::CanonClauses(
                    vec!["every text of the atlas corpus".into()],
                ),
                trip_budget: json!({ "max_items": 16 }),
            },
        )
        .await
        .expect("canon authored")
}

/// A mandate-trip job: `brief_ref` IS the mandate (§1.4 — the shape the
/// substrate's Law V.4 trigger recognizes as a fetching labor), RUNNING.
async fn trip_job(store: &PgStore, mandate_ref: Uuid, tier: Tier) -> JobRecord {
    let job = store
        .create_job(&JobDraft {
            agent_type: AgentType::Student,
            auditor_name: None,
            tier: Some(tier),
            input_refs: vec![mandate_ref],
            env_ref: None,
            brief_ref: Some(mandate_ref),
            endpoint_alias: None,
            manual_version: Version::new(1, 0, 0),
            budgets: Budgets {
                max_wall_ms: 600_000,
                max_tool_calls: 50,
                max_tokens: 100_000,
            },
        })
        .await
        .expect("trip spawns");
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await
        .expect("to LEASED");
    store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await
        .expect("to RUNNING")
}

/// Chain-append in flight (§4.2) then the deposit at the wall (Law V.4):
/// the entry stands before the item is written, exactly as a real trip
/// must write it. Returns the caller-minted item_ref.
async fn deposit(
    store: &PgStore,
    trip: &JobRecord,
    mandate_ref: Uuid,
    root: ChainEntryKind,
    filename: &str,
    content: &[u8],
) -> Uuid {
    let item_ref = Uuid::now_v7();
    store
        .append_chain_entry(
            trip.job_id,
            &ChainEntryDraft {
                chain_ref: item_ref,
                kind: root,
                mandate_ref: Some(mandate_ref),
                prompt_or_reason: "the sovereign's errand named this source".into(),
                produced: vec![item_ref],
            },
        )
        .await
        .expect("chain root stands before the item (SC-J09)");
    store
        .quarantine_deposit(
            trip.job_id,
            item_ref,
            &QuarantineDraft {
                mandate_ref: Some(mandate_ref),
                brief_ref: None,
                filename: filename.into(),
                declared_type: "txt".into(),
                content: content.to_vec(),
            },
        )
        .await
        .expect("the quarantine namespace admits its own (Law V.4)");
    item_ref
}

/// One item carried lawfully to the conjunction's brink: deposited,
/// scanned CLEAN, consented ADMITTED naming the scan it saw (Book II §1
/// steps 1–5). What remains is the admission itself.
async fn clean_admitted_item(
    store: &PgStore,
    deacon: &Deacon<'_, PgStore, MockScanner>,
    trip: &JobRecord,
    mandate_ref: Uuid,
    filename: &str,
    content: &[u8],
) -> Uuid {
    let item_ref = deposit(
        store,
        trip,
        mandate_ref,
        ChainEntryKind::Writ,
        filename,
        content,
    )
    .await;
    deacon.scan_pass(mandate_ref).await.expect("scan pass");
    let scan = store
        .latest_verdict(item_ref)
        .await
        .expect("verdict query")
        .expect("scanned");
    assert_eq!(scan.verdict, ScanVerdictKind::Clean);
    store
        .consent_admission(
            "sovereign",
            item_ref,
            ConsentScope::Item,
            ConsentDecision::Admitted,
            Some(scan.scan_id),
        )
        .await
        .expect("the sovereign admits, naming the scan");
    item_ref
}

/// CAS a config key to a value, absorbing lost races (Law XI.3: the stale
/// revision re-reads, never overwrites).
async fn force_config(store: &PgStore, key: &str, value: serde_json::Value) {
    loop {
        let current = store.get_config(key).await.expect("config key exists");
        if current.value == value {
            return;
        }
        match store
            .set_config(
                "sovereign",
                key,
                ConfigTier::Operational,
                &value,
                Some(current.revision),
            )
            .await
        {
            Ok(_) => return,
            Err(StoreError::StaleRevision { .. }) => continue,
            Err(e) => panic!("set_config {key}: {e}"),
        }
    }
}

/// SC-I01 — external-origin writes land only in the quarantine namespace,
/// rejected anywhere else regardless of writer tier, under BOTH mandate-trip
/// shapes: FETCH_PER_WRIT and FETCH_PER_CANON (Law V.4; the 0014 substrate
/// trigger backs the store walls below every API).
#[tokio::test]
async fn sc_i01_quarantine_only() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let writ = writ_mandate(&store, matrix).await;
    let canon = canon_mandate(&store, matrix).await;
    // A LIVE room for the env-item smuggling attempt.
    let (_est_job, side_env) = establish(&store, EnvKind::Student, Tier::Devout, matrix)
        .await
        .expect("a live side room");

    let shapes = [
        (writ.mandate_id, ChainEntryKind::Writ, Tier::Devout),
        (canon.mandate_id, ChainEntryKind::Canon, Tier::Canon),
    ];
    for (mandate_ref, root, tier) in shapes {
        let trip = trip_job(&store, mandate_ref, tier).await;

        // Its own namespace admits it (Law V.4).
        let item_ref = deposit(
            &store,
            &trip,
            mandate_ref,
            root,
            "fetched.txt",
            b"external text",
        )
        .await;
        assert_eq!(
            store
                .get_quarantine_item(item_ref)
                .await
                .expect("held")
                .origin_job_ref,
            trip.job_id
        );

        // Everywhere else rejects it — the internal-content surfaces, each.
        let smuggled_node = store
            .create_node(
                trip.job_id,
                Uuid::now_v7(),
                &NodeDraft {
                    filename: "smuggle.txt".into(),
                    filetype: "txt".into(),
                    size_bytes: 13,
                    raw_path: "raw/smuggled".into(),
                    raw_sha256: sha256_hex(b"external text"),
                },
            )
            .await
            .expect_err("a fetching labor writes no nodes");
        assert!(
            smuggled_node.to_string().contains("quarantine namespace"),
            "got: {smuggled_node}"
        );

        let smuggled_artifact = store
            .write_artifact(
                trip.job_id,
                "out",
                &ArtifactDraft {
                    schema_name: STAGE_RESULT_SCHEMA.to_string(),
                    schema_version: Version::new(1, 0, 0),
                    payload: json!({
                        "node_id": Uuid::now_v7().to_string(),
                        "stage": "smuggle",
                        "outcome": "OK",
                    }),
                },
            )
            .await
            .expect_err("a fetching labor writes no artifacts");
        assert!(
            smuggled_artifact
                .to_string()
                .contains("quarantine namespace"),
            "got: {smuggled_artifact}"
        );

        let smuggled_election = store
            .add_env_item(
                trip.job_id,
                side_env.env_id,
                Uuid::now_v7(),
                &json!([{ "link_seq": 0, "kind": "BRIEF", "actor": trip.job_id.to_string(),
                          "prompt_or_reason": "smuggling", "produced": [] }]),
                false,
            )
            .await
            .expect_err("a fetching labor curates no room");
        assert!(
            smuggled_election
                .to_string()
                .contains("quarantine namespace"),
            "got: {smuggled_election}"
        );
    }
}

/// SC-I02 — admission requires {scan: CLEAN, consent: ADMITTED}: all four
/// verdict states crossed with both threshold answers; only CLEAN+ADMITTED
/// admits; INFECTED/SUSPECT/ERROR are never presented as admissible — not
/// by clear_for_admission, and not even under a recorded ADMITTED consent
/// (Book II §1).
#[tokio::test]
async fn sc_i02_admission_conjunction() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let mandate = writ_mandate(&store, matrix).await;
    let trip = trip_job(&store, mandate.mandate_id, Tier::Devout).await;
    let deacon = Deacon::new(&store, MockScanner::new());

    // The 4 × 2 grid: each verdict state under each threshold answer.
    let fixtures: [(&str, &[u8], ScanVerdictKind, ConsentDecision); 8] = [
        (
            "clean_admit.txt",
            b"wholesome A",
            ScanVerdictKind::Clean,
            ConsentDecision::Admitted,
        ),
        (
            "clean_reject.txt",
            b"wholesome B",
            ScanVerdictKind::Clean,
            ConsentDecision::Rejected,
        ),
        (
            "infected_admit.txt",
            b"payload GODHEAD-MOCK:INFECTED A",
            ScanVerdictKind::Infected,
            ConsentDecision::Admitted,
        ),
        (
            "infected_reject.txt",
            b"payload GODHEAD-MOCK:INFECTED B",
            ScanVerdictKind::Infected,
            ConsentDecision::Rejected,
        ),
        (
            "suspect_admit.txt",
            b"payload GODHEAD-MOCK:SUSPECT A",
            ScanVerdictKind::Suspect,
            ConsentDecision::Admitted,
        ),
        (
            "suspect_reject.txt",
            b"payload GODHEAD-MOCK:SUSPECT B",
            ScanVerdictKind::Suspect,
            ConsentDecision::Rejected,
        ),
        (
            "error_admit.txt",
            b"payload GODHEAD-MOCK:ERROR A",
            ScanVerdictKind::Error,
            ConsentDecision::Admitted,
        ),
        (
            "error_reject.txt",
            b"payload GODHEAD-MOCK:ERROR B",
            ScanVerdictKind::Error,
            ConsentDecision::Rejected,
        ),
    ];
    let mut items = Vec::with_capacity(fixtures.len());
    for (filename, content, _, _) in &fixtures {
        items.push(
            deposit(
                &store,
                &trip,
                mandate.mandate_id,
                ChainEntryKind::Writ,
                filename,
                content,
            )
            .await,
        );
    }
    deacon
        .scan_pass(mandate.mandate_id)
        .await
        .expect("scan pass");

    // Every verdict landed as the engine spoke it, and NOTHING is
    // admissible before consent — CLEAN included (the Deacon never admits
    // alone).
    for (item_ref, (_, _, expected, _)) in items.iter().zip(&fixtures) {
        let verdict = store
            .latest_verdict(*item_ref)
            .await
            .expect("verdict query")
            .expect("scanned");
        assert_eq!(verdict.verdict, *expected);
        store
            .clear_for_admission(*item_ref)
            .await
            .expect_err("no consent, no admission (Book II §1)");
    }

    // The Manifest lists every item with its verdict — held things are
    // presented, never presented as admissible (Book II §1 step 4).
    let manifest = deacon
        .present_manifest(mandate.mandate_id, trip.job_id)
        .await
        .expect("presented");
    let listed = manifest.items.as_array().expect("items array");
    for (item_ref, (_, _, expected, _)) in items.iter().zip(&fixtures) {
        let entry = listed
            .iter()
            .find(|e| e["item_ref"] == item_ref.to_string())
            .expect("every held item is listed");
        assert_eq!(entry["verdict"], expected.as_str());
    }

    // The sovereign answers each — and the conjunction, not the consent
    // row, decides: an ADMITTED consent over a non-CLEAN item still admits
    // nothing (SC-I02).
    for (item_ref, (_, _, _, decision)) in items.iter().zip(&fixtures) {
        let scan_ref = match decision {
            ConsentDecision::Admitted => Some(
                store
                    .latest_verdict(*item_ref)
                    .await
                    .expect("verdict query")
                    .expect("scanned")
                    .scan_id,
            ),
            _ => None,
        };
        store
            .consent_admission(
                "sovereign",
                *item_ref,
                ConsentScope::Item,
                *decision,
                scan_ref,
            )
            .await
            .expect("the sovereign's answer records");
    }

    let pipe = IntakePipe::new(&store, temp_root()).expect("pipe");
    for (i, (item_ref, (_, _, verdict, decision))) in items.iter().zip(&fixtures).enumerate() {
        let clean_admitted =
            *verdict == ScanVerdictKind::Clean && *decision == ConsentDecision::Admitted;
        if clean_admitted {
            let node = deacon
                .admit(&pipe, *item_ref)
                .await
                .expect("CLEAN + ADMITTED enters the pipe");
            assert_eq!(
                store
                    .get_quarantine_item(*item_ref)
                    .await
                    .expect("item")
                    .admitted_node_ref,
                Some(node)
            );
        } else {
            deacon
                .admit(&pipe, *item_ref)
                .await
                .expect_err("only CLEAN + ADMITTED admits (SC-I02)");
            assert_eq!(
                store
                    .get_quarantine_item(*item_ref)
                    .await
                    .expect("item")
                    .admitted_node_ref,
                None,
                "fixture {i}: held, not admitted"
            );
        }
    }
}

/// SC-I03 — scan provider unreachable: the pass errs, zero verdicts land
/// for the unscanned (already-written verdicts persist — §6: partial
/// verdicts survive a mid-batch failure), every item stays held, zero
/// admissions are possible, and the failure surfaces once per pass, not
/// once per item.
#[tokio::test]
async fn sc_i03_scanner_down_holds() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let mandate = writ_mandate(&store, matrix).await;
    let trip = trip_job(&store, mandate.mandate_id, Tier::Devout).await;
    let deacon = Deacon::new(&store, MockScanner::new());

    // One item scanned while the provider lived.
    let first = deposit(
        &store,
        &trip,
        mandate.mandate_id,
        ChainEntryKind::Writ,
        "a.txt",
        b"alpha",
    )
    .await;
    let written = deacon
        .scan_pass(mandate.mandate_id)
        .await
        .expect("live pass");
    assert_eq!(written.len(), 1);

    // Two more arrive; the provider goes dark.
    let second = deposit(
        &store,
        &trip,
        mandate.mandate_id,
        ChainEntryKind::Writ,
        "b.txt",
        b"beta",
    )
    .await;
    let third = deposit(
        &store,
        &trip,
        mandate.mandate_id,
        ChainEntryKind::Writ,
        "c.txt",
        b"gamma",
    )
    .await;
    deacon.scanner().set_unreachable(true);
    let err = deacon
        .scan_pass(mandate.mandate_id)
        .await
        .expect_err("the threshold surfaces the failure");
    assert!(
        matches!(err, DeaconError::ScannerUnreachable(_)),
        "got {err}"
    );

    // Zero verdicts for the unscanned; the settled verdict persists.
    assert!(store.latest_verdict(second).await.expect("query").is_none());
    assert!(store.latest_verdict(third).await.expect("query").is_none());
    assert!(store.latest_verdict(first).await.expect("query").is_some());

    // All three still held; zero admissions possible.
    let held = store
        .quarantine_items_for(mandate.mandate_id)
        .await
        .expect("held items");
    assert_eq!(held.len(), 3);
    for item in &held {
        assert_eq!(item.admitted_node_ref, None, "held, never admitted");
        store
            .clear_for_admission(item.item_ref)
            .await
            .expect_err("nothing clears while the gate cannot see");
    }

    // Surfaced once for the pass — not once per unscanned item.
    let surfaced = store
        .read_logs(&format!("deacon:scan_pass:{}", mandate.mandate_id))
        .await
        .expect("logs");
    assert_eq!(surfaced.len(), 1, "one failure, one surface");
}

/// SC-I04 — an admitted item enters the onboard pipe at its BEGINNING: the
/// raw copy exists with its checksum, the first log is INTAKE_RAW_COPIED,
/// and normalization runs observably downstream — no shortcut path
/// (Book II §1 step 6).
#[tokio::test]
async fn sc_i04_no_shortcut_path() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let mandate = writ_mandate(&store, matrix).await;
    let trip = trip_job(&store, mandate.mandate_id, Tier::Devout).await;
    let deacon = Deacon::new(&store, MockScanner::new());
    let pipe = IntakePipe::new(&store, temp_root()).expect("pipe");

    let content: &[u8] = b"line one\r\nline two\r\n";
    let item_ref = clean_admitted_item(
        &store,
        &deacon,
        &trip,
        mandate.mandate_id,
        "admitted.md",
        content,
    )
    .await;
    let node_id = deacon.admit(&pipe, item_ref).await.expect("admitted");

    // Raw copy: the atom stands with the quarantined bytes' own checksum.
    let node = store.get_node(node_id).await.expect("node");
    assert_eq!(node.raw_sha256, sha256_hex(content));
    assert_eq!(node.filename, "admitted.md");

    // First log: INTAKE_RAW_COPIED, the pipe's own first act (doc 2 §2.2).
    let logs = store.read_logs(&node_id.to_string()).await.expect("logs");
    let first = logs.first().expect("a first snapshot");
    assert_eq!(first.event, LogEvent::IntakeRawCopied);
    assert_eq!(first.payload["filename"], "admitted.md");

    // Normalization: the dispatcher's next tick consumes the pipe's own
    // RAW_COPY flag — the admitted item is on the ordinary conveyor, not
    // some side door.
    let dispatcher = Dispatcher::new(&pipe);
    let scope = [node_id];
    let ran = dispatcher.tick_scoped(Some(&scope)).await.expect("tick");
    assert!(!ran.is_empty(), "the pipe continues past the raw copy");
    let node = store.get_node(node_id).await.expect("node");
    assert!(node.normalized, "normalization observable");
    assert!(node.derivative_sha256.is_some());
    assert!(
        store
            .read_logs(&node_id.to_string())
            .await
            .expect("logs")
            .iter()
            .any(|l| l.event == LogEvent::Normalized),
        "NORMALIZED on the record"
    );
}

/// SC-I05 — rejected and held items remain in quarantine, preserved and
/// unpurged: rows present, content byte-identical, and deletion is
/// rejected at the substrate (Book II §1 step 7).
///
/// G13 — satisfied below the criterion's words: the words bound
/// preservation "within `quarantine_retention_days`", and the purge that
/// would give that bound teeth is the deferred Duty of the House (doc 00
/// §7; SLICE_10 §5 non-goal). This test proves the preservation half only;
/// the purge half re-arms when the Duty of the House is built.
#[tokio::test]
async fn sc_i05_quarantine_preserved() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let mandate = writ_mandate(&store, matrix).await;
    let trip = trip_job(&store, mandate.mandate_id, Tier::Devout).await;
    let deacon = Deacon::new(&store, MockScanner::new());

    let held_bytes: &[u8] = b"held payload GODHEAD-MOCK:INFECTED";
    let rejected_bytes: &[u8] = b"rejected payload";
    let held = deposit(
        &store,
        &trip,
        mandate.mandate_id,
        ChainEntryKind::Writ,
        "held.txt",
        held_bytes,
    )
    .await;
    let rejected = deposit(
        &store,
        &trip,
        mandate.mandate_id,
        ChainEntryKind::Writ,
        "rejected.txt",
        rejected_bytes,
    )
    .await;
    deacon
        .scan_pass(mandate.mandate_id)
        .await
        .expect("scan pass");
    store
        .consent_admission(
            "sovereign",
            rejected,
            ConsentScope::Item,
            ConsentDecision::Rejected,
            None,
        )
        .await
        .expect("the sovereign rejects");

    // Both remain, contents intact.
    let held_item = store.get_quarantine_item(held).await.expect("held row");
    assert_eq!(held_item.content, held_bytes);
    assert_eq!(held_item.admitted_node_ref, None);
    let rejected_item = store
        .get_quarantine_item(rejected)
        .await
        .expect("rejected row");
    assert_eq!(rejected_item.content, rejected_bytes);
    assert_eq!(rejected_item.admitted_node_ref, None);

    // Deletion — of items or of their verdicts — is rejected below the API.
    for item_ref in [held, rejected] {
        let purge = sqlx::query("DELETE FROM quarantine_items WHERE item_ref = $1")
            .bind(item_ref)
            .execute(store.raw_pool())
            .await;
        assert!(purge.is_err(), "nothing at the threshold is purged");
        let purge_verdicts = sqlx::query("DELETE FROM scan_verdicts WHERE item_ref = $1")
            .bind(item_ref)
            .execute(store.raw_pool())
            .await;
        assert!(purge_verdicts.is_err(), "verdicts are testimony, kept");
    }
}

/// SC-I06 — architectural, SC-B04's mirror for the gate: no API path
/// admits unscanned material. Runtime half: the admission surfaces refuse
/// everything short of the conjunction. Arch half: `admitted_node_ref` —
/// the one column that records an admission — has exactly one writer in
/// the workspace source, `mark_admitted` in postgres.rs, which re-proves
/// the conjunction in the same act.
#[tokio::test]
async fn sc_i06_gate_arch() {
    // Runtime half.
    if let Some(store) = store().await {
        let matrix = plant_cardinal_matrix(&store).await;
        let mandate = writ_mandate(&store, matrix).await;
        let trip = trip_job(&store, mandate.mandate_id, Tier::Devout).await;
        let deacon = Deacon::new(&store, MockScanner::new());
        let pipe = IntakePipe::new(&store, temp_root()).expect("pipe");
        let item_ref = deposit(
            &store,
            &trip,
            mandate.mandate_id,
            ChainEntryKind::Writ,
            "unscanned.txt",
            b"unscanned bytes",
        )
        .await;
        // A legitimate node to aim mark_admitted at — the gate must refuse
        // on the ITEM's standing, whatever node is offered.
        let bystander = pipe
            .commit_file("bystander.txt", b"innocent")
            .await
            .expect("bystander node");

        // Unscanned: both admission surfaces refuse.
        let err = store
            .clear_for_admission(item_ref)
            .await
            .expect_err("unscanned never clears");
        assert!(err.to_string().contains("unscanned"), "got {err}");
        let err = store
            .mark_admitted(item_ref, bystander)
            .await
            .expect_err("unscanned is never recorded admitted");
        assert!(err.to_string().contains("unscanned"), "got {err}");

        // Scanned CLEAN but unconsented: still refused.
        deacon.scan_pass(mandate.mandate_id).await.expect("scan");
        let err = store
            .clear_for_admission(item_ref)
            .await
            .expect_err("the Deacon never admits alone");
        assert!(err.to_string().contains("no consent"), "got {err}");
        store
            .mark_admitted(item_ref, bystander)
            .await
            .expect_err("no consent, no admission record");

        // Consented REJECTED: still refused (only ADMITTED admits).
        store
            .consent_admission(
                "sovereign",
                item_ref,
                ConsentScope::Item,
                ConsentDecision::Rejected,
                None,
            )
            .await
            .expect("rejection records");
        let err = store
            .clear_for_admission(item_ref)
            .await
            .expect_err("REJECTED does not admit");
        assert!(
            err.to_string().contains("only ADMITTED admits"),
            "got {err}"
        );
    }

    // Arch half: single-writer sweep over crates/*/src. The needles are
    // built by concatenation so this file never matches itself if the
    // sweep ever widens beyond src/.
    let set_needle = format!("SET {}", "admitted_node_ref");
    let insert_needle = format!("INSERT INTO {}", "quarantine_items");
    let column = "admitted_node_ref";
    let crates_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ exists")
        .to_path_buf();
    let mut files = Vec::new();
    for entry in std::fs::read_dir(&crates_root).expect("read crates/") {
        let src = entry.expect("dir entry").path().join("src");
        if src.is_dir() {
            collect_rs(&src, &mut files);
        }
    }
    assert!(files.len() > 20, "the walk saw the workspace, not a stub");
    let mut writers: Vec<(PathBuf, usize)> = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).expect("source is UTF-8");
        // An UPDATE assignment of the admission column…
        for (idx, _) in text.match_indices(&set_needle) {
            writers.push((file.clone(), idx));
        }
        // …or an INSERT that names it in the quarantine column list.
        for (idx, _) in text.match_indices(&insert_needle) {
            let end = text[idx..]
                .find("VALUES")
                .map(|v| idx + v)
                .unwrap_or_else(|| (idx + 400).min(text.len()));
            if text[idx..end].contains(column) {
                writers.push((file.clone(), idx));
            }
        }
    }
    assert_eq!(
        writers.len(),
        1,
        "exactly one writer of {column} in the workspace, got {writers:?}"
    );
    let (writer_file, writer_idx) = &writers[0];
    assert!(
        writer_file.ends_with(Path::new("godhead-store").join("src").join("postgres.rs")),
        "the one writer is the store substrate, got {writer_file:?}"
    );
    let text = std::fs::read_to_string(writer_file).expect("postgres.rs");
    let fn_start = text
        .find("async fn mark_admitted")
        .expect("mark_admitted exists");
    let fn_end = text[fn_start + 1..]
        .find("async fn ")
        .map(|i| fn_start + 1 + i)
        .unwrap_or(text.len());
    assert!(
        (fn_start..fn_end).contains(writer_idx),
        "the one write sits inside mark_admitted — the surface that re-proves the conjunction"
    );
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read src dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// SC-I07a — the actor-class wall (ruling G10): a write to a sovereign- or
/// office-reserved table whose path did not authenticate as that class is
/// rejected at the substrate, whatever string it stamps — below the API,
/// 'sovereign', 'office:deacon', and 'forged' are the same object. A
/// mismatched class is the same bypass; the lawful API paths succeed for
/// exactly their class's tables.
#[tokio::test]
async fn sc_i07a_actor_class_wall() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let mandate = writ_mandate(&store, matrix).await;
    let trip = trip_job(&store, mandate.mandate_id, Tier::Devout).await;
    let deacon = Deacon::new(&store, MockScanner::new());
    let item_ref = deposit(
        &store,
        &trip,
        mandate.mandate_id,
        ChainEntryKind::Writ,
        "walled.txt",
        b"walled bytes",
    )
    .await;
    // A standing Manifest, so the forged-manifest probe would ALSO break on
    // UNIQUE(trip_job_ref) even if the wall fell — defense in depth.
    deacon
        .present_manifest(mandate.mandate_id, trip.job_id)
        .await
        .expect("presented");

    // Unauthenticated, every stamp, every reserved table: rejected.
    for stamp in ["sovereign", "office:deacon", "forged"] {
        let verdict = sqlx::query(
            r#"INSERT INTO scan_verdicts
                 (scan_id, item_ref, verdict, engine_alias, engine_version,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, 'CLEAN', 'forged-engine', '0.0.0', 'ScanVerdict', '1.0.0', $3)"#,
        )
        .bind(Uuid::now_v7())
        .bind(item_ref)
        .bind(stamp)
        .execute(store.raw_pool())
        .await
        .expect_err("scan_verdicts is office-reserved");
        assert!(
            verdict.to_string().contains("GATE_BYPASS_ATTEMPT"),
            "stamp '{stamp}': got {verdict}"
        );

        let manifest = sqlx::query(
            r#"INSERT INTO manifests
                 (manifest_id, mandate_ref, trip_job_ref, items,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, '[]', 'Manifest', '1.0.0', $4)"#,
        )
        .bind(Uuid::now_v7())
        .bind(mandate.mandate_id)
        .bind(trip.job_id)
        .bind(stamp)
        .execute(store.raw_pool())
        .await
        .expect_err("manifests is office-reserved");
        assert!(
            manifest.to_string().contains("GATE_BYPASS_ATTEMPT"),
            "stamp '{stamp}': got {manifest}"
        );

        let consent = sqlx::query(
            r#"INSERT INTO consent_records
                 (consent_id, subject_ref, decision, scope, decided_by,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, 'ADMITTED', 'ITEM', $3, 'ConsentRecord', '1.0.0', $3)"#,
        )
        .bind(Uuid::now_v7())
        .bind(item_ref)
        .bind(stamp)
        .execute(store.raw_pool())
        .await
        .expect_err("consent_records is sovereign-reserved");
        assert!(
            consent.to_string().contains("GATE_BYPASS_ATTEMPT"),
            "stamp '{stamp}': got {consent}"
        );
    }

    // The WRONG class is the same bypass: a sovereign session cannot write
    // the office's table, nor the office the sovereign's — and an office
    // session stamping anything but itself is caught by the stamp check.
    let probes: [(&str, &str); 3] = [
        (
            "sovereign",
            r#"INSERT INTO scan_verdicts
                 (scan_id, item_ref, verdict, engine_alias, engine_version,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, 'CLEAN', 'forged-engine', '0.0.0', 'ScanVerdict', '1.0.0', 'office:deacon')"#,
        ),
        (
            "office:deacon",
            r#"INSERT INTO consent_records
                 (consent_id, subject_ref, decision, scope, decided_by,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, 'ADMITTED', 'ITEM', 'sovereign', 'ConsentRecord', '1.0.0', 'sovereign')"#,
        ),
        (
            "office:deacon",
            r#"INSERT INTO scan_verdicts
                 (scan_id, item_ref, verdict, engine_alias, engine_version,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, 'CLEAN', 'forged-engine', '0.0.0', 'ScanVerdict', '1.0.0', 'forged')"#,
        ),
    ];
    for (class, sql) in probes {
        let mut tx = store.raw_pool().begin().await.expect("probe tx");
        sqlx::query(&format!("SET LOCAL godhead.actor_class = '{class}'"))
            .execute(&mut *tx)
            .await
            .expect("probe class");
        let err = sqlx::query(sql)
            .bind(Uuid::now_v7())
            .bind(item_ref)
            .execute(&mut *tx)
            .await
            .expect_err("class mismatch is a bypass");
        assert!(
            err.to_string().contains("GATE_BYPASS_ATTEMPT"),
            "class '{class}': got {err}"
        );
        tx.rollback().await.expect("probe rollback");
    }

    // The lawful paths succeed for exactly their class's tables: the
    // office's scan write, the sovereign's consent.
    let verdict = store
        .record_scan_verdict(item_ref, ScanVerdictKind::Suspect, &mock_engine())
        .await
        .expect("the office writes its own record");
    assert_eq!(verdict.envelope.produced_by, "office:deacon");
    store
        .consent_admission(
            "sovereign",
            item_ref,
            ConsentScope::Item,
            ConsentDecision::Rejected,
            None,
        )
        .await
        .expect("the sovereign answers through the sovereign surface");
}

/// SC-I07b — admission legibility (ruling G11): over the operational
/// constants the Manifest carries a standing notice naming the counts and
/// the petition-style terminal answers; below them it carries none. Never
/// blocking — the Manifest returns either way; never silent — the notice
/// rides the record itself.
///
/// G13 — satisfied below the criterion's words on one edge: the notice
/// NAMES the terminal answers (acknowledge / silence with suppressed
/// logging), but no admission-specific acknowledge/silence surface exists
/// in the Store yet — the answering mechanics stand on the petition
/// machinery (SC-C02/C03, proven in c_sovereignty.rs). The answering half
/// re-arms if a dedicated resolution surface for admission notices is
/// built.
#[tokio::test]
async fn sc_i07b_admission_legibility() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let mandate = writ_mandate(&store, matrix).await;
    let deacon = Deacon::new(&store, MockScanner::new());

    // Remember lawful restore points (self-healing if a prior crashed run
    // left the dials moved: never restore to a fixture value).
    let batch_restore = store
        .get_config("admission_batch_threshold")
        .await
        .expect("constant seeded by 0015")
        .value
        .as_i64()
        .filter(|n| *n >= 2)
        .unwrap_or(50);
    let rate_restore = store
        .get_config("admission_rate_threshold")
        .await
        .expect("constant seeded by 0015")
        .value
        .as_i64()
        .filter(|n| (1..=100_000).contains(n))
        .unwrap_or(5);

    // Over the batch constant: two items against a threshold of one.
    force_config(&store, "admission_batch_threshold", json!(1)).await;
    let trip_over = trip_job(&store, mandate.mandate_id, Tier::Devout).await;
    deposit(
        &store,
        &trip_over,
        mandate.mandate_id,
        ChainEntryKind::Writ,
        "o1.txt",
        b"one",
    )
    .await;
    deposit(
        &store,
        &trip_over,
        mandate.mandate_id,
        ChainEntryKind::Writ,
        "o2.txt",
        b"two",
    )
    .await;
    let over = deacon
        .present_manifest(mandate.mandate_id, trip_over.job_id)
        .await
        .expect("never blocking: the Manifest returns");
    let notice = over
        .standing_notice
        .expect("over the constant, a standing notice");
    assert!(notice.contains("2 items"), "names the item count: {notice}");
    assert!(
        notice.contains("batch threshold of 1"),
        "names the constant it crossed: {notice}"
    );
    assert!(
        notice.contains("acknowledge"),
        "terminal answer named: {notice}"
    );
    assert!(
        notice.contains("silence"),
        "terminal answer named: {notice}"
    );
    force_config(&store, "admission_batch_threshold", json!(batch_restore)).await;

    // Below both constants: no notice, and still no gate — the rate dial is
    // parked high for the assembly because the live shared store carries
    // real admission consents in the trailing window.
    force_config(&store, "admission_rate_threshold", json!(1_000_000)).await;
    let trip_under = trip_job(&store, mandate.mandate_id, Tier::Devout).await;
    deposit(
        &store,
        &trip_under,
        mandate.mandate_id,
        ChainEntryKind::Writ,
        "u1.txt",
        b"one",
    )
    .await;
    deposit(
        &store,
        &trip_under,
        mandate.mandate_id,
        ChainEntryKind::Writ,
        "u2.txt",
        b"two",
    )
    .await;
    let under = deacon
        .present_manifest(mandate.mandate_id, trip_under.job_id)
        .await
        .expect("never blocking: the Manifest returns");
    assert_eq!(
        under.standing_notice, None,
        "below the constants, no notice"
    );
    force_config(&store, "admission_rate_threshold", json!(rate_restore)).await;

    assert_ne!(over.manifest_id, under.manifest_id);
}

/// One Manifest per mandate-trip, never pooled (Book II §1 doctrine;
/// ruling G11 — UNIQUE(trip_job_ref) at the substrate): re-presentation
/// converges on the standing Manifest; a second trip of the same mandate
/// gets its own; a trip's Manifest cannot be re-assembled under another
/// mandate.
#[tokio::test]
async fn manifest_one_per_trip() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let mandate = writ_mandate(&store, matrix).await;
    let deacon = Deacon::new(&store, MockScanner::new());

    let trip_one = trip_job(&store, mandate.mandate_id, Tier::Devout).await;
    deposit(
        &store,
        &trip_one,
        mandate.mandate_id,
        ChainEntryKind::Writ,
        "t1.txt",
        b"first trip",
    )
    .await;
    let first = deacon
        .present_manifest(mandate.mandate_id, trip_one.job_id)
        .await
        .expect("presented");
    let again = deacon
        .present_manifest(mandate.mandate_id, trip_one.job_id)
        .await
        .expect("re-presentation converges");
    assert_eq!(
        first.manifest_id, again.manifest_id,
        "one Manifest, one trip"
    );

    // A second trip of the same mandate carries its own Manifest.
    let trip_two = trip_job(&store, mandate.mandate_id, Tier::Devout).await;
    deposit(
        &store,
        &trip_two,
        mandate.mandate_id,
        ChainEntryKind::Writ,
        "t2.txt",
        b"second trip",
    )
    .await;
    let second = deacon
        .present_manifest(mandate.mandate_id, trip_two.job_id)
        .await
        .expect("its own Manifest");
    assert_ne!(first.manifest_id, second.manifest_id);
    assert_eq!(second.mandate_ref, mandate.mandate_id);

    // Never pooled: the standing Manifest is not re-assemblable under a
    // different mandate.
    let other = writ_mandate(&store, matrix).await;
    let err = deacon
        .present_manifest(other.mandate_id, trip_one.job_id)
        .await
        .expect_err("one Manifest serves one mandate-trip");
    assert!(
        matches!(err, DeaconError::Store(StoreError::ValidationFailed(_))),
        "got {err}"
    );
}

/// §6 edge — re-scan after consent: the consent names the scan it saw, so
/// a newer verdict on the same item defeats it at the admission wall — a
/// darker verdict absolutely, and even a newer CLEAN, because a stale
/// consent admits nothing (Book II §1).
#[tokio::test]
async fn rescan_after_consent_defeats_stale_consent() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let mandate = writ_mandate(&store, matrix).await;
    let trip = trip_job(&store, mandate.mandate_id, Tier::Devout).await;
    let deacon = Deacon::new(&store, MockScanner::new());
    let pipe = IntakePipe::new(&store, temp_root()).expect("pipe");

    // Consented CLEAN, then a darker verdict lands: admission refuses,
    // the item stays held.
    let darkened = clean_admitted_item(
        &store,
        &deacon,
        &trip,
        mandate.mandate_id,
        "dark.txt",
        b"clean then",
    )
    .await;
    store
        .record_scan_verdict(darkened, ScanVerdictKind::Infected, &mock_engine())
        .await
        .expect("a re-scan is a NEW verdict");
    let err = deacon
        .admit(&pipe, darkened)
        .await
        .expect_err("the newer verdict defeats the stale consent");
    assert!(err.to_string().contains("never admissible"), "got {err}");
    assert_eq!(
        store
            .get_quarantine_item(darkened)
            .await
            .expect("item")
            .admitted_node_ref,
        None,
        "held, not admitted"
    );

    // Even a newer CLEAN defeats the old consent: it names a scan that is
    // no longer the item's latest.
    let restated = clean_admitted_item(
        &store,
        &deacon,
        &trip,
        mandate.mandate_id,
        "re.txt",
        b"clean again",
    )
    .await;
    store
        .record_scan_verdict(restated, ScanVerdictKind::Clean, &mock_engine())
        .await
        .expect("a second CLEAN, newer scan_id");
    let err = deacon
        .admit(&pipe, restated)
        .await
        .expect_err("a stale consent admits nothing");
    assert!(err.to_string().contains("stale consent"), "got {err}");
}

/// §6 edge — the Deacon's retry converges (Law I.3, the slice-1
/// idempotency shape): admitting twice yields the same node, one raw copy,
/// no double-entry into the pipe; recording a DIFFERENT node against a
/// recorded admission is rejected.
#[tokio::test]
async fn deacon_retry_converges() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let mandate = writ_mandate(&store, matrix).await;
    let trip = trip_job(&store, mandate.mandate_id, Tier::Devout).await;
    let deacon = Deacon::new(&store, MockScanner::new());
    let pipe = IntakePipe::new(&store, temp_root()).expect("pipe");

    let item_ref = clean_admitted_item(
        &store,
        &deacon,
        &trip,
        mandate.mandate_id,
        "once.txt",
        b"once only",
    )
    .await;
    let first = deacon.admit(&pipe, item_ref).await.expect("admitted");
    let retry = deacon
        .admit(&pipe, item_ref)
        .await
        .expect("retry converges");
    assert_eq!(first, retry, "same node under retry");

    // One raw copy: the pipe's first act ran exactly once.
    let copies = store
        .read_logs(&first.to_string())
        .await
        .expect("logs")
        .iter()
        .filter(|l| l.event == LogEvent::IntakeRawCopied)
        .count();
    assert_eq!(copies, 1, "raw copied exactly once (SC-N01 at the gate)");
    let converged = store
        .mark_admitted(item_ref, first)
        .await
        .expect("record converges");
    assert_eq!(converged.admitted_node_ref, Some(first));

    // A different node is not a convergence; it is a second admission,
    // and admission is recorded exactly once.
    let imposter = pipe
        .commit_file("imposter.txt", b"someone else")
        .await
        .expect("imposter node");
    let err = store
        .mark_admitted(item_ref, imposter)
        .await
        .expect_err("admission is recorded exactly once");
    assert!(err.to_string().contains("exactly once"), "got {err}");
}

/// F1 (keyed-intake idempotency): admission converges even when a prior
/// attempt CRASHED between the intake mint and the convergence witness. The
/// node id is DERIVED from the item, so the retry reads the would-be orphan
/// back instead of minting a second CLEAN atom into the corpus — the gap the
/// old mint-fresh-then-record path left open (SLICE_10 §9.2 finding F1).
#[tokio::test]
async fn admit_is_idempotent_under_retry() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let mandate = writ_mandate(&store, matrix).await;
    let trip = trip_job(&store, mandate.mandate_id, Tier::Devout).await;
    let deacon = Deacon::new(&store, MockScanner::new());
    let pipe = IntakePipe::new(&store, temp_root()).expect("pipe");

    let item_ref = clean_admitted_item(
        &store,
        &deacon,
        &trip,
        mandate.mandate_id,
        "crash.txt",
        b"body once",
    )
    .await;

    // Simulate the crash: a prior attempt minted the node under the SAME
    // derived id admit will use, then died before recording the admission.
    let node_id = godhead_deacon::admission_node_id(item_ref);
    let minted = pipe
        .commit_file_with_id(node_id, "crash.txt", b"body once")
        .await
        .expect("the crashed attempt's mint");
    assert_eq!(minted, node_id, "the id is derived, not random");
    let mid = store.get_quarantine_item(item_ref).await.expect("item");
    assert!(
        mid.admitted_node_ref.is_none(),
        "the crash left the admission unrecorded"
    );

    // Admit now: it MUST converge on the already-minted node, not mint a second.
    let admitted = deacon
        .admit(&pipe, item_ref)
        .await
        .expect("admit converges on the would-be orphan");
    assert_eq!(admitted, node_id, "the derived id keys the admission");

    // The atom was copied exactly once, despite the crash-and-retry.
    let copies = store
        .read_logs(&node_id.to_string())
        .await
        .expect("logs")
        .iter()
        .filter(|l| l.event == LogEvent::IntakeRawCopied)
        .count();
    assert_eq!(copies, 1, "one CLEAN atom, not a duplicate (F1)");

    // And the admission is recorded exactly once, on that node.
    let done = store.get_quarantine_item(item_ref).await.expect("item");
    assert_eq!(done.admitted_node_ref, Some(node_id));
}
