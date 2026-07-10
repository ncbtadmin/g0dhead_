# Roadmap Reconciliation — the sequence after Slice 11

**Prepared:** 2026-07-09; revised 2026-07-09/10, passes 2–7.

**Status:** Analysis-only — committed on proposal branch, unmerged, unpinned. Nothing here is implemented, canonically amended, or authorized until the §15 decisions return. Passes 2–7 and every visible retraction live in [REVIEW_LEDGER.md](REVIEW_LEDGER.md).

Documents 5–8 remain authoritative about required behavior. This document proposes
development *order*, application *boundaries*, and validation *gates*. It weakens,
removes, and silently satisfies nothing. Where it touches a recorded assertion
(the transport wall), it says so and routes the change through a sovereign ruling.

---

## 1. Executive assessment

Phase B's testable spec has reached **all Slice-11 floor-and-mock coverage except
SC-J08**, which was deliberately severed to the signed but undelivered Slice
11b. Real-provider integration (scanner, serving endpoints, fetch transport)
and annotated halves also remain beyond that coverage by design. What the
project holds at the committed review baseline (`ffae6a8` — Slice 11 delivered;
Slice 11b's amended specification signed docs-only) is a heavily enforced,
adversarially reviewed *library*:
twelve crates, eighteen migrations, 161 passing tests across 47 binaries, and
zero binary targets except a connectivity probe. No server exists, no API, no
client, no real embedder, no retrieval surface. Every capability is exercised
through tests or direct crate calls. The system can prove it obeys its
contracts; it cannot yet show an operator a map.

The canon requires four things this document sequences: **empirical determination
of the coherence threshold** (doc 03 §2.4 — deliberately unseeded), **a client
expression of advisory state** (doc 01 §2.5, doc 04 §6), **a production-shaped
application boundary** (doc 01 §3; doc 02 §2.1), and **a local inference
endpoint** (doc 04 §2.2, §3.1). The canon does *not* require any particular order
of those four against the remaining internal work. The sequence proposed here is
an engineering judgment, argued in §5–§6 on its merits.

Recommendation in one sentence: after a bounded foundation closure, run a
time-boxed offline semantic proof **on the production serving path** (the
capability ruling and adapter land first — §9/§15 D3–D4), build the application
spine in two tranches (semantics-independent work concurrent with the offline
phase; semantic API shapes pinned after its findings) with the spine exiting only
when the **full loop** runs through the API, then the thin client, then an
operator product gate that evaluates **both weighting modes** before any
thesis-level verdict — deferring governed-labor exposure and real outward
transport behind evidence that the map is worth governing and feeding.

The project contains two products and this sequence deliberately tests one:
the private, local-first semantic knowledge repository. The constitutionally
governed agent society is, until P5, *internal machinery* — separation of
authority the operator benefits from without operating. The client translates
offices into actions and evidence ("independent audits complete", "consent
required"); no view requires the operator to learn the bureaucracy to correct
a link. If a concept cannot be explained without the lore, that is treated as
a mechanical-clarity defect, not a lore-education gap (doc 00 §6's own wall,
applied to the UI).

The **largest unretired risk is the thesis** — transport risks are
well-understood engineering with recorded safeguards, while the knowledge-map
claim is untested by any green gate. It is not the *only* unretired risk:
server exposure, client authority, provider integration, threshold calibration,
space migration, and two-substrate recovery are all open and are scheduled
below.

## 2. Capability inventory

**Production application surface:** none. `db_probe` is a diagnostic example,
not a product binary.

**Implemented, library-only:** twelve crates and 18 committed migrations cover
the store substrate, immutable intake, sovereignty/petitions/Notary grants,
lexical ML floor and assisted-weighting seam, matrix trial/commitment,
tool-call repair, scriptoria/pairings, Concordat and Student returns, threshold
admission, and Slice 11 collection against mock transport. They are reachable
through tests or direct crate calls, not an API.

**Mock-proven but not real-integrated:** scanner endpoint, model reasoner/tool
caller, and fetch transport. The assisted-weighting path exists; the real
reasoner does not.

**Missing application/integration work:** server/API/authentication; upload and
progress transport; retrieval/search and client; deployment/health; real
embedder/reasoner/scanner/fetch; and one consistency-unit backup/restore across
Postgres plus filesystem atoms. Domain gaps are link-override laying,
`links_for_node`, and `list_matrices`.

**Deferred/residual:** Librarian, richer media, retrieval breadth, Duty of the
House, and multi-tenancy remain canon-deferred. SC-F06's integration half moves
to P5; SC-I05 purge remains Duty-of-House work; SC-J09 real transport waits for
P6; SC-J08 remains DEFERRED until signed Slice 11b delivers; the criteria
sweep's PENDING classifications are non-blocking; `coherence_threshold` stays
deliberately unseeded until P1; P0 closes shared-DB test isolation.

The criteria sweep measures behavior evidence, not operator reachability. From
this point forward both are required.

## 3. Dependency graph — HEAD to the first useful loop

Candidate loop: commit corpus → immutable intake → real embeddings → links →
Postulant emergence → inspection & correction → audit → consent → commitment →
navigation/retrieval.

| Loop stage | Exists at HEAD | Missing for application completion |
|---|---|---|
| Commit corpus | `commit_file` (in-process bytes) | upload transport; staging semantics; `commit_files` façade |
| Intake at rest | complete (pipe, dispatcher, supervisor) | status read model; progress delivery |
| Real embeddings | `Embedder` trait + lexical floor | model adapter (P1); embedding-space policy (§10); space migration; backfill-by-space |
| Links | `draw_link`, `similar_nodes`, `links_by_category` | `links_for_node` read; link-policy parameters exposed |
| Weights / live | floor formula; assisted path (mock-proven); `live_weights` | threshold value (empirical); real reasoner for assisted mode (P4 entry); legibility read model |
| Emergence | `emerge_postulant` (cited, idempotent) | threshold value; `list_postulants` read |
| Inspection & correction | category override (laying + protection); link *protection* only | **`lay_link_override` — absent at every layer (§3.1)**; `explain_link` assembly (primitives exist) |
| Audit → consent → commit | complete at crate level (floor auditors, barrier, reconcile, Notary) | the full ceremony command/read façade (§8/P2B); the executable-consent tick as a served behavior; the server as the authenticated sovereign-class path (G10) |
| Navigation / retrieval | `get_matrix` only | neighborhood traversal; matrix browse; retrieval op + baselines |

Earliest useful stopping points:
- **Stop-1 (map browse):** commit → embed → link → inspect. No threshold, no
  ceremony. Answers link relevance and comprehension.
- **Stop-2 (candidates):** + threshold set → emergence → Postulant inspection.
  Answers matrix coherence and correction burden.
- **Stop-3 (full loop):** + audit/consent/commitment → Cardinal navigation.
  Answers ceremony tolerability and retrieval value.

Nothing in Sections I/J or outward transport is upstream of any stop.

### 3.1 Override and release mechanism

The complete proposal—bond/measurement split, compatibility, lineages,
petition occurrences, transition plans, release semantics, selectors, and
acceptance criteria—lives only in
[OVERRIDE_LINEAGES.md](OVERRIDE_LINEAGES.md). D8 decides it; P2B owns it.

## 4. Canon and criteria analysis

Canon requires the empirical threshold, client-visible advisory state,
production-shaped application boundary, local inference endpoint, Document-8
behavior, gate, adversarial review, and two-commit lifecycle. It does not order
those obligations against one another. This roadmap proposes order and the
explicit amendments in [AMENDMENT_MATRIX.md](AMENDMENT_MATRIX.md); it silently
satisfies or weakens nothing.

## 5. Strongest case for transport and client first

The existing sequence maximizes continuity with recorded intent, removes the
oldest integration risks first, exposes real security/packaging constraints,
and avoids building evaluation infrastructure around semantics that later
transport constraints might reshape. Its cost is delayed evidence on the core
knowledge-map thesis and longer deferral of mandatory threshold/client work.

## 6. Strongest case for semantic validation first

The revised sequence attacks the largest unknown while rework is cheapest:
whether embeddings, links, weights, and emergence produce useful structure.
It can kill or reshape the thesis before full UI, governed labor, and outward
transport investment. Its cost is earlier model/space/evaluation complexity
and a later real-fetch proof. Bounded P1-B/P2A concurrency preserves progress
on semantics-independent spine work.

## 7. Product claims and validation order

- **Claim 1:** a private local semantic repository yields a useful,
  correctable map. P1 can reject poor floor behavior; only P4 can establish
  operator value. Both floor and assisted weighting are measured before any
  thesis-level verdict.
- **Claim 2:** governed agent labor improves the map. P5 measures one complete
  workflow after claim 1 survives.
- **Claim 3:** controlled outward collection expands the map without breaking
  provenance or consent. P6 measures real transport last.

Correctness invariants are absolute. Offline quality, assisted-model quality,
and operator outcomes remain separate empirical categories.

## 8. Recommended phase order, with entry and exit conditions

**Decision preconditions:** D3 and D4 return before P1 pins. P1's production
adapter cannot coexist with the universal transport wall.

### P0 - Foundation closure

Entry: Slice 11 is delivered; the signed, undelivered Slice 11b closes first.
Contents are fixed: regenerate the criteria sweep; resolve or schedule every
loop-blocking NARROWER/annotated half; prove fresh-database migration from zero;
isolate test state; prove reproducible raw/derivative filesystem bootstrap; and
fix only defects that prevent the P1 harness or application spine from running.
No Store refactor, performance program, or speculative cleanup enters P0.

Exit: gate green; the existing lexical-floor pipeline processes the synthetic
smoke corpus end to end at crate level; and P1's harness scope, corpora,
preregistered criteria, and final-holdout rule are committed.

### P1 - Offline semantic proof

Entry: P0 exit, D3/D4 answered, and preregistration committed before first
measurement.

- **P1-A:** deliver the space/candidate substrate and production embedder
  adapter in [SPACE_PROMOTION_AND_EPOCHS.md](SPACE_PROMOTION_AND_EPOCHS.md),
  D3 walls, evaluation harness, and the one-time workspace manifest changes
  later consumed by P2A.
- **P1-B:** run time-boxed production-path measurements on labeled, synthetic,
  and operator corpora under the role separation in Section 12. The operator
  corpus contributes structure/cost only; its relevance judgments remain
  unseen until P4.

Exit: record relevance/agreement, sensitivity/stability, latency/storage/cost,
and kill/adjust/proceed verdicts against preregistered bands. Corpus-class
threshold divergence becomes a proposed VI.1 amendment, never silent tuning.
Verdicts follow [SPACE_PROMOTION_AND_EPOCHS.md](SPACE_PROMOTION_AND_EPOCHS.md):
`proceed` uses R19, `adjust` creates a new preregistration/candidate state, and
`kill` blocks P2B. The time-box does not extend itself.

### P2 - Application spine

The scheduling shape is:

    P0 -> P1-A -> fork{P1-B | P2A} -> join -> sovereign checkpoint -> P2B

P1-B and P2A are separately pinned, reviewed, and gated; landings serialize.
Concurrency is scheduling freedom, not permission for interleaved unreviewed
migrations.

- **P2A:** server lifecycle, deployment/health, upload and `commit_files`,
  progress/job/refusal/log views, restart recovery, two-substrate restore test,
  and the minimum authority/checkpoint substrate in
  [AUTHORITY_REGISTRIES.md](AUTHORITY_REGISTRIES.md).
- **P2A exit:** an enrolled operator can invoke R19/R20 end to end; bad auth,
  stale state, hash mismatch, and replay refuse before elevation.
- **Checkpoint:** consumes P1-B evidence and P2A authority. R19 either promotes
  the exact candidate or refuses; non-proceed routes remain visible.
- **P2B:** semantic read models, neighborhood/retrieval, the work in
  [OVERRIDE_LINEAGES.md](OVERRIDE_LINEAGES.md), complete ceremony/API surface,
  and the served executors in
  [AUTHORITY_REGISTRIES.md](AUTHORITY_REGISTRIES.md).

Exit: Stop-3 is completable through the API alone, from commit through
Cardinal navigation, with no SQL, fixture, or crate call.

### P3 - Thin operator client

Expose staging/commit, intake status, neighborhood, link explanation and
correction, Postulant trial/consent, and Cardinal navigation/provenance. Render
technical translations and `legibility_state`; require no lore education.
Exit: Stop-3 is completable by the operator through the client.

### P4 - Operator product gate

- **P4-A:** connect the real reasoner to the already mock-proven assisted
  weighting path.
- **P4-B:** measure repeated use, the Section-12 operator metrics, ceremony
  cost, and paired floor/assisted weighting on the same corpora/tasks.

Exit is `continue`, `adjust`, `simplify ceremony`, or `reconsider thesis`.
Adjust/simplify returns to P4-B after a corrective slice; constitutional
simplification uses the amendment process. Thesis rejection is unreachable
until assisted weighting has actually been measured.

### P5 - One governed-labor workflow

Expose the Devout Assignment loop through the client and add the real
`propose_call` adapter capability for SC-F06. Assisted-audit measurement begins
here. Entry requires P4 `continue`; exit records claim-2 measurements.

### P6 - Real outward transport

Delete the fetch wall only in one atomic slice: `FetchEndpoint`, byte/time/trip
budgets at transport, secret scanning before persistence, real scanner,
hostile MIME/redirect/decompression/timeout/partial-download handling, and
provenance across every transform. Exit makes claim 3 measurable under the
Deacon's gate.

### P7 - Separately justified deferred capability

Retrieval breadth, Librarian, Duty of the House, multi-tenancy, new offices,
new ceremony, and general graph authorship each require their own evidence and
pin.

## 9. Transport-capability proposal (D3)

The current workspace wall conflates three capabilities:

1. **Inbound application transport:** client to `godhead-server`, the sole
   composition root. Domain, Store, and agent crates never depend on it.
2. **Model egress:** `godhead-model-adapter` exposes only governed `embed`,
   `weigh`, and later `propose_call` operations to configured local aliases.
   No generic request surface crosses into domain/ML crates.
3. **Fetch egress:** no dependency path reaches transport until P6's atomic
   wall deletion and its byte/time/trip, secret, redirect, MIME,
   decompression, partial-download, and provenance safeguards.

Reachability tests prove allowed and forbidden dependency paths, not merely
crate names. The model-adapter and its wall land together in P1-A; server
transport receives the corresponding narrow carve-out; fetch remains sealed.
SC-F06's real integration moves to model egress at P5.

D3 explicitly supersedes Slice 11's universal dependency clause and amends the
two affected architecture tests while preserving Law III.1, Law V.4, and every
fetch-deletion condition. Deferring D3 makes D1/P1 impossible unless P6 moves
first or doc 04's separate-process model architecture is amended.

## 10. Space promotion, trial, and evidence mechanisms

The space lifecycle, candidate evaluation context, certification epochs, and
atomic checkpoint promotion live only in
[SPACE_PROMOTION_AND_EPOCHS.md](SPACE_PROMOTION_AND_EPOCHS.md). Trial-cycle
identity, evidence validity, and activation-time Postulant disposition live
only in [TRIAL_AND_EVIDENCE.md](TRIAL_AND_EVIDENCE.md). D4 decides both.

## 11. Application boundary and authority registries

The v1 boundary, authority classes, reserved-operation registry,
authentication-control registry, envelopes, receipts, and executor discovery
live only in [AUTHORITY_REGISTRIES.md](AUTHORITY_REGISTRIES.md). D7 decides the
posture; P2A supplies its minimum substrate and P2B completes the surface.

## 12. Evaluation plans

**Offline (P1, time-boxed).** Three corpus classes: (a) synthetic/adversarial
— constructed clusters with distractors, duplicates, near-duplicates,
add/remove/reorder permutations; (b) a labeled corpus for offline relevance
measurement; (c) a representative operator corpus whose judgments carry into
P4. **Data hygiene:** calibration data (used to select thresholds and link
policy) is disjoint from a **holdout** portion unseen during tuning; the
operator corpus is reserved for personal-utility evaluation.
Measurements, pre-registered: suggested-link precision against labels;
cluster/matrix agreement — ARI only where labels are flat and mutually
exclusive, overlap-aware measures (pairwise/BCubed-class) where matrices may
overlap; retrieval quality on predeclared queries; **baselines strengthened
beyond filename/full-text**: plain cosine nearest-neighbor over the *same*
embeddings (the fair fight — does the structure beat raw vector search on
identical geometry?), a conventional clustering method over the same vectors,
vector search without weights/emergence/audit/commitment, and existing
folder/category structure where available; floor-vs-real-embedder deltas;
threshold sensitivity; stability under corpus mutation; latency, storage, and
model resource cost; **storage-growth measurement (append-only volume across
P1–P4)** — explanation belongs to P2's read models and cleanup stays deferred,
but the Duty of the House gets evidence, not guesses.
**Preregistration discipline:** every preregistration is a committed revision
of the criteria register; any later change cites the prior revision, states
why, and states whether results had already been observed. **Holdout hygiene
across `adjust` cycles:** a final confirmatory holdout is reserved untouched
through exploratory adjustment. Opening it is a preregistered freeze point:
policy, geometry, and evaluation code freeze, and the confirmatory result may
produce only `proceed` or `kill`. If the result instead motivates adjustment,
it is reclassified as exploratory; the adjusted candidate requires a newly
collected untouched holdout and new preregistration before `proceed` is
reachable. Interim cycles use calibration data and any preregistered rotating
interim holdouts. The sequential-testing alternative—alpha-spending across
looks—remains viable but is declined for v1 as procedure-heavy. The operator
corpus stays judgment-unseen until P4.
The offline phase may kill an approach early; it cannot validate the product
claim.

**Operator (P4).** Link comprehension; correction burden per candidate;
candidate-matrix usefulness; time to approve/reject; retrieval-task completion
vs the §12 baselines; recommendation rejection/ignore rates; confidence gained
from provenance and the audit *protocol* (floor-judgment era, recorded as
such); **weight legibility** — can the operator see why an item carries
influence and predict what changing it does; **ceremony cost, measured where
the constitution puts ceremony** — routine corrections are single sovereign
acts by design (an override is one act; no audit chain attends it), so the
metric distinguishes correction cost (should be near-zero) from
commitment/admission ceremony cost (deliberate, measured for tolerability);
**paired floor/assisted weighting comparison** (§8/P4); repeat voluntary use
across sessions and weeks.

**Category separation, maintained throughout:** correctness invariants
(absolute); offline floor quality (empirical, revisable); assisted-model
quality (weighting opens at P4, audit/labor at P5 — never claimed earlier);
operator outcomes (empirical, pre-registered, revisable only on the record).

## 13. Risks and opportunity costs

**Of the revised sequence:** transport hardening deferred (the fetch wall
stands mechanically throughout); the scanner residue pinned longer; new-domain
risk earlier (serving, API, UI); evaluation cost real (labeling is P1's
largest line); offline findings may force linking-policy rework (the point —
cheapest now); the P4 gate may return "reconsider" (facing it late costs
strictly more).

**Of the existing sequence:** the thesis risk compounds untested; the
client-expressed advisory laws and the empirical threshold — mandatory canon —
stay unbuilt; no demonstration of emergence is possible (threshold unseeded);
first user evidence arrives after maximum investment.

**Standing operational risks, either sequence:** append-only growth without
views (P2 read models carry explanation; P7 carries compaction under A.14(c)
destruction terms; §12 now collects the growth evidence); the shared live-DB
test posture until P0's isolation lands; two-substrate recovery until P2A's
restore test exists; client authority until D7 is answered.

## 14. Criteria-register authority (D5)

Recommendation: **`docs/dev/APPLICATION_CRITERIA.md`** — a development-phase
register, not canon. Doc 8 decomposes ratified law and should not carry
empirical thresholds designed for revision. Anti-shadow-canon guard: a
non-canon header ("binds phases and gates, not v1 behavior; amended by
recorded sovereign note"), plus the promotion path — any application criterion
that hardens into permanent behavior law is promoted into Document 8 through
the amendment process. **Preregistration mechanics (§12) make the register an
append-ledgered file: revisions committed, cited, and observation-status
declared.** The alternative — canonical `docs/09` with a doc 00 reading-order
amendment — is viable if application obligations should carry constitutional
weight; the cost is canon churn on every empirical revision. Doc-8 discipline
either way: citable `AC-` ids, verifiable assertions, seed tests,
`SOVEREIGN_JUDGMENT` where the operator is the check, G13 annotations.

## 15. Decision sheet — Pass-7 controlling form

Eight decisions return to the sovereign. This sheet records choices and
recommendations only; each mechanism has one normative proposal home. If a
summary here conflicts with its annex, the annex governs and the conflict is a
review finding.

**D1 — Phase sequence.** Adopt P0 → P1-A → fork{P1-B ∥ P2A} → join → sovereign
policy/space checkpoint → P2B → P3 → P4-A → P4-B(loop) → P5 → P6 → P7, with
the entry, exit, and non-proceed routes in §8 and the checkpoint mechanism in
[SPACE_PROMOTION_AND_EPOCHS.md](SPACE_PROMOTION_AND_EPOCHS.md).
Recommendation: adopt as amended. `[adopt / reject / amend]`

**D2 — Scheduling posture.** Permit bounded concurrency only after P1-A:
separately pinned and reviewed P1-B/P2A slices whose landings serialize through
the gate. Recommendation: bounded concurrency. `[offline-first / bounded concurrency]`

**D3 — Transport capabilities.** Replace the universal dependency wall with
the inbound, model-egress, and fetch-egress boundaries in §9. Deferral forces a
different D1 sequence or a doc-04 architecture amendment before P1.
Recommendation: adopt. `[adopt / amend / defer-with-stated-consequence]`

**D4 — Embedder path and space/trial policy.** Decide together: (a) the
production separate-process adapter, with in-process inference contingency
only; and (b) the space, candidate, promotion, epoch, trial-cycle, and evidence
mechanisms in [SPACE_PROMOTION_AND_EPOCHS.md](SPACE_PROMOTION_AND_EPOCHS.md)
and [TRIAL_AND_EVIDENCE.md](TRIAL_AND_EVIDENCE.md), including their D4 rows in
[AMENDMENT_MATRIX.md](AMENDMENT_MATRIX.md). Recommendation: adopt both as
amended. `[adopt a+b / amend]`

**D5 — Criteria-register authority.** Keep application criteria in the
append-ledgered, explicitly non-canonical development register described in
§14, with promotion into Document 8 for any rule that becomes permanent.
Recommendation: dev-register. `[dev-register / canonical doc 09]`

**D6 — P0 closure.** Adopt the fixed entry, contents, exclusions, and exit in
§8/P0. Recommendation: adopt. `[adopt / amend]`

**D7 — Client and command authority.** Decide the application boundary,
authority classes, reserved-operation registry, authentication-control
registry, envelope/receipt substrate, and executor discovery in
[AUTHORITY_REGISTRIES.md](AUTHORITY_REGISTRIES.md), including its D7 rows in
[AMENDMENT_MATRIX.md](AMENDMENT_MATRIX.md). Recommendation: adopt as amended.
`[adopt / amend]`

**D8 — Override release and kind-aware protection.** Decide the complete
lineage, occurrence, transition-plan, compatibility, release, effective-source,
and acceptance-criteria mechanism in
[OVERRIDE_LINEAGES.md](OVERRIDE_LINEAGES.md), including its D8 rows in
[AMENDMENT_MATRIX.md](AMENDMENT_MATRIX.md). Recommendation: adopt as amended.
`[adopt / amend]`

This sheet does not declare itself answerable. Current review status and bounded
verdicts live in [REVIEW_LEDGER.md](REVIEW_LEDGER.md).

### 15.1 The decisions, returned — 2026-07-10

The sheet was answered by the sovereign at the desk sitting of 2026-07-10
(operative record: `DECISIONS_RETURNED_D1-D8.md`, desk outputs, sha256
`193fc12d…9279`). The answers, verbatim — the riders are part of the answers,
not commentary:

>   D1  ADOPT — the P0 pin enumerates its loop-blocking NARROWER/annotated rows.
>   D2  ADOPT — worktree-per-lane is named as the concurrency mechanism: one
>       worktree per lane, one branch per lane, landings serialize through the
>       gate.
>   D3  ADOPT — the three-boundary refactor supersedes the universal wall. The
>       lockfile-level denylist is KEPT on fetch-egress until P6's atomic
>       deletion; the SLICE_11 §0 supersession is recorded by ADDENDUM to that
>       file, never by edit.
>   D4  ADOPT a+b — production adapter and the space/candidate/trial/evidence
>       mechanisms as amended. ADR-2's final DDL and Appendix-A naming return
>       through the desk before adoption. The P1-B time-box and its
>       overrun-is-a-finding clause enter the criteria register as a citable
>       rule.
>   D5  DEV-REGISTER — `docs/dev/APPLICATION_CRITERIA.md`, non-canonical header,
>       append-ledgered preregistration, promotion path into Document 8.
>   D6  ADOPT — P0 contents, exclusions, and exit as written.
>   D7  ADOPT — authority classes, registries, envelopes, receipts, executors
>       as amended; the R17a/b pairing-authority pricing (A.10/manual amendment)
>       is acknowledged and travels with its phase.
>   D8  ADOPT — lineages, occurrences, transition plans, four-kind release, and
>       selectors as amended. The IV.1/IV.5/SC-C01 constitutional amendment text
>       is drafted with the desk's co-signature. The criteria name where the
>       substrate-visible IV.1 wall lives once `user_overridden` leaves the
>       protected payload.
>
> Nothing here delivers implementation, lands a migration, or amends canon by
> itself. Adoption authorizes the phases, the ADRs, and the amendment drafting —
> each through its own recorded process, each at its owning phase.

Adoption changed the decisions' status, not the documents' authority class:
this file and its annexes remain analysis-only and non-canonical. Each annex's
status line records its owning decision's return.

## 16. ADR extraction (after decisions return)

Only accepted mechanisms leave this proposal for independently supersedable
ADRs: ADR-1 from §9 (transport); ADR-2 from
[SPACE_PROMOTION_AND_EPOCHS.md](SPACE_PROMOTION_AND_EPOCHS.md) and
[TRIAL_AND_EVIDENCE.md](TRIAL_AND_EVIDENCE.md); ADR-3 from
[AUTHORITY_REGISTRIES.md](AUTHORITY_REGISTRIES.md); and ADR-4 from
[OVERRIDE_LINEAGES.md](OVERRIDE_LINEAGES.md). Each ADR consumes only the rows
its decision owns in [AMENDMENT_MATRIX.md](AMENDMENT_MATRIX.md). D5's
development register remains separate.

## 17. Review history

Passes 2–6, including every visible retraction and historical HEAD claim, moved
verbatim to [REVIEW_LEDGER.md](REVIEW_LEDGER.md); Pass 7 is appended there.
The ledger is historical; current mechanisms live only in their owning annexes.
