//! The Aggregator's consolidation labor (Dogma Book II §3, steps 1–2 of
//! three): links from vector proximity, weights per the dial-able mode.
//! Step 3 — Postulant emergence on threshold crossing — records the
//! Postulant and writes the audit-eligibility flag (slice 5).

use crate::roster::{Reasoner, Roster};
use crate::MlError;
use godhead_schemas::{
    AgentType, Budgets, Certifies, FlagDraft, JobStatus, Law, LinkRecord, RefusalDraft,
    RefusalReason, Validator,
};
use godhead_store::{ArtifactDraft, Store, StoreError};
use semver::Version;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// The stage a consolidation pass flags.
pub const STAGE_CONSOLIDATE: &str = "aggregator:consolidate";
/// The audit-eligibility flag written on Postulant emergence (Law VI.2).
pub const STAGE_AUDIT_ELIGIBLE: &str = "aggregator:audit_eligible";
/// The schema of its summary artifact.
pub const CONSOLIDATE_RESULT_SCHEMA: &str = "aggregator.consolidate_result";
/// The schema of the emergence artifact.
pub const EMERGENCE_SCHEMA: &str = "aggregator.emergence";

/// What one pass did (returned to the caller and recorded in the artifact).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsolidateSummary {
    pub links_touched: usize,
    pub weights_set: usize,
    pub reasoner_calls: usize,
    /// The Postulant this pass's evaluation recorded, if density crossed.
    pub emerged: Option<Uuid>,
}

fn aggregator_draft(scope: &[Uuid], reasoner_alias: Option<&str>) -> godhead_schemas::JobDraft {
    godhead_schemas::JobDraft {
        agent_type: AgentType::Aggregator,
        auditor_name: None,
        tier: None,
        // The assigned scope is the job's input — attributable provenance.
        input_refs: scope.to_vec(),
        env_ref: None,
        brief_ref: None,
        endpoint_alias: reasoner_alias.map(str::to_string),
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 300_000,
            max_tool_calls: 1000,
            max_tokens: if reasoner_alias.is_some() { 100_000 } else { 1 },
        },
    }
}

/// The floor weight formula (resolves doc 4 §5.3's open marker):
/// degree-normalized similarity, `w = sim / √(deg(a) · deg(b))`.
/// A link between promiscuously-connected nodes carries less distinct
/// influence than the same similarity between otherwise-quiet nodes —
/// size and count measured, never obeyed (doc 1 §2.4).
fn floor_weight(link: &LinkRecord, degree: &HashMap<Uuid, usize>) -> f32 {
    let da = degree.get(&link.source_ref).copied().unwrap_or(1).max(1);
    let db = degree.get(&link.target_ref).copied().unwrap_or(1).max(1);
    #[allow(clippy::cast_precision_loss)] // node degrees are tiny vs f32 range
    let norm = ((da * db) as f32).sqrt();
    link.similarity / norm
}

/// One consolidation pass over the Aggregator's assigned scope (Book II §3
/// — the scope is always assigned, never ambient):
/// 1. draw links between embedded scope nodes above
///    `link_similarity_threshold`;
/// 2. recalculate weights — the floor formula, times a reasoner multiplier
///    in assisted mode. Assisted with no rostered reasoner degrades to the
///    floor (doc 4 §2.4). Human-held links are worked around, never
///    through (Handbook §4.5).
pub async fn consolidate<S: Store>(
    store: &S,
    roster: &Roster,
    category: &str,
    scope: &[Uuid],
) -> Result<ConsolidateSummary, MlError> {
    let sim_threshold = store.get_config("link_similarity_threshold").await?;
    let sim_threshold = sim_threshold.value.as_f64().ok_or_else(|| {
        StoreError::ValidationFailed("link_similarity_threshold must be numeric".into())
    })?;
    let mode = store.get_config("weight_mode").await?;
    let assisted = mode.value.as_str() == Some("assisted");
    let reasoner = if assisted {
        roster.reasoner(None)
    } else {
        None
    };

    let job = store
        .create_job(&aggregator_draft(scope, reasoner.map(|(a, _)| a)))
        .await?;
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await?;
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await?;

    // The labor, error-contained: any mid-pass failure ends in a Law VII
    // refusal — the Aggregator's job is never stranded live.
    #[allow(clippy::cast_possible_truncation)] // config threshold is a small scalar
    match run_pass(
        store,
        job.job_id,
        category,
        scope,
        sim_threshold as f32,
        reasoner.map(|(_, r)| r),
    )
    .await
    {
        Ok(summary) => Ok(summary),
        Err(err) => {
            // Budget exhaustion is refused by the store itself; a second
            // refusal would be terminal access. Refusal is best-effort —
            // the original defect is what the caller must see.
            if !matches!(err, MlError::Store(StoreError::BudgetExceeded(_))) {
                let (law, reason) = match &err {
                    MlError::Store(StoreError::LeaseConflict(_)) => {
                        (Law::XI, RefusalReason::LeaseConflict)
                    }
                    MlError::Store(StoreError::StaleRevision { .. }) => {
                        (Law::XI, RefusalReason::ValidationFailed)
                    }
                    MlError::Endpoint(_) => (Law::VIII, RefusalReason::ToolOutputInvalid),
                    _ => (Law::II, RefusalReason::ValidationFailed),
                };
                let _ = store
                    .refuse(
                        job.job_id,
                        &RefusalDraft {
                            law,
                            reason,
                            subject_refs: vec![category.to_string()],
                            detail: format!("consolidation pass could not complete: {err}"),
                            preserved_refs: vec![],
                        },
                    )
                    .await;
            }
            Err(err)
        }
    }
}

async fn run_pass<S: Store>(
    store: &S,
    job_id: Uuid,
    category: &str,
    scope: &[Uuid],
    sim_threshold: f32,
    reasoner: Option<&Arc<dyn Reasoner>>,
) -> Result<ConsolidateSummary, MlError> {
    // Step 1 — linking: vector proximity over the scope's embedded nodes.
    // Each undirected pair is drawn exactly once (canonical direction);
    // a draw that lands on a human-held link touches nothing and counts
    // for nothing.
    let mut links_touched = 0usize;
    for &node_id in scope {
        if store.get_embedding(node_id).await?.is_none() {
            continue; // unembedded nodes rest linkless until backfill
        }
        let neighbors = store
            .similar_nodes(node_id, sim_threshold, Some(scope))
            .await?;
        for (other, sim) in neighbors {
            if node_id >= other {
                continue; // the pair is drawn from its lesser endpoint only
            }
            let link = store
                .draw_link(job_id, node_id, other, sim, category)
                .await?;
            if !link.user_overridden {
                links_touched += 1;
            }
        }
    }

    // Step 2 — weights over the scope's links in this category.
    let links = store.links_by_category(category, Some(scope)).await?;
    let mut degree: HashMap<Uuid, usize> = HashMap::new();
    for link in &links {
        *degree.entry(link.source_ref).or_default() += 1;
        *degree.entry(link.target_ref).or_default() += 1;
    }
    let mut weights_set = 0usize;
    let mut reasoner_calls = 0usize;
    for link in &links {
        if link.user_overridden {
            continue; // fixed stars: worked around, never through
        }
        let mut weight = floor_weight(link, &degree);
        if let Some(r) = reasoner {
            let context = format!(
                "category {category}: similarity {:.3} between nodes of degree {} and {}",
                link.similarity,
                degree.get(&link.source_ref).copied().unwrap_or(1),
                degree.get(&link.target_ref).copied().unwrap_or(1),
            );
            let multiplier = r.weigh(&context).await?;
            reasoner_calls += 1;
            weight *= multiplier;
        }
        // Law XI.3 under concurrency: a stale revision loses and re-reads.
        // A parallel pass may have bumped the link; retry against fresh
        // state, and stand down entirely if a human hand landed mid-pass.
        let mut current = link.clone();
        let mut attempts = 0;
        loop {
            match store
                .set_link_weight(job_id, current.link_id, current.revision, weight)
                .await
            {
                Ok(_) => {
                    weights_set += 1;
                    break;
                }
                Err(StoreError::OverrideConflict(_)) => break,
                Err(StoreError::StaleRevision { .. }) if attempts < 3 => {
                    attempts += 1;
                    let fresh = store
                        .links_by_category(category, Some(scope))
                        .await?
                        .into_iter()
                        .find(|l| l.link_id == current.link_id);
                    match fresh {
                        Some(l) if !l.user_overridden => current = l,
                        _ => break, // gone or human-held: not this pass's to touch
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    // Step 3 — the threshold of form (Book II §3, Law VI.2): evaluate
    // density and, on crossing, record the Postulant. Skipped ONLY when
    // the sovereign has never set the threshold — no citation, no
    // evaluation, never a guess (Law VI.1). A transient config-read fault
    // fails the pass (→ the refusal wrapper), never a silent skip (VII.1).
    let emerged = match store.get_config("coherence_threshold").await {
        Ok(_) => {
            store
                .emerge_postulant(job_id, category, Some(scope))
                .await?
        }
        Err(StoreError::NotFound(_)) => None,
        Err(e) => return Err(e.into()),
    };

    // Close the lifecycle lawfully: artifacts, flags (one FLAG step,
    // possibly certifying two stages), terminate.
    let validator = Validator {
        id: "godhead-ml/registry".to_string(),
        version: "1.0.0".to_string(),
    };
    let artifact = store
        .write_artifact(
            job_id,
            "result",
            &ArtifactDraft {
                schema_name: CONSOLIDATE_RESULT_SCHEMA.to_string(),
                schema_version: Version::new(1, 0, 0),
                payload: serde_json::json!({
                    "category": category,
                    "links_touched": links_touched,
                    "weights_set": weights_set,
                    "reasoner_calls": reasoner_calls,
                    "mode": if reasoner.is_some() { "assisted" } else { "floor" },
                }),
            },
        )
        .await?;
    let mut flag_drafts = vec![FlagDraft {
        stage: STAGE_CONSOLIDATE.to_string(),
        certifies: Certifies {
            output_slots: vec!["result".to_string()],
            revisions: vec![artifact.revision],
        },
        validator: validator.clone(),
    }];
    if let Some(matrix) = &emerged {
        let emergence = store
            .write_artifact(
                job_id,
                "emergence",
                &ArtifactDraft {
                    schema_name: EMERGENCE_SCHEMA.to_string(),
                    schema_version: Version::new(1, 0, 0),
                    payload: serde_json::json!({
                        "matrix_id": matrix.matrix_id.to_string(),
                        "category": category,
                        "config_rev": matrix.config_rev,
                    }),
                },
            )
            .await?;
        // VI.2: the audit-eligibility readiness flag — the flag that opens
        // the human-invoked audit path.
        flag_drafts.push(FlagDraft {
            stage: STAGE_AUDIT_ELIGIBLE.to_string(),
            certifies: Certifies {
                output_slots: vec!["emergence".to_string()],
                revisions: vec![emergence.revision],
            },
            validator,
        });
    }
    let job = store.get_job(job_id).await?;
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Written)
        .await?;
    store.write_flags(job.job_id, &flag_drafts).await?;
    let job = store.get_job(job_id).await?;
    store
        .transition_job(job.job_id, job.revision, JobStatus::Terminated)
        .await?;

    Ok(ConsolidateSummary {
        links_touched,
        weights_set,
        reasoner_calls,
        emerged: emerged.map(|m| m.matrix_id),
    })
}
