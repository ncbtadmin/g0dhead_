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
| doc 03 §2.3 · doc 04 §4.2 | similarity, category, qualification, and weight leave the immutable bond payload for append-only evidence/effective-source records | D4/D8 (each decision owns only its fields) |
| doc 03 §2.3 · doc 04 §4.4 | remove the mutable `user_overridden` marker from the protected payload; derive held state from lineages or a separate projection | D8 |
| Dogma VI / A.9 · doc 03 §2.4 | `SUPERSEDED` (terminal) + space, trial-cycle, and evidence-set identity on the matrix | D4 |
| A.5 | `SPACE_ADOPTED · SPACE_ACTIVATED · SPACE_ABANDONED · SPACE_RETIRED · CANDIDATE_CERTIFIED · MATRIX_SUPERSEDED · CONFIG_CHANGED · TRIAL_SUPERSEDED · TRIAL_DISTRUSTED` | D4 |
| new A-series relation | append-only `ConfigHistory` (no prior-value recovery exists today) | D4 |
| VI.1 · A.14 · SC-D01 | PREPARING-only evaluation may cite an immutable `CandidatePolicySnapshot`; active jobs still cite the single active ConfigHistory revision; only R19 may promote the candidate snapshot | D4 |
| new A-series relations | immutable `CandidatePolicySnapshot`, `CandidateCertification`, and exact-hashed `PromotionPlan` | D4 |
| new A-series relation | transactional `GraphEpoch` as global change cursor; scoped candidate/trial validity tokens; bounded catch-up + fair final cutover requirement | D4 |
| Book II §2 · A.11 · SC-D04 · SC-D05 | append-only `TrialCycle`; `trial_cycle_ref` + `evidence_set_ref` on reports/proposal/barrier; one live cycle per matrix revision; uniqueness and barrier identity move to the cycle | D4 |
| A.4 | `STALE_TRIAL_EVIDENCE` terminal execution refusal | D4 |
| A.4 | `STALE_CERTIFICATION` promotion refusal | D4 |
| B.2 / HS §1.3d | persisted, attributed `SOVEREIGN_JUDGMENT` verdict record (Return stays immutable) | D7 |
| SC-I07b | standing-notice resolution persistence (acknowledge/silence answerable, R21) | D7 |
| A.4 | `UNSHIPPED_OPERATION` refusal reason | D7 |
| A.4 | `TARGET_RELEASED · TARGET_SUPERSEDED` terminal execution refusal reasons | D8 |
| A.7 | per-kind `OverrideLineage` epochs · released fallbacks · scoped `EffectiveSourceSelection` · immutable occurrences/resolutions/execution attempts · release and transition records | D8 |
| new A-series relations | `BondCategoryEvidence`, `EffectiveSourceSelection`, exact-hash `TransitionPlan`, `PetitionResolution`, and `PetitionExecutionAttempt` | D8 |
| IV.2 · SC-C02 · SC-C03 | immutable occurrence history; petition class ≠ successor kind; SILENCED scoped to the complete targeted lineage-state digest | D8 |
| IV.5 | four-kind release representation and as-stands selectors; every active closure and inactive effective-source supersession enumerated and hashed | D8 |
| IV.1 · SC-C01 | append-only/derived-effective-state reading + exact-hash sovereign `TransitionPlan` as lawful authority beside granted consent, for singleton and composite transitions | D8 |
| SC-C04 · SC-D10 | composite successor shape; Notary provenance links every closure in the plan | D8 |
| A.5 | `OVERRIDE_RELEASED` | D8 |
| A.12 | **deliberately unamended** — consent resolves through the immutable occurrence/plan | D8 (decision recorded) |
