# Application Authority Registries

**Status:** Analysis-only; non-canonical. D7 returned 2026-07-10: adopted as amended — riders: the R17a/b pairing-authority pricing (A.10/manual amendment) is acknowledged and travels with its phase. Signed Slice 11b rulings are labeled separately from D7 proposals.

- **Scope:** Application boundary, authentication and authority classes, reserved-operation and authentication-control registries, command envelopes/receipts, served executors, and Slice 11b authority surfaces.
- **Owning decisions:** D7; D1/D4 where the post-join checkpoint consumes this substrate; D8 for override command shapes.
- **Phase owner:** P2A for the minimum authentication/command substrate and checkpoint reachability; P2B for the complete application surface; later rows remain registered-and-refusing until their phase.
- **Criteria hooks:** SC-C06–SC-C07, SC-D03–SC-D10, SC-I02/SC-I07b, SC-J01/SC-J08, SC-K07, and the application `AC-` authority criteria.
- **Amendment rows sourced:** D7 authority, refusal, receipt, notice, judgment, pairing, and environment rows in [AMENDMENT_MATRIX.md](AMENDMENT_MATRIX.md); D8 command semantics are cross-linked, not redefined.

This is the sole normative proposal home for client authority and command
registry mechanics.

---

## 1. Application boundary and client posture

The v1 application is server-authoritative. Store, server, and inference run on
the operator's machine or home server. “Local-first” describes locality of the
system, not offline sovereign writes.

One operator client is enrolled. Browser sessions use header-carried bearer
tokens, never cookies; browser origins are pinned; TLS is optional on loopback
and required on LAN. Disconnected clients may read cached views and stage
files, but cannot queue consent, config, override, or other sovereign commands
against stale state.

Bearer tokens are revocable only if the server checks persisted session/client
state on every request. Losing the sole client cannot be solved by that
client's token. P2A therefore supplies a distinct local recovery authority,
described in §3.

The application contract is a versioned use-case façade, not the Store trait
and not database-shaped records. No speculative Store split is required; the
authority perimeter structurally seals the existing entrypoints first.

## 2. Authority classes and structural wall

Request data never chooses actor class. Authentication constructs one of four
unforgeable contexts:

- **`SovereignContext`:** an enrolled, authorized operator elevated for a
  named sovereign command.
- **`OperatorSession`:** authenticated but not elevated; sufficient for reads
  and operational administration.
- **`StandingAuthority`:** a machine/job context bound to an exact persisted
  trigger or executable consent revision. Schedulers never forge sovereign
  capability.
- **`RecoveryContext`:** host-local bootstrap/recovery authority, inaccessible
  to ordinary network handlers.

Every reserved operation accepts its declared context type in its signature,
or is structurally sealed behind a non-exported gate that does. A call-site
grep is only a tripwire; publicly callable sovereign primitives would remain a
bypass. End-to-end tests prove unauthenticated requests refuse before actor
elevation or persistence.

## 3. Authentication-control registry

Authentication control is a separate namespace from authority-over-data, but
it is literal rather than deferred to an ADR placeholder.

| ID | Control | Authority | Expected state / replay rule | Mode | Persistence and audit | Phase |
|---|---|---|---|---|---|---|
| A01 | Bootstrap and enroll first client | single-use deployment secret + host-local `RecoveryContext` | unused bootstrap generation; secret is digest-bound and consumed once | TX | client record, bootstrap generation, enrollment event | P2A |
| A02 | Issue/refresh session | enrolled client proof | active client revision; refresh token rotates; replay revokes the token family | TX | server-side session/token-family state and event | P2A |
| A03 | Rotate or re-enroll client credential | `SovereignContext` or `RecoveryContext` | expected client revision; old credential revoked atomically | TX | client revision and rotation event | P2A |
| A04 | Revoke session or client | `SovereignContext` or `RecoveryContext` | expected active session/client revision; idempotent terminal revocation | TX | revocation state checked on every request and event | P2A |
| A05 | Recover from sole-client loss | host-local `RecoveryContext` only | expected recovery generation; credential is single-use and rotates after success | TX | revoke old client/sessions + enroll replacement + recovery event | P2A |

The recovery credential is created at deployment, stored separately from
bearer tokens with host filesystem protection, usable only through a loopback
or host CLI path, and rotated after every use. ADR-3 chooses its concrete
format; the reachable recovery semantics are fixed here.

## 4. Command envelopes, receipts, and refusal identity

Every reserved command uses a Store-owned envelope:

- **K:** idempotency key bound to the canonical request digest; same key with
  different content refuses.
- **R:** expected revision/state CAS.
- **F:** freshness window.
- **H:** exact sovereign-reviewed content/plan hash.

Every registry row declares a completion mode:

- **TX:** envelope, effects, provenance, and log commit in one Store
  transaction.
- **RCPT:** a durable `CommandReceipt` records acceptance, digest, actor,
  expected state, resumable steps, and terminal outcome. Retry resumes it.
- **DISPATCH:** the command commits consent/pending work; a named zero-delay
  query and CAS-claim executor owns completion.

`stalled(window)` is monitoring only. Normal executors discover
`pending(..., 0)` immediately. An unshipped API command has no JobRecord, so
`UNSHIPPED_OPERATION` requires a `CommandRefusal` identity (command/request
ref) or an A.4 generalization; fabricating a job ID is forbidden.

All rows exist in the handler registry from P2A onward. A row whose phase has
not arrived returns the closed unshipped refusal through the same auth and
envelope perimeter.

## 5. Reserved-operation registry

| ID | Operation | Accepted outcomes | Authority / class | Phase | Envelope | Mode | Completion / restart |
|---|---|---|---|---|---|---|---|
| R01 | Human processing-seam dispatch | dispatched · refused | IV.4 seam crossing / `SovereignContext` | P2B | K·F·H | RCPT | job/readiness state rescanned |
| R02a | Direct `rebalance_now` | recalculated · refused | IV.4 outside trigger / `SovereignContext` | P2B | K·F | RCPT | `RebalanceState` + weight event |
| R02b | Configured rebalance tick | recalculated · refused | exact `rebalance_trigger` revision / `StandingAuthority` | P2B | trigger revision | RCPT | same; no invented general processing trigger |
| R03 | Invoke audit | cycle opened · refused | IV.4 / `SovereignContext` | P2B | K·R·F | RCPT | atomic TrialCycle open + durable two-auditor outbox |
| R04 | Resolve Joint Proposal | grant · decline | IV.4 commitment consent / `SovereignContext` | P2B | K·R·F·H | DISPATCH | pending proposal query + CAS Notary claim; `DECLINED` is the VI.4 halt |
| R05 | Consent decommission | consent minted · refused | IV.4 / `SovereignContext` | P2B | K·R·F·H | DISPATCH | pending decommission query + CAS Notary claim |
| R06 | Resolve admission | admit · reject | IV.4 threshold / `SovereignContext` | P2B | K·R·F·H | DISPATCH | admitted-unprocessed item query + Deacon CAS claim; converges on `admitted_node_ref` |
| R07 | Resolve petition occurrence | grant · decline · silence | IV.4 petition grant / `SovereignContext` | P2B | K·R·F·H | DISPATCH | pending occurrence query + CAS Notary claim; terminal attempts stop rediscovery |
| R08 | Author fetch mandate | authored · refused | IV.4 / `SovereignContext` | P6 | K·F·H | TX | mandate + log |
| R09 | Standalone sovereign config change | set · stale-refused | IV.4 config / `SovereignContext` | P2B | K·R·F·H | TX | ConfigHistory + active pointer; never used by `proceed` |
| R10 | Operational config change | set · stale-refused | `OperatorSession` | P2B | K·R·F | TX | ConfigHistory + active pointer |
| R11 | Lay category override | laid · refused | IV.1/D8 / `SovereignContext` | P2B | K·R·F·H | TX | singleton TransitionPlan |
| R12 | Lay link override (force/sever) | laid · refused | IV.1/D8 / `SovereignContext` | P2B | K·R·F·H | TX | complete active/inactive lineage-state plan |
| R13 | Lay weight override | laid · refused | IV.1/D8 / `SovereignContext` | P2B | K·R·F·H | TX | singleton TransitionPlan |
| R14 | Release override, per kind | released · stale-refused | IV.5/D8 / `SovereignContext` | P2B | K·R·F·H | TX | release record + singleton TransitionPlan; no WeightEvidence mint |
| R15 | Resolve bias warning | acknowledge · silence | HS §6.3 / `SovereignContext` | P2B | K·R·F | TX | exact warning occurrence/revision |
| R16 | Adopt Concordat | adopted · refused | A.14(b), HS §3.3 / `SovereignContext` | P5 | K·R·F·H | TX | adoption and log must become one transaction |
| R17a | Form Devout Assignment pairing | formed · refused | D7 recommendation: sovereign because pairing grants IX.5 read scope | P5 | K·R·F·H | TX | only LIVE, matching environments; A.10/HS amendment priced |
| R17b | Deploy Doctor, initial or replacement | deployed · refused | replacement canon-sovereign; D7 recommends same authority for initial / `SovereignContext` | P5 | K·R·F·H | TX | atomically create Doctor env + deployment ref + `CANONICAL_INSTRUCTION` pairing |
| R18 | Runtime adopt candidate space | PREPARING created · refused | D4 / `SovereignContext` | checkpoint+ | K·R·F·H | TX | distinct from P1-A seed migration |
| R19 | `promote_candidate` | activated · refused | D4 + IV.4 config / `SovereignContext` | checkpoint | K·R·F·H | RCPT | bounded catch-up; one final policy/disposition/catalog transaction |
| R20 | Abandon candidate space | abandoned · refused | D4 / `SovereignContext` | checkpoint | K·R·F·H | TX | failed generation may wait; no pin-time runtime authority |
| R21 | Resolve admission notice occurrence | acknowledge · silence | SC-I07b / `SovereignContext` | P2B | K·R·F·H | TX | `AdmissionNotice` occurrence lineage |
| R22 | Lift silenced bias scope | lifted · refused | HS §6.3, SC-K07 / `SovereignContext` | P2B | K·R·F·H | TX | append lift occurrence; later warning may re-arm |
| R23 | Render `SOVEREIGN_JUDGMENT` | passed · failed · returned-for-rework | HS §1.3d/B.2 / `SovereignContext` | P5 | K·R·F·H | TX | set-once review occurrence per Return/criterion/evidence digest |
| R24 | Retire environment | dissolved · stale-refused | signed Slice 11b IV.4 act / `SovereignContext` | P5 | K·R·F·H | TX | expected LIVE/ORPHANED revision; transition, cascade, provenance, log |

### Pairing and environment structural invariants

Signed Slice 11b requires a Doctor to bind through both an immutable deployment
reference and a `CANONICAL_INSTRUCTION` pairing. `deploy_doctor` is therefore
the only public operation that may create a CANON Teacher environment or that
pairing kind. Direct CANON-Teacher `establish_environment` and raw
`form_pairing(CANONICAL_INSTRUCTION)` are structurally sealed. The Store enforces
one-to-one existence of Doctor environment, deployment reference, and pairing;
no half-bound combination can commit.

Replacement deployment creates a fresh Doctor environment/reference/pairing
against fresh LIVE environments; it never revives or mutates the orphaned
records. The signed text makes replacement sovereign. D7's recommendation is
to make initial Doctor deployment and Devout pairing sovereign as well because
pairing opens the IX.5 cross-environment read aperture; the initial case's
A.10/manual amendment cost is explicit.

`orphan_environment` is not a human registry row. ORPHANED is dependency-loss
only. The primitive is sealed to decommission transactions/substrate triggers
or requires an unforgeable `DependencyLossContext`. R24 is the human choice
that moves LIVE/ORPHANED to DISSOLVED.

The committed `ffae6a8` Slice 11b specification sanctions R24 and the two-
instrument Doctor invariant but does not deliver migration 0019 or pin event
names. Inventory and A.5 event reconciliation wait for the Slice 11b delivery
commit.

## 6. Notice and review records

R21 cannot operate on Manifest's immutable `standing_notice: text` alone. It
requires append-only `AdmissionNotice` occurrences:

`notice_ref · manifest_ref · scope_digest · threshold facts · occurrence_no · state · revision · resolution_ref|null`

ACKNOWLEDGED and SILENCED resolve one occurrence. A later mandate-trip or rate
window creates a new occurrence when its scope digest differs; suppression is
never an unbounded subject-wide gag.

R22 similarly appends a lift occurrence to the bias-warning lineage. The
current unique warning row/`ON CONFLICT DO NOTHING` shape cannot implement
re-arming after lift. The proposal adds occurrence identity and the
`STANDING → ACKNOWLEDGED | SILENCED`, `SILENCED → LIFTED`, and fresh-occurrence
rules.

R23 writes an attributed, set-once review occurrence bound to exact Return,
criterion, and evidence digest. `returned-for-rework` does not mutate the
Return; it authorizes a separately represented successor instruction/workflow.

## 7. Served executors and completion

P2B supplies four zero-delay served executors: proposal execution,
decommission execution, admission processing, and petition execution. Each has
an authoritative pending query, CAS claim, retry state, terminal refusal, and
restart discovery. Monitoring separately queries work older than the configured
stall window.

The audit barrier-certification and reconciliation ticks also become served
supervisor behaviors. They bind the TrialCycle identity and refuse stale cycle
state. R04 `DECLINED` is the existing sovereign halt at any VI.4 depth; no
separate trial-halt operation is added, and R19 cannot impersonate one.

## 8. P2A/P2B ownership and exit

The post-P1 checkpoint precedes P2B. Therefore P2A—not P2B—must deliver:

- authentication bootstrap, enrollment, session, revocation, and recovery;
- unforgeable authority contexts and structural reserved-operation wall;
- digest-bound K/R/F/H Store envelope and CommandReceipt/CommandRefusal
  substrate;
- the complete registered-and-refusing handler skeleton; and
- live R19/R20 checkpoint callables.

P2A exits only when an enrolled operator can invoke R19/R20 end to end and
unauthenticated, stale, hash-mismatched, and replayed commands refuse before
elevation. P2B expands this substrate across the complete API, fills the served
executors/read models, and exits when Stop-3 is reachable without SQL, fixtures,
or crate calls.

## 9. Acceptance criteria

The application register pins tests proving at least:

- no request field can construct or select an authority context;
- every reserved row declares K/R/F/H requirements and TX/RCPT/DISPATCH mode;
- unshipped commands produce attributed command refusals without fake jobs;
- sole-client loss is recoverable locally and rotates recovery authority;
- revocation invalidates every subsequent bearer request;
- the checkpoint is reachable after P2A and before P2B;
- R19 alone performs `proceed`;
- pending executors run immediately while stall detection remains monitoring;
- no raw CANON environment/pairing or dependency-loss orphaning bypass exists;
- Doctor environment, deployment reference, and pairing are all-or-nothing;
- replacement never revives an orphaned Doctor;
- notices can be resolved and legitimately re-armed;
- terminal petition/trial refusals stop execution rediscovery but stay visible;
  and
- arch tests enumerate every data-authority row, auth-control row, context
  constructor, and structurally sealed primitive.

ADR-3 chooses token formats and module layout after D7 returns. The authority
classes, reachability, recovery, completion modes, and registry completeness
above are decision inputs.
