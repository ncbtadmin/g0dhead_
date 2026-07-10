//! Slice 10 — the checked-budget-sum rider (SLICE_10.md §3; ruling G7 /
//! H2-B1). The B1 aggravation was a debug-panic strand: an adversarial
//! budget census could overflow the lint's fold and kill a RUNNING labor
//! instead of failing its clause. The fold is checked now — an overflowing
//! sum saturates legibly and fails clause (e), never a panic.

use godhead_concordat::{lint_instruction, write_instruction, ConcordatError};
use godhead_schemas::{
    AcceptanceCriterion, CapabilityAction, InstructionDraft, SchemaRegistry, SourceDraw, Step,
    TestableAs, Tier,
};
use godhead_store::PgStore;
use semver::Version;
use serde_json::json;
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
    let mut reg = SchemaRegistry::new();
    godhead_concordat::register_into(&mut reg);
    Some(
        PgStore::connect(&url, reg)
            .await
            .expect("store connect + migrate"),
    )
}

/// Two steps whose budget hints sum past i64::MAX — the adversarial
/// census. Everything else conforms, so the lint reaches clause (e).
fn overflowing_draft(marker: &str) -> InstructionDraft {
    let step = |id: i32| Step {
        step_id: id,
        action: CapabilityAction::Refine,
        params: json!({}),
        expected_output: "refined.doc@1.0".to_string(),
        budget_hint_tokens: i64::MAX / 2 + 1,
    };
    InstructionDraft {
        teacher_env_ref: None,
        teacher_tier: Tier::Regular,
        target_tier: Tier::Devout,
        concordat_version: Version::new(1, 0, 0),
        objective: marker.to_string(),
        steps: vec![step(1), step(2)],
        acceptance_criteria: vec![AcceptanceCriterion {
            criterion: "every refined artifact validates against its schema".to_string(),
            testable_as: TestableAs::Validation("schema_conformance".to_string()),
        }],
        sources_drawn: vec![SourceDraw {
            matrix_ref: Uuid::now_v7(),
            draw_count: 1,
            canon_associated: false,
        }],
        supersedes_ref: None,
    }
}

/// Checked budget sum (H2-B1; SC-E05's class) — a step census that would
/// overflow i64 saturates and fails lint clause (e) cleanly: the failure
/// is a legible contract failure, never a debug-panic mid-labor; the write
/// path refuses whole and persists nothing.
#[tokio::test]
async fn budget_sum_checked_refuses() {
    let Some(store) = store().await else { return };
    let marker = format!("e05 overflow census {}", Uuid::now_v7());
    let draft = overflowing_draft(&marker);

    // The lint completes (no panic — the fold is checked) and names (e).
    let failure = lint_instruction(&store, &draft)
        .await
        .expect("the lint runs to a verdict; an overflow never panics it")
        .expect_err("an over-ceiling census fails");
    assert_eq!(failure.clause, 'e', "the budget clause: {failure}");
    assert!(
        failure.detail.contains("exceeds"),
        "the saturated total is judged against the ceiling: {failure}"
    );

    // End to end: the write refuses whole; no partial Instruction persists.
    let err = write_instruction(&store, &draft).await;
    assert!(
        matches!(err, Err(ConcordatError::LintFailed(_))),
        "the write refuses on the linted clause: {err:?}"
    );
    let persisted: i64 =
        sqlx::query_scalar("SELECT count(*) FROM instructions WHERE objective = $1")
            .bind(&marker)
            .fetch_one(store.raw_pool())
            .await
            .expect("count");
    assert_eq!(persisted, 0, "a lint-refused Instruction never lands");
}
