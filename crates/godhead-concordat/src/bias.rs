//! The Bias Doctrine (Holy Standard §6.3) — philosophy with teeth.
//! Disclosure is the hard floor; skew is the per-output record; a sustained
//! pattern escalates to a standing warning with a petition-style terminal
//! answer. The order swore, for weights, never to let measurable drift go
//! silent; bias drift is the same category, and consistency demands the
//! same graduated treatment.

use crate::ConcordatError;
use godhead_schemas::SourceDraw;
use godhead_store::Store;

/// The single v1 bias pattern scope.
pub const BIAS_SCOPE: &str = "regular_teacher_bias";

/// Computes an output's skew: the canon-associated share of draws exceeds
/// `bias_skew_threshold` (§6.3). A matrix is canon-associated per its
/// disclosed flag (derived upstream: node-level canon fetch provenance or a
/// live Canonical Instruction binding).
#[must_use]
pub fn compute_skew(sources: &[SourceDraw], skew_threshold: f64) -> bool {
    let total: i64 = sources.iter().map(|s| s.draw_count.max(0)).sum();
    if total == 0 {
        return false;
    }
    let canon: i64 = sources
        .iter()
        .filter(|s| s.canon_associated)
        .map(|s| s.draw_count.max(0))
        .sum();
    #[allow(clippy::cast_precision_loss)] // draw counts are small
    let share = canon as f64 / total as f64;
    share > skew_threshold
}

/// Discloses a Regular Teacher output (§6.3): records its draws and skew,
/// then evaluates the trailing-window pattern and raises/keeps the standing
/// warning. Returns whether the output was skewed and whether a pattern
/// warning stands.
pub async fn disclose_regular_output<S: Store>(
    store: &S,
    instruction_ref: uuid::Uuid,
    sources: &[SourceDraw],
) -> Result<(bool, bool), ConcordatError> {
    let skew_threshold = store
        .get_config("bias_skew_threshold")
        .await?
        .value
        .as_f64()
        .unwrap_or(0.50);
    let window = store
        .get_config("bias_pattern_window")
        .await?
        .value
        .as_i64()
        .unwrap_or(20);
    let pattern_threshold = store
        .get_config("bias_pattern_threshold")
        .await?
        .value
        .as_f64()
        .unwrap_or(0.60);

    let skewed = compute_skew(sources, skew_threshold);
    let share = store
        .record_regular_output(instruction_ref, sources, skewed, window)
        .await?;

    // Pattern escalation: a sustained share over the window escalates to a
    // standing warning — the same graduated-legibility machinery as weight
    // drift, not a new mechanism. A SILENCED scope is not re-raised.
    let state = store.bias_warning_state(BIAS_SCOPE).await?;
    let stands = if share > pattern_threshold {
        match state.as_deref() {
            Some("SILENCED") => false, // suppressed until the sovereign lifts it
            _ => {
                store.raise_bias_warning(BIAS_SCOPE).await?;
                true
            }
        }
    } else {
        state.as_deref() == Some("STANDING") || state.as_deref() == Some("ACKNOWLEDGED")
    };
    Ok((skewed, stands))
}
