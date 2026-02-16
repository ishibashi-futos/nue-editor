use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SmartGutterDiffKind {
    Ai,
    Git,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SmartGutterAnimation {
    SolarFlarePulse,
    MidnightGlassStatic,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SmartGutterIndicator {
    pub line: usize,
    pub diff_kind: SmartGutterDiffKind,
    pub animation: SmartGutterAnimation,
    pub focus_id: String,
    pub approval_request_id: Option<String>,
}

impl SmartGutterIndicator {
    pub fn ai_diff(
        line: usize,
        focus_id: impl Into<String>,
        approval_request_id: impl Into<String>,
    ) -> Self {
        Self {
            line,
            diff_kind: SmartGutterDiffKind::Ai,
            animation: SmartGutterAnimation::SolarFlarePulse,
            focus_id: focus_id.into(),
            approval_request_id: Some(approval_request_id.into()),
        }
    }

    pub fn git_diff(line: usize, focus_id: impl Into<String>) -> Self {
        Self {
            line,
            diff_kind: SmartGutterDiffKind::Git,
            animation: SmartGutterAnimation::MidnightGlassStatic,
            focus_id: focus_id.into(),
            approval_request_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartGutterSnapshot {
    pub file_path: PathBuf,
    pub line_count: usize,
    pub indicators: Vec<SmartGutterIndicator>,
    pub active_focus_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartGutterIndicatorsUpdatedEvent {
    pub file_path: PathBuf,
    pub indicator_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmartGutterSyncTarget {
    CommandHub,
    StructurePath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartGutterFocusIdSyncedEvent {
    pub file_path: PathBuf,
    pub focus_id: String,
    pub targets: Vec<SmartGutterSyncTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartGutterJumpRequestedEvent {
    pub file_path: PathBuf,
    pub focus_id: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartGutterApprovalRequestOpenedEvent {
    pub file_path: PathBuf,
    pub focus_id: String,
    pub approval_request_id: String,
    pub targets: Vec<SmartGutterSyncTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmartGutterServiceEvent {
    IndicatorsUpdated(SmartGutterIndicatorsUpdatedEvent),
    FocusIdSynced(SmartGutterFocusIdSyncedEvent),
    JumpRequested(SmartGutterJumpRequestedEvent),
    ApprovalRequestOpened(SmartGutterApprovalRequestOpenedEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenApprovalRequestError {
    FocusNotFound,
    MissingApprovalRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SmartGutterService {
    state: Option<SmartGutterState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SmartGutterState {
    file_path: PathBuf,
    line_count: usize,
    indicators: Vec<SmartGutterIndicator>,
    active_focus_id: Option<String>,
}

impl SmartGutterService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_buffer_opened(&mut self, file_path: &Path, content: &str) {
        self.state = Some(SmartGutterState {
            file_path: file_path.to_path_buf(),
            line_count: count_lines(content),
            indicators: Vec::new(),
            active_focus_id: None,
        });
    }

    pub fn on_buffer_updated(&mut self, content: &str) -> Option<SmartGutterServiceEvent> {
        let state = self.state.as_mut()?;
        let previous_indicator_count = state.indicators.len();
        let previous_active_focus_id = state.active_focus_id.clone();
        state.line_count = count_lines(content);
        state
            .indicators
            .retain(|indicator| indicator.line >= 1 && indicator.line <= state.line_count);
        if state
            .active_focus_id
            .as_ref()
            .is_some_and(|focus_id| !state.has_focus(focus_id))
        {
            state.active_focus_id = None;
        }

        if previous_indicator_count != state.indicators.len()
            || previous_active_focus_id != state.active_focus_id
        {
            return Some(SmartGutterServiceEvent::IndicatorsUpdated(
                SmartGutterIndicatorsUpdatedEvent {
                    file_path: state.file_path.clone(),
                    indicator_count: state.indicators.len(),
                },
            ));
        }

        None
    }

    pub fn replace_indicators(
        &mut self,
        indicators: Vec<SmartGutterIndicator>,
    ) -> Option<SmartGutterServiceEvent> {
        let state = self.state.as_mut()?;
        state.indicators = normalize_indicators(indicators, state.line_count);
        if state
            .active_focus_id
            .as_ref()
            .is_some_and(|focus_id| !state.has_focus(focus_id))
        {
            state.active_focus_id = None;
        }

        Some(SmartGutterServiceEvent::IndicatorsUpdated(
            SmartGutterIndicatorsUpdatedEvent {
                file_path: state.file_path.clone(),
                indicator_count: state.indicators.len(),
            },
        ))
    }

    pub fn has_focus(&self, focus_id: &str) -> bool {
        self.state
            .as_ref()
            .is_some_and(|state| state.has_focus(focus_id))
    }

    pub fn sync_focus_id(&mut self, focus_id: &str) -> Option<SmartGutterServiceEvent> {
        let state = self.state.as_mut()?;
        if !state.has_focus(focus_id) {
            return None;
        }
        if state.active_focus_id.as_deref() == Some(focus_id) {
            return None;
        }
        state.active_focus_id = Some(focus_id.to_string());

        Some(SmartGutterServiceEvent::FocusIdSynced(
            SmartGutterFocusIdSyncedEvent {
                file_path: state.file_path.clone(),
                focus_id: focus_id.to_string(),
                targets: vec![
                    SmartGutterSyncTarget::CommandHub,
                    SmartGutterSyncTarget::StructurePath,
                ],
            },
        ))
    }

    pub fn request_jump(&self, focus_id: &str) -> Option<SmartGutterServiceEvent> {
        let state = self.state.as_ref()?;
        let indicator = state.find_by_focus_id(focus_id)?;
        Some(SmartGutterServiceEvent::JumpRequested(
            SmartGutterJumpRequestedEvent {
                file_path: state.file_path.clone(),
                focus_id: focus_id.to_string(),
                line: indicator.line,
            },
        ))
    }

    pub fn open_approval_request(
        &self,
        focus_id: &str,
    ) -> Result<SmartGutterServiceEvent, OpenApprovalRequestError> {
        let Some(state) = self.state.as_ref() else {
            return Err(OpenApprovalRequestError::FocusNotFound);
        };
        let Some(indicator) = state.find_by_focus_id(focus_id) else {
            return Err(OpenApprovalRequestError::FocusNotFound);
        };
        let Some(approval_request_id) = indicator.approval_request_id.clone() else {
            return Err(OpenApprovalRequestError::MissingApprovalRequest);
        };

        Ok(SmartGutterServiceEvent::ApprovalRequestOpened(
            SmartGutterApprovalRequestOpenedEvent {
                file_path: state.file_path.clone(),
                focus_id: focus_id.to_string(),
                approval_request_id,
                targets: vec![
                    SmartGutterSyncTarget::CommandHub,
                    SmartGutterSyncTarget::StructurePath,
                ],
            },
        ))
    }

    pub fn snapshot(&self) -> Option<SmartGutterSnapshot> {
        let state = self.state.as_ref()?;
        Some(SmartGutterSnapshot {
            file_path: state.file_path.clone(),
            line_count: state.line_count,
            indicators: state.indicators.clone(),
            active_focus_id: state.active_focus_id.clone(),
        })
    }
}

impl SmartGutterState {
    fn has_focus(&self, focus_id: &str) -> bool {
        self.indicators
            .iter()
            .any(|indicator| indicator.focus_id == focus_id)
    }

    fn find_by_focus_id(&self, focus_id: &str) -> Option<&SmartGutterIndicator> {
        self.indicators
            .iter()
            .find(|indicator| indicator.focus_id == focus_id)
    }
}

fn count_lines(content: &str) -> usize {
    content.split('\n').count().max(1)
}

fn normalize_indicators(
    mut indicators: Vec<SmartGutterIndicator>,
    line_count: usize,
) -> Vec<SmartGutterIndicator> {
    indicators.retain(|indicator| indicator.line >= 1 && indicator.line <= line_count);
    indicators.sort_by(|left, right| indicator_sort_key(left).cmp(&indicator_sort_key(right)));
    indicators.dedup();
    indicators
}

fn indicator_sort_key(indicator: &SmartGutterIndicator) -> (usize, u8, u8, &str, &str, &str) {
    let diff_rank = match indicator.diff_kind {
        SmartGutterDiffKind::Ai => 0,
        SmartGutterDiffKind::Git => 1,
    };
    let animation_rank = match indicator.animation {
        SmartGutterAnimation::SolarFlarePulse => 0,
        SmartGutterAnimation::MidnightGlassStatic => 1,
    };

    (
        indicator.line,
        diff_rank,
        animation_rank,
        indicator.focus_id.as_str(),
        indicator.approval_request_id.as_deref().unwrap_or(""),
        if indicator.approval_request_id.is_some() {
            "1"
        } else {
            "0"
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_git差分を保持してfocus同期できる() {
        let mut service = SmartGutterService::new();
        service.on_buffer_opened(Path::new("docs/readme.md"), "one\ntwo\nthree\nfour");

        let updated = service.replace_indicators(vec![
            SmartGutterIndicator::ai_diff(2, "focus-ai", "approval-1"),
            SmartGutterIndicator::git_diff(4, "focus-git"),
        ]);
        let synced = service.sync_focus_id("focus-ai");

        assert_eq!(
            updated,
            Some(SmartGutterServiceEvent::IndicatorsUpdated(
                SmartGutterIndicatorsUpdatedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    indicator_count: 2,
                }
            ))
        );
        assert_eq!(
            synced,
            Some(SmartGutterServiceEvent::FocusIdSynced(
                SmartGutterFocusIdSyncedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    focus_id: "focus-ai".to_string(),
                    targets: vec![
                        SmartGutterSyncTarget::CommandHub,
                        SmartGutterSyncTarget::StructurePath,
                    ],
                }
            ))
        );
    }

    #[test]
    fn クリックジャンプとapproval_requestを解決できる() {
        let mut service = SmartGutterService::new();
        service.on_buffer_opened(Path::new("docs/readme.md"), "one\ntwo\nthree\nfour");
        service.replace_indicators(vec![
            SmartGutterIndicator::ai_diff(2, "focus-ai", "approval-1"),
            SmartGutterIndicator::git_diff(4, "focus-git"),
        ]);

        assert_eq!(
            service.request_jump("focus-git"),
            Some(SmartGutterServiceEvent::JumpRequested(
                SmartGutterJumpRequestedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    focus_id: "focus-git".to_string(),
                    line: 4,
                }
            ))
        );
        assert_eq!(
            service.open_approval_request("focus-ai"),
            Ok(SmartGutterServiceEvent::ApprovalRequestOpened(
                SmartGutterApprovalRequestOpenedEvent {
                    file_path: PathBuf::from("docs/readme.md"),
                    focus_id: "focus-ai".to_string(),
                    approval_request_id: "approval-1".to_string(),
                    targets: vec![
                        SmartGutterSyncTarget::CommandHub,
                        SmartGutterSyncTarget::StructurePath,
                    ],
                }
            ))
        );
        assert_eq!(
            service.open_approval_request("focus-git"),
            Err(OpenApprovalRequestError::MissingApprovalRequest)
        );
    }
}
