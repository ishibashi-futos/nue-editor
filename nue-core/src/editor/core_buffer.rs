use super::EditorBufferSnapshot;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EditorBuffer {
    pub(super) file_path: PathBuf,
    pub(super) content: String,
    pub(super) cursor_char: usize,
    pub(super) revision: u64,
    pub(super) saved_content: String,
    pub(super) undo_stack: Vec<HistoryState>,
    pub(super) redo_stack: Vec<HistoryState>,
}

impl EditorBuffer {
    pub(super) fn new(file_path: PathBuf, content: String) -> Self {
        Self {
            file_path,
            content: content.clone(),
            cursor_char: 0,
            revision: 0,
            saved_content: content,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub(super) fn current_history_state(&self) -> HistoryState {
        HistoryState {
            content: self.content.clone(),
            cursor_char: self.cursor_char,
        }
    }

    pub(super) fn apply_history_state(&mut self, state: HistoryState) {
        self.content = state.content;
        self.cursor_char = state.cursor_char;
    }

    pub(super) fn is_dirty(&self) -> bool {
        self.content != self.saved_content
    }

    pub(super) fn snapshot(&self) -> EditorBufferSnapshot {
        EditorBufferSnapshot {
            file_path: self.file_path.clone(),
            content: self.content.clone(),
            cursor_char: self.cursor_char,
            revision: self.revision,
            is_dirty: self.is_dirty(),
            can_undo: !self.undo_stack.is_empty(),
            can_redo: !self.redo_stack.is_empty(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HistoryState {
    content: String,
    cursor_char: usize,
}
