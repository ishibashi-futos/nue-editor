use std::collections::VecDeque;

use unicode_width::UnicodeWidthChar;

/// スクロールバックの既定行数。
pub const SCROLLBACK_DEFAULT_LINES: usize = 500;

/// 1 行分の表示セル。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineCell {
    ch: char,
    width: usize,
}

impl LineCell {
    fn new(ch: char) -> Self {
        let width = UnicodeWidthChar::width(ch).unwrap_or(0);
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
        let mut width = 0;
        let mut cells = Vec::new();

        for ch in raw.chars() {
            let cell = LineCell::new(ch);
            width += cell.width();
            cells.push(cell);
        }

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

    /// 既定の行数で初期化したスクロールバック。
    pub fn default() -> Self {
        Self::new(SCROLLBACK_DEFAULT_LINES)
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
}
