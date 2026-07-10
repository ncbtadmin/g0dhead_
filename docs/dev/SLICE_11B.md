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
- **ORPHANED** — *a bond severed.* A room something it depended on went away, but
  the room and its contents persist as a **read-only archive** (SC-G07: readable,
  unmountable-for-work, no write succeeds). The Canon Student's corpus and the
  Doctor's crafted packages are too valuable to strike; they are kept.
- **DISSOLVED** — *the ground struck.* A room whose own basis — its matrix — is
  gone. It cannot stand.

The load-bearing pin: **ORPHANED is what happens to a room whose *dependency*
dissolves; DISSOLVED is what happens to a room whose *own matrix* decommissions.**
A Doctor loses its source (the Canon Student) → ORPHANED. A Canon Student loses
its ground (the matrix) → DISSOLVED. This is exactly the two-word split in
§4.3 — "the Canon Student environment **dissolves**, the Doctor's goes
**ORPHANED**" — made mechanical.

### 0.2 What lawfully dissolves a Student scriptorium

The one lawful root is **matrix decommission**: `consent_decommission` →
`execute_decommission` (Notary-executed, consent-gated, Law VI.5), which today
sets `matrices.status = 'DISSOLVED'` and **stops there** — it does not touch the
environments standing on that matrix. That stopping point *is* the SLICE_07
deferred cascade.

This slice extends it: **decommissioning a matrix DISSOLVES the scriptoria that
stand on it** (an environment cannot outlive the matrix that is its ground —
`environments.matrix_ref` → the dissolved matrix ⇒ `DISSOLVED`). A Canon Student
scriptorium therefore dissolves precisely when its matrix is decommissioned —
the documented trigger ("a Devout environment whose matrix decommissions", doc 05
round-3 note 2), now a built cascade rather than a manual's promise. There is no
*direct* environment-dissolve act in v1; the matrix is the only lever, and it is
already consent-gated, so dissolution inherits that human gate for free.

### 0.3 Where the cascade is enforced — substrate vs store

Two mechanisms, split by what each guarantees:

- **The dissolution cascade lives in the STORE** (`execute_decommission`
  extended, in its existing transaction): matrix→DISSOLVED, then the environments
  on it → DISSOLVED, then the Doctors depending on a now-dissolved Canon Student
  → ORPHANED — one atomic act, one record. Decommission is a consent-gated,
  Notary-only store path with no lawful raw-SQL route (ruling G6), so the store
  is the right enforcement point, and it is where `orphan_environment` already
  lives. Driving the cascade from a trigger on `matrices` UPDATE is the rejected
  alternative: it would fire on any matrix write and split the dissolution record
  across trigger and caller.
- **The no-revival wall lives at the SUBSTRATE** (§0.5): "read-only archive, no
  resurrection" is an *integrity* guarantee that must hold against every writer,
  not a courtesy of the decommission path — so it is a trigger, like V.4 and the
  preservation walls, not store logic a future path could forget.

### 0.4 The Doctor's binding, and why it ORPHANS rather than co-dissolves

The Doctor (a **Canon Teacher**, §4.3) depends on the Canon Student's corpus.
The binding is a **deployment reference** — `doctor_env_ref → student_env_ref` —
**not** a shared-matrix `form_pairing`. This is the decision the cascade turns
on, and it is pinned deliberately: were the Doctor bound by `form_pairing` (which
requires the Doctor and Student to share one matrix), decommissioning that matrix
would DISSOLVE the Doctor alongside the Student (§0.2), and it could never reach
the ORPHANED state SC-J08 demands. Binding by reference lets the Doctor stand on
its own while depending on the Student's room: when that room DISSOLVES, the
cascade walks the references and sets each dependent Doctor **ORPHANED** — its
source gone, its packages preserved. (The §5.2 "Canonical Instruction (Doctor +
Canon Student)" bond is that reference, realized; it is a logical pairing, not a
matrix co-tenancy.)

### 0.5 The no-revival wall

Two guarantees, both substrate:

1. **Status never regresses.** A trigger on `environments` forbids
   `ORPHANED → LIVE` and any transition out of `DISSOLVED`. Orphaned is terminal
   for work; the archive is never reanimated in place. (LIVE→ORPHANED and
   LIVE/ORPHANED→DISSOLVED remain lawful — the arc only ever descends.)
2. **A fresh Student does not adopt the old Doctor.** Deploying against a new
   Canon Student environment mints a **new** Doctor env and a **new** deployment
   reference; the orphaned Doctor is untouched and stays orphaned. Revival is a
   fresh sovereign act — a new deployment — never a status flip on the old. This
   falls out of (1) plus deploy always minting fresh, but the criterion tests it
   as behavior.

### 0.6 The debt this closes, by name

SLICE_07 §Non-goals: *"No ORPHANED cascade rules from the manuals (a Doctor
orphaning when its Canon Student dissolves) — that is K's; the status field + the
unmountable behavior is built, the cascade is not."* And doc 05 round-3 note 2
added the A.8 status field "open for ratification … the type-specific behavior
lives in the manuals." Slice 11b builds that cascade and that type-specific
behavior. On delivery this debt is closed **on the record**, not silently: the §9
ledger names SLICE_07's deferral and marks it discharged.

## 1. Pinned criteria — SC-J08, home, a leg at a time

| Criterion (leg) | Enforces | Seed test |
|---|---|---|
| SC-J08 (a) — **deploy requires LIVE** | `deploy_doctor` against a `student_env_ref` that is not a LIVE Canon Student refuses `ENV_INVALID`; a LIVE Canon Student deploys (establishes the Doctor's Canon Teacher env + the deployment reference). Student first, always. | `sc_j08_deploy_requires_live` |
| SC-J08 (b) — **dissolve orphans through the pairing** | Decommissioning the Canon Student's matrix DISSOLVES the Student scriptorium (§0.2) and, through the deployment reference, ORPHANS the Doctor — one atomic act; the Doctor is thereafter a read-only archive (SC-G07). | `sc_j08_dissolve_orphans_doctor` |
| SC-J08 (c) — **no silent revival** | An orphaned Doctor is never returned to LIVE (the substrate status-regression wall); a fresh Canon Student environment does not adopt it — a new deployment mints a new Doctor, the old stays orphaned. | `sc_j08_no_silent_revival` |

## 2. What this slice builds

- **`deploy_doctor`** (store method, or a thin scriptorium orchestration over
  store walls): validates `student_env_ref` is a LIVE Canon Student (`ENV_INVALID`
  else, SC-J08a), establishes the Doctor's Canon Teacher environment (reusing the
  establish path), and records the deployment reference `doctor_env_ref →
  student_env_ref`. The Doctor holds office as a Canon Teacher; its PromptPackage
  labor (B.5) is **not** built here (non-goal) — only deployment + lifecycle.
- **The dissolution cascade** in `execute_decommission` (extended, same
  transaction): matrix DISSOLVED → its environments DISSOLVED → Doctors whose
  `student_env_ref` names a now-DISSOLVED Canon Student → ORPHANED. One record.
- **A `dissolve_environment` store step** (LIVE|ORPHANED → DISSOLVED) the cascade
  calls, mirroring `orphan_environment`; and the Doctor-orphan step over the
  deployment references.
- **The no-revival substrate wall** (migration): a trigger on `environments`
  forbidding status regression (`ORPHANED→LIVE`, out-of-`DISSOLVED`), and the
  `doctor_deployments` table (`doctor_env_ref`, `student_env_ref`, append-only,
  frozen) the cascade walks.
- **Migration(s)**: `doctor_deployments` + the status-monotonicity trigger.

## 3. Non-goals

- **No PromptPackage (B.5), no Doctor *labor*** — this slice builds the Doctor's
  deployment and lifecycle (deploy → orphan → no-revival), not what it crafts.
  The package labor is a later slice.
- **No direct environment-dissolve act** — the matrix is the only lawful lever
  in v1 (§0.2).
- **No retrieval breadth, no real transport** — unchanged; the no-HTTP wall
  stands (it is Phase 5's to delete).
- **No Devout-environment orphaning behavior beyond the shared cascade** — the
  matrix→env dissolution is general, but Devout-specific downstream behavior is
  not in scope.

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

*Presented to the sovereign 2026-07-10. Sections 0–4 pin the dissolution
semantics and SC-J08's three legs; the build bar lifts only on sign-off — the
spec crosses the desk before code. The one design decision the desk should weigh
is §0.4 (the Doctor bound by deployment reference, not shared-matrix pairing, so
it can orphan rather than co-dissolve); the rest argues the cascade and the
no-revival wall from what the substrate already holds.*
