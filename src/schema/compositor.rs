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

pub use super::output::{OutputLayout, OutputSetting, OutputTransform, Vrr};

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

/// Which outputs a workspace switch acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WorkspaceScope {
    /// Each output switches its workspaces alone; a switch acts on the focused output.
    #[default]
    PerMonitor,
    /// A switch acts on all outputs together and shows the same workspace index on each.
    Shared,
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

/// Latency trade-offs for games.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GamingOptions {
    /// Let a fullscreen window that asks for it, or that a rule names, tear
    /// instead of waiting for vblank.
    pub allow_tearing: bool,
}

impl Default for GamingOptions {
    fn default() -> Self {
        Self {
            allow_tearing: true,
        }
    }
}

/// Proton titles: xwayland-satellite hands their `WM_CLASS`, `steam_app_<id>`,
/// over as the app id.
const STEAM_GAME_APP_ID: &str = "^steam_app_";

/// What a fresh install ships: Steam games may tear, and get a variable refresh
/// rate even when windowed. Proton games are X11 windows, which cannot ask for
/// either.
fn default_window_rules() -> Vec<WindowRule> {
    vec![WindowRule {
        app_id: Some(STEAM_GAME_APP_ID.to_owned()),
        tearing: Some(true),
        vrr: Some(true),
        ..WindowRule::default()
    }]
}

crate::section! {
    pub struct Compositor in "compositor", keys CompositorKey {
        pub layout as Layout: WorkspaceMode = WorkspaceMode::Tiling,
        /// Whether a workspace switch acts on the focused output or on all outputs.
        pub workspace_scope as Scope: WorkspaceScope = WorkspaceScope::PerMonitor,
        pub focus_follows_mouse as FocusFollowsMouse: bool = false,
        /// Kanshi-style configurations sometimes overlap outputs by a pixel on
        /// purpose. The compositor's model cannot represent that, so it is
        /// refused unless this says otherwise.
        pub allow_overlapping_outputs as AllowOverlappingOutputs: bool = false,

        pub window_rules as WindowRules: Vec<WindowRule> = default_window_rules(),
        /// The output that shows the dock and notifications by default, named
        /// like [`OutputSetting::name`]. When unset, the compositor selects one.
        pub primary_output as PrimaryOutput: Option<String> = None,
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
    fn the_multi_monitor_keys_round_trip() -> Result<(), Box<dyn std::error::Error>> {
        let sample = r#"(
            workspace_scope: Shared,
            primary_output: Some("Dell Inc. U2720Q 7XK2"),
            outputs: [
                (name: "Dell Inc. U2720Q 7XK2", bar: Some(true), dock: None, deep_color: Some(true)),
                (name: "HDMI-A-1", notifications: Some(false), software_brightness: Some(0.6)),
            ],
        )"#;
        let parsed: Compositor = crate::parser::options().from_str(sample)?;

        assert_eq!(parsed.workspace_scope, WorkspaceScope::Shared);
        assert_eq!(
            parsed.primary_output.as_deref(),
            Some("Dell Inc. U2720Q 7XK2")
        );
        assert_eq!(parsed.outputs[0].bar, Some(true));
        assert_eq!(parsed.outputs[0].dock, None);
        assert_eq!(parsed.outputs[0].deep_color, Some(true));
        assert_eq!(parsed.outputs[1].notifications, Some(false));
        assert_eq!(parsed.outputs[1].software_brightness, Some(0.6));

        let text = ron::ser::to_string(&parsed)?;
        let reparsed: Compositor = ron::from_str(&text)?;
        assert_eq!(reparsed, parsed);
        Ok(())
    }

    #[test]
    fn a_file_without_the_multi_monitor_keys_scopes_workspaces_per_monitor()
    -> Result<(), ron::error::SpannedError> {
        let parsed: Compositor = crate::parser::options().from_str("(layout: Tiling)")?;
        assert_eq!(parsed.workspace_scope, WorkspaceScope::PerMonitor);
        assert_eq!(parsed.primary_output, None);
        Ok(())
    }

    #[test]
    fn a_file_with_focus_at_startup_still_loads() -> Result<(), ron::error::SpannedError> {
        let parsed: Compositor = crate::parser::options().from_str(
            r#"(outputs: [(name: "eDP-1", focus_at_startup: Some(true), scale: 2.0)])"#,
        )?;
        assert_eq!(parsed.outputs[0].focus_at_startup, Some(true));
        assert_eq!(parsed.outputs[0].scale, Some(2.0));
        Ok(())
    }

    #[test]
    fn displays_default_to_vrr_on_demand() {
        assert_eq!(Vrr::default(), Vrr::OnDemand);
    }

    #[test]
    fn steam_games_may_tear_and_use_vrr_by_default() {
        let defaults = Compositor::default();
        let steam = defaults
            .window_rules
            .iter()
            .find(|rule| rule.app_id.as_deref() == Some(STEAM_GAME_APP_ID))
            .expect("the default rules name Steam games");

        assert_eq!((steam.tearing, steam.vrr), (Some(true), Some(true)));
        assert!(
            defaults.gaming.allow_tearing,
            "the rule's tearing is inert without the global switch"
        );
    }

    #[test]
    fn a_file_without_gaming_options_still_allows_tearing() {
        let parsed: Compositor = crate::parser::options()
            .from_str("(gaming: ())")
            .expect("parses");
        assert!(parsed.gaming.allow_tearing);
    }

    #[test]
    fn defaults_round_trip_through_ron() {
        let default = Compositor::default();
        let text = ron::ser::to_string(&default).expect("serialises");
        let parsed: Compositor = ron::from_str(&text).expect("deserialises");
        assert_eq!(parsed, default);
    }
}
