use std::path::PathBuf;
use std::{cell::RefCell, rc::Rc};

use gpui::{
    App, Application, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    size,
};

pub mod command_hub;
pub mod design_system;
pub mod design_system_gpui;
pub mod editor_events;
pub mod editor_input;
pub mod legacy_explorer;
pub mod tab_bar;
pub mod terminal;
pub mod terminal_display;
pub mod ui_style;
pub mod workspace_rail;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UiLaunchRequest {
    workspace_root: Option<PathBuf>,
}

impl UiLaunchRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_workspace_root(mut self, workspace_root: Option<PathBuf>) -> Self {
        self.workspace_root = workspace_root;
        self
    }

    pub fn workspace_root(&self) -> Option<&std::path::Path> {
        self.workspace_root.as_deref()
    }
}

pub fn run_app(request: UiLaunchRequest) -> gpui::Result<()> {
    let design_system = design_system::DesignSystem::neon_night_glass();
    let launch_error = Rc::new(RefCell::new(None));
    let launch_error_for_callback = Rc::clone(&launch_error);
    let request_for_callback = request.clone();

    Application::new().run(move |cx: &mut App| {
        let design_tokens =
            match design_system_gpui::apply_design_system_at_startup(cx, &design_system) {
                Ok(tokens) => tokens,
                Err(error) => {
                    *launch_error_for_callback.borrow_mut() = Some(error);
                    cx.quit();
                    return;
                }
            };
        let root_view = RootPlaceholderView::from_request(&request_for_callback, design_tokens);

        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let bounds = Bounds::centered(None, size(px(960.0), px(640.0)), cx);
        let root_view = root_view.clone();
        if let Err(error) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_, cx| {
                let root_view = root_view.clone();
                cx.new(move |_| root_view)
            },
        ) {
            *launch_error_for_callback.borrow_mut() = Some(error);
            cx.quit();
            return;
        }

        cx.activate(true);
    });

    if let Some(error) = launch_error.borrow_mut().take() {
        return Err(error);
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct RootPlaceholderView {
    title: String,
    detail: String,
    styles: design_system_gpui::GpuiDesignTokens,
}

impl RootPlaceholderView {
    fn from_request(
        request: &UiLaunchRequest,
        styles: design_system_gpui::GpuiDesignTokens,
    ) -> Self {
        let detail = match request.workspace_root() {
            Some(workspace_root) => format!("workspace: {}", workspace_root.display()),
            None => "workspace: 未選択".to_string(),
        };

        Self {
            title: "Nue UI Placeholder (T-006)".to_string(),
            detail,
            styles,
        }
    }
}

impl Render for RootPlaceholderView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let palette = &self.styles.palette;
        let fonts = &self.styles.fonts;

        div().size_full().bg(palette.window_background).p_6().child(
            div()
                .size_full()
                .flex()
                .flex_col()
                .justify_center()
                .items_center()
                .gap_2()
                .bg(palette.panel_background)
                .border_1()
                .border_color(palette.panel_border)
                .child(
                    div()
                        .text_xl()
                        .font(fonts.emphasis.clone())
                        .text_color(palette.title_text)
                        .child(self.title.clone()),
                )
                .child(
                    div()
                        .text_sm()
                        .font(fonts.meta.clone())
                        .text_color(palette.body_text)
                        .child(self.detail.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .font(fonts.text.clone())
                        .text_color(palette.accent)
                        .child("DesignSystem adapter applied"),
                ),
        )
    }
}
