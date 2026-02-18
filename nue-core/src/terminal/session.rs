use std::collections::{BTreeMap, VecDeque};
use std::env;
use std::fmt;
use std::path::{Component, Path, PathBuf};

use crate::terminal::scrollback::{Scrollback, ScrollbackLine};

pub type TerminalCommandId = u64;
pub type TerminalAuditId = u64;
pub const DEFAULT_QUEUE_MAX_PENDING: usize = 4;

/// `tool.execution.queue_max_pending` に対応する実行設定。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolExecutionConfig {
    queue_max_pending: usize,
}

impl ToolExecutionConfig {
    /// 指定した最大長で新しい設定を構築し、最小値 1 を保証する。
    pub fn new(queue_max_pending: usize) -> Self {
        Self {
            queue_max_pending: queue_max_pending.max(1),
        }
    }

    /// 現在のキュー最大数。
    pub fn queue_max_pending(&self) -> usize {
        self.queue_max_pending
    }
}

impl Default for ToolExecutionConfig {
    fn default() -> Self {
        Self::new(DEFAULT_QUEUE_MAX_PENDING)
    }
}

/// `run_command` を enqueued する際の追加情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunCommandRequest {
    pub agent_id: String,
    pub command_line: String,
    pub cwd: Option<PathBuf>,
    pub env_overrides: BTreeMap<String, String>,
}

impl RunCommandRequest {
    pub fn new(agent_id: impl Into<String>, command_line: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            command_line: command_line.into(),
            cwd: None,
            env_overrides: BTreeMap::new(),
        }
    }

    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn with_env_override(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_overrides.insert(key.into(), value.into());
        self
    }
}

/// `run_command` の `workspace_env` / `cwd` 検証に失敗した理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunCommandContextError {
    InvalidCwd {
        attempted: PathBuf,
        workspace_root: PathBuf,
    },
    UnauthorizedEnvAddition {
        key: String,
    },
    UnauthorizedEnvModification {
        key: String,
        expected: String,
        attempted: String,
    },
}

impl fmt::Display for RunCommandContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunCommandContextError::InvalidCwd {
                attempted,
                workspace_root,
            } => write!(
                f,
                "CWD `{}` は workspace_root `{}` に固定されているため拒否されました。",
                attempted.display(),
                workspace_root.display()
            ),
            RunCommandContextError::UnauthorizedEnvAddition { key } => write!(
                f,
                "workspace_env に含まれない環境変数 `{}` の追加は許可されていません。",
                key
            ),
            RunCommandContextError::UnauthorizedEnvModification {
                key,
                expected,
                attempted,
            } => write!(
                f,
                "環境変数 `{}` は `{}` で固定されており `{}` への上書きは許可されていません。",
                key, expected, attempted
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalCommandState {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalCommandSnapshot {
    pub id: TerminalCommandId,
    pub agent_id: String,
    pub command_line: String,
    pub state: TerminalCommandState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalStatusEvent {
    pub workspace_session_id: String,
    pub command_id: TerminalCommandId,
    pub state: TerminalCommandState,
    pub queue_length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalNotificationEvent {
    pub workspace_session_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalSessionEvent {
    StatusChanged(TerminalStatusEvent),
    Notification(TerminalNotificationEvent),
    Audit(TerminalAuditEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueCommandOutcome {
    Started {
        command_id: TerminalCommandId,
    },
    Queued {
        command_id: TerminalCommandId,
        position: usize,
    },
    RejectedQueueFull,
    RejectedContextViolation {
        error: RunCommandContextError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueRejectionReason {
    QueueFull {
        queue_max_pending: usize,
        agent_id: String,
        command_line: String,
    },
    ContextViolation {
        error: RunCommandContextError,
        agent_id: String,
        command_line: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueInterruptionReason {
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalAuditPayload {
    QueueTransition {
        command_id: TerminalCommandId,
        agent_id: String,
        command_line: String,
        state: TerminalCommandState,
        queue_length: usize,
    },
    QueueRejected {
        reason: QueueRejectionReason,
    },
    QueueInterrupted {
        command_id: TerminalCommandId,
        agent_id: String,
        command_line: String,
        reason: QueueInterruptionReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalAuditEvent {
    pub id: TerminalAuditId,
    pub workspace_session_id: String,
    pub payload: TerminalAuditPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSession {
    workspace_session_id: String,
    workspace_root: PathBuf,
    workspace_env: BTreeMap<String, String>,
    tool_execution_config: ToolExecutionConfig,
    next_command_id: TerminalCommandId,
    next_audit_id: TerminalAuditId,
    running_command_id: Option<TerminalCommandId>,
    queued_command_ids: VecDeque<TerminalCommandId>,
    commands: BTreeMap<TerminalCommandId, TerminalCommandSnapshot>,
    events: VecDeque<TerminalSessionEvent>,
    scrollback: Scrollback,
}

impl TerminalSession {
    pub fn new(workspace_session_id: impl Into<String>) -> Self {
        Self::new_with_workspace_context(
            workspace_session_id,
            Self::default_workspace_root(),
            BTreeMap::new(),
        )
    }

    pub fn new_with_queue_max_pending(
        workspace_session_id: impl Into<String>,
        queue_max_pending: usize,
    ) -> Self {
        Self::new_with_tool_execution_config(
            workspace_session_id,
            ToolExecutionConfig::new(queue_max_pending),
            Self::default_workspace_root(),
            BTreeMap::new(),
        )
    }

    pub fn new_with_workspace_context(
        workspace_session_id: impl Into<String>,
        workspace_root: impl Into<PathBuf>,
        workspace_env: BTreeMap<String, String>,
    ) -> Self {
        Self::new_with_tool_execution_config(
            workspace_session_id,
            ToolExecutionConfig::default(),
            workspace_root,
            workspace_env,
        )
    }

    pub fn new_with_tool_execution_config(
        workspace_session_id: impl Into<String>,
        tool_execution_config: ToolExecutionConfig,
        workspace_root: impl Into<PathBuf>,
        workspace_env: BTreeMap<String, String>,
    ) -> Self {
        let workspace_path = workspace_root.into();
        let normalized_root =
            Self::normalize_path(&workspace_path).unwrap_or_else(|| workspace_path.clone());

        Self {
            workspace_session_id: workspace_session_id.into(),
            workspace_root: normalized_root,
            workspace_env,
            tool_execution_config,
            next_command_id: 1,
            next_audit_id: 1,
            running_command_id: None,
            queued_command_ids: VecDeque::new(),
            commands: BTreeMap::new(),
            events: VecDeque::new(),
            scrollback: Scrollback::default(),
        }
    }

    fn default_workspace_root() -> PathBuf {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }

    /// キューの最大長。
    pub fn queue_max_pending(&self) -> usize {
        self.tool_execution_config.queue_max_pending()
    }

    /// 設定をそのまま取得。
    pub fn tool_execution_config(&self) -> &ToolExecutionConfig {
        &self.tool_execution_config
    }

    pub fn enqueue_run_command(&mut self, request: RunCommandRequest) -> QueueCommandOutcome {
        if let Err(error) = self.validate_run_command_context(&request) {
            let agent_id = request.agent_id.clone();
            let command_line = request.command_line.clone();
            let audit_error = error.clone();
            let message = format!("run_command を拒否しました: {}", error);
            self.push_notification(message);
            self.push_queue_rejection_audit_event(QueueRejectionReason::ContextViolation {
                error: audit_error,
                agent_id,
                command_line,
            });
            return QueueCommandOutcome::RejectedContextViolation { error };
        }

        if self.running_command_id.is_some()
            && self.queued_command_ids.len() >= self.queue_max_pending()
        {
            let agent_id = request.agent_id.clone();
            let command_line = request.command_line.clone();
            let queue_max = self.queue_max_pending();
            self.push_notification(format!(
                "run_command キューの上限({})に達したため実行を拒否しました。先行する run_command の完了を待つか中断してください。",
                queue_max
            ));
            self.push_queue_rejection_audit_event(QueueRejectionReason::QueueFull {
                queue_max_pending: queue_max,
                agent_id,
                command_line,
            });
            return QueueCommandOutcome::RejectedQueueFull;
        }

        let RunCommandRequest {
            agent_id,
            command_line,
            ..
        } = request;

        let command_id = self.allocate_command_id();
        self.commands.insert(
            command_id,
            TerminalCommandSnapshot {
                id: command_id,
                agent_id,
                command_line,
                state: TerminalCommandState::Queued,
            },
        );

        if self.running_command_id.is_none() {
            self.running_command_id = Some(command_id);
            self.update_command_state(command_id, TerminalCommandState::Running);
            self.push_status_event(command_id, TerminalCommandState::Running);

            return QueueCommandOutcome::Started { command_id };
        }

        self.queued_command_ids.push_back(command_id);
        self.push_status_event(command_id, TerminalCommandState::Queued);
        QueueCommandOutcome::Queued {
            command_id,
            position: self.queued_command_ids.len(),
        }
    }

    pub fn complete_running_command(&mut self, success: bool) -> Option<TerminalCommandId> {
        let completed_id = self.running_command_id.take()?;
        let state = if success {
            TerminalCommandState::Completed
        } else {
            TerminalCommandState::Failed
        };

        let completed_snapshot = self.commands.get(&completed_id).cloned();
        self.update_command_state(completed_id, state);
        self.push_status_event(completed_id, state);
        if !success {
            self.push_queue_interruption_audit_event(completed_id, QueueInterruptionReason::Failed);
        }
        if let Some(snapshot) = completed_snapshot {
            self.push_completion_notification(&snapshot, success);
        }
        self.promote_next_queued_command();

        Some(completed_id)
    }

    pub fn running_command(&self) -> Option<&TerminalCommandSnapshot> {
        let command_id = self.running_command_id?;
        self.commands.get(&command_id)
    }

    /// 実行出力をスクロールバックに追加する。
    pub fn push_output_line(&mut self, raw: impl Into<String>) {
        self.scrollback.push_line(raw);
    }

    /// 現在のスクロールバック行を順に取得する。
    pub fn scrollback_lines(&self) -> Vec<ScrollbackLine> {
        self.scrollback.lines().cloned().collect()
    }

    /// スクロールバックの最後の行。
    pub fn last_scrollback_line(&self) -> Option<&ScrollbackLine> {
        self.scrollback.last_line()
    }

    /// 保存しているスクロールバック行数。
    pub fn scrollback_len(&self) -> usize {
        self.scrollback.len()
    }

    /// スクロールバックの最大行数。
    pub fn scrollback_capacity(&self) -> usize {
        self.scrollback.capacity()
    }

    pub fn command(&self, command_id: TerminalCommandId) -> Option<&TerminalCommandSnapshot> {
        self.commands.get(&command_id)
    }

    pub fn queue_len(&self) -> usize {
        self.queued_command_ids.len()
    }

    pub fn drain_events(&mut self) -> Vec<TerminalSessionEvent> {
        self.events.drain(..).collect()
    }

    fn allocate_audit_id(&mut self) -> TerminalAuditId {
        let id = self.next_audit_id;
        self.next_audit_id += 1;
        id
    }

    fn push_audit_event(&mut self, payload: TerminalAuditPayload) {
        let event = TerminalAuditEvent {
            id: self.allocate_audit_id(),
            workspace_session_id: self.workspace_session_id.clone(),
            payload,
        };
        self.events.push_back(TerminalSessionEvent::Audit(event));
    }

    fn push_queue_transition_audit_event(
        &mut self,
        command_id: TerminalCommandId,
        state: TerminalCommandState,
    ) {
        if let Some(command) = self.commands.get(&command_id) {
            self.push_audit_event(TerminalAuditPayload::QueueTransition {
                command_id,
                agent_id: command.agent_id.clone(),
                command_line: command.command_line.clone(),
                state,
                queue_length: self.queued_command_ids.len(),
            });
        }
    }

    fn push_queue_rejection_audit_event(&mut self, reason: QueueRejectionReason) {
        self.push_audit_event(TerminalAuditPayload::QueueRejected { reason });
    }

    fn push_queue_interruption_audit_event(
        &mut self,
        command_id: TerminalCommandId,
        reason: QueueInterruptionReason,
    ) {
        if let Some(command) = self.commands.get(&command_id) {
            self.push_audit_event(TerminalAuditPayload::QueueInterrupted {
                command_id,
                agent_id: command.agent_id.clone(),
                command_line: command.command_line.clone(),
                reason,
            });
        }
    }

    fn allocate_command_id(&mut self) -> TerminalCommandId {
        let command_id = self.next_command_id;
        self.next_command_id += 1;
        command_id
    }

    fn update_command_state(&mut self, command_id: TerminalCommandId, state: TerminalCommandState) {
        if let Some(command) = self.commands.get_mut(&command_id) {
            command.state = state;
        }
    }

    fn promote_next_queued_command(&mut self) {
        if self.running_command_id.is_some() {
            return;
        }

        if let Some(command_id) = self.queued_command_ids.pop_front() {
            self.running_command_id = Some(command_id);
            self.update_command_state(command_id, TerminalCommandState::Running);
            self.push_status_event(command_id, TerminalCommandState::Running);
        }
    }

    fn push_status_event(&mut self, command_id: TerminalCommandId, state: TerminalCommandState) {
        self.events
            .push_back(TerminalSessionEvent::StatusChanged(TerminalStatusEvent {
                workspace_session_id: self.workspace_session_id.clone(),
                command_id,
                state,
                queue_length: self.queued_command_ids.len(),
            }));
        self.push_queue_transition_audit_event(command_id, state);
    }

    /// コマンド完了時の通知をキューへ追加する。
    fn push_completion_notification(&mut self, snapshot: &TerminalCommandSnapshot, success: bool) {
        let outcome_text = if success { "完了" } else { "失敗" };
        self.push_notification(format!(
            "run_command `{}` (agent {}) が{}しました",
            snapshot.command_line, snapshot.agent_id, outcome_text
        ));
    }

    fn push_notification(&mut self, message: impl Into<String>) {
        self.events.push_back(TerminalSessionEvent::Notification(
            TerminalNotificationEvent {
                workspace_session_id: self.workspace_session_id.clone(),
                message: message.into(),
            },
        ));
    }

    fn normalize_path(path: &Path) -> Option<PathBuf> {
        let mut normalized = PathBuf::new();
        let mut normal_segments = 0;

        for component in path.components() {
            match component {
                Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
                Component::RootDir => normalized.push(component.as_os_str()),
                Component::CurDir => {}
                Component::ParentDir => {
                    if normal_segments == 0 {
                        return None;
                    }
                    normalized.pop();
                    normal_segments -= 1;
                }
                Component::Normal(part) => {
                    normalized.push(part);
                    normal_segments += 1;
                }
            }
        }

        Some(normalized)
    }

    fn validate_run_command_context(
        &self,
        request: &RunCommandRequest,
    ) -> Result<(), RunCommandContextError> {
        if let Some(cwd) = &request.cwd {
            if cwd
                .components()
                .any(|component| matches!(component, Component::ParentDir))
            {
                return Err(RunCommandContextError::InvalidCwd {
                    attempted: cwd.clone(),
                    workspace_root: self.workspace_root.clone(),
                });
            }

            let normalized =
                Self::normalize_path(cwd).ok_or_else(|| RunCommandContextError::InvalidCwd {
                    attempted: cwd.clone(),
                    workspace_root: self.workspace_root.clone(),
                })?;

            if normalized != self.workspace_root {
                return Err(RunCommandContextError::InvalidCwd {
                    attempted: cwd.clone(),
                    workspace_root: self.workspace_root.clone(),
                });
            }
        }

        for (key, value) in request.env_overrides.iter() {
            match self.workspace_env.get(key) {
                None => {
                    return Err(RunCommandContextError::UnauthorizedEnvAddition {
                        key: key.clone(),
                    });
                }
                Some(expected) if expected != value => {
                    return Err(RunCommandContextError::UnauthorizedEnvModification {
                        key: key.clone(),
                        expected: expected.clone(),
                        attempted: value.clone(),
                    });
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// 紐づくワークスペースセッションの識別子。
    pub fn workspace_session_id(&self) -> &str {
        &self.workspace_session_id
    }

    /// 紐づくワークスペースルート。
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// ワークスペース環境変数設定。
    pub fn workspace_env(&self) -> &BTreeMap<String, String> {
        &self.workspace_env
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_session() -> TerminalSession {
        TerminalSession::new_with_queue_max_pending("workspace-session-1", 2)
    }

    #[test]
    fn run_command受付時にrunningとqueuedを振り分ける() {
        let mut session = test_session();

        let first = session.enqueue_run_command(RunCommandRequest::new("agent-a", "cargo test"));
        let second = session.enqueue_run_command(RunCommandRequest::new("agent-b", "cargo clippy"));

        assert_eq!(first, QueueCommandOutcome::Started { command_id: 1 });
        assert_eq!(
            second,
            QueueCommandOutcome::Queued {
                command_id: 2,
                position: 1
            }
        );

        assert_eq!(session.running_command().map(|command| command.id), Some(1));
        assert_eq!(session.queue_len(), 1);

        assert_eq!(
            session.drain_events(),
            vec![
                TerminalSessionEvent::StatusChanged(TerminalStatusEvent {
                    workspace_session_id: "workspace-session-1".to_string(),
                    command_id: 1,
                    state: TerminalCommandState::Running,
                    queue_length: 0,
                }),
                TerminalSessionEvent::Audit(TerminalAuditEvent {
                    id: 1,
                    workspace_session_id: "workspace-session-1".to_string(),
                    payload: TerminalAuditPayload::QueueTransition {
                        command_id: 1,
                        agent_id: "agent-a".to_string(),
                        command_line: "cargo test".to_string(),
                        state: TerminalCommandState::Running,
                        queue_length: 0,
                    },
                }),
                TerminalSessionEvent::StatusChanged(TerminalStatusEvent {
                    workspace_session_id: "workspace-session-1".to_string(),
                    command_id: 2,
                    state: TerminalCommandState::Queued,
                    queue_length: 1,
                }),
                TerminalSessionEvent::Audit(TerminalAuditEvent {
                    id: 2,
                    workspace_session_id: "workspace-session-1".to_string(),
                    payload: TerminalAuditPayload::QueueTransition {
                        command_id: 2,
                        agent_id: "agent-b".to_string(),
                        command_line: "cargo clippy".to_string(),
                        state: TerminalCommandState::Queued,
                        queue_length: 1,
                    },
                }),
            ]
        );
    }

    #[test]
    fn 完了時に次のqueuedがrunningへ昇格しイベントが流れる() {
        let mut session = test_session();

        session.enqueue_run_command(RunCommandRequest::new("agent-a", "cargo test"));
        session.enqueue_run_command(RunCommandRequest::new("agent-b", "cargo clippy"));
        session.drain_events();

        let completed = session.complete_running_command(true);

        assert_eq!(completed, Some(1));
        assert_eq!(session.running_command().map(|command| command.id), Some(2));
        assert_eq!(
            session.command(1).map(|command| command.state),
            Some(TerminalCommandState::Completed)
        );

        assert_eq!(
            session.drain_events(),
            vec![
                TerminalSessionEvent::StatusChanged(TerminalStatusEvent {
                    workspace_session_id: "workspace-session-1".to_string(),
                    command_id: 1,
                    state: TerminalCommandState::Completed,
                    queue_length: 1,
                }),
                TerminalSessionEvent::Audit(TerminalAuditEvent {
                    id: 3,
                    workspace_session_id: "workspace-session-1".to_string(),
                    payload: TerminalAuditPayload::QueueTransition {
                        command_id: 1,
                        agent_id: "agent-a".to_string(),
                        command_line: "cargo test".to_string(),
                        state: TerminalCommandState::Completed,
                        queue_length: 1,
                    },
                }),
                TerminalSessionEvent::Notification(TerminalNotificationEvent {
                    workspace_session_id: "workspace-session-1".to_string(),
                    message: "run_command `cargo test` (agent agent-a) が完了しました".to_string(),
                }),
                TerminalSessionEvent::StatusChanged(TerminalStatusEvent {
                    workspace_session_id: "workspace-session-1".to_string(),
                    command_id: 2,
                    state: TerminalCommandState::Running,
                    queue_length: 0,
                }),
                TerminalSessionEvent::Audit(TerminalAuditEvent {
                    id: 4,
                    workspace_session_id: "workspace-session-1".to_string(),
                    payload: TerminalAuditPayload::QueueTransition {
                        command_id: 2,
                        agent_id: "agent-b".to_string(),
                        command_line: "cargo clippy".to_string(),
                        state: TerminalCommandState::Running,
                        queue_length: 0,
                    },
                }),
            ]
        );
    }

    #[test]
    fn 失敗時はfailedイベントが流れる() {
        let mut session = test_session();

        session.enqueue_run_command(RunCommandRequest::new("agent-a", "cargo test"));
        session.drain_events();

        let completed = session.complete_running_command(false);

        assert_eq!(completed, Some(1));
        assert_eq!(session.running_command(), None);
        assert_eq!(
            session.command(1).map(|command| command.state),
            Some(TerminalCommandState::Failed)
        );
        assert_eq!(
            session.drain_events(),
            vec![
                TerminalSessionEvent::StatusChanged(TerminalStatusEvent {
                    workspace_session_id: "workspace-session-1".to_string(),
                    command_id: 1,
                    state: TerminalCommandState::Failed,
                    queue_length: 0,
                }),
                TerminalSessionEvent::Audit(TerminalAuditEvent {
                    id: 2,
                    workspace_session_id: "workspace-session-1".to_string(),
                    payload: TerminalAuditPayload::QueueTransition {
                        command_id: 1,
                        agent_id: "agent-a".to_string(),
                        command_line: "cargo test".to_string(),
                        state: TerminalCommandState::Failed,
                        queue_length: 0,
                    },
                }),
                TerminalSessionEvent::Audit(TerminalAuditEvent {
                    id: 3,
                    workspace_session_id: "workspace-session-1".to_string(),
                    payload: TerminalAuditPayload::QueueInterrupted {
                        command_id: 1,
                        agent_id: "agent-a".to_string(),
                        command_line: "cargo test".to_string(),
                        reason: QueueInterruptionReason::Failed,
                    },
                }),
                TerminalSessionEvent::Notification(TerminalNotificationEvent {
                    workspace_session_id: "workspace-session-1".to_string(),
                    message: "run_command `cargo test` (agent agent-a) が失敗しました".to_string(),
                }),
            ]
        );
    }

    #[test]
    fn キュー上限超過時は拒否と通知イベントを返す() {
        let mut session = test_session();

        session.enqueue_run_command(RunCommandRequest::new("agent-a", "cmd-1"));
        session.enqueue_run_command(RunCommandRequest::new("agent-b", "cmd-2"));
        session.enqueue_run_command(RunCommandRequest::new("agent-c", "cmd-3"));
        session.drain_events();

        let result = session.enqueue_run_command(RunCommandRequest::new("agent-d", "cmd-4"));

        assert_eq!(result, QueueCommandOutcome::RejectedQueueFull);
        assert_eq!(session.queue_len(), 2);
        let expected_message = format!(
            "run_command キューの上限({})に達したため実行を拒否しました。先行する run_command の完了を待つか中断してください。",
            session.queue_max_pending()
        );
        assert_eq!(
            session.drain_events(),
            vec![
                TerminalSessionEvent::Notification(TerminalNotificationEvent {
                    workspace_session_id: "workspace-session-1".to_string(),
                    message: expected_message,
                }),
                TerminalSessionEvent::Audit(TerminalAuditEvent {
                    id: 4,
                    workspace_session_id: "workspace-session-1".to_string(),
                    payload: TerminalAuditPayload::QueueRejected {
                        reason: QueueRejectionReason::QueueFull {
                            queue_max_pending: session.queue_max_pending(),
                            agent_id: "agent-d".to_string(),
                            command_line: "cmd-4".to_string(),
                        },
                    },
                }),
            ]
        );
    }

    #[test]
    fn queue_max_pending未設定時は既定値4を使う() {
        let mut session = TerminalSession::new("workspace-session-1");

        assert_eq!(
            session.enqueue_run_command(RunCommandRequest::new("agent-a", "cmd-1")),
            QueueCommandOutcome::Started { command_id: 1 }
        );
        assert_eq!(
            session.enqueue_run_command(RunCommandRequest::new("agent-b", "cmd-2")),
            QueueCommandOutcome::Queued {
                command_id: 2,
                position: 1
            }
        );
        assert_eq!(
            session.enqueue_run_command(RunCommandRequest::new("agent-c", "cmd-3")),
            QueueCommandOutcome::Queued {
                command_id: 3,
                position: 2
            }
        );
        assert_eq!(
            session.enqueue_run_command(RunCommandRequest::new("agent-d", "cmd-4")),
            QueueCommandOutcome::Queued {
                command_id: 4,
                position: 3
            }
        );
        assert_eq!(
            session.enqueue_run_command(RunCommandRequest::new("agent-e", "cmd-5")),
            QueueCommandOutcome::Queued {
                command_id: 5,
                position: 4
            }
        );
        assert_eq!(
            session.enqueue_run_command(RunCommandRequest::new("agent-f", "cmd-6")),
            QueueCommandOutcome::RejectedQueueFull
        );
    }

    #[test]
    fn queue_max_pendingメソッドは設定値を返す() {
        let session = TerminalSession::new_with_queue_max_pending("workspace-session-3", 3);
        assert_eq!(session.queue_max_pending(), 3);
    }

    #[test]
    fn tool_execution_configは最小1でクランプされる() {
        assert_eq!(ToolExecutionConfig::new(0).queue_max_pending(), 1);
    }

    #[test]
    fn scrollback_tracks_output_lines() {
        let mut session = TerminalSession::new("workspace-session-1");

        session.push_output_line("first");
        session.push_output_line("second");

        assert_eq!(session.scrollback_len(), 2);
        let raws: Vec<String> = session
            .scrollback_lines()
            .iter()
            .map(|line| line.raw().to_string())
            .collect();
        assert_eq!(raws, vec!["first".to_string(), "second".to_string()]);
        assert_eq!(session.last_scrollback_line().unwrap().raw(), "second");
    }

    #[test]
    fn scrollback_starts_empty() {
        let session = TerminalSession::new("workspace-session-2");
        assert!(session.last_scrollback_line().is_none());
        assert_eq!(session.scrollback_len(), 0);
    }

    #[test]
    fn cwdがworkspace_rootでないと拒否される() {
        let mut session = TerminalSession::new_with_workspace_context(
            "workspace-session-env",
            "/workspace-root",
            BTreeMap::new(),
        );

        let request = RunCommandRequest::new("agent-d", "cmd").with_cwd("/other");
        let expected_error = RunCommandContextError::InvalidCwd {
            attempted: PathBuf::from("/other"),
            workspace_root: PathBuf::from("/workspace-root"),
        };

        assert_eq!(
            session.enqueue_run_command(request),
            QueueCommandOutcome::RejectedContextViolation {
                error: expected_error.clone()
            }
        );
        assert_eq!(session.queue_len(), 0);
        assert_eq!(
            session.drain_events(),
            vec![
                TerminalSessionEvent::Notification(TerminalNotificationEvent {
                    workspace_session_id: "workspace-session-env".to_string(),
                    message: format!("run_command を拒否しました: {}", expected_error),
                }),
                TerminalSessionEvent::Audit(TerminalAuditEvent {
                    id: 1,
                    workspace_session_id: "workspace-session-env".to_string(),
                    payload: TerminalAuditPayload::QueueRejected {
                        reason: QueueRejectionReason::ContextViolation {
                            error: expected_error,
                            agent_id: "agent-d".to_string(),
                            command_line: "cmd".to_string(),
                        },
                    },
                }),
            ]
        );
    }

    #[test]
    fn workspace_envにない変数の追加は拒否される() {
        let mut session = TerminalSession::new_with_workspace_context(
            "workspace-session-env",
            "/workspace-root",
            BTreeMap::new(),
        );

        let request = RunCommandRequest::new("agent-x", "cmd").with_env_override("UNSAFE", "1");
        let expected_error = RunCommandContextError::UnauthorizedEnvAddition {
            key: "UNSAFE".to_string(),
        };

        assert_eq!(
            session.enqueue_run_command(request),
            QueueCommandOutcome::RejectedContextViolation {
                error: expected_error.clone()
            }
        );
        assert_eq!(session.queue_len(), 0);
        assert_eq!(
            session.drain_events(),
            vec![
                TerminalSessionEvent::Notification(TerminalNotificationEvent {
                    workspace_session_id: "workspace-session-env".to_string(),
                    message: format!("run_command を拒否しました: {}", expected_error),
                }),
                TerminalSessionEvent::Audit(TerminalAuditEvent {
                    id: 1,
                    workspace_session_id: "workspace-session-env".to_string(),
                    payload: TerminalAuditPayload::QueueRejected {
                        reason: QueueRejectionReason::ContextViolation {
                            error: expected_error,
                            agent_id: "agent-x".to_string(),
                            command_line: "cmd".to_string(),
                        },
                    },
                }),
            ]
        );
    }

    #[test]
    fn workspace_envの既存値を上書きする操作も拒否される() {
        let mut workspace_env = BTreeMap::new();
        workspace_env.insert("SAFE".to_string(), "ALLOWED".to_string());

        let mut session = TerminalSession::new_with_workspace_context(
            "workspace-session-env",
            "/workspace-root",
            workspace_env,
        );

        let request = RunCommandRequest::new("agent-y", "cmd").with_env_override("SAFE", "DENIED");
        let expected_error = RunCommandContextError::UnauthorizedEnvModification {
            key: "SAFE".to_string(),
            expected: "ALLOWED".to_string(),
            attempted: "DENIED".to_string(),
        };

        assert_eq!(
            session.enqueue_run_command(request),
            QueueCommandOutcome::RejectedContextViolation {
                error: expected_error.clone()
            }
        );
        assert_eq!(session.queue_len(), 0);
        assert_eq!(
            session.drain_events(),
            vec![
                TerminalSessionEvent::Notification(TerminalNotificationEvent {
                    workspace_session_id: "workspace-session-env".to_string(),
                    message: format!("run_command を拒否しました: {}", expected_error),
                }),
                TerminalSessionEvent::Audit(TerminalAuditEvent {
                    id: 1,
                    workspace_session_id: "workspace-session-env".to_string(),
                    payload: TerminalAuditPayload::QueueRejected {
                        reason: QueueRejectionReason::ContextViolation {
                            error: expected_error,
                            agent_id: "agent-y".to_string(),
                            command_line: "cmd".to_string(),
                        },
                    },
                }),
            ]
        );
    }
}
