//! The Executability Lint (Holy Standard §1.3): the six mechanical clauses
//! the Teacher MUST prove before writing. Self-verification is not review
//! etiquette; it is the gate between a thought and a record. Fail any
//! clause → the Instruction is not written.

use crate::{ConcordatError, SUPPORTED_CONCORDAT};
use godhead_schemas::{
    AgentType, Budgets, CapabilityAction, FlagDraft, InstructionDraft, InstructionRecord, JobDraft,
    JobStatus, Law, RefusalDraft, RefusalReason, TestableAs, Tier,
};
use godhead_store::{Store, StoreError};
use semver::{Version, VersionReq};
use uuid::Uuid;

/// Which clause failed and why — the Teacher-particular VALIDATE_OUT
/// content. Each clause maps to a Phase B test (SC-K01).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintFailure {
    pub clause: char,
    pub detail: String,
}

impl std::fmt::Display for LintFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "clause ({}): {}", self.clause, self.detail)
    }
}

/// Runs the six-clause lint against the store and the cited Concordat.
/// Ok(()) means the Instruction is executable; Err names the failing
/// clause.
pub async fn lint_instruction<S: Store>(
    store: &S,
    draft: &InstructionDraft,
) -> Result<Result<(), LintFailure>, StoreError> {
    // The cited Concordat must exist (retained forever); its capability
    // table for the target tier drives clause (b).
    let concordat = match store.get_concordat(&draft.concordat_version).await {
        Ok(c) => c,
        Err(_) => {
            return Ok(Err(LintFailure {
                clause: 'b',
                detail: format!("cited Concordat {} is not adopted", draft.concordat_version),
            }))
        }
    };
    let table = concordat
        .capability_tables
        .get(draft.target_tier.as_str())
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let allowed: Vec<String> = table
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();

    // B.1 shape floor (proven at both ends — the double-validation covenant
    // catches a corrupted objective or an emptied step/criteria set).
    if draft.objective.trim().is_empty() {
        return Ok(Err(LintFailure {
            clause: 'a',
            detail: "the Instruction has no objective (B.1)".into(),
        }));
    }
    if draft.steps.is_empty() {
        return Ok(Err(LintFailure {
            clause: 'c',
            detail: "an Instruction with no steps does nothing (B.1)".into(),
        }));
    }

    // (a) Resolution: teacher_env_ref and every step-param ref resolve.
    if let Some(env) = draft.teacher_env_ref {
        if store.get_environment(env).await.is_err() {
            return Ok(Err(LintFailure {
                clause: 'a',
                detail: format!("teacher_env_ref {env} does not resolve"),
            }));
        }
    }
    for step in &draft.steps {
        // A present `refs` key that is not an array is malformed — never a
        // silent skip (the bypass a bare-string encoding would otherwise be).
        if let Some(refs_val) = step.params.get("refs") {
            let Some(refs) = refs_val.as_array() else {
                return Ok(Err(LintFailure {
                    clause: 'a',
                    detail: format!("step {} 'refs' must be an array", step.step_id),
                }));
            };
            for r in refs {
                let Some(id) = r.as_str().and_then(|s| Uuid::parse_str(s).ok()) else {
                    return Ok(Err(LintFailure {
                        clause: 'a',
                        detail: format!("step {} carries a non-uuid ref", step.step_id),
                    }));
                };
                let resolves: bool = sqlx_resolves(store, id).await?;
                if !resolves {
                    return Ok(Err(LintFailure {
                        clause: 'a',
                        detail: format!(
                            "step {} references {id}, which does not resolve",
                            step.step_id
                        ),
                    }));
                }
            }
        }
    }

    // (b) Capability: each step's action ∈ the tier's capability table.
    for step in &draft.steps {
        if !allowed.iter().any(|a| a == step.action.as_str()) {
            return Ok(Err(LintFailure {
                clause: 'b',
                detail: format!(
                    "step {} demands '{}', outside the {} capability table",
                    step.step_id,
                    step.action.as_str(),
                    draft.target_tier
                ),
            }));
        }
    }

    // (c) Closure: each step declares an expected_output, and each
    // `consumes` id names a prior step. No step reads what nothing made.
    let step_ids: Vec<i32> = draft.steps.iter().map(|s| s.step_id).collect();
    for step in &draft.steps {
        if step.expected_output.trim().is_empty() {
            return Ok(Err(LintFailure {
                clause: 'c',
                detail: format!("step {} declares no expected_output", step.step_id),
            }));
        }
        if let Some(consumes_val) = step.params.get("consumes") {
            let Some(consumes) = consumes_val.as_array() else {
                return Ok(Err(LintFailure {
                    clause: 'c',
                    detail: format!("step {} 'consumes' must be an array", step.step_id),
                }));
            };
            for c in consumes {
                let Some(cid) = c.as_i64() else {
                    return Ok(Err(LintFailure {
                        clause: 'c',
                        detail: format!("step {} consumes a non-integer step id", step.step_id),
                    }));
                };
                let cid = i32::try_from(cid).unwrap_or(i32::MAX);
                let is_prior = step_ids.contains(&cid) && cid < step.step_id;
                if !is_prior {
                    return Ok(Err(LintFailure {
                        clause: 'c',
                        detail: format!(
                            "step {} consumes step {cid}, which is not a prior step",
                            step.step_id
                        ),
                    }));
                }
            }
        }
    }

    // (d) Checkability: criteria present; each testable_as is a validation
    // or SOVEREIGN_JUDGMENT; at least one is machine-checkable.
    if draft.acceptance_criteria.is_empty() {
        return Ok(Err(LintFailure {
            clause: 'd',
            detail: "no acceptance criteria; an unmeasurable instruction is decoration".into(),
        }));
    }
    // Each criterion must say something; a machine-checkable one must name
    // a non-empty validation (an empty id is not a validation the executor
    // can run — clause d's substance, not just its enum variant).
    for c in &draft.acceptance_criteria {
        if c.criterion.trim().is_empty() {
            return Ok(Err(LintFailure {
                clause: 'd',
                detail: "an acceptance criterion says nothing".into(),
            }));
        }
    }
    let machine_checkable = draft
        .acceptance_criteria
        .iter()
        .any(|c| matches!(&c.testable_as, TestableAs::Validation(id) if !id.trim().is_empty()));
    if !machine_checkable {
        return Ok(Err(LintFailure {
            clause: 'd',
            detail: "no machine-checkable criterion (every one is SOVEREIGN_JUDGMENT or an empty validation id)".into(),
        }));
    }

    // (e) Budget: Σ step budget_hint ≤ the target tier ceiling (A.14).
    let ceiling_cfg = store.get_config("instruction_budget_ceiling").await?;
    let ceiling = ceiling_cfg
        .value
        .get(draft.target_tier.as_str())
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| {
            StoreError::ValidationFailed(format!(
                "instruction_budget_ceiling has no entry for tier {}",
                draft.target_tier
            ))
        })?;
    let total: i64 = draft
        .steps
        .iter()
        .map(|s| s.budget_hint_tokens.max(0))
        .sum();
    if total > ceiling {
        return Ok(Err(LintFailure {
            clause: 'e',
            detail: format!(
                "declared budget {total} exceeds the {} ceiling {ceiling}",
                draft.target_tier
            ),
        }));
    }

    // (f) Sovereignty: no fetch action (v1 bar) and no human-reserved act.
    for step in &draft.steps {
        if step.action.is_fetch() {
            return Ok(Err(LintFailure {
                clause: 'f',
                detail: format!(
                    "step {} is a fetch ('{}'); v1 Instructions carry no outward act (Handbook §1.4)",
                    step.step_id,
                    step.action.as_str()
                ),
            }));
        }
    }

    Ok(Ok(()))
}

async fn sqlx_resolves<S: Store>(store: &S, id: Uuid) -> Result<bool, StoreError> {
    // A ref resolves if it is a node, link, matrix, or environment — all
    // first-class store objects. Distinguish "not this kind" (NotFound)
    // from a real store fault, which must surface, not read as unresolved.
    for found in [
        exists(store.get_node(id).await)?,
        exists(store.get_link(id).await)?,
        exists(store.get_matrix(id).await)?,
        exists(store.get_environment(id).await)?,
    ] {
        if found {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Ok(true) if the record was found, Ok(false) if it simply is not this
/// kind (NotFound), Err on any other store fault.
fn exists<T>(result: Result<T, StoreError>) -> Result<bool, StoreError> {
    match result {
        Ok(_) => Ok(true),
        Err(StoreError::NotFound(_)) => Ok(false),
        Err(e) => Err(e),
    }
}

fn teacher_draft(tier: Tier, env: Option<Uuid>) -> JobDraft {
    JobDraft {
        agent_type: AgentType::Teacher,
        auditor_name: None,
        tier: Some(tier),
        input_refs: env.map(|e| vec![e]).unwrap_or_default(),
        env_ref: None,
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

/// How one Instruction labor halted after the job reached RUNNING. Either
/// way the job ends REFUSED, never stranded live (the established labor
/// rule); the persisted refusal detail names a clause or a stage — never
/// the draft's text (Law XV).
enum LaborHalt {
    /// VALIDATE_OUT said no — the Instruction is not written.
    Invalid(LintFailure),
    /// The store said no mid-labor (a wall at persist, a lost connection).
    Store {
        stage: &'static str,
        source: StoreError,
    },
}

/// The labor proper, from RUNNING onward: lint, persist, flag, terminate.
/// Every `?` in here is a mid-labor halt the caller converts into a Law
/// VII refusal.
async fn labor<S: Store>(
    store: &S,
    job_id: Uuid,
    draft: &InstructionDraft,
) -> Result<InstructionRecord, LaborHalt> {
    let at = |stage: &'static str| move |source: StoreError| LaborHalt::Store { stage, source };

    // VALIDATE_OUT: the lint is the gate. A failure is a Law VII refusal —
    // nothing is written.
    match lint_instruction(store, draft).await.map_err(at("lint"))? {
        Ok(()) => {}
        Err(failure) => return Err(LaborHalt::Invalid(failure)),
    }
    let instruction = store
        .persist_instruction(job_id, draft)
        .await
        .map_err(at("persist"))?;
    // The Instruction becomes a flagged (certified, immutable) artifact
    // while the job still labors — an unflagged Instruction is invisible to
    // the Student by law (§5.1). This happens before the job's own FLAG.
    let flagged = store
        .flag_instruction(job_id, instruction.instruction_id)
        .await
        .map_err(at("flag"))?;
    let out_artifact = store
        .write_artifact(
            job_id,
            "instruction",
            &godhead_store::ArtifactDraft {
                schema_name: crate::INSTRUCTION_POINTER_SCHEMA.to_string(),
                schema_version: Version::new(1, 0, 0),
                payload: serde_json::json!({ "ref": instruction.instruction_id.to_string() }),
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
                stage: "teacher:instruction".to_string(),
                certifies: godhead_schemas::Certifies {
                    output_slots: vec!["instruction".to_string()],
                    revisions: vec![out_artifact.revision],
                },
                validator: godhead_schemas::Validator {
                    id: "godhead-concordat/lint".to_string(),
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

/// The Teacher's whole labor over one Instruction (VALIDATE_OUT): spawn,
/// lint, persist, flag, terminate — or a Law VII refusal (the Instruction
/// is not written). Any halt after RUNNING ends in store.refuse — no job
/// strands live — and a failed refusal write is a hard error, never
/// swallowed (the slice-6 doctrine, applied here by the slice-9 review).
/// The persisted `skew` mark is derived from the draft's disclosed sources
/// by the store (B.1). Returns the flagged Instruction.
///
/// A halt AFTER flag_instruction leaves the Instruction certified: it
/// passed the lint and every store wall, and correction is supersession,
/// never retraction (§1.4).
pub async fn write_instruction<S: Store>(
    store: &S,
    draft: &InstructionDraft,
) -> Result<InstructionRecord, ConcordatError> {
    let job = store
        .create_job(&teacher_draft(draft.teacher_tier, draft.teacher_env_ref))
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
    let (law, detail, err) = match halt {
        LaborHalt::Invalid(failure) => (
            Law::II,
            format!(
                "Executability Lint failed clause '{}' (§1.3); the emission is not echoed (Law XV)",
                failure.clause
            ),
            ConcordatError::LintFailed(failure.to_string()),
        ),
        LaborHalt::Store { stage, source } => (
            Law::VII,
            format!(
                "the Instruction labor halted at stage '{stage}' after RUNNING; the job ends refused, never stranded (Law VII)"
            ),
            ConcordatError::Store(source),
        ),
    };
    store
        .refuse(
            job.job_id,
            &RefusalDraft {
                law,
                reason: RefusalReason::ValidationFailed,
                subject_refs: vec![],
                detail,
                preserved_refs: vec![],
            },
        )
        .await?;
    Err(err)
}

/// The Student's VALIDATE_IN (§2.3): before executing, re-prove the
/// Instruction. Concordat-version skew in either direction is a
/// SCHEMA_MISMATCH refusal (§2.4); an out-of-band corruption of the
/// schema-shape (steps, criteria, objective, budget) between flag and read
/// fails loudly — at reconstruction if a field no longer parses, at the
/// re-lint if the parsed body no longer proves. The same versioned
/// artifact is validated twice, by two strangers who never meet. (Access across the pairing bridge is
/// governed separately by `env_scoped_read` — Law IX.5; this is the
/// content re-proof, and callers compose the two.)
pub async fn read_instruction<S: Store>(
    store: &S,
    reader_job_id: Uuid,
    instruction_id: Uuid,
) -> Result<InstructionRecord, ConcordatError> {
    let instruction = store.get_instruction(instruction_id).await?;
    // Only a flagged Instruction is visible; an unflagged one is invisible
    // by law (§5.1).
    if !instruction.flagged {
        return Err(ConcordatError::SchemaMismatch(
            "the Instruction is not flagged; unflagged means uncertified (§5.1)".into(),
        ));
    }
    // Version skew (§2.4): the Student supports a Concordat range; an
    // Instruction outside it refuses at this end. Never best-effort.
    let supported = VersionReq::parse(SUPPORTED_CONCORDAT).expect("valid req");
    if !supported.matches(&instruction.concordat_version) {
        return Err(ConcordatError::SchemaMismatch(format!(
            "Concordat version {} is outside the Student's supported range {SUPPORTED_CONCORDAT} (§2.4)",
            instruction.concordat_version
        )));
    }
    // Double-validation: re-prove the body. A mismatch cannot pass
    // silently, because silence itself fails validation (§2.3). The lint
    // re-run against the persisted body catches out-of-band corruption.
    let draft = reconstruct_draft(&instruction)?;
    match lint_instruction(store, &draft).await? {
        Ok(()) => {}
        Err(failure) => {
            let _ = reader_job_id; // reader identity is provenance; the refusal is the Student's
            return Err(ConcordatError::SchemaMismatch(format!(
                "the Instruction no longer validates at read (VALIDATE_IN caught it): {failure}"
            )));
        }
    }
    Ok(instruction)
}

/// Rebuilds a lintable draft from a persisted Instruction, for the
/// Student's re-validation. Every field is required to parse back whole:
/// a silent default would hand a corrupted field a lawful disguise, so an
/// unparseable one is a SchemaMismatch, never a guess.
fn reconstruct_draft(rec: &InstructionRecord) -> Result<InstructionDraft, ConcordatError> {
    let steps = rec
        .steps
        .as_array()
        .ok_or_else(|| ConcordatError::SchemaMismatch("steps not an array".into()))?
        .iter()
        .map(|s| {
            let action = s
                .get("action")
                .and_then(|v| v.as_str())
                .and_then(|a| CapabilityAction::parse(a).ok())
                .ok_or_else(|| ConcordatError::SchemaMismatch("step action invalid".into()))?;
            // Every field persist wrote must read back whole; a default
            // here would let a corrupted field impersonate a lawful one
            // (a budget read as 0 trivially passes clause e). Unparseable
            // → SchemaMismatch, loudly.
            Ok(godhead_schemas::Step {
                step_id: s
                    .get("step_id")
                    .and_then(serde_json::Value::as_i64)
                    .and_then(|v| i32::try_from(v).ok())
                    .ok_or_else(|| ConcordatError::SchemaMismatch("step step_id invalid".into()))?,
                action,
                params: s
                    .get("params")
                    .cloned()
                    .ok_or_else(|| ConcordatError::SchemaMismatch("step params missing".into()))?,
                expected_output: s
                    .get("expected_output")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ConcordatError::SchemaMismatch("step expected_output invalid".into())
                    })?
                    .to_string(),
                budget_hint_tokens: s
                    .get("budget_hint_tokens")
                    .and_then(serde_json::Value::as_i64)
                    .ok_or_else(|| {
                        ConcordatError::SchemaMismatch("step budget_hint_tokens invalid".into())
                    })?,
            })
        })
        .collect::<Result<Vec<_>, ConcordatError>>()?;
    let criteria = rec
        .acceptance_criteria
        .as_array()
        .ok_or_else(|| ConcordatError::SchemaMismatch("criteria not an array".into()))?
        .iter()
        .map(|c| {
            Ok(godhead_schemas::AcceptanceCriterion {
                criterion: c
                    .get("criterion")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConcordatError::SchemaMismatch("criterion invalid".into()))?
                    .to_string(),
                testable_as: TestableAs::parse_stored(
                    c.get("testable_as")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            ConcordatError::SchemaMismatch("criterion testable_as invalid".into())
                        })?,
                ),
            })
        })
        .collect::<Result<Vec<_>, ConcordatError>>()?;
    Ok(InstructionDraft {
        teacher_env_ref: rec.teacher_env_ref,
        teacher_tier: rec.teacher_tier,
        target_tier: rec.target_tier,
        concordat_version: rec.concordat_version.clone(),
        objective: rec.objective.clone(),
        steps,
        acceptance_criteria: criteria,
        sources_drawn: vec![],
        supersedes_ref: rec.supersedes_ref,
    })
}
