//! 検索結果リストのカーソル移動を管理するユーティリティ。
//! Global Search や Command Hub のキーバインド（Enter/F4/Shift+F4）でのナビゲーションを支援する。

use std::ops::Rem;

use crate::search::service::{SearchMatch, SearchQuery};

/// 現在の検索結果とフォーカス位置を保持するコンテキスト。
#[derive(Debug, Clone)]
pub struct SearchNavigator {
    query: Option<SearchQuery>,
    matches: Vec<SearchMatch>,
    current_index: Option<usize>,
}

impl Default for SearchNavigator {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchNavigator {
    /// 空のナビゲータを生成する。
    pub fn new() -> Self {
        Self {
            query: None,
            matches: Vec::new(),
            current_index: None,
        }
    }

    /// 検索結果をセットし、先頭を選択状態にする。
    pub fn update(&mut self, query: Option<SearchQuery>, matches: Vec<SearchMatch>) {
        self.query = query;
        self.matches = matches;
        self.current_index = if self.matches.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    /// 現在登録されている検索クエリ。
    pub fn query(&self) -> Option<&SearchQuery> {
        self.query.as_ref()
    }

    /// 現在保持している検索結果。
    pub fn matches(&self) -> &[SearchMatch] {
        &self.matches
    }

    /// 現在選択されているマッチ。
    pub fn current(&self) -> Option<&SearchMatch> {
        self.current_index.and_then(|index| self.matches.get(index))
    }

    /// 現在のフォーカスインデックス。
    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    /// フォーカスを次のエントリへ進める。末尾からループする。
    pub fn advance(&mut self) -> Option<&SearchMatch> {
        self.advance_in_direction(NavigationDirection::Forward)
    }

    /// フォーカスを前のエントリへ戻す。先頭からループする。
    pub fn retreat(&mut self) -> Option<&SearchMatch> {
        self.advance_in_direction(NavigationDirection::Backward)
    }

    /// 指定インデックスを選択する。
    pub fn select_index(&mut self, index: usize) -> Option<&SearchMatch> {
        if index >= self.matches.len() {
            return None;
        }
        self.current_index = Some(index);
        self.matches.get(index)
    }

    /// マッチが存在するか。
    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }

    fn advance_in_direction(&mut self, direction: NavigationDirection) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            self.current_index = None;
            return None;
        }

        let next_index = if let Some(current) = self.current_index {
            direction.next_index(current, self.matches.len())
        } else {
            0
        };
        self.current_index = Some(next_index);
        self.matches.get(next_index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavigationDirection {
    Forward,
    Backward,
}

impl NavigationDirection {
    fn next_index(self, current: usize, len: usize) -> usize {
        match self {
            NavigationDirection::Forward => (current + 1).rem(len),
            NavigationDirection::Backward => {
                if current == 0 {
                    len - 1
                } else {
                    current - 1
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_match(id: usize) -> SearchMatch {
        SearchMatch {
            file_path: PathBuf::from(format!("file-{}.rs", id)),
            line_number: id,
            line_text: format!("line {}", id),
            match_start: 0,
            match_end: 0,
        }
    }

    #[test]
    fn advance_cycles_across_results() {
        let mut navigator = SearchNavigator::new();
        navigator.update(
            None,
            vec![sample_match(1), sample_match(2), sample_match(3)],
        );

        assert_eq!(navigator.current().unwrap().line_number, 1);
        navigator.advance();
        assert_eq!(navigator.current().unwrap().line_number, 2);
        navigator.advance();
        assert_eq!(navigator.current().unwrap().line_number, 3);
        navigator.advance();
        assert_eq!(navigator.current().unwrap().line_number, 1);
    }

    #[test]
    fn retreat_wraps_to_tail() {
        let mut navigator = SearchNavigator::new();
        navigator.update(None, vec![sample_match(4), sample_match(5)]);

        assert_eq!(navigator.current().unwrap().line_number, 4);
        navigator.retreat();
        assert_eq!(navigator.current().unwrap().line_number, 5);
        navigator.retreat();
        assert_eq!(navigator.current().unwrap().line_number, 4);
    }

    #[test]
    fn select_index_changes_focus() {
        let mut navigator = SearchNavigator::new();
        navigator.update(
            Some(SearchQuery::literal("needle")),
            vec![sample_match(10), sample_match(20)],
        );

        assert_eq!(navigator.current().unwrap().line_number, 10);
        navigator.select_index(1);
        assert_eq!(navigator.current().unwrap().line_number, 20);
    }

    #[test]
    fn empty_results_stay_empty() {
        let mut navigator = SearchNavigator::new();
        navigator.update(None, Vec::new());

        assert!(navigator.current().is_none());
        assert!(navigator.advance().is_none());
        assert!(navigator.retreat().is_none());
        assert!(navigator.select_index(0).is_none());
    }
}
