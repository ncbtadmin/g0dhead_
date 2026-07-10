# Space Promotion and Epochs

**Status:** Analysis-only; non-canonical and proposal-only until D4 returns.

- **Scope:** Embedding-space identity and lifecycle, storage generations, space propagation, certification and epoch serialization, candidate evaluation, checkpoint promotion, and model-change behavior.
- **Owning decisions:** D4; D1 for checkpoint placement; D7 for checkpoint authority.
- **Phase owner:** P1-A/P1-B for candidate construction and measurement; the post-join checkpoint for promotion.
- **Criteria hooks:** SC-M01â€“SC-M06, D4's proposed activation criteria, and the P1 preregistered evaluation bands.
- **Amendment rows sourced:** D4 space, vector, measurement, epoch, config-history, and taxonomy rows in [AMENDMENT_MATRIX.md](roadmap_reconciliation/AMENDMENT_MATRIX.md).

This is the sole normative proposal home for candidate-space evaluation,
certification, activation, and promotion mechanics.

---
## 10. Embedding-space proposal (D4b — separate from provider selection)

An alias does not identify a geometry. The policy makes embedding space a
first-class, immutable identity.

**EmbeddingSpace record**: `space_id` (immutable) · provider · model name +
exact revision/digest · dimensions · normalization rule · pooling rule ·
chunking/truncation policy · created_at. **The lexical floor is retroactively
`space 0`** (builtin, 256-dim, L2, unigram+bigram) — the current data already
inhabits a space; the policy names it.

**Lifecycle (pass-4 completion — the failure path was missing)**:
`PREPARING → ACTIVE | ABANDONED` and `ACTIVE → RETIRED`. A space is adopted by
a migration-class sovereign act; while PREPARING, the space-aware backfill
(generalizing `embedding_backlog`) embeds eligible nodes and builds indexes
while the prior space remains ACTIVE; activation occurs only when eligible
coverage is complete, indexes exist, and the regenerated derived graph is
certified ready. **A failed model, corrupt generation, or unsuccessful
certification marks the generation FAILED — machine-attributed, logged — and
the `ABANDONED` transition itself is the sovereign's act (R20, or the
checkpoint's adjust/kill routing): pass-6 authority unification — the
pass-5 prose had the machine abandoning while the registry reserved
abandonment to the sovereign; disposal terms stay with the sovereign
(A.14(c) posture)** — the preparing slot is never permanently occupied by
a failure, and never vacated by anyone but the hand. The displaced space becomes RETIRED on activation. **Postulant disposition at activation:** defined only in [TRIAL_AND_EVIDENCE.md](TRIAL_AND_EVIDENCE.md).
**Readers never combine embeddings, qualification, or weight evidence from
different geometries in one calculation** — every similarity, consolidation,
and density evaluation binds (space, policy revision) by predicate
(unreleased forced bonds excepted by design: geometry-neutral sovereign
facts, §3.1).

**Storage (revised pass 3 — the pass-2 recommendation had no PREPARING write
target):** **active typed generation + preparing typed generation + retired
archive.** Each *generation* is a typed, fully indexed table (or partition)
whose dimensions match its space; a PREPARING space backfills into its own
generation while ACTIVE serves; **activation atomically switches a catalog
pointer (or view) to the prepared generation** after coverage, indexing, and
graph certification; the displaced generation's vectors move to the retired
archive (space_ref, node_id, raw vector bytes, envelope) — retained (A.14(c):
destruction terms are the sovereign's), auditable, deliberately
non-computational. Final DDL belongs to ADR-2.

**Activation serialization and the certification epoch (pass-5 — the switch
must compose with live intake and the existing trial methods):** activation
is one serialized act that (i) **CAS-guards every affected matrix** on
`(matrix_id, revision, status)` — supersession increments the revision, and
`SUPERSEDED` is terminal, trigger-enforced the way 0006 freezes a professed
Cardinal; (ii) requires trial writes (report filing, proposal filing,
consent, execution) to predicate **atomically** on expected revision ∧
POSTULANT status ∧ originating `space_ref` = the active space — today's
guards check status and revision inside the method (`file_audit_report`
refuses non-Postulants and stale revisions) but carry no space predicate and
no supersession awareness; (iii) supersedes audit-eligibility and barrier
flags with the matrix, and disposes of outstanding jobs and consents per the
disposition rules above; (iv) binds graph certification to a
**transactional `GraphEpoch`** — **pass-6 correction: the pass-5 A.5-position
token was unsound as a concurrency primitive.** A.5's sequence is record
identity, not commit ordering; data writes and their `append_log` calls are
separate autocommit operations today (witness: `open_petition` inserts, then
logs, on the bare pool); and `set_config` writes **no A.5 event at all** — a
mutation can commit without moving the value activation would check. Chosen
shape (D4, of the two the finding offered): **a `GraphEpoch` row advanced
inside every graph-affecting write transaction**, participating writers
enumerated: nodes/classification; embeddings; bonds; `BondQualification`;
`WeightEvidence`; override lays, releases, and composite transitions;
graph-relevant config/policy changes; matrix emergence; space-catalog
changes. Certification records the epoch it certified; **the catalog switch
CAS-validates that epoch**: any intervening write refuses the switch, which
recertifies. The blanket-fence alternative is declined as primary (it stalls
all intake for the whole certification window) but remains ADR-2's
implementation fallback. A.5 stays the explanatory history — it is no longer
the concurrency token. **(v) Emergence serializes with activation (pass-6 —
the CAS on *found* matrices was not enough):** `emerge_postulant` today
carries no catalog predicate, so the Aggregator could evaluate the old
space, activation could dispose and switch, and the insert could then land a
new Postulant derived from a retired geometry — defeating "no old-space
Postulant survives." Every emergence insert therefore predicates atomically
on `(expected_active_space, expected_catalog_revision)` within the same
serialization unit as the switch, and the born matrix records its
`space_ref` and the exact `GraphEpoch` it emerged against. The invariant
this document fixes: **nothing lands unnoticed between certification and
switch, and nothing emerges past it**.

**`space_ref` propagation (pass-4: aligned to §3.1's records)** —
`config_rev` cites the threshold revision, not the geometry; the geometry
needs its own witness. `space_ref` lands on: embeddings (keyed
`(node_id, space_ref)` within generations); **qualification records**
(`BondQualification{bond_ref, space_ref, link_policy_rev, …}` — bonds
themselves are geometry-neutral, §3.1); **weight evidence**
(`WeightEvidence{bond_ref, space_ref, weight_policy_rev, …}` — weight never
rides the bond row, §3.1); weight-recalculation log payloads; consolidation
job provenance; matrices at emergence (the committed Cardinal's provenance
permanently names its originating space and the weight-evidence revision set
it was tried on, §3.1); `explain_link` read models. Similarity, density, and
aggregation queries bind `(space_ref, policy_rev)` by predicate. External
provenance chains (C.2) are orthogonal and unchanged.

**Trial evidence and validity:** defined only in [TRIAL_AND_EVIDENCE.md](TRIAL_AND_EVIDENCE.md).

**Model-change behavior** — the constitution answers most of it:
geometry-derived, human-untouched state is *derivative* (doc 03 §4.3):
disposable, regenerable under the new space. Human-held state persists per
Law IV — fixed stars in any geometry. **Professed Cardinals (pass-3
correction — the pass-2 wording was unlawful):** audit tries Postulants only
(Law VI.3; `file_audit_report` and the trial methods refuse non-Postulants;
the 0006 trigger freezes a professed Cardinal, and decommission is the only
door). A Cardinal therefore **remains frozen under new geometry**. Drift
between a Cardinal's committed structure and the active space's measurements
is surfaced as an **advisory drift signal through the existing graduated-
legibility machinery** — never an in-place amendment; the lawful response is
sovereign-invoked decommission followed by fresh emergence and a new trial
under the new space. If the sovereign ever wants an in-place post-commit
revision lifecycle, that is a Dogma VI amendment proposed through process —
explicitly *not* proposed here. Prior-space vectors are retained by default.

**Canonical amendments this policy entails (pass-4 — a reading is not an
amendment):** three texts literally say one vector per *node*, and
multi-space storage makes it one per *(node, space)*: **doc 03 §2.2** ("One
persisted vector per node… Never recomputed when it can be read"), **doc 04
§4.1** ("One persisted vector per node"), and **SC-M05** (doc 08 — whose
embedder call-count assertion a PREPARING backfill would fail as written).
Adopting D4b therefore adopts three named literal amendments, proposed
through the amendment process, rescoping each to *per node per valid space*
with read-never-recompute holding within a space; a deliberate space change
schedules re-embedding and never makes a stale vector authoritative. The
pass-3 "recorded reading" is retired: a reading that contradicts the
register's literal test is shadow canon (the very thing D5's anti-shadow
guard exists to forbid).

**Provider path (D4a, revised pass 2):** the production path is the canon's
own architecture — doc 04 §2.2's separate local inference process behind
`godhead-model-adapter` — **built in P1 as that phase's first deliverable**,
so all offline measurements run on the serving path the product will use.
In-process inference appears only as a named contingency (§8/P1). If the
sovereign ever prefers in-process as *architecture*, that is a deliberate
doc 04 §2.2 amendment, not proposed here.

## Candidate evaluation context

**the
`CandidateEvaluationContext` (pass-6 — P1-B must evaluate a PREPARING
candidate under a not-yet-adopted policy without pretending it is the
product graph):** a P1-A-owned evaluation harness context binding,
explicitly, the PREPARING space; a **staged policy snapshot** (candidate
thresholds and link policy — staged, not `set_config`-adopted); candidate
qualification and weight runs keyed to it; the candidate `GraphEpoch`;
**non-authoritative output tables/read models** (production consumers never
read them); and the **production model-serving adapter** (the serving path
stays real even while the graph is a candidate); the evaluation
harness (a dev binary). An in-process engine remains a named *contingency
only*, and contingency results must be replicated on the production path
before P1 exits.

## Sovereign checkpoint and verdict routing

**Verdict routing (pass-4; pass-5 names the sovereign checkpoint; pass-6
places it truthfully and makes `proceed` one act):** every verdict lands at
the **post-measurement sovereign policy/space checkpoint** — which **waits
for the P1-B ∥ P2A join (pass-6 ownership fix)**: the checkpoint's acts —
R09 threshold adoption and R19 activation, plus any post-measurement R20 —
require the sovereign capability and store-owned envelope substrate that
**P2A** owns; invoking them earlier on "recorded direct invocation under
the dev register" would be process discipline standing in for the D7
authority boundary. (R18's candidate adoption is different in kind and
does *not* wait: it is a migration-class act whose authority is the pinned
P1-A slice itself — §11.1 footnote. The other alternative — P1-A shipping
a minimum auth/envelope stack of its own — is declined as a duplicated
authority substrate; recorded.) `proceed` = **one atomic promotion act (pass-6 —
set-then-activate is unsound: writing `coherence_threshold` first would
advance the config state certification was performed under, and would
briefly impose the winning threshold on space 0):** the staged policy
snapshot (from the `CandidateEvaluationContext`) and the candidate space
promote **together** under §10's activation invariant — the SOVEREIGN-tier
config adoption citing the register revision and the catalog switch are one
serialized checkpoint act, discharging §1's empirical-determination
obligation attributably. `adjust` enters a corrective P1 slice and
**re-runs P1-B under a new committed preregistration revision** (never
tunes against observed results without the register recording it); the
candidate space **stays PREPARING** when the adjustment is policy-level —
the staged snapshot is replaced, candidate qualification/weight runs re-run
under it, and **candidate certification is redone** (embeddings are
reusable while the geometry is unchanged; the certification is not) — and
is **marked failed and sovereign-ABANDONED with a fresh candidate
prepared** when the model or geometry itself changes. `kill` **blocks P2B
and returns the roadmap to sovereign reconciliation** — the candidate space
is ABANDONED (R20), space 0 remains ACTIVE, and P2A may finish
independently, but the product sequence stops until the sovereign
re-sequences.
Time-box binds P1-B, fixed at pin; overrun is a finding, not an extension.
