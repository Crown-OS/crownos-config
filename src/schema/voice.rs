//! Voice enhancement for every app's microphone.
//!
//! The consumers are crown-voiced, which follows this file live, and the
//! settings panel's Voice page, which writes it.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VoiceEnhancement {
    Off,
    Light,
    #[default]
    NoiseSuppression,
    StudioVoice,
}

/// Which hardware crown-voiced may run enhancement on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VoiceBackend {
    /// The best backend the installer set up and this machine can run.
    #[default]
    Automatic,
    /// The CPU only, leaving GPUs and NPUs idle.
    Cpu,
}

crate::section! {
    pub struct Voice in "voice", keys VoiceKey {
        /// Makes an enhanced copy of the default microphone the system default
        /// source, so apps that open the default input are enhanced too.
        pub enhance_all_apps as EnhanceAllApps: bool = false,

        /// The enhancement the system-wide microphone runs.
        pub all_apps_enhancement as AllAppsEnhancement: VoiceEnhancement = VoiceEnhancement::NoiseSuppression,

        pub backend as Backend: VoiceBackend = VoiceBackend::Automatic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enhancing_all_apps_is_opt_in() {
        assert!(!Voice::default().enhance_all_apps);
    }

    #[test]
    fn round_trips_through_ron() {
        let voice = Voice {
            enhance_all_apps: true,
            all_apps_enhancement: VoiceEnhancement::Light,
            backend: VoiceBackend::Cpu,
        };
        let text = ron::ser::to_string_pretty(&voice, Default::default())
            .expect("serialise voice section");
        assert_eq!(ron::from_str::<Voice>(&text).ok(), Some(voice));
    }
}
