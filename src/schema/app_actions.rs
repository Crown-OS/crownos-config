//! The actions that applications registered through `crownos_keybind_v1`, and
//! the chord each one has.
//!
//! The writer is [crownpositor]. It writes this file each time an application
//! registers or removes an action, or a chord changes. To change the chord of
//! an action, add a row to `keybinds.app_keybinds`, not to this file: the
//! compositor writes the row of a running action again at the next change.
//!
//! An action stays in the list after its application stops, so the settings
//! panel can show it and the user can change its chord at any time. To remove
//! the action of an application that is not running, delete its row here.
//!
//! [crownpositor]: https://github.com/crown-os/crownpositor

use serde::{Deserialize, Serialize};

use crate::keybind::Keybind;

/// One action that an application registered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppAction {
    /// The app_id the application registered the action with.
    pub app_id: String,
    /// The name of the action, for example `toggle-recording`.
    pub name: String,
    /// What the action does, in words for the user.
    pub description: String,
    /// The chord the application asked for. `None` if it cannot be read.
    pub preferred_keys: Keybind,
    /// The chord the compositor tried to give the action: the one in
    /// `keybinds.app_keybinds`, else the one the application asked for.
    pub wanted_keys: Keybind,
    /// The chord the action has now, or had when its application stopped.
    /// `None` when the action has no chord.
    pub assigned_keys: Keybind,
    /// Whether the application has the action registered now.
    pub running: bool,
}

impl AppAction {
    /// Whether this is the action `name` of the application `app_id`.
    pub fn is_action(&self, app_id: &str, name: &str) -> bool {
        self.app_id == app_id && self.name == name
    }

    /// Whether the running action did not get the chord it should have,
    /// because the desktop or another action has it, or it is not allowed.
    pub fn is_refused(&self) -> bool {
        self.running && self.wanted_keys != Keybind::NONE && self.assigned_keys != self.wanted_keys
    }
}

crate::section! {
    pub struct AppActions in "app_actions", keys AppActionsKey {
        /// Every action that an application registered, in the order they
        /// were first registered.
        pub actions as Actions: Vec<AppAction> = Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_action_is_written_with_its_chords_as_strings() {
        let app_actions = AppActions {
            actions: vec![AppAction {
                app_id: "org.crownos.SoundRecorder".to_owned(),
                name: "toggle-recording".to_owned(),
                description: "Start or stop a recording".to_owned(),
                preferred_keys: "Super+Alt+R".parse().expect("parse chord"),
                wanted_keys: "Super+Alt+R".parse().expect("parse chord"),
                assigned_keys: Keybind::NONE,
                running: true,
            }],
        };
        let text = ron::ser::to_string_pretty(&app_actions, Default::default()).expect("serialise");

        assert!(
            text.contains("preferred_keys: \"Super+Alt+R\"")
                && text.contains("assigned_keys: \"None\""),
            "got:\n{text}"
        );
        assert_eq!(
            ron::from_str::<AppActions>(&text).expect("parse back"),
            app_actions
        );
    }

    fn action(wanted_keys: &str, assigned_keys: &str, running: bool) -> AppAction {
        AppAction {
            app_id: "recorder".to_owned(),
            name: "toggle".to_owned(),
            description: String::new(),
            preferred_keys: Keybind::NONE,
            wanted_keys: wanted_keys.parse().expect("parse chord"),
            assigned_keys: assigned_keys.parse().expect("parse chord"),
            running,
        }
    }

    #[test]
    fn an_action_is_refused_only_while_it_runs_without_its_wanted_chord() {
        assert!(action("Super+Q", "None", true).is_refused());
        assert!(!action("Super+Q", "None", false).is_refused());
        assert!(!action("Super+Alt+R", "Super+Alt+R", true).is_refused());
        assert!(!action("None", "None", true).is_refused());
    }
}
