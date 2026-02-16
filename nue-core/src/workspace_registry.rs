use crate::workspace_rail::{WorkspaceMetadata, WorkspaceRailModel};
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

#[derive(Debug, Default)]
pub struct WorkspaceRegistry {
    workspaces: Vec<WorkspaceRailModel>,
    dialog: WorkspaceRegistrationDialog,
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

    pub fn open_add_dialog(&mut self) {
        self.dialog.is_open = true;
        self.dialog.input_path.clear();
        self.dialog.validation = WorkspacePathValidation::Empty;
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
        let workspace_id = format!("workspace-{}", self.workspaces.len() + 1);
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
}
