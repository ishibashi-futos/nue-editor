use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyFileTreeNode {
    Directory(LegacyDirectoryNode),
    File(LegacyFileNode),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyDirectoryNode {
    pub name: String,
    pub absolute_path: String,
    pub children: Vec<LegacyFileTreeNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyFileNode {
    pub name: String,
    pub absolute_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyFileTree {
    root: LegacyDirectoryNode,
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

    pub fn root(&self) -> &LegacyDirectoryNode {
        &self.root
    }
}

fn build_directory_node(path: &Path) -> Result<LegacyDirectoryNode, LegacyFileTreeBuildError> {
    let name = file_name_from_path(path).unwrap_or_else(|| path.to_string_lossy().to_string());
    let absolute_path = path.to_string_lossy().to_string();
    let mut children = Vec::new();

    for entry_result in fs::read_dir(path).map_err(map_io_error)? {
        let entry = entry_result.map_err(map_io_error)?;
        let entry_path = entry.path();
        let file_type = entry.file_type().map_err(map_io_error)?;
        if file_type.is_dir() {
            children.push(LegacyFileTreeNode::Directory(build_directory_node(
                entry_path.as_path(),
            )?));
            continue;
        }
        if file_type.is_file() {
            let name = file_name_from_path(entry_path.as_path())
                .unwrap_or_else(|| entry_path.to_string_lossy().to_string());
            children.push(LegacyFileTreeNode::File(LegacyFileNode {
                name,
                absolute_path: entry_path.to_string_lossy().to_string(),
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
        .map(|name| name.to_string_lossy().to_string())
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

#[cfg(test)]
mod tests {
    use super::{LegacyFileTree, LegacyFileTreeBuildError, LegacyFileTreeNode};
    use std::fs;
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
            PathBuf::from(&root.absolute_path),
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

    struct TestDir {
        path: PathBuf,
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

            Self { path }
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
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            if self.path.exists() {
                fs::remove_dir_all(&self.path).expect("remove temp dir");
            }
        }
    }
}
