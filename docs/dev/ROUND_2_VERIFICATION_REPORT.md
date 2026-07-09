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

---

## 7. Addendum, 2026-07-09 — the sheet returned; the flickering provenance file

The instance that wrote §§1–6 was lost before the answered sheet reached it; this
addendum is its successor's, written after verifying the inheritance first-hand
(rulings hash `f8630c6f…` reproduced; sweep re-run against HEAD, byte-identical
after BOM/EOL normalization; no-HTTP wall re-verified across all twelve Cargo
files). The S1–S9 sheet returned answered via PROMPT_I v2: S1 adopted, S2 → Slice
10 (C.4 severable to 11 only if the budget breaks), S3 adopted, S4 adopted whole,
S5 adopted plus the untracked-conversation ruling, S6 adopted, S7 ratified, S8
spent, S9 commit-with-the-batch. The S5 batch lands in the commit carrying this
addendum.

**The flickering provenance file (R1).** During the evening of 2026-07-08 the
working copy of `docs/_history/07_student_handbook.md` — the directive brief,
8,932 bytes, blob `dffc483f` at HEAD — read, through one observer, as a ~9 KB
ratified-v1.0 fragment (blob `62c2487f…`) and later read as the brief again;
a six-lens disk investigation (all ten CLI transcripts, all eighteen desktop-app
audit logs, the NTFS USN journal, the git object store, an AppData census, and a
blob brute-force) resolves it as follows. The fragment's content is identified
byte-exactly: this instance reproduced `62c2487f1ab89a7a7d73964ad7c422b0a9803823`
by hashing the first 8,932 bytes of the ratified `docs/07_student_handbook.md` —
the sibling document truncated at precisely the brief's file length. Every
sighting of the flicker was made from inside the Claude-desktop (Cowork) VM's
bridged view of the repo, whose stat during the flicker served the *canonical*
file's mtime (Jul 7 13:19:12.5817948, tick-identical at 100 ns) under the
`_history` path — while the host file's true mtime (Jul 7 11:51:07.3026349) was
and is untouched. The NTFS USN journal settles the writer question: the host
file object has received **no journaled operation of any kind since the Jul 7
promotion** (last-USN pointer bracketed by write-once git objects; FileId
stable; creation time intact; parent-directory swaps likewise excluded; the
blob was never persisted to the object store), and no transcript or audit log
anywhere contains a write to the path or a restore-capable command. Conclusion:
**the flicker was a read-layer artifact of the desktop-app VM's file-sharing
bridge** — it served the same-basename sibling's content and metadata under the
`_history` path, clamped at the stale 8,932-byte directory entry — and the
"revert" was cache invalidation when the VM cycled; there was no writer, and
the provenance file on the host was never altered. The briefed candidates are
each exculpated for the flicker itself (the project-knowledge cache syncs into
AppData and holds no handbook copy; no backup/AV artifact exists and Defender's
only window activity was a 6:15 AM definition update; no editor session at
either bound). The evening's genuine out-of-session writes are separately
accounted for: `docs/_history/original spec development chat/` was created
6:46–6:47 PM by extraction of a `files.zip` (recovered from the Recycle Bin,
deleted 6:47:24 PM; contents exactly the four conversation files, no handbook;
the folder's creation stamps carry a UTC-rendered-as-local artifact typical of
the exporting tool), and `PROMPT_G_RULINGS.md` was the Cowork author-instance's
documented single Write at 7:24 PM. One residue is recorded rather than smoothed
over: `docs/00_read_before_development.md` took exactly one journaled but
timestamp-invisible operation USN-bracketed to 5:02–6:46 PM Jul 8 (content
matched HEAD throughout, so the operation was metadata-class or
content-restored); its pre-edit USN witness was overwritten minutes after
capture by this addendum's own S5 edits and survives only in the investigation
transcript — and the incident-era journal records themselves have rolled out of
the 32 MB journal, so no deeper replay is possible even elevated. The handoff's
two extra expected-status entries dissolve under the same lens: the expected
status was the Cowork observer's own 2:34 PM view through its VM bridge, one
minute before it authored the handoff — `?? .claude/` because that
environment's git does not read the user's global ignore
(`**/.claude/settings.local.json`, in force since 7/1; the census proved
`.claude/` never held anything non-ignored), and `M docs/dev/CRITERIA_SWEEP.md`
of the same served-view class: not reproducible under the host's git four
minutes later, where the working copy round-trips byte-identical to the
committed blob with its generation-time mtime (8:00:10 PM) intact. Current-good
state: `docs/_history/07_student_handbook.md` is byte-identical to HEAD
(`dffc483f` reproduced first-hand), and the tree at this instance's boot matched
the predecessor's recorded final state exactly. Proposed guard, not implemented:
extend `docs/_history/PROVENANCE.md` to also list the three tracked briefs'
hashes, giving any future instance a one-command sweep against the manifest —
a hash check reads content through whatever lens is doing the looking, so it
catches both a real rewrite and a served-content illusion of this incident's
kind, which `git status` catches only while it lasts and a directory listing
cannot catch at all.
