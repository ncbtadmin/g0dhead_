//! Section B — Handoff (Law III). SC-B01 … SC-B04.

mod common;

use godhead_schemas::{FlagStatus, JobStatus};
use godhead_store::{Store, StoreError};

/// SC-B01 — a readiness flag is writable only after its certified outputs
/// exist and validate; flag-before-output is rejected.
#[tokio::test]
async fn flag_before_output_rejected() {
    let Some(store) = common::store().await else {
        return;
    };
    // No output at all.
    let job = common::running_job(&store).await;
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Written)
        .await
        .expect("to WRITTEN");
    let err = store
        .write_flag(job.job_id, &common::flag_draft(vec!["out".into()], vec![1]))
        .await
        .expect_err("flag certifying a nonexistent output");
    assert!(matches!(err, StoreError::ValidationFailed(_)), "got {err}");

    // Output exists but the certified revision is wrong: certification must
    // name the exact state that passed VALIDATE_OUT (III.2).
    let job = common::running_job(&store).await;
    let artifact = store
        .write_artifact(job.job_id, "out", &common::widget("alpha"))
        .await
        .expect("write");
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Written)
        .await
        .expect("to WRITTEN");
    let err = store
        .write_flag(
            job.job_id,
            &common::flag_draft(vec!["out".into()], vec![artifact.revision + 7]),
        )
        .await
        .expect_err("revision drift");
    assert!(matches!(err, StoreError::ValidationFailed(_)));
}

/// SC-B02 — a flag is testimony, the state is the witness: a reader finding
/// the underlying state invalid refuses FLAG_UNTRUSTED and sets DISTRUSTED.
#[tokio::test]
async fn distrusted_flag_on_invalid_state() {
    let Some(store) = common::store().await else {
        return;
    };
    let (job, flag) = common::flagged_job(&store).await;
    // Out-of-band corruption after flagging: payload no longer validates
    // (schema requires non-empty 'name'), revision left untouched.
    sqlx::query(
        "UPDATE artifacts SET payload = '{\"name\": \"\"}' WHERE job_id = $1 AND output_slot = 'out'",
    )
    .bind(job.job_id)
    .execute(store.raw_pool())
    .await
    .expect("out-of-band mutation");

    let reader = common::running_job(&store).await;
    let err = store
        .read_certified(reader.job_id, flag.flag_id)
        .await
        .expect_err("the witness contradicts the testimony");
    assert!(matches!(err, StoreError::FlagUntrusted(_)), "got {err}");

    let flag = store.get_flag(flag.flag_id).await.expect("re-read flag");
    assert_eq!(
        flag.status,
        FlagStatus::Distrusted,
        "reader set the flag DISTRUSTED"
    );
}

/// SC-B03 — flags are never deleted: the substrate rejects deletion; change
/// happens only by one-way status supersession.
#[tokio::test]
async fn flag_deletion_rejected() {
    let Some(store) = common::store().await else {
        return;
    };
    let (_job, flag) = common::flagged_job(&store).await;

    let deletion = sqlx::query("DELETE FROM readiness_flags WHERE flag_id = $1")
        .bind(flag.flag_id)
        .execute(store.raw_pool())
        .await;
    assert!(
        deletion.is_err(),
        "deletion must be rejected at the substrate"
    );

    // Supersession works, exactly once, one-way.
    let flag = store
        .supersede_flag(flag.flag_id, flag.revision, FlagStatus::Consumed)
        .await
        .expect("ACTIVE -> CONSUMED");
    assert_eq!(flag.status, FlagStatus::Consumed);
    let err = store
        .supersede_flag(flag.flag_id, flag.revision, FlagStatus::Superseded)
        .await
        .expect_err("supersession is one-way from ACTIVE only");
    assert!(matches!(err, StoreError::ValidationFailed(_)));
}

/// SC-B04 — architectural: no API exists by which one agent process
/// addresses another; the store interface is the sole inter-agent surface.
/// The witness is the source itself: godhead-store exposes no IPC, socket,
/// or channel primitive, and its public surface is the sanctioned module set.
/// Widened per ruling G3: the IPC needle sweep now also runs workspace-wide
/// over DISCOVERED crates in arch_walls.rs::sc_b04_workspace_ipc_scan, which
/// subsumes this test's store-only sweep; this test stands (tests only
/// accumulate) and keeps the sanctioned-module assertion, which is its own.
#[test]
fn arch_no_agent_channel() {
    let src_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
    let forbidden = [
        "TcpListener",
        "TcpStream",
        "UdpSocket",
        "UnixListener",
        "UnixStream",
        "mpsc::channel",
        "broadcast::channel",
        "watch::channel",
        "std::process::Command",
    ];
    let sanctioned_mods = ["error", "interface", "postgres", "secrets", "types"];

    let mut lib_mods = Vec::new();
    for entry in std::fs::read_dir(src_dir).expect("read src") {
        let path = entry.expect("entry").path();
        let text = std::fs::read_to_string(&path).expect("read source file");
        for needle in forbidden {
            assert!(
                !text.contains(needle),
                "{} contains forbidden inter-process primitive '{needle}' (Law III.1)",
                path.display()
            );
        }
        if path.file_name().is_some_and(|n| n == "lib.rs") {
            for line in text.lines() {
                if let Some(name) = line.trim().strip_prefix("pub mod ") {
                    lib_mods.push(name.trim_end_matches(';').to_string());
                }
            }
        }
    }
    lib_mods.sort();
    assert_eq!(
        lib_mods,
        sanctioned_mods
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        "the public surface is exactly the sanctioned module set"
    );
}
