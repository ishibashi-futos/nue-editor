use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

/// Structure Path 上のセグメント種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructurePathSegmentKind {
    Project,
    Folder,
    File,
    Symbol(StructurePathSymbolKind),
}

/// Structure Path で扱うシンボルの種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructurePathSymbolKind {
    Module,
    Class,
    Method,
    Function,
}

/// エージェントのステータス表現。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StructurePathAgentStatus {
    Busy,
    Waiting,
    Error,
    #[default]
    Idle,
}

/// 差分のソース。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructurePathDiffSource {
    Ai,
    Git,
}

/// 差分の承認状態。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructurePathDiffState {
    Pending,
    Approved,
    Neutral,
}

/// セグメントに付与される差分マーカー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructurePathDiffMarker {
    pub focus_id: String,
    pub source: StructurePathDiffSource,
    pub state: StructurePathDiffState,
    pub agent_status: StructurePathAgentStatus,
}

/// Structure Path のセグメント情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructurePathSegment {
    pub id: String,
    pub label: String,
    pub kind: StructurePathSegmentKind,
    pub absolute_path: Option<PathBuf>,
    pub focus_ids: Vec<String>,
    pub agent_status: StructurePathAgentStatus,
    pub diff_markers: Vec<StructurePathDiffMarker>,
}

impl StructurePathSegment {
    fn with_path(
        kind: StructurePathSegmentKind,
        label: String,
        path: Option<PathBuf>,
        ordinal: usize,
    ) -> Self {
        Self {
            id: build_segment_id(&kind, &label, path.as_deref(), ordinal),
            label,
            kind,
            absolute_path: path,
            focus_ids: Vec::new(),
            agent_status: StructurePathAgentStatus::Idle,
            diff_markers: Vec::new(),
        }
    }
}

/// Symbol 表現。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructurePathSymbol {
    pub label: String,
    pub kind: StructurePathSymbolKind,
}

/// 差分マーカーを付与したい対象セグメントの指定方法。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructurePathMarkerTarget {
    File,
    Symbol(String),
    SegmentId(String),
}

/// 差分マーカー付与時のペイロード。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructurePathDiffMarkerPayload {
    pub focus_id: String,
    pub source: StructurePathDiffSource,
    pub state: StructurePathDiffState,
    pub agent_status: StructurePathAgentStatus,
    pub target: StructurePathMarkerTarget,
}

/// Structure Path のスナップショット。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructurePathSnapshot {
    pub segments: Vec<StructurePathSegment>,
    pub active_focus_id: Option<String>,
    pub re_scoring_hint: Option<String>,
    pub command_hub_backoff: bool,
}

/// Structure Path の状態を管理するサービス。
#[derive(Debug, Default)]
pub struct StructurePathService {
    workspace_root: Option<PathBuf>,
    file_path: Option<PathBuf>,
    symbol_stack: Vec<StructurePathSymbol>,
    segments: Vec<StructurePathSegment>,
    focus_map: HashMap<String, Vec<usize>>,
    active_focus_id: Option<String>,
    re_scoring_hint: Option<String>,
    command_hub_backoff: bool,
}

impl StructurePathService {
    /// 新規サービスを作成する。
    pub fn new() -> Self {
        Self::default()
    }

    /// ワークスペースのルートパスを設定する。変更があればセグメントを再構築する。
    pub fn set_workspace_root(&mut self, root: Option<PathBuf>) -> bool {
        if self.workspace_root == root {
            return false;
        }
        self.workspace_root = root;
        self.rebuild_segments();
        true
    }

    /// 現在アクティブなファイルパスを更新する。差分スナップショットを再生成する。
    pub fn update_active_path<P: AsRef<Path>>(&mut self, file_path: P) -> bool {
        let normalized = file_path.as_ref().to_path_buf();
        if self
            .file_path
            .as_ref()
            .map(|existing| existing == &normalized)
            == Some(true)
        {
            return false;
        }
        self.file_path = Some(normalized);
        self.rebuild_segments();
        true
    }

    /// シンボルスタック（クラス・メソッドなど）を更新する。
    pub fn set_symbol_stack(&mut self, symbols: Vec<StructurePathSymbol>) -> bool {
        if self.symbol_stack == symbols {
            return false;
        }
        self.symbol_stack = symbols;
        self.rebuild_segments();
        true
    }

    /// 差分マーカーをセグメントに付与し、アクティブフォーカスを更新する。
    pub fn mark_diff_marker(&mut self, payload: StructurePathDiffMarkerPayload) -> bool {
        let target_indices = self.identify_target_indices(&payload.target);
        if target_indices.is_empty() {
            return false;
        }

        let mut changed = false;
        let marker = StructurePathDiffMarker {
            focus_id: payload.focus_id.clone(),
            source: payload.source,
            state: payload.state,
            agent_status: payload.agent_status,
        };

        for index in target_indices {
            if let Some(segment) = self.segments.get_mut(index) {
                if !segment.focus_ids.iter().any(|id| id == &payload.focus_id) {
                    segment.focus_ids.push(payload.focus_id.clone());
                    changed = true;
                }
                if upsert_diff_marker(&mut segment.diff_markers, marker.clone()) {
                    changed = true;
                }
                if segment.agent_status != payload.agent_status {
                    segment.agent_status = payload.agent_status;
                    changed = true;
                }
            }
        }

        if self.active_focus_id.as_deref() != Some(&payload.focus_id) {
            self.active_focus_id = Some(payload.focus_id.clone());
            changed = true;
        }

        if changed {
            self.rebuild_focus_map();
        }

        changed
    }

    /// 指定した focus_id を保持するマーカーを削除する。
    pub fn clear_focus(&mut self, focus_id: &str) -> bool {
        let mut changed = false;
        for segment in &mut self.segments {
            if segment.focus_ids.iter().any(|id| id == focus_id) {
                segment.focus_ids.retain(|id| id != focus_id);
                changed = true;
            }
            if segment
                .diff_markers
                .iter()
                .any(|marker| marker.focus_id == focus_id)
            {
                segment
                    .diff_markers
                    .retain(|marker| marker.focus_id != focus_id);
                changed = true;
            }
        }
        if self.active_focus_id.as_deref() == Some(focus_id) {
            self.active_focus_id = None;
            changed = true;
        }
        if changed {
            self.rebuild_focus_map();
        }
        changed
    }

    /// Command Hub の再スコアリングラベルを更新する。
    pub fn set_re_scoring_hint(&mut self, hint: Option<String>) -> bool {
        if self.re_scoring_hint == hint {
            return false;
        }
        self.re_scoring_hint = hint;
        true
    }

    /// Command Hub の Backoff 状態を設定する。
    pub fn set_command_hub_backoff(&mut self, active: bool) -> bool {
        if self.command_hub_backoff == active {
            return false;
        }
        self.command_hub_backoff = active;
        true
    }

    /// 現在の Structure Path のスナップショットを取得する。
    pub fn snapshot(&self) -> StructurePathSnapshot {
        StructurePathSnapshot {
            segments: self.segments.clone(),
            active_focus_id: self.active_focus_id.clone(),
            re_scoring_hint: self.re_scoring_hint.clone(),
            command_hub_backoff: self.command_hub_backoff,
        }
    }

    /// Alt+番号でアクセスするセグメントを取得する（1 始まり）。
    pub fn segment_by_shortcut(&self, index: usize) -> Option<&StructurePathSegment> {
        if index == 0 {
            return None;
        }
        self.segments.get(index - 1)
    }

    /// セグメントをクリックすると最初の focus をアクティブにする。
    pub fn click_segment(&mut self, segment_id: &str) -> Option<String> {
        let target = self
            .segments
            .iter()
            .find(|segment| segment.id == segment_id)?;
        let focus = target.focus_ids.first().cloned()?;
        if self.active_focus_id.as_deref() != Some(&focus) {
            self.active_focus_id = Some(focus.clone());
        }
        Some(focus)
    }

    fn rebuild_segments(&mut self) -> bool {
        let previous_segments = self.segments.clone();
        let previous_ids: Vec<String> = previous_segments
            .iter()
            .map(|segment| segment.id.clone())
            .collect();
        let preserved_markers: HashMap<String, (Vec<String>, Vec<StructurePathDiffMarker>)> =
            previous_segments
                .into_iter()
                .map(|segment| {
                    (
                        segment.id.clone(),
                        (segment.focus_ids, segment.diff_markers),
                    )
                })
                .collect();

        let new_segments = self.build_segments();
        let new_ids: Vec<String> = new_segments
            .iter()
            .map(|segment| segment.id.clone())
            .collect();
        let changed = previous_ids != new_ids;

        self.segments = new_segments;
        for segment in &mut self.segments {
            if let Some((focus_ids, diff_markers)) = preserved_markers.get(&segment.id) {
                segment.focus_ids = focus_ids.clone();
                segment.diff_markers = diff_markers.clone();
            }
        }
        self.rebuild_focus_map();
        if let Some(active) = self.active_focus_id.clone()
            && !self.focus_map.contains_key(&active)
        {
            self.active_focus_id = None;
        }
        changed
    }

    fn rebuild_focus_map(&mut self) {
        self.focus_map.clear();
        for (index, segment) in self.segments.iter().enumerate() {
            for focus_id in &segment.focus_ids {
                self.focus_map
                    .entry(focus_id.clone())
                    .or_default()
                    .push(index);
            }
        }
    }

    fn build_segments(&self) -> Vec<StructurePathSegment> {
        let file_path = match &self.file_path {
            Some(path) => path,
            None => return Vec::new(),
        };

        let normalized_total = normalized_components(file_path);
        let (project_label, uses_workspace_root) = if let Some(root) = &self.workspace_root {
            let label = root
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| root.display().to_string());
            (label, file_path.starts_with(root))
        } else {
            (
                normalized_total
                    .first()
                    .cloned()
                    .unwrap_or_else(|| file_path.display().to_string()),
                false,
            )
        };

        let relative_components = if uses_workspace_root {
            if let Some(root) = &self.workspace_root {
                file_path
                    .strip_prefix(root)
                    .map(normalized_components)
                    .unwrap_or_else(|_| normalized_total.clone())
            } else {
                normalized_total.clone()
            }
        } else {
            normalized_total.clone()
        };

        let folder_names = if uses_workspace_root {
            relative_components
                .get(..relative_components.len().saturating_sub(1))
                .unwrap_or(&[])
                .to_vec()
        } else if relative_components.len() > 2 {
            relative_components[1..relative_components.len() - 1].to_vec()
        } else {
            Vec::new()
        };

        let file_label = relative_components
            .last()
            .cloned()
            .or_else(|| {
                file_path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| file_path.display().to_string());

        let mut segments = Vec::new();
        let mut ordinal = 0;

        let project_path = self
            .workspace_root
            .clone()
            .or_else(|| file_path.parent().map(Path::to_path_buf));
        segments.push(StructurePathSegment::with_path(
            StructurePathSegmentKind::Project,
            project_label.clone(),
            project_path.clone(),
            ordinal,
        ));
        ordinal += 1;

        let mut path_accum = project_path.clone().unwrap_or_default();
        if !uses_workspace_root && project_path.is_none() && !project_label.is_empty() {
            path_accum.push(&project_label);
        }

        for folder in folder_names {
            path_accum.push(&folder);
            segments.push(StructurePathSegment::with_path(
                StructurePathSegmentKind::Folder,
                folder.clone(),
                Some(path_accum.clone()),
                ordinal,
            ));
            ordinal += 1;
        }

        segments.push(StructurePathSegment::with_path(
            StructurePathSegmentKind::File,
            file_label,
            Some(file_path.clone()),
            ordinal,
        ));
        ordinal += 1;

        for symbol in &self.symbol_stack {
            segments.push(StructurePathSegment::with_path(
                StructurePathSegmentKind::Symbol(symbol.kind),
                symbol.label.clone(),
                None,
                ordinal,
            ));
            ordinal += 1;
        }

        segments
    }

    fn identify_target_indices(&self, target: &StructurePathMarkerTarget) -> Vec<usize> {
        match target {
            StructurePathMarkerTarget::File => self
                .segments
                .iter()
                .enumerate()
                .rev()
                .find(|(_, segment)| matches!(segment.kind, StructurePathSegmentKind::File))
                .map(|(index, _)| vec![index])
                .unwrap_or_default(),
            StructurePathMarkerTarget::Symbol(name) => self
                .segments
                .iter()
                .enumerate()
                .filter(|(_, segment)| match &segment.kind {
                    StructurePathSegmentKind::Symbol(_) => &segment.label == name,
                    _ => false,
                })
                .map(|(index, _)| index)
                .collect(),
            StructurePathMarkerTarget::SegmentId(id) => self
                .segments
                .iter()
                .enumerate()
                .filter(|(_, segment)| segment.id == *id)
                .map(|(index, _)| index)
                .collect(),
        }
    }
}

fn upsert_diff_marker(
    markers: &mut Vec<StructurePathDiffMarker>,
    marker: StructurePathDiffMarker,
) -> bool {
    if let Some(existing) = markers
        .iter_mut()
        .find(|existing| existing.focus_id == marker.focus_id)
    {
        if *existing != marker {
            *existing = marker;
            return true;
        }
        return false;
    }
    markers.push(marker);
    true
}

fn build_segment_id(
    kind: &StructurePathSegmentKind,
    label: &str,
    path: Option<&Path>,
    ordinal: usize,
) -> String {
    if let Some(path) = path {
        format!("{:?}:{:?}:{}@{}", kind, path.display(), label, ordinal)
    } else {
        format!("{:?}:{}@{}", kind, label, ordinal)
    }
}

fn normalized_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name.to_string_lossy().to_string()),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_service() -> StructurePathService {
        let mut service = StructurePathService::new();
        assert!(service.set_workspace_root(Some(PathBuf::from("workspace"))));
        assert!(service.update_active_path("workspace/src/lib.rs"));
        service
    }

    #[test]
    fn project_folder_file_segmentsが生成される() {
        let service = base_service();
        let snapshot = service.snapshot();
        assert_eq!(snapshot.segments.len(), 3);
        assert!(matches!(
            snapshot.segments[0].kind,
            StructurePathSegmentKind::Project
        ));
        assert_eq!(snapshot.segments[1].label, "src");
        assert!(matches!(
            snapshot.segments[1].kind,
            StructurePathSegmentKind::Folder
        ));
        assert_eq!(snapshot.segments[2].label, "lib.rs");
        assert!(matches!(
            snapshot.segments[2].kind,
            StructurePathSegmentKind::File
        ));
        assert!(snapshot.active_focus_id.is_none());
    }

    #[test]
    fn symbol_stackが末尾に追加される() {
        let mut service = base_service();
        let symbols = vec![StructurePathSymbol {
            label: "MyModule".to_string(),
            kind: StructurePathSymbolKind::Module,
        }];
        assert!(service.set_symbol_stack(symbols));
        let snapshot = service.snapshot();
        assert_eq!(snapshot.segments.len(), 4);
        match snapshot.segments[3].kind {
            StructurePathSegmentKind::Symbol(symbol_kind) => {
                assert_eq!(symbol_kind, StructurePathSymbolKind::Module);
            }
            _ => panic!("シンボルセグメントであるべき"),
        }
        assert_eq!(snapshot.segments[3].label, "MyModule");
    }

    #[test]
    fn focus_markerがファイルに紐付いてactive_focusになる() {
        let mut service = base_service();
        let payload = StructurePathDiffMarkerPayload {
            focus_id: "focus-ai".to_string(),
            source: StructurePathDiffSource::Ai,
            state: StructurePathDiffState::Pending,
            agent_status: StructurePathAgentStatus::Busy,
            target: StructurePathMarkerTarget::File,
        };
        assert!(service.mark_diff_marker(payload));
        let snapshot = service.snapshot();
        assert_eq!(snapshot.active_focus_id.as_deref(), Some("focus-ai"));
        let file_segment = snapshot
            .segments
            .iter()
            .find(|segment| matches!(segment.kind, StructurePathSegmentKind::File))
            .unwrap();
        assert_eq!(file_segment.focus_ids, vec!["focus-ai".to_string()]);
        assert_eq!(file_segment.diff_markers.len(), 1);
        assert_eq!(
            file_segment.diff_markers[0].source,
            StructurePathDiffSource::Ai
        );
    }

    #[test]
    fn clear_focusでフォーカスとマーカーが消える() {
        let mut service = base_service();
        let payload = StructurePathDiffMarkerPayload {
            focus_id: "focus-ai".to_string(),
            source: StructurePathDiffSource::Ai,
            state: StructurePathDiffState::Pending,
            agent_status: StructurePathAgentStatus::Busy,
            target: StructurePathMarkerTarget::File,
        };
        service.mark_diff_marker(payload);
        assert!(service.clear_focus("focus-ai"));
        let snapshot = service.snapshot();
        assert!(snapshot.active_focus_id.is_none());
        let file_segment = snapshot
            .segments
            .iter()
            .find(|segment| matches!(segment.kind, StructurePathSegmentKind::File))
            .unwrap();
        assert!(file_segment.diff_markers.is_empty());
        assert!(file_segment.focus_ids.is_empty());
    }

    #[test]
    fn shortcutとclickでフォーカスが設定される() {
        let mut service = base_service();
        let payload = StructurePathDiffMarkerPayload {
            focus_id: "focus-git".to_string(),
            source: StructurePathDiffSource::Git,
            state: StructurePathDiffState::Pending,
            agent_status: StructurePathAgentStatus::Waiting,
            target: StructurePathMarkerTarget::File,
        };
        service.mark_diff_marker(payload);
        let snapshot = service.snapshot();
        let file_segment = snapshot
            .segments
            .iter()
            .find(|segment| matches!(segment.kind, StructurePathSegmentKind::File))
            .unwrap();
        let id = file_segment.id.clone();
        assert_eq!(
            service.segment_by_shortcut(1).map(|segment| &segment.kind),
            Some(&StructurePathSegmentKind::Project)
        );
        let clicked = service.click_segment(&id);
        assert_eq!(clicked.as_deref(), Some("focus-git"));
        let changed = service.snapshot();
        assert_eq!(changed.active_focus_id.as_deref(), Some("focus-git"));
    }
}
