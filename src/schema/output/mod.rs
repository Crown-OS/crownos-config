//! The settings of each monitor, and where the monitors sit.

mod lookup;
mod shell_role;

use serde::{Deserialize, Serialize};

use super::compositor::WorkspaceMode;

pub use lookup::output_identity;
pub use shell_role::ShellRole;

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

/// When a display runs at a variable refresh rate.
///
/// Tri-state rather than a bool because "always on" and "on when it helps" are
/// genuinely different: some panels flicker at low refresh on a static
/// desktop, so `OnDemand` turns it on only while a fullscreen window is the
/// only thing on the output, or a game is visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Vrr {
    Off,
    On,
    #[default]
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
    ///
    /// Unread since the multi-monitor work; kept so old files load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_at_startup: Option<bool>,
    /// Overrides the global default for workspaces created on this output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<WorkspaceMode>,
    /// Show the bar on this output (unset: show it on all outputs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar: Option<bool>,
    /// Show the dock on this output (unset: show it only on the primary output).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dock: Option<bool>,
    /// Show notification popups on this output (unset: show them only on the primary output).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifications: Option<bool>,
    /// Scan out this output with 10 bits for each color channel instead of 8.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deep_color: Option<bool>,
    /// Dim level from 0.15 to 1.0 that the compositor writes for a monitor with no backlight and no DDC/CI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_brightness: Option<f64>,
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
        self.heads.len() == present.len() && self.heads.iter().all(|head| present.contains(head))
    }

    pub fn position_of(&self, name: &str) -> Option<(i32, i32)> {
        self.positions
            .iter()
            .find_map(|(head, x, y)| (head == name).then_some((*x, *y)))
    }
}
