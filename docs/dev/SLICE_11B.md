# Phase B — Slice 11b: The Doctor & the Dissolution Cascade
### Nascent — spec only. Pins as its own docs commit and crosses the desk BEFORE code (two-commit lifecycle, DISCIPLINE.md §5). The build bar stays down until sign-off.

> The one piece Section J severed (SLICE_11.md §7): SC-J08, the Doctor's
> deployment and its orphaning. It comes home here — but the dissolution
> semantics it rests on are **argued, not inherited**: what lawfully dissolves a
> Student scriptorium, where the cascade is enforced, and how no revival is
> possible. In closing SC-J08 this slice also pays a named debt: the ORPHANED
> cascade SLICE_07 built the ground for and deliberately deferred.

## 0. The dissolution semantics — argued

SC-J08 asserts three things (Holy Standard §4.3; doc 08 §J): a Doctor deploys
only against a **LIVE** Canon Student; **dissolving** that Student's scriptorium
drives the Doctor's environment **ORPHANED**; and no fresh Student silently
revives it. None of those words may be inherited — this section says what each
means at the substrate and the store.

### 0.1 A.8's arc, and what its states *mean*

An environment (A.8) holds one of three states, and this slice pins the
distinction the manuals imply but never spell out:

- **LIVE** — a workplace: mountable, writable, invokable.
- **ORPHANED** — *a dependency lost.* A room whose ground (its matrix) was
  decommissioned, or whose depended-on room left LIVE — kept as a **read-only
  archive** (SC-G07: readable, unmountable-for-work, no write succeeds). This is
  the **cascade** outcome: the arc reaches it by loss, never by choice. Doc 06:83
  already rules it — *"If its matrix is decommissioned, the environment goes
  `ORPHANED` (A.8): read-only archive, provenance intact."* The Canon Student's
  corpus and the Doctor's packages are too valuable to strike; they are kept.
- **DISSOLVED** — *a deliberate retirement.* A room struck by an explicit
  sovereign act (`retire_environment`, §0.2), never by any loss. It is the only
  terminal state a human reaches for on purpose; nothing cascades a room here.

The load-bearing pin (corrected from the first draft): **ORPHANED is what a
dependency-loss produces** — a matrix decommissioning (doc 06:83), or a
depended-on room leaving LIVE; **DISSOLVED is what a deliberate retirement
produces.** So §4.3's "the Canon Student environment *dissolves*, the Doctor's
goes *ORPHANED*" reads, mechanically, as: the Student **leaves LIVE** — by
decommission→ORPHANED or by retirement→DISSOLVED — and the Doctor, depending on
it, goes ORPHANED.

### 0.2 What takes a scriptorium out of LIVE — the two levers

Two lawful roots, two outcomes:

- **Matrix decommission → ORPHANED** (doc 06:83, already ruled).
  `consent_decommission` → `execute_decommission` (Notary-executed, consent-gated,
  Law VI.5) sets the matrix `DISSOLVED` and today **stops there** — it does not
  touch the environments on that matrix. That stopping point *is* the SLICE_07
  deferred cascade. This slice extends it: **decommissioning a matrix ORPHANS
  every environment standing on it** — a room whose ground is gone is *preserved*
  as a read-only archive (provenance intact, doc 06:83), not struck. This is what
  "dissolving the Canon scriptorium" (SC-J08) means in practice: the Canon
  Student's room leaves LIVE for `ORPHANED` when its matrix decommissions.
- **`retire_environment` → DISSOLVED** (new). `DISSOLVED` has exactly one lever —
  a deliberate sovereign act, `retire_environment` (human actor, no job identity:
  a IV.4 human-reserved action, closed-list amendment, author-sanctioned §0.6).
  It is the only path to `DISSOLVED`; nothing cascades a room there. A room is
  retired on purpose, or not at all.

Either lever takes the room out of LIVE — and *that departure, by either lever*,
is what the Doctor's orphaning keys on (§0.5), so no path is privileged.

### 0.3 Where each piece is enforced — substrate and store

Split by what each guarantees:

- **The matrix→environment ORPHANED cascade: STORE** (`execute_decommission`
  extended, in its existing transaction — matrix `DISSOLVED`, then the
  environments on it `ORPHANED`, one atomic act and record). Decommission is
  consent-gated, Notary-only, with no lawful raw-SQL route (ruling G6), so the
  store is its enforcement point, where `orphan_environment` already lives.
- **`retire_environment`: STORE**, a sovereign act — guarded by the substrate's
  existing human-author wall (a job-uuid actor is a `GATE_BYPASS_ATTEMPT`, IV.4)
  and the status-monotonicity wall below.
- **The Doctor-orphan cascade: SUBSTRATE.** When a Canon Student environment
  leaves LIVE — for `ORPHANED` (decommission) *or* `DISSOLVED` (retirement) — a
  trigger orphans every Doctor whose deployment names it. Keying the trigger on
  the environment's **departure from LIVE** catches *both* levers with one wall,
  so no path can forget the dependent (the "by either lever" guarantee) — it is a
  dependency-integrity rule, not a courtesy of one caller. Placing it here, not
  in each store method, is why a future dissolution path cannot silently skip it.
- **The no-revival wall: SUBSTRATE** (§0.5): "read-only archive, no
  resurrection" must hold against every writer — a trigger, like V.4 and the
  preservation walls, not store logic a path could forget.

### 0.4 The Doctor's binding — both instruments

The Doctor (a **Canon Teacher**, §4.3) binds to its Canon Student by **both**
instruments, and it needs both:

- **The dependency reference** `doctor_env_ref → student_env_ref` (§4.3's
  dependency law) — this is what the orphan cascade *walks*: when the Student
  leaves LIVE, the reference names which Doctors go `ORPHANED`.
- **The `CANONICAL_INSTRUCTION` pairing** (X.5) — because IX.5, the Pairing
  Exception, is the *only* bridge across which the Doctor may read the Student's
  CorpusManifest; without the pairing the Doctor has no lawful sight of the
  corpus it exists to weaponize. Doc 06:90's "the sovereign **pairs anew**"
  presumes exactly this pairing.

The first draft's "reference, not pairing" is retired. The pairing is not a
co-tenancy to fear but the corpus bridge, required. And the fear is moot: because
matrix decommission now **ORPHANS** rather than dissolves (§0.2), a Doctor that
shares its Student's matrix is not force-dissolved — both rooms orphan together,
and the Doctor would orphan anyway through the reference the moment its Student
left LIVE. **The Doctor orphans whenever its Student's room leaves LIVE, by
either lever** — decommission or retirement.

### 0.5 The no-revival wall

Two guarantees, both substrate:

1. **Status never regresses.** A trigger on `environments` forbids
   `ORPHANED → LIVE` and any transition out of `DISSOLVED`. Orphaned is terminal
   for work; the archive is never reanimated in place. (LIVE→ORPHANED and
   LIVE/ORPHANED→DISSOLVED remain lawful — the arc only ever descends: ORPHANED
   by loss, DISSOLVED by deliberate retirement.)
2. **A fresh Student does not adopt the old Doctor.** Deploying against a new
   Canon Student environment mints a **new** Doctor env, a **new** reference, and
   a **new** pairing (§0.4); the orphaned Doctor is untouched and stays orphaned.
   Doc 06:90's "the sovereign **pairs anew**" is exactly this — a fresh sovereign
   deployment, never a status flip on the old. This falls out of (1) plus deploy
   always minting fresh, but the criterion tests it as behavior.

### 0.6 The debt this closes, and the amendment it carries

**The SLICE_07 cascade debt, by name.** SLICE_07 §Non-goals: *"No ORPHANED
cascade rules from the manuals (a Doctor orphaning when its Canon Student
dissolves) — that is K's; the status field + the unmountable behavior is built,
the cascade is not."* And doc 05 round-3 note 2 added the A.8 status field "open
for ratification … the type-specific behavior lives in the manuals." Slice 11b
builds that cascade — including doc 06:83's own "if its matrix is decommissioned,
the environment goes ORPHANED," specified but never wired — and that
type-specific behavior. On delivery this debt is closed **on the record**: the §9
ledger names SLICE_07's deferral and marks it discharged.

**The IV.4 amendment `retire_environment` carries (author-sanctioned).** DISSOLVED
gets its one lever, and a human-reserved action gets added to IV.4's closed list
of sovereign acts: `retire_environment` (human actor, no job identity), joining
the same wall that guards override/consent/config/mandate authorship. The doc 05
IV.4 closed-list amendment rides with this slice's delivery, and it comes with its
gate-bypass test — an agent-shaped (job-uuid) retire is rejected
`GATE_BYPASS_ATTEMPT`, exactly as SC-C07's mandate entry proved for authorship.

## 1. Pinned criteria — SC-J08, home, a leg at a time

| Criterion (leg) | Enforces | Seed test |
|---|---|---|
| SC-J08 (a) — **deploy requires LIVE** | `deploy_doctor` against a `student_env_ref` that is not a LIVE Canon Student refuses `ENV_INVALID`; a LIVE Canon Student deploys — establishing the Doctor's Canon Teacher env and binding it by **both** instruments (the deployment reference AND the CANONICAL_INSTRUCTION pairing, §0.4). Student first, always. | `sc_j08_deploy_requires_live` |
| SC-J08 (b) — **leaving LIVE orphans the Doctor, by either lever** | The Canon Student's room leaving LIVE ORPHANS the Doctor through the reference (SC-G07 archive thereafter). Tested both ways: **matrix decommission** takes the Student to `ORPHANED` → Doctor orphans; **`retire_environment`** takes it to `DISSOLVED` → Doctor orphans. One substrate trigger over the departure catches both. | `sc_j08_leaving_live_orphans_doctor` |
| SC-J08 (c) — **no silent revival** | An orphaned Doctor is never returned to LIVE (the substrate status-monotonicity wall); a fresh Canon Student environment does not adopt it — a new deployment mints a new Doctor (new reference + new pairing), the old stays orphaned. | `sc_j08_no_silent_revival` |

## 2. What this slice builds

- **`deploy_doctor`** (store method, or a thin scriptorium orchestration over
  store walls): validates `student_env_ref` is a LIVE Canon Student (`ENV_INVALID`
  else, SC-J08a), establishes the Doctor's Canon Teacher environment (reusing the
  establish path), records the deployment reference `doctor_env_ref →
  student_env_ref`, and forms the `CANONICAL_INSTRUCTION` pairing over the shared
  matrix (§0.4 — both instruments). The Doctor holds office as a Canon Teacher;
  its PromptPackage labor (B.5) is **not** built here (non-goal).
- **The matrix→environment ORPHANED cascade** in `execute_decommission`
  (extended, same transaction): matrix `DISSOLVED` → every environment on it
  `ORPHANED` (doc 06:83). One atomic record. The Doctor-orphan step is NOT here —
  it rides the substrate trigger below, so it fires for retirement too.
- **`retire_environment`** (new store sovereign act): the one lever to
  `DISSOLVED` (LIVE|ORPHANED → DISSOLVED), taking a human actor and no job
  identity (IV.4). The existing agent-author wall rejects a job-uuid actor.
- **The Doctor-orphan substrate trigger** (migration): AFTER a Canon Student
  environment leaves LIVE (`OLD.status='LIVE'` → `NEW.status<>'LIVE'`), every
  Doctor whose `doctor_deployments.student_env_ref` names it goes `ORPHANED`.
  One wall, both levers (§0.3).
- **The no-revival substrate wall** (migration): a trigger on `environments`
  forbidding status regression (`ORPHANED→LIVE`, any exit from `DISSOLVED`).
- **`doctor_deployments` table** (`doctor_env_ref`, `student_env_ref`,
  append-only, frozen) — the reference the orphan trigger walks.
- **Migration(s)**: `doctor_deployments`, the Doctor-orphan trigger, the
  status-monotonicity trigger, and (author-sanctioned) the IV.4 closed-list note
  for `retire_environment`.

## 3. Non-goals

- **No PromptPackage (B.5), no Doctor *labor*** — this slice builds the Doctor's
  deployment and lifecycle (deploy → orphan → no-revival), not what it crafts.
  The package labor is a later slice.
- **No environment-dissolution beyond the two levers** — matrix decommission
  ORPHANS (§0.2) and `retire_environment` DISSOLVES; nothing else moves a room
  off LIVE, and nothing auto-dissolves.
- **No retrieval breadth, no real transport** — unchanged; the no-HTTP wall
  stands (it is Phase 5's to delete).
- **No Devout-Professor downstream behavior** — the matrix→env ORPHANED cascade
  is general (a Devout environment orphans when its matrix decommissions, doc
  06:83), but Devout-Professor-specific downstream behavior is not in scope.

## 4. Gate & delivery protocol

Doc 00 §4's three commands via the producer (`scripts/gate_report.py` — the only
voice of the gate), on the host against live Railway Postgres. The migration adds
a substrate wall, so its gate forces a `godhead-store` recompile to embed
(`cargo clean -p godhead-store`; SLICE_11 §9.1's note). Adversarial review
precedes delivery (the standing rule); the delivery ledger (§9) appends at
delivery as its own commit, carrying the review ledger, the regenerated sweep
(SC-J08 moves DEFERRED → PENDING), and the producer gate report — and names the
SLICE_07 cascade debt discharged.

---

*Presented to the sovereign 2026-07-10, pinned as commit 26c0090.*

*Amended 2026-07-10 by sovereign ruling (this commit's visible diff): §0.1–0.5
re-argued to the ruled semantics — matrix decommission cascades **ORPHANED**, not
DISSOLVED (doc 06:83); DISSOLVED's one lever is the new sovereign act
`retire_environment` (IV.4 closed-list amendment, author-sanctioned, with its
gate-bypass test); the Doctor binds by **both** instruments (the `student_env_ref`
reference that drives the orphan cascade and the `CANONICAL_INSTRUCTION` pairing
that bridges IX.5 to the corpus, §0.4); and it orphans whenever its Student's
room leaves LIVE, by either lever, caught by one substrate trigger. With the
semantics ruled, the pin is **signed and the build bar lifts** — the build
against §0 follows: `deploy_doctor`, the store cascade, `retire_environment`, the
two substrate triggers, the `doctor_deployments` table, adversarial review before
delivery, and the ledger — closing SLICE_07's cascade debt — back to this desk.*

## 9. Delivery ledger (appended at delivery — the two-commit lifecycle)

*The spec crossed the desk before code (pinned `26c0090`, amended to the ruled
semantics `ffae6a8`); this ledger appends at delivery, DISCIPLINE.md §5.*

### 9.1 What was built, against §2

- **Migration `0019_doctor.sql`** — the `doctor_deployments` table (records
  **both** instruments: `doctor_env_ref → student_env_ref` and the
  `pairing_id`; append-only, frozen, no-delete); the **Doctor-orphan** AFTER
  trigger (`godhead_orphan_dependent_doctors`, keyed on a room's departure from
  LIVE — one wall, both levers, §0.3); the **status-arc** BEFORE trigger
  (`godhead_env_status_arc`: no `ORPHANED→LIVE`, no exit from `DISSOLVED`, and
  the human-only retire guard — a uuid `retired_by` is `GATE_BYPASS_ATTEMPT`,
  IV.4); and the A.5 taxonomy extension (`ENV_DISSOLVED`, `DOCTOR_DEPLOYED`).
- **Store** — `deploy_doctor` (LIVE-Canon-Student validation → `ENV_INVALID`;
  Canon Teacher established on the Student's matrix; the pairing formed; the
  reference recorded); `retire_environment` (the one human lever to
  `DISSOLVED`); and `execute_decommission` **extended** with the per-row
  ORPHANED cascade over the matrix's rooms.
- **Schemas** — `DoctorDeployment`; the two log events.
- **Doc 05 IV.4** — `retire_environment` added to the closed list of
  human-reserved acts (Round 8, author-sanctioned).

### 9.2 The SLICE_07 cascade debt — discharged

SLICE_07 §Non-goals deferred "the ORPHANED cascade rules … a Doctor orphaning
when its Canon Student dissolves … the cascade is not [built]," and doc 06:83's
"if its matrix is decommissioned, the environment goes ORPHANED" was specified
but never wired. **Both are now built and proven.** `execute_decommission`
orphans every room on the decommissioned matrix (one atomic act and record); the
substrate trigger carries the Doctor from its Student's departure. The
decommission leg of `sc_j08_leaving_live_orphans_doctor` drives
`execute_decommission` end-to-end and asserts both the Student room and the
Doctor go ORPHANED. The debt is discharged on the record.

### 9.3 Adversarial review before delivery (the standing rule)

A four-lens finder panel (cascade, walls, deploy, conformance) piped into
three-lens refuters (code-trace / reachability / reproduction; majority-CONFIRM
to survive) — 19 agents. **5 findings raised, 0 survived** the refuter vote. But
the review is not read by the vote alone:

- **Adopted despite 0/3 — a real coverage gap the refuters' framing missed.**
  Three independent finders (cascade, walls, conformance) flagged that
  `sc_j08_no_silent_revival` proved only leg (c)'s status-monotonicity half, not
  the "a fresh Student mints a new Doctor; the old stays orphaned" half that
  §0.5(2) **explicitly pins as behavior** ("the criterion tests it as
  behavior"). The refuter angles were code-defect-oriented and refuted it as
  "no code defect" — true, but beside the point for a spec-mandated test. The
  test was extended to prove the fresh-deploy half (new env, new reference, new
  pairing; the old Doctor untouched, still ORPHANED). This is the
  not-silently-blessed rule turned on our own tooling.
- **Two residuals, disclosed not papered over.** (1) `deploy_doctor` is not
  transactional: a mid-deploy failure between `establish` and the pairing/row
  could strand a LIVE Doctor the cascade cannot reach. Its preconditions are
  guaranteed by construction (both rooms LIVE, shared matrix, both Canon; valid
  FKs), so the only trigger is a transient DB error, and a stranded Doctor is
  itself retirable; atomicity would thread a tx through two trait methods —
  disproportionate, and consistent with the store's existing non-transactional
  multi-step methods. Left as residual. (2) `retire_environment` accepts any
  non-uuid `retired_by` (including office/blank strings) as the human witness —
  which is the **ruled** SC-A08 actor model (non-uuid actors accepted by shape,
  no roster to resolve them against yet); the IV.4 wall is the job-uuid
  rejection, which holds. No change.

### 9.4 Gate & sweep

- Producer gate (`scripts/gate_report.py`, live Railway Postgres): **PASS (3
  steps)** — fmt clean, clippy clean (0 warnings, workspace denies warnings),
  **165 passed / 0 failed / 0 ignored across 48 binaries** (GATE_REPORT.txt).
  Up from Slice 11's 161/47: the `j_doctor` binary and its four SC-J08 legs.
- Criteria sweep regenerated (`docs/dev/criteria_sweep.py`): **SC-J08 moves
  DEFERRED → PENDING**, now cited by the four `sc_j08_*` legs.

*Delivered 2026-07-10.*
