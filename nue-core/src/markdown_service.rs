use std::cmp;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownFeature {
    SyntaxHighlight,
    ListContinuation,
    PairCompletion,
    OpenLink,
    PreviewSync,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownHeading {
    pub line: usize,
    pub level: u8,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownFeatureRequestedEvent {
    pub file_path: PathBuf,
    pub feature: MarkdownFeature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownDiffObservedEvent {
    pub file_path: PathBuf,
    pub revision: u64,
    pub changed_line_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownPreviewSyncedEvent {
    pub file_path: PathBuf,
    pub revision: u64,
    pub heading_count: usize,
    pub headings: Vec<MarkdownHeading>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownServiceEvent {
    FeatureRequested(MarkdownFeatureRequestedEvent),
    DiffObserved(MarkdownDiffObservedEvent),
    PreviewSynced(MarkdownPreviewSyncedEvent),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarkdownService;

impl MarkdownService {
    pub fn new() -> Self {
        Self
    }

    pub fn request_feature(
        &self,
        file_path: &Path,
        feature: MarkdownFeature,
    ) -> Option<MarkdownServiceEvent> {
        if !is_markdown_file(file_path) {
            return None;
        }

        Some(MarkdownServiceEvent::FeatureRequested(
            MarkdownFeatureRequestedEvent {
                file_path: file_path.to_path_buf(),
                feature,
            },
        ))
    }

    pub fn observe_change(
        &self,
        file_path: &Path,
        revision: u64,
        previous_content: &str,
        current_content: &str,
    ) -> Vec<MarkdownServiceEvent> {
        if !is_markdown_file(file_path) || previous_content == current_content {
            return Vec::new();
        }

        let changed_line_count = count_changed_lines(previous_content, current_content);
        let headings = collect_markdown_headings(current_content);
        let heading_count = headings.len();

        vec![
            MarkdownServiceEvent::DiffObserved(MarkdownDiffObservedEvent {
                file_path: file_path.to_path_buf(),
                revision,
                changed_line_count,
            }),
            MarkdownServiceEvent::PreviewSynced(MarkdownPreviewSyncedEvent {
                file_path: file_path.to_path_buf(),
                revision,
                heading_count,
                headings,
            }),
        ]
    }

    pub fn collect_headings(content: &str) -> Vec<MarkdownHeading> {
        collect_markdown_headings(content)
    }
}

fn is_markdown_file(file_path: &Path) -> bool {
    let Some(extension) = file_path.extension().and_then(|value| value.to_str()) else {
        return false;
    };
    extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
}

fn count_changed_lines(previous_content: &str, current_content: &str) -> usize {
    let previous_lines: Vec<&str> = previous_content.lines().collect();
    let current_lines: Vec<&str> = current_content.lines().collect();
    let max_len = cmp::max(previous_lines.len(), current_lines.len());
    let mut changed_line_count = 0;

    for line_index in 0..max_len {
        let previous_line = previous_lines.get(line_index);
        let current_line = current_lines.get(line_index);
        if previous_line != current_line {
            changed_line_count += 1;
        }
    }

    changed_line_count
}

fn collect_markdown_headings(content: &str) -> Vec<MarkdownHeading> {
    let mut headings = Vec::new();
    let mut in_fenced_code_block = false;

    for (line_index, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fenced_code_block = !in_fenced_code_block;
            continue;
        }
        if in_fenced_code_block {
            continue;
        }
        let level = trimmed.chars().take_while(|c| *c == '#').count();
        if level == 0 {
            continue;
        }
        let title = trimmed[level..].trim_start().to_string();
        let heading = MarkdownHeading {
            line: line_index,
            level: level.min(u8::MAX as usize) as u8,
            title,
        };
        headings.push(heading);
    }

    headings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdownファイルに対して機能要求イベントを返す() {
        let service = MarkdownService::new();

        let event = service.request_feature(
            Path::new("docs/readme.md"),
            MarkdownFeature::SyntaxHighlight,
        );

        assert_eq!(
            event,
            Some(MarkdownServiceEvent::FeatureRequested(
                MarkdownFeatureRequestedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    feature: MarkdownFeature::SyntaxHighlight,
                }
            ))
        );
    }

    #[test]
    fn markdown編集に対して差分とプレビュー同期イベントを返す() {
        let service = MarkdownService::new();

        let events = service.observe_change(
            Path::new("docs/readme.md"),
            3,
            "# Title\n- item",
            "# Title\n- item\n## Section",
        );

        assert_eq!(
            events,
            vec![
                MarkdownServiceEvent::DiffObserved(MarkdownDiffObservedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    revision: 3,
                    changed_line_count: 1,
                }),
                MarkdownServiceEvent::PreviewSynced(MarkdownPreviewSyncedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    revision: 3,
                    heading_count: 2,
                    headings: vec![
                        MarkdownHeading {
                            line: 0,
                            level: 1,
                            title: "Title".to_string(),
                        },
                        MarkdownHeading {
                            line: 2,
                            level: 2,
                            title: "Section".to_string(),
                        },
                    ],
                }),
            ]
        );
    }

    #[test]
    fn markdown以外はイベントを返さない() {
        let service = MarkdownService::new();

        assert_eq!(
            service.request_feature(Path::new("docs/readme.txt"), MarkdownFeature::OpenLink),
            None
        );
        assert!(
            service
                .observe_change(Path::new("docs/readme.txt"), 1, "before", "after")
                .is_empty()
        );
    }

    #[test]
    fn フェンスコードブロック内のシャープは見出しとして数えない() {
        let service = MarkdownService::new();

        let events = service.observe_change(
            Path::new("docs/readme.md"),
            2,
            "# Title",
            "# Title\n```rust\n# not heading\n```\n## Section",
        );

        assert_eq!(events.len(), 2);
        assert_eq!(
            events[1],
            MarkdownServiceEvent::PreviewSynced(MarkdownPreviewSyncedEvent {
                file_path: PathBuf::from("docs/readme.md"),
                revision: 2,
                heading_count: 2,
                headings: vec![
                    MarkdownHeading {
                        line: 0,
                        level: 1,
                        title: "Title".to_string(),
                    },
                    MarkdownHeading {
                        line: 4,
                        level: 2,
                        title: "Section".to_string(),
                    },
                ],
            })
        );
    }

    #[test]
    fn collect_headingsは行番号とレベルとタイトルを返す() {
        let content = "# Title\n\n## Section\nここには`# not heading`があります\n```python\n# code block\n```\n### Sub";

        let headings = MarkdownService::collect_headings(content);

        assert_eq!(
            headings,
            vec![
                MarkdownHeading {
                    line: 0,
                    level: 1,
                    title: "Title".to_string(),
                },
                MarkdownHeading {
                    line: 2,
                    level: 2,
                    title: "Section".to_string(),
                },
                MarkdownHeading {
                    line: 7,
                    level: 3,
                    title: "Sub".to_string(),
                },
            ]
        );
    }
}
