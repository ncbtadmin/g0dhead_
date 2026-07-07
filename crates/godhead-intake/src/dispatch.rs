use crate::pipe::IntakePipe;
use crate::{IntakeError, STAGE_NORMALIZE, STAGE_RAW_COPY};
use godhead_schemas::FlagStatus;
use godhead_store::Store;
use uuid::Uuid;

/// What one tick ran: the stage executed for a node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchedStage {
    pub node_id: Uuid,
    pub stage: &'static str,
}

/// The thin dispatcher (doc 3 §3.2): watches readiness flags, invokes the
/// mapped successor stage, consumes the flag. Intelligence lives in the
/// state, not here — the whole successor map is two entries, and it ends
/// at CLASSIFY. **The seam (doc 2 §4.1) is the absence of any rule beyond
/// at-rest**: nothing in this dispatcher can carry a node further, and no
/// other dispatcher exists.
pub struct Dispatcher<'p, 's, S> {
    pipe: &'p IntakePipe<'s, S>,
}

impl<'p, 's, S: Store> Dispatcher<'p, 's, S> {
    pub fn new(pipe: &'p IntakePipe<'s, S>) -> Self {
        Self { pipe }
    }

    /// One deterministic pass over every ACTIVE intake flag in the store —
    /// the production mode: one dispatcher, one data root.
    pub async fn tick(&self) -> Result<Vec<DispatchedStage>, IntakeError> {
        self.tick_scoped(None).await
    }

    /// One deterministic pass, optionally scoped to a node set. The
    /// ACTIVE-flag lists are snapshotted before any work, so a node
    /// advances at most one stage per tick — which is what makes every
    /// stage boundary a testable kill point (SC-N05). Scoping exists for
    /// multi-root deployments and test isolation; the successor logic is
    /// identical either way.
    pub async fn tick_scoped(
        &self,
        scope: Option<&[Uuid]>,
    ) -> Result<Vec<DispatchedStage>, IntakeError> {
        let store = self.pipe.store();
        let raw_flags = store.list_active_flags(STAGE_RAW_COPY).await?;
        let normalize_flags = store.list_active_flags(STAGE_NORMALIZE).await?;
        let mut ran = Vec::new();
        for flag in &raw_flags {
            if !self.in_scope(flag, scope).await? {
                continue;
            }
            let node_id = self.pipe.run_normalize(flag).await?;
            store
                .supersede_flag(flag.flag_id, flag.revision, FlagStatus::Consumed)
                .await?;
            ran.push(DispatchedStage {
                node_id,
                stage: STAGE_NORMALIZE,
            });
        }
        for flag in &normalize_flags {
            if !self.in_scope(flag, scope).await? {
                continue;
            }
            let node_id = self.pipe.run_classify(flag).await?;
            store
                .supersede_flag(flag.flag_id, flag.revision, FlagStatus::Consumed)
                .await?;
            ran.push(DispatchedStage {
                node_id,
                stage: crate::STAGE_CLASSIFY,
            });
        }
        Ok(ran)
    }

    async fn in_scope(
        &self,
        flag: &godhead_schemas::ReadinessFlag,
        scope: Option<&[Uuid]>,
    ) -> Result<bool, IntakeError> {
        match scope {
            None => Ok(true),
            Some(nodes) => {
                let node_id = self.pipe.node_ref_of(flag).await?;
                Ok(nodes.contains(&node_id))
            }
        }
    }
}
