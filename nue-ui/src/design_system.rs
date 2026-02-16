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

    pub fn font_context(&self) -> &FontContext {
        &self.font_context
    }

    pub fn animation_for(&self, status: AgentStatus) -> AnimationSpec {
        self.animation_convention.for_status(status)
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
}
