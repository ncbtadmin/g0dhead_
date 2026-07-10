# Proposed Canonical Amendment Matrix

**Status:** Analysis-only and proposal-only; no row is amended or authorized until its owning decision returns and the amendment process passes it.

- **Scope:** Literal canon, criteria-register, taxonomy, and canonical-schema changes required by the proposed mechanisms.
- **Owning decisions:** D4, D7, and D8, independently; adopting one never ratifies another's rows.
- **Phase owner:** The phase named by each source mechanism; extraction into ADRs occurs only after decisions return.
- **Criteria hooks:** The criteria named in each row and its owning annex.
- **Amendment rows sourced:** This file is the current normative proposal matrix. Pass-6's historical copy remains verbatim in [REVIEW_LEDGER.md](REVIEW_LEDGER.md).

Every current row must be sourced by exactly one mechanism annex. Historical
matrices in the ledger are evidence, not current definitions.

---
**The canonical amendment matrix (all proposal-only until the amendment
process passes each; adopting one decision never ratifies another's item):**

| Target | Amendment | Owner |
|--------|-----------|-------|
| doc 03 §2.2 · doc 04 §4.1 · SC-M05 | one vector per node → per node *per valid space* | D4 |
| doc 03 §2.3 · doc 04 §4.2 | similarity/weight leave the link record for space-keyed evidence | D4 |
| Dogma VI / A.9 · doc 03 §2.4 | `SUPERSEDED` (terminal) + space, trial-cycle, and evidence-set identity on the matrix | D4 |
| A.5 | `SPACE_ADOPTED · SPACE_ACTIVATED · SPACE_ABANDONED · SPACE_RETIRED · MATRIX_SUPERSEDED · CONFIG_CHANGED · TRIAL_SUPERSEDED · TRIAL_DISTRUSTED` | D4 |
| new A-series relation | append-only `ConfigHistory` (no prior-value recovery exists today) | D4 |
| new A-series relation | `GraphEpoch` control row + enumerated participating writes | D4 |
| Book II §2 · A.11 · SC-D04 · SC-D05 | append-only `TrialCycle`; `trial_cycle_ref` + `evidence_set_ref` on reports/proposal/barrier; one live cycle per matrix revision; uniqueness and barrier identity move to the cycle | D4 |
| A.4 | `STALE_TRIAL_EVIDENCE` terminal execution refusal | D4 |
| B.2 / HS §1.3d | persisted, attributed `SOVEREIGN_JUDGMENT` verdict record (Return stays immutable) | D7 |
| SC-I07b | standing-notice resolution persistence (acknowledge/silence answerable, R21) | D7 |
| A.4 | `UNSHIPPED_OPERATION` refusal reason | D7 |
| A.4 | `TARGET_RELEASED · TARGET_SUPERSEDED` refusal reasons | D8 |
| A.7 | per-kind lineages · `OverrideLineage` epochs · immutable `PetitionOccurrence`s · target/epoch binding · release records | D8 |
| IV.2 · SC-C02 · SC-C03 | occurrence immutability; petition class ≠ successor kind; SILENCED scoped to the silenced head epoch | D8 |
| IV.5 | bundled releases: every auto-closure enumerated, hashed, and seen at consent | D8 |
| SC-C01 | derived-effective-state reading + exact-hash sovereign `TransitionPlan` as lawful authority | D8 |
| SC-C04 · SC-D10 | composite successor shape; Notary provenance links every closure in the plan | D8 |
| A.5 | `OVERRIDE_RELEASED` | D8 |
| A.12 | **deliberately unamended** — consent resolves through the immutable occurrence/plan | D8 (decision recorded) |
