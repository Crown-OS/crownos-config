//! The shared shape of CrownOS settings.
//!
//! Every app in the desktop reads and writes the same types from the same
//! files, so the settings panel, the status bar and a shell script all agree
//! on what `~/.config/crownos/sound.ron` means. Each type carries the section name it
//! belongs to:
//!
//! ```ignore
//! use crownos_config::schema::Sound;
//!
//! let sound: Sound = crownos_config::load(Sound::SECTION); // ~/.config/crownos/sound.ron
//! ```
//!
//! Adding a field to one of these structs is backwards compatible for readers
//! of *newer* files only if the field has a default; prefer extending a struct
//! over introducing a parallel one, and remember that a value which fails to
//! parse silently falls back to [`Default`].
//!
//! # Keys
//!
//! Each struct is declared with [`section!`](crate::section), which also emits
//! a [`Key`](crate::Key) type per field and an enum listing them all — that is
//! what [`subscribe_key`](crate::subscribe_key) takes instead of a section
//! string plus a field name.
//!
//! The key enums are re-exported here (`AppearanceKey`, `SoundKey`, ...) but
//! the per-field key types are not: `Enabled` exists in three sections, so they
//! stay behind their module and read as `wifi::Enabled` or
//! `bluetooth::Enabled` at the call site.
//!
//! ```ignore
//! use crownos_config::schema::{appearance, AppearanceKey};
//!
//! let sub = crownos_config::subscribe_key(appearance::DarkMode, |on| set_theme(on));
//! for key in AppearanceKey::ALL {
//!     println!("{key}"); // appearance.dark_mode, appearance.accent, ...
//! }
//! ```

pub mod appearance;
pub mod bar;
pub mod bluetooth;
pub mod clipboard;
pub mod compositor;
pub mod display;
pub mod glass;
pub mod input;
pub mod keybinds;
pub mod lock;
pub mod notification;
pub mod plugins;
pub mod power;
pub mod sound;
pub mod theme;
pub mod wifi;

pub use appearance::{
    AccentColor, AnimationProfile, Appearance, AppearanceKey, CloseAnimation, MinimizeAnimation,
};
pub use bar::{Bar, BarKey, BarPosition};
pub use bluetooth::{Bluetooth, BluetoothKey};
pub use clipboard::{Clipboard, ClipboardKey};
pub use compositor::{
    Compositor, CompositorKey, GamingOptions, OutputLayout, OutputSetting, OutputTransform, Vrr,
    WindowRule, WorkspaceMode,
};
pub use display::{Display, DisplayKey};
pub use glass::{EdgeGlow, Glass, GlassKey};
pub use input::{Input, InputKey};
pub use keybinds::{Binding, Keybinds, KeybindsKey};
pub use lock::{
    Anchor, ClockStyle, Lock, LockBackground, LockKey, LockLayout, LockSlotPosition,
    LockTransition, Placement, SlotPlacement,
};
pub use notification::{Notifications, NotificationsKey};
pub use plugins::{Decision, Plugins, PluginsKey};
pub use power::{Power, PowerKey, PowerProfile};
pub use sound::{Sound, SoundKey};
pub use theme::{AccentGradient, Font, Radii, Theme, ThemeKey};
pub use wifi::{Wifi, WifiKey};

use crate::patch::{DynSection, PatchError, edit_typed, read_typed};

macro_rules! by_name {
    ($($Section:ty),* $(,)?) => {
        /// Every section name, i.e. every file `crownos-config` owns.
        pub const SECTIONS: &[&str] = &[$(<$Section>::SECTION,)*];

        /// Load the section called `section` and hand it to `read` by name.
        pub fn read_section<R>(
            section: &str,
            read: impl FnOnce(&dyn DynSection) -> Result<R, PatchError>,
        ) -> Result<R, PatchError> {
            $(if section == <$Section>::SECTION {
                return read_typed::<$Section, R>(section, read);
            })*
            Err(PatchError::UnknownSection(section.to_owned()))
        }

        /// Whether `text` is a valid value for `section.key`, without touching disk.
        pub fn check_value(section: &str, key: &str, text: &str) -> Result<(), PatchError> {
            $(if section == <$Section>::SECTION {
                return <$Section>::default().set_ron(key, text);
            })*
            Err(PatchError::UnknownSection(section.to_owned()))
        }

        /// Load, edit by name, and save only if something changed.
        pub fn edit_section<R>(
            section: &str,
            edit: impl FnOnce(&mut dyn DynSection) -> Result<R, PatchError>,
        ) -> Result<R, PatchError> {
            $(if section == <$Section>::SECTION {
                return edit_typed::<$Section, R>(section, edit);
            })*
            Err(PatchError::UnknownSection(section.to_owned()))
        }
    };
}

by_name!(
    Appearance,
    Bar,
    Bluetooth,
    Clipboard,
    Compositor,
    Display,
    Input,
    Keybinds,
    Lock,
    Notifications,
    Plugins,
    Power,
    Sound,
    Theme,
    Wifi,
);
