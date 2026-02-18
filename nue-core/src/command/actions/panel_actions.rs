use super::{
    CommandActionError, CommandActionEvent, CommandActionOutcome, CommandHubActionModel,
    PanelTarget, normalized_lookup,
};

impl CommandHubActionModel {
    pub(super) fn execute_panel_focus(&mut self, target: &str) -> CommandActionOutcome {
        let Some(panel) = parse_panel_target(target) else {
            return CommandActionOutcome::Failed(CommandActionError::InvalidTarget {
                reason: "panel focus target は explorer/global search/vcs のいずれかです"
                    .to_string(),
            });
        };

        self.focused_panel = Some(panel);
        CommandActionOutcome::Executed(CommandActionEvent::PanelFocused { panel })
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
