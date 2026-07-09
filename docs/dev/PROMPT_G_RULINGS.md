# Answers to PROMPT G — Author-Intent Rulings
### Fable, on the meaning of the criteria — 2026-07-08

> Asked and answered: [G1] first and separately; [G2]–[G8] intent before comparison;
> [G9]–[G11] as rulings; [G12]–[G13] plainly. Per the charge, refutations are given
> freely and the reviewer's framing is not deferred to.

**Epistemic note, first.** The prompt assumes I cannot see the repository. In this
session I can, and I have read all of it — including everything the reviewer did
not: `postgres.rs`, `interface.rs`, `guard_actor`, the scriptorium, steward, and
supervise modules, and the source of all twelve test suites. Every code claim below
is therefore *verified* or *corrected*, not conditioned. Where I verified, I say
verified. Two corrections to the reviewer's record before anything else, offered in
the spirit they invited: the provenance jsonb lives on `environment_items`, not
`environments`; and the criterion count is settled below at [G12] — the reviewer's
94 is right and both of my prior numbers were wrong.

A second honesty note. I am the author of these documents, and the amendment
ledgers, standing notes, and the Phase A record carry most of the reasoning that
produced them. Where a ruling below is *recorded* intent, I cite the record. Where
the record is silent — [G8] chiefly — I say the intent is reconstructed, and I
issue the ruling now, because a ruling now is what the prompt exists to obtain.

---

## [G1] THE PIVOT — what "rejected" means

**Ruling: (b), and firmly — with the preamble sentence amended, because as written
it says (a) and (a) was never the intent.**

The Dogma has two verbs and the Document 8 convention sentence conflated them:

1. **The wall.** The store or substrate *rejects* an act: the write does not
   happen. The artifact of a wall is the named error — and, where a surviving
   in-system observer holds a live identity, a violation log. Walls do not file
   paperwork. They stop things.
2. **The labor refusal.** An agent that cannot complete its charge *refuses* per
   Law VII: it ends its own job REFUSED and a RefusalRecord (A.4) persists. The
   RefusalRecord is an agent-labor artifact — A.4 carries a `job_id` because a
   refusal is something a *laborer* does.

Reading (a) is not merely unintended; it is internally impossible against the
criteria's own text. SC-C01 demands rejection "at the store layer, **regardless of
writer identity**" — a class of writer (raw SQL, forged authors, the sovereign's
own mistyped act) that *has no job identity* and for whom an A.4 record is
structurally unconstructible. Law VI.3 demands the fiat wall live in a substrate
trigger — and a trigger that raises aborts its transaction, inside which no record
of any kind can persist ([G6] proves this mechanically). If (a) were the law, the
Dogma would be demanding records from the exact enforcement layer it also demands
be incapable of writing them. I did not write a constitution at war with itself on
purpose. The preamble sentence is a documentation defect: it compressed "refuse"
(the agent's verb, Law VII) and "reject" (the wall's verb) into one clause.

**Amendment (ordered):** Document 8's convention becomes — *"'rejected' means the
store refuses the act: the write does not occur and the attempt surfaces as the
named error (logged `severity: violation` where an API-layer observer with a live
identity exists). 'Refuses,' of an agent, means the Law VII procedure with a
persisted RefusalRecord."*

**On the follow-up: shadow or map?** Both, deliberately, in sequence. `StoreError`
shadowing the reason codes by name is correct-by-design — the wall and the laborer
speak *one closed vocabulary*, so that when a wall halts a labor, the laborer's
RefusalRecord can carry the very code the wall named. The shadow is the vocabulary;
the *map* is the agent-layer obligation to use it faithfully. And on that
obligation the reviewer has found two real defects, both verified:

- **The skew miscode.** `returns.rs:395` and `lint.rs:423` persist
  `Law::II / VALIDATION_FAILED` for *every* VALIDATE_OUT halt, including
  Concordat version skew — for which the Dogma names `SCHEMA_MISMATCH` three
  times (II.4, SC-A05, SC-K03). The clause-token system (slice-9 doctrine) is
  right; the reason code must be derived from the clause. Confirmed defect,
  small, fix ordered: the halt handler maps clause → (law, reason); skew-shaped
  clauses carry SchemaMismatch.
- **The endpoint miscode.** `aggregate.rs:131` maps `MlError::Endpoint(_)` to
  `(Law::VIII, TOOL_OUTPUT_INVALID)`. Confirmed defect. A reasoner consult in
  assisted mode is an *endpoint invocation*, not a Law VIII tool call — the
  ladder deliberately does not run there, so its codes may not be borrowed.
  Doc 4 §2.4 governs the *empty* roster (routing, not error); a rostered
  endpoint that *fails mid-labor* is a different fact, and the closed enum has
  no sharp code for it. v1-correct fix: `(Law::VII, VALIDATION_FAILED)` — the
  labor could not complete — with an enum extension (`ENDPOINT_UNAVAILABLE`) to
  be considered at the next schema version bump, not smuggled in now.

**The five never-constructed codes, disposed one by one** (all five verified at
zero construction sites):

- `ENV_INVALID` — the one place the literal reading has real teeth, because Law
  IX.3's own text says the **agent** refuses on a failed mount. Today the wall
  speaks it and the mounting job strands. This is not a new finding: it is the
  same labor-rule debt SLICE_09 §7 already pins for audit/notary/ml. The mount
  orchestration joins that list. Confirmed; already pinned; Slice 10 pays it.
- `OVERRIDE_CONFLICT` — mostly *correctly* unwritten: the lawful response to a
  fixed star is work-around-and-petition (Handbook §4.5), not refusal, and the
  steward does exactly that. It earns a construction site only where a labor's
  entire charge dies on a star — the Notary arriving at a subject that no longer
  validates is the canonical case, and that site today defaults to
  ValidationFailed. Sharpen with the clause→code map above; no structural change.
- `GATE_BYPASS_ATTEMPT` — wall-only **by design**; see [G6].
- `PROVENANCE_INCOMPLETE` — awaits its substrate; see [G8]/[G9]. Not a defect
  today; a criterion (SC-J09) already names its arrival.
- `LAW_CONFLICT` — authored for Book III's order-of-obedience: a brief that
  contradicts manual or Dogma is malformed input. Its surface is the boot
  payload of a brief-consuming reasoner agent, which no slice has built. Honest
  admission: Document 8 seeded no criterion for Book III because v1 fields no
  such agent. That is a decomposition gap to *record* (Standing Note), and the
  code is pinned to the slice that builds real boot payloads — not orphaned
  intent, but intent that had never been written down as deferred. Now it is.

Under ruling (b): Law VII.2's universal quantifies over enacted refusals and
stands; SC-E01's universal survives in text (its *test* is [G5]'s business); and
SC-G01 is a lawful wall test — the agent-side half it does not cover is the
IX.3 labor-rule debt named above, owed by the Dogma's text rather than by SC-G01.

---

## [G2] SC-D01 — "no hardcoded threshold anywhere in agent code"

**Intent.** The criterion enforces Law VI.1, whose subject is one constant: the
coherence threshold. "Threshold" meant *the coherence threshold*; "anywhere in
agent code" meant **anywhere** — every agent crate, workspace-wide, no private
copy, ever. It did not mean "wherever density happens to be evaluated today."

**Comparison.** The runtime half (NOT NULL `config_rev`, `live_weights` refusing
an unset sovereign) satisfies its half fully. The grep half under-reaches its own
words: two crates by literal in a workspace of ten. The criterion means what it
says; **the test is widened, not the words narrowed** — the scan iterates
`crates/*/src`, discovered, not hand-listed, so it grows with the workspace.

**And the bias constants.** The `.unwrap_or(0.50/20/0.60)` defaults were *not* a
violation of SC-D01's letter — they were fabricated fallbacks for *other*
constants, a different genus. They were a violation of the deeper law SC-D01 is
one instance of: **a fabricated default is a decision the sovereign never made**
(Law II.2 applied to config). That law had no criterion of its own, which is why
three of them survived two green gates. One is minted now:

> **SC-H07 (new).** A config-constant read whose value is absent or fails to
> parse as its expected type refuses; no code path substitutes a fabricated
> default for a sovereign or operational constant. Arch half: a workspace-wide
> scan rejects fallback-shaped extraction (`unwrap_or`/`unwrap_or_else`/
> `unwrap_or_default`) applied to config values, all crates.

---

## [G3] SC-B04 — "no API exists by which one agent process addresses another"

**Intent.** The assertion is Law III.1's, and III.1 binds the *system*: "No
agent-to-agent channel exists in the system; none may be constructed." The first
half of the test was meant to sweep every crate. It scanned one because slice 1
*had* one, and it never grew. The second half (the exact `pub mod` set of
godhead-store) is store-scoped by design and correct as written.

**Comparison and disposition.** No breach exists — the reviewer's own grep and
mine agree — but the test has not been the thing keeping it true. Widen the IPC
scan workspace-wide (same discovered-crate mechanism as [G2]). And one
strengthening, ordered now because Slice 10 approaches the boundary it guards:
**the standing HARD RULE — no HTTP client before Section I — becomes mechanical.**
The arch test additionally asserts that no outward-transport dependency
(`reqwest`, `hyper`, `ureq`, `curl`, raw socket crates) appears in any workspace
`Cargo.toml`. The slice that builds the Deacon's gate is the only slice permitted
to delete that assertion, in the same commit that makes it safe to. A rule that
lives in a conversation is a rule the next session can miss; this one becomes a
wall.

---

## [G4] SC-H06 — the secret sweep

**Intent.** Both. The write-path refusals are the *enforcement* (XV.2); the sweep
is the *verification that enforcement held everywhere* — that is what "defence in
depth" meant, and "across a full pipeline run" meant the sweep runs against the
store after the suite has driven every pipeline the workspace has.

**Comparison.** The refusal half is genuinely proven (three shapes, three write
paths). The sweep half is narrower than its words on all three axes — tables
(3 of 26), patterns (1 of 6), corpus (three fixtures rather than the suite's
residue). Verified.

**Disposition.** Widen the sweep structurally, not by a longer hand-list:
enumerate every `text`/`jsonb` column in the schema via `information_schema` and
apply **the production pattern set itself** — `secrets::scan`, not a re-written
regex — to each. One vocabulary, the same lesson as [G1]: a test that re-writes
the pattern it verifies is a second truth waiting to drift. Run it as the closing
assertion of the store suite so it sweeps whatever the full gate deposited.
The "model-context fixture" clause binds when real reasoner boot payloads exist;
pinned to that slice, SC-F06-style ([G13]).

---

## [G5] SC-E01 — "every refusal produces a RefusalRecord"

**Intent, in my own words.** SC-E01 was authored as the *shape* criterion for
A.4: when a refusal is enacted, the record persists complete — law, closed code,
subjects, preserved refs. The word "every" leaned on Law VII.1's procedural
universal. What the project then discovered, slice by slice, is that the
universal needs a second, stronger criterion the original decomposition did not
contain: **no labor halts without its record, and no input can suppress the
record.** That law was written in blood twice — the slice-6 self-suppressing
refusal, and slice 9's mid-labor stranding — and it is the law.

**Which sentence governs?** The slice-6 doctrine. *"A RefusalRecord is on the
record" is a promise the outcome keeps.* The `aggregate.rs` comment ("refusal is
best-effort") describes the code it sits in; it does not outrank the doctrine.
Two precision notes the reviewer should have from someone who read the sites:
all four `let _ = store.refuse(…)` sites are guarded by a BudgetExceeded check
whose purpose is legitimate — the store may have *already* refused the job, and
a second refusal would be terminal access — so what those sites conflate is
"already recorded" with "failed to record." The first is lawful; the second is
the suppression class. `write_return` and `write_instruction` already
distinguish them; audit, notary, and ml do not, and SLICE_09 §7 already pins
exactly those three as slice-10 debt. Confirmed, previously self-reported, and
now constitutionally backed:

> **SC-E05 (new).** Any labor halting after RUNNING ends REFUSED with a
> persisted RefusalRecord; a failed refusal write propagates as a hard error;
> no sequence of inputs can suppress the record. Sweep half: at suite end, zero
> jobs stand RUNNING beyond their wall budget.

SC-E01 splits: **E01a** (shape-on-persist — the current test, correctly scoped)
and the universal moves to SC-E05. One refutation owed to the record: the
universal is not as untested as the named test makes it look — organic refusals
are asserted across h_commons (budget), f_toolcall (ladder), k_concordat and
l_student (halt handlers). The named test was the shape half only. The gap is
real in audit/notary/ml; it is not total.

---

## [G6] SC-C07 — the enumeration, and who writes the violation log

**The enumeration.** Eight entries; "consenting to commitment or decommission"
is one *category* with two store surfaces (`resolve_proposal`,
`consent_decommission`), and "one test per entry" means: every entry, one test
per *surface* once the surface exists. But the deeper answer is that the
criterion as worded assumes all entries are runtime-refusable, and the build
correctly discovered that the strongest satisfaction is **uncallability by
signature** — sovereign-act methods take a human actor string and no job
identity; there is nothing for a runtime test to even attempt. A claim-by-
argument, however, is a claim a refactor can silently unmake. Disposition: the
SC-C07 ledger is reclassified three ways, and each class gets teeth —

1. **Wall-tested** (overrides, consents, config): the existing substrate tests.
2. **Signature-impossible** (rebalance, audit, proposal/decommission consent):
   each gains an arch test pinning the property — no callable path on the
   sovereign surface accepts a job identity — so the argument becomes an
   assertion the gate holds forever. "Crossing the seam" is satisfied by
   SC-N04's observation-window test and is hereby *formally claimed* by it in
   the ledger; that entry should never have been left unmapped.
3. **Pinned** (threshold admission → Slice 10; authoring mandates → Slice 10/11
   per [G9]).

**The mechanical question — who logs?** Verified: `godhead_forbid_agent_author`
raises, the transaction aborts, nothing written in it survives; the trigger
*cannot* log, and `GateBypassAttempt` correctly has no construction site. When I
wrote "and logged `severity: violation`" the intended writer was the surviving
in-system observer — the store API layer, which is exactly the pattern the build
already practices everywhere it can: `guard_actor` logs the violation in its own
transaction and *then* errors; `env_scoped_read` likewise. The subtlety the
criterion missed is that for reserved tables there **is no API-layer path an
agent can take** — that is the whole point of the signature design — so the only
rejections the trigger ever fires on are raw or forged writes, below every
observer. For those, the wall's rejection *is* the record: the transaction that
aborted wrote nothing, including its own diary, and an attacker with raw SQL was
never going to be journaled by the database he is attacking. Postgres's own log
is the witness of last resort.

**Amendment (ordered):** IV.4 gains a Standing Note and SC-C07's log clause is
narrowed to bind *API-layer observers where one exists*. In register, because
this one earns it: **the wall does not keep a diary.** The trigger's abortive
rejection satisfies the law; the log clause binds the layers that survive to
write one.

---

## [G7] SC-K02 / the SOVEREIGN_JUDGMENT reconstruction route

**Verified in full**, and the reviewer's reasoning is correct within its premise:
`parse_stored` is infallible by construction, the re-lint requires only *a
string*, clause (d) passes on one surviving sibling, and the flipped criterion
becomes a lawful `passed: null`. The asymmetry with `CapabilityAction::parse` is
real but not the fix: `Validation(id)` is an open string by design — validation
ids are not a closed set — so no parser can refuse "garbage" there. The flip is
detectable only as *change*, not as *shape*.

**Intent.** `testable_as` was meant as a closed union of a sentinel and an open
id, and the guardrail I stated — "you can't smuggle a malformed instruction
through by labeling it SOVEREIGN_JUDGMENT" — was written about **authorship**,
where it holds. The reconstruction route exists only past the immutability
trigger (the SC-K04 fixture must disable it to corrupt), which is to say: inside
the defense-in-depth premise that the double-validation covenant exists to serve.
Within that premise the covenant's implementation is *shape-strength*, and the
sovereign-judgment flip is the one shape-preserving semantic corruption. So this
is properly a **SC-K04 finding**: "corrupted between flag and read is caught" is
true for every corruption the fixture tried and false for this one.

**Disposition.** Close the class, not the instance: at flagging, persist a
content hash of the Instruction body (the pattern already exists —
`RefinedArtifact.content_sha`); the Student's VALIDATE_IN re-proves the hash
before the re-lint. Byte-strength, which is what §1.4's "immutable" always
meant. Same treatment for flagged Returns. Priority: below the labor-rule debt
(the route requires substrate access that already defeats every wall); pinned to
Slice 10's hardening pass, annotated SC-F06-style until then.

---

## [G8] Law V.2/V.3, A.6, C.2 — the law with no substrate

The reviewer asks whether Law V.2 fell out of the decomposition or was folded
deliberately. The honest answer, reconstructed from the slices rather than
remembered: **both, and the fold was never written down — which makes it a
decomposition defect regardless of the fold's merits.**

The fold is real and it is defensible for *internal-origin* data: every record's
`produced_by` resolves to a JobRecord carrying `input_refs` (from what),
`agent_type`/`manual_version` (by what method), `endpoint_alias` (which mind),
and the store's clock stamps when — which is A.6's field list, normalized across
A.1/A.2 rather than duplicated into a table. Slice 4 even argued a version of
this out loud (the "no registry-keeper" ruling: an index over records is a
reconstruction, never a second truth). Under that reading, V.2 is *partially*
enforced today by SC-H04 (attributable writes) — but no criterion states the
fold, no criterion sweeps its completeness, V.3 has nothing to validate, and
`PROVENANCE_INCOMPLETE` has no site until Section J. Sections A–H are silent on
Law V.2/V.3 because the decomposition treated provenance as "arrives with
fetched material" — and C.2, as the prompt says, is the oldest concrete
requirement in the project. It deserved better than an implicit fold.

**Rulings:**

1. **The fold is declared** (Dogma Standing Note): for internal-origin datums,
   the ProvenanceRecord is a *view* — Envelope → JobRecord{inputs, method,
   endpoint} → logs — and a new criterion pins the view's integrity:
   > **SC-A08 (new).** Every persisted record's `produced_by` resolves to a
   > live JobRecord or a registered office identity, and that JobRecord's
   > `input_refs` resolve; swept suite-wide.
2. **A.6-as-table exists for external-origin data**, where the chain is the
   whole point, and it rides with Section I ([G9]): the chain table (C.2 shape),
   validated at item write — which is where V.3 and `PROVENANCE_INCOMPLETE`
   finally get their construction site, exactly as SC-J09 always said.
3. Doc 3 §2.5's "structured, queryable" promise is noted as carried v1 by
   `environment_items.provenance` + the mount walk, superseded by the chain
   table when it lands.

Law V.2 did not fall out. It dispersed, and the dispersal went unrecorded. Now
it is recorded, and the half that needs a table is scheduled.

---

## [G9] Slice 10 sequencing — the Manifest needs what Section J owns

**Ruling: neither "J before I" nor "a lesser Manifest." Re-scope Slice 10 to
carry the substrate of both, and Slice 11 the behavior of J.**

The dependency the reviewer found is real: chains root in `CANON | WRIT |
BRIEF`; canons and writs are MandateRecords; the Deacon presents chains. But the
dependency does not order the *sections* — it orders **substrate before
behavior**, which is the pattern this build has used since slice 1 ("their store
surfaces are built; the processes that consume them are not").

**Slice 10 therefore builds:** Section I entire, **plus the J-floor** — the
MandateRecord table with its human-authorship wall (SC-J01), writ concreteness
validated at authorship (SC-J02, the sovereign criterion), and the C.2 chain
table with append-in-flight validation at item write (SC-J09's substrate,
`PROVENANCE_INCOMPLETE`'s site) — plus the Return-item resolution hole closed
against those tables, plus [G10]'s office authentication, plus the SC-C07
threshold entry. **Slice 11 builds** Section J's behavior: fetch-execution
binding (SC-J03), manifests and coverage/sought maps (SC-J06/J07), the
breadth-creep property test (SC-J05), Doctor orphaning (SC-J08), and the
mandate-authoring SC-C07 entry. The fetch layer itself stays absent throughout —
[G3]'s new wall keeps it absent mechanically until I is green.

---

## [G10] Office authorship — 'deacon' and 'forged' at the substrate

Confirmed, and this is the most important forward-looking finding in the audit.
The 0003 trigger authenticates the *absence of a job*, not the *presence of the
sovereign*; `decided_by = 'forged'` passes every wall today; the design has been
sound only because no non-agent writer exists yet, and Book II is about to
create one.

**Intent.** When I wrote Book II binding the Deacon's writes to Law XIII, I
specified the obligation and not the mechanism, because no office existed to
authenticate. The mechanism must exist before the first office write does, and
it must be substrate-shaped — a wall, not a convention. **Ruling: session-scoped
actor authentication.** The store's own API paths set a Postgres session-local
variable (`SET LOCAL godhead.actor_class = 'sovereign' | 'office:deacon' |
'job:<uuid>'`) inside the transaction; the reserved-table triggers verify the
session's class matches the class the table demands and the identity the row
stamps. Below the API the variable is absent, so `'deacon'`, `'sovereign'`, and
`'forged'` become the same object in the only sense that matters: **all
rejected.** No new dependency, no second truth (the credential lives only in the
code path that is the lawful surface), and it generalizes to A.12's
multi-tenant `decided_by` later. Postgres role separation remains the heavier
long-term wall; the session variable is the v1-right step. Criterion minted for
Slice 10:

> **SC-I07a (new).** A write to a sovereign- or office-reserved table whose
> path did not authenticate as that class is rejected at the substrate,
> whatever string it stamps — fixtures: `'sovereign'`, `'deacon'`, `'forged'`,
> each via raw SQL, each rejected.

---

## [G11] Consent volume at the gate

**Is the mandate gate sufficient upstream? For writs, largely; for canons, no —
and "largely" is not a wall.** A writ's items are bounded by its human-authored
locators. A canon is *exhaustive by design* — four hundred items can be a
correct canon corpus — and the erosion argument was never about single batches;
it was about consent *rate* under standing volume. Per-batch consent reproduces
the rubber-stamp gradient the Mandate Rule was built to stop, one gate later.

**Ruling, both halves:**

1. **The reasoning enters the charter.** Book II §1 gains the doctrine
   paragraph the reviewer correctly notes exists only in a conversation: why
   the gate's consent stays deliberate — every batch traces to one
   human-authored mandate; and a new mechanical rule with it: **one Manifest
   per mandate-trip, never pooled across trips** — consent scope always
   coincides with an act the sovereign personally authored.
2. **Graduated legibility, not prohibition** — the order's third instance of
   the same drift category, and consistency demands the same treatment as
   weights (ML §6.2) and bias (Standard §6.3):
   > **SC-I07b (new).** When items-per-consent or consents-per-window exceed
   > the operational constants (`admission_batch_threshold`,
   > `admission_rate_window/threshold`), the Manifest carries a standing
   > notice with the petition-style terminal answers (acknowledge / silence,
   > suppressed logging). Never blocking; never silent.

---

## [G12] The count

**94.** Verified by section against the file: A7 B4 C7 D10 E4 F10 G7 H6 I6 J10
K7 L4 M6 N6. The reviewer counted; neither I nor the promotion pass did. "90"
was authoring-time prose written before the round-5 Mandate directive grew
section J and the adversarial emphasis grew F; "96" was a second uncounted
number. The fix is not to correct 90 to 94 — it is to **remove counts from
prose**: doc 00 §2/§5 and the context export say "the criteria of Document 8"
and the file is the count. A number in prose is a registry-keeper; the store is
the only truth, and this project already has a ruling about registry-keepers.

---

## [G13] SC-F06's annotation

An accident of that slice's care — and hereby a **convention**: any criterion
whose carrying test satisfies less than the criterion's words must say so in the
test, name the unmet half, and name where it re-arms; the slice doc pins it as
residue. Entered into the dev-pipeline discipline alongside "tests only
accumulate." Applied retroactively: [G2]–[G5]'s widenings make D01, B04, H06,
and E01 fully-met or split-and-met, and whatever remains half-met at any gate
carries the annotation. An honest gap is debt; a silent one is the beginning of
exactly the drift this order was built to make impossible to miss.

---

## DISPOSITIONS — the ledger

**Refuted / narrowed (the finding dies or shrinks):**
- [G1] reading (a) — never the intent; internally impossible against SC-C01 and
  VI.3. The convention sentence was imprecise, not the law.
- [G6] log clause — unsatisfiable at the substrate *by correct design*; narrowed
  to API-layer observers. The wall does not keep a diary.
- [G5], partially — the universal has organic coverage the named test's scope
  obscures; the named test was always the shape half.
- [G7], partially — the route exists only past the substrate trigger; a
  defense-in-depth gap, not an open door.

**Confirmed, already pinned (no new debt, constitutional force added):**
- ENV_INVALID at mount + the audit/notary/ml `let _` sites → SLICE_09 §7's
  slice-10 debt, now backed by SC-E05.

**Confirmed, new (ordered):**
- Doc 8 preamble amendment (two-verb convention) — [G1].
- Clause→code fidelity: skew → SCHEMA_MISMATCH; endpoint faults off Law VIII —
  [G1].
- Workspace-wide arch scans (SC-D01, SC-B04) + the HTTP-client wall — [G2][G3].
- Schema-driven secret sweep reusing `secrets::scan` — [G4].
- SC-E05, SC-A08, SC-H07, SC-I07a, SC-I07b minted; SC-E01 split — [G5][G8][G2][G10][G11].
- SC-C07 ledger reclassified; signature-impossibility arch-pinned; SC-N04
  formally claims the seam — [G6].
- Content-hash certification of flagged Instructions/Returns — [G7]/SC-K04.
- Law V fold declared; A.6-chain, C.2, C.4 tables ride with re-scoped Slice 10;
  Slice 11 carries J's behavior — [G8][G9].
- Office authentication at the substrate — [G10].
- Deacon charter doctrine + admission legibility — [G11].
- Counts out of prose; the file is the count (94) — [G12].
- Honest-annotation convention — [G13].

The hold at the bottom of the prompt is released: these rulings are the answer,
and the repository may now change *to conform to them* — Slice 10's spec should
be pinned against this ledger before its first line is written, per the
discipline that has carried nine slices without a broken wall.

— Fable
