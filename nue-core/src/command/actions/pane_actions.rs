use super::{
    CommandActionError, CommandActionEvent, CommandActionOutcome, CommandHubActionModel,
    ParsedCommand, normalized_lookup, pane_index_by_target, pane_move_error_to_action_error,
};
use crate::layout::pane_manager::{PaneManagerError, PaneSplitDirection};

impl CommandHubActionModel {
    pub(super) fn execute_pane_split(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        let Some(direction) = parse_pane_split_direction(command.target.as_str()) else {
            return CommandActionOutcome::Failed(CommandActionError::InvalidTarget {
                reason: "pane split direction は left/right/up/down のいずれかです".to_string(),
            });
        };

        match self.pane_manager.split_active(direction) {
            Ok(pane_id) => {
                CommandActionOutcome::Executed(CommandActionEvent::PaneSplit { pane_id, direction })
            }
            Err(PaneManagerError::NoPanes) => {
                CommandActionOutcome::Failed(CommandActionError::NotFound {
                    resource: "pane",
                    target: "active".to_string(),
                })
            }
            Err(_) => CommandActionOutcome::Failed(CommandActionError::InvalidTarget {
                reason: "pane split に失敗しました".to_string(),
            }),
        }
    }

    pub(super) fn execute_pane_next(&mut self) -> CommandActionOutcome {
        match self.pane_manager.activate_next() {
            Ok(pane_id) => {
                CommandActionOutcome::Executed(CommandActionEvent::PaneActivated { pane_id })
            }
            Err(PaneManagerError::NoPanes) => {
                CommandActionOutcome::Failed(CommandActionError::NotFound {
                    resource: "pane",
                    target: "active".to_string(),
                })
            }
            Err(_) => CommandActionOutcome::Failed(CommandActionError::InvalidTarget {
                reason: "pane の活性化に失敗しました".to_string(),
            }),
        }
    }

    pub(super) fn execute_pane_prev(&mut self) -> CommandActionOutcome {
        match self.pane_manager.activate_prev() {
            Ok(pane_id) => {
                CommandActionOutcome::Executed(CommandActionEvent::PaneActivated { pane_id })
            }
            Err(PaneManagerError::NoPanes) => {
                CommandActionOutcome::Failed(CommandActionError::NotFound {
                    resource: "pane",
                    target: "active".to_string(),
                })
            }
            Err(_) => CommandActionOutcome::Failed(CommandActionError::InvalidTarget {
                reason: "pane の活性化に失敗しました".to_string(),
            }),
        }
    }

    pub(super) fn execute_pane_close(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        let target = command.target.trim();
        let pane_id = if target.is_empty() {
            match self.pane_manager.active_pane_id() {
                Some(id) => id.to_string(),
                None => {
                    return CommandActionOutcome::Failed(CommandActionError::NotFound {
                        resource: "pane",
                        target: "active".to_string(),
                    });
                }
            }
        } else {
            let panes = self.pane_manager.panes();
            let index = match pane_index_by_target(&panes, target) {
                Some(index) => index,
                None => {
                    return CommandActionOutcome::Failed(CommandActionError::NotFound {
                        resource: "pane",
                        target: target.to_string(),
                    });
                }
            };
            panes[index].id.clone()
        };

        match self.pane_manager.close(&pane_id) {
            Ok(()) => CommandActionOutcome::Executed(CommandActionEvent::PaneClosed { pane_id }),
            Err(PaneManagerError::PaneNotFound(_)) => {
                CommandActionOutcome::Failed(CommandActionError::NotFound {
                    resource: "pane",
                    target: pane_id,
                })
            }
            Err(PaneManagerError::OnlyOnePane) => {
                CommandActionOutcome::Failed(CommandActionError::InvalidTarget {
                    reason: "pane が1つしかないため閉じられません".to_string(),
                })
            }
            Err(_) => CommandActionOutcome::Failed(CommandActionError::InvalidTarget {
                reason: "pane の閉鎖に失敗しました".to_string(),
            }),
        }
    }

    pub(super) fn execute_pane_open(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        if let Some(side_title) = parse_pane_open_side_title(&command.target) {
            self.execute_pane_open_side(side_title)
        } else {
            self.execute_pane_activate(command)
        }
    }

    pub(super) fn execute_pane_activate(
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

        let panes = self.pane_manager.panes();
        let Some(index) = pane_index_by_target(&panes, target) else {
            return CommandActionOutcome::Failed(CommandActionError::NotFound {
                resource: "pane",
                target: target.to_string(),
            });
        };
        let pane_id = panes[index].id.clone();
        match self.pane_manager.activate(&pane_id) {
            Ok(pane_id) => {
                CommandActionOutcome::Executed(CommandActionEvent::PaneActivated { pane_id })
            }
            Err(PaneManagerError::PaneNotFound(_)) => {
                CommandActionOutcome::Failed(CommandActionError::NotFound {
                    resource: "pane",
                    target: pane_id,
                })
            }
            Err(_) => CommandActionOutcome::Failed(CommandActionError::InvalidTarget {
                reason: "pane の活性化に失敗しました".to_string(),
            }),
        }
    }

    pub(super) fn execute_pane_open_side(&mut self, title: String) -> CommandActionOutcome {
        let pane_title = if title.trim().is_empty() {
            "untitled".to_string()
        } else {
            title
        };
        let pane_id = self.pane_manager.open_to_side(pane_title);
        CommandActionOutcome::Executed(CommandActionEvent::PaneOpened { pane_id })
    }

    pub(super) fn execute_pane_move_tab(
        &mut self,
        command: &ParsedCommand,
    ) -> CommandActionOutcome {
        let direction = match parse_pane_move_direction(&command.target) {
            Some(direction) => direction,
            None => {
                return CommandActionOutcome::Failed(CommandActionError::MissingTarget {
                    domain: command.domain.clone(),
                    verb: command.verb.clone(),
                });
            }
        };

        if self.pane_manager.pane_count() <= 1 {
            return CommandActionOutcome::Failed(CommandActionError::InvalidTarget {
                reason: "pane が1つしかないためタブを移動できません".to_string(),
            });
        }

        let tab_id = match self.pane_manager.active_tab_id() {
            Some(id) => id.to_string(),
            None => {
                return CommandActionOutcome::Failed(CommandActionError::NotFound {
                    resource: "tab",
                    target: "active".to_string(),
                });
            }
        };

        let from_pane_id = match self.pane_manager.active_pane_id() {
            Some(id) => id.to_string(),
            None => {
                return CommandActionOutcome::Failed(CommandActionError::NotFound {
                    resource: "pane",
                    target: "active".to_string(),
                });
            }
        };

        let target_pane_id = match direction {
            PaneMoveDirection::Next => self.pane_manager.next_pane_id(),
            PaneMoveDirection::Previous => self.pane_manager.prev_pane_id(),
        };
        let target_pane_id = match target_pane_id {
            Some(id) => id.to_string(),
            None => {
                return CommandActionOutcome::Failed(CommandActionError::InvalidTarget {
                    reason: "移動先のペインが見つかりません".to_string(),
                });
            }
        };

        match self.pane_manager.move_tab(&tab_id, &target_pane_id) {
            Ok(()) => CommandActionOutcome::Executed(CommandActionEvent::TabMoved {
                tab_id,
                from_pane_id,
                to_pane_id: target_pane_id,
            }),
            Err(error) => CommandActionOutcome::Failed(pane_move_error_to_action_error(error)),
        }
    }
}

enum PaneMoveDirection {
    Next,
    Previous,
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

fn parse_pane_move_direction(target: &str) -> Option<PaneMoveDirection> {
    let normalized = normalized_lookup(target);
    if normalized.contains("next") || normalized.contains("forward") {
        Some(PaneMoveDirection::Next)
    } else if normalized.contains("prev")
        || normalized.contains("previous")
        || normalized.contains("backward")
    {
        Some(PaneMoveDirection::Previous)
    } else {
        None
    }
}

fn parse_pane_open_side_title(target: &str) -> Option<String> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut parts = trimmed.split_whitespace();
    let first = parts.next()?;
    match normalized_lookup(first).as_str() {
        "side" => Some(parts.collect::<Vec<_>>().join(" ")),
        "to" => {
            let second = parts.next()?;
            if normalized_lookup(second) == "side" {
                Some(parts.collect::<Vec<_>>().join(" "))
            } else {
                None
            }
        }
        _ => None,
    }
}
