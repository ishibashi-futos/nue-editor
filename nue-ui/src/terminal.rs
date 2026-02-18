use std::collections::BTreeMap;
use std::path::PathBuf;

use nue_core::terminal_scrollback::ScrollbackLine;
use nue_core::terminal_session::{
    QueueCommandOutcome, RunCommandRequest, TerminalCommandId, TerminalCommandSnapshot,
    TerminalSession, TerminalSessionEvent, ToolExecutionConfig,
};

#[cfg(test)]
use nue_core::terminal_session::{
    QueueInterruptionReason, QueueRejectionReason, TerminalAuditEvent, TerminalAuditPayload,
};

/// ターミナルセッションを UI から操作するためのコントローラ。
#[derive(Debug)]
pub struct TerminalUiController {
    session: TerminalSession,
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
        self.session.drain_events()
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
    use nue_core::terminal_session::{
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
}
