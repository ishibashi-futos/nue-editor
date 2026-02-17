//! ペインごとの直前ファイル履歴を扱うモジュール。
//! `PaneHistory` はペイン単位で訪問履歴を保持し、戻る操作やスナップショットに利用可能。

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

const DEFAULT_HISTORY_CAPACITY: usize = 32;

/// ペイン履歴のスナップショット。
/// セッション復元や UI 表示のために履歴内容を外部に渡す。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaneHistorySnapshot {
    pub pane_id: String,
    pub stack: Vec<String>,
}

impl PaneHistorySnapshot {
    pub fn new(pane_id: impl Into<String>, stack: Vec<String>) -> Self {
        Self {
            pane_id: pane_id.into(),
            stack,
        }
    }
}

/// 各ペインの直前タブ履歴を保持する構造体。
/// 同一タブの連続記録を防ぎつつ、履歴長を制限することでメモリを抑える。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneHistory {
    stacks: HashMap<String, VecDeque<String>>,
    capacity: usize,
}

impl PaneHistory {
    /// 履歴容量を指定して新規作成する。
    pub fn new(capacity: usize) -> Self {
        Self {
            stacks: HashMap::new(),
            capacity: capacity.max(2),
        }
    }

    /// 指定ペインのタブ訪問を記録する。
    /// 直前と同じタブは重複して追加しない。
    pub fn record_visit(&mut self, pane_id: &str, tab_id: &str) {
        let stack = self.stacks.entry(pane_id.to_string()).or_default();
        if stack.back().map(|last| last.as_str()) == Some(tab_id) {
            return;
        }
        stack.push_back(tab_id.to_string());
        while stack.len() > self.capacity {
            stack.pop_front();
        }
    }

    /// 現在位置 (stack の末尾) より 1 つ前のタブ ID を返す。
    /// 履歴サイズが 2 未満の際は `None` を返す。
    pub fn previous(&mut self, pane_id: &str) -> Option<String> {
        let stack = self.stacks.get_mut(pane_id)?;
        if stack.len() < 2 {
            return None;
        }
        stack.pop_back();
        stack.back().cloned()
    }

    /// 指定ペイン履歴から指定タブID を除去する。
    /// 併せて履歴の連続重複を取り除き、不要なノイズを残さない。
    pub fn remove_tab(&mut self, pane_id: &str, tab_id: &str) {
        let Some(stack) = self.stacks.get_mut(pane_id) else {
            return;
        };
        let mut filtered = VecDeque::with_capacity(stack.len());
        for existing in stack.drain(..) {
            if existing == tab_id {
                continue;
            }
            if filtered
                .back()
                .map(|last| last == &existing)
                .unwrap_or(false)
            {
                continue;
            }
            filtered.push_back(existing);
        }
        let is_empty = filtered.is_empty();
        *stack = filtered;
        if is_empty {
            self.stacks.remove(pane_id);
        }
    }

    /// ペイン内の履歴をすべて破棄する。
    pub fn clear_pane(&mut self, pane_id: &str) {
        self.stacks.remove(pane_id);
    }

    /// すべての履歴を破棄する。
    pub fn clear_all(&mut self) {
        self.stacks.clear();
    }

    /// 現在保持している履歴のスナップショットを取得する。
    pub fn snapshot(&self) -> Vec<PaneHistorySnapshot> {
        self.stacks
            .iter()
            .map(|(pane_id, stack)| {
                PaneHistorySnapshot::new(pane_id, stack.iter().cloned().collect())
            })
            .collect()
    }

    /// スナップショットから履歴を再構築する。
    pub fn restore<I>(&mut self, snapshots: I)
    where
        I: IntoIterator<Item = PaneHistorySnapshot>,
    {
        self.stacks.clear();
        for snapshot in snapshots {
            let mut deque = VecDeque::new();
            for entry in snapshot.stack {
                if deque.back().map(|last| last == &entry).unwrap_or(false) {
                    continue;
                }
                deque.push_back(entry);
                while deque.len() > self.capacity {
                    deque.pop_front();
                }
            }
            if !deque.is_empty() {
                self.stacks.insert(snapshot.pane_id, deque);
            }
        }
    }

    /// テスト用: 指定ペインの履歴長を取得する。
    #[cfg(test)]
    pub fn len(&self, pane_id: &str) -> usize {
        self.stacks
            .get(pane_id)
            .map(|stack| stack.len())
            .unwrap_or(0)
    }
}

impl Default for PaneHistory {
    fn default() -> Self {
        Self::new(DEFAULT_HISTORY_CAPACITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visit_records_unique_entries_and_enforces_capacity() {
        let mut history = PaneHistory::new(3);
        history.record_visit("pane-a", "tab-1");
        history.record_visit("pane-a", "tab-2");
        history.record_visit("pane-a", "tab-2");
        history.record_visit("pane-a", "tab-3");
        history.record_visit("pane-a", "tab-4");
        assert_eq!(history.len("pane-a"), 3);
        assert_eq!(history.previous("pane-a"), Some("tab-3".to_string()));
    }

    #[test]
    fn previous_returns_none_when_insufficient_history() {
        let mut history = PaneHistory::new(4);
        history.record_visit("pane-a", "tab-1");
        assert!(history.previous("pane-a").is_none());
    }

    #[test]
    fn snapshot_and_restore_roundtrip() {
        let mut history = PaneHistory::new(10);
        history.record_visit("pane-1", "tab-a");
        history.record_visit("pane-1", "tab-b");
        history.record_visit("pane-2", "tab-x");
        let snapshots = history.snapshot();
        let mut restored = PaneHistory::new(10);
        restored.restore(snapshots);
        assert_eq!(restored.previous("pane-1"), Some("tab-a".to_string()));
        assert!(restored.previous("pane-2").is_none());
    }

    #[test]
    fn clear_pane_and_all_work() {
        let mut history = PaneHistory::new(5);
        history.record_visit("pane-1", "tab-1");
        history.clear_pane("pane-1");
        assert_eq!(history.len("pane-1"), 0);
        history.record_visit("pane-2", "tab-2");
        history.clear_all();
        assert_eq!(history.len("pane-2"), 0);
    }

    #[test]
    fn remove_tab_filters_target_and_collapses_duplicates() {
        let mut history = PaneHistory::new(10);
        history.record_visit("pane-1", "tab-1");
        history.record_visit("pane-1", "tab-2");
        history.record_visit("pane-1", "tab-1");

        history.remove_tab("pane-1", "tab-2");

        assert!(history.previous("pane-1").is_none());
    }
}
