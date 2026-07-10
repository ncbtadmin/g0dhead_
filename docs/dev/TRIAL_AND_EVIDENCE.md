# Trial and Evidence Lifecycle

**Status:** Analysis-only; non-canonical and proposal-only until D4 returns.

- **Scope:** Trial-cycle identity, evidence manifests, validity, atomic opening, activation-time Postulant disposition, and citations through reports, proposal, consent, execution, and Cardinal.
- **Owning decisions:** D4; D7 where sovereign disposition and command authority intersect.
- **Phase owner:** P2B, consuming the space substrate from P1-A and the minimum authority substrate from P2A.
- **Criteria hooks:** SC-D01–SC-D10, VI.3–VI.4, and the activation/trial `AC-` criteria below.
- **Amendment rows sourced:** D4 trial/evidence and matrix-status rows in [AMENDMENT_MATRIX.md](AMENDMENT_MATRIX.md).

This is the sole normative proposal home for trial identity, evidence validity,
and activation-time trial disposition. Space promotion consumes these records
by reference; it does not redefine them.

---

## 1. Repository constraint and the missing identity

At `ffae6a8`, trial identity stops at `(matrix_ref, matrix_revision)`. The
substrate permits one report per auditor at that revision, one Joint Proposal
at that revision, and one barrier stage for that revision. `invoke_audit`
adopts the existing report pair when called again. Those rules are coherent for
one attempt, but they make Pass 6's required
`VALID → SUPERSEDED → fresh re-audit` impossible when evidence drifts without a
matrix revision change.

The correction is a first-class, append-only **`TrialCycle`**:

`trial_cycle_ref · matrix_ref · matrix_revision · cycle_no · evidence_set_ref · prior_cycle_ref|null · status: VALID|SUPERSEDED|DISTRUSTED|CLOSED · trial_state_revision · opened_at · terminal_ref|null`

- `cycle_no` measures evidence-drift retries. It is distinct from VI.4's
  `audit_depth`, which measures consented AMEND recursion across matrix
  revisions.
- Exactly one cycle may be `VALID` for a matrix revision. Superseded,
  distrusted, and closed cycles remain immutable and citable.
- Reports, barrier certification, Joint Proposal, consent provenance,
  execution attempts, command receipts, and the committed Cardinal resolve to
  the same `trial_cycle_ref` and `evidence_set_ref`.
- Uniqueness moves from matrix revision alone to trial cycle: one report per
  `(trial_cycle_ref, auditor)`, one barrier per cycle, and one proposal per
  cycle. The matrix revision remains part of every validation predicate.
- Every report, barrier, proposal, consent, execution-attempt, job-state, or
  receipt transition advances `trial_state_revision`. Space activation binds
  and CAS-validates it; a matrix-row CAS alone cannot observe child-state
  races.

## 2. Atomic opening and served continuation

Opening is one retry-stable Store transaction. It:

1. validates `(matrix_ref, matrix_revision, POSTULANT, originating space)`;
2. assembles the evidence manifest from one coherent snapshot, or revalidates
   every captured dependency token at the end of assembly;
3. returns the standing `VALID` cycle if the same digest-bound command already
   opened it, otherwise creates the next cycle and its evidence set;
4. consumes the applicable audit-eligibility flag;
5. records `AUDIT_OPENED` citing both identities; and
6. durably enqueues exactly two auditor work items through R03's command
   receipt/outbox boundary in [AUTHORITY_REGISTRIES.md](AUTHORITY_REGISTRIES.md).

A crash after acceptance therefore rediscovers the same cycle and outstanding
auditor work. It cannot mint a second live cycle or leave an opened trial with
no served continuation. A missing eligibility flag is not silently ignored:
initial opening requires it; a drift-triggered successor uses the predecessor's
terminal invalidation record as its eligibility witness.

## 3. Immutable `TrialEvidenceSet`

The evidence set freezes what the cycle actually tried at audit-open. It binds
immutable values and identities rather than mutable revision pointers:

- `space_ref`, catalog revision, and the source graph cursor;
- the exact policy snapshot and values, including coherence and
  link-similarity thresholds;
- qualification and weight algorithms/versions, weight mode, and—where
  assisted—the reasoner/model digest and prompt/policy version;
- calculation-run identities and input/output digests;
- exact node and bond membership at the tried matrix revision;
- for every relevant node and bond, the complete `EffectiveLineageWitness`
  defined only in [OVERRIDE_LINEAGES.md](OVERRIDE_LINEAGES.md), including
  inactive released state; and
- for every tried bond, the selected qualification and effective weight source
  with space, policy, lineage epoch, effective revision, and evidence/run ref.

The annex consumes that closed witness shape by reference rather than defining
a second copy. It replaces Pass 6's incomplete choice between machine evidence
and an active override head: a released category, severance, or weight can
govern the trial even when `active_head_ref` is null.

Both auditors' reports and the Joint Proposal cite the cycle and evidence set
directly. A ConsentRecord resolves them transitively through its immutable
proposal subject; A.12 need not duplicate their fields. The Cardinal cites them
directly at commitment. An AMEND verdict creates matrix revision N+1 and opens
cycle 1 there; evidence drift at the same matrix revision opens cycle N+1 on
that revision.

## 4. Validity without global starvation

Validity is current-state revalidation, but not global-change invalidation.
The global `GraphEpoch` may remain an explanatory change cursor; an unrelated
file, category, bond, or matrix must not supersede every human-paced trial.

At barrier certification, proposal filing, consent, and Notary execution, the
Store CAS-validates the evidence set's **dependency scope**:

- matrix revision/status and `trial_state_revision`;
- active space/catalog and exact policy snapshot;
- member nodes and bonds;
- selected qualification/weight runs; and
- every relevant effective-lineage state and epoch.

Relevant drift marks the cycle `SUPERSEDED`; integrity failure marks it
`DISTRUSTED`. Advancement refuses and a successor cycle becomes eligible.
Unrelated graph writes do not affect the cycle.

If drift is detected after consent, the consent remains immutable and visible,
but it is not executable against a successor cycle. The Notary writes a
terminal `STALE_TRIAL_EVIDENCE` refusal/attempt record and closes discovery for
that execution; zero-delay executors do not retry it forever. The stale consent
continues to block activation until either a successor cycle reaches a lawful
terminal result or R19's exact promotion plan explicitly acknowledges the
terminal refusal. Nothing granted fails quietly, and an impossible grant does
not become an immortal runnable job.

## 5. Total Postulant disposition at activation

R19 evaluates the authoritative complete set of old-space Postulants inside its
transaction. Its submitted disposition domain must equal that set at the
expected catalog revision and each matrix's `trial_state_revision`; “every
Postulant it lists” is not a completeness rule.

| State at locked promotion snapshot | Activation behavior |
|---|---|
| No trial cycle ever opened | Auto-supersede; preserve matrix provenance. |
| Closed/incomplete cycle, no proposal, no live job or receipt | Auto-supersede; preserve partial evidence and refusal history. This includes zero reports or one report when all counterpart labor is terminal. |
| Proposal `DECLINED` through R04 | Auto-supersede. R04 is the existing sovereign halt; R19 does not invent a second halt operation. |
| Proposal unresolved | Block. The sovereign resolves it through R04 before promotion; R19 cannot answer a VI.3 proposal indirectly. |
| Consent granted and executable, or execution/job/receipt live | Block until execution reaches a lawful terminal result. Route COMMIT → Cardinal, AMEND → matrix revision N+1, REJECT → Dissolved. |
| Consent closed by terminal stale-evidence refusal | Block until a successor cycle resolves or the R19 plan explicitly acknowledges that refusal. |
| Consent-applied AMEND revision awaiting re-audit | Require an exact R19 disposition entry naming matrix revision, cycle state, and the sovereign-reviewed supersession consequence. |

Every old-space Postulant therefore leaves the live set by a named rule or
blocks the switch. `SUPERSEDED` is terminal, excluded from
`one_live_matrix_per_category`, preserved forever, and logged as
`MATRIX_SUPERSEDED`. Space promotion owns the atomic switch; this annex owns the
classification and trial-state predicates it consumes.

## 6. Acceptance criteria

P2B pins citable `AC-` criteria proving at least:

- one and only one `VALID` cycle per matrix revision, with retained historical
  cycles and retry convergence;
- same-revision evidence drift opens a fresh cycle without colliding with old
  reports, barrier, or proposal uniqueness;
- `audit_depth` and `cycle_no` cannot be confused;
- atomic opening cannot strand or duplicate auditor labor;
- every VI.3 handoff cites and CAS-validates the same cycle/evidence identity;
- unrelated graph writes do not invalidate a trial;
- released effective state is citable when no active head exists;
- stale post-consent execution becomes a terminal, surfaced refusal rather
  than an endless runnable grant;
- activation sees a complete, race-free Postulant/trial-state set;
- R19 cannot silently resolve an outstanding proposal; and
- COMMIT, AMEND, REJECT, decline, incomplete labor, stale consent, and live
  labor each follow exactly one disposition rule.

Final DDL belongs to ADR-2 after D4 returns. These observable identities and
failure semantics are decision inputs, not deferred implementation detail.
