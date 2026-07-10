//! godhead-audit — the trial of a Postulant (Dogma Book II §2, Law VI).
//!
//! The sovereign invokes audit; Gabriel and Lucy are spawned independently
//! with identical input and no sight of each other's work; the supervisor
//! certifies the AND-barrier; Reconciliation produces one Joint Proposal;
//! the sovereign answers; a Notary makes it so. Every arrow is a validated
//! handoff — no stage may be skipped, merged, or performed by fiat.
//!
//! v1 auditors and the Reconciler run on the deterministic floor
//! (SLICE_05 §3): real structural rules, evidence-bound, no reasoner.
//! Reasoner-assisted judgment plugs into the same jobs later.

use godhead_schemas::{
    AgentType, Amendment, AmendmentKind, AuditReport, AuditReportDraft, AuditorKind, AuditorName,
    Budgets, Certifies, Claim, ClaimSeverity, FlagDraft, JobRecord, JobStatus, Law, LogEvent,
    MatrixRecord, MatrixStatus, ProposalDraft, RefusalDraft, RefusalReason, ReportKind,
    SchemaRegistry, Severity, Validator, Verdict,
};
use godhead_store::{ArtifactDraft, Store, StoreError};
use semver::{Version, VersionReq};
use thiserror::Error;
use uuid::Uuid;

/// The stage an auditor flags.
pub const STAGE_REPORT: &str = "auditor:report";
/// The stage Reconciliation flags.
pub const STAGE_RECONCILE: &str = "reconciler:proposal";
/// Pointer-artifact schemas.
pub const REPORT_POINTER_SCHEMA: &str = "audit.report_pointer";
pub const PROPOSAL_POINTER_SCHEMA: &str = "audit.proposal_pointer";

#[derive(Debug, Error)]
pub enum AuditError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("REFUSED: {0}")]
    Refused(String),
}

/// Adds the audit schemas to a build registry (Law II.4).
pub fn register_into(reg: &mut SchemaRegistry) {
    fn pointer(payload: &serde_json::Value) -> Result<(), String> {
        let obj = payload.as_object().ok_or("payload must be an object")?;
        let id = obj
            .get("ref")
            .and_then(|v| v.as_str())
            .ok_or("field 'ref' (string) is required")?;
        uuid::Uuid::parse_str(id).map_err(|_| "ref must be a uuid".to_string())?;
        Ok(())
    }
    reg.register(
        REPORT_POINTER_SCHEMA,
        VersionReq::parse("^1.0").expect("valid req"),
        pointer,
    );
    reg.register(
        PROPOSAL_POINTER_SCHEMA,
        VersionReq::parse("^1.0").expect("valid req"),
        pointer,
    );
}

fn auditor_draft(matrix_id: Uuid, who: AuditorName) -> godhead_schemas::JobDraft {
    godhead_schemas::JobDraft {
        agent_type: AgentType::Auditor,
        auditor_name: Some(who),
        tier: None,
        // Identical input: the matrix, nothing else (Book II §2 step 2).
        input_refs: vec![matrix_id],
        env_ref: None,
        brief_ref: None,
        endpoint_alias: None, // floor auditors, v1
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 300_000,
            max_tool_calls: 100,
            max_tokens: 1,
        },
    }
}

fn reconciler_draft(matrix_id: Uuid, report_refs: [Uuid; 2]) -> godhead_schemas::JobDraft {
    godhead_schemas::JobDraft {
        agent_type: AgentType::Reconciler,
        auditor_name: None,
        tier: None,
        input_refs: vec![matrix_id, report_refs[0], report_refs[1]],
        env_ref: None,
        brief_ref: None,
        endpoint_alias: None,
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 300_000,
            max_tool_calls: 100,
            max_tokens: 1,
        },
    }
}

async fn spawn_running<S: Store>(
    store: &S,
    draft: &godhead_schemas::JobDraft,
) -> Result<JobRecord, AuditError> {
    let job = store.create_job(draft).await?;
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await?;
    Ok(store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await?)
}

/// artifact-pointer → WRITTEN → flag → TERMINATED: the lawful close shared
/// by auditor and reconciler labors.
async fn close_with_pointer<S: Store>(
    store: &S,
    job_id: Uuid,
    stage: &str,
    schema: &str,
    target: Uuid,
) -> Result<(), AuditError> {
    let artifact = store
        .write_artifact(
            job_id,
            "result",
            &ArtifactDraft {
                schema_name: schema.to_string(),
                schema_version: Version::new(1, 0, 0),
                payload: serde_json::json!({ "ref": target.to_string() }),
            },
        )
        .await?;
    let job = store.get_job(job_id).await?;
    store
        .transition_job(job_id, job.revision, JobStatus::Written)
        .await?;
    store
        .write_flag(
            job_id,
            &FlagDraft {
                stage: stage.to_string(),
                certifies: Certifies {
                    output_slots: vec!["result".to_string()],
                    revisions: vec![artifact.revision],
                },
                validator: Validator {
                    id: "godhead-audit/registry".to_string(),
                    version: "1.0.0".to_string(),
                },
            },
        )
        .await?;
    let job = store.get_job(job_id).await?;
    store
        .transition_job(job_id, job.revision, JobStatus::Terminated)
        .await?;
    Ok(())
}

/// The floor auditor's judgment over the matrix's links.
/// Gabriel reinforces the provably good: every weighted bond, affirmed
/// with the bond itself as evidence. Lucy accuses: every weightless bond
/// is a structural fault, severity high. Neither can lie — every claim
/// cites records the store re-resolves at write (the truth-binding).
async fn floor_claims<S: Store>(
    store: &S,
    matrix: &MatrixRecord,
    who: AuditorKind,
) -> Result<Vec<Claim>, AuditError> {
    let links = store
        .links_by_category(&matrix.category, None)
        .await?
        .into_iter()
        .filter(|l| matrix.link_refs.contains(&l.link_id))
        .collect::<Vec<_>>();
    let claims = match who {
        AuditorKind::Gabriel => links
            .iter()
            .filter(|l| l.weight > 0.0)
            .map(|l| Claim {
                claim: format!(
                    "the bond {} carries weight {:.4}: cohesive, sound, correctly held",
                    l.link_id, l.weight
                ),
                evidence_refs: vec![l.link_id],
                severity: None,
            })
            .collect(),
        AuditorKind::Lucy => links
            .iter()
            .filter(|l| l.weight <= 0.0)
            .map(|l| Claim {
                claim: format!(
                    "the bond {} is weightless: membership without influence is structural error",
                    l.link_id
                ),
                evidence_refs: vec![l.link_id],
                severity: Some(ClaimSeverity::High.as_str().to_string()),
            })
            .collect(),
    };
    Ok(claims)
}

/// Law VII for the trial's labors: a halt after RUNNING ends in a refusal
/// on the record — never a job stranded live — and a failed refusal write
/// propagates as a hard error, never swallowed (SC-E05). BudgetExceeded is
/// the one lawful skip: the store already enacted that refusal itself
/// (already-recorded, not failed-to-record — G5). The persisted detail
/// names the halt's stable code token, never the error's own text.
async fn refuse_labor<S: Store>(
    store: &S,
    job_id: Uuid,
    subject: Uuid,
    err: &AuditError,
) -> Result<(), StoreError> {
    if matches!(err, AuditError::Store(StoreError::BudgetExceeded(_))) {
        return Ok(());
    }
    let (law, reason) = match err {
        AuditError::Store(StoreError::LeaseConflict(_)) => (Law::XI, RefusalReason::LeaseConflict),
        AuditError::Store(StoreError::FlagUntrusted(_)) => (Law::III, RefusalReason::FlagUntrusted),
        _ => godhead_schemas::stage_code(),
    };
    let token = err.to_string();
    let token = token.split(':').next().unwrap_or("UNNAMED");
    store
        .refuse(
            job_id,
            &RefusalDraft {
                law,
                reason,
                subject_refs: vec![subject.to_string()],
                detail: format!(
                    "the trial labor halted after RUNNING ({token}); the job ends refused, \
                     never stranded (Law VII)"
                ),
                preserved_refs: vec![],
            },
        )
        .await?;
    Ok(())
}

/// One auditor's whole life: spawn, read the matrix, judge on the floor,
/// file under the truth-binding, flag, die — or refuse (Law VII); the job
/// never strands live. Public so the barrier's one-report-held case is
/// testable; production flows through `invoke_audit`.
pub async fn run_auditor<S: Store>(
    store: &S,
    matrix_id: Uuid,
    who: AuditorKind,
) -> Result<AuditReport, AuditError> {
    let name = match who {
        AuditorKind::Gabriel => AuditorName::Gabriel,
        AuditorKind::Lucy => AuditorName::Lucy,
    };
    let job = spawn_running(store, &auditor_draft(matrix_id, name)).await?;
    match auditor_labor(store, job.job_id, matrix_id, who).await {
        Ok(report) => Ok(report),
        Err(err) => {
            refuse_labor(store, job.job_id, matrix_id, &err).await?;
            Err(err)
        }
    }
}

async fn auditor_labor<S: Store>(
    store: &S,
    job_id: Uuid,
    matrix_id: Uuid,
    who: AuditorKind,
) -> Result<AuditReport, AuditError> {
    let matrix = store.get_matrix(matrix_id).await?;
    let claims = floor_claims(store, &matrix, who).await?;
    let report = store
        .file_audit_report(
            job_id,
            &AuditReportDraft {
                matrix_ref: matrix_id,
                matrix_revision: matrix.revision,
                auditor: who,
                kind: match who {
                    AuditorKind::Gabriel => ReportKind::Affirmation,
                    AuditorKind::Lucy => ReportKind::Indictment,
                },
                claims,
            },
        )
        .await?;
    close_with_pointer(
        store,
        job_id,
        STAGE_REPORT,
        REPORT_POINTER_SCHEMA,
        report.report_id,
    )
    .await?;
    Ok(report)
}

/// The sovereign invokes audit on a Postulant (Law IV.4 — human-reserved;
/// the actor string is the signature). Spawns Gabriel and Lucy
/// independently: identical input, no channel, no sight of each other.
///
/// Resumable: a re-invocation after a partial failure runs only the
/// auditor whose report is missing (the unique constraint on reports is
/// the arbiter); a fully-reported revision returns the standing pair. A
/// crashed trial can therefore always be carried to the barrier.
pub async fn invoke_audit<S: Store>(
    store: &S,
    actor: &str,
    matrix_id: Uuid,
) -> Result<(AuditReport, AuditReport), AuditError> {
    let matrix = store.get_matrix(matrix_id).await?;
    if matrix.status != MatrixStatus::Postulant {
        return Err(AuditError::Refused(format!(
            "audit tries Postulants; matrix {matrix_id} is {} (Law VI.3)",
            matrix.status
        )));
    }
    let existing = store.audit_reports_for(matrix_id, matrix.revision).await?;
    let prior_gabriel = existing
        .iter()
        .find(|r| r.auditor == AuditorKind::Gabriel)
        .cloned();
    let prior_lucy = existing
        .iter()
        .find(|r| r.auditor == AuditorKind::Lucy)
        .cloned();

    if prior_gabriel.is_none() && prior_lucy.is_none() {
        // A fresh trial: open it on the record, and consume the
        // audit-eligibility flag that surfaced it, where one stands (the
        // flag is testimony — the POSTULANT record is the authority, so a
        // missing flag never blocks the sovereign's invocation).
        consume_eligibility_flag(store, matrix_id).await?;
        store
            .write_log(
                &matrix_id.to_string(),
                LogEvent::AuditOpened,
                &serde_json::json!({
                    "matrix_revision": matrix.revision,
                    "audit_depth": matrix.audit_depth,
                    "invoked_by": actor,
                }),
                Severity::Info,
            )
            .await?;
    }
    let gabriel = match prior_gabriel {
        Some(report) => report,
        None => run_auditor(store, matrix_id, AuditorKind::Gabriel).await?,
    };
    let lucy = match prior_lucy {
        Some(report) => report,
        None => run_auditor(store, matrix_id, AuditorKind::Lucy).await?,
    };
    Ok((gabriel, lucy))
}

/// Marks the matrix's audit-eligibility flag CONSUMED when one stands
/// ACTIVE — the surfacing aid is spent by the invocation it summoned.
async fn consume_eligibility_flag<S: Store>(store: &S, matrix_id: Uuid) -> Result<(), AuditError> {
    let flags = store.list_active_flags("aggregator:audit_eligible").await?;
    for flag in flags {
        let Some(job_id) = flag.job_id else { continue };
        let Ok(artifact) = store.read_artifact(job_id, "emergence").await else {
            continue;
        };
        if artifact.payload.get("matrix_id").and_then(|v| v.as_str())
            == Some(matrix_id.to_string().as_str())
        {
            let _ = store
                .supersede_flag(
                    flag.flag_id,
                    flag.revision,
                    godhead_schemas::FlagStatus::Consumed,
                )
                .await;
        }
    }
    Ok(())
}

/// Reconciliation (Book II §2 step 4): released only by the certified
/// barrier, consuming both reports and the store, producing one Joint
/// Proposal. Floor rule: Lucy's high-severity link indictments become the
/// AMEND set, exactly; none ⇒ COMMIT; an empty matrix ⇒ REJECT.
pub async fn reconcile<S: Store>(
    store: &S,
    matrix_id: Uuid,
) -> Result<godhead_schemas::JointProposal, AuditError> {
    let matrix = store.get_matrix(matrix_id).await?;
    if !store
        .audit_barrier_certified(matrix_id, matrix.revision)
        .await?
    {
        return Err(AuditError::Refused(format!(
            "Reconciliation is not dispatchable: the barrier has not certified matrix {matrix_id} rev {} (doc 3 §3.3)",
            matrix.revision
        )));
    }
    let reports = store.audit_reports_for(matrix_id, matrix.revision).await?;
    let [gabriel, lucy] = match &reports[..] {
        [g, l] if g.auditor == AuditorKind::Gabriel => [g, l],
        [l, g] => [g, l],
        _ => {
            return Err(AuditError::Refused(
                "reconciliation requires exactly two reports".into(),
            ))
        }
    };
    let job = spawn_running(
        store,
        &reconciler_draft(matrix_id, [gabriel.report_id, lucy.report_id]),
    )
    .await?;
    let labor = reconciler_labor(
        store,
        job.job_id,
        &matrix,
        gabriel.report_id,
        lucy.report_id,
    );
    match labor.await {
        Ok(proposal) => Ok(proposal),
        Err(err) => {
            refuse_labor(store, job.job_id, matrix_id, &err).await?;
            Err(err)
        }
    }
}

async fn reconciler_labor<S: Store>(
    store: &S,
    job_id: Uuid,
    matrix: &MatrixRecord,
    gabriel_report_id: Uuid,
    lucy_report_id: Uuid,
) -> Result<godhead_schemas::JointProposal, AuditError> {
    // Both reports re-read through the barrier-gated path (Law III.3: the
    // state is the witness).
    let lucy_report = store.read_audit_report(job_id, lucy_report_id).await?;
    store.read_audit_report(job_id, gabriel_report_id).await?;

    let indicted_links: Vec<Uuid> = lucy_report
        .claims
        .iter()
        .filter(|c| c.severity.as_deref() == Some("high"))
        .flat_map(|c| c.evidence_refs.iter().copied())
        .filter(|r| matrix.link_refs.contains(r))
        .collect();

    let (verdict, changes, reasons) = if matrix.link_refs.is_empty() {
        (
            Verdict::Reject,
            Vec::new(),
            vec!["an empty matrix cannot profess: no bonds, no form".to_string()],
        )
    } else if indicted_links.is_empty() {
        (Verdict::Commit, Vec::new(), Vec::new())
    } else {
        (
            Verdict::Amend,
            indicted_links
                .iter()
                .map(|link| Amendment {
                    kind: AmendmentKind::RemoveLink.as_str().to_string(),
                    subject_ref: *link,
                })
                .collect(),
            Vec::new(),
        )
    };

    let proposal = store
        .file_joint_proposal(
            job_id,
            &ProposalDraft {
                matrix_ref: matrix.matrix_id,
                matrix_revision: matrix.revision,
                report_refs: [gabriel_report_id, lucy_report_id],
                verdict,
                changes,
                reasons,
            },
        )
        .await?;
    close_with_pointer(
        store,
        job_id,
        STAGE_RECONCILE,
        PROPOSAL_POINTER_SCHEMA,
        proposal.proposal_id,
    )
    .await?;
    Ok(proposal)
}
