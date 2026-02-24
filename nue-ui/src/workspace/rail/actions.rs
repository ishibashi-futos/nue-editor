use std::collections::VecDeque;

/// UI からアプリ層へ通知する Workspace Rail 操作イベント。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceRailUiAction {
    RequestOpenAddDialog,
    RequestSubmitAddWorkspace { input_path: String },
    RequestOpenExcludeMenu { workspace_id: String },
    RequestExcludeWorkspace { workspace_id: String },
    RequestSelectWorkspace { workspace_id: String },
}

/// GPUI イベントハンドラからアプリ層への通知を中継する最小キュー。
#[derive(Debug, Clone, Default)]
pub struct WorkspaceRailActionQueue {
    pending: VecDeque<WorkspaceRailUiAction>,
}

impl WorkspaceRailActionQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, action: WorkspaceRailUiAction) {
        self.pending.push_back(action);
    }

    pub fn drain(&mut self) -> Vec<WorkspaceRailUiAction> {
        self.pending.drain(..).collect()
    }
}
