use gpui::{Context, Window, div, prelude::*, px};

use crate::design_system_gpui::GpuiDesignTokens;

/// 左パネルの種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceLeftPanelKind {
    Explorer,
    Search,
    Vcs,
}

impl WorkspaceLeftPanelKind {
    fn label(self) -> &'static str {
        match self {
            Self::Explorer => "Explorer",
            Self::Search => "Search",
            Self::Vcs => "VCS",
        }
    }
}

/// オーバーレイ種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceOverlayKind {
    None,
    CommandHub,
}

impl WorkspaceOverlayKind {
    fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::CommandHub => "Command Hub",
        }
    }
}

/// ワークスペースセッション画面の描画入力。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSessionLayoutState {
    session_title: String,
    workspace_label: String,
    left_panel: WorkspaceLeftPanelKind,
    overlay: WorkspaceOverlayKind,
    editor_label: String,
    terminal_label: String,
}

impl WorkspaceSessionLayoutState {
    pub fn new(
        session_title: impl Into<String>,
        workspace_label: impl Into<String>,
        left_panel: WorkspaceLeftPanelKind,
        overlay: WorkspaceOverlayKind,
    ) -> Self {
        Self {
            session_title: session_title.into(),
            workspace_label: workspace_label.into(),
            left_panel,
            overlay,
            editor_label: "Editor".to_string(),
            terminal_label: "Terminal".to_string(),
        }
    }

    pub fn with_editor_label(mut self, label: impl Into<String>) -> Self {
        self.editor_label = label.into();
        self
    }

    pub fn with_terminal_label(mut self, label: impl Into<String>) -> Self {
        self.terminal_label = label.into();
        self
    }

    pub fn session_title(&self) -> &str {
        &self.session_title
    }

    pub fn workspace_label(&self) -> &str {
        &self.workspace_label
    }

    pub fn left_panel(&self) -> WorkspaceLeftPanelKind {
        self.left_panel
    }

    pub fn overlay(&self) -> WorkspaceOverlayKind {
        self.overlay
    }

    pub fn editor_label(&self) -> &str {
        &self.editor_label
    }

    pub fn terminal_label(&self) -> &str {
        &self.terminal_label
    }
}

impl Default for WorkspaceSessionLayoutState {
    fn default() -> Self {
        Self::new(
            "Session",
            "workspace: 未設定",
            WorkspaceLeftPanelKind::Explorer,
            WorkspaceOverlayKind::None,
        )
    }
}

/// 基本レイアウトの固定寸法。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceSessionLayoutSpec {
    pub rail_width_px: u16,
    pub left_panel_width_px: u16,
    pub terminal_height_px: u16,
    pub overlay_width_px: u16,
    pub overlay_height_px: u16,
    pub overlay_margin_px: u16,
}

impl Default for WorkspaceSessionLayoutSpec {
    fn default() -> Self {
        Self {
            rail_width_px: 76,
            left_panel_width_px: 280,
            terminal_height_px: 180,
            overlay_width_px: 360,
            overlay_height_px: 120,
            overlay_margin_px: 12,
        }
    }
}

/// レイアウト関係の安定性確認に使う簡易スナップショット。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSessionLayoutDebugSnapshot {
    pub rail_title: String,
    pub left_panel_title: String,
    pub editor_title: String,
    pub terminal_title: String,
    pub overlay_title: String,
    pub spec: WorkspaceSessionLayoutSpec,
}

/// ワークスペースセッション画面の基本枠描画。
#[derive(Clone, Debug)]
pub struct WorkspaceSessionLayoutView {
    state: WorkspaceSessionLayoutState,
    styles: GpuiDesignTokens,
    spec: WorkspaceSessionLayoutSpec,
}

impl WorkspaceSessionLayoutView {
    pub fn new(state: WorkspaceSessionLayoutState, styles: GpuiDesignTokens) -> Self {
        Self {
            state,
            styles,
            spec: WorkspaceSessionLayoutSpec::default(),
        }
    }

    pub fn with_spec(mut self, spec: WorkspaceSessionLayoutSpec) -> Self {
        self.spec = spec;
        self
    }

    pub fn state(&self) -> &WorkspaceSessionLayoutState {
        &self.state
    }

    pub fn spec(&self) -> WorkspaceSessionLayoutSpec {
        self.spec
    }

    pub fn debug_snapshot(&self) -> WorkspaceSessionLayoutDebugSnapshot {
        WorkspaceSessionLayoutDebugSnapshot {
            rail_title: "Rail".to_string(),
            left_panel_title: format!("Left Panel ({})", self.state.left_panel().label()),
            editor_title: self.state.editor_label().to_string(),
            terminal_title: self.state.terminal_label().to_string(),
            overlay_title: format!("Overlay ({})", self.state.overlay().label()),
            spec: self.spec,
        }
    }

    fn frame(
        &self,
        title: impl Into<String>,
        detail: impl Into<String>,
        accent: bool,
    ) -> gpui::Div {
        let palette = &self.styles.palette;
        let fonts = &self.styles.fonts;
        let border_color = if accent {
            palette.accent
        } else {
            palette.panel_border
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .justify_between()
            .p_2()
            .bg(palette.panel_background)
            .border_1()
            .border_color(border_color)
            .child(
                div()
                    .text_sm()
                    .font(fonts.emphasis.clone())
                    .text_color(palette.title_text)
                    .truncate()
                    .child(title.into()),
            )
            .child(
                div()
                    .text_xs()
                    .font(fonts.meta.clone())
                    .text_color(palette.body_text)
                    .truncate()
                    .child(detail.into()),
            )
    }
}

impl Render for WorkspaceSessionLayoutView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let palette = &self.styles.palette;
        let rail_w = px(self.spec.rail_width_px as f32);
        let left_w = px(self.spec.left_panel_width_px as f32);
        let terminal_h = px(self.spec.terminal_height_px as f32);
        let overlay_w = px(self.spec.overlay_width_px as f32);
        let overlay_h = px(self.spec.overlay_height_px as f32);
        let overlay_margin = px(self.spec.overlay_margin_px as f32);

        let rail = self.frame("Rail", self.state.workspace_label(), false);
        let left_panel = self.frame(
            format!("Left Panel ({})", self.state.left_panel().label()),
            self.state.session_title(),
            false,
        );
        let editor = self.frame(self.state.editor_label(), "pane / tabs placeholder", true);
        let terminal = self.frame(
            self.state.terminal_label(),
            "run_command / scrollback placeholder",
            false,
        );
        let overlay = self.frame(
            format!("Overlay ({})", self.state.overlay().label()),
            "command hub / modal placeholder",
            self.state.overlay() != WorkspaceOverlayKind::None,
        );

        div()
            .size_full()
            .bg(palette.window_background)
            .p_2()
            .child(
                div()
                    .size_full()
                    .flex()
                    .gap_2()
                    .child(div().w(rail_w).flex_none().child(rail))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .flex()
                            .gap_2()
                            .child(div().w(left_w).flex_none().child(left_panel))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.0))
                                    .relative()
                                    .child(
                                        div()
                                            .size_full()
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .child(div().flex_1().min_h(px(0.0)).child(editor))
                                            .child(div().h(terminal_h).flex_none().child(terminal)),
                                    )
                                    .child(
                                        div()
                                            .absolute()
                                            .top(overlay_margin)
                                            .right(overlay_margin)
                                            .w(overlay_w)
                                            .h(overlay_h)
                                            .child(overlay),
                                    ),
                            ),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design_system::DesignSystem;
    use crate::design_system_gpui::GpuiDesignTokens;

    fn test_tokens() -> GpuiDesignTokens {
        GpuiDesignTokens::from_design_system(&DesignSystem::neon_night_glass())
    }

    #[test]
    fn セッション状態を渡すと基本レイアウト情報を保持できる() {
        let state = WorkspaceSessionLayoutState::new(
            "session-1",
            "/tmp/workspace-a",
            WorkspaceLeftPanelKind::Explorer,
            WorkspaceOverlayKind::CommandHub,
        )
        .with_editor_label("Editor (main.rs)")
        .with_terminal_label("Terminal (1)");
        let view = WorkspaceSessionLayoutView::new(state.clone(), test_tokens());
        let snapshot = view.debug_snapshot();

        assert_eq!(view.state(), &state);
        assert_eq!(snapshot.rail_title, "Rail");
        assert_eq!(snapshot.left_panel_title, "Left Panel (Explorer)");
        assert_eq!(snapshot.editor_title, "Editor (main.rs)");
        assert_eq!(snapshot.terminal_title, "Terminal (1)");
        assert_eq!(snapshot.overlay_title, "Overlay (Command Hub)");
    }

    #[test]
    fn 基本レイアウトの位置関係に使う寸法が固定である() {
        let spec = WorkspaceSessionLayoutSpec::default();

        assert_eq!(spec.rail_width_px, 76);
        assert_eq!(spec.left_panel_width_px, 280);
        assert_eq!(spec.terminal_height_px, 180);
        assert!(spec.left_panel_width_px > spec.rail_width_px);
        assert!(spec.overlay_width_px > spec.overlay_height_px);
        assert!(spec.overlay_margin_px > 0);
    }
}
