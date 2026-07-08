# Phase B — Slice 9: Student Returns & Stewardship
### Pinned scope — signed off 2026-07-08

> Section L of Document 8: the Student Handbook's spine. Everything a
> Student is *for* ends in a ReturnManifest (B.2). Stands on slice 8's
> Instructions (the ReturnManifest answers one) and slices 3–4's overrides.

## 1. Pinned criteria (Document 8, section L — 4 criteria)

| Criterion | Enforces | Seed test |
|---|---|---|
| SC-L01 | ReturnManifest completion: exactly one entry per acceptance criterion (missing/extra invalidate); evidence mandatory for passed, failed, deferred | `sc_l01_completion_contract` |
| SC-L02 | Devout consolidation over human-held data leaves it untouched (structural diff); may petition, never write | `sc_l02_human_held_untouched` |
| SC-L03 | Redundant-consistency: re-running a refinement from its derivation reproduces the artifact; a dangling intra-scriptorium ref fails closure | `sc_l03_redundant_consistency` |
| SC-L04 | The unbound are unnamed: a conferral (environment) for a REGULAR tier fails validation | `sc_l04_unbound_unnamed` |

## 2. What this slice builds

- **Schemas (B.2)**: ReturnManifest (return_id, instruction_ref,
  student_env_ref, concordat_version, items, completion), ReturnItem
  (`kind: REFINED_DOC | CORPUS_ITEM | ORGANIZATION_CHANGE`, provenance),
  CompletionEntry (`criterion_index`, `passed: Option<bool>`,
  `evidence_ref`), RefinedArtifact (source_refs + method + content_sha —
  the derivation).
- **Migration 0010**: `returns`, `refined_artifacts` (derivation +
  content_sha). Add v1 log events.
- **Store methods**: `persist_return` (the completion contract validation —
  exactly one entry per criterion, evidence mandatory, `passed: null` iff
  the criterion is SOVEREIGN_JUDGMENT), `get_return`,
  `persist_refined_artifact` (records derivation + content_sha),
  `get_refined_artifact`, `refined_artifacts_in` (for the closure walk).
- **`crates/godhead-student`**:
  - `refine` — a deterministic refinement over source nodes (content is a
    stable function of the sources' derivative checksums; method named),
    persisting the artifact + derivation. `re_derive` recomputes from the
    stored derivation.
  - `redundant_consistency` — the three mechanical properties (Handbook
    §1.2): (a) conformance (each refined artifact's shape validates),
    (b) re-derivability (re_derive reproduces content_sha), (c) closure
    (no source_ref inside the scriptorium dangles).
  - `write_return` — the Student's VALIDATE_OUT: validate the completion
    contract against the answered Instruction, persist, flag — or Law VII
    refusal (a Return that does not validate never flags, never poisons).
  - `steward_consolidate` — a Devout Student consolidation that treats
    `user_overridden` state as fixed stars: works around, petitions when
    wrong, never through (Handbook §4.5). Reuses the store's override
    enforcement (slices 3–4).

## 3. Design decisions

- **`criterion_index` binds completion to the Instruction's ordered
  acceptance_criteria** (B.1). The contract is a bijection: indices
  `0..n` each appear exactly once; a `passed: null` entry is legal iff
  that criterion's `testable_as` is SOVEREIGN_JUDGMENT (§1.3d, mirrored
  from the Teacher side). Evidence (`evidence_ref`, non-nil) is mandatory
  in every case, including the deferred one.
- **Refinement is deterministic over store state** (no disk dependency):
  content_sha = SHA-256 of the sorted source derivative-checksums under
  the named method. Re-derivability is then a pure recomputation — the
  strongest honest re-derivability an ephemeral steward can offer.
- **Closure is scriptorium-scoped**: every refined artifact's source_refs
  and every env item resolve; a dangling one is debris (Handbook §1.2c).
- **SC-L02/SC-L04 largely exercise existing enforcement** from the
  Student's vantage (override rejection from slices 3–4; the REGULAR-
  establishes-nothing rule and the tier CHECK from slice 7). This slice
  wires the Student path and pins the criteria; it does not re-implement
  the walls.

## 4. Non-goals

- No Canon Student CorpusManifest / coverage map (that rides with J's
  canon loop), no fetch (J), no Deacon (I).
- No full stewardship verb set — REFINE + CONSOLIDATE are built (SC-L02/
  L03); ORGANIZE/LINK_PROPOSE/VERIFY are the manuals' later depth.
- No reasoner — refinement is a deterministic floor.

## 5. Edge cases

- Return with a missing criterion entry, an extra entry, a duplicate
  index → invalid.
- A non-SOVEREIGN criterion with `passed: null` → invalid; a SOVEREIGN
  one with `passed: Some` → invalid.
- An entry with a nil evidence_ref → invalid (evidence mandatory).
- Re-derive after a source node's derivative is unchanged → same sha;
  a dangling source_ref → closure fails.
- steward_consolidate over an overridden link/classification → untouched;
  a petition is opened.

## 6. Delivery — gate passed 2026-07-08

All 4 criteria green + 7 review/hardening regression tests
(`tests/l_student.rs`, 11 tests); slices 1–8 unregressed; fmt/clippy/test
clean. Migrations 0010 and 0011 applied to Railway. New crate
`godhead-student` (11th member).

Post-review hardening on the labor itself: `write_return` splits the
labor from the refusal discipline — every halt after RUNNING (validation,
a store wall like the pairing check, a lost connection) ends in
`store.refuse`, no job strands live, and a failed refusal write is a hard
error, never swallowed. The persisted refusal detail names a stable
clause/stage token only — never the draft's own text (Law XV; the slice-6
doctrine). validate_return also proves the cited Concordat version is
ADOPTED (SC-K03's retrievability covenant), and steward_consolidate
re-reads on a StaleRevision CAS loss instead of dropping its report.

**Adversarial review (106 agents: 7 finder lenses, every deduped finding
judged by a 3-lens refutation panel — factual, spec-intent, impact/repro):
34 raw findings → 27 confirmed (8 distinct defects), 6 refuted, all
fixed.** The slice-8 lessons cut both ways again — walls missing AND
over-refusal:

1. *The Return path skipped the A.8 Live wall* (HIGH) — persist_return and
   validate_return checked room kind but never status, while
   persist_refined_artifact in the same diff refused archived rooms. A
   certified Return could rise from an ORPHANED/DISSOLVED room. Both ends
   now refuse. Regression: `return_walls_live_bound_paired_tiered`.
2. *No job-to-room binding* (HIGH) — any Running Student job could persist
   a Return or drop permanent refinement debris into any room by naming
   it, against the env_scoped_read principle. persist_return and
   persist_refined_artifact now require the Law IX.4 binding create_job
   authenticates (`job.env_ref == the room`); `write_return` births its
   job bound. Same regression.
3. *Cross-tier / cross-pairing answering* (HIGH) — a Return from a room of
   the wrong tier, or unpaired with the Instruction's teacher room, was
   admitted. Now: `env.tier == instruction.target_tier` proven at both
   ends; a pairing row proven at the store wall (a Regular Teacher has no
   room, so nothing to pair). Same regression.
4. *The walk crashed instead of naming debris* (HIGH) — a source_ref that
   resolves as a non-node (link, another artifact — records the store
   admits by design) made `fold_sources` propagate a raw NotFound,
   aborting the whole walk. It now maps to NOT_REFINABLE → property (b)
   debris. Regression: `walk_names_debris_never_faults`.
5. *Two contradictory closure definitions* (HIGH, the over-refusal twin) —
   the walk's `resolves()` branded job-artifact env items (the sanctioned
   publication vehicle) as dangling, while slice-7's mount refused rooms
   electing slice-9's own products. Converged: the walk resolves seven
   kinds (nodes, links, matrices, environments, refined artifacts,
   Returns, jobs); the mount learned refined_artifacts and returns.
   Regression: `walk_covers_conformance_and_the_room` (walk AND mount
   bless the same room).
6. *A walk aimed at nothing certified it* (MEDIUM) — a nonexistent or
   Teacher env_ref returned a clean bill (bare SELECTs return empty). The
   walk now proves its room resolves and is a Student's before certifying
   anything. Same regression.
7. *The items half escaped both validators* (MEDIUM) — nil
   item_ref/provenance_ref persisted into certified manifests while nil
   evidence was refused. Both ends now enforce the nil floor on items;
   full item resolution is pinned as the Deacon's threshold (section I).
   Regression in `sc_l01_completion_contract`.
8. *Exact concordat-version equality was over-refusal* (MEDIUM) — the
   pinned rule is §2.4's range; a Student honestly citing an adopted
   additive minor was refused. Equality dropped, range stands.
   Regression: `validate_gates_beyond_the_contract`.

Test-adequacy fixes: the fixture's two source files now carry different
bytes (the canonical-fold and source-identity assertions can actually
fail); a substrate probe freezes a flagged Return and rejects DELETE
(§3.1, mirroring SC-K05); property (a) conformance and room-item closure
are each driven to fire; the VALIDATE_OUT gates beyond the completion
contract are each exercised. Migration 0011 adds the walk's access-path
indexes (LOW). Refuted (6): store-side version-check scope is pinned to
the completion contract; steward classification shape, partial-report
drop, error-variant asserts, and flag-fallback coverage all match house
precedent or lack a reachable failure.

Known debt (pre-existing, not this slice): `sc_m01_triggers` and
`sc_k07_pattern_escalation` can flake under parallel threads on the
shared live DB (global rebalance/bias singletons + remote latency
variance); both pass in isolation and single-threaded.

## 7. Review round 2 — pre-commit (2026-07-08)

A second adversarial pass (2 agents: store surface, crate logic) before
the commit. 16 raw findings → 8 fixed, 8 triaged to house precedent or
the round-1 refutations. Fixed:

1. *flag_return was an open door* (HIGH) — the only gate was the
   terminal-status guard: any live job of any type could certify any
   Return, and a replay appended a fresh RETURN_FLAGGED event attributed
   to a job that never flagged anything (the log then testified N
   certifications of one act). Now: RUNNING + Student + bound to the
   Return's own room (the three walls persist_return holds), and the
   already-flagged path is an idempotent read-back that logs nothing.
   Regression: `certification_walls_and_single_flag_event`.
2. *Mid-labor halts stranded the job* (HIGH) — every `?` in write_return
   after RUNNING returned with the job live and no refusal; the unpaired-
   room case made this MAINLINE (VALIDATE_OUT deliberately leaves X.5 to
   the store's wall, so the wall's rejection stranded the laborer). All
   labor now runs under a halt handler: any failure after RUNNING ends in
   store.refuse (the established labor rule). Regression:
   `mid_labor_halt_refuses_never_strands`.
3. *A hostile draft could suppress its own refusal* (HIGH) — the refusal
   detail echoed the failure text, which carries the draft's
   concordat_version; a secret-shaped semver prerelease (valid semver,
   always outside `^1.0`) made the Law XV scan reject the refusal write —
   which `let _ =` then swallowed (both halves against the slice-6
   doctrine). Now: ReturnFailure carries a stable `clause` token; the
   persisted detail names clause/stage and law only; refuse() errors
   propagate. Regression: `refusal_never_echoes_the_draft`.
4. *A certified Return could cite a never-adopted Concordat* (MEDIUM) —
   the range check passed 1.999.0 without proving adoption, breaking
   SC-K03's retrievability covenant at the citing end (and the doc
   comment falsely claimed the citation "matches the Instruction's" — it
   never did, by design; comment fixed). validate_return now resolves the
   citation via get_concordat. Same regression.
5. *Flagged-Return immutability covered six columns* (MEDIUM) — the
   provenance envelope (produced_by, produced_at, schema_*) stayed
   rewritable: who certified, and when, could be falsified at the
   substrate. Migration 0012 freezes the flagged record whole.
6. *Derivation records had no UPDATE wall at all* (MEDIUM) — 0010 forbade
   DELETE so the walk could find wrong derivations, but a coherent
   REWRITE (source_refs + content_sha together) would falsify derivation
   history undetectably — the one corruption the walk cannot catch.
   Migration 0012 freezes them at birth; the tests' corruption probes now
   plant out-of-band rows instead of rewriting honest ones.
7. *A racing override aborted the consolidation* (MEDIUM) — the sovereign's
   hand landing between the steward's read and its CAS write surfaces as
   StaleRevision (the override bumps the revision), not OverrideConflict;
   the steward then dropped the whole report after partial writes,
   against its own contract. It now re-reads and re-checks (bounded),
   petitioning the star it finds.
8. *method was free text into the append-only log* (LOW) + *the walk paid
   7 round-trips per ref* (LOW) — method is now a ≤64-char
   `[a-z0-9@._-]` token (agent-shaped prose never reaches the log);
   `resolves()` short-circuits on first hit.

Triaged, not fixed (house precedent, deliberate): log-after-commit
ordering matches every prior slice (enact_refusal commits, then logs);
the check-then-act persist shape matches the workspace's job-facing
writes (the atomic check-and-claim rule covers human acts); mount item
resolution stays ownership-blind like the rest of its resolution set
(round-1 fix #5's convergence); multiple Returns per instruction is the
pinned correction path; a halt AFTER flag_return leaves the Return
certified — it passed every wall, and correction is a fresh Return.
