use crate::IntakeError;
use godhead_schemas::FlagStatus;
use godhead_store::Store;
use uuid::Uuid;

/// Where one node stands, reconstructed from the record alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeProgress {
    pub node_id: Uuid,
    /// Stage names certified for this node (ACTIVE or CONSUMED flags),
    /// sorted and deduplicated.
    pub stages_flagged: Vec<String>,
    /// Intake jobs not yet TERMINATED or REFUSED — recovery's concern.
    pub jobs_in_flight: usize,
}

/// The observing supervisor (doc 3 §3.2): answers "where is everything?"
/// by reading flags and job records — never private memory. On restart it
/// rebuilds this index from persisted state alone; no in-flight knowledge
/// is lost because none lived only in a worker (doc 3 §4.1).
pub struct Supervisor<'s, S> {
    store: &'s S,
}

impl<'s, S: Store> Supervisor<'s, S> {
    pub fn new(store: &'s S) -> Self {
        Self { store }
    }

    pub async fn reconstruct(&self, node_ids: &[Uuid]) -> Result<Vec<NodeProgress>, IntakeError> {
        let mut progress = Vec::with_capacity(node_ids.len());
        for &node_id in node_ids {
            let jobs = self.store.list_jobs_by_input_ref(node_id).await?;
            let mut stages = Vec::new();
            let mut in_flight = 0;
            for job in &jobs {
                if !job.status.is_terminal() {
                    in_flight += 1;
                }
                for flag in self.store.list_flags_for_job(job.job_id).await? {
                    if matches!(flag.status, FlagStatus::Active | FlagStatus::Consumed) {
                        stages.push(flag.stage);
                    }
                }
            }
            stages.sort();
            stages.dedup();
            progress.push(NodeProgress {
                node_id,
                stages_flagged: stages,
                jobs_in_flight: in_flight,
            });
        }
        Ok(progress)
    }
}
