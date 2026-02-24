use crate::command::actions::{TerminalItem, WorkspaceItem};
use crate::layout::pane_manager::PaneItem;
use crate::layout::tab_manager::TabSnapshot;

pub(super) fn normalized_lookup(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

pub(super) fn workspace_index_by_target(
    workspaces: &[WorkspaceItem],
    target: &str,
) -> Option<usize> {
    let normalized_target = normalized_lookup(target);
    workspaces.iter().position(|workspace| {
        normalized_lookup(workspace.id.as_str()) == normalized_target
            || normalized_lookup(workspace.display_name.as_str()) == normalized_target
            || normalized_lookup(workspace.root_path.as_str()) == normalized_target
    })
}

pub(super) fn pane_index_by_target(panes: &[PaneItem], target: &str) -> Option<usize> {
    let normalized_target = normalized_lookup(target);
    panes.iter().position(|pane| {
        normalized_lookup(pane.id.as_str()) == normalized_target
            || normalized_lookup(pane.title.as_str()) == normalized_target
    })
}

pub(super) fn terminal_index_by_target(terminals: &[TerminalItem], target: &str) -> Option<usize> {
    let normalized_target = normalized_lookup(target);
    terminals.iter().position(|terminal| {
        normalized_lookup(terminal.id.as_str()) == normalized_target
            || normalized_lookup(terminal.title.as_str()) == normalized_target
    })
}

pub(super) fn tab_index_by_target(tabs: &[TabSnapshot], target: &str) -> Option<usize> {
    let normalized_target = normalized_lookup(target);
    tabs.iter().position(|tab| {
        normalized_lookup(tab.id.as_str()) == normalized_target
            || normalized_lookup(tab.title.as_str()) == normalized_target
    })
}
