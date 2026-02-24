use std::path::PathBuf;

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
    // T-005 で GPUI のウィンドウ生成と初回描画を接続する。
    let _application = gpui::Application::new();
    let _design_system = design_system::DesignSystem::neon_night_glass();
    let _workspace_root = request.workspace_root();
    Ok(())
}
