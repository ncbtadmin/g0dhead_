//! The rebalance trigger machinery (doc 4 §5.2, §6.4). Ingestion marks
//! *eligibility* (the store does that on every embed); *execution* happens
//! only per a trigger the user has chosen or issued. A user-configured
//! standing trigger is standing consent, not system initiative. Nothing
//! here recalculates on its own.
//!
//! Concurrency discipline: eligibility is CLAIMED atomically before a pass
//! runs (one claim wins among racing executors), never cleared after it —
//! so an ingestion arriving mid-pass keeps its mark, and a failed pass
//! restores the mark it consumed. No ingestion event is ever lost.
//!
//! SC-C07 ledger claim: "invoking rebalance outside a user-configured
//! trigger" — `rebalance_now` takes a human actor string; `rebalance_tick`
//! executes only what `rebalance_trigger` (human-administered config)
//! standing-consents to. No job-identity path to execution exists.

use crate::aggregate::{consolidate, ConsolidateSummary};
use crate::roster::Roster;
use crate::MlError;
use godhead_store::{Store, StoreError};
use uuid::Uuid;

/// What one trigger evaluation did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebalanceOutcome {
    /// The trigger did not fire (manual mode, not eligible, interval not
    /// yet elapsed, or another executor claimed the mark first). Nothing
    /// was recalculated.
    NotTriggered,
    /// Recalculation executed under a claimed mark.
    Executed(ConsolidateSummary),
}

/// The sovereign coherence-threshold revision this execution cites — or
/// None when the sovereign has never set one. A null citation is honest;
/// a fabricated revision is not (Law VI.1: cite what was read, only what
/// was read).
async fn threshold_citation<S: Store>(store: &S) -> Option<i32> {
    store
        .get_config("coherence_threshold")
        .await
        .ok()
        .map(|c| c.revision)
}

/// Claims the mark (if any), runs the pass, and restores the mark on
/// failure — a failed labor never consumes an ingestion event.
async fn execute<S: Store>(
    store: &S,
    roster: &Roster,
    category: &str,
    scope: &[Uuid],
) -> Result<(ConsolidateSummary, bool), MlError> {
    let citation = threshold_citation(store).await;
    let claimed = store
        .claim_rebalance_eligibility(category, citation)
        .await?;
    match consolidate(store, roster, category, scope).await {
        Ok(summary) => Ok((summary, claimed)),
        Err(err) => {
            if claimed {
                let _ = store.mark_rebalance_eligible(category).await;
            }
            Err(err)
        }
    }
}

/// The direct human act: "rebalance now" (doc 4 §6.4). Executes whether or
/// not the category is marked — the sovereign does not queue behind the
/// system's bookkeeping — consuming the pending mark if one exists. The
/// actor string is the signature-level guarantee that no agent path exists.
pub async fn rebalance_now<S: Store>(
    store: &S,
    roster: &Roster,
    _actor: &str,
    category: &str,
    scope: &[Uuid],
) -> Result<ConsolidateSummary, MlError> {
    let (summary, _claimed) = execute(store, roster, category, scope).await?;
    Ok(summary)
}

/// One evaluation of the configured standing trigger for a category:
/// - `{"kind":"manual"}` — never fires here; only `rebalance_now` executes.
/// - `{"kind":"on_add"}` — fires when the category is marked eligible.
/// - `{"kind":"interval","ms":N}` — fires when eligible AND N ms have
///   passed since the last recalculation (or none ever ran).
///
/// Firing claims the mark atomically: of N racing ticks, one executes and
/// the rest report NotTriggered.
pub async fn rebalance_tick<S: Store>(
    store: &S,
    roster: &Roster,
    category: &str,
    scope: &[Uuid],
) -> Result<RebalanceOutcome, MlError> {
    let trigger = store.get_config("rebalance_trigger").await?;
    let kind = trigger
        .value
        .get("kind")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            StoreError::ValidationFailed("rebalance_trigger must carry a 'kind'".into())
        })?;

    let state = store.rebalance_state(category).await?;
    let eligible = state.as_ref().is_some_and(|s| s.eligible);
    if !eligible {
        return Ok(RebalanceOutcome::NotTriggered);
    }

    let fires = match kind {
        "manual" => false,
        "on_add" => true,
        "interval" => {
            let interval_ms = trigger
                .value
                .get("ms")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| {
                    StoreError::ValidationFailed(
                        "interval rebalance_trigger must carry integer 'ms'".into(),
                    )
                })?;
            match state.as_ref().and_then(|s| s.last_recalc_at) {
                None => true, // eligible and never recalculated
                Some(last) => {
                    // Elapsed is judged store-stamp against store clock —
                    // the only clock (Law XII).
                    let now = store.store_now().await?;
                    (now - last).whole_milliseconds() >= i128::from(interval_ms)
                }
            }
        }
        other => {
            return Err(StoreError::ValidationFailed(format!(
                "unknown rebalance_trigger kind '{other}'"
            ))
            .into())
        }
    };
    if !fires {
        return Ok(RebalanceOutcome::NotTriggered);
    }
    // The standing trigger is standing consent (doc 4 §5.2). The claim is
    // the race arbiter: a competing executor that consumed the mark since
    // our read leaves nothing to execute.
    let citation = threshold_citation(store).await;
    if !store
        .claim_rebalance_eligibility(category, citation)
        .await?
    {
        return Ok(RebalanceOutcome::NotTriggered);
    }
    match consolidate(store, roster, category, scope).await {
        Ok(summary) => Ok(RebalanceOutcome::Executed(summary)),
        Err(err) => {
            // The claimed mark is restored: a failed pass never consumes
            // the ingestion event that summoned it.
            let _ = store.mark_rebalance_eligible(category).await;
            Err(err)
        }
    }
}
