use crate::envelope::Envelope;
use crate::macros::closed_enum;
use semver::Version;
use time::OffsetDateTime;
use uuid::Uuid;

closed_enum! {
    /// B.3 — the closed action vocabulary an Instruction step may demand.
    /// Fetch actions exist but are barred inside Instructions in v1 (lint
    /// clause f); the rest are stewardship labor.
    CapabilityAction {
        FetchPerWrit => "FETCH_PER_WRIT",
        FetchPerCanon => "FETCH_PER_CANON",
        Refine => "REFINE",
        Organize => "ORGANIZE",
        Consolidate => "CONSOLIDATE",
        LinkPropose => "LINK_PROPOSE",
        Verify => "VERIFY",
        CompileCorpus => "COMPILE_CORPUS",
    }
}

impl CapabilityAction {
    /// Outward acts — deployed by the sovereign with a mandate, never
    /// inside an Instruction in v1 (Handbook §1.4, lint clause f).
    pub fn is_fetch(self) -> bool {
        matches!(
            self,
            CapabilityAction::FetchPerWrit | CapabilityAction::FetchPerCanon
        )
    }
}

/// B.1 — how an acceptance criterion is checked. `SovereignJudgment` is the
/// declared path for open-ended work whose success only the human can
/// judge; it is excluded from the executor's self-check but its evidence
/// is still mandatory (§1.3d).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestableAs {
    Validation(String),
    SovereignJudgment,
}

impl TestableAs {
    pub fn is_machine_checkable(&self) -> bool {
        matches!(self, TestableAs::Validation(_))
    }

    #[must_use]
    pub fn parse_stored(s: &str) -> Self {
        if s == "SOVEREIGN_JUDGMENT" {
            TestableAs::SovereignJudgment
        } else {
            TestableAs::Validation(s.to_string())
        }
    }

    pub fn as_stored(&self) -> String {
        match self {
            TestableAs::SovereignJudgment => "SOVEREIGN_JUDGMENT".to_string(),
            TestableAs::Validation(id) => id.clone(),
        }
    }
}

/// B.1 — one step of an Instruction. `params` may carry a `refs` array
/// (uuids that must resolve — lint a) and a `consumes` array (prior
/// step_ids — lint c).
#[derive(Debug, Clone)]
pub struct Step {
    pub step_id: i32,
    pub action: CapabilityAction,
    pub params: serde_json::Value,
    /// schema_name@version the step is declared to produce (lint c).
    pub expected_output: String,
    /// Declared budget hint in tokens (lint e).
    pub budget_hint_tokens: i64,
}

/// B.1 — one acceptance criterion.
#[derive(Debug, Clone)]
pub struct AcceptanceCriterion {
    pub criterion: String,
    pub testable_as: TestableAs,
}

/// B.1 — one per-matrix draw disclosure (Regular Teacher outputs, §6.3).
#[derive(Debug, Clone)]
pub struct SourceDraw {
    pub matrix_ref: Uuid,
    pub draw_count: i64,
    pub canon_associated: bool,
}

/// What a Teacher hands down before it is linted and flagged. Ids, skew,
/// and timestamps are the store's.
#[derive(Debug, Clone)]
pub struct InstructionDraft {
    pub teacher_env_ref: Option<Uuid>,
    pub teacher_tier: crate::job::Tier,
    pub target_tier: crate::job::Tier,
    pub concordat_version: Version,
    pub objective: String,
    pub steps: Vec<Step>,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    /// Required iff teacher_tier is REGULAR (§6.3).
    pub sources_drawn: Vec<SourceDraw>,
    pub supersedes_ref: Option<Uuid>,
}

/// B.1 — the persisted InstructionRecord.
#[derive(Debug, Clone)]
pub struct InstructionRecord {
    pub instruction_id: Uuid,
    pub teacher_env_ref: Option<Uuid>,
    pub teacher_tier: crate::job::Tier,
    pub target_tier: crate::job::Tier,
    pub concordat_version: Version,
    pub objective: String,
    pub steps: serde_json::Value,
    pub acceptance_criteria: serde_json::Value,
    pub sources_drawn: serde_json::Value,
    /// Derived: canon-associated draws exceed bias_skew_threshold (§6.3).
    pub skew: bool,
    pub supersedes_ref: Option<Uuid>,
    pub flagged: bool,
    pub revision: i32,
    pub envelope: Envelope,
}

/// B.4 — the Concordat: the versioned Teacher↔Student contract. Every
/// version ever cited is retained forever (§3.3).
#[derive(Debug, Clone)]
pub struct ConcordatArtifact {
    pub version: Version,
    /// Per-tier capability tables (B.3): tier → [action strings].
    pub capability_tables: serde_json::Value,
    pub pairing_semantics: serde_json::Value,
    pub adopted_at: OffsetDateTime,
    pub adopted_by: String,
    pub envelope: Envelope,
}
