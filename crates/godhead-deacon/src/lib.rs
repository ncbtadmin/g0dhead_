//! godhead-deacon — the order's one standing functionary (Dogma Book II §1;
//! docs/dev/SLICE_10.md).
//!
//! The Deacon is an *office*, not an agent: he does not spawn and holds no
//! budgets (exempt from Laws I and XIV); every law governing writes binds
//! his writes exactly as it binds an agent's (II, III, V, XI, XII, XIII,
//! XV). His writes bear the office identity and authenticate as
//! `office:deacon` at the substrate (ruling G10). Beneath the vestment he
//! is plain, auditable, hardcoded procedure: scan, manifest, present,
//! admit — and never alone, never unscanned, never bypassed. Each of those
//! prohibitions is a store wall, not a convention.
//!
//! The scan endpoint is abstracted like all endpoints (`ScanEndpoint`);
//! this slice ships the deterministic mock only. The real provider (local
//! ClamAV daemon by default) arrives with the endpoint slice — the no-HTTP
//! wall stands, and the trait is the seam it will fill.

use godhead_intake::{IntakeError, IntakePipe};
use godhead_schemas::{Manifest, QuarantineItem, ScanEngine, ScanVerdict, ScanVerdictKind};
use godhead_store::{Store, StoreError};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DeaconError {
    /// SC-I03 — the scan provider is unreachable: all items held, zero
    /// admissions, the threshold surfaces the failure. Never admit the
    /// unscanned.
    #[error("scan provider unreachable: {0}; all items held, zero admissions (Book II §1)")]
    ScannerUnreachable(String),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Intake(#[from] IntakeError),
}

/// What one scan of one item concluded. `ItemError` is the engine failing
/// on the item (verdict ERROR — held, never admissible); a provider that
/// cannot be reached at all is not an outcome but a `ScannerDown`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanOutcome {
    Clean,
    Infected(String),
    Suspect(String),
    ItemError(String),
}

impl ScanOutcome {
    pub fn verdict(&self) -> ScanVerdictKind {
        match self {
            ScanOutcome::Clean => ScanVerdictKind::Clean,
            ScanOutcome::Infected(_) => ScanVerdictKind::Infected,
            ScanOutcome::Suspect(_) => ScanVerdictKind::Suspect,
            ScanOutcome::ItemError(_) => ScanVerdictKind::Error,
        }
    }
}

/// The provider is down — SC-I03's failure behavior applies to the pass.
#[derive(Debug, Clone)]
pub struct ScannerDown(pub String);

/// The scan endpoint, abstracted (Book II §1 step 2). Default provider in
/// deployment: the local ClamAV daemon; configurable alternates. In this
/// slice, the deterministic mock below is the only implementation — the
/// trait is the seam, and no outward transport exists behind it.
#[allow(async_fn_in_trait)]
pub trait ScanEndpoint {
    async fn scan(&self, filename: &str, content: &[u8]) -> Result<ScanOutcome, ScannerDown>;
    /// The engine's identity, alias-referenced (Law XV.1).
    fn engine(&self) -> ScanEngine;
}

/// The deterministic mock scanner: verdicts are a pure function of the
/// content bytes, so every fixture reproduces. Markers:
/// `GODHEAD-MOCK:INFECTED`, `GODHEAD-MOCK:SUSPECT`, `GODHEAD-MOCK:ERROR`;
/// anything else is CLEAN. `unreachable(true)` downs the provider whole.
pub struct MockScanner {
    unreachable: std::sync::atomic::AtomicBool,
}

impl Default for MockScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl MockScanner {
    pub fn new() -> Self {
        Self {
            unreachable: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn set_unreachable(&self, down: bool) {
        self.unreachable
            .store(down, std::sync::atomic::Ordering::SeqCst);
    }

    fn contains(content: &[u8], marker: &[u8]) -> bool {
        content.windows(marker.len()).any(|w| w == marker)
    }
}

impl ScanEndpoint for MockScanner {
    async fn scan(&self, _filename: &str, content: &[u8]) -> Result<ScanOutcome, ScannerDown> {
        if self.unreachable.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(ScannerDown("mock provider is down".into()));
        }
        if Self::contains(content, b"GODHEAD-MOCK:INFECTED") {
            Ok(ScanOutcome::Infected("mock signature hit".into()))
        } else if Self::contains(content, b"GODHEAD-MOCK:SUSPECT") {
            Ok(ScanOutcome::Suspect("mock heuristic hit".into()))
        } else if Self::contains(content, b"GODHEAD-MOCK:ERROR") {
            Ok(ScanOutcome::ItemError("mock engine failed on item".into()))
        } else {
            Ok(ScanOutcome::Clean)
        }
    }

    fn engine(&self) -> ScanEngine {
        ScanEngine {
            alias: "mock-scan".into(),
            version: "1.0.0".into(),
            signature_rev: Some("mock-r1".into()),
        }
    }
}

/// The fixed servant at the gate. Holds the store and a scan endpoint;
/// every procedure is idempotent under retry (the store's convergence
/// rules), and every halt leaves no silent partial state — the store's
/// transactions are the office's halt discipline.
pub struct Deacon<'s, S, E> {
    store: &'s S,
    scanner: E,
}

impl<'s, S: Store, E: ScanEndpoint> Deacon<'s, S, E> {
    pub fn new(store: &'s S, scanner: E) -> Self {
        Self { store, scanner }
    }

    pub fn scanner(&self) -> &E {
        &self.scanner
    }

    /// Book II §1 steps 2–3: walk the mandate's held items and scan every
    /// item lacking a settled verdict (unscanned, or ERROR — an engine
    /// failure is re-scannable; INFECTED/SUSPECT/CLEAN verdicts stand
    /// until a deliberate re-scan). Provider unreachable → zero verdicts
    /// from this pass persist beyond those already written, every item
    /// stays held, and the failure surfaces (SC-I03).
    pub async fn scan_pass(&self, mandate_ref: Uuid) -> Result<Vec<ScanVerdict>, DeaconError> {
        let items = self.store.quarantine_items_for(mandate_ref).await?;
        let mut written = Vec::new();
        for item in items {
            let settled = match self.store.latest_verdict(item.item_ref).await? {
                None => false,
                Some(v) => v.verdict != ScanVerdictKind::Error,
            };
            if settled {
                continue;
            }
            let outcome = match self.scanner.scan(&item.filename, &item.content).await {
                Ok(outcome) => outcome,
                Err(ScannerDown(why)) => {
                    // Hold all, flag, surface; never admit unscanned. The
                    // failure is logged once for the pass and returned.
                    self.store
                        .write_log(
                            &format!("deacon:scan_pass:{mandate_ref}"),
                            godhead_schemas::LogEvent::ScanRecorded,
                            &serde_json::json!({
                                "outcome": "PROVIDER_UNREACHABLE",
                                "detail": why,
                            }),
                            godhead_schemas::Severity::Warning,
                        )
                        .await?;
                    return Err(DeaconError::ScannerUnreachable(why));
                }
            };
            let verdict = self
                .store
                .record_scan_verdict(item.item_ref, outcome.verdict(), &self.scanner.engine())
                .await?;
            written.push(verdict);
        }
        Ok(written)
    }

    /// Book II §1 steps 4–5: assemble and present the Manifest for one
    /// mandate-trip — items, verdicts, full chains, and the SC-I07b
    /// standing notice when volume or rate crosses the constants. One
    /// Manifest per trip, never pooled (ruling G11); re-presentation
    /// converges on the standing Manifest.
    pub async fn present_manifest(
        &self,
        mandate_ref: Uuid,
        trip_job_ref: Uuid,
    ) -> Result<Manifest, DeaconError> {
        Ok(self
            .store
            .assemble_manifest(mandate_ref, trip_job_ref)
            .await?)
    }

    /// Book II §1 step 6: one CLEAN + ADMITTED item enters the onboard
    /// pipe at its beginning — `commit_file`, the same entry a sovereign
    /// commit takes; raw copy, first log, normalization all observable, no
    /// shortcut (SC-I04) — and the admission converges exactly once
    /// (`mark_admitted`). Everything else remains in quarantine, preserved
    /// (SC-I05). The conjunction is proven before the pipe runs and
    /// re-proven when the admission is recorded; if a darker verdict lands
    /// between the two, the recording refuses and the failure surfaces —
    /// loudly, with the minted node named, never smoothed over.
    pub async fn admit(
        &self,
        pipe: &IntakePipe<'_, S>,
        item_ref: Uuid,
    ) -> Result<Uuid, DeaconError> {
        let item: QuarantineItem = self.store.clear_for_admission(item_ref).await?;
        if let Some(node) = item.admitted_node_ref {
            return Ok(node); // already through the gate; converged (Law I.3)
        }
        let node_id = pipe.commit_file(&item.filename, &item.content).await?;
        self.store.mark_admitted(item_ref, node_id).await?;
        Ok(node_id)
    }
}
