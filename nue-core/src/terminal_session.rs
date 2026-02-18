use std::collections::{BTreeMap, VecDeque};

pub type TerminalCommandId = u64;
pub const DEFAULT_QUEUE_MAX_PENDING: usize = 4;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueCommandOutcome {
    Started {
        command_id: TerminalCommandId,
    },
    Queued {
        command_id: TerminalCommandId,
        position: usize,
    },
    RejectedQueueFull,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSession {
    workspace_session_id: String,
    queue_max_pending: usize,
    next_command_id: TerminalCommandId,
    running_command_id: Option<TerminalCommandId>,
    queued_command_ids: VecDeque<TerminalCommandId>,
    commands: BTreeMap<TerminalCommandId, TerminalCommandSnapshot>,
    events: VecDeque<TerminalSessionEvent>,
}

impl TerminalSession {
    pub fn new(workspace_session_id: impl Into<String>) -> Self {
        Self::new_with_queue_max_pending(workspace_session_id, DEFAULT_QUEUE_MAX_PENDING)
    }

    pub fn new_with_queue_max_pending(
        workspace_session_id: impl Into<String>,
        queue_max_pending: usize,
    ) -> Self {
        let queue_max_pending = queue_max_pending.max(1);
        Self {
            workspace_session_id: workspace_session_id.into(),
            queue_max_pending,
            next_command_id: 1,
            running_command_id: None,
            queued_command_ids: VecDeque::new(),
            commands: BTreeMap::new(),
            events: VecDeque::new(),
        }
    }

    pub fn enqueue_run_command(
        &mut self,
        agent_id: impl Into<String>,
        command_line: impl Into<String>,
    ) -> QueueCommandOutcome {
        if self.running_command_id.is_some()
            && self.queued_command_ids.len() >= self.queue_max_pending
        {
            self.push_notification("run_command キューが上限に達したため要求を拒否しました");
            return QueueCommandOutcome::RejectedQueueFull;
        }

        let command_id = self.allocate_command_id();
        self.commands.insert(
            command_id,
            TerminalCommandSnapshot {
                id: command_id,
                agent_id: agent_id.into(),
                command_line: command_line.into(),
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

    pub fn command(&self, command_id: TerminalCommandId) -> Option<&TerminalCommandSnapshot> {
        self.commands.get(&command_id)
    }

    pub fn queue_len(&self) -> usize {
        self.queued_command_ids.len()
    }

    pub fn drain_events(&mut self) -> Vec<TerminalSessionEvent> {
        self.events.drain(..).collect()
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

    /// 紐づくワークスペースセッションの識別子。
    pub fn workspace_session_id(&self) -> &str {
        &self.workspace_session_id
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

        let first = session.enqueue_run_command("agent-a", "cargo test");
        let second = session.enqueue_run_command("agent-b", "cargo clippy");

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
                TerminalSessionEvent::StatusChanged(TerminalStatusEvent {
                    workspace_session_id: "workspace-session-1".to_string(),
                    command_id: 2,
                    state: TerminalCommandState::Queued,
                    queue_length: 1,
                }),
            ]
        );
    }

    #[test]
    fn 完了時に次のqueuedがrunningへ昇格しイベントが流れる() {
        let mut session = test_session();

        session.enqueue_run_command("agent-a", "cargo test");
        session.enqueue_run_command("agent-b", "cargo clippy");
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
            ]
        );
    }

    #[test]
    fn 失敗時はfailedイベントが流れる() {
        let mut session = test_session();

        session.enqueue_run_command("agent-a", "cargo test");
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

        session.enqueue_run_command("agent-a", "cmd-1");
        session.enqueue_run_command("agent-b", "cmd-2");
        session.enqueue_run_command("agent-c", "cmd-3");
        session.drain_events();

        let result = session.enqueue_run_command("agent-d", "cmd-4");

        assert_eq!(result, QueueCommandOutcome::RejectedQueueFull);
        assert_eq!(session.queue_len(), 2);
        assert_eq!(
            session.drain_events(),
            vec![TerminalSessionEvent::Notification(
                TerminalNotificationEvent {
                    workspace_session_id: "workspace-session-1".to_string(),
                    message: "run_command キューが上限に達したため要求を拒否しました".to_string(),
                }
            )]
        );
    }

    #[test]
    fn queue_max_pending未設定時は既定値4を使う() {
        let mut session = TerminalSession::new("workspace-session-1");

        assert_eq!(
            session.enqueue_run_command("agent-a", "cmd-1"),
            QueueCommandOutcome::Started { command_id: 1 }
        );
        assert_eq!(
            session.enqueue_run_command("agent-b", "cmd-2"),
            QueueCommandOutcome::Queued {
                command_id: 2,
                position: 1
            }
        );
        assert_eq!(
            session.enqueue_run_command("agent-c", "cmd-3"),
            QueueCommandOutcome::Queued {
                command_id: 3,
                position: 2
            }
        );
        assert_eq!(
            session.enqueue_run_command("agent-d", "cmd-4"),
            QueueCommandOutcome::Queued {
                command_id: 4,
                position: 3
            }
        );
        assert_eq!(
            session.enqueue_run_command("agent-e", "cmd-5"),
            QueueCommandOutcome::Queued {
                command_id: 5,
                position: 4
            }
        );
        assert_eq!(
            session.enqueue_run_command("agent-f", "cmd-6"),
            QueueCommandOutcome::RejectedQueueFull
        );
    }
}
