use nue_core::git_status::GitFileStatus;
use nue_core::legacy_file_tree::{LegacyFileTreeNodeKind, LegacyFileTreeNodeStatus};
use nue_core::legacy_workspace_editor::{LegacyWorkspaceEditor, SelectFileOutcome};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyExplorerNode {
    status: LegacyFileTreeNodeStatus,
    is_selected: bool,
}

impl LegacyExplorerNode {
    pub fn status(&self) -> &LegacyFileTreeNodeStatus {
        &self.status
    }

    pub fn path(&self) -> &Path {
        &self.status.path
    }

    pub fn name(&self) -> &str {
        &self.status.name
    }

    pub fn kind(&self) -> LegacyFileTreeNodeKind {
        self.status.kind
    }

    pub fn depth(&self) -> usize {
        self.status.depth
    }

    pub fn git_status(&self) -> Option<GitFileStatus> {
        self.status.git_status
    }

    pub fn is_selected(&self) -> bool {
        self.is_selected
    }

    pub fn is_directory(&self) -> bool {
        self.status.kind == LegacyFileTreeNodeKind::Directory
    }
}

#[derive(Debug)]
pub struct LegacyExplorerModel {
    nodes: Vec<LegacyExplorerNode>,
    selected_path: Option<PathBuf>,
}

impl LegacyExplorerModel {
    pub fn new(editor: &LegacyWorkspaceEditor) -> Self {
        let mut model = Self {
            nodes: Vec::new(),
            selected_path: None,
        };
        model.refresh(editor);
        model
    }

    pub fn refresh(&mut self, editor: &LegacyWorkspaceEditor) {
        let statuses = editor.file_tree_with_git_statuses();
        self.nodes = statuses
            .into_iter()
            .map(|status| LegacyExplorerNode {
                is_selected: self.selected_path.as_ref() == Some(&status.path),
                status,
            })
            .collect();
    }

    pub fn nodes(&self) -> &[LegacyExplorerNode] {
        &self.nodes
    }

    pub fn selected_path(&self) -> Option<&Path> {
        self.selected_path.as_deref()
    }

    pub fn select_file(
        &mut self,
        editor: &mut LegacyWorkspaceEditor,
        absolute_path: &str,
    ) -> SelectFileOutcome {
        let outcome = editor.select_file(absolute_path);
        if let SelectFileOutcome::Selected(snapshot) = &outcome {
            self.selected_path = Some(snapshot.file_path.clone());
        }
        self.refresh(editor);
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nue_core::legacy_workspace_editor::LegacyWorkspaceEditor;
    use std::fs;
    use tempfile::TempDir;

    fn setup_workspace() -> (TempDir, LegacyWorkspaceEditor) {
        let temp_dir = TempDir::new().expect("一時ディレクトリが作れない");
        fs::create_dir_all(temp_dir.path().join("src/nested")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"legacy\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(temp_dir.path().join("README.md"), "Legacy Explorer\n").unwrap();
        fs::write(
            temp_dir.path().join("src/main.rs"),
            "fn main() { println!(\"hello\"); }\n",
        )
        .unwrap();
        let editor =
            LegacyWorkspaceEditor::open(temp_dir.path().to_str().unwrap()).expect("editor open");
        (temp_dir, editor)
    }

    #[test]
    fn refreshでノードが構築される() {
        let (temp_dir, editor) = setup_workspace();
        let explorer = LegacyExplorerModel::new(&editor);

        assert!(explorer.nodes().len() >= 4);
        assert_eq!(explorer.nodes()[0].depth(), 0);
        assert!(explorer.nodes()[0].is_directory());
        assert_eq!(explorer.nodes()[1].depth(), 1);
        assert!(explorer.nodes()[2].depth() >= 1);
        assert!(explorer.selected_path().is_none());
        drop(temp_dir);
    }

    #[test]
    fn select_fileで選択状態が反映される() {
        let (temp_dir, mut editor) = setup_workspace();
        let mut explorer = LegacyExplorerModel::new(&editor);
        let target = temp_dir.path().join("README.md");
        let canonical = target.canonicalize().unwrap();

        let outcome = explorer.select_file(&mut editor, canonical.to_str().unwrap());

        match outcome {
            SelectFileOutcome::Selected(snapshot) => {
                assert_eq!(snapshot.file_path, canonical);
            }
            _ => panic!("ファイル選択が失敗しました"),
        }

        assert_eq!(explorer.selected_path(), Some(canonical.as_path()));
        assert!(
            explorer
                .nodes()
                .iter()
                .any(|node| node.is_selected() && node.path() == canonical.as_path())
        );
        drop(temp_dir);
    }
}
