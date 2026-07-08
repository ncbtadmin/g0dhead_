use crate::error::StoreError;
use crate::types::{ArtifactDraft, ArtifactRecord, ComplianceMetrics};
use godhead_schemas::{
    ConfigConstant, ConfigTier, ConsentDecision, EmbeddingRecord, FlagDraft, FlagStatus, JobDraft,
    JobRecord, JobStatus, LeaseRecord, LinkRecord, LiveWeights, LogEvent, LogSnapshot, NodeDraft,
    NodeRecord, NormalizeOutcome, OverrideRecord, PetitionDraft, PetitionRecord, ReadinessFlag,
    RebalanceState, RefusalDraft, RefusalRecord, Severity,
};
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
}
