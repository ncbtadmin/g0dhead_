//! Stewardship around fixed stars (Handbook §4.5): a Devout consolidation
//! treats `user_overridden` state as immovable — it works around, petitions
//! when it believes the star is wrong, and never writes through. The
//! store's own override enforcement (slices 3–4) remains the wall beneath
//! this courtesy.

use crate::StudentError;
use godhead_schemas::{OverrideKind, PetitionDraft};
use godhead_store::{Store, StoreError};
use uuid::Uuid;

/// What the consolidation did, node by node: written toward the target,
/// petitioned (human-held, worked around), or already there.
#[derive(Debug, Clone, Default)]
pub struct ConsolidationReport {
    pub consolidated: Vec<Uuid>,
    /// (node, petition) — the steward's only voice on human-held data.
    pub petitioned: Vec<(Uuid, Uuid)>,
    pub untouched: Vec<Uuid>,
}

fn petition_for(node_id: Uuid, category: &str, proposed: &serde_json::Value) -> PetitionDraft {
    PetitionDraft {
        subject_ref: node_id,
        change_kind: OverrideKind::CategoryReassigned,
        reason: format!("stewardship consolidation proposes category '{category}' (§4.5)"),
        evidence_refs: vec![],
        proposed_change: proposed.clone(),
    }
}

/// Consolidates the scoped nodes' classifications toward one category. A
/// node under the sovereign's hand is left exactly as laid: the steward
/// petitions and moves on. Everything it did is in the report — nothing
/// silent, in either direction.
pub async fn steward_consolidate<S: Store>(
    store: &S,
    job_id: Uuid,
    scope: &[Uuid],
    category: &str,
) -> Result<ConsolidationReport, StudentError> {
    let proposed = serde_json::json!([{ "category": category }]);
    let mut report = ConsolidationReport::default();
    for &node_id in scope {
        let mut node = store.get_node(node_id).await?;
        // A sovereign override that lands mid-consolidation surfaces from
        // the CAS as StaleRevision (the override bumps the node's
        // revision), not only as OverrideConflict — so the work-around
        // loop re-reads and re-checks rather than aborting the labor and
        // losing the report (§4.5: nothing silent, in either direction).
        let mut rereads = 0;
        loop {
            let current = node
                .classification
                .get(0)
                .and_then(|c| c.get("category"))
                .and_then(|v| v.as_str());
            if current == Some(category) {
                report.untouched.push(node_id);
                break;
            }
            // A fixed star: the sovereign's hand stands on this datum.
            // Never through — petition and work around (IV.2, §4.5).
            if store.get_active_override(node_id).await?.is_some() {
                let petition = store
                    .open_petition(job_id, &petition_for(node_id, category, &proposed))
                    .await?;
                report.petitioned.push((node_id, petition.petition_id));
                break;
            }
            // The store's enforcement is the wall beneath the check above.
            match store
                .set_node_classification(job_id, node_id, node.revision, &proposed)
                .await
            {
                Ok(_) => {
                    report.consolidated.push(node_id);
                    break;
                }
                Err(StoreError::OverrideConflict(_)) => {
                    let petition = store
                        .open_petition(job_id, &petition_for(node_id, category, &proposed))
                        .await?;
                    report.petitioned.push((node_id, petition.petition_id));
                    break;
                }
                Err(StoreError::StaleRevision { .. }) if rereads < 3 => {
                    rereads += 1;
                    node = store.get_node(node_id).await?;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
    Ok(report)
}
