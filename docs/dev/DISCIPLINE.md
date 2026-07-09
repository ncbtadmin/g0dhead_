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
