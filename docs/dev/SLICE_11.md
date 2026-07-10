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
