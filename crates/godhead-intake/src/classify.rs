//! The deterministic classification floor (doc 2 §2.5): hardcoded,
//! deliberately low-trust bucket labels from filetype alone. Placeholders
//! that make the store immediately legible — explicitly overridable so the
//! later AI layer knows not to over-weight them.

/// Filetype → bucket. v1 uses filetype only; trivial content signals are a
/// documented later refinement (SLICE_02 §4).
pub fn bucket(filetype: &str) -> &'static str {
    match filetype {
        "json" | "csv" | "tsv" | "xml" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "sql" => {
            "database"
        }
        "py" | "js" | "mjs" | "ts" | "rs" | "c" | "h" | "cpp" | "hpp" | "java" | "sh" | "ps1" => {
            "programming"
        }
        "html" | "htm" | "md" | "css" => "markup",
        "txt" | "log" => "document",
        _ => "unclassified",
    }
}

/// The classification entry written to the node: one bucket, a nominal
/// starting weight (inert below the coherence threshold regardless —
/// doc 4 §5.4), and the low-trust marker.
pub fn classification(filetype: &str) -> serde_json::Value {
    serde_json::json!([{
        "category": bucket(filetype),
        "weight": 0.1,
        "low_trust": true,
        "source": "filetype_floor",
    }])
}
