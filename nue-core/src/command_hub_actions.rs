use crate::command_hub::{
    CommandHubSession, CommandMode, ParsedCommand, PickerCancelOutcome, PickerCandidate,
    PickerExecuteOutcome,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelTarget {
    Explorer,
    GlobalSearch,
    Vcs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneSplitDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceItem {
    pub id: String,
    pub display_name: String,
    pub root_path: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneItem {
    pub id: String,
    pub title: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalItem {
    pub id: String,
    pub title: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectedOpenTarget {
    Url(String),
    Path(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandActionEvent {
    WorkspaceAdded {
        workspace_id: String,
    },
    WorkspaceActivated {
        workspace_id: String,
    },
    WorkspaceRemoved {
        workspace_id: String,
    },
    PaneSplit {
        pane_id: String,
        direction: PaneSplitDirection,
    },
    PaneActivated {
        pane_id: String,
    },
    PaneClosed {
        pane_id: String,
    },
    PanelFocused {
        panel: PanelTarget,
    },
    TerminalCreated {
        terminal_id: String,
    },
    TerminalActivated {
        terminal_id: String,
    },
    TerminalSplit {
        terminal_id: String,
    },
    TerminalClosed {
        terminal_id: String,
    },
    SelectedOpened {
        target: SelectedOpenTarget,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandActionError {
    UnsupportedCommand {
        domain: String,
        verb: String,
    },
    MissingTarget {
        domain: String,
        verb: String,
    },
    NotFound {
        resource: &'static str,
        target: String,
    },
    NoSelection,
    InvalidTarget {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandActionOutcome {
    Executed(CommandActionEvent),
    Failed(CommandActionError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandHubDispatchOutcome {
    NoSelection,
    NeedsConfirmation { candidate_id: String },
    Executed(CommandActionEvent),
    Failed(CommandActionError),
    Closed { candidate_id: Option<String> },
    BackToListing { candidate_id: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandHubActionModel {
    workspaces: Vec<WorkspaceItem>,
    panes: Vec<PaneItem>,
    terminals: Vec<TerminalItem>,
    selected_text: Option<String>,
    focused_panel: Option<PanelTarget>,
}

impl CommandHubActionModel {
    pub fn new(
        workspaces: Vec<WorkspaceItem>,
        panes: Vec<PaneItem>,
        terminals: Vec<TerminalItem>,
    ) -> Self {
        Self {
            workspaces,
            panes,
            terminals,
            selected_text: None,
            focused_panel: None,
        }
    }

    pub fn set_selected_text(&mut self, selected_text: Option<String>) {
        self.selected_text = selected_text;
    }

    pub fn workspaces(&self) -> &[WorkspaceItem] {
        &self.workspaces
    }

    pub fn panes(&self) -> &[PaneItem] {
        &self.panes
    }

    pub fn terminals(&self) -> &[TerminalItem] {
        &self.terminals
    }

    pub fn focused_panel(&self) -> Option<PanelTarget> {
        self.focused_panel
    }

    pub fn candidates_for(&self, command: &ParsedCommand) -> Vec<PickerCandidate> {
        if command.mode != CommandMode::Action {
            return Vec::new();
        }

        match (command.domain.as_str(), command.verb.as_str()) {
            ("workspace", "list") => {
                workspace_candidates_for_target(command, &self.workspaces, "open", false)
            }
            ("workspace", "remove") => {
                workspace_candidates_for_target(command, &self.workspaces, "remove", true)
            }
            ("pane", "list") => pane_candidates_for_target(command, &self.panes, "open"),
            ("pane", "close") => pane_candidates_for_target(command, &self.panes, "close"),
            ("terminal", "list") => {
                terminal_candidates_for_target(command, &self.terminals, "open", false)
            }
            ("terminal", "close") | ("terminal", "kill") => {
                terminal_candidates_for_target(command, &self.terminals, "close", true)
            }
            _ => Vec::new(),
        }
    }

    pub fn execute(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        if command.mode != CommandMode::Action {
            return unsupported_command(command);
        }

        match (command.domain.as_str(), command.verb.as_str()) {
            ("workspace", "add") => self.execute_workspace_add(command),
            ("workspace", "open") | ("workspace", "activate") => {
                self.execute_workspace_activate(command)
            }
            ("workspace", "remove") => self.execute_workspace_remove(command),
            ("pane", "split") => self.execute_pane_split(command),
            ("pane", "next") => self.execute_pane_next(),
            ("pane", "prev") => self.execute_pane_prev(),
            ("pane", "close") => self.execute_pane_close(command),
            ("pane", "open") | ("pane", "activate") => self.execute_pane_activate(command),
            ("panel", "focus") => self.execute_panel_focus(command.target.as_str()),
            ("panel", verb) if verb.starts_with("focus_") => {
                let target = &verb["focus_".len()..];
                self.execute_panel_focus(target)
            }
            ("terminal", "new") => self.execute_terminal_new(),
            ("terminal", "open") | ("terminal", "activate") => {
                self.execute_terminal_activate(command)
            }
            ("terminal", "split") => self.execute_terminal_split(),
            ("terminal", "close") | ("terminal", "kill") => self.execute_terminal_close(command),
            ("selected", "open") => self.execute_selected_open(command),
            _ => unsupported_command(command),
        }
    }

    fn execute_workspace_add(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        let root_path = command.target.trim();
        if root_path.is_empty() {
            return CommandActionOutcome::Failed(CommandActionError::MissingTarget {
                domain: command.domain.clone(),
                verb: command.verb.clone(),
            });
        }

        let workspace_id = next_sequential_id(&self.workspaces, "workspace", |workspace| {
            workspace.id.as_str()
        });
        let display_name = workspace_display_name(root_path);

        for workspace in &mut self.workspaces {
            workspace.is_active = false;
        }
        self.workspaces.push(WorkspaceItem {
            id: workspace_id.clone(),
            display_name,
            root_path: root_path.to_string(),
            is_active: true,
        });

        CommandActionOutcome::Executed(CommandActionEvent::WorkspaceAdded { workspace_id })
    }

    fn execute_workspace_activate(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        let target = command.target.trim();
        if target.is_empty() {
            return CommandActionOutcome::Failed(CommandActionError::MissingTarget {
                domain: command.domain.clone(),
                verb: command.verb.clone(),
            });
        }

        let Some(index) = workspace_index_by_target(&self.workspaces, target) else {
            return CommandActionOutcome::Failed(CommandActionError::NotFound {
                resource: "workspace",
                target: target.to_string(),
            });
        };

        for workspace in &mut self.workspaces {
            workspace.is_active = false;
        }
        let workspace_id = self.workspaces[index].id.clone();
        self.workspaces[index].is_active = true;

        CommandActionOutcome::Executed(CommandActionEvent::WorkspaceActivated { workspace_id })
    }

    fn execute_workspace_remove(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        let target = command.target.trim();
        if target.is_empty() {
            return CommandActionOutcome::Failed(CommandActionError::MissingTarget {
                domain: command.domain.clone(),
                verb: command.verb.clone(),
            });
        }

        let Some(index) = workspace_index_by_target(&self.workspaces, target) else {
            return CommandActionOutcome::Failed(CommandActionError::NotFound {
                resource: "workspace",
                target: target.to_string(),
            });
        };

        let removed = self.workspaces.remove(index);
        if removed.is_active && !self.workspaces.is_empty() {
            self.workspaces[0].is_active = true;
        }

        CommandActionOutcome::Executed(CommandActionEvent::WorkspaceRemoved {
            workspace_id: removed.id,
        })
    }

    fn execute_pane_split(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        let Some(direction) = parse_pane_split_direction(command.target.as_str()) else {
            return CommandActionOutcome::Failed(CommandActionError::InvalidTarget {
                reason: "pane split direction は left/right/up/down のいずれかです".to_string(),
            });
        };
        let Some(active_index) = active_index(&self.panes, |pane| pane.is_active) else {
            return CommandActionOutcome::Failed(CommandActionError::NotFound {
                resource: "pane",
                target: "active".to_string(),
            });
        };

        let source_title = self.panes[active_index].title.clone();
        let pane_id = next_sequential_id(&self.panes, "pane", |pane| pane.id.as_str());
        for pane in &mut self.panes {
            pane.is_active = false;
        }
        self.panes.push(PaneItem {
            id: pane_id.clone(),
            title: source_title,
            is_active: true,
        });

        CommandActionOutcome::Executed(CommandActionEvent::PaneSplit { pane_id, direction })
    }

    fn execute_pane_next(&mut self) -> CommandActionOutcome {
        if self.panes.is_empty() {
            return CommandActionOutcome::Failed(CommandActionError::NotFound {
                resource: "pane",
                target: "active".to_string(),
            });
        }

        let current = active_index(&self.panes, |pane| pane.is_active).unwrap_or(0);
        let next = (current + 1) % self.panes.len();
        for pane in &mut self.panes {
            pane.is_active = false;
        }
        let pane_id = self.panes[next].id.clone();
        self.panes[next].is_active = true;

        CommandActionOutcome::Executed(CommandActionEvent::PaneActivated { pane_id })
    }

    fn execute_pane_prev(&mut self) -> CommandActionOutcome {
        if self.panes.is_empty() {
            return CommandActionOutcome::Failed(CommandActionError::NotFound {
                resource: "pane",
                target: "active".to_string(),
            });
        }

        let current = active_index(&self.panes, |pane| pane.is_active).unwrap_or(0);
        let prev = if current == 0 {
            self.panes.len() - 1
        } else {
            current - 1
        };
        for pane in &mut self.panes {
            pane.is_active = false;
        }
        let pane_id = self.panes[prev].id.clone();
        self.panes[prev].is_active = true;

        CommandActionOutcome::Executed(CommandActionEvent::PaneActivated { pane_id })
    }

    fn execute_pane_close(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        let target = command.target.trim();
        let index = if target.is_empty() {
            let Some(active) = active_index(&self.panes, |pane| pane.is_active) else {
                return CommandActionOutcome::Failed(CommandActionError::NotFound {
                    resource: "pane",
                    target: "active".to_string(),
                });
            };
            active
        } else {
            let Some(found) = pane_index_by_target(&self.panes, target) else {
                return CommandActionOutcome::Failed(CommandActionError::NotFound {
                    resource: "pane",
                    target: target.to_string(),
                });
            };
            found
        };

        let removed = self.panes.remove(index);
        if removed.is_active && !self.panes.is_empty() {
            let replacement_index = index.min(self.panes.len() - 1);
            for pane in &mut self.panes {
                pane.is_active = false;
            }
            self.panes[replacement_index].is_active = true;
        }

        CommandActionOutcome::Executed(CommandActionEvent::PaneClosed {
            pane_id: removed.id,
        })
    }

    fn execute_pane_activate(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        let target = command.target.trim();
        if target.is_empty() {
            return CommandActionOutcome::Failed(CommandActionError::MissingTarget {
                domain: command.domain.clone(),
                verb: command.verb.clone(),
            });
        }

        let Some(index) = pane_index_by_target(&self.panes, target) else {
            return CommandActionOutcome::Failed(CommandActionError::NotFound {
                resource: "pane",
                target: target.to_string(),
            });
        };

        for pane in &mut self.panes {
            pane.is_active = false;
        }
        let pane_id = self.panes[index].id.clone();
        self.panes[index].is_active = true;

        CommandActionOutcome::Executed(CommandActionEvent::PaneActivated { pane_id })
    }

    fn execute_panel_focus(&mut self, target: &str) -> CommandActionOutcome {
        let Some(panel) = parse_panel_target(target) else {
            return CommandActionOutcome::Failed(CommandActionError::InvalidTarget {
                reason: "panel focus target は explorer/global search/vcs のいずれかです"
                    .to_string(),
            });
        };

        self.focused_panel = Some(panel);
        CommandActionOutcome::Executed(CommandActionEvent::PanelFocused { panel })
    }

    fn execute_terminal_new(&mut self) -> CommandActionOutcome {
        let terminal_id =
            next_sequential_id(&self.terminals, "terminal", |terminal| terminal.id.as_str());
        let title = format!("terminal-{}", self.terminals.len() + 1);
        for terminal in &mut self.terminals {
            terminal.is_active = false;
        }
        self.terminals.push(TerminalItem {
            id: terminal_id.clone(),
            title,
            is_active: true,
        });

        CommandActionOutcome::Executed(CommandActionEvent::TerminalCreated { terminal_id })
    }

    fn execute_terminal_activate(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        let target = command.target.trim();
        if target.is_empty() {
            return CommandActionOutcome::Failed(CommandActionError::MissingTarget {
                domain: command.domain.clone(),
                verb: command.verb.clone(),
            });
        }

        let Some(index) = terminal_index_by_target(&self.terminals, target) else {
            return CommandActionOutcome::Failed(CommandActionError::NotFound {
                resource: "terminal",
                target: target.to_string(),
            });
        };

        for terminal in &mut self.terminals {
            terminal.is_active = false;
        }
        let terminal_id = self.terminals[index].id.clone();
        self.terminals[index].is_active = true;

        CommandActionOutcome::Executed(CommandActionEvent::TerminalActivated { terminal_id })
    }

    fn execute_terminal_split(&mut self) -> CommandActionOutcome {
        let Some(active_index) = active_index(&self.terminals, |terminal| terminal.is_active)
        else {
            return CommandActionOutcome::Failed(CommandActionError::NotFound {
                resource: "terminal",
                target: "active".to_string(),
            });
        };
        let title = self.terminals[active_index].title.clone();
        let terminal_id =
            next_sequential_id(&self.terminals, "terminal", |terminal| terminal.id.as_str());

        for terminal in &mut self.terminals {
            terminal.is_active = false;
        }
        self.terminals.push(TerminalItem {
            id: terminal_id.clone(),
            title,
            is_active: true,
        });

        CommandActionOutcome::Executed(CommandActionEvent::TerminalSplit { terminal_id })
    }

    fn execute_terminal_close(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        let target = command.target.trim();
        let index = if target.is_empty() {
            let Some(active) = active_index(&self.terminals, |terminal| terminal.is_active) else {
                return CommandActionOutcome::Failed(CommandActionError::NotFound {
                    resource: "terminal",
                    target: "active".to_string(),
                });
            };
            active
        } else {
            let Some(found) = terminal_index_by_target(&self.terminals, target) else {
                return CommandActionOutcome::Failed(CommandActionError::NotFound {
                    resource: "terminal",
                    target: target.to_string(),
                });
            };
            found
        };

        let removed = self.terminals.remove(index);
        if removed.is_active && !self.terminals.is_empty() {
            let replacement_index = index.min(self.terminals.len() - 1);
            for terminal in &mut self.terminals {
                terminal.is_active = false;
            }
            self.terminals[replacement_index].is_active = true;
        }

        CommandActionOutcome::Executed(CommandActionEvent::TerminalClosed {
            terminal_id: removed.id,
        })
    }

    fn execute_selected_open(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        let source = if command.target.trim().is_empty() {
            self.selected_text.clone().unwrap_or_default()
        } else {
            command.target.clone()
        };
        if source.trim().is_empty() {
            return CommandActionOutcome::Failed(CommandActionError::NoSelection);
        }

        match resolve_selected_open_target(source.trim()) {
            Ok(target) => {
                CommandActionOutcome::Executed(CommandActionEvent::SelectedOpened { target })
            }
            Err(reason) => {
                CommandActionOutcome::Failed(CommandActionError::InvalidTarget { reason })
            }
        }
    }
}

pub fn dispatch_selected_action(
    session: &mut CommandHubSession,
    model: &mut CommandHubActionModel,
) -> CommandHubDispatchOutcome {
    dispatch_execute_outcome(session.execute_selected_candidate(), model)
}

pub fn dispatch_confirmed_action(
    session: &mut CommandHubSession,
    model: &mut CommandHubActionModel,
) -> CommandHubDispatchOutcome {
    dispatch_execute_outcome(session.confirm_selected_candidate(), model)
}

pub fn dispatch_cancel_action(session: &mut CommandHubSession) -> CommandHubDispatchOutcome {
    match session.cancel_picker() {
        PickerCancelOutcome::Noop => CommandHubDispatchOutcome::NoSelection,
        PickerCancelOutcome::Closed { candidate_id } => {
            CommandHubDispatchOutcome::Closed { candidate_id }
        }
        PickerCancelOutcome::BackToListing { candidate_id } => {
            CommandHubDispatchOutcome::BackToListing { candidate_id }
        }
    }
}

fn dispatch_execute_outcome(
    picker_outcome: PickerExecuteOutcome,
    model: &mut CommandHubActionModel,
) -> CommandHubDispatchOutcome {
    match picker_outcome {
        PickerExecuteOutcome::NoSelection => CommandHubDispatchOutcome::NoSelection,
        PickerExecuteOutcome::NeedsConfirmation { candidate_id } => {
            CommandHubDispatchOutcome::NeedsConfirmation { candidate_id }
        }
        PickerExecuteOutcome::Executed(candidate) => match model.execute(&candidate.command) {
            CommandActionOutcome::Executed(event) => CommandHubDispatchOutcome::Executed(event),
            CommandActionOutcome::Failed(error) => CommandHubDispatchOutcome::Failed(error),
        },
    }
}

fn unsupported_command(command: &ParsedCommand) -> CommandActionOutcome {
    CommandActionOutcome::Failed(CommandActionError::UnsupportedCommand {
        domain: command.domain.clone(),
        verb: command.verb.clone(),
    })
}

fn workspace_candidates(
    workspaces: &[WorkspaceItem],
    verb: &str,
    requires_confirmation: bool,
) -> Vec<PickerCandidate> {
    workspaces
        .iter()
        .map(|workspace| PickerCandidate {
            id: format!("workspace::{}", workspace.id),
            label: format!("Workspace: {}", workspace.display_name),
            command: ParsedCommand {
                mode: CommandMode::Action,
                domain: "workspace".to_string(),
                verb: verb.to_string(),
                target: workspace.id.clone(),
            },
            requires_confirmation,
        })
        .collect()
}

fn workspace_candidates_for_target(
    command: &ParsedCommand,
    workspaces: &[WorkspaceItem],
    verb: &str,
    requires_confirmation: bool,
) -> Vec<PickerCandidate> {
    if command.target.trim().is_empty() {
        return workspace_candidates(workspaces, verb, requires_confirmation);
    }

    let Some(index) = workspace_index_by_target(workspaces, command.target.as_str()) else {
        return Vec::new();
    };
    workspace_candidates(&workspaces[index..index + 1], verb, requires_confirmation)
}

fn pane_candidates(panes: &[PaneItem], verb: &str) -> Vec<PickerCandidate> {
    panes
        .iter()
        .map(|pane| PickerCandidate {
            id: format!("pane::{}", pane.id),
            label: format!("Pane: {}", pane.title),
            command: ParsedCommand {
                mode: CommandMode::Action,
                domain: "pane".to_string(),
                verb: verb.to_string(),
                target: pane.id.clone(),
            },
            requires_confirmation: false,
        })
        .collect()
}

fn pane_candidates_for_target(
    command: &ParsedCommand,
    panes: &[PaneItem],
    verb: &str,
) -> Vec<PickerCandidate> {
    if command.target.trim().is_empty() {
        return pane_candidates(panes, verb);
    }

    let Some(index) = pane_index_by_target(panes, command.target.as_str()) else {
        return Vec::new();
    };
    pane_candidates(&panes[index..index + 1], verb)
}

fn terminal_candidates(
    terminals: &[TerminalItem],
    verb: &str,
    requires_confirmation: bool,
) -> Vec<PickerCandidate> {
    terminals
        .iter()
        .map(|terminal| PickerCandidate {
            id: format!("terminal::{}", terminal.id),
            label: format!("Terminal: {}", terminal.title),
            command: ParsedCommand {
                mode: CommandMode::Action,
                domain: "terminal".to_string(),
                verb: verb.to_string(),
                target: terminal.id.clone(),
            },
            requires_confirmation,
        })
        .collect()
}

fn terminal_candidates_for_target(
    command: &ParsedCommand,
    terminals: &[TerminalItem],
    verb: &str,
    requires_confirmation: bool,
) -> Vec<PickerCandidate> {
    if command.target.trim().is_empty() {
        return terminal_candidates(terminals, verb, requires_confirmation);
    }

    let Some(index) = terminal_index_by_target(terminals, command.target.as_str()) else {
        return Vec::new();
    };
    terminal_candidates(&terminals[index..index + 1], verb, requires_confirmation)
}

fn workspace_display_name(root_path: &str) -> String {
    root_path
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|segment| !segment.is_empty())
        .unwrap_or(root_path)
        .to_string()
}

fn next_sequential_id<T>(items: &[T], prefix: &str, id_accessor: impl Fn(&T) -> &str) -> String {
    let mut max_number = 0_u64;
    for item in items {
        let id = id_accessor(item);
        let Some(number) = id
            .strip_prefix(prefix)
            .and_then(|rest| rest.strip_prefix('-'))
            .and_then(|value| value.parse::<u64>().ok())
        else {
            continue;
        };
        max_number = max_number.max(number);
    }

    format!("{prefix}-{}", max_number + 1)
}

fn active_index<T>(items: &[T], is_active: impl Fn(&T) -> bool) -> Option<usize> {
    items.iter().position(is_active)
}

fn normalized_lookup(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

fn workspace_index_by_target(workspaces: &[WorkspaceItem], target: &str) -> Option<usize> {
    let normalized_target = normalized_lookup(target);
    workspaces.iter().position(|workspace| {
        normalized_lookup(workspace.id.as_str()) == normalized_target
            || normalized_lookup(workspace.display_name.as_str()) == normalized_target
            || normalized_lookup(workspace.root_path.as_str()) == normalized_target
    })
}

fn pane_index_by_target(panes: &[PaneItem], target: &str) -> Option<usize> {
    let normalized_target = normalized_lookup(target);
    panes.iter().position(|pane| {
        normalized_lookup(pane.id.as_str()) == normalized_target
            || normalized_lookup(pane.title.as_str()) == normalized_target
    })
}

fn terminal_index_by_target(terminals: &[TerminalItem], target: &str) -> Option<usize> {
    let normalized_target = normalized_lookup(target);
    terminals.iter().position(|terminal| {
        normalized_lookup(terminal.id.as_str()) == normalized_target
            || normalized_lookup(terminal.title.as_str()) == normalized_target
    })
}

fn parse_pane_split_direction(value: &str) -> Option<PaneSplitDirection> {
    let token = normalized_lookup(value);
    let first = token.split_whitespace().next().unwrap_or("");
    match first {
        "left" => Some(PaneSplitDirection::Left),
        "right" => Some(PaneSplitDirection::Right),
        "up" => Some(PaneSplitDirection::Up),
        "down" => Some(PaneSplitDirection::Down),
        _ => None,
    }
}

fn parse_panel_target(value: &str) -> Option<PanelTarget> {
    let target = normalized_lookup(value);
    match target.as_str() {
        "explorer" => Some(PanelTarget::Explorer),
        "global search" | "global_search" | "globalsearch" => Some(PanelTarget::GlobalSearch),
        "vcs" => Some(PanelTarget::Vcs),
        _ => None,
    }
}

fn resolve_selected_open_target(value: &str) -> Result<SelectedOpenTarget, String> {
    if value.is_empty() {
        return Err("選択文字列が空です".to_string());
    }
    if value.starts_with("https://") {
        return Ok(SelectedOpenTarget::Url(value.to_string()));
    }
    if value.contains("://") {
        return Err("https:// 以外のURLスキームは開けません".to_string());
    }

    Ok(SelectedOpenTarget::Path(value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_hub::{CommandHubSession, PickerCandidate};

    fn action_command(domain: &str, verb: &str, target: &str) -> ParsedCommand {
        ParsedCommand {
            mode: CommandMode::Action,
            domain: domain.to_string(),
            verb: verb.to_string(),
            target: target.to_string(),
        }
    }

    fn model() -> CommandHubActionModel {
        CommandHubActionModel::new(
            vec![
                WorkspaceItem {
                    id: "workspace-1".to_string(),
                    display_name: "Nue".to_string(),
                    root_path: "/work/nue".to_string(),
                    is_active: true,
                },
                WorkspaceItem {
                    id: "workspace-2".to_string(),
                    display_name: "Docs".to_string(),
                    root_path: "/work/docs".to_string(),
                    is_active: false,
                },
            ],
            vec![
                PaneItem {
                    id: "pane-1".to_string(),
                    title: "README.md".to_string(),
                    is_active: true,
                },
                PaneItem {
                    id: "pane-2".to_string(),
                    title: "specs/spec-nue.md".to_string(),
                    is_active: false,
                },
            ],
            vec![TerminalItem {
                id: "terminal-1".to_string(),
                title: "zsh".to_string(),
                is_active: true,
            }],
        )
    }

    fn destructive_workspace_candidate(target: &str) -> PickerCandidate {
        PickerCandidate {
            id: format!("workspace::{target}"),
            label: format!("Workspace: {target}"),
            command: action_command("workspace", "remove", target),
            requires_confirmation: true,
        }
    }

    #[test]
    fn workspace_listは候補を返しadd_removeを実行できる() {
        let mut model = model();
        let list_command = action_command("workspace", "list", "");
        let add_command = action_command("workspace", "add", "/work/new-project");
        let remove_command = action_command("workspace", "remove", "workspace-2");

        let candidates = model.candidates_for(&list_command);
        assert_eq!(candidates.len(), 2);
        assert_eq!(
            candidates[0].command,
            action_command("workspace", "open", "workspace-1")
        );

        assert_eq!(
            model.execute(&add_command),
            CommandActionOutcome::Executed(CommandActionEvent::WorkspaceAdded {
                workspace_id: "workspace-3".to_string(),
            })
        );
        assert_eq!(model.workspaces().len(), 3);

        assert_eq!(
            model.execute(&remove_command),
            CommandActionOutcome::Executed(CommandActionEvent::WorkspaceRemoved {
                workspace_id: "workspace-2".to_string(),
            })
        );
        assert_eq!(model.workspaces().len(), 2);
    }

    #[test]
    fn paneコマンドはsplit_next_prev_close_listを実行できる() {
        let mut model = model();

        assert_eq!(
            model.execute(&action_command("pane", "split", "right")),
            CommandActionOutcome::Executed(CommandActionEvent::PaneSplit {
                pane_id: "pane-3".to_string(),
                direction: PaneSplitDirection::Right,
            })
        );
        assert_eq!(model.panes().len(), 3);

        assert_eq!(
            model.execute(&action_command("pane", "next", "")),
            CommandActionOutcome::Executed(CommandActionEvent::PaneActivated {
                pane_id: "pane-1".to_string(),
            })
        );
        assert_eq!(
            model.execute(&action_command("pane", "prev", "")),
            CommandActionOutcome::Executed(CommandActionEvent::PaneActivated {
                pane_id: "pane-3".to_string(),
            })
        );

        let close_candidates = model.candidates_for(&action_command("pane", "close", ""));
        assert_eq!(close_candidates.len(), 3);

        assert_eq!(
            model.execute(&action_command("pane", "close", "pane-2")),
            CommandActionOutcome::Executed(CommandActionEvent::PaneClosed {
                pane_id: "pane-2".to_string(),
            })
        );
        assert_eq!(model.panes().len(), 2);
    }

    #[test]
    fn panel_terminal_selectedコマンドを実行できる() {
        let mut model = model();
        model.set_selected_text(Some("https://example.com/docs".to_string()));

        assert_eq!(
            model.execute(&action_command("panel", "focus", "global search")),
            CommandActionOutcome::Executed(CommandActionEvent::PanelFocused {
                panel: PanelTarget::GlobalSearch,
            })
        );
        assert_eq!(model.focused_panel(), Some(PanelTarget::GlobalSearch));

        assert_eq!(
            model.execute(&action_command("terminal", "new", "")),
            CommandActionOutcome::Executed(CommandActionEvent::TerminalCreated {
                terminal_id: "terminal-2".to_string(),
            })
        );
        assert_eq!(model.terminals().len(), 2);

        assert_eq!(
            model.execute(&action_command("terminal", "close", "terminal-1")),
            CommandActionOutcome::Executed(CommandActionEvent::TerminalClosed {
                terminal_id: "terminal-1".to_string(),
            })
        );
        assert_eq!(model.terminals().len(), 1);

        assert_eq!(
            model.execute(&action_command("selected", "open", "")),
            CommandActionOutcome::Executed(CommandActionEvent::SelectedOpened {
                target: SelectedOpenTarget::Url("https://example.com/docs".to_string()),
            })
        );
    }

    #[test]
    fn 破壊的コマンド候補はtarget指定時に対象のみに絞り込む() {
        let model = model();

        let workspace_candidates =
            model.candidates_for(&action_command("workspace", "remove", "workspace-2"));
        assert_eq!(workspace_candidates.len(), 1);
        assert_eq!(
            workspace_candidates[0].command,
            action_command("workspace", "remove", "workspace-2")
        );

        let pane_candidates = model.candidates_for(&action_command("pane", "close", "pane-2"));
        assert_eq!(pane_candidates.len(), 1);
        assert_eq!(
            pane_candidates[0].command,
            action_command("pane", "close", "pane-2")
        );

        let terminal_candidates =
            model.candidates_for(&action_command("terminal", "kill", "terminal-1"));
        assert_eq!(terminal_candidates.len(), 1);
        assert_eq!(
            terminal_candidates[0].command,
            action_command("terminal", "close", "terminal-1")
        );
    }

    #[test]
    fn selected_openは解決不能時に理由付きエラーを返す() {
        let mut model = model();
        model.set_selected_text(Some("ftp://example.com/file".to_string()));

        assert_eq!(
            model.execute(&action_command("selected", "open", "")),
            CommandActionOutcome::Failed(CommandActionError::InvalidTarget {
                reason: "https:// 以外のURLスキームは開けません".to_string(),
            })
        );
    }

    #[test]
    fn dispatchは確認必須候補のキャンセルを通知する() {
        let mut model = model();
        let mut session = CommandHubSession::new();
        let candidate = destructive_workspace_candidate("workspace-2");
        session.apply_input("> Workspace: Remove", vec![candidate.clone()]);

        assert_eq!(
            dispatch_selected_action(&mut session, &mut model),
            CommandHubDispatchOutcome::NeedsConfirmation {
                candidate_id: candidate.id.clone(),
            }
        );
        assert_eq!(
            dispatch_cancel_action(&mut session),
            CommandHubDispatchOutcome::BackToListing {
                candidate_id: Some(candidate.id),
            }
        );
    }

    #[test]
    fn dispatchは一覧表示中キャンセルをclosedとして通知する() {
        let mut session = CommandHubSession::new();
        let candidate = destructive_workspace_candidate("workspace-2");
        session.apply_input("> Workspace: Remove", vec![candidate.clone()]);

        assert_eq!(
            dispatch_cancel_action(&mut session),
            CommandHubDispatchOutcome::Closed {
                candidate_id: Some(candidate.id),
            }
        );
    }

    #[test]
    fn dispatchは確認後の失敗を通知する() {
        let mut model = model();
        let mut session = CommandHubSession::new();
        let candidate = destructive_workspace_candidate("workspace-999");
        session.apply_input("> Workspace: Remove", vec![candidate.clone()]);
        let _ = dispatch_selected_action(&mut session, &mut model);

        assert_eq!(
            dispatch_confirmed_action(&mut session, &mut model),
            CommandHubDispatchOutcome::Failed(CommandActionError::NotFound {
                resource: "workspace",
                target: "workspace-999".to_string(),
            })
        );
    }
}
