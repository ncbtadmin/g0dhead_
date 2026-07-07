# Phase B — Slice 3: Human Sovereignty
### Pinned scope — signed off 2026-07-07

> Section C of Document 8: Law IV made mechanical — overrides, petitions,
> consents, and the Notary's grant execution (IV.5). Stands on slices 1–2.

## 1. Pinned criteria (Document 8, section C — 7 criteria)

| Criterion | Enforces | Seed test |
|---|---|---|
| SC-C01 | Mutating an overridden record without resolving consent_ref is rejected at the store, regardless of writer | `sc_c01_override_protection` |
| SC-C02 | Recurring petition on same (subject, kind) increments occurrence_count, OPEN → ESCALATED | `sc_c02_petition_escalation` |
| SC-C03 | SILENCED suppresses future matches; suppressed attempts logged `severity: suppressed`, never purged | `sc_c03_silenced_suppression` |
| SC-C04 | Grant execution lays a successor override: `user_overridden: true`, `basis: GRANTED_PETITION`, prior_ref + consent_ref resolve | `sc_c04_grant_lays_successor` |
| SC-C05 | Post-grant agent mutation still rejected — protection persists through the grant | `sc_c05_post_grant_protection` |
| SC-C06 | GRANTED petition with no completed Notary execution inside the stall window is surfaced | `sc_c06_stall_surfaced` |
| SC-C07 | IV.4 entries have no agent-callable path; attempts are refused GATE_BYPASS_ATTEMPT | `sc_c07_gate_bypass` |

## 2. What this slice builds

- **Schemas (A.7, A.12)**: OverrideRecord (kind, basis, prior_ref successor
  chain, consent_ref required iff GRANTED_PETITION, protected_state),
  PetitionRecord (status lineage, occurrence_count, execution_job_ref,
  plus `proposed_change` — a v1 mechanical necessity: the Notary applies
  *exactly* what was petitioned, so the petition must carry it),
  ConsentRecord (decision, scope, decided_by).
- **The live override surface is node classification** (`CATEGORY_REASSIGNED`).
  `LINK_SEVERED / LINK_FORCED / WEIGHT_CORRECTED` get schema now;
  enforcement tests arrive with the link and weight slices.
- **Store methods**: `lay_category_override` (sovereign hand: applies and
  protects in one act), `get_active_override`, `open_petition` (escalation
  and suppression logic), `resolve_petition` (GRANTED mints the consent),
  `execute_grant` (the transactional chain-validated apply + successor
  override), `stalled_grants`, `get_petition`. Sovereign-act methods take a
  human actor string and no job identity — agent-uncallable by signature.
- **`crates/godhead-notary`**: the Notary's first labor — spawned on a
  GRANTED-unexecuted petition, validates `override → petition → consent`,
  applies exactly the granted change, dies. Full Book I lifecycle. A
  subject that no longer validates → Law VII refusal, petition stands
  GRANTED-unexecuted (surfaced by SC-C06's machinery).
- **Migration 0003**: the three tables, `petition_stall_ms` (operational,
  default 60000), and the **agent-author trigger**: override, consent, and
  config rows whose `produced_by` is job-UUID-shaped are rejected at the
  substrate with a GATE_BYPASS_ATTEMPT message. Agent writes always stamp
  their job UUID; sovereign acts stamp a human actor string — so the
  distinction is structural, not honor-system. The successor override laid
  under a grant is stamped with the consent's decider: the authority is
  the consent, never the Notary (IV.5 — the grant changes the datum, never
  its keeper).

## 3. SC-C07 coverage ledger

Tested this slice (surfaces exist): granting/resolving petitions, minting
consents, laying overrides, altering config constants. Pinned to future
slices (surfaces don't exist yet, structurally uncallable today):
crossing the seam → section M/ML slice · invoking rebalance → M · invoking
audit → D · admitting at the threshold → I · authoring fetch mandates → J.
None of these may silently drop; each future slice's spec must claim its
entry.

## 4. Non-goals

- No links, weights, matrices, audits, or Deacon.
- No petition UI/notification surface — the three terminal answers are
  store calls; presentation is the client's concern, later.
- Amendment/decommission Notary labors (SC-D07/D09/D10) wait for section D.

## 5. Edge cases

- Petition on a subject with no override (agents may petition any
  human-held state question — but v1 restricts to existing overrides;
  petitioning an un-overridden subject is VALIDATION_FAILED).
- Grant executed twice (Notary retry): converges, no double-apply
  (execution_job_ref + revision CAS).
- Silence then re-petition ×N: one record, counting, logged each time.
- execute_grant on a bogus or non-GRANTED petition: refused.

## 6. Delivery — gate passed 2026-07-07

All 7 criteria green (`crates/godhead-notary/tests/c_sovereignty.rs`),
slices 1–2 unregressed — 38 tests workspace-wide; fmt/clippy/test clean.
Implementation notes vs. the design:

- Petition lineage semantics: a GRANTED-unexecuted petition is not
  re-petitionable (the loop must close first); a GRANTED-executed lineage
  re-opens as ESCALATED with the count advanced — execution history
  survives in the consent, the override chain, and the logs.
- `stalled_grants(0)` doubles as the dispatcher's executable-consent query
  (`grants_tick`); the SC-C06 surface and the summoning rule are the same
  state read with two windows.
- The agent-author trigger also covers `config_constants`, closing the
  "order that tunes its own law" door at the substrate.
- Petition and consent tables carry no-delete triggers (IV.3: declines are
  signal; no cleanup purges them).
