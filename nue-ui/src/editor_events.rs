use std::path::PathBuf;

use nue_core::{
    editor_core::EditorCoreEvent,
    minimap_service::{FocusSyncTarget, MinimapFocusIdSyncedEvent, MinimapOverlaysUpdatedEvent},
    smart_gutter_service::{
        SmartGutterApprovalRequestOpenedEvent, SmartGutterFocusIdSyncedEvent,
        SmartGutterIndicatorsUpdatedEvent, SmartGutterJumpRequestedEvent, SmartGutterSyncTarget,
    },
};

/// Minimap 描画用に UI が保持する状態。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimapUiState {
    pub file_path: PathBuf,
    pub overlay_count: usize,
    pub focus_id: Option<String>,
    pub focus_targets: Vec<FocusSyncTarget>,
}

/// Smart Gutter が参照するジャンプリクエスト情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartGutterJumpRequest {
    pub focus_id: String,
    pub line: usize,
}

/// Smart Gutter 上で承認リクエストが開かれた履歴。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartGutterApprovalRequest {
    pub focus_id: String,
    pub approval_request_id: String,
}

/// Smart Gutter 描画用の状態スナップショット。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartGutterUiState {
    pub file_path: PathBuf,
    pub indicator_count: usize,
    pub focus_id: Option<String>,
    pub focus_targets: Vec<SmartGutterSyncTarget>,
    pub last_jump_request: Option<SmartGutterJumpRequest>,
    pub last_approval_request: Option<SmartGutterApprovalRequest>,
}

/// EditorCore が生成するイベントを UI 側で集約する購読者。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EditorEventSubscriber {
    minimap_state: Option<MinimapUiState>,
    smart_gutter_state: Option<SmartGutterUiState>,
}

impl EditorEventSubscriber {
    /// 新しい購読者を作成する。
    pub fn new() -> Self {
        Self::default()
    }

    /// 最新のイベントを反映する。
    pub fn apply_event(&mut self, event: &EditorCoreEvent) {
        match event {
            EditorCoreEvent::MinimapOverlaysUpdated(payload) => {
                self.apply_minimap_overlays_updated(payload)
            }
            EditorCoreEvent::MinimapFocusIdSynced(payload) => {
                self.apply_minimap_focus_synced(payload)
            }
            EditorCoreEvent::SmartGutterIndicatorsUpdated(payload) => {
                self.apply_smart_gutter_indicators_updated(payload)
            }
            EditorCoreEvent::SmartGutterFocusIdSynced(payload) => {
                self.apply_smart_gutter_focus_synced(payload)
            }
            EditorCoreEvent::SmartGutterJumpRequested(payload) => {
                self.apply_smart_gutter_jump_request(payload)
            }
            EditorCoreEvent::SmartGutterApprovalRequestOpened(payload) => {
                self.apply_smart_gutter_approval_request(payload)
            }
            _ => {}
        }
    }

    /// 最後に記録された Minimap 状態を返す。
    pub fn minimap_state(&self) -> Option<&MinimapUiState> {
        self.minimap_state.as_ref()
    }

    /// 最後に記録された Smart Gutter 状態を返す。
    pub fn smart_gutter_state(&self) -> Option<&SmartGutterUiState> {
        self.smart_gutter_state.as_ref()
    }

    fn apply_minimap_overlays_updated(&mut self, payload: &MinimapOverlaysUpdatedEvent) {
        let (focus_id, focus_targets) = self
            .minimap_state
            .as_ref()
            .filter(|state| state.file_path == payload.file_path)
            .map(|state| (state.focus_id.clone(), state.focus_targets.clone()))
            .unwrap_or((None, Vec::new()));

        self.minimap_state = Some(MinimapUiState {
            file_path: payload.file_path.clone(),
            overlay_count: payload.overlay_count,
            focus_id,
            focus_targets,
        });
    }

    fn apply_minimap_focus_synced(&mut self, payload: &MinimapFocusIdSyncedEvent) {
        let overlay_count = self
            .minimap_state
            .as_ref()
            .filter(|state| state.file_path == payload.file_path)
            .map(|state| state.overlay_count)
            .unwrap_or(0);

        self.minimap_state = Some(MinimapUiState {
            file_path: payload.file_path.clone(),
            overlay_count,
            focus_id: Some(payload.focus_id.clone()),
            focus_targets: payload.targets.clone(),
        });
    }

    fn apply_smart_gutter_indicators_updated(
        &mut self,
        payload: &SmartGutterIndicatorsUpdatedEvent,
    ) {
        let (focus_id, focus_targets, last_jump_request, last_approval_request) = self
            .smart_gutter_state
            .as_ref()
            .filter(|state| state.file_path == payload.file_path)
            .map(|state| {
                (
                    state.focus_id.clone(),
                    state.focus_targets.clone(),
                    state.last_jump_request.clone(),
                    state.last_approval_request.clone(),
                )
            })
            .unwrap_or((None, Vec::new(), None, None));

        self.smart_gutter_state = Some(SmartGutterUiState {
            file_path: payload.file_path.clone(),
            indicator_count: payload.indicator_count,
            focus_id,
            focus_targets,
            last_jump_request,
            last_approval_request,
        });
    }

    fn apply_smart_gutter_focus_synced(&mut self, payload: &SmartGutterFocusIdSyncedEvent) {
        let (indicator_count, last_jump_request, last_approval_request) = self
            .smart_gutter_state
            .as_ref()
            .filter(|state| state.file_path == payload.file_path)
            .map(|state| {
                (
                    state.indicator_count,
                    state.last_jump_request.clone(),
                    state.last_approval_request.clone(),
                )
            })
            .unwrap_or((0, None, None));

        self.smart_gutter_state = Some(SmartGutterUiState {
            file_path: payload.file_path.clone(),
            indicator_count,
            focus_id: Some(payload.focus_id.clone()),
            focus_targets: payload.targets.clone(),
            last_jump_request,
            last_approval_request,
        });
    }

    fn apply_smart_gutter_jump_request(&mut self, payload: &SmartGutterJumpRequestedEvent) {
        let (indicator_count, focus_id, focus_targets, last_approval_request) = self
            .smart_gutter_state
            .as_ref()
            .filter(|state| state.file_path == payload.file_path)
            .map(|state| {
                (
                    state.indicator_count,
                    state.focus_id.clone(),
                    state.focus_targets.clone(),
                    state.last_approval_request.clone(),
                )
            })
            .unwrap_or((0, None, Vec::new(), None));

        self.smart_gutter_state = Some(SmartGutterUiState {
            file_path: payload.file_path.clone(),
            indicator_count,
            focus_id,
            focus_targets,
            last_jump_request: Some(SmartGutterJumpRequest {
                focus_id: payload.focus_id.clone(),
                line: payload.line,
            }),
            last_approval_request,
        });
    }

    fn apply_smart_gutter_approval_request(
        &mut self,
        payload: &SmartGutterApprovalRequestOpenedEvent,
    ) {
        let (indicator_count, focus_id, focus_targets, last_jump_request) = self
            .smart_gutter_state
            .as_ref()
            .filter(|state| state.file_path == payload.file_path)
            .map(|state| {
                (
                    state.indicator_count,
                    state.focus_id.clone(),
                    state.focus_targets.clone(),
                    state.last_jump_request.clone(),
                )
            })
            .unwrap_or((0, None, Vec::new(), None));

        self.smart_gutter_state = Some(SmartGutterUiState {
            file_path: payload.file_path.clone(),
            indicator_count,
            focus_id,
            focus_targets,
            last_jump_request,
            last_approval_request: Some(SmartGutterApprovalRequest {
                focus_id: payload.focus_id.clone(),
                approval_request_id: payload.approval_request_id.clone(),
            }),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nue_core::minimap_service::FocusSyncTarget;
    use nue_core::smart_gutter_service::SmartGutterSyncTarget;

    fn sample_path(name: &str) -> PathBuf {
        PathBuf::from(format!("/project/{}.rs", name))
    }

    #[test]
    fn minimap_state_updates_overlay_and_focus() {
        let mut subscriber = EditorEventSubscriber::new();
        subscriber.apply_event(&EditorCoreEvent::MinimapOverlaysUpdated(
            MinimapOverlaysUpdatedEvent {
                file_path: sample_path("main"),
                overlay_count: 2,
            },
        ));

        let state = subscriber.minimap_state().unwrap();
        assert_eq!(state.overlay_count, 2);
        assert!(state.focus_id.is_none());

        subscriber.apply_event(&EditorCoreEvent::MinimapFocusIdSynced(
            MinimapFocusIdSyncedEvent {
                file_path: sample_path("main"),
                focus_id: "focus-main".into(),
                targets: vec![FocusSyncTarget::CommandHub],
            },
        ));

        let state = subscriber.minimap_state().unwrap();
        assert_eq!(state.focus_id.as_deref(), Some("focus-main"));
        assert_eq!(state.focus_targets, vec![FocusSyncTarget::CommandHub]);
    }

    #[test]
    fn smart_gutter_remembers_indicator_and_focus_and_requests() {
        let mut subscriber = EditorEventSubscriber::new();
        let path = sample_path("lib");

        subscriber.apply_event(&EditorCoreEvent::SmartGutterIndicatorsUpdated(
            SmartGutterIndicatorsUpdatedEvent {
                file_path: path.clone(),
                indicator_count: 3,
            },
        ));

        let state = subscriber.smart_gutter_state().unwrap();
        assert_eq!(state.indicator_count, 3);

        subscriber.apply_event(&EditorCoreEvent::SmartGutterFocusIdSynced(
            SmartGutterFocusIdSyncedEvent {
                file_path: path.clone(),
                focus_id: "focus-lib".into(),
                targets: vec![SmartGutterSyncTarget::CommandHub],
            },
        ));

        let state = subscriber.smart_gutter_state().unwrap();
        assert_eq!(state.focus_id.as_deref(), Some("focus-lib"));
        assert_eq!(state.focus_targets, vec![SmartGutterSyncTarget::CommandHub]);

        subscriber.apply_event(&EditorCoreEvent::SmartGutterJumpRequested(
            SmartGutterJumpRequestedEvent {
                file_path: path.clone(),
                focus_id: "focus-lib".into(),
                line: 42,
            },
        ));

        let state = subscriber.smart_gutter_state().unwrap();
        assert_eq!(
            state.last_jump_request,
            Some(SmartGutterJumpRequest {
                focus_id: "focus-lib".into(),
                line: 42,
            })
        );

        subscriber.apply_event(&EditorCoreEvent::SmartGutterApprovalRequestOpened(
            SmartGutterApprovalRequestOpenedEvent {
                file_path: path,
                focus_id: "focus-lib".into(),
                approval_request_id: "approval-1".into(),
                targets: vec![SmartGutterSyncTarget::StructurePath],
            },
        ));

        let state = subscriber.smart_gutter_state().unwrap();
        assert_eq!(
            state.last_approval_request,
            Some(SmartGutterApprovalRequest {
                focus_id: "focus-lib".into(),
                approval_request_id: "approval-1".into(),
            })
        );
    }
}
