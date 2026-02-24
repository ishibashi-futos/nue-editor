use std::path::PathBuf;
use std::{cell::RefCell, rc::Rc};

use gpui::{
    App, Application, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};

pub mod command_hub;
pub mod design_system;
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
    let root_view = RootPlaceholderView::from_request(&request);
    let _design_system = design_system::DesignSystem::neon_night_glass();
    let launch_error = Rc::new(RefCell::new(None));
    let launch_error_for_callback = Rc::clone(&launch_error);

    Application::new().run(move |cx: &mut App| {
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
}

impl RootPlaceholderView {
    fn from_request(request: &UiLaunchRequest) -> Self {
        let detail = match request.workspace_root() {
            Some(workspace_root) => format!("workspace: {}", workspace_root.display()),
            None => "workspace: 未選択".to_string(),
        };

        Self {
            title: "Nue UI Placeholder (T-005)".to_string(),
            detail,
        }
    }
}

impl Render for RootPlaceholderView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0x111827))
            .p_6()
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .items_center()
                    .gap_2()
                    .bg(rgb(0x1f2937))
                    .border_1()
                    .border_color(rgb(0x4b5563))
                    .text_color(rgb(0xe5e7eb))
                    .child(div().text_xl().child(self.title.clone()))
                    .child(div().text_sm().child(self.detail.clone())),
            )
    }
}
