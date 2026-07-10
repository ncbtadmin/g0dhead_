//! godhead-collector — the outward collection labor (Section J behavior;
//! Handbook §1.3/§1.4; docs/dev/SLICE_11.md).
//!
//! A `FETCH_PER_WRIT` (Devout) or `FETCH_PER_CANON` (Canon) trip executes a
//! human-authored mandate: it fetches the mandate's **typed locators** — a
//! writ's `demands` targets or a canon's `sources` (the 2026-07-09 ruling) —
//! and lands each result in the quarantine namespace with its ProvenanceChain
//! appended in flight. The locators come ONLY from the persisted, validated
//! mandate, never from any free text; that is the invariant SC-J05 proves.
//!
//! The outward transport is abstracted behind `FetchEndpoint`, exactly as the
//! Deacon's scan is behind `ScanEndpoint`. This slice ships the deterministic
//! **instrumented** mock only — it records every locator it is asked to fetch,
//! so the property test can compare that record against the mandate. The real
//! provider arrives with the endpoint slice; the no-HTTP wall stands until the
//! commit that makes its deletion safe (§0).

use godhead_schemas::{
    ChainEntryDraft, ChainEntryKind, Law, Locator, MandateKind, QuarantineDraft, RefusalDraft,
    RefusalReason, Tier,
};
use godhead_store::{Store, StoreError};
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum CollectorError {
    /// SC-J03 — the trip's binding failed: no resolving mandate, or the
    /// mandate's kind does not match the trip's tier.
    #[error("trip binding failed: {0}")]
    Binding(String),
    #[error(transparent)]
    Store(#[from] StoreError),
}

/// What one source yielded — the bytes and their declared shape. Whether the
/// item can be *normalized* is decided later, in the onboard pipe after
/// admission (SC-J10's raw-storage half); the fetch layer only carries what it
/// pulled.
#[derive(Debug, Clone)]
pub struct FetchedItem {
    pub filename: String,
    pub declared_type: String,
    pub content: Vec<u8>,
}

/// Why a source yielded nothing usable. SC-J10: fetch-layer garbage — corrupt,
/// deceptive, or unfetchable — is refused **at source**, recorded as an unmet
/// target, and never laundered into the quarantine namespace.
#[derive(Debug, Clone, Error)]
pub enum FetchFault {
    #[error("unreachable: {0}")]
    Unreachable(String),
    #[error("no such source: {0}")]
    NotFound(String),
    #[error("garbage at source: {0}")]
    Garbage(String),
}

/// The fetch engine's identity, alias-referenced (Law XV.1) — the seam the
/// real transport provider fills.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchEngine {
    pub alias: String,
    pub version: String,
}

/// The outward transport, abstracted (§0). The real provider (an HTTP/source
/// client) is NOT wired this slice; the deterministic mock below is the only
/// implementation, and no outward transport exists behind it.
#[allow(async_fn_in_trait)]
pub trait FetchEndpoint {
    async fn fetch(&self, locator: &Locator) -> Result<FetchedItem, FetchFault>;
    fn engine(&self) -> FetchEngine;
}

enum MockResponse {
    Item(FetchedItem),
    Fault(FetchFault),
}

/// The deterministic, **instrumented** mock fetcher: it records every locator
/// it is asked to fetch (`requested()`), so SC-J05 can assert the fetched set
/// equals the mandate's demands. Known locators return their fixture (an item
/// or a fault); an unknown locator returns a clean item derived from the
/// locator's own value, so a mandate's whole locator set is fetchable without
/// per-locator setup. `set_unreachable(true)` downs the provider whole.
#[derive(Default)]
pub struct MockFetcher {
    requested: Mutex<Vec<Locator>>,
    fixtures: HashMap<String, MockResponse>,
    unreachable: std::sync::atomic::AtomicBool,
}

impl MockFetcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// A known locator resolves to this item.
    pub fn with_item(mut self, locator_value: &str, item: FetchedItem) -> Self {
        self.fixtures
            .insert(locator_value.to_string(), MockResponse::Item(item));
        self
    }

    /// A known locator faults (SC-J10 garbage/unreachable/not-found).
    pub fn with_fault(mut self, locator_value: &str, fault: FetchFault) -> Self {
        self.fixtures
            .insert(locator_value.to_string(), MockResponse::Fault(fault));
        self
    }

    pub fn set_unreachable(&self, down: bool) {
        self.unreachable
            .store(down, std::sync::atomic::Ordering::SeqCst);
    }

    /// SC-J05's witness: exactly which locators the trip asked for.
    pub fn requested(&self) -> Vec<Locator> {
        self.requested.lock().expect("requested mutex").clone()
    }
}

impl FetchEndpoint for MockFetcher {
    async fn fetch(&self, locator: &Locator) -> Result<FetchedItem, FetchFault> {
        self.requested
            .lock()
            .expect("requested mutex")
            .push(locator.clone());
        if self.unreachable.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(FetchFault::Unreachable("mock provider is down".into()));
        }
        match self.fixtures.get(locator.value()) {
            Some(MockResponse::Item(item)) => Ok(item.clone()),
            Some(MockResponse::Fault(fault)) => Err(fault.clone()),
            None => Ok(FetchedItem {
                filename: format!("{}.txt", locator.value().replace(['/', ':'], "_")),
                declared_type: "txt".into(),
                content: format!("fixture body for {}", locator.value()).into_bytes(),
            }),
        }
    }

    fn engine(&self) -> FetchEngine {
        FetchEngine {
            alias: "mock-fetch".into(),
            version: "1.0.0".into(),
        }
    }
}

/// One target's outcome on a trip.
#[derive(Debug, Clone)]
pub struct DepositedItem {
    pub target_index: usize,
    pub item_ref: Uuid,
}

#[derive(Debug, Clone)]
pub struct UnmetTarget {
    pub target_index: usize,
    pub why: String,
}

/// What one trip collected: the items landed in quarantine (each keyed to its
/// target's index) and the targets that yielded nothing (refused at source).
#[derive(Debug, Clone, Default)]
pub struct TripSummary {
    pub mandate_ref: Option<Uuid>,
    pub deposited: Vec<DepositedItem>,
    pub unmet: Vec<UnmetTarget>,
}

/// Execute a mandate trip (`FETCH_PER_WRIT` / `FETCH_PER_CANON`).
///
/// SC-J03: the trip binds a resolving `mandate_ref` (its own `brief_ref`) whose
/// kind matches the trip's tier (WRIT→Devout, CANON→Canon); a missing mandate
/// or a cross-match refuses. SC-J05: the fetched set is drawn ONLY from the
/// mandate's persisted locators (`trip_locators`), never from free text.
/// SC-J09: each item's ProvenanceChain entry is appended BEFORE its quarantine
/// deposit. SC-J10: a source that faults is refused at source — recorded unmet,
/// never deposited. The labor rule holds: a store halt after RUNNING ends in
/// `store.refuse`, and a failed refusal write propagates.
pub async fn run_trip<S: Store, F: FetchEndpoint>(
    store: &S,
    fetcher: &F,
    trip_job_id: Uuid,
) -> Result<TripSummary, CollectorError> {
    match run_trip_inner(store, fetcher, trip_job_id).await {
        Ok(summary) => Ok(summary),
        Err(err) => {
            // BudgetExceeded is already-recorded (the store enacted it); every
            // other post-RUNNING halt ends in a refusal, and a failed refusal
            // write propagates rather than stranding the trip (labor rule).
            if !matches!(&err, CollectorError::Store(StoreError::BudgetExceeded(_))) {
                let (law, reason) = match &err {
                    CollectorError::Binding(_) => (Law::V, RefusalReason::ValidationFailed),
                    CollectorError::Store(_) => godhead_schemas::stage_code(),
                };
                store
                    .refuse(
                        trip_job_id,
                        &RefusalDraft {
                            law,
                            reason,
                            subject_refs: vec![trip_job_id.to_string()],
                            detail: "the collection trip halted after RUNNING; the job ends \
                                     refused, never stranded (Law VII; SC-J03)"
                                .to_string(),
                            preserved_refs: vec![],
                        },
                    )
                    .await?;
            }
            Err(err)
        }
    }
}

async fn run_trip_inner<S: Store, F: FetchEndpoint>(
    store: &S,
    fetcher: &F,
    trip_job_id: Uuid,
) -> Result<TripSummary, CollectorError> {
    let job = store.get_job(trip_job_id).await?;
    let mandate_ref = job.brief_ref.ok_or_else(|| {
        CollectorError::Binding(format!(
            "trip {trip_job_id} carries no mandate; a fetch executes a human charter (SC-J03)"
        ))
    })?;
    let mandate = store.get_mandate(mandate_ref).await?;

    // SC-J03: kind must match tier. WRIT→Devout, CANON→Canon; cross rejected.
    let required_tier = match mandate.kind {
        MandateKind::Writ => Tier::Devout,
        MandateKind::Canon => Tier::Canon,
    };
    if job.tier != Some(required_tier) {
        return Err(CollectorError::Binding(format!(
            "a {} mandate feeds a {required_tier} trip; this trip is {:?} (SC-J03)",
            mandate.kind.as_str(),
            job.tier
        )));
    }

    // The targets — drawn ONLY from the persisted, validated mandate (SC-J05).
    let locators = mandate.trip_locators().map_err(CollectorError::Binding)?;
    let root_kind = match mandate.kind {
        MandateKind::Writ => ChainEntryKind::Writ,
        MandateKind::Canon => ChainEntryKind::Canon,
    };

    let mut summary = TripSummary {
        mandate_ref: Some(mandate_ref),
        ..Default::default()
    };
    for (target_index, locator) in locators.iter().enumerate() {
        match fetcher.fetch(locator).await {
            Ok(item) => {
                // SC-J09 in flight: the chain entry stands before the item.
                let item_ref = Uuid::now_v7();
                store
                    .append_chain_entry(
                        trip_job_id,
                        &ChainEntryDraft {
                            chain_ref: item_ref,
                            kind: root_kind,
                            mandate_ref: Some(mandate_ref),
                            prompt_or_reason: format!(
                                "the mandate named this {} target",
                                locator.kind()
                            ),
                            produced: vec![item_ref],
                        },
                    )
                    .await?;
                store
                    .quarantine_deposit(
                        trip_job_id,
                        item_ref,
                        &QuarantineDraft {
                            mandate_ref: Some(mandate_ref),
                            brief_ref: None,
                            filename: item.filename,
                            declared_type: item.declared_type,
                            content: item.content,
                        },
                    )
                    .await?;
                summary.deposited.push(DepositedItem {
                    target_index,
                    item_ref,
                });
            }
            Err(fault) => {
                // SC-J10: refused at source — never laundered into quarantine.
                summary.unmet.push(UnmetTarget {
                    target_index,
                    why: fault.to_string(),
                });
            }
        }
    }
    Ok(summary)
}
