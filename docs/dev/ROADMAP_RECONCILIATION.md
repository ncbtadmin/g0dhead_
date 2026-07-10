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

### 3.1 The link-override labor — semantics and criteria (P2 Tranche B; revised passes 3–6)

`LINK_SEVERED` and `LINK_FORCED` have carried schema (A.7) and *protection*
enforcement since slices 3–4 (`draw_link` skips overridden rows;
`set_link_weight` refuses `OVERRIDE_CONFLICT`), but no laying surface exists at
any layer (SLICE_03 §2 deferred it; slice 4 built the walls, not the hand).
Proposed semantics, for the P2B pin:

- **The bond/measurement split, with qualification and versioned weight
  (pass-3 correction, pass-4 completion).** The relationship record splits
  into a **geometry-neutral bond** (endpoints in canonical order, category,
  held-state, history — and, per pass 4, **nothing computed**: the protected
  bond row is immutable, so machine writes never touch a `user_overridden`
  record and SC-C01/IV.1 hold by construction) and two kinds of
  space-keyed evidence:
  `BondQualification {bond_ref · space_ref · link_policy_rev · similarity ·
  qualified}` — because "measured in a space" is not "qualifies in a space":
  a measurement can fall below `link_similarity_threshold`, and the policy
  revision can change after measurement. Every graph, density, and
  consolidation query binds **(active space, applicable policy revision)**;
  a machine bond counts in a space only where a `qualified` record under the
  bound policy exists — never merely because the space measured it.
  `WeightEvidence {bond_ref · space_ref · weight_policy_rev · mode · weight}`
  — weight derives from space-specific similarity, degree, mode, and reasoner
  output, so it is evidence, not a scalar on the neutral bond; reweighting
  under space B **never overwrites** the weight that explained a Cardinal
  committed under space A. **A Cardinal retains, at commitment, its
  `space_ref` and the weight-evidence revision set it was tried on**; retired
  qualification and weight evidence stay queryable for historical
  explanation. A forced bond exists because the hand said so and needs no
  qualification in any space. Exact DDL belongs to ADR-2.
- **Severing** never deletes. The bond gains a severed state in one
  transaction with the chained OverrideRecord (kind `LINK_SEVERED`) and the
  `LINK_SEVERED` log event (already in A.5). Every consumer — reads, weights,
  density, consolidation — excludes severed bonds, in every space. History
  persists.
- **Forcing** creates a human-held bond at birth: category assigned by the
  hand; per-space similarity recorded as evidence where computable
  (informational — the bond's authority derives from the hand, not the
  metric); OverrideRecord (kind `LINK_FORCED`) + `LINK_DRAWN` log with
  sovereign provenance.
- **Field-specific protection via simultaneous per-kind heads (pass-3
  correction, pass-4 completion).** Canon separates `LINK_FORCED` from
  `WEIGHT_CORRECTED`, and both can stand on one bond at once — so the
  active-protection model becomes **one active lineage per
  `(subject_ref, override_kind)`**, not newest-per-subject. Guards derive
  protection from **all unreleased heads**: the weight-evidence path refuses
  where a `WEIGHT_CORRECTED` head stands; existence/severance paths refuse
  where `LINK_FORCED`/`LINK_SEVERED` stands. Because computed weight lives in
  `WeightEvidence` records rather than on the protected bond (above), machine
  reweighting of a forced-but-not-weight-corrected bond writes *new evidence*
  and mutates no `user_overridden` record — SC-C01 is satisfied, not
  excepted.
- **Kind compatibility and precedence (pass-5 — per-kind heads made the
  legal combinations decidable, then left them undecided).** By subject
  domain: `CATEGORY_REASSIGNED` holds nodes; the other three hold bonds. On
  one bond: `LINK_FORCED ∥ WEIGHT_CORRECTED` — **compatible** (existence and
  category held; influence held; independent lineages, independent release).
  `LINK_SEVERED ∥ WEIGHT_CORRECTED` — laying a weight correction on a
  severed bond is **refused** (`OVERRIDE_CONFLICT`-class reason: a severed
  bond is excluded from every consumer; there is no influence to hold);
  laying `LINK_SEVERED` while a `WEIGHT_CORRECTED` head stands closes the
  weight head **in the same transaction**, by a chained release record
  attributed to the sever act. `LINK_FORCED ∥ LINK_SEVERED` — **mutually
  exclusive by definition**; laying either while the other stands is an
  **atomic supersession**: one sovereign act, one transaction, closing the
  opposing head with a chained release record and opening the new head —
  never two simultaneous heads, never silent (every closure is its own
  attributed record naming the act that caused it). The strict alternative —
  refuse, demand explicit release first, two ceremonies — is recorded and
  declined: routine corrections are single sovereign acts (doc 04 §4.4; the
  P4 ceremony metric depends on it), and the chained records preserve full
  visibility. **The aggregate marker:** the link row's `user_overridden`
  boolean becomes a derived "one or more heads active" marker for display
  and indexing — **never again a field-level mutation guard** (guards
  consult per-kind heads only). That re-scopes doc 04 §4.4's "overrides set
  `user_overridden:T`" and doc 03 §2.3's field list — two more named
  amendments (§10, D4/D8).
- **Petition binding to lineage heads (pass-5 — the pass-4 release sentence
  is REFUTED as written; pass-6 — one `target_override_ref` proved
  insufficient and the single mutable petition row proved lossy; three
  artifacts replace both).** Pass 4 claimed release defeats
  granted-unexecuted Notary acts because "the grant's chain no longer
  resolves against a released head." The repository refutes the mechanism:
  `PetitionRecord` carries `subject_ref` and `change_kind` but no head
  reference; `open_petition` verifies only that *some* active override
  exists — kind-blind; `execute_grant` loads whichever override is active at
  execution. The recurrence path is lossier still: migration 0003 keeps
  **one mutable row per `(subject_ref, change_kind)`**, and an ESCALATED
  recurrence overwrites `reason`/`proposed_change` and **NULLs
  `consent_ref`, `execution_job_ref`, `resolved_at`** — erasing, already
  today, the execution witness a prior grant left on the aggregate. Three
  artifacts carry the correction (P2B substrate; representation decided at
  D8):
  **(1) `OverrideLineage` — the per-kind concurrency control.** One
  persistent control record per `(subject_ref, override_kind)`: a
  **monotonic `lineage_epoch`** advancing on every lay, succession, and
  release — **while active and while empty**, defeating the empty-head ABA
  (a stale command that observed "no active head" cannot succeed against
  the same apparent null after an intervening lay-and-release); the current
  head or null; the last transition reference. Every mutating act CAS-es
  the epoch; composite acts lock their **complete expected lineage set in
  deterministic kind order** under **one shared `transition_id`**, and the
  derived aggregate marker updates in the same transaction. The lineage
  record is the **concurrency witness** — `OverrideRecord` and
  `OverrideReleaseRecord` remain the append-only history of acts.
  **(2) `PetitionOccurrence` — immutable asks under the canonical
  aggregate.** The `(subject, kind)` lineage aggregate stays — SC-C02's
  recurrence-escalation and SC-C03's suppression semantics require it — but
  every attempt becomes an **immutable occurrence** binding: the exact
  target head set with expected lineage epochs; the requested transition
  and its content hash; and, on resolution, the decision, consent, and
  execution refs. **Consents and successor provenance cite the occurrence,
  never the mutable aggregate.** `change_kind` stops being overloaded
  (today it is both petition class and successor kind — a
  sever-this-forced-bond ask cannot state that its target is `LINK_FORCED`
  while its result is `LINK_SEVERED`): the occurrence names the petition
  class and the requested resulting kind separately. **SILENCED scope,
  decided:** silence binds **the exact head epoch silenced** — recurrence
  against the same standing head stays suppressed (`severity: suppressed`,
  SC-C03, never purged); a petition against a *new* head is a new question
  and opens a fresh occurrence. The subject/kind-wide alternative is
  declined: one silencing would gag agents across arbitrarily many future
  distinct hands, beyond what IV.2's escalation semantics imply. Either
  scope is an IV.2/SC-C02/SC-C03/A.7 amendment; the chosen one is named in
  §21's matrix.
  **(3) `TransitionPlan` — the consent object for anything touching more
  than one head.** The compatibility table permits composite acts —
  severing a forced, weight-corrected bond closes `LINK_FORCED`, closes
  `WEIGHT_CORRECTED`, and opens `LINK_SEVERED` — and one target ref cannot
  authorize three closures: **the Notary must never close a head the
  sovereign did not review.** A granted composite petition therefore binds
  an immutable plan: subject; every expected active head with its lineage
  epoch; every closure; resulting kind and protected state;
  effective-state consequences; one content hash; one `transition_id`. The
  Notary validates the whole plan against the live lineage set and refuses
  any mismatch (`TARGET_RELEASED`/`TARGET_SUPERSEDED` — new closed A.4
  reasons); **direct sovereign composite acts bind the same plan shape**
  (SC-C01 resolution below). The same-kind-only alternative — refuse
  composite petitions, route to direct acts — is declined: it re-splits the
  single-act correction model doc 04 §4.4 and the P4 ceremony metric depend
  on. Recorded here; decided at D8.
- **Release, per kind (D8):** a release names the current head of exactly
  one kind, CAS-guards that head's lineage epoch (a stale release fails),
  leaves other active kinds untouched, and — via the occurrence/plan
  binding above — **defeats any earlier-granted, unexecuted Notary act
  targeting that head**, refusal surfacing on the petition as ever;
  released state is never silently re-frozen. **Density after release:**
  only a bond with an *active, unreleased* `LINK_FORCED` head counts in
  every space; once released it counts only where active-space
  qualification exists — a historically forced bond does not haunt the
  density after the hand lifts.
- **Release semantics, all four kinds (pass-6 — the table pass 5 gave for
  two):** for each kind on release: *retained state* · *effective read* ·
  *lawful replacement trigger* · *stale-work fence*.
  **`CATEGORY_REASSIGNED`** — node keeps the corrected classification
  as-stands; reads serve it unchanged; replacement only by fresh
  classification under a lawful processing trigger, which must carry the
  category-lineage epoch captured at its start (release advances it, so
  pre-release classification work refuses); a later hand may of course
  re-lay.
  **`LINK_SEVERED`** — the bond *stays severed as-stands* (release lifts
  protection against machine change, it does not resurrect the bond);
  consumers continue excluding it; replacement = the machine may re-draw
  under current policy only via lawful linking triggers carrying the
  post-release epoch, or the hand forces.
  **`LINK_FORCED`** — the bond persists as an ordinary machine bond
  candidate; effective existence follows active-space qualification (the
  density rule above); severance becomes machine-possible again only under
  post-release-epoch policy evaluation.
  **`WEIGHT_CORRECTED`** — see the dedicated bullet below: as-stands value
  effective until lawful recalculation, epoch-fenced.
  Under a later **cross-kind transition**, retained released state neither
  blocks nor revives: the transition's plan enumerates whatever heads then
  stand; released history is citable, never load-bearing.
- **SC-C01, resolved rather than assumed (pass-6).** SC-C01 reads:
  "Mutating a record bearing `user_overridden: true` without a resolving
  `consent_ref` is rejected at the store layer, **regardless of writer
  identity**" — which a direct sovereign forced↔severed transition would
  violate the moment it altered held state or the materialized marker
  without a petition consent. Chosen resolution, both halves: **(a)
  effective bond state derives entirely from append-only lineage acts** —
  the protected base record is never mutated by anyone; the materialized
  aggregate marker, where kept for indexing, is maintained in the same
  transaction as the lineage act and is lineage-derived bookkeeping, not a
  substantive mutation of a held record; **(b) SC-C01 is amended** to
  recognize an authenticated, exact-hash sovereign `TransitionPlan` as
  lawful authority alongside a resolving `consent_ref` — IV.5's "personally
  releases it" is satisfied only by a closure the sovereign *saw*: every
  command that automatically closes another head **enumerates and hashes
  every closure** (this is why R12 carries H in §11.1). Alternative
  declined: leaving SC-C01 unamended and pretending the derived-state
  reading suffices — the criterion's text names record mutation, and the
  marker is a record.
- **`lay_weight_override` joins the P2B labor set** (`WEIGHT_CORRECTED` had
  schema and no hand, same as the link kinds) — required by P4's
  weight-legibility metric, which asks the operator to *change* influence and
  observe the result; without the surface the metric would be unmeasurable.
- **`WEIGHT_CORRECTED` across spaces (pass-5 — the choice, made explicit):
  geometry-neutral, projected into every space.** The correction is the
  hand's statement of influence — a Law-IV fixed star, like a forced bond:
  while the head stands, the corrected value is the effective weight in
  *every* space, and space-scoped `WeightEvidence` records around it remain
  evidence, not authority. The space-scoped alternative is recorded and
  declined: a correction that lapsed in a new geometry would be a hand
  reversed by migration — IV.1 through the back door. On space activation,
  the standing correction surfaces an **advisory drift signal** (tried under
  old geometry) through the same graduated-legibility channel as Cardinals —
  review is advisory; release is only ever the sovereign's. **Commons-return
  as-stands for weight (pass-5; representation corrected pass-6):** the
  released value is **geometry-neutral state carried on the release act and
  the lineage control record — not a `WeightEvidence` row** (pass 5's
  "terminal evidence record" was incompatible with its own schema:
  `WeightEvidence` is space-keyed, the released value is not; a per-space
  family of minted rows is declined — spaces adopted later would lack their
  row). Projected into every space, it remains the effective weight until
  lawful replacement. **The stale-recalculation race, closed (pass-6):** a
  calculation may begin while the correction stands and land after release —
  an active-head check at write time would let pre-release work instantly
  bury the as-stands value. Therefore **every machine `WeightEvidence` write
  carries the weight-lineage epoch captured when its calculation began**;
  release advances the epoch; pre-release work refuses at write. **Effective
  selection, literal:** (1) an active `WEIGHT_CORRECTED` head; else (2) the
  `RELEASED_AS_STANDS` value for the current lineage epoch; else (3) machine
  evidence computed **against that same epoch** after a lawful trigger.
  Immediate fallback to older machine evidence is forbidden — the machine
  restoration IV.5's as-stands rule exists to prevent. **Sever interplay:**
  when a sever transition auto-closes a weight head (compatibility table),
  the closure records the last held value historically but mints **no
  effective as-stands state** — a severed bond is excluded from every
  consumer, so there is nothing for the value to be effective *in*; if the
  bond is later re-forced before any recalculation, the old corrected value
  does **not** revive (no machine restoration, no silent re-hold) — weight
  is machine-managed under the then-current epoch unless the hand lays a
  fresh `WEIGHT_CORRECTED`.
- **Forced bonds count toward emergence density while their `LINK_FORCED`
  head stands unreleased** (in every space, per the split above; after
  release, only where qualified). The trial may still exclude a forced bond
  from matrix *membership* — membership amendment edits `link_refs`, never
  the bond — bonds outlive structures (VI.5), in both directions.
- **Binding:** identical to the category pattern — sovereign hand lays with
  `consent_ref NULL`; granted petitions lay successors with `basis
  GRANTED_PETITION` + `consent_ref`; re-laying chains `prior_ref`; the
  substrate's agent-author and actor-class walls apply unchanged.
- **Release (pass-3 correction — now decision D8, not a silent assumption).**
  IV.5's "human-held until the sovereign personally releases it" has no
  release semantics anywhere: A.7 pins `user_overridden: true` permanently,
  migration 0003 enforces it with `CHECK (user_overridden)`, the newest
  record is always the active protection, and no release status, record, or
  taxonomy event exists. The intended semantics stand — commons-return
  **as-stands**, no machine restoration, re-evaluation only on lawful
  triggers — but they are **not representable without a canonical/schema
  amendment**. D8 chooses the representation; the recommendation is a
  separate **`OverrideReleaseRecord`** (keeps A.7's invariant that every
  OverrideRecord is a laid hand; the release is its own attributed act,
  chained to the override it closes, **atomically closing exactly that
  kind's head — the subject's aggregate held marker clears only when the
  last head closes**, pass-5 wording fix: the earlier "clearing the
  subject's held state" contradicted per-kind release) plus an
  `OVERRIDE_RELEASED` event added by taxonomy version bump.
  The alternative (an ACTIVE|RELEASED status column on A.7) is viable and
  cheaper; both are put to D8. Adopting D1 does **not** authorize this
  amendment.
- **Retry/concurrency (pass-5 correction; pass-6 completion):** CAS on the
  **monotonic `lineage_epoch`s of the complete expected lineage set**,
  locked in deterministic kind order under one `transition_id`, inside one
  transaction — the lineage set, not the bond row, is the concurrency unit
  (a weight correction never mutates the bond, so bond revision guards
  nothing; where an act *does* touch the bond, the bond CAS rides along,
  subordinate). A single-head CAS cannot implement a composite act, and a
  null-head check cannot survive the empty-head ABA — the epoch advances
  while empty precisely so an old "no head stood" observation is
  unreplayable. Double-submit protection is **store-owned**: idempotency
  (key bound to the request digest) and expected-state live in the
  sovereign store method's own transaction (§11) — an application-layer key
  alone dies with the process.

Proposed criteria (register `AC-` ids, pinned at P2B): sever-preserves-history
and excludes-from-all-consumers-in-all-spaces; forced-bonds-count-toward-
density-in-every-space-while-unreleased; **machine bonds count only where
qualified under the bound space and applicable link-policy revision**
(pass-5 wording fix — "where measured" contradicted the split's own
qualification rule); field-specific protection (existence held ∧ weight
machine-managed ∧ weight held only under `WEIGHT_CORRECTED`); kind
compatibility enforced (refusals and atomic supersessions per the table
above); petition-target binding (open records the head; grant re-verifies;
execution refuses a moved lineage); both-lay-chained-records; post-act agent
mutation refused per kind (mirror of SC-C01/C05); release semantics per D8's
answer, including `RELEASED_AS_STANDS` effectiveness until lawful
recalculation; **epoch-fenced machine writes** (evidence carrying a stale
lineage epoch refuses — the pre-release calculation race); **composite
plans enumerate and hash every closure** (a Notary or direct act closing an
unreviewed head is unrepresentable); **occurrence immutability** (recurrence
never rewrites a resolved ask's target, consent, or execution witness);
retry convergence on lineage-set CAS. `links_for_node` and
`list_matrices` remain small read surfaces.

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
slice, so P1-B and P2A touch disjoint files thereafter**; **the
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
a failure, and never vacated by anyone but the hand. The displaced space becomes RETIRED on activation. **Postulants at
activation (pass-4 stranding fix; pass-5 total disposition — "untried" is not
a represented status, and a declined proposal leaves POSTULANT standing
indefinitely):** the one-live-matrix-per-category rule means an old-space
Postulant left standing blocks new-space emergence in its category forever.
Activation therefore disposes of **every** old-space POSTULANT by exactly one
of three rules, keyed to trial state. **Auto-supersede** — pristine (no audit
opened); **trial incomplete with no live labor (pass-6 addition — the pass-5
table had no bucket for it): audit opened, one report filed, the other
auditor refused or terminated, no barrier, no proposal, no running job** —
an unfinished trial nothing consented to; reports or barrier complete with
no proposal filed; proposal DECLINED or trial halted: derivative,
incomplete, or terminally-declined state (doc 03 §4.3), moved to
`SUPERSEDED` by the activation act, its partial or complete trial evidence
preserved and citable. **Block activation** — consent GRANTED but unexecuted
(nothing the sovereign grants may quietly fail to happen, SC-C06: the tick
executes first, **and the result routes by the actual verdict — pass-6
correction of "yields a Cardinal": COMMIT → CARDINAL, thereafter handled as
a Cardinal (frozen under new geometry); AMEND → POSTULANT revision N+1,
which joins the explicit-disposition list below; REJECT → DISSOLVED,
nothing left to dispose**); execution running, or audit/reconciliation jobs
live (Law VII — no job strands live: they complete or refuse first, or the
sovereign halts them, which routes the matrix to the declined/halted
bucket). **Explicit sovereign disposition, carried by the activation
command itself (pass-6 — "the sovereign disposes it" is now a represented
act, not an unpersisted ceremony): R19 accepts an exact-hashed disposition
map covering every Postulant it lists** — proposal filed and unresolved (a
machine supersession would answer a question that is the sovereign's to
answer); AMEND applied and awaiting re-audit (a consent shaped that
structure; the machine does not dispose of what a consent touched). The
activation act refuses to switch until its map covers each listed matrix;
a separately registered trial-halt operation is declined for now (the
existing refusal/kill-switch machinery halts live labor, and the map owns
activation-time disposition) — recorded, revisitable at D7. `SUPERSEDED` is
a new matrix status entered by **explicit canonical amendment, targets
named:** Dogma VI / A.9's closed status grammar and doc 03 §2.4's
`POSTULANT → CARDINAL | DISSOLVED` line gain the status (**terminal — no
transition out**); migration 0005's `one_live_matrix_per_category` predicate
excludes it; a `MATRIX_SUPERSEDED` taxonomy event records each one. A
superseded Postulant is preserved, unreviving, and its category is free to
re-emerge under the new geometry. (Alternatives
considered and declined: blocking activation on trial-or-dissolution of every
old Postulant forces trials on stale geometry; space-scoping the uniqueness
rule puts two live candidates per category in front of the operator.)
**No old-space POSTULANT survives activation in the live set by accident —
each leaves by a named rule or blocks the switch visibly.**
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

**Immutable trial evidence — `TrialEvidenceSet` (pass-5; the identity D4 was
missing):** pass 4's "the weight-evidence revision set it was tried on" is
not yet an identity anything can retain. Floor weight is degree-normalized
(`w = sim/√(deg(a)·deg(b))`, aggregate.rs) — one policy revision yields
different values as the graph grows; assisted weighting consults a rostered
reasoner and varies at a fixed policy revision; audit reports and Joint
Proposals bind `matrix_ref + matrix_revision` and nothing else; and
`config_constants` is one row per key, updated in place — **and prior values
are not recoverable at all: `set_config` writes no A.5 event (pass-6
correction of pass 5's own "recoverable by log replay" claim — there is no
CONFIG_CHANGED event to replay; an append-only `ConfigHistory` or
`CONFIG_CHANGED` taxonomy entry is a named amendment, §21 matrix)**.
The trial therefore freezes an **immutable evidence manifest at audit-open**
— cited by `AUDIT_OPENED`, never assembled retrospectively at commitment —
binding **immutable values and algorithm identities, never mutable revision
pointers alone (pass-6 expansion)**: `space_ref`; the `GraphEpoch` at
freeze; the coherence threshold and link-similarity threshold **verbatim**;
the link-qualification algorithm and version; the weight formula and
version; the weight mode; the reasoner provider/model digest and applicable
prompt/policy version where assisted; a **distinct calculation-run
identity** with input/output digests; the **effective weight source for
every tried bond** (machine evidence ref with its lineage epoch, or the
`WEIGHT_CORRECTED` head it stood under); the classification source and
applicable category-override heads; exact node/bond membership at the tried
revision; and the active override-head epoch set for member bonds. **Both
auditors' reports, the Joint Proposal, the consent, and the committed
Cardinal cite the same evidence-set identity** — "tried on" becomes a
reference, not a recollection. An AMEND verdict's next matrix revision
freezes a **new** evidence set at re-audit-open, chained to its predecessor.

**Validity is a state machine, not a property of having been recorded
(pass-6):** identity says what the trial saw; it does not say the world
still looks like that. Chosen rule — **current-state revalidation** (the
snapshot-isolated alternative is declined: immutable history must not let
commitment land from state the sovereign has since changed): the evidence
set carries `VALID → SUPERSEDED | DISTRUSTED`, and its `GraphEpoch` and
expected head set are **CAS-revalidated at every VI.3 handoff** — barrier
certification, proposal filing, consent, and Notary execution. A member
bond severed or forced, an override laid or released, a relevant policy
change, a space or graph-epoch advance, or any bound head moving marks the
set SUPERSEDED (DISTRUSTED where integrity, not drift, is the cause) and
**refuses advancement — the lawful continuation is re-audit under a fresh
evidence set**, exactly IV.5's "the world moved on" doctrine applied to
trials. **Trial opening is one atomic, retry-stable store operation
(pass-6):** it CAS-validates `(matrix revision, POSTULANT, space_ref)`,
creates **or returns** the single evidence set for that matrix revision,
consumes the audit-eligibility flag, records `AUDIT_OPENED`, and returns
the standing trial identity — a crash after `AUDIT_OPENED` and before
report filing must converge on the same trial, never mint a second
evidence set. DDL is ADR-2's; the observable semantics are fixed here so
D4 is answerable.

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
