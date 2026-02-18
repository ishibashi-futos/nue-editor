use nue_core::editor_core::{
    EditorContextMenuItem, EditorCore, ExecuteEditorContextMenuOutcome,
    ExecuteMarkdownFeatureOutcome, KeyChord, OpenEditorContextMenuOutcome, ShortcutDispatchOutcome,
};
use nue_core::markdown_service::MarkdownFeature;

/// EditorCore への入力操作を UI から抽象化するコントローラ。
pub struct EditorInputController<'a> {
    core: &'a mut EditorCore,
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
}
