use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const DEFAULT_HISTORY_CAPACITY: usize = 32;

/// タブの状態スナップショット。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabSnapshot {
    pub id: String,
    pub title: String,
    pub pinned: bool,
    pub is_active: bool,
}

/// タブ定義。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabDefinition {
    pub title: String,
    pub pinned: bool,
}

impl TabDefinition {
    /// タイトルだけ指定してタブ定義を作成する。
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            pinned: false,
        }
    }

    /// ピン付きのタブ定義を作成する。
    pub fn pinned(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            pinned: true,
        }
    }
}

/// タブ管理者エラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabManagerError {
    TabNotFound(String),
    InvalidTargetIndex(usize),
    AlreadySingleTab,
    ReopenHistoryEmpty,
}

impl TabManagerError {
    fn not_found(tab_id: &str) -> Self {
        TabManagerError::TabNotFound(tab_id.to_string())
    }
}

#[derive(Debug, Clone)]
struct Tab {
    id: String,
    title: String,
    pinned: bool,
}

#[derive(Debug, Clone)]
struct ClosedTab {
    tab: Tab,
    index: usize,
}

/// タブの並び・アクティブ状態・履歴を管理する構造体。
#[derive(Debug, Clone)]
pub struct TabManager {
    tabs: Vec<Tab>,
    active_index: usize,
    next_tab_sequence: u64,
    closed_history: VecDeque<ClosedTab>,
    history_capacity: usize,
}

impl TabManager {
    /// 空の状態からタブマネージャーを初期化する。
    pub fn new(history_capacity: usize) -> Self {
        Self {
            tabs: Vec::new(),
            active_index: 0,
            next_tab_sequence: 1,
            closed_history: VecDeque::with_capacity(history_capacity),
            history_capacity,
        }
    }

    /// 指定した定義列からタブを構築する。
    pub fn with_definitions(defs: impl IntoIterator<Item = TabDefinition>) -> Self {
        let mut manager = Self::new(DEFAULT_HISTORY_CAPACITY);
        for def in defs {
            manager.open_tab(def.title, def.pinned);
        }
        if manager.tabs.is_empty() {
            manager.open_tab("untitled", false);
        }
        manager.active_index = 0;
        manager
    }

    /// スナップショットから状態を復元する。
    pub fn from_snapshots(
        snapshots: impl IntoIterator<Item = TabSnapshot>,
        history_capacity: usize,
    ) -> Self {
        let mut manager = Self::new(history_capacity);
        let snapshots: Vec<TabSnapshot> = snapshots.into_iter().collect();
        if snapshots.is_empty() {
            manager.open_tab("untitled", false);
            manager.active_index = 0;
            return manager;
        }
        let mut highest_seq = 0_u64;
        manager.tabs = snapshots
            .iter()
            .map(|snapshot| {
                highest_seq =
                    highest_seq.max(Self::extract_sequence(snapshot.id.as_str()).unwrap_or(0));
                Tab {
                    id: snapshot.id.clone(),
                    title: snapshot.title.clone(),
                    pinned: snapshot.pinned,
                }
            })
            .collect();
        manager.next_tab_sequence = highest_seq + 1;
        manager.active_index = snapshots
            .iter()
            .position(|snapshot| snapshot.is_active)
            .unwrap_or(0)
            .min(manager.tabs.len().saturating_sub(1));
        manager
    }

    /// タブの一覧スナップショットを返す。
    pub fn tabs(&self) -> Vec<TabSnapshot> {
        self.tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| TabSnapshot {
                id: tab.id.clone(),
                title: tab.title.clone(),
                pinned: tab.pinned,
                is_active: index == self.active_index,
            })
            .collect()
    }

    /// タブの位置を取得する。
    pub fn index_of(&self, tab_id: &str) -> Result<usize, TabManagerError> {
        self.find_index(tab_id)
    }

    /// 現在アクティブなタブのID。
    pub fn active_tab_id(&self) -> Option<&str> {
        self.tabs.get(self.active_index).map(|tab| tab.id.as_str())
    }

    /// タブを追加し、追加したタブID を返す。
    pub fn add_tab(&mut self, title: impl Into<String>, pinned: bool) -> String {
        self.open_tab(title, pinned)
    }

    /// タブを再配置する。
    pub fn reorder(&mut self, tab_id: &str, target_index: usize) -> Result<(), TabManagerError> {
        let current_index = self.find_index(tab_id)?;
        if target_index >= self.tabs.len() {
            return Err(TabManagerError::InvalidTargetIndex(target_index));
        }
        let active_id = self.active_tab_id().map(|id| id.to_string());
        let tab = self.tabs.remove(current_index);
        let insert_index = target_index.min(self.tabs.len());
        self.tabs.insert(insert_index, tab);
        if let Some(active) = active_id {
            self.active_index = self
                .find_index(&active)
                .expect("アクティブタブは常に存在するはず");
        }
        Ok(())
    }

    /// タブをピン付けする。
    pub fn pin(&mut self, tab_id: &str) -> Result<(), TabManagerError> {
        self.set_pin(tab_id, true)
    }

    /// タブのピンを外す。
    pub fn unpin(&mut self, tab_id: &str) -> Result<(), TabManagerError> {
        self.set_pin(tab_id, false)
    }

    /// 指定タブを閉じる。
    pub fn close_tab(&mut self, tab_id: &str) -> Result<(), TabManagerError> {
        if self.tabs.len() <= 1 {
            return Err(TabManagerError::AlreadySingleTab);
        }
        let index = self.find_index(tab_id)?;
        let closed = self.remove_at(index);
        self.record_closed(closed);
        Ok(())
    }

    /// 指定タブ以外を全て閉じる。
    pub fn close_others(&mut self, tab_id: &str) -> Result<(), TabManagerError> {
        let target_index = self.find_index(tab_id)?;
        let remaining_index = target_index;
        let close_indexes: Vec<_> = self
            .tabs
            .iter()
            .enumerate()
            .filter_map(|(idx, _)| if idx != target_index { Some(idx) } else { None })
            .collect();
        for index in close_indexes.iter().copied().rev() {
            let closed = self.remove_at(index);
            self.record_closed(closed);
        }
        // アクティブ位置をターゲットへ戻す
        self.active_index = remaining_index.min(self.tabs.len().saturating_sub(1));
        Ok(())
    }

    /// 指定タブの右側を全て閉じる。
    pub fn close_to_right(&mut self, tab_id: &str) -> Result<(), TabManagerError> {
        let start_index = self.find_index(tab_id)?;
        let close_indexes: Vec<_> = (start_index + 1..self.tabs.len()).collect();
        for index in close_indexes.iter().copied().rev() {
            let closed = self.remove_at(index);
            self.record_closed(closed);
        }
        Ok(())
    }

    /// 閉じたタブを最後に閉じた順で再度開く。
    pub fn reopen_last_closed(&mut self) -> Result<String, TabManagerError> {
        let closed = self
            .closed_history
            .pop_back()
            .ok_or(TabManagerError::ReopenHistoryEmpty)?;
        let insert_index = closed.index.min(self.tabs.len());
        self.tabs.insert(insert_index, closed.tab);
        self.active_index = insert_index;
        Ok(self.tabs[self.active_index].id.clone())
    }

    /// クローズ履歴のサイズ。
    pub fn closed_history_len(&self) -> usize {
        self.closed_history.len()
    }

    fn find_index(&self, tab_id: &str) -> Result<usize, TabManagerError> {
        self.tabs
            .iter()
            .position(|tab| tab.id == tab_id)
            .ok_or_else(|| TabManagerError::not_found(tab_id))
    }

    fn extract_sequence(id: &str) -> Option<u64> {
        id.strip_prefix("tab-")
            .and_then(|rest| rest.parse::<u64>().ok())
    }

    fn set_pin(&mut self, tab_id: &str, pinned: bool) -> Result<(), TabManagerError> {
        let index = self.find_index(tab_id)?;
        self.tabs[index].pinned = pinned;
        Ok(())
    }

    fn open_tab(&mut self, title: impl Into<String>, pinned: bool) -> String {
        let id = format!("tab-{}", self.next_tab_sequence);
        self.next_tab_sequence += 1;
        self.tabs.push(Tab {
            id: id.clone(),
            title: title.into(),
            pinned,
        });
        self.active_index = self.tabs.len() - 1;
        id
    }

    fn remove_at(&mut self, index: usize) -> ClosedTab {
        let was_active = self.active_index == index;
        let tab = self.tabs.remove(index);
        if self.tabs.is_empty() {
            self.active_index = 0;
        } else if was_active {
            self.active_index = index.min(self.tabs.len() - 1);
        } else if index < self.active_index {
            self.active_index -= 1;
        }
        ClosedTab { tab, index }
    }

    fn record_closed(&mut self, closed: ClosedTab) {
        if self.closed_history.len() == self.history_capacity {
            self.closed_history.pop_front();
        }
        self.closed_history.push_back(closed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_manager() -> TabManager {
        TabManager::with_definitions(vec![
            TabDefinition::new("main.rs"),
            TabDefinition::new("lib.rs"),
            TabDefinition::new("tests.rs"),
            TabDefinition::new("README.md"),
        ])
    }

    #[test]
    fn reorder_moves_tab_position() {
        let mut manager = setup_manager();
        let tab_id = manager.tabs()[0].id.clone();
        manager.reorder(&tab_id, 3).unwrap();
        assert_eq!(manager.tabs()[3].id, tab_id);
    }

    #[test]
    fn pin_and_unpin_toggle_flag() {
        let mut manager = setup_manager();
        let tab_id = manager.tabs()[1].id.clone();
        manager.pin(&tab_id).unwrap();
        assert!(manager.tabs()[1].pinned);
        manager.unpin(&tab_id).unwrap();
        assert!(!manager.tabs()[1].pinned);
    }

    #[test]
    fn close_others_keeps_target_only() {
        let mut manager = setup_manager();
        let target = manager.tabs()[2].id.clone();
        manager.close_others(&target).unwrap();
        let snapshots = manager.tabs();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].id, target);
        assert_eq!(manager.closed_history_len(), 3);
    }

    #[test]
    fn close_to_right_removes_tail_tabs() {
        let mut manager = setup_manager();
        let pivot = manager.tabs()[1].id.clone();
        manager.close_to_right(&pivot).unwrap();
        assert_eq!(manager.tabs().len(), 2);
        assert_eq!(manager.closed_history_len(), 2);
    }

    #[test]
    fn reopen_last_closed_restores_tab() {
        let mut manager = setup_manager();
        let closing = manager.tabs()[2].id.clone();
        manager.close_tab(&closing).unwrap();
        let reopened = manager.reopen_last_closed().unwrap();
        assert_eq!(reopened, closing);
        assert_eq!(manager.tabs().len(), 4);
        assert_eq!(manager.active_tab_id(), Some(reopened.as_str()));
    }

    #[test]
    fn from_snapshots_rebuilds_state_and_sequence() {
        let snapshots = vec![
            TabSnapshot {
                id: "tab-2".to_string(),
                title: "README.md".to_string(),
                pinned: false,
                is_active: false,
            },
            TabSnapshot {
                id: "tab-7".to_string(),
                title: "lib.rs".to_string(),
                pinned: true,
                is_active: true,
            },
        ];
        let mut manager = TabManager::from_snapshots(snapshots, 4);
        assert_eq!(manager.active_tab_id(), Some("tab-7"));
        assert!(manager.tabs().iter().any(|tab| tab.pinned));
        assert_eq!(manager.index_of("tab-7").unwrap(), 1);
        let appended = manager.add_tab("new.rs", false);
        assert_eq!(appended, "tab-8");
        assert_eq!(manager.tabs().len(), 3);
    }

    #[test]
    fn from_snapshots_falls_back_to_single_tab_when_empty() {
        let manager = TabManager::from_snapshots(Vec::<TabSnapshot>::new(), 3);
        let tabs = manager.tabs();
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0].title, "untitled");
        assert_eq!(manager.active_tab_id(), Some(tabs[0].id.as_str()));
    }

    #[test]
    fn index_of_returns_position_when_present() {
        let manager = setup_manager();
        let target = manager.tabs()[2].id.clone();
        assert_eq!(manager.index_of(&target).unwrap(), 2);
    }
}
