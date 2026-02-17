use crate::{
    pane_manager::{PaneLayoutRestoreError, PaneLayoutSnapshot, PaneManager},
    tab_manager::{DEFAULT_HISTORY_CAPACITY, TabManager, TabSnapshot},
};
use serde::{Deserialize, Serialize};

/// セッション復元用に収集する状態。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub panes: Vec<PaneLayoutSnapshot>,
    pub tabs: Vec<TabSnapshot>,
    pub active_pane_id: Option<String>,
}

/// 復元処理が失敗した理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionRestoreError {
    PaneLayout(PaneLayoutRestoreError),
}

impl SessionState {
    /// 現在の PaneManager / TabManager からセッション状態をキャプチャする。
    pub fn capture(pane_manager: &PaneManager, tab_manager: &TabManager) -> Self {
        Self {
            panes: pane_manager.layout_snapshot(),
            tabs: tab_manager.tabs(),
            active_pane_id: pane_manager.active_pane_id().map(|id| id.to_string()),
        }
    }

    /// 保存済み状態を復元する。復元に失敗した理由を返す。
    pub fn restore(
        &self,
        pane_manager: &mut PaneManager,
        tab_manager: &mut TabManager,
    ) -> Result<(), SessionRestoreError> {
        pane_manager
            .restore_layout(self.panes.clone())
            .map_err(SessionRestoreError::PaneLayout)?;
        *tab_manager = TabManager::from_snapshots(self.tabs.clone(), DEFAULT_HISTORY_CAPACITY);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pane_manager::{PaneItem, PaneManager, PaneSplitDirection};
    use crate::tab_manager::{TabDefinition, TabManager};

    fn build_two_pane_manager() -> PaneManager {
        PaneManager::from_items(vec![
            PaneItem {
                id: "pane-master".to_string(),
                title: "main.rs".to_string(),
                is_active: true,
            },
            PaneItem {
                id: "pane-secondary".to_string(),
                title: "lib.rs".to_string(),
                is_active: false,
            },
        ])
    }

    #[test]
    fn capture_and_restore_preserves_state() {
        let mut pane_manager = build_two_pane_manager();
        pane_manager
            .split_active(PaneSplitDirection::Right)
            .expect("split should succeed");
        let tab_manager = TabManager::with_definitions(vec![
            TabDefinition::new("main.rs"),
            TabDefinition::new("lib.rs"),
            TabDefinition::new("mod.rs"),
        ]);

        let state = SessionState::capture(&pane_manager, &tab_manager);

        let mut restored_pane = build_two_pane_manager();
        let mut restored_tabs = TabManager::with_definitions(vec![TabDefinition::new("dummy")]);

        state
            .restore(&mut restored_pane, &mut restored_tabs)
            .unwrap();

        assert_eq!(restored_pane.panes().len(), pane_manager.panes().len());
        assert_eq!(restored_tabs.tabs(), state.tabs);
        assert_eq!(
            restored_pane.active_pane_id(),
            pane_manager.active_pane_id()
        );
    }
}
