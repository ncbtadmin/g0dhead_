use crate::error::StoreError;
use crate::interface::Store;
use crate::secrets;
use crate::types::{ArtifactDraft, ArtifactRecord, ComplianceMetrics};
use godhead_schemas::{
    AgentType, AuditorName, Budgets, ConfigConstant, ConfigTier, Envelope, FlagDraft, FlagStatus,
    IntakeStatus, JobDraft, JobRecord, JobStatus, Law, LeaseRecord, LogEvent, LogSnapshot,
    NodeDraft, NodeRecord, NormalizeOutcome, ReadinessFlag, RefusalDraft, RefusalReason,
    RefusalRecord, SchemaRegistry, Severity, Tier, RECORD_SCHEMA_VERSION,
};
use semver::Version;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Row;
use uuid::Uuid;

/// Payload keys the store issues itself (Law XII / A.1). An agent-supplied
/// value for any of these is rejected — SC-H03's mechanical form.
const STORE_ISSUED_KEYS: &[&str] = &["produced_at", "produced_by"];

/// The Postgres substrate behind the abstracted store interface.
pub struct PgStore {
    pool: PgPool,
    registry: SchemaRegistry,
}

impl PgStore {
    /// Connects, runs migrations, and binds the build's schema registry
    /// (Law II.4 — the build declares what it supports).
    pub async fn connect(database_url: &str, registry: SchemaRegistry) -> Result<Self, StoreError> {
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(database_url)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool, registry })
    }

    /// Raw pool access for tests that must simulate out-of-band corruption
    /// or attempt writes the store's own API refuses to offer.
    pub fn raw_pool(&self) -> &PgPool {
        &self.pool
    }

    // ---- row decoding ----

    fn envelope_from_row(row: &PgRow) -> Result<Envelope, StoreError> {
        let version: String = row.try_get("schema_version")?;
        Ok(Envelope {
            schema_name: row.try_get("schema_name")?,
            schema_version: Version::parse(&version)
                .map_err(|e| StoreError::ValidationFailed(format!("stored schema_version: {e}")))?,
            produced_by: row.try_get("produced_by")?,
            produced_at: row.try_get("produced_at")?,
        })
    }

    fn job_from_row(row: &PgRow) -> Result<JobRecord, StoreError> {
        let agent_type: String = row.try_get("agent_type")?;
        let auditor_name: Option<String> = row.try_get("auditor_name")?;
        let tier: Option<String> = row.try_get("tier")?;
        let status: String = row.try_get("status")?;
        let manual_version: String = row.try_get("manual_version")?;
        let input_refs: serde_json::Value = row.try_get("input_refs")?;
        let input_refs: Vec<String> = serde_json::from_value(input_refs)
            .map_err(|e| StoreError::ValidationFailed(format!("stored input_refs: {e}")))?;
        let input_refs = input_refs
            .iter()
            .map(|s| Uuid::parse_str(s))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| StoreError::ValidationFailed(format!("stored input_refs: {e}")))?;
        Ok(JobRecord {
            job_id: row.try_get("job_id")?,
            agent_type: AgentType::parse(&agent_type).map_err(StoreError::from_schema)?,
            auditor_name: auditor_name
                .as_deref()
                .map(AuditorName::parse)
                .transpose()
                .map_err(StoreError::from_schema)?,
            tier: tier
                .as_deref()
                .map(Tier::parse)
                .transpose()
                .map_err(StoreError::from_schema)?,
            status: JobStatus::parse(&status).map_err(StoreError::from_schema)?,
            attempt: row.try_get("attempt")?,
            input_refs,
            env_ref: row.try_get("env_ref")?,
            brief_ref: row.try_get("brief_ref")?,
            endpoint_alias: row.try_get("endpoint_alias")?,
            manual_version: Version::parse(&manual_version)
                .map_err(|e| StoreError::ValidationFailed(format!("stored manual_version: {e}")))?,
            budgets: Budgets {
                max_wall_ms: row.try_get("max_wall_ms")?,
                max_tool_calls: row.try_get("max_tool_calls")?,
                max_tokens: row.try_get("max_tokens")?,
            },
            started_at: row.try_get("started_at")?,
            heartbeat_at: row.try_get("heartbeat_at")?,
            finished_at: row.try_get("finished_at")?,
            revision: row.try_get("revision")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn artifact_from_row(row: &PgRow) -> Result<ArtifactRecord, StoreError> {
        Ok(ArtifactRecord {
            job_id: row.try_get("job_id")?,
            output_slot: row.try_get("output_slot")?,
            payload: row.try_get("payload")?,
            authoritative: row.try_get("authoritative")?,
            quarantine_marked: row.try_get("quarantine_marked")?,
            revision: row.try_get("revision")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn flag_from_row(row: &PgRow) -> Result<ReadinessFlag, StoreError> {
        let certifies: serde_json::Value = row.try_get("certifies")?;
        let validator: serde_json::Value = row.try_get("validator")?;
        let status: String = row.try_get("status")?;
        Ok(ReadinessFlag {
            flag_id: row.try_get("flag_id")?,
            job_id: row.try_get("job_id")?,
            stage: row.try_get("stage")?,
            certifies: serde_json::from_value(certifies)
                .map_err(|e| StoreError::ValidationFailed(format!("stored certifies: {e}")))?,
            validator: serde_json::from_value(validator)
                .map_err(|e| StoreError::ValidationFailed(format!("stored validator: {e}")))?,
            status: FlagStatus::parse(&status).map_err(StoreError::from_schema)?,
            revision: row.try_get("revision")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn refusal_from_row(row: &PgRow) -> Result<RefusalRecord, StoreError> {
        let law: String = row.try_get("law")?;
        let reason: String = row.try_get("reason")?;
        let subject_refs: serde_json::Value = row.try_get("subject_refs")?;
        let preserved_refs: serde_json::Value = row.try_get("preserved_refs")?;
        Ok(RefusalRecord {
            refusal_id: row.try_get("refusal_id")?,
            job_id: row.try_get("job_id")?,
            law: Law::parse(&law).map_err(StoreError::from_schema)?,
            reason: RefusalReason::parse(&reason).map_err(StoreError::from_schema)?,
            subject_refs: serde_json::from_value(subject_refs)
                .map_err(|e| StoreError::ValidationFailed(format!("stored subject_refs: {e}")))?,
            detail: row.try_get("detail")?,
            preserved_refs: serde_json::from_value(preserved_refs)
                .map_err(|e| StoreError::ValidationFailed(format!("stored preserved_refs: {e}")))?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn log_from_row(row: &PgRow) -> Result<LogSnapshot, StoreError> {
        let event: String = row.try_get("event")?;
        let severity: String = row.try_get("severity")?;
        Ok(LogSnapshot {
            log_id: row.try_get("log_id")?,
            seq: row.try_get("seq")?,
            subject_ref: row.try_get("subject_ref")?,
            event: LogEvent::parse(&event).map_err(StoreError::from_schema)?,
            payload: row.try_get("payload")?,
            prior_ref: row.try_get("prior_ref")?,
            severity: Severity::parse(&severity).map_err(StoreError::from_schema)?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn lease_from_row(row: &PgRow) -> Result<LeaseRecord, StoreError> {
        Ok(LeaseRecord {
            lease_id: row.try_get("lease_id")?,
            subject_ref: row.try_get("subject_ref")?,
            job_id: row.try_get("job_id")?,
            ttl_ms: row.try_get("ttl_ms")?,
            active: row.try_get("active")?,
            acquired_at: row.try_get("acquired_at")?,
            heartbeat_at: row.try_get("heartbeat_at")?,
            expires_at: row.try_get("expires_at")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn node_from_row(row: &PgRow) -> Result<NodeRecord, StoreError> {
        let intake_status: String = row.try_get("intake_status")?;
        Ok(NodeRecord {
            node_id: row.try_get("node_id")?,
            filename: row.try_get("filename")?,
            filetype: row.try_get("filetype")?,
            size_bytes: row.try_get("size_bytes")?,
            raw_path: row.try_get("raw_path")?,
            raw_sha256: row.try_get("raw_sha256")?,
            derivative_path: row.try_get("derivative_path")?,
            derivative_sha256: row.try_get("derivative_sha256")?,
            normalized: row.try_get("normalized")?,
            intake_status: IntakeStatus::parse(&intake_status).map_err(StoreError::from_schema)?,
            classification: row.try_get("classification")?,
            notice: row.try_get("notice")?,
            revision: row.try_get("revision")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn config_from_row(row: &PgRow) -> Result<ConfigConstant, StoreError> {
        let tier: String = row.try_get("tier")?;
        Ok(ConfigConstant {
            key: row.try_get("key")?,
            tier: ConfigTier::parse(&tier).map_err(StoreError::from_schema)?,
            value: row.try_get("value")?,
            revision: row.try_get("revision")?,
            changed_at: row.try_get("changed_at")?,
            changed_by: row.try_get("changed_by")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    // ---- internal machinery ----

    /// Fetches a job plus its Law XIV wall-budget verdict, computed against
    /// the store's clock (Law XII), never the caller's.
    async fn fetch_job(&self, job_id: Uuid) -> Result<(JobRecord, bool), StoreError> {
        let row = sqlx::query(
            r#"SELECT *,
                 (status = 'RUNNING' AND started_at IS NOT NULL
                  AND now() > started_at + (max_wall_ms::double precision * interval '1 millisecond'))
                 AS budget_exhausted
               FROM job_records WHERE job_id = $1"#,
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            // Law XIII.1: an unknown job identity is an anonymous write.
            StoreError::NotFound(format!("no such job {job_id}: anonymous writes are rejected"))
        })?;
        let exhausted: bool = row.try_get("budget_exhausted")?;
        Ok((Self::job_from_row(&row)?, exhausted))
    }

    /// Law I.4 + Law XIV, applied before any identity-bearing operation.
    /// `permit_terminate_after_flag` is true only for the FLAGGED→TERMINATED
    /// transition, the sole act remaining to a flagged job.
    async fn guard_actor(
        &self,
        job_id: Uuid,
        operation: &str,
        permit_terminate_after_flag: bool,
    ) -> Result<JobRecord, StoreError> {
        let (job, budget_exhausted) = self.fetch_job(job_id).await?;
        let blocked = match job.status {
            JobStatus::Flagged => !permit_terminate_after_flag,
            JobStatus::Terminated | JobStatus::Refused => true,
            _ => false,
        };
        if blocked {
            let detail = format!(
                "job {job_id} attempted '{operation}' after {} — access ends at FLAG or REFUSED (Law I.4)",
                job.status
            );
            self.append_log(
                &job_id.to_string(),
                LogEvent::Violation,
                &serde_json::json!({ "operation": operation, "status": job.status.as_str() }),
                Severity::Violation,
                "store",
            )
            .await?;
            return Err(StoreError::TerminalAccess(detail));
        }
        if budget_exhausted {
            self.enact_refusal(
                &job,
                &RefusalDraft {
                    law: Law::XIV,
                    reason: RefusalReason::BudgetExceeded,
                    subject_refs: vec![job_id.to_string()],
                    detail: format!(
                        "max_wall_ms {} exceeded mid-labor; partials preserved non-authoritative, leases released",
                        job.budgets.max_wall_ms
                    ),
                    preserved_refs: vec![],
                },
                "store",
            )
            .await?;
            return Err(StoreError::BudgetExceeded(format!(
                "job {job_id}: wall budget exhausted; the labor is refused (Law XIV.2)"
            )));
        }
        Ok(job)
    }

    /// The append-only write (Law V.1): chains prior_ref to the subject's
    /// latest snapshot; nothing overwrites.
    async fn append_log(
        &self,
        subject_ref: &str,
        event: LogEvent,
        payload: &serde_json::Value,
        severity: Severity,
        produced_by: &str,
    ) -> Result<LogSnapshot, StoreError> {
        let serialized = payload.to_string();
        if let Some(pattern) = secrets::scan(&serialized) {
            // Cannot log the violation via a payload that itself carries the
            // secret; record the pattern name only (Law XV.1).
            return Err(StoreError::SecretDetected(format!(
                "log payload matched secret pattern '{pattern}' (Law XV.2)"
            )));
        }
        let row = sqlx::query(
            r#"INSERT INTO log_snapshots
                 (log_id, subject_ref, event, payload, prior_ref, severity,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4,
                 (SELECT log_id FROM log_snapshots WHERE subject_ref = $2 ORDER BY seq DESC LIMIT 1),
                 $5, 'LogSnapshot', $6, $7)
               RETURNING *"#,
        )
        .bind(Uuid::now_v7())
        .bind(subject_ref)
        .bind(event.as_str())
        .bind(payload)
        .bind(severity.as_str())
        .bind(RECORD_SCHEMA_VERSION)
        .bind(produced_by)
        .fetch_one(&self.pool)
        .await?;
        Self::log_from_row(&row)
    }

    /// Law VII, mechanically: refusal record + quarantine-marked partials +
    /// released leases + REFUSED status, in one transaction. Nothing else
    /// is touched (VII.3).
    async fn enact_refusal(
        &self,
        job: &JobRecord,
        draft: &RefusalDraft,
        produced_by: &str,
    ) -> Result<RefusalRecord, StoreError> {
        let serialized = format!("{} {}", draft.detail, draft.subject_refs.join(" "));
        if let Some(pattern) = secrets::scan(&serialized) {
            return Err(StoreError::SecretDetected(format!(
                "refusal detail matched secret pattern '{pattern}' (Law XV.2)"
            )));
        }
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"INSERT INTO refusal_records
                 (refusal_id, job_id, law, reason, subject_refs, detail, preserved_refs,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, 'RefusalRecord', $8, $9)
               RETURNING *"#,
        )
        .bind(Uuid::now_v7())
        .bind(job.job_id)
        .bind(draft.law.as_str())
        .bind(draft.reason.as_str())
        .bind(serde_json::to_value(&draft.subject_refs).expect("string vec serializes"))
        .bind(&draft.detail)
        .bind(serde_json::to_value(&draft.preserved_refs).expect("string vec serializes"))
        .bind(RECORD_SCHEMA_VERSION)
        .bind(produced_by)
        .fetch_one(&mut *tx)
        .await?;
        // VII.5: partials become non-authoritative and invisible to readers.
        sqlx::query(
            "UPDATE artifacts SET authoritative = false, quarantine_marked = true WHERE job_id = $1",
        )
        .bind(job.job_id)
        .execute(&mut *tx)
        .await?;
        // Law I.4 / XIV.2: the refusing agent's leases are released.
        sqlx::query(
            "UPDATE lease_records SET active = false, expires_at = now() WHERE job_id = $1 AND active",
        )
        .bind(job.job_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"UPDATE job_records
               SET status = 'REFUSED', finished_at = now(), revision = revision + 1
               WHERE job_id = $1 AND status IN ('PENDING','LEASED','RUNNING','WRITTEN')"#,
        )
        .bind(job.job_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        self.append_log(
            &job.job_id.to_string(),
            LogEvent::Refusal,
            &serde_json::json!({ "law": draft.law.as_str(), "reason": draft.reason.as_str() }),
            Severity::Info,
            produced_by,
        )
        .await?;
        Self::refusal_from_row(&row)
    }
}

impl Store for PgStore {
    async fn create_job(&self, draft: &JobDraft) -> Result<JobRecord, StoreError> {
        // Law XIV.1: the dispatcher MUST NOT spawn without budgets.
        draft.budgets.validate().map_err(StoreError::from_schema)?;
        if let Some(alias) = &draft.endpoint_alias {
            if let Some(pattern) = secrets::scan(alias) {
                return Err(StoreError::SecretDetected(format!(
                    "endpoint_alias matched secret pattern '{pattern}' — endpoints are referenced by alias only (Law XV.1)"
                )));
            }
        }
        let input_refs: Vec<String> = draft.input_refs.iter().map(Uuid::to_string).collect();
        let row = sqlx::query(
            r#"INSERT INTO job_records
                 (job_id, agent_type, auditor_name, tier, input_refs, env_ref, brief_ref,
                  endpoint_alias, manual_version, max_wall_ms, max_tool_calls, max_tokens,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                       'JobRecord', $13, 'dispatcher')
               RETURNING *, false AS budget_exhausted"#,
        )
        .bind(Uuid::now_v7())
        .bind(draft.agent_type.as_str())
        .bind(draft.auditor_name.map(AuditorName::as_str))
        .bind(draft.tier.map(Tier::as_str))
        .bind(serde_json::to_value(&input_refs).expect("string vec serializes"))
        .bind(draft.env_ref)
        .bind(draft.brief_ref)
        .bind(&draft.endpoint_alias)
        .bind(draft.manual_version.to_string())
        .bind(draft.budgets.max_wall_ms)
        .bind(draft.budgets.max_tool_calls)
        .bind(draft.budgets.max_tokens)
        .bind(RECORD_SCHEMA_VERSION)
        .fetch_one(&self.pool)
        .await?;
        Self::job_from_row(&row)
    }

    async fn get_job(&self, job_id: Uuid) -> Result<JobRecord, StoreError> {
        Ok(self.fetch_job(job_id).await?.0)
    }

    async fn transition_job(
        &self,
        job_id: Uuid,
        expected_revision: i32,
        to: JobStatus,
    ) -> Result<JobRecord, StoreError> {
        if matches!(to, JobStatus::Flagged | JobStatus::Refused) {
            return Err(StoreError::ValidationFailed(format!(
                "status {to} is entered only through its own protocol (write_flag / refuse), never by direct transition"
            )));
        }
        let job = self
            .guard_actor(
                job_id,
                &format!("transition:{to}"),
                to == JobStatus::Terminated,
            )
            .await?;
        // Law I.1: forward-only; any other transition is rejected.
        if !job.status.may_transition_to(to) {
            return Err(StoreError::ValidationFailed(format!(
                "transition {} -> {to} is not forward along the lifecycle (Law I.1)",
                job.status
            )));
        }
        // Law XI.3: compare-and-swap; a stale revision loses and re-reads.
        let result = sqlx::query(
            r#"UPDATE job_records
               SET status = $3,
                   revision = revision + 1,
                   started_at = CASE WHEN $3 = 'RUNNING' THEN now() ELSE started_at END,
                   heartbeat_at = now(),
                   finished_at = CASE WHEN $3 = 'TERMINATED' THEN now() ELSE finished_at END
               WHERE job_id = $1 AND revision = $2 AND status = $4"#,
        )
        .bind(job_id)
        .bind(expected_revision)
        .bind(to.as_str())
        .bind(job.status.as_str())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(StoreError::StaleRevision {
                expected: expected_revision,
                subject: job_id.to_string(),
            });
        }
        // Law I.1: each transition writes a log snapshot.
        self.append_log(
            &job_id.to_string(),
            LogEvent::JobTransition,
            &serde_json::json!({ "from": job.status.as_str(), "to": to.as_str() }),
            Severity::Info,
            "store",
        )
        .await?;
        Ok(self.fetch_job(job_id).await?.0)
    }

    async fn write_artifact(
        &self,
        job_id: Uuid,
        output_slot: &str,
        draft: &ArtifactDraft,
    ) -> Result<ArtifactRecord, StoreError> {
        let job = self.guard_actor(job_id, "write_artifact", false).await?;
        // Outputs are written during WORK; the lifecycle admits no other window.
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "artifacts are written in RUNNING, not {} (Law I.1)",
                job.status
            )));
        }
        if output_slot.is_empty() {
            return Err(StoreError::ValidationFailed(
                "output_slot must be non-empty (Law I.3 keys writes by it)".into(),
            ));
        }
        // Law XII: store-issued fields cannot arrive from an agent.
        if let Some(obj) = draft.payload.as_object() {
            for key in STORE_ISSUED_KEYS {
                if obj.contains_key(*key) {
                    return Err(StoreError::ValidationFailed(format!(
                        "payload carries store-issued field '{key}' — timestamps and authorship are issued by the store (Laws XII, XIII)"
                    )));
                }
            }
        }
        // Law II: validate BEFORE any write; nothing partial persists.
        self.registry
            .check(&draft.schema_name, &draft.schema_version, &draft.payload)
            .map_err(StoreError::from_schema)?;
        // Law XV.2: outbound writes are scanned.
        let serialized = draft.payload.to_string();
        if let Some(pattern) = secrets::scan(&serialized) {
            self.append_log(
                &job_id.to_string(),
                LogEvent::Violation,
                &serde_json::json!({ "operation": "write_artifact", "secret_pattern": pattern }),
                Severity::Violation,
                "store",
            )
            .await?;
            return Err(StoreError::SecretDetected(format!(
                "artifact payload matched secret pattern '{pattern}' (Law XV.2)"
            )));
        }
        // Law I.3: keyed upsert — a retry converges on its own keys.
        let row = sqlx::query(
            r#"INSERT INTO artifacts
                 (job_id, output_slot, payload, schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, $1::text)
               ON CONFLICT (job_id, output_slot) DO UPDATE
               SET payload = EXCLUDED.payload,
                   schema_name = EXCLUDED.schema_name,
                   schema_version = EXCLUDED.schema_version,
                   revision = artifacts.revision + 1,
                   produced_at = now()
               RETURNING *"#,
        )
        .bind(job_id)
        .bind(output_slot)
        .bind(&draft.payload)
        .bind(&draft.schema_name)
        .bind(draft.schema_version.to_string())
        .fetch_one(&self.pool)
        .await?;
        Self::artifact_from_row(&row)
    }

    async fn read_artifact(
        &self,
        job_id: Uuid,
        output_slot: &str,
    ) -> Result<ArtifactRecord, StoreError> {
        let row = sqlx::query(
            r#"SELECT * FROM artifacts
               WHERE job_id = $1 AND output_slot = $2
                 AND authoritative AND NOT quarantine_marked"#,
        )
        .bind(job_id)
        .bind(output_slot)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            StoreError::NotFound(format!(
                "no authoritative artifact at ({job_id}, {output_slot}) — partials of refused jobs are invisible (Law VII.5)"
            ))
        })?;
        Self::artifact_from_row(&row)
    }

    async fn write_flag(
        &self,
        job_id: Uuid,
        draft: &FlagDraft,
    ) -> Result<ReadinessFlag, StoreError> {
        let job = self.guard_actor(job_id, "write_flag", false).await?;
        // I.1: FLAG follows VALIDATE_OUT follows WRITE — the job stands WRITTEN.
        if job.status != JobStatus::Written {
            return Err(StoreError::ValidationFailed(format!(
                "a flag certifies written state; job is {} not WRITTEN (Laws I.1, III.2)",
                job.status
            )));
        }
        if draft.stage.is_empty() {
            return Err(StoreError::ValidationFailed(
                "flag stage must be non-empty".into(),
            ));
        }
        if draft.certifies.output_slots.len() != draft.certifies.revisions.len() {
            return Err(StoreError::ValidationFailed(
                "certifies.output_slots and certifies.revisions must correspond one-to-one (III.2)"
                    .into(),
            ));
        }
        // III.2: a flag is written only after its certified outputs exist and
        // validate — flag-before-output is rejected (SC-B01).
        for (slot, expected_rev) in draft
            .certifies
            .output_slots
            .iter()
            .zip(draft.certifies.revisions.iter())
        {
            let artifact = self.read_artifact(job_id, slot).await.map_err(|_| {
                StoreError::ValidationFailed(format!(
                    "flag certifies output '{slot}' which does not exist authoritatively — a flag is written only after its outputs (Law III.2)"
                ))
            })?;
            if artifact.revision != *expected_rev {
                return Err(StoreError::ValidationFailed(format!(
                    "flag certifies '{slot}' at revision {expected_rev} but the store holds revision {} (Law III.2)",
                    artifact.revision
                )));
            }
            self.registry
                .check(
                    &artifact.envelope.schema_name,
                    &artifact.envelope.schema_version,
                    &artifact.payload,
                )
                .map_err(StoreError::from_schema)?;
        }
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"INSERT INTO readiness_flags
                 (flag_id, job_id, stage, certifies, validator,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, 'ReadinessFlag', $6, $2::text)
               ON CONFLICT (job_id, stage) DO UPDATE
               SET certifies = EXCLUDED.certifies,
                   validator = EXCLUDED.validator,
                   revision = readiness_flags.revision + 1
               RETURNING *"#,
        )
        .bind(Uuid::now_v7())
        .bind(job_id)
        .bind(&draft.stage)
        .bind(serde_json::to_value(&draft.certifies).expect("certifies serializes"))
        .bind(serde_json::to_value(&draft.validator).expect("validator serializes"))
        .bind(RECORD_SCHEMA_VERSION)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query(
            r#"UPDATE job_records SET status = 'FLAGGED', revision = revision + 1
               WHERE job_id = $1 AND status = 'WRITTEN'"#,
        )
        .bind(job_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        self.append_log(
            &job_id.to_string(),
            LogEvent::JobTransition,
            &serde_json::json!({ "from": "WRITTEN", "to": "FLAGGED", "stage": draft.stage }),
            Severity::Info,
            "store",
        )
        .await?;
        Self::flag_from_row(&row)
    }

    async fn get_flag(&self, flag_id: Uuid) -> Result<ReadinessFlag, StoreError> {
        let row = sqlx::query("SELECT * FROM readiness_flags WHERE flag_id = $1")
            .bind(flag_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StoreError::NotFound(format!("no such flag {flag_id}")))?;
        Self::flag_from_row(&row)
    }

    async fn supersede_flag(
        &self,
        flag_id: Uuid,
        expected_revision: i32,
        to: FlagStatus,
    ) -> Result<ReadinessFlag, StoreError> {
        // III.4: ACTIVE is the only state that transitions; nothing revives.
        if to == FlagStatus::Active {
            return Err(StoreError::ValidationFailed(
                "a flag cannot be returned to ACTIVE; supersession is one-way (Law III.4)".into(),
            ));
        }
        let result = sqlx::query(
            r#"UPDATE readiness_flags
               SET status = $3, revision = revision + 1
               WHERE flag_id = $1 AND revision = $2 AND status = 'ACTIVE'"#,
        )
        .bind(flag_id)
        .bind(expected_revision)
        .bind(to.as_str())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            // Distinguish stale CAS from a flag already superseded/missing.
            let current = self.get_flag(flag_id).await?;
            if current.status != FlagStatus::Active {
                return Err(StoreError::ValidationFailed(format!(
                    "flag {flag_id} is {} — supersession is one-way (Law III.4)",
                    current.status
                )));
            }
            return Err(StoreError::StaleRevision {
                expected: expected_revision,
                subject: flag_id.to_string(),
            });
        }
        self.get_flag(flag_id).await
    }

    async fn read_certified(
        &self,
        reader_job_id: Uuid,
        flag_id: Uuid,
    ) -> Result<Vec<ArtifactRecord>, StoreError> {
        self.guard_actor(reader_job_id, "read_certified", false)
            .await?;
        let flag = self.get_flag(flag_id).await?;
        if flag.status != FlagStatus::Active {
            return Err(StoreError::ValidationFailed(format!(
                "flag {flag_id} is {} — only ACTIVE flags are actionable (Law III.4)",
                flag.status
            )));
        }
        // III.3: the flag is testimony; the state is the witness.
        let mut witnessed = Vec::with_capacity(flag.certifies.output_slots.len());
        let mut defect: Option<String> = None;
        for (slot, expected_rev) in flag
            .certifies
            .output_slots
            .iter()
            .zip(flag.certifies.revisions.iter())
        {
            match self.read_artifact(flag.job_id, slot).await {
                Ok(artifact) => {
                    if artifact.revision != *expected_rev {
                        defect = Some(format!(
                            "'{slot}' certified at revision {expected_rev}, store holds {}",
                            artifact.revision
                        ));
                        break;
                    }
                    if let Err(e) = self.registry.check(
                        &artifact.envelope.schema_name,
                        &artifact.envelope.schema_version,
                        &artifact.payload,
                    ) {
                        defect = Some(format!("'{slot}' no longer validates: {e}"));
                        break;
                    }
                    witnessed.push(artifact);
                }
                Err(_) => {
                    defect = Some(format!(
                        "certified output '{slot}' is gone or non-authoritative"
                    ));
                    break;
                }
            }
        }
        if let Some(detail) = defect {
            // The reader sets the flag DISTRUSTED and refuses (III.3).
            sqlx::query(
                r#"UPDATE readiness_flags SET status = 'DISTRUSTED', revision = revision + 1
                   WHERE flag_id = $1 AND status = 'ACTIVE'"#,
            )
            .bind(flag_id)
            .execute(&self.pool)
            .await?;
            return Err(StoreError::FlagUntrusted(format!(
                "flag {flag_id} distrusted by reader {reader_job_id}: {detail} (Law III.3)"
            )));
        }
        Ok(witnessed)
    }

    async fn refuse(
        &self,
        job_id: Uuid,
        draft: &RefusalDraft,
    ) -> Result<RefusalRecord, StoreError> {
        let job = self.guard_actor(job_id, "refuse", false).await?;
        // Law I.1: REFUSED is reachable from any live state (guard passed ⇒ live).
        self.enact_refusal(&job, draft, &job_id.to_string()).await
    }

    async fn acquire_lease(
        &self,
        job_id: Uuid,
        subject_ref: Uuid,
        ttl_ms: i64,
    ) -> Result<LeaseRecord, StoreError> {
        self.guard_actor(job_id, "acquire_lease", false).await?;
        if ttl_ms <= 0 {
            return Err(StoreError::ValidationFailed(
                "lease ttl_ms must be positive".into(),
            ));
        }
        // XI.2: an expired lease does not block; retire it first.
        sqlx::query(
            r#"UPDATE lease_records SET active = false
               WHERE subject_ref = $1 AND active AND expires_at <= now()"#,
        )
        .bind(subject_ref)
        .execute(&self.pool)
        .await?;
        // XI.1: acquire-or-refuse. The partial unique index arbitrates races;
        // a losing insert affects zero rows and refuses immediately.
        let row = sqlx::query(
            r#"INSERT INTO lease_records
                 (lease_id, subject_ref, job_id, ttl_ms, expires_at,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4,
                       now() + ($4::double precision * interval '1 millisecond'),
                       'LeaseRecord', $5, $3::text)
               ON CONFLICT (subject_ref) WHERE active DO NOTHING
               RETURNING *"#,
        )
        .bind(Uuid::now_v7())
        .bind(subject_ref)
        .bind(job_id)
        .bind(ttl_ms)
        .bind(RECORD_SCHEMA_VERSION)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(row) => Self::lease_from_row(&row),
            None => Err(StoreError::LeaseConflict(format!(
                "subject {subject_ref} is leased; no waiting, no spinning — the dispatcher reschedules (Law XI.1)"
            ))),
        }
    }

    async fn heartbeat_lease(
        &self,
        job_id: Uuid,
        lease_id: Uuid,
    ) -> Result<LeaseRecord, StoreError> {
        self.guard_actor(job_id, "heartbeat_lease", false).await?;
        let row = sqlx::query(
            r#"UPDATE lease_records
               SET heartbeat_at = now(),
                   expires_at = now() + (ttl_ms::double precision * interval '1 millisecond')
               WHERE lease_id = $1 AND job_id = $2 AND active AND expires_at > now()
               RETURNING *"#,
        )
        .bind(lease_id)
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            StoreError::LeaseConflict(format!(
                "lease {lease_id} is not live under job {job_id} — expired leases route to recovery (Law XI.2)"
            ))
        })?;
        Self::lease_from_row(&row)
    }

    async fn release_lease(&self, job_id: Uuid, lease_id: Uuid) -> Result<(), StoreError> {
        let result = sqlx::query(
            r#"UPDATE lease_records SET active = false, expires_at = now()
               WHERE lease_id = $1 AND job_id = $2 AND active"#,
        )
        .bind(lease_id)
        .bind(job_id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(StoreError::NotFound(format!(
                "no active lease {lease_id} held by job {job_id}"
            )));
        }
        Ok(())
    }

    async fn write_log(
        &self,
        subject_ref: &str,
        event: LogEvent,
        payload: &serde_json::Value,
        severity: Severity,
    ) -> Result<LogSnapshot, StoreError> {
        self.append_log(subject_ref, event, payload, severity, "store")
            .await
    }

    async fn read_logs(&self, subject_ref: &str) -> Result<Vec<LogSnapshot>, StoreError> {
        let rows = sqlx::query("SELECT * FROM log_snapshots WHERE subject_ref = $1 ORDER BY seq")
            .bind(subject_ref)
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(Self::log_from_row).collect()
    }

    async fn set_config(
        &self,
        changed_by: &str,
        key: &str,
        tier: ConfigTier,
        value: &serde_json::Value,
        expected_revision: Option<i32>,
    ) -> Result<ConfigConstant, StoreError> {
        let serialized = value.to_string();
        if let Some(pattern) = secrets::scan(&serialized) {
            return Err(StoreError::SecretDetected(format!(
                "config value matched secret pattern '{pattern}' — secrets live only in the config secret store (Law XV.1)"
            )));
        }
        match expected_revision {
            None => {
                let row = sqlx::query(
                    r#"INSERT INTO config_constants
                         (key, tier, value, changed_by, schema_name, schema_version, produced_by)
                       VALUES ($1, $2, $3, $4, 'ConfigConstant', $5, $4)
                       ON CONFLICT (key) DO NOTHING
                       RETURNING *"#,
                )
                .bind(key)
                .bind(tier.as_str())
                .bind(value)
                .bind(changed_by)
                .bind(RECORD_SCHEMA_VERSION)
                .fetch_optional(&self.pool)
                .await?;
                match row {
                    Some(row) => Self::config_from_row(&row),
                    None => Err(StoreError::ValidationFailed(format!(
                        "config key '{key}' already exists; updates require expected_revision (Law XI.3)"
                    ))),
                }
            }
            Some(expected) => {
                let row = sqlx::query(
                    r#"UPDATE config_constants
                       SET value = $3, tier = $4, revision = revision + 1,
                           changed_at = now(), changed_by = $5
                       WHERE key = $1 AND revision = $2
                       RETURNING *"#,
                )
                .bind(key)
                .bind(expected)
                .bind(value)
                .bind(tier.as_str())
                .bind(changed_by)
                .fetch_optional(&self.pool)
                .await?;
                match row {
                    Some(row) => Self::config_from_row(&row),
                    None => {
                        self.get_config(key).await?; // NotFound if the key is absent
                        Err(StoreError::StaleRevision {
                            expected,
                            subject: format!("config:{key}"),
                        })
                    }
                }
            }
        }
    }

    async fn get_config(&self, key: &str) -> Result<ConfigConstant, StoreError> {
        let row = sqlx::query("SELECT * FROM config_constants WHERE key = $1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StoreError::NotFound(format!("no config constant '{key}'")))?;
        Self::config_from_row(&row)
    }

    async fn compliance_metrics(&self, job_ids: &[Uuid]) -> Result<ComplianceMetrics, StoreError> {
        let rows = sqlx::query(
            "SELECT status, count(*) AS n FROM job_records WHERE job_id = ANY($1) GROUP BY status",
        )
        .bind(job_ids)
        .fetch_all(&self.pool)
        .await?;
        let mut metrics = ComplianceMetrics {
            total: 0,
            compliant: 0,
            refused: 0,
            in_flight: 0,
        };
        for row in rows {
            let status: String = row.try_get("status")?;
            let n: i64 = row.try_get("n")?;
            let n = usize::try_from(n)
                .map_err(|e| StoreError::ValidationFailed(format!("count overflow: {e}")))?;
            metrics.total += n;
            match JobStatus::parse(&status).map_err(StoreError::from_schema)? {
                // VII.4: refusal is compliance; no metric scores it as error.
                JobStatus::Refused => {
                    metrics.compliant += n;
                    metrics.refused += n;
                }
                JobStatus::Terminated | JobStatus::Flagged => metrics.compliant += n,
                _ => metrics.in_flight += n,
            }
        }
        Ok(metrics)
    }

    async fn create_node(
        &self,
        job_id: Uuid,
        node_id: Uuid,
        draft: &NodeDraft,
    ) -> Result<NodeRecord, StoreError> {
        let job = self.guard_actor(job_id, "create_node", false).await?;
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "nodes are created during WORK, not {} (Law I.1)",
                job.status
            )));
        }
        if draft.raw_sha256.len() != 64 || !draft.raw_sha256.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err(StoreError::ValidationFailed(
                "raw_sha256 must be a 64-hex-char SHA-256 digest".into(),
            ));
        }
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"INSERT INTO nodes
                 (node_id, filename, filetype, size_bytes, raw_path, raw_sha256,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, $6, 'NodeRecord', $7, $8::text)
               RETURNING *"#,
        )
        .bind(node_id)
        .bind(&draft.filename)
        .bind(&draft.filetype)
        .bind(draft.size_bytes)
        .bind(&draft.raw_path)
        .bind(&draft.raw_sha256)
        .bind(RECORD_SCHEMA_VERSION)
        .bind(job_id)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        // First-log-on-copy (doc 2 §2.2): the required fields, in the same act.
        self.append_log(
            &node_id.to_string(),
            LogEvent::IntakeRawCopied,
            &serde_json::json!({
                "filename": draft.filename,
                "filetype": draft.filetype,
                "size_bytes": draft.size_bytes,
                "normalized": false,
            }),
            Severity::Info,
            &job_id.to_string(),
        )
        .await?;
        Self::node_from_row(&row)
    }

    async fn get_node(&self, node_id: Uuid) -> Result<NodeRecord, StoreError> {
        let row = sqlx::query("SELECT * FROM nodes WHERE node_id = $1")
            .bind(node_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StoreError::NotFound(format!("no such node {node_id}")))?;
        Self::node_from_row(&row)
    }

    async fn set_node_derivative(
        &self,
        job_id: Uuid,
        node_id: Uuid,
        expected_revision: i32,
        outcome: &NormalizeOutcome,
    ) -> Result<NodeRecord, StoreError> {
        let job = self
            .guard_actor(job_id, "set_node_derivative", false)
            .await?;
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "node mutations happen during WORK, not {} (Law I.1)",
                job.status
            )));
        }
        let (d_path, d_sha, normalized, status, notice, severity) = match outcome {
            NormalizeOutcome::Normalized {
                derivative_path,
                derivative_sha256,
            } => (
                Some(derivative_path.as_str()),
                Some(derivative_sha256.as_str()),
                true,
                IntakeStatus::Normalized,
                None,
                Severity::Info,
            ),
            NormalizeOutcome::DecodeFailed { reason } => (
                None,
                None,
                false,
                IntakeStatus::DecodeFailed,
                Some(reason.as_str()),
                Severity::Warning,
            ),
            NormalizeOutcome::Unsupported { notice } => (
                None,
                None,
                false,
                IntakeStatus::Unsupported,
                Some(notice.as_str()),
                Severity::Warning,
            ),
        };
        let row = sqlx::query(
            r#"UPDATE nodes
               SET derivative_path = $3, derivative_sha256 = $4, normalized = $5,
                   intake_status = $6, notice = $7, revision = revision + 1
               WHERE node_id = $1 AND revision = $2
               RETURNING *"#,
        )
        .bind(node_id)
        .bind(expected_revision)
        .bind(d_path)
        .bind(d_sha)
        .bind(normalized)
        .bind(status.as_str())
        .bind(notice)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            // Missing node vs. stale CAS — but never a silent overwrite.
            StoreError::StaleRevision {
                expected: expected_revision,
                subject: format!("node:{node_id}"),
            }
        })?;
        // Flag, don't bury (doc 2 §2.3): failures are surfaced states.
        self.append_log(
            &node_id.to_string(),
            LogEvent::Normalized,
            &serde_json::json!({
                "outcome": status.as_str(),
                "derivative_sha256": d_sha,
                "notice": notice,
            }),
            severity,
            &job_id.to_string(),
        )
        .await?;
        Self::node_from_row(&row)
    }

    async fn set_node_classification(
        &self,
        job_id: Uuid,
        node_id: Uuid,
        expected_revision: i32,
        classification: &serde_json::Value,
    ) -> Result<NodeRecord, StoreError> {
        let job = self
            .guard_actor(job_id, "set_node_classification", false)
            .await?;
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "node mutations happen during WORK, not {} (Law I.1)",
                job.status
            )));
        }
        if !classification.is_array() {
            return Err(StoreError::ValidationFailed(
                "classification must be an array of bucket entries (doc 2 §2.5)".into(),
            ));
        }
        let row = sqlx::query(
            r#"UPDATE nodes
               SET classification = $3, revision = revision + 1
               WHERE node_id = $1 AND revision = $2
               RETURNING *"#,
        )
        .bind(node_id)
        .bind(expected_revision)
        .bind(classification)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| StoreError::StaleRevision {
            expected: expected_revision,
            subject: format!("node:{node_id}"),
        })?;
        self.append_log(
            &node_id.to_string(),
            LogEvent::Classified,
            &serde_json::json!({ "classification": classification, "low_trust": true }),
            Severity::Info,
            &job_id.to_string(),
        )
        .await?;
        Self::node_from_row(&row)
    }

    async fn list_active_flags(&self, stage: &str) -> Result<Vec<ReadinessFlag>, StoreError> {
        let rows = sqlx::query(
            "SELECT * FROM readiness_flags WHERE stage = $1 AND status = 'ACTIVE' ORDER BY produced_at",
        )
        .bind(stage)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(Self::flag_from_row).collect()
    }

    async fn list_flags_for_job(&self, job_id: Uuid) -> Result<Vec<ReadinessFlag>, StoreError> {
        let rows =
            sqlx::query("SELECT * FROM readiness_flags WHERE job_id = $1 ORDER BY produced_at")
                .bind(job_id)
                .fetch_all(&self.pool)
                .await?;
        rows.iter().map(Self::flag_from_row).collect()
    }

    async fn list_jobs_by_input_ref(&self, input_ref: Uuid) -> Result<Vec<JobRecord>, StoreError> {
        let rows =
            sqlx::query("SELECT * FROM job_records WHERE input_refs @> $1 ORDER BY produced_at")
                .bind(serde_json::json!([input_ref.to_string()]))
                .fetch_all(&self.pool)
                .await?;
        rows.iter().map(Self::job_from_row).collect()
    }
}
