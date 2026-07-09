# g0dhead_ — Document 8
# Phase B Success Criteria
### The Testable Spec — v1.0 (Phase A deliverable)

> Every criterion is a verifiable assertion that seeds at least one test in Phase B's verification gate (`cargo test` + `cargo clippy` + `cargo fmt --check`; warnings fixed, not shipped). IDs are stable and citable. Sources: Central Dogma v1.0, Holy Standard v1.0, Student Handbook v1.0 (canonical, promoted 2026-07-07), docs 1–4 as amended. Conventions: "rejected" means the store refuses the act — the write does not occur and the attempt surfaces as the named error (logged `severity: violation` where an API-layer observer with a live identity exists); "refuses," of an agent, means the Law VII procedure with a persisted RefusalRecord (ruling G1); "the store" means the abstracted store interface — criteria hold across substrate swaps. Sections F and J carry adversarial suites by sovereign directive.

---

## A. Lifecycle & Contract (Laws I–II)

- **SC-A01** — A job's status transitions only forward along `PENDING → LEASED → RUNNING → WRITTEN → FLAGGED → TERMINATED`, with `REFUSED` reachable from any live state; any other transition is rejected by the store. (I.1)
- **SC-A02** — Retrying a job whose record reads `FLAGGED` performs zero writes and terminates. (I.3)
- **SC-A03** — Retrying a job with partial outputs converges: post-retry state is identical to a single clean run (writes keyed `(job_id, output_slot)`). (I.3)
- **SC-A04** — A write failing its declared schema validation is rejected with `VALIDATION_FAILED`; no partial write persists. (II.2)
- **SC-A05** — Input bearing an out-of-range `schema_version` → `SCHEMA_MISMATCH` refusal before any processing; no best-effort parse path exists. (II.4)
- **SC-A06** — Every persisted record carries a complete Envelope; the store rejects records missing any envelope field. (Book I conventions)
- **SC-A07** — After FLAG or REFUSED, further store access under that job's identity is rejected and logged `severity: violation`. (I.4)
- **SC-A08** — Every persisted record's `produced_by` resolves to a live JobRecord or a registered office identity, and that JobRecord's `input_refs` resolve; swept suite-wide. (V.2 as dispersed — Dogma Standing Note r6; minted by ruling G8)

## B. Handoff (Law III)

- **SC-B01** — A readiness flag is writable only after its certified outputs exist and validate; a flag-before-output write is rejected. (III.2)
- **SC-B02** — A reader encountering a flag whose underlying state fails validation (fixture: mutate state out-of-band after flagging) refuses with `FLAG_UNTRUSTED` and sets the flag `DISTRUSTED`. (III.3)
- **SC-B03** — Flags are never deleted: deletion attempts are rejected; supersession occurs only by status transition. (III.4)
- **SC-B04** — Architectural: no API exists by which one agent process addresses another; the store interface is the sole inter-agent surface (compile-time/arch test). (III.1)

## C. Human Sovereignty (Law IV)

- **SC-C01** — Mutating a record bearing `user_overridden: true` without a resolving `consent_ref` is rejected at the store layer, regardless of writer identity. (IV.1)
- **SC-C02** — A recurring petition on the same `(subject_ref, change_kind)` increments `occurrence_count` and transitions `OPEN → ESCALATED`, presenting the three terminal answers. (IV.2)
- **SC-C03** — `SILENCED` suppresses future matching petitions; suppressed attempts are still logged at `severity: suppressed` and never purged. (IV.2, IV.3)
- **SC-C04** — Executing a `GRANTED` petition lays a successor override: result reads `user_overridden: true`, `basis: GRANTED_PETITION`, with `prior_ref` and `consent_ref` resolving. (IV.5)
- **SC-C05** — Post-grant, an ordinary agent mutation of the granted datum is rejected — protection persisted through the grant. (IV.5)
- **SC-C06** — A `GRANTED` petition with no completed Notary execution job within the stall window is surfaced by the supervisor. (IV.5)
- **SC-C07** — Each entry in IV.4's closed list — including **authoring fetch mandates** — has no agent-callable path; an attempted invocation is rejected `GATE_BYPASS_ATTEMPT` — and logged `severity: violation` where an API-layer observer with a live identity exists; the wall does not keep a diary (one test per entry-surface, once the surface exists). (IV.4; narrowed by ruling G6)

## D. The Commitment Chain (Law VI; Aggregators, Auditors, Notaries)

- **SC-D01** — A link-density evaluation lacking `config_rev` fails validation; no hardcoded threshold exists anywhere in agent code (arch/grep test + runtime assertion). (VI.1)
- **SC-D02** — An Aggregator pass finding density ≥ threshold creates exactly one Postulant record plus one audit-eligibility flag; below threshold, neither; weights over the cluster become live at the same event and are inert below it. (VI.2)
- **SC-D03** — **Fiat-impossibility:** no write path sets matrix status `CARDINAL` without a resolving, cross-referencing `proposal_ref → consent_ref → act` chain; a direct status write is rejected regardless of writer. (VI.3)
- **SC-D04** — Audit spawns Gabriel's and Lucy's jobs with identical input and no read access to each other's in-progress reports (pre-barrier cross-read is rejected). (Book II §2)
- **SC-D05** — Reconciliation is not dispatchable until both audit flags are present AND both underlying reports validate; either missing or invalid holds the barrier. (Book II §2; doc 3 §3.3 as amended)
- **SC-D06** — An audit report containing any claim whose `evidence_refs` fail to resolve fails VALIDATE_OUT and never flags (truth-binding). (Book II §2)
- **SC-D07** — On `AMEND` + consent, the Notary applies exactly the enumerated changes — a structural diff between rev N and N+1 shows those changes and nothing else. (VI.4)
- **SC-D08** — Recursion reaches fixpoint: a zero-amendment `COMMIT` closes the loop; every cycle logs its depth; sovereign halt exits cleanly at any depth. (VI.4)
- **SC-D09** — Decommission requires human consent and a Notary execution; the dissolved matrix's links persist. (VI.5)
- **SC-D10** — A Notary presented with any consent whose reference chain does not resolve refuses; with a valid chain it executes exactly once (idempotent under retry per SC-A03) and writes provenance linking every reference. (Book II §3)

## E. Refusal (Law VII)

- **SC-E01a** — An enacted refusal's RefusalRecord persists complete: the law, a reason code from the closed enum, subject refs, and preserved refs. (VII.2; the shape half of former SC-E01, split by ruling G5 — the universal is SC-E05)
- **SC-E02** — A refusing agent performs no mutation of the offending state beyond quarantine-marking (diff test: pre/post refusal state identical except marks). (VII.3)
- **SC-E03** — `REFUSED` is distinct from failure in job records; the reference metrics query scores refusals as compliance, not error. (VII.4)
- **SC-E04** — Partial outputs of a refused job are non-authoritative: downstream reader validation rejects them. (VII.5)
- **SC-E05** — Any labor halting after RUNNING ends REFUSED with a persisted RefusalRecord; a failed refusal write propagates as a hard error; no sequence of inputs can suppress the record. Sweep half: at suite end, zero jobs stand RUNNING beyond their wall budget. (VII.1; minted by ruling G5)

## F. Tool-Calling (Law VIII) — adversarial-heavy by directive

- **SC-F01** — Arguments failing the tool's schema are never executed; the repair ladder engages with the validator's errors fed back verbatim. (VIII.2–3)
- **SC-F02** — A hallucinated tool name is invalid; the valid-tool roster is fed back; after `tool_repair_attempts` the call refuses `TOOL_MALFORMED`. (VIII.2–3)
- **SC-F03** — Missing required field, wrong type, and prose-where-structure-required each independently render a call invalid and unexecuted (three fixtures). (VIII.2)
- **SC-F04** — Repair attempts are capped at the config value and counted in the job record; exhaustion refuses — no unbounded repair loop is possible. (VIII.3)
- **SC-F05** — Tool output failing its declared output schema is never consumed; an idempotent-marked tool is re-executed exactly once; a non-idempotent tool refuses immediately with `TOOL_OUTPUT_INVALID`. (VIII.4)
- **SC-F06** — Structured/grammar-constrained generation is enabled for tool-argument production whenever the serving endpoint supports it (config assertion + integration test against the local endpoint). (VIII.1)
- **SC-F07** — **The gauntlet:** against a mock model emitting, in sequence and shuffled — a valid call; malformed JSON; schema-invalid JSON; pure prose; prose wrapping JSON; a hallucinated tool; extra unknown fields; null for a required field — every case terminates in either a validated execution or a clean refusal. **Zero malformed executions across the suite** is the pass condition. (VIII.5)
- **SC-F08** — Property test: no sequence of malformed model outputs, of any length, can cause a tool execution — a rejected call has zero observable side effects (side-effect audit harness). (VIII.2)
- **SC-F09** — Every executed call's arguments and outputs are attributable in provenance: `job_id` and `endpoint_alias` present and resolving. (VIII; XIII.2)
- **SC-F10** — Outbound writes containing secret-shaped strings (fixture set: key formats, tokens) are refused and logged `severity: violation`; no secret reaches store, log, or provenance. (XV.1–2)

## G. Scriptoria & Titles (Laws IX–X)

- **SC-G01** — Mounting an environment that fails floor validation refuses `ENV_INVALID`; no work occurs. (IX.3)
- **SC-G02** — An environment-bound agent's read outside its contents index — allowlist excepted (schemas, config, own job/lease records) — is rejected by the store and logged. (IX.4)
- **SC-G03** — The Pairing Exception grants exactly flagged handoff artifacts: paired agent reads counterpart's flagged artifact (allowed), counterpart's unflagged draft (rejected); an unpaired agent is rejected for both. (IX.5)
- **SC-G04** — Conferral is deterministic and reproducible: recomputing `hash(env_id) mod roster_length` from the record alone reproduces the styled name; a collision with a living environment takes the ordinal. (X.1, X.4)
- **SC-G05** — A Teacher environment whose tier and title disagree fails validation; a pairing record naming a REGULAR tier fails validation. (X.2, X.5)
- **SC-G06** — Mount-validation walks the provenance surface: one item whose chain fails to resolve root-to-leaf renders the whole room `ENV_INVALID`. (Handbook §2.2)
- **SC-G07** — An `ORPHANED` environment is readable as archive and unmountable for work; no write against it succeeds. (A.8; Standard §4.3)

## H. The Commons (Laws XI–XV)

- **SC-H01** — A second lease acquisition on a leased subject refuses `LEASE_CONFLICT` immediately — no waiting, no spinning; the dispatcher reschedules. (XI.1)
- **SC-H02** — An expired lease over an unfinished job routes to recovery and converges per SC-A03; concurrent writers under CAS: the stale revision loses and re-reads, never overwrites (race harness). (XI.2–3)
- **SC-H03** — All persisted timestamps are store-issued UTC; a record carrying an agent-supplied timestamp is rejected; ordering assertions use store sequence. (XII)
- **SC-H04** — An anonymous write (missing job identity) is rejected at the store; every write in a full pipeline run is attributable. (XIII.1)
- **SC-H05** — A job spawned without budgets fails validation; budget exhaustion mid-work refuses `BUDGET_EXCEEDED`, releases leases, marks partials non-authoritative, terminates. (XIV)
- **SC-H06** — No credential or endpoint key appears in any store record, log, provenance, or model-context fixture across a full pipeline run (scan assertion); endpoints are referenced by alias only. (XV.1)
- **SC-H07** — A config-constant read whose value is absent or fails to parse as its expected type refuses; no code path substitutes a fabricated default for a sovereign or operational constant. Arch half: a workspace-wide scan rejects fallback-shaped extraction (`unwrap_or`/`unwrap_or_else`/`unwrap_or_default`) applied to config values, all crates. (Law II.2 applied to config; A.14; minted by ruling G2)

## I. The Deacon's Threshold

- **SC-I01** — An external-origin write outside the quarantine namespace is rejected regardless of writer tier (tested under both `FETCH_PER_CANON` and `FETCH_PER_WRIT`). (V.4)
- **SC-I02** — Admission requires `{scan: CLEAN, consent: ADMITTED}`: all four verdict states tested; only CLEAN+ADMITTED passes; INFECTED/SUSPECT/ERROR are never presented as admissible. (Book II §1)
- **SC-I03** — Scan provider unreachable: all items held and flagged, zero admissions, threshold surfaces the failure. (Book II §1, failure behavior)
- **SC-I04** — Admitted items enter the onboard pipe at its beginning: raw copy, first log, normalization all observable — no shortcut path exists. (Book II §1, step 6)
- **SC-I05** — Rejected and held items remain in quarantine, preserved and unpurged, within `quarantine_retention_days`. (Book II §1, step 7)
- **SC-I06** — Architectural: no API path admits unscanned material (mirror of SC-B04 for the gate). (Book II §1)
- **SC-I07a** — A write to a sovereign- or office-reserved table whose path did not authenticate as that class is rejected at the substrate, whatever string it stamps — fixtures: `'sovereign'`, `'deacon'`, `'forged'`, each via raw SQL, each rejected. (XIII.1; Book II; minted by ruling G10)
- **SC-I07b** — When items-per-consent or consents-per-window exceed the operational constants (`admission_batch_threshold`, `admission_rate_window/threshold`), the Manifest carries a standing notice with the petition-style terminal answers (acknowledge / silence, suppressed logging). Never blocking; never silent. (Book II §1 doctrine; minted by ruling G11)

## J. The Mandate Rule & the Loops — adversarial-heavy by directive

- **SC-J01** — A MandateRecord written under any agent identity is rejected at the store: mandates are human-authored by construction. (IV.4; C.4)
- **SC-J02** — **Writ concreteness (sovereign criterion):** a writ demand that is query-shaped or unresolvable fails validation *at authorship, before any trip*. Adversarial fixtures: `"find things about X"`, a bare topic string, an empty locator, a malformed URI, an unknown `source_id`, a locator field containing search operators. A writ of named, resolvable locators validates. This criterion **is** the enforced boundary between the writ system and the deferred breadth system. (C.4)
- **SC-J03** — `FETCH_PER_CANON` / `FETCH_PER_WRIT` execution lacking a resolving `mandate_ref` refuses; mandate kind must match tier (WRIT→Devout, CANON→Canon; cross-matches rejected). (§1.4; B.3)
- **SC-J04** — An Instruction containing any fetch step fails lint clause (f) and is never written (v1 prohibition). (Standard §1.3f)
- **SC-J05** — Property test: no writ text, however adversarially worded, can produce an outward fetch to a locator not enumerated in its demands — the fetch layer resolves targets only from the validated mandate, never from free text. (C.4; §1.4)
- **SC-J06** — CollectionManifest: every collected item maps to a writ target by `target_index`; an unmapped item fails manifest validation (no padding); an unmet target carries empty `item_refs` and is flagged. (C.5)
- **SC-J07** — CorpusManifest coverage: every canon clause maps; unmet clauses trigger the gap duty — the Student refuses, flagging exactly the unmet clauses, and writes nothing sourced outside the canon. (§1.3; C.3)
- **SC-J08** — Doctor deployment with `student_env_ref` not `LIVE` refuses `ENV_INVALID`; dissolving the Canon scriptorium orphans the Doctor's; no silent revival — a fresh Canon environment does not re-enable the orphaned Doctor without a new sovereign pairing. (Standard §4.3)
- **SC-J09** — Chain-append in flight: an item write whose producing ProvenanceChain entry is absent refuses `PROVENANCE_INCOMPLETE`; chain roots are restricted to `CANON | WRIT | BRIEF`. (Handbook §4.2; C.2)
- **SC-J10** — A collected item the system cannot normalize is stored raw, marked `normalizable: false`, and surfaces an incompatibility notice; fetch-layer garbage (corrupt, deceptive, unfetchable) is refused at source, not laundered through quarantine. (Handbook §4.4; doc 2 §2.4)

## K. The Teacher's Lint & the Concordat

- **SC-K01** — Each Executability Lint clause (a)–(f) has a violating fixture that prevents the Instruction from being written; a fully conforming Instruction passes (seven tests). (Standard §1.3)
- **SC-K02** — A `SOVEREIGN_JUDGMENT` criterion is excluded from the executor's self-check; its Return entry reads `passed: null` with mandatory evidence; an Instruction with zero machine-checkable criteria fails lint. (§1.3d)
- **SC-K03** — Concordat version skew in either direction refuses `SCHEMA_MISMATCH` at the skewed end; every cited Concordat version remains retrievable forever. (§2.4, §3.3)
- **SC-K04** — Double-validation: an Instruction corrupted between Teacher-flag and Student-read is caught by the Student's VALIDATE_IN (out-of-band mutation fixture). (§2.3)
- **SC-K05** — A flagged Instruction is immutable: edits are rejected; correction flows through `supersedes_ref` and the chain resolves. (§1.4)
- **SC-K06** — A Regular Teacher output missing `sources_drawn` fails validation; `skew` computes correctly against fixture draws and `bias_skew_threshold`. (§6.3)
- **SC-K07** — Pattern escalation: crossing `bias_pattern_threshold` over `bias_pattern_window` raises exactly one standing warning; *acknowledge* keeps counting; *silence* suppresses with `severity: suppressed` logging and no re-raise until lifted. (§6.3)

## L. Student Returns & Stewardship

- **SC-L01** — ReturnManifest completion carries exactly one entry per acceptance criterion — missing or extra entries invalidate; evidence is mandatory for passed, failed, and deferred alike. (B.2)
- **SC-L02** — Devout consolidation over a matrix containing human-held data leaves that data untouched (structural diff) and may petition it, never write it. (Handbook §4.5)
- **SC-L03** — Redundant-consistency: re-running a refinement from its recorded derivation reproduces the artifact; a dangling intra-scriptorium reference fails the closure check. (Handbook §1.2)
- **SC-L04** — The unbound are unnamed: Regular Students and Regular Teachers carry no conferral; a conferral record for a REGULAR tier fails validation. (X.1)

## M. Weights & the ML Floor

- **SC-M01** — Ingestion without a configured trigger marks recalculation-eligibility and performs zero recalculation; each configured trigger kind (manual, on-add, interval) executes it. (doc 4 §5.2 as amended, §6.4)
- **SC-M02** — Below the coherence threshold, weights are inert: a sub-threshold cluster's weights exert no force in any consumer query. (doc 4 §5.4; VI.2)
- **SC-M03** — Mode dial: the same ingestion completes under assisted and numerical-floor modes; floor mode makes zero reasoner calls (call-count assertion). (doc 4 §5.3)
- **SC-M04** — Empty endpoint roster: every stage completes or degrades to its floor with zero crashes — "no model" is routing, not error. (doc 4 §2.4)
- **SC-M05** — One persisted embedding per node: a repeat request reads, never recomputes (embedder call-count assertion). (doc 3 §2.2)
- **SC-M06** — Embedder-down intake: the file rests normalized and linkless, flagged for backfill; later embedding backfills without touching the raw atom. (doc 2 §1.2 as amended)

## N. Intake & Endurance

- **SC-N01** — Raw is copied exactly once: the atom's checksum is stable across the full pipeline, audit, and commitment cycle; no downstream process rewrites it. (doc 2 §1.3; doc 3 §4.2)
- **SC-N02** — The first log snapshot writes on raw copy with all required fields; rotation preserves priors — nothing overwrites, ever (log-chain walk). (doc 2 §2.2; V.1)
- **SC-N03** — A decode failure is logged and flagged, the file stored — never silently accepted; an unsupported type is stored raw with an incompatibility notice — never rejected. (doc 2 §2.3–2.4)
- **SC-N04** — The seam holds: after at-rest, no further job dispatches absent human invocation or a configured trigger (observation-window test). (doc 2 §4.1 as amended)
- **SC-N05** — Kill-and-restart at every stage boundary: the supervisor reconstructs its index from flags and job records; the pipeline resumes; the end state equals an uninterrupted run (one test per boundary). (doc 3 §4.1)
- **SC-N06** — Derivative regeneration: discard a derivative, re-derive from raw — provenance updates, nothing else changes, no data loss. (doc 3 §4.3)

---

## ADVERSARIAL EMPHASIS (by sovereign directive)

Sections **F** and **J** are not satisfied by example-based tests alone. Phase B must give both property-based/fuzz treatment, with these two properties as the headline invariants:

1. **No sequence of model outputs, of any length or malformation, can cause a tool execution that did not pass schema validation** (F: SC-F07/F08 generalized).
2. **No mandate text, however adversarially worded, can widen the set of locators actually fetched beyond the validated demands of a human-authored mandate** (J: SC-J02/J05 generalized).

Everything else in this document defends the store from bad state; these two defend it from bad *minds* — the weak local model and the vaguely-worded errand being the two most likely real-world doors. Test them like doors.

*End of the testable spec. Phase A closes here; Phase B builds against this document and the ratified drafts it cites.*

---

**AMENDMENTS — 2026-07-09** *(post-ratification, per the author's rulings in `docs/dev/PROMPT_G_RULINGS.md` — sha256 `f8630c6fbd6c6a1a7532cc6f323e65b91435cd9a5ed4e957770078c860ab89c0` — adopted by sovereign decision S5).* Preamble: the two-verb convention (G1) replaces the conflated "rejected" sentence — the wall rejects, the agent refuses. SC-C07's log clause narrowed to API-layer observers (G6: the wall does not keep a diary). SC-E01 split (G5): SC-E01a carries the shape half; the universal moved to minted SC-E05. Minted this round: SC-A08 (G8), SC-E05 (G5), SC-H07 (G2), SC-I07a (G10), SC-I07b (G11) — each cites its construction site in `docs/dev/CRITERIA_SWEEP.md`, which is generated, not kept. No count is stated in prose: the file is the count (G12).