use nue_core::workspace_rail::WorkspaceRailState;
use nue_core::workspace_registry::{
    ExcludeWorkspaceOutcome, WorkspacePathValidation, WorkspaceRegistry,
};

/// UI 側が参照するワークスペースステータス更新情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRailStatusUpdate {
    pub workspace_id: String,
    pub state: WorkspaceRailState,
    pub revision: u64,
}

/// WorkspaceRegistry 上の削除操作の結果を UI へ伝える列挙型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceRailExcludeOutcome {
    /// 指定したワークスペースが正常に除外された。
    Excluded { workspace_id: String },
    /// コンテキストメニューは閉じた状態で対応する対象がない。
    ContextMenuClosed,
    /// 対象ワークスペースが見つからなかった。
    WorkspaceNotFound,
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use nue_core::workspace_rail::WorkspaceRailState;
    use nue_core::workspace_registry::{AddWorkspaceOutcome, WorkspacePathValidation};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempWorkspaceDir {
        path: PathBuf,
    }

    impl TempWorkspaceDir {
        fn new(prefix: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("現在時刻が必要")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("{prefix}-{unique}"));
            fs::create_dir_all(&path).expect("テンポラリディレクトリを作成できる");
            Self { path }
        }

        fn path_str(&self) -> &str {
            self.path
                .to_str()
                .expect("テンポラリディレクトリはUTF-8である")
        }
    }

    impl Drop for TempWorkspaceDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn create_workspace(registry: &mut WorkspaceRegistry, dir: &TempWorkspaceDir) -> String {
        registry.open_add_dialog();
        registry.update_dialog_path(dir.path_str());
        match registry.submit_add() {
            AddWorkspaceOutcome::Added { workspace_id } => workspace_id,
            other => panic!("ワークスペース追加に失敗: {other:?}"),
        }
    }

    #[test]
    fn drain_status_updates_returns_mapped_events() {
        let mut registry = WorkspaceRegistry::new();
        let dir = TempWorkspaceDir::new("workspace-test");
        let workspace_id = create_workspace(&mut registry, &dir);

        registry.update_workspace_status_from_server(&workspace_id, WorkspaceRailState::Busy);
        let updates = WorkspaceRailConnector::drain_status_updates(&mut registry);

        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].workspace_id, workspace_id);
        assert_eq!(updates[0].state, WorkspaceRailState::Busy);
        assert_eq!(updates[0].revision, 1);
        assert!(WorkspaceRailConnector::drain_status_updates(&mut registry).is_empty());
    }

    #[test]
    fn exclude_selected_workspace_reports_outcome() {
        let mut registry = WorkspaceRegistry::new();
        let dir = TempWorkspaceDir::new("workspace-test");
        let workspace_id = create_workspace(&mut registry, &dir);

        registry.open_exclude_context_menu(&workspace_id);
        let outcome = WorkspaceRailConnector::exclude_selected_workspace(&mut registry);

        assert_eq!(
            outcome,
            WorkspaceRailExcludeOutcome::Excluded { workspace_id }
        );
    }

    #[test]
    fn validation_message_covers_all_variants() {
        assert_eq!(
            WorkspaceRailConnector::validation_message(&WorkspacePathValidation::Empty),
            "パスが空です。"
        );
        assert_eq!(
            WorkspaceRailConnector::validation_message(&WorkspacePathValidation::MustBeAbsolute),
            "絶対パスを指定してください。"
        );
        assert_eq!(
            WorkspaceRailConnector::validation_message(
                &WorkspacePathValidation::NotFoundOrInaccessible,
            ),
            "指定したパスが存在しないかアクセスできません。"
        );
        assert_eq!(
            WorkspaceRailConnector::validation_message(&WorkspacePathValidation::MustBeDirectory),
            "ディレクトリを指定してください。"
        );
        assert_eq!(
            WorkspaceRailConnector::validation_message(&WorkspacePathValidation::Duplicate),
            "同じワークスペースが既に登録されています。"
        );
        assert_eq!(
            WorkspaceRailConnector::validation_message(&WorkspacePathValidation::Valid),
            "指定したパスは有効です。"
        );
    }
}
