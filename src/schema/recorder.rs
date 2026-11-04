//! The voice recorder: how new recordings are captured and saved.
//!
//! The consumer is crownos-sound-recorder, which writes this file whenever one
//! of its pickers changes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RecorderFormat {
    /// Opus in Ogg at voice bit rate.
    #[default]
    Opus,
    /// Lossless 24-bit FLAC.
    Flac,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RecorderEnhancement {
    Off,
    Light,
    #[default]
    NoiseSuppression,
    StudioVoice,
}

crate::section! {
    pub struct Recorder in "recorder", keys RecorderKey {
        pub format as Format: RecorderFormat = RecorderFormat::Opus,

        /// The live enhancement new recordings start with, and the default of "Enhance".
        pub enhancement as Enhancement: RecorderEnhancement = RecorderEnhancement::NoiseSuppression,

        /// The PipeWire node name of the microphone. `None` is the system default.
        pub input as Input: Option<String> = None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_ron() {
        let recorder = Recorder {
            format: RecorderFormat::Flac,
            enhancement: RecorderEnhancement::StudioVoice,
            input: Some("alsa_input.usb".into()),
        };
        let text = ron::ser::to_string_pretty(&recorder, Default::default())
            .expect("serialise recorder section");
        assert_eq!(ron::from_str::<Recorder>(&text).ok(), Some(recorder));
    }
}
