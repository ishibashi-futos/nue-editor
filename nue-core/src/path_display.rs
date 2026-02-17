use std::borrow::Cow;
use std::path::{Path, PathBuf};

use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

const DOTS: &str = "...";
const MAX_DISPLAY_WIDTH: usize = 40;

/// UI 用の文字列に変換するパス表示ポリシーを提供する。
pub(crate) fn display_path(path: &Path) -> Cow<'_, str> {
    match path.to_str() {
        Some(text) if !should_truncate(text) => Cow::Borrowed(text),
        Some(text) => Cow::Owned(truncate_path(text)),
        None => {
            let encoded =
                utf8_percent_encode(&path.to_string_lossy(), NON_ALPHANUMERIC).to_string();
            Cow::Owned(truncate_path(&encoded))
        }
    }
}

fn truncate_path(text: &str) -> String {
    if !should_truncate(text) {
        return text.to_string();
    }

    let trimmed_len = MAX_DISPLAY_WIDTH - DOTS.len();
    if trimmed_len == 0 {
        return DOTS.to_string();
    }

    let tail_len = trimmed_len / 2;
    let head_len = trimmed_len - tail_len;
    let start = text.chars().take(head_len).collect::<String>();
    let end = text
        .chars()
        .rev()
        .take(tail_len)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();

    format!("{start}{DOTS}{end}")
}

fn should_truncate(text: &str) -> bool {
    text.chars().nth(MAX_DISPLAY_WIDTH).is_some()
}

/// Path/PathBuf に対して UI 表示用の文字列を提供する拡張トレイト。
pub trait PathDisplayExt {
    fn display_for_ui(&self) -> Cow<'_, str>;
}

impl PathDisplayExt for Path {
    fn display_for_ui(&self) -> Cow<'_, str> {
        display_path(self)
    }
}

impl PathDisplayExt for PathBuf {
    fn display_for_ui(&self) -> Cow<'_, str> {
        display_path(self.as_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[cfg(unix)]
    use std::ffi::OsString;
    #[cfg(unix)]
    use std::os::unix::ffi::OsStringExt;

    #[test]
    fn display_short_path() {
        let path = PathBuf::from("/workspace/README.md");
        assert_eq!(display_path(&path), "/workspace/README.md");
    }

    #[cfg(unix)]
    #[test]
    fn display_non_utf8_path() {
        let os_string = OsString::from_vec(vec![0xff, 0xfe, 0xfa]);
        let path = PathBuf::from(os_string);
        let displayed = display_path(&path);
        assert!(displayed.contains('%'));
    }

    #[test]
    fn display_long_path_truncates_middle() {
        let long = PathBuf::from("/very/long/path/that/exceeds/the/limit/of/display/logic.rs");
        let displayed = display_path(&long);
        assert!(displayed.chars().count() <= MAX_DISPLAY_WIDTH);
        assert!(displayed.contains("..."));
    }

    #[test]
    fn 日本語を含む40文字未満のパスは省略しない() {
        let path = PathBuf::from("/workspace/日本語日本語日本語.rs");
        assert_eq!(display_path(&path), "/workspace/日本語日本語日本語.rs");
    }
}
