use crate::workspace_rail::{WorkspaceMetadata, WorkspaceRailModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspacePathValidation {
    Empty,
    MustBeAbsolute,
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
        let normalized_path = normalize_path(self.dialog.input_path.as_str());
        let validation = self.validate_path(normalized_path.as_str());
        if validation != WorkspacePathValidation::Valid {
            self.dialog.validation = validation.clone();
            return AddWorkspaceOutcome::ValidationFailed { reason: validation };
        }

        let workspace_id = format!("workspace-{}", self.workspaces.len() + 1);
        let display_name = workspace_display_name(normalized_path.as_str());
        let metadata = WorkspaceMetadata::new(
            workspace_id.clone(),
            normalized_path.as_str(),
            display_name.as_str(),
        );
        self.workspaces.push(WorkspaceRailModel::new(metadata));
        self.dialog.is_open = false;
        self.dialog.input_path.clear();
        self.dialog.validation = WorkspacePathValidation::Empty;

        AddWorkspaceOutcome::Added { workspace_id }
    }

    fn validate_path(&self, path: &str) -> WorkspacePathValidation {
        let normalized_path = normalize_path(path);
        if normalized_path.is_empty() {
            return WorkspacePathValidation::Empty;
        }
        if !normalized_path.starts_with('/') {
            return WorkspacePathValidation::MustBeAbsolute;
        }
        if self
            .workspaces
            .iter()
            .any(|workspace| workspace.metadata().root_path == normalized_path)
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
        let mut registry = WorkspaceRegistry::new();
        registry.open_add_dialog();
        let validation = registry.update_dialog_path("/tmp/workspace-alpha");
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
        assert_eq!(metadata.root_path, "/tmp/workspace-alpha");
        assert_eq!(metadata.display_name, "workspace-alpha");
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
        let mut registry = WorkspaceRegistry::new();
        registry.open_add_dialog();
        registry.update_dialog_path("/tmp/workspace-alpha");
        assert!(matches!(
            registry.submit_add(),
            AddWorkspaceOutcome::Added { .. }
        ));

        registry.open_add_dialog();
        let validation = registry.update_dialog_path("/tmp/workspace-alpha");

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
}
