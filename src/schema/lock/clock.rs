//! How the lock-screen clock's digits look and where they sit.

use serde::{Deserialize, Serialize};

/// Where the hours, the minutes and the date sit against each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ClockLayout {
    /// Hours and minutes side by side across the middle, under the date.
    #[default]
    Centered,
    /// Hours and minutes side by side at the left, over the date.
    Left,
    /// Hours over minutes at the left, the date beside the hours.
    Stacked,
}

/// The typeface the clock's digits are drawn in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ClockFont {
    /// Condensed monoline digits with round bowls.
    #[default]
    Sans,
    /// Monoline digits with a grotesque's squarer bowls.
    Grotesk,
    /// High-contrast digits: heavy stems and hairline curves.
    Serif,
}

/// Where the clock sits and how it is bent, as the Settings handles leave it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClockFrame {
    /// Logical pixels from where the layout puts the clock.
    pub offset: [f32; 2],
    pub scale: f32,
    /// How far each corner of the clock's box moves — top left, top right,
    /// bottom right, bottom left — as a fraction of the box's size.
    pub warp: [[f32; 2]; 4],
    /// Logical pixels the hours move on their own.
    pub hours: [f32; 2],
    /// Logical pixels the minutes move on their own.
    pub minutes: [f32; 2],
}

impl Default for ClockFrame {
    fn default() -> Self {
        Self {
            offset: [0.0; 2],
            scale: 1.0,
            warp: [[0.0; 2]; 4],
            hours: [0.0; 2],
            minutes: [0.0; 2],
        }
    }
}

impl ClockFrame {
    pub const MIN_SCALE: f32 = 0.25;
    pub const MAX_SCALE: f32 = 4.0;
    /// The furthest a warped corner moves, as a fraction of the box.
    pub const MAX_WARP: f32 = 1.0;

    pub fn is_warped(&self) -> bool {
        self.warp.iter().flatten().any(|shift| *shift != 0.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClockLook {
    pub layout: ClockLayout,
    pub font: ClockFont,
    /// From hairline, 100, to heavy, 900.
    pub weight: f32,
    /// How much taller than the font draws them the digits stand: stems
    /// lengthen while bowls stay round and strokes keep their width.
    pub stretch: f32,
    /// From frosted glass showing the background through the digits, 0, to
    /// solid white, 1.
    pub whiteness: f32,
    /// Draws the separator between side-by-side hours and minutes.
    pub colon: bool,
    /// Strength of the shadow under the digits, `0.0..=1.0`.
    pub shadow: f32,
    pub frame: ClockFrame,
    /// Stands the background's subject, cut out of it on the fly, in front of
    /// the clock.
    pub depth: bool,
}

impl Default for ClockLook {
    fn default() -> Self {
        Self {
            layout: ClockLayout::Centered,
            font: ClockFont::Sans,
            weight: Self::DEFAULT_WEIGHT,
            stretch: Self::DEFAULT_STRETCH,
            whiteness: Self::DEFAULT_WHITENESS,
            colon: true,
            shadow: 0.0,
            frame: ClockFrame::default(),
            depth: true,
        }
    }
}

impl ClockLook {
    pub const MIN_STRETCH: f32 = 1.0;
    pub const MAX_STRETCH: f32 = 5.0;
    pub const DEFAULT_STRETCH: f32 = 3.5;
    pub const MIN_WEIGHT: f32 = 100.0;
    pub const MAX_WEIGHT: f32 = 900.0;
    pub const DEFAULT_WEIGHT: f32 = 700.0;
    pub const DEFAULT_WHITENESS: f32 = 0.5;

    pub fn clamped_weight(&self) -> f32 {
        self.weight.clamp(Self::MIN_WEIGHT, Self::MAX_WEIGHT)
    }

    pub fn clamped_stretch(&self) -> f32 {
        self.stretch.clamp(Self::MIN_STRETCH, Self::MAX_STRETCH)
    }

    pub fn clamped_whiteness(&self) -> f32 {
        self.whiteness.clamp(0.0, 1.0)
    }
}
