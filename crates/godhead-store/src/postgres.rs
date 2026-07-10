use crate::error::StoreError;
use crate::interface::Store;
use crate::secrets;
use crate::types::{ArtifactDraft, ArtifactRecord, ComplianceMetrics};
use godhead_schemas::{
    roman_ordinal, roster_index, validate_mandate_shape, AgentType, AmendmentKind, AuditReport,
    AuditReportDraft, AuditorKind, AuditorName, Budgets, ChainEntry, ChainEntryDraft,
    ChainEntryKind, Claim, ConcordatArtifact, ConfigConstant, ConfigTier, ConsentDecision,
    ConsentRecord, ConsentScope, EmbeddingRecord, EnvItem, EnvKind, EnvStatus, Envelope,
    EnvironmentRecord, FlagDraft, FlagStatus, InstructionDraft, InstructionRecord, IntakeStatus,
    JobDraft, JobRecord, JobStatus, JointProposal, Law, LeaseRecord, LinkRecord, LiveWeights,
    Locator, LogEvent, LogSnapshot, MandateDemands, MandateDraft, MandateKind, MandateRecord,
    Manifest, MatrixRecord, MatrixStatus, NodeDraft, NodeRecord, NormalizeOutcome, OverrideBasis,
    OverrideKind, OverrideRecord, PairingKind, PairingRecord, PetitionDraft, PetitionRecord,
    PetitionStatus, ProposalDraft, QuarantineDraft, QuarantineItem, ReadinessFlag, RebalanceState,
    RefinedArtifact, RefusalDraft, RefusalReason, RefusalRecord, ReportKind, ReturnDraft,
    ReturnManifest, ScanEngine, ScanVerdict, ScanVerdictKind, SchemaRegistry, Severity, SourceDraw,
    Tier, Verdict, WritTarget, RECORD_SCHEMA_VERSION,
};
use semver::Version;
use sha2::{Digest, Sha256};
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Row;
use uuid::Uuid;

/// Ruling G10 — the actor classes the reserved-table triggers verify. The
/// class strings are constants of this file, never caller input: the
/// credential lives only in the code path that is the lawful surface.
const SOVEREIGN_CLASS: &str = "sovereign";
const DEACON_CLASS: &str = "office:deacon";
/// The one standing functionary's write identity (A.1: produced_by is a
/// job ref or an office id).
pub const DEACON_OFFICE_ID: &str = "office:deacon";

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

    fn environment_from_row(row: &PgRow) -> Result<EnvironmentRecord, StoreError> {
        let kind: String = row.try_get("kind")?;
        let tier: String = row.try_get("tier")?;
        let status: String = row.try_get("status")?;
        Ok(EnvironmentRecord {
            env_id: row.try_get("env_id")?,
            kind: EnvKind::parse(&kind).map_err(StoreError::from_schema)?,
            matrix_ref: row.try_get("matrix_ref")?,
            tier: Tier::parse(&tier).map_err(StoreError::from_schema)?,
            title: row.try_get("title")?,
            name: row.try_get("name")?,
            established_by: row.try_get("established_by")?,
            established_at: row.try_get("established_at")?,
            status: EnvStatus::parse(&status).map_err(StoreError::from_schema)?,
            revision: row.try_get("revision")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn env_item_from_row(row: &PgRow) -> Result<EnvItem, StoreError> {
        Ok(EnvItem {
            env_id: row.try_get("env_id")?,
            item_ref: row.try_get("item_ref")?,
            provenance: row.try_get("provenance")?,
            flagged: row.try_get("flagged")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn pairing_from_row(row: &PgRow) -> Result<PairingRecord, StoreError> {
        let kind: String = row.try_get("kind")?;
        Ok(PairingRecord {
            pairing_id: row.try_get("pairing_id")?,
            kind: PairingKind::parse(&kind).map_err(StoreError::from_schema)?,
            teacher_env_ref: row.try_get("teacher_env_ref")?,
            student_env_ref: row.try_get("student_env_ref")?,
            matrix_ref: row.try_get("matrix_ref")?,
            formed_at: row.try_get("formed_at")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn instruction_from_row(row: &PgRow) -> Result<InstructionRecord, StoreError> {
        let teacher_tier: String = row.try_get("teacher_tier")?;
        let target_tier: String = row.try_get("target_tier")?;
        let concordat_version: String = row.try_get("concordat_version")?;
        Ok(InstructionRecord {
            instruction_id: row.try_get("instruction_id")?,
            teacher_env_ref: row.try_get("teacher_env_ref")?,
            teacher_tier: Tier::parse(&teacher_tier).map_err(StoreError::from_schema)?,
            target_tier: Tier::parse(&target_tier).map_err(StoreError::from_schema)?,
            concordat_version: Version::parse(&concordat_version).map_err(|e| {
                StoreError::ValidationFailed(format!("stored concordat_version: {e}"))
            })?,
            objective: row.try_get("objective")?,
            steps: row.try_get("steps")?,
            acceptance_criteria: row.try_get("acceptance_criteria")?,
            sources_drawn: row.try_get("sources_drawn")?,
            skew: row.try_get("skew")?,
            supersedes_ref: row.try_get("supersedes_ref")?,
            flagged: row.try_get("flagged")?,
            content_sha: row.try_get("content_sha")?,
            revision: row.try_get("revision")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn concordat_from_row(row: &PgRow) -> Result<ConcordatArtifact, StoreError> {
        let version: String = row.try_get("version")?;
        Ok(ConcordatArtifact {
            version: Version::parse(&version).map_err(|e| {
                StoreError::ValidationFailed(format!("stored concordat version: {e}"))
            })?,
            capability_tables: row.try_get("capability_tables")?,
            pairing_semantics: row.try_get("pairing_semantics")?,
            adopted_at: row.try_get("adopted_at")?,
            adopted_by: row.try_get("adopted_by")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    /// The mount-time provenance walk (Handbook §2.2, SC-G06): a chain must
    /// be non-empty, rooted in a human hand (CANON|WRIT|BRIEF), contiguous
    /// in `link_seq`, and every `produced` ref must resolve to a live
    /// record. A room that cannot explain itself is invalid.
    async fn validate_provenance_chain(&self, chain: &serde_json::Value) -> Result<(), StoreError> {
        let entries = chain.as_array().ok_or_else(|| {
            StoreError::EnvInvalid("provenance chain must be an array (C.2)".into())
        })?;
        if entries.is_empty() {
            return Err(StoreError::EnvInvalid(
                "an item without provenance has no arrival story (IX.2)".into(),
            ));
        }
        // Collect (link_seq, kind) so the human-root check keys on the
        // actual root — the minimum link_seq — not array position, which
        // an out-of-order chain could otherwise use to slip a non-human
        // root past the floor.
        let mut ordered: Vec<(i64, String)> = Vec::with_capacity(entries.len());
        for entry in entries {
            let obj = entry.as_object().ok_or_else(|| {
                StoreError::EnvInvalid("each chain entry must be an object".into())
            })?;
            let seq = obj
                .get("link_seq")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| {
                    StoreError::EnvInvalid("each chain entry needs an integer link_seq".into())
                })?;
            // Every entry carries an actor — the arrival's author (C.2).
            let actor = obj.get("actor").and_then(|v| v.as_str());
            if actor.is_none_or(str::is_empty) {
                return Err(StoreError::EnvInvalid(
                    "each chain entry needs a non-empty actor (C.2)".into(),
                ));
            }
            let kind = obj
                .get("kind")
                .and_then(|v| v.as_str())
                .ok_or_else(|| StoreError::EnvInvalid("each chain entry needs a kind".into()))?
                .to_string();
            // Every produced ref resolves to a live record.
            if let Some(produced) = obj.get("produced").and_then(|v| v.as_array()) {
                for reference in produced {
                    let id = reference
                        .as_str()
                        .and_then(|s| Uuid::parse_str(s).ok())
                        .ok_or_else(|| {
                            StoreError::EnvInvalid("a produced ref is not a uuid".into())
                        })?;
                    let resolves: bool = sqlx::query_scalar(
                        r#"SELECT EXISTS(SELECT 1 FROM nodes WHERE node_id = $1)
                            OR EXISTS(SELECT 1 FROM links WHERE link_id = $1)
                            OR EXISTS(SELECT 1 FROM matrices WHERE matrix_id = $1)"#,
                    )
                    .bind(id)
                    .fetch_one(&self.pool)
                    .await?;
                    if !resolves {
                        return Err(StoreError::EnvInvalid(format!(
                            "chain entry produces {id}, which resolves to no live record"
                        )));
                    }
                }
            }
            ordered.push((seq, kind));
        }
        ordered.sort_by_key(|(seq, _)| *seq);
        // The root — the entry with the least link_seq — begins in a human
        // hand (CANON|WRIT|BRIEF).
        let root_kind = ordered.first().expect("chain is non-empty").1.as_str();
        if !matches!(root_kind, "CANON" | "WRIT" | "BRIEF") {
            return Err(StoreError::EnvInvalid(format!(
                "the chain's root is '{root_kind}'; every chain begins in a human hand (CANON|WRIT|BRIEF)"
            )));
        }
        // Contiguous root-to-leaf: sorted seqs increase by exactly one.
        for pair in ordered.windows(2) {
            if pair[0].0 == pair[1].0 {
                return Err(StoreError::EnvInvalid(
                    "chain link_seq values are not distinct".into(),
                ));
            }
            if pair[1].0 != pair[0].0 + 1 {
                return Err(StoreError::EnvInvalid(
                    "the chain does not walk root-to-leaf: a gap in link_seq".into(),
                ));
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

    fn return_from_row(row: &PgRow) -> Result<ReturnManifest, StoreError> {
        let concordat_version: String = row.try_get("concordat_version")?;
        Ok(ReturnManifest {
            return_id: row.try_get("return_id")?,
            instruction_ref: row.try_get("instruction_ref")?,
            student_env_ref: row.try_get("student_env_ref")?,
            concordat_version: Version::parse(&concordat_version).map_err(|e| {
                StoreError::ValidationFailed(format!("stored concordat_version: {e}"))
            })?,
            items: row.try_get("items")?,
            completion: row.try_get("completion")?,
            flagged: row.try_get("flagged")?,
            content_sha: row.try_get("content_sha")?,
            revision: row.try_get("revision")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn refined_artifact_from_row(row: &PgRow) -> Result<RefinedArtifact, StoreError> {
        let source_refs: serde_json::Value = row.try_get("source_refs")?;
        Ok(RefinedArtifact {
            artifact_id: row.try_get("artifact_id")?,
            env_ref: row.try_get("env_ref")?,
            source_refs: Self::uuid_vec(&source_refs, "source_refs")?,
            method: row.try_get("method")?,
            content_sha: row.try_get("content_sha")?,
            revision: row.try_get("revision")?,
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

    /// Ruling G10: authenticate this transaction's actor class. `SET LOCAL`
    /// is transaction-scoped; below the API the variable is absent, so
    /// 'deacon', 'sovereign', and 'forged' are all rejected alike by the
    /// reserved-table triggers.
    async fn set_actor_class(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        class: &'static str,
    ) -> Result<(), StoreError> {
        // SET LOCAL takes no bind parameters; `class` is one of this file's
        // constants, never caller input.
        sqlx::query(&format!("SET LOCAL godhead.actor_class = '{class}'"))
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    /// Canonical JSON: object keys sorted, no whitespace — deterministic
    /// regardless of serde feature unification, so a hash computed at FLAG
    /// re-proves at every read (ruling G7).
    fn canonical_json(value: &serde_json::Value, out: &mut String) {
        match value {
            serde_json::Value::Object(map) => {
                out.push('{');
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                for (i, key) in keys.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    out.push_str(&serde_json::Value::String((*key).clone()).to_string());
                    out.push(':');
                    Self::canonical_json(&map[*key], out);
                }
                out.push('}');
            }
            serde_json::Value::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    Self::canonical_json(item, out);
                }
                out.push(']');
            }
            leaf => out.push_str(&leaf.to_string()),
        }
    }

    fn sha256_of_canonical(body: &serde_json::Value) -> String {
        let mut text = String::new();
        Self::canonical_json(body, &mut text);
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let digest = hasher.finalize();
        let mut hex = String::with_capacity(64);
        for byte in digest {
            use std::fmt::Write;
            write!(hex, "{byte:02x}").expect("writing hex to a String cannot fail");
        }
        hex
    }

    /// The canonical body of an Instruction (ruling G7): every field the
    /// certification covers, in one deterministic shape. Time-varying
    /// clauses stay with the reconstruction path; the hash answers bytes.
    fn instruction_body(rec: &InstructionRecord) -> serde_json::Value {
        serde_json::json!({
            "objective": rec.objective,
            "teacher_env_ref": rec.teacher_env_ref.map(|u| u.to_string()),
            "teacher_tier": rec.teacher_tier.as_str(),
            "target_tier": rec.target_tier.as_str(),
            "concordat_version": rec.concordat_version.to_string(),
            "steps": rec.steps,
            "acceptance_criteria": rec.acceptance_criteria,
            "sources_drawn": rec.sources_drawn,
            "skew": rec.skew,
            "supersedes_ref": rec.supersedes_ref.map(|u| u.to_string()),
        })
    }

    /// The canonical body of a ReturnManifest (ruling G7).
    fn return_body(rec: &ReturnManifest) -> serde_json::Value {
        serde_json::json!({
            "instruction_ref": rec.instruction_ref.to_string(),
            "student_env_ref": rec.student_env_ref.to_string(),
            "concordat_version": rec.concordat_version.to_string(),
            "items": rec.items,
            "completion": rec.completion,
        })
    }

    /// Ruling G7's read half: a flagged record whose stored hash no longer
    /// re-proves is byte-corrupt — SCHEMA_MISMATCH, never a best-effort
    /// read. Records flagged before the hash era (no stored hash) pass
    /// through: their integrity story is the reconstruction path's.
    fn prove_content_sha(
        kind: &str,
        id: Uuid,
        flagged: bool,
        stored: Option<&str>,
        body: &serde_json::Value,
    ) -> Result<(), StoreError> {
        if !flagged {
            return Ok(());
        }
        let Some(stored) = stored else {
            return Ok(());
        };
        let computed = Self::sha256_of_canonical(body);
        if computed != stored {
            return Err(StoreError::SchemaMismatch(format!(
                "the flagged {kind} {id} does not re-prove its content hash; \
                 byte-integrity is broken between flag and read (ruling G7, SC-K04)"
            )));
        }
        Ok(())
    }

    /// H3(2) — write-side config contracts: per-key type + semantic floor,
    /// registered here beside the schema registry, enforced at write.
    /// Read-side refusals remain as depth. A key without a contract is
    /// accepted as-is (its contract arrives when its consumer does).
    fn config_contract(key: &str, value: &serde_json::Value) -> Result<(), String> {
        fn int_at_least(value: &serde_json::Value, floor: i64) -> Result<(), String> {
            match value.as_i64() {
                Some(n) if n >= floor => Ok(()),
                Some(n) => Err(format!("must be an integer >= {floor}, got {n}")),
                None => Err("must be an integer".into()),
            }
        }
        fn unit_interval(value: &serde_json::Value) -> Result<(), String> {
            match value.as_f64() {
                Some(x) if (0.0..=1.0).contains(&x) => Ok(()),
                Some(x) => Err(format!("must be a number in [0, 1], got {x}")),
                None => Err("must be a number".into()),
            }
        }
        fn string_array(value: &serde_json::Value, non_empty: bool) -> Result<(), String> {
            let Some(items) = value.as_array() else {
                return Err("must be an array of strings".into());
            };
            if non_empty && items.is_empty() {
                return Err("must be a non-empty array".into());
            }
            if items
                .iter()
                .any(|v| v.as_str().is_none_or(|s| s.trim().is_empty()))
            {
                return Err("every entry must be a non-empty string".into());
            }
            Ok(())
        }
        // honorific_set is NOT a flat string array (that was the bug the Slice
        // 11 opening round caught): it is the nested shape the conferral +
        // mount paths read — `{"teacher": {TIER: title, …}, "student":
        // [honorific, …]}` (seed migration 0007). The contract validates that
        // structure, so the value the store admits is exactly the value
        // mount_environment can read.
        fn honorific_shape(value: &serde_json::Value) -> Result<(), String> {
            let Some(obj) = value.as_object() else {
                return Err("must be an object with 'teacher' and 'student'".into());
            };
            let Some(teacher) = obj.get("teacher").and_then(|t| t.as_object()) else {
                return Err("must carry a 'teacher' object of tier -> title".into());
            };
            if teacher.is_empty()
                || teacher
                    .values()
                    .any(|v| v.as_str().is_none_or(|s| s.trim().is_empty()))
            {
                return Err("every 'teacher' title must be a non-empty string".into());
            }
            let Some(student) = obj.get("student").and_then(|s| s.as_array()) else {
                return Err("must carry a 'student' array of honorifics".into());
            };
            if student.is_empty()
                || student
                    .iter()
                    .any(|v| v.as_str().is_none_or(|s| s.trim().is_empty()))
            {
                return Err("'student' must be a non-empty array of non-empty strings".into());
            }
            Ok(())
        }
        let result = match key {
            "bias_pattern_window" => int_at_least(value, 1),
            "bias_skew_threshold" | "bias_pattern_threshold" | "coherence_threshold" => {
                unit_interval(value)
            }
            "petition_stall_ms"
            | "lease_ttl_ms"
            | "admission_batch_threshold"
            | "admission_rate_window_ms"
            | "admission_rate_threshold"
            | "quarantine_retention_days" => int_at_least(value, 1),
            "tool_repair_attempts" => int_at_least(value, 0),
            "name_roster" => string_array(value, true),
            "honorific_set" => honorific_shape(value),
            "known_source_ids" => string_array(value, false),
            _ => Ok(()),
        };
        result.map_err(|why| format!("config '{key}' {why} (write-side contract, H3(2))"))
    }

    // ---- threshold & J-floor row decoding ----

    fn mandate_from_row(row: &PgRow) -> Result<MandateRecord, StoreError> {
        Ok(MandateRecord {
            mandate_id: row.try_get("mandate_id")?,
            kind: MandateKind::parse(row.try_get::<String, _>("kind")?.as_str())
                .map_err(StoreError::from_schema)?,
            teacher_env_ref: row.try_get("teacher_env_ref")?,
            matrix_ref: row.try_get("matrix_ref")?,
            demands: row.try_get("demands")?,
            sources: row.try_get("sources")?,
            trip_budget: row.try_get("trip_budget")?,
            authored_at: row.try_get("authored_at")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn chain_from_row(row: &PgRow) -> Result<ChainEntry, StoreError> {
        let produced: serde_json::Value = row.try_get("produced")?;
        Ok(ChainEntry {
            chain_ref: row.try_get("chain_ref")?,
            link_seq: row.try_get("link_seq")?,
            kind: ChainEntryKind::parse(row.try_get::<String, _>("kind")?.as_str())
                .map_err(StoreError::from_schema)?,
            actor_job_ref: row.try_get("actor_job_ref")?,
            mandate_ref: row.try_get("mandate_ref")?,
            prompt_or_reason: row.try_get("prompt_or_reason")?,
            produced: Self::uuid_vec(&produced, "produced")?,
            at: row.try_get("at")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn quarantine_from_row(row: &PgRow) -> Result<QuarantineItem, StoreError> {
        Ok(QuarantineItem {
            item_ref: row.try_get("item_ref")?,
            origin_job_ref: row.try_get("origin_job_ref")?,
            mandate_ref: row.try_get("mandate_ref")?,
            brief_ref: row.try_get("brief_ref")?,
            filename: row.try_get("filename")?,
            declared_type: row.try_get("declared_type")?,
            content: row.try_get("content")?,
            scan_ref: row.try_get("scan_ref")?,
            consent_ref: row.try_get("consent_ref")?,
            admitted_node_ref: row.try_get("admitted_node_ref")?,
            held_since: row.try_get("held_since")?,
            revision: row.try_get("revision")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn verdict_from_row(row: &PgRow) -> Result<ScanVerdict, StoreError> {
        Ok(ScanVerdict {
            scan_id: row.try_get("scan_id")?,
            item_ref: row.try_get("item_ref")?,
            verdict: ScanVerdictKind::parse(row.try_get::<String, _>("verdict")?.as_str())
                .map_err(StoreError::from_schema)?,
            engine: ScanEngine {
                alias: row.try_get("engine_alias")?,
                version: row.try_get("engine_version")?,
                signature_rev: row.try_get("signature_rev")?,
            },
            scanned_at: row.try_get("scanned_at")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    fn manifest_from_row(row: &PgRow) -> Result<Manifest, StoreError> {
        Ok(Manifest {
            manifest_id: row.try_get("manifest_id")?,
            mandate_ref: row.try_get("mandate_ref")?,
            trip_job_ref: row.try_get("trip_job_ref")?,
            items: row.try_get("items")?,
            standing_notice: row.try_get("standing_notice")?,
            presented_at: row.try_get("presented_at")?,
            envelope: Self::envelope_from_row(row)?,
        })
    }

    /// The admission conjunction (SC-I02), proven wherever admission is
    /// decided: the item's LATEST verdict is CLEAN, a resolving consent
    /// says ADMITTED over exactly this item (or a Manifest batch holding
    /// it), and the consent names the very scan it saw — a newer, darker
    /// verdict defeats a stale consent. Only CLEAN + ADMITTED passes;
    /// INFECTED, SUSPECT, and ERROR are never admissible (Book II §1).
    async fn prove_admission_conjunction(&self, item: &QuarantineItem) -> Result<(), StoreError> {
        let latest = self.latest_verdict(item.item_ref).await?.ok_or_else(|| {
            StoreError::ValidationFailed(format!(
                "item {} is unscanned; the Deacon never admits the unscanned (Book II §1)",
                item.item_ref
            ))
        })?;
        if latest.verdict != ScanVerdictKind::Clean {
            return Err(StoreError::ValidationFailed(format!(
                "item {} stands {}; INFECTED, SUSPECT, and ERROR are never admissible (SC-I02)",
                item.item_ref, latest.verdict
            )));
        }
        let consent_id = item.consent_ref.ok_or_else(|| {
            StoreError::ValidationFailed(format!(
                "item {} has no consent; the Deacon never admits alone (Book II §1)",
                item.item_ref
            ))
        })?;
        let consent = self.get_consent(consent_id).await?;
        if consent.decision != ConsentDecision::Admitted {
            return Err(StoreError::ValidationFailed(format!(
                "consent {consent_id} says {}; only ADMITTED admits (SC-I02)",
                consent.decision
            )));
        }
        // The consent binds the scan it saw.
        let consented_scan: Option<Uuid> =
            sqlx::query_scalar("SELECT scan_ref FROM consent_records WHERE consent_id = $1")
                .bind(consent_id)
                .fetch_one(&self.pool)
                .await?;
        match consent.scope {
            ConsentScope::Item => {
                if consent.subject_ref != item.item_ref {
                    return Err(StoreError::ValidationFailed(format!(
                        "consent {consent_id} was given over {}, not item {} (SC-I02)",
                        consent.subject_ref, item.item_ref
                    )));
                }
                if consented_scan != Some(latest.scan_id) {
                    return Err(StoreError::ValidationFailed(format!(
                        "consent {consent_id} saw a different scan than the item's latest; \
                         a stale consent admits nothing (Book II §1)"
                    )));
                }
            }
            ConsentScope::Batch => {
                let manifest = self.get_manifest(consent.subject_ref).await?;
                let listed_scan = manifest
                    .items
                    .as_array()
                    .and_then(|items| {
                        items.iter().find(|entry| {
                            entry.get("item_ref").and_then(|v| v.as_str())
                                == Some(item.item_ref.to_string()).as_deref()
                        })
                    })
                    .and_then(|entry| entry.get("scan_id").and_then(|v| v.as_str()))
                    .and_then(|s| Uuid::parse_str(s).ok());
                if listed_scan != Some(latest.scan_id) {
                    return Err(StoreError::ValidationFailed(format!(
                        "the batch consent's Manifest does not carry item {} under its \
                         latest scan; a stale or absent listing admits nothing (Book II §1)",
                        item.item_ref
                    )));
                }
            }
        }
        Ok(())
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
        // Law IX.4: a binding is not self-declared. If a job is born bound
        // to an environment, the store authenticates the binding against
        // store-verified relationships — the env must exist and be LIVE,
        // its tier must match the job's tier, and its matrix must be one
        // the job is actually working (in input_refs). An agent cannot
        // enter a room of another tier or another matrix by naming it.
        if let Some(env_id) = draft.env_ref {
            let env = self.get_environment(env_id).await?;
            if env.status != EnvStatus::Live {
                return Err(StoreError::ValidationFailed(format!(
                    "cannot bind to environment {env_id}: it is {} (a non-LIVE room is no workplace)",
                    env.status
                )));
            }
            if draft.tier != Some(env.tier) {
                return Err(StoreError::ValidationFailed(format!(
                    "cannot bind to environment {env_id}: its tier {} does not match the job's tier {:?} (Law IX.4)",
                    env.tier, draft.tier
                )));
            }
            if !draft.input_refs.contains(&env.matrix_ref) {
                return Err(StoreError::ValidationFailed(format!(
                    "cannot bind to environment {env_id}: its matrix {} is not among the job's inputs (Law IX.4)",
                    env.matrix_ref
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
        // XIII.1 means what it says (H3(3)): the releasing path carries an
        // authenticated identity. Post-FLAG release is lawful — it is part
        // of termination — so the terminal permit rides along.
        self.guard_actor(job_id, "release_lease", true).await?;
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
        // H3(2) — the write-side contract: structural prevention over
        // policy. A malformed constant never lands; the read-side refusals
        // remain as depth.
        Self::config_contract(key, value).map_err(StoreError::ValidationFailed)?;
        // Ruling G10: config is human/deployment administration — the
        // single-statement path gains its transaction so the class
        // credential can ride it (H6(e)'s costed case).
        let mut tx = self.pool.begin().await?;
        Self::set_actor_class(&mut tx, SOVEREIGN_CLASS).await?;
        let result = match expected_revision {
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
                .fetch_optional(&mut *tx)
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
                .fetch_optional(&mut *tx)
                .await?;
                match row {
                    Some(row) => Self::config_from_row(&row),
                    None => Err(StoreError::StaleRevision {
                        expected,
                        subject: format!("config:{key}"),
                    }),
                }
            }
        };
        match result {
            Ok(config) => {
                tx.commit().await?;
                Ok(config)
            }
            Err(StoreError::StaleRevision { expected, subject }) => {
                tx.rollback().await?;
                self.get_config(key).await?; // NotFound if the key is absent
                Err(StoreError::StaleRevision { expected, subject })
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
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
        Self::set_actor_class(&mut tx, SOVEREIGN_CLASS).await?;
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
        // One transaction: the consent and the answered petition land
        // together, under the sovereign's class (ruling G10).
        let mut tx = self.pool.begin().await?;
        Self::set_actor_class(&mut tx, SOVEREIGN_CLASS).await?;
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
                .execute(&mut *tx)
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
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
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
        // The successor override is a consent-authorized write: the
        // authority is the consent — the sovereign's act, executed by
        // hands that hold no judgment — so the transaction bears the
        // sovereign's class (ruling G10; IV.5).
        Self::set_actor_class(&mut tx, SOVEREIGN_CLASS).await?;
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

    async fn get_link(&self, link_id: Uuid) -> Result<LinkRecord, StoreError> {
        let row = sqlx::query("SELECT * FROM links WHERE link_id = $1")
            .bind(link_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StoreError::NotFound(format!("no such link {link_id}")))?;
        Self::link_from_row(&row)
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
        Self::set_actor_class(&mut tx, SOVEREIGN_CLASS).await?;
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
        let mut tx = self.pool.begin().await?;
        Self::set_actor_class(&mut tx, SOVEREIGN_CLASS).await?;
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
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
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

    async fn establish_environment(
        &self,
        job_id: Uuid,
        kind: EnvKind,
        matrix_ref: Uuid,
        tier: Tier,
    ) -> Result<EnvironmentRecord, StoreError> {
        let job = self
            .guard_actor(job_id, "establish_environment", false)
            .await?;
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "an environment is established during WORK, not {} (Law I.1)",
                job.status
            )));
        }
        // X.1: the unbound establish nothing; identity is earned by binding.
        if tier == Tier::Regular {
            return Err(StoreError::ValidationFailed(
                "Regulars establish no environment; the unbound are unnamed (Law X.1)".into(),
            ));
        }
        // The matrix must exist (the room is built around it).
        self.get_matrix(matrix_ref).await?;
        let env_id = Uuid::now_v7();

        // Conferral (X.2, X.4): title from tier for Teachers, a flat
        // honorific by hash for Students; name from the roster by hash.
        let honorifics = self.get_config("honorific_set").await?;
        let title = match kind {
            EnvKind::Teacher => honorifics
                .value
                .get("teacher")
                .and_then(|t| t.get(tier.as_str()))
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    StoreError::ValidationFailed(format!(
                        "honorific_set has no teacher title for tier {tier}"
                    ))
                })?
                .to_string(),
            EnvKind::Student => {
                let set = honorifics
                    .value
                    .get("student")
                    .and_then(|v| v.as_array())
                    .filter(|a| !a.is_empty())
                    .ok_or_else(|| {
                        StoreError::ValidationFailed("honorific_set.student is empty".into())
                    })?;
                let idx = roster_index(env_id, set.len());
                set[idx]
                    .as_str()
                    .ok_or_else(|| {
                        StoreError::ValidationFailed(
                            "honorific_set.student entry not a string".into(),
                        )
                    })?
                    .to_string()
            }
        };
        let roster_cfg = self.get_config("name_roster").await?;
        let roster = roster_cfg
            .value
            .as_array()
            .filter(|a| !a.is_empty())
            .ok_or_else(|| StoreError::ValidationFailed("name_roster is empty".into()))?;
        let base = roster[roster_index(env_id, roster.len())]
            .as_str()
            .ok_or_else(|| StoreError::ValidationFailed("name_roster entry not a string".into()))?
            .to_string();
        // Ordinal (X.4): a name already borne takes the next. Counted over
        // ALL environments ever (records are never deleted — Law V.1), so
        // an ordinal is never reused even after a lower bearer orphans;
        // and matched by exact base or "base <ordinal>" with a space
        // delimiter, so a base that prefixes another name (Solo vs
        // Solomon) never over-counts. A living collision therefore always
        // yields ≥ 2 (X.4's letter); an all-archived base still advances,
        // which is the safer reading (no name reuse).
        let bearers: i64 = sqlx::query_scalar(
            r#"SELECT count(*) FROM environments
               WHERE name = $1 OR name LIKE $2"#,
        )
        .bind(&base)
        .bind(format!("{base} %"))
        .fetch_one(&self.pool)
        .await?;
        let ordinal = roman_ordinal(u32::try_from(bearers + 1).unwrap_or(1));
        let name = if ordinal.is_empty() {
            base
        } else {
            format!("{base} {ordinal}")
        };

        let row = sqlx::query(
            r#"INSERT INTO environments
                 (env_id, kind, matrix_ref, tier, title, name, established_by,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, 'EnvironmentRecord', $8, $7::text)
               RETURNING *"#,
        )
        .bind(env_id)
        .bind(kind.as_str())
        .bind(matrix_ref)
        .bind(tier.as_str())
        .bind(&title)
        .bind(&name)
        .bind(job_id)
        .bind(RECORD_SCHEMA_VERSION)
        .fetch_one(&self.pool)
        .await?;
        self.append_log(
            &env_id.to_string(),
            LogEvent::EnvEstablished,
            &serde_json::json!({
                "kind": kind.as_str(),
                "tier": tier.as_str(),
                "title": title,
                "name": name,
                "matrix": matrix_ref.to_string(),
            }),
            Severity::Info,
            &job_id.to_string(),
        )
        .await?;
        Self::environment_from_row(&row)
    }

    async fn get_environment(&self, env_id: Uuid) -> Result<EnvironmentRecord, StoreError> {
        let row = sqlx::query("SELECT * FROM environments WHERE env_id = $1")
            .bind(env_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StoreError::NotFound(format!("no such environment {env_id}")))?;
        Self::environment_from_row(&row)
    }

    async fn add_env_item(
        &self,
        job_id: Uuid,
        env_id: Uuid,
        item_ref: Uuid,
        provenance: &serde_json::Value,
        flagged: bool,
    ) -> Result<EnvItem, StoreError> {
        self.guard_actor(job_id, "add_env_item", false).await?;
        let env = self.get_environment(env_id).await?;
        // SC-G07: an ORPHANED/DISSOLVED room is not a workplace.
        if env.status != EnvStatus::Live {
            return Err(StoreError::ValidationFailed(format!(
                "environment {env_id} is {}; a non-LIVE room is a read-only archive (A.8)",
                env.status
            )));
        }
        // H3(4), XI.1 strict: an environment is a mutable subject; its
        // curation is written under its lease — acquire-or-refuse, no
        // waiting, no spinning. Released in the same breath; a failure in
        // between expires with the TTL (XI.2).
        let lease = self.acquire_lease(job_id, env_id, 60_000).await?;
        let written = sqlx::query(
            r#"INSERT INTO environment_items
                 (env_id, item_ref, provenance, flagged,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, 'EnvItem', $5, $6::text)
               ON CONFLICT (env_id, item_ref) DO UPDATE
               SET provenance = EXCLUDED.provenance, flagged = EXCLUDED.flagged
               RETURNING *"#,
        )
        .bind(env_id)
        .bind(item_ref)
        .bind(provenance)
        .bind(flagged)
        .bind(RECORD_SCHEMA_VERSION)
        .bind(job_id)
        .fetch_one(&self.pool)
        .await;
        let release = self.release_lease(job_id, lease.lease_id).await;
        let row = written?;
        release?;
        Self::env_item_from_row(&row)
    }

    async fn env_items(&self, env_id: Uuid) -> Result<Vec<EnvItem>, StoreError> {
        let rows =
            sqlx::query("SELECT * FROM environment_items WHERE env_id = $1 ORDER BY produced_at")
                .bind(env_id)
                .fetch_all(&self.pool)
                .await?;
        rows.iter().map(Self::env_item_from_row).collect()
    }

    async fn mount_environment(
        &self,
        job_id: Uuid,
        env_id: Uuid,
    ) -> Result<EnvironmentRecord, StoreError> {
        self.guard_actor(job_id, "mount_environment", false).await?;
        let env = self.get_environment(env_id).await?;
        // SC-G07: ORPHANED/DISSOLVED are unmountable for work.
        if env.status != EnvStatus::Live {
            return Err(StoreError::EnvInvalid(format!(
                "environment {env_id} is {}; a non-LIVE room is not mountable for work (A.8)",
                env.status
            )));
        }
        // SC-G05: tier and title must agree (X.2). Teacher titles track the
        // specificity axis; a Student bears a flat honorific.
        let honorifics = self.get_config("honorific_set").await?;
        match env.kind {
            EnvKind::Teacher => {
                let expected = honorifics
                    .value
                    .get("teacher")
                    .and_then(|t| t.get(env.tier.as_str()))
                    .and_then(|v| v.as_str());
                if expected != Some(env.title.as_str()) {
                    return Err(StoreError::EnvInvalid(format!(
                        "Teacher environment tier {} and title '{}' disagree (Law X.2)",
                        env.tier, env.title
                    )));
                }
            }
            EnvKind::Student => {
                let set = honorifics.value.get("student").and_then(|v| v.as_array());
                let ok = set.is_some_and(|a| a.iter().any(|h| h.as_str() == Some(&env.title)));
                if !ok {
                    return Err(StoreError::EnvInvalid(format!(
                        "Student environment honorific '{}' is not in the flat set (Law X.3)",
                        env.title
                    )));
                }
            }
        }
        // SC-G01/G06: every item resolves and its chain walks root-to-leaf.
        // Slice 9 adds refined artifacts and Returns as first-class records
        // a scriptorium lawfully elects — the mount must know them, or a
        // room publishing its own product becomes unmountable.
        for item in self.env_items(env_id).await? {
            let resolves: bool = sqlx::query_scalar(
                r#"SELECT EXISTS(SELECT 1 FROM nodes WHERE node_id = $1)
                    OR EXISTS(SELECT 1 FROM links WHERE link_id = $1)
                    OR EXISTS(SELECT 1 FROM artifacts WHERE job_id = $1)
                    OR EXISTS(SELECT 1 FROM refined_artifacts WHERE artifact_id = $1)
                    OR EXISTS(SELECT 1 FROM returns WHERE return_id = $1)"#,
            )
            .bind(item.item_ref)
            .fetch_one(&self.pool)
            .await?;
            if !resolves {
                return Err(StoreError::EnvInvalid(format!(
                    "environment {env_id}: item {} resolves to no live record (IX.2)",
                    item.item_ref
                )));
            }
            self.validate_provenance_chain(&item.provenance).await?;
        }
        Ok(env)
    }

    async fn env_scoped_read(
        &self,
        reader_job_id: Uuid,
        env_id: Uuid,
        target_ref: Uuid,
    ) -> Result<(), StoreError> {
        let reader = self
            .guard_actor(reader_job_id, "env_scoped_read", false)
            .await?;
        // The reader must actually be bound to the room it claims: env_id
        // is not the caller's to assert — the job's env_ref is (IX.4). An
        // agent cannot borrow another environment's scope by naming it.
        if reader.env_ref != Some(env_id) {
            self.append_log(
                &env_id.to_string(),
                LogEvent::Violation,
                &serde_json::json!({
                    "operation": "env_scoped_read",
                    "reader": reader_job_id.to_string(),
                    "claimed_env": env_id.to_string(),
                    "actual_env": reader.env_ref.map(|e| e.to_string()),
                    "not_bound": true,
                }),
                Severity::Violation,
                "store",
            )
            .await?;
            return Err(StoreError::Forbidden(format!(
                "job {reader_job_id} is not bound to environment {env_id}; it may not read its scope (Law IX.4)"
            )));
        }
        // In the reader's own contents index → permitted (IX.4).
        let in_index: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM environment_items WHERE env_id = $1 AND item_ref = $2)",
        )
        .bind(env_id)
        .bind(target_ref)
        .fetch_one(&self.pool)
        .await?;
        if in_index {
            return Ok(());
        }
        // The global allowlist: the reader's own job or its own leases.
        if target_ref == reader_job_id {
            return Ok(());
        }
        let own_lease: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM lease_records WHERE lease_id = $1 AND job_id = $2)",
        )
        .bind(target_ref)
        .bind(reader_job_id)
        .fetch_one(&self.pool)
        .await?;
        if own_lease {
            return Ok(());
        }
        // The Pairing Exception (IX.5): a flagged item of a paired
        // counterpart environment, and nothing else of it.
        let paired_flagged: bool = sqlx::query_scalar(
            r#"SELECT EXISTS(
                 SELECT 1 FROM pairings p
                 JOIN environment_items ei ON ei.item_ref = $2 AND ei.flagged
                 WHERE ((p.teacher_env_ref = $1 AND p.student_env_ref = ei.env_id)
                     OR (p.student_env_ref = $1 AND p.teacher_env_ref = ei.env_id)))"#,
        )
        .bind(env_id)
        .bind(target_ref)
        .fetch_one(&self.pool)
        .await?;
        if paired_flagged {
            return Ok(());
        }
        // Out of scope: the wall agents hit (IX.4). Logged, then refused.
        self.append_log(
            &env_id.to_string(),
            LogEvent::Violation,
            &serde_json::json!({
                "operation": "env_scoped_read",
                "reader": reader_job_id.to_string(),
                "target": target_ref.to_string(),
                "out_of_scope": true,
            }),
            Severity::Violation,
            "store",
        )
        .await?;
        Err(StoreError::Forbidden(format!(
            "read of {target_ref} is outside environment {env_id}'s scope (Law IX.4)"
        )))
    }

    async fn form_pairing(
        &self,
        teacher_env_ref: Uuid,
        student_env_ref: Uuid,
        matrix_ref: Uuid,
        kind: PairingKind,
    ) -> Result<PairingRecord, StoreError> {
        let teacher = self.get_environment(teacher_env_ref).await?;
        let student = self.get_environment(student_env_ref).await?;
        if teacher.kind != EnvKind::Teacher || student.kind != EnvKind::Student {
            return Err(StoreError::ValidationFailed(
                "a pairing binds one Teacher environment and one Student environment (X.5)".into(),
            ));
        }
        // Both rooms must be live workplaces — an archive does not pair,
        // and the Exception must never serve a dissolved room's artifacts.
        if teacher.status != EnvStatus::Live || student.status != EnvStatus::Live {
            return Err(StoreError::ValidationFailed(
                "a pairing binds two LIVE environments; an archive does not pair (A.8, IX.5)"
                    .into(),
            ));
        }
        // IX.5: the Exception is scoped to the SHARED matrix — both rooms
        // and the pairing must name the same one (X.5: they occupy the
        // same node).
        if teacher.matrix_ref != student.matrix_ref || teacher.matrix_ref != matrix_ref {
            return Err(StoreError::ValidationFailed(format!(
                "a pairing binds two rooms over one shared matrix; got teacher {}, student {}, pairing {matrix_ref} (Law X.5, IX.5)",
                teacher.matrix_ref, student.matrix_ref
            )));
        }
        // X.5: tiers must match the pairing kind; REGULAR never pairs (and
        // cannot reach here — no REGULAR environment exists).
        let required = match kind {
            PairingKind::DevoutAssignment => Tier::Devout,
            PairingKind::CanonicalInstruction => Tier::Canon,
        };
        if teacher.tier != required || student.tier != required {
            return Err(StoreError::ValidationFailed(format!(
                "{kind} pairs {required} with {required}; got teacher {} and student {} (Law X.5)",
                teacher.tier, student.tier
            )));
        }
        let row = sqlx::query(
            r#"INSERT INTO pairings
                 (pairing_id, kind, teacher_env_ref, student_env_ref, matrix_ref,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, 'PairingRecord', $6, 'sovereign')
               RETURNING *"#,
        )
        .bind(Uuid::now_v7())
        .bind(kind.as_str())
        .bind(teacher_env_ref)
        .bind(student_env_ref)
        .bind(matrix_ref)
        .bind(RECORD_SCHEMA_VERSION)
        .fetch_one(&self.pool)
        .await?;
        self.append_log(
            &matrix_ref.to_string(),
            LogEvent::PairingFormed,
            &serde_json::json!({
                "kind": kind.as_str(),
                "teacher_env": teacher_env_ref.to_string(),
                "student_env": student_env_ref.to_string(),
            }),
            Severity::Info,
            "sovereign",
        )
        .await?;
        Self::pairing_from_row(&row)
    }

    async fn orphan_environment(&self, env_id: Uuid) -> Result<EnvironmentRecord, StoreError> {
        let env = self.get_environment(env_id).await?;
        if env.status == EnvStatus::Dissolved {
            return Err(StoreError::ValidationFailed(
                "a dissolved environment does not orphan; it is already gone (A.8)".into(),
            ));
        }
        let row = sqlx::query(
            r#"UPDATE environments SET status = 'ORPHANED', revision = revision + 1
               WHERE env_id = $1 AND status = 'LIVE' RETURNING *"#,
        )
        .bind(env_id)
        .fetch_optional(&self.pool)
        .await?;
        let record = match row {
            Some(row) => Self::environment_from_row(&row)?,
            None => return self.get_environment(env_id).await, // already orphaned
        };
        self.append_log(
            &env_id.to_string(),
            LogEvent::EnvOrphaned,
            &serde_json::json!({ "prior_status": "LIVE" }),
            Severity::Warning,
            "store",
        )
        .await?;
        Ok(record)
    }

    async fn adopt_concordat(
        &self,
        actor: &str,
        version: &Version,
        capability_tables: &serde_json::Value,
        pairing_semantics: &serde_json::Value,
    ) -> Result<ConcordatArtifact, StoreError> {
        // Every version is retained forever (§3.3): adopting a version that
        // already exists is refused rather than overwriting it.
        let row = sqlx::query(
            r#"INSERT INTO concordat_artifacts
                 (version, capability_tables, pairing_semantics, adopted_by,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, 'ConcordatArtifact', $5, $4)
               ON CONFLICT (version) DO NOTHING
               RETURNING *"#,
        )
        .bind(version.to_string())
        .bind(capability_tables)
        .bind(pairing_semantics)
        .bind(actor)
        .bind(RECORD_SCHEMA_VERSION)
        .fetch_optional(&self.pool)
        .await?;
        let concordat = match row {
            Some(row) => Self::concordat_from_row(&row)?,
            None => {
                return Err(StoreError::ValidationFailed(format!(
                    "Concordat {version} is already adopted; every version is retained, never rewritten (§3.3)"
                )))
            }
        };
        self.append_log(
            &format!("concordat:{version}"),
            LogEvent::ConcordatAdopted,
            &serde_json::json!({ "version": version.to_string(), "adopted_by": actor }),
            Severity::Info,
            actor,
        )
        .await?;
        Ok(concordat)
    }

    async fn get_concordat(&self, version: &Version) -> Result<ConcordatArtifact, StoreError> {
        let row = sqlx::query("SELECT * FROM concordat_artifacts WHERE version = $1")
            .bind(version.to_string())
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StoreError::SchemaMismatch(format!("no Concordat version {version}")))?;
        Self::concordat_from_row(&row)
    }

    async fn persist_instruction(
        &self,
        job_id: Uuid,
        draft: &InstructionDraft,
    ) -> Result<InstructionRecord, StoreError> {
        let job = self
            .guard_actor(job_id, "persist_instruction", false)
            .await?;
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "an Instruction is written during WORK, not {} (Law I.1)",
                job.status
            )));
        }
        if job.agent_type != AgentType::Teacher {
            return Err(StoreError::ValidationFailed(format!(
                "only a Teacher writes Instructions, not {} (Holy Standard §1)",
                job.agent_type
            )));
        }
        // B.1: sources_drawn is required iff teacher_tier REGULAR — both
        // directions (§6.3). A Regular output must disclose; a conferred
        // Teacher carries none.
        if draft.teacher_tier == Tier::Regular && draft.sources_drawn.is_empty() {
            return Err(StoreError::ValidationFailed(
                "a Regular Teacher output must carry sources_drawn (Holy Standard §6.3)".into(),
            ));
        }
        if draft.teacher_tier != Tier::Regular && !draft.sources_drawn.is_empty() {
            return Err(StoreError::ValidationFailed(
                "sources_drawn is a Regular-Teacher disclosure; a conferred Teacher carries none (B.1)".into(),
            ));
        }
        // skew is DERIVED, never trusted from a caller (B.1): the
        // canon-associated share of disclosed draws exceeds
        // bias_skew_threshold (§6.3). A mistyped constant refuses — a
        // fabricated default is a decision the sovereign never made
        // (SC-H07; this very site is the class's third survivor, caught
        // by the slice-10 sweep).
        let skew_threshold = self
            .get_config("bias_skew_threshold")
            .await?
            .value
            .as_f64()
            .ok_or_else(|| {
                StoreError::ValidationFailed(
                    "bias_skew_threshold is not a number; no code path substitutes a \
                     fabricated default for a constant (SC-H07)"
                        .into(),
                )
            })?;
        // Checked folds: an adversarial draw census saturates legibly, it
        // never debug-panics a RUNNING labor (B1 aggravation; SC-E05).
        let total: i64 = draft
            .sources_drawn
            .iter()
            .map(|s| s.draw_count.max(0))
            .try_fold(0i64, i64::checked_add)
            .unwrap_or(i64::MAX);
        let canon: i64 = draft
            .sources_drawn
            .iter()
            .filter(|s| s.canon_associated)
            .map(|s| s.draw_count.max(0))
            .try_fold(0i64, i64::checked_add)
            .unwrap_or(i64::MAX);
        #[allow(clippy::cast_precision_loss)] // draw counts are small
        let skew = total > 0 && (canon as f64 / total as f64) > skew_threshold;
        // A Devout/Canon Teacher works from an environment; a Regular does
        // not (B.1: teacher_env_ref null for Regulars).
        match draft.teacher_tier {
            Tier::Regular if draft.teacher_env_ref.is_some() => {
                return Err(StoreError::ValidationFailed(
                    "a Regular Teacher establishes no environment (X.1)".into(),
                ))
            }
            Tier::Devout | Tier::Canon if draft.teacher_env_ref.is_none() => {
                return Err(StoreError::ValidationFailed(
                    "a conferred Teacher writes from its environment (B.1)".into(),
                ))
            }
            _ => {}
        }
        // The supersedes target, if any, must resolve.
        if let Some(prior) = draft.supersedes_ref {
            self.get_instruction(prior).await?;
        }
        let steps = serialize_steps(&draft.steps);
        let criteria = serialize_criteria(&draft.acceptance_criteria);
        let sources = serialize_sources(&draft.sources_drawn);
        let row = sqlx::query(
            r#"INSERT INTO instructions
                 (instruction_id, teacher_env_ref, teacher_tier, target_tier, concordat_version,
                  objective, steps, acceptance_criteria, sources_drawn, skew, supersedes_ref,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'InstructionRecord', $12, $13::text)
               RETURNING *"#,
        )
        .bind(Uuid::now_v7())
        .bind(draft.teacher_env_ref)
        .bind(draft.teacher_tier.as_str())
        .bind(draft.target_tier.as_str())
        .bind(draft.concordat_version.to_string())
        .bind(&draft.objective)
        .bind(steps)
        .bind(criteria)
        .bind(sources)
        .bind(skew)
        .bind(draft.supersedes_ref)
        .bind(RECORD_SCHEMA_VERSION)
        .bind(job_id)
        .fetch_one(&self.pool)
        .await?;
        Self::instruction_from_row(&row)
    }

    async fn flag_instruction(
        &self,
        job_id: Uuid,
        instruction_id: Uuid,
    ) -> Result<InstructionRecord, StoreError> {
        self.guard_actor(job_id, "flag_instruction", false).await?;
        // Ruling G7: certification is byte-strength — the canonical body's
        // hash is persisted in the same act that flags, and re-proven at
        // every read of the flagged record. CAS on revision: the body the
        // hash covers is the body that flags.
        let current = self.get_instruction(instruction_id).await?;
        let content_sha = Self::sha256_of_canonical(&Self::instruction_body(&current));
        let row = sqlx::query(
            r#"UPDATE instructions
               SET flagged = true, content_sha = $3, revision = revision + 1
               WHERE instruction_id = $1 AND NOT flagged AND revision = $2
               RETURNING *"#,
        )
        .bind(instruction_id)
        .bind(current.revision)
        .bind(&content_sha)
        .fetch_optional(&self.pool)
        .await?;
        let record = match row {
            Some(row) => Self::instruction_from_row(&row)?,
            None => {
                let now = self.get_instruction(instruction_id).await?;
                if !now.flagged {
                    return Err(StoreError::StaleRevision {
                        expected: current.revision,
                        subject: format!("instruction:{instruction_id}"),
                    });
                }
                now // already flagged: idempotent read-back
            }
        };
        self.append_log(
            &instruction_id.to_string(),
            LogEvent::InstructionFlagged,
            &serde_json::json!({ "target_tier": record.target_tier.as_str() }),
            Severity::Info,
            &job_id.to_string(),
        )
        .await?;
        Ok(record)
    }

    async fn get_instruction(&self, instruction_id: Uuid) -> Result<InstructionRecord, StoreError> {
        let row = sqlx::query("SELECT * FROM instructions WHERE instruction_id = $1")
            .bind(instruction_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StoreError::NotFound(format!("no such instruction {instruction_id}")))?;
        let record = Self::instruction_from_row(&row)?;
        // Ruling G7's read half: every read of a flagged Instruction
        // re-proves the certification hash — VALIDATE_IN's byte-integrity
        // gate, enforced at the store so no read path can skip it.
        Self::prove_content_sha(
            "Instruction",
            instruction_id,
            record.flagged,
            record.content_sha.as_deref(),
            &Self::instruction_body(&record),
        )?;
        Ok(record)
    }

    async fn record_regular_output(
        &self,
        job_id: Uuid,
        instruction_ref: Uuid,
        sources: &[SourceDraw],
        skew: bool,
        window: i64,
    ) -> Result<f64, StoreError> {
        // H3(3): the disclosure carries the disclosing job's identity —
        // XIII.1 means what it says; 'store' was an anonymous surface.
        self.guard_actor(job_id, "record_regular_output", false)
            .await?;
        if window < 1 {
            return Err(StoreError::ValidationFailed(
                "the trailing window is at least 1 (§6.3; a zero window silently \
                 disables escalation — B2)"
                    .into(),
            ));
        }
        sqlx::query(
            r#"INSERT INTO regular_outputs
                 (output_id, instruction_ref, sources_drawn, skew,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, 'RegularOutput', $5, $6)"#,
        )
        .bind(Uuid::now_v7())
        .bind(instruction_ref)
        .bind(serialize_sources(sources))
        .bind(skew)
        .bind(RECORD_SCHEMA_VERSION)
        .bind(job_id.to_string())
        .execute(&self.pool)
        .await?;
        // The trailing-window skew share (§6.3): of the last `window`
        // Regular outputs, what fraction are skewed.
        let share: Option<f64> = sqlx::query_scalar(
            r#"SELECT avg(CASE WHEN skew THEN 1.0 ELSE 0.0 END)::float8
               FROM (SELECT skew FROM regular_outputs ORDER BY produced_at DESC LIMIT $1) w"#,
        )
        .bind(window)
        .fetch_one(&self.pool)
        .await?;
        Ok(share.unwrap_or(0.0))
    }

    async fn bias_warning_state(&self, scope: &str) -> Result<Option<String>, StoreError> {
        let status: Option<String> =
            sqlx::query_scalar("SELECT status FROM bias_warnings WHERE scope = $1")
                .bind(scope)
                .fetch_optional(&self.pool)
                .await?;
        Ok(status)
    }

    async fn raise_bias_warning(&self, job_id: Uuid, scope: &str) -> Result<(), StoreError> {
        // H3(3): the raise carries the disclosing job's identity (XIII.1).
        self.guard_actor(job_id, "raise_bias_warning", false)
            .await?;
        // Raise iff none stands: a SILENCED scope is not re-raised until
        // lifted; a STANDING one keeps counting without a second raise.
        let inserted = sqlx::query(
            r#"INSERT INTO bias_warnings
                 (scope, schema_name, schema_version, produced_by)
               VALUES ($1, 'BiasWarning', $2, $3)
               ON CONFLICT (scope) DO NOTHING"#,
        )
        .bind(scope)
        .bind(RECORD_SCHEMA_VERSION)
        .bind(job_id.to_string())
        .execute(&self.pool)
        .await?;
        if inserted.rows_affected() > 0 {
            self.append_log(
                &format!("bias:{scope}"),
                LogEvent::BiasWarning,
                &serde_json::json!({ "scope": scope, "status": "STANDING" }),
                Severity::Warning,
                &job_id.to_string(),
            )
            .await?;
        }
        Ok(())
    }

    async fn resolve_bias_warning(
        &self,
        actor: &str,
        scope: &str,
        acknowledge: bool,
    ) -> Result<(), StoreError> {
        let (status, severity) = if acknowledge {
            ("ACKNOWLEDGED", Severity::Warning)
        } else {
            ("SILENCED", Severity::Suppressed)
        };
        let updated = sqlx::query(
            r#"UPDATE bias_warnings
               SET status = $2, resolved_at = now(), revision = revision + 1
               WHERE scope = $1"#,
        )
        .bind(scope)
        .bind(status)
        .execute(&self.pool)
        .await?;
        if updated.rows_affected() == 0 {
            return Err(StoreError::NotFound(format!(
                "no standing bias warning for scope '{scope}'"
            )));
        }
        self.append_log(
            &format!("bias:{scope}"),
            LogEvent::BiasWarning,
            &serde_json::json!({ "scope": scope, "status": status, "by": actor }),
            severity,
            actor,
        )
        .await?;
        Ok(())
    }

    async fn persist_return(
        &self,
        job_id: Uuid,
        draft: &ReturnDraft,
    ) -> Result<ReturnManifest, StoreError> {
        let job = self.guard_actor(job_id, "persist_return", false).await?;
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "a Return is written during WORK, not {} (Law I.1)",
                job.status
            )));
        }
        if job.agent_type != AgentType::Student {
            return Err(StoreError::ValidationFailed(format!(
                "only a Student writes Returns, not {} (Student Handbook §1)",
                job.agent_type
            )));
        }
        // The answered Instruction must resolve and be certified — an
        // unflagged Instruction is invisible to the Student (§5.1), so no
        // Return can answer one.
        let instruction = self.get_instruction(draft.instruction_ref).await?;
        if !instruction.flagged {
            return Err(StoreError::ValidationFailed(
                "the answered Instruction is not flagged; unflagged means uncertified (§5.1)"
                    .into(),
            ));
        }
        // B.2: a Return rises from a Student's room.
        let env = self.get_environment(draft.student_env_ref).await?;
        if env.kind != EnvKind::Student {
            return Err(StoreError::ValidationFailed(format!(
                "student_env_ref names a {} environment; a Return rises from a Student's room (B.2)",
                env.kind
            )));
        }
        if env.status != EnvStatus::Live {
            return Err(StoreError::ValidationFailed(format!(
                "environment {} is {}; an archived room takes no new work (A.8)",
                draft.student_env_ref, env.status
            )));
        }
        // IX.4: the writing job must be bound to the room the Return
        // rises from — an agent cannot answer from a room by naming it.
        if job.env_ref != Some(draft.student_env_ref) {
            return Err(StoreError::ValidationFailed(format!(
                "job {job_id} is not bound to environment {}; a Return rises from the job's own room (Law IX.4)",
                draft.student_env_ref
            )));
        }
        // B.1: the Instruction was linted against target_tier's capability
        // table; a room of another tier answers a contract that never
        // bound it.
        if env.tier != instruction.target_tier {
            return Err(StoreError::ValidationFailed(format!(
                "the Return rises from a {} room; the Instruction binds {} (B.1)",
                env.tier, instruction.target_tier
            )));
        }
        // X.5: a conferred Teacher's Instruction is answered across the
        // pairing bridge; an unpaired room has no standing to answer. A
        // Regular Teacher has no room, so nothing to pair against.
        if let Some(teacher_env) = instruction.teacher_env_ref {
            let paired: bool = sqlx::query_scalar(
                r#"SELECT EXISTS(SELECT 1 FROM pairings
                     WHERE teacher_env_ref = $1 AND student_env_ref = $2)"#,
            )
            .bind(teacher_env)
            .bind(draft.student_env_ref)
            .fetch_one(&self.pool)
            .await?;
            if !paired {
                return Err(StoreError::ValidationFailed(format!(
                    "no pairing binds room {} to the Instruction's teacher room {teacher_env} (X.5)",
                    draft.student_env_ref
                )));
            }
        }
        // B.2: every returned item carries real provenance — the nil floor,
        // mirroring the evidence rule.
        for (i, item) in draft.items.iter().enumerate() {
            if item.item_ref.is_nil() || item.provenance_ref.is_nil() {
                return Err(StoreError::ValidationFailed(format!(
                    "item {i} carries a nil ref; a Return hands back things that exist (B.2)"
                )));
            }
        }
        // SC-L01, the items half — full item resolution at the Deacon's
        // threshold (SLICE_09 §6 finding 7's pin; docs/dev/SLICE_10.md §1).
        // The resolution set is generous but real. An item_ref may name:
        //   - a quarantine item (external material handed back under
        //     stewardship, A.12),
        //   - a node (a corpus atom, doc 3 §2.1),
        //   - a refined artifact (the Student's own product, Handbook §1.2b),
        //   - a link (an organization change, doc 3 §2.3).
        // A provenance_ref may name:
        //   - a ProvenanceChain (C.2: a chain_ref in provenance_chains — the
        //     arrival story of external-origin material), or
        //   - an elected item whose election carries its chain (env-item
        //     provenance, A.8's contents index).
        // A ref outside that set names nothing: refused before any write,
        // never persisted (B.2).
        for (i, item) in draft.items.iter().enumerate() {
            let (item_resolves, provenance_resolves): (bool, bool) = sqlx::query_as(
                r#"SELECT
                     EXISTS(SELECT 1 FROM quarantine_items WHERE item_ref = $1)
                       OR EXISTS(SELECT 1 FROM nodes WHERE node_id = $1)
                       OR EXISTS(SELECT 1 FROM refined_artifacts WHERE artifact_id = $1)
                       OR EXISTS(SELECT 1 FROM links WHERE link_id = $1),
                     EXISTS(SELECT 1 FROM provenance_chains WHERE chain_ref = $2)
                       OR EXISTS(SELECT 1 FROM environment_items WHERE item_ref = $2)"#,
            )
            .bind(item.item_ref)
            .bind(item.provenance_ref)
            .fetch_one(&self.pool)
            .await?;
            if !item_resolves {
                return Err(StoreError::ValidationFailed(format!(
                    "item {i}'s item_ref resolves to no quarantine item, node, refined \
                     artifact, or link; a Return hands back things that exist (B.2)"
                )));
            }
            if !provenance_resolves {
                return Err(StoreError::ValidationFailed(format!(
                    "item {i}'s provenance_ref resolves to no provenance chain or \
                     elected-item provenance; a Return's items carry real provenance (B.2)"
                )));
            }
        }
        // B.2: the completion contract, criterion by criterion, against the
        // answered Instruction's stored acceptance_criteria.
        validate_completion_contract(&instruction.acceptance_criteria, &draft.completion)?;
        let items = serialize_items(&draft.items);
        let completion = serialize_completion(&draft.completion);
        let row = sqlx::query(
            r#"INSERT INTO returns
                 (return_id, instruction_ref, student_env_ref, concordat_version,
                  items, completion, schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, $6, 'ReturnManifest', $7, $8::text)
               RETURNING *"#,
        )
        .bind(Uuid::now_v7())
        .bind(draft.instruction_ref)
        .bind(draft.student_env_ref)
        .bind(draft.concordat_version.to_string())
        .bind(items)
        .bind(completion)
        .bind(RECORD_SCHEMA_VERSION)
        .bind(job_id)
        .fetch_one(&self.pool)
        .await?;
        Self::return_from_row(&row)
    }

    async fn flag_return(
        &self,
        job_id: Uuid,
        return_id: Uuid,
    ) -> Result<ReturnManifest, StoreError> {
        // Certification is the trust boundary of the Return: only the
        // RUNNING Student job bound to the Return's own room flags it —
        // the same three walls persist_return holds (Law IX.4).
        let job = self.guard_actor(job_id, "flag_return", false).await?;
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "a Return is certified during WORK, not {} (Law I.1)",
                job.status
            )));
        }
        if job.agent_type != AgentType::Student {
            return Err(StoreError::ValidationFailed(format!(
                "only a Student certifies Returns, not {} (Student Handbook §3.1)",
                job.agent_type
            )));
        }
        let existing = self.get_return(return_id).await?;
        if job.env_ref != Some(existing.student_env_ref) {
            return Err(StoreError::ValidationFailed(format!(
                "job {job_id} is not bound to the Return's room {}; certification rises from the job's own room (Law IX.4)",
                existing.student_env_ref
            )));
        }
        // Ruling G7: hash at flag, byte-strength, CAS on revision.
        let content_sha = Self::sha256_of_canonical(&Self::return_body(&existing));
        let row = sqlx::query(
            r#"UPDATE returns
               SET flagged = true, content_sha = $3, revision = revision + 1
               WHERE return_id = $1 AND NOT flagged AND revision = $2
               RETURNING *"#,
        )
        .bind(return_id)
        .bind(existing.revision)
        .bind(&content_sha)
        .fetch_optional(&self.pool)
        .await?;
        // Already flagged: idempotent read-back, but no second
        // RETURN_FLAGGED event — the log testifies one certification.
        let Some(row) = row else {
            let now = self.get_return(return_id).await?;
            if !now.flagged {
                return Err(StoreError::StaleRevision {
                    expected: existing.revision,
                    subject: format!("return:{return_id}"),
                });
            }
            return Ok(now);
        };
        let record = Self::return_from_row(&row)?;
        self.append_log(
            &return_id.to_string(),
            LogEvent::ReturnFlagged,
            &serde_json::json!({ "instruction_ref": record.instruction_ref.to_string() }),
            Severity::Info,
            &job_id.to_string(),
        )
        .await?;
        Ok(record)
    }

    async fn get_return(&self, return_id: Uuid) -> Result<ReturnManifest, StoreError> {
        let row = sqlx::query("SELECT * FROM returns WHERE return_id = $1")
            .bind(return_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StoreError::NotFound(format!("no such return {return_id}")))?;
        let record = Self::return_from_row(&row)?;
        // Ruling G7's read half, same as the Instruction's.
        Self::prove_content_sha(
            "Return",
            return_id,
            record.flagged,
            record.content_sha.as_deref(),
            &Self::return_body(&record),
        )?;
        Ok(record)
    }

    async fn persist_refined_artifact(
        &self,
        job_id: Uuid,
        env_ref: Uuid,
        source_refs: &[Uuid],
        method: &str,
        content_sha: &str,
    ) -> Result<RefinedArtifact, StoreError> {
        let job = self
            .guard_actor(job_id, "persist_refined_artifact", false)
            .await?;
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "a refinement is recorded during WORK, not {} (Law I.1)",
                job.status
            )));
        }
        if job.agent_type != AgentType::Student {
            return Err(StoreError::ValidationFailed(format!(
                "refinement is Student stewardship, not {}'s (Handbook §1.2)",
                job.agent_type
            )));
        }
        if source_refs.is_empty() {
            return Err(StoreError::ValidationFailed(
                "a refinement over nothing is debris; the derivation names its sources (§1.2b)"
                    .into(),
            ));
        }
        // The method is a short token, not free text: it is persisted and
        // echoed into the append-only log, so agent-shaped prose (or a
        // secret-shaped string) must never fit through it (Law XV).
        if method.is_empty()
            || method.len() > 64
            || !method
                .bytes()
                .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'@' | b'.' | b'-' | b'_'))
        {
            return Err(StoreError::ValidationFailed(
                "the derivation names its method as a token of [a-z0-9@.-_], at most 64 chars (§1.2b, Law XV)"
                    .into(),
            ));
        }
        if content_sha.len() != 64
            || !content_sha
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        {
            return Err(StoreError::ValidationFailed(
                "content_sha must be 64 lowercase hex chars (SHA-256)".into(),
            ));
        }
        let env = self.get_environment(env_ref).await?;
        if env.kind != EnvKind::Student {
            return Err(StoreError::ValidationFailed(format!(
                "refined artifacts land in a Student's scriptorium, not a {} room (§1.2)",
                env.kind
            )));
        }
        if env.status != EnvStatus::Live {
            return Err(StoreError::ValidationFailed(format!(
                "environment {env_ref} is {}; an archived room takes no new work (A.8)",
                env.status
            )));
        }
        // IX.4: the refining job must be bound to the room it writes into —
        // an unbound job naming a victim's room would leave permanent,
        // unremovable debris there (no_delete stands on the record).
        if job.env_ref != Some(env_ref) {
            return Err(StoreError::ValidationFailed(format!(
                "job {job_id} is not bound to environment {env_ref}; refinement lands in the job's own room (Law IX.4)"
            )));
        }
        // Source resolution is deliberately NOT proven here: the closure
        // walk (Handbook §1.2c) is the detector for a dangling derivation
        // ref — debris is found by verification, not silently prevented
        // into unfindability.
        let refs = serde_json::Value::Array(
            source_refs
                .iter()
                .map(|u| serde_json::Value::String(u.to_string()))
                .collect(),
        );
        let row = sqlx::query(
            r#"INSERT INTO refined_artifacts
                 (artifact_id, env_ref, source_refs, method, content_sha,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, 'RefinedArtifact', $6, $7::text)
               RETURNING *"#,
        )
        .bind(Uuid::now_v7())
        .bind(env_ref)
        .bind(refs)
        .bind(method)
        .bind(content_sha)
        .bind(RECORD_SCHEMA_VERSION)
        .bind(job_id)
        .fetch_one(&self.pool)
        .await?;
        let record = Self::refined_artifact_from_row(&row)?;
        self.append_log(
            &record.artifact_id.to_string(),
            LogEvent::Refined,
            &serde_json::json!({
                "env_ref": env_ref.to_string(),
                "method": method,
                "content_sha": content_sha,
            }),
            Severity::Info,
            &job_id.to_string(),
        )
        .await?;
        Ok(record)
    }

    async fn get_refined_artifact(&self, artifact_id: Uuid) -> Result<RefinedArtifact, StoreError> {
        let row = sqlx::query("SELECT * FROM refined_artifacts WHERE artifact_id = $1")
            .bind(artifact_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| {
                StoreError::NotFound(format!("no such refined artifact {artifact_id}"))
            })?;
        Self::refined_artifact_from_row(&row)
    }

    async fn refined_artifacts_in(
        &self,
        env_ref: Uuid,
    ) -> Result<Vec<RefinedArtifact>, StoreError> {
        let rows =
            sqlx::query("SELECT * FROM refined_artifacts WHERE env_ref = $1 ORDER BY produced_at")
                .bind(env_ref)
                .fetch_all(&self.pool)
                .await?;
        rows.iter().map(Self::refined_artifact_from_row).collect()
    }
    // ---- the threshold & the J-floor (Dogma Book II §1; docs/dev/SLICE_10.md) ----

    async fn author_mandate(
        &self,
        actor: &str,
        draft: &MandateDraft,
    ) -> Result<MandateRecord, StoreError> {
        // The shape half of SC-J02: kind/recipient coherence, typed
        // locators, query-shaped demands rejected.
        validate_mandate_shape(draft).map_err(StoreError::ValidationFailed)?;
        if !draft.trip_budget.is_object() {
            return Err(StoreError::ValidationFailed(
                "trip_budget must be an object (C.4)".into(),
            ));
        }
        // The resolution half: the recipient exists and is what the kind
        // demands; a source_id names a registered source or fails —
        // "unknown source_id" is an authorship failure, not a trip failure.
        match draft.kind {
            MandateKind::Canon => {
                let env = self
                    .get_environment(draft.teacher_env_ref.expect("shape-validated"))
                    .await?;
                if env.kind != EnvKind::Teacher {
                    return Err(StoreError::ValidationFailed(format!(
                        "a canon collects for a Teacher; {} is a {} room (C.4)",
                        env.env_id, env.kind
                    )));
                }
                if env.status != EnvStatus::Live {
                    return Err(StoreError::ValidationFailed(format!(
                        "the recipient room is {}; a canon charters living work (A.8)",
                        env.status
                    )));
                }
            }
            MandateKind::Writ => {
                let matrix = self
                    .get_matrix(draft.matrix_ref.expect("shape-validated"))
                    .await?;
                if matrix.status != MatrixStatus::Cardinal {
                    return Err(StoreError::ValidationFailed(format!(
                        "a writ feeds a Cardinal matrix; {} is {} (Handbook §1.2)",
                        matrix.matrix_id, matrix.status
                    )));
                }
            }
        }
        // A v1 trip's fetch targets are typed locators — a WRIT's `demands`
        // targets or a CANON's `sources` (C.4 ruling 2026-07-09) — and every
        // source_id among them resolves to a registered source at AUTHORSHIP
        // or fails: the IDENTICAL wall for both kinds, an authorship failure
        // never a trip failure (SC-J02).
        let fetch_targets: &[WritTarget] = match &draft.demands {
            MandateDemands::WritTargets(targets) => targets,
            MandateDemands::CanonClauses(_) => &draft.sources,
        };
        if fetch_targets
            .iter()
            .any(|t| matches!(t.locator, Locator::SourceId(_)))
        {
            let registry = self.get_config("known_source_ids").await?;
            // known_source_ids parses as an array or this read refuses — an
            // empty roster fabricated from a mistyped constant would reject
            // every trip under the WRONG reason (unknown source), swallowing
            // the type error (SC-H07).
            let known: Vec<&str> = registry
                .value
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                .ok_or_else(|| {
                    StoreError::ValidationFailed(
                        "known_source_ids is not an array; no fabricated default \
                         stands in for a constant (SC-H07)"
                            .into(),
                    )
                })?;
            for (i, target) in fetch_targets.iter().enumerate() {
                if let Locator::SourceId(id) = &target.locator {
                    if !known.contains(&id.as_str()) {
                        return Err(StoreError::ValidationFailed(format!(
                            "mandate target {i} names unknown source_id '{id}'; targets \
                             resolve at authorship, before any trip (SC-J02)"
                        )));
                    }
                }
            }
        }
        let demands = match &draft.demands {
            MandateDemands::CanonClauses(clauses) => serde_json::Value::Array(
                clauses
                    .iter()
                    .map(|c| serde_json::json!({ "clause": c }))
                    .collect(),
            ),
            MandateDemands::WritTargets(targets) => serde_json::Value::Array(
                targets
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "locator": { "kind": t.locator.kind(), "value": t.locator.value() },
                            "note": t.note,
                        })
                    })
                    .collect(),
            ),
        };
        let sources = serde_json::Value::Array(
            draft
                .sources
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "locator": { "kind": t.locator.kind(), "value": t.locator.value() },
                        "note": t.note,
                    })
                })
                .collect(),
        );
        let serialized = format!("{demands} {sources} {}", draft.trip_budget);
        if let Some(pattern) = secrets::scan(&serialized) {
            return Err(StoreError::SecretDetected(format!(
                "mandate text matched secret pattern '{pattern}' (Law XV.2)"
            )));
        }
        let mandate_id = Uuid::now_v7();
        let mut tx = self.pool.begin().await?;
        Self::set_actor_class(&mut tx, SOVEREIGN_CLASS).await?;
        let row = sqlx::query(
            r#"INSERT INTO mandates
                 (mandate_id, kind, teacher_env_ref, matrix_ref, demands, sources, trip_budget,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, 'MandateRecord', $8, $9)
               RETURNING *"#,
        )
        .bind(mandate_id)
        .bind(draft.kind.as_str())
        .bind(draft.teacher_env_ref)
        .bind(draft.matrix_ref)
        .bind(&demands)
        .bind(&sources)
        .bind(&draft.trip_budget)
        .bind(RECORD_SCHEMA_VERSION)
        .bind(actor)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        self.append_log(
            &mandate_id.to_string(),
            LogEvent::MandateAuthored,
            &serde_json::json!({ "kind": draft.kind.as_str() }),
            Severity::Info,
            actor,
        )
        .await?;
        Self::mandate_from_row(&row)
    }

    async fn get_mandate(&self, mandate_id: Uuid) -> Result<MandateRecord, StoreError> {
        let row = sqlx::query("SELECT * FROM mandates WHERE mandate_id = $1")
            .bind(mandate_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StoreError::NotFound(format!("no such mandate {mandate_id}")))?;
        Self::mandate_from_row(&row)
    }

    async fn append_chain_entry(
        &self,
        job_id: Uuid,
        draft: &ChainEntryDraft,
    ) -> Result<ChainEntry, StoreError> {
        let job = self
            .guard_actor(job_id, "append_chain_entry", false)
            .await?;
        if let Some(pattern) = secrets::scan(&draft.prompt_or_reason) {
            return Err(StoreError::SecretDetected(format!(
                "chain entry matched secret pattern '{pattern}' (Law XV.2)"
            )));
        }
        if draft.prompt_or_reason.trim().is_empty() {
            return Err(StoreError::ProvenanceIncomplete(
                "an arrival without a story is exactly what the chain exists to prevent (C.2)"
                    .into(),
            ));
        }
        // A labor appends under its own mandate only.
        if let Some(mandate_ref) = draft.mandate_ref {
            if job.brief_ref != Some(mandate_ref) {
                return Err(StoreError::ValidationFailed(format!(
                    "job {job_id} cites mandate {mandate_ref} but labors under {:?}; \
                     a chain entry cites the hand that sent it (C.2)",
                    job.brief_ref
                )));
            }
            self.get_mandate(mandate_ref).await?;
        }
        let produced = serde_json::Value::Array(
            draft
                .produced
                .iter()
                .map(|u| serde_json::Value::String(u.to_string()))
                .collect(),
        );
        // The store issues the next seq; the grammar trigger holds the root
        // and the gapless-append rules. Chains are single-writer per subject
        // (one in-flight fetching labor per item, §4.2), so a same-seq race
        // is not a normal path — but should two appends collide on the
        // (chain_ref, link_seq) primary key, the loser surfaces a typed
        // StaleRevision: the caller re-reads the chain and appends afresh,
        // never a silent gap or an opaque error.
        let row = sqlx::query(
            r#"INSERT INTO provenance_chains
                 (chain_ref, link_seq, kind, actor_job_ref, mandate_ref, prompt_or_reason,
                  produced, schema_name, schema_version, produced_by)
               VALUES ($1,
                 (SELECT COALESCE(MAX(link_seq) + 1, 0) FROM provenance_chains WHERE chain_ref = $1),
                 $2, $3, $4, $5, $6, 'ChainEntry', $7, $8)
               RETURNING *"#,
        )
        .bind(draft.chain_ref)
        .bind(draft.kind.as_str())
        .bind(job_id)
        .bind(draft.mandate_ref)
        .bind(&draft.prompt_or_reason)
        .bind(&produced)
        .bind(RECORD_SCHEMA_VERSION)
        .bind(job_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db) if db.message().contains("PROVENANCE_INCOMPLETE") => {
                StoreError::ProvenanceIncomplete(db.message().to_string())
            }
            sqlx::Error::Database(db) if db.code().as_deref() == Some("23505") => {
                StoreError::StaleRevision {
                    expected: -1,
                    subject: format!("provenance_chain:{}", draft.chain_ref),
                }
            }
            _ => StoreError::Db(e),
        })?;
        let entry = Self::chain_from_row(&row)?;
        self.append_log(
            &draft.chain_ref.to_string(),
            LogEvent::ChainAppended,
            &serde_json::json!({ "link_seq": entry.link_seq, "kind": entry.kind.as_str() }),
            Severity::Info,
            &job_id.to_string(),
        )
        .await?;
        Ok(entry)
    }

    async fn chain_for(&self, chain_ref: Uuid) -> Result<Vec<ChainEntry>, StoreError> {
        let rows =
            sqlx::query("SELECT * FROM provenance_chains WHERE chain_ref = $1 ORDER BY link_seq")
                .bind(chain_ref)
                .fetch_all(&self.pool)
                .await?;
        rows.iter().map(Self::chain_from_row).collect()
    }

    async fn quarantine_deposit(
        &self,
        job_id: Uuid,
        item_ref: Uuid,
        draft: &QuarantineDraft,
    ) -> Result<QuarantineItem, StoreError> {
        let job = self
            .guard_actor(job_id, "quarantine_deposit", false)
            .await?;
        if job.status != JobStatus::Running {
            return Err(StoreError::ValidationFailed(format!(
                "external material lands mid-labor, not at {} (Law I.1)",
                job.status
            )));
        }
        if draft.filename.trim().is_empty() || draft.declared_type.trim().is_empty() {
            return Err(StoreError::ValidationFailed(
                "a quarantined item names its filename and declared type (A.12)".into(),
            ));
        }
        if let Some(pattern) = secrets::scan(&format!("{} {}", draft.filename, draft.declared_type))
        {
            return Err(StoreError::SecretDetected(format!(
                "quarantine metadata matched secret pattern '{pattern}' (Law XV.2)"
            )));
        }
        // The deposit rides the depositor's OWN human charter (§1.4).
        match (draft.mandate_ref, draft.brief_ref) {
            (Some(mandate_ref), _) => {
                if job.brief_ref != Some(mandate_ref) {
                    return Err(StoreError::ValidationFailed(format!(
                        "job {job_id} deposits under mandate {mandate_ref} but labors under {:?}; \
                         every arrival cites the hand that sent it (§1.4)",
                        job.brief_ref
                    )));
                }
            }
            (None, Some(brief_ref)) => {
                if job.brief_ref != Some(brief_ref) {
                    return Err(StoreError::ValidationFailed(format!(
                        "job {job_id} deposits under brief {brief_ref} but labors under {:?} (§1.4)",
                        job.brief_ref
                    )));
                }
            }
            (None, None) => {
                return Err(StoreError::ValidationFailed(
                    "every arrival began in a human hand: a mandate or a brief (§1.4)".into(),
                ));
            }
        }
        // SC-J09, the substrate half: the producing chain entry stands
        // BEFORE the item is written — memory is not trusted to survive
        // until homecoming (§4.2).
        let produced_entry_stands: bool = sqlx::query_scalar(
            r#"SELECT EXISTS(SELECT 1 FROM provenance_chains
               WHERE chain_ref = $1 AND produced ? $2)"#,
        )
        .bind(item_ref)
        .bind(item_ref.to_string())
        .fetch_one(&self.pool)
        .await?;
        if !produced_entry_stands {
            return Err(StoreError::ProvenanceIncomplete(format!(
                "no chain entry produces item {item_ref}; chain-append is in-flight, \
                 never reconstructed (§4.2, SC-J09)"
            )));
        }
        let row = sqlx::query(
            r#"INSERT INTO quarantine_items
                 (item_ref, origin_job_ref, mandate_ref, brief_ref, filename, declared_type,
                  content, schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, 'QuarantineItem', $8, $9)
               ON CONFLICT (item_ref) DO NOTHING
               RETURNING *"#,
        )
        .bind(item_ref)
        .bind(job_id)
        .bind(draft.mandate_ref)
        .bind(draft.brief_ref)
        .bind(&draft.filename)
        .bind(&draft.declared_type)
        .bind(&draft.content)
        .bind(RECORD_SCHEMA_VERSION)
        .bind(job_id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        let item = match row {
            Some(row) => Self::quarantine_from_row(&row)?,
            // Law I.3: a retried deposit converges on the standing item.
            None => self.get_quarantine_item(item_ref).await?,
        };
        // The log names the PERSISTED item's metadata, not the incoming
        // draft's: on a converging redeposit the standing item wins (its
        // substance is frozen), so a draft that differs from it must never
        // be what the record shows.
        self.append_log(
            &item_ref.to_string(),
            LogEvent::Quarantined,
            &serde_json::json!({ "filename": item.filename, "declared_type": item.declared_type }),
            Severity::Info,
            &job_id.to_string(),
        )
        .await?;
        Ok(item)
    }

    async fn get_quarantine_item(&self, item_ref: Uuid) -> Result<QuarantineItem, StoreError> {
        let row = sqlx::query("SELECT * FROM quarantine_items WHERE item_ref = $1")
            .bind(item_ref)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StoreError::NotFound(format!("no such quarantine item {item_ref}")))?;
        Self::quarantine_from_row(&row)
    }

    async fn quarantine_items_for(
        &self,
        mandate_ref: Uuid,
    ) -> Result<Vec<QuarantineItem>, StoreError> {
        let rows = sqlx::query(
            "SELECT * FROM quarantine_items WHERE mandate_ref = $1 ORDER BY held_since, item_ref",
        )
        .bind(mandate_ref)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(Self::quarantine_from_row).collect()
    }

    async fn record_scan_verdict(
        &self,
        item_ref: Uuid,
        verdict: ScanVerdictKind,
        engine: &ScanEngine,
    ) -> Result<ScanVerdict, StoreError> {
        self.get_quarantine_item(item_ref).await?;
        if engine.alias.trim().is_empty() || engine.version.trim().is_empty() {
            return Err(StoreError::ValidationFailed(
                "a verdict names its engine by alias and version (A.12; Law XV.1)".into(),
            ));
        }
        let scan_id = Uuid::now_v7();
        let mut tx = self.pool.begin().await?;
        Self::set_actor_class(&mut tx, DEACON_CLASS).await?;
        let row = sqlx::query(
            r#"INSERT INTO scan_verdicts
                 (scan_id, item_ref, verdict, engine_alias, engine_version, signature_rev,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, $6, 'ScanVerdict', $7, $8)
               RETURNING *"#,
        )
        .bind(scan_id)
        .bind(item_ref)
        .bind(verdict.as_str())
        .bind(&engine.alias)
        .bind(&engine.version)
        .bind(&engine.signature_rev)
        .bind(RECORD_SCHEMA_VERSION)
        .bind(DEACON_OFFICE_ID)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query(
            "UPDATE quarantine_items SET scan_ref = $2, revision = revision + 1 WHERE item_ref = $1",
        )
        .bind(item_ref)
        .bind(scan_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        let severity = if verdict == ScanVerdictKind::Clean {
            Severity::Info
        } else {
            Severity::Warning
        };
        self.append_log(
            &item_ref.to_string(),
            LogEvent::ScanRecorded,
            &serde_json::json!({ "verdict": verdict.as_str(), "engine": engine.alias }),
            severity,
            DEACON_OFFICE_ID,
        )
        .await?;
        Self::verdict_from_row(&row)
    }

    async fn latest_verdict(&self, item_ref: Uuid) -> Result<Option<ScanVerdict>, StoreError> {
        let row = sqlx::query(
            r#"SELECT * FROM scan_verdicts WHERE item_ref = $1
               ORDER BY scanned_at DESC, scan_id DESC LIMIT 1"#,
        )
        .bind(item_ref)
        .fetch_optional(&self.pool)
        .await?;
        row.as_ref().map(Self::verdict_from_row).transpose()
    }

    async fn assemble_manifest(
        &self,
        mandate_ref: Uuid,
        trip_job_ref: Uuid,
    ) -> Result<Manifest, StoreError> {
        // One Manifest per mandate-trip: re-assembly converges.
        if let Some(row) = sqlx::query("SELECT * FROM manifests WHERE trip_job_ref = $1")
            .bind(trip_job_ref)
            .fetch_optional(&self.pool)
            .await?
        {
            let existing = Self::manifest_from_row(&row)?;
            if existing.mandate_ref != mandate_ref {
                return Err(StoreError::ValidationFailed(format!(
                    "trip {trip_job_ref} already has a Manifest under mandate {}; \
                     one Manifest serves one mandate-trip (ruling G11)",
                    existing.mandate_ref
                )));
            }
            return Ok(existing);
        }
        self.get_mandate(mandate_ref).await?;
        let trip = self.get_job(trip_job_ref).await?;
        if trip.brief_ref != Some(mandate_ref) {
            return Err(StoreError::ValidationFailed(format!(
                "job {trip_job_ref} is not the mandate's own trip; Manifests are never \
                 pooled across trips (ruling G11)"
            )));
        }
        let items = sqlx::query(
            r#"SELECT * FROM quarantine_items
               WHERE mandate_ref = $1 AND origin_job_ref = $2
               ORDER BY held_since, item_ref"#,
        )
        .bind(mandate_ref)
        .bind(trip_job_ref)
        .fetch_all(&self.pool)
        .await?;
        let mut entries = Vec::with_capacity(items.len());
        for row in &items {
            let item = Self::quarantine_from_row(row)?;
            let verdict = self.latest_verdict(item.item_ref).await?;
            let chain = self.chain_for(item.item_ref).await?;
            entries.push(serde_json::json!({
                "item_ref": item.item_ref.to_string(),
                "filename": item.filename,
                "verdict": verdict.as_ref().map(|v| v.verdict.as_str()).unwrap_or("UNSCANNED"),
                "scan_id": verdict.as_ref().map(|v| v.scan_id.to_string()),
                "chain": chain.iter().map(|e| serde_json::json!({
                    "link_seq": e.link_seq,
                    "kind": e.kind.as_str(),
                    "actor": e.actor_job_ref.to_string(),
                    "prompt_or_reason": e.prompt_or_reason,
                    "produced": e.produced.iter().map(|u| u.to_string()).collect::<Vec<_>>(),
                })).collect::<Vec<_>>(),
            }));
        }
        // SC-I07b — graduated legibility, never blocking, never silent.
        let batch_threshold = self
            .get_config("admission_batch_threshold")
            .await?
            .value
            .as_i64()
            .ok_or_else(|| {
                StoreError::ValidationFailed(
                    "admission_batch_threshold is not an integer; no fabricated default \
                     stands in for it (SC-H07)"
                        .into(),
                )
            })?;
        let window_ms = self
            .get_config("admission_rate_window_ms")
            .await?
            .value
            .as_i64()
            .ok_or_else(|| {
                StoreError::ValidationFailed(
                    "admission_rate_window_ms is not an integer (SC-H07)".into(),
                )
            })?;
        let rate_threshold = self
            .get_config("admission_rate_threshold")
            .await?
            .value
            .as_i64()
            .ok_or_else(|| {
                StoreError::ValidationFailed(
                    "admission_rate_threshold is not an integer (SC-H07)".into(),
                )
            })?;
        let recent_consents: i64 = sqlx::query_scalar(
            r#"SELECT count(*) FROM consent_records
               WHERE scan_ref IS NOT NULL
                 AND decided_at > now() - ($1::double precision * interval '1 millisecond')"#,
        )
        .bind(window_ms as f64)
        .fetch_one(&self.pool)
        .await?;
        let over_batch = entries.len() as i64 > batch_threshold;
        let over_rate = recent_consents > rate_threshold;
        let standing_notice = (over_batch || over_rate).then(|| {
            format!(
                "STANDING NOTICE (SC-I07b): {} items against a batch threshold of \
                 {batch_threshold}; {recent_consents} admission consents in the trailing \
                 window against a threshold of {rate_threshold}. The gate does not block; \
                 it makes the rate legible. Terminal answers: acknowledge, or silence \
                 with suppressed logging (Book II §1).",
                entries.len()
            )
        });
        let manifest_id = Uuid::now_v7();
        let mut tx = self.pool.begin().await?;
        Self::set_actor_class(&mut tx, DEACON_CLASS).await?;
        let row = sqlx::query(
            r#"INSERT INTO manifests
                 (manifest_id, mandate_ref, trip_job_ref, items, standing_notice,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, 'Manifest', $6, $7)
               ON CONFLICT (trip_job_ref) DO NOTHING
               RETURNING *"#,
        )
        .bind(manifest_id)
        .bind(mandate_ref)
        .bind(trip_job_ref)
        .bind(serde_json::Value::Array(entries))
        .bind(&standing_notice)
        .bind(RECORD_SCHEMA_VERSION)
        .bind(DEACON_OFFICE_ID)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        let manifest = match row {
            Some(row) => Self::manifest_from_row(&row)?,
            None => {
                // A racing assembly landed first; converge on it.
                let row = sqlx::query("SELECT * FROM manifests WHERE trip_job_ref = $1")
                    .bind(trip_job_ref)
                    .fetch_one(&self.pool)
                    .await?;
                return Self::manifest_from_row(&row);
            }
        };
        self.append_log(
            &manifest.manifest_id.to_string(),
            LogEvent::ManifestPresented,
            &serde_json::json!({
                "mandate": mandate_ref.to_string(),
                "items": manifest.items.as_array().map(Vec::len).unwrap_or(0),
                "standing_notice": manifest.standing_notice.is_some(),
            }),
            Severity::Info,
            DEACON_OFFICE_ID,
        )
        .await?;
        Ok(manifest)
    }

    async fn get_manifest(&self, manifest_id: Uuid) -> Result<Manifest, StoreError> {
        let row = sqlx::query("SELECT * FROM manifests WHERE manifest_id = $1")
            .bind(manifest_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StoreError::NotFound(format!("no such manifest {manifest_id}")))?;
        Self::manifest_from_row(&row)
    }

    async fn consent_admission(
        &self,
        actor: &str,
        subject_ref: Uuid,
        scope: ConsentScope,
        decision: ConsentDecision,
        scan_ref: Option<Uuid>,
    ) -> Result<Uuid, StoreError> {
        if !matches!(
            decision,
            ConsentDecision::Admitted | ConsentDecision::Rejected
        ) {
            return Err(StoreError::ValidationFailed(format!(
                "{decision} is not a threshold answer; the sovereign admits or rejects (A.12)"
            )));
        }
        // What the consent binds, by scope.
        let item_refs: Vec<Uuid> = match scope {
            ConsentScope::Item => {
                let item = self.get_quarantine_item(subject_ref).await?;
                if decision == ConsentDecision::Admitted {
                    let latest = self.latest_verdict(subject_ref).await?;
                    let Some(latest) = latest else {
                        return Err(StoreError::ValidationFailed(
                            "consent over an unscanned item admits nothing; the Deacon \
                             never presents the unscanned as admissible (Book II §1)"
                                .into(),
                        ));
                    };
                    if scan_ref != Some(latest.scan_id) {
                        return Err(StoreError::ValidationFailed(
                            "an admitting consent names the scan it saw, and it must be \
                             the item's latest (Book II §1)"
                                .into(),
                        ));
                    }
                }
                vec![item.item_ref]
            }
            ConsentScope::Batch => {
                if scan_ref.is_some() {
                    return Err(StoreError::ValidationFailed(
                        "a batch consent binds through its Manifest's listings, not a \
                         single scan (Book II §1)"
                            .into(),
                    ));
                }
                let manifest = self.get_manifest(subject_ref).await?;
                manifest
                    .items
                    .as_array()
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|e| e.get("item_ref").and_then(|v| v.as_str()))
                            .filter_map(|s| Uuid::parse_str(s).ok())
                            .collect()
                    })
                    .unwrap_or_default()
            }
        };
        let consent_id = Uuid::now_v7();
        let mut tx = self.pool.begin().await?;
        Self::set_actor_class(&mut tx, SOVEREIGN_CLASS).await?;
        sqlx::query(
            r#"INSERT INTO consent_records
                 (consent_id, subject_ref, decision, scope, decided_by, scan_ref,
                  schema_name, schema_version, produced_by)
               VALUES ($1, $2, $3, $4, $5, $6, 'ConsentRecord', $7, $5)"#,
        )
        .bind(consent_id)
        .bind(subject_ref)
        .bind(if decision == ConsentDecision::Admitted {
            "ADMITTED"
        } else {
            "REJECTED"
        })
        .bind(scope.as_str())
        .bind(actor)
        .bind(scan_ref)
        .bind(RECORD_SCHEMA_VERSION)
        .execute(&mut *tx)
        .await?;
        for item_ref in &item_refs {
            sqlx::query(
                r#"UPDATE quarantine_items SET consent_ref = $2, revision = revision + 1
                   WHERE item_ref = $1"#,
            )
            .bind(item_ref)
            .bind(consent_id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        let event = if decision == ConsentDecision::Admitted {
            LogEvent::Admitted
        } else {
            LogEvent::Rejected
        };
        for item_ref in &item_refs {
            self.append_log(
                &item_ref.to_string(),
                event,
                &serde_json::json!({ "consent": consent_id.to_string(), "scope": scope.as_str() }),
                Severity::Info,
                actor,
            )
            .await?;
        }
        Ok(consent_id)
    }

    async fn clear_for_admission(&self, item_ref: Uuid) -> Result<QuarantineItem, StoreError> {
        let item = self.get_quarantine_item(item_ref).await?;
        if item.admitted_node_ref.is_some() {
            return Ok(item); // already through the gate; converged
        }
        self.prove_admission_conjunction(&item).await?;
        Ok(item)
    }

    async fn mark_admitted(
        &self,
        item_ref: Uuid,
        node_ref: Uuid,
    ) -> Result<QuarantineItem, StoreError> {
        let item = self.get_quarantine_item(item_ref).await?;
        match item.admitted_node_ref {
            Some(existing) if existing == node_ref => return Ok(item), // converged
            Some(existing) => {
                return Err(StoreError::ValidationFailed(format!(
                    "item {item_ref} was admitted as node {existing}; admission is \
                     recorded exactly once (Law I.3)"
                )));
            }
            None => {}
        }
        self.get_node(node_ref).await?;
        self.prove_admission_conjunction(&item).await?;
        let updated = sqlx::query(
            r#"UPDATE quarantine_items SET admitted_node_ref = $2, revision = revision + 1
               WHERE item_ref = $1 AND admitted_node_ref IS NULL"#,
        )
        .bind(item_ref)
        .bind(node_ref)
        .execute(&self.pool)
        .await?;
        if updated.rows_affected() == 0 {
            // A racing admission landed first; converge iff it agrees.
            let now = self.get_quarantine_item(item_ref).await?;
            return match now.admitted_node_ref {
                Some(existing) if existing == node_ref => Ok(now),
                Some(existing) => Err(StoreError::ValidationFailed(format!(
                    "item {item_ref} was admitted as node {existing}; admission is \
                     recorded exactly once (Law I.3)"
                ))),
                None => Err(StoreError::ValidationFailed(format!(
                    "admission of {item_ref} raced and did not land; re-read and retry (XI.3)"
                ))),
            };
        }
        self.append_log(
            &item_ref.to_string(),
            LogEvent::Admitted,
            &serde_json::json!({ "node": node_ref.to_string() }),
            Severity::Info,
            DEACON_OFFICE_ID,
        )
        .await?;
        self.get_quarantine_item(item_ref).await
    }
}

fn serialize_steps(steps: &[godhead_schemas::Step]) -> serde_json::Value {
    serde_json::Value::Array(
        steps
            .iter()
            .map(|s| {
                serde_json::json!({
                    "step_id": s.step_id,
                    "action": s.action.as_str(),
                    "params": s.params,
                    "expected_output": s.expected_output,
                    "budget_hint_tokens": s.budget_hint_tokens,
                })
            })
            .collect(),
    )
}

fn serialize_criteria(criteria: &[godhead_schemas::AcceptanceCriterion]) -> serde_json::Value {
    serde_json::Value::Array(
        criteria
            .iter()
            .map(|c| {
                serde_json::json!({
                    "criterion": c.criterion,
                    "testable_as": c.testable_as.as_stored(),
                })
            })
            .collect(),
    )
}

fn serialize_sources(sources: &[SourceDraw]) -> serde_json::Value {
    serde_json::Value::Array(
        sources
            .iter()
            .map(|s| {
                serde_json::json!({
                    "matrix_ref": s.matrix_ref.to_string(),
                    "draw_count": s.draw_count,
                    "canon_associated": s.canon_associated,
                })
            })
            .collect(),
    )
}

fn serialize_items(items: &[godhead_schemas::ReturnItem]) -> serde_json::Value {
    serde_json::Value::Array(
        items
            .iter()
            .map(|i| {
                serde_json::json!({
                    "item_ref": i.item_ref.to_string(),
                    "kind": i.kind.as_str(),
                    "provenance_ref": i.provenance_ref.to_string(),
                })
            })
            .collect(),
    )
}

fn serialize_completion(entries: &[godhead_schemas::CompletionEntry]) -> serde_json::Value {
    serde_json::Value::Array(
        entries
            .iter()
            .map(|e| {
                serde_json::json!({
                    "criterion_index": e.criterion_index,
                    "passed": e.passed,
                    "evidence_ref": e.evidence_ref.to_string(),
                })
            })
            .collect(),
    )
}

/// The B.2 completion contract, proven against the answered Instruction's
/// stored acceptance_criteria: indices 0..n each answered exactly once
/// (missing/extra/duplicate invalidate); evidence mandatory in every case;
/// `passed` is None iff the criterion is SOVEREIGN_JUDGMENT (§1.3d). A
/// malformed stored criterion fails loudly — never a silent skip (the
/// slice-8 lesson: a check is only as good as the inputs it inspects).
fn validate_completion_contract(
    criteria: &serde_json::Value,
    completion: &[godhead_schemas::CompletionEntry],
) -> Result<(), StoreError> {
    let criteria = criteria.as_array().ok_or_else(|| {
        StoreError::ValidationFailed("stored acceptance_criteria is not an array".into())
    })?;
    let n = criteria.len();
    if completion.len() != n {
        return Err(StoreError::ValidationFailed(format!(
            "completion carries {} entries for {n} criteria — the contract is exactly one each (B.2)",
            completion.len()
        )));
    }
    let mut seen = vec![false; n];
    for entry in completion {
        let Ok(idx) = usize::try_from(entry.criterion_index) else {
            return Err(StoreError::ValidationFailed(format!(
                "criterion_index {} names no criterion (B.2)",
                entry.criterion_index
            )));
        };
        if idx >= n {
            return Err(StoreError::ValidationFailed(format!(
                "criterion_index {idx} is beyond the Instruction's {n} criteria (B.2)"
            )));
        }
        if seen[idx] {
            return Err(StoreError::ValidationFailed(format!(
                "criterion {idx} is answered twice; the contract is exactly one entry each (B.2)"
            )));
        }
        seen[idx] = true;
        if entry.evidence_ref.is_nil() {
            return Err(StoreError::ValidationFailed(format!(
                "criterion {idx} carries a nil evidence_ref; evidence is mandatory in every case (B.2)"
            )));
        }
        let testable_as = criteria[idx]
            .get("testable_as")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                StoreError::ValidationFailed(format!(
                    "stored criterion {idx} carries no testable_as; a malformed criterion never passes silently"
                ))
            })?;
        let sovereign = testable_as == "SOVEREIGN_JUDGMENT";
        match (sovereign, entry.passed) {
            (true, Some(_)) => {
                return Err(StoreError::ValidationFailed(format!(
                    "criterion {idx} is SOVEREIGN_JUDGMENT; its verdict is the sovereign's to render, not the Student's (§1.3d)"
                )));
            }
            (false, None) => {
                return Err(StoreError::ValidationFailed(format!(
                    "criterion {idx} is machine-checkable; the Student renders a verdict (B.2)"
                )));
            }
            _ => {}
        }
    }
    Ok(())
}
