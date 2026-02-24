use nue_core::workspace::registry::{
    ExcludeWorkspaceOutcome, WorkspacePathValidation, WorkspaceRegistry,
};

use super::{WorkspaceRailExcludeOutcome, WorkspaceRailStatusUpdate};

/// WorkspaceRegistry との橋渡しを行うヘルパーモジュール。
pub struct WorkspaceRailConnector;

impl WorkspaceRailConnector {
    /// Registry から蓄積されたステータス更新を取り出す。
    pub fn drain_status_updates(
        registry: &mut WorkspaceRegistry,
    ) -> Vec<WorkspaceRailStatusUpdate> {
        registry
            .drain_status_events()
            .into_iter()
            .map(|event| WorkspaceRailStatusUpdate {
                workspace_id: event.workspace_id,
                state: event.state,
                revision: event.revision,
            })
            .collect()
    }

    /// コンテキストメニューから選択されているワークスペースを除外する。
    pub fn exclude_selected_workspace(
        registry: &mut WorkspaceRegistry,
    ) -> WorkspaceRailExcludeOutcome {
        match registry.exclude_context_menu_target() {
            ExcludeWorkspaceOutcome::Excluded { workspace_id } => {
                WorkspaceRailExcludeOutcome::Excluded { workspace_id }
            }
            ExcludeWorkspaceOutcome::ContextMenuClosed => {
                WorkspaceRailExcludeOutcome::ContextMenuClosed
            }
            ExcludeWorkspaceOutcome::WorkspaceNotFound => {
                WorkspaceRailExcludeOutcome::WorkspaceNotFound
            }
        }
    }

    /// パス検証の結果に応じた UI 表示用のメッセージを取得する。
    pub fn validation_message(validation: &WorkspacePathValidation) -> &'static str {
        match validation {
            WorkspacePathValidation::Empty => "パスが空です。",
            WorkspacePathValidation::MustBeAbsolute => "絶対パスを指定してください。",
            WorkspacePathValidation::NotFoundOrInaccessible => {
                "指定したパスが存在しないかアクセスできません。"
            }
            WorkspacePathValidation::MustBeDirectory => "ディレクトリを指定してください。",
            WorkspacePathValidation::Duplicate => "同じワークスペースが既に登録されています。",
            WorkspacePathValidation::Valid => "指定したパスは有効です。",
        }
    }
}
