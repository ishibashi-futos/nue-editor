use super::{
    CommandActionError, CommandActionEvent, CommandActionOutcome, CommandHubActionModel,
    ParsedCommand, TerminalItem, active_index, next_sequential_id, terminal_index_by_target,
};

impl CommandHubActionModel {
    pub(super) fn execute_terminal_new(&mut self) -> CommandActionOutcome {
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

    pub(super) fn execute_terminal_activate(
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

    pub(super) fn execute_terminal_split(&mut self) -> CommandActionOutcome {
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

    pub(super) fn execute_terminal_close(
        &mut self,
        command: &ParsedCommand,
    ) -> CommandActionOutcome {
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
}
