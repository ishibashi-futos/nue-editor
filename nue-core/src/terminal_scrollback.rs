use std::collections::VecDeque;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// スクロールバックの既定行数。
pub const SCROLLBACK_DEFAULT_LINES: usize = 500;
const TAB_STOP_COLUMNS: usize = 8;
const ANSI_ESCAPE: char = '\u{1b}';
const ZERO_WIDTH_JOINER: char = '\u{200d}';

/// 1 行分の表示セル。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineCell {
    ch: char,
    width: usize,
}

impl LineCell {
    fn new(ch: char, width: usize) -> Self {
        Self { ch, width }
    }

    /// セルに表示されている文字。
    pub fn ch(&self) -> char {
        self.ch
    }

    /// セルの幅（半角=1, 全角=2, 制御文字=0）。
    pub fn width(&self) -> usize {
        self.width
    }
}

/// スクロールバックの 1 行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollbackLine {
    raw: String,
    width: usize,
    cells: Vec<LineCell>,
}

impl ScrollbackLine {
    fn new(raw: String) -> Self {
        let raw = strip_ansi_escape_sequences(&raw);
        let mut width = 0;
        let mut cells = Vec::new();
        append_cells_from_text(&raw, &mut cells, &mut width);

        Self { raw, width, cells }
    }

    /// 行の文字列（末尾の改行は含まれない）。
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// 行全体の文字幅。
    pub fn width(&self) -> usize {
        self.width
    }

    /// 各セルの情報。
    pub fn cells(&self) -> &[LineCell] {
        &self.cells
    }
}

/// スクロールバックを保持する構造体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scrollback {
    max_lines: usize,
    lines: VecDeque<ScrollbackLine>,
}

impl Scrollback {
    /// 指定した行数でスクロールバックを初期化する。
    pub fn new(max_lines: usize) -> Self {
        let max_lines = max_lines.max(1);
        Self {
            max_lines,
            lines: VecDeque::with_capacity(max_lines),
        }
    }

    /// 新しい行を末尾に追加する。容量を超えたら先頭が破棄される。
    pub fn push_line(&mut self, raw: impl Into<String>) {
        if self.lines.len() >= self.max_lines {
            self.lines.pop_front();
        }

        let line = ScrollbackLine::new(raw.into());
        self.lines.push_back(line);
    }

    /// 保持している行を先頭から順に列挙するイテレータ。
    pub fn lines(&self) -> impl Iterator<Item = &ScrollbackLine> {
        self.lines.iter()
    }

    /// 末尾の行（存在する場合）。
    pub fn last_line(&self) -> Option<&ScrollbackLine> {
        self.lines.back()
    }

    /// 現在保持している行数。
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// 最大行数。
    pub fn capacity(&self) -> usize {
        self.max_lines
    }

    /// 空かどうか。
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

impl Default for Scrollback {
    fn default() -> Self {
        Self::new(SCROLLBACK_DEFAULT_LINES)
    }
}

fn append_cells_from_text(raw: &str, cells: &mut Vec<LineCell>, width: &mut usize) {
    let mut chars = raw.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\t' {
            let tab_padding = tab_padding(*width);
            for _ in 0..tab_padding {
                cells.push(LineCell::new(' ', 1));
            }
            *width += tab_padding;
            continue;
        }

        let mut cluster = String::new();
        cluster.push(ch);
        extend_grapheme_cluster(&mut chars, &mut cluster);
        let cluster_width = UnicodeWidthStr::width(cluster.as_str());
        append_cluster_cells(cells, &cluster, cluster_width);
        *width += cluster_width;
    }
}

fn append_cluster_cells(cells: &mut Vec<LineCell>, cluster: &str, cluster_width: usize) {
    let mut cluster_chars = cluster.chars();
    if let Some(first) = cluster_chars.next() {
        cells.push(LineCell::new(first, cluster_width));
    }

    for ch in cluster_chars {
        cells.push(LineCell::new(ch, 0));
    }
}

fn extend_grapheme_cluster(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    cluster: &mut String,
) {
    loop {
        let Some(&next) = chars.peek() else {
            return;
        };

        if is_zero_width_extension(next) {
            cluster.push(next);
            chars.next();
            continue;
        }

        if next != ZERO_WIDTH_JOINER {
            return;
        }

        cluster.push(next);
        chars.next();

        let Some(joined) = chars.next() else {
            return;
        };
        cluster.push(joined);
    }
}

fn is_zero_width_extension(ch: char) -> bool {
    UnicodeWidthChar::width(ch).unwrap_or(0) == 0
        && !matches!(ch, '\n' | '\r' | '\t')
        && ch != ZERO_WIDTH_JOINER
}

fn tab_padding(current_width: usize) -> usize {
    let remainder = current_width % TAB_STOP_COLUMNS;
    if remainder == 0 {
        TAB_STOP_COLUMNS
    } else {
        TAB_STOP_COLUMNS - remainder
    }
}

fn strip_ansi_escape_sequences(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != ANSI_ESCAPE {
            result.push(ch);
            continue;
        }

        match chars.next() {
            Some('[') => skip_csi_sequence(&mut chars),
            Some(']') => skip_osc_sequence(&mut chars),
            Some('P' | 'X' | '^' | '_') => skip_st_sequence(&mut chars),
            Some(_) | None => {}
        }
    }

    result
}

fn skip_csi_sequence(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    for ch in chars.by_ref() {
        if ('\u{40}'..='\u{7e}').contains(&ch) {
            return;
        }
    }
}

fn skip_osc_sequence(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(ch) = chars.next() {
        if ch == '\u{7}' {
            return;
        }
        if ch == ANSI_ESCAPE && chars.next_if_eq(&'\\').is_some() {
            return;
        }
    }
}

fn skip_st_sequence(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(ch) = chars.next() {
        if ch == ANSI_ESCAPE && chars.next_if_eq(&'\\').is_some() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 行数が上限を超えると古い行が破棄される() {
        let mut scrollback = Scrollback::new(3);

        for index in 0..5 {
            scrollback.push_line(format!("line-{}", index));
        }

        let collected: Vec<&str> = scrollback.lines().map(|line| line.raw()).collect();

        assert_eq!(collected, vec!["line-2", "line-3", "line-4"]);
    }

    #[test]
    fn 全角文字の幅が正しく計算される() {
        let mut scrollback = Scrollback::default();

        scrollback.push_line("abc");
        scrollback.push_line("あいう");

        let second = scrollback.lines().nth(1).expect("2行目が存在するはず");
        assert_eq!(second.width(), 6);
        let widths: Vec<usize> = second.cells().iter().map(|cell| cell.width()).collect();
        assert_eq!(widths, vec![2, 2, 2]);
    }

    #[test]
    fn 空行も保持できる() {
        let mut scrollback = Scrollback::default();

        scrollback.push_line("");

        assert_eq!(scrollback.last_line().unwrap().width(), 0);
        assert_eq!(scrollback.last_line().unwrap().raw(), "");
    }

    #[test]
    fn ansiエスケープシーケンスは除去される() {
        let mut scrollback = Scrollback::default();

        scrollback.push_line("\u{1b}[31merror\u{1b}[0m");

        let line = scrollback.last_line().expect("行が存在するはず");
        let chars: Vec<char> = line.cells().iter().map(|cell| cell.ch()).collect();
        assert_eq!(line.raw(), "error");
        assert_eq!(chars, vec!['e', 'r', 'r', 'o', 'r']);
        assert_eq!(line.width(), 5);
    }

    #[test]
    fn タブは次のタブストップまで空白展開される() {
        let mut scrollback = Scrollback::default();

        scrollback.push_line("a\tb");

        let line = scrollback.last_line().expect("行が存在するはず");
        let chars: Vec<char> = line.cells().iter().map(|cell| cell.ch()).collect();
        let widths: Vec<usize> = line.cells().iter().map(|cell| cell.width()).collect();
        assert_eq!(line.width(), 9);
        assert_eq!(chars, vec!['a', ' ', ' ', ' ', ' ', ' ', ' ', ' ', 'b']);
        assert_eq!(widths, vec![1, 1, 1, 1, 1, 1, 1, 1, 1]);
    }

    #[test]
    fn zwj絵文字は1つの表示幅として集計される() {
        let mut scrollback = Scrollback::default();

        scrollback.push_line("👨‍👩‍👧‍👦!");

        let line = scrollback.last_line().expect("行が存在するはず");
        assert_eq!(line.width(), 3);
        let width_sum: usize = line.cells().iter().map(|cell| cell.width()).sum();
        assert_eq!(width_sum, 3);
        assert_eq!(line.cells()[0].width(), 2);
        assert!(
            line.cells()
                .iter()
                .skip(1)
                .take(6)
                .all(|cell| cell.width() == 0)
        );
    }
}
