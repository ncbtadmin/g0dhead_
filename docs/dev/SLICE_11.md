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

---

*§0 recorded 2026-07-09 by sovereign ruling — "the boundary is argued, not
inherited." Sections 1+ (pinned criteria, what-this-builds, riders, non-goals,
gate) pin after the opening adversarial round (condition 1), and carry the F1
keyed-intake rider (condition 2) and any pre-11 riders the round confirms.*
