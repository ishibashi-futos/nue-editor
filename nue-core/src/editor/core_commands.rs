use super::{
    CommandExecutionOutcome, ContextMenuItemExecutedEvent, CopyOutcome, EditorCommand,
    EditorContextMenu, EditorContextMenuItem, EditorCore, EditorCoreEvent,
    ExecuteEditorContextMenuOutcome, RegisterShortcutOutcome, SaveTrigger, ShortcutDispatchOutcome,
    ShortcutDispatchedEvent, ShortcutRegisteredEvent,
};
use crate::editor::core_support::default_shortcut_bindings;

impl EditorCore {
    pub fn register_shortcut(
        &mut self,
        chord: super::KeyChord,
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

    pub fn register_default_shortcuts(
        &mut self,
    ) -> Vec<(super::KeyChord, RegisterShortcutOutcome)> {
        let bindings = default_shortcut_bindings();
        let mut registered = Vec::with_capacity(bindings.len());
        for (chord, command) in bindings {
            let outcome = self.register_shortcut(chord.clone(), command);
            registered.push((chord, outcome));
        }
        registered
    }

    pub fn dispatch_shortcut(&mut self, chord: &super::KeyChord) -> ShortcutDispatchOutcome {
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

    fn request_copy(&mut self) -> CopyOutcome {
        let Some(buffer) = self.active_buffer.as_ref() else {
            return CopyOutcome::NoBuffer;
        };
        self.events
            .push_back(EditorCoreEvent::CopyRequested(super::CopyRequestedEvent {
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
                super::MarkdownMenuRequestedEvent {
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
                super::MarkdownPreviewRequestedEvent {
                    file_path: buffer.file_path.clone(),
                },
            ));
    }

    pub(super) fn close_context_menu(&mut self) {
        self.context_menu = EditorContextMenu::default();
    }
}
