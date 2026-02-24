use std::borrow::Cow;

use gpui::{App, Font, FontFallbacks, Rgba, font, rgb};

use crate::design::system::{DesignSystem, FONT_KEY_EMPHASIS, FONT_KEY_META, FONT_KEY_TEXT};

#[derive(Debug, Clone, PartialEq)]
pub struct GpuiDesignTokens {
    pub palette: GpuiPalette,
    pub fonts: GpuiFontSet,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpuiPalette {
    pub window_background: Rgba,
    pub panel_background: Rgba,
    pub panel_border: Rgba,
    pub title_text: Rgba,
    pub body_text: Rgba,
    pub accent: Rgba,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpuiFontSet {
    pub text: Font,
    pub emphasis: Font,
    pub meta: Font,
}

pub fn apply_design_system_at_startup(
    cx: &mut App,
    design_system: &DesignSystem,
) -> gpui::Result<GpuiDesignTokens> {
    register_bundled_fonts(cx, design_system)?;
    Ok(GpuiDesignTokens::from_design_system(design_system))
}

fn register_bundled_fonts(cx: &mut App, design_system: &DesignSystem) -> gpui::Result<()> {
    let fonts = design_system
        .font_context()
        .bundled_fonts()
        .iter()
        .map(|font| Cow::Borrowed(font.bytes))
        .collect();

    cx.text_system().add_fonts(fonts)
}

impl GpuiDesignTokens {
    pub fn from_design_system(design_system: &DesignSystem) -> Self {
        let palette = design_system.palette();
        let fallback_fonts = FontFallbacks::from_fonts(
            design_system
                .font_context()
                .fallback_fonts()
                .iter()
                .map(|font_name| (*font_name).to_string())
                .collect(),
        );

        let text_font_name = design_system
            .font_context()
            .resolve(FONT_KEY_TEXT)
            .map(|font| font.font_name)
            .expect("DesignSystem に text フォント定義が必要です");
        let emphasis_font_name = design_system
            .font_context()
            .resolve(FONT_KEY_EMPHASIS)
            .map(|font| font.font_name)
            .expect("DesignSystem に emphasis フォント定義が必要です");
        let meta_font_name = design_system
            .font_context()
            .resolve(FONT_KEY_META)
            .map(|font| font.font_name)
            .expect("DesignSystem に meta フォント定義が必要です");

        Self {
            palette: GpuiPalette {
                window_background: rgb(hex_rgb(palette.deep_abyss.hex)),
                panel_background: rgb(hex_rgb(palette.atmosphere.hex)),
                panel_border: rgb(hex_rgb(palette.midnight_glass.hex)),
                title_text: rgb(hex_rgb(palette.cloud_white.hex)),
                body_text: rgb(hex_rgb(palette.dusty_grey.hex)),
                accent: rgb(hex_rgb(palette.neon_cyan.hex)),
            },
            fonts: GpuiFontSet {
                text: font_with_fallbacks(font(text_font_name), fallback_fonts.clone()),
                emphasis: font_with_fallbacks(
                    font(emphasis_font_name).bold(),
                    fallback_fonts.clone(),
                ),
                meta: font_with_fallbacks(font(meta_font_name).italic(), fallback_fonts),
            },
        }
    }
}

fn font_with_fallbacks(mut font: Font, fallbacks: FontFallbacks) -> Font {
    font.fallbacks = Some(fallbacks);
    font
}

fn hex_rgb(hex: &str) -> u32 {
    let Some(stripped) = hex.strip_prefix('#') else {
        panic!("カラーコードは #RRGGBB 形式である必要があります: {hex}");
    };
    assert!(
        stripped.len() == 6,
        "カラーコードは #RRGGBB 形式である必要があります: {hex}"
    );
    u32::from_str_radix(stripped, 16).expect("カラーコードの16進数変換に失敗しました")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design::system::DesignSystem;

    #[test]
    fn gpuiトークンへデザインシステムを変換できる() {
        let tokens = GpuiDesignTokens::from_design_system(&DesignSystem::neon_night_glass());

        assert_eq!(u32::from(tokens.palette.window_background), 0x0B0E14FF);
        assert_eq!(u32::from(tokens.palette.panel_background), 0x1A1D23FF);
        assert_eq!(u32::from(tokens.palette.accent), 0x00F5FFFF);
        assert_eq!(tokens.fonts.text.family.as_ref(), "JetBrainsMono-Regular");
        assert_eq!(tokens.fonts.emphasis.family.as_ref(), "JetBrainsMono-Bold");
        assert_eq!(tokens.fonts.meta.family.as_ref(), "JetBrainsMono-Italic");
        assert_eq!(
            tokens
                .fonts
                .text
                .fallbacks
                .as_ref()
                .expect("フォールバックが必要")
                .fallback_list(),
            &[
                "Hiragino Sans".to_string(),
                "Yu Gothic UI".to_string(),
                "Meiryo".to_string(),
                "Noto Sans CJK JP".to_string(),
                "Noto Sans JP".to_string(),
                "Menlo".to_string(),
                "Monaco".to_string(),
                "Courier New".to_string()
            ]
        );
    }
}
