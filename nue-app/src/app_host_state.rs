use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppHostState {
    workspaces: WorkspaceListState,
    active_workspace: ActiveWorkspaceState,
    ui_shell: UiShellState,
    notifications: NotificationCenterState,
    sessions: WorkspaceSessionListState,
}

impl AppHostState {
    pub fn empty() -> Self {
        Self {
            workspaces: WorkspaceListState::default(),
            active_workspace: ActiveWorkspaceState::default(),
            ui_shell: UiShellState::default(),
            notifications: NotificationCenterState::default(),
            sessions: WorkspaceSessionListState::default(),
        }
    }

    pub fn workspaces(&self) -> &WorkspaceListState {
        &self.workspaces
    }

    pub fn workspaces_mut(&mut self) -> &mut WorkspaceListState {
        &mut self.workspaces
    }

    pub fn active_workspace(&self) -> &ActiveWorkspaceState {
        &self.active_workspace
    }

    pub fn active_workspace_mut(&mut self) -> &mut ActiveWorkspaceState {
        &mut self.active_workspace
    }

    pub fn ui_shell(&self) -> &UiShellState {
        &self.ui_shell
    }

    pub fn ui_shell_mut(&mut self) -> &mut UiShellState {
        &mut self.ui_shell
    }

    pub fn notifications(&self) -> &NotificationCenterState {
        &self.notifications
    }

    pub fn notifications_mut(&mut self) -> &mut NotificationCenterState {
        &mut self.notifications
    }

    pub fn sessions(&self) -> &WorkspaceSessionListState {
        &self.sessions
    }

    pub fn sessions_mut(&mut self) -> &mut WorkspaceSessionListState {
        &mut self.sessions
    }
}

impl Default for AppHostState {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkspaceId(String);

impl WorkspaceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkspaceSessionId(String);

impl WorkspaceSessionId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceListState {
    order: Vec<WorkspaceId>,
    entries: BTreeMap<WorkspaceId, WorkspaceListEntry>,
}

impl WorkspaceListState {
    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub fn get(&self, id: &WorkspaceId) -> Option<&WorkspaceListEntry> {
        self.entries.get(id)
    }

    pub fn upsert(&mut self, entry: WorkspaceListEntry) {
        let id = entry.id.clone();
        if !self.entries.contains_key(&id) {
            self.order.push(id.clone());
        }
        self.entries.insert(id, entry);
    }

    pub fn remove(&mut self, id: &WorkspaceId) -> Option<WorkspaceListEntry> {
        let removed = self.entries.remove(id);
        if removed.is_some() {
            self.order.retain(|current| current != id);
        }
        removed
    }

    pub fn ordered(&self) -> Vec<&WorkspaceListEntry> {
        self.order
            .iter()
            .filter_map(|id| self.entries.get(id))
            .collect()
    }
}

impl Default for WorkspaceListState {
    fn default() -> Self {
        Self {
            order: Vec::new(),
            entries: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceListEntry {
    id: WorkspaceId,
    root_path: PathBuf,
    label: String,
    status: WorkspaceStatus,
}

impl WorkspaceListEntry {
    pub fn new(id: WorkspaceId, root_path: PathBuf, label: impl Into<String>) -> Self {
        Self {
            id,
            root_path,
            label: label.into(),
            status: WorkspaceStatus::Registered,
        }
    }

    pub fn id(&self) -> &WorkspaceId {
        &self.id
    }

    pub fn root_path(&self) -> &PathBuf {
        &self.root_path
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn status(&self) -> WorkspaceStatus {
        self.status
    }

    pub fn with_status(mut self, status: WorkspaceStatus) -> Self {
        self.status = status;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceStatus {
    Registered,
    Loading,
    Ready,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActiveWorkspaceState {
    active_workspace_id: Option<WorkspaceId>,
}

impl ActiveWorkspaceState {
    pub fn current(&self) -> Option<&WorkspaceId> {
        self.active_workspace_id.as_ref()
    }

    pub fn set(&mut self, workspace_id: Option<WorkspaceId>) {
        self.active_workspace_id = workspace_id;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiShellState {
    screen: ScreenKind,
    left_panel: LeftPanelKind,
    overlay: OverlayKind,
}

impl UiShellState {
    pub fn screen(&self) -> ScreenKind {
        self.screen
    }

    pub fn set_screen(&mut self, screen: ScreenKind) {
        self.screen = screen;
    }

    pub fn left_panel(&self) -> LeftPanelKind {
        self.left_panel
    }

    pub fn set_left_panel(&mut self, panel: LeftPanelKind) {
        self.left_panel = panel;
    }

    pub fn overlay(&self) -> OverlayKind {
        self.overlay
    }

    pub fn set_overlay(&mut self, overlay: OverlayKind) {
        self.overlay = overlay;
    }
}

impl Default for UiShellState {
    fn default() -> Self {
        Self {
            screen: ScreenKind::WorkspaceSelection,
            left_panel: LeftPanelKind::Explorer,
            overlay: OverlayKind::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenKind {
    WorkspaceSelection,
    WorkspaceSession,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftPanelKind {
    Explorer,
    Search,
    Vcs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayKind {
    None,
    CommandHub,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationCenterState {
    next_id: u64,
    items: Vec<NotificationEntry>,
}

impl NotificationCenterState {
    pub fn items(&self) -> &[NotificationEntry] {
        &self.items
    }

    pub fn push(&mut self, level: NotificationLevel, message: impl Into<String>) -> NotificationId {
        let id = NotificationId(self.next_id);
        self.next_id += 1;
        self.items.push(NotificationEntry {
            id,
            level,
            message: message.into(),
        });
        id
    }

    pub fn remove(&mut self, id: NotificationId) -> Option<NotificationEntry> {
        let index = self.items.iter().position(|item| item.id == id)?;
        Some(self.items.remove(index))
    }
}

impl Default for NotificationCenterState {
    fn default() -> Self {
        Self {
            next_id: 1,
            items: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NotificationId(u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationEntry {
    id: NotificationId,
    level: NotificationLevel,
    message: String,
}

impl NotificationEntry {
    pub fn id(&self) -> NotificationId {
        self.id
    }

    pub fn level(&self) -> NotificationLevel {
        self.level
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSessionListState {
    order: Vec<WorkspaceSessionId>,
    entries: BTreeMap<WorkspaceSessionId, WorkspaceSessionListEntry>,
}

impl WorkspaceSessionListState {
    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub fn get(&self, id: &WorkspaceSessionId) -> Option<&WorkspaceSessionListEntry> {
        self.entries.get(id)
    }

    pub fn upsert(&mut self, entry: WorkspaceSessionListEntry) {
        let id = entry.id.clone();
        if !self.entries.contains_key(&id) {
            self.order.push(id.clone());
        }
        self.entries.insert(id, entry);
    }

    pub fn remove(&mut self, id: &WorkspaceSessionId) -> Option<WorkspaceSessionListEntry> {
        let removed = self.entries.remove(id);
        if removed.is_some() {
            self.order.retain(|current| current != id);
        }
        removed
    }

    pub fn ordered(&self) -> Vec<&WorkspaceSessionListEntry> {
        self.order
            .iter()
            .filter_map(|id| self.entries.get(id))
            .collect()
    }
}

impl Default for WorkspaceSessionListState {
    fn default() -> Self {
        Self {
            order: Vec::new(),
            entries: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSessionListEntry {
    id: WorkspaceSessionId,
    workspace_id: WorkspaceId,
    title: String,
    status: WorkspaceSessionStatus,
}

impl WorkspaceSessionListEntry {
    pub fn new(
        id: WorkspaceSessionId,
        workspace_id: WorkspaceId,
        title: impl Into<String>,
        status: WorkspaceSessionStatus,
    ) -> Self {
        Self {
            id,
            workspace_id,
            title: title.into(),
            status,
        }
    }

    pub fn id(&self) -> &WorkspaceSessionId {
        &self.id
    }

    pub fn workspace_id(&self) -> &WorkspaceId {
        &self.workspace_id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn status(&self) -> WorkspaceSessionStatus {
        self.status
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceSessionStatus {
    Starting,
    Ready,
    Failed,
    Closed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_single_session_use_same_app_host_state_type() {
        let mut state = AppHostState::empty();
        assert!(state.workspaces().is_empty());
        assert!(state.sessions().is_empty());
        assert_eq!(state.active_workspace().current(), None);
        assert_eq!(state.ui_shell().screen(), ScreenKind::WorkspaceSelection);

        let workspace_id = WorkspaceId::new("ws-1");
        state.workspaces_mut().upsert(
            WorkspaceListEntry::new(
                workspace_id.clone(),
                PathBuf::from("/tmp/ws-1"),
                "workspace-1",
            )
            .with_status(WorkspaceStatus::Ready),
        );
        state.active_workspace_mut().set(Some(workspace_id.clone()));
        state
            .ui_shell_mut()
            .set_screen(ScreenKind::WorkspaceSession);
        state.sessions_mut().upsert(WorkspaceSessionListEntry::new(
            WorkspaceSessionId::new("session-1"),
            workspace_id.clone(),
            "session-1",
            WorkspaceSessionStatus::Ready,
        ));

        assert_eq!(state.workspaces().len(), 1);
        assert_eq!(state.sessions().len(), 1);
        assert_eq!(state.active_workspace().current(), Some(&workspace_id));
        assert_eq!(state.ui_shell().screen(), ScreenKind::WorkspaceSession);
    }

    #[test]
    fn ui_can_mutate_substates_without_touching_other_responsibilities() {
        let mut state = AppHostState::empty();
        let workspace_id = WorkspaceId::new("ws-ui");

        state.workspaces_mut().upsert(WorkspaceListEntry::new(
            workspace_id.clone(),
            PathBuf::from("/tmp/ws-ui"),
            "UI Workspace",
        ));
        state.active_workspace_mut().set(Some(workspace_id.clone()));
        state.ui_shell_mut().set_left_panel(LeftPanelKind::Search);
        state.ui_shell_mut().set_overlay(OverlayKind::CommandHub);
        let notification_id = state
            .notifications_mut()
            .push(NotificationLevel::Info, "loaded");
        state.sessions_mut().upsert(WorkspaceSessionListEntry::new(
            WorkspaceSessionId::new("session-ui"),
            workspace_id.clone(),
            "UI Session",
            WorkspaceSessionStatus::Starting,
        ));

        assert_eq!(state.ui_shell().left_panel(), LeftPanelKind::Search);
        assert_eq!(state.ui_shell().overlay(), OverlayKind::CommandHub);
        assert_eq!(state.notifications().items().len(), 1);
        assert_eq!(
            state.notifications().items()[0],
            NotificationEntry {
                id: notification_id,
                level: NotificationLevel::Info,
                message: "loaded".to_string(),
            }
        );
        assert_eq!(state.sessions().ordered().len(), 1);
        assert_eq!(state.active_workspace().current(), Some(&workspace_id));
    }
}
