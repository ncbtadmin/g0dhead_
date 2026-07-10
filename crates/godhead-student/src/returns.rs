//! The Student's VALIDATE_OUT (Handbook §3.1): the completion contract,
//! proven before anything persists. The same contract is enforced again
//! inside the store — the double-validation covenant holds at both ends,
//! and a Return that does not validate never flags, never poisons.

use crate::{StudentError, RETURN_POINTER_SCHEMA, SUPPORTED_CONCORDAT};
use godhead_schemas::{
    AgentType, Budgets, CompletionEntry, EnvKind, EnvStatus, EnvironmentRecord, FlagDraft,
    JobDraft, JobStatus, RefusalDraft, ReturnDraft, ReturnManifest,
};
use godhead_store::{Store, StoreError};
use semver::{Version, VersionReq};
use uuid::Uuid;

/// Why the Return failed its VALIDATE_OUT — the Student-particular content.
///
/// `clause` is a stable token naming the gate that fired; it is the ONLY
/// part that reaches a persisted refusal detail. `detail` may carry
/// draft-shaped text (a version string, a uuid the caller chose) and is
/// therefore ephemeral feedback only — persisted fields never echo the
/// emission (Law XV; the slice-6 doctrine).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnFailure {
    pub clause: &'static str,
    pub detail: String,
}

impl std::fmt::Display for ReturnFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.detail)
    }
}

fn fail(clause: &'static str, detail: impl Into<String>) -> ReturnFailure {
    ReturnFailure {
        clause,
        detail: detail.into(),
    }
}

/// Proves the Return against the answered Instruction: the Instruction
/// resolves and is certified, the Concordat citation is within the
/// Student's declared range AND names an adopted version (every cited
/// version stays retrievable forever — SC-K03's covenant), the room is a
/// Student's, and the completion contract holds criterion by criterion.
pub async fn validate_return<S: Store>(
    store: &S,
    draft: &ReturnDraft,
) -> Result<Result<(), ReturnFailure>, StoreError> {
    let instruction = match store.get_instruction(draft.instruction_ref).await {
        Ok(i) => i,
        Err(StoreError::NotFound(_)) => {
            return Ok(Err(fail(
                "instruction-unresolved",
                format!(
                    "the answered Instruction {} does not resolve",
                    draft.instruction_ref
                ),
            )))
        }
        Err(e) => return Err(e),
    };
    if !instruction.flagged {
        return Ok(Err(fail(
            "instruction-unflagged",
            "the answered Instruction is not flagged; unflagged means uncertified (§5.1)",
        )));
    }
    // Version skew (§2.4): the Student's declared RANGE is the rule — a
    // Return lawfully cites the (compatible) version it labors under,
    // which need not equal the Instruction's.
    let supported = VersionReq::parse(SUPPORTED_CONCORDAT).expect("valid req");
    if !supported.matches(&draft.concordat_version) {
        return Ok(Err(fail(
            "concordat-skew",
            format!(
                "Concordat version {} is outside the Student's supported range {SUPPORTED_CONCORDAT} (§2.4)",
                draft.concordat_version
            ),
        )));
    }
    // The citation must also RESOLVE: a certified record citing a
    // never-adopted version would break the retrievability covenant
    // (SC-K03) the moment it flags.
    match store.get_concordat(&draft.concordat_version).await {
        Ok(_) => {}
        // The store names a missing version SchemaMismatch (its Law II
        // vocabulary); either spelling means the citation does not resolve.
        Err(StoreError::NotFound(_)) | Err(StoreError::SchemaMismatch(_)) => {
            return Ok(Err(fail(
                "concordat-unadopted",
                format!(
                    "Concordat version {} was never adopted; a Return cites a retrievable version (SC-K03)",
                    draft.concordat_version
                ),
            )))
        }
        Err(e) => return Err(e),
    }
    match store.get_environment(draft.student_env_ref).await {
        Ok(env) => {
            if env.kind != EnvKind::Student {
                return Ok(Err(fail(
                    "room-kind",
                    format!(
                        "student_env_ref names a {} environment; a Return rises from a Student's room (B.2)",
                        env.kind
                    ),
                )));
            }
            if env.status != EnvStatus::Live {
                return Ok(Err(fail(
                    "room-archived",
                    format!(
                        "the room is {}; an archived room takes no new work (A.8)",
                        env.status
                    ),
                )));
            }
            if env.tier != instruction.target_tier {
                return Ok(Err(fail(
                    "tier-mismatch",
                    format!(
                        "the Return rises from a {} room; the Instruction binds {} (B.1)",
                        env.tier, instruction.target_tier
                    ),
                )));
            }
        }
        Err(StoreError::NotFound(_)) => {
            return Ok(Err(fail(
                "room-unresolved",
                format!("student_env_ref {} does not resolve", draft.student_env_ref),
            )))
        }
        Err(e) => return Err(e),
    }
    // The pairing bridge (X.5) is proven by the store's wall at persist —
    // the trait exposes no pairing read for this end to re-prove.
    // B.2: the nil floor on items, mirroring the evidence rule. (Item
    // resolution is the Deacon's threshold, section I.)
    for (i, item) in draft.items.iter().enumerate() {
        if item.item_ref.is_nil() || item.provenance_ref.is_nil() {
            return Ok(Err(fail(
                "nil-item-ref",
                format!("item {i} carries a nil ref; a Return hands back things that exist (B.2)"),
            )));
        }
    }
    if let Err(detail) = completion_contract(&instruction.acceptance_criteria, &draft.completion) {
        return Ok(Err(fail("completion-contract", detail)));
    }
    Ok(Ok(()))
}

/// B.2 — the completion contract against the stored acceptance_criteria:
/// indices 0..n each answered exactly once; evidence mandatory in every
/// case; `passed` is None iff the criterion is SOVEREIGN_JUDGMENT (§1.3d).
/// A malformed stored criterion fails loudly — never a silent skip.
fn completion_contract(
    criteria: &serde_json::Value,
    completion: &[CompletionEntry],
) -> Result<(), String> {
    let criteria = criteria
        .as_array()
        .ok_or("stored acceptance_criteria is not an array")?;
    let n = criteria.len();
    if completion.len() != n {
        return Err(format!(
            "completion carries {} entries for {n} criteria — the contract is exactly one each (B.2)",
            completion.len()
        ));
    }
    let mut seen = vec![false; n];
    for entry in completion {
        let Ok(idx) = usize::try_from(entry.criterion_index) else {
            return Err(format!(
                "criterion_index {} names no criterion (B.2)",
                entry.criterion_index
            ));
        };
        if idx >= n {
            return Err(format!(
                "criterion_index {idx} is beyond the Instruction's {n} criteria (B.2)"
            ));
        }
        if seen[idx] {
            return Err(format!(
                "criterion {idx} is answered twice; the contract is exactly one entry each (B.2)"
            ));
        }
        seen[idx] = true;
        if entry.evidence_ref.is_nil() {
            return Err(format!(
                "criterion {idx} carries a nil evidence_ref; evidence is mandatory in every case (B.2)"
            ));
        }
        let testable_as = criteria[idx]
            .get("testable_as")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                format!("stored criterion {idx} carries no testable_as; a malformed criterion never passes silently")
            })?;
        let sovereign = testable_as == "SOVEREIGN_JUDGMENT";
        match (sovereign, entry.passed) {
            (true, Some(_)) => {
                return Err(format!(
                    "criterion {idx} is SOVEREIGN_JUDGMENT; its verdict is the sovereign's to render (§1.3d)"
                ))
            }
            (false, None) => {
                return Err(format!(
                    "criterion {idx} is machine-checkable; the Student renders a verdict (B.2)"
                ))
            }
            _ => {}
        }
    }
    Ok(())
}

/// The job is born BOUND to the room when the room is a live, mountable
/// workplace (env_ref + matrix in inputs — Law IX.4, authenticated by
/// create_job); when the room does not resolve or is archived, the job is
/// born unbound so the VALIDATE_OUT gate can name the refusal on record.
fn student_draft(env: Option<&EnvironmentRecord>, instruction: Uuid) -> JobDraft {
    let mut input_refs = vec![instruction];
    if let Some(e) = env {
        input_refs.push(e.matrix_ref);
    }
    JobDraft {
        agent_type: AgentType::Student,
        auditor_name: None,
        tier: env.map(|e| e.tier),
        input_refs,
        env_ref: env.map(|e| e.env_id),
        brief_ref: None,
        endpoint_alias: None,
        manual_version: Version::new(1, 0, 0),
        budgets: Budgets {
            max_wall_ms: 300_000,
            max_tool_calls: 100,
            max_tokens: 100_000,
        },
    }
}

/// How one Return labor halted after the job reached RUNNING. Either way
/// the job ends REFUSED, never stranded live (the established labor rule);
/// the persisted refusal detail names a clause or a stage — never the
/// draft's text (Law XV).
enum LaborHalt {
    /// VALIDATE_OUT said no — the Return is not written.
    Invalid(ReturnFailure),
    /// The store said no mid-labor (a wall at persist, a lost connection).
    Store {
        stage: &'static str,
        source: StoreError,
    },
}

/// The labor proper, from RUNNING onward: validate, persist, flag,
/// terminate. Every `?` in here is a mid-labor halt the caller converts
/// into a Law VII refusal.
async fn labor<S: Store>(
    store: &S,
    job_id: Uuid,
    draft: &ReturnDraft,
) -> Result<ReturnManifest, LaborHalt> {
    let at = |stage: &'static str| move |source: StoreError| LaborHalt::Store { stage, source };

    // VALIDATE_OUT: the contract is the gate. A failure is a Law VII
    // refusal — nothing is written.
    match validate_return(store, draft)
        .await
        .map_err(at("validate"))?
    {
        Ok(()) => {}
        Err(failure) => return Err(LaborHalt::Invalid(failure)),
    }
    let manifest = store
        .persist_return(job_id, draft)
        .await
        .map_err(at("persist"))?;
    // The Return becomes a flagged (certified, immutable) artifact while
    // the job still labors — an unflagged Return feeds no one (§3.1).
    let flagged = store
        .flag_return(job_id, manifest.return_id)
        .await
        .map_err(at("flag"))?;
    let out_artifact = store
        .write_artifact(
            job_id,
            "return",
            &godhead_store::ArtifactDraft {
                schema_name: RETURN_POINTER_SCHEMA.to_string(),
                schema_version: Version::new(1, 0, 0),
                payload: serde_json::json!({ "ref": manifest.return_id.to_string() }),
            },
        )
        .await
        .map_err(at("artifact"))?;
    let job = store.get_job(job_id).await.map_err(at("artifact"))?;
    store
        .transition_job(job_id, job.revision, JobStatus::Written)
        .await
        .map_err(at("written"))?;
    store
        .write_flag(
            job_id,
            &FlagDraft {
                stage: "student:return".to_string(),
                certifies: godhead_schemas::Certifies {
                    output_slots: vec!["return".to_string()],
                    revisions: vec![out_artifact.revision],
                },
                validator: godhead_schemas::Validator {
                    id: "godhead-student/validate".to_string(),
                    version: "1.0.0".to_string(),
                },
            },
        )
        .await
        .map_err(at("job-flag"))?;
    let job = store.get_job(job_id).await.map_err(at("job-flag"))?;
    store
        .transition_job(job_id, job.revision, JobStatus::Terminated)
        .await
        .map_err(at("terminate"))?;
    Ok(flagged)
}

/// The Student's whole labor over one Return (VALIDATE_OUT): spawn,
/// validate, persist, flag, terminate — or a Law VII refusal. Any halt
/// after RUNNING ends in store.refuse — no job strands live — and a
/// failed refusal write is a hard error, never swallowed (the slice-6
/// doctrine). Returns the flagged ReturnManifest.
///
/// A halt AFTER flag_return leaves the Return certified: it passed every
/// wall, and certification is not retracted by the pointer-artifact
/// stages failing — the refusal quarantines the job's artifacts, and a
/// correction is a fresh Return (§3.1).
pub async fn write_return<S: Store>(
    store: &S,
    draft: &ReturnDraft,
) -> Result<ReturnManifest, StudentError> {
    // The labor rises from the Student's room. A room that does not
    // resolve, or is no longer a workplace, still gets a laborer — one
    // whose whole labor is the recorded refusal.
    let env = match store.get_environment(draft.student_env_ref).await {
        Ok(env) if env.status == EnvStatus::Live => Some(env),
        Ok(_) => None,
        Err(StoreError::NotFound(_)) => None,
        Err(e) => return Err(e.into()),
    };
    let job = store
        .create_job(&student_draft(env.as_ref(), draft.instruction_ref))
        .await?;
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Leased)
        .await?;
    let job = store
        .transition_job(job.job_id, job.revision, JobStatus::Running)
        .await?;

    let halt = match labor(store, job.job_id, draft).await {
        Ok(flagged) => return Ok(flagged),
        Err(halt) => halt,
    };
    // The persisted detail references the clause or stage and the law
    // only — never the draft's own text, which is caller-shaped and would
    // poison the Law XV scan (and with it, the refusal record itself).
    // The (law, reason) pair comes from the ONE shared clause→code map
    // (godhead_schemas::halt_code — ruling G1): skew-shaped clauses carry
    // SCHEMA_MISMATCH (II.4, SC-A05, SC-K03); the rest VALIDATION_FAILED.
    let (law, reason, detail, err) = match halt {
        LaborHalt::Invalid(failure) => {
            let (law, reason) = godhead_schemas::halt_code(failure.clause);
            (
                law,
                reason,
                format!(
                    "Return VALIDATE_OUT failed clause '{}' (B.2, §3.1); the emission is not echoed (Law XV)",
                    failure.clause
                ),
                StudentError::ReturnInvalid(failure.to_string()),
            )
        }
        LaborHalt::Store { stage, source } => {
            let (law, reason) = godhead_schemas::stage_code();
            (
                law,
                reason,
                format!(
                    "the Return labor halted at stage '{stage}' after RUNNING; the job ends refused, never stranded (Law VII)"
                ),
                StudentError::Store(source),
            )
        }
    };
    store
        .refuse(
            job.job_id,
            &RefusalDraft {
                law,
                reason,
                subject_refs: vec![draft.instruction_ref.to_string()],
                detail,
                preserved_refs: vec![],
            },
        )
        .await?;
    Err(err)
}
