//! Semantic design tokens shared by every CrownOS surface.
//!
//! The palettes themselves ship with the toolkits; this section only holds
//! sparse edits to them, keyed by dotted slot names such as `surface.bg`, plus
//! the shape and type tokens a theme can retune. A slot left out keeps the
//! toolkit's built-in color, so an empty file is the stock look.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A custom accent gradient, as `#rrggbb` or `#rrggbbaa` strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccentGradient {
    /// The lighter stop, painted toward the top or left of an accent surface.
    pub start: String,
    /// The deeper stop.
    pub end: String,
}

/// Corner radii in logical pixels, from small controls to whole panels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Radii {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

impl Default for Radii {
    fn default() -> Self {
        Self {
            sm: 6.0,
            md: 10.0,
            lg: 16.0,
            xl: 24.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Font {
    pub family: String,
    pub mono: String,
    /// Multiplies every text size.
    pub scale: f32,
}

impl Default for Font {
    fn default() -> Self {
        Self {
            family: "Inter".to_owned(),
            mono: "JetBrains Mono".to_owned(),
            scale: 1.0,
        }
    }
}

crate::section! {
    pub struct Theme in "theme", keys ThemeKey {
        /// Replaces the accent chosen in `appearance.ron`.
        pub accent as Accent: Option<AccentGradient> = None,
        /// Slot edits applied only to the light palette.
        pub light as Light: BTreeMap<String, String> = BTreeMap::new(),
        /// Slot edits applied only to the dark palette.
        pub dark as Dark: BTreeMap<String, String> = BTreeMap::new(),
        pub radii as RadiiKey: Radii = Radii::default(),
        pub font as FontKey: Font = Font::default(),
    }
}

/// Every color token a theme may set or a plugin may paint with.
pub const COLOR_TOKENS: &[&str] = &[
    "accent.start",
    "accent.end",
    "surface.bg",
    "surface.raised",
    "surface.sunken",
    "surface.border",
    "surface.hairline",
    "surface.hover",
    "surface.shadow",
    "text.primary",
    "text.body",
    "text.muted",
    "text.disabled",
    "text.on_accent",
    "text.icon",
    "text.icon_muted",
    "control.track",
    "control.knob",
    "control.knob_shadow",
    "status.success",
    "status.warning",
    "status.danger",
    "status.neutral",
    "status.danger_bg",
    "status.on_danger",
    "popover.bg",
    "popover.border",
    "popover.text",
    "popover.muted_text",
    "popover.hover_bg",
    "popover.trigger_bg",
    "radio.unselected_bg",
    "radio.unselected_border",
    "radio.hole",
    "radio.text",
    "menu.bg",
    "menu.border",
    "menu.text",
    "menu.shortcut_text",
    "menu.disabled_text",
    "menu.header_text",
    "menu.separator",
    "menu.selected_bg",
    "menu.selected_text",
    "sidebar.bg",
    "sidebar.separator",
    "sidebar.text",
    "sidebar.selected_text",
    "sidebar.subitem_text",
    "sidebar.brand_text",
    "sidebar.icon",
    "sidebar.selected_icon",
    "sidebar.hover_bg",
    "sidebar.selected_bg",
    "sidebar.selected_border",
    "toggle_off.start",
    "toggle_off.end",
    "frame.tint_top",
    "frame.tint_bottom",
    "frame.highlight",
    "frame.control_fill",
    "frame.control_glyph",
    "frame.border",
    "frame.label",
    "bar.fg",
    "bar.fg_hover",
];

pub fn is_color_token(name: &str) -> bool {
    COLOR_TOKENS.contains(&name)
}

/// `#rrggbb` or `#rrggbbaa` as straight RGBA in `0.0..=1.0`.
pub fn parse_hex(text: &str) -> Option<[f32; 4]> {
    let digits = text.strip_prefix('#').unwrap_or(text);
    let value = u32::from_str_radix(digits, 16).ok()?;
    let channel = |shift: u32| ((value >> shift) & 0xff) as f32 / 255.0;
    match digits.len() {
        6 => Some([channel(16), channel(8), channel(0), 1.0]),
        8 => Some([channel(24), channel(16), channel(8), channel(0)]),
        _ => None,
    }
}

impl Theme {
    /// The overrides for the active mode.
    pub fn slots(&self, dark_mode: bool) -> &BTreeMap<String, String> {
        if dark_mode { &self.dark } else { &self.light }
    }

    /// An overridden token for the active mode, parsed.
    pub fn color(&self, dark_mode: bool, token: &str) -> Option<[f32; 4]> {
        self.slots(dark_mode).get(token).and_then(|hex| parse_hex(hex))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pre_tokens_theme_file_still_parses() {
        let theme: Theme = crate::parser::options()
            .from_str(r##"(accent: Some((start: "#8f6dfb", end: "#6d48e8")), dark: {"surface.bg": "#101010"})"##)
            .expect("legacy shape");
        assert_eq!(theme.dark["surface.bg"], "#101010");
        assert_eq!(theme.radii, Radii::default());
        assert_eq!(theme.accent.map(|accent| accent.end).as_deref(), Some("#6d48e8"));
    }

    #[test]
    fn hex_parses_both_widths() {
        assert_eq!(parse_hex("#ff0000"), Some([1.0, 0.0, 0.0, 1.0]));
        assert_eq!(parse_hex("00ff0080").map(|c| c[3]), Some(128.0 / 255.0));
        assert_eq!(parse_hex("#12345"), None);
    }

    #[test]
    fn color_reads_the_active_mode() {
        let mut theme = Theme::default();
        theme.light.insert("bar.fg".into(), "#000000".into());
        assert_eq!(theme.color(false, "bar.fg"), Some([0.0, 0.0, 0.0, 1.0]));
        assert_eq!(theme.color(true, "bar.fg"), None);
    }
}
