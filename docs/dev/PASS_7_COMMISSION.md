# PASS 7 — commission, decomposition map, and rules of engagement

Authored by Fable (author-side reviewer, Cowork session) on 2026-07-10, at
the sovereign's direction, for execution by the instance working this
worktree. Analysis-only: no implementation, migrations, tests, ADRs, or
canonical amendments. Branch commits are normal discipline here; **nothing
merges to master until the sovereign directs.**

## 0. Ground truth at commissioning

- Worktree: `C:\Users\mbc21\g0dhead_roadmap_pass7`, branch
  `codex/roadmap-pass7`, base **ffae6a8** ("docs-only (Slice 11b spec
  amendment): dissolution semantics, ruled") — verified against the main
  repo's worktree registry.
- The copied roadmap is **verified complete and true**:
  `docs/dev/ROADMAP_RECONCILIATION.md`, sha256
  `7eb8cd62d634b142032adbb74bae723d4c6258bafe8f79f3d27a33ecc951d62f`,
  153,669 bytes, 21 sections (§21 at line 2032), 2,109 non-empty lines,
  final line ending "…return before the next slice pins.*" — canaries match
  the authoring side exactly.
- Standing hazard, recorded: Cowork-sandbox file reads of the **main** tree
  can serve stale truncated content (twice witnessed). All hashing and
  copying is host-side; committed objects (`git show HEAD:`) are always
  safe. This worktree's mount verified fresh at commissioning.
- The main tree carries Slice 11b's in-flight build. The git-quiet rule
  applies there; this branch exists so the roadmap never collides with it.

## 1. Commit order

1. **First commit: the roadmap verbatim** — one file, added by name, the
   sha256 above quoted in the commit message. History starts from the exact
   artifact six passes produced.
2. After that commit lands, the **main-tree untracked original is deleted**
   (sovereign's hand, recorded) — one live copy, ever.
3. Decomposition commits follow (§2), then Pass-7 finding commits (§3).
   Small commits, each naming what moved or changed; add by name, never
   `git add -A`.

## 2. Decomposition map — one normative home per mechanism

Rules: the controlling document and the decision sheet **link, never
restate** (restatement is where passes 4–6 bred divergence). Every annex
opens with: scope; owning decision(s); phase owner; criteria hooks; the
amendment-matrix rows it sources. Superseded mechanisms live only in the
ledger. Prior ledgers move **verbatim** — visible retractions and
historical HEAD claims intact.

| Target file | Receives (from the roadmap's §§) |
|---|---|
| `ROADMAP_RECONCILIATION.md` (controlling; slims to ~400 lines) | §1–§2 (current-HEAD inventory), §4–§7, §8 phase order with entry/exit conditions linking annexes, §9 (D3 transport), §12 evaluation plans + holdout hygiene, §13, §14, §15 decision sheet D1–D8 as links + recommendations only, §16 ADR extraction |
| `TRIAL_AND_EVIDENCE.md` | §10's TrialEvidenceSet bundle, validity state machine, atomic trial opening, the Postulant disposition table, evidence citations on reports/proposal/consent/Cardinal |
| `SPACE_PROMOTION_AND_EPOCHS.md` | §10's EmbeddingSpace identity/lifecycle, three-generation storage, `space_ref` propagation, GraphEpoch + emergence serialization, `CandidateEvaluationContext`, checkpoint composition and promotion, Cardinal-frozen/model-change |
| `OVERRIDE_LINEAGES.md` | §3.1 entire: bond/measurement split, compatibility table, `OverrideLineage`/`PetitionOccurrence`/`TransitionPlan`, four-kind release table, effective-selection order, SC-C01 resolution, §3.1 criteria |
| `AUTHORITY_REGISTRIES.md` | §11 + §11.1 registry, envelope and receipt semantics, executor ticks, the authentication-control registry (populate it — it is currently a pointer with no rows) |
| `AMENDMENT_MATRIX.md` | §21's matrix, plus every amendment Pass 7 adds; owner column (D4/D7/D8) is load-bearing |
| `REVIEW_LEDGER.md` | §17–§21 verbatim, then the Pass-7 ledger |

Placement latitude: §9 and §12 may move to annexes if the controlling doc
reads better; the one-normative-home rule is what binds, not this table's
row boundaries.

## 3. Pass-7 scope — the audit findings

Verify every claim against repository and canon before editing; record
CONFIRM / AMEND / REJECT with exact witnesses; preserve disagreement where
evidence refutes. **Findings 1–5 arrive pre-conceded by the pass-6 author**
(verified against his own text) with fix shapes — re-verify them anyway;
concession is not evidence:

1. **Trial re-audit identity** — "one evidence set per matrix revision"
   (atomic-open rule) collides with supersession→re-audit at an unchanged
   revision. Fix shape: one **live** set per (matrix, revision); superseded
   sets retained; a trial-cycle counter. → `TRIAL_AND_EVIDENCE.md`.
2. **Checkpoint precedes its envelope substrate** — the uniform store-owned
   envelope is P2B's; the checkpoint runs post-join, pre-P2B. Fix shape:
   the **minimum sovereign command substrate moves into P2A's Tranche-A
   contents explicitly** (semantics-independent authority machinery).
   → `AUTHORITY_REGISTRIES.md` + controlling doc's P2A contents.
3. **Atomic promotion still split** — R09+R19 are two commands, two
   envelopes. Fix shape: **R19 alone carries the staged snapshot** — one
   command, one envelope, one transaction; R09 is not part of `proceed`.
   → `SPACE_PROMOTION_AND_EPOCHS.md` + registry.
4. **GraphEpoch starvation** — continuous intake re-advances the epoch
   between certification and switch indefinitely. State the **liveness
   requirement** (activation completes under continuous intake); candidate
   shapes: bounded delta-drain + final micro-fence, or writer
   classification / scoped epochs; ADR-2 picks the implementation.
   → `SPACE_PROMOTION_AND_EPOCHS.md`.
5. **Release selection order** — as literally written, as-stands shadows
   same-epoch post-trigger machine evidence forever. Fix shape: active head
   → post-release lawful evidence at current epoch → as-stands (or
   consume-on-replacement). → `OVERRIDE_LINEAGES.md`.
6. **D7 completeness + Slice 11b surfaces** — to verify fresh at current
   HEAD: `retire_environment`, Doctor deployment, `ENV_DISSOLVED` /
   `DOCTOR_DEPLOYED`, migration 0019 (once delivered) — are any of these
   sovereign operations the registry must carry? Plus the open pendings:
   R17a's authority, the trial-halt decision, the unpopulated
   authentication-control registry. → `AUTHORITY_REGISTRIES.md`.

Then: **a composition sweep of each annex after extraction** — extraction
itself can breed divergence, and the sheet's links must resolve to exactly
one normative statement per mechanism.

## 4. Post-11b reconciliation (deferred, named now)

When Slice 11b delivers on master: rebase `codex/roadmap-pass7` onto the
delivery commit; re-run the current-HEAD inventory (crates, migrations,
tests, delivered slices); audit the registry and matrix against 11b's new
surfaces. One commit, its own ledger entry.

## 5. Rules of engagement (the standing ones, binding here)

Witnesses from committed objects; dispositions with exact citations;
disagreement preserved; visible retractions; scope discipline (no phase
redesign absent a proven impossibility); ledgers are history, never
rewritten; **no self-certification of contradiction-freedom and no
answerability declaration on the pass's own authority** — six passes of
precedent, each self-assessment outperformed by the next independent
review. The Pass-7 ledger goes to `REVIEW_LEDGER.md`: findings, witnesses,
choices, declined alternatives, owners, criteria, amendment costs, and a
bounded verdict. The eight decisions return to the sovereign before the
next slice pins.
