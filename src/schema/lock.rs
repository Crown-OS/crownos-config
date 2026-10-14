//! The look and behaviour of the lock and login screen, drawn by the compositor.
//!
//! The compositor cannot read user files: `crownplugind` resolves this
//! section, reads any image it names and pushes both over
//! `crownos_plugin_host_v1`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ClockStyle {
    #[default]
    Classic,
    Large,
    Minimal,
    Stacked,
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
pub enum LockTransition {
    #[default]
    Rise,
    Fade,
    Zoom,
    None,
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
        pub transition as Transition: LockTransition = LockTransition::Rise,
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
}
