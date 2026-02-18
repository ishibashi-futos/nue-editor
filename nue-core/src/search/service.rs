use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub enum TextCriteria {
    Literal(String),
    Regex(Regex),
}

impl TextCriteria {
    pub fn literal(pattern: impl Into<String>) -> Self {
        Self::Literal(pattern.into())
    }

    pub fn regex(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self::Regex(Regex::new(pattern)?))
    }

    pub fn matches(&self, haystack: &str) -> bool {
        !self.match_ranges(haystack).is_empty()
    }

    fn match_ranges(&self, haystack: &str) -> Vec<(usize, usize)> {
        let mut ranges = Vec::new();
        match self {
            TextCriteria::Literal(literal) => {
                if literal.is_empty() {
                    return ranges;
                }
                let mut start = 0;
                while let Some(position) = haystack[start..].find(literal) {
                    let match_start = start + position;
                    let match_end = match_start + literal.len();
                    ranges.push((match_start, match_end));
                    start = match_end;
                }
            }
            TextCriteria::Regex(regex) => {
                for mat in regex.find_iter(haystack) {
                    ranges.push((mat.start(), mat.end()));
                }
            }
        }
        ranges
    }

    /// 現在の条件が正規表現かどうかを判定する。
    pub fn is_regex(&self) -> bool {
        matches!(self, TextCriteria::Regex(_))
    }

    /// パターン文字列を取り出す。正規表現の場合はそのままの文字列を返す。
    pub fn as_str(&self) -> &str {
        match self {
            TextCriteria::Literal(text) => text.as_str(),
            TextCriteria::Regex(regex) => regex.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pattern: TextCriteria,
    respect_gitignore: bool,
    folder_filter: Option<TextCriteria>,
    file_filter: Option<TextCriteria>,
}

impl SearchQuery {
    pub fn new(pattern: TextCriteria) -> Self {
        Self {
            pattern,
            respect_gitignore: true,
            folder_filter: None,
            file_filter: None,
        }
    }

    pub fn literal(pattern: impl Into<String>) -> Self {
        Self::new(TextCriteria::literal(pattern))
    }

    pub fn regex(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self::new(TextCriteria::regex(pattern)?))
    }

    pub fn with_respect_gitignore(mut self, respect: bool) -> Self {
        self.respect_gitignore = respect;
        self
    }

    pub fn with_folder_filter(mut self, filter: TextCriteria) -> Self {
        self.folder_filter = Some(filter);
        self
    }

    pub fn with_file_filter(mut self, filter: TextCriteria) -> Self {
        self.file_filter = Some(filter);
        self
    }

    pub fn pattern(&self) -> &TextCriteria {
        &self.pattern
    }

    pub fn folder_filter(&self) -> Option<&TextCriteria> {
        self.folder_filter.as_ref()
    }

    pub fn file_filter(&self) -> Option<&TextCriteria> {
        self.file_filter.as_ref()
    }

    pub fn respect_gitignore(&self) -> bool {
        self.respect_gitignore
    }
}

impl PartialEq for TextCriteria {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TextCriteria::Literal(lhs), TextCriteria::Literal(rhs)) => lhs == rhs,
            (TextCriteria::Regex(lhs), TextCriteria::Regex(rhs)) => lhs.as_str() == rhs.as_str(),
            _ => false,
        }
    }
}

impl Eq for TextCriteria {}

impl PartialEq for SearchQuery {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
            && self.respect_gitignore == other.respect_gitignore
            && self.folder_filter == other.folder_filter
            && self.file_filter == other.file_filter
    }
}

impl Eq for SearchQuery {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_text: String,
    pub match_start: usize,
    pub match_end: usize,
}

#[derive(Debug)]
pub enum SearchError {
    Io(io::Error),
}

impl From<io::Error> for SearchError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug)]
pub struct SearchService {
    root: PathBuf,
    gitignore_entries: Option<HashSet<PathBuf>>,
}

impl SearchService {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            gitignore_entries: None,
        }
    }

    pub fn search(&mut self, query: &SearchQuery) -> Result<Vec<SearchMatch>, SearchError> {
        if self.gitignore_entries.is_none() {
            self.gitignore_entries = Some(self.load_gitignore()?);
        }

        let mut matches = Vec::new();
        for entry in WalkDir::new(&self.root)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let relative_path = entry
                .path()
                .strip_prefix(&self.root)
                .unwrap_or(entry.path());
            if query.respect_gitignore() && self.is_ignored(relative_path) {
                continue;
            }
            if let Some(folder_filter) = query.folder_filter()
                && !folder_filter.matches(relative_path.to_string_lossy().as_ref())
            {
                continue;
            }
            if let Some(file_filter) = query.file_filter()
                && !file_filter.matches(
                    entry
                        .path()
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or(""),
                )
            {
                continue;
            }

            let content = fs::read_to_string(entry.path())?;
            for (line_idx, line) in content.lines().enumerate() {
                if !query.pattern().matches(line) {
                    continue;
                }
                let ranges = query.pattern().match_ranges(line);
                for (start, end) in ranges {
                    matches.push(SearchMatch {
                        file_path: entry.path().to_path_buf(),
                        line_number: line_idx + 1,
                        line_text: line.to_string(),
                        match_start: start,
                        match_end: end,
                    });
                }
            }
        }

        Ok(matches)
    }

    fn load_gitignore(&self) -> Result<HashSet<PathBuf>, SearchError> {
        let gitignore_path = self.root.join(".gitignore");
        if !gitignore_path.exists() {
            return Ok(HashSet::new());
        }
        let contents = fs::read_to_string(gitignore_path)?;
        Ok(contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(PathBuf::from)
            .collect())
    }

    fn is_ignored(&self, relative_path: &Path) -> bool {
        if let Some(entries) = &self.gitignore_entries {
            entries
                .iter()
                .any(|ignored| relative_path.starts_with(ignored))
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_file(base: &Path, relative: &str, content: &str) {
        let path = base.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn literal_search_reports_matches() {
        let tree = tempdir().unwrap();
        write_file(tree.path(), "notes.md", "first line\nneedle found\nlast");

        let mut service = SearchService::new(tree.path());
        let query = SearchQuery::literal("needle");
        let matches = service.search(&query).unwrap();

        assert!(!matches.is_empty());
    }

    #[test]
    fn regex_search_respects_pattern() {
        let tree = tempdir().unwrap();
        write_file(tree.path(), "data.rs", "alpha beta\nfoo_bar\nfoo123");

        let mut service = SearchService::new(tree.path());
        let query = SearchQuery::regex(r"foo_\w+").unwrap();
        let matches = service.search(&query).unwrap();

        assert_eq!(matches.len(), 1);
        assert!(matches[0].line_text.contains("foo_bar"));
    }

    #[test]
    fn gitignore_toggle_controls_scope() {
        let tree = tempdir().unwrap();
        fs::write(tree.path().join(".gitignore"), "ignored.md\n").unwrap();
        write_file(tree.path(), "ignored.md", "needle\n");

        let mut service = SearchService::new(tree.path());
        let base_query = SearchQuery::literal("needle");
        assert!(service.search(&base_query).unwrap().is_empty());

        let inclusive_query = base_query.with_respect_gitignore(false);
        assert!(!service.search(&inclusive_query).unwrap().is_empty());
    }

    #[test]
    fn folder_and_file_filters_scope_search() {
        let tree = tempdir().unwrap();
        write_file(tree.path(), "src/app/main.rs", "needle here\n");
        write_file(tree.path(), "src/app/helper.rs", "needle helper\n");
        write_file(tree.path(), "src/other/main.rs", "needle other\n");

        let mut service = SearchService::new(tree.path());
        let query = SearchQuery::literal("needle")
            .with_folder_filter(TextCriteria::literal("app"))
            .with_file_filter(TextCriteria::regex(r"^main\.rs$").unwrap());

        let matches = service.search(&query).unwrap();

        assert_eq!(matches.len(), 1);
        assert!(matches[0].file_path.ends_with(Path::new("src/app/main.rs")));
    }

    #[test]
    fn gitignoreの相対パス指定で配下ファイルを除外する() {
        let tree = tempdir().unwrap();
        fs::write(tree.path().join(".gitignore"), "build\n").unwrap();
        write_file(tree.path(), "build/output.log", "needle\n");
        write_file(tree.path(), "src/keep.log", "needle\n");

        let mut service = SearchService::new(tree.path());
        let query = SearchQuery::literal("needle");
        let matches = service.search(&query).unwrap();

        assert_eq!(matches.len(), 1);
        assert!(matches[0].file_path.ends_with(Path::new("src/keep.log")));
    }
}
