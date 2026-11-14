//! One default value per section, for the exact RON each key starts with.
//!
//! The list is checked against the `section!` declarations the parser found, so a new section
//! that is not added here fails `config-docs` instead of silently missing from the reference.

use crownos_config::DynSection;
use crownos_config::schema::{
    Agents, Appearance, Bar, Bluetooth, Camera, Clipboard, Compositor, Display, Files, Glass,
    Input, Keybinds, Lock, Notifications, Plugins, Power, Recorder, Screenshot, Sound, Theme,
    VisualContext, Voice, Wifi,
};

pub(crate) fn defaults() -> Vec<Box<dyn DynSection>> {
    vec![
        Box::new(Agents::default()),
        Box::new(Appearance::default()),
        Box::new(Bar::default()),
        Box::new(Bluetooth::default()),
        Box::new(Camera::default()),
        Box::new(Clipboard::default()),
        Box::new(Compositor::default()),
        Box::new(Display::default()),
        Box::new(Files::default()),
        Box::new(Glass::default()),
        Box::new(Input::default()),
        Box::new(Keybinds::default()),
        Box::new(Lock::default()),
        Box::new(Notifications::default()),
        Box::new(Plugins::default()),
        Box::new(Power::default()),
        Box::new(Recorder::default()),
        Box::new(Screenshot::default()),
        Box::new(Sound::default()),
        Box::new(Theme::default()),
        Box::new(VisualContext::default()),
        Box::new(Voice::default()),
        Box::new(Wifi::default()),
    ]
}
