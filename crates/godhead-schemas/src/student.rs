use crate::envelope::Envelope;
use crate::macros::closed_enum;
use semver::Version;
use uuid::Uuid;

closed_enum! {
    /// B.2 — the kinds of thing a Student hands back.
    ReturnItemKind {
        RefinedDoc => "REFINED_DOC",
        CorpusItem => "CORPUS_ITEM",
        OrganizationChange => "ORGANIZATION_CHANGE",
    }
}

/// B.2 — one returned item.
#[derive(Debug, Clone)]
pub struct ReturnItem {
    pub item_ref: Uuid,
    pub kind: ReturnItemKind,
    pub provenance_ref: Uuid,
}

/// B.2 — one completion entry: criterion-by-criterion. `passed` is None
/// iff the answered criterion is SOVEREIGN_JUDGMENT (verdict rendered at
/// sovereign review); `evidence_ref` is mandatory in every case.
#[derive(Debug, Clone)]
pub struct CompletionEntry {
    pub criterion_index: i32,
    pub passed: Option<bool>,
    pub evidence_ref: Uuid,
}

/// What a Student hands back before it is validated and flagged.
#[derive(Debug, Clone)]
pub struct ReturnDraft {
    pub instruction_ref: Uuid,
    pub student_env_ref: Uuid,
    pub concordat_version: Version,
    pub items: Vec<ReturnItem>,
    pub completion: Vec<CompletionEntry>,
}

/// B.2 — the persisted ReturnManifest.
#[derive(Debug, Clone)]
pub struct ReturnManifest {
    pub return_id: Uuid,
    pub instruction_ref: Uuid,
    pub student_env_ref: Uuid,
    pub concordat_version: Version,
    pub items: serde_json::Value,
    pub completion: serde_json::Value,
    pub flagged: bool,
    /// Byte-integrity certification (ruling G7): SHA-256 of the canonical
    /// body, persisted at FLAG, re-proven at every read of the flagged
    /// record. None until flagged.
    pub content_sha: Option<String>,
    pub revision: i32,
    pub envelope: Envelope,
}

/// A refined artifact and its derivation (Handbook §1.2b): the recorded
/// source refs and method are sufficient to reproduce `content_sha`.
#[derive(Debug, Clone)]
pub struct RefinedArtifact {
    pub artifact_id: Uuid,
    pub env_ref: Uuid,
    pub source_refs: Vec<Uuid>,
    pub method: String,
    pub content_sha: String,
    pub revision: i32,
    pub envelope: Envelope,
}
