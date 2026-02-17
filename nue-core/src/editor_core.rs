use crate::markdown_service::{
    MarkdownDiffObservedEvent, MarkdownFeature, MarkdownFeatureRequestedEvent, MarkdownHeading,
    MarkdownPreviewSyncedEvent, MarkdownService, MarkdownServiceEvent,
};
use crate::minimap_service::{
    MinimapFocusIdSyncedEvent, MinimapOverlay, MinimapOverlaysUpdatedEvent, MinimapService,
    MinimapServiceEvent, MinimapSnapshot,
};
use crate::path_display::PathDisplayExt;
use crate::search_navigator::SearchNavigator;
use crate::search_service::{SearchError, SearchMatch, SearchQuery, SearchService};
use crate::smart_gutter_service::{
    OpenApprovalRequestError, SmartGutterApprovalRequestOpenedEvent, SmartGutterFocusIdSyncedEvent,
    SmartGutterIndicator, SmartGutterIndicatorsUpdatedEvent, SmartGutterJumpRequestedEvent,
    SmartGutterService, SmartGutterServiceEvent, SmartGutterSnapshot,
};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};

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
    QuickOpen,
    FindInFile,
    FindInWorkspace,
    Copy,
    OpenMarkdownMenu,
    OpenMarkdownPreview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveTrigger {
    Manual,
    Shortcut(KeyChord),
    ContextMenu,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorSaveRequest {
    pub file_path: PathBuf,
    pub content: String,
    pub revision: u64,
    pub trigger: SaveTrigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorBufferSnapshot {
    pub file_path: PathBuf,
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
    TargetNotFound,
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
    QuickOpen,
    FindInFile,
    FindInWorkspace,
    Copy(CopyOutcome),
    OpenMarkdownMenu,
    OpenMarkdownPreview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutDispatchOutcome {
    Unhandled,
    Executed {
        command: EditorCommand,
        outcome: CommandExecutionOutcome,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EditorContextMenu {
    pub is_open: bool,
    pub target_file_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorContextMenuItem {
    Save,
    Copy,
    MarkdownMenu,
    MarkdownPreview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenEditorContextMenuOutcome {
    Opened { file_path: PathBuf },
    NoBuffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyOutcome {
    NoBuffer,
    Copied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecuteEditorContextMenuOutcome {
    Executed {
        item: EditorContextMenuItem,
        outcome: CommandExecutionOutcome,
    },
    ContextMenuClosed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteMarkdownFeatureOutcome {
    NoBuffer,
    NotMarkdownFile,
    Executed { feature: MarkdownFeature },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateMinimapOverlaysOutcome {
    NoBuffer,
    Updated { overlay_count: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncMinimapFocusOutcome {
    NoBuffer,
    FocusNotFound,
    Synced { focus_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateSmartGutterIndicatorsOutcome {
    NoBuffer,
    Updated { indicator_count: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncSmartGutterFocusOutcome {
    NoBuffer,
    FocusNotFound,
    Synced { focus_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JumpToSmartGutterDiffOutcome {
    NoBuffer,
    FocusNotFound,
    Jumped { focus_id: String, line: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenSmartGutterApprovalRequestOutcome {
    NoBuffer,
    FocusNotFound,
    MissingApprovalRequest,
    Opened {
        focus_id: String,
        approval_request_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferOpenedEvent {
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferEditedEvent {
    pub file_path: PathBuf,
    pub revision: u64,
    pub cursor_char: usize,
    pub is_dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorMovedEvent {
    pub file_path: PathBuf,
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
pub struct ContextMenuOpenedEvent {
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMenuItemExecutedEvent {
    pub file_path: PathBuf,
    pub item: EditorContextMenuItem,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyRequestedEvent {
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownMenuRequestedEvent {
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownPreviewRequestedEvent {
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResultsUpdatedEvent {
    pub query: SearchQuery,
    pub matches: Vec<SearchMatch>,
    pub focus_index: Option<usize>,
    pub focus_match: Option<SearchMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchFocusChangedEvent {
    pub focus_index: Option<usize>,
    pub focus_match: Option<SearchMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchWorkspaceOutcome {
    Matches { match_count: usize },
    NoMatches,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedEvent {
    pub file_path: PathBuf,
    pub revision: u64,
}

macro_rules! impl_event_path_display {
    ($event:ident) => {
        impl $event {
            pub fn file_path_display(&self) -> std::borrow::Cow<'_, str> {
                self.file_path.display_for_ui()
            }
        }
    };
}

impl_event_path_display!(BufferOpenedEvent);
impl_event_path_display!(BufferEditedEvent);
impl_event_path_display!(CursorMovedEvent);
impl_event_path_display!(ContextMenuOpenedEvent);
impl_event_path_display!(ContextMenuItemExecutedEvent);
impl_event_path_display!(CopyRequestedEvent);
impl_event_path_display!(MarkdownMenuRequestedEvent);
impl_event_path_display!(MarkdownPreviewRequestedEvent);
impl_event_path_display!(SavedEvent);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorCoreEvent {
    BufferOpened(BufferOpenedEvent),
    BufferEdited(BufferEditedEvent),
    CursorMoved(CursorMovedEvent),
    SaveRequested(EditorSaveRequest),
    Saved(SavedEvent),
    ShortcutRegistered(ShortcutRegisteredEvent),
    ShortcutDispatched(ShortcutDispatchedEvent),
    ContextMenuOpened(ContextMenuOpenedEvent),
    ContextMenuItemExecuted(ContextMenuItemExecutedEvent),
    CopyRequested(CopyRequestedEvent),
    MarkdownMenuRequested(MarkdownMenuRequestedEvent),
    MarkdownPreviewRequested(MarkdownPreviewRequestedEvent),
    MarkdownFeatureRequested(MarkdownFeatureRequestedEvent),
    MarkdownDiffObserved(MarkdownDiffObservedEvent),
    MarkdownPreviewSynced(MarkdownPreviewSyncedEvent),
    MinimapOverlaysUpdated(MinimapOverlaysUpdatedEvent),
    MinimapFocusIdSynced(MinimapFocusIdSyncedEvent),
    SearchResultsUpdated(SearchResultsUpdatedEvent),
    SearchFocusChanged(SearchFocusChangedEvent),
    SmartGutterIndicatorsUpdated(SmartGutterIndicatorsUpdatedEvent),
    SmartGutterFocusIdSynced(SmartGutterFocusIdSyncedEvent),
    SmartGutterJumpRequested(SmartGutterJumpRequestedEvent),
    SmartGutterApprovalRequestOpened(SmartGutterApprovalRequestOpenedEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownPreviewSnapshot {
    pub file_path: PathBuf,
    pub revision: u64,
    pub headings: Vec<MarkdownHeading>,
}

#[derive(Debug)]
pub struct EditorCore {
    active_buffer: Option<EditorBuffer>,
    shortcuts: HashMap<KeyChord, EditorCommand>,
    context_menu: EditorContextMenu,
    markdown_service: MarkdownService,
    minimap_service: MinimapService,
    smart_gutter_service: SmartGutterService,
    search_service: SearchService,
    search_navigator: SearchNavigator,
    events: VecDeque<EditorCoreEvent>,
    markdown_preview_snapshot: Option<MarkdownPreviewSnapshot>,
}

impl EditorCore {
    pub fn new() -> Self {
        Self {
            active_buffer: None,
            shortcuts: HashMap::new(),
            context_menu: EditorContextMenu::default(),
            markdown_service: MarkdownService::new(),
            minimap_service: MinimapService::new(),
            smart_gutter_service: SmartGutterService::new(),
            search_service: SearchService::new("."),
            search_navigator: SearchNavigator::new(),
            events: VecDeque::new(),
            markdown_preview_snapshot: None,
        }
    }

    pub fn open_file(
        &mut self,
        file_path: impl Into<PathBuf>,
        content: impl Into<String>,
    ) -> EditorBufferSnapshot {
        let file_path = file_path.into();
        let content = content.into();
        let buffer = EditorBuffer::new(file_path.clone(), content);
        let snapshot = buffer.snapshot();

        self.active_buffer = Some(buffer);
        self.markdown_preview_snapshot = None;
        self.minimap_service
            .on_buffer_opened(file_path.as_path(), snapshot.content.as_str());
        self.smart_gutter_service
            .on_buffer_opened(file_path.as_path(), snapshot.content.as_str());
        self.close_context_menu();
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

        let previous_content = buffer.content.clone();
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
        let current_content = buffer.content.clone();
        self.sync_services_after_buffer_update(current_content.as_str());
        self.push_buffer_edited_event(file_path.clone(), revision, cursor_char, is_dirty);
        self.push_markdown_observation_events(
            &file_path,
            revision,
            previous_content.as_str(),
            current_content.as_str(),
        );

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

        let previous_content = buffer.content.clone();
        buffer.redo_stack.push(buffer.current_history_state());
        buffer.apply_history_state(previous_state);
        buffer.revision += 1;

        let is_dirty = buffer.is_dirty();
        let revision = buffer.revision;
        let cursor_char = buffer.cursor_char;
        let file_path = buffer.file_path.clone();
        let current_content = buffer.content.clone();
        self.sync_services_after_buffer_update(current_content.as_str());
        self.push_buffer_edited_event(file_path.clone(), revision, cursor_char, is_dirty);
        self.push_markdown_observation_events(
            &file_path,
            revision,
            previous_content.as_str(),
            current_content.as_str(),
        );

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

        let previous_content = buffer.content.clone();
        buffer.undo_stack.push(buffer.current_history_state());
        buffer.apply_history_state(next_state);
        buffer.revision += 1;

        let is_dirty = buffer.is_dirty();
        let revision = buffer.revision;
        let cursor_char = buffer.cursor_char;
        let file_path = buffer.file_path.clone();
        let current_content = buffer.content.clone();
        self.sync_services_after_buffer_update(current_content.as_str());
        self.push_buffer_edited_event(file_path.clone(), revision, cursor_char, is_dirty);
        self.push_markdown_observation_events(
            &file_path,
            revision,
            previous_content.as_str(),
            current_content.as_str(),
        );

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

    pub fn register_default_shortcuts(&mut self) -> Vec<(KeyChord, RegisterShortcutOutcome)> {
        let bindings = default_shortcut_bindings();
        let mut registered = Vec::with_capacity(bindings.len());
        for (chord, command) in bindings {
            let outcome = self.register_shortcut(chord.clone(), command);
            registered.push((chord, outcome));
        }
        registered
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
            EditorCommand::QuickOpen => CommandExecutionOutcome::QuickOpen,
            EditorCommand::FindInFile => CommandExecutionOutcome::FindInFile,
            EditorCommand::FindInWorkspace => CommandExecutionOutcome::FindInWorkspace,
            EditorCommand::Copy => CommandExecutionOutcome::Copy(self.request_copy()),
            EditorCommand::OpenMarkdownMenu => {
                self.request_markdown_menu();
                CommandExecutionOutcome::OpenMarkdownMenu
            }
            EditorCommand::OpenMarkdownPreview => {
                self.request_markdown_preview();
                CommandExecutionOutcome::OpenMarkdownPreview
            }
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

    /// 検索対象ルートを書き換え、ナビゲータをリセットする。
    pub fn set_search_root(&mut self, root: impl Into<PathBuf>) {
        self.search_service = SearchService::new(root);
        self.search_navigator.update(None, Vec::new());
    }

    /// グローバル検索（Workspace）を実行し、結果を返す。
    pub fn search_workspace(
        &mut self,
        query: SearchQuery,
    ) -> Result<SearchWorkspaceOutcome, SearchError> {
        let matches = self.search_service.search(&query)?;
        let match_count = matches.len();
        let matches_for_event = matches.clone();
        self.search_navigator.update(Some(query.clone()), matches);
        self.push_search_results_event(query, matches_for_event);

        Ok(if match_count == 0 {
            SearchWorkspaceOutcome::NoMatches
        } else {
            SearchWorkspaceOutcome::Matches { match_count }
        })
    }

    /// 現在の検索フォーカスを参照する。
    pub fn current_search_match(&self) -> Option<&SearchMatch> {
        self.search_navigator.current()
    }

    /// 次の検索結果にフォーカスを移す。
    pub fn advance_search_result(&mut self) -> Option<SearchMatch> {
        let next = self.search_navigator.advance().cloned();
        self.push_search_focus_event();
        next
    }

    /// 前の検索結果にフォーカスを戻す。
    pub fn retreat_search_result(&mut self) -> Option<SearchMatch> {
        let prev = self.search_navigator.retreat().cloned();
        self.push_search_focus_event();
        prev
    }

    /// 指定インデックスの検索結果を選択する。
    pub fn select_search_result(&mut self, index: usize) -> Option<SearchMatch> {
        let selected = self.search_navigator.select_index(index).cloned();
        if selected.is_some() {
            self.push_search_focus_event();
        }
        selected
    }

    fn push_search_results_event(&mut self, query: SearchQuery, matches: Vec<SearchMatch>) {
        let focus_index = self.search_navigator.current_index();
        let focus_match = self.search_navigator.current().cloned();
        self.events.push_back(EditorCoreEvent::SearchResultsUpdated(
            SearchResultsUpdatedEvent {
                query,
                matches,
                focus_index,
                focus_match,
            },
        ));
    }

    fn push_search_focus_event(&mut self) {
        let focus_index = self.search_navigator.current_index();
        let focus_match = self.search_navigator.current().cloned();
        self.events.push_back(EditorCoreEvent::SearchFocusChanged(
            SearchFocusChangedEvent {
                focus_index,
                focus_match,
            },
        ));
    }

    pub fn execute_markdown_feature(
        &mut self,
        feature: MarkdownFeature,
    ) -> ExecuteMarkdownFeatureOutcome {
        let Some(buffer) = self.active_buffer.as_ref() else {
            return ExecuteMarkdownFeatureOutcome::NoBuffer;
        };

        let Some(event) = self
            .markdown_service
            .request_feature(buffer.file_path.as_path(), feature)
        else {
            return ExecuteMarkdownFeatureOutcome::NotMarkdownFile;
        };

        self.push_markdown_service_event(event);
        ExecuteMarkdownFeatureOutcome::Executed { feature }
    }

    pub fn sync_with_markdown_preview(&mut self, line: usize) -> CursorMoveOutcome {
        if self.active_buffer.is_none() {
            return CursorMoveOutcome::NoBuffer;
        }
        let char_index = {
            let buffer = self.active_buffer.as_ref().unwrap();
            line_to_char_index(buffer.content.as_str(), line)
        };
        self.set_cursor(char_index)
    }

    pub fn jump_to_markdown_heading(&mut self, heading: &MarkdownHeading) -> CursorMoveOutcome {
        self.sync_with_markdown_preview(heading.line)
    }

    pub fn markdown_preview_snapshot(&self) -> Option<&MarkdownPreviewSnapshot> {
        self.markdown_preview_snapshot.as_ref()
    }

    pub fn jump_to_markdown_heading_by_index(&mut self, index: usize) -> CursorMoveOutcome {
        if self.active_buffer.is_none() {
            return CursorMoveOutcome::NoBuffer;
        }
        let Some(snapshot) = self.markdown_preview_snapshot.as_ref() else {
            return CursorMoveOutcome::TargetNotFound;
        };
        let Some(heading) = snapshot.headings.get(index).cloned() else {
            return CursorMoveOutcome::TargetNotFound;
        };
        self.jump_to_markdown_heading(&heading)
    }

    pub fn minimap_snapshot(&self) -> Option<MinimapSnapshot> {
        self.minimap_service.snapshot()
    }

    pub fn update_minimap_overlays(
        &mut self,
        overlays: Vec<MinimapOverlay>,
    ) -> UpdateMinimapOverlaysOutcome {
        if self.active_buffer.is_none() {
            return UpdateMinimapOverlaysOutcome::NoBuffer;
        }
        let Some(event) = self.minimap_service.replace_overlays(overlays) else {
            return UpdateMinimapOverlaysOutcome::NoBuffer;
        };
        let overlay_count = self
            .minimap_service
            .snapshot()
            .map(|snapshot| snapshot.overlays.len())
            .unwrap_or(0);
        self.push_minimap_service_event(event);

        UpdateMinimapOverlaysOutcome::Updated { overlay_count }
    }

    pub fn sync_minimap_focus_id(&mut self, focus_id: &str) -> SyncMinimapFocusOutcome {
        if self.active_buffer.is_none() {
            return SyncMinimapFocusOutcome::NoBuffer;
        }
        if !self.minimap_service.has_focus(focus_id) {
            return SyncMinimapFocusOutcome::FocusNotFound;
        }
        if let Some(event) = self.minimap_service.sync_focus_id(focus_id) {
            self.push_minimap_service_event(event);
        }

        SyncMinimapFocusOutcome::Synced {
            focus_id: focus_id.to_string(),
        }
    }

    pub fn smart_gutter_snapshot(&self) -> Option<SmartGutterSnapshot> {
        self.smart_gutter_service.snapshot()
    }

    pub fn update_smart_gutter_indicators(
        &mut self,
        indicators: Vec<SmartGutterIndicator>,
    ) -> UpdateSmartGutterIndicatorsOutcome {
        if self.active_buffer.is_none() {
            return UpdateSmartGutterIndicatorsOutcome::NoBuffer;
        }
        let Some(event) = self.smart_gutter_service.replace_indicators(indicators) else {
            return UpdateSmartGutterIndicatorsOutcome::NoBuffer;
        };
        let indicator_count = self
            .smart_gutter_service
            .snapshot()
            .map(|snapshot| snapshot.indicators.len())
            .unwrap_or(0);
        self.push_smart_gutter_service_event(event);

        UpdateSmartGutterIndicatorsOutcome::Updated { indicator_count }
    }

    pub fn sync_smart_gutter_focus_id(&mut self, focus_id: &str) -> SyncSmartGutterFocusOutcome {
        if self.active_buffer.is_none() {
            return SyncSmartGutterFocusOutcome::NoBuffer;
        }
        if !self.smart_gutter_service.has_focus(focus_id) {
            return SyncSmartGutterFocusOutcome::FocusNotFound;
        }
        if let Some(event) = self.smart_gutter_service.sync_focus_id(focus_id) {
            self.push_smart_gutter_service_event(event);
        }

        SyncSmartGutterFocusOutcome::Synced {
            focus_id: focus_id.to_string(),
        }
    }

    pub fn jump_to_smart_gutter_focus(&mut self, focus_id: &str) -> JumpToSmartGutterDiffOutcome {
        if self.active_buffer.is_none() {
            return JumpToSmartGutterDiffOutcome::NoBuffer;
        }
        let Some(event) = self.smart_gutter_service.request_jump(focus_id) else {
            return JumpToSmartGutterDiffOutcome::FocusNotFound;
        };
        let line = match &event {
            SmartGutterServiceEvent::JumpRequested(event) => event.line,
            _ => return JumpToSmartGutterDiffOutcome::FocusNotFound,
        };
        self.push_smart_gutter_service_event(event);

        JumpToSmartGutterDiffOutcome::Jumped {
            focus_id: focus_id.to_string(),
            line,
        }
    }

    pub fn open_smart_gutter_approval_request(
        &mut self,
        focus_id: &str,
    ) -> OpenSmartGutterApprovalRequestOutcome {
        if self.active_buffer.is_none() {
            return OpenSmartGutterApprovalRequestOutcome::NoBuffer;
        }
        let event = match self.smart_gutter_service.open_approval_request(focus_id) {
            Ok(event) => event,
            Err(OpenApprovalRequestError::FocusNotFound) => {
                return OpenSmartGutterApprovalRequestOutcome::FocusNotFound;
            }
            Err(OpenApprovalRequestError::MissingApprovalRequest) => {
                return OpenSmartGutterApprovalRequestOutcome::MissingApprovalRequest;
            }
        };
        let approval_request_id = match &event {
            SmartGutterServiceEvent::ApprovalRequestOpened(event) => {
                event.approval_request_id.clone()
            }
            _ => return OpenSmartGutterApprovalRequestOutcome::FocusNotFound,
        };
        self.push_smart_gutter_service_event(event);

        OpenSmartGutterApprovalRequestOutcome::Opened {
            focus_id: focus_id.to_string(),
            approval_request_id,
        }
    }

    pub fn context_menu(&self) -> &EditorContextMenu {
        &self.context_menu
    }

    pub fn open_context_menu(&mut self) -> OpenEditorContextMenuOutcome {
        let Some(buffer) = self.active_buffer.as_ref() else {
            self.close_context_menu();
            return OpenEditorContextMenuOutcome::NoBuffer;
        };
        let file_path = buffer.file_path.clone();
        self.context_menu = EditorContextMenu {
            is_open: true,
            target_file_path: Some(file_path.clone()),
        };
        self.events
            .push_back(EditorCoreEvent::ContextMenuOpened(ContextMenuOpenedEvent {
                file_path: file_path.clone(),
            }));
        OpenEditorContextMenuOutcome::Opened { file_path }
    }

    pub fn execute_context_menu_item(
        &mut self,
        item: EditorContextMenuItem,
    ) -> ExecuteEditorContextMenuOutcome {
        if !self.context_menu.is_open {
            return ExecuteEditorContextMenuOutcome::ContextMenuClosed;
        }
        let Some(file_path) = self.context_menu.target_file_path.clone() else {
            self.close_context_menu();
            return ExecuteEditorContextMenuOutcome::ContextMenuClosed;
        };

        let outcome = match item {
            EditorContextMenuItem::Save => {
                CommandExecutionOutcome::Save(self.request_save(SaveTrigger::ContextMenu))
            }
            EditorContextMenuItem::Copy => CommandExecutionOutcome::Copy(self.request_copy()),
            EditorContextMenuItem::MarkdownMenu => {
                self.request_markdown_menu();
                CommandExecutionOutcome::OpenMarkdownMenu
            }
            EditorContextMenuItem::MarkdownPreview => {
                self.request_markdown_preview();
                CommandExecutionOutcome::OpenMarkdownPreview
            }
        };
        self.events
            .push_back(EditorCoreEvent::ContextMenuItemExecuted(
                ContextMenuItemExecutedEvent { file_path, item },
            ));
        self.close_context_menu();

        ExecuteEditorContextMenuOutcome::Executed { item, outcome }
    }

    fn push_buffer_edited_event(
        &mut self,
        file_path: PathBuf,
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

    fn push_markdown_observation_events(
        &mut self,
        file_path: &Path,
        revision: u64,
        previous_content: &str,
        current_content: &str,
    ) {
        let events = self.markdown_service.observe_change(
            file_path,
            revision,
            previous_content,
            current_content,
        );
        for event in events {
            self.push_markdown_service_event(event);
        }
    }

    fn push_markdown_service_event(&mut self, event: MarkdownServiceEvent) {
        match event {
            MarkdownServiceEvent::FeatureRequested(event) => self
                .events
                .push_back(EditorCoreEvent::MarkdownFeatureRequested(event)),
            MarkdownServiceEvent::DiffObserved(event) => self
                .events
                .push_back(EditorCoreEvent::MarkdownDiffObserved(event)),
            MarkdownServiceEvent::PreviewSynced(event) => {
                self.update_markdown_preview_snapshot(&event);
                self.events
                    .push_back(EditorCoreEvent::MarkdownPreviewSynced(event));
            }
        }
    }

    fn update_markdown_preview_snapshot(&mut self, event: &MarkdownPreviewSyncedEvent) {
        self.markdown_preview_snapshot = Some(MarkdownPreviewSnapshot {
            file_path: event.file_path.clone(),
            revision: event.revision,
            headings: event.headings.clone(),
        });
    }

    fn sync_services_after_buffer_update(&mut self, current_content: &str) {
        if let Some(event) = self.minimap_service.on_buffer_updated(current_content) {
            self.push_minimap_service_event(event);
        }
        if let Some(event) = self.smart_gutter_service.on_buffer_updated(current_content) {
            self.push_smart_gutter_service_event(event);
        }
    }

    fn push_minimap_service_event(&mut self, event: MinimapServiceEvent) {
        match event {
            MinimapServiceEvent::OverlaysUpdated(event) => self
                .events
                .push_back(EditorCoreEvent::MinimapOverlaysUpdated(event)),
            MinimapServiceEvent::FocusIdSynced(event) => self
                .events
                .push_back(EditorCoreEvent::MinimapFocusIdSynced(event)),
        }
    }

    fn push_smart_gutter_service_event(&mut self, event: SmartGutterServiceEvent) {
        match event {
            SmartGutterServiceEvent::IndicatorsUpdated(event) => self
                .events
                .push_back(EditorCoreEvent::SmartGutterIndicatorsUpdated(event)),
            SmartGutterServiceEvent::FocusIdSynced(event) => self
                .events
                .push_back(EditorCoreEvent::SmartGutterFocusIdSynced(event)),
            SmartGutterServiceEvent::JumpRequested(event) => self
                .events
                .push_back(EditorCoreEvent::SmartGutterJumpRequested(event)),
            SmartGutterServiceEvent::ApprovalRequestOpened(event) => self
                .events
                .push_back(EditorCoreEvent::SmartGutterApprovalRequestOpened(event)),
        }
    }

    fn request_copy(&mut self) -> CopyOutcome {
        let Some(buffer) = self.active_buffer.as_ref() else {
            return CopyOutcome::NoBuffer;
        };
        self.events
            .push_back(EditorCoreEvent::CopyRequested(CopyRequestedEvent {
                file_path: buffer.file_path.clone(),
            }));
        CopyOutcome::Copied
    }

    fn request_markdown_menu(&mut self) {
        let Some(buffer) = self.active_buffer.as_ref() else {
            return;
        };
        self.events
            .push_back(EditorCoreEvent::MarkdownMenuRequested(
                MarkdownMenuRequestedEvent {
                    file_path: buffer.file_path.clone(),
                },
            ));
    }

    fn request_markdown_preview(&mut self) {
        let Some(buffer) = self.active_buffer.as_ref() else {
            return;
        };
        self.events
            .push_back(EditorCoreEvent::MarkdownPreviewRequested(
                MarkdownPreviewRequestedEvent {
                    file_path: buffer.file_path.clone(),
                },
            ));
    }

    fn close_context_menu(&mut self) {
        self.context_menu = EditorContextMenu::default();
    }
}

impl Default for EditorCore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EditorBuffer {
    file_path: PathBuf,
    content: String,
    cursor_char: usize,
    revision: u64,
    saved_content: String,
    undo_stack: Vec<HistoryState>,
    redo_stack: Vec<HistoryState>,
}

impl EditorBuffer {
    fn new(file_path: PathBuf, content: String) -> Self {
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

fn line_to_char_index(content: &str, line: usize) -> usize {
    if line == 0 {
        return 0;
    }
    let mut char_index = 0;
    let mut current_line = 0;

    for ch in content.chars() {
        char_index += 1;
        if ch == '\n' {
            current_line += 1;
            if current_line == line {
                return char_index;
            }
        }
    }

    char_index
}

fn default_shortcut_bindings() -> Vec<(KeyChord, EditorCommand)> {
    vec![
        (
            KeyChord::new("S", vec![KeyModifier::CmdOrCtrl]),
            EditorCommand::Save,
        ),
        (
            KeyChord::new("Z", vec![KeyModifier::CmdOrCtrl]),
            EditorCommand::Undo,
        ),
        (
            KeyChord::new("Z", vec![KeyModifier::CmdOrCtrl, KeyModifier::Shift]),
            EditorCommand::Redo,
        ),
        (
            KeyChord::new("Y", vec![KeyModifier::CmdOrCtrl]),
            EditorCommand::Redo,
        ),
        (
            KeyChord::new("P", vec![KeyModifier::CmdOrCtrl]),
            EditorCommand::QuickOpen,
        ),
        (
            KeyChord::new("F", vec![KeyModifier::CmdOrCtrl]),
            EditorCommand::FindInFile,
        ),
        (
            KeyChord::new("F", vec![KeyModifier::CmdOrCtrl, KeyModifier::Shift]),
            EditorCommand::FindInWorkspace,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search_service::TextCriteria;
    use std::fs;
    use tempfile::tempdir;

    fn save_chord() -> KeyChord {
        KeyChord::new("S", vec![KeyModifier::CmdOrCtrl])
    }

    fn undo_chord() -> KeyChord {
        KeyChord::new("Z", vec![KeyModifier::CmdOrCtrl])
    }

    fn redo_chord() -> KeyChord {
        KeyChord::new("Z", vec![KeyModifier::CmdOrCtrl, KeyModifier::Shift])
    }

    fn redo_windows_chord() -> KeyChord {
        KeyChord::new("Y", vec![KeyModifier::CmdOrCtrl])
    }

    fn quick_open_chord() -> KeyChord {
        KeyChord::new("P", vec![KeyModifier::CmdOrCtrl])
    }

    fn find_in_file_chord() -> KeyChord {
        KeyChord::new("F", vec![KeyModifier::CmdOrCtrl])
    }

    fn find_in_workspace_chord() -> KeyChord {
        KeyChord::new("F", vec![KeyModifier::CmdOrCtrl, KeyModifier::Shift])
    }

    #[test]
    fn ファイルを開くとバッファとカーソル初期値を保持する() {
        let mut core = EditorCore::new();

        let snapshot = core.open_file("docs/readme.md", "# heading");

        assert_eq!(
            snapshot,
            EditorBufferSnapshot {
                file_path: PathBuf::from("docs/readme.md"),
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
                file_path: PathBuf::from("docs/readme.md"),
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
        assert_eq!(request.file_path, PathBuf::from("docs/readme.md"));
        assert_eq!(request.content, "Hello!");
        assert_eq!(request.revision, 1);

        assert_eq!(
            core.mark_saved(request.revision),
            MarkSavedOutcome::Saved { revision: 1 }
        );
        assert!(!core.snapshot().expect("バッファが存在する").is_dirty);
    }

    #[test]
    fn バッファ未オープン時の保存要求はnobufferを返す() {
        let mut core = EditorCore::new();
        assert_eq!(
            core.request_save(SaveTrigger::Manual),
            SaveOutcome::NoBuffer
        );
    }

    #[test]
    fn 未編集バッファの保存要求はnotdirtyを返す() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "Hello");

        assert_eq!(
            core.request_save(SaveTrigger::Manual),
            SaveOutcome::NotDirty
        );
    }

    #[test]
    fn mark_savedは古いrevisionならstaleを返す() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "Hello");
        core.set_cursor(5);
        core.insert_text("!");

        assert_eq!(
            core.mark_saved(0),
            MarkSavedOutcome::StaleRevision {
                current_revision: 1
            }
        );
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

    #[test]
    fn 既定ショートカット登録でrequired_chordsを一括登録できる() {
        let mut core = EditorCore::new();
        let registered = core.register_default_shortcuts();

        assert_eq!(registered.len(), 7);
        assert!(
            registered
                .iter()
                .all(|(_, outcome)| *outcome == RegisterShortcutOutcome::Registered)
        );
    }

    #[test]
    fn cmd_ctrl_p_f_shift_fでeditor_commandを実行できる() {
        let mut core = EditorCore::new();
        core.register_default_shortcuts();

        assert_eq!(
            core.dispatch_shortcut(&quick_open_chord()),
            ShortcutDispatchOutcome::Executed {
                command: EditorCommand::QuickOpen,
                outcome: CommandExecutionOutcome::QuickOpen,
            }
        );
        assert_eq!(
            core.dispatch_shortcut(&find_in_file_chord()),
            ShortcutDispatchOutcome::Executed {
                command: EditorCommand::FindInFile,
                outcome: CommandExecutionOutcome::FindInFile,
            }
        );
        assert_eq!(
            core.dispatch_shortcut(&find_in_workspace_chord()),
            ShortcutDispatchOutcome::Executed {
                command: EditorCommand::FindInWorkspace,
                outcome: CommandExecutionOutcome::FindInWorkspace,
            }
        );
    }

    #[test]
    fn cmd_ctrl_zとcmd_shift_zとctrl_yでundo_redoを実行できる() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "Hello");
        core.set_cursor(5);
        core.insert_text(" world");
        core.register_default_shortcuts();

        assert_eq!(
            core.dispatch_shortcut(&undo_chord()),
            ShortcutDispatchOutcome::Executed {
                command: EditorCommand::Undo,
                outcome: CommandExecutionOutcome::Undo(HistoryOutcome::Applied {
                    revision: 2,
                    cursor_char: 5,
                    is_dirty: false,
                }),
            }
        );
        assert_eq!(
            core.dispatch_shortcut(&redo_chord()),
            ShortcutDispatchOutcome::Executed {
                command: EditorCommand::Redo,
                outcome: CommandExecutionOutcome::Redo(HistoryOutcome::Applied {
                    revision: 3,
                    cursor_char: 11,
                    is_dirty: true,
                }),
            }
        );
        assert_eq!(
            core.dispatch_shortcut(&undo_chord()),
            ShortcutDispatchOutcome::Executed {
                command: EditorCommand::Undo,
                outcome: CommandExecutionOutcome::Undo(HistoryOutcome::Applied {
                    revision: 4,
                    cursor_char: 5,
                    is_dirty: false,
                }),
            }
        );
        assert_eq!(
            core.dispatch_shortcut(&redo_windows_chord()),
            ShortcutDispatchOutcome::Executed {
                command: EditorCommand::Redo,
                outcome: CommandExecutionOutcome::Redo(HistoryOutcome::Applied {
                    revision: 5,
                    cursor_char: 11,
                    is_dirty: true,
                }),
            }
        );
    }

    #[test]
    fn 右クリックメニューを開いて保存メニューからsave_requestを実行できる() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "Hello");
        core.set_cursor(5);
        core.insert_text("!");

        assert_eq!(
            core.open_context_menu(),
            OpenEditorContextMenuOutcome::Opened {
                file_path: PathBuf::from("docs/readme.md"),
            }
        );
        assert_eq!(
            core.execute_context_menu_item(EditorContextMenuItem::Save),
            ExecuteEditorContextMenuOutcome::Executed {
                item: EditorContextMenuItem::Save,
                outcome: CommandExecutionOutcome::Save(SaveOutcome::Requested(EditorSaveRequest {
                    file_path: PathBuf::from("docs/readme.md"),
                    content: "Hello!".to_string(),
                    revision: 1,
                    trigger: SaveTrigger::ContextMenu,
                })),
            }
        );
        assert_eq!(core.context_menu(), &EditorContextMenu::default());
    }

    #[test]
    fn 右クリックメニューでコピーとmarkdownメニュー要求を発火できる() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "# Hello");

        assert_eq!(
            core.open_context_menu(),
            OpenEditorContextMenuOutcome::Opened {
                file_path: PathBuf::from("docs/readme.md"),
            }
        );
        assert_eq!(
            core.execute_context_menu_item(EditorContextMenuItem::Copy),
            ExecuteEditorContextMenuOutcome::Executed {
                item: EditorContextMenuItem::Copy,
                outcome: CommandExecutionOutcome::Copy(CopyOutcome::Copied),
            }
        );

        assert_eq!(
            core.open_context_menu(),
            OpenEditorContextMenuOutcome::Opened {
                file_path: PathBuf::from("docs/readme.md"),
            }
        );
        assert_eq!(
            core.execute_context_menu_item(EditorContextMenuItem::MarkdownMenu),
            ExecuteEditorContextMenuOutcome::Executed {
                item: EditorContextMenuItem::MarkdownMenu,
                outcome: CommandExecutionOutcome::OpenMarkdownMenu,
            }
        );
    }

    #[test]
    fn 右クリックメニューからmarkdown_preview要求を送れる() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "# Hello");
        core.drain_events();

        assert_eq!(
            core.open_context_menu(),
            OpenEditorContextMenuOutcome::Opened {
                file_path: PathBuf::from("docs/readme.md"),
            }
        );
        core.drain_events();

        assert_eq!(
            core.execute_context_menu_item(EditorContextMenuItem::MarkdownPreview),
            ExecuteEditorContextMenuOutcome::Executed {
                item: EditorContextMenuItem::MarkdownPreview,
                outcome: CommandExecutionOutcome::OpenMarkdownPreview,
            }
        );

        assert_eq!(
            core.drain_events(),
            vec![
                EditorCoreEvent::MarkdownPreviewRequested(MarkdownPreviewRequestedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                }),
                EditorCoreEvent::ContextMenuItemExecuted(ContextMenuItemExecutedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    item: EditorContextMenuItem::MarkdownPreview,
                }),
            ]
        );
    }

    #[test]
    fn バッファなしまたはメニュー未オープンでは実行できない() {
        let mut core = EditorCore::new();

        assert_eq!(
            core.open_context_menu(),
            OpenEditorContextMenuOutcome::NoBuffer
        );
        assert_eq!(
            core.execute_context_menu_item(EditorContextMenuItem::Save),
            ExecuteEditorContextMenuOutcome::ContextMenuClosed
        );
    }

    #[test]
    fn markdownサブ機能をサービス経由で要求できる() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "# Heading");
        core.drain_events();

        assert_eq!(
            core.execute_markdown_feature(MarkdownFeature::SyntaxHighlight),
            ExecuteMarkdownFeatureOutcome::Executed {
                feature: MarkdownFeature::SyntaxHighlight,
            }
        );
        assert_eq!(
            core.execute_markdown_feature(MarkdownFeature::ListContinuation),
            ExecuteMarkdownFeatureOutcome::Executed {
                feature: MarkdownFeature::ListContinuation,
            }
        );
        assert_eq!(
            core.execute_markdown_feature(MarkdownFeature::PairCompletion),
            ExecuteMarkdownFeatureOutcome::Executed {
                feature: MarkdownFeature::PairCompletion,
            }
        );
        assert_eq!(
            core.execute_markdown_feature(MarkdownFeature::OpenLink),
            ExecuteMarkdownFeatureOutcome::Executed {
                feature: MarkdownFeature::OpenLink,
            }
        );
        assert_eq!(
            core.execute_markdown_feature(MarkdownFeature::PreviewSync),
            ExecuteMarkdownFeatureOutcome::Executed {
                feature: MarkdownFeature::PreviewSync,
            }
        );
        assert_eq!(
            core.drain_events(),
            vec![
                EditorCoreEvent::MarkdownFeatureRequested(MarkdownFeatureRequestedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    feature: MarkdownFeature::SyntaxHighlight,
                }),
                EditorCoreEvent::MarkdownFeatureRequested(MarkdownFeatureRequestedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    feature: MarkdownFeature::ListContinuation,
                }),
                EditorCoreEvent::MarkdownFeatureRequested(MarkdownFeatureRequestedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    feature: MarkdownFeature::PairCompletion,
                }),
                EditorCoreEvent::MarkdownFeatureRequested(MarkdownFeatureRequestedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    feature: MarkdownFeature::OpenLink,
                }),
                EditorCoreEvent::MarkdownFeatureRequested(MarkdownFeatureRequestedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    feature: MarkdownFeature::PreviewSync,
                }),
            ]
        );
    }

    #[test]
    fn markdown_preview同期でカーソルが指定行へ移動する() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "line1\nline2\nline3");
        core.drain_events();

        assert_eq!(
            core.sync_with_markdown_preview(2),
            CursorMoveOutcome::Moved { cursor_char: 12 }
        );
    }

    #[test]
    fn markdown_headingクリックで対象行へジャンプできる() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "first\n## heading\ncontent");
        core.drain_events();

        let heading = MarkdownHeading {
            line: 1,
            level: 2,
            title: "heading".to_string(),
        };

        assert_eq!(
            core.jump_to_markdown_heading(&heading),
            CursorMoveOutcome::Moved { cursor_char: 6 }
        );
    }

    #[test]
    fn markdown編集で差分検知とプレビュー同期イベントを発火する() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "# Heading\n- item");
        core.drain_events();
        core.set_cursor(16);
        core.drain_events();

        assert!(matches!(
            core.insert_text("\n## Section"),
            EditOutcome::Edited {
                revision: 1,
                is_dirty: true,
                ..
            }
        ));

        assert_eq!(
            core.drain_events(),
            vec![
                EditorCoreEvent::MinimapOverlaysUpdated(MinimapOverlaysUpdatedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    overlay_count: 0,
                }),
                EditorCoreEvent::SmartGutterIndicatorsUpdated(
                    crate::smart_gutter_service::SmartGutterIndicatorsUpdatedEvent {
                        file_path: PathBuf::from("docs/readme.md"),
                        indicator_count: 0,
                    },
                ),
                EditorCoreEvent::BufferEdited(BufferEditedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    revision: 1,
                    cursor_char: 27,
                    is_dirty: true,
                }),
                EditorCoreEvent::MarkdownDiffObserved(MarkdownDiffObservedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    revision: 1,
                    changed_line_count: 1,
                }),
                EditorCoreEvent::MarkdownPreviewSynced(MarkdownPreviewSyncedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    revision: 1,
                    heading_count: 2,
                    headings: vec![
                        MarkdownHeading {
                            line: 0,
                            level: 1,
                            title: "Heading".to_string(),
                        },
                        MarkdownHeading {
                            line: 2,
                            level: 2,
                            title: "Section".to_string(),
                        },
                    ],
                }),
            ]
        );
    }

    #[test]
    fn markdown_preview_snapshotで見出しが保持される() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "# Heading\n- item");
        core.drain_events();

        let end_pos = core
            .snapshot()
            .expect("バッファが開かれている")
            .content
            .chars()
            .count();
        core.set_cursor(end_pos);
        core.drain_events();

        assert!(matches!(
            core.insert_text("\n## Section"),
            EditOutcome::Edited {
                revision: 1,
                is_dirty: true,
                ..
            }
        ));

        let snapshot = core
            .markdown_preview_snapshot()
            .expect("Markdown preview スナップショットが得られる");

        assert_eq!(snapshot.file_path, PathBuf::from("docs/readme.md"));
        assert_eq!(snapshot.revision, 1);
        assert_eq!(snapshot.headings.len(), 2);
        assert_eq!(
            snapshot.headings[0],
            MarkdownHeading {
                line: 0,
                level: 1,
                title: "Heading".to_string(),
            }
        );
        assert_eq!(
            snapshot.headings[1],
            MarkdownHeading {
                line: 2,
                level: 2,
                title: "Section".to_string(),
            }
        );
    }

    #[test]
    fn markdown_preview_index指定で見出しジャンプできる() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "# Heading\n- item");
        core.drain_events();

        let end_pos = core
            .snapshot()
            .expect("バッファが開かれている")
            .content
            .chars()
            .count();
        core.set_cursor(end_pos);
        core.drain_events();

        assert!(matches!(
            core.insert_text("\n## Section"),
            EditOutcome::Edited {
                revision: 1,
                is_dirty: true,
                ..
            }
        ));

        core.drain_events();

        assert_eq!(
            core.jump_to_markdown_heading_by_index(1),
            CursorMoveOutcome::Moved { cursor_char: 17 }
        );
    }

    #[test]
    fn markdown_preview未同期では見出しジャンプはtarget_not_foundになる() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "# Heading\n- item");
        core.drain_events();

        assert_eq!(
            core.jump_to_markdown_heading_by_index(0),
            CursorMoveOutcome::TargetNotFound
        );
    }

    #[test]
    fn markdown_preview範囲外indexはtarget_not_foundになる() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "# Heading\n- item");
        core.drain_events();

        let end_pos = core
            .snapshot()
            .expect("バッファが開かれている")
            .content
            .chars()
            .count();
        core.set_cursor(end_pos);
        core.drain_events();
        core.insert_text("\n## Section");
        core.drain_events();

        assert_eq!(
            core.jump_to_markdown_heading_by_index(2),
            CursorMoveOutcome::TargetNotFound
        );
    }

    #[test]
    fn minimapに検索結果とai_git差分オーバーレイを反映できる() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "one\ntwo\nthree\nfour");
        core.drain_events();

        assert_eq!(
            core.update_minimap_overlays(vec![
                MinimapOverlay::search_result(2),
                MinimapOverlay::ai_diff(3, "focus-ai-1"),
                MinimapOverlay::git_diff(4, "focus-git-1"),
            ]),
            UpdateMinimapOverlaysOutcome::Updated { overlay_count: 3 }
        );

        assert_eq!(
            core.minimap_snapshot(),
            Some(MinimapSnapshot {
                file_path: PathBuf::from("docs/readme.md"),
                line_count: 4,
                overlays: vec![
                    MinimapOverlay::search_result(2),
                    MinimapOverlay::ai_diff(3, "focus-ai-1"),
                    MinimapOverlay::git_diff(4, "focus-git-1"),
                ],
                active_focus_id: None,
            })
        );
        assert_eq!(
            core.drain_events(),
            vec![EditorCoreEvent::MinimapOverlaysUpdated(
                MinimapOverlaysUpdatedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    overlay_count: 3,
                }
            )]
        );
    }

    #[test]
    fn minimapのfocus_id同期でcommand_hubとstructure_path向けイベントを発火する() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "one\ntwo\nthree\nfour");
        core.drain_events();
        core.update_minimap_overlays(vec![
            MinimapOverlay::ai_diff(3, "focus-ai-1"),
            MinimapOverlay::git_diff(4, "focus-git-1"),
        ]);
        core.drain_events();

        assert_eq!(
            core.sync_minimap_focus_id("focus-ai-1"),
            SyncMinimapFocusOutcome::Synced {
                focus_id: "focus-ai-1".to_string(),
            }
        );
        assert_eq!(
            core.minimap_snapshot()
                .expect("minimap snapshot が存在する")
                .active_focus_id,
            Some("focus-ai-1".to_string())
        );
        assert_eq!(
            core.drain_events(),
            vec![EditorCoreEvent::MinimapFocusIdSynced(
                MinimapFocusIdSyncedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    focus_id: "focus-ai-1".to_string(),
                    targets: vec![
                        crate::minimap_service::FocusSyncTarget::CommandHub,
                        crate::minimap_service::FocusSyncTarget::StructurePath,
                    ],
                }
            )]
        );
    }

    #[test]
    fn minimapの同一focus_id再同期はfocus_not_foundにしない() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "one\ntwo\nthree\nfour");
        core.drain_events();
        core.update_minimap_overlays(vec![MinimapOverlay::ai_diff(3, "focus-ai-1")]);
        core.drain_events();

        assert_eq!(
            core.sync_minimap_focus_id("focus-ai-1"),
            SyncMinimapFocusOutcome::Synced {
                focus_id: "focus-ai-1".to_string(),
            }
        );
        core.drain_events();

        assert_eq!(
            core.sync_minimap_focus_id("focus-ai-1"),
            SyncMinimapFocusOutcome::Synced {
                focus_id: "focus-ai-1".to_string(),
            }
        );
        assert!(core.drain_events().is_empty());
    }

    #[test]
    fn undoでminimapとsmart_gutterの更新イベントを再発火する() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.txt", "one\ntwo\nthree");
        core.drain_events();
        core.set_cursor(13);
        core.drain_events();
        core.insert_text("\nfour");
        core.update_minimap_overlays(vec![MinimapOverlay::ai_diff(4, "focus-ai-1")]);
        core.update_smart_gutter_indicators(vec![
            crate::smart_gutter_service::SmartGutterIndicator::ai_diff(
                4,
                "focus-ai-1",
                "approval-1",
            ),
        ]);
        core.sync_minimap_focus_id("focus-ai-1");
        core.sync_smart_gutter_focus_id("focus-ai-1");
        core.drain_events();

        assert_eq!(
            core.undo(),
            HistoryOutcome::Applied {
                revision: 2,
                cursor_char: 13,
                is_dirty: false,
            }
        );
        assert_eq!(
            core.drain_events(),
            vec![
                EditorCoreEvent::MinimapOverlaysUpdated(MinimapOverlaysUpdatedEvent {
                    file_path: PathBuf::from("docs/readme.txt"),
                    overlay_count: 0,
                }),
                EditorCoreEvent::SmartGutterIndicatorsUpdated(
                    crate::smart_gutter_service::SmartGutterIndicatorsUpdatedEvent {
                        file_path: PathBuf::from("docs/readme.txt"),
                        indicator_count: 0,
                    },
                ),
                EditorCoreEvent::BufferEdited(BufferEditedEvent {
                    file_path: PathBuf::from("docs/readme.txt"),
                    revision: 2,
                    cursor_char: 13,
                    is_dirty: false,
                }),
            ]
        );
    }

    #[test]
    fn smart_gutterにai_git差分を登録してfocus同期できる() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "one\ntwo\nthree\nfour");
        core.drain_events();

        assert_eq!(
            core.update_smart_gutter_indicators(vec![
                crate::smart_gutter_service::SmartGutterIndicator::ai_diff(
                    2,
                    "focus-ai-1",
                    "approval-1",
                ),
                crate::smart_gutter_service::SmartGutterIndicator::git_diff(4, "focus-git-1"),
            ]),
            UpdateSmartGutterIndicatorsOutcome::Updated { indicator_count: 2 }
        );
        assert_eq!(
            core.sync_smart_gutter_focus_id("focus-ai-1"),
            SyncSmartGutterFocusOutcome::Synced {
                focus_id: "focus-ai-1".to_string(),
            }
        );
        assert_eq!(
            core.drain_events(),
            vec![
                EditorCoreEvent::SmartGutterIndicatorsUpdated(
                    crate::smart_gutter_service::SmartGutterIndicatorsUpdatedEvent {
                        file_path: PathBuf::from("docs/readme.md"),
                        indicator_count: 2,
                    },
                ),
                EditorCoreEvent::SmartGutterFocusIdSynced(
                    crate::smart_gutter_service::SmartGutterFocusIdSyncedEvent {
                        file_path: PathBuf::from("docs/readme.md"),
                        focus_id: "focus-ai-1".to_string(),
                        targets: vec![
                            crate::smart_gutter_service::SmartGutterSyncTarget::CommandHub,
                            crate::smart_gutter_service::SmartGutterSyncTarget::StructurePath,
                        ],
                    },
                ),
            ]
        );
    }

    #[test]
    fn smart_gutterのクリックジャンプとapproval_request連携イベントを発火できる() {
        let mut core = EditorCore::new();
        core.open_file("docs/readme.md", "one\ntwo\nthree\nfour");
        core.update_smart_gutter_indicators(vec![
            crate::smart_gutter_service::SmartGutterIndicator::ai_diff(
                2,
                "focus-ai-1",
                "approval-1",
            ),
            crate::smart_gutter_service::SmartGutterIndicator::git_diff(4, "focus-git-1"),
        ]);
        core.drain_events();

        assert_eq!(
            core.jump_to_smart_gutter_focus("focus-git-1"),
            JumpToSmartGutterDiffOutcome::Jumped {
                focus_id: "focus-git-1".to_string(),
                line: 4,
            }
        );
        assert_eq!(
            core.open_smart_gutter_approval_request("focus-ai-1"),
            OpenSmartGutterApprovalRequestOutcome::Opened {
                focus_id: "focus-ai-1".to_string(),
                approval_request_id: "approval-1".to_string(),
            }
        );
        assert_eq!(
            core.drain_events(),
            vec![
                EditorCoreEvent::SmartGutterJumpRequested(
                    crate::smart_gutter_service::SmartGutterJumpRequestedEvent {
                        file_path: PathBuf::from("docs/readme.md"),
                        focus_id: "focus-git-1".to_string(),
                        line: 4,
                    },
                ),
                EditorCoreEvent::SmartGutterApprovalRequestOpened(
                    crate::smart_gutter_service::SmartGutterApprovalRequestOpenedEvent {
                        file_path: PathBuf::from("docs/readme.md"),
                        focus_id: "focus-ai-1".to_string(),
                        approval_request_id: "approval-1".to_string(),
                        targets: vec![
                            crate::smart_gutter_service::SmartGutterSyncTarget::CommandHub,
                            crate::smart_gutter_service::SmartGutterSyncTarget::StructurePath,
                        ],
                    },
                ),
            ]
        );
    }

    #[test]
    fn markdown以外のファイルでmarkdown機能要求は実行されない() {
        let mut core = EditorCore::new();
        core.open_file("docs/note.txt", "plain");
        core.drain_events();

        assert_eq!(
            core.execute_markdown_feature(MarkdownFeature::SyntaxHighlight),
            ExecuteMarkdownFeatureOutcome::NotMarkdownFile
        );
        assert!(core.drain_events().is_empty());
    }

    fn write_search_file(base: &std::path::Path, relative: &str, content: &str) {
        let path = base.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn search_workspace_emits_results_and_focus_event() {
        let dir = tempdir().unwrap();
        write_search_file(dir.path(), "docs/notes.md", "needle line\nunused");

        let mut core = EditorCore::new();
        core.set_search_root(dir.path());
        let outcome = core
            .search_workspace(SearchQuery::literal("needle"))
            .unwrap();
        assert_eq!(outcome, SearchWorkspaceOutcome::Matches { match_count: 1 });

        let events = core.drain_events();
        assert_eq!(events.len(), 1);
        if let EditorCoreEvent::SearchResultsUpdated(event) = &events[0] {
            assert_eq!(event.matches.len(), 1);
            assert_eq!(event.focus_index, Some(0));
            assert!(event.focus_match.is_some());
            match event.query.pattern() {
                TextCriteria::Literal(text) => assert_eq!(text, "needle"),
                _ => panic!("patternはliteralであるべき"),
            }
            assert_eq!(event.matches[0].line_number, 1);
        } else {
            panic!("SearchResultsUpdatedイベントが期待される");
        }
    }

    #[test]
    fn navigating_search_results_emits_focus_change_event() {
        let dir = tempdir().unwrap();
        write_search_file(
            dir.path(),
            "docs/notes.md",
            "needle one\nneedle two\nignored",
        );

        let mut core = EditorCore::new();
        core.set_search_root(dir.path());
        let outcome = core
            .search_workspace(SearchQuery::literal("needle"))
            .unwrap();
        assert_eq!(outcome, SearchWorkspaceOutcome::Matches { match_count: 2 });
        core.drain_events();

        let next = core.advance_search_result().unwrap();
        assert_eq!(next.line_number, 2);

        let events = core.drain_events();
        assert_eq!(
            events,
            vec![EditorCoreEvent::SearchFocusChanged(
                SearchFocusChangedEvent {
                    focus_index: Some(1),
                    focus_match: Some(next.clone()),
                }
            )]
        );
    }
}
