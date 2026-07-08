# Phase B — Slice 5: The Commitment Chain
### Pinned scope — signed off 2026-07-07 ("keep chuggin", sovereign directive)

> Section D of Document 8: Law VI end to end — Postulant emergence, the
> Auditors, the AND-barrier, Reconciliation, sovereign consent, and the
> Notary's commitment labors. Matrices are not declared; they are grown,
> tried, and professed.

## 1. Pinned criteria (Document 8, section D — 10 criteria)

| Criterion | Enforces | Seed test |
|---|---|---|
| SC-D01 | Density evaluation lacking config_rev fails; no hardcoded threshold in agent code | `sc_d01_config_citation` (arch scan + NOT NULL) |
| SC-D02 | Crossing ⇒ exactly one Postulant + one audit-eligibility flag; below ⇒ neither; weights live at the same event | `sc_d02_emergence` |
| SC-D03 | Fiat-impossibility: no write path sets CARDINAL without a resolving proposal→consent chain | `sc_d03_fiat_impossible` |
| SC-D04 | Gabriel and Lucy spawn with identical input, no pre-barrier cross-read | `sc_d04_auditor_isolation` |
| SC-D05 | Reconciliation held until both flags present AND both reports validate | `sc_d05_and_barrier` |
| SC-D06 | A claim whose evidence does not resolve fails VALIDATE_OUT and never flags | `sc_d06_truth_binding` |
| SC-D07 | AMEND applies exactly the enumerated changes (structural diff N vs N+1) | `sc_d07_amend_exact` |
| SC-D08 | Recursion reaches fixpoint; every cycle logs depth; sovereign halt exits cleanly | `sc_d08_fixpoint_and_halt` |
| SC-D09 | Decommission = human consent + Notary; dissolved matrix's links persist | `sc_d09_decommission` |
| SC-D10 | Notary refuses unresolvable chains; valid chain executes exactly once | `sc_d10_notary_chain` |

## 2. What this slice builds

- **Schemas (A.9, A.11)**: MatrixRecord (status `POSTULANT|CARDINAL|
  DISSOLVED`, revision, audit_depth, node/link refs, `config_rev` NOT NULL
  — the mandatory citation, emerged_by, committed chain), AuditReport
  (auditor, kind, claims with evidence_refs), JointProposal (verdict
  `COMMIT|AMEND|REJECT`, changes, reasons, consent_ref). Amendment kinds,
  closed: `REMOVE_LINK`, `REMOVE_NODE` (membership edits — the link
  records themselves are never destroyed; bonds outlive structures).
- **Migration 0005**: the three tables (no-delete triggers); the
  **fiat-impossibility trigger** — any transition to CARDINAL is rejected
  at the substrate unless `committed_proposal_ref → consent_ref` resolves
  and cross-references (verdict COMMIT, consent GRANTED on that proposal,
  matrix match); `readiness_flags.job_id` becomes nullable for
  office-authored flags (the supervisor's composite barrier flag — doc 3
  §3.3).
- **Store methods**: `write_flags` (one WRITTEN→FLAGGED transition may
  certify several stages — Law I.3's idempotent-upsert reading),
  `emerge_postulant` (threshold evaluation with mandatory citation; one
  live matrix per category), `file_audit_report` (truth-binding at write:
  every evidence_ref must resolve to a live node or link, else refusal and
  no flag), `read_audit_report` (pre-barrier isolation: an Auditor on the
  same matrix cannot read its counterpart's report until the barrier
  certifies), `certify_audit_barrier` (the supervisor as certifier: both
  reports present, flagged, and re-validated ⇒ composite flag),
  `file_joint_proposal`, `resolve_proposal` (human; GRANTED mints the
  consent), and the Notary's chain-validated matrix ops (`commit_matrix`,
  `amend_matrix`, `reject_matrix`, `dissolve_matrix`).
- **`crates/godhead-audit`**: `invoke_audit` (human actor — Law IV.4;
  spawns Gabriel and Lucy with identical input, logs AUDIT_OPENED with
  depth), the **floor auditors** (Gabriel affirms weighted links; Lucy
  indicts zero-weight links, severity high — deterministic, evidence-bound),
  `run_auditor`, `barrier`, `reconcile` (floor rule: Lucy's high-severity
  link indictments ⇒ AMEND removing exactly those links; none ⇒ COMMIT;
  an empty matrix ⇒ REJECT), `halt` (= sovereign DECLINE on the pending
  proposal; the Postulant stands at any depth).
- **`godhead-notary`**: `execute_proposal` — validates
  `proposal → consent → verdict`, applies exactly what was consented
  (COMMIT → CARDINAL; AMEND → revision N+1, audit_depth+1, membership
  diff = the enumerated changes and nothing else; REJECT → DISSOLVED),
  idempotent under retry; `execute_decommission` for SC-D09.
- **Aggregator step 3** (Book II §3): `consolidate` now ends with the
  emergence evaluation — density ≥ threshold creates the Postulant during
  WORK and the finish writes two flags: the consolidate result and the
  audit-eligibility flag.

## 3. Design decisions

- **Auditors and the Reconciler run on the deterministic floor in v1**:
  the protocol machinery (isolation, truth-binding, barrier, chain) is
  built and tested in full; reasoner-assisted judgment plugs into the same
  jobs when a live reasoner endpoint lands. The floor rules are real
  rules, not stubs — Lucy's indictment of weightless bonds is a genuine
  structural check.
- **One live matrix per category** (partial unique index): a category with
  a standing POSTULANT or CARDINAL does not re-emerge. Postulant
  membership is a snapshot at emergence; it changes only by audited
  amendment. (Cluster-detection finer than category-granularity is the
  breadth of a later slice.)
- **Emergence when the sovereign has spoken**: if `coherence_threshold` is
  unset, consolidate skips the emergence evaluation entirely — no
  citation, no evaluation, never a guess (Law VI.1).
- **REJECT + consent ⇒ DISSOLVED**: a Postulant whose trial the sovereign
  confirms as failed dissolves (A.9's arc); its links persist. Declining
  any proposal leaves the Postulant standing (decline is signal).
- **SC-C07 ledger**: "invoking audit" is claimed this slice —
  `invoke_audit`, `resolve_proposal`, and `decommission` take a human
  actor string; no job-identity path exists.

## 4. Non-goals

- No reasoner-assisted audit content, no environments/scriptoria (G), no
  Teachers/Students (K/L), no Deacon (I), no mandates (J).
- No sub-category cluster detection; no matrix re-emergence after
  dissolution (a fresh crossing after DISSOLVED may emerge anew — that
  path exists — but no automatic revival).
- No UI surfaces; consent remains a store call.

## 5. Edge cases

- Emergence idempotence: repeat consolidation with a live matrix ⇒ no
  second Postulant, no second eligibility flag.
- Barrier with one report corrupted out-of-band ⇒ held (III.3 at the
  barrier).
- Notary retry after successful commitment ⇒ converges, no double-apply.
- Halt at depth 0 and at depth ≥1 both exit cleanly, Postulant standing.
- Direct CARDINAL write with fabricated refs ⇒ rejected at substrate.

## 6. Delivery — gate passed 2026-07-07

All 10 criteria green (`crates/godhead-audit/tests/d_commitment.rs`);
slices 1–4 unregressed — 55 tests workspace-wide; fmt/clippy/test clean.
Migrations 0005 + 0006 applied to Railway.

**Adversarial review (27 agents, 3 lenses × verify): 24 findings — 5
refuted, 19 confirmed, 19 fixed before delivery.** The ledger:

1. *The sovereign speaks once, mechanically* — `resolve_proposal` was a
   non-atomic check-then-act; two racing answers could both land. Now one
   transaction whose guarded UPDATE (`WHERE consent_ref IS NULL`) is the
   arbiter, backed by a substrate set-once trigger on `consent_ref`
   (migration 0006).
2. *A Cardinal cannot be re-tried* — the four trial store methods now
   require POSTULANT, and the hardened commitment-chain trigger makes a
   professed Cardinal immutable in place: commitment provenance and
   membership frozen; decommission is the only door.
3. *No labor strands live* — run_auditor, reconcile, run_matrix_act, and
   the slice-3 run_grant now end every mid-labor failure in a Law VII
   refusal (the rule slices 4 established, applied uniformly).
4. *Trials are crash-recoverable* — invoke_audit resumes: it runs only the
   auditor whose report is missing (the unique report constraint is the
   arbiter) instead of wedging on "already under trial".
5. *State and record land together* — matrix mutations + their
   COMMITTED/AMENDED/DECOMMISSIONED logs, and Postulant emergence + its
   log, are single transactions.
6. *Barrier idempotence* — office flags gained a partial unique index
   (UNIQUE(job_id,stage) never bound NULL job_ids); racing certifications
   converge on one flag. The double-validation covenant extends to
   `file_joint_proposal`, which re-validates both reports at consumption.
7. Lesser: emergence distinguishes threshold-unset (skip) from a transient
   config fault (fail the pass); Notary executors require RUNNING;
   proposal resolution logs its own PROPOSAL_RESOLVED event; the
   audit-eligibility flag is consumed by the invocation it summons —
   while the POSTULANT record stays the sole authority (no registry-keeper),
   so a crash-lost flag never blocks the sovereign.
