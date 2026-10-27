//! How lock-screen layers come and go.
//!
//! A [`Transition`] is a list of [`Effect`]s played together over one duration.
//! Each effect describes how a layer *enters*; a layer leaving plays it in
//! reverse, except that motion carries on through: a layer that slides in from
//! the bottom leaves through the top, and one that zooms in from smaller leaves
//! by growing.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Edge {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Easing {
    Linear,
    EaseIn,
    #[default]
    EaseOut,
    EaseInOut,
    /// Overshoots slightly before settling.
    Spring,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Effect {
    Fade,
    /// Starts blurred by `radius` logical pixels.
    Blur {
        radius: f32,
    },
    /// Starts `distance` logical pixels toward `from`.
    Slide {
        from: Edge,
        distance: f32,
    },
    /// Starts scaled by `from`.
    Zoom {
        from: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Transition {
    pub effects: Vec<Effect>,
    pub duration_ms: u32,
    pub easing: Easing,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            effects: vec![Effect::Fade],
            duration_ms: 300,
            easing: Easing::EaseOut,
        }
    }
}

impl Transition {
    pub fn none() -> Self {
        Self {
            effects: Vec::new(),
            duration_ms: 0,
            easing: Easing::Linear,
        }
    }

    pub fn is_instant(&self) -> bool {
        self.duration_ms == 0 || self.effects.is_empty()
    }
}

/// The three moments the lock screen animates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LockTransitions {
    /// The clock arriving as the screen locks.
    pub appear: Transition,
    /// The clock giving way to the password prompt, and back after a while
    /// without input.
    pub reveal: Transition,
    /// Everything leaving once the session unlocks.
    pub dismiss: Transition,
}

impl Default for LockTransitions {
    fn default() -> Self {
        Self {
            appear: Transition {
                effects: vec![Effect::Fade, Effect::Blur { radius: 24.0 }],
                duration_ms: 420,
                easing: Easing::EaseOut,
            },
            reveal: Transition {
                effects: vec![
                    Effect::Fade,
                    Effect::Blur { radius: 12.0 },
                    Effect::Slide {
                        from: Edge::Bottom,
                        distance: 40.0,
                    },
                ],
                duration_ms: 320,
                easing: Easing::EaseInOut,
            },
            dismiss: Transition {
                effects: vec![Effect::Fade, Effect::Zoom { from: 0.94 }],
                duration_ms: 360,
                easing: Easing::EaseIn,
            },
        }
    }
}
