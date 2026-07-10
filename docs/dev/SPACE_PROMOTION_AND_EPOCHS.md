# Space Promotion and Epochs

**Status:** Analysis-only; non-canonical and proposal-only until D4 returns.

- **Scope:** Embedding-space identity and lifecycle, generation storage, candidate policy and certification, change cursors, bounded cutover, atomic promotion, and model-change behavior.
- **Owning decisions:** D4; D1 for checkpoint placement; D7 for checkpoint authority.
- **Phase owner:** P1-A/P1-B for candidate construction and measurement; P2A for the minimum command substrate; the post-join checkpoint for promotion.
- **Criteria hooks:** SC-D01, SC-M01–SC-M06, the D4 activation criteria below, and P1's preregistered evaluation bands.
- **Amendment rows sourced:** D4 space, vector, candidate-policy, measurement, epoch, config-history, and taxonomy rows in [AMENDMENT_MATRIX.md](AMENDMENT_MATRIX.md).

This is the sole normative proposal home for candidate-space evaluation,
certification, activation, and promotion. Trial-cycle validity and
activation-time trial disposition live in
[TRIAL_AND_EVIDENCE.md](TRIAL_AND_EVIDENCE.md).

---

## 1. Embedding-space identity and lifecycle

An alias does not identify a geometry. `EmbeddingSpace` is immutable:

`space_id · provider · model name · exact revision/digest · dimensions · normalization rule · pooling rule · chunking/truncation policy · created_at`

The lexical floor is retroactively `space 0` (builtin, 256-dimensional, L2,
unigram+bigram). Lifecycle is:

`PREPARING → ACTIVE | ABANDONED` and `ACTIVE → RETIRED`.

A PREPARING generation backfills eligible nodes and builds indexes while the
prior generation remains ACTIVE. A failed model, corrupt generation, or failed
certification marks the generation `FAILED` with machine attribution. The
space does not become ABANDONED automatically; R20 is a fresh sovereign act.
If failure occurs before the post-join authority substrate exists, the failed
candidate waits visibly for R20. A development pin is not authority for a
runtime decision made after observing failure.

The first P1-A candidate may be created by an exact, pinned seed migration.
That bootstrap is not the runtime R18 command. Later candidate adoption,
including a geometry-changing `adjust`, uses R18 through the authority
perimeter. This separates migration authorship from contingent sovereign acts.

## 2. Three-generation storage and propagation

Storage is **active typed generation + preparing typed generation + retired
archive**. Active and preparing generations are fully typed and indexed for
their dimensions. Retired vectors retain `(space_ref, node_ref, raw vector
bytes, envelope)` for audit and explanation but are non-computational.
Destruction terms remain sovereign under A.14(c).

Candidate qualification and weight rows are written directly into the
PREPARING generation's production-shaped tables. They are non-authoritative
because every product reader predicates on the ACTIVE catalog pointer—not
because they live in a disposable harness table. Activation therefore switches
visibility; it never copies candidate rows after cutover.

`space_ref` propagates to embeddings; `BondQualification`; `WeightEvidence`;
weight-recalculation and consolidation provenance; matrices at emergence; the
Cardinal's tried evidence identity; and `explain_link` views. Qualification and
weight records also carry the lineage epoch/effective revision required by
[OVERRIDE_LINEAGES.md](OVERRIDE_LINEAGES.md). Readers never combine geometry,
qualification, or weight evidence from different spaces in one calculation.

## 3. Candidate policy is an immutable staged object

P1-B must evaluate a PREPARING space under policy values that do not yet govern
space 0. The mechanism is a persisted, immutable **`CandidatePolicySnapshot`**:

`candidate_policy_ref · preregistration_revision · candidate_space_ref · exact config values · policy/algorithm versions · created_at · digest`

Every candidate density, qualification, weight, and emergence evaluation cites
that snapshot. Candidate jobs may not read private literals or write active
`config_constants`. Product jobs continue citing the one active ConfigHistory
revision.

This requires a named VI.1/A.14/SC-D01 amendment: a job operating exclusively
inside a PREPARING candidate context may cite an immutable candidate snapshot;
only R19 can promote it. Without that amendment the harness would contradict
VI.1's single revisioned threshold rule and become shadow canon.

`CandidateEvaluationContext` binds the PREPARING space, candidate policy,
candidate generation, production model adapter, and evaluation register
revision. Its read models remain invisible to production consumers. An
in-process model engine is contingency-only; any result used for a verdict must
be replicated on the production serving path.

## 4. Change cursors, scoped validity, and progress

Pass 6's singleton `GraphEpoch` is safe as a change detector but unsound as a
universal validity token: continuous unrelated intake can starve activation and
invalidate every human-paced trial. Pass 7 separates roles:

- **`GraphEpoch` is a transactional global change cursor.** Every
  graph-affecting write advances it inside that write's transaction. It supports
  ordered delta discovery, explanation, and catch-up accounting.
- **Candidate certification uses a candidate cursor and bounded delta set.**
  It catches up changes relevant to the candidate generation.
- **Trial validity uses dependency-scoped identities**, defined in
  [TRIAL_AND_EVIDENCE.md](TRIAL_AND_EVIDENCE.md), not equality with the global
  cursor.
- **Activation binds catalog, policy, candidate-certification, and trial-state
  revisions.** It does not classify child trial state from a matrix CAS alone.

ADR-2 chooses the physical scoped-token shape—per-space/category epochs,
dependency digests, or equivalent—but must preserve these observable
boundaries. It must also price contention on the global cursor row; an
implementation may replace the singleton with an equivalent transactional
sequence if the semantics remain.

### Bounded cutover under continuous intake

Activation must eventually complete under sustained bounded intake. The
recommended algorithm is:

1. certify a candidate at cursor E;
2. repeatedly apply relevant deltas after E while normal writers continue;
3. when lag falls below a pinned bound, acquire a **short, fair final writer
   micro-fence** for affected scopes only;
4. drain the remaining delta, certify the exact final state, and execute R19's
   transaction; and
5. release the fence whether promotion commits or refuses.

This is not a blanket freeze for the certification window. ADR-2 may instead
choose scoped epochs/writer classification if it proves the same safety and
liveness. Criteria pin maximum catch-up lag, bounded retries, maximum fence
duration, fairness, and a surfaced `STALE_CERTIFICATION` refusal rather than an
unbounded loop.

## 5. Emergence and certification identity

`emerge_postulant` must evaluate and insert against one coherent state. It
either performs density/membership evaluation and insertion in one suitable
snapshot transaction, or captures the expected candidate policy, space,
catalog, and scoped graph cursor before evaluation and CAS-validates all of
them at insertion. Merely checking catalog identity after earlier reads is not
enough. The matrix records the source cursor it evaluated, distinct from the
cursor advance caused by its own insertion.

Before promotion, P1-A/P1-B produce an immutable **`CandidateCertification`**:

`certification_ref · candidate_space_ref · candidate_policy_ref · generation_digest · coverage_digest · index_digest · source_cursor · applied_delta_high_water · trial_state_high_water · status · certified_at`

Certification proves complete eligible coverage, built indexes, sealed output
digests, no live candidate writer below the cutover watermark, and the exact
Postulant/trial-state snapshot to be disposed. A failed certification is
machine-attributed; abandonment remains R20.

## 6. `proceed` is R19 alone

`proceed` is one digest-bound **R19 `promote_candidate` command**, one durable
receipt, and one final Store transaction. R09 remains the ordinary standalone
sovereign-config command and is not called by `proceed`.

R19's immutable `PromotionPlan` binds:

- candidate space and catalog revision;
- `CandidatePolicySnapshot` reference and digest;
- preregistration revision;
- `CandidateCertification` reference and every coverage/index/output digest;
- expected active config history and catalog pointer;
- expected live/scoped graph and trial-state revisions; and
- the exact complete Postulant disposition plan defined in
  [TRIAL_AND_EVIDENCE.md](TRIAL_AND_EVIDENCE.md).

The R19 transaction revalidates the plan; applies every disposition; appends
ConfigHistory and moves the active policy pointer; switches the catalog;
retires the displaced space; advances the relevant cursors; and writes the
attributed promotion log/terminal receipt. Its hash covers the entire plan, not
only the disposition map. Because the bundle includes the sovereign
`coherence_threshold`, R19 carries IV.4 config authority as well as D4
activation authority. Operational-tier values retain their tier; inclusion in
the stronger atomic command does not silently reclassify them.

P2A supplies the minimum authentication, capability, envelope, receipt, and
R19/R20 callable needed before this checkpoint. P2B expands the same substrate
to the complete registry. P2A exits only when an enrolled operator can invoke
R19/R20 end to end and unauthenticated, stale, hash-mismatched, and replayed
commands refuse before elevation.

## 7. Verdict routes

- `proceed`: execute the R19 plan above.
- policy-only `adjust`: keep the geometry PREPARING, create a new preregistration
  and `CandidatePolicySnapshot`, rerun qualification/weight/evaluation, and
  issue a new certification.
- geometry/model `adjust`: mark the old generation failed as applicable, wait
  for R20, and use runtime R18 to create a fresh candidate.
- `kill`: R20 abandons the candidate, space 0 remains ACTIVE, P2B stays blocked,
  and the roadmap returns to sovereign reconciliation.

The P1-B time-box is fixed at pin. Overrun is a finding, not an extension.

## 8. Model-change behavior

Human-untouched geometry state is derivative and regenerable. Human-held state
persists as fixed stars through the lineage mechanism. A professed Cardinal
remains frozen under new geometry; drift is advisory. The lawful replacement
is sovereign decommission followed by fresh emergence and trial. An in-place
Cardinal revision lifecycle would require a separate Dogma VI amendment and is
not proposed here.

## 9. Acceptance criteria and canonical costs

P1/ADR-2 criteria must prove at least:

- one vector per node per valid space and no mixed-space computation;
- PREPARING writes remain product-invisible until one catalog switch;
- candidate outputs require no post-switch copy;
- every candidate computation cites one immutable policy snapshot;
- emergence cannot insert from a stale evaluated state;
- candidate certification binds exact generation, policy, cursor, and trial
  snapshot identities;
- promotion completes under sustained bounded intake within pinned catch-up and
  fence limits;
- unrelated graph writes do not invalidate trials;
- R19 is one envelope/receipt/transaction and R09 is absent from `proceed`;
- R19 refuses any changed policy, certification, catalog, graph, trial, or
  disposition input;
- bootstrap seed adoption cannot authorize runtime abandonment; and
- Cardinals stay frozen while drift remains legible.

Canonical costs are itemized in [AMENDMENT_MATRIX.md](AMENDMENT_MATRIX.md):
per-space vectors; bond/measurement separation; space and matrix statuses;
candidate-policy scope; ConfigHistory; GraphEpoch/change-cursor semantics;
trial-cycle identity; and the required taxonomy events. Final DDL and the
physical catch-up/fence choice belong to ADR-2 only after D4 returns.
