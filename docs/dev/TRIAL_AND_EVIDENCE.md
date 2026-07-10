# Trial and Evidence Lifecycle

**Status:** Analysis-only; non-canonical and proposal-only until D4 returns.

- **Scope:** Trial-cycle identity, evidence manifests, validity, atomic opening, Postulant disposition at activation, and citations through reports, proposal, consent, and Cardinal.
- **Owning decisions:** D4; D7 where sovereign disposition and command authority intersect.
- **Phase owner:** P2B, consuming the space substrate from P1-A and the authority substrate from P2A.
- **Criteria hooks:** SC-D01â€“SC-D10, VI.3â€“VI.4, and the activation/trial AC- criteria proposed here.
- **Amendment rows sourced:** D4 trial/evidence and matrix-status rows in [AMENDMENT_MATRIX.md](roadmap_reconciliation/AMENDMENT_MATRIX.md).

This is the sole normative proposal home for trial identity, evidence validity,
and activation-time trial disposition.

---

## Postulant disposition during space activation
**Postulants at
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

## Trial evidence and validity

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
