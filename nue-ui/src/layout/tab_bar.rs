use nue_core::layout::tab_manager::{
    DEFAULT_HISTORY_CAPACITY, TabManager, TabManagerError, TabSnapshot,
};

/// UI/ショートカット/Command Hubのいずれからも呼び出可能なタブ操作を抽象化するモジュール。
///
/// `TabManager` をラップし、UIからの drag/reorder、ピン・クローズ・Close Others/Close to Right、
/// 再度開く(Reopen)といった操作を統一的な API で提供する。
#[derive(Debug)]
pub struct TabBarUiController {
    tab_manager: TabManager,
}

/// 操作対象の指定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabTarget {
    /// 現在アクティブなタブ。
    Active,
    /// IDを直接指定したタブ。
    Id(String),
}

/// UI/ショートカットから使われるタブ操作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabAction {
    Pin(TabTarget),
    Unpin(TabTarget),
    Close(TabTarget),
    CloseOthers(TabTarget),
    CloseToRight(TabTarget),
    ReopenClosed,
    Reorder { target: TabTarget, index: usize },
}

/// キーボードショートカット向けの操作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabKeyboardShortcut {
    Close,
    CloseOthers,
    CloseToRight,
    ReopenClosed,
    PinToggle,
    MoveLeft,
    MoveRight,
}

/// タブ操作の実行結果イベント。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabActionEvent {
    Pinned { tab_id: String },
    Unpinned { tab_id: String },
    Closed { tab_id: String },
    ClosedOthers { tab_id: String },
    ClosedToRight { tab_id: String },
    Reopened { tab_id: String },
    Reordered { tab_id: String, new_index: usize },
}

/// 操作に失敗した場合のエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabActionError {
    NotFound(String),
    InvalidTargetIndex(usize),
    AlreadySingleTab,
    ReopenHistoryEmpty,
    MissingActiveTab,
}

impl TabBarUiController {
    /// スナップショットからタブ一覧を復元したコントローラを作成する。
    pub fn from_snapshots(tabs: Vec<TabSnapshot>) -> Self {
        Self {
            tab_manager: TabManager::from_snapshots(tabs, DEFAULT_HISTORY_CAPACITY),
        }
    }

    /// ブロックなしで使用するための簡易初期化。
    pub fn new() -> Self {
        Self {
            tab_manager: TabManager::new(DEFAULT_HISTORY_CAPACITY),
        }
    }

    /// 現在のタブスナップショットを返す。
    pub fn tabs(&self) -> Vec<TabSnapshot> {
        self.tab_manager.tabs()
    }

    /// 現在アクティブなタブID。
    pub fn active_tab_id(&self) -> Option<String> {
        self.tab_manager.active_tab_id().map(|id| id.to_string())
    }

    /// UI/Command Hubからの操作を実行する。
    pub fn handle_action(&mut self, action: TabAction) -> Result<TabActionEvent, TabActionError> {
        match action {
            TabAction::Pin(target) => {
                let tab_id = self.resolve_target(target)?;
                self.tab_manager
                    .pin(&tab_id)
                    .map_err(TabActionError::from)?;
                Ok(TabActionEvent::Pinned { tab_id })
            }
            TabAction::Unpin(target) => {
                let tab_id = self.resolve_target(target)?;
                self.tab_manager
                    .unpin(&tab_id)
                    .map_err(TabActionError::from)?;
                Ok(TabActionEvent::Unpinned { tab_id })
            }
            TabAction::Close(target) => {
                let tab_id = self.resolve_target(target)?;
                self.tab_manager
                    .close_tab(&tab_id)
                    .map_err(TabActionError::from)?;
                Ok(TabActionEvent::Closed { tab_id })
            }
            TabAction::CloseOthers(target) => {
                let tab_id = self.resolve_target(target)?;
                self.tab_manager
                    .close_others(&tab_id)
                    .map_err(TabActionError::from)?;
                Ok(TabActionEvent::ClosedOthers { tab_id })
            }
            TabAction::CloseToRight(target) => {
                let tab_id = self.resolve_target(target)?;
                self.tab_manager
                    .close_to_right(&tab_id)
                    .map_err(TabActionError::from)?;
                Ok(TabActionEvent::ClosedToRight { tab_id })
            }
            TabAction::ReopenClosed => {
                let tab_id = self
                    .tab_manager
                    .reopen_last_closed()
                    .map_err(TabActionError::from)?;
                Ok(TabActionEvent::Reopened { tab_id })
            }
            TabAction::Reorder { target, index } => {
                let tab_id = self.resolve_target(target)?;
                self.tab_manager
                    .reorder(&tab_id, index)
                    .map_err(TabActionError::from)?;
                Ok(TabActionEvent::Reordered {
                    tab_id,
                    new_index: index,
                })
            }
        }
    }

    /// タブ用ショートカットを処理する。
    pub fn handle_keyboard_shortcut(
        &mut self,
        shortcut: TabKeyboardShortcut,
    ) -> Result<TabActionEvent, TabActionError> {
        match shortcut {
            TabKeyboardShortcut::Close => self.handle_action(TabAction::Close(TabTarget::Active)),
            TabKeyboardShortcut::CloseOthers => {
                self.handle_action(TabAction::CloseOthers(TabTarget::Active))
            }
            TabKeyboardShortcut::CloseToRight => {
                self.handle_action(TabAction::CloseToRight(TabTarget::Active))
            }
            TabKeyboardShortcut::ReopenClosed => self.handle_action(TabAction::ReopenClosed),
            TabKeyboardShortcut::PinToggle => {
                let tab_id = self.resolve_active_tab()?.clone();
                let tabs = self.tab_manager.tabs();
                let is_pinned = tabs
                    .into_iter()
                    .find(|tab| tab.id == tab_id)
                    .map(|tab| tab.pinned)
                    .unwrap_or(false);
                if is_pinned {
                    self.handle_action(TabAction::Unpin(TabTarget::Id(tab_id)))
                } else {
                    self.handle_action(TabAction::Pin(TabTarget::Id(tab_id)))
                }
            }
            TabKeyboardShortcut::MoveLeft => {
                self.handle_keyboard_reorder(|current, _len| current.saturating_sub(1))
            }
            TabKeyboardShortcut::MoveRight => {
                self.handle_keyboard_reorder(|current, len| (current + 1).min(len - 1))
            }
        }
    }

    fn handle_keyboard_reorder<F>(
        &mut self,
        next_index: F,
    ) -> Result<TabActionEvent, TabActionError>
    where
        F: Fn(usize, usize) -> usize,
    {
        let tab_id = self.resolve_active_tab()?;
        let tabs = self.tab_manager.tabs();
        let current_index = tabs
            .iter()
            .position(|tab| tab.id == *tab_id)
            .ok_or_else(|| TabActionError::NotFound(tab_id.clone()))?;
        let len = tabs.len();
        let target_index = next_index(current_index, len);
        self.handle_action(TabAction::Reorder {
            target: TabTarget::Id(tab_id.clone()),
            index: target_index,
        })
    }

    fn resolve_target(&self, target: TabTarget) -> Result<String, TabActionError> {
        match target {
            TabTarget::Active => self.resolve_active_tab(),
            TabTarget::Id(id) => Ok(id),
        }
    }

    fn resolve_active_tab(&self) -> Result<String, TabActionError> {
        self.tab_manager
            .active_tab_id()
            .map(|id| id.to_string())
            .ok_or(TabActionError::MissingActiveTab)
    }
}

impl Default for TabBarUiController {
    fn default() -> Self {
        Self::new()
    }
}

impl From<TabManagerError> for TabActionError {
    fn from(error: TabManagerError) -> Self {
        match error {
            TabManagerError::TabNotFound(id) => TabActionError::NotFound(id),
            TabManagerError::InvalidTargetIndex(index) => TabActionError::InvalidTargetIndex(index),
            TabManagerError::AlreadySingleTab => TabActionError::AlreadySingleTab,
            TabManagerError::ReopenHistoryEmpty => TabActionError::ReopenHistoryEmpty,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tabs() -> Vec<TabSnapshot> {
        vec![
            TabSnapshot {
                id: "tab-1".into(),
                title: "main.rs".into(),
                pinned: false,
                is_active: true,
            },
            TabSnapshot {
                id: "tab-2".into(),
                title: "lib.rs".into(),
                pinned: false,
                is_active: false,
            },
            TabSnapshot {
                id: "tab-3".into(),
                title: "README.md".into(),
                pinned: false,
                is_active: false,
            },
        ]
    }

    #[test]
    fn pin_toggle_shortcut_tracks_current_state() {
        let mut controller = TabBarUiController::from_snapshots(sample_tabs());
        assert!(!controller.tabs()[0].pinned);

        let event = controller
            .handle_keyboard_shortcut(TabKeyboardShortcut::PinToggle)
            .unwrap();
        assert_eq!(
            event,
            TabActionEvent::Pinned {
                tab_id: "tab-1".into()
            }
        );
        assert!(controller.tabs()[0].pinned);

        let event = controller
            .handle_keyboard_shortcut(TabKeyboardShortcut::PinToggle)
            .unwrap();
        assert_eq!(
            event,
            TabActionEvent::Unpinned {
                tab_id: "tab-1".into()
            }
        );
        assert!(!controller.tabs()[0].pinned);
    }

    #[test]
    fn close_others_leaves_only_target() {
        let mut controller = TabBarUiController::from_snapshots(sample_tabs());
        controller
            .handle_action(TabAction::CloseOthers(TabTarget::Id("tab-2".into())))
            .unwrap();
        let tabs = controller.tabs();
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0].id, "tab-2");
    }

    #[test]
    fn close_to_right_removes_tail_tabs() {
        let mut controller = TabBarUiController::from_snapshots(sample_tabs());
        controller
            .handle_action(TabAction::CloseToRight(TabTarget::Id("tab-1".into())))
            .unwrap();
        let tabs = controller.tabs();
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0].id, "tab-1");
    }

    #[test]
    fn reopen_restores_latest_closed() {
        let mut controller = TabBarUiController::from_snapshots(sample_tabs());
        controller
            .handle_action(TabAction::Close(TabTarget::Id("tab-3".into())))
            .unwrap();
        assert_eq!(controller.tabs().len(), 2);

        let event = controller.handle_action(TabAction::ReopenClosed).unwrap();
        assert_eq!(
            event,
            TabActionEvent::Reopened {
                tab_id: "tab-3".into()
            }
        );
        assert_eq!(controller.tabs().len(), 3);
        assert_eq!(controller.active_tab_id().unwrap(), "tab-3");
    }

    #[test]
    fn reorder_via_action_keyboard_left_right() {
        let mut controller = TabBarUiController::from_snapshots(sample_tabs());
        // Move right
        let event = controller
            .handle_keyboard_shortcut(TabKeyboardShortcut::MoveRight)
            .unwrap();
        assert_eq!(
            event,
            TabActionEvent::Reordered {
                tab_id: "tab-1".into(),
                new_index: 1
            }
        );

        // Move left
        let event = controller
            .handle_keyboard_shortcut(TabKeyboardShortcut::MoveLeft)
            .unwrap();
        assert_eq!(
            event,
            TabActionEvent::Reordered {
                tab_id: "tab-1".into(),
                new_index: 0
            }
        );
    }
}
