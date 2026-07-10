# Roadmap Reconciliation — the sequence after Slice 11
### Prepared 2026-07-09, revised 2026-07-09/10 (passes 2–6) · analysis only — UNCOMMITTED, UNPINNED
### Nothing here is implemented, amended, or authorized until the §15 decisions return and the next slice receives a pinned scope. Pass-2 dispositions: §17. Pass-3: §18. Pass-4: §19. Pass-5: §20. Pass-6 dispositions, the amendment matrix, and the standing readiness verdict: §21.

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

The complete proposalâ€”bond/measurement split, compatibility, lineages,
petition occurrences, transition plans, release semantics, selectors, and
acceptance criteriaâ€”lives only in
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
Doctor & dissolution cascade, pinned 26c0090) precedes P0** (pass-6 inventory
reconciliation; the two-commit lifecycle owns it, not this roadmap).
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
**Verdict routing:** the post-measurement checkpoint follows the P1-B Ã¢Ë†Â¥ P2A
join and uses the promotion mechanism in
[SPACE_PROMOTION_AND_EPOCHS.md](SPACE_PROMOTION_AND_EPOCHS.md). proceed is
one atomic policy-and-space act; djust re-enters P1 under a new committed
preregistration revision; kill blocks P2B and returns to sovereign
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
Tranche A (semantics-independent; may run concurrent with P1-B — pass-5
wording fix, consistent with the fork above): server binary
and lifecycle; deployment config and health; the §11 v1 client-authority
posture; upload transport and `commit_files`; intake status and
`watch_progress` read models; job/refusal/log read models;
recovery-on-restart; the **two-substrate backup/restore procedure** with a
restore test. Nothing in P1's findings can invalidate this tranche.
Tranche B (semantics-shaped; pins after the P1/P2A join): neighborhood, link,
explain-link, Postulant, matrix, and retrieval read models; the §3.1
link-and-weight override labor with its criteria; `links_for_node`;
`list_matrices`; link-policy and threshold parameters as configuration
surfaces; **the complete ceremony surface — classified honestly (pass-3) as
store/domain/orchestration work with criteria, not pure façade**: invoke audit
through its lawful trigger; read Gabriel's and Lucy's report states; read
barrier and reconciliation state; review the exact proposal revision;
**command-envelope ownership at the store layer (pass-5 — generalized from
the single resolve_proposal line):** the §11 envelope (idempotency key,
expected revision/state, freshness, exact hash where applicable) is
**store-owned for every reserved command in the §11.1 registry** — the
sovereign store method validates and records the envelope inside its own
transaction, so a crash between application layer and store can neither
double-apply nor lose a sovereign command; application-layer keys alone die
with the process (today only `set_config` carries `expected_revision`;
`resolve_proposal` accepts `(actor, proposal_id, decision)` bare, and the
petition, admission, decommission, and override surfaces carry no envelope
at all). One uniform store-side envelope mechanism, shaped at ADR-3, DDL at
ADR-2; **the production orchestration set (pass-4
completion — Stop-3 was still mechanically unreachable without it):** the
intake dispatcher deliberately ends at classification (the seam), and the
trial's transitions are test-invoked today, so P2B owns, each with retry,
idempotency, refusal, and restart criteria: the **processing-seam
dispatcher** (the human-reserved service that crosses the seam on the
operator's invocation or configured trigger — embedding/backfill,
consolidation, weight recalculation, emergence; this is IV.4's
seam-crossing entry given its production surface, D7-registered); the
**audit-barrier certification tick** (the supervisor behavior behind
`certify_audit_barrier`, which `invoke_audit` does not perform); the
**reconciliation dispatcher tick** (invoking `reconcile` behind the
certified barrier); and the **proposal-execution tick, with its restart
discovery named (pass-5)** — petition grants have a served dispatcher rule
(`stalled_grants` feeding `grants_tick`, godhead-notary), but consented
matrix proposals have **point lookup only** (`get_proposal`) and are
executed only by test callers today; the tick therefore requires a
**pending-consented-proposals query** (GRANTED proposals lacking a completed
execution, older than the stall window — the exact mirror of
`stalled_grants`) **plus a CAS claim at execution start**, so a restarted
supervisor rediscovers unexecuted consents from the store alone. Observe
Notary dispatch, execution, refusal, and completion; navigate the resulting
Cardinal; read the full proposal→consent→act chain. **Two more owned
executors (pass-6 — doc 05's Notary note already promises a dispatcher "on
any executable consent flag," and only petition grants have one):** the
**decommission-execution tick** (pending-decommission query + CAS claim —
R05's consent today mints a record no served path consumes) and the
**admission-processing tick** (admitted-yet-unprocessed quarantine
discovery — R06's admitted items must enter processing without a test
caller). Discovery for all ticks is immediate (zero-delay query); the
configured stall window is SC-C06 monitoring, never execution delay.
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

## 11. Application boundary and client authority

**Boundary.** A use-case façade, not the Store trait and not database-shaped
records. Candidate operations: `commit_files · get_node · get_neighborhood ·
list_postulants · explain_link · override_link · override_category ·
review_proposal · resolve_proposal · resolve_petition · consent_admission ·
consent_decommission · get_matrix · get_provenance · watch_progress ·
search · legibility_state` — plus the ceremony surfaces §8/P2B enumerates.
(Pass-5: the earlier generic `record_consent` is **retired** — the real
authority is operation-specific, and a generic consent verb would smuggle
distinct IV.4 entries under one name; each consent-bearing operation appears
by name in the §11.1 registry.) Versioned (v1) command/query contract; DTOs translating
records into operator-meaningful state; transport HTTP+JSON with SSE or a
socket for `watch_progress`; client packaging (browser vs Tauri-class) stays
open and does not change the contract. No speculative Store split — a
refactor is proposed when integration meets a concrete obstruction.

**Client authority (v1 posture — decision D7, ADR after answer).** The server
holds the G10 sovereign actor-class, which creates a confused-deputy risk
unless entry is narrow. The v1 answer, made explicit rather than letting
packaging decide it:

- One server-authoritative deployment; the backend holds authoritative state.
  "Local-first" means the store, server, and inference all live on the
  operator's own machine or home server — locality of the *system*, not
  offline-writability of the client.
- One enrolled operator client: enrollment via a pre-shared token at
  deployment; sessions via issued bearer tokens (header-carried, never
  cookies — which retires CSRF for the browser case); origin pinning for
  browser clients; TLS optional at loopback, required on LAN; revocation =
  token invalidation at the server; a lost client is revoked, not recovered.
- **Reserved operations are separately named (pass-3 correction — the pass-2
  list was not the 1:1 map it claimed):** the API's reserved-operation set is
  the union of (a) **Law IV.4's eight entries, enumerated exactly** — crossing
  the intelligent-processing seam (human-invoked processing dispatch);
  invoking rebalance outside a configured trigger; invoking audit; consenting
  to commitment; consenting to decommission; admitting external material at
  the threshold; authoring fetch mandates; **granting petitions**; altering
  sovereign config — and (b) the **non-IV.4 sovereign hands**, labeled as
  such, not smuggled under "consent": laying/releasing overrides
  (IV.1/IV.5/D8), weight correction, **resolving bias warnings**
  (`resolve_bias_warning`, interface.rs — a sovereign-actor store surface the
  pass-3 list missed), Concordat adoption, and — **pass-5 addition —
  pairing/re-pairing**: `form_pairing` today takes no actor and its
  implementation hardcodes `produced_by = 'sovereign'`, while the Holy
  Standard names re-pairing "a fresh sovereign act" (doc 06 §4.3) — an
  authority the perimeter must carry, supplied P5. **The handler registry is
  complete-but-phased (pass-4):** the registry names every reserved operation
  *now*, each entry carrying its supplying phase (ceremony surfaces and seam
  dispatch P2B; agent-labor operations P5; fetch mandates P6); an unshipped
  entry is registered-and-refusing, never absent — so the arch test that pins
  the enumeration pins the whole registry from its first landing, and no
  later phase adds an operation the registry never named. **The registry is
  literal, not a prose claim — §11.1 (pass-5).**
- **Enumeration is not authentication (pass-3 correction; perimeter
  corrected pass 4).** Today's store methods accept an actor *string* and
  elevate the transaction themselves — any caller that can reach them can
  supply `'sovereign'`. The application layer therefore supplies what the
  string cannot: authentication middleware mints an **unforgeable sovereign
  context** (a capability type constructible only inside the auth module —
  not request data, not a header echo); actor identity is derived from that
  context, never from the request body; **actor-class elevation occurs only
  after authorization succeeds**, inside the named handler. **The call-site
  wall is drawn around reserved *operations*, not sovereign *store* surfaces
  (pass-4 correction):** `invoke_audit` (godhead-audit) and `rebalance_now`
  (godhead-ml) are **public library functions**, not store methods — a wall
  watching only Store-trait sovereign surfaces never sees them, and the
  processing-seam dispatcher (§8/P2B) will be a third such function.
  **Enforcement shape (pass-5 correction — pass 4 presented two branches as
  equivalent; they are not):** while the Store trait and the library
  functions remain publicly callable, a gating module plus a source-level
  call-site test proves only that **no current caller bypasses the gate — it
  does not make the bypass uncallable**; every future code path still
  compiles against the same public surfaces. The recommendation is therefore
  **capability-bearing signatures on every reserved operation** — store
  methods and library functions alike take the sovereign-context type, whose
  single construction site lives in the auth module, making an ungated call
  *unconstructible*, not merely untested-for. The gating-module branch
  remains lawful **only with structural sealing**: the sovereign entrypoints
  split into a non-exported trait/module reachable solely through the gate,
  visibility-enforced — which is the P2A refactor this document already
  reserves, priced honestly rather than presented as the equal cheap branch.
  Either way, the arch test enumerates **every entry in the §11.1
  registry** — store methods, library functions, and dispatchers — as a
  tripwire *behind* the structural wall, never in place of it. End-to-end
  tests run the full path from unauthenticated request through persistence,
  proving rejection before elevation.
- Sovereign command envelope: idempotency key; expected revision; **exact
  proposal hash** for consent commands (consent binds to the revision the
  sovereign actually reviewed — a store-method extension, §8/P2B); request
  freshness window.
- Disconnected client: may read cached views and stage files locally; **no
  offline sovereign writes, no queued consents** (a consent decided against
  stale state violates exact-revision freshness); commits are staged-not-
  committed until online (doc 02 §1.1's deliberate commit survives intact).
- Tauri-class local IPC narrows the network threat model but changes no
  authority rule above.

### 11.1 The reserved-operation registry (pass-5 — literal, complete, phased)

The completeness claim is a table, not prose. Envelope codes (store-owned,
§8/P2B): **K** idempotency key, **bound to the request digest** (pass-6 — the
same key with different content refuses, never converges) · **R** expected
revision/state CAS · **F** freshness window · **H** exact content hash ·
**rcpt** = **durable command receipt (pass-6):** a single Store transaction
cannot envelope a multi-step library operation (`invoke_audit` and
`rebalance_now` perform several Store writes and can fail between them), so
multi-step reserved operations record an accepted command (key, hash, actor,
freshness, expected state), progress steps, and a terminal
completion/refusal — retry resumes the standing command, duplicates converge
on it, and no one pretends the downstream effects were one transaction.
Class `sov-cap` = requires the unforgeable sovereign context; `op-session` =
authenticated operator session without elevation; `machine` = job identity
under recorded standing authority (R01b — never a forged capability).
**Every entry whose phase has not arrived is registered and refusing (closed
reason `UNSHIPPED_OPERATION`) from the registry's first landing — never
absent.** The arch test enumerates every row.

| ID | Operation → callable | Accepted outcomes | Authority | Class | Phase | Env | Restart / discovery |
|----|----------------------|-------------------|-----------|-------|-------|-----|---------------------|
| R01a | Seam dispatch, human "run now" → dispatcher fn (§8/P2B, new) | dispatched · refused | IV.4 — seam crossing (live invocation) | sov-cap | P2B | K·F·rcpt | job records + readiness flags rescanned |
| R01b | Seam dispatch, configured trigger → tick (machine executor; **pass-6 split — a scheduler must not forge a sovereign capability**) | dispatched · refused | recorded **standing trigger revision** (IV.4's "user-configured trigger" carve-out; the authority is the recorded configuration, cited per run) | machine (job identity + trigger rev) | P2B | trigger rev + rcpt | same |
| R02 | Direct rebalance → `rebalance_now` (godhead-ml) | recalculated · refused | IV.4 — rebalance **outside** trigger (within-trigger runs are machine, per the R01b principle — IV.4's own wording already splits this) | sov-cap | P2B | K·F·rcpt | `WEIGHT_RECALC` log + `RebalanceState` |
| R03 | Invoke audit → `invoke_audit` (godhead-audit) | audit opened · refused | IV.4 — invoking audit | sov-cap | P2B | K·R·F·rcpt | trial job records; `AUDIT_OPENED` cites evidence set (§10); opening is the atomic op of §10 |
| R04 | Resolve proposal → `resolve_proposal` | grant · decline | IV.4 — consent to commitment | sov-cap | P2B | K·R·F·H | pending-consented-proposals query → execution tick (§8/P2B) |
| R05 | Consent decommission → `consent_decommission` | consent minted · refused | IV.4 — consent to decommission | sov-cap | P2B | K·R·F | **pending-decommission query + CAS-claimed Notary tick (pass-6 — canon's dispatcher note already promises it; no served path exists)** |
| R06 | Consent admission → `consent_admission` | admit · reject | IV.4 — threshold admission | sov-cap | P2B | K·R·F·H | quarantine / Manifest state (0014); **admitted-unprocessed discovery tick (pass-6)** |
| R07 | Resolve petition → `resolve_petition` | grant · decline · silence | IV.4 — granting petitions | sov-cap | P2B | K·R·F·**H (pass-6: hash over the immutable occurrence/TransitionPlan)** | `stalled_grants(0)` immediate discovery → `grants_tick`; stall window is monitoring, not delay |
| R08 | Author mandate → `author_mandate` | authored · refused | IV.4 — fetch-mandate authorship (C.4) | sov-cap | P6 | K·F·H | mandates table (0013) |
| R09 | Sovereign config → `set_config`, SOVEREIGN tier (incl. checkpoint threshold adoption, atomic with R19 at `proceed`) | set · stale-refused | IV.4 — altering sovereign config | sov-cap | checkpoint¹ | K·R·F | `config_constants`; **ConfigHistory once amended (pass-6: no A.5 event exists today)** |
| R10 | Operational config → `set_config`, OPERATIONAL tier | set · stale-refused | operational tier (non-reserved authority, same envelope) | op-session | P2B | K·R·F | same |
| R11 | Lay category override → `lay_category_override` | laid · refused | IV.1 hand (non-IV.4) | sov-cap | P2B | K·R·F | override lineage |
| R12 | Lay link override (sever / force) → §3.1 surface (new) | laid, with chained closures per compatibility table · refused | IV.1 / doc 04 §4.4 (non-IV.4) | sov-cap | P2B | K·R·F·**H (pass-6: hash over every closure and resulting state — the TransitionPlan)** | per-kind lineage epochs |
| R13 | Lay weight override → `lay_weight_override` (new) | laid · refused | IV.1 weight correction (non-IV.4) | sov-cap | P2B | K·R·F | lineage head |
| R14 | Release override, per kind → D8 surface (new) | released (weight: + `RELEASED_AS_STANDS` evidence) · stale-refused | IV.5 / D8 (non-IV.4) | sov-cap | P2B | K·R·F | release records + `OVERRIDE_RELEASED` |
| R15 | Resolve bias warning → `resolve_bias_warning` | acknowledge · silence | HS §6.3 (non-IV.4) | sov-cap | P2B | K·**R (expected warning state — pass-6)**·F | `bias_warning_state` |
| R16 | Adopt Concordat → `adopt_concordat` | adopted · refused | A.14(b), HS §3.3 (non-IV.4) | sov-cap | P5 | K·R·F·H | versions retained forever (§3.3) |
| R17a | Form pairing (initial) → `form_pairing` (today actor-less; impl hardcodes `'sovereign'`) | formed · refused (tier mismatch) | **implementation-sovereign; canon silent on initial formation (pass-6 split — A.10 carries no authority field; only re-pairing is canon-named): the D7 answer confirms or assigns this authority** | sov-cap (pending confirmation) | P5 | K·R·F | pairing records |
| R17b | Re-form pairing after orphaning → `form_pairing` successor path | formed · refused | HS §4.3: "re-pairing is a fresh sovereign act" — canon-explicit | sov-cap | P5 | K·R·F | pairing records + A.8 env status |
| R18 | Adopt embedding space → §10 surface (new) | PREPARING created · refused | D4b migration-class (non-IV.4) | sov-cap | P1-A¹ | K·F | space catalog |
| R19 | Activate embedding space (atomic policy+space promotion at `proceed`) → §10 surface (new) | activated · refused (blockers listed; stale `GraphEpoch`) | D4b + §10 activation invariant | sov-cap | checkpoint¹ | K·R·F·**H (pass-6: hash over the explicit Postulant disposition map)**·rcpt | catalog + disposition list |
| R20 | Abandon embedding space (incl. after machine-marked generation failure — §10 authority unification) → §10 surface (new) | abandoned | D4b (non-IV.4) | sov-cap | checkpoint¹ | K·R·F | catalog |
| R21 | Resolve admission standing notice → surface (new — **no store fn exists; pass-6**) | acknowledge · silence | SC-I07b (Book II §1 doctrine, ruling G11) | sov-cap | P2B | K·R·F | Manifest notice state |
| R22 | Lift a silenced scope (bias pattern) → surface (new — **canon names the lift; no surface exists; pass-6**) | lifted · refused | HS §6.3 / SC-K07: "not re-raised until the sovereign lifts it" | sov-cap | P2B | K·R·F | `bias_warning_state` |
| R23 | Render `SOVEREIGN_JUDGMENT` verdict on a Return criterion → attributed review record (new — **the Return is flagged and immutable; the verdict record is separate; pass-6**) | passed · failed · returned-for-rework | HS §1.3d / B.2: "verdict rendered at sovereign review" (non-IV.4) | sov-cap | P5 | K·R·F·H | review records keyed to `(return_ref, criterion_ref)` |

¹ Checkpoint-phased (pass-6 — the pass-5 footnote's "recorded direct
invocation under the dev register" was process discipline standing in for
the D7 boundary, and is withdrawn). The split is truthful about what each
act needs: **R18 adoption cannot wait for the join** — P1-B evaluates the
PREPARING candidate — and does not need to: adoption is a **migration-class
sovereign act** (§10), and the pinned P1-A slice *is* that act's authority,
identically to every one of the 18 migrations to date (pin-time signature,
two-commit lifecycle). R20 abandoning a failed candidate before the
checkpoint rides the same pinned-slice authority or the checkpoint's
adjust/kill routing. **R09 and R19 wait for the post-join checkpoint**,
where P2A's capability and store-owned envelope substrate exist. Full API
handlers for all four ship with the P2B registry, refusing until then per
the unshipped rule.

**Excluded by design:** machine executors — `execute_grant`,
`execute_matrix_proposal`, the ticks, backfill — are agent labor under job
identity, dispatched by served rules, never operator-invocable operations;
they appear in the restart/discovery column, not as rows. **Client
enrollment, credential rotation, and revocation are deliberately not rows
here (pass-6):** they govern who may hold a session at all — a distinct
**authentication-control registry** owned by ADR-3, with the same
literal-table discipline, kept separate so authority-over-data and
authority-over-access never share a namespace. A generic
`record_consent` does not exist here (each consent authority is
operation-specific). Every row's outcomes column names **every** accepted
outcome — decline, silence, and refusal are first-class results, not error
paths. **Every consent that creates asynchronous labor names its authoritative
pending query, CAS claim, restart behavior, and supplying phase in its row
(pass-6)** — a consent exposed through the API never depends on a test
caller; and discovery is **immediate** (the zero-delay query `grants_tick`
already demonstrates: `stalled_grants(0)`), with the configured stall window
serving SC-C06 *monitoring*, never delaying normal execution.

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

## 15. Decision sheet (revised, pass 6)

Eight decisions — D7 added by sovereign instruction (client authority), D8 by
pass-3 finding (representation gaps no other decision lawfully covers).
Recommendations are this document's; decisions are the sovereign's.

  **D1 — The revised phase sequence (P0 → P1-A → fork{ P1-B ∥ P2A } → join →
  **sovereign policy/space checkpoint** → P2B → P3 → P4-A → P4-B(loop) → P5 →
  P6 → P7; checkpoint node made explicit by pass 6 — it consumes P2A's
  authority substrate and P1-B's evidence; §8 as amended by pass 4:
  the fork moves after P1-A — P2A consumes the workspace manifests and walls
  P1-A builds — and every non-`proceed` verdict now has a route: P1 `adjust`
  re-runs P1-B under a new preregistration revision; P1 `kill` blocks P2B and
  returns to sovereign reconciliation; P4 `adjust` and `simplify ceremony`
  re-run P4-B before P5).**
  Adopting D1 with D4a **requires adopting or amending D3 before P1** — see
  D3's coupling note.
  Recommendation: adopt as amended.          [adopt / reject / amend]

  **D2 — Offline-before-spine vs bounded concurrency.**
  Recommendation: bounded concurrency, as a *scheduling* freedom only —
  separately pinned, separately reviewed, serially landed slices; the shared
  surfaces (workspace manifests, D3 walls) land once in P1-A and are consumed
  thereafter.                                [offline-first / bounded concurrency]

  **D3 — Replace the universal transport wall with capability boundaries
  (inbound / model-egress / fetch-egress), reachability-enforced (§9);
  detaches SC-F06's half to model-egress (completed P5); preserves the
  fetch-deletion conditions verbatim.**
  **Coupling (pass-3):** this is not an independent choice. P1's production
  adapter is impossible under the universal wall — **deferring D3 forces
  either P6-before-P1 (the existing sequence) or an in-process architecture
  amendment to doc 04 §2.2**. The lawful combinations are: {D3 adopt/amend +
  D1 adopt} or {D3 defer + D1 reject/amend}. The sheet offers no incoherent
  middle.
  Recommendation: adopt.                     [adopt / amend / defer-with-stated-consequence]

  **D4 — Embedder path and embedding-space policy (§10, as amended pass 5).**
  (a) Path: production adapter built in P1-A on doc 04 §2.2's
  separate-process architecture; in-process inference as named contingency
  only. (b) Policy: EmbeddingSpace identity; **PREPARING→ACTIVE|ABANDONED,
  ACTIVE→RETIRED** with **three-generation storage** (active typed +
  preparing typed + retired archive, atomic catalog switch); the
  **bond/measurement split completed** (§3.1 — geometry-neutral bonds,
  space-keyed `BondQualification` and `WeightEvidence` records, weight never
  riding the bond row, unreleased forced bonds counting in every space);
  **`TrialEvidenceSet` (pass-5)** — an immutable evidence manifest frozen at
  audit-open and cited identically by both reports, the Joint Proposal, the
  consent, and the Cardinal (space, data/config epoch, threshold snapshot,
  qualification set, weight run identity, per-bond effective weight source,
  membership, override-head epoch; AMEND freezes the successor set at
  re-audit); **total activation disposition of old-space Postulants
  (pass-5)** — auto-supersede (pristine / barrier-complete-unproposed /
  declined-or-halted), block (granted-unexecuted, running execution, live
  trial jobs), explicit sovereign disposition (unresolved proposals,
  AMEND-awaiting-re-audit), with `SUPERSEDED` terminal; **activation
  serialization (pass-5; token corrected pass-6)** — per-matrix CAS,
  space-predicated trial writes, and a **transactional `GraphEpoch`
  CAS-validated at the catalog switch** (the A.5-position token was unsound:
  identity ≠ commit order, logs are autocommit-separate, `set_config` logs
  nothing), **with emergence sharing the serialization unit** (every
  `emerge_postulant` insert predicates on expected active space + catalog
  revision and records the epoch it emerged against); named amendment
  targets: Dogma VI/A.9's status grammar **plus
  A.9's new space/evidence identity**, doc 03 §2.4, migration 0005's
  predicate, **A.5's closed taxonomy** (D4 owns six new events:
  `SPACE_ADOPTED`, `SPACE_ACTIVATED`, `SPACE_ABANDONED`, `SPACE_RETIRED`,
  `MATRIX_SUPERSEDED`, `CONFIG_CHANGED` — **`OVERRIDE_RELEASED` moved to
  D8's ownership, pass-6: adopting D4 must not ratify a D8 event**);
  **five further named literal amendments carried by
  adoption** (doc 03 §2.2, doc 04 §4.1, SC-M05 — per node *per valid space*;
  doc 03 §2.3 and doc 04 §4.2 — similarity/weight move off the link record
  into evidence); `space_ref` propagation per §10; floor named space 0;
  Law-IV fixed stars; **Cardinals frozen under new geometry — drift is
  advisory, replacement is decommission + fresh emergence**. **Pass-6
  additions:** the evidence manifest binds **immutable values and algorithm
  identities, never mutable revision pointers** (threshold and
  link-similarity values verbatim; qualification and weight algorithm
  versions; reasoner digest and prompt/policy version; calculation-run
  input/output digests) — and gains a **validity state machine**:
  current-state revalidation chosen (snapshot isolation declined), the
  evidence epoch and expected head set CAS-revalidated at barrier
  certification, proposal filing, consent, and Notary execution, drift →
  SUPERSEDED/DISTRUSTED and re-audit; **trial opening is one atomic,
  retry-stable store operation** (one evidence set per matrix revision,
  eligibility consumed, `AUDIT_OPENED` in-transaction); the disposition
  table gains the **incomplete-trial bucket** and routes executed consents
  **by verdict** (COMMIT/AMEND/REJECT — not "always a Cardinal"), with the
  explicit-disposition branch carried as **R19's exact-hashed map**;
  generation failure is machine-marked, **abandonment is the sovereign's
  (R20)**; P1-B evaluates through a **`CandidateEvaluationContext`**
  (PREPARING space, staged policy snapshot, non-authoritative outputs,
  production adapter); `proceed` is **one atomic policy+space promotion**
  (set-then-activate declined — it invalidates certification and briefly
  governs space 0 with the winning threshold); `adjust` re-certifies under
  the replaced snapshot; a **final untouched holdout** survives all adjust
  cycles; **`ConfigHistory`/`CONFIG_CHANGED` is a named amendment** (prior
  config values are unrecoverable today — `set_config` writes no event).
  DDL to ADR-2; the post-join sovereign checkpoint (§8/P1) exercises
  adoption, promotion, and abandonment.
  Recommendation: adopt both as amended.     [adopt a+b / amend]

  **D5 — Criteria-register authority (§14).**
  Recommendation: dev-register with promotion path and append-ledgered
  preregistration.                           [dev-register / canonical doc 09]

  **D6 — P0 fixed closure checklist (§8/P0, exit as revised).**
  Recommendation: adopt; fixed boundary; exclusions binding.
                                             [adopt / amend]

  **D7 — Client-authority v1 posture (§11, as amended passes 3–5):**
  server-authoritative; one enrolled operator client; **separately named
  reserved operations covering IV.4's entries exactly plus the labeled
  non-IV.4 sovereign hands — now including `resolve_bias_warning` and
  (pass-5) pairing/re-pairing** (`form_pairing` is actor-less today and
  hardcodes sovereign authorship; doc 06 §4.3 names re-pairing a fresh
  sovereign act); the registry now **literal — §11.1** (stable IDs, every
  accepted outcome, authority source, class, phase, envelope fields,
  unshipped behavior, restart/discovery; generic `record_consent` retired);
  an **unforgeable sovereign context** minted only by authentication
  middleware, actor identity never derived from request data, elevation only
  after authorization; **enforcement by capability-bearing signatures on
  every reserved operation (pass-5 — the recommended shape):** a gating
  module with publicly callable entrypoints is *not* equivalent (a call-site
  test proves no current bypass, not an uncallable one) and is lawful only
  with structural sealing of the sovereign entrypoints — priced as the P2A
  refactor; the arch test covers every §11.1 row as a tripwire behind the
  type-level wall; end-to-end unauthenticated-to-persistence tests;
  **store-owned command envelopes for every registry row (pass-5)** —
  idempotency, expected revision/state, freshness, and exact hashes validated
  and recorded inside the sovereign store method's own transaction, never
  application-layer only; no offline sovereign writes. **Pass-6 additions:**
  the registry splits **live invocation from standing-trigger execution**
  (R01a/R01b — a scheduler never forges a capability; it acts under a
  recorded trigger revision) and **initial pairing from re-pairing**
  (R17a/R17b — canon names only re-pairing sovereign; initial formation is
  implementation-sovereign pending the D7 answer); gains **R21 admission-
  notice resolution (SC-I07b), R22 silenced-scope lift (SC-K07 names the
  lift; no surface exists), R23 persisted `SOVEREIGN_JUDGMENT` verdicts
  (B.2's review verdict currently has no record)**; **multi-step operations
  carry durable command receipts** (single-transaction envelopes cannot
  cover `invoke_audit`/`rebalance_now`); **idempotency keys bind to request
  digests**; missing async executors are owned (decommission-execution and
  admission-processing ticks — canon's dispatcher note promises them);
  discovery is zero-delay with the stall window as monitoring only;
  enrollment/rotation/revocation live in ADR-3's separate
  **authentication-control registry**; and the pass-5 "recorded direct
  invocation" footnote is withdrawn — **checkpoint acts wait for P2A's
  capability substrate at the join**. ADR extracted after answer.
  Recommendation: adopt as amended.          [adopt / amend]

  **D8 — Override-release representation and kind-aware link protection
  (§3.1; new pass 3, amended passes 4–5).** (a) Release: canon/schema
  amendment making release representable — recommended shape: **per-kind
  override lineages** (one active head per `(subject_ref, override_kind)`);
  release as a separate `OverrideReleaseRecord` (attributed, chained,
  **closing exactly one kind's head** by compare-and-set — the aggregate
  held marker clears only when the last head closes) + `OVERRIDE_RELEASED`
  taxonomy event; alternative: an ACTIVE|RELEASED status on A.7.
  **Petition-head binding rides with (a) (pass-5 — the release-defeats-grant
  claim was refuted against the schemas):** petitions gain
  `target_override_ref` recorded at open against the petitioned kind's
  head; the grant binds consent to that target and re-verifies it; the
  executor refuses a moved lineage (`TARGET_RELEASED`/`TARGET_SUPERSEDED`);
  laying, succession, and release all **CAS the same lineage head** — never
  the bond row (a weight act need not touch the bond). (b) Protection is
  override-kind-aware; guards derive from **all unreleased heads** under the
  **pass-5 compatibility table** (§3.1): FORCED ∥ WEIGHT-CORRECTED
  compatible; WEIGHT_CORRECTED on a severed bond refused; FORCED ↔ SEVERED
  mutually exclusive with **atomic supersession** (one act, chained closure
  records — the two-ceremony alternative recorded and declined); the link
  row's `user_overridden` becomes a derived aggregate marker, **never a
  mutation guard**. `LINK_FORCED` holds existence/category only; **weight
  lives entirely off the protected record** (§3.1's `WeightEvidence` —
  SC-C01 by construction); **`WEIGHT_CORRECTED` is geometry-neutral,
  projected into every space (pass-5 choice)** — space-scoped correction
  declined as IV.1-by-migration; drift under a new space is advisory;
  **release mints a `RELEASED_AS_STANDS` evidence record** — the last
  human-held value stays effective until the next lawful recalculation, and
  immediate fallback to older machine evidence is forbidden as machine
  restoration. `lay_weight_override` joins P2B. **Pass-6 additions:** the
  concurrency substrate is the **`OverrideLineage` control record** — a
  monotonic `lineage_epoch` per `(subject, kind)` advancing while active
  *and empty* (the empty-head ABA is otherwise unguardable), composite acts
  locking their full expected lineage set in deterministic kind order under
  one `transition_id`; petitions become a **canonical lineage aggregate over
  immutable `PetitionOccurrence` records** (migration 0003's single mutable
  row overwrites `reason`/`proposed_change` and NULLs
  `consent_ref`/`execution_job_ref`/`resolved_at` on recurrence — already
  erasing execution witnesses today), with consents and successors citing
  the occurrence; the occurrence separates **petition class from resulting
  kind** (today one overloaded `change_kind`); **SILENCED binds the exact
  head epoch silenced** (subject/kind-wide gag declined); composite consent
  binds an immutable **`TransitionPlan`** (every expected head + epoch,
  every closure, result, one hash, one transition id — the Notary never
  closes a head the sovereign did not review; the same-kind-only
  alternative declined as re-splitting the single-act model); a **four-kind
  release table** completes CATEGORY_REASSIGNED and LINK_SEVERED semantics;
  `RELEASED_AS_STANDS` is **geometry-neutral state on the release act and
  lineage record, not a space-keyed evidence row** (pass-5's shape was
  schema-incompatible); **machine `WeightEvidence` writes carry the
  weight-lineage epoch captured at calculation start** (release advances
  it — the pre-release calculation race refuses at write), with the literal
  effective-selection order (active head → as-stands for current epoch →
  same-epoch post-trigger machine evidence); sever-closure of a weight head
  mints **no** effective as-stands (severed bonds are unread) and a later
  re-force does **not** revive the old value; and **SC-C01 is resolved, not
  assumed** — effective state derives from append-only lineage acts (the
  held base record is never mutated; the materialized marker is
  lineage-derived bookkeeping) **and** SC-C01's text is amended to admit
  the authenticated exact-hash sovereign `TransitionPlan` as lawful
  authority beside granted consent. `OVERRIDE_RELEASED` is **owned here, by
  D8** — adopting D4 does not ratify it. Neither (a) nor (b) is authorized
  by D1.
  Recommendation: adopt (a) with occurrences, lineage epochs, and
  TransitionPlans; adopt (b) with the compatibility table, the four-kind
  release table, and geometry-neutral weight correction.
                                             [adopt a+b / amend]

## 16. ADR extraction (after decisions return)

Accepted mechanisms leave this document for independently supersedable ADRs,
with only sequencing consequences and status pointers retained here:
**ADR-1 Transport capabilities** (D3 — boundaries, reachability tests, SC-F06
reassignment, SLICE_11 §0 supersession text); **ADR-2 Embedding spaces** (D4b
— generation DDL, lifecycle mechanics including failure-marking, sovereign
abandonment, and the total Postulant disposition,
`BondQualification`/`WeightEvidence`/`TrialEvidenceSet` DDL, the
**`GraphEpoch` participants and the fence-vs-catch-up implementation**, the
evidence **validity state machine and atomic trial opening**, the
`CandidateEvaluationContext`, atomic policy+space promotion, propagation,
backfill/index/activation procedure, the named canonical amendments);
**ADR-3 Client authority** (D7 — enrollment, tokens, sovereign-context
capability, **capability-bearing signatures as the recommended perimeter**
[the sealed-module branch priced as its P2A refactor], the §11.1 registry
with its splits and receipts, the store-owned envelope mechanism and
**durable command receipts**, the separate **authentication-control
registry**, threat model by packaging); **ADR-4 Override semantics** (D8 +
§3.1 — **`OverrideLineage` epochs, immutable `PetitionOccurrence`s,
`TransitionPlan`s**, the kind-compatibility and four-kind release tables,
release representation including geometry-neutral `RELEASED_AS_STANDS`,
epoch-fenced evidence writes, the SC-C01 resolution, the link/weight labor
set). The criteria register (D5) is born separate.

## 17. Review history

Passes 2â€“6, including every visible retraction and historical HEAD claim, moved
verbatim to [REVIEW_LEDGER.md](REVIEW_LEDGER.md). The ledger is historical;
current mechanisms live only in their owning annexes.
