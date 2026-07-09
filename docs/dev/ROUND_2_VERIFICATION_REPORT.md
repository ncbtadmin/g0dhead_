# Round-2 Verification Report — the combined ledger
### Claude Code (CLI instance), 2026-07-08 · repo at 0e451c6, tree otherwise clean

This is the report commissioned across PROMPT_A–F and ordered into final shape by
PROMPT_H [H7.4]. It consolidates: the Phase 1 adjudication, the A–F verdicts, the
E patches, the PROMPT_G rulings reconciliation, the [H6] spot-checks, the generated
criteria sweep, and the completed S1–S9 decision sheet. Committed docs-only under
S8; all code, migrations, and Cargo.* remain frozen until the sheet returns.

**Authority model (PROMPT_H):** author rulings bind interpretation; first-hand code
reads outrank everyone on what the code does; the sovereign outranks everyone on
what happens next. Every verdict below names its class.

**Evidence discipline:** VERIFIED = first-hand, witness cited. RULED(author, §) =
PROMPT_G_RULINGS.md (28,838 B, sha256 f8630c6f… — hash verified). INFERENCE stays
labeled. The transcript corpus postdating the review memos is contaminated with
their quotations (E0); every corpus claim here was drawn from pre-review files.

---

## 1. Phase 1's true state (rendered earlier, unchanged)

Commit 0e451c6 is a real but narrow improvement committed under a claim broader
than its content: the SUPPORTED_CONCORDAT unification is sound; the bias-config
fix is read-side only; `reconstruct_draft`'s hardening addressed parseability
while the defect was semantic; and the pass shipped without the adversarial round
the standing rule attaches to safety-invariant changes. Nothing argues for revert;
everything it left open is disposed below.

## 2. The A–F adjudication ledger (all first-hand)

| Finding | Verdict | Witness / disposition |
|---|---|---|
| A1 five unconstructed codes | VERIFIED (grep: zero sites) | disposed by G1: two verbs; miscodes fix-ordered; ENV_INVALID = slice-10 labor debt |
| A2 no provenance table | VERIFIED | disposed by G8: fold declared (internal view + SC-A08); A.6 chain table rides with Slice 10 |
| A3 SC-D01 frozen allowlist | VERIFIED (test read) | G2: test widens, discovered crates; SC-H07 minted for the fabricated-default class |
| A4 SC-B04 one-crate scan | VERIFIED | G3: widens workspace-wide; the no-HTTP rule becomes an arch wall |
| A5 SC-H06 depth-one sweep | VERIFIED | G4: information_schema × `secrets::scan` |
| A6 SC-C07 unreachable log clause | VERIFIED (RAISE aborts tx) | G6: criterion narrowed — the wall does not keep a diary; ledger reclassified with arch pins; SC-N04 claims the seam |
| A7 SC-E01 hand-built witness; 4 swallow sites | VERIFIED | G5: E01 splits; SC-E05 minted; BudgetExceeded guards must distinguish already-recorded from failed-to-record |
| A8 94 ≠ "ninety"; SC-F06 the honest model | VERIFIED (counted: 94) | G12 counts out of prose; G13 annotation convention |
| B1 four semantic re-lint holes | VERIFIED — three aggravations ours (debug-panic strand; both-ends clamp; SOVEREIGN_JUDGMENT flip) | G7: content-hash at flag + re-proof at read subsumes the class |
| B2 config fix read-side only | VERIFIED (incl. window=0 kills escalation) | H3(2): write-side per-key contracts RULED |
| B3 "zombie cannot write" FALSE | VERIFIED (release_lease unguarded; bias surfaces identity-less vs XIII.1; SC-H05 one path) | G10 + SC-A08 + H3(3) riders |
| B4 trigger-disable global; gate nondeterministic | VERIFIED | H3(5): planted-row pattern; serialized singleton tests |
| B5 plural-ranges hedge | CLAIM (Standard §3.4 unread by us) | Standing Note pinned to the const; no action until agents ship separately |
| C0 overrides ratified in doc 00 §4 | VERIFIED | reviewer self-refuted its earlier claim |
| C2/C3 gate-block census & erosion | VERIFIED from pre-review transcripts; E0-corrected and extended (slices 1–3 all carried step counts; slices 7–8 lost `result:` entirely; slice 9 healed partially in a fresh session) | H3(6): the report gets a producer |
| C4 H4 no-producer | VERIFIED (verify.py knows c/cpp/python/js only) | the generator rider is the cure |
| C5 slice 5 | VERIFIED: gated in fact, never block-reported | absorbed into the producer + tree-evidence rule |
| C6 fourth override unwritten | VERIFIED with positive witness (reconciliation enumerated exactly three collisions with the skill text on-screen) | S6 text on the sheet |
| C7 evidence not in tree; BUILD.md stale; three wrong counts | VERIFIED (94 counted; "90"/"ninety"/"96" all wrong) | S5 batch |
| F1 office-shaped hole | VERIFIED (`'deacon'` would be admitted; `decided_by='forged'` passes) | G10: session-scoped actor-class auth; SC-I07a |
| F2 Manifest needs J's records | VERIFIED mechanism; BRIEF roots need no MandateRecord | G9 re-scope; residual choice = S2 |
| F3 consent volume unbounded at the gate | VERIFIED mechanism | G11: charter doctrine + one-Manifest-per-trip + SC-I07b |
| F4 validator by-eye audit never recorded | CLOSED POSITIVE by performing it: `validate_raw` (ladder.rs:61–93) is strict-total, single execution site; by-eye guarantee holds. NEW: a panicking tool `execute()` unwinds without refusal → SC-E05 class | |
| F5 deferral of the non-deferrable unrecorded | PARTIALLY REFUTED (slice docs record the exclusion) — the unengaged "day-one" designation survives | SLICE_10.md must cite and adjudicate the crossing |

E patches: E0 confirmed against primary transcripts and extended; E1/E4/E5/E6
quotes verified in the in-tree origin conversation; E2 confirmed with one
precision; E3 completed (94, counted). PROMPT_D's discipline adopted; its "ten
memory files" count refuted (five at the time).

## 3. PROMPT_G/H reconciliation — acceptance and the four corrections owed

The thirteen rulings are accepted as binding interpretation; nothing I VERIFIED
is contradicted by any of them. Corrections and nuances from the [H6] spot-checks:

- **(a) VERIFIED.** `guard_actor` (postgres.rs:696–704) and `env_scoped_read`
  (3636–3649, 3696–3708) both log-then-error — the API-layer-observer precedent
  G6 leans on is real.
- **(b) VERIFIED.** `flag_instruction` (3971–3998) persists no content hash;
  `flag_return` is the same shape. G7's gap is exactly as stated.
- **(c) VERIFIED.** Both halt handlers stamp `ValidationFailed` for every
  VALIDATE_OUT halt including skew clauses.
- **(d) VERIFIED with a nuance:** `plant_artifact` (l_student.rs:576) is a
  working no-DISABLE-TRIGGER fixture, but it covers plant-new-row corruption;
  mutate-in-place of a *flagged* record (the SC-K04 shape) still requires the
  pinned-connection `session_replication_role` path H3(5a) itself carves out.
- **(e) VERIFIED with a correction in the ruling's favor:** the mechanical
  caveat on G10 is smaller than feared. The consent-class sovereign methods
  already run explicit transactions (`lay_category_override` :1768,
  `resolve_proposal` :3063) — `SET LOCAL` drops in directly. The wrapping cost
  is confined to single-statement reserved-table paths (`set_config` :1427+)
  and the future office write paths. Costable; no blocker; the requirement
  stands as ruled.
- One integrity note: the rulings file's closing line ("the hold is released")
  is superseded by PROMPT_H's own authority model — only [H7]'s report and
  reconciliation work self-executes; code remains frozen pending the sheet.
- One tree note: `docs/_history/original spec development chat/` is untracked,
  like the rulings file — the project's oldest primary source is in the working
  tree but not in history. Folded into S5/S9 consideration.

## 4. What the build process actually is (the C commission)

Doc 00 §4 defines the gate's commands; SKILL.md defines a report format that only
`verify.py` can emit and `verify.py` cannot see Rust; BUILD.md — delegated the
reproduction duty — froze at slice 1. So the process as practiced was: the right
commands, run faithfully, against live Postgres, reported in a hand-composed
block that eroded field-by-field with session depth, evidenced only in
transcripts, with one slice (5) gated in fact but never reported in form, and one
override (host execution) taken by evacuation and recorded nowhere. None of the
nine slices shipped ungated; none of the nine gates left tree-resident evidence.
The riders (gate-report producer; evidence lands in docs/dev/; fourth-override
text) close each gap at its cause.

## 5. The criteria sweep

`docs/dev/criteria_sweep.py` (stdlib-only, re-runnable) generated
`docs/dev/CRITERIA_SWEEP.md` against HEAD: **94 criteria** — 16 DEFERRED (exactly
sections I+J, mechanically confirmed), 12 adjudicated this round (8 NARROWER,
1 SPLIT, 1 HALF+ANNOTATED, 2 MEETS-variants), **66 PENDING** — the parallel,
non-blocking sweep S1 commissions, now a table that diffs instead of a narrative
that decays.

## 6. The decision sheet — S1–S9, with this instance's verdicts

| # | Decision | My verdict |
|---|----------|-----------|
| S1 | Slice 10 gate scope | **adopt** — matches the ledger-convergent rec |
| S2 | C.4 mandate substrate in 10 or 11 | **10**, contingent on S4 surviving un-trimmed; if the slice budget strains, 11 costs nothing (BRIEF roots suffice — verified) |
| S3 | Content-hash certification | **adopt** |
| S4 | Hardening-rider bundle | **adopt whole** — every rider traces to a VERIFIED defect or a RULED order; trimming any one reopens a named hole |
| S5 | Doc amendments batch | **adopt** (docs-only; includes tracking the origin-conversation folder or ruling it stays untracked) |
| S6 | Fourth override text | **adopt as proposed** — it records what C6/E2 proved was never argued |
| S7 | Memory discipline + RULED tag | **ratify** — already in force |
| S8 | Docs-only bar lift | exercised for exactly three artifacts: this report, the sweep script, the sweep output |
| S9 | PROMPT_G_RULINGS.md | **hold untracked** until S5 lands, then commit with the batch — one docs commit, one story |

**Frozen pending the sheet's return:** all code, all migrations, all Cargo.*,
Slice 10's spec. The no-HTTP rule holds (verified again this round: no outward
transport anywhere in the dependency tree).
