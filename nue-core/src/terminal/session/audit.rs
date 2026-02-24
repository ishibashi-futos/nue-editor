use super::{
    QueueInterruptionReason, QueueRejectionReason, TerminalAuditEvent, TerminalAuditPayload,
    TerminalCommandId, TerminalCommandState, TerminalSession, TerminalSessionEvent,
};

impl TerminalSession {
    fn allocate_audit_id(&mut self) -> super::TerminalAuditId {
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

    pub(super) fn push_queue_transition_audit_event(
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

    pub(super) fn push_queue_rejection_audit_event(&mut self, reason: QueueRejectionReason) {
        self.push_audit_event(TerminalAuditPayload::QueueRejected { reason });
    }

    pub(super) fn push_queue_interruption_audit_event(
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
}
