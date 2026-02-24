use nue_core::terminal::scrollback::{
    LineCell, SCROLLBACK_DEFAULT_LINES, Scrollback, ScrollbackLine,
};

/// 表示用に整形したセル。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalCell {
    ch: char,
    width: usize,
}

impl TerminalCell {
    /// {@link nue_core::terminal::scrollback::LineCell} から変換する。
    fn from_line_cell(cell: &LineCell) -> Self {
        Self {
            ch: cell.ch(),
            width: cell.width(),
        }
    }

    /// 表示用文字。
    pub fn ch(&self) -> char {
        self.ch
    }

    /// セルの幅（半角=1, 全角=2, 制御文字=0）。
    pub fn width(&self) -> usize {
        self.width
    }
}

/// 1 行分の表示結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedLine {
    raw: String,
    original_width: usize,
    display_width: usize,
    cells: Vec<TerminalCell>,
}

impl RenderedLine {
    /// 元の文字列を取得する。
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// 元の行の幅。
    pub fn original_width(&self) -> usize {
        self.original_width
    }

    /// 実際に描画する幅。
    pub fn display_width(&self) -> usize {
        self.display_width
    }

    /// 描画セル。
    pub fn cells(&self) -> &[TerminalCell] {
        &self.cells
    }
}

/// スクロールバックを表示するためのレイアウト生成器。
pub struct TerminalScrollbackRenderer {
    max_columns: usize,
    max_lines: usize,
}

impl TerminalScrollbackRenderer {
    /// 指定した列幅・行数で描画器を初期化する。
    pub fn new(max_columns: usize, max_lines: usize) -> Self {
        Self {
            max_columns,
            max_lines: max_lines.max(1),
        }
    }

    /// 列幅だけ指定したバリアント。行数はデフォルトのスクロールバックサイズを使う。
    pub fn with_columns(max_columns: usize) -> Self {
        Self::new(max_columns, SCROLLBACK_DEFAULT_LINES)
    }

    /// スクロールバック全体（`Scrollback`）から描画用の行一覧を作る。
    pub fn render_from_scrollback(&self, scrollback: &Scrollback) -> Vec<RenderedLine> {
        self.render(scrollback.lines())
    }

    /// 任意の行列から描画コンテンツを生成する。
    pub fn render<'a>(
        &self,
        lines: impl IntoIterator<Item = &'a ScrollbackLine>,
    ) -> Vec<RenderedLine> {
        let iter: Vec<&ScrollbackLine> = lines.into_iter().collect();
        let start = iter.len().saturating_sub(self.max_lines);
        iter[start..]
            .iter()
            .map(|line| self.render_line(line))
            .collect()
    }

    fn render_line(&self, line: &ScrollbackLine) -> RenderedLine {
        let mut used_columns = 0;
        let mut cells = Vec::new();

        for cell in line.cells() {
            let width = cell.width();
            if width > 0 && used_columns + width > self.max_columns {
                break;
            }
            used_columns += width;
            cells.push(TerminalCell::from_line_cell(cell));
        }

        RenderedLine {
            raw: line.raw().to_string(),
            original_width: line.width(),
            display_width: used_columns,
            cells,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nue_core::terminal::scrollback::Scrollback;

    fn sample_scrollback() -> Scrollback {
        let mut scrollback = Scrollback::new(4);
        for index in 0..5 {
            scrollback.push_line(format!("line-{}", index));
        }
        scrollback
    }

    #[test]
    fn window_limits_number_of_lines() {
        let scrollback = sample_scrollback();
        let renderer = TerminalScrollbackRenderer::new(80, 3);

        let rendered = renderer.render_from_scrollback(&scrollback);

        assert_eq!(rendered.len(), 3);
        assert_eq!(rendered[0].raw(), "line-2");
        assert_eq!(rendered[2].raw(), "line-4");
    }

    #[test]
    fn wide_characters_are_truncated_by_max_columns() {
        let mut scrollback = Scrollback::new(4);
        scrollback.push_line("あいう");
        let renderer = TerminalScrollbackRenderer::new(2, 4);

        let rendered = renderer.render_from_scrollback(&scrollback);
        assert_eq!(rendered.len(), 1);
        let line = &rendered[0];
        assert_eq!(line.display_width(), 2);
        assert_eq!(line.cells().len(), 1);
        assert_eq!(line.cells()[0].ch(), 'あ');
    }

    #[test]
    fn zero_width_cells_are_preserved_even_when_width_zero() {
        let mut scrollback = Scrollback::new(4);
        scrollback.push_line("a\u{0301}");
        let renderer = TerminalScrollbackRenderer::new(1, 4);

        let rendered = renderer.render_from_scrollback(&scrollback);
        assert_eq!(rendered.len(), 1);
        let line = &rendered[0];
        let cells: Vec<char> = line.cells().iter().map(|cell| cell.ch()).collect();
        assert_eq!(cells, vec!['a', '\u{0301}']);
        assert_eq!(line.display_width(), 1);
    }

    #[test]
    fn zwj絵文字は表示幅2として切り詰め判定される() {
        let mut scrollback = Scrollback::new(4);
        scrollback.push_line("👨‍👩‍👧‍👦!");
        let renderer = TerminalScrollbackRenderer::new(2, 4);

        let rendered = renderer.render_from_scrollback(&scrollback);
        assert_eq!(rendered.len(), 1);
        let line = &rendered[0];
        let cells: Vec<char> = line.cells().iter().map(|cell| cell.ch()).collect();
        assert_eq!(line.display_width(), 2);
        assert_eq!(cells, vec!['👨', '‍', '👩', '‍', '👧', '‍', '👦']);
    }
}
