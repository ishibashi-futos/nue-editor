use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MinimapOverlayKind {
    SearchResult,
    AiDiff,
    GitDiff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MinimapOverlayColor {
    SolarFlare,
    NeonCyan,
    MidnightGlass,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MinimapOverlay {
    pub line: usize,
    pub kind: MinimapOverlayKind,
    pub color: MinimapOverlayColor,
    pub focus_id: Option<String>,
}

impl MinimapOverlay {
    pub fn search_result(line: usize) -> Self {
        Self {
            line,
            kind: MinimapOverlayKind::SearchResult,
            color: MinimapOverlayColor::SolarFlare,
            focus_id: None,
        }
    }

    pub fn ai_diff(line: usize, focus_id: impl Into<String>) -> Self {
        Self {
            line,
            kind: MinimapOverlayKind::AiDiff,
            color: MinimapOverlayColor::NeonCyan,
            focus_id: Some(focus_id.into()),
        }
    }

    pub fn git_diff(line: usize, focus_id: impl Into<String>) -> Self {
        Self {
            line,
            kind: MinimapOverlayKind::GitDiff,
            color: MinimapOverlayColor::MidnightGlass,
            focus_id: Some(focus_id.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimapSnapshot {
    pub file_path: PathBuf,
    pub line_count: usize,
    pub overlays: Vec<MinimapOverlay>,
    pub active_focus_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimapOverlaysUpdatedEvent {
    pub file_path: PathBuf,
    pub overlay_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusSyncTarget {
    CommandHub,
    StructurePath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimapFocusIdSyncedEvent {
    pub file_path: PathBuf,
    pub focus_id: String,
    pub targets: Vec<FocusSyncTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MinimapServiceEvent {
    OverlaysUpdated(MinimapOverlaysUpdatedEvent),
    FocusIdSynced(MinimapFocusIdSyncedEvent),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MinimapService {
    state: Option<MinimapState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MinimapState {
    file_path: PathBuf,
    line_count: usize,
    overlays: Vec<MinimapOverlay>,
    active_focus_id: Option<String>,
}

impl MinimapService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_buffer_opened(&mut self, file_path: &Path, content: &str) {
        self.state = Some(MinimapState {
            file_path: file_path.to_path_buf(),
            line_count: count_lines(content),
            overlays: Vec::new(),
            active_focus_id: None,
        });
    }

    pub fn on_buffer_updated(&mut self, content: &str) -> Option<MinimapServiceEvent> {
        let state = self.state.as_mut()?;
        let previous_overlay_count = state.overlays.len();
        let previous_active_focus_id = state.active_focus_id.clone();
        state.line_count = count_lines(content);
        state
            .overlays
            .retain(|overlay| overlay.line >= 1 && overlay.line <= state.line_count);
        if state
            .active_focus_id
            .as_ref()
            .is_some_and(|focus_id| !state.has_focus(focus_id))
        {
            state.active_focus_id = None;
        }

        if previous_overlay_count != state.overlays.len()
            || previous_active_focus_id != state.active_focus_id
        {
            return Some(MinimapServiceEvent::OverlaysUpdated(
                MinimapOverlaysUpdatedEvent {
                    file_path: state.file_path.clone(),
                    overlay_count: state.overlays.len(),
                },
            ));
        }

        None
    }

    pub fn replace_overlays(
        &mut self,
        overlays: Vec<MinimapOverlay>,
    ) -> Option<MinimapServiceEvent> {
        let state = self.state.as_mut()?;
        state.overlays = normalize_overlays(overlays, state.line_count);
        if state
            .active_focus_id
            .as_ref()
            .is_some_and(|focus_id| !state.has_focus(focus_id))
        {
            state.active_focus_id = None;
        }

        Some(MinimapServiceEvent::OverlaysUpdated(
            MinimapOverlaysUpdatedEvent {
                file_path: state.file_path.clone(),
                overlay_count: state.overlays.len(),
            },
        ))
    }

    pub fn sync_focus_id(&mut self, focus_id: &str) -> Option<MinimapServiceEvent> {
        let state = self.state.as_mut()?;
        if !state.has_focus(focus_id) {
            return None;
        }
        if state.active_focus_id.as_deref() == Some(focus_id) {
            return None;
        }
        state.active_focus_id = Some(focus_id.to_string());

        Some(MinimapServiceEvent::FocusIdSynced(
            MinimapFocusIdSyncedEvent {
                file_path: state.file_path.clone(),
                focus_id: focus_id.to_string(),
                targets: vec![FocusSyncTarget::CommandHub, FocusSyncTarget::StructurePath],
            },
        ))
    }

    pub fn has_focus(&self, focus_id: &str) -> bool {
        self.state
            .as_ref()
            .is_some_and(|state| state.has_focus(focus_id))
    }

    pub fn snapshot(&self) -> Option<MinimapSnapshot> {
        let state = self.state.as_ref()?;
        Some(MinimapSnapshot {
            file_path: state.file_path.clone(),
            line_count: state.line_count,
            overlays: state.overlays.clone(),
            active_focus_id: state.active_focus_id.clone(),
        })
    }
}

impl MinimapState {
    fn has_focus(&self, focus_id: &str) -> bool {
        self.overlays
            .iter()
            .any(|overlay| overlay.focus_id.as_deref() == Some(focus_id))
    }
}

fn count_lines(content: &str) -> usize {
    content.split('\n').count().max(1)
}

fn normalize_overlays(mut overlays: Vec<MinimapOverlay>, line_count: usize) -> Vec<MinimapOverlay> {
    overlays.retain(|overlay| overlay.line >= 1 && overlay.line <= line_count);
    overlays.sort_by(|left, right| overlay_sort_key(left).cmp(&overlay_sort_key(right)));
    overlays.dedup();
    overlays
}

fn overlay_sort_key(overlay: &MinimapOverlay) -> (usize, u8, u8, &str) {
    let kind_rank = match overlay.kind {
        MinimapOverlayKind::SearchResult => 0,
        MinimapOverlayKind::AiDiff => 1,
        MinimapOverlayKind::GitDiff => 2,
    };
    let color_rank = match overlay.color {
        MinimapOverlayColor::SolarFlare => 0,
        MinimapOverlayColor::NeonCyan => 1,
        MinimapOverlayColor::MidnightGlass => 2,
    };

    (
        overlay.line,
        kind_rank,
        color_rank,
        overlay.focus_id.as_deref().unwrap_or(""),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn オーバーレイ更新で行範囲外を除外してイベントを返す() {
        let mut service = MinimapService::new();
        service.on_buffer_opened(Path::new("docs/readme.md"), "one\ntwo");

        let event = service.replace_overlays(vec![
            MinimapOverlay::search_result(2),
            MinimapOverlay::search_result(3),
            MinimapOverlay::ai_diff(1, "focus-ai"),
        ]);

        assert_eq!(
            event,
            Some(MinimapServiceEvent::OverlaysUpdated(
                MinimapOverlaysUpdatedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    overlay_count: 2,
                }
            ))
        );
        assert_eq!(
            service.snapshot(),
            Some(MinimapSnapshot {
                file_path: PathBuf::from("docs/readme.md"),
                line_count: 2,
                overlays: vec![
                    MinimapOverlay::ai_diff(1, "focus-ai"),
                    MinimapOverlay::search_result(2),
                ],
                active_focus_id: None,
            })
        );
    }

    #[test]
    fn focus_id同期でcommand_hubとstructure_pathを対象にする() {
        let mut service = MinimapService::new();
        service.on_buffer_opened(Path::new("docs/readme.md"), "one\ntwo");
        service.replace_overlays(vec![MinimapOverlay::ai_diff(2, "focus-ai")]);

        let event = service.sync_focus_id("focus-ai");

        assert_eq!(
            event,
            Some(MinimapServiceEvent::FocusIdSynced(
                MinimapFocusIdSyncedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    focus_id: "focus-ai".to_string(),
                    targets: vec![FocusSyncTarget::CommandHub, FocusSyncTarget::StructurePath],
                }
            ))
        );
        assert_eq!(
            service
                .snapshot()
                .expect("snapshotが存在する")
                .active_focus_id,
            Some("focus-ai".to_string())
        );
    }
}
