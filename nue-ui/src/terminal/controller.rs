use std::collections::BTreeMap;
use std::mem;
use std::path::PathBuf;

use nue_core::terminal::scrollback::ScrollbackLine;
use nue_core::terminal::session::{
    QueueCommandOutcome, QueueInterruptionReason, QueueRejectionReason, RunCommandRequest,
    TerminalAuditEvent, TerminalAuditId, TerminalAuditPayload, TerminalCommandId,
    TerminalCommandSnapshot, TerminalCommandState, TerminalSession, TerminalSessionEvent,
    ToolExecutionConfig,
};

/// TerminalAuditEvent を UI で表示しやすく整形した結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalAuditMessage {
    pub id: TerminalAuditId,
    pub workspace_session_id: String,
    pub payload: TerminalAuditPayload,
    pub summary: String,
}

impl TerminalAuditMessage {
    fn from_event(event: &TerminalAuditEvent) -> Self {
        Self {
            id: event.id,
            workspace_session_id: event.workspace_session_id.clone(),
            payload: event.payload.clone(),
            summary: summarize_audit_payload(&event.payload),
        }
    }
}

fn summarize_audit_payload(payload: &TerminalAuditPayload) -> String {
    match payload {
        TerminalAuditPayload::QueueTransition {
            command_line,
            agent_id,
            state,
            queue_length,
            ..
        } => format!(
            "run_command `{}` (agent {}) が{}（キュー長：{}）",
            command_line,
            agent_id,
            describe_state(*state),
            queue_length
        ),
        TerminalAuditPayload::QueueRejected { reason } => format_queue_rejection(reason),
        TerminalAuditPayload::QueueInterrupted {
            command_line,
            agent_id,
            reason,
            ..
        } => format!(
            "run_command `{}` (agent {}) は{}により中断されました。",
            command_line,
            agent_id,
            describe_interruption(reason),
        ),
    }
}

fn format_queue_rejection(reason: &QueueRejectionReason) -> String {
    match reason {
        QueueRejectionReason::QueueFull {
            queue_max_pending,
            agent_id,
            command_line,
        } => format!(
            "run_command `{}` (agent {}) はキュー上限({}) に達したため拒否されました。",
            command_line, agent_id, queue_max_pending
        ),
        QueueRejectionReason::ContextViolation {
            error,
            agent_id,
            command_line,
        } => format!(
            "run_command `{}` (agent {}) はコンテキスト制約で拒否されました：{}",
            command_line, agent_id, error
        ),
    }
}

fn describe_state(state: TerminalCommandState) -> &'static str {
    match state {
        TerminalCommandState::Queued => "キューに入りました",
        TerminalCommandState::Running => "実行を開始しました",
        TerminalCommandState::Completed => "完了しました",
        TerminalCommandState::Failed => "失敗しました",
    }
}

fn describe_interruption(reason: &QueueInterruptionReason) -> &'static str {
    match reason {
        QueueInterruptionReason::Failed => "失敗",
        QueueInterruptionReason::Cancelled => "キャンセル",
    }
}

/// ターミナルセッションを UI から操作するためのコントローラ。
#[derive(Debug)]
pub struct TerminalUiController {
    session: TerminalSession,
    latest_audit_messages: Vec<TerminalAuditMessage>,
}

impl TerminalUiController {
    /// デフォルトのキューサイズで新しいコントローラを作る。
    pub fn new(
        workspace_session_id: impl Into<String>,
        workspace_root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            session: TerminalSession::new_with_workspace_context(
                workspace_session_id,
                workspace_root,
                BTreeMap::new(),
            ),
            latest_audit_messages: Vec::new(),
        }
    }

    /// キューの最大長を指定したバリアント。
    pub fn with_queue_max_pending(
        workspace_session_id: impl Into<String>,
        workspace_root: impl Into<PathBuf>,
        queue_max_pending: usize,
    ) -> Self {
        Self {
            session: TerminalSession::new_with_tool_execution_config(
                workspace_session_id,
                ToolExecutionConfig::new(queue_max_pending),
                workspace_root,
                BTreeMap::new(),
            ),
            latest_audit_messages: Vec::new(),
        }
    }

    /// ワークスペースセッションの識別子。
    pub fn workspace_session_id(&self) -> &str {
        self.session.workspace_session_id()
    }

    /// 入力された run_command をキューに積む。
    pub fn run_command(
        &mut self,
        agent_id: impl Into<String>,
        command_line: impl Into<String>,
    ) -> QueueCommandOutcome {
        let request = RunCommandRequest::new(agent_id, command_line);
        self.session.enqueue_run_command(request)
    }

    /// 実行中のコマンドを完了させ、次のコマンドを昇格させる。
    pub fn complete_running_command(&mut self, success: bool) -> Option<TerminalCommandId> {
        self.session.complete_running_command(success)
    }

    /// 現在実行中のコマンドのスナップショット。
    pub fn running_command(&self) -> Option<TerminalCommandSnapshot> {
        self.session.running_command().cloned()
    }

    /// キューに積まれているコマンド数。
    pub fn queue_len(&self) -> usize {
        self.session.queue_len()
    }

    /// キュー上限。
    pub fn queue_max_pending(&self) -> usize {
        self.session.queue_max_pending()
    }

    /// 直近に生成されたステータス/通知イベントを取り出す。
    pub fn drain_events(&mut self) -> Vec<TerminalSessionEvent> {
        let events = self.session.drain_events();
        self.latest_audit_messages = events
            .iter()
            .filter_map(|event| match event {
                TerminalSessionEvent::Audit(audit) => Some(TerminalAuditMessage::from_event(audit)),
                _ => None,
            })
            .collect();
        events
    }

    /// 直近の監査イベントを UI 表示用に取得する。
    pub fn drain_audit_messages(&mut self) -> Vec<TerminalAuditMessage> {
        mem::take(&mut self.latest_audit_messages)
    }

    /// 実行出力をスクロールバックに追加する。
    pub fn push_output_line(&mut self, raw: impl Into<String>) {
        self.session.push_output_line(raw);
    }

    /// スクロールバックの全行を取得する。
    pub fn scrollback_lines(&self) -> Vec<ScrollbackLine> {
        self.session.scrollback_lines()
    }

    /// スクロールバックの最後の行。
    pub fn last_scrollback_line(&self) -> Option<ScrollbackLine> {
        self.session.last_scrollback_line().cloned()
    }

    /// スクロールバックの行数。
    pub fn scrollback_len(&self) -> usize {
        self.session.scrollback_len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nue_core::terminal::session::{
        TerminalCommandState, TerminalNotificationEvent, TerminalStatusEvent,
    };

    #[test]
    fn run_commandの状態がイベントに含まれる() {
        let mut controller =
            TerminalUiController::new("workspace-session-1", "/workspace-session-1");

        let first = controller.run_command("agent-a", "cargo test");
        let second = controller.run_command("agent-b", "cargo clippy");

        assert_eq!(first, QueueCommandOutcome::Started { command_id: 1 });
        assert_eq!(
            second,
            QueueCommandOutcome::Queued {
                command_id: 2,
                position: 1
            }
        );
        assert_eq!(controller.queue_len(), 1);
        assert_eq!(controller.running_command().unwrap().id, 1);

        assert_eq!(
            controller.drain_events(),
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
    fn キュー上限超過で通知イベントを生成する() {
        let mut controller = TerminalUiController::with_queue_max_pending(
            "workspace-session-1",
            "/workspace-session-1",
            2,
        );

        controller.run_command("agent-a", "cmd-1");
        controller.run_command("agent-b", "cmd-2");
        controller.run_command("agent-c", "cmd-3");
        controller.drain_events();

        let result = controller.run_command("agent-d", "cmd-4");

        assert_eq!(result, QueueCommandOutcome::RejectedQueueFull);
        let expected_message = format!(
            "run_command キューの上限({})に達したため実行を拒否しました。先行する run_command の完了を待つか中断してください。",
            controller.queue_max_pending(),
        );
        assert_eq!(
            controller.drain_events(),
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
                            queue_max_pending: controller.queue_max_pending(),
                            agent_id: "agent-d".to_string(),
                            command_line: "cmd-4".to_string(),
                        },
                    },
                }),
            ]
        );
    }

    #[test]
    fn 連続完了でキューから昇格する() {
        let mut controller =
            TerminalUiController::new("workspace-session-2", "/workspace-session-2");
        controller.run_command("agent-a", "cmd-1");
        controller.run_command("agent-b", "cmd-2");
        controller.drain_events();

        let completed = controller.complete_running_command(true);

        assert_eq!(completed, Some(1));
        assert_eq!(controller.running_command().unwrap().id, 2);
        assert_eq!(controller.queue_len(), 0);
    }

    #[test]
    fn complete_running_command_generates_notifications() {
        let mut controller =
            TerminalUiController::new("workspace-session-3", "/workspace-session-3");
        controller.run_command("agent-a", "build");
        controller.run_command("agent-b", "fmt");
        controller.drain_events();

        controller.complete_running_command(true);
        assert_eq!(
            controller.drain_events(),
            vec![
                TerminalSessionEvent::StatusChanged(TerminalStatusEvent {
                    workspace_session_id: "workspace-session-3".to_string(),
                    command_id: 1,
                    state: TerminalCommandState::Completed,
                    queue_length: 1,
                }),
                TerminalSessionEvent::Audit(TerminalAuditEvent {
                    id: 3,
                    workspace_session_id: "workspace-session-3".to_string(),
                    payload: TerminalAuditPayload::QueueTransition {
                        command_id: 1,
                        agent_id: "agent-a".to_string(),
                        command_line: "build".to_string(),
                        state: TerminalCommandState::Completed,
                        queue_length: 1,
                    },
                }),
                TerminalSessionEvent::Notification(TerminalNotificationEvent {
                    workspace_session_id: "workspace-session-3".to_string(),
                    message: "run_command `build` (agent agent-a) が完了しました".to_string(),
                }),
                TerminalSessionEvent::StatusChanged(TerminalStatusEvent {
                    workspace_session_id: "workspace-session-3".to_string(),
                    command_id: 2,
                    state: TerminalCommandState::Running,
                    queue_length: 0,
                }),
                TerminalSessionEvent::Audit(TerminalAuditEvent {
                    id: 4,
                    workspace_session_id: "workspace-session-3".to_string(),
                    payload: TerminalAuditPayload::QueueTransition {
                        command_id: 2,
                        agent_id: "agent-b".to_string(),
                        command_line: "fmt".to_string(),
                        state: TerminalCommandState::Running,
                        queue_length: 0,
                    },
                }),
            ]
        );

        controller.complete_running_command(false);
        assert_eq!(
            controller.drain_events(),
            vec![
                TerminalSessionEvent::StatusChanged(TerminalStatusEvent {
                    workspace_session_id: "workspace-session-3".to_string(),
                    command_id: 2,
                    state: TerminalCommandState::Failed,
                    queue_length: 0,
                }),
                TerminalSessionEvent::Audit(TerminalAuditEvent {
                    id: 5,
                    workspace_session_id: "workspace-session-3".to_string(),
                    payload: TerminalAuditPayload::QueueTransition {
                        command_id: 2,
                        agent_id: "agent-b".to_string(),
                        command_line: "fmt".to_string(),
                        state: TerminalCommandState::Failed,
                        queue_length: 0,
                    },
                }),
                TerminalSessionEvent::Audit(TerminalAuditEvent {
                    id: 6,
                    workspace_session_id: "workspace-session-3".to_string(),
                    payload: TerminalAuditPayload::QueueInterrupted {
                        command_id: 2,
                        agent_id: "agent-b".to_string(),
                        command_line: "fmt".to_string(),
                        reason: QueueInterruptionReason::Failed,
                    },
                }),
                TerminalSessionEvent::Notification(TerminalNotificationEvent {
                    workspace_session_id: "workspace-session-3".to_string(),
                    message: "run_command `fmt` (agent agent-b) が失敗しました".to_string(),
                }),
            ]
        );
    }

    #[test]
    fn scrollback_lines_are_exposed() {
        let mut controller =
            TerminalUiController::new("workspace-session-1", "/workspace-session-1");

        controller.push_output_line("first");
        controller.push_output_line("次の行");

        assert_eq!(controller.scrollback_len(), 2);
        let lines = controller.scrollback_lines();
        assert_eq!(lines[0].raw(), "first");
        assert_eq!(lines[1].raw(), "次の行");
        assert_eq!(lines[1].width(), 6);

        assert_eq!(
            controller
                .last_scrollback_line()
                .map(|line| line.raw().to_string()),
            Some("次の行".to_string())
        );
    }

    #[test]
    fn drain_audit_messages_formats_transitions() {
        let mut controller =
            TerminalUiController::new("workspace-session-audit", "/workspace-session-audit");

        controller.run_command("agent-a", "cargo test");
        controller.run_command("agent-b", "cargo clippy");

        controller.drain_events();
        let audit_messages = controller.drain_audit_messages();

        assert_eq!(audit_messages.len(), 2);
        assert_eq!(
            audit_messages[0].summary,
            "run_command `cargo test` (agent agent-a) が実行を開始しました（キュー長：0）"
        );
        assert_eq!(
            audit_messages[1].summary,
            "run_command `cargo clippy` (agent agent-b) がキューに入りました（キュー長：1）"
        );
        assert!(controller.drain_audit_messages().is_empty());
    }

    #[test]
    fn drain_audit_messages_formats_queue_rejections() {
        let mut controller = TerminalUiController::with_queue_max_pending(
            "workspace-session-reject",
            "/workspace-session-reject",
            2,
        );

        controller.run_command("agent-a", "cmd-1");
        controller.run_command("agent-b", "cmd-2");
        controller.run_command("agent-c", "cmd-3");

        controller.drain_events();
        controller.drain_audit_messages();

        let _ = controller.run_command("agent-d", "cmd-4");
        controller.drain_events();
        let audit_messages = controller.drain_audit_messages();

        assert_eq!(audit_messages.len(), 1);
        assert_eq!(
            audit_messages[0].summary,
            "run_command `cmd-4` (agent agent-d) はキュー上限(2) に達したため拒否されました。"
        );

        if let TerminalAuditPayload::QueueRejected { reason } = &audit_messages[0].payload {
            match reason {
                QueueRejectionReason::QueueFull {
                    queue_max_pending,
                    agent_id,
                    command_line,
                } => {
                    assert_eq!(*queue_max_pending, 2);
                    assert_eq!(agent_id, "agent-d");
                    assert_eq!(command_line, "cmd-4");
                }
                _ => panic!("unexpected rejection reason"),
            }
        } else {
            panic!("expected QueueRejected payload");
        }
    }

    #[test]
    fn drain_audit_messages_formats_interruptions() {
        let mut controller = TerminalUiController::new(
            "workspace-session-interrupt",
            "/workspace-session-interrupt",
        );

        controller.run_command("agent-z", "cmd-1");
        controller.drain_events();
        controller.drain_audit_messages();

        controller.complete_running_command(false);
        controller.drain_events();
        let audit_messages = controller.drain_audit_messages();

        assert_eq!(audit_messages.len(), 2);
        assert_eq!(
            audit_messages[0].summary,
            "run_command `cmd-1` (agent agent-z) が失敗しました（キュー長：0）"
        );
        assert_eq!(
            audit_messages[1].summary,
            "run_command `cmd-1` (agent agent-z) は失敗により中断されました。"
        );
    }

    #[test]
    fn drain_eventsを繰り返しても監査メッセージが累積しない() {
        let mut controller =
            TerminalUiController::new("workspace-session-non-leak", "/workspace-session-non-leak");

        controller.run_command("agent-a", "cmd-1");
        controller.drain_events();

        controller.complete_running_command(true);
        controller.drain_events();
        let audit_messages = controller.drain_audit_messages();

        assert_eq!(audit_messages.len(), 1);
        assert_eq!(
            audit_messages[0].summary,
            "run_command `cmd-1` (agent agent-a) が完了しました（キュー長：0）"
        );
    }
}
