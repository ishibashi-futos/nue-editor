use crate::editor_core::{
    CursorMoveOutcome, EditOutcome, EditorBufferSnapshot, EditorCore, HistoryOutcome,
    MarkSavedOutcome, SaveOutcome, SaveTrigger,
};
use crate::legacy_file_tree::{LegacyFileTree, LegacyFileTreeBuildError};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct LegacyWorkspaceEditor {
    workspace_root: String,
    file_tree: LegacyFileTree,
    editor_core: EditorCore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyWorkspaceEditorOpenError {
    InvalidWorkspaceRoot(LegacyFileTreeBuildError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectFileOutcome {
    Selected(EditorBufferSnapshot),
    FileNotFound,
    NotAFile,
    OutsideWorkspace,
    Io,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveFileOutcome {
    Saved { file_path: String, revision: u64 },
    NoBuffer,
    NotDirty,
    Io,
}

impl LegacyWorkspaceEditor {
    pub fn open(workspace_root: &str) -> Result<Self, LegacyWorkspaceEditorOpenError> {
        let file_tree = LegacyFileTree::build(workspace_root)
            .map_err(LegacyWorkspaceEditorOpenError::InvalidWorkspaceRoot)?;
        let canonical_root = fs::canonicalize(PathBuf::from(workspace_root)).map_err(|_| {
            LegacyWorkspaceEditorOpenError::InvalidWorkspaceRoot(LegacyFileTreeBuildError::Io)
        })?;

        Ok(Self {
            workspace_root: canonical_root.to_string_lossy().to_string(),
            file_tree,
            editor_core: EditorCore::new(),
        })
    }

    pub fn file_tree(&self) -> &LegacyFileTree {
        &self.file_tree
    }

    pub fn select_file(&mut self, absolute_path: &str) -> SelectFileOutcome {
        let canonical_path = match fs::canonicalize(PathBuf::from(absolute_path)) {
            Ok(path) => path,
            Err(error) => {
                if error.kind() == std::io::ErrorKind::NotFound {
                    return SelectFileOutcome::FileNotFound;
                }
                return SelectFileOutcome::Io;
            }
        };
        if !self.is_inside_workspace(canonical_path.as_path()) {
            return SelectFileOutcome::OutsideWorkspace;
        }
        let metadata = match fs::metadata(canonical_path.as_path()) {
            Ok(metadata) => metadata,
            Err(_) => return SelectFileOutcome::Io,
        };
        if !metadata.is_file() {
            return SelectFileOutcome::NotAFile;
        }

        let content = match fs::read_to_string(canonical_path.as_path()) {
            Ok(content) => content,
            Err(_) => return SelectFileOutcome::Io,
        };
        SelectFileOutcome::Selected(
            self.editor_core
                .open_file(canonical_path.to_string_lossy().to_string(), content),
        )
    }

    pub fn set_cursor(&mut self, cursor_char: usize) -> CursorMoveOutcome {
        self.editor_core.set_cursor(cursor_char)
    }

    pub fn insert_text(&mut self, text: &str) -> EditOutcome {
        self.editor_core.insert_text(text)
    }

    pub fn undo(&mut self) -> HistoryOutcome {
        self.editor_core.undo()
    }

    pub fn redo(&mut self) -> HistoryOutcome {
        self.editor_core.redo()
    }

    pub fn save_active_file(&mut self) -> SaveFileOutcome {
        let save_request = match self.editor_core.request_save(SaveTrigger::Manual) {
            SaveOutcome::NoBuffer => return SaveFileOutcome::NoBuffer,
            SaveOutcome::NotDirty => return SaveFileOutcome::NotDirty,
            SaveOutcome::Requested(request) => request,
        };

        if fs::write(
            save_request.file_path.as_str(),
            save_request.content.as_bytes(),
        )
        .is_err()
        {
            return SaveFileOutcome::Io;
        }
        match self.editor_core.mark_saved(save_request.revision) {
            MarkSavedOutcome::Saved { revision } => SaveFileOutcome::Saved {
                file_path: save_request.file_path,
                revision,
            },
            MarkSavedOutcome::NoBuffer | MarkSavedOutcome::StaleRevision { .. } => {
                SaveFileOutcome::Io
            }
        }
    }

    pub fn snapshot(&self) -> Option<EditorBufferSnapshot> {
        self.editor_core.snapshot()
    }

    fn is_inside_workspace(&self, file_path: &Path) -> bool {
        file_path.starts_with(Path::new(self.workspace_root.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor_core::{CursorMoveOutcome, EditOutcome, HistoryOutcome};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn ファイル選択でeditor_coreにバッファを開ける() {
        let fixture = WorkspaceFixture::new("legacy-workspace-editor-open");
        let file_path = fixture.write_file("notes/today.md", "Hello");
        let mut workspace_editor =
            LegacyWorkspaceEditor::open(fixture.path_str()).expect("workspaceを開けるべき");

        let outcome = workspace_editor.select_file(file_path.to_str().expect("utf-8 path"));
        let canonical_file_path = fs::canonicalize(&file_path)
            .expect("比較用にテストファイルパスを正規化できる必要がある");

        let SelectFileOutcome::Selected(snapshot) = outcome else {
            panic!("ファイル選択が成功する想定");
        };
        assert_eq!(
            snapshot.file_path,
            canonical_file_path.to_string_lossy().to_string()
        );
        assert_eq!(snapshot.content, "Hello");
        assert_eq!(snapshot.cursor_char, 0);
        assert_eq!(snapshot.revision, 0);
        assert!(!snapshot.is_dirty);
    }

    #[test]
    fn 選択後に編集保存undo_redoを実行できる() {
        let fixture = WorkspaceFixture::new("legacy-workspace-editor-edit");
        let file_path = fixture.write_file("notes/today.md", "Hello");
        let mut workspace_editor =
            LegacyWorkspaceEditor::open(fixture.path_str()).expect("workspaceを開けるべき");

        let select = workspace_editor.select_file(file_path.to_str().expect("utf-8 path"));
        assert!(matches!(select, SelectFileOutcome::Selected(_)));

        assert_eq!(
            workspace_editor.set_cursor(5),
            CursorMoveOutcome::Moved { cursor_char: 5 }
        );
        assert_eq!(
            workspace_editor.insert_text(" world"),
            EditOutcome::Edited {
                revision: 1,
                cursor_char: 11,
                is_dirty: true,
            }
        );

        assert_eq!(
            workspace_editor.save_active_file(),
            SaveFileOutcome::Saved {
                file_path: fs::canonicalize(&file_path)
                    .expect("比較用にテストファイルパスを正規化できる必要がある")
                    .to_string_lossy()
                    .to_string(),
                revision: 1,
            }
        );
        assert_eq!(
            fs::read_to_string(&file_path).expect("保存結果を読めるべき"),
            "Hello world"
        );

        assert_eq!(
            workspace_editor.undo(),
            HistoryOutcome::Applied {
                revision: 2,
                cursor_char: 5,
                is_dirty: true,
            }
        );
        assert_eq!(
            workspace_editor
                .snapshot()
                .expect("アクティブバッファがある")
                .content,
            "Hello"
        );

        assert_eq!(
            workspace_editor.redo(),
            HistoryOutcome::Applied {
                revision: 3,
                cursor_char: 11,
                is_dirty: false,
            }
        );
        assert_eq!(
            workspace_editor
                .snapshot()
                .expect("アクティブバッファがある")
                .content,
            "Hello world"
        );
    }

    #[test]
    fn ワークスペース外ファイルは選択できない() {
        let fixture = WorkspaceFixture::new("legacy-workspace-editor-outside");
        let outside_fixture = WorkspaceFixture::new("legacy-workspace-editor-outside-target");
        let outside_file = outside_fixture.write_file("outside.md", "blocked");
        let mut workspace_editor =
            LegacyWorkspaceEditor::open(fixture.path_str()).expect("workspaceを開けるべき");

        assert_eq!(
            workspace_editor.select_file(outside_file.to_str().expect("utf-8 path")),
            SelectFileOutcome::OutsideWorkspace
        );
    }

    struct WorkspaceFixture {
        path: PathBuf,
    }

    impl WorkspaceFixture {
        fn new(prefix: &str) -> Self {
            static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

            let mut path = std::env::temp_dir();
            let sequence = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time after unix epoch")
                .as_nanos();
            path.push(format!("nue-editor-{prefix}-{sequence}-{timestamp}"));

            fs::create_dir_all(&path).expect("create temp dir");

            Self { path }
        }

        fn path_str(&self) -> &str {
            self.path.to_str().expect("utf-8 path")
        }

        fn write_file(&self, relative_path: &str, content: &str) -> PathBuf {
            let file_path = self.path.join(relative_path);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).expect("create parent dir");
            }
            fs::write(&file_path, content).expect("write test file");
            file_path
        }
    }

    impl Drop for WorkspaceFixture {
        fn drop(&mut self) {
            if self.path.exists() {
                fs::remove_dir_all(&self.path).expect("remove temp dir");
            }
        }
    }
}
