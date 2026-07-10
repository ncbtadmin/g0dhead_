//! The Bias Doctrine (Holy Standard §6.3) — philosophy with teeth.
//! Disclosure is the hard floor; skew is the per-output record; a sustained
//! pattern escalates to a standing warning with a petition-style terminal
//! answer. The order swore, for weights, never to let measurable drift go
//! silent; bias drift is the same category, and consistency demands the
//! same graduated treatment.

use crate::ConcordatError;
use godhead_schemas::SourceDraw;
use godhead_store::{Store, StoreError};

/// The single v1 bias pattern scope.
pub const BIAS_SCOPE: &str = "regular_teacher_bias";

/// Computes an output's skew: the canon-associated share of draws exceeds
/// `bias_skew_threshold` (§6.3). A matrix is canon-associated per its
/// disclosed flag (derived upstream: node-level canon fetch provenance or a
/// live Canonical Instruction binding).
#[must_use]
pub fn compute_skew(sources: &[SourceDraw], skew_threshold: f64) -> bool {
    // Checked folds: an adversarial draw census must saturate legibly, not
    // debug-panic mid-labor (the B1 aggravation; SC-E05's class).
    let total: i64 = sources
        .iter()
        .map(|s| s.draw_count.max(0))
        .try_fold(0i64, i64::checked_add)
        .unwrap_or(i64::MAX);
    if total == 0 {
        return false;
    }
    let canon: i64 = sources
        .iter()
        .filter(|s| s.canon_associated)
        .map(|s| s.draw_count.max(0))
        .try_fold(0i64, i64::checked_add)
        .unwrap_or(i64::MAX);
    #[allow(clippy::cast_precision_loss)] // draw counts are small
    let share = canon as f64 / total as f64;
    share > skew_threshold
}

/// Discloses a Regular Teacher output (§6.3) under the disclosing job's
/// identity — every write path carries an authenticated identity (XIII.1;
/// H3(3)): records its draws and skew, then evaluates the trailing-window
/// pattern and raises/keeps the standing warning. Returns whether the
/// output was skewed and whether a pattern warning stands.
pub async fn disclose_regular_output<S: Store>(
    store: &S,
    job_id: uuid::Uuid,
    instruction_ref: uuid::Uuid,
    sources: &[SourceDraw],
) -> Result<(bool, bool), ConcordatError> {
    // A malformed sovereign constant refuses, never a fabricated default —
    // a threshold the sovereign never set is a decision the sovereign never
    // made (Law II.2), and a bias detector guessing its own threshold is a
    // legibility failure inside the legibility module.
    let skew_threshold = store
        .get_config("bias_skew_threshold")
        .await?
        .value
        .as_f64()
        .ok_or_else(|| {
            StoreError::ValidationFailed("bias_skew_threshold is not a number (A.14)".into())
        })?;
    let window = store
        .get_config("bias_pattern_window")
        .await?
        .value
        .as_i64()
        .ok_or_else(|| {
            StoreError::ValidationFailed("bias_pattern_window is not an integer (A.14)".into())
        })?;
    let pattern_threshold = store
        .get_config("bias_pattern_threshold")
        .await?
        .value
        .as_f64()
        .ok_or_else(|| {
            StoreError::ValidationFailed("bias_pattern_threshold is not a number (A.14)".into())
        })?;

    let skewed = compute_skew(sources, skew_threshold);
    let share = store
        .record_regular_output(job_id, instruction_ref, sources, skewed, window)
        .await?;

    // Pattern escalation: a sustained share over the window escalates to a
    // standing warning — the same graduated-legibility machinery as weight
    // drift, not a new mechanism. A SILENCED scope is not re-raised.
    let state = store.bias_warning_state(BIAS_SCOPE).await?;
    let stands = if share > pattern_threshold {
        match state.as_deref() {
            Some("SILENCED") => false, // suppressed until the sovereign lifts it
            _ => {
                store.raise_bias_warning(job_id, BIAS_SCOPE).await?;
                true
            }
        }
    } else {
        state.as_deref() == Some("STANDING") || state.as_deref() == Some("ACKNOWLEDGED")
    };
    Ok((skewed, stands))
}
