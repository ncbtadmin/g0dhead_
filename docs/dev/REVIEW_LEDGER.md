# Review Ledger â€” Roadmap Reconciliation

**Status:** Analysis-only historical record; non-canonical and proposal-only.

- **Scope:** Pass dispositions, repository witnesses, visible retractions, and bounded verdicts.
- **Owning decisions:** D1â€“D8, historically; this file does not decide them.
- **Phase owner:** None. This is review history, not implementation scope.
- **Criteria hooks:** The exact criteria cited by each preserved ledger entry.
- **Amendment rows sourced:** Historical copies only. The current proposal matrix lives in [AMENDMENT_MATRIX.md](AMENDMENT_MATRIX.md).

The text from the former Â§Â§17â€“21 follows verbatim. Historical mechanisms and
HEAD claims remain evidence of what each pass said, not current normative
definitions.

---
## 17. Pass-2 reconciliation ledger (2026-07-09)

Legend: CONFIRM = concern valid, adopted as posed. AMEND = valid, adopted with
a stated modification. REJECT = does not survive the evidence. Every
disposition names its witness. Preserved without relitigation, per
instruction: the canon-authority correction and the five accepted additions.

  1. **Link-override labor — CONFIRM.** Semantics and AC set defined at §3.1;
     witnesses: A.7 (godhead-schemas/sovereignty.rs), migration 0003's
     override machinery, draw_link/set_link_weight guards (postgres.rs), A.5's
     existing LINK_SEVERED event, SLICE_03 §2's deferral note. One addition
     beyond the ask: *lifting* was never defined for any override kind —
     IV.5's "until the sovereign personally releases it" had no release
     semantics; §3.1 supplies them (commons-return as-stands, no machine
     restoration).
  2. **Bounded concurrency — CONFIRM,** with the delivery-discipline
     clarification written into §8/P2 and one addition: the recorded
     parallel-session hazard (project memory, 2026-07-08) makes the
     disjoint-surfaces + serialized-landings rule explicit, not implied.
  3. **Temporary Candle adapter — the challenge is CONFIRMED and this
     document's own prior recommendation is REJECTED.** The temporary adapter
     existed to avoid deciding D3 before P1 — but D3 sits on the same sheet
     the sovereign answers before any phase runs; there was no sequencing
     gain, only duplicate implementation, dependency weight
     (candle/tokenizers vs one HTTP client), and measurement-validity risk
     (quantization/serving deltas this document itself flagged). Revised:
     D3 precedes P1; P1 builds and owns the production adapter (embedder
     half) with the reachability walls in the same slice; in-process is a
     named contingency only. Ownership gap closed: P1 delivers the adapter
     its own exit condition consumes; P4-entry delivers the reasoner half.
  4. **P0 exit — CONFIRM** (shape b, plus a crate-level smoke run): P0 proves
     the pipeline can process the synthetic corpus on the floor and that P1's
     scope/criteria are pinned; the measurement harness is P1's to build.
  5. **P2 exit at Stop-3 — CONFIRM.** Every enumerated ceremony surface has
     an existing store witness (invoke_audit, audit_reports_for,
     audit_barrier_certified, get/resolve_proposal, notary labors, get_matrix,
     the log chain) — this is façade work, correctly P2B, plus one promotion:
     the executable-consent tick becomes a served behavior. P3 becomes pure
     translation.
  6. **SC-F06 double assignment — CONFIRM.** Resolved via D3: detached from
     the fetch bundle, bound to model-egress, completed at P5 (constrained
     generation concerns tool-argument production, which first meets a real
     endpoint in agent labor, not in embed/weigh calls). §4, §8/P5, §8/P6,
     §9 updated; if D3 is declined it reverts whole to P6. One assignment
     under either answer.
  7. **Assisted weighting before the verdict — CONFIRM, adopted by
     mechanism.** Doc 04 §5.3 names assisted mode the intended default of the
     GodHead proper, and assisted *weighting* is claim-1 machinery (the prior
     draft mis-bundled it with claim-2's reasoner uses). The assisted path is
     already built and mock-proven (SC-M03; aggregate.rs) — only the real
     mind is absent. Adopted: reasoner-adapter half is a P4 *entry
     condition*; P4 runs paired floor/assisted on the same corpora; a
     thesis-rejection verdict is unreachable on floor-only evidence (stated
     in §8/P4, enforced by the gate's own text). Teacher/Student exposure
     stays P5.
  8. **Embedding-space persistence — CONFIRM.** Storage options compared;
     recommendation (c) active-typed + archival; lifecycle
     PREPARING→ACTIVE→RETIRED with activation preconditions; same-space
     reader rule; `space_ref` propagation enumerated (embeddings, links,
     weight logs, consolidation provenance, matrices, explanations) —
     `config_rev` confirmed insufficient as a geometry witness (it cites the
     threshold revision only: migration 0005, matrix_from_row). Floor named
     space 0. DDL to ADR-2 after D4.
  9. **Client authority — CONFIRM;** v1 posture written into §11, D7 added to
     the sheet by sovereign instruction; the confused-deputy risk is answered
     by enumerated sovereign-class entry mapped one-to-one to IV.4's closed
     list — the constitutional list becomes the API's literal allowlist,
     arch-pinned.
  10. **Evaluation rigor — CONFIRM, all items:** same-geometry kNN and
      clustering baselines (the fair fight), calibration/holdout/operator
      separation, overlap-aware agreement where matrices overlap,
      preregistration as committed register revisions with observation-status
      disclosure, storage-growth measurement P1–P4.
  11. **Document structure — CONFIRM:** combined while unpinned; §16 names
      the post-decision ADR extractions.
  12. **Precision edits — CONFIRM both:** "largest unretired risk" (§1, §6 —
      the stronger claim fails against the open server/auth/provider/
      threshold/migration/recovery risks, now enumerated beside it);
      "one slice from full coverage" qualified to floor-and-mock coverage
      (§1).

**Corrected dependency sequence:** decisions D1–D7 → P0 → P1 (requires D3+D4;
delivers adapter + walls + offline verdict; register pinned at entry) →
P2A (may overlap P1; disjoint, separately pinned) → P2B (post-P1 findings;
exit = Stop-3 via API) → P3 (exit = Stop-3 via client) → P4-entry (reasoner
half) → P4 (paired verdict) → P5 (governed labor; SC-F06 completes) → P6
(fetch; wall deletes atomically) → P7 (separately justified).

**New contradictions introduced by this pass — checked:** (i) P1's old
"runs under existing walls" rationale deleted everywhere (D3 now precedes
P1); (ii) P5's "adapter lands here" corrected to "reasoner consumed here,
integrated at P4-entry"; (iii) SC-F06 appears in exactly one phase per D3
branch; (iv) P0's exit no longer references an artifact P0 does not build;
(v) the decision sheet's growth from six to seven is by explicit sovereign
instruction. ~~No further contradictions found.~~ **RETRACTED — the sovereign's
independent review found five further contradictions this pass introduced;
see §18. The retraction stands here visibly, per house rule: a retraction
that hides is worse than the error.**

**Pass-2 readiness verdict — SUPERSEDED by §18.** (Pass 2 claimed
decision-readiness; the claim did not survive repository inspection.)

## 18. Pass-3 reconciliation ledger (2026-07-09)

Sovereign review of pass 2 found five decision-blocking contradictions and
four ownership gaps introduced or left by the pass-2 mechanisms. All were
verified against the repository before this pass; **every finding held — zero
refuted**. Dispositions:

  1. **D3's defer branch made P1 impossible — CONFIRM.** Pass 2 left D3 as an
     independent [adopt/defer/amend] while P1 required the adapter the wall
     forbids. Fixed: coupling stated in D1 and D3; the sheet now offers only
     coherent combinations. Witness: pass-2 §8/P1 entry vs §15/D3.
  2. **Three D4 contradictions — CONFIRM all.** (a) Forced links vs
     space-bound links: resolved by the bond/measurement split (§3.1, §10) —
     geometry-neutral bonds, space-keyed evidence, forced bonds counting in
     every space. (b) No PREPARING write target in the two-store storage:
     resolved by three generations (active/preparing typed + retired archive)
     with atomic catalog switch. (c) **Cardinal amendments were unlawful as
     written** — audit tries Postulants only (`file_audit_report` refuses
     non-Postulants; the 0006 trigger freezes professed Cardinals;
     decommission is the only door). Resolved by choosing the frozen branch
     explicitly: drift is advisory through graduated legibility; replacement
     is decommission + fresh emergence; in-place revision would be a Dogma VI
     amendment nobody is proposing. Witnesses: postgres.rs trial methods,
     migration 0006, Dogma VI.3/VI.5.
  3. **D7 was not the 1:1 IV.4 map it claimed, and enumeration is not
     authentication — CONFIRM both.** The reserved-operation set now
     enumerates IV.4's eight entries exactly (the seam and petition grants
     were missing) plus the labeled non-IV.4 sovereign hands ("override" is
     IV.1/IV.5's vocabulary, not IV.4's). The deeper defect — store methods
     accept an actor string and self-elevate, so reachability equals
     sovereignty — is answered by the unforgeable sovereign context,
     post-authorization elevation, call-site wall, and end-to-end tests
     (§11). Witnesses: doc 05 IV.4's prose list; the sovereign-act method
     signatures (interface.rs); the G10 elevation sites (postgres.rs).
  4. **Override release was unrepresentable — CONFIRM.** A.7 pins
     `user_overridden: true`; migration 0003 enforces `CHECK
     (user_overridden)`; the newest record is always active; no release
     status, record, or event exists. The pass-2 lift semantics were sound
     and unimplementable. Now decision **D8(a)** — a canon/schema amendment
     the sovereign makes explicitly, never inherited from D1. Witnesses:
     sovereignty.rs A.7, migration 0003, `get_active_override`.
  5. **Forced links conflated LINK_FORCED with WEIGHT_CORRECTED — CONFIRM.**
     Canon separates the kinds; the row-level bit cannot. Resolved:
     kind-aware protection (existence/category held; weight machine-managed
     unless separately corrected), `lay_weight_override` added to P2B (P4's
     weight-change metric was otherwise unmeasurable). Decision **D8(b)**.
     Witnesses: A.7's kind enum; the links table's single `user_overridden`
     column (migration 0004).
  6. **Ownership gaps — CONFIRM all four.** Embedding-space infrastructure →
     owned by new **P1-A** (a pinned build slice; measurement is P1-B).
     Reasoner integration → owned by new **P4-A** (a pinned tranche, not an
     unowned "entry condition"). SC-F06 → requires new adapter capability
     (constrained `propose_call` implementing the `ToolCaller` seam), named
     at §9 and P5 — embed/weigh cannot exercise it. Stop-3 → reclassified as
     store/domain/orchestration work with criteria (consent-freshness
     store extension; the proposals-execution tick that petition grants have
     and matrix proposals lack). Witnesses: `ToolCaller::propose_call`
     (godhead-toolcall/lib.rs), `resolve_proposal`'s signature,
     `grants_tick` vs the absent proposal tick.
  7. **Sequencing corrections — CONFIRM all four:** P4 `adjust` loops to
     re-measurement (never advances to P5); P0-fork/join expressed; the
     operator corpus participates in P1-B for structure/cost only, judgments
     reserved to P4; the disjoint-surfaces claim qualified (manifests and
     walls land once, in P1-A).

**Corrected dependency sequence (supersedes §17's):** decisions D1–D8 →
P0 → fork{ P1-A → P1-B ∥ P2A } → join → P2B → P3 → P4-A → P4-B (loop on
`adjust`) → P5 → P6 → P7.

**Readiness verdict — SUPERSEDED (pass 4): READY ONLY AFTER A FOCUSED
PASS 4.** (Pass 3's original verdict claimed readiness after its own
amendment; the sovereign's independent mechanism-versus-mechanism review
found new contradictions among the pass-3 mechanisms themselves — the third
consecutive demonstration that a pass cannot certify its own composition.
The focused pass is applied at §19, which carries the standing verdict.)

## 19. Pass-4 reconciliation ledger (2026-07-09/10)

The sovereign's independent mechanism-versus-mechanism review of pass 3
named six areas and instructed that this pass address **only** those six —
no sequence-versus-canon relitigation, no broader rescoping. That
instruction was followed: every edit above traces to one of the six. All six
were verified against the repository before any edit; **every finding held —
zero refuted**, the third consecutive pass in which the sovereign's review
survived inspection intact. Dispositions:

  1. **Weight on the bond row broke SC-C01 under reweighting — CONFIRM;
     weight moves off the record entirely.** Pass 3's split left similarity
     space-keyed but weight riding the bond row, so machine reweighting under
     any space mutates rows — including protected ones — or else demands an
     SC-C01 exception. Resolved (§3.1, §10): qualification and weight become
     space-keyed evidence records — `BondQualification{bond_ref, space_ref,
     link_policy_rev, similarity, qualified}` and `WeightEvidence{bond_ref,
     space_ref, weight_policy_rev, mode, weight}` — the bond row is never
     mutated by measurement, so a protected bond is immutable **by
     construction**; a committed Cardinal retains its `space_ref` and the
     weight-evidence revision set it was tried on. Witnesses: A.7's kind
     enum; migrations 0003/0004's override machinery; SC-C01's
     protected-bond assertion.
  2. **D8's single-head release semantics — CONFIRM all three defects.**
     Simultaneous `LINK_FORCED` + `WEIGHT_CORRECTED` on one subject was
     unrepresentable under newest-record-wins (`get_active_override` returns
     one record); a release racing a Notary re-freeze could resurrect the
     closed head; released-bond density behavior was undefined. Resolved
     (§3.1, D8): **per-kind lineages** — one active head per `(subject_ref,
     override_kind)`; release closes exactly one head by compare-and-set;
     guards derive from all unreleased heads; a forced bond counts toward
     density only while its `LINK_FORCED` head stands unreleased, and
     returns to the commons as-stands on release. Witnesses:
     `get_active_override`; migration 0003's `CHECK (user_overridden)`.
  3. **Space lifecycle had no failure path and stranded Postulants; three
     canon texts contradicted multi-space storage — CONFIRM all.** Resolved
     (§10): `PREPARING → ACTIVE | ABANDONED` (a failed generation is
     attributed, logged, disposable — never a permanent occupation of the
     preparing slot); untried old-space Postulants move to **`SUPERSEDED`**
     at activation, by named amendment (Dogma VI/A.9's status grammar,
     doc 03 §2.4, migration 0005's `one_live_matrix_per_category` predicate,
     taxonomy event) — the declined alternatives are recorded; and the
     pass-3 "recorded reading" of SC-M05 is retired in favor of **three
     named literal amendments** (doc 03 §2.2, doc 04 §4.1, SC-M05 — "one
     persisted vector per node" rescoped to per node per valid space),
     because a reading that contradicts a register's literal test is shadow
     canon. Witnesses: migration 0005; doc 03 §2.2/§2.4; doc 04 §4.1;
     SC-M05's embedder call-count assertion.
  4. **Fork placement and verdict routing — CONFIRM.** Pass 3 forked at P0
     while P2A consumed the workspace manifests and walls P1-A builds; P1's
     `adjust`/`kill` and P4's `simplify ceremony` verdicts had no routes.
     Resolved (§8, D1): **P0 → P1-A → fork{ P1-B ∥ P2A } → join → P2B**;
     P1 `adjust` re-runs P1-B under a new preregistration revision; P1
     `kill` blocks P2B, returns to sovereign reconciliation, P2A may finish;
     P4 `adjust` → corrective slice → P4-B; `simplify ceremony` → corrective
     client slice, or Dogma amendment through process where the ceremony is
     constitutional — either way P4-B re-runs before P5. Witness: pass-3
     §18's own sequence line against §8/P2A's stated inputs.
  5. **Stop-3 was mechanically unreachable through served paths — CONFIRM.**
     The intake dispatcher deliberately ends at the classification seam;
     `certify_audit_barrier` is a store method nothing served invokes;
     `reconcile` is test-invoked; matrix proposals lack the tick petition
     grants have. Resolved (§8/P2B): four owned orchestration behaviors,
     each with retry/idempotency/refusal/restart criteria — the
     **processing-seam dispatcher** (IV.4's seam-crossing entry,
     D7-registered), the **audit-barrier certification tick**, the
     **reconciliation dispatcher tick**, and the **proposal-execution tick**.
     Witnesses: godhead-audit/src/lib.rs (`invoke_audit`, `reconcile`);
     interface.rs `certify_audit_barrier`; `grants_tick` vs the absent
     proposal tick.
  6. **D7's wall watched the wrong perimeter — CONFIRM.** `invoke_audit`
     (godhead-audit/src/lib.rs) and `rebalance_now`
     (godhead-ml/src/rebalance.rs) are **public library functions**; a
     call-site wall scoped to sovereign *store* surfaces never sees them,
     and the P2B seam dispatcher would be a third invisible entry. Resolved
     (§11, D7): the perimeter is drawn around reserved **operations** —
     capability threaded into every reserved signature, or one gating
     reserved-operation module (shape chosen at ADR-3); the arch test
     enumerates every entry of a **complete-but-phased handler registry**
     (every operation named now, each carrying its supplying phase;
     unshipped = registered-and-refusing); and `resolve_bias_warning`
     (interface.rs) joins the non-IV.4 sovereign hands the pass-3 list
     missed. Witnesses: the two `pub async fn` definitions; interface.rs:615.

**Corrected dependency sequence (supersedes §18's):** decisions D1–D8 →
P0 → P1-A → fork{ P1-B ∥ P2A } → join → P2B → P3 → P4-A → P4-B (loop on
`adjust`/`simplify`) → P5 → P6 → P7.

**Decision posture after this pass:** the sovereign's stated pass-4 posture
(D1 amend, D2 adopt, D3 adopt-with-coupling, D4 amend, D5 adopt, D6 adopt,
D7 amend, D8 amend) is applied above — every "amend" is written into its
section and its sheet entry, so the sheet is now answerable as it stands.

**Standing readiness verdict: THE SIX NAMED COMPOSITIONS CLOSE AGAINST THE
REPOSITORY; ~~THE SHEET IS ANSWERABLE~~. THIS PASS DOES NOT CERTIFY ITSELF
CONTRADICTION-FREE.** **The answerability claim is RETRACTED (pass 5,
visibly, per house rule):** the sovereign's independent review found D4, D7,
and D8 still carrying unresolved state-identity, atomicity, provenance, and
authority semantics — and one of this pass's own §3.1 mechanisms (the
release-defeats-grant claim) was refuted against the petition schemas. The
answerability assertion was exactly the kind of composition claim this
verdict's own caveat disclaimed the authority to make. Pass-5 dispositions
and the standing verdict: §20. The remainder stands as history: passes 2 and
3 each certified themselves contradiction-free and each was wrong; the
pattern, not the intention, is the evidence; the six pass-4 findings remain
resolved with their witnesses.

## 20. Pass-5 reconciliation ledger (2026-07-10)

The sovereign ordered an independent mechanism-composition review scoped to
**D4, D7, D8, their phase handoffs, acceptance criteria, and canonical
amendment costs** — with the explicit instruction to verify every claim
against the current repository before editing and to preserve disagreement
where evidence refutes a finding. Every claim was verified against HEAD
(791ddbd — Slice 11 mid-build; 18 migrations) before any edit. **All eleven
findings CONFIRM; none refute.** One casualty on this document's side: a
pass-4 §3.1 mechanism was **refuted as written** and is corrected, marked in
place. Scope was obeyed — no phase redesign, no reopened roadmap questions.
Dispositions:

  1. **Petition-head binding — CONFIRM; the pass-4 release sentence
     REFUTED as written.** `PetitionRecord` carries `subject_ref` +
     `change_kind` and **no head reference**; `open_petition` checks only
     that *some* override exists (kind-blind:
     `get_active_override(draft.subject_ref)`); `execute_grant` loads
     **whichever override is active at execution** as the successor's
     `prior_ref` — so pass 4's "the grant's chain no longer resolves against
     a released head" described a mechanism the schemas do not implement;
     released-then-relaid, an old grant would attach to the new head.
     Semantics chosen (§3.1, D8): `target_override_ref` recorded at open
     against the petitioned kind's head; grant binds consent to the target
     and re-verifies; executor refuses a moved lineage
     (`TARGET_RELEASED`/`TARGET_SUPERSEDED`); lay, succession, and release
     CAS **the lineage head, never the bond row** (a weight act need not
     touch the bond); OPEN petitions persist but cannot be granted against a
     vanished target; racing petitions serialize through the
     one-lineage-per-(subject, kind) rule + grant-time CAS. Witnesses:
     sovereignty.rs 91–105; postgres.rs `open_petition` (the IV.2 existence
     check), `execute_grant` (the `get_active_override` load; also
     v1-refuses non-category kinds — the fix must precede P2B's new
     executable kinds). Only `CATEGORY_REASSIGNED` executes today.
  2. **Kind-aware combinations — CONFIRM; table defined (§3.1).** Four
     kinds (A.7). By domain: category→nodes; the three link kinds→bonds.
     FORCED ∥ WEIGHT_CORRECTED compatible; WEIGHT_CORRECTED on severed
     **refused**; FORCED ↔ SEVERED mutually exclusive with **atomic
     supersession** (one act, chained closure records; the two-ceremony
     alternative recorded and declined — doc 04 §4.4's single-act
     correction doctrine); guards read per-kind heads only; the link row's
     `user_overridden` becomes a derived aggregate marker, never a mutation
     guard; the D8 bullet's "clearing the subject's held state" corrected to
     per-kind closure. Witnesses: OverrideKind enum; links DDL (0004,
     whole-row boolean); the `NOT user_overridden` guards (postgres.rs
     draw_link/set_link_weight).
  3. **`WEIGHT_CORRECTED` across spaces and after release — CONFIRM; choice
     made: geometry-neutral (§3.1).** The correction is a Law-IV fixed star
     projected into every space (space-scoped correction declined:
     IV.1-by-migration); drift under new geometry is advisory. As-stands
     release: the release transaction mints a `RELEASED_AS_STANDS` evidence
     record — the last human-held value stays effective until the next
     lawful recalculation; immediate fallback to older machine evidence is
     named machine restoration and forbidden. Witness: doc 04 §5; IV.1/IV.5;
     §10's own fixed-star language.
  4. **Immutable trial evidence — CONFIRM; `TrialEvidenceSet` introduced
     (§10, D4).** Floor weight is degree-normalized (`sim/√(deg·deg)`,
     aggregate.rs:57–64) — same policy, different values as the graph grows;
     assisted mode consults a rostered reasoner (aggregate.rs:88–89) —
     varies at fixed revision; `AuditReport`/`JointProposal` bind
     `matrix_ref + matrix_revision` only (matrix.rs:110–147);
     `config_constants` is one row per key, updated in place (0001 DDL;
     `set_config`) — `config_rev` resolves to the current value. The
     manifest freezes at audit-open, cited by `AUDIT_OPENED`, binding the
     eight identities listed at §10; reports, proposal, consent, and
     Cardinal cite the same set; AMEND freezes the successor at re-audit.
     DDL to ADR-2; semantics fixed here.
  5. **Total disposition of old-space Postulants — CONFIRM (§10, D4).**
     "Untried" is not a status (`MatrixStatus`: POSTULANT/CARDINAL/
     DISSOLVED); a declined proposal leaves POSTULANT standing. Disposition:
     auto-supersede (pristine; barrier-complete-unproposed;
     declined/halted), **block** (granted-unexecuted — SC-C06; running
     execution; live trial jobs — Law VII), **explicit sovereign
     disposition at the activation act** (unresolved proposals;
     AMEND-awaiting-re-audit — the machine does not dispose of what a
     consent shaped). No old-space POSTULANT survives by accident.
     Witnesses: matrix.rs A.9 enum; 0005 matrices DDL.
  6. **Activation serialization + certification epoch — CONFIRM (§10, D4;
     §8/P1).** Invariant fixed: per-matrix CAS; SUPERSEDED terminal
     (trigger-enforced like 0006); supersession bumps revision; trial
     writes predicate atomically on revision ∧ POSTULANT ∧ originating
     `space_ref` (today's guards check status/revision in-method, no space
     predicate — postgres.rs `file_audit_report`); barrier flags supersede
     with the matrix; certification binds a **data/config high-water mark
     CAS-validated at the catalog switch** (fence vs catch-up to ADR-2).
     P1's verdicts mapped to the lifecycle at the newly named
     **post-measurement sovereign checkpoint**: `proceed` = threshold
     adoption (SOVEREIGN-tier `set_config`, register-cited) + candidate
     activation before P2B; `adjust` = candidate stays PREPARING
     (policy-level) or ABANDONED + fresh candidate (geometry change);
     `kill` = ABANDONED, space 0 stays ACTIVE, P2B blocked.
  7. **D7 enforcement — CONFIRM; the branches are not equivalent (§11).**
     While `invoke_audit`/`rebalance_now` (pub library fns) and the pub
     Store trait remain callable, a gating module + call-site test proves
     no *current* bypass, not an uncallable one. Recommended:
     **capability-bearing signatures on every reserved operation**; the
     gating-module branch lawful only with structural sealing (non-exported
     sovereign entrypoints — the P2A refactor, priced). The arch test
     becomes a tripwire behind the type-level wall, not the wall.
  8. **Literal registry — CONFIRM; §11.1 added.** Twenty rows, stable IDs,
     every accepted outcome (grant/decline/silence; admit/reject;
     acknowledge/silence...), authority source, class, phase, envelope
     fields, uniform unshipped-refusal, restart/discovery. Generic
     `record_consent` retired from §11's candidate list (authority is
     operation-specific). **`form_pairing` added (R17):** the method takes
     no actor and the implementation hardcodes `produced_by = 'sovereign'`
     (postgres.rs), while doc 06 §4.3 names re-pairing "a fresh sovereign
     act" — a sovereign operation the perimeter had never listed; supplied
     P5. Space lifecycle ops (R18–R20) enter as sovereign acts with
     store-before-handler phasing noted.
  9. **Envelope ownership + proposal-tick discovery — CONFIRM (§8/P2B,
     D7).** Envelopes are store-owned for every registry row — validated
     and recorded inside the sovereign method's transaction (crash-atomic);
     today only `set_config` carries `expected_revision`;
     `resolve_proposal` is bare. The proposal-execution tick's restart
     source is named: a **pending-consented-proposals query** (mirror of
     `stalled_grants`, which godhead-notary's `grants_tick` consumes) plus
     a CAS claim at execution start; matrix proposals today have
     `get_proposal` point lookup only. Witnesses: interface.rs 266
     (`stalled_grants`), 454 (`get_proposal`), 459 (`resolve_proposal`),
     471 (`execute_matrix_proposal`, Notary-executed).
  10. **Acceptance criteria + amendment ledger — CONFIRM (§3.1, D4).** The
      §3.1 criterion now reads **"machine bonds count only where qualified
      under the bound space and applicable link-policy revision"** ("where
      measured" contradicted the split's own rule). The amendment ledger
      grew by inspection: **doc 03 §2.3** and **doc 04 §4.2** literally
      place `similarity/weight` on the link record (witnessed verbatim) —
      the split moves them to evidence; **doc 04 §4.4**'s
      "overrides set `user_overridden:T`" re-scopes to the aggregate
      marker; **A.9** gains space/evidence identity in addition to
      SUPERSEDED; **A.5** gains six named events by taxonomy version bump
      (`SPACE_ADOPTED`, `SPACE_ACTIVATED`, `SPACE_ABANDONED`,
      `SPACE_RETIRED`, `MATRIX_SUPERSEDED`, `OVERRIDE_RELEASED` — the
      current enum has none of them, log.rs). Every amendment stays a
      proposal until the amendment process ratifies it; none is a
      "reading."
  11. **Precision cleanup — CONFIRM, scoped.** §8/Tranche A now says
      "concurrent with P1-B"; P1-A's own contents name the workspace-
      manifest edits; §17/§18's superseded sequences stand untouched as
      history.

**Sequence: unchanged.** P0 → P1-A → fork{ P1-B ∥ P2A } → join → P2B → P3 →
P4-A → P4-B (loop) → P5 → P6 → P7 — pass 5 altered semantics inside D4, D7,
and D8 and the P1 checkpoint's contents, not the order of phases.

**Canonical amendment ledger, consolidated (all proposed-until-ratified;
adoption of D4b/D8 carries them to the amendment process, and D1 authorizes
none of them):** doc 03 §2.2 + doc 04 §4.1 + SC-M05 (per node per valid
space); doc 03 §2.3 + doc 04 §4.2 (similarity/weight move to evidence
records); doc 04 §4.4 (aggregate marker, per-kind guards); Dogma VI/A.9 +
doc 03 §2.4 (SUPERSEDED terminal; space + evidence-set identity); A.7
(per-kind lineages, petition target binding, release records); A.5 (six new
events); migration-level counterparts follow the schema amendments at
implementation time.

**Standing readiness verdict — bounded, uncertified:** the eleven findings
close against the repository with the witnesses above; D4, D7, and D8 now
state their state-identity, atomicity, provenance, and authority semantics
explicitly, with alternatives recorded where a choice was made. **This pass
does not certify itself contradiction-free** — it is the third consecutive
pass to correct a predecessor that thought otherwise, and it corrected one
of its own author's pass-4 mechanisms against the schemas. What is asserted
is bounded: findings verified, semantics chosen and stated, scope obeyed,
costs named. Whether the sheet is now answerable is the sovereign's call to
make — on the record, this document has lost the standing to make it
first.

**[Pass-6 annotation, 2026-07-10 — appended, not rewritten:** the bounded
facts above stand, but the implicit completeness of "D4, D7, and D8 now
state their … semantics explicitly" did not survive the next independent
review: twenty-two further findings, §21 — among them one more of this
pass's own mechanisms corrected against its own schema
(`RELEASED_AS_STANDS` as a space-keyed evidence row for a geometry-neutral
value) and one of its factual claims corrected against the code (config
values are not "recoverable by log replay"; `set_config` logs nothing).**]**

## 21. Pass-6 reconciliation ledger and amendment matrix (2026-07-10)

Independent mechanism-composition review, scoped to D4/D7/D8, their
P1/P2/P5/P6 ownership, acceptance criteria and failure semantics, the
reserved-operation registry, the canonical amendment ledger, and the
current-HEAD inventory. Every claim verified against the repository and
canon before editing. **Baseline (AMEND):** the review was ordered against
75ad38b (Slice 11 delivery); the tree stands one commit further at
**26c0090** — a docs-only Slice 11b spec pin. The full delta
791ddbd→26c0090 touches godhead-collector, doc 07 (the canon-`sources`
ruling), the sweep, the gate report, and the two slice files — **no Store,
sovereignty, trial, schema, or Dogma surface; every pass-5 witness stands**
(verified by `git diff --name-only`). Of the substantive findings, **all
CONFIRM; none refute** — and two more of pass 5's own statements fell to its
own evidence (marked in §20's annotation). Dispositions (owner and criteria
named inline; declined alternatives italicized):

  1. **Composite consent / `TransitionPlan` — CONFIRM; shape B chosen.**
     One `target_override_ref` cannot authorize the three head-movements a
     lawful sever-of-forced-weight-corrected performs; `change_kind` is
     overloaded (petition class ∧ successor kind — 0003's CHECK + UNIQUE
     witness). Plan binds every expected head+epoch, every closure, result,
     one hash, one id; Notary refuses any mismatch. *Same-kind-only
     declined: re-splits the single-act model.* Owner: P2B/D8. Criteria:
     §3.1 list ("plans enumerate and hash every closure").
  2. **Immutable `PetitionOccurrence` — CONFIRM, worse than claimed.** The
     ESCALATED recurrence branch overwrites `reason`/`proposed_change` and
     **NULLs `consent_ref`/`execution_job_ref`/`resolved_at`** (postgres.rs
     `open_petition`; 0003 `UNIQUE (subject_ref, change_kind)`) — erasing an
     executed grant's witness from the aggregate today. Lineage aggregate
     retained (SC-C02/C03 need it); occurrences immutable; consents and
     successors cite occurrences. **SILENCED binds the exact head epoch
     silenced** — *subject/kind-wide gag declined (over-broad against
     IV.2).* Owner: P2B/D8. Criteria: occurrence-immutability (§3.1).
  3. **Monotonic `OverrideLineage` epochs — CONFIRM.** Singular head-CAS
     cannot serialize composite acts; the empty-head ABA is real (null
     observation replayable across lay+release). Epoch advances active and
     empty; deterministic kind-order locking; one transition id; marker
     updated in-transaction. Owner: P2B/D8 substrate.
  4. **Four-kind release table + weight-race fence + as-stands geometry —
     CONFIRM; pass-5's evidence-row shape corrected.** Table added for
     CATEGORY_REASSIGNED/LINK_SEVERED (retained state, effective read,
     lawful trigger, epoch fence, cross-kind behavior); machine
     `WeightEvidence` writes carry the calculation-start lineage epoch
     (release advances it → pre-release work refuses); effective order
     literal (head → as-stands@epoch → same-epoch machine evidence);
     `RELEASED_AS_STANDS` is **geometry-neutral lineage state** — *per-space
     minted family declined (later spaces would lack rows)*; sever-closure
     mints no effective value and re-force revives nothing. Owner: P2B/D8.
  5. **SC-C01 vs direct transitions — CONFIRM; resolved both halves.**
     SC-C01's "regardless of writer identity" is verbatim (doc 08:29).
     Effective state derives from append-only acts (base record never
     mutated; marker = in-transaction lineage bookkeeping) **and** SC-C01
     amended to admit exact-hash sovereign TransitionPlans; R12 carries H.
     *Leaving SC-C01 unamended declined — the marker is a record.*
  6. **Emergence/activation race — CONFIRM.** `emerge_postulant`
     (interface.rs:390) carries no catalog predicate; a retired-geometry
     Postulant could land post-switch. Emergence predicates on
     `(expected_active_space, expected_catalog_revision)` in the switch's
     serialization unit; born matrices record `space_ref` + `GraphEpoch`.
     Owner: P1-A substrate, D4.
  7. **A.5 as high-water token — CONFIRM; `GraphEpoch` chosen (shape A).**
     A.5 sequence is identity, not commit order; writes and `append_log`
     are separate autocommit ops (witness: `open_petition`); `set_config`
     logs nothing. Transactional GraphEpoch row advanced inside every
     graph-affecting write (participants enumerated §10); switch
     CAS-validates it. *Blanket fence declined as primary (stalls intake);
     remains ADR-2's fallback.* A.5 stays explanatory history.
  8. **TrialEvidenceSet validity — CONFIRM; current-state revalidation
     (shape B) chosen.** Epoch + expected head set CAS-revalidated at
     barrier, proposal, consent, execution; drift → SUPERSEDED/DISTRUSTED →
     re-audit. *Snapshot isolation declined: immutable history must not
     commit state the sovereign has since changed.* Owner: P2B trial
     surfaces, D4.
  9. **Atomic trial opening — CONFIRM.** One store-owned op:
     CAS(revision ∧ POSTULANT ∧ space) + create-or-return the single
     evidence set per matrix revision + consume eligibility +
     `AUDIT_OPENED`, retry-convergent. Crash after open cannot mint a
     second set. Owner: P2B, D4; registry R03 notes it.
  10. **Disposition totality + verdict routing — CONFIRM.** Added the
      incomplete-trial bucket (one report, counterpart refused/terminated,
      no barrier/proposal/jobs → auto-supersede); "yields a Cardinal"
      corrected to COMMIT→CARDINAL / AMEND→rev N+1 (→ explicit list) /
      REJECT→DISSOLVED; the explicit branch is **R19's exact-hashed
      disposition map** — *separate trial-halt operation declined for now
      (kill-switch machinery + the map cover it); recorded, revisitable at
      D7.*
  11. **`CandidateEvaluationContext` — CONFIRM.** P1-A-owned; PREPARING
      space, staged (unadopted) policy snapshot, candidate runs, candidate
      GraphEpoch, non-authoritative outputs, production adapter. The
      production-graph invariant stays intact while P1-B measures.
  12. **`proceed` composition — CONFIRM; atomic promotion (shape A)
      chosen.** Set-then-activate invalidates the certification it just
      relied on and briefly governs space 0 with the winning threshold;
      staged snapshot + candidate space promote as one checkpoint act.
      Policy-level `adjust` replaces the snapshot, re-runs candidate
      qualification/weight, **recertifies** (embeddings reusable while
      geometry holds). *Shape B declined.*
  13. **Checkpoint authority — CONFIRM; wait-for-join chosen.** R09/R18–R20
      under "recorded direct invocation" was process discipline, not the
      D7 boundary; the checkpoint now follows the P1-B ∥ P2A join and
      consumes P2A's capability + envelope substrate. *P1-A shipping its
      own minimum auth stack declined (duplicated authority substrate).*
      Notation updated (below). **ABANDONED authority unified:** machine
      marks generation failure; the transition is the sovereign's R20.
  14. **Immutable policy bundle — CONFIRM, including against pass 5
      itself.** The manifest binds verbatim values and algorithm/digest
      identities (list at §10); "recoverable by log replay" was false — no
      CONFIG_CHANGED event exists; `ConfigHistory`/`CONFIG_CHANGED` is a
      named D4 amendment.
  15. **Registry completeness — CONFIRM.** Added R21 (admission standing
      notice, SC-I07b — no store fn exists), R22 (silenced-scope lift —
      SC-K07/HS §6.3 name the lift; no surface), R23 (persisted
      `SOVEREIGN_JUDGMENT` verdict — the Return is immutable; the verdict
      is a separate attributed record; P5). **R17 split:** re-pairing is
      canon-sovereign (HS §4.3); initial formation is
      implementation-sovereign only (A.10 carries no authority field) —
      the D7 answer confirms or assigns it. Enrollment/rotation/revocation
      → ADR-3's separate authentication-control registry.
  16. **Async executor ownership — CONFIRM, with canon witness.** Doc 05's
      Notary note promises a dispatcher "on any executable consent flag";
      only petition grants have one (`grants_tick`/`stalled_grants(0)`).
      Added: decommission-execution tick, admission-processing tick, each
      with pending query + CAS claim; discovery immediate, stall window =
      SC-C06 monitoring only. Owner: P2B.
  17. **Live vs standing-trigger authority — CONFIRM.** R01 split into
      R01a (human run-now, sov-cap) and R01b (machine tick under a
      recorded standing trigger revision — a scheduler never forges a
      capability); R02 already splits by IV.4's own "outside a
      user-configured trigger" wording, now stated.
  18. **Durable command receipts — CONFIRM.** `invoke_audit`/
      `rebalance_now` are multi-write library operations; single-transaction
      envelopes cannot cover them. Receipt/state-machine semantics added
      (accepted → progress → terminal; retry resumes; duplicates converge);
      registry rows R01–R03, R19 carry `rcpt`.
  19. **Envelope corrections — CONFIRM.** R07 gains H (occurrence/plan);
      R12 gains H (every closure); R19 gains H (disposition map); R15
      gains R (expected warning state); idempotency keys bind to request
      digests (legend).
  20. **Amendment matrix — CONFIRM; completed below.** One sub-decision:
      consent identity **resolves through the immutable occurrence/plan**
      rather than carrying target/hash fields directly — **A.12 is
      deliberately not amended**; *hash-on-consent declined (the envelope
      validates H in-transaction and the occurrence is immutable).*
  21. **Holdout hygiene — CONFIRM.** A final untouched holdout survives
      all `adjust` cycles, consulted once pre-verdict; *sequential-testing
      procedure recorded as viable, declined for v1.* Operator corpus
      stays judgment-unseen until P4.
  22. **Live inventory — CONFIRM, reconciled.** §1/§2/P0 now state
      26c0090: twelve crates, 18 migrations, 161 tests across 47 binaries
      (SLICE_11.md ledger + GATE_REPORT.txt agree), Slice 11 + F1
      delivered, Slice 11b pinned and preceding P0. Historical HEAD
      references in §17–§20 preserved untouched.

**The canonical amendment matrix (all proposal-only until the amendment
process passes each; adopting one decision never ratifies another's item):**

| Target | Amendment | Owner |
|--------|-----------|-------|
| doc 03 §2.2 · doc 04 §4.1 · SC-M05 | one vector per node → per node *per valid space* | D4 |
| doc 03 §2.3 · doc 04 §4.2 | similarity/weight leave the link record for space-keyed evidence | D4 |
| Dogma VI / A.9 · doc 03 §2.4 | `SUPERSEDED` (terminal) + space and evidence-set identity on the matrix | D4 |
| A.5 | `SPACE_ADOPTED · SPACE_ACTIVATED · SPACE_ABANDONED · SPACE_RETIRED · MATRIX_SUPERSEDED · CONFIG_CHANGED` | D4 |
| new A-series relation | append-only `ConfigHistory` (no prior-value recovery exists today) | D4 |
| new A-series relation | `GraphEpoch` control row + enumerated participating writes | D4 |
| A.11 | `evidence_set_ref` on AuditReport and JointProposal; validity states | D4 |
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

**Corrected dependency notation (ownership changed at the checkpoint):**
P0 → P1-A → fork{ P1-B ∥ P2A } → join → **sovereign policy/space
checkpoint** → P2B → P3 → P4-A → P4-B (loop) → P5 → P6 → P7.

**Standing readiness verdict — bounded facts only:** the twenty-two
findings close against repository and canon with the witnesses above;
every semantic choice names its declined alternative, phase owner,
criteria hook, and amendment cost; the live inventory now describes
26c0090. **Git-status at completion: this pass wrote exactly one file —
this one (untracked, uncommitted, unpinned).** The worktree separately
carries the CLI instance's in-flight Slice 11b modifications (migration
0019_doctor.sql, `ENV_DISSOLVED`/`DOCTOR_DEPLOYED` taxonomy additions,
et al.) overlaid with sandbox-bridge truncation artifacts on file tails —
neither of this pass's making, neither touched by it; all repository
witnesses above were taken from committed objects (`git show HEAD:`),
immune to both.
**This pass does not certify itself contradiction-free and does not declare
the sheet answerable** — five passes of precedent say the next independent
mechanism-composition review is the only instrument with standing to find
what this one composed wrong, and the sovereign's own review has now
out-performed the document's self-assessment six times running. The
decisions and their costs are stated in one place; the verdict on
readiness is the sovereign's.

---

*Prepared and revised under the analysis-only authorizations of
2026-07-09/10. Uncommitted; enters the tree under the two-commit lifecycle
when the sovereign so directs. No implementation, migration, dependency,
arch-test, or canon change is authorized by this document — D8 in particular
authorizes nothing until answered, and the amendment matrix (§21) stays
proposal-only until the amendment process passes each item. The eight
decisions above return before the next slice pins.*
