//! リポジトリのコミットログを取得するユーティリティ。
//! 選択されたリポジトリ単位で最新の履歴を取得し、UI 側で表示できる形に変換する。

use std::io::{self, Error, ErrorKind};
use std::path::Path;
use std::process::Command;

/// デフォルトで取得するコミット件数。
pub const DEFAULT_COMMIT_LOG_LIMIT: usize = 25;
const MIN_COMMIT_LOG_LIMIT: usize = 1;
const MAX_COMMIT_LOG_LIMIT: usize = 100;

/// コミットログのエントリー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitLogEntry {
    /// フルコミットハッシュ。
    pub id: String,
    /// `git log --oneline` で表示される短縮ハッシュ。
    pub short_id: String,
    /// コミットしたユーザー名。
    pub author: String,
    /// ISO 8601 形式の日付文字列。
    pub date: String,
    /// コミットメッセージ。
    pub message: String,
}

/// 指定リポジトリの最新コミットログを取得する。
///
/// `limit` で取得件数を指定し、0 以下や `MAX_COMMIT_LOG_LIMIT` を超える値は範囲内に補正される。
pub fn fetch_commit_log(repo_root: &Path, limit: usize) -> io::Result<Vec<CommitLogEntry>> {
    let canonical_root = repo_root.canonicalize()?;
    let limit = limit.clamp(MIN_COMMIT_LOG_LIMIT, MAX_COMMIT_LOG_LIMIT);
    let max_count_arg = format!("--max-count={limit}");
    let output = Command::new("git")
        .arg("-C")
        .arg(&canonical_root)
        .arg("log")
        .arg(&max_count_arg)
        .arg("--pretty=format:%H\x1f%h\x1f%an\x1f%ad\x1f%s")
        .arg("--date=iso")
        .output()?;

    if !output.status.success() {
        return Err(Error::other("git log サブコマンドが失敗しました"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();

    for line in stdout.lines() {
        if line.is_empty() {
            continue;
        }
        entries.push(parse_commit_line(line)?);
    }

    Ok(entries)
}

fn parse_commit_line(line: &str) -> io::Result<CommitLogEntry> {
    let mut segments = line.split('\x1f');
    let id = segments
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "コミットハッシュが不足しています"))?
        .to_string();
    let short_id = segments
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "短縮ハッシュが不足しています"))?
        .to_string();
    let author = segments
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "作者情報が不足しています"))?
        .to_string();
    let date = segments
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "日付情報が不足しています"))?
        .to_string();
    let message = segments
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "メッセージが不足しています"))?
        .to_string();

    Ok(CommitLogEntry {
        id,
        short_id,
        author,
        date,
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io;
    use std::path::Path;
    use std::process::Command;
    use tempfile::TempDir;

    fn run_git(repo: &Path, args: &[&str]) -> io::Result<()> {
        let status = Command::new("git").current_dir(repo).args(args).status()?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::other(format!("git {:?} が失敗しました", args)))
        }
    }

    fn prepare_repo() -> io::Result<TempDir> {
        let temp = TempDir::new()?;
        let repo_root = temp.path();
        run_git(repo_root, &["init", "-b", "main"])?;
        run_git(repo_root, &["config", "user.name", "Test User"])?;
        run_git(repo_root, &["config", "user.email", "test@example.com"])?;
        Ok(temp)
    }

    fn commit_file(repo: &Path, path: &str, contents: &str, message: &str) -> io::Result<()> {
        let file_path = repo.join(path);
        fs::write(&file_path, contents)?;
        run_git(repo, &["add", path])?;
        run_git(repo, &["commit", "-m", message])?;
        Ok(())
    }

    #[test]
    fn fetch_commit_log_returns_latest_history() -> io::Result<()> {
        let temp = prepare_repo()?;
        let repo_root = temp.path();
        commit_file(repo_root, "README.md", "initial", "initial commit")?;
        commit_file(repo_root, "README.md", "second", "second commit")?;

        let entries = fetch_commit_log(repo_root, 5)?;
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].message, "second commit");
        assert_eq!(entries[1].message, "initial commit");
        assert_ne!(entries[0].id, entries[1].id);
        Ok(())
    }

    #[test]
    fn limit_is_clamped_and_returns_available_history() -> io::Result<()> {
        let temp = prepare_repo()?;
        let repo_root = temp.path();
        commit_file(repo_root, "a.txt", "a", "first")?;

        let entries_high_limit = fetch_commit_log(repo_root, MAX_COMMIT_LOG_LIMIT + 10)?;
        assert_eq!(entries_high_limit.len(), 1);

        let entries_zero_limit = fetch_commit_log(repo_root, 0)?;
        assert_eq!(entries_zero_limit.len(), 1);
        Ok(())
    }
}
