use crate::error::StoreError;
use crate::types::{ArtifactDraft, ArtifactRecord, ComplianceMetrics};
use godhead_schemas::{
    AuditReport, AuditReportDraft, ConcordatArtifact, ConfigConstant, ConfigTier, ConsentDecision,
    EmbeddingRecord, EnvItem, EnvKind, EnvironmentRecord, FlagDraft, FlagStatus, InstructionDraft,
    InstructionRecord, JobDraft, JobRecord, JobStatus, JointProposal, LeaseRecord, LinkRecord,
    LiveWeights, LogEvent, LogSnapshot, MatrixRecord, NodeDraft, NodeRecord, NormalizeOutcome,
    OverrideRecord, PairingKind, PairingRecord, PetitionDraft, PetitionRecord, ProposalDraft,
    ReadinessFlag, RebalanceState, RefusalDraft, RefusalRecord, Severity, SourceDraw, Tier,
};
use semver::Version;
use uuid::Uuid;

/// The single abstracted store interface (doc 3 §1.1) — the sole surface
/// through which agents touch persistent state (Law III.1). Criteria hold
/// across substrate swaps: the slice-1 tests exercise this trait, not the
/// Postgres type.
///
/// Identity: every mutating call names the job it acts under (Law XIII —
/// the store rejects anonymous writes; an unknown job_id IS anonymity).
/// Config mutation deliberately takes a human/deployment actor string
/// instead of a job identity: no constant of either tier is agent-writable
/// (Law IV.4), and the signature is where that impossibility lives.
//
// async fn in a public trait warns about un-nameable future types for
// generic callers; slice 1 calls through concrete or `impl Store` bounds
// only, so the suppression is narrow and deliberate.
#[allow(async_fn_in_trait)]
pub trait Store {
    // -- jobs (Law I) --

    /// Spawn-time record creation. Budgets are validated (Law XIV) before
    /// any write; status starts PENDING.
    async fn create_job(&self, draft: &JobDraft) -> Result<JobRecord, StoreError>;

    async fn get_job(&self, job_id: Uuid) -> Result<JobRecord, StoreError>;

    /// Forward-only transitions (Law I.1) under compare-and-swap (Law XI.3).
    /// FLAGGED is entered only via `write_flag`, REFUSED only via `refuse`;
    /// this method rejects both targets.
    async fn transition_job(
        &self,
        job_id: Uuid,
        expected_revision: i32,
        to: JobStatus,
    ) -> Result<JobRecord, StoreError>;

    // -- artifacts (Laws I.3, II) --

    /// Idempotent keyed write: upserts on `(job_id, output_slot)`. Validates
    /// the declared schema before any write; scans for secrets (Law XV.2);
    /// rejects store-issued field names in the payload (Law XII).
    async fn write_artifact(
        &self,
        job_id: Uuid,
        output_slot: &str,
        draft: &ArtifactDraft,
    ) -> Result<ArtifactRecord, StoreError>;

    /// Reads an authoritative artifact. Partials of refused jobs are
    /// invisible here (Law VII.5).
    async fn read_artifact(
        &self,
        job_id: Uuid,
        output_slot: &str,
    ) -> Result<ArtifactRecord, StoreError>;

    // -- flags (Law III) --

    /// The FLAG step: certifies integrity (III.2). Writable only after the
    /// certified outputs exist and validate; atomically moves the job
    /// WRITTEN → FLAGGED.
    async fn write_flag(
        &self,
        job_id: Uuid,
        draft: &FlagDraft,
    ) -> Result<ReadinessFlag, StoreError>;

    async fn get_flag(&self, flag_id: Uuid) -> Result<ReadinessFlag, StoreError>;

    /// Status supersession under CAS — the only lawful way a flag changes
    /// (III.4; deletion is rejected at the substrate).
    async fn supersede_flag(
        &self,
        flag_id: Uuid,
        expected_revision: i32,
        to: FlagStatus,
    ) -> Result<ReadinessFlag, StoreError>;

    /// Law III.3 — a flag is testimony; the state is the witness. Re-validates
    /// every certified output under the reader's identity; on failure sets
    /// the flag DISTRUSTED and returns FLAG_UNTRUSTED.
    async fn read_certified(
        &self,
        reader_job_id: Uuid,
        flag_id: Uuid,
    ) -> Result<Vec<ArtifactRecord>, StoreError>;

    // -- refusal (Law VII) --

    /// Refuse, flag, preserve: writes the RefusalRecord, quarantine-marks
    /// the job's partials non-authoritative, releases its leases, and moves
    /// the job to REFUSED. Mutates nothing else (VII.3).
    async fn refuse(&self, job_id: Uuid, draft: &RefusalDraft)
        -> Result<RefusalRecord, StoreError>;

    // -- leases (Law XI) --

    /// Acquire-or-refuse; no waiting, no spinning. An expired lease on the
    /// subject does not block acquisition (XI.2 routes the stale job to
    /// recovery).
    async fn acquire_lease(
        &self,
        job_id: Uuid,
        subject_ref: Uuid,
        ttl_ms: i64,
    ) -> Result<LeaseRecord, StoreError>;

    async fn heartbeat_lease(
        &self,
        job_id: Uuid,
        lease_id: Uuid,
    ) -> Result<LeaseRecord, StoreError>;

    async fn release_lease(&self, job_id: Uuid, lease_id: Uuid) -> Result<(), StoreError>;

    // -- logs (Law V) --

    /// Appends a snapshot, chaining `prior_ref` to the subject's previous
    /// snapshot (V.1 rotation). Payload is secret-scanned (XV.2).
    async fn write_log(
        &self,
        subject_ref: &str,
        event: LogEvent,
        payload: &serde_json::Value,
        severity: Severity,
    ) -> Result<LogSnapshot, StoreError>;

    /// All snapshots for a subject, in store-sequence order (Law XII.2).
    async fn read_logs(&self, subject_ref: &str) -> Result<Vec<LogSnapshot>, StoreError>;

    // -- config (A.14) --

    /// Human/deployment administration only — there is deliberately no
    /// job-identity parameter. CAS on update: `expected_revision` must match
    /// unless the key is new.
    async fn set_config(
        &self,
        changed_by: &str,
        key: &str,
        tier: ConfigTier,
        value: &serde_json::Value,
        expected_revision: Option<i32>,
    ) -> Result<ConfigConstant, StoreError>;

    async fn get_config(&self, key: &str) -> Result<ConfigConstant, StoreError>;

    // -- metrics (Law VII.4) --

    /// The reference metrics query: refusals score as compliance, never error.
    async fn compliance_metrics(&self, job_ids: &[Uuid]) -> Result<ComplianceMetrics, StoreError>;

    // -- nodes (doc 3 §2.1; doc 2) --

    /// Creates the atom and writes its first log snapshot (INTAKE_RAW_COPIED
    /// with filename, filetype, size, normalized-state) in the same act —
    /// first-log-on-copy is a store guarantee (doc 2 §2.2), not caller
    /// diligence. Raw reference fields exist only in this call.
    async fn create_node(
        &self,
        job_id: Uuid,
        node_id: Uuid,
        draft: &NodeDraft,
    ) -> Result<NodeRecord, StoreError>;

    async fn get_node(&self, node_id: Uuid) -> Result<NodeRecord, StoreError>;

    /// Records a normalization outcome (success, decode failure, or
    /// unsupported type) under CAS, logging NORMALIZED — severity warning
    /// when the outcome is a surfaced failure (flag, don't bury).
    async fn set_node_derivative(
        &self,
        job_id: Uuid,
        node_id: Uuid,
        expected_revision: i32,
        outcome: &NormalizeOutcome,
    ) -> Result<NodeRecord, StoreError>;

    /// Records the floor classification under CAS, logging CLASSIFIED.
    async fn set_node_classification(
        &self,
        job_id: Uuid,
        node_id: Uuid,
        expected_revision: i32,
        classification: &serde_json::Value,
    ) -> Result<NodeRecord, StoreError>;

    // -- orchestration reads (doc 3 §3.2) --

    /// ACTIVE flags for one stage, in store order — the dispatcher's watch.
    async fn list_active_flags(&self, stage: &str) -> Result<Vec<ReadinessFlag>, StoreError>;

    /// Every flag a job has written, any status.
    async fn list_flags_for_job(&self, job_id: Uuid) -> Result<Vec<ReadinessFlag>, StoreError>;

    /// Jobs whose input_refs contain the given ref — the supervisor's
    /// reconstruction primitive (doc 3 §4.1: the index is rebuilt from
    /// flags and job records, never from private memory).
    async fn list_jobs_by_input_ref(&self, input_ref: Uuid) -> Result<Vec<JobRecord>, StoreError>;

    // -- sovereignty (Law IV) --
    // Sovereign acts take a human actor string and no job identity: the
    // signature is where agent-uncallability lives, and the substrate's
    // agent-author trigger backs it below the API (SC-C07).

    /// The sovereign's hand on a node's classification: applies the change
    /// and lays the protection in one act. Re-laying chains prior_ref.
    async fn lay_category_override(
        &self,
        actor: &str,
        node_id: Uuid,
        classification: &serde_json::Value,
    ) -> Result<OverrideRecord, StoreError>;

    /// The latest override on a subject — the active protection.
    async fn get_active_override(
        &self,
        subject_ref: Uuid,
    ) -> Result<Option<OverrideRecord>, StoreError>;

    /// An agent's only voice on human-held state (IV.2). One lineage per
    /// (subject, kind): recurrence escalates OPEN → ESCALATED; a SILENCED
    /// lineage suppresses the attempt, still logged `severity: suppressed`.
    async fn open_petition(
        &self,
        job_id: Uuid,
        draft: &PetitionDraft,
    ) -> Result<PetitionRecord, StoreError>;

    /// The three terminal answers: GRANTED (mints the consent), DECLINED,
    /// SILENCED. Human only.
    async fn resolve_petition(
        &self,
        actor: &str,
        petition_id: Uuid,
        decision: ConsentDecision,
    ) -> Result<PetitionRecord, StoreError>;

    /// IV.5, transactional: validates the chain override → petition →
    /// consent, applies exactly the granted change, lays the successor
    /// override (stamped with the consent's decider — the authority is the
    /// consent, never the Notary), and closes the loop on the petition.
    /// Idempotent: a retry finding the grant executed returns the successor.
    async fn execute_grant(
        &self,
        notary_job_id: Uuid,
        petition_id: Uuid,
    ) -> Result<OverrideRecord, StoreError>;

    /// GRANTED petitions with no completed execution older than the stall
    /// window — what the supervisor surfaces (SC-C06). Nothing the
    /// sovereign grants may quietly fail to happen.
    async fn stalled_grants(&self, stall_ms: i64) -> Result<Vec<PetitionRecord>, StoreError>;

    async fn get_petition(&self, petition_id: Uuid) -> Result<PetitionRecord, StoreError>;

    // -- embeddings & links (doc 3 §2.2–2.3; doc 4) --

    /// Persists a node's vector. Converges on conflict (the existing row is
    /// returned untouched — one embedding per node, doc 3 §2.2), logs
    /// EMBEDDED, and marks the node's category recalculation-eligible
    /// (ingestion event, doc 4 §5.2).
    async fn put_embedding(
        &self,
        job_id: Uuid,
        node_id: Uuid,
        embedder_alias: &str,
        vector: &[f32],
    ) -> Result<EmbeddingRecord, StoreError>;

    async fn get_embedding(&self, node_id: Uuid) -> Result<Option<EmbeddingRecord>, StoreError>;

    /// Normalized nodes with no persisted vector — the "flagged for
    /// backfill" surface (SC-M06). Scope limits to a node set.
    async fn embedding_backlog(
        &self,
        scope: Option<&[Uuid]>,
    ) -> Result<Vec<NodeRecord>, StoreError>;

    /// Native pgvector cosine similarity against a node's stored vector,
    /// most similar first, at or above `min_similarity`.
    async fn similar_nodes(
        &self,
        node_id: Uuid,
        min_similarity: f32,
        scope: Option<&[Uuid]>,
    ) -> Result<Vec<(Uuid, f32)>, StoreError>;

    /// Draws (or refreshes) the bond between two nodes in canonical order.
    /// An overridden link is never touched (doc 4 §4.4). Logs LINK_DRAWN on
    /// creation and marks the category eligible.
    async fn draw_link(
        &self,
        job_id: Uuid,
        a: Uuid,
        b: Uuid,
        similarity: f32,
        category: &str,
    ) -> Result<LinkRecord, StoreError>;

    async fn links_by_category(
        &self,
        category: &str,
        scope: Option<&[Uuid]>,
    ) -> Result<Vec<LinkRecord>, StoreError>;

    /// A single link by id — a link is a first-class store object (doc 3
    /// §2.3); resolution checks (lint a) need to resolve one.
    async fn get_link(&self, link_id: Uuid) -> Result<LinkRecord, StoreError>;

    /// CAS weight write; an overridden link refuses OVERRIDE_CONFLICT —
    /// recalculation works around fixed stars (Handbook §4.5).
    async fn set_link_weight(
        &self,
        job_id: Uuid,
        link_id: Uuid,
        expected_revision: i32,
        weight: f32,
    ) -> Result<LinkRecord, StoreError>;

    /// The consumer weight surface (SC-M02): evaluates category link-density
    /// against the sovereign coherence_threshold, citing the config revision
    /// read (Law VI.1 — no threshold set is a refusal, never a guess).
    /// Below the line, weights are inert: present in records, absent here.
    async fn live_weights(
        &self,
        category: &str,
        scope: Option<&[Uuid]>,
    ) -> Result<LiveWeights, StoreError>;

    async fn rebalance_state(&self, category: &str) -> Result<Option<RebalanceState>, StoreError>;

    /// Atomically claims a category's pending eligibility: sets
    /// eligible=false and stamps last_recalc_at iff it was eligible,
    /// returning whether the claim won. This is the tick's
    /// check-and-act in one statement — N racing executors, one claim.
    /// `config_rev` cites the sovereign threshold revision the execution
    /// runs under, or None when the sovereign has never set one (a null
    /// citation, never a fabricated revision — Law VI.1).
    async fn claim_rebalance_eligibility(
        &self,
        category: &str,
        config_rev: Option<i32>,
    ) -> Result<bool, StoreError>;

    /// Marks a category recalculation-eligible. The store calls this
    /// itself on ingestion events (doc 4 §5.2); the public surface exists
    /// so an executor whose claimed pass failed can restore the mark it
    /// consumed — eligibility is never lost to a failed labor.
    async fn mark_rebalance_eligible(&self, category: &str) -> Result<(), StoreError>;

    /// The store's clock (Law XII: the only clock any elapsed-time
    /// judgment may consult).
    async fn store_now(&self) -> Result<time::OffsetDateTime, StoreError>;

    // -- multi-stage FLAG (Law I.3: flag writes are idempotent upserts) --

    /// The FLAG step certifying several stages in one act: every draft's
    /// certified outputs must exist and validate; the WRITTEN → FLAGGED
    /// transition happens exactly once. `write_flag` is the single-draft
    /// special case.
    async fn write_flags(
        &self,
        job_id: Uuid,
        drafts: &[FlagDraft],
    ) -> Result<Vec<ReadinessFlag>, StoreError>;

    // -- matrices & the commitment chain (Law VI) --

    /// The Aggregator's step 3 (VI.2): evaluates the category's
    /// link-density against the sovereign coherence threshold — citing the
    /// revision read, always — and on crossing creates the Postulant.
    /// Returns None below the threshold or when a live matrix already
    /// stands (one live matrix per category; emergence is idempotent).
    /// Errors if the sovereign threshold is unset: no citation, no
    /// evaluation (Law VI.1).
    async fn emerge_postulant(
        &self,
        job_id: Uuid,
        category: &str,
        scope: Option<&[Uuid]>,
    ) -> Result<Option<MatrixRecord>, StoreError>;

    async fn get_matrix(&self, matrix_id: Uuid) -> Result<MatrixRecord, StoreError>;

    async fn live_matrix_for_category(
        &self,
        category: &str,
    ) -> Result<Option<MatrixRecord>, StoreError>;

    /// Files an audit report under the truth-binding (Book II §2): every
    /// claim's evidence_refs MUST resolve to live nodes or links; a report
    /// carrying an unsupported word is refused before any write and never
    /// flags.
    async fn file_audit_report(
        &self,
        job_id: Uuid,
        draft: &AuditReportDraft,
    ) -> Result<AuditReport, StoreError>;

    /// Reads a report under the isolation rule (SC-D04): the filer may
    /// read its own; anyone else reads only after the AND-barrier has
    /// certified that matrix revision. A pre-barrier cross-read is
    /// rejected and logged.
    async fn read_audit_report(
        &self,
        reader_job_id: Uuid,
        report_id: Uuid,
    ) -> Result<AuditReport, StoreError>;

    /// Both reports for a matrix revision, in auditor order — the
    /// barrier's and Reconciliation's read.
    async fn audit_reports_for(
        &self,
        matrix_id: Uuid,
        matrix_revision: i32,
    ) -> Result<Vec<AuditReport>, StoreError>;

    /// The supervisor as certifier, not driver (doc 3 §3.3): when both
    /// auditors' flags are present and both underlying reports re-validate,
    /// writes the composite barrier flag (office-authored, no job).
    /// Missing or invalid prerequisites hold the barrier (error).
    async fn certify_audit_barrier(&self, matrix_id: Uuid) -> Result<ReadinessFlag, StoreError>;

    /// True iff the barrier flag stands for this matrix revision.
    async fn audit_barrier_certified(
        &self,
        matrix_id: Uuid,
        matrix_revision: i32,
    ) -> Result<bool, StoreError>;

    /// Reconciliation's output (Book II §2 step 4): one Joint Proposal per
    /// matrix revision, filed only behind a certified barrier, its
    /// amendments resolving into the matrix's membership.
    async fn file_joint_proposal(
        &self,
        job_id: Uuid,
        draft: &ProposalDraft,
    ) -> Result<JointProposal, StoreError>;

    async fn get_proposal(&self, proposal_id: Uuid) -> Result<JointProposal, StoreError>;

    /// The sovereign's answer to a Joint Proposal — human only. GRANTED
    /// mints the consent the Notary's chain requires; DECLINED leaves the
    /// Postulant standing (decline is signal, and the halt of Law VI.4).
    async fn resolve_proposal(
        &self,
        actor: &str,
        proposal_id: Uuid,
        decision: ConsentDecision,
    ) -> Result<JointProposal, StoreError>;

    /// The Notary's commitment labor (VI.3–VI.5): validates the full
    /// proposal → consent chain, then applies exactly the consented
    /// verdict — COMMIT → CARDINAL, AMEND → revision N+1 with precisely
    /// the enumerated changes, REJECT → DISSOLVED (links persist).
    /// Idempotent: a retry finding the verdict applied converges.
    async fn execute_matrix_proposal(
        &self,
        notary_job_id: Uuid,
        proposal_id: Uuid,
    ) -> Result<MatrixRecord, StoreError>;

    /// Human-invoked decommission (VI.5): mints the consent the Notary
    /// executes against. Returns the consent id.
    async fn consent_decommission(&self, actor: &str, matrix_id: Uuid) -> Result<Uuid, StoreError>;

    /// The Notary's decommission labor: CARDINAL → DISSOLVED under a
    /// resolving consent; the dissolved matrix's links persist.
    async fn execute_decommission(
        &self,
        notary_job_id: Uuid,
        matrix_id: Uuid,
        consent_id: Uuid,
    ) -> Result<MatrixRecord, StoreError>;

    // -- environments & pairings (Laws IX–X) --

    /// Establishes an environment around a matrix, conferring title and
    /// name deterministically at establishment (X.1, X.4) — the conferral
    /// is recorded immutably. The establishing job's tier is the
    /// environment's; REGULAR establishes nothing.
    async fn establish_environment(
        &self,
        job_id: Uuid,
        kind: EnvKind,
        matrix_ref: Uuid,
        tier: Tier,
    ) -> Result<EnvironmentRecord, StoreError>;

    async fn get_environment(&self, env_id: Uuid) -> Result<EnvironmentRecord, StoreError>;

    /// Curates the contents index (an election, a published artifact).
    /// Refused on a non-LIVE environment: an ORPHANED room is not a
    /// workplace (SC-G07). `provenance` is a ProvenanceChain (C.2 shape).
    async fn add_env_item(
        &self,
        job_id: Uuid,
        env_id: Uuid,
        item_ref: Uuid,
        provenance: &serde_json::Value,
        flagged: bool,
    ) -> Result<EnvItem, StoreError>;

    async fn env_items(&self, env_id: Uuid) -> Result<Vec<EnvItem>, StoreError>;

    /// The Law IX.3 mount: floor validation before any work. ENV_INVALID
    /// on any failure — record malformed, tier/title disagreement, an
    /// item that does not resolve, or a provenance chain that does not
    /// walk root-to-leaf (SC-G01, G05, G06). ORPHANED/DISSOLVED are
    /// unmountable for work (SC-G07).
    async fn mount_environment(
        &self,
        job_id: Uuid,
        env_id: Uuid,
    ) -> Result<EnvironmentRecord, StoreError>;

    /// Scoping has force (IX.4) with the Pairing Exception (IX.5): a read
    /// is permitted iff `target_ref` is in the env's contents index, or an
    /// allowlist item (the reader's own job or lease), or a *flagged* item
    /// of a paired counterpart environment. Otherwise rejected and logged
    /// (severity: violation). Ok(()) means permitted.
    async fn env_scoped_read(
        &self,
        reader_job_id: Uuid,
        env_id: Uuid,
        target_ref: Uuid,
    ) -> Result<(), StoreError>;

    /// Forms a pairing (X.5): tiers must match kind; REGULAR anywhere
    /// fails. The pairing record is the grant the Exception reads.
    async fn form_pairing(
        &self,
        teacher_env_ref: Uuid,
        student_env_ref: Uuid,
        matrix_ref: Uuid,
        kind: PairingKind,
    ) -> Result<PairingRecord, StoreError>;

    /// LIVE → ORPHANED: the dependency is lost; the room becomes a
    /// read-only archive (A.8).
    async fn orphan_environment(&self, env_id: Uuid) -> Result<EnvironmentRecord, StoreError>;

    // -- the Concordat & Instructions (Holy Standard §1, §3) --

    /// Adopts a Concordat version (human/sovereign — A.14 test (b)). Every
    /// version ever adopted is retained forever (§3.3).
    async fn adopt_concordat(
        &self,
        actor: &str,
        version: &Version,
        capability_tables: &serde_json::Value,
        pairing_semantics: &serde_json::Value,
    ) -> Result<ConcordatArtifact, StoreError>;

    async fn get_concordat(&self, version: &Version) -> Result<ConcordatArtifact, StoreError>;

    /// Persists an Instruction body (unflagged). The lint runs in the
    /// concordat crate before this; here the store validates B.1 shape and
    /// the supersedes chain. `skew` is DERIVED from `sources_drawn` against
    /// `bias_skew_threshold` (B.1: skew is derived, §6.3) — never trusted
    /// from a caller.
    async fn persist_instruction(
        &self,
        job_id: Uuid,
        draft: &InstructionDraft,
    ) -> Result<InstructionRecord, StoreError>;

    /// FLAG the Instruction (Teacher's VALIDATE_OUT passed): sets it
    /// flagged and immutable. Idempotent.
    async fn flag_instruction(
        &self,
        job_id: Uuid,
        instruction_id: Uuid,
    ) -> Result<InstructionRecord, StoreError>;

    async fn get_instruction(&self, instruction_id: Uuid) -> Result<InstructionRecord, StoreError>;

    /// Records a Regular Teacher output's bias disclosure (§6.3): the
    /// draws and the computed skew. Returns the trailing-window skew share
    /// after this output, for the escalation decision.
    async fn record_regular_output(
        &self,
        instruction_ref: Uuid,
        sources: &[SourceDraw],
        skew: bool,
        window: i64,
    ) -> Result<f64, StoreError>;

    async fn bias_warning_state(&self, scope: &str) -> Result<Option<String>, StoreError>;

    /// Raises the standing warning for a scope if none stands (idempotent);
    /// logs BIAS_WARNING.
    async fn raise_bias_warning(&self, scope: &str) -> Result<(), StoreError>;

    /// The terminal answer (§6.3): `acknowledge` keeps it standing and
    /// counting; `silence` suppresses it (logged severity: suppressed),
    /// not re-raised until lifted.
    async fn resolve_bias_warning(
        &self,
        actor: &str,
        scope: &str,
        acknowledge: bool,
    ) -> Result<(), StoreError>;
}
