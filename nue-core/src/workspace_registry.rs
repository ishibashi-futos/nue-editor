use crate::workspace_rail::{
    TransitionOutcome, WorkspaceMetadata, WorkspaceRailModel, WorkspaceRailState,
};
use std::collections::VecDeque;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspacePathValidation {
    Empty,
    MustBeAbsolute,
    NotFoundOrInaccessible,
    MustBeDirectory,
    Duplicate,
    Valid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRegistrationDialog {
    pub is_open: bool,
    pub input_path: String,
    pub validation: WorkspacePathValidation,
}

impl Default for WorkspaceRegistrationDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            input_path: String::new(),
            validation: WorkspacePathValidation::Empty,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddWorkspaceOutcome {
    Added { workspace_id: String },
    ValidationFailed { reason: WorkspacePathValidation },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkspaceContextMenu {
    pub is_open: bool,
    pub target_workspace_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenWorkspaceContextMenuOutcome {
    Opened { workspace_id: String },
    WorkspaceNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExcludeWorkspaceOutcome {
    Excluded { workspace_id: String },
    ContextMenuClosed,
    WorkspaceNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceStatusUpdatedEvent {
    pub workspace_id: String,
    pub state: WorkspaceRailState,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceStatusUpdateOutcome {
    Updated(WorkspaceStatusUpdatedEvent),
    Noop,
    WorkspaceNotFound,
}

#[derive(Debug, Default)]
pub struct WorkspaceRegistry {
    workspaces: Vec<WorkspaceRailModel>,
    dialog: WorkspaceRegistrationDialog,
    context_menu: WorkspaceContextMenu,
    status_events: VecDeque<WorkspaceStatusUpdatedEvent>,
}

impl WorkspaceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn dialog(&self) -> &WorkspaceRegistrationDialog {
        &self.dialog
    }

    pub fn workspaces(&self) -> &[WorkspaceRailModel] {
        &self.workspaces
    }

    pub fn context_menu(&self) -> &WorkspaceContextMenu {
        &self.context_menu
    }

    pub fn update_workspace_status_from_server(
        &mut self,
        workspace_id: &str,
        next_state: WorkspaceRailState,
    ) -> WorkspaceStatusUpdateOutcome {
        let Some(index) = self.workspace_index_by_id(workspace_id) else {
            return WorkspaceStatusUpdateOutcome::WorkspaceNotFound;
        };
        let workspace = self
            .workspaces
            .get_mut(index)
            .expect("workspace_index_by_idで存在確認済み");
        match workspace.apply_core_state(next_state) {
            TransitionOutcome::Noop => WorkspaceStatusUpdateOutcome::Noop,
            TransitionOutcome::Changed(_) => {
                let event = WorkspaceStatusUpdatedEvent {
                    workspace_id: workspace.metadata().workspace_id.clone(),
                    state: workspace.snapshot().state,
                    revision: workspace.snapshot().revision,
                };
                self.status_events.push_back(event.clone());
                WorkspaceStatusUpdateOutcome::Updated(event)
            }
        }
    }

    pub fn drain_status_events(&mut self) -> Vec<WorkspaceStatusUpdatedEvent> {
        self.status_events.drain(..).collect()
    }

    pub fn open_add_dialog(&mut self) {
        self.dialog.is_open = true;
        self.dialog.input_path.clear();
        self.dialog.validation = WorkspacePathValidation::Empty;
    }

    pub fn open_exclude_context_menu(
        &mut self,
        workspace_id: &str,
    ) -> OpenWorkspaceContextMenuOutcome {
        if self.workspace_index_by_id(workspace_id).is_none() {
            self.close_context_menu();
            return OpenWorkspaceContextMenuOutcome::WorkspaceNotFound;
        }

        self.context_menu = WorkspaceContextMenu {
            is_open: true,
            target_workspace_id: Some(workspace_id.to_string()),
        };
        OpenWorkspaceContextMenuOutcome::Opened {
            workspace_id: workspace_id.to_string(),
        }
    }

    pub fn exclude_context_menu_target(&mut self) -> ExcludeWorkspaceOutcome {
        if !self.context_menu.is_open {
            return ExcludeWorkspaceOutcome::ContextMenuClosed;
        }

        let Some(target_workspace_id) = self.context_menu.target_workspace_id.as_deref() else {
            self.close_context_menu();
            return ExcludeWorkspaceOutcome::WorkspaceNotFound;
        };

        let Some(index) = self.workspace_index_by_id(target_workspace_id) else {
            self.close_context_menu();
            return ExcludeWorkspaceOutcome::WorkspaceNotFound;
        };
        let removed_workspace_id = self
            .workspaces
            .remove(index)
            .metadata()
            .workspace_id
            .clone();
        self.close_context_menu();

        ExcludeWorkspaceOutcome::Excluded {
            workspace_id: removed_workspace_id,
        }
    }

    pub fn update_dialog_path(&mut self, path: impl Into<String>) -> WorkspacePathValidation {
        self.dialog.input_path = path.into();
        self.dialog.validation = self.validate_path(self.dialog.input_path.as_str());
        self.dialog.validation.clone()
    }

    pub fn submit_add(&mut self) -> AddWorkspaceOutcome {
        let validation = self.validate_path(self.dialog.input_path.as_str());
        if validation != WorkspacePathValidation::Valid {
            self.dialog.validation = validation.clone();
            return AddWorkspaceOutcome::ValidationFailed { reason: validation };
        }

        let canonical_path = canonicalize_path(self.dialog.input_path.as_str())
            .expect("validate_pathで有効なパスのみ到達する");
        let workspace_id = self.next_workspace_id();
        let display_name = workspace_display_name(canonical_path.as_str());
        let metadata = WorkspaceMetadata::new(
            workspace_id.clone(),
            canonical_path.as_str(),
            display_name.as_str(),
        );
        self.workspaces.push(WorkspaceRailModel::new(metadata));
        self.dialog.is_open = false;
        self.dialog.input_path.clear();
        self.dialog.validation = WorkspacePathValidation::Empty;

        AddWorkspaceOutcome::Added { workspace_id }
    }

    fn validate_path(&self, path: &str) -> WorkspacePathValidation {
        let canonical_path = match canonicalize_path(path) {
            Ok(path) => path,
            Err(reason) => return reason,
        };
        if self
            .workspaces
            .iter()
            .filter_map(|workspace| canonicalize_path(workspace.metadata().root_path.as_str()).ok())
            .any(|root_path| root_path == canonical_path)
        {
            return WorkspacePathValidation::Duplicate;
        }

        WorkspacePathValidation::Valid
    }

    fn next_workspace_id(&self) -> String {
        let max_sequence = self
            .workspaces
            .iter()
            .filter_map(|workspace| {
                workspace
                    .metadata()
                    .workspace_id
                    .strip_prefix("workspace-")
                    .and_then(|suffix| suffix.parse::<usize>().ok())
            })
            .max()
            .unwrap_or(0);
        format!("workspace-{}", max_sequence + 1)
    }

    fn workspace_index_by_id(&self, workspace_id: &str) -> Option<usize> {
        self.workspaces
            .iter()
            .position(|workspace| workspace.metadata().workspace_id == workspace_id)
    }

    fn close_context_menu(&mut self) {
        self.context_menu = WorkspaceContextMenu::default();
    }
}

fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed == "/" {
        return "/".to_string();
    }

    let normalized = trimmed.trim_end_matches('/');
    normalized.to_string()
}

fn canonicalize_path(path: &str) -> Result<String, WorkspacePathValidation> {
    let normalized_path = normalize_path(path);
    if normalized_path.is_empty() {
        return Err(WorkspacePathValidation::Empty);
    }
    if !Path::new(normalized_path.as_str()).is_absolute() {
        return Err(WorkspacePathValidation::MustBeAbsolute);
    }

    let metadata = fs::metadata(normalized_path.as_str())
        .map_err(|_| WorkspacePathValidation::NotFoundOrInaccessible)?;
    if !metadata.is_dir() {
        return Err(WorkspacePathValidation::MustBeDirectory);
    }

    let canonical = fs::canonicalize(normalized_path.as_str())
        .map_err(|_| WorkspacePathValidation::NotFoundOrInaccessible)?;
    Ok(canonical.to_string_lossy().into_owned())
}

fn workspace_display_name(path: &str) -> String {
    if path == "/" {
        return "/".to_string();
    }
    path.rsplit('/')
        .find(|segment| !segment.is_empty())
        .unwrap_or(path)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace_rail::WorkspaceRailState;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(prefix: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("現在時刻が必要")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("{prefix}-{unique}"));
            std::fs::create_dir_all(&path).expect("テスト用ディレクトリを作成できる必要がある");

            Self { path }
        }

        fn path_str(&self) -> &str {
            self.path
                .to_str()
                .expect("テスト用ディレクトリはUTF-8パスである必要がある")
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn プラス操作で追加ダイアログが開く() {
        let mut registry = WorkspaceRegistry::new();

        registry.open_add_dialog();

        assert!(registry.dialog().is_open);
        assert_eq!(registry.dialog().input_path, "");
        assert_eq!(registry.dialog().validation, WorkspacePathValidation::Empty);
    }

    #[test]
    fn 空パスは検証エラーになる() {
        let mut registry = WorkspaceRegistry::new();
        registry.open_add_dialog();

        let validation = registry.update_dialog_path("");

        assert_eq!(validation, WorkspacePathValidation::Empty);
        assert_eq!(registry.dialog().validation, WorkspacePathValidation::Empty);
    }

    #[test]
    fn 相対パスは検証エラーになる() {
        let mut registry = WorkspaceRegistry::new();
        registry.open_add_dialog();

        let validation = registry.update_dialog_path("project/src");

        assert_eq!(validation, WorkspacePathValidation::MustBeAbsolute);
        assert_eq!(
            registry.dialog().validation,
            WorkspacePathValidation::MustBeAbsolute
        );
    }

    #[test]
    fn 有効な絶対パスは追加されworkspace_railに反映される() {
        let test_dir = TestDir::new("workspace-alpha");
        let canonical = std::fs::canonicalize(test_dir.path_str())
            .expect("テスト用ディレクトリは正規化できる必要がある")
            .to_string_lossy()
            .into_owned();
        let mut registry = WorkspaceRegistry::new();
        registry.open_add_dialog();
        let validation = registry.update_dialog_path(test_dir.path_str());
        assert_eq!(validation, WorkspacePathValidation::Valid);

        let result = registry.submit_add();

        assert_eq!(
            result,
            AddWorkspaceOutcome::Added {
                workspace_id: "workspace-1".to_string(),
            }
        );
        assert_eq!(registry.workspaces().len(), 1);
        let metadata = registry.workspaces()[0].metadata();
        assert_eq!(metadata.workspace_id, "workspace-1");
        assert_eq!(metadata.root_path, canonical);
        assert!(metadata.display_name.starts_with("workspace-alpha-"));
        assert_eq!(
            registry.workspaces()[0].snapshot().state,
            WorkspaceRailState::Idle
        );
        assert!(!registry.dialog().is_open);
        assert_eq!(registry.dialog().input_path, "");
        assert_eq!(registry.dialog().validation, WorkspacePathValidation::Empty);
    }

    #[test]
    fn 同一パスは重複として拒否される() {
        let test_dir = TestDir::new("workspace-duplicate");
        let mut registry = WorkspaceRegistry::new();
        registry.open_add_dialog();
        registry.update_dialog_path(test_dir.path_str());
        assert!(matches!(
            registry.submit_add(),
            AddWorkspaceOutcome::Added { .. }
        ));

        registry.open_add_dialog();
        let validation = registry.update_dialog_path(test_dir.path_str());

        assert_eq!(validation, WorkspacePathValidation::Duplicate);
        assert_eq!(
            registry.submit_add(),
            AddWorkspaceOutcome::ValidationFailed {
                reason: WorkspacePathValidation::Duplicate,
            }
        );
        assert_eq!(registry.workspaces().len(), 1);
        assert!(registry.dialog().is_open);
    }

    #[test]
    fn 実在しない絶対パスは検証エラーになる() {
        let missing_path = std::env::temp_dir().join("nue-editor-missing-workspace-path");
        if missing_path.exists() {
            std::fs::remove_dir_all(&missing_path).expect("既存ディレクトリを削除できる必要がある");
        }

        let mut registry = WorkspaceRegistry::new();
        registry.open_add_dialog();
        let validation = registry.update_dialog_path(
            missing_path
                .to_str()
                .expect("テストパスはUTF-8である必要がある"),
        );

        assert_eq!(validation, WorkspacePathValidation::NotFoundOrInaccessible);
        assert_eq!(
            registry.submit_add(),
            AddWorkspaceOutcome::ValidationFailed {
                reason: WorkspacePathValidation::NotFoundOrInaccessible,
            }
        );
        assert_eq!(registry.workspaces().len(), 0);
    }

    #[test]
    fn 正規化後に同一実体となるパスは重複として拒否される() {
        let test_dir = TestDir::new("workspace-equivalent");
        let canonical = std::fs::canonicalize(test_dir.path_str())
            .expect("テスト用ディレクトリは正規化できる必要がある");
        let equivalent = canonical.join(".").join("subdir").join("..");
        std::fs::create_dir_all(canonical.join("subdir"))
            .expect("比較用サブディレクトリを作成できる必要がある");

        let mut registry = WorkspaceRegistry::new();
        registry.open_add_dialog();
        registry.update_dialog_path(test_dir.path_str());
        assert!(matches!(
            registry.submit_add(),
            AddWorkspaceOutcome::Added { .. }
        ));

        registry.open_add_dialog();
        let validation = registry.update_dialog_path(
            equivalent
                .to_str()
                .expect("テストパスはUTF-8である必要がある"),
        );

        assert_eq!(validation, WorkspacePathValidation::Duplicate);
        assert_eq!(registry.workspaces().len(), 1);
    }

    #[test]
    fn 既存workspaceを右クリックすると除外メニューが開く() {
        let first = TestDir::new("workspace-first");
        let second = TestDir::new("workspace-second");
        let mut registry = WorkspaceRegistry::new();
        registry.open_add_dialog();
        registry.update_dialog_path(first.path_str());
        assert!(matches!(
            registry.submit_add(),
            AddWorkspaceOutcome::Added { .. }
        ));
        registry.open_add_dialog();
        registry.update_dialog_path(second.path_str());
        assert!(matches!(
            registry.submit_add(),
            AddWorkspaceOutcome::Added { .. }
        ));

        let opened = registry.open_exclude_context_menu("workspace-2");

        assert_eq!(
            opened,
            OpenWorkspaceContextMenuOutcome::Opened {
                workspace_id: "workspace-2".to_string(),
            }
        );
        assert_eq!(
            registry.context_menu(),
            &WorkspaceContextMenu {
                is_open: true,
                target_workspace_id: Some("workspace-2".to_string()),
            }
        );
    }

    #[test]
    fn 右クリック対象が存在しない場合は除外メニューを開けない() {
        let mut registry = WorkspaceRegistry::new();

        let opened = registry.open_exclude_context_menu("workspace-99");

        assert_eq!(opened, OpenWorkspaceContextMenuOutcome::WorkspaceNotFound);
        assert_eq!(registry.context_menu(), &WorkspaceContextMenu::default());
    }

    #[test]
    fn 除外メニューからworkspaceを除外すると一覧が更新される() {
        let first = TestDir::new("workspace-first");
        let second = TestDir::new("workspace-second");
        let mut registry = WorkspaceRegistry::new();
        registry.open_add_dialog();
        registry.update_dialog_path(first.path_str());
        assert!(matches!(
            registry.submit_add(),
            AddWorkspaceOutcome::Added { .. }
        ));
        registry.open_add_dialog();
        registry.update_dialog_path(second.path_str());
        assert!(matches!(
            registry.submit_add(),
            AddWorkspaceOutcome::Added { .. }
        ));
        assert!(matches!(
            registry.open_exclude_context_menu("workspace-1"),
            OpenWorkspaceContextMenuOutcome::Opened { .. }
        ));

        let removed = registry.exclude_context_menu_target();

        assert_eq!(
            removed,
            ExcludeWorkspaceOutcome::Excluded {
                workspace_id: "workspace-1".to_string(),
            }
        );
        assert_eq!(registry.workspaces().len(), 1);
        assert_eq!(
            registry.workspaces()[0].metadata().workspace_id,
            "workspace-2".to_string()
        );
        assert_eq!(registry.context_menu(), &WorkspaceContextMenu::default());
    }

    #[test]
    fn 除外メニューが閉じている時は除外操作できない() {
        let mut registry = WorkspaceRegistry::new();

        let removed = registry.exclude_context_menu_target();

        assert_eq!(removed, ExcludeWorkspaceOutcome::ContextMenuClosed);
    }

    #[test]
    fn 除外後に再追加してもworkspace_idは重複しない() {
        let first = TestDir::new("workspace-first");
        let second = TestDir::new("workspace-second");
        let third = TestDir::new("workspace-third");
        let mut registry = WorkspaceRegistry::new();
        registry.open_add_dialog();
        registry.update_dialog_path(first.path_str());
        assert!(matches!(
            registry.submit_add(),
            AddWorkspaceOutcome::Added { workspace_id } if workspace_id == "workspace-1"
        ));
        registry.open_add_dialog();
        registry.update_dialog_path(second.path_str());
        assert!(matches!(
            registry.submit_add(),
            AddWorkspaceOutcome::Added { workspace_id } if workspace_id == "workspace-2"
        ));
        assert!(matches!(
            registry.open_exclude_context_menu("workspace-1"),
            OpenWorkspaceContextMenuOutcome::Opened { .. }
        ));
        assert!(matches!(
            registry.exclude_context_menu_target(),
            ExcludeWorkspaceOutcome::Excluded { .. }
        ));

        registry.open_add_dialog();
        registry.update_dialog_path(third.path_str());

        assert!(matches!(
            registry.submit_add(),
            AddWorkspaceOutcome::Added { workspace_id } if workspace_id == "workspace-3"
        ));
    }

    #[test]
    fn 基盤サーバ状態更新でworkspace_railのライブ更新イベントを取り出せる() {
        let first = TestDir::new("workspace-first");
        let mut registry = WorkspaceRegistry::new();
        registry.open_add_dialog();
        registry.update_dialog_path(first.path_str());
        assert!(matches!(
            registry.submit_add(),
            AddWorkspaceOutcome::Added { workspace_id } if workspace_id == "workspace-1"
        ));

        let outcome =
            registry.update_workspace_status_from_server("workspace-1", WorkspaceRailState::Busy);

        assert_eq!(
            outcome,
            WorkspaceStatusUpdateOutcome::Updated(WorkspaceStatusUpdatedEvent {
                workspace_id: "workspace-1".to_string(),
                state: WorkspaceRailState::Busy,
                revision: 1,
            })
        );
        assert_eq!(
            registry.workspaces()[0].snapshot().state,
            WorkspaceRailState::Busy
        );
        assert_eq!(
            registry.drain_status_events(),
            vec![WorkspaceStatusUpdatedEvent {
                workspace_id: "workspace-1".to_string(),
                state: WorkspaceRailState::Busy,
                revision: 1,
            }]
        );
        assert!(registry.drain_status_events().is_empty());
    }

    #[test]
    fn 基盤サーバ状態更新で同一状態の場合はnoopになる() {
        let first = TestDir::new("workspace-first");
        let mut registry = WorkspaceRegistry::new();
        registry.open_add_dialog();
        registry.update_dialog_path(first.path_str());
        assert!(matches!(
            registry.submit_add(),
            AddWorkspaceOutcome::Added { workspace_id } if workspace_id == "workspace-1"
        ));

        let outcome =
            registry.update_workspace_status_from_server("workspace-1", WorkspaceRailState::Idle);

        assert_eq!(outcome, WorkspaceStatusUpdateOutcome::Noop);
        assert!(registry.drain_status_events().is_empty());
    }

    #[test]
    fn 基盤サーバ状態更新は未登録workspaceを拒否する() {
        let mut registry = WorkspaceRegistry::new();

        let outcome = registry
            .update_workspace_status_from_server("workspace-unknown", WorkspaceRailState::Error);

        assert_eq!(outcome, WorkspaceStatusUpdateOutcome::WorkspaceNotFound);
        assert!(registry.drain_status_events().is_empty());
    }
}
