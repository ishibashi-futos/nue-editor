#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandMode {
    Action,
    Navigation,
    Intent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCommand {
    pub mode: CommandMode,
    pub domain: String,
    pub verb: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerCandidate {
    pub id: String,
    pub label: String,
    pub command: ParsedCommand,
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerViewState {
    Closed,
    Listing,
    Confirming,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerSnapshot {
    pub state: PickerViewState,
    pub selected_candidate_id: Option<String>,
    pub visible_candidates: Vec<PickerCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerExecuteOutcome {
    NoSelection,
    NeedsConfirmation { candidate_id: String },
    Executed(PickerCandidate),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerCancelOutcome {
    Noop,
    Closed,
    BackToListing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPicker {
    state: PickerViewState,
    visible_candidates: Vec<PickerCandidate>,
    selected_index: Option<usize>,
    confirming_index: Option<usize>,
}

pub fn parse_command(input: &str) -> ParsedCommand {
    let trimmed = input.trim();
    if let Some(action_input) = trimmed.strip_prefix('>') {
        return parse_action_mode(action_input);
    }
    if let Some(navigation_input) = trimmed.strip_prefix(':') {
        return parse_navigation_mode(navigation_input);
    }

    ParsedCommand {
        mode: CommandMode::Intent,
        domain: "intent".to_string(),
        verb: "search".to_string(),
        target: normalize_target(trimmed),
    }
}

fn parse_action_mode(input: &str) -> ParsedCommand {
    let (domain_raw, body_raw) = input.split_once(':').unwrap_or(("action", input));

    let body = body_raw.trim();
    let (verb_raw, target_raw) = split_head_token(body);
    let verb = normalize_keyword(verb_raw);
    let verb = if verb.is_empty() {
        "run".to_string()
    } else {
        verb
    };
    let target = parse_action_target(target_raw);

    let domain = normalize_keyword(domain_raw);
    let domain = if domain.is_empty() {
        "action".to_string()
    } else {
        domain
    };

    ParsedCommand {
        mode: CommandMode::Action,
        domain,
        verb,
        target,
    }
}

fn parse_navigation_mode(input: &str) -> ParsedCommand {
    ParsedCommand {
        mode: CommandMode::Navigation,
        domain: "navigation".to_string(),
        verb: "open".to_string(),
        target: normalize_target(input),
    }
}

impl CommandPicker {
    pub fn new() -> Self {
        Self {
            state: PickerViewState::Closed,
            visible_candidates: Vec::new(),
            selected_index: None,
            confirming_index: None,
        }
    }

    pub fn open(&mut self, candidates: Vec<PickerCandidate>) {
        self.state = PickerViewState::Listing;
        self.selected_index = if candidates.is_empty() { None } else { Some(0) };
        self.confirming_index = None;
        self.visible_candidates = candidates;
    }

    pub fn snapshot(&self) -> PickerSnapshot {
        PickerSnapshot {
            state: self.state,
            selected_candidate_id: self
                .selected_index
                .and_then(|index| self.visible_candidates.get(index))
                .map(|candidate| candidate.id.clone()),
            visible_candidates: self.visible_candidates.clone(),
        }
    }

    pub fn request_execute_selected(&mut self) -> PickerExecuteOutcome {
        let Some(index) = self.selected_index else {
            return PickerExecuteOutcome::NoSelection;
        };
        if self.state != PickerViewState::Listing {
            return PickerExecuteOutcome::NoSelection;
        }
        let Some(candidate) = self.visible_candidates.get(index).cloned() else {
            return PickerExecuteOutcome::NoSelection;
        };

        if candidate.requires_confirmation {
            self.state = PickerViewState::Confirming;
            self.confirming_index = Some(index);
            return PickerExecuteOutcome::NeedsConfirmation {
                candidate_id: candidate.id,
            };
        }

        self.close();
        PickerExecuteOutcome::Executed(candidate)
    }

    pub fn confirm_execute(&mut self) -> PickerExecuteOutcome {
        if self.state != PickerViewState::Confirming {
            return PickerExecuteOutcome::NoSelection;
        }
        let Some(index) = self.confirming_index else {
            return PickerExecuteOutcome::NoSelection;
        };
        let Some(candidate) = self.visible_candidates.get(index).cloned() else {
            return PickerExecuteOutcome::NoSelection;
        };

        self.close();
        PickerExecuteOutcome::Executed(candidate)
    }

    pub fn cancel(&mut self) -> PickerCancelOutcome {
        if self.state == PickerViewState::Closed {
            return PickerCancelOutcome::Noop;
        }

        if self.state == PickerViewState::Confirming {
            self.state = PickerViewState::Listing;
            self.confirming_index = None;
            return PickerCancelOutcome::BackToListing;
        }

        self.close();
        PickerCancelOutcome::Closed
    }

    fn close(&mut self) {
        self.state = PickerViewState::Closed;
        self.selected_index = None;
        self.confirming_index = None;
        self.visible_candidates.clear();
    }
}

impl Default for CommandPicker {
    fn default() -> Self {
        Self::new()
    }
}

fn normalize_keyword(value: &str) -> String {
    let normalized = value
        .trim()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>();

    normalized
        .split_whitespace()
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<&str>>()
        .join("_")
}

fn normalize_target(value: &str) -> String {
    value
        .split_whitespace()
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_ascii_lowercase())
        .collect::<Vec<String>>()
        .join(" ")
}

fn parse_action_target(value: &str) -> String {
    let target = value.trim();
    if target.is_empty() {
        return String::new();
    }

    if let Some(literal) = parse_literal_flag_target(target) {
        return literal;
    }
    if let Some(literal) = parse_double_quoted_target(target) {
        return literal;
    }

    normalize_target(target)
}

fn parse_literal_flag_target(value: &str) -> Option<String> {
    let (flag, literal_body) = split_head_token(value);
    if flag.eq_ignore_ascii_case("--literal") {
        return Some(literal_body.to_string());
    }
    None
}

fn parse_double_quoted_target(value: &str) -> Option<String> {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        return Some(value[1..value.len() - 1].to_string());
    }
    None
}

fn split_head_token(value: &str) -> (&str, &str) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return ("", "");
    }

    if let Some((index, character)) = trimmed.char_indices().find(|(_, ch)| ch.is_whitespace()) {
        let next_index = index + character.len_utf8();
        let head = &trimmed[..index];
        let tail = trimmed[next_index..].trim_start();
        return (head, tail);
    }

    (trimmed, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action_candidate() -> PickerCandidate {
        PickerCandidate {
            id: "candidate-action-1".to_string(),
            label: "Terminal: Split Terminal".to_string(),
            command: ParsedCommand {
                mode: CommandMode::Action,
                domain: "terminal".to_string(),
                verb: "split".to_string(),
                target: "terminal".to_string(),
            },
            requires_confirmation: false,
        }
    }

    fn destructive_candidate() -> PickerCandidate {
        PickerCandidate {
            id: "candidate-action-2".to_string(),
            label: "Workspace: Remove current".to_string(),
            command: ParsedCommand {
                mode: CommandMode::Action,
                domain: "workspace".to_string(),
                verb: "remove".to_string(),
                target: "current".to_string(),
            },
            requires_confirmation: true,
        }
    }

    #[test]
    fn action_modeはdomain_verb_targetを正規化して解析する() {
        let parsed = parse_command("> Terminal: Split Terminal");

        assert_eq!(
            parsed,
            ParsedCommand {
                mode: CommandMode::Action,
                domain: "terminal".to_string(),
                verb: "split".to_string(),
                target: "terminal".to_string(),
            }
        );
    }

    #[test]
    fn navigation_modeは共通フォーマットで正規化する() {
        let parsed = parse_command(":src/lib.rs");

        assert_eq!(
            parsed,
            ParsedCommand {
                mode: CommandMode::Navigation,
                domain: "navigation".to_string(),
                verb: "open".to_string(),
                target: "src/lib.rs".to_string(),
            }
        );
    }

    #[test]
    fn action_modeはダブルクオートtargetをliteralとして保持する() {
        let parsed = parse_command("> Workspace: Add \"~/Work/My Project\"");

        assert_eq!(
            parsed,
            ParsedCommand {
                mode: CommandMode::Action,
                domain: "workspace".to_string(),
                verb: "add".to_string(),
                target: "~/Work/My Project".to_string(),
            }
        );
    }

    #[test]
    fn action_modeはliteralフラグ指定時にtargetの大文字小文字を保持する() {
        let parsed = parse_command("> Terminal: Run --LiTeRaL Cargo Test -- --nocapture");

        assert_eq!(
            parsed,
            ParsedCommand {
                mode: CommandMode::Action,
                domain: "terminal".to_string(),
                verb: "run".to_string(),
                target: "Cargo Test -- --nocapture".to_string(),
            }
        );
    }

    #[test]
    fn pickerは候補一覧を表示しキャンセルで閉じる() {
        let mut picker = CommandPicker::new();

        picker.open(vec![action_candidate()]);
        assert_eq!(
            picker.snapshot(),
            PickerSnapshot {
                state: PickerViewState::Listing,
                selected_candidate_id: Some("candidate-action-1".to_string()),
                visible_candidates: vec![action_candidate()],
            }
        );

        assert_eq!(picker.cancel(), PickerCancelOutcome::Closed);
        assert_eq!(picker.snapshot().state, PickerViewState::Closed);
    }

    #[test]
    fn 確認必須候補は確認フローを経由して実行される() {
        let mut picker = CommandPicker::new();
        let destructive = destructive_candidate();

        picker.open(vec![destructive.clone()]);

        let needs_confirmation = picker.request_execute_selected();
        assert_eq!(
            needs_confirmation,
            PickerExecuteOutcome::NeedsConfirmation {
                candidate_id: destructive.id.clone(),
            }
        );
        assert_eq!(picker.snapshot().state, PickerViewState::Confirming);

        let executed = picker.confirm_execute();
        assert_eq!(executed, PickerExecuteOutcome::Executed(destructive));
        assert_eq!(picker.snapshot().state, PickerViewState::Closed);
    }

    #[test]
    fn 確認ダイアログ中のキャンセルは一覧表示へ戻る() {
        let mut picker = CommandPicker::new();

        picker.open(vec![destructive_candidate()]);
        let _ = picker.request_execute_selected();

        assert_eq!(picker.cancel(), PickerCancelOutcome::BackToListing);
        assert_eq!(picker.snapshot().state, PickerViewState::Listing);
    }
}
