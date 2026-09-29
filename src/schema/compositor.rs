//! Window management: tiling, keybindings, window rules, outputs.
//!
//! How windows are *arranged*, not how they look — gaps, borders and animation
//! are in [`appearance`](crate::schema::appearance), because a user changing them
//! is changing the theme rather than the tiling.
//!
//! The consumer is [crownpositor], which reads this file live — a rebind takes
//! effect without a restart. It keeps its own compiled form of these values
//! (regexes, chords, geometry in signed pixels); this is only the on-disk
//! vocabulary, so the file stays a stable contract while the compositor's
//! internal types move around.
//!
//! [crownpositor]: https://github.com/crown-os/crownpositor

use serde::{Deserialize, Serialize};

/// How a workspace arranges the windows on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WorkspaceMode {
    /// The compositor owns every window's geometry.
    #[default]
    Tiling,
    /// Windows keep the size and position they were given, and wear a titlebar.
    Floating,
}

impl WorkspaceMode {
    pub fn is_floating(self) -> bool {
        matches!(self, Self::Floating)
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::Tiling => Self::Floating,
            Self::Floating => Self::Tiling,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OutputTransform {
    #[default]
    Normal,
    R90,
    R180,
    R270,
    Flipped,
    Flipped90,
    Flipped180,
    Flipped270,
}

/// One row of the keybinding table. Both fields are strings so the file stays
/// hand-editable; a bad row is logged and skipped rather than failing the load.
///
/// Deliberately not a [`Keybind`](crate::Keybind): the compositor's actions are
/// its own vocabulary, and the chord spellings it accepts (`"Super+Shift+Q"`)
/// are a superset of what the settings panel's single-shortcut fields need.
/// Matched at a window's first buffer commit, the earliest point `app_id`,
/// `title` and the size hints exist.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowRule {
    /// Regex, unanchored: `"blender"` matches `"org.blender.Blender"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    /// Regex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floating: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fullscreen: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximized: Option<bool>,
    /// Zero-based workspace index on the target output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<u16>,
    /// Connector name or `"MAKE MODEL SERIAL"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// `false` opens the window without stealing focus.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<u16>,
    /// Present this window's frames without waiting for vblank while it is
    /// fullscreen. For X11 games, which cannot ask through
    /// `wp_tearing_control_v1`. Needs [`GamingOptions::allow_tearing`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tearing: Option<bool>,
    /// Treat this window as a game for [`Vrr::OnDemand`], fullscreen or not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vrr: Option<bool>,
}

/// Latency trade-offs a desktop would not make by default.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GamingOptions {
    /// Let a fullscreen window that asks for it tear instead of waiting for
    /// vblank.
    pub allow_tearing: bool,
}

/// When a display runs at a variable refresh rate.
///
/// Tri-state rather than a bool because "always on" and "on when it helps" are
/// genuinely different: some panels flicker at low refresh on a static
/// desktop, so `OnDemand` turns it on only while a fullscreen window is the
/// only thing on the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Vrr {
    #[default]
    Off,
    On,
    OnDemand,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputSetting {
    /// Connector name (`"eDP-1"`) or `"MAKE MODEL SERIAL"`.
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// `"2560x1440@144.000"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<OutputTransform>,
    /// Fallback position, used when no [`OutputLayout`] matches the monitors
    /// that are actually plugged in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<(i32, i32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vrr: Option<Vrr>,
    /// Show this output's contents on another one. Names it the same way
    /// [`Self::name`] does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_of: Option<String>,
    /// DRM `max bpc`. Set 8 for a panel that flickers at 10 or 12.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bpc: Option<u8>,
    /// Drive the panel in BT.2100. Refused when its EDID claims no HDR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hdr: Option<bool>,
    /// cd/m² that SDR white maps to in HDR mode. BT.2408 says 203.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdr_reference_luminance: Option<u32>,
    /// Path to this output's ICC calibration profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icc_profile: Option<String>,
    /// Take focus at startup instead of the first output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_at_startup: Option<bool>,
    /// Overrides the global default for workspaces created on this output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<WorkspaceMode>,
}

/// Where the monitors sit, remembered per *set* of monitors.
///
/// Position is the one setting that depends on which other displays are
/// present — a laptop docked to two screens wants a different arrangement from
/// the same laptop on its own, while its scale and mode do not change. So
/// positions live here, keyed by the set, and everything else stays in
/// [`OutputSetting`] where it can be hand-edited once.
///
/// A `Vec` rather than a map keyed by the set, because RON cannot express a
/// map with a composite key readably, and these files are meant to be edited
/// by hand.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputLayout {
    /// The outputs present, named as in [`OutputSetting::name`]. Matched as a
    /// set, so order does not matter.
    pub heads: Vec<String>,
    /// `(name, x, y)` for each output enabled in this arrangement.
    pub positions: Vec<(String, i32, i32)>,
    /// Outputs switched off in this arrangement.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub disabled: Vec<String>,
}

impl OutputLayout {
    /// Whether this arrangement is the one for the monitors now plugged in.
    pub fn matches(&self, present: &[String]) -> bool {
        self.heads.len() == present.len()
            && self.heads.iter().all(|head| present.contains(head))
    }

    pub fn position_of(&self, name: &str) -> Option<(i32, i32)> {
        self.positions
            .iter()
            .find_map(|(head, x, y)| (head == name).then_some((*x, *y)))
    }
}

crate::section! {
    pub struct Compositor in "compositor", keys CompositorKey {
        pub layout as Layout: WorkspaceMode = WorkspaceMode::Tiling,
        pub focus_follows_mouse as FocusFollowsMouse: bool = false,
        /// Kanshi-style configurations sometimes overlap outputs by a pixel on
        /// purpose. The compositor's model cannot represent that, so it is
        /// refused unless this says otherwise.
        pub allow_overlapping_outputs as AllowOverlappingOutputs: bool = false,

        pub window_rules as WindowRules: Vec<WindowRule> = Vec::new(),
        pub outputs as Outputs: Vec<OutputSetting> = Vec::new(),
        pub output_layouts as OutputLayouts: Vec<OutputLayout> = Vec::new(),
        pub startup as Startup: Vec<String> = Vec::new(),
        pub gaming as Gaming: GamingOptions = GamingOptions::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_documented_file_shape_round_trips() {
        let sample = r#"(
            layout: Floating,
            keybinds: [
                (keys: "Super+Q", action: "close-window"),
            ],
            window_rules: [
                (app_id: "Nautilus", floating: true),
                (title: "^(Open|Save)", floating: true, focus: false),
                (app_id: "^steam_app_", tearing: true, vrr: true),
            ],
            outputs: [
                (name: "eDP-1", scale: 2.0, position: (0, 0)),
            ],
            startup: [
                "crownbar",
                "swaybg -i /usr/share/backgrounds/crown.png",
            ],
            gaming: (allow_tearing: true),
        )"#;

        // Parsed the way `load` does, so what this asserts is what a hand-edited
        // file actually gets.
        let parsed: Compositor = crate::parser::options()
            .from_str(sample)
            .expect("the documented shape must parse");

        assert_eq!(parsed.layout, WorkspaceMode::Floating);
        // Omitted fields fall back rather than failing the whole section.
        assert_eq!(
            parsed.focus_follows_mouse,
            Compositor::default().focus_follows_mouse
        );
        assert_eq!(parsed.window_rules.len(), 3);
        assert_eq!(parsed.window_rules[2].tearing, Some(true));
        assert_eq!(parsed.window_rules[2].vrr, Some(true));
        assert_eq!(parsed.window_rules[0].app_id.as_deref(), Some("Nautilus"));
        assert_eq!(parsed.window_rules[0].floating, Some(true));
        assert_eq!(parsed.window_rules[0].focus, None, "omitted stays unset");
        assert_eq!(parsed.window_rules[1].focus, Some(false));
        assert_eq!(parsed.outputs[0].scale, Some(2.0));
        assert!(parsed.gaming.allow_tearing);
    }

    /// The compositor never writes this file, but [`load`](crate::load) does
    /// when it is missing — so what it writes has to be readable again.
    #[test]
    fn round_trips_through_save_and_load() {
        let config = Compositor {
            window_rules: vec![WindowRule {
                app_id: Some("blender".into()),
                floating: Some(true),
                ..Default::default()
            }],
            outputs: vec![OutputSetting {
                name: "eDP-1".into(),
                scale: Some(2.0),
                ..Default::default()
            }],
            ..Default::default()
        };

        let text = ron::ser::to_string(&config).expect("serialises");
        let parsed: Compositor = ron::from_str(&text).expect("deserialises");
        assert_eq!(parsed, config);
    }

    #[test]
    fn defaults_round_trip_through_ron() {
        let default = Compositor::default();
        let text = ron::ser::to_string(&default).expect("serialises");
        let parsed: Compositor = ron::from_str(&text).expect("deserialises");
        assert_eq!(parsed, default);
    }
}
