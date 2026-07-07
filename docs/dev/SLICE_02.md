# Phase B — Slice 2: Intake & Endurance
### Pinned scope — signed off 2026-07-07

> Section N of Document 8: the deterministic onboard pipe (doc 2) and the
> endurance guarantees (doc 3 §4), standing on the slice-1 substrate.
> Nothing is built here that does not resolve to SC-N01–N06.

---

## 1. Problem statement

Carry a committed file deterministically from the human commit to *at rest in
the GodHead with baseline structure* — raw copied exactly once, first log on
copy, normalization to clean UTF-8, floor classification — and prove the
endurance guarantees: the seam holds, every stage boundary survives
kill-and-restart, and derivatives are disposable and regenerable.

## 2. Pinned criteria (Document 8, section N — 6 criteria)

| Criterion | Enforces | Seed test |
|---|---|---|
| SC-N01 | Raw copied exactly once; checksum stable across the whole cycle | `sc_n01_raw_copied_exactly_once` |
| SC-N02 | First log snapshot on raw copy, required fields; rotation preserves priors | `sc_n02_first_log_and_rotation_chain` |
| SC-N03 | Decode failure logged+flagged, stored; unsupported type stored + notice, never rejected | `sc_n03_decode_failure_and_unsupported` |
| SC-N04 | The seam: after at-rest, zero dispatches absent human invocation/trigger | `sc_n04_seam_holds` |
| SC-N05 | Kill-and-restart at every stage boundary; supervisor reconstructs; end state = uninterrupted run | `sc_n05_kill_and_restart_each_boundary` |
| SC-N06 | Derivative discarded and re-derived from raw; provenance updates, nothing else changes | `sc_n06_derivative_regeneration` |

## 3. What this slice builds

- **NodeRecord** (doc 3 §2.1) in `godhead-schemas`: raw by reference
  (relative path under a raw root + SHA-256), derivative by reference,
  `normalized` state, `intake_status` (closed: `RAW | NORMALIZED |
  DECODE_FAILED | UNSUPPORTED`), low-trust floor classification. Raw fields
  are settable only at creation — no store method takes them afterward
  (raw-copied-once, structurally).
- **Store node methods** on the `Store` trait: `create_node`, `get_node`,
  `set_node_derivative`, `set_node_classification` — all identity-guarded,
  leased, CAS-revisioned like every mutable subject. Migration
  `0002_nodes.sql` adds the table and admits `INTAKE` to the JobRecord
  agent-type enum (v1 addition, same justification as `JOB_TRANSITION`:
  intake floor labor needs JobRecords for idempotency and recovery, and the
  A.2 roster names only true agents).
- **`crates/godhead-intake`** — the deterministic pipe:
  - `normalize`: strict decode to clean UTF-8 (UTF-8, UTF-16 LE/BE by BOM;
    line endings → LF). No lossy decode: a faulty normalization is surfaced,
    never buried.
  - `classify`: hardcoded filetype→bucket floor (`database`, `programming`,
    `markup`, `document`, `unclassified`), all marked low-trust/overridable.
  - `pipe`: the stage jobs — RAW_COPY (human commit), NORMALIZE, CLASSIFY —
    each a full slice-1 lifecycle (job → lease → work → artifact → flag →
    terminate), handing off through readiness flags only.
  - `dispatch`: the thin dispatcher — `tick()` consumes ACTIVE intake flags
    and runs exactly the mapped successor stage. Its successor map ends at
    CLASSIFY: the seam is the absence of any rule beyond at-rest.
  - `supervise`: the observing supervisor — `reconstruct()` rebuilds the
    where-is-everything index purely from flags and job records.

## 4. Constraints & decisions

- **Initial supported set finalized** (resolves doc 2 §2.4 `[Fable]`):
  text-native and near-text types that decode without interpretation —
  txt, md, json, py, html/htm, css, js/mjs/ts, rs, c/h, cpp/hpp, java, sh,
  ps1, toml, yaml/yml, xml, csv, tsv, log, ini, cfg, sql. **Text-layer PDFs
  and RTF are deferred within the slice** (both need parsing beyond a
  decode; they join the ladder later without schema change). Everything
  else: stored raw, `UNSUPPORTED`, incompatibility notice — never rejected.
- **Raw root is a constructor parameter** of the intake pipe (deployment
  concern); tests use per-run temp directories.
- **The dispatcher runs stages inline and deterministically** in v1 — no
  background workers. Intelligence lives in the state (flags), not the
  dispatcher; concurrency arrives when a real workload demands it.
- Classification uses filetype only in v1; doc 2 §2.5's "trivial content
  signals" are an optional floor refinement, not built yet.
- Whitespace standardization v1 = line endings (CRLF/CR → LF) + BOM strip;
  deeper canonicalization (Unicode NFC) deliberately deferred.

## 5. Non-goals

- No embeddings/links at intake (SC-M06's embedder-at-intake behavior is
  section M's slice; the node schema leaves room, nothing more).
- No Deacon, no quarantine, no agents, no environments, no weights.
- No standing triggers or scheduling — `tick()` is invoked by tests/humans;
  trigger configuration is a later slice's surface.
- No frontend, no upload transport — `commit_file` takes bytes in-process.

## 6. Edge cases

- Invalid UTF-8 in a supported type (DECODE_FAILED, raw preserved).
- Unsupported binary type (stored, noticed, never rejected).
- Kill after each flag: RAW_COPY boundary, NORMALIZE boundary.
- Renormalize after rest: raw checksum stable, classification untouched.
- Re-run of a stage job converges (inherits SC-A03 machinery).

## 7. Delivery — gate passed 2026-07-07

All 6 criteria green (`tests/n_intake.rs`), slice-1 suite unregressed —
31 tests total against live Railway Postgres; fmt/clippy/test all clean.
Implementation notes vs. the design:

- Migration `0002_nodes.sql` additionally guards the atom at the substrate:
  a trigger rejects any rewrite of a node's raw reference fields, below
  even the store API (SC-N01 hardened).
- `Dispatcher::tick` gained a scoped variant (`tick_scoped`): the successor
  logic is identical, but a tick can be limited to a node set — needed for
  multi-root deployments and for test isolation on a shared database. The
  unscoped `tick()` remains the production mode.
- Normalization verifies the atom's checksum before every decode
  (derivative-to-source integrity, doc 3 §4.3); a mismatch is a Law VII
  refusal, never a silent re-derive.
- The renormalize labor flags `intake:renormalize`; the dispatcher maps no
  successor for it — regeneration ends where it starts, on the record.
