//! Desktop-wide keyboard shortcuts.
//!
//! Shortcuts that belong to the desktop itself rather than to one feature —
//! the ones a compositor grabs globally and acts on no matter what has focus.
//! A shortcut that only makes sense while a particular thing is running stays
//! with that thing instead: dictation's push-to-talk chord lives in
//! [`input`](crate::schema::input), because it is meaningless with dictation
//! switched off.
//!
//! The consumer is the compositor. It is the only process that can honour a
//! global chord — every other CrownOS app is a Wayland client and only ever
//! sees keys while it is focused, which is exactly the state a launcher
//! shortcut has to work from *outside*. It reads this file live, so rebinding
//! takes effect without a restart or a re-login.

use serde::{Deserialize, Serialize};

use crate::keybind::Keybind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub keys: String,
    pub action: String,
}

/// The chord the user set for one action that an application registered
/// through `crownos_keybind_v1`. `keys: "None"` removes the chord of the action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppKeybind {
    /// The app_id the application registered the action with.
    pub app_id: String,
    /// The name of the action, for example `toggle-recording`.
    pub name: String,
    /// The chord the action gets in place of the one the application asked for.
    pub keys: Keybind,
}

crate::section! {
    pub struct Keybinds in "keybinds", keys KeybindsKey {
        pub launcher as Launcher: Keybind = Keybind::SUPER_SPACE,
        pub dictation as Dictation: Keybind = Keybind::SUPER_CTRL,
        pub custom_keybinds as CustomKeybinds: Vec<Binding> = Vec::new(),
        pub app_keybinds as AppKeybinds: Vec<AppKeybind> = Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keybind::Mods;

    #[test]
    fn the_launchpad_opens_on_super_space_by_default() {
        assert_eq!(Keybinds::default().launcher, Keybind::SUPER_SPACE);
    }

    #[test]
    fn dictation_listens_on_a_modifier_only_chord_by_default() {
        let dictation = Keybinds::default().dictation;

        assert_eq!(
            dictation.mods,
            Mods {
                meta: true,
                ctrl: true,
                alt: false,
                shift: false
            }
        );
        assert_eq!(
            dictation.key, None,
            "a modifier-only chord cannot collide with an application's own bindings"
        );
    }

    /// The written form is what lands in the file and what the settings panel
    /// shows, and modifiers print in one fixed order however they were typed.
    #[test]
    fn the_dictation_chord_is_written_super_ctrl_whichever_way_it_is_spelled() {
        assert_eq!(Keybinds::default().dictation.to_string(), "Super+Ctrl");
        assert_eq!(
            "Ctrl+Super".parse::<Keybind>().unwrap(),
            Keybinds::default().dictation
        );
    }

    #[test]
    fn the_shortcuts_are_written_as_strings() {
        let text = ron::ser::to_string_pretty(&Keybinds::default(), Default::default())
            .expect("serialise keybinds section");

        assert!(
            text.contains("launcher: \"Super+Space\"")
                && text.contains("dictation: \"Super+Ctrl\""),
            "the shortcuts should be hand-editable, got:\n{text}"
        );
        assert_eq!(
            ron::from_str::<Keybinds>(&text).expect("parse back"),
            Keybinds::default()
        );
    }

    #[test]
    fn an_app_keybind_is_written_as_a_hand_editable_row() {
        let keybinds = Keybinds {
            app_keybinds: vec![AppKeybind {
                app_id: "org.crownos.SoundRecorder".to_owned(),
                name: "toggle-recording".to_owned(),
                keys: "Super+Alt+R".parse().expect("parse chord"),
            }],
            ..Keybinds::default()
        };
        let text = ron::ser::to_string_pretty(&keybinds, Default::default()).expect("serialise");

        assert!(text.contains("keys: \"Super+Alt+R\""), "got:\n{text}");
        assert_eq!(
            ron::from_str::<Keybinds>(&text).expect("parse back"),
            keybinds
        );
    }

    /// Unbinding is a value the file can hold, not a parse failure — a user who
    /// clears the field in the panel gets `"None"` written out and read back.
    #[test]
    fn an_unbound_launcher_survives_the_file() {
        let cleared = Keybinds {
            dictation: Keybind::NONE,
            launcher: Keybind::NONE,

            custom_keybinds: Vec::new(),
            app_keybinds: Vec::new(),
        };
        let text = ron::ser::to_string_pretty(&cleared, Default::default()).expect("serialise");

        assert!(text.contains("launcher: \"None\""), "got:\n{text}");
        assert_eq!(
            ron::from_str::<Keybinds>(&text).expect("parse back"),
            cleared
        );
    }
}
