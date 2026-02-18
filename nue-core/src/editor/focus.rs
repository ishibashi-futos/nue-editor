use std::collections::{HashMap, VecDeque};

pub const DEFAULT_SYNC_BUDGET_MS: u64 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusArea {
    Pane,
    Tab,
    CommandHub,
    Terminal,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    next_focus_nonce: u64,
    focus_ids_by_target: HashMap<FocusTarget, String>,
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
            next_focus_nonce: 1,
            focus_ids_by_target: HashMap::new(),
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

        let focus_id = self.allocate_focus_id(&target, requested_at_ms);
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

    fn allocate_focus_id(&mut self, target: &FocusTarget, requested_at_ms: u64) -> String {
        if let Some(existing) = self.focus_ids_by_target.get(target) {
            return existing.clone();
        }

        let file_path_hash = fnv1a64(target.target_id.as_bytes());
        let hunk_checksum =
            fnv1a32(format!("{}:{}", focus_area_name(target.area), target.target_id).as_bytes());
        let (hunk_start_line, hunk_end_line) = extract_line_span(&target.target_id);
        let timestamp = requested_at_ms;
        let nonce = self.next_focus_nonce;
        self.next_focus_nonce += 1;

        let randomness_seed = [
            file_path_hash.to_be_bytes().as_slice(),
            &hunk_start_line.to_be_bytes(),
            &hunk_end_line.to_be_bytes(),
            &hunk_checksum.to_be_bytes(),
            &timestamp.to_be_bytes(),
            &nonce.to_be_bytes(),
        ]
        .concat();
        let randomness_hash = fnv1a128(&randomness_seed);
        let randomness = randomness_hash.to_be_bytes()[6..16]
            .try_into()
            .expect("10バイト長を保証");
        let focus_id = encode_ulid(timestamp, randomness);
        self.focus_ids_by_target
            .insert(target.clone(), focus_id.clone());

        focus_id
    }
}

const ULID_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const ULID_RANDOM_MASK: u128 = (1u128 << 80) - 1;

fn focus_area_name(area: FocusArea) -> &'static str {
    match area {
        FocusArea::Pane => "pane",
        FocusArea::Tab => "tab",
        FocusArea::CommandHub => "command_hub",
        FocusArea::Terminal => "terminal",
    }
}

fn extract_line_span(target_id: &str) -> (u32, u32) {
    let Some((_, line_span)) = target_id.rsplit_once(':') else {
        return (0, 0);
    };
    let Some((start, end)) = line_span.split_once('-') else {
        return (0, 0);
    };
    let Ok(start) = start.parse::<u32>() else {
        return (0, 0);
    };
    let Ok(end) = end.parse::<u32>() else {
        return (0, 0);
    };

    (start, end)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn fnv1a32(bytes: &[u8]) -> u32 {
    let mut hash = 0x811c9dc5u32;
    for byte in bytes {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}

fn fnv1a128(bytes: &[u8]) -> u128 {
    let mut hash = 0x6c62272e07bb014262b821756295c58du128;
    for byte in bytes {
        hash ^= u128::from(*byte);
        hash = hash.wrapping_mul(0x0000000001000000000000000000013bu128);
    }
    hash
}

fn encode_ulid(timestamp_ms: u64, randomness: [u8; 10]) -> String {
    let mut value = (u128::from(timestamp_ms & ((1u64 << 48) - 1)) << 80)
        | (u128::from_be_bytes([
            0,
            0,
            0,
            0,
            0,
            0,
            randomness[0],
            randomness[1],
            randomness[2],
            randomness[3],
            randomness[4],
            randomness[5],
            randomness[6],
            randomness[7],
            randomness[8],
            randomness[9],
        ]) & ULID_RANDOM_MASK);

    let mut chars = [b'0'; 26];
    for index in (0..26).rev() {
        let digit = (value & 0b1_1111) as usize;
        chars[index] = ULID_ALPHABET[digit];
        value >>= 5;
    }

    String::from_utf8(chars.to_vec()).expect("ULIDはASCIIのみ")
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

        let EnqueueFocusOutcome::Queued {
            focus_id,
            queue_position,
        } = layer.enqueue_focus(FocusTarget::pane("pane-left"), 100)
        else {
            panic!("キュー登録される想定");
        };
        assert_eq!(queue_position, 1);

        let task = layer.start_next_sync().expect("同期タスクが必要");
        assert_eq!(
            task,
            FocusSyncTask {
                focus_id: focus_id.clone(),
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
                    focus_id: focus_id.clone(),
                    target: FocusTarget::pane("pane-left"),
                    queue_length: 1,
                    deadline_at_ms: 150,
                }),
                FocusLayerEvent::SyncStarted(FocusSyncStartedEvent {
                    workspace_session_id: "workspace-session-1".to_string(),
                    focus_id: focus_id.clone(),
                    target: FocusTarget::pane("pane-left"),
                }),
            ]
        );
    }

    #[test]
    fn 同期が50ms以内で完了するとactive_focusへ反映される() {
        let mut layer = test_layer();

        let EnqueueFocusOutcome::Queued { focus_id, .. } =
            layer.enqueue_focus(FocusTarget::tab("tab-main"), 10)
        else {
            panic!("キュー登録される想定");
        };
        layer.start_next_sync();
        layer.drain_events();

        let outcome = layer.finish_inflight_sync(55).expect("同期完了する");

        assert_eq!(
            outcome,
            FocusSyncOutcome {
                focus_id: focus_id.clone(),
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
                    focus_id: focus_id.clone(),
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
                focus_id: focus_id.clone(),
                target: FocusTarget::tab("tab-main"),
                reflected_at_ms: 55,
                latency_ms: 45,
                within_budget: true,
            })]
        );
    }

    #[test]
    fn 同期遅延が50msちょうどなら予算内として扱う() {
        let mut layer = test_layer();

        let EnqueueFocusOutcome::Queued { focus_id, .. } =
            layer.enqueue_focus(FocusTarget::tab("tab-main"), 10)
        else {
            panic!("キュー登録される想定");
        };
        layer.start_next_sync();
        layer.drain_events();

        let outcome = layer.finish_inflight_sync(60).expect("同期完了する");

        assert_eq!(
            outcome,
            FocusSyncOutcome {
                focus_id: focus_id.clone(),
                target: FocusTarget::tab("tab-main"),
                latency_ms: 50,
                within_budget: true,
            }
        );
    }

    #[test]
    fn 同期が50msを超過して完了した場合は超過フラグを保持する() {
        let mut layer = test_layer();

        let EnqueueFocusOutcome::Queued { focus_id, .. } =
            layer.enqueue_focus(FocusTarget::terminal("terminal-main"), 200)
        else {
            panic!("キュー登録される想定");
        };
        layer.start_next_sync();
        layer.drain_events();

        let outcome = layer.finish_inflight_sync(260).expect("同期完了する");

        assert_eq!(
            outcome,
            FocusSyncOutcome {
                focus_id: focus_id.clone(),
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
                    focus_id: focus_id.clone(),
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

    #[test]
    fn focus_idはulid形式で採番する() {
        let mut layer = test_layer();

        let EnqueueFocusOutcome::Queued { focus_id, .. } =
            layer.enqueue_focus(FocusTarget::pane("pane-left"), 100)
        else {
            panic!("キュー登録される想定");
        };

        assert_eq!(focus_id.len(), 26);
        assert!(
            focus_id
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'A'..=b'Z'))
        );
    }

    #[test]
    fn 同一ターゲットのフォーカス要求はfocus_idを再利用する() {
        let mut layer = test_layer();

        let first = layer.enqueue_focus(FocusTarget::pane("pane-left"), 100);
        let second = layer.enqueue_focus(FocusTarget::pane("pane-left"), 110);

        let EnqueueFocusOutcome::Queued {
            focus_id: first_focus_id,
            ..
        } = first
        else {
            panic!("キュー登録される想定");
        };
        let EnqueueFocusOutcome::Queued {
            focus_id: second_focus_id,
            ..
        } = second
        else {
            panic!("キュー登録される想定");
        };

        assert_eq!(first_focus_id, second_focus_id);
    }
}
