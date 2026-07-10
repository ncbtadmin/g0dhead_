use crate::normalize::DecodeResult;
use crate::{
    classify, normalize, sha256_hex, IntakeError, STAGE_CLASSIFY, STAGE_NORMALIZE, STAGE_RAW_COPY,
    STAGE_RENORMALIZE, STAGE_RESULT_SCHEMA,
};
use godhead_schemas::{
    AgentType, Budgets, Certifies, FlagDraft, JobDraft, JobRecord, JobStatus, Law, NodeDraft,
    NodeRecord, NormalizeOutcome, ReadinessFlag, RefusalDraft, RefusalReason, Validator,
};
use godhead_store::{ArtifactDraft, Store};
use semver::Version;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// The deterministic intake pipe. Raw bytes land under `<data_root>/raw/`,
/// derivatives under `<data_root>/derived/`; the store holds references
/// (doc 3 §1.3). Every stage is a full Book I lifecycle.
pub struct IntakePipe<'s, S> {
    store: &'s S,
    data_root: PathBuf,
}

impl<'s, S: Store> IntakePipe<'s, S> {
    pub fn new(store: &'s S, data_root: impl Into<PathBuf>) -> std::io::Result<Self> {
        let data_root = data_root.into();
        std::fs::create_dir_all(data_root.join("raw"))?;
        std::fs::create_dir_all(data_root.join("derived"))?;
        Ok(Self { store, data_root })
    }

    pub fn store(&self) -> &'s S {
        self.store
    }

    pub fn data_root(&self) -> &Path {
        &self.data_root
    }

    fn stage_job_draft(node_id: Uuid) -> JobDraft {
        JobDraft {
            agent_type: AgentType::Intake,
            auditor_name: None,
            tier: None,
            input_refs: vec![node_id],
            env_ref: None,
            brief_ref: None,
            // Modelless floor labor (A.2): no endpoint, ever.
            endpoint_alias: None,
            manual_version: Version::new(1, 0, 0),
            budgets: Budgets {
                max_wall_ms: 120_000,
                max_tool_calls: 10,
                max_tokens: 1,
            },
        }
    }

    async fn spawn_running(&self, node_id: Uuid) -> Result<JobRecord, IntakeError> {
        let job = self
            .store
            .create_job(&Self::stage_job_draft(node_id))
            .await?;
        let job = self
            .store
            .transition_job(job.job_id, job.revision, JobStatus::Leased)
            .await?;
        Ok(self
            .store
            .transition_job(job.job_id, job.revision, JobStatus::Running)
            .await?)
    }

    /// artifact → WRITTEN → flag → release lease → TERMINATED: the back half
    /// of Law I.1's lifecycle, identical for every stage.
    async fn finish_stage(
        &self,
        job_id: Uuid,
        node_id: Uuid,
        lease_id: Uuid,
        stage: &str,
        outcome: &str,
        sha256: Option<String>,
    ) -> Result<ReadinessFlag, IntakeError> {
        let artifact = self
            .store
            .write_artifact(
                job_id,
                "result",
                &ArtifactDraft {
                    schema_name: STAGE_RESULT_SCHEMA.to_string(),
                    schema_version: Version::new(1, 0, 0),
                    payload: serde_json::json!({
                        "node_id": node_id.to_string(),
                        "stage": stage,
                        "outcome": outcome,
                        "sha256": sha256,
                    }),
                },
            )
            .await?;
        let job = self.store.get_job(job_id).await?;
        self.store
            .transition_job(job_id, job.revision, JobStatus::Written)
            .await?;
        let flag = self
            .store
            .write_flag(
                job_id,
                &FlagDraft {
                    stage: stage.to_string(),
                    certifies: Certifies {
                        output_slots: vec!["result".to_string()],
                        revisions: vec![artifact.revision],
                    },
                    validator: Validator {
                        id: "godhead-intake/registry".to_string(),
                        version: "1.0.0".to_string(),
                    },
                },
            )
            .await?;
        self.store.release_lease(job_id, lease_id).await?;
        let job = self.store.get_job(job_id).await?;
        self.store
            .transition_job(job_id, job.revision, JobStatus::Terminated)
            .await?;
        Ok(flag)
    }

    /// The node a stage flag speaks about: the flagging job's input ref.
    /// Pre-validation routing only — the successor's read_certified is the
    /// Law III.3 witness check.
    pub async fn node_ref_of(&self, flag: &ReadinessFlag) -> Result<Uuid, IntakeError> {
        let flag_job = flag.job_id.ok_or_else(|| {
            IntakeError::NotFound(format!(
                "flag {} is office-authored, not a stage flag",
                flag.flag_id
            ))
        })?;
        let owner = self.store.get_job(flag_job).await?;
        owner.input_refs.first().copied().ok_or_else(|| {
            IntakeError::NotFound(format!("flag {} has no node input_ref", flag.flag_id))
        })
    }

    /// The human commit (doc 2 §2.1–2.2): raw copied into persistence
    /// exactly once, first log in the same act, RAW_COPY flagged.
    pub async fn commit_file(&self, filename: &str, bytes: &[u8]) -> Result<Uuid, IntakeError> {
        self.commit_file_with_id(Uuid::now_v7(), filename, bytes)
            .await
    }

    /// As `commit_file`, but the caller chooses the node id — so a keyed
    /// intake (an admission) can derive a STABLE id from its subject and
    /// converge on retry instead of minting a duplicate atom (F1;
    /// `Deacon::admit`). The one-active-lease-per-subject rule serializes two
    /// concurrent runs on the same id, and the node write is keyed on it.
    pub async fn commit_file_with_id(
        &self,
        node_id: Uuid,
        filename: &str,
        bytes: &[u8],
    ) -> Result<Uuid, IntakeError> {
        let job = self.spawn_running(node_id).await?;
        let lease = self
            .store
            .acquire_lease(job.job_id, node_id, 60_000)
            .await?;
        let filetype = filename
            .rsplit_once('.')
            .map(|(_, ext)| ext.to_ascii_lowercase())
            .unwrap_or_default();
        let raw_rel = format!("raw/{node_id}");
        std::fs::write(self.data_root.join(&raw_rel), bytes)?;
        let sha = sha256_hex(bytes);
        self.store
            .create_node(
                job.job_id,
                node_id,
                &NodeDraft {
                    filename: filename.to_string(),
                    filetype,
                    size_bytes: i64::try_from(bytes.len()).expect("file size fits i64"),
                    raw_path: raw_rel,
                    raw_sha256: sha.clone(),
                },
            )
            .await?;
        self.finish_stage(
            job.job_id,
            node_id,
            lease.lease_id,
            STAGE_RAW_COPY,
            "OK",
            Some(sha),
        )
        .await?;
        Ok(node_id)
    }

    /// Reads the atom from disk, verifies it against the recorded checksum
    /// (doc 3 §4.3), decodes, and records the outcome on the node. A
    /// checksum mismatch is a Law VII refusal, never a silent re-derive.
    async fn normalize_node(
        &self,
        job_id: Uuid,
        node: &NodeRecord,
    ) -> Result<(&'static str, Option<String>), IntakeError> {
        let bytes = std::fs::read(self.data_root.join(&node.raw_path))?;
        if sha256_hex(&bytes) != node.raw_sha256 {
            self.store
                .refuse(
                    job_id,
                    &RefusalDraft {
                        law: Law::V,
                        reason: RefusalReason::ValidationFailed,
                        subject_refs: vec![node.node_id.to_string()],
                        detail: "raw bytes on disk do not match the recorded atom checksum"
                            .to_string(),
                        preserved_refs: vec![node.raw_path.clone()],
                    },
                )
                .await?;
            return Err(IntakeError::Refused(format!(
                "node {}: atom checksum mismatch — refused and preserved (Law VII)",
                node.node_id
            )));
        }
        let (outcome, label, sha) = match normalize::normalize(&node.filetype, &bytes) {
            DecodeResult::Text(text) => {
                let rel = format!("derived/{}.txt", node.node_id);
                std::fs::write(self.data_root.join(&rel), text.as_bytes())?;
                let dsha = sha256_hex(text.as_bytes());
                (
                    NormalizeOutcome::Normalized {
                        derivative_path: rel,
                        derivative_sha256: dsha.clone(),
                    },
                    "NORMALIZED",
                    Some(dsha),
                )
            }
            DecodeResult::Failed(reason) => (
                NormalizeOutcome::DecodeFailed { reason },
                "DECODE_FAILED",
                None,
            ),
            DecodeResult::Unsupported => (
                NormalizeOutcome::Unsupported {
                    notice: format!(
                        "filetype '{}' is outside the v1 supported set; stored raw awaiting future support (doc 2 §2.4)",
                        node.filetype
                    ),
                },
                "UNSUPPORTED",
                None,
            ),
        };
        self.store
            .set_node_derivative(job_id, node.node_id, node.revision, &outcome)
            .await?;
        Ok((label, sha))
    }

    /// The NORMALIZE stage, consuming a RAW_COPY flag.
    pub async fn run_normalize(&self, prior: &ReadinessFlag) -> Result<Uuid, IntakeError> {
        let node_id = self.node_ref_of(prior).await?;
        let job = self.spawn_running(node_id).await?;
        // Law III.3: the flag is testimony; the state is re-validated.
        self.store.read_certified(job.job_id, prior.flag_id).await?;
        let lease = self
            .store
            .acquire_lease(job.job_id, node_id, 60_000)
            .await?;
        let node = self.store.get_node(node_id).await?;
        let (label, sha) = self.normalize_node(job.job_id, &node).await?;
        self.finish_stage(
            job.job_id,
            node_id,
            lease.lease_id,
            STAGE_NORMALIZE,
            label,
            sha,
        )
        .await?;
        Ok(node_id)
    }

    /// The CLASSIFY stage, consuming a NORMALIZE flag. Runs for failed and
    /// unsupported nodes too — they rest classified `unclassified`, stored
    /// and surfaced, never rejected (doc 2 §2.4).
    pub async fn run_classify(&self, prior: &ReadinessFlag) -> Result<Uuid, IntakeError> {
        let node_id = self.node_ref_of(prior).await?;
        let job = self.spawn_running(node_id).await?;
        self.store.read_certified(job.job_id, prior.flag_id).await?;
        let lease = self
            .store
            .acquire_lease(job.job_id, node_id, 60_000)
            .await?;
        let node = self.store.get_node(node_id).await?;
        let classification = classify::classification(&node.filetype);
        self.store
            .set_node_classification(job.job_id, node_id, node.revision, &classification)
            .await?;
        self.finish_stage(
            job.job_id,
            node_id,
            lease.lease_id,
            STAGE_CLASSIFY,
            "OK",
            None,
        )
        .await?;
        Ok(node_id)
    }

    /// Human-invoked derivative regeneration (doc 3 §4.3): discard the
    /// derivative, re-derive from the preserved raw. The atom is the ground
    /// truth; the derivative is always reproducible from it.
    pub async fn renormalize(&self, node_id: Uuid) -> Result<NodeRecord, IntakeError> {
        let job = self.spawn_running(node_id).await?;
        let lease = self
            .store
            .acquire_lease(job.job_id, node_id, 60_000)
            .await?;
        let node = self.store.get_node(node_id).await?;
        if let Some(old) = &node.derivative_path {
            let path = self.data_root.join(old);
            if path.exists() {
                std::fs::remove_file(path)?;
            }
        }
        let (label, sha) = self.normalize_node(job.job_id, &node).await?;
        self.finish_stage(
            job.job_id,
            node_id,
            lease.lease_id,
            STAGE_RENORMALIZE,
            label,
            sha,
        )
        .await?;
        Ok(self.store.get_node(node_id).await?)
    }
}
