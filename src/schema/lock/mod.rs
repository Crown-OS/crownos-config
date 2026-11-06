//! The look and behaviour of the lock screen, drawn by the compositor.
//!
//! The compositor reads this section live. `crownplugind` adds what only it
//! knows — a theme's layout tree and plugin slots, with their images — over
//! `crownos_plugin_host_v1`.

mod clock;

use serde::{Deserialize, Serialize};

pub use clock::{ClockFont, ClockFrame, ClockLayout, ClockLook};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LockBackground {
    /// The user's desktop wallpaper.
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
        pub clock as ClockKey: ClockLook = ClockLook::default(),
        /// strftime pattern.
        pub time_format as TimeFormat: String = "%H:%M".to_owned(),
        pub date_format as DateFormat: String = "%a, %b %-d".to_owned(),
        pub background as Background: LockBackground = LockBackground::Desktop,
        /// Darkening over the background, `0.0..=1.0`.
        pub dim as Dim: f32 = 0.0,
        pub auth_field as AuthField: Placement = Placement::default(),
        pub slots as Slots: Vec<SlotPlacement> = Vec::new(),
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
        assert_eq!(lock.dim, 0.0);
    }

    #[test]
    fn a_partial_clock_keeps_the_other_defaults() {
        let lock: Lock = crate::parser::options()
            .from_str(
                r#"(clock: (layout: Stacked, font: Serif, weight: 250.5, stretch: 2.4, frame: (warp: ((0.0, 0.0), (0.1, -0.2), (0.0, 0.0), (0.0, 0.0)))))"#,
            )
            .expect("parses");
        assert_eq!(lock.clock.layout, ClockLayout::Stacked);
        assert_eq!(lock.clock.font, ClockFont::Serif);
        assert_eq!(lock.clock.weight, 250.5);
        assert_eq!(lock.clock.stretch, 2.4);
        assert!(lock.clock.frame.is_warped());
        assert_eq!(lock.clock.frame.scale, 1.0);
        assert_eq!(lock.clock.whiteness, ClockLook::DEFAULT_WHITENESS);
        assert!(lock.clock.depth);
    }

    #[test]
    fn out_of_range_looks_clamp() {
        let look = ClockLook {
            weight: 2000.0,
            stretch: 0.2,
            whiteness: -1.0,
            ..ClockLook::default()
        };
        assert_eq!(look.clamped_weight(), ClockLook::MAX_WEIGHT);
        assert_eq!(look.clamped_stretch(), ClockLook::MIN_STRETCH);
        assert_eq!(look.clamped_whiteness(), 0.0);
    }
}
