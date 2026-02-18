use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::workspace::git_status::GitFileStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyFileTreeNode {
    Directory(LegacyDirectoryNode),
    File(LegacyFileNode),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyDirectoryNode {
    pub name: String,
    pub absolute_path: PathBuf,
    pub children: Vec<LegacyFileTreeNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyFileNode {
    pub name: String,
    pub absolute_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyFileTree {
    root: LegacyDirectoryNode,
}

/// フラット化されたビューでのノード種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyFileTreeNodeKind {
    Directory,
    File,
}

/// Git 状態付きでノード情報を表現する構造体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyFileTreeNodeStatus {
    pub path: PathBuf,
    pub name: String,
    pub kind: LegacyFileTreeNodeKind,
    pub git_status: Option<GitFileStatus>,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyFileTreeBuildError {
    RootNotFound,
    RootIsNotDirectory,
    Io,
}

impl LegacyFileTree {
    pub fn build(root_path: &str) -> Result<Self, LegacyFileTreeBuildError> {
        let path = fs::canonicalize(PathBuf::from(root_path)).map_err(map_io_error)?;
        if !path.is_dir() {
            return Err(LegacyFileTreeBuildError::RootIsNotDirectory);
        }
        let root = build_directory_node(path.as_path())?;
        Ok(Self { root })
    }

    /// Git ステータスと併せて各ノードを先行順で出力する。
    pub fn flatten_with_git_statuses(
        &self,
        statuses: &HashMap<PathBuf, GitFileStatus>,
    ) -> Vec<LegacyFileTreeNodeStatus> {
        let mut flattened = Vec::new();
        traverse_with_status(&self.root, 0, statuses, &mut flattened);
        flattened
    }

    pub fn root(&self) -> &LegacyDirectoryNode {
        &self.root
    }
}

fn build_directory_node(path: &Path) -> Result<LegacyDirectoryNode, LegacyFileTreeBuildError> {
    let name = file_name_from_path(path).unwrap_or_else(|| path.display().to_string());
    let absolute_path = path.to_path_buf();
    let mut children = Vec::new();

    for entry_result in fs::read_dir(path).map_err(map_io_error)? {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let entry_path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };
        if file_type.is_dir() {
            if let Ok(directory) = build_directory_node(entry_path.as_path()) {
                children.push(LegacyFileTreeNode::Directory(directory));
            }
            continue;
        }
        if file_type.is_file() {
            let name = file_name_from_path(entry_path.as_path())
                .unwrap_or_else(|| entry_path.display().to_string());
            children.push(LegacyFileTreeNode::File(LegacyFileNode {
                name,
                absolute_path: entry_path,
            }));
        }
    }

    children.sort_by(compare_tree_node);
    Ok(LegacyDirectoryNode {
        name,
        absolute_path,
        children,
    })
}

fn file_name_from_path(path: &Path) -> Option<String> {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
}

fn compare_tree_node(left: &LegacyFileTreeNode, right: &LegacyFileTreeNode) -> std::cmp::Ordering {
    match (left, right) {
        (LegacyFileTreeNode::Directory(left), LegacyFileTreeNode::Directory(right)) => {
            left.name.cmp(&right.name)
        }
        (LegacyFileTreeNode::Directory(_), LegacyFileTreeNode::File(_)) => std::cmp::Ordering::Less,
        (LegacyFileTreeNode::File(_), LegacyFileTreeNode::Directory(_)) => {
            std::cmp::Ordering::Greater
        }
        (LegacyFileTreeNode::File(left), LegacyFileTreeNode::File(right)) => {
            left.name.cmp(&right.name)
        }
    }
}

fn map_io_error(error: std::io::Error) -> LegacyFileTreeBuildError {
    if error.kind() == std::io::ErrorKind::NotFound {
        return LegacyFileTreeBuildError::RootNotFound;
    }
    LegacyFileTreeBuildError::Io
}

fn traverse_with_status(
    node: &LegacyDirectoryNode,
    depth: usize,
    statuses: &HashMap<PathBuf, GitFileStatus>,
    flattened: &mut Vec<LegacyFileTreeNodeStatus>,
) -> Option<GitFileStatus> {
    let entry_index = flattened.len();
    flattened.push(LegacyFileTreeNodeStatus {
        path: node.absolute_path.clone(),
        name: node.name.clone(),
        kind: LegacyFileTreeNodeKind::Directory,
        git_status: None,
        depth,
    });

    let mut aggregated_status = statuses.get(&node.absolute_path).copied();

    for child in &node.children {
        match child {
            LegacyFileTreeNode::Directory(child_dir) => {
                let child_status = traverse_with_status(child_dir, depth + 1, statuses, flattened);
                aggregated_status = merge_status_option(aggregated_status, child_status);
            }
            LegacyFileTreeNode::File(file) => {
                let file_status = statuses.get(&file.absolute_path).copied();
                aggregated_status = merge_status_option(aggregated_status, file_status);
                flattened.push(LegacyFileTreeNodeStatus {
                    path: file.absolute_path.clone(),
                    name: file.name.clone(),
                    kind: LegacyFileTreeNodeKind::File,
                    git_status: file_status,
                    depth: depth + 1,
                });
            }
        }
    }

    if let Some(status) = aggregated_status {
        flattened[entry_index].git_status = Some(status);
    }

    aggregated_status
}

fn merge_status_option(
    left: Option<GitFileStatus>,
    right: Option<GitFileStatus>,
) -> Option<GitFileStatus> {
    match (left, right) {
        (None, None) => None,
        (Some(status), None) | (None, Some(status)) => Some(status),
        (Some(left_status), Some(right_status)) => Some(left_status.merge(right_status)),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LegacyFileTree, LegacyFileTreeBuildError, LegacyFileTreeNode, LegacyFileTreeNodeKind,
    };
    use crate::workspace::git_status::GitFileStatus;
    use std::collections::HashMap;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn ルート配下のファイルとディレクトリをツリーで取得できる() {
        let root_dir = TestDir::new("legacy-tree-root");
        root_dir.create_dir("src/nested");
        root_dir.create_file("Cargo.toml", "[package]\nname = \"demo\"\n");
        root_dir.create_file("src/main.rs", "fn main() {}\n");
        root_dir.create_file("src/nested/lib.rs", "pub fn hello() {}\n");

        let tree = LegacyFileTree::build(root_dir.path().to_str().expect("utf-8 path"))
            .expect("ディレクトリからツリーを構築できるべき");

        let root = tree.root();
        assert_eq!(
            root.absolute_path,
            fs::canonicalize(root_dir.path()).expect("canonicalize root path")
        );
        assert_eq!(root.children.len(), 2);

        match &root.children[0] {
            LegacyFileTreeNode::Directory(src) => {
                assert_eq!(src.name, "src");
                assert_eq!(src.children.len(), 2);
            }
            LegacyFileTreeNode::File(_) => {
                panic!("src がディレクトリとして解決されるべき");
            }
        }

        match &root.children[1] {
            LegacyFileTreeNode::File(cargo) => {
                assert_eq!(cargo.name, "Cargo.toml");
            }
            LegacyFileTreeNode::Directory(_) => {
                panic!("Cargo.toml がファイルとして解決されるべき");
            }
        }
    }

    #[test]
    fn 存在しないルートはエラーになる() {
        let missing = std::env::temp_dir().join("nue-editor-missing-legacy-tree-root");
        if missing.exists() {
            fs::remove_dir_all(&missing).expect("cleanup missing root");
        }

        assert_eq!(
            LegacyFileTree::build(missing.to_str().expect("utf-8 path")),
            Err(LegacyFileTreeBuildError::RootNotFound)
        );
    }

    #[test]
    fn ルートがファイルの場合はエラーになる() {
        let root_dir = TestDir::new("legacy-tree-file-root");
        root_dir.create_file("only-file.txt", "hello");
        let file_path = root_dir.path().join("only-file.txt");

        assert_eq!(
            LegacyFileTree::build(file_path.to_str().expect("utf-8 path")),
            Err(LegacyFileTreeBuildError::RootIsNotDirectory)
        );
    }

    #[cfg(unix)]
    #[test]
    fn 子ディレクトリの読み取りに失敗しても構築を継続する() {
        let mut root_dir = TestDir::new("legacy-tree-partial-io");
        root_dir.create_dir("open");
        root_dir.create_file("open/ok.txt", "ok");
        root_dir.create_dir("blocked");
        root_dir.restrict_dir("blocked");

        let tree = LegacyFileTree::build(root_dir.path().to_str().expect("utf-8 path"))
            .expect("部分的なアクセス不能はスキップされるべき");

        let root = tree.root();
        assert_eq!(root.children.len(), 1);
        match &root.children[0] {
            LegacyFileTreeNode::Directory(directory) => {
                assert_eq!(directory.name, "open");
            }
            LegacyFileTreeNode::File(_) => {
                panic!("open ディレクトリだけが残る想定");
            }
        }
    }

    #[test]
    fn flatten_with_git_statuses_reports_directory_and_file_statuses() {
        let root_dir = TestDir::new("legacy-tree-git-status");
        root_dir.create_dir("src");
        root_dir.create_file("README.md", "root");
        root_dir.create_file("src/lib.rs", "pub fn hi() {}");

        let tree =
            LegacyFileTree::build(root_dir.path().to_str().unwrap()).expect("tree build succeeds");

        let mut statuses = HashMap::new();
        statuses.insert(
            fs::canonicalize(root_dir.path().join("src/lib.rs")).unwrap(),
            GitFileStatus::Untracked,
        );
        statuses.insert(
            fs::canonicalize(root_dir.path().join("README.md")).unwrap(),
            GitFileStatus::Modified,
        );

        let flattened = tree.flatten_with_git_statuses(&statuses);

        assert_eq!(flattened.len(), 4);
        assert_eq!(flattened[0].kind, LegacyFileTreeNodeKind::Directory);
        assert_eq!(flattened[0].depth, 0);
        assert_eq!(flattened[0].git_status, Some(GitFileStatus::Modified));

        assert_eq!(flattened[1].name, "src");
        assert_eq!(flattened[1].kind, LegacyFileTreeNodeKind::Directory);
        assert_eq!(flattened[1].depth, 1);
        assert_eq!(flattened[1].git_status, Some(GitFileStatus::Untracked));

        assert_eq!(flattened[2].name, "lib.rs");
        assert_eq!(flattened[2].kind, LegacyFileTreeNodeKind::File);
        assert_eq!(flattened[2].depth, 2);
        assert_eq!(flattened[2].git_status, Some(GitFileStatus::Untracked));

        assert_eq!(flattened[3].name, "README.md");
        assert_eq!(flattened[3].kind, LegacyFileTreeNodeKind::File);
        assert_eq!(flattened[3].depth, 1);
        assert_eq!(flattened[3].git_status, Some(GitFileStatus::Modified));
    }

    struct TestDir {
        path: PathBuf,
        #[cfg(unix)]
        restricted_dirs: Vec<PathBuf>,
    }

    impl TestDir {
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

            Self {
                path,
                #[cfg(unix)]
                restricted_dirs: Vec::new(),
            }
        }

        fn path(&self) -> &PathBuf {
            &self.path
        }

        fn create_dir(&self, relative_path: &str) {
            fs::create_dir_all(self.path.join(relative_path)).expect("create nested dir");
        }

        fn create_file(&self, relative_path: &str, content: &str) {
            let file_path = self.path.join(relative_path);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).expect("create parent dir");
            }
            fs::write(file_path, content).expect("write test file");
        }

        #[cfg(unix)]
        fn restrict_dir(&mut self, relative_path: &str) {
            let path = self.path.join(relative_path);
            let mut permissions = fs::metadata(&path).expect("read metadata").permissions();
            permissions.set_mode(0o000);
            fs::set_permissions(&path, permissions).expect("set restricted permissions");
            self.restricted_dirs.push(path);
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            #[cfg(unix)]
            for restricted_dir in &self.restricted_dirs {
                if restricted_dir.exists() {
                    let mut permissions = fs::metadata(restricted_dir)
                        .expect("read restricted metadata")
                        .permissions();
                    permissions.set_mode(0o755);
                    fs::set_permissions(restricted_dir, permissions)
                        .expect("restore restricted permissions");
                }
            }
            if self.path.exists() {
                fs::remove_dir_all(&self.path).expect("remove temp dir");
            }
        }
    }
}
