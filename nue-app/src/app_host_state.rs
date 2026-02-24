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

    pub fn session_mut(
        &mut self,
        id: &WorkspaceSessionId,
    ) -> Option<&mut WorkspaceSessionListEntry> {
        self.sessions.get_mut(id)
    }

    pub fn create_workspace_session(
        &mut self,
        id: WorkspaceSessionId,
        workspace_id: WorkspaceId,
        title: impl Into<String>,
    ) -> Result<(), WorkspaceSessionCreationError> {
        self.sessions.insert_new(WorkspaceSessionListEntry::new(
            id,
            workspace_id,
            title,
            WorkspaceSessionStatus::Starting,
        ))
    }

    pub fn destroy_workspace_session(
        &mut self,
        id: &WorkspaceSessionId,
    ) -> Option<WorkspaceSessionListEntry> {
        self.sessions.remove(id)
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
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

    pub fn set_status(&mut self, status: WorkspaceStatus) {
        self.status = status;
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
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

    fn get_mut(&mut self, id: &WorkspaceSessionId) -> Option<&mut WorkspaceSessionListEntry> {
        self.entries.get_mut(id)
    }

    pub fn insert_new(
        &mut self,
        entry: WorkspaceSessionListEntry,
    ) -> Result<(), WorkspaceSessionCreationError> {
        let id = entry.id.clone();
        if self.entries.contains_key(&id) {
            return Err(WorkspaceSessionCreationError::DuplicateSessionId { id });
        }
        self.order.push(id.clone());
        self.entries.insert(id, entry);
        Ok(())
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

    pub fn ordered_for_workspace(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Vec<&WorkspaceSessionListEntry> {
        self.ordered()
            .into_iter()
            .filter(|entry| entry.workspace_id() == workspace_id)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSessionListEntry {
    id: WorkspaceSessionId,
    workspace_id: WorkspaceId,
    title: String,
    status: WorkspaceSessionStatus,
    bundle: WorkspaceSessionStateBundle,
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
            bundle: WorkspaceSessionStateBundle::default(),
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

    pub fn bundle(&self) -> &WorkspaceSessionStateBundle {
        &self.bundle
    }

    pub fn bundle_mut(&mut self) -> &mut WorkspaceSessionStateBundle {
        &mut self.bundle
    }

    pub fn with_bundle(mut self, bundle: WorkspaceSessionStateBundle) -> Self {
        self.bundle = bundle;
        self
    }

    pub fn set_status(&mut self, status: WorkspaceSessionStatus) {
        self.status = status;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceSessionStatus {
    Starting,
    Ready,
    Failed,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceSessionCreationError {
    DuplicateSessionId { id: WorkspaceSessionId },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkspaceSessionStateBundle {
    editor: EditorSessionState,
    explorer: ExplorerSessionState,
    command_hub: CommandHubSessionState,
    tabs: TabSessionState,
    terminal: TerminalSessionPanelState,
    subscribers: SessionSubscribersState,
}

impl WorkspaceSessionStateBundle {
    pub fn editor(&self) -> &EditorSessionState {
        &self.editor
    }

    pub fn editor_mut(&mut self) -> &mut EditorSessionState {
        &mut self.editor
    }

    pub fn explorer(&self) -> &ExplorerSessionState {
        &self.explorer
    }

    pub fn explorer_mut(&mut self) -> &mut ExplorerSessionState {
        &mut self.explorer
    }

    pub fn command_hub(&self) -> &CommandHubSessionState {
        &self.command_hub
    }

    pub fn command_hub_mut(&mut self) -> &mut CommandHubSessionState {
        &mut self.command_hub
    }

    pub fn tabs(&self) -> &TabSessionState {
        &self.tabs
    }

    pub fn tabs_mut(&mut self) -> &mut TabSessionState {
        &mut self.tabs
    }

    pub fn terminal(&self) -> &TerminalSessionPanelState {
        &self.terminal
    }

    pub fn terminal_mut(&mut self) -> &mut TerminalSessionPanelState {
        &mut self.terminal
    }

    pub fn subscribers(&self) -> &SessionSubscribersState {
        &self.subscribers
    }

    pub fn subscribers_mut(&mut self) -> &mut SessionSubscribersState {
        &mut self.subscribers
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EditorSessionState {
    active_document_path: Option<PathBuf>,
    dirty_document_count: usize,
}

impl EditorSessionState {
    pub fn active_document_path(&self) -> Option<&PathBuf> {
        self.active_document_path.as_ref()
    }

    pub fn set_active_document_path(&mut self, path: Option<PathBuf>) {
        self.active_document_path = path;
    }

    pub fn dirty_document_count(&self) -> usize {
        self.dirty_document_count
    }

    pub fn set_dirty_document_count(&mut self, count: usize) {
        self.dirty_document_count = count;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExplorerSessionState {
    selected_path: Option<PathBuf>,
    expanded_paths: Vec<PathBuf>,
}

impl ExplorerSessionState {
    pub fn selected_path(&self) -> Option<&PathBuf> {
        self.selected_path.as_ref()
    }

    pub fn set_selected_path(&mut self, path: Option<PathBuf>) {
        self.selected_path = path;
    }

    pub fn expanded_paths(&self) -> &[PathBuf] {
        &self.expanded_paths
    }

    pub fn set_expanded_paths(&mut self, paths: Vec<PathBuf>) {
        self.expanded_paths = paths;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandHubSessionState {
    is_open: bool,
    query: String,
}

impl CommandHubSessionState {
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn set_is_open(&mut self, is_open: bool) {
        self.is_open = is_open;
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TabSessionState {
    active_tab_id: Option<String>,
    tab_order: Vec<String>,
}

impl TabSessionState {
    pub fn active_tab_id(&self) -> Option<&str> {
        self.active_tab_id.as_deref()
    }

    pub fn set_active_tab_id(&mut self, tab_id: Option<String>) {
        self.active_tab_id = tab_id;
    }

    pub fn tab_order(&self) -> &[String] {
        &self.tab_order
    }

    pub fn set_tab_order(&mut self, tab_order: Vec<String>) {
        self.tab_order = tab_order;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TerminalSessionPanelState {
    active_terminal_id: Option<String>,
    terminal_ids: Vec<String>,
}

impl TerminalSessionPanelState {
    pub fn active_terminal_id(&self) -> Option<&str> {
        self.active_terminal_id.as_deref()
    }

    pub fn set_active_terminal_id(&mut self, terminal_id: Option<String>) {
        self.active_terminal_id = terminal_id;
    }

    pub fn terminal_ids(&self) -> &[String] {
        &self.terminal_ids
    }

    pub fn set_terminal_ids(&mut self, terminal_ids: Vec<String>) {
        self.terminal_ids = terminal_ids;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SessionSubscribersState {
    editor_subscriber_attached: bool,
    explorer_subscriber_attached: bool,
    command_hub_subscriber_attached: bool,
    tab_subscriber_attached: bool,
    terminal_subscriber_attached: bool,
}

impl SessionSubscribersState {
    pub fn editor_subscriber_attached(&self) -> bool {
        self.editor_subscriber_attached
    }

    pub fn set_editor_subscriber_attached(&mut self, attached: bool) {
        self.editor_subscriber_attached = attached;
    }

    pub fn explorer_subscriber_attached(&self) -> bool {
        self.explorer_subscriber_attached
    }

    pub fn set_explorer_subscriber_attached(&mut self, attached: bool) {
        self.explorer_subscriber_attached = attached;
    }

    pub fn command_hub_subscriber_attached(&self) -> bool {
        self.command_hub_subscriber_attached
    }

    pub fn set_command_hub_subscriber_attached(&mut self, attached: bool) {
        self.command_hub_subscriber_attached = attached;
    }

    pub fn tab_subscriber_attached(&self) -> bool {
        self.tab_subscriber_attached
    }

    pub fn set_tab_subscriber_attached(&mut self, attached: bool) {
        self.tab_subscriber_attached = attached;
    }

    pub fn terminal_subscriber_attached(&self) -> bool {
        self.terminal_subscriber_attached
    }

    pub fn set_terminal_subscriber_attached(&mut self, attached: bool) {
        self.terminal_subscriber_attached = attached;
    }
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
        state
            .sessions
            .insert_new(WorkspaceSessionListEntry::new(
                WorkspaceSessionId::new("session-1"),
                workspace_id.clone(),
                "session-1",
                WorkspaceSessionStatus::Ready,
            ))
            .expect("session insert for test");

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
        state
            .sessions
            .insert_new(WorkspaceSessionListEntry::new(
                WorkspaceSessionId::new("session-ui"),
                workspace_id.clone(),
                "UI Session",
                WorkspaceSessionStatus::Starting,
            ))
            .expect("session insert for test");

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

    #[test]
    fn app_layer_can_create_and_destroy_workspace_session_bundles() {
        let mut state = AppHostState::empty();
        let workspace_id = WorkspaceId::new("ws-1");
        let session_id = WorkspaceSessionId::new("session-1");

        state
            .create_workspace_session(session_id.clone(), workspace_id.clone(), "Session 1")
            .expect("session should be created");

        let session = state
            .sessions()
            .get(&session_id)
            .expect("session should exist after creation");
        assert_eq!(session.workspace_id(), &workspace_id);
        assert_eq!(session.status(), WorkspaceSessionStatus::Starting);
        assert!(!session.bundle().command_hub().is_open());
        assert_eq!(session.bundle().tabs().tab_order(), &[] as &[String]);

        let removed = state.destroy_workspace_session(&session_id);
        assert!(removed.is_some());
        assert!(state.sessions().get(&session_id).is_none());
    }

    #[test]
    fn session_bundles_scale_to_multiple_workspaces() {
        let mut state = AppHostState::empty();
        let workspace_a = WorkspaceId::new("ws-a");
        let workspace_b = WorkspaceId::new("ws-b");

        state
            .create_workspace_session(
                WorkspaceSessionId::new("session-a1"),
                workspace_a.clone(),
                "A-1",
            )
            .expect("a1");
        state
            .create_workspace_session(
                WorkspaceSessionId::new("session-a2"),
                workspace_a.clone(),
                "A-2",
            )
            .expect("a2");
        state
            .create_workspace_session(
                WorkspaceSessionId::new("session-b1"),
                workspace_b.clone(),
                "B-1",
            )
            .expect("b1");

        assert_eq!(
            state.sessions().ordered_for_workspace(&workspace_a).len(),
            2
        );
        assert_eq!(
            state.sessions().ordered_for_workspace(&workspace_b).len(),
            1
        );
    }

    #[test]
    fn session_creation_rejects_duplicate_session_id() {
        let mut state = AppHostState::empty();
        let workspace_id = WorkspaceId::new("ws-dup");
        let session_id = WorkspaceSessionId::new("session-dup");

        state
            .create_workspace_session(session_id.clone(), workspace_id.clone(), "first")
            .expect("first creation");
        let error = state
            .create_workspace_session(session_id.clone(), workspace_id, "second")
            .expect_err("duplicate should fail");

        assert_eq!(
            error,
            WorkspaceSessionCreationError::DuplicateSessionId { id: session_id }
        );
    }

    #[test]
    fn duplicate_session_creation_does_not_reset_existing_bundle_state() {
        let mut state = AppHostState::empty();
        let workspace_id = WorkspaceId::new("ws-bundle");
        let session_id = WorkspaceSessionId::new("session-bundle");

        state
            .create_workspace_session(session_id.clone(), workspace_id.clone(), "first")
            .expect("first creation");

        let session = state
            .session_mut(&session_id)
            .expect("created session should exist");
        session
            .bundle_mut()
            .command_hub_mut()
            .set_query("keep-this-query");
        session
            .bundle_mut()
            .tabs_mut()
            .set_tab_order(vec!["tab-1".to_string(), "tab-2".to_string()]);
        session
            .bundle_mut()
            .terminal_mut()
            .set_terminal_ids(vec!["term-1".to_string()]);

        let duplicate_error = state
            .create_workspace_session(session_id.clone(), workspace_id, "second")
            .expect_err("duplicate should fail");
        assert_eq!(
            duplicate_error,
            WorkspaceSessionCreationError::DuplicateSessionId {
                id: session_id.clone()
            }
        );

        let session_after = state
            .sessions()
            .get(&session_id)
            .expect("original session should remain");
        assert_eq!(
            session_after.bundle().command_hub().query(),
            "keep-this-query"
        );
        assert_eq!(
            session_after.bundle().tabs().tab_order(),
            &["tab-1".to_string(), "tab-2".to_string()]
        );
        assert_eq!(
            session_after.bundle().terminal().terminal_ids(),
            &["term-1".to_string()]
        );
    }
}
