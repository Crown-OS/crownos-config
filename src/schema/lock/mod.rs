//! The look and behaviour of the lock screen, drawn by the compositor.
//!
//! The compositor reads this section live. `crownplugind` adds what only it
//! knows — a theme's layout tree and plugin slots, with their images — over
//! `crownos_plugin_host_v1`.

mod clock;
mod transition;

use serde::{Deserialize, Serialize};

pub use clock::{ClockArrangement, ClockFrame, ClockLook};
pub use transition::{Easing, Edge, Effect, LockTransitions, Transition};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ClockStyle {
    #[default]
    Classic,
    Large,
    Minimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LockLayout {
    #[default]
    Center,
    Left,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LockBackground {
    /// The user's desktop wallpaper, blurred.
    #[default]
    Desktop,
    /// Any image the user can read.
    Image(String),
    /// A flat theme color token.
    Token(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Anchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    #[default]
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

/// An anchor plus an offset from it in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Placement {
    pub anchor: Anchor,
    pub offset: [f32; 2],
}

/// Where a plugin's lock-screen slot sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LockSlotPosition {
    /// Replaces the clock.
    Clock,
    AboveClock,
    #[default]
    BelowClock,
    Bottom,
}

/// The user's placement of one plugin slot, overriding the plugin's default.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlotPlacement {
    /// `plugin:<id>/<widget>`.
    pub widget: String,
    pub position: LockSlotPosition,
    #[serde(default)]
    pub offset: [f32; 2],
}

crate::section! {
    pub struct Lock in "lock", keys LockKey {
        pub clock_style as ClockStyleKey: ClockStyle = ClockStyle::Classic,
        pub clock as ClockKey: ClockLook = ClockLook::default(),
        /// strftime pattern.
        pub time_format as TimeFormat: String = "%H:%M".to_owned(),
        pub date_format as DateFormat: String = "%A, %e %B".to_owned(),
        /// A system-installed font family; `None` keeps the default.
        pub font as Font: Option<String> = None,
        pub layout as Layout: LockLayout = LockLayout::Center,
        pub background as Background: LockBackground = LockBackground::Desktop,
        /// Darkening over the background, `0.0..=1.0`.
        pub dim as Dim: f32 = 0.25,
        /// Blur strength as a fraction of the desktop blur.
        pub blur as BlurKey: f32 = 1.0,
        pub auth_field as AuthField: Placement = Placement::default(),
        pub slots as Slots: Vec<SlotPlacement> = Vec::new(),
        pub transitions as Transitions: LockTransitions = LockTransitions::default(),
        /// Shows the user's name above the password field.
        pub show_user as ShowUser: bool = true,
        /// Seconds without input before the prompt gives way to the clock again.
        pub prompt_timeout_secs as PromptTimeoutSecs: u32 = 20,
        pub use_on_login_screen as UseOnLoginScreen: bool = true,
        pub show_notifications as ShowNotifications: bool = false,
        /// Seconds of inactivity before locking; 0 means never.
        pub idle_lock_after_secs as IdleLockAfterSecs: u32 = 300,
        pub lock_on_suspend as LockOnSuspend: bool = true,
    }
}

impl Lock {
    /// The user's placement for `widget`, if they moved it.
    pub fn slot_placement(&self, widget: &str) -> Option<&SlotPlacement> {
        self.slots.iter().find(|slot| slot.widget == widget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hand_written_lock_file_parses() {
        let lock: Lock = crate::parser::options()
            .from_str(
                r#"(
                    background: Image("/home/me/Pictures/lock.jpg"),
                    auth_field: (anchor: Bottom, offset: (0.0, -80.0)),
                    slots: [(widget: "plugin:dev.a.media/now", position: Bottom)],
                )"#,
            )
            .expect("parses");
        assert_eq!(
            lock.background,
            LockBackground::Image("/home/me/Pictures/lock.jpg".into())
        );
        assert_eq!(lock.auth_field.anchor, Anchor::Bottom);
        assert_eq!(
            lock.slot_placement("plugin:dev.a.media/now")
                .map(|slot| slot.position),
            Some(LockSlotPosition::Bottom)
        );
        assert_eq!(lock.dim, 0.25);
    }

    #[test]
    fn a_partial_clock_keeps_the_other_defaults() {
        let lock: Lock = crate::parser::options()
            .from_str(
                r#"(clock: (arrangement: Stacked, stretch: 2.4, frame: (warp: ((0.0, 0.0), (0.1, -0.2), (0.0, 0.0), (0.0, 0.0)))))"#,
            )
            .expect("parses");
        assert_eq!(lock.clock.arrangement, ClockArrangement::Stacked);
        assert_eq!(lock.clock.stretch, 2.4);
        assert!(lock.clock.frame.is_warped());
        assert_eq!(lock.clock.frame.scale, 1.0);
        assert!(lock.clock.morph_digits);
    }

    #[test]
    fn a_partial_transition_keeps_the_other_defaults() {
        let lock: Lock = crate::parser::options()
            .from_str(
                r#"(transitions: (reveal: (effects: [Slide(from: Top, distance: 30.0)], duration_ms: 200)))"#,
            )
            .expect("parses");
        assert_eq!(
            lock.transitions.reveal.effects,
            [Effect::Slide {
                from: Edge::Top,
                distance: 30.0
            }]
        );
        assert_eq!(lock.transitions.reveal.easing, Easing::EaseOut);
        assert_eq!(lock.transitions.appear, LockTransitions::default().appear);
    }
}
