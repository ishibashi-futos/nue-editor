use super::{
    CommandActionError, CommandActionEvent, CommandActionOutcome, CommandHubActionModel,
    ParsedCommand, tab_error_to_action_error, tab_index_by_target,
};

impl CommandHubActionModel {
    pub(super) fn execute_tab_pin(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        match self.resolve_tab_id(&command.target) {
            Ok(tab_id) => match self.tab_manager.pin(&tab_id) {
                Ok(()) => CommandActionOutcome::Executed(CommandActionEvent::TabPinned { tab_id }),
                Err(error) => {
                    CommandActionOutcome::Failed(tab_error_to_action_error(&tab_id, error))
                }
            },
            Err(error) => CommandActionOutcome::Failed(error),
        }
    }

    pub(super) fn execute_tab_unpin(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        match self.resolve_tab_id(&command.target) {
            Ok(tab_id) => match self.tab_manager.unpin(&tab_id) {
                Ok(()) => {
                    CommandActionOutcome::Executed(CommandActionEvent::TabUnpinned { tab_id })
                }
                Err(error) => {
                    CommandActionOutcome::Failed(tab_error_to_action_error(&tab_id, error))
                }
            },
            Err(error) => CommandActionOutcome::Failed(error),
        }
    }

    pub(super) fn execute_tab_close(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        match self.resolve_tab_id(&command.target) {
            Ok(tab_id) => match self.tab_manager.close_tab(&tab_id) {
                Ok(()) => CommandActionOutcome::Executed(CommandActionEvent::TabClosed { tab_id }),
                Err(error) => {
                    CommandActionOutcome::Failed(tab_error_to_action_error(&tab_id, error))
                }
            },
            Err(error) => CommandActionOutcome::Failed(error),
        }
    }

    pub(super) fn execute_tab_close_others(
        &mut self,
        command: &ParsedCommand,
    ) -> CommandActionOutcome {
        match self.resolve_tab_id(&command.target) {
            Ok(tab_id) => match self.tab_manager.close_others(&tab_id) {
                Ok(()) => {
                    CommandActionOutcome::Executed(CommandActionEvent::TabClosedOthers { tab_id })
                }
                Err(error) => {
                    CommandActionOutcome::Failed(tab_error_to_action_error(&tab_id, error))
                }
            },
            Err(error) => CommandActionOutcome::Failed(error),
        }
    }

    pub(super) fn execute_tab_close_to_right(
        &mut self,
        command: &ParsedCommand,
    ) -> CommandActionOutcome {
        match self.resolve_tab_id(&command.target) {
            Ok(tab_id) => match self.tab_manager.close_to_right(&tab_id) {
                Ok(()) => {
                    CommandActionOutcome::Executed(CommandActionEvent::TabClosedToRight { tab_id })
                }
                Err(error) => {
                    CommandActionOutcome::Failed(tab_error_to_action_error(&tab_id, error))
                }
            },
            Err(error) => CommandActionOutcome::Failed(error),
        }
    }

    pub(super) fn execute_tab_reopen(&mut self, _command: &ParsedCommand) -> CommandActionOutcome {
        match self.tab_manager.reopen_last_closed() {
            Ok(tab_id) => {
                CommandActionOutcome::Executed(CommandActionEvent::TabReopened { tab_id })
            }
            Err(error) => CommandActionOutcome::Failed(tab_error_to_action_error("", error)),
        }
    }

    pub(super) fn execute_tab_reorder(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        let raw_target = command.target.trim();
        if raw_target.is_empty() {
            return CommandActionOutcome::Failed(CommandActionError::MissingTarget {
                domain: command.domain.clone(),
                verb: command.verb.clone(),
            });
        }
        let parts: Vec<&str> = raw_target.split_whitespace().collect();
        if parts.len() < 2 {
            return CommandActionOutcome::Failed(CommandActionError::InvalidTarget {
                reason: "タブIDと移動先インデックスを空白区切りで指定してください".to_string(),
            });
        }
        let index_token = parts.last().expect("partsは2件以上");
        let tab_descriptor = parts[..parts.len() - 1].join(" ").trim().to_string();
        if tab_descriptor.is_empty() {
            return CommandActionOutcome::Failed(CommandActionError::MissingTarget {
                domain: command.domain.clone(),
                verb: command.verb.clone(),
            });
        }

        let new_index = match index_token.parse::<usize>() {
            Ok(value) => value,
            Err(_) => {
                return CommandActionOutcome::Failed(CommandActionError::InvalidTarget {
                    reason: "インデックスは非負整数でなければなりません".to_string(),
                });
            }
        };

        match self.resolve_tab_id(&tab_descriptor) {
            Ok(tab_id) => match self.tab_manager.reorder(&tab_id, new_index) {
                Ok(()) => CommandActionOutcome::Executed(CommandActionEvent::TabReordered {
                    tab_id,
                    new_index,
                }),
                Err(error) => {
                    CommandActionOutcome::Failed(tab_error_to_action_error(&tab_id, error))
                }
            },
            Err(error) => CommandActionOutcome::Failed(error),
        }
    }

    fn resolve_tab_id(&self, target: &str) -> Result<String, CommandActionError> {
        let trimmed = target.trim();
        if trimmed.is_empty() {
            return self
                .tab_manager
                .active_tab_id()
                .map(|id| id.to_string())
                .ok_or_else(|| CommandActionError::NotFound {
                    resource: "tab",
                    target: "active".to_string(),
                });
        }

        let tabs = self.tab_manager.tabs();
        let Some(index) = tab_index_by_target(&tabs, trimmed) else {
            return Err(CommandActionError::NotFound {
                resource: "tab",
                target: trimmed.to_string(),
            });
        };

        Ok(tabs[index].id.clone())
    }
}
