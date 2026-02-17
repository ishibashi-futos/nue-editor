use serde::{Deserialize, Serialize};

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
        self.active_index = index;
        Ok(new_id)
    }

    /// 指定ID のペインをアクティブ化する。
    pub fn activate(&mut self, pane_id: &str) -> Result<String, PaneManagerError> {
        let index = self.find_index(pane_id)?;
        self.active_index = index;
        Ok(self.panes[index].id.clone())
    }

    /// 次のペインにフォーカスを移す。
    pub fn activate_next(&mut self) -> Result<String, PaneManagerError> {
        if self.panes.is_empty() {
            return Err(PaneManagerError::NoPanes);
        }
        self.active_index = (self.active_index + 1) % self.panes.len();
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
        Ok(self.panes[self.active_index].id.clone())
    }

    /// 指定されたペインを閉じる。
    pub fn close(&mut self, pane_id: &str) -> Result<(), PaneManagerError> {
        if self.panes.len() <= 1 {
            return Err(PaneManagerError::OnlyOnePane);
        }
        let index = self.find_index(pane_id)?;
        self.panes.remove(index);
        if self.active_index >= self.panes.len() {
            self.active_index = self.panes.len() - 1;
        } else if index <= self.active_index && self.active_index > 0 {
            self.active_index -= 1;
        }
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
        new_id
    }

    /// 指定タブを別ペインへ移動する。
    pub fn move_tab(&mut self, tab_id: &str, target_pane_id: &str) -> Result<(), PaneManagerError> {
        let source_index = self
            .panes
            .iter()
            .position(|pane| pane.contains_tab(tab_id))
            .ok_or_else(|| PaneManagerError::TabNotFound(tab_id.to_string()))?;
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
        Ok(())
    }

    fn new_with_capacity(capacity: usize) -> Self {
        Self {
            panes: Vec::with_capacity(capacity),
            active_index: 0,
            next_pane_sequence: 1,
            next_tab_sequence: 1,
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
}
