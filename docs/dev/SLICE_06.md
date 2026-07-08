# Phase B — Slice 6: The Tool-Call
### Pinned scope — signed off 2026-07-07

> Section F of Document 8: Law VIII — a tool call is a handoff wearing
> gloves. Adversarial-heavy by sovereign directive: this slice carries one
> of the spec's two headline invariants — **no sequence of model outputs,
> of any length or malformation, can cause a tool execution that did not
> pass schema validation.** These defend the store from bad *minds*; the
> weak local model is the most likely real-world door. Test it like a door.

## 1. Pinned criteria (Document 8, section F — 10 criteria)

| Criterion | Enforces | Seed test |
|---|---|---|
| SC-F01 | Schema-failing args never executed; ladder engages with errors fed back verbatim | `sc_f01_ladder_engages` |
| SC-F02 | Hallucinated tool → invalid; roster fed back; exhaustion refuses TOOL_MALFORMED | `sc_f02_hallucinated_tool` |
| SC-F03 | Missing field / wrong type / prose-where-structure: three fixtures, each invalid and unexecuted | `sc_f03_three_fixtures` |
| SC-F04 | Repair attempts capped at config, counted on the record; no unbounded loop | `sc_f04_repair_cap` |
| SC-F05 | Invalid output never consumed; idempotent → exactly one re-execution; non-idempotent → immediate refusal | `sc_f05_output_validation` |
| SC-F06 | Constrained generation enabled whenever the endpoint supports it | `sc_f06_constrained_generation` |
| SC-F07 | The gauntlet: eight cases, shuffled — zero malformed executions | `sc_f07_gauntlet` |
| SC-F08 | Property/fuzz: no malformed sequence of any length causes an execution; zero side effects | `sc_f08_no_execution_property` |
| SC-F09 | Executed calls attributable: job_id + endpoint_alias present and resolving | `sc_f09_provenance` |
| SC-F10 | Secret-shaped outbound writes refused and logged; nothing reaches store/log/provenance | `sc_f10_secret_scan` |

## 2. What this slice builds — `crates/godhead-toolcall`

- **The Tool contract**: `Tool` trait — name, description (fed back in the
  roster), `idempotent` marker, input validator, output validator,
  `execute`. A `ToolRegistry` holds the roster. Test tools carry atomic
  execution counters: the side-effect audit is the counter.
- **The ToolCaller seam**: the trait a serving endpoint implements —
  `propose_call(context, feedback, constrained) -> raw text` plus
  `supports_constrained()`. v1 ships scripted/mock callers (there is no
  live reasoner endpoint yet); Law VIII.5 is honored structurally: the
  ladder is identical for every caller — the ladder IS the leniency.
- **Strict parsing (VIII.2, II.2)**: a raw emission is a call iff its
  trimmed entirety parses as a JSON object with exactly the keys `tool`
  (string) and `arguments` (object). No extraction from prose, no repair,
  no best-effort — prose wrapping perfect JSON is as invalid as noise.
- **The recovery ladder (VIII.3)**: on an invalid call, the validator's
  errors and the valid-tool roster are fed back verbatim and the call is
  regenerated, at most `tool_repair_attempts` times (config, seeded 2);
  exhausted → Law VII refusal, reason TOOL_MALFORMED, attempts on the
  record (refusal detail; the success path records attempts in the
  `tool_call` artifact).
- **Output validation (VIII.4)**: invalid output → one re-execution iff
  the tool is marked idempotent; otherwise, and on second failure, refuse
  TOOL_OUTPUT_INVALID. Consumed output is persisted as the job's
  `tool_call` artifact — which passes the store's Law XV secret scan or
  dies trying (SC-F10 rides the slice-1 machinery).
- **The fuzz harness (SC-F08)**: a seeded deterministic generator (no new
  dependencies) producing hundreds of malformed sequences — random noise,
  near-miss mutations of a valid call (dropped fields, wrong types,
  renamed tools, prose prefixes, injected keys, nulls) — each driven
  through a fresh job. Pass condition: zero tool executions, zero
  artifacts, every job ends in a clean refusal.

## 3. Design decisions

- **SC-F06 scope**: no live serving endpoint exists in v1, so "enabled
  whenever the endpoint supports it" is enforced at the harness seam: the
  ladder passes `constrained=true` to any caller that declares support,
  asserted against mock callers both ways. The integration half of SC-F06
  re-arms when a real local endpoint (Ollama) lands — noted as a pinned
  residue, not silently dropped.
- **Attempt counting**: "counted in the job record" is realized as the
  job's own records — the `tool_call` artifact (success) or the
  RefusalRecord detail (exhaustion), both attributable to the job_id.
  A.2's `attempt` field remains the dispatcher's invocation counter.
- **Unknown argument fields** are the tool validator's to reject; the
  slice's test tools reject them (strict-by-default is the recommended
  posture the Concordat's capability tables will inherit).

## 4. Non-goals

- No live reasoner/LLM endpoint, no Ollama integration.
- No agent manuals (K/L) — this is the substrate their tool use runs on.
- No multi-tool transactions or tool-call chains; one call, one ladder.

## 5. Edge cases

- Repair that succeeds on the last permitted attempt.
- Valid call naming a tool whose input validator panics — a validator
  defect must not become an execution (caught as invalid).
- Idempotent re-execution that fails twice → refusal, two executions max.
- Empty roster: every call is a hallucination; refusal, never a crash.

## 6. Delivery — gate passed 2026-07-07

All 10 criteria green + 4 review regression tests (`tests/f_toolcall.rs`,
14 tests); slices 1–5 unregressed — 69 tests workspace-wide; fmt/clippy/
test clean. The headline invariant (SC-F08) is proven by a 150-seed fuzz
over 18 malformation shapes × length variation: zero executions, zero
artifacts, across every seed.

**Adversarial review (18 agents, 3 lenses × verify): 15 findings — 3
refuted, 12 confirmed (collapsing to 5 distinct defects), all fixed
before delivery.** The ledger:

1. *A model could suppress its own refusal record* (HIGH) — the ladder
   echoed the model's tool name into the refusal detail; a secret-shaped
   name tripped the store's Law XV scan; `let _ =` swallowed the error,
   leaving the job RUNNING with no record. Root-cause fix: **no
   model-controlled text ever reaches a persisted field** — validator
   errors are model feedback only (in-memory); persisted refusal details
   are law-referencing and count-only. And `refuse` now propagates store
   errors instead of swallowing them — "a RefusalRecord is on the record"
   is a promise the outcome keeps. Regression:
   `secret_shaped_tool_name_still_refuses_on_record`.
2. *A panicking validator was not caught* (MEDIUM) — `validate_input`/
   `validate_output` now run under `catch_unwind`; a validator defect is
   an invalid call, never an execution. Regression:
   `panicking_input_validator_caught_as_invalid`.
3. *u32/u64 counter mismatch* (LOW) — a `tool_repair_attempts > u32::MAX`
   could wrap the counter into an infinite loop. Attempts are now u64 and
   the cap is clamped to a ceiling (`REPAIR_CAP_CEILING`).
4. *Weak/absent tests* (SPEC) — verbatim-feedback now asserted (SC-F01);
   SC-F07 emits its eight cases in-sequence, shuffled, with a suite-wide
   zero-malformed-execution ledger; SC-F04 reads the persisted refusal;
   last-attempt-recovery and empty-roster edge cases now tested.
5. *Attempt accounting* (LOW) — the output-invalid refusal detail now
   carries the repair count too.

Pinned residue: the integration half of SC-F06 (a live local endpoint
honoring the grammar) re-arms when Ollama lands — §3.
