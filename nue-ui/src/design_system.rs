pub const FONT_KEY_TEXT: &str = "nue-font::text";
pub const FONT_KEY_EMPHASIS: &str = "nue-font::emphasis";
pub const FONT_KEY_META: &str = "nue-font::meta";
pub const FONT_LICENSE_SIL_OFL_1_1: &str = "SIL Open Font License 1.1";

const FONT_FILE_REGULAR: &str = "JetBrainsMono-Regular.ttf";
const FONT_FILE_BOLD: &str = "JetBrainsMono-Bold.ttf";
const FONT_FILE_ITALIC: &str = "JetBrainsMono-Italic.ttf";
const FONT_BYTES_REGULAR: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
const FONT_BYTES_BOLD: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf");
const FONT_BYTES_ITALIC: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Italic.ttf");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Busy,
    Waiting,
    Error,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationPattern {
    Pulse,
    Blink,
    Flash,
    Fade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpacityRange {
    pub min_percent: u8,
    pub max_percent: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationSpec {
    pub pattern: AnimationPattern,
    pub cycle_ms: u16,
    pub opacity: OpacityRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NamedColor {
    pub name: &'static str,
    pub hex: &'static str,
}

const COLOR_DEEP_ABYSS: NamedColor = NamedColor {
    name: "Deep Abyss",
    hex: "#0B0E14",
};
const COLOR_ATMOSPHERE: NamedColor = NamedColor {
    name: "Atmosphere",
    hex: "#1A1D23",
};
const COLOR_SPACE_GREY: NamedColor = NamedColor {
    name: "Space Grey",
    hex: "#242933",
};
const COLOR_MIDNIGHT_GLASS: NamedColor = NamedColor {
    name: "Midnight Glass",
    hex: "#3D4455",
};
const COLOR_NEON_CYAN: NamedColor = NamedColor {
    name: "Neon Cyan",
    hex: "#00F5FF",
};
const COLOR_ELECTRIC_LIME: NamedColor = NamedColor {
    name: "Electric Lime",
    hex: "#32FF7E",
};
const COLOR_SOLAR_FLARE: NamedColor = NamedColor {
    name: "Solar Flare",
    hex: "#FFF200",
};
const COLOR_CYBER_MAGENTA: NamedColor = NamedColor {
    name: "Cyber Magenta",
    hex: "#FF006E",
};
const COLOR_ETHER_PURPLE: NamedColor = NamedColor {
    name: "Ether Purple",
    hex: "#BF5AF2",
};
const COLOR_CLOUD_WHITE: NamedColor = NamedColor {
    name: "Cloud White",
    hex: "#F8F9FA",
};
const COLOR_DUSTY_GREY: NamedColor = NamedColor {
    name: "Dusty Grey",
    hex: "#949DB1",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorPalette {
    pub deep_abyss: NamedColor,
    pub atmosphere: NamedColor,
    pub space_grey: NamedColor,
    pub midnight_glass: NamedColor,
    pub neon_cyan: NamedColor,
    pub electric_lime: NamedColor,
    pub solar_flare: NamedColor,
    pub cyber_magenta: NamedColor,
    pub ether_purple: NamedColor,
    pub cloud_white: NamedColor,
    pub dusty_grey: NamedColor,
}

impl ColorPalette {
    pub fn status_color(&self, status: AgentStatus) -> NamedColor {
        match status {
            AgentStatus::Busy => self.neon_cyan,
            AgentStatus::Waiting => self.solar_flare,
            AgentStatus::Error => self.cyber_magenta,
            AgentStatus::Idle => self.dusty_grey,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecularEdge {
    pub color: NamedColor,
    pub width_tenths_px: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlassMaterial {
    pub background: NamedColor,
    pub background_alpha_percent: u8,
    pub backdrop_blur_px: u8,
    pub fine_grain_opacity_percent: u8,
    pub specular_edge: SpecularEdge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlassSurface {
    CommandHub,
    PrimaryPanel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusSurface {
    CommandHub,
    StructurePath,
    WorkspaceRail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffSurface {
    SmartGutter,
    ApprovalUi,
    Gutter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffState {
    Pending,
    Approved,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffStyle {
    pub surface: DiffSurface,
    pub state: DiffState,
    pub color: NamedColor,
    pub emphasis_percent: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlassStyleObject {
    pub surface: GlassSurface,
    pub background: NamedColor,
    pub background_alpha_percent: u8,
    pub backdrop_blur_px: u8,
    pub fine_grain_opacity_percent: u8,
    pub specular_edge: SpecularEdge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlowSpec {
    pub intensity_percent: u8,
    pub radius_px: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionTiming {
    pub enter_ms: u16,
    pub exit_ms: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentStatusStyle {
    pub surface: StatusSurface,
    pub status: AgentStatus,
    pub primary_color: NamedColor,
    pub secondary_color: Option<NamedColor>,
    pub animation: AnimationSpec,
    pub transition: TransitionTiming,
    pub glow: GlowSpec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Regular,
    Bold,
    Italic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontDefinition {
    pub key: &'static str,
    pub font_name: &'static str,
    pub style: FontStyle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontContext {
    bindings: Vec<FontDefinition>,
    bundled_fonts: Vec<BundledFont>,
    fallback_fonts: Vec<&'static str>,
}

impl FontContext {
    pub fn from_bundled_fonts(
        bundled_fonts: Vec<BundledFont>,
        fallback_fonts: Vec<&'static str>,
    ) -> Self {
        let bindings = bundled_fonts
            .iter()
            .map(BundledFont::to_definition)
            .collect();

        Self {
            bindings,
            bundled_fonts,
            fallback_fonts,
        }
    }

    pub fn bindings(&self) -> &[FontDefinition] {
        &self.bindings
    }

    pub fn bundled_fonts(&self) -> &[BundledFont] {
        &self.bundled_fonts
    }

    pub fn fallback_fonts(&self) -> &[&'static str] {
        &self.fallback_fonts
    }

    pub fn resolve(&self, key: &str) -> Option<&FontDefinition> {
        self.bindings.iter().find(|binding| binding.key == key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationConvention {
    pub busy: AnimationSpec,
    pub waiting: AnimationSpec,
    pub error: AnimationSpec,
    pub idle: AnimationSpec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundledFont {
    pub key: &'static str,
    pub font_name: &'static str,
    pub style: FontStyle,
    pub file_name: &'static str,
    pub license: &'static str,
    pub bytes: &'static [u8],
}

impl BundledFont {
    fn to_definition(&self) -> FontDefinition {
        FontDefinition {
            key: self.key,
            font_name: self.font_name,
            style: self.style,
        }
    }
}

impl AnimationConvention {
    pub fn for_status(&self, status: AgentStatus) -> AnimationSpec {
        match status {
            AgentStatus::Busy => self.busy,
            AgentStatus::Waiting => self.waiting,
            AgentStatus::Error => self.error,
            AgentStatus::Idle => self.idle,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignSystem {
    palette: ColorPalette,
    glass_material: GlassMaterial,
    font_context: FontContext,
    animation_convention: AnimationConvention,
}

impl DesignSystem {
    pub fn neon_night_glass() -> Self {
        let palette = neon_night_palette();

        Self {
            palette,
            glass_material: neon_night_glass_material(palette),
            font_context: neon_night_font_context(),
            animation_convention: neon_night_animation_convention(),
        }
    }

    pub fn palette(&self) -> ColorPalette {
        self.palette
    }

    pub fn glass_material(&self) -> GlassMaterial {
        self.glass_material
    }

    pub fn glass_style_for_surface(&self, surface: GlassSurface) -> GlassStyleObject {
        match surface {
            GlassSurface::CommandHub => {
                glass_style_from_material(self.glass_material, surface, 70, 20)
            }
            GlassSurface::PrimaryPanel => {
                glass_style_from_material(self.glass_material, surface, 64, 16)
            }
        }
    }

    pub fn font_context(&self) -> &FontContext {
        &self.font_context
    }

    pub fn animation_for(&self, status: AgentStatus) -> AnimationSpec {
        self.animation_convention.for_status(status)
    }

    pub fn status_style_for(
        &self,
        surface: StatusSurface,
        status: AgentStatus,
    ) -> AgentStatusStyle {
        let palette = self.palette();
        let animation = self.animation_for(status);
        let primary_color = palette.status_color(status);
        let secondary_color = match status {
            AgentStatus::Idle => Some(palette.cloud_white),
            _ => None,
        };
        let transition = transition_timing_for(status);
        let glow = glow_spec_for(surface, status);

        AgentStatusStyle {
            surface,
            status,
            primary_color,
            secondary_color,
            animation,
            transition,
            glow,
        }
    }

    pub fn diff_style_for(&self, surface: DiffSurface, state: DiffState) -> DiffStyle {
        let palette = self.palette();
        let color = diff_color_for_state(palette, state);
        let emphasis_percent = diff_emphasis_for_surface(surface, state);

        DiffStyle {
            surface,
            state,
            color,
            emphasis_percent,
        }
    }
}

fn neon_night_palette() -> ColorPalette {
    ColorPalette {
        deep_abyss: COLOR_DEEP_ABYSS,
        atmosphere: COLOR_ATMOSPHERE,
        space_grey: COLOR_SPACE_GREY,
        midnight_glass: COLOR_MIDNIGHT_GLASS,
        neon_cyan: COLOR_NEON_CYAN,
        electric_lime: COLOR_ELECTRIC_LIME,
        solar_flare: COLOR_SOLAR_FLARE,
        cyber_magenta: COLOR_CYBER_MAGENTA,
        ether_purple: COLOR_ETHER_PURPLE,
        cloud_white: COLOR_CLOUD_WHITE,
        dusty_grey: COLOR_DUSTY_GREY,
    }
}

fn neon_night_glass_material(palette: ColorPalette) -> GlassMaterial {
    GlassMaterial {
        background: palette.atmosphere,
        background_alpha_percent: 70,
        backdrop_blur_px: 20,
        fine_grain_opacity_percent: 8,
        specular_edge: SpecularEdge {
            color: palette.midnight_glass,
            width_tenths_px: 5,
        },
    }
}

fn neon_night_font_context() -> FontContext {
    FontContext::from_bundled_fonts(
        vec![
            BundledFont {
                key: FONT_KEY_TEXT,
                font_name: "JetBrainsMono-Regular",
                style: FontStyle::Regular,
                file_name: FONT_FILE_REGULAR,
                license: FONT_LICENSE_SIL_OFL_1_1,
                bytes: FONT_BYTES_REGULAR,
            },
            BundledFont {
                key: FONT_KEY_EMPHASIS,
                font_name: "JetBrainsMono-Bold",
                style: FontStyle::Bold,
                file_name: FONT_FILE_BOLD,
                license: FONT_LICENSE_SIL_OFL_1_1,
                bytes: FONT_BYTES_BOLD,
            },
            BundledFont {
                key: FONT_KEY_META,
                font_name: "JetBrainsMono-Italic",
                style: FontStyle::Italic,
                file_name: FONT_FILE_ITALIC,
                license: FONT_LICENSE_SIL_OFL_1_1,
                bytes: FONT_BYTES_ITALIC,
            },
        ],
        vec!["Menlo", "Monaco", "Courier New"],
    )
}

fn neon_night_animation_convention() -> AnimationConvention {
    AnimationConvention {
        busy: AnimationSpec {
            pattern: AnimationPattern::Pulse,
            cycle_ms: 900,
            opacity: OpacityRange {
                min_percent: 40,
                max_percent: 100,
            },
        },
        waiting: AnimationSpec {
            pattern: AnimationPattern::Blink,
            cycle_ms: 1000,
            opacity: OpacityRange {
                min_percent: 25,
                max_percent: 100,
            },
        },
        error: AnimationSpec {
            pattern: AnimationPattern::Flash,
            cycle_ms: 320,
            opacity: OpacityRange {
                min_percent: 0,
                max_percent: 100,
            },
        },
        idle: AnimationSpec {
            pattern: AnimationPattern::Fade,
            cycle_ms: 2400,
            opacity: OpacityRange {
                min_percent: 55,
                max_percent: 70,
            },
        },
    }
}

fn glass_style_from_material(
    material: GlassMaterial,
    surface: GlassSurface,
    background_alpha_percent: u8,
    backdrop_blur_px: u8,
) -> GlassStyleObject {
    GlassStyleObject {
        surface,
        background: material.background,
        background_alpha_percent,
        backdrop_blur_px,
        fine_grain_opacity_percent: material.fine_grain_opacity_percent,
        specular_edge: material.specular_edge,
    }
}

fn transition_timing_for(status: AgentStatus) -> TransitionTiming {
    match status {
        AgentStatus::Busy => TransitionTiming {
            enter_ms: 140,
            exit_ms: 220,
        },
        AgentStatus::Waiting => TransitionTiming {
            enter_ms: 180,
            exit_ms: 260,
        },
        AgentStatus::Error => TransitionTiming {
            enter_ms: 80,
            exit_ms: 180,
        },
        AgentStatus::Idle => TransitionTiming {
            enter_ms: 280,
            exit_ms: 420,
        },
    }
}

fn glow_spec_for(surface: StatusSurface, status: AgentStatus) -> GlowSpec {
    let (base_intensity, base_radius) = match status {
        AgentStatus::Busy => (82_u16, 10_u8),
        AgentStatus::Waiting => (56_u16, 8_u8),
        AgentStatus::Error => (88_u16, 13_u8),
        AgentStatus::Idle => (12_u16, 4_u8),
    };
    let (surface_boost, surface_radius_boost) = match surface {
        StatusSurface::CommandHub => (8_u16, 2_u8),
        StatusSurface::StructurePath => (4_u16, 1_u8),
        StatusSurface::WorkspaceRail => (12_u16, 2_u8),
    };
    let intensity_with_surface = base_intensity + surface_boost;
    let intensity = intensity_with_surface.min(100) as u8;

    GlowSpec {
        intensity_percent: intensity,
        radius_px: base_radius + surface_radius_boost,
    }
}

fn diff_color_for_state(palette: ColorPalette, state: DiffState) -> NamedColor {
    match state {
        DiffState::Pending => palette.solar_flare,
        DiffState::Approved => palette.electric_lime,
        DiffState::Neutral => palette.cloud_white,
    }
}

fn diff_emphasis_for_surface(surface: DiffSurface, state: DiffState) -> u8 {
    match (surface, state) {
        (DiffSurface::SmartGutter, DiffState::Pending) => 80,
        (DiffSurface::SmartGutter, DiffState::Approved) => 76,
        (DiffSurface::SmartGutter, DiffState::Neutral) => 62,
        (DiffSurface::ApprovalUi, DiffState::Pending) => 84,
        (DiffSurface::ApprovalUi, DiffState::Approved) => 78,
        (DiffSurface::ApprovalUi, DiffState::Neutral) => 64,
        (DiffSurface::Gutter, DiffState::Pending) => 72,
        (DiffSurface::Gutter, DiffState::Approved) => 68,
        (DiffSurface::Gutter, DiffState::Neutral) => 58,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neon_night_glassのカラーパレットとglassマテリアルを参照できる() {
        let design_system = DesignSystem::neon_night_glass();
        let palette = design_system.palette();
        let glass = design_system.glass_material();

        assert_eq!(palette.deep_abyss.hex, "#0B0E14");
        assert_eq!(palette.atmosphere.hex, "#1A1D23");
        assert_eq!(palette.neon_cyan.hex, "#00F5FF");
        assert_eq!(palette.solar_flare.hex, "#FFF200");
        assert_eq!(palette.ether_purple.hex, "#BF5AF2");
        assert_eq!(palette.cloud_white.hex, "#F8F9FA");
        assert_eq!(palette.dusty_grey.hex, "#949DB1");
        assert_eq!(glass.background, palette.atmosphere);
        assert_eq!(glass.background_alpha_percent, 70);
        assert_eq!(glass.backdrop_blur_px, 20);
        assert_eq!(glass.specular_edge.color, palette.midnight_glass);
        assert_eq!(glass.specular_edge.width_tenths_px, 5);
    }

    #[test]
    fn fontcontextは必須キーとフォールバック順序を持つ() {
        let design_system = DesignSystem::neon_night_glass();
        let font_context = design_system.font_context();

        assert_eq!(font_context.bindings().len(), 3);
        assert_eq!(
            font_context.resolve(FONT_KEY_TEXT),
            Some(&FontDefinition {
                key: FONT_KEY_TEXT,
                font_name: "JetBrainsMono-Regular",
                style: FontStyle::Regular,
            })
        );
        assert_eq!(
            font_context.resolve(FONT_KEY_EMPHASIS),
            Some(&FontDefinition {
                key: FONT_KEY_EMPHASIS,
                font_name: "JetBrainsMono-Bold",
                style: FontStyle::Bold,
            })
        );
        assert_eq!(
            font_context.resolve(FONT_KEY_META),
            Some(&FontDefinition {
                key: FONT_KEY_META,
                font_name: "JetBrainsMono-Italic",
                style: FontStyle::Italic,
            })
        );
        assert_eq!(
            font_context.fallback_fonts(),
            &["Menlo", "Monaco", "Courier New"]
        );
    }

    #[test]
    fn fontcontextはjetbrainsmono埋め込みカタログを持つ() {
        let design_system = DesignSystem::neon_night_glass();
        let font_context = design_system.font_context();
        let catalog = font_context.bundled_fonts();

        assert_eq!(catalog.len(), 3);
        assert_eq!(catalog[0].file_name, "JetBrainsMono-Regular.ttf");
        assert_eq!(catalog[1].file_name, "JetBrainsMono-Bold.ttf");
        assert_eq!(catalog[2].file_name, "JetBrainsMono-Italic.ttf");
        assert!(
            catalog
                .iter()
                .all(|font| font.license == "SIL Open Font License 1.1")
        );
        assert!(catalog.iter().all(|font| !font.bytes.is_empty()));
    }

    #[test]
    fn ステータスごとに色とアニメーション規約を参照できる() {
        let design_system = DesignSystem::neon_night_glass();
        let palette = design_system.palette();

        assert_eq!(
            palette.status_color(AgentStatus::Busy),
            NamedColor {
                name: "Neon Cyan",
                hex: "#00F5FF",
            }
        );
        assert_eq!(
            design_system.animation_for(AgentStatus::Busy),
            AnimationSpec {
                pattern: AnimationPattern::Pulse,
                cycle_ms: 900,
                opacity: OpacityRange {
                    min_percent: 40,
                    max_percent: 100,
                },
            }
        );
        assert_eq!(
            design_system.animation_for(AgentStatus::Waiting),
            AnimationSpec {
                pattern: AnimationPattern::Blink,
                cycle_ms: 1000,
                opacity: OpacityRange {
                    min_percent: 25,
                    max_percent: 100,
                },
            }
        );
        assert_eq!(
            design_system.animation_for(AgentStatus::Error),
            AnimationSpec {
                pattern: AnimationPattern::Flash,
                cycle_ms: 320,
                opacity: OpacityRange {
                    min_percent: 0,
                    max_percent: 100,
                },
            }
        );
        assert_eq!(
            design_system.animation_for(AgentStatus::Idle),
            AnimationSpec {
                pattern: AnimationPattern::Fade,
                cycle_ms: 2400,
                opacity: OpacityRange {
                    min_percent: 55,
                    max_percent: 70,
                },
            }
        );
    }

    #[test]
    fn glass共通スタイルはcommand_hub向けプリセットを返す() {
        let design_system = DesignSystem::neon_night_glass();
        let style = design_system.glass_style_for_surface(GlassSurface::CommandHub);

        assert_eq!(style.surface, GlassSurface::CommandHub);
        assert_eq!(style.backdrop_blur_px, 20);
        assert_eq!(style.fine_grain_opacity_percent, 8);
        assert_eq!(style.specular_edge.width_tenths_px, 5);
        assert_eq!(style.background_alpha_percent, 70);
    }

    #[test]
    fn glass共通スタイルは主要パネル向けプリセットを返す() {
        let design_system = DesignSystem::neon_night_glass();
        let style = design_system.glass_style_for_surface(GlassSurface::PrimaryPanel);

        assert_eq!(style.surface, GlassSurface::PrimaryPanel);
        assert_eq!(style.backdrop_blur_px, 16);
        assert_eq!(style.fine_grain_opacity_percent, 8);
        assert_eq!(style.specular_edge.width_tenths_px, 5);
        assert_eq!(style.background_alpha_percent, 64);
    }

    #[test]
    fn agent_status_styleはcommand_hub向けbusy規約を返す() {
        let design_system = DesignSystem::neon_night_glass();
        let style = design_system.status_style_for(StatusSurface::CommandHub, AgentStatus::Busy);

        assert_eq!(style.surface, StatusSurface::CommandHub);
        assert_eq!(style.status, AgentStatus::Busy);
        assert_eq!(style.primary_color.hex, "#00F5FF");
        assert_eq!(style.secondary_color, None);
        assert_eq!(style.animation.pattern, AnimationPattern::Pulse);
        assert_eq!(
            style.transition,
            TransitionTiming {
                enter_ms: 140,
                exit_ms: 220,
            }
        );
        assert_eq!(
            style.glow,
            GlowSpec {
                intensity_percent: 90,
                radius_px: 12,
            }
        );
    }

    #[test]
    fn agent_status_styleはstructure_path向けidleデュアルトーンを返す() {
        let design_system = DesignSystem::neon_night_glass();
        let style = design_system.status_style_for(StatusSurface::StructurePath, AgentStatus::Idle);

        assert_eq!(style.surface, StatusSurface::StructurePath);
        assert_eq!(style.status, AgentStatus::Idle);
        assert_eq!(style.primary_color.hex, "#949DB1");
        assert_eq!(
            style.secondary_color.map(|color| color.hex),
            Some("#F8F9FA")
        );
        assert_eq!(style.animation.pattern, AnimationPattern::Fade);
        assert_eq!(
            style.transition,
            TransitionTiming {
                enter_ms: 280,
                exit_ms: 420,
            }
        );
        assert_eq!(
            style.glow,
            GlowSpec {
                intensity_percent: 16,
                radius_px: 5,
            }
        );
    }

    #[test]
    fn agent_status_styleはworkspace_rail向けerrorグローを強める() {
        let design_system = DesignSystem::neon_night_glass();
        let style =
            design_system.status_style_for(StatusSurface::WorkspaceRail, AgentStatus::Error);

        assert_eq!(style.surface, StatusSurface::WorkspaceRail);
        assert_eq!(style.status, AgentStatus::Error);
        assert_eq!(style.primary_color.hex, "#FF006E");
        assert_eq!(style.secondary_color, None);
        assert_eq!(style.animation.pattern, AnimationPattern::Flash);
        assert_eq!(
            style.transition,
            TransitionTiming {
                enter_ms: 80,
                exit_ms: 180,
            }
        );
        assert_eq!(
            style.glow,
            GlowSpec {
                intensity_percent: 100,
                radius_px: 15,
            }
        );
    }

    #[test]
    fn diff_styleはsmart_gutter向け未承認スタイルを返す() {
        let design_system = DesignSystem::neon_night_glass();
        let style = design_system.diff_style_for(DiffSurface::SmartGutter, DiffState::Pending);

        assert_eq!(style.surface, DiffSurface::SmartGutter);
        assert_eq!(style.state, DiffState::Pending);
        assert_eq!(style.color.hex, "#FFF200");
        assert_eq!(style.emphasis_percent, 80);
    }

    #[test]
    fn diff_styleはapproval_ui向け承認済みスタイルを返す() {
        let design_system = DesignSystem::neon_night_glass();
        let style = design_system.diff_style_for(DiffSurface::ApprovalUi, DiffState::Approved);

        assert_eq!(style.surface, DiffSurface::ApprovalUi);
        assert_eq!(style.state, DiffState::Approved);
        assert_eq!(style.color.hex, "#32FF7E");
        assert_eq!(style.emphasis_percent, 78);
    }

    #[test]
    fn diff_styleはgutter向け中立スタイルを返す() {
        let design_system = DesignSystem::neon_night_glass();
        let style = design_system.diff_style_for(DiffSurface::Gutter, DiffState::Neutral);

        assert_eq!(style.surface, DiffSurface::Gutter);
        assert_eq!(style.state, DiffState::Neutral);
        assert_eq!(style.color.hex, "#F8F9FA");
        assert_eq!(style.emphasis_percent, 58);
    }
}
