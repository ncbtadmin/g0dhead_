# Phase B — Slice 7: Scriptoria & Titles
### Pinned scope — signed off 2026-07-07

> Section G of Document 8: Laws IX–X — an environment is memory the
> memoryless are permitted. The shared floor that binds both kinds
> (TEACHER, STUDENT) before either manual applies; the ground K and L
> stand on. In-world: the Scriptorium.

## 1. Pinned criteria (Document 8, section G — 7 criteria)

| Criterion | Enforces | Seed test |
|---|---|---|
| SC-G01 | Mounting a floor-invalid environment refuses ENV_INVALID; no work occurs | `sc_g01_mount_invalid` |
| SC-G02 | Out-of-index read rejected + logged; allowlist (own job/lease, config) excepted | `sc_g02_scoping_wall` |
| SC-G03 | Pairing Exception: flagged counterpart artifact allowed; unflagged rejected; unpaired rejected for both | `sc_g03_pairing_exception` |
| SC-G04 | Conferral deterministic: `hash(env_id) mod roster_len` reproduces the name; living collision → ordinal | `sc_g04_conferral` |
| SC-G05 | Teacher env whose tier/title disagree fails; a pairing naming REGULAR fails | `sc_g05_tier_title` |
| SC-G06 | Mount walks provenance: one item whose chain fails root-to-leaf → whole room ENV_INVALID | `sc_g06_provenance_walk` |
| SC-G07 | ORPHANED env readable as archive, unmountable for work, no write succeeds | `sc_g07_orphaned` |

## 2. What this slice builds

- **Schemas (A.8, A.10)**: EnvironmentRecord (kind, matrix_ref, tier,
  established_by {title, name, job}, status `LIVE|ORPHANED|DISSOLVED`),
  PairingRecord, EnvItem (contents-index row: item_ref, provenance chain,
  `flagged` — is this a certified handoff artifact). Deterministic
  conferral helper `roster_index(env_id, len)` (a stable hash — FNV-1a
  over the uuid bytes), shared by the store and the reproducibility test.
- **Migration 0007**: environments / environment_items / pairings tables
  (no-delete triggers); seed the two conferral config constants
  `name_roster` (pseudo-Eastern-European, grey-bureaucratic) and
  `honorific_set` (teacher titles by tier; the flat student set).
- **Store methods** (the enforcement — Law IX/X live here):
  - `establish_environment(job, kind, matrix, tier)` — confers title+name
    deterministically (title from tier for Teachers, honorific by hash for
    Students; name from the roster by hash, ordinal on a living collision),
    records `established_by` immutably (X.1), status LIVE. REGULAR tier
    establishes nothing.
  - `add_env_item(job, env, item_ref, chain, flagged)` — curates the
    contents index; refused on a non-LIVE environment (SC-G07).
  - `mount_environment(job, env)` — the Law IX.3 mount: floor validation
    (record validates ∧ title↔tier agree ∧ every item resolves ∧ every
    item's provenance chain walks root-to-leaf); ENV_INVALID on any
    failure; ORPHANED/DISSOLVED are unmountable-for-work.
  - `env_scoped_read(reader_job, env, target)` — IX.4 + IX.5: allowed iff
    target ∈ the env's contents index, OR target is an allowlist item
    (reader's own job/lease), OR the Pairing Exception applies (a pairing
    binds reader_env↔counterpart and target is a *flagged* item of the
    counterpart). Otherwise rejected and logged (severity: violation).
  - `form_pairing(teacher_env, student_env, matrix, kind)` — tiers must
    match kind (DEVOUT_ASSIGNMENT→both DEVOUT, CANONICAL_INSTRUCTION→both
    CANON); REGULAR anywhere → invalid (X.5).
  - `orphan_environment(env)` — status LIVE→ORPHANED (dependency lost).
- **`crates/godhead-scriptorium`**: thin establishment/mount orchestration
  (spawn the establishing job, establish, mount) — what K/L will call —
  plus the integration tests (dev-deps intake+ml to grow a bound matrix).

## 3. Design decisions

- **Provenance floor, not the full A.6 system**: an EnvItem carries an
  inline ProvenanceChain (C.2 shape: entries `{link_seq, kind, actor,
  produced}`; root kind ∈ CANON|WRIT|BRIEF; contiguous seq; every
  `produced` ref resolves to a live record). This satisfies the mount
  walk (SC-G06, Handbook §2.2) without pulling in A.6/J/K/L.
- **`flagged` as the handoff bit**: the Pairing Exception grants *flagged*
  artifacts only (IX.5). A published Instruction/Return is `flagged:true`;
  a working draft is `false`. The richer B.6/C.1 index contents (elections,
  published, received) are the manuals' to build atop this floor.
- **Conferral title source**: Teachers deterministic by tier
  (Professor↔Devout, Doctor↔Canon); Students pick a flat honorific by the
  same hash. Names from the roster by `roster_index`; ordinal is the count
  of living environments already bearing the base name, recorded
  immutably (so SC-G04's "reproduce from the record" holds forever even as
  living state changes).
- **Environments bind to a matrix**: tests grow a Postulant the ordinary
  way (intake → embed → consolidate → emerge) and bind to it.

## 4. Non-goals

- No Teacher/Student manual behavior (K/L): no Instructions, Returns,
  elections, PromptPackages — only the floor those consume.
- No A.6 ProvenanceRecord system, no mandates (J), no Deacon (I).
- No ORPHANED cascade rules from the manuals (a Doctor orphaning when its
  Canon Student dissolves) — that is K's; the status field + unmountable
  behavior is built, the cascade is not.

## 5. Edge cases

- Mount an env with one dangling item ref → ENV_INVALID (whole room).
- Provenance chain with a non-CANON/WRIT/BRIEF root → ENV_INVALID.
- Pairing read of a flagged vs unflagged counterpart item.
- Conferral collision: two living envs hashing to the same roster entry —
  the second takes the ordinal, recorded immutably.
- Write (add_env_item) against an ORPHANED env → rejected.

## 6. Delivery — gate passed 2026-07-07

All 7 criteria green + 4 review regression tests (`tests/g_scriptoria.rs`,
11 tests); slices 1–6 unregressed — 82 tests workspace-wide; fmt/clippy/
test clean. Migrations 0007 + 0008 applied to Railway.

**Adversarial review (15 agents, 3 lenses × verify, one lens dedicated to
scoping security): 12 findings — 3 refuted, 9 confirmed, all fixed.** The
two HIGH findings shared a root cause and were the important catch:

1. *Binding was forgeable* (HIGH×2) — `env_scoped_read` trusted the job's
   `env_ref`, but `create_job` wrote that caller-set field unvalidated, so
   any actor could bind a job to *any* environment and read its whole
   index (unflagged drafts included), and reach a paired counterpart's
   flagged artifacts with no pairing of its own. Root fix: **binding is now
   authenticated at `create_job`** — an `env_ref` must reference a LIVE
   environment whose tier matches the job's and whose matrix the job is
   actually working (in `input_refs`). An agent cannot enter a room of
   another tier or matrix by naming it. Regression:
   `env_ref_binding_is_authenticated`.
2. *Provenance root by array position* (HIGH) — the human-root check keyed
   on `i == 0`, so an out-of-order chain slipped a non-human root past the
   floor. Now keyed on the minimum `link_seq`; each entry must also carry
   an `actor` (C.2 shape).
3. *Ordinal over-count / reuse* (MEDIUM×2) — `name LIKE 'base%'`
   over-counted prefixes (Solo vs Solomon) and, counting only LIVE
   bearers, could reuse an ordinal after an orphan. Now: exact-base or
   `base <ord>` with a space delimiter, counted over all environments ever
   (records are never deleted), so ordinals are monotonic and never reused.
4. *Pairing not matrix-scoped* (MEDIUM) — `form_pairing` never checked the
   two rooms share the pairing's matrix; now both rooms and the pairing
   must name one matrix, and both must be LIVE (IX.5, X.5).
5. *Conferral mutable* (LOW) — migration 0008 adds a positive immutability
   trigger: an established environment's identity fields (kind, matrix,
   tier, title, name, establisher) are frozen; only status/revision change.

Note: cross-tenant isolation beyond tier+matrix is the deferred
multi-tenancy work (doc 3 §4.5); the floor authenticates binding as far as
the single-operator model has relationships to check.
