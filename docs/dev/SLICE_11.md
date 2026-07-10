# Phase B — Slice 11: Section J behavior (the Mandate Rule in motion)
### Nascent — not yet pinned for build. This file opens with §0 (the wall boundary, ruled 2026-07-09) and the acceptance conditions carried from Slice 10; the buildable spec (§1 onward) pins only after the opening adversarial round.

> Slice 11 builds Section J's *behavior* — fetch-execution binding, manifest
> coverage/sought maps, the breadth-creep property test, Doctor orphaning
> (SC-J03/J05/J06/J07/J08/J10) — and claims the mandate-authoring SC-C07 entry
> (per G9). It stands on the J-floor substrate Slice 10 delivered (mandates,
> the provenance chain table, SC-J01/J02/J09). It does **not** open the
> boundary: the no-HTTP wall stands through this slice (§0).

## Opening conditions (from the Slice 10 acceptance, 2026-07-09 — binding)

1. **This slice opens with an adversarial round, before its spec pins.** The
   eight-lens / three-refuter panel that hit the session API limit during
   Slice 10's delivery (four finder lenses and the entire verify stage never
   ran) is re-run in full over the slice-10 surfaces first. Any CONFIRMED
   finding becomes a **pre-11 rider** — fixed and recorded before Slice 11
   behavior lands, per the not-silently-blessed rule — and only then does the
   buildable spec below pin and cross the desk.
2. **The F1 keyed-intake hardening is a named rider of this slice**, not loose
   debt: `commit_file` becomes idempotent (a deterministic node id derived
   from `item_ref`, keyed / ON CONFLICT), so the Deacon's `admit` cannot orphan
   a duplicate CLEAN atom on a crash or concurrent retry. It ships with a test.

The gate report comes from the producer (`scripts/gate_report.py`) — the only
voice of the gate.

## 0. The wall stands through Slice 11 — the boundary, argued (ruling 2026-07-09)

The no-HTTP wall is not inherited into this slice by default; it is **kept on
argument**, and the argument is this.

Section J is a body of *behavior* — a fetching labor executes a mandate's trip,
its results are bound to the writ that sought them, the Manifest maps what was
sought against what came back, and breadth is held to the mandate's own
demands. Every one of those behaviors is **built and proven without any real
outward transport**, against a `FetchEndpoint` trait fronted by an
**instrumented deterministic mock** — the same seam-behind-a-trait pattern the
Deacon's threshold used in Slice 10, where `ScanEndpoint` and its deterministic
mock carried the whole of Section I while no daemon was wired. The fetch mock is
*instrumented*: it records, in its own ledger, exactly which locators it was
asked to fetch. That record is what makes the central proof mechanical rather
than aspirational — **SC-J05's property test asserts locator-set equality
against the mandate through the mock's own record**: the set of locators the
fetch layer actually requested must equal the set the mandate authorized,
compared against what the mock witnessed it was asked for. No byte crosses a
real boundary, and yet no-breadth-creep is *proven*, not promised. If Section J
can be built and its safety property proven against the trait and its
instrumented mock — and it can — then no outward transport is needed to earn
this slice, and the wall stays up. The mechanical wall test
`no_outward_transport_wall` (no outward-transport dependency in any workspace
`Cargo.toml` **or** `Cargo.lock`) stands unchanged through delivery.

**The wall deletes in Phase 5, and only there, and only atomically.** Its
deletion is safe exactly when the machinery that makes a real outward fetch safe
all exists at once — so the wall is deleted in the *same commit* that lands, together:

- **transport behind the `FetchEndpoint` trait** — the real provider replacing
  the mock seam, and nothing reaching the network except through it;
- **trip-budget enforcement at the transport layer** — a mandate's `trip_budget`
  actually bounds the real fetch (its requests, its bytes, its time) at the
  point bytes move, not merely at authorship;
- **the Law XV scan of fetched bytes ahead of the quarantine write** — nothing
  external is persisted, even into quarantine, before the outbound-secret scan
  has passed over it; the scan precedes the write, never trails it;
- **SC-F06's integration half** — the real-endpoint half of the tool-calling
  criterion that Slice 6 named in-test and pinned to the endpoint slice (the
  G13 annotation), closed here where the endpoints become real.

Delete the wall without any one of those four and a named hole reopens: an
untrusted byte reaching a store surface unscanned, a trip with no ceiling, a
transport path outside the trait, or an integration criterion still half-proven.
So the deletion is not a line removed when convenient — it is the last line of
the commit that makes removal safe, in Phase 5. Until that commit, the wall is
argued to stand: this slice proves Section J against the trait, and the boundary
holds because it is reasoned, not because the slice before it happened to hold it.

## 0.1 The opening adversarial round — outcome (2026-07-09)

The round ran (condition 1) as the coverage gap the aborted Slice 10 review left:
the **four finder lenses that never ran** (quarantine-wall, mandate-writ
concreteness, riders-integrity/config/identity, arch-walls) plus a **fix-diff /
refutation audit** of the three fixes and two refutations Slice 10 landed in-loop.
The eleven findings the aborted round's completed lenses had already produced
(F1–F11, dispositioned in `SLICE_10.md §9.2`) were not re-run. Each new finding
got the independent three-lens refuter panel (code-trace / reachability /
reproduction; majority-CONFIRM to survive) that the aborted round never reached.
Twenty-three agents, no errors. Six candidates: **three survived, three refuted.**

**Confirmed → pre-11 riders, landed (fixed with tests before §1+ pins):**

| Rider | Severity | What it was, and the fix |
|---|---|---|
| R11-1: `honorific_set` write-side contract | major (3/3) | The H3(2) contract lumped `honorific_set` with `name_roster` as a flat string array, but its shape is the nested `{"teacher": {tier: title}, "student": [honorific]}` object the conferral and mount paths read (seed 0007). So the *correct* value was unwritable through `set_config`, and the only contract-passing value (a flat array) bricked every environment mount. Fixed: `honorific_set` gets its own object-shape contract (`postgres.rs` `config_contract`); non-mutating test in `e05_riders.rs::config_write_contracts`. |
| R11-2: wall blind to `[dependencies.<name>]` | major (3/3) | `manifest_dep_names` recognized only section-form `[dependencies]` headers; Cargo's per-dependency **table form** `[dependencies.interprocess]` slipped it, and since the lockfile half does not check the raw-socket list, a table-form IPC/raw-socket crate escaped **both** halves — a hole in the exact wall §0 rests on. Fixed: the parser now reads the table form (and a rename inside it); regression test `manifest_table_form_is_caught`. |
| R11-3: wall omits `ws`/`websocket`/`soketto` | minor (2/3) | WebSocket transport is in the wall's scope (`tungstenite`/`tokio-tungstenite` are listed) but these were absent, leaving a lawful `ws = "…"` a green path. Fixed: added to the substrate list (verified absent from `Cargo.lock` first, since that list is lock-checked). |

**Recorded seam — not a live bug, but §1+ must resolve it (not silently blessed):**
Law V.4's `godhead_quarantine_only` wall fires only for **mandate-rooted** fetch
jobs (`brief_ref ∈ mandates`), while the schema and `quarantine_deposit` also
admit **brief-rooted** external arrivals. The panel majority-refuted this as a
*live* bypass — there is no fetch layer yet, and by C.4 a mandate is the charter
of every *outward* act while a brief charters *internal* work, so the design
record (`i_deacon.rs`) has Slice 11 trips take the mandate-rooted
FETCH_PER_CANON/WRIT shape, which the wall covers — but all three verifiers
flagged it as a real forward-looking seam. **When §1+ pins the fetch behavior it
must close this explicitly:** either every external fetch is mandate-rooted by
construction (and the brief-rooted deposit path is documented/constrained to
internal use), or the V.4 wall extends to brief-rooted external arrivals. Pinned
here so the pin faces it rather than inherits it.

**Refuted, with two byproducts worth keeping:**
- The claim that `gate_report.py` would report a **false FAIL** when ≥2 binaries
  flake was refuted *empirically* — two verifiers built a two-binary crate and
  ran `cargo test --no-fail-fast`; cargo emits a per-binary `to rerun pass` hint
  for each failed target (plus the consolidated summary), so the producer's
  `rerun_specs` catches them all. The producer's rerun logic is sound.
- The claim that `sc_a08`'s resolution union omits real tables (causing false
  "dangling") was refuted: `artifacts`/`embeddings`/`environment_items`/
  `concordat_artifacts` mint no standalone uuid a lawful `input_ref` could cite
  (composite or text keys; their uuids resolve via the tables already walked) —
  which closes the open observation `SLICE_10.md §9.3` had left.

---

*§0 recorded 2026-07-09 by sovereign ruling — "the boundary is argued, not
inherited." The opening adversarial round has run (§0.1): three confirmed
findings landed as pre-11 riders R11-1..3, the V.4 brief-rooted seam is pinned
for the pin to resolve. Sections 1+ (pinned criteria, what-this-builds, the
full rider list incl. the F1 keyed-intake rider, non-goals, gate) pin next, as
their own docs commit crossing the desk before implementation — the two-commit
lifecycle (DISCIPLINE.md §5).*

## 0.2 Canon-fetch scope — ruled (2026-07-09, during the build)

A design question surfaced building the fetch layer: a CANON mandate's `demands`
are *freeform, exhaustiveness-defining clauses* (C.4), not locators, so turning
them into fetch targets is open-ended search — the deferred breadth system. The
sovereign ruled:

> **v1 canon trips fetch sovereign-named concrete sources.** C.4's CANON
> `demands` remain freeform exhaustiveness clauses, untouched; the CANON mandate
> **gains a `sources` field of typed locators** under the identical SC-J02
> validation (locators, never queries — a canon with empty `sources` simply has
> no v1 trip to run). `FETCH_PER_CANON` executes against `sources` via the same
> mandate-rooted machinery as writs; **SC-J05's set-equality property covers
> both kinds**; C.3 coverage and the gap duty map against **clauses**, so unmet
> clauses surface for the sovereign to widen by naming more sources.

Grounding: Handbook §5.1 ("v1 mandates name concrete targets — breadth discovers
them") and the Phase A ruling that minimal canon-fetch is v1 scope while
breadth-tuning is not. The fetch layer is therefore **unified**: writ targets and
canon sources are one typed-locator set under one SC-J02 wall and one SC-J05
property; the freeform clauses remain the coverage/exhaustiveness surface, and
discovering sources from clauses stays deferred. The **C.4 `sources` note is an
author-sanctioned doc amendment** (doc 07 §C.4) that rides with the delivery.
Budget fallback: if the `sources` field breaks the slice budget, fall back to
deferring canon collection (the earlier option) with a G13 annotation — but the
record says build it.

---

# Part II — the pinned spec (§1+), presented to the desk 2026-07-09

*Pinned after the opening round (§0.1), as its own docs commit, before any
Section-J behavior code moves — the two-commit lifecycle. The build bar stays
down until this crosses the desk.*

## 1. Pinned criteria — Section J behavior (Document 8 §J)

Section J's substrate (SC-J01/J02/J09) shipped in Slice 10. This slice pins the
*behaviors*, each proven against the `FetchEndpoint` trait and its instrumented
mock (§0) — the no-HTTP wall stands throughout.

| Criterion | Enforces | Seed test |
|---|---|---|
| SC-J03 | Fetch-execution binding: a `FETCH_PER_CANON`/`FETCH_PER_WRIT` trip lacking a resolving `mandate_ref` refuses; mandate kind must match tier (WRIT→Devout, CANON→Canon; cross-matches rejected) — every fetch trip is mandate-rooted by construction (§1.4; B.3; V.4 closure (a), §4) | `sc_j03_fetch_binds_mandate` |
| SC-J04 *(met by Slice 8's lint — claimed, G13)* | An Instruction carrying any fetch step fails Executability-Lint clause (f) and is never written (v1 prohibition). Enforced since Slice 8 (`k_concordat.rs` clause-(f) fixture); this slice adds the Section-J-named test and annotates the criterion met where it already lives | `sc_j04_fetch_step_never_lints` |
| **SC-J05 (headline property, sovereign directive)** | No writ text, however adversarially worded, produces a fetch to a locator not enumerated in its validated demands — the fetch layer resolves targets **only** from the mandate, never from free text. Proven mechanically: the instrumented mock records every locator it is asked to fetch, and the property test asserts **set-equality between the mock's record and the mandate's demands** across a fuzzed corpus of adversarial writ texts (C.4; §1.4; doc 08 §"two doors") | `sc_j05_no_text_widens_fetch` |
| SC-J06 | CollectionManifest (C.5): every collected item maps to a writ target by `target_index`; an unmapped item fails manifest validation (no padding); an unmet target carries empty `item_refs` and is flagged | `sc_j06_collection_maps_or_flags` |
| SC-J07 | CorpusManifest (C.3) coverage: every canon clause maps; unmet clauses trigger the gap duty — the Student refuses, flags exactly the unmet clauses, and writes nothing sourced outside the canon | `sc_j07_corpus_coverage_gap_duty` |
| SC-J08 | Doctor deployment with `student_env_ref` not `LIVE` refuses `ENV_INVALID`; dissolving the Canon scriptorium orphans the Doctor's (builds on `EnvStatus::Orphaned`); no silent revival — a fresh Canon environment does not re-enable an orphaned Doctor without a new sovereign pairing | `sc_j08_doctor_orphan_no_revival` |
| SC-J09 *(fetch half — re-armed, **claimed not inherited**)* | The chain-append-in-flight behavior, now exercised through a live fetch trip: a `FETCH_PER_CANON`/`FETCH_PER_WRIT` labor appends its ProvenanceChain entry **before** the item write (§4.2), and an item deposited whose producing entry is absent refuses `PROVENANCE_INCOMPLETE`. Slice 10 shipped the substrate half and said this fetch half re-arms here (G13); this slice claims it in behavior — the substrate wall is *exercised* by a real trip, not assumed | `sc_j09_chain_append_in_flight_fetch` |
| SC-J10 | A collected item the system cannot normalize is stored raw, marked `normalizable: false`, and surfaces an incompatibility notice; fetch-layer garbage (corrupt, deceptive, unfetchable) is refused **at source**, never laundered through quarantine | `sc_j10_unnormalizable_marked_not_laundered` |
| SC-C07 *(mandate-authoring entry — claimed, G9)* | IV.4 "authoring fetch mandates" is a human-reserved action: the authorship surface takes a human actor and no job identity; an agent-shaped invocation is rejected. Slice 10 built `author_mandate` + the SC-J01 wall; this slice claims the IV.4 one-test-per-surface entry now that mandate authoring is exercised in behavior | `sc_c07_mandate_authoring_entry` |

## 2. What this slice builds

- **The `FetchEndpoint` trait + instrumented deterministic mock** (the §0 seam,
  mirroring Slice 10's `ScanEndpoint`): the mock resolves each demanded locator
  to fixture bytes and **records, in its own ledger, every locator it was asked
  to fetch** — that record is SC-J05's witness. An `unreachable`/garbage mode
  serves SC-J10. No real transport; the wall stands (arch test unchanged).
- **The fetch labor** (`FETCH_PER_CANON` / `FETCH_PER_WRIT` execution): resolves
  targets **only** from the validated mandate's demands, drives the mock, and
  deposits results into quarantine (Slice 10's `quarantine_deposit`) with the
  chain appended in flight (SC-J09). Binds to a resolving `mandate_ref`, kind
  matched to tier (SC-J03).
- **Schemas C.3 / C.5**: `CorpusManifest` (canon clause → coverage map) and
  `CollectionManifest` (writ `target_index` → `item_refs`), with the sought/met
  maps SC-J06/J07 validate; the `normalizable` flag + incompatibility notice
  (SC-J10).
- **The Doctor** (Canon Student) deployment + orphaning (SC-J08): reuses the
  scriptorium mount walls and `EnvStatus::Orphaned`; orphaning cascades from a
  dissolved Canon scriptorium through the pairing; revival requires a new
  sovereign pairing, never silent.
- **Store methods + migrations**: fetch-execution binding, manifest
  assembly/validation, Doctor deployment/orphaning, and the V.4 closure of §4
  (the extended quarantine-only wall). Migrations numbered at build time.

## 3. Riders (carried from birth)

| Rider | Order | Seed test |
|---|---|---|
| **F1 keyed-intake idempotency (condition 2 — named from birth):** `commit_file` becomes idempotent — a deterministic node id derived from `item_ref` (or a caller-supplied keyed id) + `ON CONFLICT` — so the Deacon's `admit` cannot orphan a duplicate CLEAN atom on a crash between the mint and `mark_admitted`, or under two concurrent `admit` calls. The §6 "keyed writes" guarantee Slice 10 corrected becomes true. | S10 acceptance (2) | `admit_is_idempotent_under_retry` |
| Dedicated tests for the Slice 10 legibility fixes F6/F7/F11 (pinned in SLICE_10 §9.2 — folded here so they are not lost) | S10 §9.2 | `chain_race_is_typed`, `deposit_converge_logs_standing`, `ladder_budget_skips_second_refuse` |
| Test hygiene: complete per-test `#[serial]` coverage of the singleton-touching binaries so the multithreaded gate is clean on the first pass (extends the S4 rider; the producer's serial-rerun is the backstop, not the goal) | S10 §9.1 | gate deterministic multithreaded |

R11-1..3 (the opening round's confirmed findings) already landed in `dce06c2`;
they are riders of this slice by authority, recorded in §0.1, not re-listed here.
The `config_constants` defense-in-depth `no_delete` (F2) remains dispositioned
out of the raw-SQL threat model (G6), recorded not scheduled.

## 4. The V.4 brief-rooted seam — resolved (both closures)

The seam §0.1 recorded is closed here, adopting the desk's starting position —
**both closures** — the cost being real but small:

- **(a) Mandate-rooted by construction.** SC-J03 binds every fetch trip to a
  resolving `mandate_ref` with kind matched to tier, so no external fetch is
  brief-rooted. `quarantine_deposit`'s external path requires that mandate; the
  BRIEF chain root (SC-J09 admits `CANON | WRIT | BRIEF`) is retained **only**
  for internal-origin provenance chains (a BRIEF-rooted internal chain — the
  JobDraft's `brief_ref` — never an external arrival), and the brief branch of
  `quarantine_deposit` is documented and guarded as internal, not a fetch path.
- **(b) The wall extended.** `godhead_quarantine_only` gains an OR'd clause:
  a job that has deposited external material to quarantine
  (`EXISTS(SELECT 1 FROM quarantine_items WHERE origin_job_ref = writer)`) is
  barred from the internal namespace (nodes/artifacts/environment_items) whatever
  its charter — defense-in-depth over (a), catching a depositing job that later
  attempts a direct internal write.

**The honest cost, disclosed (not an argument against):** closure (b) is one
OR'd subquery in the trigger plus its test, and it carries a *timing*
limitation — a job that writes an internal row **before** it deposits is not
caught by (b) alone. That case is fully covered by (a) (such a job is not a
lawful fetcher — fetchers are mandate-rooted and deposit before admission via a
separate intake job), so (b) rides as belt-and-suspenders, not the load-bearing
guard. The cost is real and small; it does not outweigh closing the seam from
both sides, so both are adopted. Admission's own node mint is unaffected: it runs
under a separate intake stage job that has deposited nothing, so it clears (b).

## 5. Design decisions (the §0 constraints, mechanical)

- **The mock is the whole outward surface.** SC-J05 is proven against the mock's
  record, never a network; the wall's arch test (`no_outward_transport_wall`,
  now table-form-aware and ws-covering after R11-2/R11-3) stays green — a Slice 11
  that tried to add real transport would trip its own wall.
- **Targets resolve from the mandate, not from text — at one seam.** The fetch
  labor reads locators from the persisted, validated `MandateRecord` demands
  only; no code path lets writ prose reach the fetch call. SC-J05 fuzzes the
  prose and asserts the mock saw exactly the mandate's set.
- **Gap duty is a refusal, not a silent partial** (SC-J07): an uncovered canon
  clause halts the Student in the established labor-rule shape, flagging the
  unmet clauses, writing nothing outside the canon.
- **Tests only accumulate; any half met below its words carries its G13
  annotation** naming the unmet half and where it re-arms (real transport →
  Phase 5, §6).

## 6. Non-goals

- **No real transport, no HTTP client, no real scan/fetch provider** — the wall
  stands; the traits and their mocks are the whole outward surface. Real
  transport, trip-budget enforcement at the transport layer, the Law XV scan of
  fetched bytes ahead of the quarantine write, and SC-F06's integration half are
  **Phase 5**, in the one commit that deletes the wall (§0).
- **No retrieval breadth / no query system** — SC-J02/J05 are the enforced wall
  against it; the breadth system is deferred by directive.
- **No Duty of the House** (quarantine purge/rotation — deferred, doc 00 §7).
- **No multi-tenancy** (deferred).

## 7. Budget & severability

Section J is one coherent behavior (fetch → manifest → coverage → Doctor); it
does not sever cleanly, and its criteria interlock (SC-J05 needs the fetch
labor; SC-J06/J07 need the manifests; SC-J08 needs the Doctor). If the slice
budget breaks in practice, the **only** severable piece is the Doctor/orphaning
(SC-J08) — it moves whole to a follow-up, recorded here, because it stands on
the pairing/orphaning machinery rather than on the fetch path. The F1 rider is
**not** severable (condition 2 names it from birth). Nothing else is.

**SC-J08 SEVERED (2026-07-10, invoking this clause).** The budget broke in
practice: the Doctor's deployment maps cleanly onto existing machinery (a Doctor
is a Canon Teacher env, Holy Standard §4.3; it pairs to the Canon Student via
`form_pairing`'s CanonicalInstruction kind, and the not-LIVE→`ENV_INVALID`
validation is a pre-check), but the **orphan cascade** — dissolving the Canon
Student environment must drive the Doctor's to `ORPHANED`, with no silent
revival — rests on the environment-**dissolution** path (matrix decommission),
which is not yet built and whose topology carries real risk to add correctly.
Rather than rush that into adversarially-reviewed code at the end of a long
build, SC-J08 moves whole to its own follow-up slice (**Slice 11b — the Doctor**):
deploy_doctor (validate LIVE Canon Student → `ENV_INVALID` else; establish the
Canon Teacher; form the CanonicalInstruction pairing), the dissolve→orphan
cascade over that pairing, and the no-revival rule. This slice delivers the rest
of Section J behavior (SC-J03/J04/J05/J06/J07/J09-fetch/J10 + the SC-C07 mandate
entry + the V.4 both-closures); the delivery ledger (§9) carries the severance
in its criteria account, and CRITERIA_SWEEP marks SC-J08 DEFERRED with this
pointer (G13). Section J is *behaviorally* complete but for the Doctor's loop.

## 8. Gate & delivery protocol

The gate is doc 00 §4's three commands via the producer (`scripts/gate_report.py`
— the only voice of the gate), on the host against live Railway Postgres.
Adversarial review precedes delivery (the standing rule); the delivery ledger
(§9) appends at delivery as its own commit (the two-commit lifecycle), carrying
the adversarial ledger, the regenerated sweep, and the producer gate report.

---

*Presented to the sovereign 2026-07-09. The build bar for Section J behavior
lifts only on sign-off — the spec crosses the desk before code, as for eleven
slices. On sign-off: build against the `FetchEndpoint` mock (wall standing),
carry the F1 rider from the first commit, close the V.4 seam both ways (§4),
and bring the delivery ledger to this same desk. The no-HTTP wall is not this
slice's to delete.*

---

## 9. Delivery ledger (2026-07-10)

Slice 11's Section-J behavior is delivered on `master`, gate green, against live
Railway Postgres. The delivery appends here as its own commit — the two-commit
lifecycle (DISCIPLINE.md §5): the spec (§0–§8) crossed the desk before code; this
ledger crosses it at delivery.

### 9.1 Gate — from the producer

`scripts/gate_report.py`: **PASS (3 steps)** — fmt clean, clippy clean, `cargo
test --workspace --no-fail-fast` **161 passed, 0 failed, 0 ignored across 47
binaries**, zero DATABASE_URL skips. (Migration 0018 required a godhead-store
recompile to embed — sqlx embeds migrations at compile time — so its increment's
gate ran a full rebuild; noted so the next migration-only change forces the same.)

### 9.2 The build — five gated commits, spec before code

The build opened with the adversarial round (§0.1 → pre-11 riders `dce06c2`) and
the signed-off pin (`8bb56dd`), then proceeded in gated, pushed increments:

| # | Commit | What |
|---|---|---|
| 1 | `efc9b74` | F1 keyed-intake idempotency (the rider named from birth): `commit_file_with_id` + a derived `admission_node_id` (uuid v5) so `admit` converges instead of orphaning a duplicate CLEAN atom. |
| 2 | `aa40bfd` | the fetch-execution core + the canon-`sources` ruling (§0.2): the `FetchEndpoint` trait + instrumented `MockFetcher`, `run_trip`, and the unified writ/canon locator set (migration 0017). |
| 3 | `7956094` | the collection manifests — `validate_sought`/`validate_coverage` (pure, unit-tested) + the assembly labors + the canon gap duty. |
| 4 | `791ddbd` | the V.4 both-closures (migration 0018), the SC-J04 lint entry, the SC-C07 mandate-authoring entry; SC-J08 severed. |

### 9.3 Criteria delivered

Against the `FetchEndpoint` mock, the no-HTTP wall standing throughout:
**SC-J03** (fetch binds a resolving mandate, kind↔tier), **SC-J04** (fetch step
never lints), **SC-J05** (the headline property — no text widens the fetched set
beyond the mandate's locators, proven via the mock's own record over a fuzzed
adversarial corpus, both writ AND canon kinds), **SC-J06** (CollectionManifest
maps or flags, no padding), **SC-J07** (CorpusManifest coverage + the gap duty
refusal), **SC-J09 fetch-half** (chain appended in flight, claimed through a real
trip), **SC-J10** (garbage refused at source, never laundered), the **SC-C07**
mandate-authoring entry, and the **V.4 both-closures** (§4). The **F1** rider
shipped in the first commit.

**SC-J08 (the Doctor) SEVERED to Slice 11b** (§7): the deployment maps onto
existing pairing machinery, but its orphan cascade rests on the unbuilt
environment-dissolution path; rather than rush that topology into reviewed code,
it moves whole. The sweep marks it DEFERRED with the §7 pointer. Section J is
behaviorally complete but for the Doctor's loop.

### 9.4 Adversarial review — before delivery (the standing rule)

A four-lens adversarial review ran over the new surfaces (fetch labor, canon
sources, manifests + gap duty, V.4 closure + F1), each finding independently
refuted by a three-lens skeptic panel (10 agents, no errors). Two findings
survived — **both confirmed, both fixed before delivery**; the fetch-labor and
V.4/F1 lenses surfaced nothing.

| Finding | Severity | Fix |
|---|---|---|
| The v1 canon-fetch path — the slice's marquee §0.2 ruling — shipped with **zero test coverage**: every mandate fixture passed `sources: []`, so the canon branches (`trip_locators` Canon arm, canon-source shape validation, canon source_id resolution) executed with a non-empty `sources` in NO test, and SC-J05's "covers both kinds" claim was provably unmet (only writ trips were fuzzed). A regression defeating the canon-source wall would have passed green. | **major (3/3)** | **Fixed**: `sc_j05_canon_sources_fetch` (a FETCH_PER_CANON trip fetches its `sources` exactly, never the freeform clauses — the canon analogue of the writ property) and `sc_j02_canon_sources_concreteness` (query-shaped / malformed / unknown-source_id canon sources each fail at authorship, the identical wall). The canon branches are now exercised; SC-J05 truly covers both kinds. |
| The gap-duty refusal named unmet clauses by re-deriving indices from clause **text** membership, so a canon with duplicate clause text would name a *covered* clause's index too (`"1 of 2 uncovered (indices [0,1])"`). Safe direction (still refuses; no holed manifest), but the sovereign's only signal (Law XV forbids the text) pointed at the wrong clause. | **minor (3/3)** | **Fixed**: the unmet indices are computed from the coverage map's own empty POSITIONS (`assemble_corpus_manifest`), which `validate_coverage` matched to the clauses in order — never by text. |

Both fixes are gate-verified in this delivery. No finding was a reachable
runtime bypass; the fetch-labor lens confirmed no text reaches the fetch call,
and the V.4/F1 lens confirmed closure (b) and the keyed-intake convergence hold.

### 9.5 Criteria sweep — regenerated

`docs/dev/criteria_sweep.py`: **99 criteria** (PENDING 81, NARROWER 8, MINTED 5,
DEFERRED 1). The Section-J behaviors moved **DEFERRED → PENDING** now that the
collector/concordat/deacon suites cite them; **SC-J08 alone stands DEFERRED** (the
severed Doctor). Only that one deferral remains between Phase B and full
Section-J coverage.

### 9.6 Doc amendment riding with delivery

Doc 07 §C.4 gained the CANON `sources` field (author-sanctioned per the §0.2
ruling): a canon's freeform `demands` clauses stay the coverage surface, and its
`sources` name the concrete v1 targets the trip fetches under the identical
SC-J02 wall — breadth (discovering sources from clauses) stays deferred.
