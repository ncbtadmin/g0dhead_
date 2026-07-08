use crate::error::StoreError;
use crate::interface::Store;
use crate::secrets;
use crate::types::{ArtifactDraft, ArtifactRecord, ComplianceMetrics};
use godhead_schemas::{
    AgentType, AmendmentKind, AuditReport, AuditReportDraft, AuditorKind, AuditorName, Budgets,
    Claim, ConfigConstant, ConfigTier, ConsentDecision, ConsentRecord, ConsentScope,
    EmbeddingRecord, Envelope, FlagDraft, FlagStatus, IntakeStatus, JobDraft, JobRecord, JobStatus,
    JointProposal, Law, LeaseRecord, LinkRecord, LiveWeights, LogEvent, LogSnapshot, MatrixRecord,
    MatrixStatus, NodeDraft, NodeRecord, NormalizeOutcome, OverrideBasis, OverrideKind,
    OverrideRecord, PetitionDraft, PetitionRecord, PetitionStatus, ProposalDraft, ReadinessFlag,
    RebalanceState, RefusalDraft, RefusalReason, RefusalRecord, ReportKind, SchemaRegistry,
    Severity, Tier, Verdict, RECORD_SCHEMA_VERSION,
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

    fn override_from_row(row: &PgRow) -> Result<OverrideRecord, StoreError> {
        let kind: String = row.try_get("kind")?;
        let basis: String = row.try_get("basis")?;
        Ok(OverrideRecord {
            override_id: row.try_get("override_id")?,
            subject_ref: row.try_get("subject_ref")?,
            kind: OverrideKind::parse(&kind).map_err(StoreError::from_schema)?,
            basis: OverrideBasis::parse(&basis).map_err(StoreError::from_schema)?,
            prior_ref: row.try_get("prior_ref")?,
            consent_ref: row.try_get("consent_ref")?,
            protected_state: row.try_get("protected_state")?,
            user_overridden: row.try_get("user_overridden")?,
            laid_at: row.try_get("laid_at")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn petition_from_row(row: &PgRow) -> Result<PetitionRecord, StoreError> {
        let change_kind: String = row.try_get("change_kind")?;
        let status: String = row.try_get("status")?;
        let evidence_refs: serde_json::Value = row.try_get("evidence_refs")?;
        Ok(PetitionRecord {
            petition_id: row.try_get("petition_id")?,
            subject_ref: row.try_get("subject_ref")?,
            change_kind: OverrideKind::parse(&change_kind).map_err(StoreError::from_schema)?,
            reason: row.try_get("reason")?,
            evidence_refs: serde_json::from_value(evidence_refs)
                .map_err(|e| StoreError::ValidationFailed(format!("stored evidence_refs: {e}")))?,
            proposed_change: row.try_get("proposed_change")?,
            status: PetitionStatus::parse(&status).map_err(StoreError::from_schema)?,
            occurrence_count: row.try_get("occurrence_count")?,
            consent_ref: row.try_get("consent_ref")?,
            execution_job_ref: row.try_get("execution_job_ref")?,
            resolved_at: row.try_get("resolved_at")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn consent_from_row(row: &PgRow) -> Result<ConsentRecord, StoreError> {
        let decision: String = row.try_get("decision")?;
        let scope: String = row.try_get("scope")?;
        Ok(ConsentRecord {
            consent_id: row.try_get("consent_id")?,
            subject_ref: row.try_get("subject_ref")?,
            decision: ConsentDecision::parse(&decision).map_err(StoreError::from_schema)?,
            scope: ConsentScope::parse(&scope).map_err(StoreError::from_schema)?,
            decided_by: row.try_get("decided_by")?,
            decided_at: row.try_get("decided_at")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    async fn get_consent(&self, consent_id: Uuid) -> Result<ConsentRecord, StoreError> {
        let row = sqlx::query("SELECT * FROM consent_records WHERE consent_id = $1")
            .bind(consent_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StoreError::NotFound(format!("no such consent {consent_id}")))?;
        Self::consent_from_row(&row)
    }

    fn embedding_from_row(row: &PgRow) -> Result<EmbeddingRecord, StoreError> {
        let vector: pgvector::Vector = row.try_get("embedding")?;
        Ok(EmbeddingRecord {
            node_id: row.try_get("node_id")?,
            vector: vector.to_vec(),
            embedder_alias: row.try_get("embedder_alias")?,
            dims: row.try_get("dims")?,
            revision: row.try_get("revision")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn link_from_row(row: &PgRow) -> Result<LinkRecord, StoreError> {
        Ok(LinkRecord {
            link_id: row.try_get("link_id")?,
            source_ref: row.try_get("source_ref")?,
            target_ref: row.try_get("target_ref")?,
            similarity: row.try_get("similarity")?,
            weight: row.try_get("weight")?,
            category: row.try_get("category")?,
            user_overridden: row.try_get("user_overridden")?,
            revision: row.try_get("revision")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn rebalance_from_row(row: &PgRow) -> Result<RebalanceState, StoreError> {
        Ok(RebalanceState {
            category: row.try_get("category")?,
            eligible: row.try_get("eligible")?,
            marked_at: row.try_get("marked_at")?,
            last_recalc_at: row.try_get("last_recalc_at")?,
            config_rev: row.try_get("config_rev")?,
            revision: row.try_get("revision")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn uuid_vec(value: &serde_json::Value, field: &str) -> Result<Vec<Uuid>, StoreError> {
        let strings: Vec<String> = serde_json::from_value(value.clone())
            .map_err(|e| StoreError::ValidationFailed(format!("stored {field}: {e}")))?;
        strings
            .iter()
            .map(|s| Uuid::parse_str(s))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| StoreError::ValidationFailed(format!("stored {field}: {e}")))
    }

    fn matrix_from_row(row: &PgRow) -> Result<MatrixRecord, StoreError> {
        let status: String = row.try_get("status")?;
        let node_refs: serde_json::Value = row.try_get("node_refs")?;
        let link_refs: serde_json::Value = row.try_get("link_refs")?;
        Ok(MatrixRecord {
            matrix_id: row.try_get("matrix_id")?,
            status: MatrixStatus::parse(&status).map_err(StoreError::from_schema)?,
            category: row.try_get("category")?,
            revision: row.try_get("revision")?,
            audit_depth: row.try_get("audit_depth")?,
            node_refs: Self::uuid_vec(&node_refs, "node_refs")?,
            link_refs: Self::uuid_vec(&link_refs, "link_refs")?,
            emerged_by: row.try_get("emerged_by")?,
            config_rev: row.try_get("config_rev")?,
            committed_proposal_ref: row.try_get("committed_proposal_ref")?,
            committed_consent_ref: row.try_get("committed_consent_ref")?,
            committed_at: row.try_get("committed_at")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn report_from_row(row: &PgRow) -> Result<AuditReport, StoreError> {
        let auditor: String = row.try_get("auditor")?;
        let kind: String = row.try_get("kind")?;
        let claims: serde_json::Value = row.try_get("claims")?;
        Ok(AuditReport {
            report_id: row.try_get("report_id")?,
            job_id: row.try_get("job_id")?,
            matrix_ref: row.try_get("matrix_ref")?,
            matrix_revision: row.try_get("matrix_revision")?,
            auditor: AuditorKind::parse(&auditor).map_err(StoreError::from_schema)?,
            kind: ReportKind::parse(&kind).map_err(StoreError::from_schema)?,
            claims: serde_json::from_value(claims)
                .map_err(|e| StoreError::ValidationFailed(format!("stored claims: {e}")))?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn proposal_from_row(row: &PgRow) -> Result<JointProposal, StoreError> {
        let verdict: String = row.try_get("verdict")?;
        let report_refs: serde_json::Value = row.try_get("report_refs")?;
        let changes: serde_json::Value = row.try_get("changes")?;
        let reasons: serde_json::Value = row.try_get("reasons")?;
        Ok(JointProposal {
            proposal_id: row.try_get("proposal_id")?,
            job_id: row.try_get("job_id")?,
            matrix_ref: row.try_get("matrix_ref")?,
            matrix_revision: row.try_get("matrix_revision")?,
            report_refs: Self::uuid_vec(&report_refs, "report_refs")?,
            verdict: Verdict::parse(&verdict).map_err(StoreError::from_schema)?,
            changes: serde_json::from_value(changes)
                .map_err(|e| StoreError::ValidationFailed(format!("stored changes: {e}")))?,
            reasons: serde_json::from_value(reasons)
                .map_err(|e| StoreError::ValidationFailed(format!("stored reasons: {e}")))?,
            consent_ref: row.try_get("consent_ref")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    /// The truth-binding (Book II §2): every evidence ref resolves to a
    /// live node or link, or the claim does not validate.
    async fn validate_claims(&self, claims: &[Claim]) -> Result<(), StoreError> {
        for claim in claims {
            if claim.claim.is_empty() {
                return Err(StoreError::ValidationFailed(
                    "a claim must say something (Book II §2)".into(),
                ));
            }
            if claim.evidence_refs.is_empty() {
                return Err(StoreError::ValidationFailed(format!(
                    "claim '{}' carries no evidence — an unsupported word does not validate",
                    claim.claim
                )));
            }
            for evidence in &claim.evidence_refs {
                let resolves: bool = sqlx::query_scalar(
                    r#"SELECT EXISTS(SELECT 1 FROM nodes WHERE node_id = $1)
                        OR EXISTS(SELECT 1 FROM links WHERE link_id = $1)"#,
                )
                .bind(evidence)
                .fetch_one(&self.pool)
                .await?;
                if !resolves {
                    return Err(StoreError::ValidationFailed(format!(
                        "claim '{}': evidence {evidence} does not resolve to a live record (truth-binding)",
                        claim.claim
                    )));
                }
            }
        }
        Ok(())
    }

    /// The node's floor bucket — the category eligibility and links hang on.
    fn primary_category(node: &NodeRecord) -> String {
        node.classification
            .get(0)
            .and_then(|entry| entry.get("category"))
            .and_then(|v| v.as_str())
            .unwrap_or("unclassified")
            .to_string()
    }

    /// Doc 4 §5.2: an ingestion event makes recalculation eligible — and
    /// nothing more. Execution belongs to the user's trigger.
    async fn mark_rebalance_eligible(&self, category: &str) -> Result<(), StoreError> {
        sqlx::query(
            r#"INSERT INTO rebalance_state
                 (category, eligible, marked_at, schema_name, schema_version, produced_by)
               VALUES ($1, true, now(), 'RebalanceState', $2, 'store')
               ON CONFLICT (category) DO UPDATE
               SET eligible = true, marked_at = now(),
                   revision = rebalance_state.revision + 1"#,
        )
        .bind(category)
        .bind(RECORD_SCHEMA_VERSION)
        .execute(&self.pool)
        .await?;
        Ok(())
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
        let mut flags = self
            .write_flags(job_id, std::slice::from_ref(draft))
            .await?;
        flags
            .pop()
            .ok_or_else(|| StoreError::ValidationFailed("flag write returned nothing".into()))
    }

    async fn write_flags(
        &self,
        job_id: Uuid,
        drafts: &[FlagDraft],
    ) -> Result<Vec<ReadinessFlag>, StoreError> {
        if drafts.is_empty() {
            return Err(StoreError::ValidationFailed(
                "the FLAG step certifies at least one stage".into(),
            ));
        }
        let job = self.guard_actor(job_id, "write_flag", false).await?;
        // I.1: FLAG follows VALIDATE_OUT follows WRITE — the job stands WRITTEN.
        if job.status != JobStatus::Written {
            return Err(StoreError::ValidationFailed(format!(
                "a flag certifies written state; job is {} not WRITTEN (Laws I.1, III.2)",
                job.status
            )));
        }
        for draft in drafts {
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
            // III.2: a flag is written only after its certified outputs exist
            // and validate — flag-before-output is rejected (SC-B01).
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
        }
        let mut tx = self.pool.begin().await?;
        let mut flags = Vec::with_capacity(drafts.len());
        for draft in drafts {
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
            flags.push(Self::flag_from_row(&row)?);
        }
        // The WRITTEN → FLAGGED transition happens exactly once, however
        // many stages this FLAG step certifies.
        sqlx::query(
            r#"UPDATE job_records SET status = 'FLAGGED', revision = revision + 1
               WHERE job_id = $1 AND status = 'WRITTEN'"#,
        )
        .bind(job_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        let stages: Vec<&str> = drafts.iter().map(|d| d.stage.as_str()).collect();
        self.append_log(
            &job_id.to_string(),
            LogEvent::JobTransition,
            &serde_json::json!({ "from": "WRITTEN", "to": "FLAGGED", "stages": stages }),
            Severity::Info,
            "store",
        )
        .await?;
        Ok(flags)
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
        let flag_job = flag.job_id.ok_or_else(|| {
            StoreError::ValidationFailed(format!(
                "flag {flag_id} is office-authored; it certifies no job outputs to read"
            ))
        })?;
        let mut witnessed = Vec::with_capacity(flag.certifies.output_slots.len());
        let mut defect: Option<String> = None;
        for (slot, expected_rev) in flag
            .certifies
            .output_slots
            .iter()
            .zip(flag.certifies.revisions.iter())
        {
            match self.read_artifact(flag_job, slot).await {
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
        // Law IV.1 / SC-C01: a human-held classification is not agent-writable;
        // the only path through is a granted petition (execute_grant).
        if self.get_active_override(node_id).await?.is_some() {
            return Err(StoreError::OverrideConflict(format!(
                "node {node_id} classification is human-held (user_overridden); petition, never write (Law IV.1–IV.2)"
            )));
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

    async fn lay_category_override(
        &self,
        actor: &str,
        node_id: Uuid,
        classification: &serde_json::Value,
    ) -> Result<OverrideRecord, StoreError> {
        if !classification.is_array() {
            return Err(StoreError::ValidationFailed(
                "classification must be an array of bucket entries (doc 2 §2.5)".into(),
            ));
        }
        if let Some(pattern) = secrets::scan(&classification.to_string()) {
            return Err(StoreError::SecretDetected(format!(
                "override state matched secret pattern '{pattern}' (Law XV.2)"
            )));
        }
        let node = self.get_node(node_id).await?;
        let prior = self.get_active_override(node_id).await?;
        let mut tx = self.pool.begin().await?;
        let updated = sqlx::query(
            r#"UPDATE nodes SET classification = $3, revision = revision + 1
               WHERE node_id = $1 AND revision = $2"#,
        )
        .bind(node_id)
        .bind(node.revision)
        .bind(classification)
        .execute(&mut *tx)
        .await?;
        if updated.rows_affected() == 0 {
            return Err(StoreError::StaleRevision {
                expected: node.revision,
                subject: format!("node:{node_id}"),
            });
        }
        let row = sqlx::query(
            r#"INSERT INTO override_records
                 (override_id, subject_ref, kind, basis, prior_ref, consent_ref, protected_state,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, 'CATEGORY_REASSIGNED', 'SOVEREIGN_HAND', $3, NULL, $4,
                       'OverrideRecord', $5, $6)
               RETURNING *"#,
        )
        .bind(Uuid::now_v7())
        .bind(node_id)
        .bind(prior.map(|p| p.override_id))
        .bind(classification)
        .bind(RECORD_SCHEMA_VERSION)
        .bind(actor)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        self.append_log(
            &node_id.to_string(),
            LogEvent::OverrideLaid,
            &serde_json::json!({ "kind": "CATEGORY_REASSIGNED", "basis": "SOVEREIGN_HAND" }),
            Severity::Info,
            actor,
        )
        .await?;
        Self::override_from_row(&row)
    }

    async fn get_active_override(
        &self,
        subject_ref: Uuid,
    ) -> Result<Option<OverrideRecord>, StoreError> {
        let row = sqlx::query(
            r#"SELECT * FROM override_records WHERE subject_ref = $1
               ORDER BY produced_at DESC, override_id DESC LIMIT 1"#,
        )
        .bind(subject_ref)
        .fetch_optional(&self.pool)
        .await?;
        row.as_ref().map(Self::override_from_row).transpose()
    }

    async fn open_petition(
        &self,
        job_id: Uuid,
        draft: &PetitionDraft,
    ) -> Result<PetitionRecord, StoreError> {
        let job = self.guard_actor(job_id, "open_petition", false).await?;
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "petitions are opened during WORK, not {} (Law I.1)",
                job.status
            )));
        }
        let serialized = format!("{} {}", draft.reason, draft.proposed_change);
        if let Some(pattern) = secrets::scan(&serialized) {
            return Err(StoreError::SecretDetected(format!(
                "petition matched secret pattern '{pattern}' (Law XV.2)"
            )));
        }
        // v1: petitions concern existing overrides — an agent believing a
        // hand was laid in error (IV.2).
        if self.get_active_override(draft.subject_ref).await?.is_none() {
            return Err(StoreError::ValidationFailed(format!(
                "subject {} carries no override; there is nothing to petition (IV.2)",
                draft.subject_ref
            )));
        }
        let existing = sqlx::query(
            "SELECT * FROM petition_records WHERE subject_ref = $1 AND change_kind = $2",
        )
        .bind(draft.subject_ref)
        .bind(draft.change_kind.as_str())
        .fetch_optional(&self.pool)
        .await?;
        let existing = existing.as_ref().map(Self::petition_from_row).transpose()?;

        match existing {
            None => {
                let row = sqlx::query(
                    r#"INSERT INTO petition_records
                         (petition_id, subject_ref, change_kind, reason, evidence_refs,
                          proposed_change, schema_name, schema_version, produced_by)
                       VALUES ($1, $2, $3, $4, $5, $6, 'PetitionRecord', $7, $8::text)
                       RETURNING *"#,
                )
                .bind(Uuid::now_v7())
                .bind(draft.subject_ref)
                .bind(draft.change_kind.as_str())
                .bind(&draft.reason)
                .bind(serde_json::to_value(&draft.evidence_refs).expect("string vec serializes"))
                .bind(&draft.proposed_change)
                .bind(RECORD_SCHEMA_VERSION)
                .bind(job_id)
                .fetch_one(&self.pool)
                .await?;
                self.append_log(
                    &draft.subject_ref.to_string(),
                    LogEvent::PetitionOpened,
                    &serde_json::json!({ "change_kind": draft.change_kind.as_str(), "occurrence": 1 }),
                    Severity::Info,
                    &job_id.to_string(),
                )
                .await?;
                Self::petition_from_row(&row)
            }
            Some(prior) => match prior.status {
                // IV.2: SILENCED auto-suppresses — still counted, still
                // logged severity: suppressed, never purged (SC-C03).
                PetitionStatus::Silenced => {
                    let row = sqlx::query(
                        r#"UPDATE petition_records SET occurrence_count = occurrence_count + 1
                           WHERE petition_id = $1 RETURNING *"#,
                    )
                    .bind(prior.petition_id)
                    .fetch_one(&self.pool)
                    .await?;
                    self.append_log(
                        &draft.subject_ref.to_string(),
                        LogEvent::PetitionOpened,
                        &serde_json::json!({
                            "change_kind": draft.change_kind.as_str(),
                            "suppressed": true,
                        }),
                        Severity::Suppressed,
                        &job_id.to_string(),
                    )
                    .await?;
                    Self::petition_from_row(&row)
                }
                // A pending grant is not re-petitionable; the loop must
                // close first (IV.5).
                PetitionStatus::Granted if prior.execution_job_ref.is_none() => {
                    Err(StoreError::ValidationFailed(format!(
                        "petition {} is GRANTED and awaiting execution; nothing to ask",
                        prior.petition_id
                    )))
                }
                // Recurrence escalates: OPEN/DECLINED/executed-GRANTED
                // lineages all become ESCALATED with the count advanced.
                _ => {
                    let row = sqlx::query(
                        r#"UPDATE petition_records
                           SET occurrence_count = occurrence_count + 1, status = 'ESCALATED',
                               reason = $2, proposed_change = $3, consent_ref = NULL,
                               execution_job_ref = NULL, resolved_at = NULL
                           WHERE petition_id = $1 RETURNING *"#,
                    )
                    .bind(prior.petition_id)
                    .bind(&draft.reason)
                    .bind(&draft.proposed_change)
                    .fetch_one(&self.pool)
                    .await?;
                    self.append_log(
                        &draft.subject_ref.to_string(),
                        LogEvent::PetitionOpened,
                        &serde_json::json!({
                            "change_kind": draft.change_kind.as_str(),
                            "occurrence": prior.occurrence_count + 1,
                            "escalated": true,
                        }),
                        Severity::Info,
                        &job_id.to_string(),
                    )
                    .await?;
                    Self::petition_from_row(&row)
                }
            },
        }
    }

    async fn resolve_petition(
        &self,
        actor: &str,
        petition_id: Uuid,
        decision: ConsentDecision,
    ) -> Result<PetitionRecord, StoreError> {
        let petition = self.get_petition(petition_id).await?;
        if !matches!(
            petition.status,
            PetitionStatus::Open | PetitionStatus::Escalated
        ) {
            return Err(StoreError::ValidationFailed(format!(
                "petition {petition_id} is {}; only OPEN or ESCALATED petitions resolve",
                petition.status
            )));
        }
        let (new_status, consent_id) = match decision {
            ConsentDecision::Granted => {
                let consent_id = Uuid::now_v7();
                sqlx::query(
                    r#"INSERT INTO consent_records
                         (consent_id, subject_ref, decision, scope, decided_by,
                          schema_name, schema_version, produced_by)
                       VALUES ($1, $2, 'GRANTED', 'ITEM', $3, 'ConsentRecord', $4, $3)"#,
                )
                .bind(consent_id)
                .bind(petition_id)
                .bind(actor)
                .bind(RECORD_SCHEMA_VERSION)
                .execute(&self.pool)
                .await?;
                (PetitionStatus::Granted, Some(consent_id))
            }
            ConsentDecision::Declined => (PetitionStatus::Declined, None),
            ConsentDecision::Silenced => (PetitionStatus::Silenced, None),
            other => {
                return Err(StoreError::ValidationFailed(format!(
                    "{other} is not a petition answer; the terminal answers are GRANTED, DECLINED, SILENCED (IV.2)"
                )))
            }
        };
        let row = sqlx::query(
            r#"UPDATE petition_records
               SET status = $2, consent_ref = $3, resolved_at = now()
               WHERE petition_id = $1 RETURNING *"#,
        )
        .bind(petition_id)
        .bind(new_status.as_str())
        .bind(consent_id)
        .fetch_one(&self.pool)
        .await?;
        self.append_log(
            &petition.subject_ref.to_string(),
            LogEvent::PetitionResolved,
            &serde_json::json!({ "petition": petition_id.to_string(), "decision": new_status.as_str() }),
            Severity::Info,
            actor,
        )
        .await?;
        Self::petition_from_row(&row)
    }

    async fn execute_grant(
        &self,
        notary_job_id: Uuid,
        petition_id: Uuid,
    ) -> Result<OverrideRecord, StoreError> {
        let job = self
            .guard_actor(notary_job_id, "execute_grant", false)
            .await?;
        if job.agent_type != AgentType::Notary {
            return Err(StoreError::ValidationFailed(format!(
                "only a summoned Notary executes consent, not {} (Book II §3)",
                job.agent_type
            )));
        }
        let petition = self.get_petition(petition_id).await?;
        if petition.status != PetitionStatus::Granted {
            return Err(StoreError::ValidationFailed(format!(
                "petition {petition_id} is {}, not GRANTED; there is no consent to execute",
                petition.status
            )));
        }
        // Idempotent retry (SC-A03 discipline): already executed → converge
        // on the successor already laid.
        if petition.execution_job_ref.is_some() {
            let active = self.get_active_override(petition.subject_ref).await?;
            if let Some(successor) = active {
                if successor.consent_ref == petition.consent_ref {
                    return Ok(successor);
                }
            }
            return Err(StoreError::ValidationFailed(format!(
                "petition {petition_id} records an execution but no matching successor override resolves"
            )));
        }
        // The chain: override → petition → consent, every link resolving.
        let consent_id = petition.consent_ref.ok_or_else(|| {
            StoreError::ValidationFailed(format!(
                "GRANTED petition {petition_id} carries no consent_ref — the chain does not resolve"
            ))
        })?;
        let consent = self.get_consent(consent_id).await?;
        if consent.decision != ConsentDecision::Granted || consent.subject_ref != petition_id {
            return Err(StoreError::ValidationFailed(format!(
                "consent {consent_id} does not grant petition {petition_id} — the chain does not resolve"
            )));
        }
        let prior = self
            .get_active_override(petition.subject_ref)
            .await?
            .ok_or_else(|| {
                StoreError::ValidationFailed(format!(
                    "no active override on subject {} — the chain does not resolve",
                    petition.subject_ref
                ))
            })?;
        if petition.change_kind != OverrideKind::CategoryReassigned {
            return Err(StoreError::ValidationFailed(format!(
                "{} has no executable surface in v1 (SLICE_03 §2)",
                petition.change_kind
            )));
        }
        // The subject must still validate when the Notary arrives (IV.5).
        let node = self.get_node(petition.subject_ref).await?;

        let mut tx = self.pool.begin().await?;
        let updated = sqlx::query(
            r#"UPDATE nodes SET classification = $3, revision = revision + 1
               WHERE node_id = $1 AND revision = $2"#,
        )
        .bind(node.node_id)
        .bind(node.revision)
        .bind(&petition.proposed_change)
        .execute(&mut *tx)
        .await?;
        if updated.rows_affected() == 0 {
            return Err(StoreError::StaleRevision {
                expected: node.revision,
                subject: format!("node:{}", node.node_id),
            });
        }
        // The successor override: the datum stays human-held (IV.5). Stamped
        // with the consent's decider — the authority is the consent.
        let row = sqlx::query(
            r#"INSERT INTO override_records
                 (override_id, subject_ref, kind, basis, prior_ref, consent_ref, protected_state,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, 'GRANTED_PETITION', $4, $5, $6, 'OverrideRecord', $7, $8)
               RETURNING *"#,
        )
        .bind(Uuid::now_v7())
        .bind(petition.subject_ref)
        .bind(petition.change_kind.as_str())
        .bind(prior.override_id)
        .bind(consent_id)
        .bind(&petition.proposed_change)
        .bind(RECORD_SCHEMA_VERSION)
        .bind(&consent.decided_by)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query("UPDATE petition_records SET execution_job_ref = $2 WHERE petition_id = $1")
            .bind(petition_id)
            .bind(notary_job_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        // Provenance linking all four: override, petition, consent, result (IV.5).
        self.append_log(
            &petition.subject_ref.to_string(),
            LogEvent::OverrideLaid,
            &serde_json::json!({
                "basis": "GRANTED_PETITION",
                "prior_override": prior.override_id.to_string(),
                "petition": petition_id.to_string(),
                "consent": consent_id.to_string(),
                "executed_by_job": notary_job_id.to_string(),
            }),
            Severity::Info,
            &consent.decided_by,
        )
        .await?;
        Self::override_from_row(&row)
    }

    async fn stalled_grants(&self, stall_ms: i64) -> Result<Vec<PetitionRecord>, StoreError> {
        let rows = sqlx::query(
            r#"SELECT * FROM petition_records
               WHERE status = 'GRANTED' AND execution_job_ref IS NULL
                 AND resolved_at < now() - ($1::double precision * interval '1 millisecond')
               ORDER BY resolved_at"#,
        )
        .bind(stall_ms)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(Self::petition_from_row).collect()
    }

    async fn get_petition(&self, petition_id: Uuid) -> Result<PetitionRecord, StoreError> {
        let row = sqlx::query("SELECT * FROM petition_records WHERE petition_id = $1")
            .bind(petition_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StoreError::NotFound(format!("no such petition {petition_id}")))?;
        Self::petition_from_row(&row)
    }

    async fn put_embedding(
        &self,
        job_id: Uuid,
        node_id: Uuid,
        embedder_alias: &str,
        vector: &[f32],
    ) -> Result<EmbeddingRecord, StoreError> {
        let job = self.guard_actor(job_id, "put_embedding", false).await?;
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "embeddings are persisted during WORK, not {} (Law I.1)",
                job.status
            )));
        }
        if vector.len() != 256 {
            return Err(StoreError::ValidationFailed(format!(
                "embedding must be 256-dimensional, got {}",
                vector.len()
            )));
        }
        let node = self.get_node(node_id).await?;
        // One transaction: vector + log + eligibility mark land together or
        // not at all. A crash cannot consume the retry key (the embedding's
        // existence) while losing the ingestion's eligibility side effect.
        let mut tx = self.pool.begin().await?;
        let inserted = sqlx::query(
            r#"INSERT INTO embeddings
                 (node_id, embedding, embedder_alias, dims,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, 256, 'EmbeddingRecord', $4, $5::text)
               ON CONFLICT (node_id) DO NOTHING
               RETURNING *"#,
        )
        .bind(node_id)
        .bind(pgvector::Vector::from(vector.to_vec()))
        .bind(embedder_alias)
        .bind(RECORD_SCHEMA_VERSION)
        .bind(job_id)
        .fetch_optional(&mut *tx)
        .await?;
        match inserted {
            Some(row) => {
                let subject = node_id.to_string();
                sqlx::query(
                    r#"INSERT INTO log_snapshots
                         (log_id, subject_ref, event, payload, prior_ref, severity,
                          schema_name, schema_version, produced_by)
                       VALUES ($1, $2, 'EMBEDDED', $3,
                         (SELECT log_id FROM log_snapshots WHERE subject_ref = $2
                          ORDER BY seq DESC LIMIT 1),
                         'info', 'LogSnapshot', $4, $5::text)"#,
                )
                .bind(Uuid::now_v7())
                .bind(&subject)
                .bind(serde_json::json!({ "embedder_alias": embedder_alias, "dims": 256 }))
                .bind(RECORD_SCHEMA_VERSION)
                .bind(job_id)
                .execute(&mut *tx)
                .await?;
                // Ingestion event: recalculation becomes eligible (doc 4 §5.2).
                sqlx::query(
                    r#"INSERT INTO rebalance_state
                         (category, eligible, marked_at, schema_name, schema_version, produced_by)
                       VALUES ($1, true, now(), 'RebalanceState', $2, 'store')
                       ON CONFLICT (category) DO UPDATE
                       SET eligible = true, marked_at = now(),
                           revision = rebalance_state.revision + 1"#,
                )
                .bind(Self::primary_category(&node))
                .bind(RECORD_SCHEMA_VERSION)
                .execute(&mut *tx)
                .await?;
                tx.commit().await?;
                Self::embedding_from_row(&row)
            }
            // One vector per node: the existing row wins, untouched.
            None => {
                tx.rollback().await?;
                self.get_embedding(node_id).await?.ok_or_else(|| {
                    StoreError::NotFound(format!("embedding vanished for {node_id}"))
                })
            }
        }
    }

    async fn get_embedding(&self, node_id: Uuid) -> Result<Option<EmbeddingRecord>, StoreError> {
        let row = sqlx::query("SELECT * FROM embeddings WHERE node_id = $1")
            .bind(node_id)
            .fetch_optional(&self.pool)
            .await?;
        row.as_ref().map(Self::embedding_from_row).transpose()
    }

    async fn embedding_backlog(
        &self,
        scope: Option<&[Uuid]>,
    ) -> Result<Vec<NodeRecord>, StoreError> {
        let rows = sqlx::query(
            r#"SELECT n.* FROM nodes n
               LEFT JOIN embeddings e ON e.node_id = n.node_id
               WHERE n.normalized AND e.node_id IS NULL
                 AND ($1::uuid[] IS NULL OR n.node_id = ANY($1))
               ORDER BY n.produced_at"#,
        )
        .bind(scope.map(<[Uuid]>::to_vec))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(Self::node_from_row).collect()
    }

    async fn similar_nodes(
        &self,
        node_id: Uuid,
        min_similarity: f32,
        scope: Option<&[Uuid]>,
    ) -> Result<Vec<(Uuid, f32)>, StoreError> {
        let rows = sqlx::query(
            r#"SELECT e2.node_id, (1 - (e2.embedding <=> e1.embedding))::float4 AS sim
               FROM embeddings e1
               JOIN embeddings e2 ON e2.node_id <> e1.node_id
               WHERE e1.node_id = $1
                 AND ($3::uuid[] IS NULL OR e2.node_id = ANY($3))
                 AND (1 - (e2.embedding <=> e1.embedding)) >= $2
               ORDER BY e2.embedding <=> e1.embedding"#,
        )
        .bind(node_id)
        .bind(f64::from(min_similarity))
        .bind(scope.map(<[Uuid]>::to_vec))
        .fetch_all(&self.pool)
        .await?;
        rows.iter()
            .map(|row| Ok((row.try_get("node_id")?, row.try_get("sim")?)))
            .collect()
    }

    async fn draw_link(
        &self,
        job_id: Uuid,
        a: Uuid,
        b: Uuid,
        similarity: f32,
        category: &str,
    ) -> Result<LinkRecord, StoreError> {
        let job = self.guard_actor(job_id, "draw_link", false).await?;
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "links are drawn during WORK, not {} (Law I.1)",
                job.status
            )));
        }
        if a == b {
            return Err(StoreError::ValidationFailed(
                "a node does not bond with itself".into(),
            ));
        }
        // Note: drawing a link marks NO rebalance eligibility. Eligibility
        // marks on *ingestion* events (doc 4 §5.2); link consolidation is
        // the recalculation itself, not an ingestion.
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        // Race-proof creation: ON CONFLICT arbitrates two concurrent
        // first-draws of the same pair; the loser falls to the update path.
        let inserted = sqlx::query(
            r#"INSERT INTO links
                 (link_id, source_ref, target_ref, similarity, category,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, 'LinkRecord', $6, $7::text)
               ON CONFLICT (source_ref, target_ref) DO NOTHING
               RETURNING *"#,
        )
        .bind(Uuid::now_v7())
        .bind(lo)
        .bind(hi)
        .bind(similarity)
        .bind(category)
        .bind(RECORD_SCHEMA_VERSION)
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?;
        if let Some(row) = inserted {
            self.append_log(
                &lo.to_string(),
                LogEvent::LinkDrawn,
                &serde_json::json!({
                    "target": hi.to_string(),
                    "similarity": similarity,
                    "category": category,
                }),
                Severity::Info,
                &job_id.to_string(),
            )
            .await?;
            return Self::link_from_row(&row);
        }
        // The pair exists: refresh similarity in place. The single-statement
        // guard on user_overridden means a human hand laid mid-race is never
        // reverted (doc 4 §4.4); zero rows updated ⇒ overridden ⇒ return it
        // untouched.
        let row = sqlx::query(
            r#"UPDATE links SET similarity = $3, revision = revision + 1
               WHERE source_ref = $1 AND target_ref = $2 AND NOT user_overridden
               RETURNING *"#,
        )
        .bind(lo)
        .bind(hi)
        .bind(similarity)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(row) => Self::link_from_row(&row),
            None => {
                let row =
                    sqlx::query("SELECT * FROM links WHERE source_ref = $1 AND target_ref = $2")
                        .bind(lo)
                        .bind(hi)
                        .fetch_optional(&self.pool)
                        .await?
                        .ok_or_else(|| {
                            StoreError::NotFound(format!("link {lo}<->{hi} vanished mid-draw"))
                        })?;
                Self::link_from_row(&row)
            }
        }
    }

    async fn links_by_category(
        &self,
        category: &str,
        scope: Option<&[Uuid]>,
    ) -> Result<Vec<LinkRecord>, StoreError> {
        let rows = sqlx::query(
            r#"SELECT * FROM links WHERE category = $1
                 AND ($2::uuid[] IS NULL OR (source_ref = ANY($2) AND target_ref = ANY($2)))
               ORDER BY produced_at"#,
        )
        .bind(category)
        .bind(scope.map(<[Uuid]>::to_vec))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(Self::link_from_row).collect()
    }

    async fn set_link_weight(
        &self,
        job_id: Uuid,
        link_id: Uuid,
        expected_revision: i32,
        weight: f32,
    ) -> Result<LinkRecord, StoreError> {
        self.guard_actor(job_id, "set_link_weight", false).await?;
        let row = sqlx::query(
            r#"UPDATE links SET weight = $3, revision = revision + 1
               WHERE link_id = $1 AND revision = $2 AND NOT user_overridden
               RETURNING *"#,
        )
        .bind(link_id)
        .bind(expected_revision)
        .bind(weight)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(row) => Self::link_from_row(&row),
            None => {
                let current = sqlx::query("SELECT * FROM links WHERE link_id = $1")
                    .bind(link_id)
                    .fetch_optional(&self.pool)
                    .await?
                    .ok_or_else(|| StoreError::NotFound(format!("no such link {link_id}")))?;
                let link = Self::link_from_row(&current)?;
                if link.user_overridden {
                    return Err(StoreError::OverrideConflict(format!(
                        "link {link_id} is human-held; recalculation works around fixed stars (Handbook §4.5)"
                    )));
                }
                Err(StoreError::StaleRevision {
                    expected: expected_revision,
                    subject: format!("link:{link_id}"),
                })
            }
        }
    }

    async fn live_weights(
        &self,
        category: &str,
        scope: Option<&[Uuid]>,
    ) -> Result<LiveWeights, StoreError> {
        // Law VI.1: a density evaluation lacking config citation is invalid;
        // an unset sovereign threshold is a refusal, never a guess.
        let threshold = self.get_config("coherence_threshold").await.map_err(|_| {
            StoreError::ValidationFailed(
                "coherence_threshold is not set; a density evaluation must cite the sovereign constant (Law VI.1)"
                    .into(),
            )
        })?;
        let threshold_value = threshold.value.as_f64().ok_or_else(|| {
            StoreError::ValidationFailed("coherence_threshold must be numeric".into())
        })?;
        let links = self.links_by_category(category, scope).await?;
        let mut nodes = std::collections::HashSet::new();
        for link in &links {
            nodes.insert(link.source_ref);
            nodes.insert(link.target_ref);
        }
        #[allow(clippy::cast_precision_loss)] // link/node counts are tiny vs f32 range
        let density = if nodes.is_empty() {
            0.0
        } else {
            links.len() as f32 / nodes.len() as f32
        };
        let live = f64::from(density) >= threshold_value;
        Ok(LiveWeights {
            category: category.to_string(),
            live,
            density,
            config_rev: threshold.revision,
            // Below the threshold, weights are inert: no force in any consumer.
            weights: if live {
                links.iter().map(|l| (l.link_id, l.weight)).collect()
            } else {
                Vec::new()
            },
        })
    }

    async fn rebalance_state(&self, category: &str) -> Result<Option<RebalanceState>, StoreError> {
        let row = sqlx::query("SELECT * FROM rebalance_state WHERE category = $1")
            .bind(category)
            .fetch_optional(&self.pool)
            .await?;
        row.as_ref().map(Self::rebalance_from_row).transpose()
    }

    async fn claim_rebalance_eligibility(
        &self,
        category: &str,
        config_rev: Option<i32>,
    ) -> Result<bool, StoreError> {
        // Atomic check-and-claim: the `AND eligible` predicate is the
        // arbiter — N racing executors, exactly one claim. Marks laid by
        // concurrent ingestions after this statement survive untouched
        // (nothing clears post-pass), so no ingestion event is ever lost.
        let result = sqlx::query(
            r#"UPDATE rebalance_state
               SET eligible = false, last_recalc_at = now(), config_rev = $2,
                   revision = revision + 1
               WHERE category = $1 AND eligible"#,
        )
        .bind(category)
        .bind(config_rev)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn mark_rebalance_eligible(&self, category: &str) -> Result<(), StoreError> {
        PgStore::mark_rebalance_eligible(self, category).await
    }

    async fn store_now(&self) -> Result<time::OffsetDateTime, StoreError> {
        Ok(sqlx::query_scalar("SELECT now()")
            .fetch_one(&self.pool)
            .await?)
    }

    async fn emerge_postulant(
        &self,
        job_id: Uuid,
        category: &str,
        scope: Option<&[Uuid]>,
    ) -> Result<Option<MatrixRecord>, StoreError> {
        let job = self.guard_actor(job_id, "emerge_postulant", false).await?;
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "emergence is recorded during WORK, not {} (Law I.1)",
                job.status
            )));
        }
        // VI.2: emergence detection is the Aggregator's alone; no other
        // agent may declare a Postulant.
        if job.agent_type != AgentType::Aggregator {
            return Err(StoreError::ValidationFailed(format!(
                "emergence belongs to the Aggregators; {} may not declare a Postulant (Law VI.2)",
                job.agent_type
            )));
        }
        // Law VI.1: the evaluation cites the revision it read, or it does
        // not happen.
        let threshold = self.get_config("coherence_threshold").await.map_err(|_| {
            StoreError::ValidationFailed(
                "coherence_threshold is not set; a density evaluation must cite the sovereign constant (Law VI.1)"
                    .into(),
            )
        })?;
        let threshold_value = threshold.value.as_f64().ok_or_else(|| {
            StoreError::ValidationFailed("coherence_threshold must be numeric".into())
        })?;
        // One live matrix per category: emergence is idempotent.
        if self.live_matrix_for_category(category).await?.is_some() {
            return Ok(None);
        }
        let links = self.links_by_category(category, scope).await?;
        let mut nodes = std::collections::HashSet::new();
        for link in &links {
            nodes.insert(link.source_ref);
            nodes.insert(link.target_ref);
        }
        #[allow(clippy::cast_precision_loss)] // counts are tiny vs f32 range
        let density = if nodes.is_empty() {
            0.0f32
        } else {
            links.len() as f32 / nodes.len() as f32
        };
        if f64::from(density) < threshold_value {
            return Ok(None);
        }
        let mut node_refs: Vec<String> = nodes.iter().map(Uuid::to_string).collect();
        node_refs.sort();
        let link_refs: Vec<String> = links.iter().map(|l| l.link_id.to_string()).collect();
        // One transaction: the Postulant and its emergence record land
        // together (Law V.1). ON CONFLICT on the live-matrix partial index
        // arbitrates racing passes: the loser records nothing.
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"INSERT INTO matrices
                 (matrix_id, category, node_refs, link_refs, emerged_by, config_rev,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, $6, 'MatrixRecord', $7, $5::text)
               ON CONFLICT (category) WHERE status IN ('POSTULANT','CARDINAL') DO NOTHING
               RETURNING *"#,
        )
        .bind(Uuid::now_v7())
        .bind(category)
        .bind(serde_json::to_value(&node_refs).expect("string vec serializes"))
        .bind(serde_json::to_value(&link_refs).expect("string vec serializes"))
        .bind(job_id)
        .bind(threshold.revision)
        .bind(RECORD_SCHEMA_VERSION)
        .fetch_optional(&mut *tx)
        .await?;
        let Some(row) = row else {
            tx.rollback().await?;
            return Ok(None);
        };
        let matrix = Self::matrix_from_row(&row)?;
        let subject = matrix.matrix_id.to_string();
        sqlx::query(
            r#"INSERT INTO log_snapshots
                 (log_id, subject_ref, event, payload, prior_ref, severity,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, 'POSTULANT_EMERGED', $3,
                 (SELECT log_id FROM log_snapshots WHERE subject_ref = $2
                  ORDER BY seq DESC LIMIT 1),
                 'info', 'LogSnapshot', $4, $5::text)"#,
        )
        .bind(Uuid::now_v7())
        .bind(&subject)
        .bind(serde_json::json!({
            "category": category,
            "density": density,
            "config_rev": threshold.revision,
            "audit_depth": 0,
        }))
        .bind(RECORD_SCHEMA_VERSION)
        .bind(job_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(Some(matrix))
    }

    async fn get_matrix(&self, matrix_id: Uuid) -> Result<MatrixRecord, StoreError> {
        let row = sqlx::query("SELECT * FROM matrices WHERE matrix_id = $1")
            .bind(matrix_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StoreError::NotFound(format!("no such matrix {matrix_id}")))?;
        Self::matrix_from_row(&row)
    }

    async fn live_matrix_for_category(
        &self,
        category: &str,
    ) -> Result<Option<MatrixRecord>, StoreError> {
        let row = sqlx::query(
            "SELECT * FROM matrices WHERE category = $1 AND status IN ('POSTULANT','CARDINAL')",
        )
        .bind(category)
        .fetch_optional(&self.pool)
        .await?;
        row.as_ref().map(Self::matrix_from_row).transpose()
    }

    async fn file_audit_report(
        &self,
        job_id: Uuid,
        draft: &AuditReportDraft,
    ) -> Result<AuditReport, StoreError> {
        let job = self.guard_actor(job_id, "file_audit_report", false).await?;
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "reports are filed during WORK, not {} (Law I.1)",
                job.status
            )));
        }
        if job.agent_type != AgentType::Auditor {
            return Err(StoreError::ValidationFailed(format!(
                "only an Auditor files audit reports, not {} (Book II §2)",
                job.agent_type
            )));
        }
        let matrix = self.get_matrix(draft.matrix_ref).await?;
        if matrix.status != MatrixStatus::Postulant {
            return Err(StoreError::ValidationFailed(format!(
                "audit tries Postulants; matrix {} is {} (Law VI.3)",
                matrix.matrix_id, matrix.status
            )));
        }
        if matrix.revision != draft.matrix_revision {
            return Err(StoreError::ValidationFailed(format!(
                "report addresses matrix revision {} but the store holds {} — the world moved on",
                draft.matrix_revision, matrix.revision
            )));
        }
        // The truth-binding: a claim whose evidence does not resolve fails
        // VALIDATE_OUT here, and the report never exists to flag (SC-D06).
        self.validate_claims(&draft.claims).await?;
        let expected_kind = match draft.auditor {
            AuditorKind::Gabriel => ReportKind::Affirmation,
            AuditorKind::Lucy => ReportKind::Indictment,
        };
        if draft.kind != expected_kind {
            return Err(StoreError::ValidationFailed(format!(
                "{} files {}, nothing else (Book II §2)",
                draft.auditor, expected_kind
            )));
        }
        let row = sqlx::query(
            r#"INSERT INTO audit_reports
                 (report_id, job_id, matrix_ref, matrix_revision, auditor, kind, claims,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, 'AuditReport', $8, $2::text)
               RETURNING *"#,
        )
        .bind(Uuid::now_v7())
        .bind(job_id)
        .bind(draft.matrix_ref)
        .bind(draft.matrix_revision)
        .bind(draft.auditor.as_str())
        .bind(draft.kind.as_str())
        .bind(serde_json::to_value(&draft.claims).expect("claims serialize"))
        .bind(RECORD_SCHEMA_VERSION)
        .fetch_one(&self.pool)
        .await?;
        self.append_log(
            &draft.matrix_ref.to_string(),
            LogEvent::ReportFiled,
            &serde_json::json!({
                "auditor": draft.auditor.as_str(),
                "kind": draft.kind.as_str(),
                "claims": draft.claims.len(),
                "matrix_revision": draft.matrix_revision,
            }),
            Severity::Info,
            &job_id.to_string(),
        )
        .await?;
        Self::report_from_row(&row)
    }

    async fn read_audit_report(
        &self,
        reader_job_id: Uuid,
        report_id: Uuid,
    ) -> Result<AuditReport, StoreError> {
        self.guard_actor(reader_job_id, "read_audit_report", false)
            .await?;
        let row = sqlx::query("SELECT * FROM audit_reports WHERE report_id = $1")
            .bind(report_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StoreError::NotFound(format!("no such report {report_id}")))?;
        let report = Self::report_from_row(&row)?;
        if report.job_id == reader_job_id {
            return Ok(report);
        }
        // SC-D04: until the barrier certifies, no one reads another's
        // report — the auditors work blind to each other by construction.
        if !self
            .audit_barrier_certified(report.matrix_ref, report.matrix_revision)
            .await?
        {
            self.append_log(
                &report.matrix_ref.to_string(),
                LogEvent::Violation,
                &serde_json::json!({
                    "operation": "read_audit_report",
                    "reader": reader_job_id.to_string(),
                    "report": report_id.to_string(),
                    "pre_barrier": true,
                }),
                Severity::Violation,
                "store",
            )
            .await?;
            return Err(StoreError::Forbidden(format!(
                "report {report_id} is sealed until the AND-barrier certifies (Book II §2, SC-D04)"
            )));
        }
        Ok(report)
    }

    async fn audit_reports_for(
        &self,
        matrix_id: Uuid,
        matrix_revision: i32,
    ) -> Result<Vec<AuditReport>, StoreError> {
        let rows = sqlx::query(
            r#"SELECT * FROM audit_reports
               WHERE matrix_ref = $1 AND matrix_revision = $2 ORDER BY auditor"#,
        )
        .bind(matrix_id)
        .bind(matrix_revision)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(Self::report_from_row).collect()
    }

    async fn certify_audit_barrier(&self, matrix_id: Uuid) -> Result<ReadinessFlag, StoreError> {
        let matrix = self.get_matrix(matrix_id).await?;
        if matrix.status != MatrixStatus::Postulant {
            return Err(StoreError::ValidationFailed(format!(
                "audit tries Postulants; matrix {matrix_id} is {} (Law VI.3)",
                matrix.status
            )));
        }
        let reports = self.audit_reports_for(matrix_id, matrix.revision).await?;
        if reports.len() != 2 {
            return Err(StoreError::ValidationFailed(format!(
                "the barrier holds: {} of 2 reports present for matrix {matrix_id} rev {} (doc 3 §3.3)",
                reports.len(),
                matrix.revision
            )));
        }
        for report in &reports {
            // Both filing jobs must have flagged (their labor certified)…
            let filer = self.get_job(report.job_id).await?;
            if !matches!(filer.status, JobStatus::Flagged | JobStatus::Terminated) {
                return Err(StoreError::ValidationFailed(format!(
                    "the barrier holds: {}'s filing job is {} — uncertified labor (Law III.2)",
                    report.auditor, filer.status
                )));
            }
            // …and the underlying state must re-validate (Law III.3: a flag
            // is testimony; the state is the witness).
            self.validate_claims(&report.claims).await.map_err(|e| {
                StoreError::FlagUntrusted(format!(
                    "the barrier holds: {}'s report no longer validates: {e}",
                    report.auditor
                ))
            })?;
        }
        // The composite readiness flag — office-authored: the supervisor
        // certifies, the dispatcher invokes (doc 3 §3.2–3.3).
        let stage = format!("supervisor:audit_barrier:{matrix_id}:{}", matrix.revision);
        let certifies = godhead_schemas::Certifies {
            output_slots: reports.iter().map(|r| r.report_id.to_string()).collect(),
            revisions: vec![matrix.revision; 2],
        };
        let validator = godhead_schemas::Validator {
            id: "supervisor/audit_barrier".to_string(),
            version: "1.0.0".to_string(),
        };
        // One certification per barrier: the office-flag unique index
        // arbitrates racing supervisors; the loser adopts the standing flag.
        let row = sqlx::query(
            r#"INSERT INTO readiness_flags
                 (flag_id, job_id, stage, certifies, validator,
                  schema_name, schema_version, produced_by)
               VALUES ($1, NULL, $2, $3, $4, 'ReadinessFlag', $5, 'supervisor')
               ON CONFLICT (stage) WHERE job_id IS NULL DO NOTHING
               RETURNING *"#,
        )
        .bind(Uuid::now_v7())
        .bind(&stage)
        .bind(serde_json::to_value(&certifies).expect("certifies serializes"))
        .bind(serde_json::to_value(&validator).expect("validator serializes"))
        .bind(RECORD_SCHEMA_VERSION)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(row) => Self::flag_from_row(&row),
            None => {
                let row = sqlx::query(
                    "SELECT * FROM readiness_flags WHERE stage = $1 AND job_id IS NULL",
                )
                .bind(&stage)
                .fetch_one(&self.pool)
                .await?;
                Self::flag_from_row(&row)
            }
        }
    }

    async fn audit_barrier_certified(
        &self,
        matrix_id: Uuid,
        matrix_revision: i32,
    ) -> Result<bool, StoreError> {
        let stage = format!("supervisor:audit_barrier:{matrix_id}:{matrix_revision}");
        let exists: bool = sqlx::query_scalar(
            r#"SELECT EXISTS(SELECT 1 FROM readiness_flags
               WHERE stage = $1 AND status IN ('ACTIVE','CONSUMED'))"#,
        )
        .bind(stage)
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    async fn file_joint_proposal(
        &self,
        job_id: Uuid,
        draft: &ProposalDraft,
    ) -> Result<JointProposal, StoreError> {
        let job = self
            .guard_actor(job_id, "file_joint_proposal", false)
            .await?;
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "proposals are filed during WORK, not {} (Law I.1)",
                job.status
            )));
        }
        if job.agent_type != AgentType::Reconciler {
            return Err(StoreError::ValidationFailed(format!(
                "only Reconciliation files a Joint Proposal, not {} (Book II §2)",
                job.agent_type
            )));
        }
        // The barrier released this labor; it had better be certified.
        if !self
            .audit_barrier_certified(draft.matrix_ref, draft.matrix_revision)
            .await?
        {
            return Err(StoreError::ValidationFailed(
                "no certified barrier stands behind this proposal (doc 3 §3.3)".into(),
            ));
        }
        let matrix = self.get_matrix(draft.matrix_ref).await?;
        if matrix.status != MatrixStatus::Postulant {
            return Err(StoreError::ValidationFailed(format!(
                "audit tries Postulants; matrix {} is {} (Law VI.3)",
                matrix.matrix_id, matrix.status
            )));
        }
        if matrix.revision != draft.matrix_revision {
            return Err(StoreError::ValidationFailed(format!(
                "proposal addresses revision {} but the store holds {}",
                draft.matrix_revision, matrix.revision
            )));
        }
        let reports = self
            .audit_reports_for(draft.matrix_ref, draft.matrix_revision)
            .await?;
        let mut expected: Vec<Uuid> = reports.iter().map(|r| r.report_id).collect();
        let mut given = draft.report_refs.to_vec();
        expected.sort();
        given.sort();
        if expected != given {
            return Err(StoreError::ValidationFailed(
                "the proposal must cite exactly the two barrier-certified reports".into(),
            ));
        }
        // Law III.3 at consumption — the double-validation covenant: the
        // proposal is filed against reports that STILL validate, whatever
        // happened to them since the barrier.
        for report in &reports {
            self.validate_claims(&report.claims).await.map_err(|e| {
                StoreError::FlagUntrusted(format!(
                    "{}'s report no longer validates at proposal time: {e}",
                    report.auditor
                ))
            })?;
        }
        // A.11 shape rules, and every amendment resolves into membership.
        match draft.verdict {
            Verdict::Amend if draft.changes.is_empty() => {
                return Err(StoreError::ValidationFailed(
                    "AMEND requires enumerated changes (A.11)".into(),
                ))
            }
            Verdict::Commit | Verdict::Reject if !draft.changes.is_empty() => {
                return Err(StoreError::ValidationFailed(
                    "only AMEND carries changes (A.11)".into(),
                ))
            }
            Verdict::Reject if draft.reasons.is_empty() => {
                return Err(StoreError::ValidationFailed(
                    "REJECT requires reasons (A.11)".into(),
                ))
            }
            _ => {}
        }
        for change in &draft.changes {
            let kind = AmendmentKind::parse(&change.kind).map_err(StoreError::from_schema)?;
            let member = match kind {
                AmendmentKind::RemoveLink => matrix.link_refs.contains(&change.subject_ref),
                AmendmentKind::RemoveNode => matrix.node_refs.contains(&change.subject_ref),
            };
            if !member {
                return Err(StoreError::ValidationFailed(format!(
                    "amendment {} {} does not resolve into the matrix's membership",
                    change.kind, change.subject_ref
                )));
            }
        }
        let row = sqlx::query(
            r#"INSERT INTO joint_proposals
                 (proposal_id, job_id, matrix_ref, matrix_revision, report_refs, verdict,
                  changes, reasons, schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'JointProposal', $9, $2::text)
               RETURNING *"#,
        )
        .bind(Uuid::now_v7())
        .bind(job_id)
        .bind(draft.matrix_ref)
        .bind(draft.matrix_revision)
        .bind(
            serde_json::to_value(
                draft
                    .report_refs
                    .iter()
                    .map(Uuid::to_string)
                    .collect::<Vec<_>>(),
            )
            .expect("refs serialize"),
        )
        .bind(draft.verdict.as_str())
        .bind(serde_json::to_value(&draft.changes).expect("changes serialize"))
        .bind(serde_json::to_value(&draft.reasons).expect("reasons serialize"))
        .bind(RECORD_SCHEMA_VERSION)
        .fetch_one(&self.pool)
        .await?;
        self.append_log(
            &draft.matrix_ref.to_string(),
            LogEvent::ProposalFiled,
            &serde_json::json!({
                "verdict": draft.verdict.as_str(),
                "changes": draft.changes.len(),
                "matrix_revision": draft.matrix_revision,
                "audit_depth": matrix.audit_depth,
            }),
            Severity::Info,
            &job_id.to_string(),
        )
        .await?;
        Self::proposal_from_row(&row)
    }

    async fn get_proposal(&self, proposal_id: Uuid) -> Result<JointProposal, StoreError> {
        let row = sqlx::query("SELECT * FROM joint_proposals WHERE proposal_id = $1")
            .bind(proposal_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StoreError::NotFound(format!("no such proposal {proposal_id}")))?;
        Self::proposal_from_row(&row)
    }

    async fn resolve_proposal(
        &self,
        actor: &str,
        proposal_id: Uuid,
        decision: ConsentDecision,
    ) -> Result<JointProposal, StoreError> {
        let proposal = self.get_proposal(proposal_id).await?;
        if !matches!(
            decision,
            ConsentDecision::Granted | ConsentDecision::Declined
        ) {
            return Err(StoreError::ValidationFailed(format!(
                "{decision} is not an answer to a proposal; consent or decline (Book II §2)"
            )));
        }
        let decision_text = match decision {
            ConsentDecision::Granted => "GRANTED",
            _ => "DECLINED",
        };
        // One transaction, one arbiter: the guarded UPDATE is the atomic
        // check-and-claim — of N racing answers exactly one lands, and the
        // losers' consent inserts roll back with them. The sovereign
        // speaks once, mechanically (backed by the set-once trigger).
        let consent_id = Uuid::now_v7();
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"INSERT INTO consent_records
                 (consent_id, subject_ref, decision, scope, decided_by,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, 'ITEM', $4, 'ConsentRecord', $5, $4)"#,
        )
        .bind(consent_id)
        .bind(proposal_id)
        .bind(decision_text)
        .bind(actor)
        .bind(RECORD_SCHEMA_VERSION)
        .execute(&mut *tx)
        .await?;
        let claimed = sqlx::query(
            r#"UPDATE joint_proposals SET consent_ref = $2
               WHERE proposal_id = $1 AND consent_ref IS NULL"#,
        )
        .bind(proposal_id)
        .bind(consent_id)
        .execute(&mut *tx)
        .await?;
        if claimed.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(StoreError::ValidationFailed(format!(
                "proposal {proposal_id} is already answered; the sovereign speaks once"
            )));
        }
        tx.commit().await?;
        // Declined → the Postulant stands; the decline is logged, and is
        // signal (Book II §2 step 5, and Law VI.4's sovereign halt).
        self.append_log(
            &proposal.matrix_ref.to_string(),
            LogEvent::ProposalResolved,
            &serde_json::json!({
                "proposal": proposal_id.to_string(),
                "decision": decision_text,
            }),
            Severity::Info,
            actor,
        )
        .await?;
        self.get_proposal(proposal_id).await
    }

    async fn execute_matrix_proposal(
        &self,
        notary_job_id: Uuid,
        proposal_id: Uuid,
    ) -> Result<MatrixRecord, StoreError> {
        let job = self
            .guard_actor(notary_job_id, "execute_matrix_proposal", false)
            .await?;
        if job.agent_type != AgentType::Notary {
            return Err(StoreError::ValidationFailed(format!(
                "only a summoned Notary executes consent, not {} (Book II §3)",
                job.agent_type
            )));
        }
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "consent is executed during WORK, not {} (Law I.1)",
                job.status
            )));
        }
        let proposal = self.get_proposal(proposal_id).await?;
        // The chain: proposal → consent, resolving and cross-referencing.
        let consent_id = proposal.consent_ref.ok_or_else(|| {
            StoreError::ValidationFailed(format!(
                "proposal {proposal_id} has no consent — the chain does not resolve (Law VI.3)"
            ))
        })?;
        let consent = self.get_consent(consent_id).await?;
        if consent.subject_ref != proposal_id {
            return Err(StoreError::ValidationFailed(format!(
                "consent {consent_id} does not answer proposal {proposal_id} — the chain does not cross-reference"
            )));
        }
        if consent.decision != ConsentDecision::Granted {
            return Err(StoreError::ValidationFailed(format!(
                "consent {consent_id} is {}, not GRANTED; there is nothing to execute",
                consent.decision
            )));
        }
        let matrix = self.get_matrix(proposal.matrix_ref).await?;
        // Idempotency (SC-D10): a retry finding the verdict applied converges.
        let already_applied = match proposal.verdict {
            Verdict::Commit => {
                matrix.status == MatrixStatus::Cardinal
                    && matrix.committed_proposal_ref == Some(proposal_id)
            }
            Verdict::Amend => matrix.revision > proposal.matrix_revision,
            Verdict::Reject => matrix.status == MatrixStatus::Dissolved,
        };
        if already_applied {
            return Ok(matrix);
        }
        // The verdict applies to a standing Postulant and nothing else:
        // a Cardinal is not re-tried, a dissolved matrix does not rise.
        if matrix.status != MatrixStatus::Postulant {
            return Err(StoreError::ValidationFailed(format!(
                "the verdict applies to a Postulant; matrix {} is {} (Law VI.3, VI.5)",
                matrix.matrix_id, matrix.status
            )));
        }
        if matrix.revision != proposal.matrix_revision {
            return Err(StoreError::ValidationFailed(format!(
                "proposal addresses revision {} but the matrix stands at {} — the world moved on (Law VII)",
                proposal.matrix_revision, matrix.revision
            )));
        }
        // One transaction: the state change and its provenance land
        // together or not at all — the mandatory record cannot be lost to
        // a crash between them (Law V.1).
        let mut tx = self.pool.begin().await?;
        let applied = match proposal.verdict {
            Verdict::Commit => {
                // The substrate's commitment-chain trigger re-validates the
                // whole chain beneath this statement (VI.3).
                sqlx::query(
                    r#"UPDATE matrices
                       SET status = 'CARDINAL', committed_proposal_ref = $2,
                           committed_consent_ref = $3, committed_at = now(),
                           revision = revision + 1
                       WHERE matrix_id = $1 AND revision = $4"#,
                )
                .bind(matrix.matrix_id)
                .bind(proposal_id)
                .bind(consent_id)
                .bind(matrix.revision)
                .execute(&mut *tx)
                .await?
            }
            Verdict::Amend => {
                // Exactly the enumerated changes — no more, no less (VI.4).
                let mut node_refs = matrix.node_refs.clone();
                let mut link_refs = matrix.link_refs.clone();
                for change in &proposal.changes {
                    let kind =
                        AmendmentKind::parse(&change.kind).map_err(StoreError::from_schema)?;
                    match kind {
                        AmendmentKind::RemoveLink => link_refs.retain(|r| *r != change.subject_ref),
                        AmendmentKind::RemoveNode => node_refs.retain(|r| *r != change.subject_ref),
                    }
                }
                let node_refs: Vec<String> = node_refs.iter().map(Uuid::to_string).collect();
                let link_refs: Vec<String> = link_refs.iter().map(Uuid::to_string).collect();
                sqlx::query(
                    r#"UPDATE matrices
                       SET node_refs = $2, link_refs = $3,
                           revision = revision + 1, audit_depth = audit_depth + 1
                       WHERE matrix_id = $1 AND revision = $4"#,
                )
                .bind(matrix.matrix_id)
                .bind(serde_json::to_value(&node_refs).expect("refs serialize"))
                .bind(serde_json::to_value(&link_refs).expect("refs serialize"))
                .bind(matrix.revision)
                .execute(&mut *tx)
                .await?
            }
            Verdict::Reject => {
                // The trial failed and the sovereign confirmed it: the
                // Postulant dissolves; its links persist untouched (VI.5).
                sqlx::query(
                    r#"UPDATE matrices SET status = 'DISSOLVED', revision = revision + 1
                       WHERE matrix_id = $1 AND revision = $2"#,
                )
                .bind(matrix.matrix_id)
                .bind(matrix.revision)
                .execute(&mut *tx)
                .await?
            }
        };
        if applied.rows_affected() == 0 {
            return Err(StoreError::StaleRevision {
                expected: matrix.revision,
                subject: format!("matrix:{}", matrix.matrix_id),
            });
        }
        let (event, depth) = match proposal.verdict {
            Verdict::Commit => (LogEvent::Committed, matrix.audit_depth),
            Verdict::Amend => (LogEvent::Amended, matrix.audit_depth + 1),
            Verdict::Reject => (LogEvent::Decommissioned, matrix.audit_depth),
        };
        // Provenance linking every reference (SC-D10): proposal, consent,
        // executor, depth — the loop closes on the record, atomically.
        let subject = matrix.matrix_id.to_string();
        sqlx::query(
            r#"INSERT INTO log_snapshots
                 (log_id, subject_ref, event, payload, prior_ref, severity,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4,
                 (SELECT log_id FROM log_snapshots WHERE subject_ref = $2
                  ORDER BY seq DESC LIMIT 1),
                 'info', 'LogSnapshot', $5, $6::text)"#,
        )
        .bind(Uuid::now_v7())
        .bind(&subject)
        .bind(event.as_str())
        .bind(serde_json::json!({
            "proposal": proposal_id.to_string(),
            "consent": consent_id.to_string(),
            "executed_by_job": notary_job_id.to_string(),
            "verdict": proposal.verdict.as_str(),
            "audit_depth": depth,
        }))
        .bind(RECORD_SCHEMA_VERSION)
        .bind(notary_job_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        self.get_matrix(matrix.matrix_id).await
    }

    async fn consent_decommission(&self, actor: &str, matrix_id: Uuid) -> Result<Uuid, StoreError> {
        let matrix = self.get_matrix(matrix_id).await?;
        if matrix.status != MatrixStatus::Cardinal {
            return Err(StoreError::ValidationFailed(format!(
                "decommission reverses commitment; matrix {matrix_id} is {} (Law VI.5)",
                matrix.status
            )));
        }
        let consent_id = Uuid::now_v7();
        sqlx::query(
            r#"INSERT INTO consent_records
                 (consent_id, subject_ref, decision, scope, decided_by,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, 'GRANTED', 'ITEM', $3, 'ConsentRecord', $4, $3)"#,
        )
        .bind(consent_id)
        .bind(matrix_id)
        .bind(actor)
        .bind(RECORD_SCHEMA_VERSION)
        .execute(&self.pool)
        .await?;
        Ok(consent_id)
    }

    async fn execute_decommission(
        &self,
        notary_job_id: Uuid,
        matrix_id: Uuid,
        consent_id: Uuid,
    ) -> Result<MatrixRecord, StoreError> {
        let job = self
            .guard_actor(notary_job_id, "execute_decommission", false)
            .await?;
        if job.agent_type != AgentType::Notary {
            return Err(StoreError::ValidationFailed(format!(
                "only a summoned Notary executes consent, not {} (Book II §3)",
                job.agent_type
            )));
        }
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "consent is executed during WORK, not {} (Law I.1)",
                job.status
            )));
        }
        let consent = self.get_consent(consent_id).await?;
        if consent.subject_ref != matrix_id || consent.decision != ConsentDecision::Granted {
            return Err(StoreError::ValidationFailed(format!(
                "consent {consent_id} does not grant the decommission of matrix {matrix_id}"
            )));
        }
        let matrix = self.get_matrix(matrix_id).await?;
        if matrix.status == MatrixStatus::Dissolved {
            return Ok(matrix); // idempotent retry converges
        }
        // One transaction: dissolution and its record land together.
        let mut tx = self.pool.begin().await?;
        let applied = sqlx::query(
            r#"UPDATE matrices SET status = 'DISSOLVED', revision = revision + 1
               WHERE matrix_id = $1 AND revision = $2"#,
        )
        .bind(matrix_id)
        .bind(matrix.revision)
        .execute(&mut *tx)
        .await?;
        if applied.rows_affected() == 0 {
            return Err(StoreError::StaleRevision {
                expected: matrix.revision,
                subject: format!("matrix:{matrix_id}"),
            });
        }
        let subject = matrix_id.to_string();
        sqlx::query(
            r#"INSERT INTO log_snapshots
                 (log_id, subject_ref, event, payload, prior_ref, severity,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, 'DECOMMISSIONED', $3,
                 (SELECT log_id FROM log_snapshots WHERE subject_ref = $2
                  ORDER BY seq DESC LIMIT 1),
                 'info', 'LogSnapshot', $4, $5::text)"#,
        )
        .bind(Uuid::now_v7())
        .bind(&subject)
        .bind(serde_json::json!({
            "consent": consent_id.to_string(),
            "executed_by_job": notary_job_id.to_string(),
        }))
        .bind(RECORD_SCHEMA_VERSION)
        .bind(notary_job_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        self.get_matrix(matrix_id).await
    }
}
