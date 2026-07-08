# Phase B — Slice 8: The Teacher's Lint & the Concordat
### Pinned scope — signed off 2026-07-07

> Section K of Document 8: the Holy Standard's core — the Instruction as a
> rigidly standardized handoff, the Executability Lint as the Teacher's
> mandatory VALIDATE_OUT, the Concordat as the versioned Teacher↔Student
> contract, and the Bias Doctrine with teeth. Stands on slice 7's
> environments.

## 1. Pinned criteria (Document 8, section K — 7 criteria)

| Criterion | Enforces | Seed test |
|---|---|---|
| SC-K01 | Each lint clause (a)–(f) has a violating fixture that blocks the write; a conforming Instruction passes (7 tests) | `sc_k01_lint_clauses` |
| SC-K02 | SOVEREIGN_JUDGMENT excluded from self-check, Return entry `passed: null` + evidence; zero machine-checkable criteria fails lint | `sc_k02_sovereign_judgment` |
| SC-K03 | Concordat version skew (either direction) → SCHEMA_MISMATCH at the skewed end; every cited version retrievable forever | `sc_k03_version_skew` |
| SC-K04 | Double-validation: an Instruction corrupted between flag and read is caught by the Student's VALIDATE_IN | `sc_k04_double_validation` |
| SC-K05 | A flagged Instruction is immutable; edits rejected; correction flows through `supersedes_ref` and the chain resolves | `sc_k05_immutable_supersede` |
| SC-K06 | A Regular Teacher output missing `sources_drawn` fails; `skew` computes against draws and `bias_skew_threshold` | `sc_k06_bias_disclosure` |
| SC-K07 | Pattern escalation crossing `bias_pattern_threshold` over the window raises one warning; acknowledge keeps counting; silence suppresses | `sc_k07_pattern_escalation` |

## 2. What this slice builds

- **Schemas (B.1, B.4)**: InstructionRecord (teacher_env_ref, tiers,
  concordat_version, objective, steps `[{step_id, action, params,
  expected_output, budget_hint}]`, acceptance_criteria `[{criterion,
  testable_as}]`, sources_drawn iff REGULAR, skew, supersedes_ref),
  ConcordatArtifact (version, schema refs, capability_tables,
  pairing_semantics). Closed enums: capability actions (B.3), `TestableAs`
  (`Validation(id) | SovereignJudgment`).
- **Migration 0009**: `concordat_artifacts` (version PK — every version
  retained forever, no-delete), `instructions` (flagged-immutable via
  trigger; supersedes chain), `regular_outputs` (bias disclosure rows),
  `bias_warnings` (one standing warning per scope). Seed **Concordat
  v1.0.0** with the B.3 capability tables; seed
  `instruction_budget_ceiling` (operational). Add v1 log events.
- **Store methods**: `adopt_concordat` (human — sovereign tier per A.14
  (b)), `get_concordat(version)` (any cited version, forever),
  `persist_instruction` (B.1 shape validation + supersedes-chain +
  immutability), `get_instruction`, `record_regular_output` (skew
  computed), `bias_pattern_state`, `raise_bias_warning`,
  `resolve_bias_warning` (acknowledge/silence).
- **`crates/godhead-concordat`**:
  - `lint` — the six-clause Executability Lint (§1.3), each a mechanical
    check against store + Concordat:
    (a) Resolution: teacher_env_ref and every step-param ref resolve.
    (b) Capability: each `step.action` ∈ the Concordat capability table
        for `target_tier`.
    (c) Closure: each step declares `expected_output`; each `consumes`
        references only prior step_ids.
    (d) Checkability: acceptance_criteria present; each `testable_as` is a
        validation_id or SOVEREIGN_JUDGMENT; ≥1 machine-checkable.
    (e) Budget: Σ step budget_hint ≤ the tier ceiling (A.14).
    (f) Sovereignty: no fetch action (v1 bar) and no human-reserved action.
  - `write_instruction` — Teacher's VALIDATE_OUT: lint → persist → flag,
    or Law VII refusal (nothing written).
  - `read_instruction` — Student's VALIDATE_IN: Concordat-version skew
    check (SCHEMA_MISMATCH), then re-prove B.1 shape (double-validation
    catches out-of-band corruption → FLAG_UNTRUSTED).
  - `bias` — the doctrine: sources_drawn required for Regular outputs;
    skew per `bias_skew_threshold`; pattern escalation over
    `bias_pattern_window`/`bias_pattern_threshold` with the
    acknowledge/silence terminal option.

## 3. Design decisions

- **Concordat capability tables are the source of truth for clause (b)**;
  the day-one Concordat v1.0.0 seeds B.3. Clause (f) additionally bars
  fetch actions regardless (v1 — Instructions carry no outward act).
- **Version skew is range-based** (§2.4/§3.4): a Student declares a
  supported Concordat semver range (`manual_version`-style); an Instruction
  whose `concordat_version` is out of range → SCHEMA_MISMATCH at the
  Student's VALIDATE_IN. Additive = minor, breaking = major.
- **Immutability by substrate trigger**: a flagged Instruction's body is
  frozen; a correction is a new Instruction with `supersedes_ref` (§1.4).
- **Bias scope is a single v1 scope** (`regular_teacher_bias`): one
  standing warning; the graduated-legibility machinery mirrors weight
  drift (ML §6.2) and the petition terminal answers.
- **`record_regular_output` is the escalation driver**: each Regular
  Teacher output is disclosed; after recording, the trailing-window skew
  share is evaluated and the standing warning raised/kept.

## 4. Non-goals

- No Canon Teacher PromptPackage (B.5) / Doctor labor — that is K's Canon
  half; this slice builds the Instruction + lint + Concordat + bias, which
  SC-K01..K07 cover.
- No Student Return execution (L) — `read_instruction` is the Student's
  VALIDATE_IN only; the ReturnManifest is section L.
- No fetch/mandate surface (J), no Deacon (I).
- No live reasoner — the lint and bias are deterministic floor mechanics.

## 5. Edge cases

- Lint clause (d): an Instruction whose every criterion is
  SOVEREIGN_JUDGMENT fails (no machine-checkable floor).
- Version skew both directions (Instruction newer than Student, and older
  than the Student's floor).
- Corruption between flag and read → the Student's re-validation catches
  it, not the flag.
- Supersede chain resolves; the superseded Instruction stays readable
  (what version-one read must remain provable).
- Bias: acknowledge then keep crossing → the warning keeps counting;
  silence → suppressed, not re-raised until lifted.

## 6. Delivery — gate passed 2026-07-07

All 7 criteria green + 4 review regression tests (`tests/k_concordat.rs`,
11 tests); slices 1–7 unregressed — 92 tests workspace-wide; fmt/clippy/
test clean. Migration 0009 applied to Railway.

**Adversarial review (13 agents, 3 lenses × verify, one lens dedicated to
lint completeness): 10 findings — 2 refuted, 8 confirmed (5 distinct), all
fixed.** The lint-completeness lens earned its keep:

1. *Silent clause skip on non-array shape* (MEDIUM) — a step whose `refs`
   (or `consumes`) was a bare string instead of an array skipped clause
   (a)/(c) entirely, so a dangling reference passed. Now a present-but-
   non-array `refs`/`consumes` fails its clause. Regression:
   `non_array_refs_fails_the_clause`.
2. *Clause (a) omitted link resolution* (MEDIUM) — `sqlx_resolves` checked
   node/matrix/environment but not links, over-refusing a legitimate
   link-referencing step. Added `get_link` to the Store trait and a
   NotFound-vs-fault discriminator (`exists`) so a real store fault
   surfaces instead of reading as unresolved. Regression:
   `link_ref_resolves`.
3. *Clause (d) accepted an empty validation id* (MEDIUM) — the
   machine-checkable floor tested only the enum variant, so
   `Validation("")` satisfied it. Now a machine-checkable criterion must
   name a non-empty validation, and each criterion must say something.
   Regression: `empty_validation_id_fails_clause_d`.
4. *`skew` was caller-trusted, not derived* (MEDIUM/spec) — B.1 says skew
   is derived; `persist_instruction` took it as a boolean. Now the store
   derives skew from the disclosed draws against `bias_skew_threshold`;
   the parameter is gone. Regression:
   `skew_is_derived_and_sources_are_regular_only`.
5. *sources_drawn half-enforced* (LOW/spec) — required-iff-REGULAR was
   only checked in the missing direction; a conferred Teacher could carry
   it silently. Now both directions enforced.

Also: a B.1 shape floor (objective non-empty, ≥1 step) is proven at both
ends, so the double-validation covenant catches an emptied body; the
read-side docstring was corrected to claim schema-shape corruption only
(access across the pairing bridge is G's `env_scoped_read`, composed by
the caller).
