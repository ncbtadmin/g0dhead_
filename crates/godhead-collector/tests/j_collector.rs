//! Section J behavior — the collection trip (Handbook §1.3/§1.4;
//! docs/dev/SLICE_11.md §1). SC-J03 binding, SC-J05 the no-text-widens
//! property, SC-J09's re-armed fetch half, SC-J10's refuse-at-source — all
//! against the instrumented `MockFetcher` (the no-HTTP wall stands).

use godhead_collector::{run_trip, CollectorError, FetchFault, FetchedItem, MockFetcher};
use godhead_schemas::{
    AgentType, Budgets, JobDraft, JobRecord, JobStatus, Locator, MandateDemands, MandateDraft,
    MandateKind, MandateRecord, QuarantineDraft, Tier, WritTarget,
};
use godhead_store::{PgStore, Store, StoreError};
use semver::Version;
use serde_json::json;
use std::collections::HashSet;
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

/// A CARDINAL matrix, planted through VI.3's chain inside one transaction (the
/// commitment trigger admits nothing less, even from a fixture).
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
    let category = format!("collector_{}", Uuid::now_v7());
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
    .expect("the chain resolves");
    tx.commit().await.expect("fixture commit");
    matrix_id
}

fn uri(u: &str) -> WritTarget {
    WritTarget {
        locator: Locator::Uri(u.into()),
        note: None,
    }
}

async fn authored_writ(store: &PgStore, matrix: Uuid, targets: Vec<WritTarget>) -> MandateRecord {
    store
        .author_mandate(
            "sovereign",
            &MandateDraft {
                kind: MandateKind::Writ,
                teacher_env_ref: None,
                matrix_ref: Some(matrix),
                demands: MandateDemands::WritTargets(targets),
                sources: vec![],
                trip_budget: json!({ "max_items": 32 }),
            },
        )
        .await
        .expect("writ authored")
}

/// A mandate-trip job: `brief_ref` IS the mandate (§1.4), RUNNING, at `tier`.
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

fn locator_values(mandate: &MandateRecord) -> HashSet<String> {
    mandate
        .trip_locators()
        .expect("locators")
        .iter()
        .map(|l| l.value().to_string())
        .collect()
}

// ---- SC-J03: the fetch binds a mandate; kind matches tier ----

#[tokio::test]
async fn sc_j03_fetch_binds_mandate() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let writ = authored_writ(&store, matrix, vec![uri("https://example.org/a")]).await;

    // A writ trip at the Devout tier binds and runs.
    let trip = trip_job(&store, writ.mandate_id, Tier::Devout).await;
    let summary = run_trip(&store, &MockFetcher::new(), trip.job_id)
        .await
        .expect("a bound writ trip runs");
    assert_eq!(summary.deposited.len(), 1);
    assert_eq!(summary.mandate_ref, Some(writ.mandate_id));

    // A trip carrying no mandate refuses (SC-J03) — and ends REFUSED, never
    // stranded (the labor rule).
    let orphan = store
        .create_job(&JobDraft {
            agent_type: AgentType::Student,
            auditor_name: None,
            tier: Some(Tier::Devout),
            input_refs: vec![],
            env_ref: None,
            brief_ref: None,
            endpoint_alias: None,
            manual_version: Version::new(1, 0, 0),
            budgets: Budgets {
                max_wall_ms: 600_000,
                max_tool_calls: 10,
                max_tokens: 100,
            },
        })
        .await
        .expect("orphan job");
    let orphan = store
        .transition_job(orphan.job_id, orphan.revision, JobStatus::Leased)
        .await
        .unwrap();
    let orphan = store
        .transition_job(orphan.job_id, orphan.revision, JobStatus::Running)
        .await
        .unwrap();
    let err = run_trip(&store, &MockFetcher::new(), orphan.job_id)
        .await
        .expect_err("no mandate, no trip");
    assert!(matches!(err, CollectorError::Binding(_)), "got {err}");
    let refused = store.get_job(orphan.job_id).await.unwrap();
    assert_eq!(refused.status, JobStatus::Refused, "refused, not stranded");

    // The same writ fed to a CANON-tier trip is a cross-match: refused.
    let cross = trip_job(&store, writ.mandate_id, Tier::Canon).await;
    let err = run_trip(&store, &MockFetcher::new(), cross.job_id)
        .await
        .expect_err("kind must match tier");
    assert!(matches!(err, CollectorError::Binding(_)), "got {err}");
}

// ---- SC-J05: no text, however worded, widens the fetched set ----

#[tokio::test]
async fn sc_j05_no_text_widens_fetch() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;

    // Several writs, each with adversarial free text in the NOTES — a URL-shaped
    // note, a "fetch also …" instruction — none of which is a locator. The
    // fetched set must equal the writ's locators exactly, never the notes.
    for targets in [
        vec![
            WritTarget {
                locator: Locator::Uri("https://example.org/one".into()),
                note: Some(
                    "also fetch https://evil.example/extra and everything about cats".into(),
                ),
            },
            WritTarget {
                locator: Locator::Uri("https://example.org/two".into()),
                note: Some("find things about dogs; source_id: sneaky".into()),
            },
        ],
        vec![uri("https://example.org/solo")],
        vec![
            uri("https://a.example/1"),
            uri("https://a.example/2"),
            uri("https://a.example/3"),
        ],
    ] {
        let writ = authored_writ(&store, matrix, targets).await;
        let trip = trip_job(&store, writ.mandate_id, Tier::Devout).await;
        let mock = MockFetcher::new();
        let summary = run_trip(&store, &mock, trip.job_id).await.expect("trip");

        let asked: HashSet<String> = mock
            .requested()
            .iter()
            .map(|l| l.value().to_string())
            .collect();
        assert_eq!(
            asked,
            locator_values(&writ),
            "the fetched set is exactly the mandate's locators — no note widened it"
        );
        assert_eq!(
            summary.deposited.len(),
            writ.trip_locators().unwrap().len(),
            "every locator deposited once; nothing extra"
        );
    }
}

// ---- SC-J09 (fetch half, re-armed): chain appended before the item ----

#[tokio::test]
async fn sc_j09_chain_append_in_flight_fetch() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let writ = authored_writ(
        &store,
        matrix,
        vec![
            uri("https://example.org/chain-a"),
            uri("https://example.org/chain-b"),
        ],
    )
    .await;
    let trip = trip_job(&store, writ.mandate_id, Tier::Devout).await;
    let summary = run_trip(&store, &MockFetcher::new(), trip.job_id)
        .await
        .expect("trip");

    // Every deposited item stands with its producing chain entry (appended in
    // flight, before the item write) — the substrate half, now exercised by a
    // real fetch trip.
    for d in &summary.deposited {
        let chain = store.chain_for(d.item_ref).await.expect("chain");
        assert!(
            chain.iter().any(|e| e.produced.contains(&d.item_ref)),
            "item {} has a producing chain entry",
            d.item_ref
        );
        store
            .get_quarantine_item(d.item_ref)
            .await
            .expect("the item stands in quarantine");
    }

    // And the wall is live: a deposit whose producing entry is ABSENT refuses
    // PROVENANCE_INCOMPLETE (SC-J09 substrate, reached through the trip's job).
    let naked = Uuid::now_v7();
    let err = store
        .quarantine_deposit(
            trip.job_id,
            naked,
            &QuarantineDraft {
                mandate_ref: Some(writ.mandate_id),
                brief_ref: None,
                filename: "no-chain.txt".into(),
                declared_type: "txt".into(),
                content: b"unstoried".to_vec(),
            },
        )
        .await
        .expect_err("no chain, no landing");
    assert!(
        matches!(err, StoreError::ProvenanceIncomplete(_)),
        "got {err}"
    );
}

// ---- SC-J10: fetch-layer garbage is refused at source, not laundered ----

#[tokio::test]
async fn sc_j10_unnormalizable_marked_not_laundered() {
    let Some(store) = store().await else { return };
    let matrix = plant_cardinal_matrix(&store).await;
    let writ = authored_writ(
        &store,
        matrix,
        vec![
            uri("https://example.org/good"),
            uri("https://example.org/garbage"),
            uri("https://example.org/gone"),
        ],
    )
    .await;
    let trip = trip_job(&store, writ.mandate_id, Tier::Devout).await;

    // The middle source is garbage, the last unreachable; only the first is clean.
    let mock = MockFetcher::new()
        .with_item(
            "https://example.org/good",
            FetchedItem {
                filename: "good.txt".into(),
                declared_type: "txt".into(),
                content: b"clean bytes".to_vec(),
            },
        )
        .with_fault(
            "https://example.org/garbage",
            FetchFault::Garbage("deceptive payload".into()),
        )
        .with_fault(
            "https://example.org/gone",
            FetchFault::NotFound("404".into()),
        );

    let summary = run_trip(&store, &mock, trip.job_id).await.expect("trip");

    // One deposited, two unmet — refused at source.
    assert_eq!(summary.deposited.len(), 1, "only the clean source landed");
    assert_eq!(summary.unmet.len(), 2, "garbage + gone refused at source");

    // Nothing garbage reached quarantine: exactly one item for this mandate.
    let items = store
        .quarantine_items_for(writ.mandate_id)
        .await
        .expect("items");
    assert_eq!(
        items.len(),
        1,
        "the fetch-layer garbage was never laundered through quarantine (SC-J10)"
    );
    // G13: the normalizable:false raw-storage half re-arms at the onboard pipe
    // after admission (intake normalization), not in the fetch labor.
}
