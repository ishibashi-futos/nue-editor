use super::{
    QueueCommandOutcome, QueueInterruptionReason, QueueRejectionReason, RunCommandRequest,
    TerminalCommandSnapshot, TerminalCommandState, TerminalSession, TerminalStatusEvent,
    validate_run_command_context,
};

impl TerminalSession {
    pub fn enqueue_run_command(&mut self, request: RunCommandRequest) -> QueueCommandOutcome {
        if let Err(error) =
            validate_run_command_context(&self.workspace_root, &self.workspace_env, &request)
        {
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

    pub fn complete_running_command(&mut self, success: bool) -> Option<super::TerminalCommandId> {
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

    fn allocate_command_id(&mut self) -> super::TerminalCommandId {
        let command_id = self.next_command_id;
        self.next_command_id += 1;
        command_id
    }

    fn update_command_state(
        &mut self,
        command_id: super::TerminalCommandId,
        state: TerminalCommandState,
    ) {
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

    fn push_status_event(
        &mut self,
        command_id: super::TerminalCommandId,
        state: TerminalCommandState,
    ) {
        self.events
            .push_back(super::TerminalSessionEvent::StatusChanged(
                TerminalStatusEvent {
                    workspace_session_id: self.workspace_session_id.clone(),
                    command_id,
                    state,
                    queue_length: self.queued_command_ids.len(),
                },
            ));
        self.push_queue_transition_audit_event(command_id, state);
    }
}
