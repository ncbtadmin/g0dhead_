# Phase B — Slice 1: The Store Substrate & Book I Enforcement
### Pinned scope — signed off 2026-07-07

> Per `00_read_before_development.md` §5: *"The store substrate and the Book I enforcement layer (criteria sections A, B, E, H) are the natural foundation — everything else stands on them. Do not begin with agents; begin with the ground they cannot corrupt."*
>
> This document is the slice's SPEC.md in the dev-pipeline sense. Nothing is built in this slice that does not resolve to a criterion listed here. The verification gate is `cargo test` + `cargo clippy` + `cargo fmt --check`; warnings are fixed, not shipped.

---

## 1. Problem statement

Build the ground the agents cannot corrupt: the single abstracted store interface (doc 3 §1.1) with a Postgres substrate, enforcing the Book I invariants of Laws I–III, VII, and XI–XV at the write layer. No agents, no ML, no intake pipe — only the substrate every later slice stands on.

## 2. Pinned criteria (Document 8, sections A, B, E, H — 21 criteria)

Every criterion seeds at least one test. Test names are the commitment; they may gain companions but never disappear (tests only accumulate).

| Criterion | Enforces | Seed test(s) |
|---|---|---|
| SC-A01 | Forward-only job status transitions; `REFUSED` from any live state | `job_status_forward_only`, `invalid_transitions_rejected` |
| SC-A02 | Retry of `FLAGGED` job writes nothing | `retry_flagged_job_writes_nothing` |
| SC-A03 | Retry with partial outputs converges via `(job_id, output_slot)` keys | `retry_partial_job_converges` |
| SC-A04 | Schema-invalid write rejected atomically, `VALIDATION_FAILED` | `invalid_write_rejected_no_partial` |
| SC-A05 | Out-of-range `schema_version` → `SCHEMA_MISMATCH` before processing | `out_of_range_schema_version_refused` |
| SC-A06 | Envelope completeness on every record | `envelope_completeness_enforced` |
| SC-A07 | Post-FLAG/REFUSED access rejected, logged `severity: violation` | `post_terminal_access_rejected_and_logged` |
| SC-B01 | Flag writable only after certified outputs exist and validate | `flag_before_output_rejected` |
| SC-B02 | Reader re-validation; failure → `FLAG_UNTRUSTED` + `DISTRUSTED` | `distrusted_flag_on_invalid_state` |
| SC-B03 | Flags never deleted; supersession by status only | `flag_deletion_rejected` |
| SC-B04 | No inter-agent API exists; store is the sole surface | `arch_no_agent_channel` (architectural assertion) |
| SC-E01 | RefusalRecord names law, closed reason code, subjects, preserved refs | `refusal_record_complete` |
| SC-E02 | Refusal mutates nothing beyond quarantine marks (pre/post diff) | `refusal_preserves_state` |
| SC-E03 | `REFUSED` ≠ failure in job records and reference metrics | `refused_scored_as_compliance` |
| SC-E04 | Refused partials invisible to downstream readers | `refused_partials_invisible` |
| SC-H01 | Second lease acquisition refuses `LEASE_CONFLICT` immediately | `second_lease_refused_immediately` |
| SC-H02 | Expired-lease recovery converges; CAS race: stale revision loses | `expired_lease_recovery`, `cas_race_harness` |
| SC-H03 | All timestamps store-issued UTC; agent-supplied timestamps rejected | `agent_timestamp_rejected`, `ordering_by_store_sequence` |
| SC-H04 | Anonymous writes rejected; every write attributable | `anonymous_write_rejected` |
| SC-H05 | Budget-less job fails validation; exhaustion refuses `BUDGET_EXCEEDED` | `missing_budgets_rejected`, `budget_exhaustion_refuses` |
| SC-H06 | No secret in any record/log/provenance; alias-only endpoints | `secret_scan_blocks_write` |

## 3. What this slice builds

- **`crates/godhead-schemas`** — the Appendix A record types this slice needs: **A.1 Envelope, A.2 JobRecord, A.3 ReadinessFlag, A.4 RefusalRecord, A.5 LogSnapshot, A.13 LeaseRecord, A.14 ConfigConstant**. Closed enums (`#[non_exhaustive]` is *not* used — enums are closed per Book I conventions and extended only by version bump), UUIDv7 IDs, `schema_name@schema_version` declaration and range validation.
- **`crates/godhead-store`** — the `Store` trait (the sole inter-agent surface) and its Postgres substrate over `sqlx`, enforcing at the write layer: forward-only transitions, idempotent keyed writes, envelope/schema validation, flag-after-output certification, no-delete supersession, refusal-with-preservation, acquire-or-refuse leases with heartbeat/expiry, CAS `revision` integers, store-issued `now()` (client timestamps structurally impossible — the record types offer no timestamp parameter on write), attributable writes (job identity mandatory), mandatory budgets, and the Law XV outbound secret-pattern scan.
- **Migrations** — SQL migrations for the slice-1 tables, run via `sqlx::migrate!`.
- **Tests** — one per seed test name above, against the live Postgres (`DATABASE_URL`). The race harness (SC-H02) uses concurrent tasks against the real database; CAS and clock semantics are only honest against real Postgres.

## 4. Constraints & decisions

- **Language: Rust** (doc 00 §4 override 1). Async via tokio; Postgres via sqlx 0.8; TLS via rustls (Railway requires TLS).
- **Substrate: Railway-hosted Postgres** (user decision, 2026-07-07). Connection string arrives via `DATABASE_URL` in an untracked `.env` (see `.env.example`). Tests are `#[ignore]`-free but skip with a loud message if `DATABASE_URL` is unset — the gate reports skipped steps, never silently passes.
- **Criteria hold across substrate swaps** (Document 8 conventions): all tests exercise the `Store` trait, not the Postgres type directly, so a later self-hosted/local substrate re-runs the identical suite.
- **The store enforces; agents merely comply.** Wherever a criterion allows enforcement at either layer, it lands in the store (VI.3's principle: the guarantee lives in validation, not in the character of the executor).
- **pgvector is not needed in this slice** (embeddings arrive with the ML slice) but the Railway database should be provisioned pgvector-capable so no substrate migration is needed later.

## 5. Non-goals (fenced out of this slice)

- No agents of any kind — no dispatcher, no supervisor, no Deacon (their *store surfaces* — flags, job records, leases — are built; the processes that consume them are not).
- No ML: no embeddings, no links, no matrices, no weights (sections D, M).
- No intake pipe (section N), no quarantine/threshold (section I), no mandates (section J), no Concordat/Instructions (section K), no environments/scriptoria (section G).
- No frontend, no multi-tenancy, nothing deferred by doc 00 §7.
- Schemas A.6–A.12 (provenance, overrides/petitions, environments, matrices, pairings, audit reports, scan verdicts) wait for their slices; the Envelope's `produced_by` reference is typed but its targets beyond JobRecord are not yet built.

## 6. Edge cases the tests must cover

- Crash-shaped retries: re-run after each of `WRITTEN`, `FLAGGED`, partial-output states (SC-A02/A03).
- Out-of-band mutation between flag and read (SC-B02 fixture).
- Concurrent lease/CAS contention under real parallelism (SC-H02).
- Terminal-state access attempts under a dead job's identity (SC-A07).
- Secret-shaped strings in every write path: record bodies, log payloads, refusal details (SC-H06).

## 7. Blockers — resolved 2026-07-07

1. ~~Rust toolchain~~ — installed (rustc 1.96.1); gate operational.
2. ~~`DATABASE_URL` rotation~~ — completed. Note: editing the password variable in Railway does **not** change the live database password (it only seeds first boot); the rotation was finished with `ALTER USER` directly, verified: new credential connects, leaked credential rejected. The leaked string in git history is now inert.

## 8. Delivery — gate passed 2026-07-07

All 21 criteria implemented and green: 25 tests across `tests/a_lifecycle.rs`, `tests/b_handoff.rs`, `tests/e_refusal.rs`, `tests/h_commons.rs`, run against live Railway Postgres. `cargo fmt --check`, `cargo clippy --workspace --all-targets` (deny-level), and `cargo test --workspace` all pass. Implementation notes vs. §3 as designed:

- The generic write surface is the `artifacts` table, keyed `(job_id, output_slot)`; refusal marks rows `authoritative=false, quarantine_marked=true` rather than touching payloads.
- Flag/log/refusal deletion is blocked by a substrate trigger (`godhead_forbid_delete`), below even the store's API.
- Law XIV wall-budget enforcement lives in the actor guard: any identity-bearing call under an exhausted RUNNING job triggers the store-authored BUDGET_EXCEEDED refusal.
- The A.5 event taxonomy gained `JOB_TRANSITION` (v1 addition; Law I.1 requires per-transition snapshots and the Dogma's list lacks an entry for them).
- Post-FLAG, the only permitted act is the transition to TERMINATED; everything else is `TERMINAL_ACCESS` + violation log (Law I.4 reading documented in `job.rs`).

---

*Slice 2 candidates (not pinned, for orientation only): section N intake & endurance atop this substrate, then C (sovereignty) — to be scoped and signed off separately.*
