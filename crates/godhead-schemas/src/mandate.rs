//! C.4 — the MandateRecord: the human-authored charter of every outward act
//! (Handbook §1.4; Dogma IV.4 round 5). Two kinds: the canon — exhaustive,
//! for a Teacher, coverage-mapped — and the writ — a bounded errand for a
//! matrix, naming concrete, resolvable targets. Locators, never queries:
//! the concreteness check is the schema-enforced line between the writ
//! system and the deferred breadth system (SC-J02), and it fires at
//! authorship, before any trip.

use crate::envelope::Envelope;
use crate::macros::closed_enum;
use time::OffsetDateTime;
use uuid::Uuid;

closed_enum! {
    /// C.4 — the two kinds of mandate. The tiers do not smear (§1.4):
    /// canonical-ness remains exhaustive collection for a Teacher.
    MandateKind {
        Canon => "CANON",
        Writ => "WRIT",
    }
}

/// C.4 — a writ target's locator: a URI or a registered source id. Typed,
/// never free text — a writ says *fetch these sources*, never *find things
/// about X*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Locator {
    Uri(String),
    SourceId(String),
}

impl Locator {
    pub fn kind(&self) -> &'static str {
        match self {
            Locator::Uri(_) => "uri",
            Locator::SourceId(_) => "source_id",
        }
    }

    pub fn value(&self) -> &str {
        match self {
            Locator::Uri(v) | Locator::SourceId(v) => v,
        }
    }
}

/// C.4 — one writ demand: a resolvable locator and an optional note.
#[derive(Debug, Clone)]
pub struct WritTarget {
    pub locator: Locator,
    pub note: Option<String>,
}

/// The demands, by kind: freeform exhaustiveness-defining clauses for a
/// canon; typed locators for a writ.
#[derive(Debug, Clone)]
pub enum MandateDemands {
    CanonClauses(Vec<String>),
    WritTargets(Vec<WritTarget>),
}

/// What the sovereign authors (authorship is human by construction — the
/// store rejects agent-written mandates, Dogma IV.4).
#[derive(Debug, Clone)]
pub struct MandateDraft {
    pub kind: MandateKind,
    /// Required iff CANON: the Teacher the corpus is collected for.
    pub teacher_env_ref: Option<Uuid>,
    /// Required iff WRIT: the matrix the errand feeds.
    pub matrix_ref: Option<Uuid>,
    pub demands: MandateDemands,
    /// v1 canon-fetch (2026-07-09 ruling; C.4 `sources`): a CANON names
    /// concrete, typed sources under the *identical* SC-J02 validation — its
    /// freeform `demands` clauses stay the coverage surface. A WRIT names its
    /// targets in `demands` and leaves this empty; a canon with empty
    /// `sources` simply has no v1 trip to run (breadth discovers sources, and
    /// breadth is deferred).
    pub sources: Vec<WritTarget>,
    pub trip_budget: serde_json::Value,
}

/// C.4 — the persisted MandateRecord.
#[derive(Debug, Clone)]
pub struct MandateRecord {
    pub mandate_id: Uuid,
    pub kind: MandateKind,
    pub teacher_env_ref: Option<Uuid>,
    pub matrix_ref: Option<Uuid>,
    pub demands: serde_json::Value,
    /// The typed locators a v1 canon trip fetches (`[{locator:{kind,value},
    /// note}]`); `[]` for a writ, whose targets live in `demands` (C.4 ruling
    /// 2026-07-09).
    pub sources: serde_json::Value,
    pub trip_budget: serde_json::Value,
    pub authored_at: OffsetDateTime,
    pub envelope: Envelope,
}

impl MandateRecord {
    /// The typed locators this mandate's v1 trip fetches: a WRIT's are its
    /// `demands` targets, a CANON's are its `sources` (the 2026-07-09 ruling).
    /// The set is drawn ONLY from these persisted, validated fields — never
    /// from any freeform text — which is what SC-J05 asserts.
    pub fn trip_locators(&self) -> Result<Vec<Locator>, String> {
        let field = match self.kind {
            MandateKind::Writ => &self.demands,
            MandateKind::Canon => &self.sources,
        };
        let array = field
            .as_array()
            .ok_or_else(|| "mandate targets are a locator array (C.4)".to_string())?;
        let mut out = Vec::with_capacity(array.len());
        for (i, entry) in array.iter().enumerate() {
            let loc = entry
                .get("locator")
                .ok_or_else(|| format!("target {i} carries no locator (C.4)"))?;
            let value = loc
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("target {i}'s locator names no value (C.4)"))?;
            let locator = match loc.get("kind").and_then(|v| v.as_str()) {
                Some("uri") => Locator::Uri(value.to_string()),
                Some("source_id") => Locator::SourceId(value.to_string()),
                other => return Err(format!("target {i}: unknown locator kind {other:?} (C.4)")),
            };
            out.push(locator);
        }
        Ok(out)
    }
}

/// Characters whose presence marks a locator as search-shaped rather than
/// named: whitespace turns a locator into prose; quotes, wildcards, pipes,
/// and angle/brace metacharacters are search-operator or injection shapes.
/// Over-strictness is the correct failure direction here — the day a
/// mandate may contain a search is the day the breadth system ships, and
/// not before (C.4).
const QUERY_SHAPED: &[char] = &[
    ' ', '\t', '\n', '\r', '"', '\'', '*', '|', '<', '>', '{', '}', '^', '`', '\\',
];

/// SC-J02, the shape half — validated at authorship, before any trip. A
/// URI must be scheme-qualified (`scheme://rest`), non-empty past the
/// scheme, and free of query-shaped characters; a source id must be a
/// plain registered-id token (its *resolution* against the known-source
/// registry is the store's half — an unknown source_id fails there).
/// Returns the offending description on failure.
pub fn validate_locator_shape(locator: &Locator) -> Result<(), String> {
    let value = locator.value();
    if value.is_empty() {
        return Err("an empty locator names nothing (C.4)".into());
    }
    if let Some(bad) = value.chars().find(|c| QUERY_SHAPED.contains(c)) {
        return Err(format!(
            "locator contains {bad:?} — query-shaped, not a named target; a writ says fetch THESE sources, never find-things-about (C.4/SC-J02)"
        ));
    }
    match locator {
        Locator::Uri(uri) => {
            let Some((scheme, rest)) = uri.split_once("://") else {
                return Err(format!(
                    "'{uri}' is not a scheme-qualified URI; a bare topic is a query wearing a locator's field (C.4/SC-J02)"
                ));
            };
            if scheme.is_empty()
                || !scheme
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_lowercase())
                || !scheme
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "+.-".contains(c))
            {
                return Err(format!("'{uri}' carries a malformed scheme (C.4)"));
            }
            if rest.is_empty() {
                return Err(format!(
                    "'{uri}' names a scheme and nothing behind it (C.4)"
                ));
            }
            Ok(())
        }
        Locator::SourceId(id) => {
            let mut chars = id.chars();
            let head_ok = chars.next().is_some_and(|c| c.is_ascii_alphanumeric());
            let tail_ok = id.len() <= 64
                && id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c));
            if !(head_ok && tail_ok) {
                return Err(format!("'{id}' is not a well-formed source id token (C.4)"));
            }
            Ok(())
        }
    }
}

/// SC-J02 over a whole draft's shape: kind/recipient coherence and every
/// demand's form. The store's `author_mandate` runs this and then the
/// resolution half (recipient records resolve; source ids are registered).
pub fn validate_mandate_shape(draft: &MandateDraft) -> Result<(), String> {
    match draft.kind {
        MandateKind::Canon => {
            if draft.teacher_env_ref.is_none() || draft.matrix_ref.is_some() {
                return Err(
                    "a CANON mandate names a teacher_env_ref and no matrix_ref (C.4)".into(),
                );
            }
            let MandateDemands::CanonClauses(clauses) = &draft.demands else {
                return Err("a CANON mandate's demands are clauses (C.4)".into());
            };
            if clauses.is_empty() {
                return Err("a canon with no clauses demands nothing (C.4)".into());
            }
            if let Some(i) = clauses.iter().position(|c| c.trim().is_empty()) {
                return Err(format!("canon clause {i} is empty (C.4)"));
            }
            // v1 canon-fetch (C.4 ruling 2026-07-09): `sources` are typed
            // locators under the IDENTICAL SC-J02 wall as writ targets. Empty
            // is lawful — a canon with no sources simply has no v1 trip.
            for (i, source) in draft.sources.iter().enumerate() {
                validate_locator_shape(&source.locator)
                    .map_err(|why| format!("canon source {i}: {why}"))?;
            }
            Ok(())
        }
        MandateKind::Writ => {
            if draft.matrix_ref.is_none() || draft.teacher_env_ref.is_some() {
                return Err(
                    "a WRIT mandate names a matrix_ref and no teacher_env_ref (C.4)".into(),
                );
            }
            let MandateDemands::WritTargets(targets) = &draft.demands else {
                return Err("a WRIT mandate's demands are typed locators (C.4)".into());
            };
            if targets.is_empty() {
                return Err("a writ with no targets is an errand to nowhere (C.4)".into());
            }
            if !draft.sources.is_empty() {
                return Err("a WRIT names its targets in demands, not sources (C.4 ruling)".into());
            }
            for (i, target) in targets.iter().enumerate() {
                validate_locator_shape(&target.locator)
                    .map_err(|why| format!("writ demand {i}: {why}"))?;
            }
            Ok(())
        }
    }
}
