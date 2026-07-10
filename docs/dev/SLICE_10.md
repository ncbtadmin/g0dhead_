# Phase B — Slice 10: The Deacon's Threshold & the J-Floor
### Pinned scope — presented 2026-07-09 · **signed off 2026-07-09** (with push authorization for the three standing commits, and the delivery-desk note carried into §8/§9)

> Section I of Document 8 entire, plus the substrate of Section J and the
> hardening-rider bundle, exactly as scoped by ruling G9 and conditioned by the
> answered decision sheet (S1: gate scope adopted; S2: C.4 rides here, severable
> only if the budget breaks; S3: content-hash adopted; S4: riders adopted whole).
> Stands on all nine delivered slices. The order's one standing functionary takes
> his post; the first outward act (Slice 11) remains impossible until he does.

## 0. The crossing — Book II §1's day-one designation, adjudicated (F5)

Book II §1 designates the Threshold Protocol **day-one law, non-deferrable** — and
this is the tenth slice. The crossing is adjudicated here, late and on the record
(finding F5; Dogma Standing Notes round 6, item 4): the build order doc 00 §5
fixed — foundation first, *"do not begin with agents; begin with the ground they
cannot corrupt"* — inverted the designation without ever engaging it, and no slice
record faced the words "non-deferrable" until this one. The inversion was
nonetheless safe, for two reasons that were true the whole time and are now
written down. First, dependency order: the Threshold Protocol is store-enforced
law — Law V.4's quarantine wall, A.12's verdict and consent records, admission
into the onboard pipe at its beginning — and every one of those surfaces stands
on the substrate, intake, consent machinery, and scriptorium walls that slices
1–9 built; a day-one Deacon would have guarded a gate with no ground under it.
Second, the no-HTTP wall: no outward transport has ever existed in the workspace
(re-verified at every review round; made an arch test by this slice), so no
external material could reach any boundary while the gate stood unbuilt — the
door the Deacon guards was bricked shut until he could take his post. What
"day-one" governs is not the calendar but the ordering constraint that matters:
**the threshold must stand before the first outward act** — and it will. Slice
11's fetch behavior cannot ship until this slice's gate is green, and the wall
that keeps that true is deletable only by the slice that makes deletion safe, in
the same commit that makes it safe.

## 1. Pinned criteria — Section I + the J-floor (Document 8 as amended 2026-07-09)

| Criterion | Enforces | Seed test |
|---|---|---|
| SC-I01 | External-origin writes land only in the quarantine namespace, rejected anywhere else regardless of writer tier (both `FETCH_PER_CANON` and `FETCH_PER_WRIT` job shapes) | `sc_i01_quarantine_only` |
| SC-I02 | Admission requires `{scan: CLEAN, consent: ADMITTED}` — all four verdict states tested; only CLEAN+ADMITTED passes; INFECTED/SUSPECT/ERROR never presented admissible | `sc_i02_admission_conjunction` |
| SC-I03 | Scan provider unreachable → all items held and flagged, zero admissions, threshold surfaces the failure | `sc_i03_scanner_down_holds` |
| SC-I04 | Admitted items enter the onboard pipe at its beginning — raw copy, first log, normalization all observable; no shortcut path | `sc_i04_no_shortcut_path` |
| SC-I05 | Rejected and held items remain in quarantine, preserved and unpurged, within `quarantine_retention_days` (purge itself is the deferred Duty of the House — the test proves preservation, G13-annotated for the purge half) | `sc_i05_quarantine_preserved` |
| SC-I06 | Architectural: no API path admits unscanned material (SC-B04's mirror for the gate) | `sc_i06_gate_arch` |
| SC-I07a | A write to a sovereign- or office-reserved table whose path did not authenticate as that class is rejected at the substrate, whatever string it stamps — `'sovereign'`, `'deacon'`, `'forged'`, each via raw SQL, each rejected (G10) | `sc_i07a_actor_class_wall` |
| SC-I07b | Items-per-consent or consents-per-window over the operational constants → the Manifest carries a standing notice with petition-style terminal answers (acknowledge / silence, suppressed logging); never blocking, never silent (G11) | `sc_i07b_admission_legibility` |
| SC-J01 | A MandateRecord written under any agent identity is rejected at the store — mandates are human-authored by construction (C.4) | `sc_j01_mandates_human_authored` |
| SC-J02 | **Writ concreteness (sovereign criterion):** query-shaped or unresolvable demands fail at authorship, before any trip — six adversarial fixtures (`"find things about X"`, bare topic, empty locator, malformed URI, unknown `source_id`, search operators) rejected; named resolvable locators validate | `sc_j02_writ_concreteness` |
| SC-J09 *(substrate half)* | The ProvenanceChain table (C.2 shape): an item write whose producing chain entry is absent refuses `PROVENANCE_INCOMPLETE`; roots restricted to `CANON \| WRIT \| BRIEF` — the in-flight *fetch* half re-arms with Slice 11's trips, said so in-test (G13) | `sc_j09_chain_append_substrate` |
| SC-C07 *(threshold entry — claimed)* | "Admitting external material at the threshold" (IV.4): the admission surface takes a human actor string and no job identity; an agent-shaped invocation is rejected `GATE_BYPASS_ATTEMPT` at the wall | `sc_c07_threshold_entry` |
| SC-L01 *(items half — the Return-item hole)* | Certified manifests' `item_ref`/`provenance_ref` fully resolve against the quarantine and chain tables (SLICE_09 §6 finding 7's pin: "full item resolution is pinned as the Deacon's threshold") | `return_items_resolve_at_threshold` |

## 2. What this slice builds

- **The Deacon** (`crates/godhead-deacon`) — the order's one standing
  functionary: an *office*, not an agent. He does not spawn and holds no budgets
  (exempt from Laws I and XIV by Book II); every write law binds him (II, III,
  V, XI, XII, XIII, XV). His writes bear the office identity (`produced_by:
  office_id`, A.1) and authenticate as `office:deacon` under G10. Labors:
  `scan_pass` (walk quarantine, obtain verdicts through the scan trait, persist
  ScanVerdicts), `assemble_manifest` (items + full provenance chains + verdicts,
  **one Manifest per mandate-trip, never pooled** — Book II §1 doctrine),
  `present_manifest` (the sovereign surface), `admit` (CLEAN+ADMITTED items
  enter the onboard pipe at its beginning — reusing slice 2's intake entry, no
  shortcut).
- **The scan endpoint behind a trait** (`ScanEndpoint`): deterministic mock
  provider in-tree (verdict fixtures, unreachable-mode); the real ClamAV daemon
  provider is *not* wired in this slice — the no-HTTP wall stands, and the trait
  is the seam the endpoint slice will fill.
- **Schemas (A.12, C.2, C.4)**: ScanVerdict, QuarantineItem, Manifest (items,
  chains, verdicts, `mandate_ref`, standing-notice field), MandateRecord
  (kind CANON|WRIT, recipient, demands with typed locators `uri | source_id`,
  trip_budget), ProvenanceChain entries (link_seq, kind, actor, prompt_or_reason,
  at, produced).
- **Migrations (planned 0013–0015, exact split at build time)**: quarantine
  namespace + scan_verdicts + quarantine_items + manifests; mandates +
  provenance_chains (append-only, root CHECK, append-in-flight validation at
  item write); the actor-class triggers (G10) on every sovereign- and
  office-reserved table; office identity registration; admission-legibility
  constants (`admission_batch_threshold`, `admission_rate_window`,
  `admission_rate_threshold` — operational tier; the A.14 day-one list
  amendment in doc 05 rides with this slice's delivery).
- **Store methods**: quarantine write path (V.4 wall), `record_scan_verdict`,
  `assemble/get_manifest`, `consent_admission` (human actor string, no job
  identity — the SC-C07 entry), `admit_item` (the conjunction wall + onboard
  handoff), `author_mandate` (human-only, SC-J02 validation at authorship),
  `append_chain_entry` / chain validation at item write, and the Return-item
  resolution extension to `persist_return`/`validate_return`.
- **G10 actor-class authentication**: store API paths set
  `SET LOCAL godhead.actor_class` inside their transactions; reserved-table
  triggers verify session class against table demand and row stamp; below the
  API the variable is absent, so `'deacon'`, `'sovereign'`, and `'forged'` are
  the same object: rejected.

## 3. The hardening riders (S4 — adopted whole, each with tests)

| Rider | Order | Seed test / artifact |
|---|---|---|
| SC-E05: the four swallow sites end in `store.refuse` with propagating errors — `godhead-notary/src/lib.rs:113`, `godhead-audit/src/lib.rs:217`, `godhead-ml/src/aggregate.rs:134`, `godhead-ml/src/slave.rs:155` (SLICE_09 §7 debt); BudgetExceeded guards distinguish already-recorded from failed-to-record | G5, H3(1) | `sc_e05_no_labor_strands` |
| SC-E05: mount failure refuses (`ENV_INVALID` agent-side record — IX.3's labor-rule debt, per G1) | G1/G5 | `mount_failure_refuses_not_strands` |
| SC-E05: a panicking tool `execute()` unwinds into refusal, never a strand (F4-new) | G5 | `panicking_tool_refuses` |
| SC-E05: suite-end sweep — zero jobs RUNNING beyond wall budget | G5 | `sc_e05_suite_end_sweep` |
| Checked budget sum — `checked_add`, refuse on overflow (B1 aggravation: debug-panic strand) | G7/H2-B1 | `budget_sum_checked_refuses` |
| Clause→code map as ONE shared helper in `godhead-schemas` (H4 NEW-1): skew → `SCHEMA_MISMATCH` (returns.rs:395, lint.rs:423); endpoint faults off Law VIII → `(VII, VALIDATION_FAILED)` (aggregate.rs:131) | G1 | `skew_carries_schema_mismatch`, `endpoint_fault_off_law_viii` — closes SC-A05/SC-K03 NARROWER rows |
| Content-hash certification (S3/G7): hash at flag, re-proof at VALIDATE_IN, flagged Instructions AND Returns; field-wise reconstruction retained only for time-varying clauses | G7 | `content_sha_at_flag_reproves_at_read` — closes SC-K04's class |
| Write-side config contracts: per-key type + semantic floor (window ≥ 1 integer; thresholds numeric in [0,1]; roster non-empty string array; ceilings positive per-tier map), registered beside the schema registry, enforced at write; read-side refusals retained as depth (window=0 becomes unrepresentable) | H3(2) | `config_write_contracts` |
| Identity fixes: `record_regular_output` + `raise_bias_warning` take the disclosing job's identity; `release_lease` gains `guard_actor` (XIII.1 means what it says) | H3(3) | `bias_surfaces_identified`, `lease_release_guarded` — heals SC-H05's NARROWER row |
| Env-lease rule, strict XI.1: `add_env_item` and environment status transitions acquire the environment's lease | H3(4) | `env_mutation_under_lease` |
| Arch widenings: SC-D01 scan iterates discovered `crates/*/src`; SC-B04 IPC scan workspace-wide; SC-H06 sweep = `information_schema`-driven × production `secrets::scan` | G2/G3/G4 | `sc_d01_workspace_scan`, `sc_b04_workspace_ipc_scan`, `sc_h06_schema_driven_sweep` |
| SC-H07: workspace fallback-shape arch scan (`unwrap_or*` on config values, all crates) | G2 | `sc_h07_no_fabricated_defaults` |
| **The HTTP wall, mechanical**: no outward-transport dependency in any workspace `Cargo.toml` OR `Cargo.lock`; deletable only by the slice that makes deletion safe, in the same commit | G3 | `no_outward_transport_wall` |
| SC-C07 signature arch-pins: no sovereign surface accepts a job identity (rebalance, audit, proposal/decommission consent) — claims-by-argument become claims-by-test | G6 | `sc_c07_signature_pins` |
| Gate-report producer: `scripts/gate_report.py` runs the three cargo steps, parses output, emits the SKILL.md block (step count, unverifiable line, all fields) — no future block is hand-composed; gate evidence lands in `docs/dev/` | H3(6), C-commission | first producer-emitted report ships with this slice's delivery |
| Test hygiene: `ALTER TABLE DISABLE TRIGGER` fixtures (k_concordat.rs:354, :822) replaced by the planted-row pattern; where a fixture must mutate a *flagged* record in place (the SC-K04 shape), a dedicated non-pool connection under `session_replication_role='replica'` (H6(d) nuance); `sc_m01_triggers` + `sc_k07_pattern_escalation` serialized | H3(5) | `k_concordat` fixtures reworked; serial buckets; gate deterministic |
| SC-A08: provenance-view integrity sweep (every `produced_by` resolves to a live JobRecord or registered office identity; `input_refs` resolve), suite-wide — plus the **one-time archaeology pass** over the existing store (H4 NEW-2), result recorded in this slice's delivery ledger | G8 | `sc_a08_provenance_view_integrity` |

## 4. Design decisions & mechanical nuances

- **G10 costing, verified (H6(e))**: the consent-class sovereign methods already
  run explicit transactions (`lay_category_override` postgres.rs:1768,
  `resolve_proposal` :3063) — `SET LOCAL` drops in directly. Transaction
  wrapping is added **only** on single-statement reserved-table paths
  (`set_config` :1427+) and the new office write paths. The requirement is the
  ruling's; this is the verified cost.
- **The Deacon holds office, not title (X.6), and no model**: plain, auditable,
  hardcoded procedure beneath the vestment. He may not be bypassed, may not
  authorize in the sovereign's place, may not admit the unscanned; no Student
  enters its own findings around him — each of these is a store wall, not a
  convention.
- **Consent binds a specific scan**: admission validates the conjunction against
  the item's *current* verdict chain — consent names its `scan_ref`; a newer
  non-CLEAN verdict on the same item defeats admissibility (edge-cased below).
- **Chain table is external-origin only** (G8): internal provenance remains the
  declared view (Envelope → JobRecord → logs), swept by SC-A08; the table is
  where V.3 and `PROVENANCE_INCOMPLETE` finally construct.
- **Manifest discipline**: `mandate_ref` on every Manifest; one Manifest per
  mandate-trip, never pooled (Book II §1 doctrine, mechanical via uniqueness on
  the trip's execution job); the standing-notice field carries SC-I07b's
  graduated legibility.
- **The labor rule reaches the office**: every Deacon procedure that halts
  mid-write follows the established halt discipline — no silent partial state;
  office procedures are idempotent under retry (the slice-1 idempotency shape).
- **Tests only accumulate**; any criterion satisfied below its words carries the
  G13 annotation naming the unmet half and where it re-arms (SC-I05's purge
  half → Duty of the House; SC-J09's fetch half → Slice 11).

## 5. Non-goals

- **No Section J behavior**: SC-J03/J05/J06/J07/J08/J10 — fetch-execution
  binding, manifest coverage/sought maps, the breadth-creep property test,
  Doctor orphaning — all Slice 11.
- **No fetch layer, no HTTP client, no real scan provider**: the wall stands;
  the trait and its deterministic mock are the whole outward surface this slice
  touches.
- **No Duty of the House** (quarantine purge/rotation — deferred by doc 00 §7).
- **No retrieval breadth, no multi-tenancy** (deferred).
- **The mandate-authoring SC-C07 entry is Slice 11's** (G9): this slice builds
  the `author_mandate` surface and its human-authorship wall (SC-J01), but the
  IV.4 entry's one-test-per-surface obligation is claimed by the slice that
  exercises mandates in behavior — deferral recorded here per the G13
  convention, so no entry silently drops (SLICE_03 §3's standing rule).

## 6. Edge cases

- All four verdict states × both consent decisions: only CLEAN+ADMITTED admits;
  ERROR is held exactly like SUSPECT/INFECTED, never auto-retried into admission.
- Scanner unreachable mid-batch: partial verdicts persist, zero admissions from
  the failed remainder, surfaced once (not once per item).
- Consent scope ITEM vs BATCH: batch consent admits exactly the batch's CLEAN
  members; a non-CLEAN member is never carried in by its batch.
- Re-scan after consent: newer non-CLEAN verdict defeats the stale consent
  (consent names its scan_ref); admission refuses, item stays held.
- Deacon retry: `admit`'s *recording* converges exactly once — `mark_admitted`
  sets `admitted_node_ref` under a `WHERE ... IS NULL` guard and the substrate
  freezes it (Law I.3), so the item is admitted as exactly one node and a
  completed retry reads that node back. The intake *mint* itself is not yet
  keyed: `commit_file` issues a fresh node id each call, so a crash in the
  window between the mint and the recording — or two concurrent `admit` calls
  on one item — can leave a duplicate CLEAN atom orphaned in the corpus (the
  losing `mark_admitted` refuses; the extra node is legitimately-admitted
  content, never unscanned or unconsented, so no threshold invariant is
  touched). Closing that window needs a deterministic, keyed intake entry;
  it is pinned as the sole idempotency-hardening debt of this slice (§9,
  finding F1; G13-annotated), the intake change to be made and reviewed on
  its own rather than rushed through the threshold delivery.
- Writ authorship: the six SC-J02 adversarial fixtures each rejected at
  authorship; a mixed writ (five good locators, one query-shaped) rejected
  whole — no partial mandate persists.
- Chain roots: an item write whose chain roots in FETCH/FOLLOW_UP (no human
  root) refuses; BRIEF-rooted internal chains validate without a MandateRecord
  (verified: JobDraft carries `brief_ref`).
- Actor-class fixtures: `'sovereign'`, `'deacon'`, `'forged'` via raw SQL — all
  rejected below the API; the same strings through the lawful API paths succeed
  for exactly their class's tables.
- A Return citing quarantine items: items resolve or the manifest refuses
  (`return_items_resolve_at_threshold`) — nil floor (slice 9) plus full
  resolution (this slice).

## 7. Budget & severability (S2, recorded as ordered)

The sovereign's S2 answer: **C.4 rides with this slice**. If the slice budget
breaks in practice, the C.4 mandate substrate (MandateRecord + authorship wall +
SC-J01/J02) is the **only severable piece** — it moves whole to Slice 11 and
this file records the move. Severance is safe because BRIEF-rooted chains need
no MandateRecord (verified); under severance, SC-J09's root validation restricts
to BRIEF until the mandates table lands, G13-annotated in-test. Nothing else in
this spec is severable: the Deacon without G10 is an office-shaped hole (F1),
and the riders trace each to a VERIFIED defect or a RULED order — trimming any
one reopens a named hole (S4, adopted whole).

## 8. Gate & delivery protocol

The gate is doc 00 §4's three commands, run on the host per the fourth override,
against live Railway Postgres. **The gate report comes from the producer**
(`scripts/gate_report.py`) from this slice forward — hand-composed blocks are
retired; the report and gate evidence land in `docs/dev/` with the delivery.
Adversarial review precedes delivery (standing rule: anything touching a safety
invariant), and its ledger enters this file's §9 on delivery, alongside the
SC-A08 archaeology result and the regenerated criteria sweep. The two known
flaky tests run in their serial buckets before any regression is suspected.

---

*Presented to the sovereign 2026-07-09. Signed off: 2026-07-09, in session —
"Slice 10 opens as pinned. The code bar lifts for this spec and its riders,
and for nothing else." No line of Slice 10 code, no migration, and no
`Cargo.*` change preceded the signature — the spec crossed the desk before
code moved, as it has for nine slices. On delivery, §9 carries the
adversarial ledger, the SC-A08 archaeology result, the regenerated sweep,
and the first producer-emitted gate report — the delivery crosses the same
desk this spec crossed.*

---

## 9. Delivery ledger (2026-07-09)

Slice 10 delivered on `master`, gate green, against live Railway Postgres. The
delivery crosses the desk the spec crossed (§8): what follows is the account.

### 9.1 Gate — from the producer

`scripts/gate_report.py` ran doc 00 §4's three steps and emitted
`docs/dev/GATE_REPORT.txt` (the first producer report; hand-composed blocks are
retired, H3(6)). Result: **PASS (3 steps)** — `cargo fmt --check` clean, `cargo
clippy --workspace --all-targets` clean (workspace lints deny warnings), `cargo
test --workspace --no-fail-fast` **144 passed, 0 failed, 0 ignored across 44
binaries**, zero DATABASE_URL skips (a skipped-DB run is reported as NOT a full
pass and would fail the producer). The authoritative pass ran clean
multithreaded; no serial rerun was needed.

**A producer correction shipped with this first report.** The multithreaded
`cargo test --workspace` is nondeterministic on the shared live Railway DB: the
singleton-touching tests (bias, rebalance, concordat adoption) can lose a race
under parallel threads — the delivery saw `k_concordat` come back with 8 reds
on one multithreaded pass and **15/15 green single-threaded** (661 s), the exact
shape §8 anticipated. The S4 rider serialized only two named tests; that was
incomplete. Rather than claim a determinism the multithreaded gate does not
have, or crawl the whole suite single-threaded, the producer now runs
multithreaded (fast) and **reruns any red binary in its own serial bucket**
(`--test-threads=1`) before a regression is suspected — clearing it only if it
then passes green, and reporting every serial rerun by name, never silently. A
binary that fails even single-threaded keeps the gate red. This is §8's
"serial buckets before any regression is suspected," made mechanical in the one
place that must never hand-wave a red.

The first cut of that logic had a hole its own first run exposed, and the catch
is worth recording: cargo's default is **fail-fast** — it stops at the first red
test binary — so an early flake aborted the run after two binaries, and the
serial-rerun pass, seeing only that one aborted binary, cleared it and reported
a **false PASS** while forty-odd binaries never executed. The "0 passed; 10
failed across 2 binaries" line did not match a 144-test suite, which is what
gave it away. The producer now runs `--no-fail-fast`, so the whole suite always
executes and every flaky binary is collected for its own serial rerun; a gate
that reports PASS has run everything. Completing the per-test serialization so
even the first multithreaded pass is clean is pinned as test-hygiene follow-up
(it extends the S4 rider that began it).

### 9.2 Adversarial review — partial-automated, completed in-loop

An eight-lens adversarial workflow was launched (each finding to be refuted by a
three-lens skeptic panel — code-trace / reachability / reproduction — majority
CONFIRM required to survive). It **hit the session API limit at 4 of 41 agents**:
four finder lenses (`quarantine-wall-v4`, `mandate-writ-concreteness`,
`riders-integrity-config-identity`, `arch-walls-sc-l01`) and the **entire verify
stage** never ran, so the workflow's own "0 survived / 11 refuted" is an artifact
of the verifiers erroring, not real refutation. The eleven candidate findings
were therefore verified by hand against the code (the author having read every
slice-10 surface first-hand), and the four unrun lenses' surfaces reviewed
directly. Disposition:

| # | Finding (finder lens) | Verdict | Disposition |
|---|---|---|---|
| F1 | `admit` can double-enter intake on crash/concurrent retry — `commit_file` mints a fresh node id, so the convergence witness is set only after the mint (deacon lib) | **CONFIRMED, minor** | Not a threshold bypass (the orphan is legitimately-admitted CLEAN content, never unscanned/unconsented). §6's "keyed writes" claim **corrected** to the true guarantee; the keyed-intake hardening pinned as this slice's sole idempotency debt (G13). |
| F6 | chain-append comment claimed a re-read/convergence that did not exist; a same-seq race surfaced as an opaque `Db` error (postgres.rs) | **CONFIRMED, minor** | **Fixed**: comment corrected; the `(chain_ref, link_seq)` unique-violation now maps to a typed `StaleRevision` (re-read and re-append), never opaque. |
| F7 | `quarantine_deposit`'s convergence log carried the *incoming draft's* metadata, not the standing item's (postgres.rs) | **CONFIRMED, minor** | **Fixed**: the log now names the persisted item's fields; a divergent redeposit can never be what the record shows. |
| F11 | the tool-call ladder lacked the BudgetExceeded skip its four sibling halt sites have → a budget expiry at the persist step double-refused, writing a false Law I.4 Violation and masking BUDGET_EXCEEDED as TERMINAL_ACCESS (toolcall ladder) | **CONFIRMED, minor** | **Fixed**: the skip added verbatim from the reviewed pattern (already-recorded, not failed-to-record — SC-E05/G5); the true reason propagates. |
| F10 | mount's halt handler stamps every non-budget fault `(IX, ENV_INVALID, "failed floor validation")`, mislabeling a rare store-stage fault (scriptorium) | Acknowledged | Ruling G1 **ordered** ENV_INVALID as the mount-failure code (the labor-rule debt IX.3 owed); the coarser categorization is the ruling's, recorded not silently kept. |
| F2 | `config_constants` has no substrate DELETE wall — a raw-SQL DELETE un-sets a sovereign constant | Out of threat model | Below-API raw SQL is explicitly outside the wall's remit (ruling G6: "an attacker with raw SQL was never going to be journaled by the database he is attacking"); the API is the lawful surface, and the G10 class wall guards INSERT/UPDATE. A defense-in-depth `no_delete` on `config_constants` (it supersedes by revision, never deletes) is a reasonable consistency hardening, recorded here, not scope-crept into the threshold delivery. |
| F4 | SC-J09's item↔chain linkage lives in the store API; a raw-SQL insert into `quarantine_items` bypasses it | Out of threat model | Same boundary as F2: the chain's own grammar (root, gapless, immutable) is substrate-hard; the item-write's chain precondition is API-enforced, as the criterion's test exercises. |
| F3, F8 | TRUNCATE bypasses the row-level `no_delete`/preservation triggers | Out of threat model | A universal Postgres property (TRUNCATE fires no row-level BEFORE DELETE trigger) at the same privilege tier as `DROP TABLE`; the whole codebase's preservation rests on row-level triggers equally. Recorded, not fixed. |
| F5 | `append_chain_entry` had no job-status gate, so a post-homecoming job could reconstruct a chain | **REFUTED** | `guard_actor(_, _, false)` blocks `Flagged`/`Terminated`/`Refused`; only in-flight jobs (Draft/Queued/Leased/Running) append — exactly append-in-flight. The finder missed that `permit=false` blocks `Flagged`. |
| F9 | the slave halt site demotes a failed refusal write to an ignorable summary string, stranding a RUNNING job while the tick returns Ok | **REFUTED** | slave.rs propagates the failed refuse via `.map_err(...)?`; `backfill_tick`'s per-node containment is documented by-design, and a failed refuse (DB unreachable) is caught by lease-TTL recovery and the SC-E05 suite-end sweep. |

No finding was a reachable safety-wall bypass via the lawful API; the gate was
green before and after the three fixes. The four unrun finder lenses were
reviewed by hand: the G10 class wall covers every reserved-table INSERT/UPDATE
path inside a `set_actor_class` transaction (no self.pool reserved-table write
escapes it); `validate_locator_shape` rejects the query-shaped set and the
resolution half rejects unknown source ids against the (empty, v1) registry;
the write-side config contracts and content-hash paths hold; the arch walls
(no-outward-transport over Cargo.toml **and** Cargo.lock, the workspace scans)
assert non-vacuously. A full re-run of the automated panel over the fix diff is
available once the session window resets, but the fixes are localized,
non-safety-invariant legibility corrections gate-verified here; dedicated tests
for F6/F7/F11 are pinned (they extend patterns already tested at their sibling
sites).

### 9.3 SC-A08 archaeology — the one-time whole-store pass (H4 NEW-2)

The in-gate `sc_a08_provenance_view_integrity` is watermark-scoped; the delivery
runs the same sweep over the **entire** historical store (31 envelope-bearing
tables, 21,170 jobs, 17,703 input_refs walked). Result: **0 `produced_by`
violations** — every stamp resolves to a live JobRecord, the registered
`office:deacon`, or a human/deployment actor string by shape. **14 dangling
input_refs** surfaced (0.08%), and each was characterized: **all 14 sit on
STUDENT/REFUSED jobs**, each citing a sibling-timestamped id that resolves in no
table (not even the union-omitted ones) — the exact fingerprint of negative-path
refusal tests, where a Student job is refused *because* its cited input never
resolved. This is benign cross-binary test residue accumulated over nine slices
on the shared live DB; the watermark-scoped in-gate test correctly never sees it.
An observation, not a fix: the SC-A08 invariant as literally worded ("every
input_ref resolves") has a legitimate exception — a job refused for an
unresolvable input — which the watermark scoping sidesteps in-gate; the
whole-store view records it plainly.

### 9.4 Criteria sweep — regenerated

`docs/dev/criteria_sweep.py` regenerated `CRITERIA_SWEEP.md` against HEAD: **99
criteria** (PENDING 75, NARROWER 8, DEFERRED 7, MINTED 5). Section I and the
J-floor criteria moved **DEFERRED → PENDING** now that the deacon suite cites
them (`i_deacon.rs`, `j_floor.rs`); the five MINTED riders (SC-E05, SC-A08,
SC-H07, SC-I07a, SC-I07b) carry their construction in this slice and now show
their citing tests. The remaining DEFERRED are the unbuilt Section-J behaviors
(Slice 11).

### 9.5 Doc amendments riding with delivery

Doc 05 gained its **round-7** Standing Note: the admission-legibility trio
(`admission_batch_threshold` 50, `admission_rate_window_ms` 3,600,000,
`admission_rate_threshold` 5) joins A.14's day-one operational list (ruling G11;
SC-I07b) — thresholds of notice on the Manifest, never of blocking, operational
under all three tier tests. This is the A.14 amendment §2 promised would ride
with the slice's delivery.
