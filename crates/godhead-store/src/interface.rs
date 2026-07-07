use crate::error::StoreError;
use crate::types::{ArtifactDraft, ArtifactRecord, ComplianceMetrics};
use godhead_schemas::{
    ConfigConstant, ConfigTier, FlagDraft, FlagStatus, JobDraft, JobRecord, JobStatus, LeaseRecord,
    LogEvent, LogSnapshot, NodeDraft, NodeRecord, NormalizeOutcome, ReadinessFlag, RefusalDraft,
    RefusalRecord, Severity,
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
}
