use nue_core::workspace::rail::WorkspaceRailState;
use nue_core::workspace::registry::{
    ExcludeWorkspaceOutcome, WorkspacePathValidation, WorkspaceRegistry,
};
use std::collections::VecDeque;

use gpui::{Context, Window, div, prelude::*, px};

use crate::design_system_gpui::GpuiDesignTokens;

/// UI 側が参照するワークスペースステータス更新情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRailStatusUpdate {
    pub workspace_id: String,
    pub state: WorkspaceRailState,
    pub revision: u64,
}

/// WorkspaceRegistry 上の削除操作の結果を UI へ伝える列挙型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceRailExcludeOutcome {
    /// 指定したワークスペースが正常に除外された。
    Excluded { workspace_id: String },
    /// コンテキストメニューは閉じた状態で対応する対象がない。
    ContextMenuClosed,
    /// 対象ワークスペースが見つからなかった。
    WorkspaceNotFound,
}

/// Rail に表示するワークスペース1件分の描画モデル。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRailItemViewModel {
    pub workspace_id: String,
    pub display_name: String,
    pub root_path: String,
    pub state: WorkspaceRailState,
    pub revision: u64,
    pub is_selected: bool,
}

impl WorkspaceRailItemViewModel {
    fn from_registry_model(
        model: &nue_core::workspace::rail::WorkspaceRailModel,
        selected_workspace_id: Option<&str>,
    ) -> Self {
        let snapshot = model.snapshot();
        let metadata = model.metadata();
        Self {
            workspace_id: metadata.workspace_id.clone(),
            display_name: metadata.display_name.clone(),
            root_path: metadata.root_path.clone(),
            state: snapshot.state,
            revision: snapshot.revision,
            is_selected: selected_workspace_id
                .is_some_and(|selected| selected == metadata.workspace_id.as_str()),
        }
    }

    fn status_label(&self) -> &'static str {
        status_label(self.state)
    }
}

/// Workspace Rail 実ビューが参照する状態。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkspaceRailViewState {
    items: Vec<WorkspaceRailItemViewModel>,
    add_dialog_open: bool,
    add_dialog_input_path: String,
    add_dialog_validation_message: Option<String>,
    context_menu_target_workspace_id: Option<String>,
}

impl WorkspaceRailViewState {
    pub fn from_registry(
        registry: &WorkspaceRegistry,
        selected_workspace_id: Option<&str>,
    ) -> Self {
        let dialog = registry.dialog();
        let validation_message = if dialog.is_open {
            Some(WorkspaceRailConnector::validation_message(&dialog.validation).to_string())
        } else {
            None
        };
        let context_menu_target_workspace_id = registry
            .context_menu()
            .is_open
            .then(|| registry.context_menu().target_workspace_id.clone())
            .flatten();

        Self {
            items: registry
                .workspaces()
                .iter()
                .map(|model| {
                    WorkspaceRailItemViewModel::from_registry_model(model, selected_workspace_id)
                })
                .collect(),
            add_dialog_open: dialog.is_open,
            add_dialog_input_path: dialog.input_path.clone(),
            add_dialog_validation_message: validation_message,
            context_menu_target_workspace_id,
        }
    }

    pub fn items(&self) -> &[WorkspaceRailItemViewModel] {
        &self.items
    }

    pub fn add_dialog_open(&self) -> bool {
        self.add_dialog_open
    }

    pub fn add_dialog_input_path(&self) -> &str {
        &self.add_dialog_input_path
    }

    pub fn add_dialog_validation_message(&self) -> Option<&str> {
        self.add_dialog_validation_message.as_deref()
    }

    pub fn context_menu_target_workspace_id(&self) -> Option<&str> {
        self.context_menu_target_workspace_id.as_deref()
    }
}

/// UI からアプリ層へ通知する Workspace Rail 操作イベント。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceRailUiAction {
    RequestOpenAddDialog,
    RequestSubmitAddWorkspace { input_path: String },
    RequestOpenExcludeMenu { workspace_id: String },
    RequestExcludeWorkspace { workspace_id: String },
    RequestSelectWorkspace { workspace_id: String },
}

/// GPUI イベントハンドラからアプリ層への通知を中継する最小キュー。
#[derive(Debug, Default)]
pub struct WorkspaceRailActionQueue {
    pending: VecDeque<WorkspaceRailUiAction>,
}

impl WorkspaceRailActionQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, action: WorkspaceRailUiAction) {
        self.pending.push_back(action);
    }

    pub fn drain(&mut self) -> Vec<WorkspaceRailUiAction> {
        self.pending.drain(..).collect()
    }
}

/// テスト・デバッグ用の簡易スナップショット。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRailDebugSnapshot {
    pub item_rows: Vec<String>,
    pub add_button_label: String,
    pub exclude_button_label: String,
    pub add_dialog_open: bool,
    pub add_dialog_validation_message: Option<String>,
}

/// Workspace Rail の実ビュー（一覧/ステータス/追加・除外導線）。
#[derive(Clone, Debug)]
pub struct WorkspaceRailView {
    state: WorkspaceRailViewState,
    styles: GpuiDesignTokens,
}

impl WorkspaceRailView {
    pub fn new(state: WorkspaceRailViewState, styles: GpuiDesignTokens) -> Self {
        Self { state, styles }
    }

    pub fn state(&self) -> &WorkspaceRailViewState {
        &self.state
    }

    pub fn set_state(&mut self, state: WorkspaceRailViewState) {
        self.state = state;
    }

    pub fn debug_snapshot(&self) -> WorkspaceRailDebugSnapshot {
        WorkspaceRailDebugSnapshot {
            item_rows: self
                .state
                .items
                .iter()
                .map(|item| {
                    format!(
                        "{} [{}] rev:{}{}",
                        item.display_name,
                        item.status_label(),
                        item.revision,
                        if item.is_selected { " (selected)" } else { "" }
                    )
                })
                .collect(),
            add_button_label: "追加".to_string(),
            exclude_button_label: "除外".to_string(),
            add_dialog_open: self.state.add_dialog_open,
            add_dialog_validation_message: self.state.add_dialog_validation_message.clone(),
        }
    }

    pub fn notify_open_add_dialog(&self, queue: &mut WorkspaceRailActionQueue) {
        queue.push(WorkspaceRailUiAction::RequestOpenAddDialog);
    }

    pub fn notify_submit_add_workspace(
        &self,
        queue: &mut WorkspaceRailActionQueue,
        input_path: impl Into<String>,
    ) {
        queue.push(WorkspaceRailUiAction::RequestSubmitAddWorkspace {
            input_path: input_path.into(),
        });
    }

    pub fn notify_open_exclude_menu(
        &self,
        queue: &mut WorkspaceRailActionQueue,
        workspace_id: impl Into<String>,
    ) {
        queue.push(WorkspaceRailUiAction::RequestOpenExcludeMenu {
            workspace_id: workspace_id.into(),
        });
    }

    pub fn notify_exclude_workspace(
        &self,
        queue: &mut WorkspaceRailActionQueue,
        workspace_id: impl Into<String>,
    ) {
        queue.push(WorkspaceRailUiAction::RequestExcludeWorkspace {
            workspace_id: workspace_id.into(),
        });
    }

    pub fn notify_select_workspace(
        &self,
        queue: &mut WorkspaceRailActionQueue,
        workspace_id: impl Into<String>,
    ) {
        queue.push(WorkspaceRailUiAction::RequestSelectWorkspace {
            workspace_id: workspace_id.into(),
        });
    }

    fn status_chip(&self, state: WorkspaceRailState) -> gpui::Div {
        let palette = &self.styles.palette;
        let color = match state {
            WorkspaceRailState::Busy => palette.title_text,
            WorkspaceRailState::Waiting => palette.accent,
            WorkspaceRailState::Error => palette.body_text,
            WorkspaceRailState::Idle => palette.panel_border,
        };
        div()
            .h(px(8.0))
            .w(px(8.0))
            .rounded_full()
            .bg(color)
            .flex_none()
    }

    fn item_row(&self, item: &WorkspaceRailItemViewModel) -> gpui::Div {
        let palette = &self.styles.palette;
        let fonts = &self.styles.fonts;
        let border_color = if item.is_selected {
            palette.accent
        } else {
            palette.panel_border
        };

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_1()
            .p_2()
            .bg(palette.panel_background)
            .border_1()
            .border_color(border_color)
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font(fonts.emphasis.clone())
                            .text_color(palette.title_text)
                            .truncate()
                            .child(item.display_name.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(self.status_chip(item.state))
                            .child(
                                div()
                                    .text_xs()
                                    .font(fonts.meta.clone())
                                    .text_color(palette.body_text)
                                    .child(item.status_label().to_string()),
                            ),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .font(fonts.meta.clone())
                    .text_color(palette.body_text)
                    .truncate()
                    .child(item.root_path.clone()),
            )
    }
}

impl Render for WorkspaceRailView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let palette = &self.styles.palette;
        let fonts = &self.styles.fonts;

        let mut list = div().w_full().flex().flex_col().gap_2();
        for item in &self.state.items {
            list = list.child(self.item_row(item));
        }
        if self.state.items.is_empty() {
            list = list.child(
                div()
                    .w_full()
                    .p_2()
                    .border_1()
                    .border_color(palette.panel_border)
                    .bg(palette.panel_background)
                    .child(
                        div()
                            .text_xs()
                            .font(fonts.meta.clone())
                            .text_color(palette.body_text)
                            .child("ワークスペース未登録"),
                    ),
            );
        }

        let add_dialog_detail = if self.state.add_dialog_open {
            self.state
                .add_dialog_validation_message
                .as_deref()
                .unwrap_or("追加ダイアログを開いています。")
        } else {
            "追加ダイアログは閉じています。"
        };
        let exclude_target = self
            .state
            .context_menu_target_workspace_id
            .as_deref()
            .unwrap_or("なし");

        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_2()
            .bg(palette.panel_background)
            .border_1()
            .border_color(palette.panel_border)
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font(fonts.emphasis.clone())
                            .text_color(palette.title_text)
                            .child("Workspace Rail"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font(fonts.meta.clone())
                            .text_color(palette.body_text)
                            .child(format!("{}件", self.state.items.len())),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .p_1()
                            .border_1()
                            .border_color(palette.accent)
                            .child(
                                div()
                                    .text_xs()
                                    .font(fonts.meta.clone())
                                    .text_color(palette.accent)
                                    .child("追加"),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .p_1()
                            .border_1()
                            .border_color(palette.panel_border)
                            .child(
                                div()
                                    .text_xs()
                                    .font(fonts.meta.clone())
                                    .text_color(palette.body_text)
                                    .child("除外"),
                            ),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .p_2()
                    .border_1()
                    .border_color(palette.panel_border)
                    .child(
                        div()
                            .text_xs()
                            .font(fonts.meta.clone())
                            .text_color(palette.body_text)
                            .child(format!("追加: {}", add_dialog_detail)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font(fonts.meta.clone())
                            .text_color(palette.body_text)
                            .child(format!("除外対象: {}", exclude_target)),
                    ),
            )
            .child(div().flex_1().min_h(px(0.0)).child(list))
    }
}

fn status_label(state: WorkspaceRailState) -> &'static str {
    match state {
        WorkspaceRailState::Busy => "Busy",
        WorkspaceRailState::Waiting => "Waiting",
        WorkspaceRailState::Error => "Error",
        WorkspaceRailState::Idle => "Idle",
    }
}

/// WorkspaceRegistry との橋渡しを行うヘルパーモジュール。
pub struct WorkspaceRailConnector;

impl WorkspaceRailConnector {
    /// Registry から蓄積されたステータス更新を取り出す。
    pub fn drain_status_updates(
        registry: &mut WorkspaceRegistry,
    ) -> Vec<WorkspaceRailStatusUpdate> {
        registry
            .drain_status_events()
            .into_iter()
            .map(|event| WorkspaceRailStatusUpdate {
                workspace_id: event.workspace_id,
                state: event.state,
                revision: event.revision,
            })
            .collect()
    }

    /// コンテキストメニューから選択されているワークスペースを除外する。
    pub fn exclude_selected_workspace(
        registry: &mut WorkspaceRegistry,
    ) -> WorkspaceRailExcludeOutcome {
        match registry.exclude_context_menu_target() {
            ExcludeWorkspaceOutcome::Excluded { workspace_id } => {
                WorkspaceRailExcludeOutcome::Excluded { workspace_id }
            }
            ExcludeWorkspaceOutcome::ContextMenuClosed => {
                WorkspaceRailExcludeOutcome::ContextMenuClosed
            }
            ExcludeWorkspaceOutcome::WorkspaceNotFound => {
                WorkspaceRailExcludeOutcome::WorkspaceNotFound
            }
        }
    }

    /// パス検証の結果に応じた UI 表示用のメッセージを取得する。
    pub fn validation_message(validation: &WorkspacePathValidation) -> &'static str {
        match validation {
            WorkspacePathValidation::Empty => "パスが空です。",
            WorkspacePathValidation::MustBeAbsolute => "絶対パスを指定してください。",
            WorkspacePathValidation::NotFoundOrInaccessible => {
                "指定したパスが存在しないかアクセスできません。"
            }
            WorkspacePathValidation::MustBeDirectory => "ディレクトリを指定してください。",
            WorkspacePathValidation::Duplicate => "同じワークスペースが既に登録されています。",
            WorkspacePathValidation::Valid => "指定したパスは有効です。",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design_system::DesignSystem;
    use crate::design_system_gpui::GpuiDesignTokens;
    use nue_core::workspace::rail::WorkspaceRailState;
    use nue_core::workspace::registry::{AddWorkspaceOutcome, WorkspacePathValidation};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempWorkspaceDir {
        path: PathBuf,
    }

    impl TempWorkspaceDir {
        fn new(prefix: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("現在時刻が必要")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("{prefix}-{unique}"));
            fs::create_dir_all(&path).expect("テンポラリディレクトリを作成できる");
            Self { path }
        }

        fn path_str(&self) -> &str {
            self.path
                .to_str()
                .expect("テンポラリディレクトリはUTF-8である")
        }
    }

    impl Drop for TempWorkspaceDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn create_workspace(registry: &mut WorkspaceRegistry, dir: &TempWorkspaceDir) -> String {
        registry.open_add_dialog();
        registry.update_dialog_path(dir.path_str());
        match registry.submit_add() {
            AddWorkspaceOutcome::Added { workspace_id } => workspace_id,
            other => panic!("ワークスペース追加に失敗: {other:?}"),
        }
    }

    fn test_tokens() -> GpuiDesignTokens {
        GpuiDesignTokens::from_design_system(&DesignSystem::neon_night_glass())
    }

    #[test]
    fn drain_status_updates_returns_mapped_events() {
        let mut registry = WorkspaceRegistry::new();
        let dir = TempWorkspaceDir::new("workspace-test");
        let workspace_id = create_workspace(&mut registry, &dir);

        registry.update_workspace_status_from_server(&workspace_id, WorkspaceRailState::Busy);
        let updates = WorkspaceRailConnector::drain_status_updates(&mut registry);

        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].workspace_id, workspace_id);
        assert_eq!(updates[0].state, WorkspaceRailState::Busy);
        assert_eq!(updates[0].revision, 1);
        assert!(WorkspaceRailConnector::drain_status_updates(&mut registry).is_empty());
    }

    #[test]
    fn exclude_selected_workspace_reports_outcome() {
        let mut registry = WorkspaceRegistry::new();
        let dir = TempWorkspaceDir::new("workspace-test");
        let workspace_id = create_workspace(&mut registry, &dir);

        registry.open_exclude_context_menu(&workspace_id);
        let outcome = WorkspaceRailConnector::exclude_selected_workspace(&mut registry);

        assert_eq!(
            outcome,
            WorkspaceRailExcludeOutcome::Excluded { workspace_id }
        );
    }

    #[test]
    fn validation_message_covers_all_variants() {
        assert_eq!(
            WorkspaceRailConnector::validation_message(&WorkspacePathValidation::Empty),
            "パスが空です。"
        );
        assert_eq!(
            WorkspaceRailConnector::validation_message(&WorkspacePathValidation::MustBeAbsolute),
            "絶対パスを指定してください。"
        );
        assert_eq!(
            WorkspaceRailConnector::validation_message(
                &WorkspacePathValidation::NotFoundOrInaccessible,
            ),
            "指定したパスが存在しないかアクセスできません。"
        );
        assert_eq!(
            WorkspaceRailConnector::validation_message(&WorkspacePathValidation::MustBeDirectory),
            "ディレクトリを指定してください。"
        );
        assert_eq!(
            WorkspaceRailConnector::validation_message(&WorkspacePathValidation::Duplicate),
            "同じワークスペースが既に登録されています。"
        );
        assert_eq!(
            WorkspaceRailConnector::validation_message(&WorkspacePathValidation::Valid),
            "指定したパスは有効です。"
        );
    }

    #[test]
    fn view_state_from_registry_reflects_items_and_dialog_status() {
        let mut registry = WorkspaceRegistry::new();
        let dir_a = TempWorkspaceDir::new("workspace-a");
        let dir_b = TempWorkspaceDir::new("workspace-b");
        let workspace_a = create_workspace(&mut registry, &dir_a);
        let workspace_b = create_workspace(&mut registry, &dir_b);
        registry.update_workspace_status_from_server(&workspace_b, WorkspaceRailState::Waiting);
        registry.open_add_dialog();
        registry.update_dialog_path("relative/path");
        registry.open_exclude_context_menu(&workspace_b);

        let state = WorkspaceRailViewState::from_registry(&registry, Some(workspace_b.as_str()));

        assert_eq!(state.items().len(), 2);
        assert_eq!(state.items()[0].workspace_id, workspace_a);
        assert_eq!(state.items()[1].workspace_id, workspace_b);
        assert_eq!(state.items()[1].state, WorkspaceRailState::Waiting);
        assert!(state.items()[1].is_selected);
        assert!(state.add_dialog_open());
        assert_eq!(state.add_dialog_input_path(), "relative/path");
        assert_eq!(
            state.add_dialog_validation_message(),
            Some("絶対パスを指定してください。")
        );
        assert_eq!(
            state.context_menu_target_workspace_id(),
            Some("workspace-2")
        );
    }

    #[test]
    fn workspace_rail_view_debug_snapshot_shows_workspace_rows_and_status() {
        let state = WorkspaceRailViewState {
            items: vec![
                WorkspaceRailItemViewModel {
                    workspace_id: "workspace-1".to_string(),
                    display_name: "alpha".to_string(),
                    root_path: "/tmp/alpha".to_string(),
                    state: WorkspaceRailState::Busy,
                    revision: 2,
                    is_selected: false,
                },
                WorkspaceRailItemViewModel {
                    workspace_id: "workspace-2".to_string(),
                    display_name: "beta".to_string(),
                    root_path: "/tmp/beta".to_string(),
                    state: WorkspaceRailState::Error,
                    revision: 4,
                    is_selected: true,
                },
            ],
            add_dialog_open: true,
            add_dialog_input_path: "/tmp".to_string(),
            add_dialog_validation_message: Some("指定したパスは有効です。".to_string()),
            context_menu_target_workspace_id: Some("workspace-2".to_string()),
        };
        let view = WorkspaceRailView::new(state, test_tokens());
        let snapshot = view.debug_snapshot();

        assert_eq!(snapshot.item_rows.len(), 2);
        assert!(snapshot.item_rows[0].contains("alpha [Busy]"));
        assert!(snapshot.item_rows[1].contains("beta [Error]"));
        assert!(snapshot.item_rows[1].contains("(selected)"));
        assert_eq!(snapshot.add_button_label, "追加");
        assert_eq!(snapshot.exclude_button_label, "除外");
        assert!(snapshot.add_dialog_open);
    }

    #[test]
    fn workspace_rail_view_notifies_add_and_exclude_actions_to_app_layer_queue() {
        let view = WorkspaceRailView::new(WorkspaceRailViewState::default(), test_tokens());
        let mut queue = WorkspaceRailActionQueue::new();

        view.notify_open_add_dialog(&mut queue);
        view.notify_submit_add_workspace(&mut queue, "/tmp/project");
        view.notify_open_exclude_menu(&mut queue, "workspace-1");
        view.notify_exclude_workspace(&mut queue, "workspace-1");
        view.notify_select_workspace(&mut queue, "workspace-1");

        assert_eq!(
            queue.drain(),
            vec![
                WorkspaceRailUiAction::RequestOpenAddDialog,
                WorkspaceRailUiAction::RequestSubmitAddWorkspace {
                    input_path: "/tmp/project".to_string()
                },
                WorkspaceRailUiAction::RequestOpenExcludeMenu {
                    workspace_id: "workspace-1".to_string()
                },
                WorkspaceRailUiAction::RequestExcludeWorkspace {
                    workspace_id: "workspace-1".to_string()
                },
                WorkspaceRailUiAction::RequestSelectWorkspace {
                    workspace_id: "workspace-1".to_string()
                },
            ]
        );
    }
}
