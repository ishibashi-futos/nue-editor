use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KeyModifier {
    CmdOrCtrl,
    Shift,
    Alt,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyChord {
    pub key: String,
    pub modifiers: Vec<KeyModifier>,
}

impl KeyChord {
    pub fn new(key: impl Into<String>, modifiers: Vec<KeyModifier>) -> Self {
        let mut unique_modifiers = modifiers;
        unique_modifiers.sort_unstable();
        unique_modifiers.dedup();

        Self {
            key: key.into().trim().to_ascii_lowercase(),
            modifiers: unique_modifiers,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorCommand {
    Save,
    Undo,
    Redo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveTrigger {
    Manual,
    Shortcut(KeyChord),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorSaveRequest {
    pub file_path: String,
    pub content: String,
    pub revision: u64,
    pub trigger: SaveTrigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorBufferSnapshot {
    pub file_path: String,
    pub content: String,
    pub cursor_char: usize,
    pub revision: u64,
    pub is_dirty: bool,
    pub can_undo: bool,
    pub can_redo: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorMoveOutcome {
    NoBuffer,
    Moved { cursor_char: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditOutcome {
    NoBuffer,
    Edited {
        revision: u64,
        cursor_char: usize,
        is_dirty: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryOutcome {
    NoBuffer,
    NoHistory,
    Applied {
        revision: u64,
        cursor_char: usize,
        is_dirty: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveOutcome {
    NoBuffer,
    NotDirty,
    Requested(EditorSaveRequest),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkSavedOutcome {
    NoBuffer,
    StaleRevision { current_revision: u64 },
    Saved { revision: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterShortcutOutcome {
    Registered,
    Updated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandExecutionOutcome {
    Save(SaveOutcome),
    Undo(HistoryOutcome),
    Redo(HistoryOutcome),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutDispatchOutcome {
    Unhandled,
    Executed {
        command: EditorCommand,
        outcome: CommandExecutionOutcome,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferOpenedEvent {
    pub file_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferEditedEvent {
    pub file_path: String,
    pub revision: u64,
    pub cursor_char: usize,
    pub is_dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorMovedEvent {
    pub file_path: String,
    pub cursor_char: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutRegisteredEvent {
    pub chord: KeyChord,
    pub command: EditorCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutDispatchedEvent {
    pub chord: KeyChord,
    pub command: EditorCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedEvent {
    pub file_path: String,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorCoreEvent {
    BufferOpened(BufferOpenedEvent),
    BufferEdited(BufferEditedEvent),
    CursorMoved(CursorMovedEvent),
    SaveRequested(EditorSaveRequest),
    Saved(SavedEvent),
    ShortcutRegistered(ShortcutRegisteredEvent),
    ShortcutDispatched(ShortcutDispatchedEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorCore {
    active_buffer: Option<EditorBuffer>,
    shortcuts: HashMap<KeyChord, EditorCommand>,
    events: VecDeque<EditorCoreEvent>,
}

impl EditorCore {
    pub fn new() -> Self {
        Self {
            active_buffer: None,
            shortcuts: HashMap::new(),
            events: VecDeque::new(),
        }
    }

    pub fn open_file(
        &mut self,
        file_path: impl Into<String>,
        content: impl Into<String>,
    ) -> EditorBufferSnapshot {
        let file_path = file_path.into();
        let content = content.into();
        let buffer = EditorBuffer::new(file_path.clone(), content);
        let snapshot = buffer.snapshot();

        self.active_buffer = Some(buffer);
        self.events
            .push_back(EditorCoreEvent::BufferOpened(BufferOpenedEvent {
                file_path,
            }));

        snapshot
    }

    pub fn snapshot(&self) -> Option<EditorBufferSnapshot> {
        self.active_buffer.as_ref().map(EditorBuffer::snapshot)
    }

    pub fn set_cursor(&mut self, cursor_char: usize) -> CursorMoveOutcome {
        let Some(buffer) = self.active_buffer.as_mut() else {
            return CursorMoveOutcome::NoBuffer;
        };

        let clamped_cursor = cursor_char.min(buffer.content.chars().count());
        let file_path = buffer.file_path.clone();
        let changed = buffer.cursor_char != clamped_cursor;
        buffer.cursor_char = clamped_cursor;

        if changed {
            self.events
                .push_back(EditorCoreEvent::CursorMoved(CursorMovedEvent {
                    file_path,
                    cursor_char: clamped_cursor,
                }));
        }

        CursorMoveOutcome::Moved {
            cursor_char: clamped_cursor,
        }
    }

    pub fn insert_text(&mut self, text: &str) -> EditOutcome {
        let Some(buffer) = self.active_buffer.as_mut() else {
            return EditOutcome::NoBuffer;
        };
        if text.is_empty() {
            return EditOutcome::Edited {
                revision: buffer.revision,
                cursor_char: buffer.cursor_char,
                is_dirty: buffer.is_dirty(),
            };
        }

        buffer.undo_stack.push(buffer.current_history_state());
        buffer.redo_stack.clear();

        let insertion_byte_index = char_to_byte_index(&buffer.content, buffer.cursor_char);
        buffer.content.insert_str(insertion_byte_index, text);
        buffer.cursor_char += text.chars().count();
        buffer.revision += 1;

        let is_dirty = buffer.is_dirty();
        let revision = buffer.revision;
        let cursor_char = buffer.cursor_char;
        let file_path = buffer.file_path.clone();
        self.push_buffer_edited_event(file_path, revision, cursor_char, is_dirty);

        EditOutcome::Edited {
            revision,
            cursor_char,
            is_dirty,
        }
    }

    pub fn undo(&mut self) -> HistoryOutcome {
        let Some(buffer) = self.active_buffer.as_mut() else {
            return HistoryOutcome::NoBuffer;
        };
        let Some(previous_state) = buffer.undo_stack.pop() else {
            return HistoryOutcome::NoHistory;
        };

        buffer.redo_stack.push(buffer.current_history_state());
        buffer.apply_history_state(previous_state);
        buffer.revision += 1;

        let is_dirty = buffer.is_dirty();
        let revision = buffer.revision;
        let cursor_char = buffer.cursor_char;
        let file_path = buffer.file_path.clone();
        self.push_buffer_edited_event(file_path, revision, cursor_char, is_dirty);

        HistoryOutcome::Applied {
            revision,
            cursor_char,
            is_dirty,
        }
    }

    pub fn redo(&mut self) -> HistoryOutcome {
        let Some(buffer) = self.active_buffer.as_mut() else {
            return HistoryOutcome::NoBuffer;
        };
        let Some(next_state) = buffer.redo_stack.pop() else {
            return HistoryOutcome::NoHistory;
        };

        buffer.undo_stack.push(buffer.current_history_state());
        buffer.apply_history_state(next_state);
        buffer.revision += 1;

        let is_dirty = buffer.is_dirty();
        let revision = buffer.revision;
        let cursor_char = buffer.cursor_char;
        let file_path = buffer.file_path.clone();
        self.push_buffer_edited_event(file_path, revision, cursor_char, is_dirty);

        HistoryOutcome::Applied {
            revision,
            cursor_char,
            is_dirty,
        }
    }

    pub fn request_save(&mut self, trigger: SaveTrigger) -> SaveOutcome {
        let Some(buffer) = self.active_buffer.as_ref() else {
            return SaveOutcome::NoBuffer;
        };
        if !buffer.is_dirty() {
            return SaveOutcome::NotDirty;
        }

        let request = EditorSaveRequest {
            file_path: buffer.file_path.clone(),
            content: buffer.content.clone(),
            revision: buffer.revision,
            trigger,
        };
        self.events
            .push_back(EditorCoreEvent::SaveRequested(request.clone()));

        SaveOutcome::Requested(request)
    }

    pub fn mark_saved(&mut self, revision: u64) -> MarkSavedOutcome {
        let Some(buffer) = self.active_buffer.as_mut() else {
            return MarkSavedOutcome::NoBuffer;
        };
        if revision != buffer.revision {
            return MarkSavedOutcome::StaleRevision {
                current_revision: buffer.revision,
            };
        }

        buffer.saved_content = buffer.content.clone();
        self.events.push_back(EditorCoreEvent::Saved(SavedEvent {
            file_path: buffer.file_path.clone(),
            revision,
        }));

        MarkSavedOutcome::Saved { revision }
    }

    pub fn register_shortcut(
        &mut self,
        chord: KeyChord,
        command: EditorCommand,
    ) -> RegisterShortcutOutcome {
        let previous = self.shortcuts.insert(chord.clone(), command);
        self.events.push_back(EditorCoreEvent::ShortcutRegistered(
            ShortcutRegisteredEvent { chord, command },
        ));

        if previous.is_some() {
            RegisterShortcutOutcome::Updated
        } else {
            RegisterShortcutOutcome::Registered
        }
    }

    pub fn dispatch_shortcut(&mut self, chord: &KeyChord) -> ShortcutDispatchOutcome {
        let Some(command) = self.shortcuts.get(chord).copied() else {
            return ShortcutDispatchOutcome::Unhandled;
        };

        let outcome = match command {
            EditorCommand::Save => CommandExecutionOutcome::Save(
                self.request_save(SaveTrigger::Shortcut(chord.clone())),
            ),
            EditorCommand::Undo => CommandExecutionOutcome::Undo(self.undo()),
            EditorCommand::Redo => CommandExecutionOutcome::Redo(self.redo()),
        };

        self.events.push_back(EditorCoreEvent::ShortcutDispatched(
            ShortcutDispatchedEvent {
                chord: chord.clone(),
                command,
            },
        ));

        ShortcutDispatchOutcome::Executed { command, outcome }
    }

    pub fn drain_events(&mut self) -> Vec<EditorCoreEvent> {
        self.events.drain(..).collect()
    }

    fn push_buffer_edited_event(
        &mut self,
        file_path: String,
        revision: u64,
        cursor_char: usize,
        is_dirty: bool,
    ) {
        self.events
            .push_back(EditorCoreEvent::BufferEdited(BufferEditedEvent {
                file_path,
                revision,
                cursor_char,
                is_dirty,
            }));
    }
}

impl Default for EditorCore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EditorBuffer {
    file_path: String,
    content: String,
    cursor_char: usize,
    revision: u64,
    saved_content: String,
    undo_stack: Vec<HistoryState>,
    redo_stack: Vec<HistoryState>,
}

impl EditorBuffer {
    fn new(file_path: String, content: String) -> Self {
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

    fn current_history_state(&self) -> HistoryState {
        HistoryState {
            content: self.content.clone(),
            cursor_char: self.cursor_char,
        }
    }

    fn apply_history_state(&mut self, state: HistoryState) {
        self.content = state.content;
        self.cursor_char = state.cursor_char;
    }

    fn is_dirty(&self) -> bool {
        self.content != self.saved_content
    }

    fn snapshot(&self) -> EditorBufferSnapshot {
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
struct HistoryState {
    content: String,
    cursor_char: usize,
}

fn char_to_byte_index(content: &str, char_index: usize) -> usize {
    content
        .char_indices()
        .nth(char_index)
        .map(|(byte_index, _)| byte_index)
        .unwrap_or(content.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn save_chord() -> KeyChord {
        KeyChord::new("S", vec![KeyModifier::CmdOrCtrl])
    }

    #[test]
    fn ファイルを開くとバッファとカーソル初期値を保持する() {
        let mut core = EditorCore::new();

        let snapshot = core.open_file("docs/readme.md", "# heading");

        assert_eq!(
            snapshot,
            EditorBufferSnapshot {
                file_path: "docs/readme.md".to_string(),
                content: "# heading".to_string(),
                cursor_char: 0,
                revision: 0,
                is_dirty: false,
                can_undo: false,
                can_redo: false,
            }
        );
        assert_eq!(
            core.drain_events(),
            vec![EditorCoreEvent::BufferOpened(BufferOpenedEvent {
                file_path: "docs/readme.md".to_string(),
            })]
        );
    }

    #[test]
    fn 編集後にundo_redoとカーソル保持ができる() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "Hello");

        assert_eq!(
            core.set_cursor(5),
            CursorMoveOutcome::Moved { cursor_char: 5 }
        );
        assert_eq!(
            core.insert_text(" world"),
            EditOutcome::Edited {
                revision: 1,
                cursor_char: 11,
                is_dirty: true,
            }
        );
        assert_eq!(
            core.snapshot().expect("バッファが存在する").content,
            "Hello world".to_string()
        );

        assert_eq!(
            core.undo(),
            HistoryOutcome::Applied {
                revision: 2,
                cursor_char: 5,
                is_dirty: false,
            }
        );
        assert_eq!(
            core.snapshot().expect("バッファが存在する").content,
            "Hello".to_string()
        );

        assert_eq!(
            core.redo(),
            HistoryOutcome::Applied {
                revision: 3,
                cursor_char: 11,
                is_dirty: true,
            }
        );
        assert_eq!(
            core.snapshot().expect("バッファが存在する").content,
            "Hello world".to_string()
        );
    }

    #[test]
    fn 保存フローはrequestとmark_savedでdirtyを更新する() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "Hello");
        core.set_cursor(5);
        core.insert_text("!");

        let save = core.request_save(SaveTrigger::Manual);
        let SaveOutcome::Requested(request) = save else {
            panic!("保存要求が返る想定");
        };
        assert_eq!(request.file_path, "docs/readme.md");
        assert_eq!(request.content, "Hello!");
        assert_eq!(request.revision, 1);

        assert_eq!(
            core.mark_saved(request.revision),
            MarkSavedOutcome::Saved { revision: 1 }
        );
        assert!(!core.snapshot().expect("バッファが存在する").is_dirty);
    }

    #[test]
    fn ショートカット登録とcmd_ctrl_s実行で保存を発火できる() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "Hello");
        core.set_cursor(5);
        core.insert_text("!");

        assert_eq!(
            core.register_shortcut(save_chord(), EditorCommand::Save),
            RegisterShortcutOutcome::Registered
        );

        let outcome = core.dispatch_shortcut(&save_chord());
        let ShortcutDispatchOutcome::Executed {
            command,
            outcome: CommandExecutionOutcome::Save(SaveOutcome::Requested(request)),
        } = outcome
        else {
            panic!("ショートカット保存が実行される想定");
        };

        assert_eq!(command, EditorCommand::Save);
        assert_eq!(request.content, "Hello!");
    }
}
