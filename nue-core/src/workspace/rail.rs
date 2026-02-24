#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceRailState {
    Busy,
    Waiting,
    Error,
    Idle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceMetadata {
    pub workspace_id: String,
    pub root_path: String,
    pub display_name: String,
}

impl WorkspaceMetadata {
    pub fn new(
        workspace_id: impl Into<String>,
        root_path: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            root_path: root_path.into(),
            display_name: display_name.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionSource {
    Core,
    Ui,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    CoreToUi,
    UiToCore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRailTransition {
    pub from: WorkspaceRailState,
    pub to: WorkspaceRailState,
    pub source: TransitionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceRailSnapshot {
    pub state: WorkspaceRailState,
    pub revision: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceRailSyncEvent {
    pub direction: SyncDirection,
    pub state: WorkspaceRailState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionOutcome {
    Noop,
    Changed(WorkspaceRailSyncEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRailModel {
    metadata: WorkspaceMetadata,
    snapshot: WorkspaceRailSnapshot,
    transitions: Vec<WorkspaceRailTransition>,
}

impl WorkspaceRailModel {
    pub fn new(metadata: WorkspaceMetadata) -> Self {
        Self {
            metadata,
            snapshot: WorkspaceRailSnapshot {
                state: WorkspaceRailState::Idle,
                revision: 0,
            },
            transitions: Vec::new(),
        }
    }

    pub fn metadata(&self) -> &WorkspaceMetadata {
        &self.metadata
    }

    pub fn snapshot(&self) -> WorkspaceRailSnapshot {
        self.snapshot
    }

    pub fn transitions(&self) -> &[WorkspaceRailTransition] {
        &self.transitions
    }

    pub fn apply_core_state(&mut self, next: WorkspaceRailState) -> TransitionOutcome {
        self.apply_transition(next, TransitionSource::Core, SyncDirection::CoreToUi)
    }

    pub fn apply_ui_state(&mut self, next: WorkspaceRailState) -> TransitionOutcome {
        self.apply_transition(next, TransitionSource::Ui, SyncDirection::UiToCore)
    }

    fn apply_transition(
        &mut self,
        next: WorkspaceRailState,
        source: TransitionSource,
        direction: SyncDirection,
    ) -> TransitionOutcome {
        if self.snapshot.state == next {
            return TransitionOutcome::Noop;
        }

        let previous = self.snapshot.state;
        self.snapshot.state = next;
        self.snapshot.revision += 1;
        self.transitions.push(WorkspaceRailTransition {
            from: previous,
            to: next,
            source,
        });

        TransitionOutcome::Changed(WorkspaceRailSyncEvent {
            direction,
            state: next,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_model() -> WorkspaceRailModel {
        WorkspaceRailModel::new(WorkspaceMetadata::new(
            "workspace-1",
            "/tmp/workspace-1",
            "workspace-1",
        ))
    }

    #[test]
    fn 新規作成時はidleで初期化される() {
        let model = test_model();

        assert_eq!(model.snapshot().state, WorkspaceRailState::Idle);
        assert_eq!(model.snapshot().revision, 0);
        assert!(model.transitions().is_empty());
    }

    #[test]
    fn core更新時は状態遷移とui向け同期イベントを生成する() {
        let mut model = test_model();

        let result = model.apply_core_state(WorkspaceRailState::Busy);

        assert_eq!(
            result,
            TransitionOutcome::Changed(WorkspaceRailSyncEvent {
                direction: SyncDirection::CoreToUi,
                state: WorkspaceRailState::Busy,
            })
        );
        assert_eq!(model.snapshot().state, WorkspaceRailState::Busy);
        assert_eq!(model.snapshot().revision, 1);
        assert_eq!(
            model.transitions(),
            &[WorkspaceRailTransition {
                from: WorkspaceRailState::Idle,
                to: WorkspaceRailState::Busy,
                source: TransitionSource::Core,
            }]
        );
    }

    #[test]
    fn ui更新時は状態遷移とcore向け同期イベントを生成する() {
        let mut model = test_model();

        let result = model.apply_ui_state(WorkspaceRailState::Error);

        assert_eq!(
            result,
            TransitionOutcome::Changed(WorkspaceRailSyncEvent {
                direction: SyncDirection::UiToCore,
                state: WorkspaceRailState::Error,
            })
        );
        assert_eq!(model.snapshot().state, WorkspaceRailState::Error);
        assert_eq!(model.snapshot().revision, 1);
        assert_eq!(
            model.transitions(),
            &[WorkspaceRailTransition {
                from: WorkspaceRailState::Idle,
                to: WorkspaceRailState::Error,
                source: TransitionSource::Ui,
            }]
        );
    }

    #[test]
    fn 同じ状態への更新はnoopになる() {
        let mut model = test_model();

        assert_eq!(
            model.apply_core_state(WorkspaceRailState::Idle),
            TransitionOutcome::Noop
        );
        assert_eq!(model.snapshot().state, WorkspaceRailState::Idle);
        assert_eq!(model.snapshot().revision, 0);
        assert!(model.transitions().is_empty());
    }

    #[test]
    fn 遷移履歴は順序を保って保持される() {
        let mut model = test_model();

        model.apply_core_state(WorkspaceRailState::Busy);
        model.apply_ui_state(WorkspaceRailState::Waiting);

        assert_eq!(model.snapshot().state, WorkspaceRailState::Waiting);
        assert_eq!(model.snapshot().revision, 2);
        assert_eq!(
            model.transitions(),
            &[
                WorkspaceRailTransition {
                    from: WorkspaceRailState::Idle,
                    to: WorkspaceRailState::Busy,
                    source: TransitionSource::Core,
                },
                WorkspaceRailTransition {
                    from: WorkspaceRailState::Busy,
                    to: WorkspaceRailState::Waiting,
                    source: TransitionSource::Ui,
                },
            ]
        );
    }
}
