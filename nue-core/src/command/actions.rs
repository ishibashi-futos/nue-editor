use crate::{
    command::state::{
        CommandHubSession, CommandMode, ParsedCommand, PickerCancelOutcome, PickerCandidate,
        PickerExecuteOutcome,
    },
    layout::pane_manager::{
        PaneItem, PaneLayoutSnapshot, PaneManager, PaneManagerError, PaneSplitDirection,
    },
    layout::tab_manager::{DEFAULT_HISTORY_CAPACITY, TabManager, TabManagerError, TabSnapshot},
};
use std::{cell::RefCell, rc::Rc};

mod target_lookup;
use target_lookup::{
    normalized_lookup, pane_index_by_target, tab_index_by_target, terminal_index_by_target,
    workspace_index_by_target,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelTarget {
    Explorer,
    GlobalSearch,
    Vcs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceItem {
    pub id: String,
    pub display_name: String,
    pub root_path: String,
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
    PaneOpened {
        pane_id: String,
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
    TabPinned {
        tab_id: String,
    },
    TabUnpinned {
        tab_id: String,
    },
    TabClosed {
        tab_id: String,
    },
    TabClosedOthers {
        tab_id: String,
    },
    TabClosedToRight {
        tab_id: String,
    },
    TabMoved {
        tab_id: String,
        from_pane_id: String,
        to_pane_id: String,
    },
    TabReopened {
        tab_id: String,
    },
    TabReordered {
        tab_id: String,
        new_index: usize,
    },
    SelectedOpened {
        target: SelectedOpenTarget,
    },
}

/// 上位レイヤに CommandHub の実行結果を伝える受け口。
pub trait CommandHubStateSink: 'static {
    /// モデルがコマンドを実行した際に呼び出される。
    fn handle_event(&mut self, event: &CommandActionEvent, snapshot: &CommandHubStateSnapshot);
}

/// デフォルトでは何もしない Sink。
pub struct NoopCommandHubStateSink;

impl CommandHubStateSink for NoopCommandHubStateSink {
    fn handle_event(&mut self, _event: &CommandActionEvent, _snapshot: &CommandHubStateSnapshot) {}
}

/// モデルの状態を切り出したスナップショット。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandHubStateSnapshot {
    pub workspaces: Vec<WorkspaceItem>,
    pub panes: Vec<PaneItem>,
    pub pane_layouts: Vec<PaneLayoutSnapshot>,
    pub terminals: Vec<TerminalItem>,
    pub tabs: Vec<TabSnapshot>,
    pub focused_panel: Option<PanelTarget>,
}

impl CommandHubStateSnapshot {
    fn from_model(model: &CommandHubActionModel) -> Self {
        Self {
            workspaces: model.workspaces().to_vec(),
            panes: model.panes(),
            pane_layouts: model.pane_layouts(),
            terminals: model.terminals().to_vec(),
            tabs: model.tabs(),
            focused_panel: model.focused_panel(),
        }
    }
}

/// CommandHub の実行結果を反映する共有状態。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandHubStateStore {
    pub workspaces: Vec<WorkspaceItem>,
    pub panes: Vec<PaneItem>,
    pub pane_layouts: Vec<PaneLayoutSnapshot>,
    pub terminals: Vec<TerminalItem>,
    pub tabs: Vec<TabSnapshot>,
    pub focused_panel: Option<PanelTarget>,
}

impl CommandHubStateStore {
    /// 新しい空の状態を作成する。
    pub fn new() -> Self {
        Self::default()
    }

    /// スナップショットの内容でストアを更新する。
    pub fn sync_from_snapshot(&mut self, snapshot: &CommandHubStateSnapshot) {
        self.workspaces = snapshot.workspaces.clone();
        self.panes = snapshot.panes.clone();
        self.pane_layouts = snapshot.pane_layouts.clone();
        self.terminals = snapshot.terminals.clone();
        self.tabs = snapshot.tabs.clone();
        self.focused_panel = snapshot.focused_panel;
    }
}

/// 共有状態を更新するための Sink。
pub struct CommandHubStateStoreSink {
    store: Rc<RefCell<CommandHubStateStore>>,
}

impl CommandHubStateStoreSink {
    pub fn new(store: Rc<RefCell<CommandHubStateStore>>) -> Self {
        Self { store }
    }
}

impl CommandHubStateSink for CommandHubStateStoreSink {
    fn handle_event(&mut self, _event: &CommandActionEvent, snapshot: &CommandHubStateSnapshot) {
        self.store.borrow_mut().sync_from_snapshot(snapshot);
    }
}

/// Command Hub の実行結果を反映するアプリケーション側の実体状態。
#[derive(Debug)]
pub struct CommandHubApplicationState {
    workspaces: Vec<WorkspaceItem>,
    pane_manager: PaneManager,
    terminals: Vec<TerminalItem>,
    tab_manager: TabManager,
    focused_panel: Option<PanelTarget>,
}

impl CommandHubApplicationState {
    /// スナップショットから初期状態を組み立てる。
    pub fn new(snapshot: &CommandHubStateSnapshot) -> Self {
        Self {
            workspaces: snapshot.workspaces.clone(),
            pane_manager: PaneManager::from_layout_snapshots(snapshot.pane_layouts.clone())
                .expect("pane layout snapshot が不正です"),
            terminals: snapshot.terminals.clone(),
            tab_manager: TabManager::from_snapshots(
                snapshot.tabs.clone(),
                DEFAULT_HISTORY_CAPACITY,
            ),
            focused_panel: snapshot.focused_panel,
        }
    }

    /// 最新のスナップショットの状態で置き換える。
    pub fn sync_from_snapshot(&mut self, snapshot: &CommandHubStateSnapshot) {
        self.workspaces = snapshot.workspaces.clone();
        self.pane_manager = PaneManager::from_layout_snapshots(snapshot.pane_layouts.clone())
            .expect("pane layout snapshot が不正です");
        self.terminals = snapshot.terminals.clone();
        self.tab_manager =
            TabManager::from_snapshots(snapshot.tabs.clone(), DEFAULT_HISTORY_CAPACITY);
        self.focused_panel = snapshot.focused_panel;
    }

    pub fn workspaces(&self) -> &[WorkspaceItem] {
        &self.workspaces
    }

    pub fn pane_manager(&self) -> &PaneManager {
        &self.pane_manager
    }

    pub fn terminals(&self) -> &[TerminalItem] {
        &self.terminals
    }

    pub fn tab_manager(&self) -> &TabManager {
        &self.tab_manager
    }

    pub fn focused_panel(&self) -> Option<PanelTarget> {
        self.focused_panel
    }
}

/// 実体状態を親レイヤーへ伝える Sink。
pub struct CommandHubApplicationStateSink {
    state: Rc<RefCell<CommandHubApplicationState>>,
}

impl CommandHubApplicationStateSink {
    pub fn new(state: Rc<RefCell<CommandHubApplicationState>>) -> Self {
        Self { state }
    }
}

impl CommandHubStateSink for CommandHubApplicationStateSink {
    fn handle_event(&mut self, _event: &CommandActionEvent, snapshot: &CommandHubStateSnapshot) {
        self.state.borrow_mut().sync_from_snapshot(snapshot);
    }
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

pub struct CommandHubActionModel {
    workspaces: Vec<WorkspaceItem>,
    pane_manager: PaneManager,
    terminals: Vec<TerminalItem>,
    tab_manager: TabManager,
    selected_text: Option<String>,
    focused_panel: Option<PanelTarget>,
    state_sink: Box<dyn CommandHubStateSink>,
}

impl std::fmt::Debug for CommandHubActionModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandHubActionModel")
            .field("workspaces", &self.workspaces)
            .field("pane_manager", &self.pane_manager)
            .field("terminals", &self.terminals)
            .field("tab_manager", &self.tab_manager)
            .field("selected_text", &self.selected_text)
            .field("focused_panel", &self.focused_panel)
            .finish()
    }
}

impl CommandHubActionModel {
    pub fn new(
        workspaces: Vec<WorkspaceItem>,
        panes: Vec<PaneItem>,
        terminals: Vec<TerminalItem>,
        tabs: Vec<TabSnapshot>,
    ) -> Self {
        Self::with_state_sink(
            workspaces,
            panes,
            terminals,
            tabs,
            Box::new(NoopCommandHubStateSink),
        )
    }

    /// Sink を指定して作成するコンストラクタ。
    pub fn with_state_sink(
        workspaces: Vec<WorkspaceItem>,
        panes: Vec<PaneItem>,
        terminals: Vec<TerminalItem>,
        tabs: Vec<TabSnapshot>,
        state_sink: Box<dyn CommandHubStateSink>,
    ) -> Self {
        Self {
            workspaces,
            pane_manager: PaneManager::from_items(panes),
            terminals,
            tab_manager: TabManager::from_snapshots(tabs, DEFAULT_HISTORY_CAPACITY),
            selected_text: None,
            focused_panel: None,
            state_sink,
        }
    }

    pub fn tabs(&self) -> Vec<TabSnapshot> {
        self.tab_manager.tabs()
    }

    pub fn sync_tabs(&mut self, snapshots: Vec<TabSnapshot>) {
        self.tab_manager = TabManager::from_snapshots(snapshots, DEFAULT_HISTORY_CAPACITY);
    }

    pub fn set_selected_text(&mut self, selected_text: Option<String>) {
        self.selected_text = selected_text;
    }

    /// UI 側で最新のワークスペース一覧を反映する。
    pub fn set_workspaces(&mut self, workspaces: Vec<WorkspaceItem>) {
        self.workspaces = workspaces;
    }

    /// UI 側のペイン情報を再構築する。
    pub fn set_panes(&mut self, panes: Vec<PaneItem>) {
        self.pane_manager = PaneManager::from_items(panes);
    }

    /// UI 側のターミナル一覧を上書きする。
    pub fn set_terminals(&mut self, terminals: Vec<TerminalItem>) {
        self.terminals = terminals;
    }

    /// UI 側のタブ状態を再同期する。
    pub fn set_tabs(&mut self, tabs: Vec<TabSnapshot>) {
        self.sync_tabs(tabs);
    }

    /// UI 側でフォーカスパネルの状態を更新する。
    pub fn set_focused_panel(&mut self, panel: Option<PanelTarget>) {
        self.focused_panel = panel;
    }

    /// 実行結果を受けて上位レイヤの状態を更新する Sink を差し替える。
    pub fn set_state_sink(&mut self, sink: Box<dyn CommandHubStateSink>) {
        self.state_sink = sink;
    }

    pub fn workspaces(&self) -> &[WorkspaceItem] {
        &self.workspaces
    }

    pub fn panes(&self) -> Vec<PaneItem> {
        self.pane_manager.panes()
    }

    pub fn pane_layouts(&self) -> Vec<PaneLayoutSnapshot> {
        self.pane_manager.layout_snapshot()
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
            ("pane", "list") => {
                let panes = self.pane_manager.panes();
                pane_candidates_for_target(command, &panes, "open")
            }
            ("pane", "close") => {
                let panes = self.pane_manager.panes();
                pane_candidates_for_target(command, &panes, "close")
            }
            ("terminal", "list") => {
                terminal_candidates_for_target(command, &self.terminals, "open", false)
            }
            ("terminal", "close") | ("terminal", "kill") => {
                terminal_candidates_for_target(command, &self.terminals, "close", true)
            }
            ("tab", "pin") => {
                let tabs = self.tabs();
                tab_candidates_for_target(&tabs, &command.target, "pin", false)
            }
            ("tab", "unpin") => {
                let tabs = self.tabs();
                tab_candidates_for_target(&tabs, &command.target, "unpin", false)
            }
            ("tab", "close") => {
                let tabs = self.tabs();
                tab_candidates_for_target(&tabs, &command.target, "close", true)
            }
            ("tab", "close_others") => {
                let tabs = self.tabs();
                tab_candidates_for_target(&tabs, &command.target, "close_others", true)
            }
            ("tab", "close_to_right") => {
                let tabs = self.tabs();
                tab_candidates_for_target(&tabs, &command.target, "close_to_right", true)
            }
            _ => Vec::new(),
        }
    }

    pub fn execute(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
        if command.mode != CommandMode::Action {
            return unsupported_command(command);
        }

        let outcome = match (command.domain.as_str(), command.verb.as_str()) {
            ("workspace", "add") => self.execute_workspace_add(command),
            ("workspace", "open") | ("workspace", "activate") => {
                self.execute_workspace_activate(command)
            }
            ("workspace", "remove") => self.execute_workspace_remove(command),
            ("pane", "split") => self.execute_pane_split(command),
            ("pane", "next") => self.execute_pane_next(),
            ("pane", "prev") => self.execute_pane_prev(),
            ("pane", "close") => self.execute_pane_close(command),
            ("pane", "move") => self.execute_pane_move_tab(command),
            ("pane", "open") => {
                if let Some(side_title) = parse_pane_open_side_title(&command.target) {
                    self.execute_pane_open_side(side_title)
                } else {
                    self.execute_pane_activate(command)
                }
            }
            ("pane", "activate") => self.execute_pane_activate(command),
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
            ("tab", "pin") => self.execute_tab_pin(command),
            ("tab", "unpin") => self.execute_tab_unpin(command),
            ("tab", "close") => self.execute_tab_close(command),
            ("tab", "close_others") => self.execute_tab_close_others(command),
            ("tab", "close_to_right") => self.execute_tab_close_to_right(command),
            ("tab", "reopen") => self.execute_tab_reopen(command),
            ("tab", "reorder") => self.execute_tab_reorder(command),
            _ => unsupported_command(command),
        };

        if let CommandActionOutcome::Executed(ref event) = outcome {
            let snapshot = CommandHubStateSnapshot::from_model(self);
            self.state_sink.handle_event(event, &snapshot);
        }

        outcome
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

    fn execute_pane_next(&mut self) -> CommandActionOutcome {
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

    fn execute_pane_prev(&mut self) -> CommandActionOutcome {
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

    fn execute_pane_close(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
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

    fn execute_pane_activate(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
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

    fn execute_pane_open_side(&mut self, title: String) -> CommandActionOutcome {
        let pane_title = if title.trim().is_empty() {
            "untitled".to_string()
        } else {
            title
        };
        let pane_id = self.pane_manager.open_to_side(pane_title);
        CommandActionOutcome::Executed(CommandActionEvent::PaneOpened { pane_id })
    }

    fn execute_pane_move_tab(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
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

    fn execute_tab_pin(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
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

    fn execute_tab_unpin(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
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

    fn execute_tab_close(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
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

    fn execute_tab_close_others(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
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

    fn execute_tab_close_to_right(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
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

    fn execute_tab_reopen(&mut self, _command: &ParsedCommand) -> CommandActionOutcome {
        match self.tab_manager.reopen_last_closed() {
            Ok(tab_id) => {
                CommandActionOutcome::Executed(CommandActionEvent::TabReopened { tab_id })
            }
            Err(error) => CommandActionOutcome::Failed(tab_error_to_action_error("", error)),
        }
    }

    fn execute_tab_reorder(&mut self, command: &ParsedCommand) -> CommandActionOutcome {
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
        let index_token = parts.last().unwrap();
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

fn tab_candidate_for_tab(
    tab: &TabSnapshot,
    verb: &str,
    requires_confirmation: bool,
) -> PickerCandidate {
    PickerCandidate {
        id: format!("tab::{verb}::{}", tab.id),
        label: format!("Tab: {}", tab.title),
        command: ParsedCommand {
            mode: CommandMode::Action,
            domain: "tab".to_string(),
            verb: verb.to_string(),
            target: tab.id.clone(),
        },
        requires_confirmation,
    }
}

fn tab_candidates(
    tabs: &[TabSnapshot],
    verb: &str,
    requires_confirmation: bool,
) -> Vec<PickerCandidate> {
    tabs.iter()
        .map(|tab| tab_candidate_for_tab(tab, verb, requires_confirmation))
        .collect()
}

fn tab_candidates_for_target(
    tabs: &[TabSnapshot],
    target: &str,
    verb: &str,
    requires_confirmation: bool,
) -> Vec<PickerCandidate> {
    if target.trim().is_empty() {
        return tab_candidates(tabs, verb, requires_confirmation);
    }
    let Some(index) = tab_index_by_target(tabs, target) else {
        return Vec::new();
    };
    vec![tab_candidate_for_tab(
        &tabs[index],
        verb,
        requires_confirmation,
    )]
}

fn tab_error_to_action_error(tab_id: &str, error: TabManagerError) -> CommandActionError {
    match error {
        TabManagerError::TabNotFound(_) => CommandActionError::NotFound {
            resource: "tab",
            target: tab_id.to_string(),
        },
        TabManagerError::InvalidTargetIndex(index) => CommandActionError::InvalidTarget {
            reason: format!("tab index {index} は無効です"),
        },
        TabManagerError::AlreadySingleTab => CommandActionError::InvalidTarget {
            reason: "tab が1つしかないため閉じられません".to_string(),
        },
        TabManagerError::ReopenHistoryEmpty => CommandActionError::InvalidTarget {
            reason: "閉じたタブがありません".to_string(),
        },
    }
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

enum PaneMoveDirection {
    Next,
    Previous,
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

fn pane_move_error_to_action_error(error: PaneManagerError) -> CommandActionError {
    match error {
        PaneManagerError::TabNotFound(tab_id) => CommandActionError::NotFound {
            resource: "tab",
            target: tab_id,
        },
        PaneManagerError::SingleTabPane(pane_id) => CommandActionError::InvalidTarget {
            reason: format!("{pane_id} には移動対象のタブがありません"),
        },
        PaneManagerError::PaneNotFound(pane_id) => CommandActionError::NotFound {
            resource: "pane",
            target: pane_id,
        },
        _ => CommandActionError::InvalidTarget {
            reason: "タブの移動に失敗しました".to_string(),
        },
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
    use crate::command::state::{CommandHubSession, PickerCandidate};
    use crate::layout::pane_manager::{PaneItem, PaneSplitDirection};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn action_command(domain: &str, verb: &str, target: &str) -> ParsedCommand {
        ParsedCommand {
            mode: CommandMode::Action,
            domain: domain.to_string(),
            verb: verb.to_string(),
            target: target.to_string(),
        }
    }

    struct RecordingSink {
        events: Rc<RefCell<Vec<CommandActionEvent>>>,
    }

    impl RecordingSink {
        fn new(events: Rc<RefCell<Vec<CommandActionEvent>>>) -> Self {
            Self { events }
        }
    }

    impl CommandHubStateSink for RecordingSink {
        fn handle_event(
            &mut self,
            event: &CommandActionEvent,
            _snapshot: &CommandHubStateSnapshot,
        ) {
            self.events.borrow_mut().push(event.clone());
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
            vec![
                TabSnapshot {
                    id: "tab-1".to_string(),
                    title: "README.md".to_string(),
                    pinned: false,
                    is_active: true,
                },
                TabSnapshot {
                    id: "tab-2".to_string(),
                    title: "lib.rs".to_string(),
                    pinned: false,
                    is_active: false,
                },
                TabSnapshot {
                    id: "tab-3".to_string(),
                    title: "mod.rs".to_string(),
                    pinned: true,
                    is_active: false,
                },
            ],
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
    fn state_sink_receives_executed_events() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut model = model();
        model.set_state_sink(Box::new(RecordingSink::new(events.clone())));

        assert_eq!(
            model.execute(&action_command("pane", "close", "pane-2")),
            CommandActionOutcome::Executed(CommandActionEvent::PaneClosed {
                pane_id: "pane-2".to_string(),
            })
        );

        let recorded = events.borrow();
        assert_eq!(recorded.len(), 1);
        assert_eq!(
            recorded[0],
            CommandActionEvent::PaneClosed {
                pane_id: "pane-2".to_string()
            }
        );
    }

    #[test]
    fn state_store_sink_reflects_model_state() {
        let store = Rc::new(RefCell::new(CommandHubStateStore::new()));
        let mut model = model();
        model.set_state_sink(Box::new(CommandHubStateStoreSink::new(store.clone())));

        let _ = model.execute(&action_command("pane", "close", "pane-2"));
        {
            let snapshot = store.borrow();
            assert_eq!(snapshot.panes.len(), 1);
            assert_eq!(snapshot.panes[0].id, "pane-1");
        }

        let _ = model.execute(&action_command("panel", "focus", "global search"));
        {
            let snapshot = store.borrow();
            assert_eq!(snapshot.focused_panel, Some(PanelTarget::GlobalSearch));
        }

        let _ = model.execute(&action_command("terminal", "new", ""));
        {
            let snapshot = store.borrow();
            assert_eq!(snapshot.terminals.len(), 2);
            assert!(snapshot.terminals.iter().any(|terminal| terminal.is_active));
        }
    }

    #[test]
    fn application_state_reflects_latest_snapshot() {
        let mut model = model();
        let _ = model.execute(&action_command("pane", "close", "pane-2"));
        let snapshot = CommandHubStateSnapshot::from_model(&model);
        let state = CommandHubApplicationState::new(&snapshot);

        assert_eq!(state.workspaces(), snapshot.workspaces.as_slice());
        assert_eq!(state.pane_manager().panes().len(), snapshot.panes.len());
        assert_eq!(state.terminals().len(), snapshot.terminals.len());
        assert_eq!(state.tab_manager().tabs(), snapshot.tabs);
        assert_eq!(state.focused_panel(), snapshot.focused_panel);
    }

    #[test]
    fn application_state_preserves_pane_layout_tabs() {
        let mut model = model();
        model
            .pane_manager
            .add_tab_to_pane("pane-1", "extra.md")
            .expect("tab 追加成功");
        let expected_layout = model.pane_manager.layout_snapshot();
        let snapshot = CommandHubStateSnapshot::from_model(&model);

        let state = CommandHubApplicationState::new(&snapshot);

        assert_eq!(state.pane_manager().layout_snapshot(), expected_layout);
    }

    #[test]
    fn application_state_sink_updates_shared_state() {
        let mut model = model();
        let initial_snapshot = CommandHubStateSnapshot::from_model(&model);
        let shared_state = Rc::new(RefCell::new(CommandHubApplicationState::new(
            &initial_snapshot,
        )));
        let mut sink = CommandHubApplicationStateSink::new(shared_state.clone());

        let outcome = model.execute(&action_command("terminal", "new", ""));
        let terminal_id = match outcome {
            CommandActionOutcome::Executed(CommandActionEvent::TerminalCreated { terminal_id }) => {
                terminal_id
            }
            _ => panic!("Terminal 新規作成が失敗しました"),
        };
        let snapshot = CommandHubStateSnapshot::from_model(&model);
        sink.handle_event(
            &CommandActionEvent::TerminalCreated {
                terminal_id: terminal_id.clone(),
            },
            &snapshot,
        );

        let borrowed = shared_state.borrow();
        assert_eq!(borrowed.terminals().len(), snapshot.terminals.len());
        assert!(
            borrowed
                .terminals()
                .iter()
                .any(|terminal| terminal.id == terminal_id)
        );
        assert_eq!(borrowed.pane_manager().panes().len(), snapshot.panes.len());
        assert_eq!(borrowed.tab_manager().tabs(), snapshot.tabs);
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
                pane_id: "pane-2".to_string(),
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
    fn pane_open_side_creates_new_pane_and_notifies() {
        let mut model = model();
        let outcome = model.execute(&action_command("pane", "open", "side"));

        let pane_id =
            if let CommandActionOutcome::Executed(CommandActionEvent::PaneOpened { pane_id }) =
                outcome
            {
                pane_id
            } else {
                panic!("pane open side が PaneOpened を返すはず");
            };

        let panes = model.panes();
        assert_eq!(panes.len(), 3);
        assert!(
            panes
                .iter()
                .any(|pane| pane.id == pane_id && pane.title == "untitled")
        );
    }

    #[test]
    fn pane_open_side_can_take_custom_title() {
        let mut model = model();
        let _ = model.execute(&action_command("pane", "open", "side README"));

        let panes = model.panes();
        let target = panes
            .iter()
            .find(|pane| pane.is_active)
            .expect("新規ペインがアクティブであるはず");
        assert_eq!(target.title, "README");
    }

    #[test]
    fn pane_move_tab_next_transfers_active_tab() {
        let mut model = model();
        model
            .pane_manager
            .add_tab_to_pane("pane-1", "extra.md")
            .expect("tab 追加成功");

        let outcome = model.execute(&action_command("pane", "move", "tab next"));

        let event = if let CommandActionOutcome::Executed(CommandActionEvent::TabMoved {
            tab_id,
            from_pane_id,
            to_pane_id,
        }) = outcome
        {
            assert_eq!(from_pane_id, "pane-1");
            assert_eq!(to_pane_id, "pane-2");
            assert!(tab_id.starts_with("tab-"));
            tab_id
        } else {
            panic!("TabMoved イベントが返るはず");
        };

        let panes = model.panes();
        assert_eq!(panes[1].title, "extra.md");
        assert!(
            model.tab_manager.tabs().iter().any(|tab| tab.id == event),
            "移動したタブが TabManager に存在するはず"
        );
    }

    #[test]
    fn pane_move_requires_direction_keyword() {
        let mut model = model();
        let outcome = model.execute(&action_command("pane", "move", "tab"));
        assert!(matches!(
            outcome,
            CommandActionOutcome::Failed(CommandActionError::MissingTarget { .. })
        ));
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

    #[test]
    fn tab_close_can_be_reopened() {
        let mut model = model();
        let close_cmd = action_command("tab", "close", "tab-2");

        assert_eq!(
            model.execute(&close_cmd),
            CommandActionOutcome::Executed(CommandActionEvent::TabClosed {
                tab_id: "tab-2".to_string()
            })
        );
        assert_eq!(model.tabs().len(), 2);

        assert_eq!(
            model.execute(&action_command("tab", "reopen", "")),
            CommandActionOutcome::Executed(CommandActionEvent::TabReopened {
                tab_id: "tab-2".to_string()
            })
        );
        assert_eq!(model.tabs().len(), 3);
    }

    #[test]
    fn tab_reorder_dispatches_event() {
        let mut model = model();
        let reorder_cmd = action_command("tab", "reorder", "tab-3 0");

        assert_eq!(
            model.execute(&reorder_cmd),
            CommandActionOutcome::Executed(CommandActionEvent::TabReordered {
                tab_id: "tab-3".to_string(),
                new_index: 0,
            })
        );
    }
}
