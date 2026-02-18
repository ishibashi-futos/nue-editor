use nue_core::{
    structure_path::{StructurePathAgentStatus, StructurePathDiffState},
    workspace_rail::WorkspaceRailState,
};

use crate::design_system::{
    AgentStatus, AgentStatusStyle, DesignSystem, DiffState, DiffStyle, DiffSurface,
    GlassStyleObject, GlassSurface, OverlayFocusState, OverlayLayerStyle, OverlayZIndexManager,
    StatusSurface,
};

/// UI が DesignSystem のスタイルを安全に取り出すためのガイド。
#[derive(Debug, Clone)]
pub struct UiStyleGuide {
    design_system: DesignSystem,
    overlay_manager: OverlayZIndexManager,
}

impl UiStyleGuide {
    /// Neon Night Glass 設定を使ったガイドを作成する。
    pub fn neon_night_glass() -> Self {
        Self {
            design_system: DesignSystem::neon_night_glass(),
            overlay_manager: OverlayZIndexManager::new(),
        }
    }

    /// Command Hub 用の Glass スタイルを解決する。
    pub fn command_hub_glass_style(&self) -> GlassStyleObject {
        self.design_system
            .glass_style_for_surface(GlassSurface::CommandHub)
    }

    /// Primary Panel 用の Glass スタイルを解決する。
    pub fn primary_panel_glass_style(&self) -> GlassStyleObject {
        self.design_system
            .glass_style_for_surface(GlassSurface::PrimaryPanel)
    }

    /// Command Hub のステータス表示用スタイル。
    pub fn command_hub_status_style(&self, status: AgentStatus) -> AgentStatusStyle {
        self.design_system
            .status_style_for(StatusSurface::CommandHub, status)
    }

    /// Workspace Rail のステータスタイプに対応するスタイル。
    pub fn workspace_rail_status_style(&self, state: WorkspaceRailState) -> AgentStatusStyle {
        self.design_system.workspace_rail_status_style_for(state)
    }

    /// Structure Path のステータス記号を DesignSystem にマップする。
    pub fn structure_path_status_style(
        &self,
        status: StructurePathAgentStatus,
    ) -> AgentStatusStyle {
        let mapped = match status {
            StructurePathAgentStatus::Busy => AgentStatus::Busy,
            StructurePathAgentStatus::Waiting => AgentStatus::Waiting,
            StructurePathAgentStatus::Error => AgentStatus::Error,
            StructurePathAgentStatus::Idle => AgentStatus::Idle,
        };
        self.design_system
            .status_style_for(StatusSurface::StructurePath, mapped)
    }

    /// Structure Path の差分状態に適した DiffStyle を返す。
    pub fn structure_path_diff_style(&self, diff_state: StructurePathDiffState) -> DiffStyle {
        let mapped = match diff_state {
            StructurePathDiffState::Pending => DiffState::Pending,
            StructurePathDiffState::Approved => DiffState::Approved,
            StructurePathDiffState::Neutral => DiffState::Neutral,
        };
        self.design_system
            .diff_style_for(DiffSurface::StructurePath, mapped)
    }

    /// Smart Gutter 用の DiffStyle を取得する。
    pub fn smart_gutter_diff_style(&self, diff_state: DiffState) -> DiffStyle {
        self.design_system
            .diff_style_for(DiffSurface::SmartGutter, diff_state)
    }

    /// オーバーレイのフォーカス状況に応じたレイヤー一覧を解決する。
    pub fn overlay_layer_styles(&self, focus_state: &OverlayFocusState) -> Vec<OverlayLayerStyle> {
        self.overlay_manager.resolve(focus_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design_system::{
        DiffState, DiffSurface, OverlayFocusState, OverlayLayer, StatusSurface,
    };

    #[test]
    fn overlay_layer_stylesはcommand_hubを最上位に置く() {
        let guide = UiStyleGuide::neon_night_glass();
        let styles = guide.overlay_layer_styles(&OverlayFocusState::default());

        assert_eq!(styles.len(), 4);
        assert_eq!(styles[0].layer, OverlayLayer::CommandHub);
        assert_eq!(styles[1].layer, OverlayLayer::SmartGutter);
        assert_eq!(styles[2].layer, OverlayLayer::Decoration);
        assert_eq!(styles[3].layer, OverlayLayer::Minimap);
    }

    #[test]
    fn structure_path_stylesはstatusとdiffを整合させる() {
        let guide = UiStyleGuide::neon_night_glass();
        let status_style = guide.structure_path_status_style(StructurePathAgentStatus::Waiting);
        assert_eq!(status_style.surface, StatusSurface::StructurePath);
        assert_eq!(status_style.status, AgentStatus::Waiting);

        let diff_style = guide.structure_path_diff_style(StructurePathDiffState::Pending);
        assert_eq!(diff_style.surface, DiffSurface::StructurePath);
        assert_eq!(diff_style.state, DiffState::Pending);
    }

    #[test]
    fn smart_gutter_diff_styleはsmart_gutter向けを返す() {
        let guide = UiStyleGuide::neon_night_glass();
        let diff_style = guide.smart_gutter_diff_style(DiffState::Approved);

        assert_eq!(diff_style.surface, DiffSurface::SmartGutter);
        assert_eq!(diff_style.state, DiffState::Approved);
    }
}
