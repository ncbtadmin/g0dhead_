# g0dhead_ — The Working Discipline (tree mirror)
### Ratified by sovereign decision S7, 2026-07-09 — mirrored into the tree per the rule it states

This file records the evidentiary and testing discipline that governs Phase B work.
It is a mirror: the discipline was practiced before it was written down, and it is
written down because a rule that lives in a conversation is a rule the next session
can miss.

## 1. Per-claim status tagging (memory & reports)

Every claim carried in project memory or a verification report bears one tag:

- **CLAIM** — asserted somewhere, verified nowhere. Carried only with its source named.
- **VERIFIED(witness)** — established first-hand; the witness (file:line, command
  output, hash) is cited. A VERIFIED without a witness is a CLAIM wearing a costume.
- **REFUTED(why)** — checked and found false; the refutation is recorded, not deleted.
- **INFERENCE** — a conclusion from verified facts, labeled as such; it does not
  promote itself.
- **REFUTED-BY-REVIEWER** — an external reviewer's refutation, accepted after check.
- **RULED(author, ref)** — an author-intent ruling from `docs/dev/PROMPT_G_RULINGS.md`
  (or a successor rulings file), cited by section. Binds what a criterion or law
  *meant*. Distinct from VERIFIED (a code fact) and from SOVEREIGN (a decision only
  the sovereign makes).

**The authority model:** author rulings bind interpretation; first-hand code reads
outrank everyone on what the code *does*; the sovereign outranks everyone on what
happens *next*.

**The tree-witness rule:** nothing in memory may assert what nothing in the tree can
witness. Memory points; the tree proves. Index lines describe, never assert.

## 2. The honest-annotation convention (ruling G13)

Any test satisfying less than its criterion's words says so **in the test**: it names
the unmet half and names where that half re-arms; the slice doc pins it as residue.
This sits beside the standing rule that **tests only accumulate** — a test, once
green at a gate, is never deleted or weakened except by an amendment that records
why. An honest gap is debt; a silent one is the beginning of exactly the drift this
order was built to make impossible to miss.

## 3. Counts live in files, not prose (ruling G12)

A number in prose is a registry-keeper, and the store is the only truth. The
criterion count is `docs/08_phase_b_success_criteria.md` itself, swept by
`docs/dev/criteria_sweep.py`; the test count is the gate's output. Prose cites the
file, never the number.

## 4. Commit hygiene

While untracked material sits deliberately in the working tree (the origin
conversation; see `docs/_history/PROVENANCE.md`), commits add files **by name** —
never `git add -A`, never `git add .`.

## 5. The two-commit slice lifecycle (standing from Slice 11)

A slice's specification and its delivery are **two commits, not one**, so the
spec is a durable, timestamped witness that predates the code it governs.

- **The spec commits at pin-time — before any code moves against it.** When a
  slice's criteria pin (§1 onward), that pinned spec is its own docs-only commit
  and crosses the desk *then*, ahead of implementation. Where the sovereign
  rules a boundary or crossing ahead of the criteria (a §0), that ruling may be
  recorded earlier still, as its own act — as `SLICE_11.md` §0 was.
- **The delivery ledger appends at delivery.** The §9 delivery ledger — gate
  report, adversarial ledger, archaeology, regenerated sweep — is written and
  committed with the delivered code, crossing the same desk the spec crossed.

The rule makes the sequence auditable from the tree alone: a spec commit whose
timestamp precedes its slice's first code commit *is* the witness that the spec
crossed the desk before the code moved — the discipline every slice already
practiced, now provable without a transcript. Slices 1–10 recorded spec and
delivery in one commit; from Slice 11 the two are distinct, and the producer
(`scripts/gate_report.py`) remains the only voice of the gate in the delivery
half.

## 6. Vocabulary reconciliation (the returned decisions, 2026-07-10)

The desk's old coarse phase words were retired at the 2026-07-10 sitting; the
D1 phase sequence (`ROADMAP_RECONCILIATION.md` §15.1) names their successors.
Both lanes use the new words, so no one says "Phase 5" again and means two
different things:

| Old (desk/CLI usage) | New (D1) |
|---|---|
| "Phase 4 — SC-C07 ledger sweep" | absorbed: entries live in the R-registry rows and their arch-pins |
| "Phase 5 — transport + endpoints + wall deletion + SC-F06 half" | **split**: model egress → P1-A (adapter) and P5 (`propose_call`, SC-F06 half); fetch egress + wall deletion → **P6** |
| "Phase 6 — desktop client" | **P3** (thin client), P4 (product gate) |
| SC-J09 fetch half re-arm | P6 |
| SC-I05 purge half / Duty of the House | P7, now with §12's storage-growth *evidence* feeding it |
| retire's shape-guard → actor-class upgrade (11b residual) | P2A (authority substrate) |
| deploy_doctor atomicity residual (11b) | P2B (R17b all-or-nothing invariant) |
