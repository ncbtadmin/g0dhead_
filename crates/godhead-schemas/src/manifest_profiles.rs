//! C.3 / C.5 — the collection manifests (Handbook §1.3, §5.2). Both are
//! ReturnManifest (B.2) profiles whose items are all `CORPUS_ITEM`, extended
//! with a map that binds admitted items back to the mandate's demands:
//!
//!   - **CollectionManifest (C.5, the writ trip):** `sought` — one entry per
//!     writ target index, listing the items answering it (SC-J06).
//!   - **CorpusManifest (C.3, the canon loop):** `coverage` — one entry per
//!     canon clause, listing the items answering it; an unmet clause is the
//!     gap duty (SC-J07): the Student refuses, flagging exactly the unmet
//!     clauses, and pads nothing from outside the canon.
//!
//! The validation is merciless by design (§5.2): no item may be padding (every
//! item maps), and no gap may hide (every unmet target/clause is flagged).

use uuid::Uuid;

/// C.5 — one writ target's answer: the items collected for target `target_index`.
/// Empty `item_refs` = the target went unmet (flagged).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoughtEntry {
    pub target_index: i32,
    pub item_refs: Vec<Uuid>,
}

/// C.3 — one canon clause's answer. Empty `item_refs` = the clause is unmet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageEntry {
    pub canon_clause: String,
    pub item_refs: Vec<Uuid>,
}

/// SC-J07's gap duty: a canon that could not be exhausted. The Student refuses
/// (Law VII), naming exactly the unmet clauses; the sovereign answers by
/// naming more sources (or narrowing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GapDuty {
    pub unmet_clauses: Vec<String>,
}

/// SC-J06 — validate a CollectionManifest's `sought` map against the items
/// actually collected and the writ's target count. Returns the target indices
/// that went unmet (each flagged with empty `item_refs`), or an error naming
/// the first structural fault. Mercilessly: every collected item must map to
/// exactly one target (no padding), every listed item must be a collected one
/// (no phantom), and every target index in `0..target_count` must have exactly
/// one entry (no silent gap, no duplicate).
pub fn validate_sought(
    collected: &[Uuid],
    target_count: usize,
    sought: &[SoughtEntry],
) -> Result<Vec<i32>, String> {
    if sought.len() != target_count {
        return Err(format!(
            "the sought map has {} entries for {target_count} writ targets — one per target, no more (C.5)",
            sought.len()
        ));
    }
    let mut seen_targets = std::collections::HashSet::new();
    let mut mapped_items: std::collections::HashMap<Uuid, i32> = std::collections::HashMap::new();
    let collected_set: std::collections::HashSet<Uuid> = collected.iter().copied().collect();
    let mut unmet = Vec::new();
    for entry in sought {
        if entry.target_index < 0 || entry.target_index as usize >= target_count {
            return Err(format!(
                "sought entry names target_index {} outside 0..{target_count} (C.5)",
                entry.target_index
            ));
        }
        if !seen_targets.insert(entry.target_index) {
            return Err(format!(
                "target_index {} appears twice in the sought map (C.5)",
                entry.target_index
            ));
        }
        if entry.item_refs.is_empty() {
            unmet.push(entry.target_index);
        }
        for item in &entry.item_refs {
            if !collected_set.contains(item) {
                return Err(format!(
                    "sought target {} lists item {item} that was not collected — no phantom (SC-J06)",
                    entry.target_index
                ));
            }
            if let Some(other) = mapped_items.insert(*item, entry.target_index) {
                return Err(format!(
                    "item {item} answers both target {other} and {} — an item maps once (SC-J06)",
                    entry.target_index
                ));
            }
        }
    }
    // No padding: every collected item maps to some target.
    for item in collected {
        if !mapped_items.contains_key(item) {
            return Err(format!(
                "collected item {item} maps to no writ target — a manifest pads nothing (SC-J06)"
            ));
        }
    }
    unmet.sort_unstable();
    Ok(unmet)
}

/// SC-J07 — validate a CorpusManifest's `coverage` map against the collected
/// items and the canon's clauses. Same mercilessness as `validate_sought`, and
/// then the **gap duty**: if any clause is unmet (empty `item_refs`), return
/// `Err(GapDuty)` naming exactly the unmet clauses — the Student refuses rather
/// than pads. `Ok(())` only when every clause is covered.
pub fn validate_coverage(
    collected: &[Uuid],
    clauses: &[String],
    coverage: &[CoverageEntry],
) -> Result<Result<(), GapDuty>, String> {
    if coverage.len() != clauses.len() {
        return Err(format!(
            "the coverage map has {} entries for {} canon clauses — one per clause (C.3)",
            coverage.len(),
            clauses.len()
        ));
    }
    let collected_set: std::collections::HashSet<Uuid> = collected.iter().copied().collect();
    let mut mapped_items: std::collections::HashMap<Uuid, &str> = std::collections::HashMap::new();
    let mut unmet_clauses = Vec::new();
    for (entry, clause) in coverage.iter().zip(clauses) {
        if &entry.canon_clause != clause {
            return Err(format!(
                "coverage entry {:?} does not match canon clause {clause:?} (C.3 — one entry per clause, in order)",
                entry.canon_clause
            ));
        }
        if entry.item_refs.is_empty() {
            unmet_clauses.push(clause.clone());
        }
        for item in &entry.item_refs {
            if !collected_set.contains(item) {
                return Err(format!(
                    "clause {clause:?} lists item {item} that was not collected — nothing sourced outside the canon (SC-J07)"
                ));
            }
            if let Some(other) = mapped_items.insert(*item, clause) {
                return Err(format!(
                    "item {item} answers both clause {other:?} and {clause:?} — an item maps once (SC-J07)"
                ));
            }
        }
    }
    for item in collected {
        if !mapped_items.contains_key(item) {
            return Err(format!(
                "collected item {item} answers no canon clause — the corpus pads nothing (SC-J07)"
            ));
        }
    }
    if unmet_clauses.is_empty() {
        Ok(Ok(()))
    } else {
        // The gap duty: refuse, naming the unmet clauses (Law VII; §1.3).
        Ok(Err(GapDuty { unmet_clauses }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    #[test]
    fn sought_maps_and_flags() {
        let a = id(1);
        let b = id(2);
        // Two targets: target 0 met by [a], target 1 unmet.
        let unmet = validate_sought(
            &[a, b],
            2,
            &[
                SoughtEntry {
                    target_index: 0,
                    item_refs: vec![a, b],
                },
                SoughtEntry {
                    target_index: 1,
                    item_refs: vec![],
                },
            ],
        )
        .expect("valid map");
        assert_eq!(unmet, vec![1], "the unmet target is flagged");
    }

    #[test]
    fn sought_rejects_padding_and_phantom() {
        let a = id(1);
        let orphan = id(9);
        // An item collected but mapped to no target = padding.
        let err = validate_sought(
            &[a, orphan],
            1,
            &[SoughtEntry {
                target_index: 0,
                item_refs: vec![a],
            }],
        )
        .unwrap_err();
        assert!(err.contains("pads nothing"), "{err}");
        // A target listing an item that was never collected = phantom.
        let err = validate_sought(
            &[a],
            1,
            &[SoughtEntry {
                target_index: 0,
                item_refs: vec![a, id(9)],
            }],
        )
        .unwrap_err();
        assert!(err.contains("no phantom"), "{err}");
    }

    #[test]
    fn coverage_gap_duty_fires() {
        let a = id(1);
        let clauses = vec!["clause one".to_string(), "clause two".to_string()];
        // Clause two unmet → gap duty naming it.
        let gap = validate_coverage(
            &[a],
            &clauses,
            &[
                CoverageEntry {
                    canon_clause: "clause one".into(),
                    item_refs: vec![a],
                },
                CoverageEntry {
                    canon_clause: "clause two".into(),
                    item_refs: vec![],
                },
            ],
        )
        .expect("structurally valid")
        .expect_err("clause two is unmet");
        assert_eq!(gap.unmet_clauses, vec!["clause two".to_string()]);
    }

    #[test]
    fn coverage_all_met_passes() {
        let a = id(1);
        let clauses = vec!["only clause".to_string()];
        validate_coverage(
            &[a],
            &clauses,
            &[CoverageEntry {
                canon_clause: "only clause".into(),
                item_refs: vec![a],
            }],
        )
        .expect("structurally valid")
        .expect("every clause covered");
    }
}
