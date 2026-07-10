# Roadmap Reconciliation — the sequence after Slice 11
### Prepared 2026-07-09, revised 2026-07-09/10 (passes 2–7) · analysis only — COMMITTED ON PROPOSAL BRANCH, UNMERGED, UNPINNED
### Nothing here is implemented, canonically amended, or authorized until the §15 decisions return. Passes 2–7 and every visible retraction live in [REVIEW_LEDGER.md](REVIEW_LEDGER.md).

Documents 5–8 remain authoritative about required behavior. This document proposes
development *order*, application *boundaries*, and validation *gates*. It weakens,
removes, and silently satisfies nothing. Where it touches a recorded assertion
(the transport wall), it says so and routes the change through a sovereign ruling.

---

## 1. Executive assessment

Phase B's testable spec has reached full **floor-and-mock coverage** of the
Document 8 criteria as of Slice 11's delivery (75ad38b); real-provider
integration (scanner, serving endpoints, fetch transport) and the annotated
halves remain, by design, beyond that coverage. What the project holds at HEAD
(26c0090 — Slice 11 delivered, Slice 11b's spec pinned docs-only; inventory
refreshed pass 6) is a heavily enforced, adversarially reviewed *library*:
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

**Production-shaped and reachable through an application surface** —
NONE. This row is the finding. (`db_probe` is a diagnostic example, not a
surface.)

**Implemented as domain/infrastructure libraries (test-exercised only)** —
the store substrate and its 18 migrations (lifecycle, artifacts, flags,
refusals, leases, logs, config, secret scan, J-floor mandates, threshold
admission, actor-class, threshold events, canon sources, V.4 closure); intake
(commit→normalize→classify, dispatcher, supervisor, F1 keyed-intake
idempotency — delivered with Slice 11); sovereignty (overrides on
classification, petitions, consents, Notary grant execution); ML floor (roster
traits, 256-dim lexical embedder, Vectoring Slave, Aggregator
consolidate/emergence, rebalance triggers); commitment chain (audit invocation,
AND-barrier, reconciliation, Notary matrix labors, fiat-impossibility);
tool-call ladder; scriptoria (establish/mount/scoped reads/pairings/
conferral); Concordat (six-clause lint, double-validation, bias doctrine);
Student returns (completion contract, refine/re-derive, consistency walk,
steward); the Deacon's threshold (quarantine, verdicts, manifests, admission
conjunction, actor-class authentication); **the collector (Slice 11,
delivered 75ad38b: Section-J fetch execution, collection manifests, canon
sources, both V.4 closures — mock transport)**.

**Proven against deterministic mocks (by design, real provider absent)** —
scanning (`ScanEndpoint` mock; ClamAV unwired); tool-call constrained
generation (`ScriptedCaller`; SC-F06 integration half pinned); reasoner
(test mock only — note the assisted weighting *path* is built and SC-M03-tested
against the mock; only the real mind is absent); fetch (Slice 11 proves Section
J behavior against the instrumented `FetchEndpoint` mock).

**Missing integration surfaces** — server binary and lifecycle; versioned
application API; authentication and client authority (§11); upload transport
(`commit_file` takes in-process bytes); progress/state-change delivery; any
retrieval or search surface; deployment config and health; **coherent
backup/restore across both substrates** (the atoms live on the filesystem,
their references and all derived state in Postgres — doc 03 §1.3's split means
snapshot, restore, and migration must treat the pair as one consistency unit,
and nothing does yet); the client; real embedder; real reasoner; real scanner;
real fetch transport. Plus the three domain gaps named in §3 (link-override
laying, node-incident links, matrix listing).

**Explicitly deferred by canon (doc 00 §7)** — the Librarian and richer-media
degradation; retrieval breadth; the Duty of the House; multi-tenancy.

**Known residues, NARROWER rows, annotated halves** — SC-F06 integration half
(ownership resolved in §8/P5 — see §17 item 6); SC-I05 purge half (Duty of the
House); SC-J09 fetch half (Slice 11 delivered its mock half; real-transport
validation at P6); Slice 11's mock-proven J criteria carry the same G13 shape
until transport is real; the criteria sweep's remaining PENDING rows
(classification in progress, non-blocking); ~~F1 keyed-intake (named Slice 11
rider)~~ **delivered with Slice 11 (75ad38b)**; **Slice 11b (the Doctor and
the dissolution cascade) — spec pinned docs-only at 26c0090, undelivered; it
precedes P0**; `coherence_threshold` unseeded (deliberate — P1 is where the
sovereign finally sets it on evidence, at the §8/P1 checkpoint);
known shared-DB test serialization (managed, not solved — P0 closes it).

**Criterion satisfaction vs application readiness** — the sweep measures
whether tests match criteria text; nothing in it measures whether an operator
action can reach the behavior. Both measures are needed from here on; only the
first exists today.

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
| Audit → consent → commit | complete at crate level (floor auditors, barrier, reconcile, Notary) | the full ceremony command/read façade (§8/P2 Tranche B); the executable-consent tick as a served behavior; the server as the authenticated sovereign-class path (G10) |
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

What canon obligates (unchanged by this document): the four requirements named
in §1; all doc-8 behavior criteria; the deferred list of doc 00 §7; the
adversarial-review rule; the gate; the two-commit lifecycle; the Slice 11
opening conditions.

What this reconciliation changes if adopted: development order; the addition of
application-boundary and evaluation work with its own criteria register (§14);
one recorded assertion — the universal transport wall — which SLICE_11 §0
currently preserves until "Phase 5" and which §9 proposes to replace with
capability boundaries. **That replacement requires a new sovereign ruling; it
is decision D3 and is not made by this document.** Under D3, SC-F06's
integration half detaches from the fetch-deletion bundle and binds to the
model-egress capability (completed at P5, where constrained tool-argument
generation first meets a real serving endpoint); if D3 is declined, SC-F06
stays with the universal wall's deletion at P6. One assignment, ruled by D3's
answer — never both.

Obligation tracking across the resequencing (nothing silently satisfied):
SC-I05's purge half stays with the Duty of the House (P7). SC-J03/05/06/07/
08/10 are mock-proven in Slice 11 and receive real-transport validation in P6,
recorded in-test per G13. A P1 finding that no single coherence threshold
serves all corpus classes would be a **Dogma VI.1 amendment proposed through
process**, never a quiet parameterization. The application phases add
criteria; they retire none.

## 5. The strongest case for the existing sequence (transport → client)

1. **Completion before exposure.** Finishing fetch transport and the real
   scanner closes every doc-8 residue and seals Phase B as a finished box.
2. **Risk retirement while small.** Hostile-content handling and
   transport-boundary enforcement are the sharpest security edges; retiring
   them first means no user-facing surface ever coexists with an unhardened
   fetch path.
3. **Momentum and context.** The mandate machinery is warm; transport
   completes the arc. The application phases switch domains with new failure
   classes and real switching cost.
4. **No rework risk from evaluation.** Outward reach is semantics-independent;
   nothing the offline evaluation finds can invalidate it. APIs built before
   semantics are measured risk reshaping.

Point 4 is the strongest and survives partially — answered by the two-tranche
spine (§8, D2) rather than by rejecting the resequencing.

## 6. The strongest case for the revised sequence (validation first)

1. **The largest unretired risk is the thesis.** Transport risks are
   well-understood engineering with safeguards already recorded (SLICE_11 §0).
   The knowledge-map claim is the project's reason to exist and is untested by
   any green gate.
2. **The canon's own unfinished obligations point here.** The threshold cannot
   be set without empirical work; the advisory model cannot exist without a
   client; the topology cannot exist without a server; the local endpoint is
   prescribed. The revised sequence does mandatory work that also produces
   evidence.
3. **Emergence is inert until the empirical phase happens anyway.** With the
   threshold unseeded, no deployment can form a single Postulant.
4. **Evidence compounds.** Offline findings shape API read models; operator
   findings shape governed-labor exposure; both shape what transport must
   serve.

## 7. Product claims and validation order

The three-claim decomposition (knowledge map / governed labor / external
reach) is sound and the dependency graph permits map-first (§3). Boundaries
the record must keep:

- **Floor vs assisted judgment, split by claim.** Reasoner-assisted
  *weighting* is claim-1 machinery — doc 04 §5.3 names assisted mode the
  intended rich default of the GodHead proper — so the product verdict must
  see it (P4 evaluates both modes; §8). Reasoner-assisted *audit and labor*
  is claim-2 machinery and arrives at P5; the early gate measures the audit
  **protocol** (understandable, tolerable, trustworthy as procedure) and may
  not claim to have measured assisted-audit quality before P5.
- **Offline vs human relevance.** Offline evaluation establishes floor
  quality and structural behavior against labeled evidence only; it can
  reject a poor approach early and tune parameters. It cannot establish
  operator value. A successful offline result does not settle claim 1; only
  P4 can.

## 8. Recommended phase order, with entry and exit conditions

**Decision precondition:** D3 (transport capabilities) and D4 (embedder path +
space policy) are answered before P1 pins — P1 builds on both.

**P0 — Foundation closure (fixed, bounded).**
Entry: Slice 11 delivered — **satisfied (75ad38b: adversarial round completed,
confirmed findings closed, F1 rider shipped, gate green via producer, 161
tests)** — and **any sovereign-pinned successor slice closed: Slice 11b (the
Doctor & dissolution cascade, pinned at 26c0090 and amended at `ffae6a8`) is
signed but undelivered and precedes P0**. Its two-commit lifecycle owns delivery,
not this roadmap.
Contents, exhaustively: (1) confirmed Slice-11 findings; (2) keyed-intake rider
verification; (3) criteria-sweep regeneration; NARROWER and annotated halves
classified, loop-blocking ones resolved or scheduled; (4) **fresh-database
bootstrap test** — all migrations from zero, verifying extensions, functions,
triggers, indexes, order; (5) test-state isolation — one chosen mechanism so
concurrent runs cannot interfere; (6) reproducible raw/derivative filesystem
bootstrap; (7) any defect that prevents the P1 harness or spine from running.
Exclusions binding: no Store refactor, no performance work, no speculative
cleanup.
Exit: checklist complete; gate green; **the existing pipeline processes the
synthetic smoke corpus end-to-end at crate level on the lexical floor** (a
smoke run over existing crates, not the measurement harness); **and P1's
harness scope, corpora, and pre-registered criteria are pinned** (register
revision committed). P0 does not build P1's instrument; it proves the ground
the instrument stands on.

**P1 — Offline semantic proof (two pinned slices; measurement time-boxed).**
Entry: P0 exit; **D3 and D4 answered — a hard precondition: deferring D3
leaves the universal wall standing, which makes the P1 adapter impossible
(§15/D3 states the coupling)**; evaluation criteria and thresholds
pre-registered as a committed register revision *before first measurement*;
three corpora prepared (§12).

*P1-A — semantic infrastructure (a pinned build slice with its own criteria;
pass-3 ownership fix — these measurements cannot run on substrate nobody
builds):* the **EmbeddingSpace schema and lifecycle machinery** (§10 —
generations, space-aware backfill, index build, activation switch, `space_ref`
propagation and the bond/measurement split of §3.1); **the production
`godhead-model-adapter` (embedder half)** — loopback-validated,
redirect-refusing, alias-only, budgeted; **D3's reachability walls**, landing
in the same slice as the first governed transport dependency; **the cargo
workspace-manifest edits that admit the new crates (pass-5 — named here in
P1-A's own contents, not only in P2's dependency explanation): adapter,
server skeleton, and any shared-surface manifest change land once in this
slice, so P1-B and P2A touch disjoint files thereafter**; the candidate-evaluation context and production-path harness specified in [SPACE_PROMOTION_AND_EPOCHS.md](SPACE_PROMOTION_AND_EPOCHS.md).

*P1-B — measurement (time-boxed):* the §12 offline measurements on the
production serving path. Optional drop-if-slipping sub-goal: an offline
assisted-weighting probe (`weigh()` against labeled links) feeding P4's
design. **Corpus-role discipline (pass-3 fix):** the operator corpus
participates in P1-B for *structural, latency, and storage* behavior only —
no relevance judgments are collected or tuned against it; its judgments are
reserved intact for P4.
Exit: metrics recorded (labeled + synthetic corpora for relevance/agreement;
all three for structure/cost); threshold sensitivity and stability curves;
storage-growth measurement begun (§12); kill/adjust/proceed verdict against
the pre-registered bands. A finding that corpus classes need different
thresholds becomes a proposed VI.1 amendment (§4), decided by the sovereign —
never enacted by this phase.
**Verdict routing:** the post-measurement checkpoint follows the P1-B ∥ P2A
join and uses the promotion mechanism in
[SPACE_PROMOTION_AND_EPOCHS.md](SPACE_PROMOTION_AND_EPOCHS.md). `proceed` is
one atomic R19 policy-and-space act; `adjust` re-enters P1 under a new committed
preregistration revision; `kill` blocks P2B and returns to sovereign
reconciliation. The P1-B time-box is fixed at pin; overrun is a finding.

**P2 — Application spine (two tranches, separately pinned and reviewed).**
Shape (pass-4 correction — pass 3 forked too early): **the fork follows
P1-A, not P0.** P1-A lands the shared surfaces (workspace manifests, D3
walls, space substrate, adapter) that P2A consumes; only then do P1-B and
P2A run concurrently:

    P0 → P1-A → fork{ P1-B ∥ P2A } → join → sovereign policy/space
    checkpoint → P2B
    (pass-6: the checkpoint node is explicit — it consumes P2A's authority
    substrate and P1-B's evidence, and P2B's Tranche-B pin follows it)

Concurrency discipline (D2): concurrency is a *scheduling* statement, never a
delivery-discipline statement. P1-B and P2A are separately pinned, separately
adversarially reviewed, separately gated slices; landings serialize through
the gate; no interleaved unreviewed migrations or partially governed changes
exist on the delivery branch at any moment. (The project has a recorded
parallel-session hazard — the standing git-quiet check applies.) With the
fork moved behind P1-A, the shared-surface qualification resolves itself:
P1-B and P2A genuinely touch disjoint surfaces.
Tranche A (semantics-independent; may run concurrent with P1-B): server binary
and lifecycle; deployment config and health; upload transport and
`commit_files`; intake status, `watch_progress`, and job/refusal/log read
models; recovery-on-restart; the **two-substrate backup/restore procedure**
with a restore test; and the minimum P2A authority substrate defined in
[AUTHORITY_REGISTRIES.md](AUTHORITY_REGISTRIES.md)—authentication and local
recovery, unforgeable contexts, structural gate, K/R/F/H envelope,
CommandReceipt/CommandRefusal, complete registered-and-refusing handler
skeleton, and live R19/R20 checkpoint callables. P2A exits only when an
enrolled operator can invoke R19/R20 and unauthenticated, stale,
hash-mismatched, and replayed commands refuse before elevation. Nothing in
P1's semantic findings can invalidate this substrate.
Tranche B (semantics-shaped; pins after the P1/P2A join): neighborhood, link,
explain-link, Postulant, matrix, and retrieval read models; the override labor
and criteria in [OVERRIDE_LINEAGES.md](OVERRIDE_LINEAGES.md); `links_for_node`;
`list_matrices`; link-policy and threshold parameters as configuration
surfaces; the complete ceremony, authority, envelope, receipt, and served-executor
surface specified in [AUTHORITY_REGISTRIES.md](AUTHORITY_REGISTRIES.md).
Exit: **Stop-3 completable end-to-end through the API alone** (curl or
equivalent) — commit through commitment through navigation, no SQL, no
fixtures, no crate calls. P3 translates a complete contract; it discovers no
missing backend operations.

**P3 — Thin operator client.**
Entry: P2 exit. Six views or equivalents (staging/commit; intake status;
neighborhood; link explanation & correction; Postulant review/audit/consent/
commitment; Cardinal navigation with provenance). Register retained with
plain-technical translations throughout; the graduated-legibility ladder
renders from the API's `legibility_state` (doc 04 §6.5).
Exit: Stop-3 completable by the operator through the client alone.

**P4 — Operator evaluation and product gate (two tranches; pass-3 ownership
fix).**
*P4-A — integration tranche (a pinned slice, not an unowned "entry
condition"):* the reasoner half of `godhead-model-adapter` (the
assisted-weighting path is already built and mock-proven — SC-M03; this
tranche wires the real mind), with its own criteria and gate.
*P4-B — evaluation:* entry = P3 exit + P4-A delivered + operator thresholds
confirmed unchanged since their P1-era pre-registration or amended on the
record. Contents: repeated real use across sessions; the §12 operator
metrics; **paired evaluation of numerical-floor and reasoner-assisted
weighting on the same corpora and tasks** (doc 04 §5.3's dial, measured at
last).
Exit: the explicit decision — continue / adjust / simplify ceremony /
reconsider the thesis. **An `adjust` verdict routes into a corrective slice
and returns to P4-B for re-measurement; it does not advance to P5** (pass-3
sequencing fix — adjustment without re-evaluation would advance on the very
evidence it invalidated). **`Simplify ceremony` routes the same way (pass-4):**
interaction-level simplification enters a corrective client slice;
simplification that touches constitutional ceremony (the trial's shape,
consent structure) is a **Dogma amendment proposed through process** — either
way, P4-B re-runs before P5. **A reconsider-the-thesis verdict is reachable
only after assisted weighting has been measured**; if assisted mode is
unavailable, the gate may return continue/adjust/simplify but may not reject
the thesis on floor-mode evidence alone.

**P5 — One governed-labor workflow.**
Entry: P4 *continue* verdict (an `adjust` loops within P4). The Devout
Assignment loop (brief → Instruction → Student work → Return → review →
matrix change) exposed through the client, consuming the already-integrated
reasoner; assisted-audit measurement opens here as its own category;
**SC-F06's integration half completes here**, per D3's detachment ruling —
with the pass-3 precision that this is *new adapter capability, not reuse*:
the adapter's governed surface (embed, weigh) gains a third model operation,
constrained `propose_call`, implementing the existing `ToolCaller` seam
against the real endpoint; SC-F06 exercises that capability, which
embed/weigh cannot.
Exit: claim-2 measurements recorded against pre-registered bands.

**P6 — Real outward transport.**
Entry: P5 exit (or P4, if the sovereign reorders on evidence). The fetch wall
deletes only here, atomically, with the fetch-specific safeguards SLICE_11 §0
records: transport behind `FetchEndpoint`; request/byte/time/trip-budget
enforcement where bytes move; secret scanning before fetched bytes reach
persistence; real scanner-provider integration; hostile MIME/redirect/
decompression/timeout/partial-download handling; provenance across every
transformation. (SC-F06 completes at P5 under D3; it appears here only if D3
is declined.) Slice 11's mock-proven J criteria receive real-transport
validation.
Exit: claim-3 measurable end-to-end under the Deacon's gate.

**P7 — Deferred capabilities, separately justified.**
Retrieval breadth, the Librarian, Duty of the House (now owed the storage-
growth evidence §12 collects), multi-tenancy, new offices, new ceremonial
states, general graph authorship — each enters only by its own justification.

## 9. Transport-capability proposal (requires a new sovereign ruling — D3)

The current wall bans transport *dependencies* workspace-wide. It conflates
three capabilities with different risks:

1. **Inbound application transport** — client → server. Owner: a
   `godhead-server` crate, the composition root; it may depend on the
   application layer and domain; **no domain, store, or agent crate may depend
   on it**.
2. **Model egress** — backend → configured local inference endpoint. Owner: a
   `godhead-model-adapter` crate; it may depend on an HTTP client; it
   implements `godhead-ml`'s `Embedder`/`Reasoner` traits and **exposes model
   operations only** — no generic request surface. The governed operation set
   grows by phase: `embed` (P1-A), `weigh` (P4-A), constrained `propose_call`
   implementing the `ToolCaller` seam (P5, where SC-F06 completes).
   `godhead-ml` does not depend on it; adapters are constructed and rostered
   only in the composition root.
3. **Fetch egress** — collection labor → externally named resources. **No
   fetch-related crate may have any dependency path to any transport
   implementation until P6's atomic commit.** Deletion conditions preserved
   verbatim (§8/P6).

Enforcement by **dependency-graph reachability, not name exceptions**: an arch
test consuming `cargo metadata`'s resolved graph (transitive closure) asserts
(a) inbound-server packages reachable from `godhead-server` only; (b)
HTTP-client packages reachable from `godhead-model-adapter` only; (c) zero
transport packages reachable from fetch-surface crates; (d) `godhead-ml` does
not reach the adapter; (e) no domain/store/agent crate reaches the server.
Token greps remain as depth beneath the graph test.

Localhost constraints: endpoint identities persist by alias only (Law
XV/XIII.2); alias→URL resolution in deployment configuration, never in
records; scheme http(s), host loopback-only during the localhost phase;
redirects refused; timeouts and response-size ceilings in the adapter.

**Timing (revised, pass 2):** D3 is decided **before P1 pins**, and the
reachability walls land in the same slice that introduces the first governed
transport dependency (P1's adapter). This amends two RULED arch tests
(`no_outward_transport_wall`, `sc_b04_workspace_ipc_scan` — the latter gains
the server carve-out with its inverse assertion), supersedes SLICE_11 §0's
universal clause while preserving its fetch-deletion conditions, and
**detaches SC-F06's integration half from the fetch bundle, binding it to the
model-egress capability (P5)**. Law III.1 and Law V.4 are untouched; the
recorded author-intent reading is that III.1 binds agents, not the operator's
own client/server surface.

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
why, and states whether results had already been observed. **Holdout
hygiene across `adjust` cycles (pass-6 — a new preregistration revision does
not make an observed holdout unseen):** a **final holdout is reserved
untouched across all adjust cycles** and consulted exactly once, at the
last pre-verdict evaluation; interim cycles evaluate on the calibration
split and any rotating interim holdouts the preregistration names. (The
predeclared sequential-testing alternative — alpha-spending across looks —
is recorded as viable and declined for v1 as procedure-heavy.) The operator
corpus stays judgment-unseen until P4, as already required.
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

Passes 2â€“6, including every visible retraction and historical HEAD claim, moved
verbatim to [REVIEW_LEDGER.md](REVIEW_LEDGER.md). The ledger is historical;
current mechanisms live only in their owning annexes.
