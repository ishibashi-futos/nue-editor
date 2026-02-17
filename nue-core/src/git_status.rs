use std::collections::HashMap;
use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// ファイル単位での Git 差分種別を表現する列挙型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitFileStatus {
    Untracked,
    Modified,
    Deleted,
}

impl GitFileStatus {
    /// ステータスを優先度として比較し、より高い影響度を保つ。
    pub(crate) fn merge(self, other: GitFileStatus) -> GitFileStatus {
        use GitFileStatus::*;
        match (self, other) {
            (Deleted, _) | (_, Deleted) => Deleted,
            (Modified, _) | (_, Modified) => Modified,
            _ => Untracked,
        }
    }
}

/// インデックス/作業ツリーの変更種別をざっくり表現する列挙型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    TypeChanged,
    Untracked,
    Conflict,
    Unknown,
}

/// Git ステータス出力の 1 行分を保持する構造体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitStatusEntry {
    pub path: PathBuf,
    pub index_status: Option<GitChangeKind>,
    pub worktree_status: Option<GitChangeKind>,
}

/// workspace_root 直下の `git status --porcelain` をパースし、ファイルごとのステータスマップを返す。
pub fn collect_git_statuses(root: &Path) -> HashMap<PathBuf, GitFileStatus> {
    let mut statuses = HashMap::new();
    for entry in collect_git_status_entries(root) {
        if let Some(kind) = entry.index_status.or(entry.worktree_status) {
            let file_status = map_change_kind_to_file_status(kind);
            statuses
                .entry(entry.path)
                .and_modify(|existing: &mut GitFileStatus| *existing = existing.merge(file_status))
                .or_insert(file_status);
        }
    }

    statuses
}

/// Git ステータス行をパースして変更情報のベクトルで返す。
pub fn collect_git_status_entries(root: &Path) -> Vec<GitStatusEntry> {
    let output = Command::new("git")
        .args(["status", "--porcelain=1", "-z", "--untracked-files=normal"])
        .current_dir(root)
        .output();

    let output = match output {
        Ok(output) if output.status.success() => output,
        _ => return Vec::new(),
    };

    let records = output
        .stdout
        .split(|byte| *byte == b'\0')
        .collect::<Vec<_>>();
    let mut entries = Vec::new();
    let mut index = 0;
    while index < records.len() {
        let record = records[index];
        index += 1;
        if record.is_empty() {
            continue;
        }
        let Some((entry, consumes_extra_path)) = parse_status_record(root, record) else {
            continue;
        };
        entries.push(entry);
        if consumes_extra_path {
            index += 1;
        }
    }

    entries
}

fn parse_status_record(root: &Path, record: &[u8]) -> Option<(GitStatusEntry, bool)> {
    if record.len() < 4 {
        return None;
    }
    let index_char = record[0] as char;
    let worktree_char = record[1] as char;
    let path_bytes = record.get(3..)?;
    if path_bytes.is_empty() {
        return None;
    }

    let relative_path = path_from_git_bytes(path_bytes);
    let canonical_path = normalize_status_path(root, relative_path.as_path());
    let index_status = match (index_char, worktree_char) {
        ('?', '?') => None,
        _ => status_char_to_kind(index_char),
    };
    let worktree_status = if worktree_char == '?' {
        Some(GitChangeKind::Untracked)
    } else {
        status_char_to_kind(worktree_char)
    };

    if index_status.is_none() && worktree_status.is_none() {
        return None;
    }

    Some((
        GitStatusEntry {
            path: canonical_path,
            index_status,
            worktree_status,
        },
        matches!(index_char, 'R' | 'C') || matches!(worktree_char, 'R' | 'C'),
    ))
}

fn status_char_to_kind(ch: char) -> Option<GitChangeKind> {
    match ch {
        'M' => Some(GitChangeKind::Modified),
        'A' => Some(GitChangeKind::Added),
        'D' => Some(GitChangeKind::Deleted),
        'R' => Some(GitChangeKind::Renamed),
        'C' => Some(GitChangeKind::Copied),
        'T' => Some(GitChangeKind::TypeChanged),
        'U' => Some(GitChangeKind::Conflict),
        '?' => Some(GitChangeKind::Untracked),
        ' ' => None,
        _ => Some(GitChangeKind::Unknown),
    }
}

fn map_change_kind_to_file_status(kind: GitChangeKind) -> GitFileStatus {
    use GitChangeKind::*;
    match kind {
        Deleted => GitFileStatus::Deleted,
        Untracked => GitFileStatus::Untracked,
        _ => GitFileStatus::Modified,
    }
}

pub(crate) fn normalize_status_path(root: &Path, relative: &Path) -> PathBuf {
    let candidate = root.join(relative);
    match candidate.canonicalize() {
        Ok(canonical) => canonical,
        Err(_) => candidate,
    }
}

#[cfg(unix)]
fn path_from_git_bytes(path_bytes: &[u8]) -> PathBuf {
    PathBuf::from(OsString::from_vec(path_bytes.to_vec()))
}

#[cfg(not(unix))]
fn path_from_git_bytes(path_bytes: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(path_bytes).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn init_repo(dir: &TempDir) {
        let status = Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .status()
            .expect("git init 失敗");
        assert!(status.success());
    }

    fn run_git(dir: &TempDir, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .status()
            .expect("git コマンド実行失敗");
        assert!(status.success());
    }

    fn configure_user(dir: &TempDir) {
        run_git(dir, &["config", "user.email", "nue@example.dev"]);
        run_git(dir, &["config", "user.name", "Nue Tester"]);
    }

    #[test]
    fn git_status_output_is_parsed() {
        let dir = TempDir::new().expect("tempdir");
        fs::write(dir.path().join("tracked.txt"), "initial").unwrap();
        init_repo(&dir);
        configure_user(&dir);
        run_git(&dir, &["add", "tracked.txt"]);
        run_git(&dir, &["commit", "-m", "init"]);

        fs::write(dir.path().join("modified.txt"), "first").unwrap();
        run_git(&dir, &["add", "modified.txt"]);
        run_git(&dir, &["commit", "-m", "add modified"]);
        fs::write(dir.path().join("modified.txt"), "changed").unwrap();
        fs::write(dir.path().join("deleted.txt"), "bye").unwrap();
        run_git(&dir, &["add", "deleted.txt"]);
        run_git(&dir, &["commit", "-m", "add deleted"]);
        run_git(&dir, &["rm", "deleted.txt"]);
        fs::write(dir.path().join("untracked.txt"), "temp").unwrap();

        let statuses = collect_git_statuses(dir.path());
        let modified_path = fs::canonicalize(dir.path().join("modified.txt")).unwrap();
        let untracked_path = fs::canonicalize(dir.path().join("untracked.txt")).unwrap();
        let tracked_path = fs::canonicalize(dir.path().join("tracked.txt")).unwrap();

        assert_eq!(statuses.get(&modified_path), Some(&GitFileStatus::Modified));
        assert_eq!(
            statuses.get(&dir.path().join("deleted.txt")),
            Some(&GitFileStatus::Deleted)
        );
        assert_eq!(
            statuses.get(&untracked_path),
            Some(&GitFileStatus::Untracked)
        );
        assert!(!statuses.contains_key(&tracked_path));
    }

    #[test]
    fn git_status_parses_quoted_non_ascii_paths() {
        let dir = TempDir::new().expect("tempdir");
        init_repo(&dir);
        configure_user(&dir);
        run_git(&dir, &["config", "core.quotepath", "true"]);

        let filename = "日本語.txt";
        let file = dir.path().join(filename);
        fs::write(&file, "first").unwrap();
        run_git(&dir, &["add", filename]);
        run_git(&dir, &["commit", "-m", "initial"]);
        fs::write(&file, "changed").unwrap();

        let statuses = collect_git_statuses(dir.path());
        let path = fs::canonicalize(&file).unwrap();
        assert_eq!(statuses.get(&path), Some(&GitFileStatus::Modified));
    }
}
