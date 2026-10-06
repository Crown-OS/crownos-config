//! The lit edge of the compositor's frosted glass.
//!
//! Every piece of glass the compositor draws — window frames, a client's own
//! blurred background, panels, menus — has a bevelled rim that refracts what is
//! behind it and catches the light. This is how that rim looks, separately for
//! each kind of surface, so a panel can glow while windows stay quiet.

use serde::{Deserialize, Serialize};

/// The rim of one kind of glass.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EdgeGlow {
    /// Off leaves the edge flat: no bevel, no highlight, no inner shadow.
    pub enabled: bool,
    /// Width of the bevel in logical pixels.
    pub width: f64,
    /// Brightness of the highlight along the edge, 0 to 1.
    pub intensity: f64,
    /// Where the light comes from, in degrees clockwise from straight up. The
    /// highlight sits on the two edges facing along this line.
    pub light_angle: f64,
    /// How much the highlight takes its colour from what is behind the glass:
    /// 0 is white light, 1 is lit entirely in the colours of the window below.
    pub adaptive_tint: f64,
    /// Depth of the shade just inside the edge, 0 to 1.
    pub inner_shadow: f64,
}

impl EdgeGlow {
    pub const WINDOW: Self = Self {
        enabled: true,
        width: 2.0,
        intensity: 0.3,
        light_angle: -45.0,
        adaptive_tint: 0.6,
        inner_shadow: 0.28,
    };

    /// Panels sit over everything, so their rim is what separates them from
    /// it: a touch brighter than a window's.
    pub const PANEL: Self = Self {
        intensity: 0.4,
        ..Self::WINDOW
    };

    /// Menus are small and short-lived; a thin, quiet rim keeps them from
    /// looking like buttons.
    pub const MENU: Self = Self {
        width: 1.5,
        intensity: 0.25,
        inner_shadow: 0.2,
        ..Self::WINDOW
    };
}

impl Default for EdgeGlow {
    fn default() -> Self {
        Self::WINDOW
    }
}

crate::section! {
    pub struct Glass in "glass", keys GlassKey {
        /// Window frames and the blurred backgrounds windows ask for.
        pub windows as Windows: EdgeGlow = EdgeGlow::WINDOW,
        /// Layer-shell surfaces: bars, docks, launchers, notifications.
        pub panels as Panels: EdgeGlow = EdgeGlow::PANEL,
        /// Popups and the compositor's own menus.
        pub menus as Menus: EdgeGlow = EdgeGlow::MENU,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_partial_rim_keeps_the_rest_of_its_defaults() {
        let glass: Glass = ron::from_str("(panels: (intensity: 0.9))").expect("parses");
        assert_eq!(glass.panels.intensity, 0.9);
        assert_eq!(glass.panels.width, EdgeGlow::WINDOW.width);
        assert_eq!(glass.windows, EdgeGlow::WINDOW);
        assert_eq!(glass.menus, EdgeGlow::MENU);
    }
}
