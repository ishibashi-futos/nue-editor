use crate::pane_history::{PaneHistory, PaneHistorySnapshot};
use serde::{Deserialize, Serialize};

const HISTORY_CAPACITY: usize = 32;

/// ペイン分割の方向を表す列挙型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneSplitDirection {
    Left,
    Right,
    Up,
    Down,
}

/// UI に公開するペイン情報のスナップショット。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneItem {
    pub id: String,
    pub title: String,
    pub is_active: bool,
}

/// セッション復元に向けたペイン内タブのスナップショット。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaneTabSnapshot {
    pub id: String,
    pub title: String,
}

/// ペイン構成・タブ順序・アクション状態を含むスナップショット。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaneLayoutSnapshot {
    pub id: String,
    pub tabs: Vec<PaneTabSnapshot>,
    pub active_tab_id: Option<String>,
    pub is_active: bool,
}

/// セッションからの復元中に発生するエラー種別。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneLayoutRestoreError {
    EmptyLayout,
    EmptyPane(String),
    MultipleActivePanes,
}

/// PaneManager が返す操作エラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneManagerError {
    NoPanes,
    PaneNotFound(String),
    OnlyOnePane,
    TabNotFound(String),
    SingleTabPane(String),
    HistoryUnavailable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PaneState {
    id: String,
    tabs: Vec<PaneTab>,
    active_tab_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PaneTab {
    id: String,
    title: String,
}

/// ペインとタブの整合性を管理するオーケストレータ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneManager {
    panes: Vec<PaneState>,
    active_index: usize,
    next_pane_sequence: u64,
    next_tab_sequence: u64,
    history: PaneHistory,
}

impl PaneManager {
    /// 初期状態を PaneItem から再構築する。
    pub fn from_items(items: Vec<PaneItem>) -> Self {
        let mut manager = Self::new_with_capacity(items.len().max(1));
        let mut explicit_active = None;
        for item in items {
            manager.track_pane_sequence(&item.id);
            let tab_id = manager.allocate_tab_id();
            manager.panes.push(PaneState {
                id: item.id.clone(),
                tabs: vec![PaneTab {
                    id: tab_id,
                    title: item.title,
                }],
                active_tab_index: 0,
            });
            if item.is_active {
                explicit_active = Some(manager.panes.len() - 1);
            }
        }
        if manager.panes.is_empty() {
            manager.push_blank_pane();
            explicit_active = Some(0);
        }
        manager.active_index = explicit_active.unwrap_or(0).min(manager.panes.len() - 1);
        manager.record_initial_history();
        manager
    }

    /// 現在のペイン構成をスナップショットとして取得する。
    pub fn layout_snapshot(&self) -> Vec<PaneLayoutSnapshot> {
        self.panes
            .iter()
            .enumerate()
            .map(|(index, pane)| PaneLayoutSnapshot {
                id: pane.id.clone(),
                tabs: pane
                    .tabs
                    .iter()
                    .map(|tab| PaneTabSnapshot {
                        id: tab.id.clone(),
                        title: tab.title.clone(),
                    })
                    .collect(),
                active_tab_id: pane
                    .tabs
                    .get(pane.active_tab_index)
                    .map(|tab| tab.id.clone()),
                is_active: index == self.active_index,
            })
            .collect()
    }

    /// スナップショットからペイン構成を再構築する。
    pub fn restore_layout(
        &mut self,
        snapshots: Vec<PaneLayoutSnapshot>,
    ) -> Result<(), PaneLayoutRestoreError> {
        if snapshots.is_empty() {
            return Err(PaneLayoutRestoreError::EmptyLayout);
        }
        let mut reconstructed = Vec::with_capacity(snapshots.len());
        let mut max_pane_seq = 0_u64;
        let mut max_tab_seq = 0_u64;
        let mut active_index = None;

        for (index, layout_snapshot) in snapshots.into_iter().enumerate() {
            let PaneLayoutSnapshot {
                id,
                tabs,
                active_tab_id,
                is_active,
            } = layout_snapshot;
            if tabs.is_empty() {
                return Err(PaneLayoutRestoreError::EmptyPane(id));
            }

            max_pane_seq = max_pane_seq.max(Self::extract_sequence(id.as_str()).unwrap_or(0));
            let mut pane_tabs = Vec::with_capacity(tabs.len());
            let mut active_tab_index = 0;
            let mut found_active_tab = false;

            for (tab_index, pane_tab_snapshot) in tabs.into_iter().enumerate() {
                max_tab_seq = max_tab_seq
                    .max(Self::extract_tab_sequence(pane_tab_snapshot.id.as_str()).unwrap_or(0));
                if !found_active_tab
                    && active_tab_id
                        .as_deref()
                        .is_some_and(|target| target == pane_tab_snapshot.id)
                {
                    active_tab_index = tab_index;
                    found_active_tab = true;
                }
                pane_tabs.push(PaneTab {
                    id: pane_tab_snapshot.id,
                    title: pane_tab_snapshot.title,
                });
            }

            reconstructed.push(PaneState {
                id: id.clone(),
                tabs: pane_tabs,
                active_tab_index,
            });

            if is_active {
                if active_index.is_some() {
                    return Err(PaneLayoutRestoreError::MultipleActivePanes);
                }
                active_index = Some(index);
            }
        }

        self.panes = reconstructed;
        self.active_index = active_index
            .unwrap_or(0)
            .min(self.panes.len().saturating_sub(1));
        self.next_pane_sequence = max_pane_seq + 1;
        self.next_tab_sequence = max_tab_seq + 1;
        Ok(())
    }

    /// 現在のペイン一覧を PaneItem として返す。
    pub fn panes(&self) -> Vec<PaneItem> {
        self.panes
            .iter()
            .enumerate()
            .map(|(index, pane)| PaneItem {
                id: pane.id.clone(),
                title: pane.active_tab().title.clone(),
                is_active: index == self.active_index,
            })
            .collect()
    }

    /// カーソルが向いているペインID を返す。
    pub fn active_pane_id(&self) -> Option<&str> {
        self.panes
            .get(self.active_index)
            .map(|pane| pane.id.as_str())
    }

    /// 指定方向でアクティブペインを分割し、新しいペインID を返す。
    pub fn split_active(
        &mut self,
        direction: PaneSplitDirection,
    ) -> Result<String, PaneManagerError> {
        let previous_active_index = self.active_index;
        let (titles, active_tab_index) = {
            let active = self
                .panes
                .get(self.active_index)
                .ok_or(PaneManagerError::NoPanes)?;
            let titles = active
                .tabs
                .iter()
                .map(|tab| tab.title.clone())
                .collect::<Vec<_>>();
            let index = active
                .active_tab_index
                .min(active.tabs.len().saturating_sub(1));
            (titles, index)
        };
        let duplicated_tabs = titles
            .into_iter()
            .map(|title| PaneTab {
                id: self.allocate_tab_id(),
                title,
            })
            .collect::<Vec<_>>();
        let new_id = self.allocate_pane_id();
        let new_state = PaneState {
            id: new_id.clone(),
            tabs: duplicated_tabs,
            active_tab_index,
        };
        let insert_index = match direction {
            PaneSplitDirection::Left | PaneSplitDirection::Up => self.active_index,
            PaneSplitDirection::Right | PaneSplitDirection::Down => self.active_index + 1,
        };
        let index = insert_index.min(self.panes.len());
        self.panes.insert(index, new_state);
        let old_index = if insert_index <= previous_active_index {
            previous_active_index + 1
        } else {
            previous_active_index
        };
        self.active_index = index;
        if old_index < self.panes.len() {
            self.record_visit_for_pane(old_index);
        }
        self.record_visit_for_pane(index);
        Ok(new_id)
    }

    /// 指定ID のペインをアクティブ化する。
    pub fn activate(&mut self, pane_id: &str) -> Result<String, PaneManagerError> {
        let index = self.find_index(pane_id)?;
        self.active_index = index;
        self.record_visit_for_pane(index);
        Ok(self.panes[index].id.clone())
    }

    /// 次のペインにフォーカスを移す。
    pub fn activate_next(&mut self) -> Result<String, PaneManagerError> {
        if self.panes.is_empty() {
            return Err(PaneManagerError::NoPanes);
        }
        self.active_index = (self.active_index + 1) % self.panes.len();
        self.record_visit_for_pane(self.active_index);
        Ok(self.panes[self.active_index].id.clone())
    }

    /// 前のペインにフォーカスを移す。
    pub fn activate_prev(&mut self) -> Result<String, PaneManagerError> {
        if self.panes.is_empty() {
            return Err(PaneManagerError::NoPanes);
        }
        if self.active_index == 0 {
            self.active_index = self.panes.len() - 1;
        } else {
            self.active_index -= 1;
        }
        self.record_visit_for_pane(self.active_index);
        Ok(self.panes[self.active_index].id.clone())
    }

    /// 指定されたペインを閉じる。
    pub fn close(&mut self, pane_id: &str) -> Result<(), PaneManagerError> {
        if self.panes.len() <= 1 {
            return Err(PaneManagerError::OnlyOnePane);
        }
        let index = self.find_index(pane_id)?;
        let removed = self.panes.remove(index);
        self.history.clear_pane(&removed.id);
        if self.active_index >= self.panes.len() {
            self.active_index = self.panes.len() - 1;
        } else if index <= self.active_index && self.active_index > 0 {
            self.active_index -= 1;
        }
        self.record_visit_for_pane(self.active_index);
        Ok(())
    }

    /// アクティブペインの横に新しいペインを開いてタイトルを設定する。
    pub fn open_to_side(&mut self, title: impl Into<String>) -> String {
        let new_id = self.allocate_pane_id();
        let tab_id = self.allocate_tab_id();
        let new_state = PaneState {
            id: new_id.clone(),
            tabs: vec![PaneTab {
                id: tab_id,
                title: title.into(),
            }],
            active_tab_index: 0,
        };
        let insert_index = (self.active_index + 1).min(self.panes.len());
        self.panes.insert(insert_index, new_state);
        self.active_index = insert_index;
        self.record_visit_for_pane(self.active_index);
        new_id
    }

    /// 指定タブを別ペインへ移動する。
    pub fn move_tab(&mut self, tab_id: &str, target_pane_id: &str) -> Result<(), PaneManagerError> {
        let source_index = self
            .panes
            .iter()
            .position(|pane| pane.contains_tab(tab_id))
            .ok_or_else(|| PaneManagerError::TabNotFound(tab_id.to_string()))?;
        let source_pane_id = self.panes[source_index].id.clone();
        if self.panes[source_index].tabs.len() <= 1 {
            return Err(PaneManagerError::SingleTabPane(
                self.panes[source_index].id.clone(),
            ));
        }
        let tab = self.panes[source_index]
            .remove_tab(tab_id)
            .ok_or_else(|| PaneManagerError::TabNotFound(tab_id.to_string()))?;
        let target_index = self.find_index(target_pane_id)?;
        self.panes[target_index].tabs.push(tab);
        self.panes[target_index].active_tab_index = self.panes[target_index].tabs.len() - 1;
        if source_index < self.panes.len() {
            self.record_visit_for_pane(source_index);
        }
        self.history.remove_tab(&source_pane_id, tab_id);
        self.record_visit_for_pane(target_index);
        Ok(())
    }

    /// 指定ペインにタブを追加し、そのタブID を返す。
    pub fn add_tab_to_pane(
        &mut self,
        pane_id: &str,
        title: impl Into<String>,
    ) -> Result<String, PaneManagerError> {
        let index = self.find_index(pane_id)?;
        let tab_id = self.allocate_tab_id();
        self.panes[index].tabs.push(PaneTab {
            id: tab_id.clone(),
            title: title.into(),
        });
        self.panes[index].active_tab_index = self.panes[index].tabs.len() - 1;
        self.record_visit_for_pane(index);
        Ok(tab_id)
    }

    /// 指定ペイン内のタブをアクティブ化する。
    pub fn activate_tab(
        &mut self,
        pane_id: &str,
        tab_id: &str,
    ) -> Result<String, PaneManagerError> {
        let index = self.find_index(pane_id)?;
        let tab_index = self.panes[index]
            .tabs
            .iter()
            .position(|tab| tab.id == tab_id)
            .ok_or_else(|| PaneManagerError::TabNotFound(tab_id.to_string()))?;
        self.panes[index].active_tab_index = tab_index;
        self.record_visit_for_pane(index);
        Ok(tab_id.to_string())
    }

    /// 指定ペインで直前にアクティブだったタブへ戻る。
    pub fn back_to_previous_tab(&mut self, pane_id: &str) -> Result<String, PaneManagerError> {
        let index = self.find_index(pane_id)?;
        let previous_id = self
            .history
            .previous(pane_id)
            .ok_or_else(|| PaneManagerError::HistoryUnavailable(pane_id.to_string()))?;
        let tab_index = self.panes[index]
            .tabs
            .iter()
            .position(|tab| tab.id == previous_id)
            .ok_or_else(|| PaneManagerError::TabNotFound(previous_id.clone()))?;
        self.panes[index].active_tab_index = tab_index;
        self.record_visit_for_pane(index);
        Ok(previous_id)
    }

    /// 履歴スナップショットを取得する。
    pub fn history_snapshot(&self) -> Vec<PaneHistorySnapshot> {
        self.history.snapshot()
    }

    /// 保存済み履歴を復元する。
    pub fn restore_history(&mut self, snapshots: Vec<PaneHistorySnapshot>) {
        self.history.restore(snapshots);
    }

    fn new_with_capacity(capacity: usize) -> Self {
        Self {
            panes: Vec::with_capacity(capacity),
            active_index: 0,
            next_pane_sequence: 1,
            next_tab_sequence: 1,
            history: PaneHistory::new(HISTORY_CAPACITY),
        }
    }

    fn push_blank_pane(&mut self) {
        let id = self.allocate_pane_id();
        let tab_id = self.allocate_tab_id();
        self.panes.push(PaneState {
            id,
            tabs: vec![PaneTab {
                id: tab_id,
                title: "untitled".to_string(),
            }],
            active_tab_index: 0,
        });
    }

    fn allocate_pane_id(&mut self) -> String {
        let id = format!("pane-{}", self.next_pane_sequence);
        self.next_pane_sequence += 1;
        id
    }

    fn allocate_tab_id(&mut self) -> String {
        let id = format!("tab-{}", self.next_tab_sequence);
        self.next_tab_sequence += 1;
        id
    }

    fn find_index(&self, pane_id: &str) -> Result<usize, PaneManagerError> {
        self.panes
            .iter()
            .position(|pane| pane.id == pane_id)
            .ok_or_else(|| PaneManagerError::PaneNotFound(pane_id.to_string()))
    }

    fn track_pane_sequence(&mut self, candidate: &str) {
        if let Some(number) = candidate
            .strip_prefix("pane-")
            .and_then(|rest| rest.parse::<u64>().ok())
        {
            self.next_pane_sequence = self.next_pane_sequence.max(number + 1);
        }
    }

    fn extract_sequence(id: &str) -> Option<u64> {
        id.strip_prefix("pane-")
            .and_then(|rest| rest.parse::<u64>().ok())
    }

    fn extract_tab_sequence(id: &str) -> Option<u64> {
        id.strip_prefix("tab-")
            .and_then(|rest| rest.parse::<u64>().ok())
    }

    fn record_initial_history(&mut self) {
        for index in 0..self.panes.len() {
            self.record_visit_for_pane(index);
        }
    }

    fn record_visit_for_pane(&mut self, pane_index: usize) {
        if let Some(pane) = self.panes.get(pane_index)
            && let Some(tab) = pane.tabs.get(pane.active_tab_index)
        {
            self.history.record_visit(&pane.id, &tab.id);
        }
    }
}

impl PaneState {
    fn active_tab(&self) -> &PaneTab {
        &self.tabs[self.active_tab_index]
    }

    fn contains_tab(&self, tab_id: &str) -> bool {
        self.tabs.iter().any(|tab| tab.id == tab_id)
    }

    fn remove_tab(&mut self, tab_id: &str) -> Option<PaneTab> {
        let position = self.tabs.iter().position(|tab| tab.id == tab_id)?;
        let removed = self.tabs.remove(position);
        if self.active_tab_index >= self.tabs.len() && !self.tabs.is_empty() {
            self.active_tab_index = self.tabs.len() - 1;
        }
        Some(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manager_with_two_panes() -> PaneManager {
        PaneManager::from_items(vec![
            PaneItem {
                id: "pane-1".to_string(),
                title: "README.md".to_string(),
                is_active: true,
            },
            PaneItem {
                id: "pane-2".to_string(),
                title: "spec.md".to_string(),
                is_active: false,
            },
        ])
    }

    #[test]
    fn split_active_creates_new_pane_and_moves_focus() {
        let mut manager = manager_with_two_panes();
        assert_eq!(manager.panes().len(), 2);

        let new_id = manager
            .split_active(PaneSplitDirection::Right)
            .expect("split に成功するはず");
        assert_eq!(new_id, "pane-3");
        let panes = manager.panes();
        assert_eq!(panes.len(), 3);
        assert!(panes[1].is_active);
        assert!(!panes[2].is_active);
    }

    #[test]
    fn activate_next_and_prev_wraps() {
        let mut manager = manager_with_two_panes();
        assert_eq!(manager.activate_next().unwrap(), "pane-2");
        assert_eq!(manager.activate_next().unwrap(), "pane-1");
        assert_eq!(manager.activate_prev().unwrap(), "pane-2");
    }

    #[test]
    fn close_deactivates_target_when_only_one_remaining() {
        let mut manager = manager_with_two_panes();
        manager.activate("pane-2").unwrap();
        manager.close("pane-1").unwrap();
        assert_eq!(manager.panes().len(), 1);
        assert_eq!(manager.active_pane_id(), Some("pane-2"));
    }

    #[test]
    fn open_to_side_inserts_new_pane_next_to_active() {
        let mut manager = manager_with_two_panes();
        let new_id = manager.open_to_side("Commands");
        assert_eq!(new_id, "pane-3");
        let panes = manager.panes();
        assert_eq!(panes.len(), 3);
        assert!(panes[1].is_active);
    }

    #[test]
    fn move_tab_transfers_between_panes() {
        let mut manager = manager_with_two_panes();
        // 先頭ペインに追加のタブを配置して移動対象を作る
        manager.panes[0].tabs.push(PaneTab {
            id: "tab-extra".to_string(),
            title: "extra.md".to_string(),
        });
        let tab_id = "tab-extra";
        manager.move_tab(tab_id, "pane-2").unwrap();
        assert_eq!(manager.panes()[0].title, "README.md");
        assert_eq!(manager.panes()[1].title, "extra.md");
    }

    #[test]
    fn layout_snapshot_and_restore_roundtrip() {
        let mut manager = manager_with_two_panes();
        manager.activate("pane-2").unwrap();
        let snapshot = manager.layout_snapshot();

        let mut restored = manager_with_two_panes();
        restored.restore_layout(snapshot).unwrap();

        assert_eq!(restored.panes().len(), 2);
        assert_eq!(restored.active_pane_id(), Some("pane-2"));
    }

    #[test]
    fn restore_layout_fails_on_empty_snapshot() {
        let mut manager = manager_with_two_panes();
        assert_eq!(
            manager.restore_layout(Vec::new()),
            Err(PaneLayoutRestoreError::EmptyLayout)
        );
    }

    #[test]
    fn restore_layout_fails_on_multiple_active_panes() {
        let mut manager = manager_with_two_panes();
        let mut snapshot = manager.layout_snapshot();
        snapshot[0].is_active = true;
        snapshot[1].is_active = true;

        assert_eq!(
            manager.restore_layout(snapshot),
            Err(PaneLayoutRestoreError::MultipleActivePanes)
        );
    }

    #[test]
    fn back_to_previous_tab_skips_tabs_moved_out_from_history() {
        let mut manager = manager_with_two_panes();
        let initial_tab = manager.layout_snapshot()[0]
            .active_tab_id
            .clone()
            .expect("active tab が存在する");
        manager
            .add_tab_to_pane("pane-1", "previous.md")
            .expect("tab 追加成功");
        let moved_tab = manager
            .add_tab_to_pane("pane-1", "moved.md")
            .expect("tab 追加成功");

        manager
            .move_tab(&moved_tab, "pane-2")
            .expect("tab 移動成功");

        let returned = manager
            .back_to_previous_tab("pane-1")
            .expect("有効タブへ戻れるはず");
        assert_eq!(returned, initial_tab);
    }
}
