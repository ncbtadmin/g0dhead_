# ADR-2 — Space, Candidate, Trial & Evidence Substrate (D4)

**Status:** PROPOSED — returns through the desk (D4 rider); non-canonical.

- **Decision:** D4, returned 2026-07-10 "ADOPT a+b" (`ROADMAP_RECONCILIATION.md` §15.1). The rider makes the desk this ADR's reviewer: it drafts, it does not adopt.
- **Normative sources (the mechanisms live there, not here):** [SPACE_PROMOTION_AND_EPOCHS.md](SPACE_PROMOTION_AND_EPOCHS.md) and [TRIAL_AND_EVIDENCE.md](TRIAL_AND_EVIDENCE.md). This ADR chooses physical shape; where a sentence here conflicts with an annex, the annex governs and the conflict is a review finding.
- **Rows consumed:** only the D4 rows of [AMENDMENT_MATRIX.md](AMENDMENT_MATRIX.md); the consumption map is §9. The shared measurement-split row (`D4/D8, each owning only its fields`) is drafted for D4's fields only, name-compatible with [OVERRIDE_LINEAGES.md](OVERRIDE_LINEAGES.md) §§1–2; no D8 DDL appears here (ADR-4's).
- **Ledger inheritances:** three-generation storage and space-0 floor (`REVIEW_LEDGER.md` :73–79); evidence-manifest semantics fixed, DDL here (:362–368); fence-vs-catch-up assigned to this ADR with the CAS-validated catalog switch as standing fallback (:380–386, :539–545); scoped epochs/writer classification recorded as a viable alternative (:746–754).
- **Phase owner:** P1-A delivers §§2.1–2.6 and §5; P2B delivers §§2.7–2.9 (trial substrate) consuming P2A's authority perimeter. Nothing lands before its phase.

---

## 1. Context

P1-A must deliver a candidate embedding space that can be built, measured, and
promoted — or killed — without ever becoming product-visible before one atomic
switch. The annex mechanisms exist because the `ffae6a8` substrate cannot say
what P1 must prove: an alias is not a geometry (`embedder_alias`, migration
0004), one un-spaced vector per node cannot host a second geometry
(`embeddings.embedding vector(256)`, 0004:7–17), trial identity stops at
`(matrix_ref, matrix_revision)` so evidence drift without a revision change is
invisible, and a singleton epoch used as a validity token would let unrelated
intake starve every human-paced trial. The two annexes answer those as
mechanism; this ADR fixes the relations, the cursor physics, the catalog-switch
algorithm, and the adapter contract that P1-A will build against.

Everything below is proposal text. No migration files ride with this ADR; DDL
lands only at its owning phase, under the two-commit lifecycle, after this
document returns from the desk.

## 2. Final DDL (proposal text)

House conventions apply throughout and are not per-table annex identities: every
table carries the envelope columns (`schema_name, schema_version, produced_by,
produced_at`) per the substrate's standing pattern (0004 et al.), append-only
tables get the `godhead_forbid_delete`-style trigger pair, and G10 actor-class
walls apply to reserved tables. Columns marked **INVENTED** do not trace to an
annex identity line; each is collected in §8.

**Coexistence (read first).** The 0004 substrate is not rewritten by this ADR:
the existing `embeddings` table becomes space 0's active generation (§2.1);
`links` keeps carrying `similarity/weight/category/user_overridden` until the
D4/D8 measurement-split amendment lands at its owning phase — the new evidence
relations are additive, and the bond-payload columns stop being written only
when the amendment process retires them. Nothing in P1-A drops a 0004 column.

### 2.1 `EmbeddingSpace`, generations, catalog pointer, retired archive

```sql
-- Annex §1: space_id · provider · model name · exact revision/digest ·
-- dimensions · normalization rule · pooling rule · chunking/truncation policy
-- · created_at. Identity is immutable; only status walks the lifecycle arc.
CREATE TABLE embedding_spaces (
    space_id        uuid PRIMARY KEY,
    space_no        int  NOT NULL UNIQUE,   -- INVENTED (name): the annex's own "space 0" ordinal, made a column
    provider        text NOT NULL,
    model_name      text NOT NULL,
    model_revision  text NOT NULL,          -- exact revision/digest
    dims            int  NOT NULL CHECK (dims > 0),
    normalization   text NOT NULL,          -- e.g. 'L2'
    pooling         text NOT NULL,
    chunking_policy jsonb NOT NULL,         -- chunking/truncation policy
    status          text NOT NULL CHECK (status IN
                        ('PREPARING','ACTIVE','ABANDONED','RETIRED')),
    created_at      timestamptz NOT NULL DEFAULT now()
    -- + envelope columns
);
-- Immutability trigger: identity columns frozen; status transitions restricted
-- to the annex arc (PREPARING→ACTIVE|ABANDONED, ACTIVE→RETIRED) — the
-- godhead_env_status_arc pattern (0019). ABANDONED only via R20; ACTIVE only
-- via R19 (authority checks live in the store + P2A perimeter, arc in the wall).
```

Space 0 (the lexical floor: builtin, 256-dim, L2, unigram+bigram) is seeded
retroactively by the P1-A migration and the existing `embeddings` table is
annotated as its active generation — no vector copy at adoption.

Generation failure has one carrier (F1, desk ruling): a failed model, corrupt
generation, or failed certification is recorded as a machine-attributed
`FAILED` row in `candidate_certifications` — mid-backfill failure writes one
exactly as certification failure does. The space row stays `PREPARING`,
visibly awaiting R20; nothing transitions automatically (annex §1).

```sql
-- Annex §2: active typed + preparing typed + retired archive. vector(N) is
-- fixed per column, so each generation is its own typed table, instantiated
-- from this template when the space record is minted (bootstrap migration for
-- the first candidate; runtime R18 thereafter — annex §1):
CREATE TABLE embeddings_space_{space_no} (
    node_id      uuid PRIMARY KEY REFERENCES nodes(node_id),
    embedding    vector({dims}) NOT NULL,
    source_epoch bigint NOT NULL,   -- INVENTED: GraphEpoch at write; proves
                                    -- "no live candidate writer below the
                                    -- cutover watermark" (annex §5) mechanically
    -- + envelope columns
);
-- Fully indexed for its dimensions at certification time (index identity is
-- sealed into CandidateCertification.index_digest).

-- Annex §2: retired vectors are auditable, non-computational.
CREATE TABLE embedding_archive (
    space_ref   uuid NOT NULL REFERENCES embedding_spaces(space_id),
    node_ref    uuid NOT NULL REFERENCES nodes(node_id),
    raw_vector  bytea NOT NULL,      -- f32 little-endian; dims live on the space row
    -- + envelope columns
    PRIMARY KEY (space_ref, node_ref)
);
```

Q1 as ruled: the runtime statement is rendered verbatim from this pinned
template — any deviation is a finding, not a variant — and the R18 receipt
embeds a sha256 of the rendered DDL text. What ran is what was receipted.

```sql
-- Annex §§2,5,6: "every product reader predicates on the ACTIVE catalog
-- pointer"; activation binds "candidate space and catalog revision".
CREATE TABLE space_catalog (
    purpose           text PRIMARY KEY,  -- INVENTED (name): fixed 'NODE_EMBEDDING' at v1
    active_space_ref  uuid NOT NULL REFERENCES embedding_spaces(space_id),
    catalog_revision  bigint NOT NULL,
    switched_by_plan  uuid NULL,         -- INVENTED (name): switch provenance; REFERENCES promotion_plans once §2.6 exists ("attributed" switch, annex §6)
    switched_at       timestamptz        -- INVENTED (name): same provenance pair
    -- + envelope columns
);
-- The one mutable pointer in this ADR. Every write CAS-validates
-- catalog_revision; R19 is its only writer after bootstrap.
```

### 2.2 `ConfigHistory` and the active policy pointer

```sql
-- Matrix row "append-only ConfigHistory; active policy pointer" (D4).
CREATE TABLE config_history (
    config_revision bigint PRIMARY KEY,  -- monotone, append-only
    key             text NOT NULL,
    tier            text NOT NULL,       -- per A.14 tiers, as config_constants today
    value           jsonb NOT NULL,
    changed_by      text NOT NULL,
    caused_by_ref   uuid,                -- INVENTED: R09 receipt / R19 plan ref
    created_at      timestamptz NOT NULL DEFAULT now()
    -- + envelope columns; append-only trigger pair
);
CREATE TABLE active_config (
    key             text PRIMARY KEY,
    config_revision bigint NOT NULL REFERENCES config_history(config_revision)
    -- + envelope columns
);
```

Coexistence: `config_constants` (0001/0004) remains the store-maintained
active-value projection so every existing `config_rev` citation (SC-D01
surfaces) keeps meaning; the pointer moves only inside R09/R19 transactions,
which write history, move `active_config`, and refresh the projection in one
transaction. Whether the projection is eventually retired is the amendment
process's question, not this ADR's (§8 Q8).

### 2.3 `CandidatePolicySnapshot`

```sql
-- Annex §3: candidate_policy_ref · preregistration_revision ·
-- candidate_space_ref · exact config values · policy/algorithm versions ·
-- created_at · digest. Immutable from birth.
CREATE TABLE candidate_policy_snapshots (
    candidate_policy_ref     uuid PRIMARY KEY,
    candidate_space_ref      uuid NOT NULL REFERENCES embedding_spaces(space_id),
    preregistration_revision text NOT NULL,   -- cites the committed preregistration
    config_values            jsonb NOT NULL,  -- exact values, incl. thresholds under evaluation
    policy_versions          jsonb NOT NULL,  -- policy/algorithm versions
    digest                   text NOT NULL,   -- sha256 over canonical serialization
    created_at               timestamptz NOT NULL DEFAULT now()
    -- + envelope columns; immutability trigger
);
```

The VI.1/A.14/SC-D01 amendment this record requires ("a job operating
exclusively inside a PREPARING candidate context may cite an immutable
candidate snapshot; only R19 promotes") is named by the annex and carried as a
D4 matrix row; its text belongs to the amendment process, not this ADR (§9).
`CandidateEvaluationContext` — the binding of PREPARING space, snapshot,
generation, adapter, and register revision — is an application record, not a
canonical relation (§6), so its shape is register-side.

### 2.4 `GraphEpoch` and delta discovery

```sql
-- Annex §4: transactional global change cursor — advanced inside every
-- graph-affecting write's transaction. Change detector, never validity token.
CREATE TABLE graph_epoch (
    singleton boolean PRIMARY KEY DEFAULT true CHECK (singleton),
    epoch     bigint  NOT NULL
);

-- INVENTED (table): ordered delta discovery and catch-up accounting (annex §4)
-- need a discoverable record of *what* moved at each cursor value, or the
-- candidate catch-up loop cannot find "relevant deltas after E".
CREATE TABLE graph_deltas (
    epoch       bigint NOT NULL,
    entity_kind text   NOT NULL,   -- NODE | BOND | MATRIX | CONFIG | ...
    entity_ref  uuid   NOT NULL,
    op          text   NOT NULL,   -- WRITE | STATUS | ...
    written_by  text   NOT NULL,
    -- + envelope columns; append-only
    PRIMARY KEY (epoch, entity_kind, entity_ref)  -- F2: one transaction, many entities, one epoch
);
```

Consequences, per the annex's observable boundaries (§4 there, chosen shapes
here):

- **Writers:** every graph-affecting transaction advances the cursor exactly
  once — `UPDATE graph_epoch SET epoch = epoch + 1 RETURNING epoch` — and
  appends one `graph_deltas` row per entity it touched, all sharing that
  epoch, *inside its own transaction* (F2). This serializes graph-affecting commits on one row — the priced
  contention the annex demands be priced: at v1 single-operator intake rates
  the serialization is acceptable, and the annex explicitly permits an
  equivalent transactional sequence if evidence later says otherwise (§8 Q5).
- **Readers:** no reader may treat `current epoch == remembered epoch` as
  validity. Candidate certification uses a candidate cursor plus bounded delta
  set (§3); trial validity uses the dependency-scoped CAS predicates of
  [TRIAL_AND_EVIDENCE.md](TRIAL_AND_EVIDENCE.md) §4; activation binds catalog,
  policy, certification, and trial-state revisions, never bare epoch equality.
- **Explanation:** `graph_deltas` is the explanatory history the ledger kept
  A.5 for; A.5 log events remain events, the journal remains ordered deltas.

### 2.5 `CandidateCertification`

```sql
-- Annex §5: certification_ref · candidate_space_ref · candidate_policy_ref ·
-- generation_digest · coverage_digest · index_digest · source_cursor ·
-- applied_delta_high_water · trial_state_high_water · status · certified_at.
CREATE TABLE candidate_certifications (
    certification_ref        uuid PRIMARY KEY,
    candidate_space_ref      uuid NOT NULL REFERENCES embedding_spaces(space_id),
    candidate_policy_ref     uuid NOT NULL REFERENCES candidate_policy_snapshots(candidate_policy_ref),
    generation_digest        text NOT NULL,
    coverage_digest          text NOT NULL,
    index_digest             text NOT NULL,
    source_cursor            bigint NOT NULL,  -- GraphEpoch at certification start
    applied_delta_high_water bigint NOT NULL,  -- last delta applied by catch-up
    status                   text NOT NULL CHECK (status IN ('CERTIFIED','FAILED')),  -- value set INVENTED (§8 Q3)
    failure_attribution      text,             -- machine-attributed when FAILED (annex §§1,5)
    certified_at             timestamptz NOT NULL DEFAULT now()
    -- + envelope columns; immutable once written
);

-- trial_state_high_water, normalized: "the exact Postulant/trial-state
-- snapshot to be disposed" is per-matrix, and R19's completeness rule needs it
-- queryable, not sealed in jsonb.
CREATE TABLE certification_trial_snapshot (
    certification_ref    uuid NOT NULL REFERENCES candidate_certifications(certification_ref),
    matrix_ref           uuid NOT NULL REFERENCES matrices(matrix_id),
    matrix_revision      int  NOT NULL,
    matrix_status        text NOT NULL,
    trial_state_revision bigint NOT NULL,
    PRIMARY KEY (certification_ref, matrix_ref)
);
```

Staleness is not a certification status: a certification is immutable, and a
promotion attempted against a moved world surfaces as R19's
`STALE_CERTIFICATION` refusal (A.4 row), never as an in-place status flip.

### 2.6 `PromotionPlan`

```sql
-- Annex §6: R19's immutable, digest-bound plan. Its hash covers the ENTIRE
-- plan — header and every disposition row — not only the disposition map.
CREATE TABLE promotion_plans (
    plan_ref                  uuid PRIMARY KEY,
    candidate_space_ref       uuid NOT NULL REFERENCES embedding_spaces(space_id),
    expected_catalog_revision bigint NOT NULL,
    candidate_policy_ref      uuid NOT NULL REFERENCES candidate_policy_snapshots(candidate_policy_ref),
    candidate_policy_digest   text NOT NULL,   -- echoed and revalidated, not merely joined
    preregistration_revision  text NOT NULL,
    certification_ref         uuid NOT NULL REFERENCES candidate_certifications(certification_ref),
    certification_digests     jsonb NOT NULL,  -- every coverage/index/output digest, echoed
    expected_config_revision  bigint NOT NULL, -- expected active config history
    expected_active_space_ref uuid NOT NULL,   -- expected catalog pointer
    expected_graph_cursor     bigint NOT NULL, -- expected live/scoped graph revision
    plan_hash                 text NOT NULL,
    created_at                timestamptz NOT NULL DEFAULT now()
    -- + envelope columns; immutable
);

CREATE TABLE promotion_plan_dispositions (
    plan_ref                 uuid NOT NULL REFERENCES promotion_plans(plan_ref),
    matrix_ref               uuid NOT NULL REFERENCES matrices(matrix_id),
    matrix_revision          int  NOT NULL,
    trial_state_revision     bigint NOT NULL,
    rule                     text NOT NULL CHECK (rule IN
        ('AUTO_SUPERSEDE','SUPERSEDE_REVIEWED','ACKNOWLEDGE_STALE_REFUSAL')),  -- names confirmed (Q7)
    trial_cycle_ref          uuid NULL REFERENCES trial_cycles(trial_cycle_ref),  -- F3: cycle named to the sovereign
    cycle_status             text NULL,                                           -- F3: its state at plan time
    acknowledged_refusal_ref uuid,
    PRIMARY KEY (plan_ref, matrix_ref),
    CHECK ((trial_cycle_ref IS NOT NULL AND cycle_status IS NOT NULL)
           = (rule = 'SUPERSEDE_REVIEWED')),                                      -- F3
    CHECK ((acknowledged_refusal_ref IS NOT NULL)
           = (rule = 'ACKNOWLEDGE_STALE_REFUSAL'))                                -- F3: prose rule made constraint
);
```

The three rule names map onto [TRIAL_AND_EVIDENCE.md](TRIAL_AND_EVIDENCE.md)
§5's table: auto-supersede rows, the consent-applied-AMEND "exact disposition
entry naming the sovereign-reviewed supersession consequence", and the explicit
acknowledgment of a terminal stale-consent refusal. Blocking states are not
plan rows — they refuse the plan. R19's transaction revalidates the whole plan
(hash and every expected revision), applies every disposition, appends
ConfigHistory and moves the pointer, switches the catalog, retires the
displaced space, advances the cursors, and writes the attributed receipt —
envelope and receipt substrate are D7's (ADR-3), consumed by reference.

### 2.7 `TrialCycle`

```sql
-- Annex §1 (trial): identity verbatim.
CREATE TABLE trial_cycles (
    trial_cycle_ref      uuid PRIMARY KEY,
    matrix_ref           uuid NOT NULL REFERENCES matrices(matrix_id),
    matrix_revision      int  NOT NULL,
    cycle_no             int  NOT NULL,   -- evidence-drift retries; NOT VI.4 audit_depth
    evidence_set_ref     uuid NOT NULL REFERENCES trial_evidence_sets(evidence_set_ref),
    prior_cycle_ref      uuid NULL REFERENCES trial_cycles(trial_cycle_ref),
    status               text NOT NULL CHECK (status IN
                             ('VALID','SUPERSEDED','DISTRUSTED','CLOSED')),
    trial_state_revision bigint NOT NULL,  -- snapshot at open; live value is on matrices (§2.10)
    opened_at            timestamptz NOT NULL DEFAULT now(),
    terminal_ref         uuid NULL,
    -- + envelope columns; rows immutable except the status walk to a terminal
    UNIQUE (matrix_ref, matrix_revision, cycle_no)
);
CREATE UNIQUE INDEX one_valid_cycle_per_revision
    ON trial_cycles (matrix_ref, matrix_revision) WHERE status = 'VALID';
```

Uniqueness moves from matrix revision to cycle as the annex rules: one report
per `(trial_cycle_ref, auditor)`, one barrier per cycle, one proposal per cycle
— delivered at P2B as `ALTER`s adding `trial_cycle_ref` to `audit_reports`,
barrier, and proposal tables with the corresponding unique indexes, while the
matrix revision stays in every validation predicate. Atomic opening (annex §2)
is store behavior against these relations, not additional DDL, except that its
"durably enqueues exactly two auditor work items" leans on R03's outbox — D7
substrate, consumed by reference.

### 2.8 `TrialEvidenceSet`

```sql
-- Annex §3 (trial): the frozen manifest of what the cycle actually tried.
CREATE TABLE trial_evidence_sets (
    evidence_set_ref        uuid PRIMARY KEY,
    space_ref               uuid NOT NULL REFERENCES embedding_spaces(space_id),
    catalog_revision        bigint NOT NULL,
    source_cursor           bigint NOT NULL,  -- source graph cursor
    candidate_policy_ref    uuid NULL REFERENCES candidate_policy_snapshots(candidate_policy_ref),
    active_config_revision  bigint NULL,      -- product-context trials cite this
    policy_values           jsonb NOT NULL,   -- exact values incl. coherence + link-similarity thresholds
    algorithm_versions      jsonb NOT NULL,   -- qualification/weight algorithms/versions, weight mode
    reasoner_identity       jsonb NULL,       -- model digest + prompt/policy version, where assisted
    run_manifest            jsonb NOT NULL,   -- calculation-run identities, input/output digests
    sealed_digest           text NOT NULL,   -- INVENTED (name): digest sealing the frozen set (the freeze itself is the annex's; the digest mechanism is chosen here)
    opened_at               timestamptz NOT NULL DEFAULT now(),
    -- + envelope columns; immutable
    CHECK ((candidate_policy_ref IS NULL) <> (active_config_revision IS NULL))  -- exactly one policy citation (matrix row: active jobs cite one active ConfigHistory revision)
);

-- Exact node and bond membership at the tried matrix revision, with the
-- per-member lineage witness. Normalized so §4's dependency-scoped CAS
-- predicates can query membership rather than parse a blob.
CREATE TABLE trial_evidence_members (
    evidence_set_ref           uuid NOT NULL REFERENCES trial_evidence_sets(evidence_set_ref),
    member_kind                text NOT NULL CHECK (member_kind IN ('NODE','BOND')),
    member_ref                 uuid NOT NULL,
    lineage_epoch              bigint,        -- D8 name-compatible (OVERRIDE_LINEAGES §2); no FK until ADR-4's DDL exists
    effective_revision         bigint,        -- D8 name-compatible, same rule
    lineage_witness            jsonb NOT NULL, -- the complete EffectiveLineageWitness — shape OWNED by OVERRIDE_LINEAGES; consumed here by reference, never redefined
    selected_qualification_ref uuid,           -- bonds: chosen BondQualification row
    selected_weight_ref        uuid,           -- bonds: chosen WeightEvidence row (effective weight source)
    PRIMARY KEY (evidence_set_ref, member_kind, member_ref)
);
```

The lineage-witness columns are deliberately FK-less placeholders: their
referents are ADR-4's DDL, which must land before P2B opens trials (both are
P2B-owned, so the ordering is a phase-internal sequencing rule, recorded here
so nobody reads the missing FK as a design).

### 2.9 Measurement split — D4's fields only

Shared matrix row: `similarity, category, qualification, and weight leave the
immutable bond payload for append-only evidence/effective-source records —
D4/D8, each owning only its fields.` D4 owns the measurement identities below;
the columns marked (D8) are name-compatible placeholders whose semantics ADR-4
ratifies — they appear so one table serves both owners, but this ADR proposes
no meaning for them beyond the names in
[OVERRIDE_LINEAGES.md](OVERRIDE_LINEAGES.md) §§1–2.

```sql
-- OVERRIDE_LINEAGES §1 shape, D4 fields: bond_ref · space_ref ·
-- link_policy_ref · similarity · qualified · run_ref.
CREATE TABLE bond_qualifications (
    qualification_ref  uuid PRIMARY KEY,  -- INVENTED (pk name)
    bond_ref           uuid NOT NULL REFERENCES links(link_id),  -- 'Bond' after the split amendment
    space_ref          uuid NOT NULL REFERENCES embedding_spaces(space_id),
    link_config_revision bigint NULL REFERENCES config_history(config_revision),          -- Q9: typed XOR pair
    link_candidate_ref   uuid   NULL REFERENCES candidate_policy_snapshots(candidate_policy_ref),
    similarity         real NOT NULL,
    qualified          boolean NOT NULL,
    lineage_epoch      bigint,            -- (D8)
    effective_revision bigint,            -- (D8)
    run_ref            uuid NOT NULL,
    CHECK ((link_config_revision IS NULL) <> (link_candidate_ref IS NULL))
    -- + envelope columns; append-only
);

-- OVERRIDE_LINEAGES §1 shape, D4 fields: bond_ref · space_ref ·
-- weight_policy_ref · mode · weight · run_ref.
CREATE TABLE weight_evidence (
    weight_evidence_ref uuid PRIMARY KEY,  -- INVENTED (pk name)
    bond_ref            uuid NOT NULL REFERENCES links(link_id),
    space_ref           uuid NOT NULL REFERENCES embedding_spaces(space_id),
    weight_config_revision bigint NULL REFERENCES config_history(config_revision),        -- Q9: typed XOR pair
    weight_candidate_ref   uuid   NULL REFERENCES candidate_policy_snapshots(candidate_policy_ref),
    mode                text NOT NULL,     -- floor | assisted, as weight_mode today
    weight              real NOT NULL,
    lineage_epoch       bigint,            -- (D8)
    effective_revision  bigint,            -- (D8)
    run_ref             uuid NOT NULL,
    CHECK ((weight_config_revision IS NULL) <> (weight_candidate_ref IS NULL))
    -- + envelope columns; append-only
);

-- OVERRIDE_LINEAGES §1: "append-only category source, policy/run identity,
-- lineage epoch, and effective revision." D4's reach: the category value and
-- the policy/run identity that produced it.
CREATE TABLE bond_category_evidence (
    category_evidence_ref uuid PRIMARY KEY, -- INVENTED (pk name)
    bond_ref              uuid NOT NULL REFERENCES links(link_id),
    category              text NOT NULL,
    policy_ref            text NOT NULL,
    run_ref               uuid NOT NULL,
    lineage_epoch         bigint,           -- (D8)
    effective_revision    bigint            -- (D8)
    -- + envelope columns; append-only
);
```

Candidate qualification and weight rows are written into these same
production-shaped tables under the PREPARING space's `space_ref` — they are
non-authoritative because every product reader predicates on the catalog
pointer, not because they live elsewhere (annex §2). No post-switch copy.

### 2.10 Matrix identity additions

```sql
-- Matrix row: "`SUPERSEDED` terminal status plus space, trial-cycle, and
-- evidence-set identity on matrices" (Dogma VI · A.9 · doc 03 §2.4).
ALTER TABLE matrices ADD COLUMN space_ref uuid REFERENCES embedding_spaces(space_id);  -- originating space (annex trial-write predicate)
ALTER TABLE matrices ADD COLUMN source_cursor bigint;         -- emergence's evaluated cursor, distinct from its own insertion's advance (annex §5)
ALTER TABLE matrices ADD COLUMN trial_state_revision bigint NOT NULL DEFAULT 0;  -- advanced by every trial-side transition (trial annex §1)
-- status CHECK gains 'SUPERSEDED': terminal, trigger-enforced like the 0006
-- Cardinal wall; excluded from one_live_matrix_per_category; logged MATRIX_SUPERSEDED.
```

Trial-cycle and evidence-set identity are reachable by key (`trial_cycles` is
unique per `(matrix_ref, matrix_revision, cycle_no)` with one VALID row) rather
than duplicated as mutable matrix columns; if the desk reads the row's "on
matrices" as literal columns, that is §8 Q10.

### 2.11 Taxonomy

- **A.5 events (D4 row, verbatim):** `SPACE_ADOPTED · SPACE_ACTIVATED ·
  SPACE_ABANDONED · SPACE_RETIRED · CANDIDATE_CERTIFIED · MATRIX_SUPERSEDED ·
  CONFIG_CHANGED · TRIAL_SUPERSEDED · TRIAL_DISTRUSTED` — extended into the
  A.5 CHECK constraint by each event's owning phase migration, the 0019
  precedent.
- **A.4 refusals (D4 rows):** `STALE_TRIAL_EVIDENCE` (terminal execution
  refusal, trial annex §4) and `STALE_CERTIFICATION` (promotion refusal,
  space annex §4).

## 3. The catalog switch: bounded catch-up + final micro-fence — decided

**Decision: adopt the annex's recommended algorithm** (space annex §4.1 steps
1–5) as the primary: certify at cursor E; apply relevant deltas repeatedly
while writers continue; when lag < pinned bound, take a **short, fair,
scope-limited final writer micro-fence**; drain, certify exact final state,
execute R19's transaction; release the fence on commit or refusal alike.

Physical shape: the fence is a Postgres advisory-lock pair per affected scope
(the candidate generation's writer entry points and the matrices in the
disposition domain), acquired in a fixed global order with a pinned maximum
hold (criteria-pinned, §7) and queue-fair acquisition; writers classify
themselves only at those entry points, so unrelated writers never wait. Catch-up
reads `graph_deltas WHERE epoch > applied_high_water` filtered to
candidate-relevant entity kinds — the bounded delta set the annex names.

**Rationale:** it is the only shape on the table that proves *both* safety
(the certified state is exactly the switched state — no window between last
validation and pointer move) and liveness (progress under sustained bounded
intake, because catch-up runs unfenced and the fence is entered only below a
pinned lag bound, for a pinned maximum duration).

**Rejected — blanket certification-window freeze:** already declined in review
(`REVIEW_LEDGER.md` :544, "stalls intake"); it buys safety by suspending the
store's first duty.

**Rejected as primary, retained as standing fallback — pure CAS-validated
catalog switch** (`REVIEW_LEDGER.md` :545): optimistic re-certification with a
CAS on catalog/cursor/trial revisions at the switch is safe but its liveness
under sustained intake is exactly the starvation the annex names; unbounded
optimistic retry converges only when intake pauses. If the micro-fence proves
untenable in practice (P1-A evidence, not argument), the fallback is this CAS
switch with `STALE_CERTIFICATION` surfacing every miss — never a longer fence.

**Rejected — scoped epochs / writer classification as the primary mechanism**
(`REVIEW_LEDGER.md` :752 keeps it viable): it can meet the same criteria but
prices a per-scope epoch lattice onto every writer today for a contention
problem v1's single-operator intake does not yet exhibit. Re-openable with
evidence under the same criteria.

## 4. `GraphEpoch`: change cursor, not validity token

The separation of roles is the annex's (§4); the shapes this ADR fixes:

- **DDL:** singleton `graph_epoch` row + append-only `graph_deltas` journal
  (§2.4). No table gains an `epoch_valid_until`-style column anywhere — that
  would be the validity-token reading by the back door.
- **Reader consequence — certification:** candidate cursors
  (`source_cursor`, `applied_delta_high_water`) live on the certification and
  are CAS-checked at R19 against expected revisions, not against "no writes
  happened anywhere".
- **Reader consequence — trials:** cycle validity is the dependency-scoped
  CAS predicate set of the trial annex §4 (matrix revision/status,
  trial_state_revision, space/catalog + policy citation, membership, selected
  runs, effective-lineage state). Unrelated graph writes advance the global
  cursor freely and must not supersede a cycle — provable because no trial
  predicate mentions the global cursor.
- **Reader consequence — explanation:** `explain_link` and audit views read
  `graph_deltas` for ordered history; A.5 stays event history.
- **Writer consequence:** one row-update serialization point for
  graph-affecting transactions, priced and accepted at v1 (§2.4); the
  sequence-based equivalent is pre-authorized by the annex if evidence demands
  it, with semantics (transactional advance, ordered discovery) preserved.

## 5. Production embedder adapter — interface contract only

D3's boundaries stand: this contract lives behind the **model-egress**
boundary (`godhead-model-adapter`, governed operations to configured local
aliases); no vendor code, no transport in domain crates, fetch-egress stays
sealed until P6. The seam follows the house pattern (`ScanEndpoint`,
`FetchEndpoint`): a trait, an instrumented deterministic mock, a real provider
that arrives at its phase.

```rust
/// The adapter's professed identity — the fields EmbeddingSpace freezes.
pub struct EmbedderIdentity {
    pub provider: String,
    pub model_name: String,
    pub model_revision: String,   // exact revision/digest
    pub dims: u32,
    pub normalization: String,
    pub pooling: String,
    pub chunking_policy: serde_json::Value,
}

pub trait EmbedEndpoint: Send + Sync {
    /// Professed identity; verified against the space record before ANY
    /// vector is accepted. A space is minted FROM this, never assumed.
    fn identity(&self) -> Result<EmbedderIdentity, EmbedError>;
    /// Embed one prepared input under the space's chunking/truncation policy.
    fn embed(&self, input: EmbedInput) -> Result<Vec<f32>, EmbedError>;
}
```

(Async/batching surface is the implementing phase's; the contract is the
method pair and the taxonomy.)

**Failure taxonomy (closed, refuse-don't-guess):**

| Variant | Meaning | Consumer behavior |
|---|---|---|
| `ENDPOINT_UNAVAILABLE` | separate process absent/unresponsive | retry per job policy; never fall back to a different geometry |
| `IDENTITY_MISMATCH` | professed identity ≠ space identity | hard refusal; a changed model is a NEW space (R18), never a remap |
| `DIMENSION_MISMATCH` | returned dims ≠ space dims | hard refusal; corrupt generation marks FAILED with attribution |
| `INPUT_REJECTED` | input violates model constraints after policy application | per-item refusal, recorded |
| `MALFORMED_OUTPUT` | NaN/Inf/non-numeric/wrong shape | hard refusal; attribution on the generation if systematic |
| `TIMEOUT` | pinned budget exceeded | retry per job policy; budget is config, cited |
| `INTERNAL(detail)` | attributed catch-all | refusal; never silent |

Production is the **separate-process adapter** (D4a as answered); an
in-process engine is contingency only, and any contingency result feeding a
verdict must be replicated on the production serving path (annex §3). The
deterministic mock is the gate's embedder; nothing in P1-A's gate calls a real
model.

## 6. Appendix-A naming proposal (naming only — promotion is the amendment process's)

**Proposed for A-numbers (canonical relations — law cites them):**
`EmbeddingSpace` · `SpaceCatalog` (the pointer; product readers predicate on
it) · `ConfigHistory` (+ its active pointer) · `CandidatePolicySnapshot` ·
`CandidateCertification` · `PromotionPlan` (with its disposition rows) ·
`GraphEpoch` · `TrialCycle` · `TrialEvidenceSet` (with its member rows) ·
`BondQualification` · `WeightEvidence` · `BondCategoryEvidence`.

**Proposed for dev-register residence (application records — consistent with
the matrix's "deliberate non-canonical application records" note):**
`CandidateEvaluationContext` and every candidate-harness read model ·
calculation-run bookkeeping behind `run_ref`/`run_manifest` ·
`graph_deltas` (the journal is implementation of discovery; the **cursor** is
the canonical object — if the desk wants the journal canonical for
explanation's sake, that is §8 Q6) · the `embedding_archive` (audit
convenience over sovereign-retained bytes; A.14(c) destruction terms already
govern the bytes themselves).

## 7. Criteria hooks

**Existing SC rows this substrate serves:** SC-M01–SC-M06 (SC-M05 as amended:
one vector per node *per valid space*); SC-D01 (every evaluation cites its
policy — extended by the candidate-snapshot amendment row); SC-D04/SC-D05
(report/barrier identity moves to cycle scope); SC-D02/SC-D03/SC-D06–SC-D10
(trial machinery citing cycle + evidence set through VI.3 handoffs); VI.3/VI.4
(proposal/consent citation; `audit_depth` vs `cycle_no` distinction).

**AC rows P1's preregistration will need** (named now, minted into the
register at P0 exit — numbering is the register's, not this ADR's; the
formulations are the annexes' §9/§6 lists):

- AC (space): one-vector-per-node-per-valid-space; no mixed-space computation.
- AC (space): PREPARING writes product-invisible until one catalog switch.
- AC (space): candidate outputs need no post-switch copy.
- AC (space): every candidate computation cites one immutable snapshot.
- AC (space): emergence cannot insert from a stale evaluated state.
- AC (space): certification binds exact generation/policy/cursor/trial identities.
- AC (space): promotion completes under sustained bounded intake within pinned
  catch-up lag, bounded retries, maximum fence duration, and fairness bounds
  (§3's pinned numbers live in the register row, not prose).
- AC (space): unrelated graph writes do not invalidate trials.
- AC (space): R19 is one envelope/receipt/transaction; R09 absent from `proceed`.
- AC (space): R19 refuses any changed policy/certification/catalog/graph/
  trial/disposition input.
- AC (space): bootstrap seed adoption cannot authorize runtime abandonment.
- AC (space): Cardinals stay frozen while drift remains legible.
- AC (rider): the P1-B time-box and its overrun-is-a-finding clause — the D4
  rider says this enters the register as a citable rule.
- AC (trial, P2B pins): the eleven-item list of
  [TRIAL_AND_EVIDENCE.md](TRIAL_AND_EVIDENCE.md) §6, named here for
  completeness; P2B's pin owns their register entries.

## 8. Open questions for the desk

Each with a recommendation; format: Qn — question — **rec:** recommendation.

1. Q1 — Per-space typed generation tables require `CREATE TABLE` at runtime R18 (template §2.1); is dynamic DDL inside a receipted sovereign command acceptable, or must every generation be migration-authored? — **rec:** accept templated DDL under R18's receipt (the template is pinned here; the first candidate is bootstrap-migrated anyway, so runtime DDL first fires post-P2A behind the authority perimeter).
2. Q2 — `graph_deltas` is an invented table (ordered delta discovery needs a discoverable record); approve its existence and shape? — **rec:** approve; without it, catch-up degenerates to full rescans and "bounded delta set" is unmeasurable.
3. Q3 — `CandidateCertification.status` value set is invented (`CERTIFIED|FAILED`); staleness deliberately lives in R19's `STALE_CERTIFICATION` refusal, not a status flip. Confirm? — **rec:** confirm two-value set; immutability of certifications is worth more than a third status.
4. Q4 — `space_catalog.purpose` key (invented) fixes a single-purpose catalog (`NODE_EMBEDDING`) with room for future purposes; acceptable, or single-row table? — **rec:** keep the purpose key; it costs one constant now and saves a rekeying migration when a second vector purpose arrives.
5. Q5 — GraphEpoch physical shape: single-row `UPDATE … RETURNING` (serializes graph-affecting commits) vs transactional-sequence emulation. — **rec:** single row at v1; the annex pre-authorizes the sequence swap on evidence, and overrun-is-a-finding discipline applies to contention too.
6. Q6 — Is `graph_deltas` dev-register (implementation journal, my proposal §6) or A-series (explanatory history beside A.5)? — **rec:** dev-register now, promotable by D5's path if explanation surfaces come to cite it in law.
7. Q7 — Disposition rule names `AUTO_SUPERSEDE | SUPERSEDE_REVIEWED | ACKNOWLEDGE_STALE_REFUSAL` are invented labels for the annex §5 rows; confirm or rename at the desk. — **rec:** confirm; three names, three annex rules, no residue.
8. Q8 — `config_constants` coexistence: keep it as the store-maintained active-value projection indefinitely, or retire it once `ConfigHistory` + pointer are proven? — **rec:** keep through P2B (SC-D01 citations keep meaning), revisit at the amendment that retires the bond-payload columns.
9. Q9 — `link_policy_ref`/`weight_policy_ref` typing (the pre-ruling names): text refs resolving to either a ConfigHistory revision or a CandidatePolicySnapshot (context-dependent), or two nullable typed columns with an XOR check (the `trial_evidence_sets` pattern)? — **rec:** two typed columns + XOR, matching §2.8; a stringly union in an append-only evidence row is a parsing debt forever. — **Ruled (§10): TYPED XOR PAIR adopted, applied in §2.9** (`link_config_revision`/`link_candidate_ref`, `weight_config_revision`/`weight_candidate_ref`); `bond_category_evidence.policy_ref` deliberately untouched — its trigger semantics ride ADR-4's D8 pass.
10. Q10 — Matrix row says space, **trial-cycle, and evidence-set identity on matrices**; this ADR puts space_ref/source_cursor/trial_state_revision on the row and reaches cycle/evidence identity by key (§2.10). Literal columns instead? — **rec:** by-key; a mutable current-cycle pointer on the matrix is the revision-churn the cycle table exists to absorb.
11. Q11 — `EmbeddingSpace.space_no` ordinal (invented column name for the annex's own "space 0" usage): keep as UNIQUE int, or derive the floor's identity some other way? — **rec:** keep; the generation-table template needs a stable short name and "space 0" is already the annexes' vocabulary.
12. Q12 — `certification_trial_snapshot` normalizes the annex's `trial_state_high_water` field into per-matrix rows; confirm the normalization? — **rec:** confirm; R19's completeness rule ("submitted disposition domain must equal the set") is a set-equality query against it.

## 9. D4 matrix-row consumption map (self-review record)

| D4 row (AMENDMENT_MATRIX.md) | Disposition here |
|---|---|
| one vector per node per valid space (SC-M05) | Consumed: §2.1 generations; hook §7 |
| measurement split (shared D4/D8) | Consumed for D4's fields: §2.9; D8 columns name-compatible placeholders, no D8 semantics drafted |
| `SUPERSEDED` + space/trial/evidence identity on matrices | Consumed: §2.10 (+ Q10 on the by-key reading) |
| A.5 event names | Consumed: §2.11, names verbatim; CHECK extension rides each owning phase's migration |
| append-only `ConfigHistory` + active pointer | Consumed: §2.2 |
| VI.1·A.14·SC-D01 candidate-snapshot amendment | **Deferred with reason:** canon amendment text is the amendment process's, not an ADR's; the relation it names is §2.3 |
| `CandidatePolicySnapshot`/`CandidateCertification`/`PromotionPlan` | Consumed: §§2.3, 2.5, 2.6 |
| transactional `GraphEpoch`, scoped validity, bounded catch-up/fair cutover | Consumed: §§2.4, 3, 4 |
| append-only `TrialCycle` + cycle-scoped uniqueness | Consumed: §2.7 (uniqueness `ALTER`s at P2B) |
| A.4 `STALE_TRIAL_EVIDENCE` | Consumed: §2.11 |
| A.4 `STALE_CERTIFICATION` | Consumed: §2.11 |

---

*Drafted 2026-07-10 by the roadmap lane under the ADR-2 commission; returns
through the desk per the D4 rider. Adoption, if it comes, authorizes the
owning phases to land this DDL under the two-commit lifecycle — it does not
land anything by itself.*

## 10. Desk rulings (2026-07-10)

Reviewed at the desk per the D4 rider (DESK_REVIEW_ADR2, desk outputs, 2026-07-10).
Verdict: APPROVED WITH AMENDMENTS — findings F1–F3 and rulings Q1/Q9 are applied
in this commit; the ruling text below is the desk's, quoted.

**Findings (applied):**
- **F1** — Generation failure gains its carrier: a failed model, corrupt
  generation, or failed certification is recorded as a machine-attributed
  `FAILED` row in `candidate_certifications`; mid-backfill failure writes one
  exactly as certification failure does; the space row stays `PREPARING`,
  visibly awaiting R20 (space annex §1). Sentence added in §2.1; no new column.
- **F2** — One graph-affecting transaction advances the cursor exactly once and
  appends one `graph_deltas` row per touched entity, all sharing that epoch;
  primary key is now `(epoch, entity_kind, entity_ref)`. §2.4 amended.
- **F3** — `SUPERSEDE_REVIEWED` dispositions now name the cycle to the
  sovereign, as trial annex §5 demands ("matrix revision, cycle state, and the
  sovereign-reviewed supersession consequence"): `trial_cycle_ref` +
  `cycle_status`, CHECK-required iff that rule; `acknowledged_refusal_ref`'s
  prose rule is now a CHECK. §2.6 amended.

**Rulings on §8:**
- **Q1 ACCEPTED, hardened** — templated runtime DDL is lawful under R18's
  receipt, on two conditions now in §2.1: the statement is rendered verbatim
  from the pinned template (deviation is a finding, not a variant), and the
  receipt embeds a sha256 of the rendered DDL text. What ran is what was
  receipted.
- **Q2 APPROVED** — `graph_deltas` exists; shape as amended by F2.
- **Q3 CONFIRMED** — two-value certification status; staleness is R19's
  refusal; certifications stay immutable.
- **Q4 CONFIRMED** — keep the `purpose` key; one constant now beats a rekeying
  migration later.
- **Q5 CONFIRMED** — single-row epoch at v1; the annex pre-authorizes the
  sequence swap on evidence; contention overrun is a finding.
- **Q6 DEV-REGISTER** — the cursor is the canonical object; the journal is
  discovery's implementation, D5-promotable if explanation surfaces come to
  cite it in law.
- **Q7 CONFIRMED** — three names, three annex rows, no residue.
- **Q8 KEEP through P2B** — `config_constants` remains the active-value
  projection so SC-D01 citations keep meaning; retirement is priced at the
  amendment that retires the bond-payload columns, not before.
- **Q9 TYPED XOR PAIR adopted** — applied in §2.9; a stringly union in
  append-only evidence is parsing debt forever. `bond_category_evidence.policy_ref`
  typing is NOT changed here: that column's trigger semantics ride ADR-4's D8
  pass.
- **Q10 BY-KEY CONFIRMED** — the one-VALID unique key satisfies matrix row 22's
  "identity on matrices"; a mutable current-cycle pointer is the churn the
  cycle table absorbs. This ruling is the recorded reading of that row — cite
  it at delivery.
- **Q11 CONFIRMED** — "space 0" is already the record's vocabulary; make it a
  column.
- **Q12 CONFIRMED** — per-matrix normalization stands; R19's completeness rule
  becomes a set-equality query, which is the point.
