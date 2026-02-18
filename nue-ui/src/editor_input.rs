use nue_core::editor_core::{
    EditorContextMenuItem, EditorCore, ExecuteEditorContextMenuOutcome,
    ExecuteMarkdownFeatureOutcome, KeyChord, OpenEditorContextMenuOutcome, ShortcutDispatchOutcome,
};
use nue_core::markdown_service::MarkdownFeature;

/// EditorCore への入力操作を UI から抽象化するコントローラ。
pub struct EditorInputController<'a> {
    core: &'a mut EditorCore,
}

/// UI からの入力イベントを定義する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorInputEvent {
    Shortcut(KeyChord),
    OpenContextMenu,
    ExecuteContextMenuItem(EditorContextMenuItem),
    MarkdownFeature(MarkdownFeature),
}

/// UI 入力イベントの結果を表す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorInputOutcome {
    Shortcut(ShortcutDispatchOutcome),
    ContextMenuOpened(OpenEditorContextMenuOutcome),
    ContextMenuExecuted(ExecuteEditorContextMenuOutcome),
    MarkdownFeature(ExecuteMarkdownFeatureOutcome),
}

impl<'a> EditorInputController<'a> {
    /// 指定した EditorCore インスタンスに入力操作を委譲する構造体を作成する。
    pub fn new(core: &'a mut EditorCore) -> Self {
        Self { core }
    }

    /// ショートカット入力を受け取り `dispatch_shortcut` を呼び出す。
    pub fn dispatch_shortcut(&mut self, chord: &KeyChord) -> ShortcutDispatchOutcome {
        self.core.dispatch_shortcut(chord)
    }

    /// 右クリックメニューを開く指示を `open_context_menu` に転送する。
    pub fn open_context_menu(&mut self) -> OpenEditorContextMenuOutcome {
        self.core.open_context_menu()
    }

    /// コンテキストメニュー項目を選択したときに `execute_context_menu_item` を実行する。
    pub fn execute_context_menu_item(
        &mut self,
        item: EditorContextMenuItem,
    ) -> ExecuteEditorContextMenuOutcome {
        self.core.execute_context_menu_item(item)
    }

    /// Markdown 固有機能の命令を `execute_markdown_feature` に渡す。
    pub fn execute_markdown_feature(
        &mut self,
        feature: MarkdownFeature,
    ) -> ExecuteMarkdownFeatureOutcome {
        self.core.execute_markdown_feature(feature)
    }

    /// UI から渡された入力イベントを処理する。
    pub fn handle_event(&mut self, event: EditorInputEvent) -> EditorInputOutcome {
        match event {
            EditorInputEvent::Shortcut(chord) => {
                EditorInputOutcome::Shortcut(self.dispatch_shortcut(&chord))
            }
            EditorInputEvent::OpenContextMenu => {
                EditorInputOutcome::ContextMenuOpened(self.open_context_menu())
            }
            EditorInputEvent::ExecuteContextMenuItem(item) => {
                EditorInputOutcome::ContextMenuExecuted(self.execute_context_menu_item(item))
            }
            EditorInputEvent::MarkdownFeature(feature) => {
                EditorInputOutcome::MarkdownFeature(self.execute_markdown_feature(feature))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nue_core::editor_core::{
        CommandExecutionOutcome, CopyOutcome, EditorCommand, EditorContextMenuItem,
        ExecuteEditorContextMenuOutcome, ExecuteMarkdownFeatureOutcome, KeyChord, KeyModifier,
        OpenEditorContextMenuOutcome, SaveOutcome,
    };
    use nue_core::markdown_service::MarkdownFeature;
    use std::path::PathBuf;

    fn sample_path(name: &str, ext: &str) -> PathBuf {
        PathBuf::from(format!("/project/{}.{}", name, ext))
    }

    #[test]
    fn dispatch_shortcutはregister_default_shortcutsの登録内容を利用できる() {
        let mut core = EditorCore::new();
        let file_path = sample_path("main", "rs");
        core.open_file(&file_path, "fn main() {}\n");
        core.register_default_shortcuts();
        let mut controller = EditorInputController::new(&mut core);
        let chord = KeyChord::new("s", vec![KeyModifier::CmdOrCtrl]);

        let outcome = controller.dispatch_shortcut(&chord);

        assert!(matches!(
            outcome,
            ShortcutDispatchOutcome::Executed { command, outcome }
            if command == EditorCommand::Save
                && outcome == CommandExecutionOutcome::Save(SaveOutcome::NotDirty)
        ));
    }

    #[test]
    fn context_menu_propagates_to_editor_core() {
        let mut core = EditorCore::new();
        let file_path = sample_path("explorer", "rs");
        core.open_file(&file_path, "fn context() {}\n");
        let mut controller = EditorInputController::new(&mut core);

        let opened = controller.open_context_menu();
        assert!(matches!(
            opened,
            OpenEditorContextMenuOutcome::Opened { file_path: ref path } if path == &file_path
        ));

        let executed = controller.execute_context_menu_item(EditorContextMenuItem::Copy);
        assert!(matches!(
            executed,
            ExecuteEditorContextMenuOutcome::Executed { item, outcome }
            if item == EditorContextMenuItem::Copy
                && matches!(outcome, CommandExecutionOutcome::Copy(CopyOutcome::Copied))
        ));
        assert!(!controller.core.context_menu().is_open);
    }

    #[test]
    fn markdown_feature_runs_on_markdown_file() {
        let mut core = EditorCore::new();
        let file_path = sample_path("notes", "md");
        core.open_file(&file_path, "# Title\n\n- item\n");
        let mut controller = EditorInputController::new(&mut core);

        let outcome = controller.execute_markdown_feature(MarkdownFeature::SyntaxHighlight);
        assert_eq!(
            outcome,
            ExecuteMarkdownFeatureOutcome::Executed {
                feature: MarkdownFeature::SyntaxHighlight,
            }
        );
    }

    #[test]
    fn handle_event_dispatches_shortcut_chords() {
        let mut core = EditorCore::new();
        let file_path = sample_path("main", "rs");
        core.open_file(&file_path, "fn main() {}\n");
        core.register_default_shortcuts();

        let mut controller = EditorInputController::new(&mut core);
        let chord = KeyChord::new("s", vec![KeyModifier::CmdOrCtrl]);
        let event = EditorInputEvent::Shortcut(chord.clone());

        let outcome = controller.handle_event(event);

        assert_eq!(
            outcome,
            EditorInputOutcome::Shortcut(ShortcutDispatchOutcome::Executed {
                command: EditorCommand::Save,
                outcome: CommandExecutionOutcome::Save(SaveOutcome::NotDirty)
            })
        );
        // `handle_event` should accept multiple invocations without reinitializing the controller.
        let second_event = EditorInputEvent::Shortcut(chord);
        let second_outcome = controller.handle_event(second_event);

        assert!(matches!(
            second_outcome,
            EditorInputOutcome::Shortcut(ShortcutDispatchOutcome::Executed { .. })
        ));
    }

    #[test]
    fn handle_event_for_context_menu_and_markdown() {
        let mut core = EditorCore::new();
        let file_path = sample_path("context", "md");
        core.open_file(&file_path, "fn context() {}\n");
        let mut controller = EditorInputController::new(&mut core);

        let open_outcome = controller.handle_event(EditorInputEvent::OpenContextMenu);
        assert!(matches!(
            open_outcome,
            EditorInputOutcome::ContextMenuOpened(OpenEditorContextMenuOutcome::Opened { file_path: opened_path })
            if opened_path == file_path
        ));

        let execute_outcome = controller.handle_event(EditorInputEvent::ExecuteContextMenuItem(
            EditorContextMenuItem::Copy,
        ));
        assert!(matches!(
            execute_outcome,
            EditorInputOutcome::ContextMenuExecuted(ExecuteEditorContextMenuOutcome::Executed { item, outcome })
            if item == EditorContextMenuItem::Copy
                && matches!(outcome, CommandExecutionOutcome::Copy(CopyOutcome::Copied))
        ));

        let markdown_outcome = controller.handle_event(EditorInputEvent::MarkdownFeature(
            MarkdownFeature::SyntaxHighlight,
        ));
        assert_eq!(
            markdown_outcome,
            EditorInputOutcome::MarkdownFeature(ExecuteMarkdownFeatureOutcome::Executed {
                feature: MarkdownFeature::SyntaxHighlight
            })
        );
    }
}
