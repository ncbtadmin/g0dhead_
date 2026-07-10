# Override Lineages and Release Semantics

**Status:** Analysis-only; non-canonical and proposal-only until D8 returns.

- **Scope:** Bond/measurement separation, override compatibility, lineage concurrency, petitions, transition plans, release, effective-state selection, and P2B criteria.
- **Owning decisions:** D8; D4 only where measurement records and embedding spaces intersect.
- **Phase owner:** P2B.
- **Criteria hooks:** SC-C01â€“SC-C06, SC-D10, and the proposed AC- criteria listed below.
- **Amendment rows sourced:** D8 rows in [AMENDMENT_MATRIX.md](roadmap_reconciliation/AMENDMENT_MATRIX.md), plus D4 measurement-split rows explicitly cross-linked there.

This is the sole normative proposal home for override and release mechanics.
The controlling roadmap and decision sheet link here and do not restate them.

---
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
