//! How the lock-screen clock's digits look and where they sit.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ClockArrangement {
    /// Hours and minutes side by side, with the separator between.
    #[default]
    Row,
    /// Hours over minutes, without the separator.
    Stacked,
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
    pub arrangement: ClockArrangement,
    /// 100 to 900; `None` keeps the clock style's weight.
    pub weight: Option<u16>,
    /// How much taller than the font draws them the digits stand.
    pub stretch: f32,
    /// Digits of frosted glass showing the blurred background through them.
    pub glass: bool,
    /// Strength of the shadow under the digits, `0.0..=1.0`.
    pub shadow: f32,
    /// A digit that changes blurs out as the new one blurs in.
    pub morph_digits: bool,
    pub frame: ClockFrame,
    /// A picture of the background's subject on a transparent ground, framed
    /// like the background, drawn over the clock so the subject stands in
    /// front of it.
    pub depth: Option<String>,
}

impl Default for ClockLook {
    fn default() -> Self {
        Self {
            arrangement: ClockArrangement::Row,
            weight: None,
            stretch: 1.0,
            glass: false,
            shadow: 0.0,
            morph_digits: true,
            frame: ClockFrame::default(),
            depth: None,
        }
    }
}

impl ClockLook {
    pub const MIN_STRETCH: f32 = 1.0;
    pub const MAX_STRETCH: f32 = 3.0;
    pub const MIN_WEIGHT: u16 = 100;
    pub const MAX_WEIGHT: u16 = 900;
}
