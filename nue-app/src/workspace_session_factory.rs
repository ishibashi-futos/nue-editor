use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tokio::sync::mpsc;

use crate::app_host_state::{
    AppHostState, NotificationId, NotificationLevel, WorkspaceId, WorkspaceListEntry,
    WorkspaceSessionCreationError, WorkspaceSessionId, WorkspaceSessionStatus, WorkspaceStatus,
};
use nue_core::command::actions::{CommandHubActionModel, TerminalItem, WorkspaceItem};
use nue_core::workspace::legacy_workspace_editor::{
    LegacyWorkspaceEditor, LegacyWorkspaceEditorOpenError,
};
use nue_core::workspace::rail::{WorkspaceRailModel, WorkspaceRailState};
use nue_core::workspace::registry::WorkspaceRegistry;
use nue_ui::command_hub::CommandHubUiController;
use nue_ui::editor_events::EditorEventSubscriber;
use nue_ui::legacy_explorer::LegacyExplorerModel;
use nue_ui::tab_bar::TabBarUiController;
use nue_ui::terminal::TerminalUiController;

const DEFAULT_TERMINAL_ID: &str = "terminal-1";

pub struct WorkspaceSessionFactory {
    runtimes: BTreeMap<WorkspaceSessionId, WorkspaceSessionRuntimeBundle>,
}

impl WorkspaceSessionFactory {
    pub fn new() -> Self {
        Self {
            runtimes: BTreeMap::new(),
        }
    }

    pub fn runtime(
        &self,
        session_id: &WorkspaceSessionId,
    ) -> Option<&WorkspaceSessionRuntimeBundle> {
        self.runtimes.get(session_id)
    }

    pub fn runtime_mut(
        &mut self,
        session_id: &WorkspaceSessionId,
    ) -> Option<&mut WorkspaceSessionRuntimeBundle> {
        self.runtimes.get_mut(session_id)
    }

    pub fn create_for_workspace_path(
        &mut self,
        app_state: &mut AppHostState,
        workspace_root: impl AsRef<Path>,
    ) -> Result<WorkspaceSessionCreated, WorkspaceSessionFactoryError> {
        let workspace_root = workspace_root.as_ref();
        let canonical_root = std::fs::canonicalize(workspace_root).map_err(|_| {
            WorkspaceSessionFactoryError::WorkspacePathUnavailable {
                path: workspace_root.to_path_buf(),
            }
        })?;

        let workspace_label = canonical_root
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| canonical_root.display().to_string());
        let workspace_id = WorkspaceId::new(format!("workspace:{}", canonical_root.display()));

        self.create_for_workspace_descriptor(
            app_state,
            WorkspaceDescriptor {
                workspace_id,
                root_path: canonical_root,
                display_name: workspace_label,
                rail_state: WorkspaceRailState::Idle,
            },
        )
    }

    pub fn create_for_registry_workspace(
        &mut self,
        app_state: &mut AppHostState,
        registry: &WorkspaceRegistry,
        workspace_id: &str,
    ) -> Result<WorkspaceSessionCreated, WorkspaceSessionFactoryError> {
        let Some(workspace) = registry
            .workspaces()
            .iter()
            .find(|workspace| workspace.metadata().workspace_id == workspace_id)
        else {
            return Err(WorkspaceSessionFactoryError::RegistryWorkspaceNotFound {
                workspace_id: workspace_id.to_string(),
            });
        };

        self.create_for_workspace_descriptor(
            app_state,
            WorkspaceDescriptor::from_rail_model(workspace),
        )
    }

    pub fn notify_error(
        &self,
        app_state: &mut AppHostState,
        error: &WorkspaceSessionFactoryError,
    ) -> NotificationId {
        let notification = error.to_notification();
        app_state
            .notifications_mut()
            .push(notification.level, notification.message)
    }

    fn create_for_workspace_descriptor(
        &mut self,
        app_state: &mut AppHostState,
        descriptor: WorkspaceDescriptor,
    ) -> Result<WorkspaceSessionCreated, WorkspaceSessionFactoryError> {
        let session_id = workspace_session_id_for(&descriptor.workspace_id);
        let session_title = descriptor.display_name.clone();
        let runtime_bundle =
            WorkspaceSessionRuntimeBundle::new(&session_id, &descriptor).map_err(|source| {
                WorkspaceSessionFactoryError::EditorInitializationFailed {
                    workspace_id: descriptor.workspace_id.as_str().to_string(),
                    root_path: descriptor.root_path.clone(),
                    source,
                }
            })?;

        app_state.workspaces_mut().upsert(
            WorkspaceListEntry::new(
                descriptor.workspace_id.clone(),
                descriptor.root_path.clone(),
                descriptor.display_name.clone(),
            )
            .with_status(map_workspace_status(descriptor.rail_state)),
        );

        app_state
            .create_workspace_session(
                session_id.clone(),
                descriptor.workspace_id.clone(),
                session_title,
            )
            .map_err(
                |source| WorkspaceSessionFactoryError::SessionCreationRejected {
                    workspace_id: descriptor.workspace_id.as_str().to_string(),
                    root_path: descriptor.root_path.clone(),
                    source,
                },
            )?;

        if let Some(session_entry) = app_state.session_mut(&session_id) {
            session_entry.set_status(WorkspaceSessionStatus::Ready);

            let tabs = runtime_bundle.tab_bar.tabs();
            let active_tab_id = runtime_bundle.tab_bar.active_tab_id();
            let tab_order = tabs.into_iter().map(|tab| tab.id).collect();
            let bundle = session_entry.bundle_mut();
            bundle.tabs_mut().set_active_tab_id(active_tab_id);
            bundle.tabs_mut().set_tab_order(tab_order);
            bundle
                .terminal_mut()
                .set_active_terminal_id(Some(DEFAULT_TERMINAL_ID.to_string()));
            bundle
                .terminal_mut()
                .set_terminal_ids(vec![DEFAULT_TERMINAL_ID.to_string()]);
            bundle
                .subscribers_mut()
                .set_editor_subscriber_attached(true);
            bundle
                .subscribers_mut()
                .set_explorer_subscriber_attached(true);
            bundle
                .subscribers_mut()
                .set_command_hub_subscriber_attached(true);
            bundle.subscribers_mut().set_tab_subscriber_attached(true);
            bundle
                .subscribers_mut()
                .set_terminal_subscriber_attached(true);
        }

        app_state
            .select_workspace_for_session_display(&descriptor.workspace_id)
            .expect("新規作成したワークスペースは表示選択できるはず");

        self.runtimes.insert(session_id.clone(), runtime_bundle);

        Ok(WorkspaceSessionCreated {
            workspace_id: descriptor.workspace_id,
            session_id,
        })
    }
}

impl Default for WorkspaceSessionFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSessionCreated {
    pub workspace_id: WorkspaceId,
    pub session_id: WorkspaceSessionId,
}

pub struct WorkspaceSessionRuntimeBundle {
    legacy_editor: LegacyWorkspaceEditor,
    explorer: LegacyExplorerModel,
    command_hub: CommandHubUiController,
    editor_events: EditorEventSubscriber,
    tab_bar: TabBarUiController,
    terminal_ui: TerminalUiController,
    terminal_pty: TerminalPtyBridge,
    terminal_runtime_ingress: TerminalPtyRuntimeIngress,
}

impl WorkspaceSessionRuntimeBundle {
    fn new(
        session_id: &WorkspaceSessionId,
        workspace: &WorkspaceDescriptor,
    ) -> Result<Self, LegacyWorkspaceEditorOpenError> {
        let legacy_editor =
            LegacyWorkspaceEditor::open(workspace.root_path.to_str().unwrap_or_default())?;
        let explorer = LegacyExplorerModel::new(&legacy_editor);
        let tab_bar = TabBarUiController::new();
        let terminal_ui =
            TerminalUiController::new(session_id.as_str(), workspace.root_path.clone());
        let command_hub = CommandHubUiController::new(CommandHubActionModel::new(
            vec![WorkspaceItem {
                id: workspace.workspace_id.as_str().to_string(),
                display_name: workspace.display_name.clone(),
                root_path: workspace.root_path.display().to_string(),
                is_active: true,
            }],
            Vec::new(),
            vec![TerminalItem {
                id: DEFAULT_TERMINAL_ID.to_string(),
                title: "Terminal".to_string(),
                is_active: true,
            }],
            tab_bar.tabs(),
        ));
        let (terminal_pty, terminal_runtime_ingress) = terminal_pty_bridge_pair();

        Ok(Self {
            legacy_editor,
            explorer,
            command_hub,
            editor_events: EditorEventSubscriber::new(),
            tab_bar,
            terminal_ui,
            terminal_pty,
            terminal_runtime_ingress,
        })
    }

    pub fn legacy_editor(&self) -> &LegacyWorkspaceEditor {
        &self.legacy_editor
    }

    pub fn legacy_editor_mut(&mut self) -> &mut LegacyWorkspaceEditor {
        &mut self.legacy_editor
    }

    pub fn explorer(&self) -> &LegacyExplorerModel {
        &self.explorer
    }

    pub fn explorer_mut(&mut self) -> &mut LegacyExplorerModel {
        &mut self.explorer
    }

    pub fn command_hub(&self) -> &CommandHubUiController {
        &self.command_hub
    }

    pub fn command_hub_mut(&mut self) -> &mut CommandHubUiController {
        &mut self.command_hub
    }

    pub fn editor_events(&self) -> &EditorEventSubscriber {
        &self.editor_events
    }

    pub fn editor_events_mut(&mut self) -> &mut EditorEventSubscriber {
        &mut self.editor_events
    }

    pub fn tab_bar(&self) -> &TabBarUiController {
        &self.tab_bar
    }

    pub fn tab_bar_mut(&mut self) -> &mut TabBarUiController {
        &mut self.tab_bar
    }

    pub fn terminal_ui(&self) -> &TerminalUiController {
        &self.terminal_ui
    }

    pub fn terminal_ui_mut(&mut self) -> &mut TerminalUiController {
        &mut self.terminal_ui
    }

    pub fn terminal_pty(&self) -> &TerminalPtyBridge {
        &self.terminal_pty
    }

    pub fn terminal_pty_mut(&mut self) -> &mut TerminalPtyBridge {
        &mut self.terminal_pty
    }

    pub fn terminal_runtime_ingress_mut(&mut self) -> &mut TerminalPtyRuntimeIngress {
        &mut self.terminal_runtime_ingress
    }
}

#[derive(Debug)]
pub struct TerminalPtyBridge {
    request_tx: mpsc::UnboundedSender<TerminalPtyRequest>,
    event_rx: mpsc::UnboundedReceiver<TerminalPtyEvent>,
}

impl TerminalPtyBridge {
    pub fn send_request(&self, request: TerminalPtyRequest) -> Result<(), TerminalPtyRequest> {
        self.request_tx.send(request).map_err(|error| error.0)
    }

    pub fn try_recv_event(&mut self) -> Result<TerminalPtyEvent, mpsc::error::TryRecvError> {
        self.event_rx.try_recv()
    }
}

#[derive(Debug)]
pub struct TerminalPtyRuntimeIngress {
    request_rx: mpsc::UnboundedReceiver<TerminalPtyRequest>,
    event_tx: mpsc::UnboundedSender<TerminalPtyEvent>,
}

impl TerminalPtyRuntimeIngress {
    pub fn try_recv_request(&mut self) -> Result<TerminalPtyRequest, mpsc::error::TryRecvError> {
        self.request_rx.try_recv()
    }

    pub fn send_event(&self, event: TerminalPtyEvent) -> Result<(), TerminalPtyEvent> {
        self.event_tx.send(event).map_err(|error| error.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalPtyRequest {
    RunCommand {
        terminal_id: String,
        command_line: String,
    },
    Interrupt {
        terminal_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalPtyEvent {
    Started { terminal_id: String },
    StdoutLine { terminal_id: String, line: String },
    Exited { terminal_id: String, success: bool },
}

fn terminal_pty_bridge_pair() -> (TerminalPtyBridge, TerminalPtyRuntimeIngress) {
    let (request_tx, request_rx) = mpsc::unbounded_channel();
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    (
        TerminalPtyBridge {
            request_tx,
            event_rx,
        },
        TerminalPtyRuntimeIngress {
            request_rx,
            event_tx,
        },
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceSessionFactoryError {
    RegistryWorkspaceNotFound {
        workspace_id: String,
    },
    WorkspacePathUnavailable {
        path: PathBuf,
    },
    EditorInitializationFailed {
        workspace_id: String,
        root_path: PathBuf,
        source: LegacyWorkspaceEditorOpenError,
    },
    SessionCreationRejected {
        workspace_id: String,
        root_path: PathBuf,
        source: WorkspaceSessionCreationError,
    },
}

impl WorkspaceSessionFactoryError {
    pub fn to_notification(&self) -> WorkspaceSessionFactoryNotification {
        let message = match self {
            Self::RegistryWorkspaceNotFound { workspace_id } => {
                format!("ワークスペース `{workspace_id}` が WorkspaceRegistry に存在しません。")
            }
            Self::WorkspacePathUnavailable { path } => {
                format!("ワークスペースを開けません: {}", path.display())
            }
            Self::EditorInitializationFailed { root_path, .. } => {
                format!(
                    "ワークスペースセッション初期化に失敗しました: {}",
                    root_path.display()
                )
            }
            Self::SessionCreationRejected { workspace_id, .. } => {
                format!("ワークスペースセッションを作成できませんでした: {workspace_id}")
            }
        };
        WorkspaceSessionFactoryNotification {
            level: NotificationLevel::Error,
            message,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSessionFactoryNotification {
    pub level: NotificationLevel,
    pub message: String,
}

#[derive(Debug, Clone)]
struct WorkspaceDescriptor {
    workspace_id: WorkspaceId,
    root_path: PathBuf,
    display_name: String,
    rail_state: WorkspaceRailState,
}

impl WorkspaceDescriptor {
    fn from_rail_model(model: &WorkspaceRailModel) -> Self {
        Self {
            workspace_id: WorkspaceId::new(model.metadata().workspace_id.clone()),
            root_path: PathBuf::from(model.metadata().root_path.clone()),
            display_name: model.metadata().display_name.clone(),
            rail_state: model.snapshot().state,
        }
    }
}

fn workspace_session_id_for(workspace_id: &WorkspaceId) -> WorkspaceSessionId {
    WorkspaceSessionId::new(format!("session:{}", workspace_id.as_str()))
}

fn map_workspace_status(state: WorkspaceRailState) -> WorkspaceStatus {
    match state {
        WorkspaceRailState::Busy | WorkspaceRailState::Waiting => WorkspaceStatus::Loading,
        WorkspaceRailState::Error => WorkspaceStatus::Error,
        WorkspaceRailState::Idle => WorkspaceStatus::Ready,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_host_state::ScreenKind;
    use nue_core::workspace::registry::{AddWorkspaceOutcome, WorkspacePathValidation};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestWorkspaceDir {
        path: PathBuf,
    }

    impl TestWorkspaceDir {
        fn new(prefix: &str) -> Self {
            static SEQ: AtomicU64 = AtomicU64::new(1);
            let unique = SEQ.fetch_add(1, Ordering::Relaxed);
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("時刻取得")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "nue-app-{prefix}-{}-{ts}-{unique}",
                std::process::id()
            ));
            fs::create_dir_all(path.join("src")).expect("workspace 作成");
            fs::write(path.join("README.md"), "# test\n").expect("README 作成");
            fs::write(path.join("src/main.rs"), "fn main() {}\n").expect("main 作成");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestWorkspaceDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn workspace_pathからセッションを生成できる() {
        let workspace = TestWorkspaceDir::new("path-factory");
        let mut app_state = AppHostState::empty();
        let mut factory = WorkspaceSessionFactory::new();

        let created = factory
            .create_for_workspace_path(&mut app_state, workspace.path())
            .expect("セッション生成");

        let session = app_state
            .sessions()
            .get(&created.session_id)
            .expect("session が state に入る");
        assert_eq!(session.status(), WorkspaceSessionStatus::Ready);
        assert_eq!(app_state.ui_shell().screen(), ScreenKind::WorkspaceSession);
        assert_eq!(
            app_state.active_workspace().current(),
            Some(&created.workspace_id)
        );
        assert!(factory.runtime(&created.session_id).is_some());

        let runtime = factory.runtime(&created.session_id).expect("runtime 取得");
        assert!(!runtime.explorer().nodes().is_empty());
        assert_eq!(
            runtime.terminal_ui().workspace_session_id(),
            created.session_id.as_str()
        );
        assert_eq!(
            session.bundle().terminal().active_terminal_id(),
            Some(DEFAULT_TERMINAL_ID)
        );
        assert!(session.bundle().subscribers().editor_subscriber_attached());
    }

    #[test]
    fn workspace_registryの登録項目からセッションを生成できる() {
        let workspace = TestWorkspaceDir::new("registry-factory");
        let mut registry = WorkspaceRegistry::new();
        registry.open_add_dialog();
        let validation = registry.update_dialog_path(workspace.path().display().to_string());
        assert_eq!(validation, WorkspacePathValidation::Valid);
        let added_id = match registry.submit_add() {
            AddWorkspaceOutcome::Added { workspace_id } => workspace_id,
            AddWorkspaceOutcome::ValidationFailed { reason } => {
                panic!("登録失敗: {reason:?}")
            }
        };

        let mut app_state = AppHostState::empty();
        let mut factory = WorkspaceSessionFactory::new();
        let created = factory
            .create_for_registry_workspace(&mut app_state, &registry, &added_id)
            .expect("registry から生成");

        assert_eq!(created.workspace_id.as_str(), added_id);
        let session = app_state
            .sessions()
            .get(&created.session_id)
            .expect("session 作成");
        assert_eq!(session.status(), WorkspaceSessionStatus::Ready);
        assert_eq!(app_state.workspaces().len(), 1);
        assert_eq!(
            app_state.workspaces().ordered()[0].status(),
            WorkspaceStatus::Ready
        );
    }

    #[test]
    fn 失敗をエラー通知へ変換できる() {
        let mut app_state = AppHostState::empty();
        let factory = WorkspaceSessionFactory::new();
        let error = WorkspaceSessionFactoryError::RegistryWorkspaceNotFound {
            workspace_id: "missing".to_string(),
        };

        let notification_id = factory.notify_error(&mut app_state, &error);

        let items = app_state.notifications().items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id(), notification_id);
        assert_eq!(items[0].level(), NotificationLevel::Error);
        assert!(items[0].message().contains("WorkspaceRegistry"));
    }

    #[test]
    fn terminal_pty受け口をtokio_channelで接続できる() {
        let workspace = TestWorkspaceDir::new("terminal-bridge");
        let mut app_state = AppHostState::empty();
        let mut factory = WorkspaceSessionFactory::new();
        let created = factory
            .create_for_workspace_path(&mut app_state, workspace.path())
            .expect("セッション生成");

        let runtime = factory
            .runtime_mut(&created.session_id)
            .expect("runtime 取得");
        runtime
            .terminal_pty()
            .send_request(TerminalPtyRequest::RunCommand {
                terminal_id: DEFAULT_TERMINAL_ID.to_string(),
                command_line: "echo test".to_string(),
            })
            .expect("request 送信");

        let request = runtime
            .terminal_runtime_ingress_mut()
            .try_recv_request()
            .expect("request 受信");
        assert_eq!(
            request,
            TerminalPtyRequest::RunCommand {
                terminal_id: DEFAULT_TERMINAL_ID.to_string(),
                command_line: "echo test".to_string(),
            }
        );

        runtime
            .terminal_runtime_ingress_mut()
            .send_event(TerminalPtyEvent::Started {
                terminal_id: DEFAULT_TERMINAL_ID.to_string(),
            })
            .expect("event 送信");
        let event = runtime
            .terminal_pty_mut()
            .try_recv_event()
            .expect("event 受信");
        assert_eq!(
            event,
            TerminalPtyEvent::Started {
                terminal_id: DEFAULT_TERMINAL_ID.to_string(),
            }
        );
    }
}
