use crate::workspace::git_status::{GitChangeKind, collect_git_status_entries};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const MAX_REPOSITORY_SEARCH_DEPTH: usize = 5;

/// リポジトリ変更の種別とパスを表現する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryChange {
    pub path: PathBuf,
    pub kind: GitChangeKind,
}

/// ステージ済み/作業ツリー上の変更を分けて保持する。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RepositoryChangeSet {
    pub staged: Vec<RepositoryChange>,
    pub unstaged: Vec<RepositoryChange>,
}

impl RepositoryChangeSet {
    pub fn total(&self) -> usize {
        self.staged.len() + self.unstaged.len()
    }
}

/// 検出済みリポジトリのスナップショット。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositorySnapshot {
    pub id: String,
    pub root: PathBuf,
    pub relative_root: PathBuf,
    pub change_set: RepositoryChangeSet,
}

/// ワークスペース内のリポジトリ検出と変更一覧取得を担うサービス。
pub struct RepositoryService {
    workspace_root: PathBuf,
    repository_roots: Vec<PathBuf>,
    snapshots: Vec<RepositorySnapshot>,
}

impl RepositoryService {
    /// ワークスペースルートを指定してサービスを初期化する。
    pub fn new(workspace_root: impl Into<PathBuf>) -> io::Result<Self> {
        let workspace_root = workspace_root.into();
        let canonical_root = fs::canonicalize(&workspace_root)?;
        Ok(Self {
            workspace_root: canonical_root,
            repository_roots: Vec::new(),
            snapshots: Vec::new(),
        })
    }

    /// 現在のワークスペース内を走査して `.git` を持つリポジトリを検出し、スナップショットを更新する。
    pub fn detect_repositories(&mut self) -> io::Result<&[RepositorySnapshot]> {
        self.repository_roots =
            find_repositories(&self.workspace_root, MAX_REPOSITORY_SEARCH_DEPTH)?;
        self.repository_roots.sort();
        self.repository_roots.dedup();
        self.refresh();
        Ok(&self.snapshots)
    }

    /// 最後に検出されたリポジトリのスナップショット一覧。
    pub fn snapshots(&self) -> &[RepositorySnapshot] {
        &self.snapshots
    }

    /// 指定されたリポジトリ ID に対応するスナップショットを取得する。
    pub fn snapshot_by_id(&self, id: &str) -> Option<&RepositorySnapshot> {
        self.snapshots.iter().find(|snapshot| snapshot.id == id)
    }

    /// 現在検出済みのリポジトリに対して変更一覧を再取得する。
    pub fn refresh(&mut self) {
        self.snapshots = self
            .repository_roots
            .iter()
            .map(|root| self.build_snapshot(root))
            .collect();
    }

    /// ワークスペースルートのパスを取得する。
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    fn build_snapshot(&self, root: &Path) -> RepositorySnapshot {
        let change_set = collect_change_set(root);
        let relative_root = root
            .strip_prefix(&self.workspace_root)
            .map(PathBuf::from)
            .unwrap_or_else(|_| root.to_path_buf());
        let relative_root = if relative_root.as_os_str().is_empty() {
            PathBuf::from(".")
        } else {
            relative_root
        };
        let id = if relative_root.as_os_str().is_empty() {
            ".".to_string()
        } else {
            relative_root.to_string_lossy().to_string()
        };

        RepositorySnapshot {
            id,
            root: root.to_path_buf(),
            relative_root,
            change_set,
        }
    }
}

fn collect_change_set(repo_root: &Path) -> RepositoryChangeSet {
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();

    for entry in collect_git_status_entries(repo_root) {
        let relative_path = entry
            .path
            .strip_prefix(repo_root)
            .map(PathBuf::from)
            .unwrap_or_else(|_| entry.path.clone());
        if let Some(kind) = entry.index_status {
            staged.push(RepositoryChange {
                path: relative_path.clone(),
                kind,
            });
        }
        if let Some(kind) = entry.worktree_status {
            unstaged.push(RepositoryChange {
                path: relative_path.clone(),
                kind,
            });
        }
    }

    staged.sort_by(|a, b| a.path.cmp(&b.path));
    unstaged.sort_by(|a, b| a.path.cmp(&b.path));

    RepositoryChangeSet { staged, unstaged }
}

fn find_repositories(root: &Path, depth: usize) -> io::Result<Vec<PathBuf>> {
    let mut repositories = Vec::new();

    fn visit(path: &Path, depth: usize, repositories: &mut Vec<PathBuf>) -> io::Result<()> {
        if !path.is_dir() {
            return Ok(());
        }
        let git_marker = path.join(".git");
        if git_marker.exists()
            && let Ok(canonical) = fs::canonicalize(path)
        {
            repositories.push(canonical);
        }
        if depth == 0 {
            return Ok(());
        }
        for entry_result in fs::read_dir(path)? {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(_) => continue,
            };
            if entry.file_name() == std::ffi::OsStr::new(".git") {
                continue;
            }
            if !entry
                .file_type()
                .map(|file_type| file_type.is_dir())
                .unwrap_or(false)
            {
                continue;
            }
            let child = entry.path();
            visit(&child, depth - 1, repositories)?;
        }
        Ok(())
    }

    visit(root, depth, &mut repositories)?;
    Ok(repositories)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use tempfile::TempDir;

    fn run_git(repo: &Path, args: &[&str]) {
        let status = Command::new("git")
            .current_dir(repo)
            .args(args)
            .status()
            .expect("git コマンド実行失敗");
        assert!(status.success());
    }

    fn configure_user(repo: &Path) {
        run_git(repo, &["config", "user.email", "nue@example.dev"]);
        run_git(repo, &["config", "user.name", "Nue Tester"]);
    }

    fn init_repo(repo: &Path) {
        run_git(repo, &["init"]);
        configure_user(repo);
    }

    #[test]
    fn detects_root_and_nested_repositories() {
        let workspace = TempDir::new().expect("tempdir");
        fs::create_dir(workspace.path().join(".git")).unwrap();
        fs::create_dir_all(workspace.path().join("packages/widget/.git")).unwrap();

        let mut service = RepositoryService::new(workspace.path()).expect("service init");
        let snapshots = service.detect_repositories().expect("detect failed");
        let mut roots: Vec<_> = snapshots
            .iter()
            .map(|snapshot| snapshot.relative_root.clone())
            .collect();
        roots.sort();

        assert_eq!(roots.len(), 2);
        assert!(roots.contains(&PathBuf::from(".")));
        assert!(roots.contains(&PathBuf::from("packages/widget")));
    }

    #[test]
    fn change_set_reports_staged_and_unstaged_files() {
        let workspace = TempDir::new().expect("tempdir");
        init_repo(workspace.path());

        let tracked = workspace.path().join("tracked.txt");
        fs::write(&tracked, "initial").unwrap();
        run_git(workspace.path(), &["add", "tracked.txt"]);
        run_git(workspace.path(), &["commit", "-m", "initial"]);

        fs::write(&tracked, "unstaged").unwrap();
        let staged = workspace.path().join("staged.txt");
        fs::write(&staged, "pending").unwrap();
        run_git(workspace.path(), &["add", "staged.txt"]);

        let untracked = workspace.path().join("untracked.txt");
        fs::write(&untracked, "scratch").unwrap();

        let mut service = RepositoryService::new(workspace.path()).expect("service");
        let snapshots = service.detect_repositories().expect("detect failed");
        assert_eq!(snapshots.len(), 1);

        let change_set = &snapshots[0].change_set;
        assert!(
            change_set
                .staged
                .iter()
                .any(|change| change.path == Path::new("staged.txt"))
        );
        let unstaged_paths: Vec<_> = change_set
            .unstaged
            .iter()
            .map(|change| change.path.clone())
            .collect();
        assert!(unstaged_paths.contains(&PathBuf::from("tracked.txt")));
        assert!(unstaged_paths.contains(&PathBuf::from("untracked.txt")));
    }
}
