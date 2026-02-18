use nue_core::terminal_session::{
    QueueCommandOutcome, TerminalCommandId, TerminalCommandSnapshot, TerminalSession,
    TerminalSessionEvent,
};

/// ターミナルセッションを UI から操作するためのコントローラ。
#[derive(Debug)]
pub struct TerminalUiController {
    session: TerminalSession,
}

impl TerminalUiController {
    /// デフォルトのキューサイズで新しいコントローラを作る。
    pub fn new(workspace_session_id: impl Into<String>) -> Self {
        Self {
            session: TerminalSession::new(workspace_session_id),
        }
    }

    /// キューの最大長を指定したバリアント。
    pub fn with_queue_max_pending(
        workspace_session_id: impl Into<String>,
        queue_max_pending: usize,
    ) -> Self {
        Self {
            session: TerminalSession::new_with_queue_max_pending(
                workspace_session_id,
                queue_max_pending,
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
        self.session.enqueue_run_command(agent_id, command_line)
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

    /// 直近に生成されたステータス/通知イベントを取り出す。
    pub fn drain_events(&mut self) -> Vec<TerminalSessionEvent> {
        self.session.drain_events()
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
        let mut controller = TerminalUiController::new("workspace-session-1");

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
    fn キュー上限超過で通知イベントを生成する() {
        let mut controller = TerminalUiController::with_queue_max_pending("workspace-session-1", 2);

        controller.run_command("agent-a", "cmd-1");
        controller.run_command("agent-b", "cmd-2");
        controller.run_command("agent-c", "cmd-3");
        controller.drain_events();

        let result = controller.run_command("agent-d", "cmd-4");

        assert_eq!(result, QueueCommandOutcome::RejectedQueueFull);
        assert_eq!(
            controller.drain_events(),
            vec![TerminalSessionEvent::Notification(
                TerminalNotificationEvent {
                    workspace_session_id: "workspace-session-1".to_string(),
                    message: "run_command キューが上限に達したため要求を拒否しました".to_string(),
                }
            )]
        );
    }

    #[test]
    fn 連続完了でキューから昇格する() {
        let mut controller = TerminalUiController::new("workspace-session-2");
        controller.run_command("agent-a", "cmd-1");
        controller.run_command("agent-b", "cmd-2");
        controller.drain_events();

        let completed = controller.complete_running_command(true);

        assert_eq!(completed, Some(1));
        assert_eq!(controller.running_command().unwrap().id, 2);
        assert_eq!(controller.queue_len(), 0);
    }
}
