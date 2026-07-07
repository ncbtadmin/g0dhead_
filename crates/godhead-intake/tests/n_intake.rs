//! Section N — Intake & Endurance. SC-N01 … SC-N06.

use godhead_intake::{Dispatcher, IntakePipe, Supervisor, STAGE_NORMALIZE, STAGE_RAW_COPY};
use godhead_schemas::{IntakeStatus, LogEvent, Severity};
use godhead_store::{PgStore, Store};
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
    std::env::temp_dir().join(format!("godhead_test_{}", Uuid::now_v7()))
}

/// Drives a committed node to at-rest (two ticks: normalize, classify).
/// Ticks are scoped to the node: the test database is shared, and each
/// test's files live under its own temp root.
async fn run_to_rest(pipe: &IntakePipe<'_, PgStore>, node_id: Uuid) {
    let dispatcher = Dispatcher::new(pipe);
    let scope = [node_id];
    dispatcher.tick_scoped(Some(&scope)).await.expect("tick 1");
    dispatcher.tick_scoped(Some(&scope)).await.expect("tick 2");
}

/// The comparable end-state of a node: everything the pipeline determines.
async fn node_summary(
    store: &PgStore,
    node_id: Uuid,
) -> (bool, IntakeStatus, serde_json::Value, Option<String>) {
    let node = store.get_node(node_id).await.expect("get node");
    (
        node.normalized,
        node.intake_status,
        node.classification,
        node.derivative_sha256,
    )
}

async fn node_events(store: &PgStore, node_id: Uuid) -> Vec<LogEvent> {
    store
        .read_logs(&node_id.to_string())
        .await
        .expect("logs")
        .iter()
        .map(|l| l.event)
        .collect()
}

/// SC-N01 — raw is copied exactly once: the atom's checksum is stable
/// across the full cycle, and no path — API or substrate — rewrites it.
#[tokio::test]
async fn sc_n01_raw_copied_exactly_once() {
    let Some(store) = store().await else { return };
    let pipe = IntakePipe::new(&store, temp_root()).expect("pipe");
    let bytes = b"hello world\r\nsecond line\r\n";
    let node_id = pipe.commit_file("hello.txt", bytes).await.expect("commit");
    run_to_rest(&pipe, node_id).await;

    let node = store.get_node(node_id).await.expect("node");
    let disk = std::fs::read(pipe.data_root().join(&node.raw_path)).expect("raw on disk");
    assert_eq!(disk, bytes, "raw bytes exactly as committed");
    let sha_at_rest = node.raw_sha256.clone();

    // The full downstream cycle — including regeneration — leaves the atom alone.
    pipe.renormalize(node_id).await.expect("renormalize");
    let node = store.get_node(node_id).await.expect("node again");
    assert_eq!(node.raw_sha256, sha_at_rest, "checksum stable");
    let disk = std::fs::read(pipe.data_root().join(&node.raw_path)).expect("raw still on disk");
    assert_eq!(godhead_intake::sha256_hex(&disk), sha_at_rest);

    // Even a raw SQL writer cannot rewrite the atom (substrate trigger).
    let smash = sqlx::query("UPDATE nodes SET raw_sha256 = repeat('0', 64) WHERE node_id = $1")
        .bind(node_id)
        .execute(store.raw_pool())
        .await;
    assert!(smash.is_err(), "the atom is immutable at the substrate");
}

/// SC-N02 — the first log snapshot writes on raw copy with all required
/// fields; rotation preserves priors — the chain walks, nothing overwrites.
#[tokio::test]
async fn sc_n02_first_log_and_rotation_chain() {
    let Some(store) = store().await else { return };
    let pipe = IntakePipe::new(&store, temp_root()).expect("pipe");
    let node_id = pipe
        .commit_file("notes.md", b"# notes\n")
        .await
        .expect("commit");

    let logs = store.read_logs(&node_id.to_string()).await.expect("logs");
    let first = logs.first().expect("first snapshot exists");
    assert_eq!(first.event, LogEvent::IntakeRawCopied);
    assert_eq!(first.payload["filename"], "notes.md");
    assert_eq!(first.payload["filetype"], "md");
    assert_eq!(first.payload["size_bytes"], 8);
    assert_eq!(first.payload["normalized"], false);
    let first_id = first.log_id;
    let first_payload = first.payload.clone();

    run_to_rest(&pipe, node_id).await;
    pipe.renormalize(node_id).await.expect("renormalize");

    let logs = store.read_logs(&node_id.to_string()).await.expect("logs");
    assert!(logs.len() >= 4, "copy, normalize, classify, renormalize");
    // The chain walks root to leaf; each snapshot rotates its prior.
    assert!(logs[0].prior_ref.is_none());
    for pair in logs.windows(2) {
        assert_eq!(
            pair[1].prior_ref,
            Some(pair[0].log_id),
            "rotation chain intact"
        );
    }
    // Nothing overwrote the first snapshot.
    assert_eq!(logs[0].log_id, first_id);
    assert_eq!(logs[0].payload, first_payload);
}

/// SC-N03 — a decode failure is logged and flagged, the file stored, never
/// silently accepted; an unsupported type is stored raw with an
/// incompatibility notice, never rejected.
#[tokio::test]
async fn sc_n03_decode_failure_and_unsupported() {
    let Some(store) = store().await else { return };
    let pipe = IntakePipe::new(&store, temp_root()).expect("pipe");

    // Invalid UTF-8 in a supported type.
    let bad_bytes = &[0x68, 0x69, 0xC3, 0x28, 0x0A];
    let bad = pipe
        .commit_file("bad.txt", bad_bytes)
        .await
        .expect("commit never rejects");
    run_to_rest(&pipe, bad).await;
    let node = store.get_node(bad).await.expect("node");
    assert_eq!(node.intake_status, IntakeStatus::DecodeFailed);
    assert!(!node.normalized);
    assert!(node.notice.is_some(), "the failure is surfaced on the node");
    let disk = std::fs::read(pipe.data_root().join(&node.raw_path)).expect("raw preserved");
    assert_eq!(disk, bad_bytes);
    let logs = store.read_logs(&bad.to_string()).await.expect("logs");
    assert!(
        logs.iter()
            .any(|l| l.event == LogEvent::Normalized && l.severity == Severity::Warning),
        "decode failure logged as a warning, not buried"
    );
    assert!(
        logs.iter().any(|l| l.event == LogEvent::Classified),
        "the node still rests with baseline structure"
    );

    // A type outside the supported set.
    let png = pipe
        .commit_file("image.png", &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A])
        .await
        .expect("unsupported types are stored, never rejected");
    run_to_rest(&pipe, png).await;
    let node = store.get_node(png).await.expect("node");
    assert_eq!(node.intake_status, IntakeStatus::Unsupported);
    let notice = node.notice.expect("incompatibility notice surfaced");
    assert!(notice.contains("png"));
    assert_eq!(node.classification[0]["category"], "unclassified");
}

/// SC-N04 — the seam holds: after at-rest, no further stage dispatches
/// absent human invocation. The dispatcher's successor map simply ends.
#[tokio::test]
async fn sc_n04_seam_holds() {
    let Some(store) = store().await else { return };
    let pipe = IntakePipe::new(&store, temp_root()).expect("pipe");
    let node_id = pipe
        .commit_file("data.json", b"{\"k\": 1}\n")
        .await
        .expect("commit");
    run_to_rest(&pipe, node_id).await;

    let jobs_at_rest = store
        .list_jobs_by_input_ref(node_id)
        .await
        .expect("jobs")
        .len();
    assert_eq!(
        jobs_at_rest, 3,
        "raw_copy + normalize + classify, nothing more"
    );

    // The observation window: repeated ticks dispatch nothing, spawn nothing.
    let dispatcher = Dispatcher::new(&pipe);
    let scope = [node_id];
    for _ in 0..3 {
        let ran = dispatcher.tick_scoped(Some(&scope)).await.expect("tick");
        assert!(
            ran.is_empty(),
            "no dispatch beyond at-rest without a human act"
        );
    }
    let jobs_after = store
        .list_jobs_by_input_ref(node_id)
        .await
        .expect("jobs")
        .len();
    assert_eq!(
        jobs_after, jobs_at_rest,
        "zero jobs spawned across the window"
    );
}

/// SC-N05 — kill-and-restart at every stage boundary: fresh components
/// reconstruct from flags and job records alone, resume, and the end state
/// equals an uninterrupted run.
#[tokio::test]
async fn sc_n05_kill_and_restart_each_boundary() {
    let Some(store) = store().await else { return };
    let root = temp_root();
    let content: &[u8] = b"fn main() {}\n";

    // Control: uninterrupted run.
    let pipe = IntakePipe::new(&store, &root).expect("pipe");
    let control = pipe.commit_file("main.rs", content).await.expect("commit");
    run_to_rest(&pipe, control).await;
    let control_summary = node_summary(&store, control).await;
    let control_events = node_events(&store, control).await;

    // Boundary 1: killed after RAW_COPY flagged, before normalize.
    // Boundary 2: killed after NORMALIZE flagged, before classify.
    for ticks_before_kill in [0usize, 1] {
        // The doomed workers live in an inner scope; leaving it is the
        // "kill" — every in-memory pipe and dispatcher state is discarded.
        let (node, scope);
        {
            let pipe = IntakePipe::new(&store, &root).expect("pipe");
            node = pipe.commit_file("main.rs", content).await.expect("commit");
            scope = [node];
            let dispatcher = Dispatcher::new(&pipe);
            for _ in 0..ticks_before_kill {
                dispatcher
                    .tick_scoped(Some(&scope))
                    .await
                    .expect("pre-kill tick");
            }
        }

        // Restart: fresh supervisor rebuilds the index from the record alone.
        let supervisor = Supervisor::new(&store);
        let progress = supervisor.reconstruct(&[node]).await.expect("reconstruct");
        let expected_stages: Vec<String> = match ticks_before_kill {
            0 => vec![STAGE_RAW_COPY.to_string()],
            _ => vec![STAGE_NORMALIZE.to_string(), STAGE_RAW_COPY.to_string()],
        };
        assert_eq!(
            progress[0].stages_flagged, expected_stages,
            "index rebuilt from flags"
        );
        assert_eq!(
            progress[0].jobs_in_flight, 0,
            "no job died mid-stage at a boundary"
        );

        // Fresh pipe resumes from the flags.
        let pipe = IntakePipe::new(&store, &root).expect("resumed pipe");
        let dispatcher = Dispatcher::new(&pipe);
        for _ in 0..(2 - ticks_before_kill) {
            dispatcher
                .tick_scoped(Some(&scope))
                .await
                .expect("resume tick");
        }
        assert_eq!(
            node_summary(&store, node).await,
            control_summary,
            "boundary {ticks_before_kill}: end state equals uninterrupted run"
        );
        assert_eq!(
            node_events(&store, node).await,
            control_events,
            "boundary {ticks_before_kill}: same event history"
        );
    }
}

/// SC-N06 — derivative regeneration: discard the derivative, re-derive from
/// raw — provenance updates, nothing else changes, no data loss.
#[tokio::test]
async fn sc_n06_derivative_regeneration() {
    let Some(store) = store().await else { return };
    let pipe = IntakePipe::new(&store, temp_root()).expect("pipe");
    let node_id = pipe
        .commit_file("doc.md", b"line one\r\nline two\r\n")
        .await
        .expect("commit");
    run_to_rest(&pipe, node_id).await;

    let before = store.get_node(node_id).await.expect("node");
    let derivative_path = before.derivative_path.clone().expect("derivative exists");
    let derived_before =
        std::fs::read_to_string(pipe.data_root().join(&derivative_path)).expect("read derivative");
    assert_eq!(
        derived_before, "line one\nline two\n",
        "line endings standardized"
    );
    let events_before = node_events(&store, node_id).await.len();

    let after = pipe.renormalize(node_id).await.expect("renormalize");

    // Re-derived from raw: identical content, updated provenance, no loss.
    let derived_after =
        std::fs::read_to_string(pipe.data_root().join(&derivative_path)).expect("re-derived");
    assert_eq!(
        derived_after, derived_before,
        "derivative reproduced exactly from raw"
    );
    assert_eq!(after.derivative_sha256, before.derivative_sha256);
    assert_eq!(after.raw_sha256, before.raw_sha256, "atom untouched");
    assert_eq!(
        after.classification, before.classification,
        "nothing else changed"
    );
    assert_eq!(after.intake_status, IntakeStatus::Normalized);
    assert!(
        after.revision > before.revision,
        "the change is on the record"
    );
    let events_after = node_events(&store, node_id).await;
    assert_eq!(events_after.len(), events_before + 1, "provenance updated");
    assert_eq!(*events_after.last().expect("last"), LogEvent::Normalized);
}
