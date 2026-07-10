# Proposed Canonical Amendment Matrix

**Status:** Analysis-only. `PROPOSED` rows remain unauthorized until their owning decision returns and the amendment process passes them. `SANCTIONED / PENDING DELIVERY` rows reflect the signed `ffae6a8` Slice 11b ruling but are not counted as delivered implementation.

- **Scope:** Literal canon, criteria-register, taxonomy, and canonical-schema changes required by the proposed mechanisms.
- **Owning decisions:** D4, D7, and D8 independently; adopting one never ratifies another's rows.
- **Phase owner:** The phase named by each source mechanism; ADR extraction occurs only after decisions return.
- **Criteria hooks:** The criteria named in each row and owning annex.
- **Amendment rows sourced:** This file is the current normative proposal matrix. Pass 6's historical copy remains verbatim in [REVIEW_LEDGER.md](REVIEW_LEDGER.md).

Every row is sourced by exactly one mechanism annex unless the owner cell
explicitly splits fields between decisions. Historical matrices in the ledger
are evidence, not current definitions.

---

| Target | Amendment | Owner | Status |
|---|---|---|---|
| doc 03 §2.2 · doc 04 §4.1 · SC-M05 | one vector per node → per node *per valid space* | D4 | PROPOSED |
| doc 03 §2.3 · doc 04 §4.2 | similarity, category, qualification, and weight leave the immutable bond payload for append-only evidence/effective-source records | D4/D8, each owning only its fields | PROPOSED |
| doc 03 §2.3 · doc 04 §4.4 | remove the mutable `user_overridden` marker from the protected payload; derive held state from lineages or a separate projection | D8 | PROPOSED |
| Dogma VI · A.9 · doc 03 §2.4 | `SUPERSEDED` terminal status plus space, trial-cycle, and evidence-set identity on matrices | D4 | PROPOSED |
| A.5 | `SPACE_ADOPTED · SPACE_ACTIVATED · SPACE_ABANDONED · SPACE_RETIRED · CANDIDATE_CERTIFIED · MATRIX_SUPERSEDED · CONFIG_CHANGED · TRIAL_SUPERSEDED · TRIAL_DISTRUSTED` | D4 | PROPOSED |
| new A-series relation | append-only `ConfigHistory`; active policy pointer | D4 | PROPOSED |
| VI.1 · A.14 · SC-D01 | PREPARING-only jobs may cite immutable `CandidatePolicySnapshot`; active jobs still cite one active ConfigHistory revision; only R19 promotes | D4 | PROPOSED |
| new A-series relations | immutable `CandidatePolicySnapshot`, `CandidateCertification`, and exact-hashed `PromotionPlan` | D4 | PROPOSED |
| new A-series relation | transactional `GraphEpoch` as change cursor; scoped validity tokens; bounded catch-up and fair cutover requirement | D4 | PROPOSED |
| Book II §2 · A.11 · SC-D04 · SC-D05 | append-only `TrialCycle`; cycle/evidence refs on reports, proposal, barrier; one live cycle per matrix revision; uniqueness and barrier identity move to cycle | D4 | PROPOSED |
| A.4 | `STALE_TRIAL_EVIDENCE` terminal execution refusal | D4 | PROPOSED |
| A.4 | `STALE_CERTIFICATION` promotion refusal | D4 | PROPOSED |
| new A-series relations | `CommandReceipt`, outbox/step state, and `CommandRefusal` with command/request identity | D7 | PROPOSED |
| A.4 | `UNSHIPPED_OPERATION` may cite command/request ref rather than fabricated job ID | D7 | PROPOSED |
| A.5 | `COMMAND_ACCEPTED · COMMAND_COMPLETED · ADMISSION_NOTICE_RESOLVED · BIAS_SCOPE_LIFTED · SOVEREIGN_JUDGMENT_RECORDED` | D7 | PROPOSED |
| B.2 · HS §1.3d | append-only, attributed `SOVEREIGN_JUDGMENT` review occurrence; Return remains immutable | D7 | PROPOSED |
| SC-I07b · A.12 | append-only `AdmissionNotice` occurrence, revision, scope digest, acknowledge/silence resolution, and lawful re-arm | D7 | PROPOSED |
| HS §6.3 · SC-K07 | bias-warning occurrence/lift lineage; SILENCED → LIFTED and lawful later re-arm | D7 | PROPOSED |
| A.10 · IX.5 · Holy Standard §§4–5 | initial persistent pairing is a sovereign act because it opens the Pairing Exception; Devout and Doctor commands are operation-specific | D7 | PROPOSED |
| IV.4 · A.8 · signed Slice 11b §0.2/§0.6 | `retire_environment` is the sole human lever from LIVE/ORPHANED to DISSOLVED | Slice 11b; D7 owns perimeter | SANCTIONED / PENDING DELIVERY |
| A.8 · Holy Standard §4.3 · signed Slice 11b §§0.1–0.5 | ORPHANED is dependency-loss only; no revival; replacement mints fresh records | Slice 11b; D7 owns sealing | SANCTIONED / PENDING DELIVERY |
| new A-series relation · A.10 · signed Slice 11b §0.4 | immutable Doctor deployment reference and one-to-one Doctor environment + deployment + `CANONICAL_INSTRUCTION` pairing invariant | Slice 11b; D7 owns command | SANCTIONED / PENDING DELIVERY |
| A.4 | `TARGET_RELEASED · TARGET_SUPERSEDED` terminal petition-execution refusal reasons | D8 | PROPOSED |
| A.7 | per-kind `OverrideLineage` epochs, released fallbacks, scoped `EffectiveSourceSelection`, immutable occurrences/resolutions/execution attempts, release and transition records | D8 | PROPOSED |
| new A-series relations | `BondCategoryEvidence`, `EffectiveSourceSelection`, exact-hash `TransitionPlan`, `PetitionResolution`, and `PetitionExecutionAttempt` | D8 | PROPOSED |
| IV.2 · SC-C02 · SC-C03 | immutable occurrence history; petition class ≠ successor kind; SILENCED binds the complete targeted lineage-state digest | D8 | PROPOSED |
| IV.5 | four-kind release representation and as-stands selectors; every active closure and inactive effective-source supersession is enumerated and hashed | D8 | PROPOSED |
| IV.1 · SC-C01 | append-only/derived-effective-state reading plus exact-hash sovereign `TransitionPlan` as authority beside granted consent, for singleton and composite transitions | D8 | PROPOSED |
| SC-C04 · SC-D10 | successor provenance links every closure, fallback supersession, consent/sovereign plan, and terminal attempt | D8 | PROPOSED |
| A.5 | `OVERRIDE_RELEASED` | D8 | PROPOSED |
| A.12 | deliberately unamended for override consent: immutable ConsentRecord resolves through immutable occurrence/plan | D8 | DECISION RECORDED |

## Post-Slice-11b reconciliation queue

The signed `ffae6a8` specification does not deliver migration 0019 or pin the
final A.5 event names `ENV_DISSOLVED` / `DOCTOR_DEPLOYED`. After Slice 11b lands,
the required rebase pass verifies the committed migration, schemas, tests,
taxonomy, and exact event names before adding them to delivered inventory or
this matrix. In-flight main-tree files are not evidence of delivery.

## Deliberate non-canonical application records

Client, session, token-family, and local-recovery records are P2A application
schema governed by ADR-3 and [AUTHORITY_REGISTRIES.md](AUTHORITY_REGISTRIES.md).
They are not promoted into Appendix A by this roadmap. If later behavior law
depends on their exact grammar, D5's promotion path applies.
