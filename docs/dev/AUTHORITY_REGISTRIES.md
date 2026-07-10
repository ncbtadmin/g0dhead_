# Application Authority Registries

**Status:** Analysis-only; non-canonical and proposal-only until D7 returns.

- **Scope:** Application boundary, authentication and authority classes, reserved-operation and authentication-control registries, command envelopes and receipts, and served executor discovery.
- **Owning decisions:** D7; D1/D4 where the post-join checkpoint consumes the authority substrate; D8 for override commands.
- **Phase owner:** P2A for the minimum authentication and command substrate; P2B for the complete application and ceremony surface; later rows remain registered-and-refusing until their phase.
- **Criteria hooks:** SC-C06â€“SC-C07, SC-D03, SC-I02/SC-I07b, SC-J01, SC-K07, and the application AC- authority criteria.
- **Amendment rows sourced:** D7 authority, refusal, receipt, notice, judgment, and taxonomy rows in [AMENDMENT_MATRIX.md](AMENDMENT_MATRIX.md); D8 command rows are cross-linked, not redefined.

This is the sole normative proposal home for client authority and command
registry mechanics.

---
## 11. Application boundary and client authority

**Boundary.** A use-case façade, not the Store trait and not database-shaped
records. Candidate operations: `commit_files · get_node · get_neighborhood ·
list_postulants · explain_link · override_link · override_category ·
review_proposal · resolve_proposal · resolve_petition · consent_admission ·
consent_decommission · get_matrix · get_provenance · watch_progress ·
search · legibility_state` — plus the ceremony surfaces §8/P2B enumerates.
(Pass-5: the earlier generic `record_consent` is **retired** — the real
authority is operation-specific, and a generic consent verb would smuggle
distinct IV.4 entries under one name; each consent-bearing operation appears
by name in the §11.1 registry.) Versioned (v1) command/query contract; DTOs translating
records into operator-meaningful state; transport HTTP+JSON with SSE or a
socket for `watch_progress`; client packaging (browser vs Tauri-class) stays
open and does not change the contract. No speculative Store split — a
refactor is proposed when integration meets a concrete obstruction.

**Client authority (v1 posture — decision D7, ADR after answer).** The server
holds the G10 sovereign actor-class, which creates a confused-deputy risk
unless entry is narrow. The v1 answer, made explicit rather than letting
packaging decide it:

- One server-authoritative deployment; the backend holds authoritative state.
  "Local-first" means the store, server, and inference all live on the
  operator's own machine or home server — locality of the *system*, not
  offline-writability of the client.
- One enrolled operator client: enrollment via a pre-shared token at
  deployment; sessions via issued bearer tokens (header-carried, never
  cookies — which retires CSRF for the browser case); origin pinning for
  browser clients; TLS optional at loopback, required on LAN; revocation =
  token invalidation at the server; a lost client is revoked, not recovered.
- **Reserved operations are separately named (pass-3 correction — the pass-2
  list was not the 1:1 map it claimed):** the API's reserved-operation set is
  the union of (a) **Law IV.4's eight entries, enumerated exactly** — crossing
  the intelligent-processing seam (human-invoked processing dispatch);
  invoking rebalance outside a configured trigger; invoking audit; consenting
  to commitment; consenting to decommission; admitting external material at
  the threshold; authoring fetch mandates; **granting petitions**; altering
  sovereign config — and (b) the **non-IV.4 sovereign hands**, labeled as
  such, not smuggled under "consent": laying/releasing overrides
  (IV.1/IV.5/D8), weight correction, **resolving bias warnings**
  (`resolve_bias_warning`, interface.rs — a sovereign-actor store surface the
  pass-3 list missed), Concordat adoption, and — **pass-5 addition —
  pairing/re-pairing**: `form_pairing` today takes no actor and its
  implementation hardcodes `produced_by = 'sovereign'`, while the Holy
  Standard names re-pairing "a fresh sovereign act" (doc 06 §4.3) — an
  authority the perimeter must carry, supplied P5. **The handler registry is
  complete-but-phased (pass-4):** the registry names every reserved operation
  *now*, each entry carrying its supplying phase (ceremony surfaces and seam
  dispatch P2B; agent-labor operations P5; fetch mandates P6); an unshipped
  entry is registered-and-refusing, never absent — so the arch test that pins
  the enumeration pins the whole registry from its first landing, and no
  later phase adds an operation the registry never named. **The registry is
  literal, not a prose claim — §11.1 (pass-5).**
- **Enumeration is not authentication (pass-3 correction; perimeter
  corrected pass 4).** Today's store methods accept an actor *string* and
  elevate the transaction themselves — any caller that can reach them can
  supply `'sovereign'`. The application layer therefore supplies what the
  string cannot: authentication middleware mints an **unforgeable sovereign
  context** (a capability type constructible only inside the auth module —
  not request data, not a header echo); actor identity is derived from that
  context, never from the request body; **actor-class elevation occurs only
  after authorization succeeds**, inside the named handler. **The call-site
  wall is drawn around reserved *operations*, not sovereign *store* surfaces
  (pass-4 correction):** `invoke_audit` (godhead-audit) and `rebalance_now`
  (godhead-ml) are **public library functions**, not store methods — a wall
  watching only Store-trait sovereign surfaces never sees them, and the
  processing-seam dispatcher (§8/P2B) will be a third such function.
  **Enforcement shape (pass-5 correction — pass 4 presented two branches as
  equivalent; they are not):** while the Store trait and the library
  functions remain publicly callable, a gating module plus a source-level
  call-site test proves only that **no current caller bypasses the gate — it
  does not make the bypass uncallable**; every future code path still
  compiles against the same public surfaces. The recommendation is therefore
  **capability-bearing signatures on every reserved operation** — store
  methods and library functions alike take the sovereign-context type, whose
  single construction site lives in the auth module, making an ungated call
  *unconstructible*, not merely untested-for. The gating-module branch
  remains lawful **only with structural sealing**: the sovereign entrypoints
  split into a non-exported trait/module reachable solely through the gate,
  visibility-enforced — which is the P2A refactor this document already
  reserves, priced honestly rather than presented as the equal cheap branch.
  Either way, the arch test enumerates **every entry in the §11.1
  registry** — store methods, library functions, and dispatchers — as a
  tripwire *behind* the structural wall, never in place of it. End-to-end
  tests run the full path from unauthenticated request through persistence,
  proving rejection before elevation.
- Sovereign command envelope: idempotency key; expected revision; **exact
  proposal hash** for consent commands (consent binds to the revision the
  sovereign actually reviewed — a store-method extension, §8/P2B); request
  freshness window.
- Disconnected client: may read cached views and stage files locally; **no
  offline sovereign writes, no queued consents** (a consent decided against
  stale state violates exact-revision freshness); commits are staged-not-
  committed until online (doc 02 §1.1's deliberate commit survives intact).
- Tauri-class local IPC narrows the network threat model but changes no
  authority rule above.

### 11.1 The reserved-operation registry (pass-5 — literal, complete, phased)

The completeness claim is a table, not prose. Envelope codes (store-owned,
§8/P2B): **K** idempotency key, **bound to the request digest** (pass-6 — the
same key with different content refuses, never converges) · **R** expected
revision/state CAS · **F** freshness window · **H** exact content hash ·
**rcpt** = **durable command receipt (pass-6):** a single Store transaction
cannot envelope a multi-step library operation (`invoke_audit` and
`rebalance_now` perform several Store writes and can fail between them), so
multi-step reserved operations record an accepted command (key, hash, actor,
freshness, expected state), progress steps, and a terminal
completion/refusal — retry resumes the standing command, duplicates converge
on it, and no one pretends the downstream effects were one transaction.
Class `sov-cap` = requires the unforgeable sovereign context; `op-session` =
authenticated operator session without elevation; `machine` = job identity
under recorded standing authority (R01b — never a forged capability).
**Every entry whose phase has not arrived is registered and refusing (closed
reason `UNSHIPPED_OPERATION`) from the registry's first landing — never
absent.** The arch test enumerates every row.

| ID | Operation → callable | Accepted outcomes | Authority | Class | Phase | Env | Restart / discovery |
|----|----------------------|-------------------|-----------|-------|-------|-----|---------------------|
| R01a | Seam dispatch, human "run now" → dispatcher fn (§8/P2B, new) | dispatched · refused | IV.4 — seam crossing (live invocation) | sov-cap | P2B | K·F·rcpt | job records + readiness flags rescanned |
| R01b | Seam dispatch, configured trigger → tick (machine executor; **pass-6 split — a scheduler must not forge a sovereign capability**) | dispatched · refused | recorded **standing trigger revision** (IV.4's "user-configured trigger" carve-out; the authority is the recorded configuration, cited per run) | machine (job identity + trigger rev) | P2B | trigger rev + rcpt | same |
| R02 | Direct rebalance → `rebalance_now` (godhead-ml) | recalculated · refused | IV.4 — rebalance **outside** trigger (within-trigger runs are machine, per the R01b principle — IV.4's own wording already splits this) | sov-cap | P2B | K·F·rcpt | `WEIGHT_RECALC` log + `RebalanceState` |
| R03 | Invoke audit → `invoke_audit` (godhead-audit) | audit opened · refused | IV.4 — invoking audit | sov-cap | P2B | K·R·F·rcpt | trial job records; `AUDIT_OPENED` cites evidence set (§10); opening is the atomic op of §10 |
| R04 | Resolve proposal → `resolve_proposal` | grant · decline | IV.4 — consent to commitment | sov-cap | P2B | K·R·F·H | pending-consented-proposals query → execution tick (§8/P2B) |
| R05 | Consent decommission → `consent_decommission` | consent minted · refused | IV.4 — consent to decommission | sov-cap | P2B | K·R·F | **pending-decommission query + CAS-claimed Notary tick (pass-6 — canon's dispatcher note already promises it; no served path exists)** |
| R06 | Consent admission → `consent_admission` | admit · reject | IV.4 — threshold admission | sov-cap | P2B | K·R·F·H | quarantine / Manifest state (0014); **admitted-unprocessed discovery tick (pass-6)** |
| R07 | Resolve petition → `resolve_petition` | grant · decline · silence | IV.4 — granting petitions | sov-cap | P2B | K·R·F·**H (pass-6: hash over the immutable occurrence/TransitionPlan)** | `stalled_grants(0)` immediate discovery → `grants_tick`; stall window is monitoring, not delay |
| R08 | Author mandate → `author_mandate` | authored · refused | IV.4 — fetch-mandate authorship (C.4) | sov-cap | P6 | K·F·H | mandates table (0013) |
| R09 | Sovereign config → `set_config`, SOVEREIGN tier (incl. checkpoint threshold adoption, atomic with R19 at `proceed`) | set · stale-refused | IV.4 — altering sovereign config | sov-cap | checkpoint¹ | K·R·F | `config_constants`; **ConfigHistory once amended (pass-6: no A.5 event exists today)** |
| R10 | Operational config → `set_config`, OPERATIONAL tier | set · stale-refused | operational tier (non-reserved authority, same envelope) | op-session | P2B | K·R·F | same |
| R11 | Lay category override → `lay_category_override` | laid · refused | IV.1 hand (non-IV.4) | sov-cap | P2B | K·R·F | override lineage |
| R12 | Lay link override (sever / force) → §3.1 surface (new) | laid, with chained closures per compatibility table · refused | IV.1 / doc 04 §4.4 (non-IV.4) | sov-cap | P2B | K·R·F·**H (pass-6: hash over every closure and resulting state — the TransitionPlan)** | per-kind lineage epochs |
| R13 | Lay weight override → `lay_weight_override` (new) | laid · refused | IV.1 weight correction (non-IV.4) | sov-cap | P2B | K·R·F | lineage head |
| R14 | Release override, per kind → D8 surface (new) | released (weight: + `RELEASED_AS_STANDS` evidence) · stale-refused | IV.5 / D8 (non-IV.4) | sov-cap | P2B | K·R·F | release records + `OVERRIDE_RELEASED` |
| R15 | Resolve bias warning → `resolve_bias_warning` | acknowledge · silence | HS §6.3 (non-IV.4) | sov-cap | P2B | K·**R (expected warning state — pass-6)**·F | `bias_warning_state` |
| R16 | Adopt Concordat → `adopt_concordat` | adopted · refused | A.14(b), HS §3.3 (non-IV.4) | sov-cap | P5 | K·R·F·H | versions retained forever (§3.3) |
| R17a | Form pairing (initial) → `form_pairing` (today actor-less; impl hardcodes `'sovereign'`) | formed · refused (tier mismatch) | **implementation-sovereign; canon silent on initial formation (pass-6 split — A.10 carries no authority field; only re-pairing is canon-named): the D7 answer confirms or assigns this authority** | sov-cap (pending confirmation) | P5 | K·R·F | pairing records |
| R17b | Re-form pairing after orphaning → `form_pairing` successor path | formed · refused | HS §4.3: "re-pairing is a fresh sovereign act" — canon-explicit | sov-cap | P5 | K·R·F | pairing records + A.8 env status |
| R18 | Adopt embedding space → §10 surface (new) | PREPARING created · refused | D4b migration-class (non-IV.4) | sov-cap | P1-A¹ | K·F | space catalog |
| R19 | Activate embedding space (atomic policy+space promotion at `proceed`) → §10 surface (new) | activated · refused (blockers listed; stale `GraphEpoch`) | D4b + §10 activation invariant | sov-cap | checkpoint¹ | K·R·F·**H (pass-6: hash over the explicit Postulant disposition map)**·rcpt | catalog + disposition list |
| R20 | Abandon embedding space (incl. after machine-marked generation failure — §10 authority unification) → §10 surface (new) | abandoned | D4b (non-IV.4) | sov-cap | checkpoint¹ | K·R·F | catalog |
| R21 | Resolve admission standing notice → surface (new — **no store fn exists; pass-6**) | acknowledge · silence | SC-I07b (Book II §1 doctrine, ruling G11) | sov-cap | P2B | K·R·F | Manifest notice state |
| R22 | Lift a silenced scope (bias pattern) → surface (new — **canon names the lift; no surface exists; pass-6**) | lifted · refused | HS §6.3 / SC-K07: "not re-raised until the sovereign lifts it" | sov-cap | P2B | K·R·F | `bias_warning_state` |
| R23 | Render `SOVEREIGN_JUDGMENT` verdict on a Return criterion → attributed review record (new — **the Return is flagged and immutable; the verdict record is separate; pass-6**) | passed · failed · returned-for-rework | HS §1.3d / B.2: "verdict rendered at sovereign review" (non-IV.4) | sov-cap | P5 | K·R·F·H | review records keyed to `(return_ref, criterion_ref)` |

¹ Checkpoint-phased (pass-6 — the pass-5 footnote's "recorded direct
invocation under the dev register" was process discipline standing in for
the D7 boundary, and is withdrawn). The split is truthful about what each
act needs: **R18 adoption cannot wait for the join** — P1-B evaluates the
PREPARING candidate — and does not need to: adoption is a **migration-class
sovereign act** (§10), and the pinned P1-A slice *is* that act's authority,
identically to every one of the 18 migrations to date (pin-time signature,
two-commit lifecycle). R20 abandoning a failed candidate before the
checkpoint rides the same pinned-slice authority or the checkpoint's
adjust/kill routing. **R09 and R19 wait for the post-join checkpoint**,
where P2A's capability and store-owned envelope substrate exist. Full API
handlers for all four ship with the P2B registry, refusing until then per
the unshipped rule.

**Excluded by design:** machine executors — `execute_grant`,
`execute_matrix_proposal`, the ticks, backfill — are agent labor under job
identity, dispatched by served rules, never operator-invocable operations;
they appear in the restart/discovery column, not as rows. **Client
enrollment, credential rotation, and revocation are deliberately not rows
here (pass-6):** they govern who may hold a session at all — a distinct
**authentication-control registry** owned by ADR-3, with the same
literal-table discipline, kept separate so authority-over-data and
authority-over-access never share a namespace. A generic
`record_consent` does not exist here (each consent authority is
operation-specific). Every row's outcomes column names **every** accepted
outcome — decline, silence, and refusal are first-class results, not error
paths. **Every consent that creates asynchronous labor names its authoritative
pending query, CAS claim, restart behavior, and supplying phase in its row
(pass-6)** — a consent exposed through the API never depends on a test
caller; and discovery is **immediate** (the zero-delay query `grants_tick`
already demonstrates: `stalled_grants(0)`), with the configured stall window
serving SC-C06 *monitoring*, never delaying normal execution.

## Served ceremony and executor ownership

**the complete ceremony surface — classified honestly (pass-3) as
store/domain/orchestration work with criteria, not pure façade**: invoke audit
through its lawful trigger; read Gabriel's and Lucy's report states; read
barrier and reconciliation state; review the exact proposal revision;
**command-envelope ownership at the store layer (pass-5 — generalized from
the single resolve_proposal line):** the §11 envelope (idempotency key,
expected revision/state, freshness, exact hash where applicable) is
**store-owned for every reserved command in the §11.1 registry** — the
sovereign store method validates and records the envelope inside its own
transaction, so a crash between application layer and store can neither
double-apply nor lose a sovereign command; application-layer keys alone die
with the process (today only `set_config` carries `expected_revision`;
`resolve_proposal` accepts `(actor, proposal_id, decision)` bare, and the
petition, admission, decommission, and override surfaces carry no envelope
at all). One uniform store-side envelope mechanism, shaped at ADR-3, DDL at
ADR-2; **the production orchestration set (pass-4
completion — Stop-3 was still mechanically unreachable without it):** the
intake dispatcher deliberately ends at classification (the seam), and the
trial's transitions are test-invoked today, so P2B owns, each with retry,
idempotency, refusal, and restart criteria: the **processing-seam
dispatcher** (the human-reserved service that crosses the seam on the
operator's invocation or configured trigger — embedding/backfill,
consolidation, weight recalculation, emergence; this is IV.4's
seam-crossing entry given its production surface, D7-registered); the
**audit-barrier certification tick** (the supervisor behavior behind
`certify_audit_barrier`, which `invoke_audit` does not perform); the
**reconciliation dispatcher tick** (invoking `reconcile` behind the
certified barrier); and the **proposal-execution tick, with its restart
discovery named (pass-5)** — petition grants have a served dispatcher rule
(`stalled_grants` feeding `grants_tick`, godhead-notary), but consented
matrix proposals have **point lookup only** (`get_proposal`) and are
executed only by test callers today; the tick therefore requires a
**pending-consented-proposals query** (GRANTED proposals lacking a completed
execution, older than the stall window — the exact mirror of
`stalled_grants`) **plus a CAS claim at execution start**, so a restarted
supervisor rediscovers unexecuted consents from the store alone. Observe
Notary dispatch, execution, refusal, and completion; navigate the resulting
Cardinal; read the full proposal→consent→act chain. **Two more owned
executors (pass-6 — doc 05's Notary note already promises a dispatcher "on
any executable consent flag," and only petition grants have one):** the
**decommission-execution tick** (pending-decommission query + CAS claim —
R05's consent today mints a record no served path consumes) and the
**admission-processing tick** (admitted-yet-unprocessed quarantine
discovery — R06's admitted items must enter processing without a test
caller). Discovery for all ticks is immediate (zero-delay query); the
configured stall window is SC-C06 monitoring, never execution delay.
