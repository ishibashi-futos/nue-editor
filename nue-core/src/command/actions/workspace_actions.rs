use super::{
    CommandActionError, CommandActionEvent, CommandActionOutcome, CommandHubActionModel,
    ParsedCommand, WorkspaceItem, next_sequential_id, workspace_index_by_target,
};

impl CommandHubActionModel {
    pub(super) fn execute_workspace_add(
        &mut self,
        command: &ParsedCommand,
    ) -> CommandActionOutcome {
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

    pub(super) fn execute_workspace_activate(
        &mut self,
        command: &ParsedCommand,
    ) -> CommandActionOutcome {
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

    pub(super) fn execute_workspace_remove(
        &mut self,
        command: &ParsedCommand,
    ) -> CommandActionOutcome {
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
