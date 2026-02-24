use crate::{
    layout::pane_history::PaneHistorySnapshot,
    layout::pane_manager::{PaneLayoutRestoreError, PaneLayoutSnapshot, PaneManager},
    layout::tab_manager::{DEFAULT_HISTORY_CAPACITY, TabManager, TabSnapshot},
};
use serde::{Deserialize, Serialize};

/// セッション復元用に収集する状態。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub panes: Vec<PaneLayoutSnapshot>,
    pub tabs: Vec<TabSnapshot>,
    pub active_pane_id: Option<String>,
    pub pane_history: Vec<PaneHistorySnapshot>,
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
            pane_history: pane_manager.history_snapshot(),
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
        pane_manager.restore_history(self.pane_history.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::pane_manager::{PaneItem, PaneManager, PaneSplitDirection};
    use crate::layout::tab_manager::{TabDefinition, TabManager};

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
        let pane_id = pane_manager.panes()[0].id.clone();
        let initial_tab_id = pane_manager
            .layout_snapshot()
            .into_iter()
            .find(|pane| pane.id == pane_id)
            .and_then(|pane| pane.tabs.first().map(|tab| tab.id.clone()))
            .expect("initial tab exists");
        let extra_tab_id = pane_manager
            .add_tab_to_pane(&pane_id, "extra.rs")
            .expect("tab added");
        pane_manager
            .activate_tab(&pane_id, &initial_tab_id)
            .unwrap();
        pane_manager.activate_tab(&pane_id, &extra_tab_id).unwrap();
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
        assert_eq!(
            restored_pane.back_to_previous_tab(&pane_id).unwrap(),
            initial_tab_id
        );
    }

    #[test]
    fn back_to_previous_tab_tracks_history() {
        let mut manager = build_two_pane_manager();
        let pane_id = manager.panes()[0].id.clone();
        let first_tab_id = manager
            .layout_snapshot()
            .into_iter()
            .find(|pane| pane.id == pane_id)
            .and_then(|pane| pane.tabs.first().map(|tab| tab.id.clone()))
            .expect("initial tab exists");
        let new_tab_id = manager
            .add_tab_to_pane(&pane_id, "extra.rs")
            .expect("tab added");
        manager.activate_tab(&pane_id, &first_tab_id).unwrap();
        manager.activate_tab(&pane_id, &new_tab_id).unwrap();

        let previous = manager.back_to_previous_tab(&pane_id).unwrap();

        assert_eq!(previous, first_tab_id);
        let pane_title = manager
            .panes()
            .into_iter()
            .find(|pane| pane.id == pane_id)
            .unwrap()
            .title;
        assert_eq!(pane_title, "main.rs");
    }
}
