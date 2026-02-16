use std::collections::VecDeque;

pub const DEFAULT_SYNC_BUDGET_MS: u64 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusArea {
    Pane,
    Tab,
    CommandHub,
    Terminal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusTarget {
    pub area: FocusArea,
    pub target_id: String,
}

impl FocusTarget {
    pub fn pane(target_id: impl Into<String>) -> Self {
        Self {
            area: FocusArea::Pane,
            target_id: target_id.into(),
        }
    }

    pub fn tab(target_id: impl Into<String>) -> Self {
        Self {
            area: FocusArea::Tab,
            target_id: target_id.into(),
        }
    }

    pub fn command_hub(target_id: impl Into<String>) -> Self {
        Self {
            area: FocusArea::CommandHub,
            target_id: target_id.into(),
        }
    }

    pub fn terminal(target_id: impl Into<String>) -> Self {
        Self {
            area: FocusArea::Terminal,
            target_id: target_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusSyncTask {
    pub focus_id: String,
    pub target: FocusTarget,
    pub requested_at_ms: u64,
    pub deadline_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusSnapshot {
    pub focus_id: String,
    pub target: FocusTarget,
    pub reflected_at_ms: u64,
    pub within_budget: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusLayerSnapshot {
    pub revision: u64,
    pub active_focus: Option<FocusSnapshot>,
    pub pending_queue_len: usize,
    pub has_inflight_task: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnqueueFocusOutcome {
    Noop,
    Queued {
        focus_id: String,
        queue_position: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusSyncOutcome {
    pub focus_id: String,
    pub target: FocusTarget,
    pub latency_ms: u64,
    pub within_budget: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusQueuedEvent {
    pub workspace_session_id: String,
    pub focus_id: String,
    pub target: FocusTarget,
    pub queue_length: usize,
    pub deadline_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusSyncStartedEvent {
    pub workspace_session_id: String,
    pub focus_id: String,
    pub target: FocusTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusSyncedEvent {
    pub workspace_session_id: String,
    pub focus_id: String,
    pub target: FocusTarget,
    pub reflected_at_ms: u64,
    pub latency_ms: u64,
    pub within_budget: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusLayerEvent {
    Queued(FocusQueuedEvent),
    SyncStarted(FocusSyncStartedEvent),
    Synced(FocusSyncedEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusLayer {
    workspace_session_id: String,
    sync_budget_ms: u64,
    revision: u64,
    next_focus_sequence: u64,
    active_focus: Option<FocusSnapshot>,
    pending_tasks: VecDeque<FocusSyncTask>,
    inflight_task: Option<FocusSyncTask>,
    events: VecDeque<FocusLayerEvent>,
}

impl FocusLayer {
    pub fn new(workspace_session_id: impl Into<String>) -> Self {
        Self::new_with_budget(workspace_session_id, DEFAULT_SYNC_BUDGET_MS)
    }

    pub fn new_with_budget(workspace_session_id: impl Into<String>, sync_budget_ms: u64) -> Self {
        Self {
            workspace_session_id: workspace_session_id.into(),
            sync_budget_ms,
            revision: 0,
            next_focus_sequence: 1,
            active_focus: None,
            pending_tasks: VecDeque::new(),
            inflight_task: None,
            events: VecDeque::new(),
        }
    }

    pub fn snapshot(&self) -> FocusLayerSnapshot {
        FocusLayerSnapshot {
            revision: self.revision,
            active_focus: self.active_focus.clone(),
            pending_queue_len: self.pending_tasks.len(),
            has_inflight_task: self.inflight_task.is_some(),
        }
    }

    pub fn enqueue_focus(
        &mut self,
        target: FocusTarget,
        requested_at_ms: u64,
    ) -> EnqueueFocusOutcome {
        if self.inflight_task.is_none()
            && self.pending_tasks.is_empty()
            && self
                .active_focus
                .as_ref()
                .is_some_and(|active| active.target == target)
        {
            return EnqueueFocusOutcome::Noop;
        }

        let focus_id = self.allocate_focus_id();
        let deadline_at_ms = requested_at_ms.saturating_add(self.sync_budget_ms);
        self.pending_tasks.push_back(FocusSyncTask {
            focus_id: focus_id.clone(),
            target: target.clone(),
            requested_at_ms,
            deadline_at_ms,
        });

        let queue_position = self.pending_tasks.len();
        self.events
            .push_back(FocusLayerEvent::Queued(FocusQueuedEvent {
                workspace_session_id: self.workspace_session_id.clone(),
                focus_id: focus_id.clone(),
                target,
                queue_length: queue_position,
                deadline_at_ms,
            }));

        EnqueueFocusOutcome::Queued {
            focus_id,
            queue_position,
        }
    }

    pub fn start_next_sync(&mut self) -> Option<FocusSyncTask> {
        if self.inflight_task.is_some() {
            return None;
        }

        let task = self.pending_tasks.pop_front()?;
        self.events
            .push_back(FocusLayerEvent::SyncStarted(FocusSyncStartedEvent {
                workspace_session_id: self.workspace_session_id.clone(),
                focus_id: task.focus_id.clone(),
                target: task.target.clone(),
            }));
        self.inflight_task = Some(task.clone());
        Some(task)
    }

    pub fn finish_inflight_sync(&mut self, reflected_at_ms: u64) -> Option<FocusSyncOutcome> {
        let task = self.inflight_task.take()?;
        let latency_ms = reflected_at_ms.saturating_sub(task.requested_at_ms);
        let within_budget = latency_ms <= self.sync_budget_ms;
        self.revision += 1;
        self.active_focus = Some(FocusSnapshot {
            focus_id: task.focus_id.clone(),
            target: task.target.clone(),
            reflected_at_ms,
            within_budget,
        });

        self.events
            .push_back(FocusLayerEvent::Synced(FocusSyncedEvent {
                workspace_session_id: self.workspace_session_id.clone(),
                focus_id: task.focus_id.clone(),
                target: task.target.clone(),
                reflected_at_ms,
                latency_ms,
                within_budget,
            }));

        Some(FocusSyncOutcome {
            focus_id: task.focus_id,
            target: task.target,
            latency_ms,
            within_budget,
        })
    }

    pub fn drain_events(&mut self) -> Vec<FocusLayerEvent> {
        self.events.drain(..).collect()
    }

    fn allocate_focus_id(&mut self) -> String {
        let focus_id = format!("focus-{}", self.next_focus_sequence);
        self.next_focus_sequence += 1;
        focus_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_layer() -> FocusLayer {
        FocusLayer::new("workspace-session-1")
    }

    #[test]
    fn フォーカス要求はイベントキューに積まれ同期対象として取り出せる() {
        let mut layer = test_layer();

        let outcome = layer.enqueue_focus(FocusTarget::pane("pane-left"), 100);

        assert_eq!(
            outcome,
            EnqueueFocusOutcome::Queued {
                focus_id: "focus-1".to_string(),
                queue_position: 1,
            }
        );

        let task = layer.start_next_sync().expect("同期タスクが必要");
        assert_eq!(
            task,
            FocusSyncTask {
                focus_id: "focus-1".to_string(),
                target: FocusTarget::pane("pane-left"),
                requested_at_ms: 100,
                deadline_at_ms: 150,
            }
        );

        assert_eq!(
            layer.snapshot(),
            FocusLayerSnapshot {
                revision: 0,
                active_focus: None,
                pending_queue_len: 0,
                has_inflight_task: true,
            }
        );

        assert_eq!(
            layer.drain_events(),
            vec![
                FocusLayerEvent::Queued(FocusQueuedEvent {
                    workspace_session_id: "workspace-session-1".to_string(),
                    focus_id: "focus-1".to_string(),
                    target: FocusTarget::pane("pane-left"),
                    queue_length: 1,
                    deadline_at_ms: 150,
                }),
                FocusLayerEvent::SyncStarted(FocusSyncStartedEvent {
                    workspace_session_id: "workspace-session-1".to_string(),
                    focus_id: "focus-1".to_string(),
                    target: FocusTarget::pane("pane-left"),
                }),
            ]
        );
    }

    #[test]
    fn 同期が50ms以内で完了するとactive_focusへ反映される() {
        let mut layer = test_layer();

        layer.enqueue_focus(FocusTarget::tab("tab-main"), 10);
        layer.start_next_sync();
        layer.drain_events();

        let outcome = layer.finish_inflight_sync(55).expect("同期完了する");

        assert_eq!(
            outcome,
            FocusSyncOutcome {
                focus_id: "focus-1".to_string(),
                target: FocusTarget::tab("tab-main"),
                latency_ms: 45,
                within_budget: true,
            }
        );

        assert_eq!(
            layer.snapshot(),
            FocusLayerSnapshot {
                revision: 1,
                active_focus: Some(FocusSnapshot {
                    focus_id: "focus-1".to_string(),
                    target: FocusTarget::tab("tab-main"),
                    reflected_at_ms: 55,
                    within_budget: true,
                }),
                pending_queue_len: 0,
                has_inflight_task: false,
            }
        );

        assert_eq!(
            layer.drain_events(),
            vec![FocusLayerEvent::Synced(FocusSyncedEvent {
                workspace_session_id: "workspace-session-1".to_string(),
                focus_id: "focus-1".to_string(),
                target: FocusTarget::tab("tab-main"),
                reflected_at_ms: 55,
                latency_ms: 45,
                within_budget: true,
            })]
        );
    }

    #[test]
    fn 同期が50msを超過して完了した場合は超過フラグを保持する() {
        let mut layer = test_layer();

        layer.enqueue_focus(FocusTarget::terminal("terminal-main"), 200);
        layer.start_next_sync();
        layer.drain_events();

        let outcome = layer.finish_inflight_sync(260).expect("同期完了する");

        assert_eq!(
            outcome,
            FocusSyncOutcome {
                focus_id: "focus-1".to_string(),
                target: FocusTarget::terminal("terminal-main"),
                latency_ms: 60,
                within_budget: false,
            }
        );

        assert_eq!(
            layer.snapshot(),
            FocusLayerSnapshot {
                revision: 1,
                active_focus: Some(FocusSnapshot {
                    focus_id: "focus-1".to_string(),
                    target: FocusTarget::terminal("terminal-main"),
                    reflected_at_ms: 260,
                    within_budget: false,
                }),
                pending_queue_len: 0,
                has_inflight_task: false,
            }
        );
    }

    #[test]
    fn 同一ターゲットが反映済みなら追加要求はnoopにする() {
        let mut layer = test_layer();

        layer.enqueue_focus(FocusTarget::command_hub("palette"), 0);
        layer.start_next_sync();
        layer.finish_inflight_sync(20);
        layer.drain_events();

        let outcome = layer.enqueue_focus(FocusTarget::command_hub("palette"), 21);

        assert_eq!(outcome, EnqueueFocusOutcome::Noop);
        assert!(layer.drain_events().is_empty());
    }
}
