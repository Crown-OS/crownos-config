//! Screenshots and screen recordings: where they land and how they are encoded.
//!
//! The consumer is crownos-screenshot, which reads this file each time it runs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ImageFormat {
    #[default]
    Png,
    Jpeg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VideoCodec {
    #[default]
    H264,
    Hevc,
    Av1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VideoContainer {
    #[default]
    Mp4,
    Mkv,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RecordingQuality {
    Low,
    Medium,
    #[default]
    High,
    Lossless,
}

crate::section! {
    pub struct Screenshot in "screenshot", keys ScreenshotKey {
        /// Folder screenshots are saved to. `None` is `$XDG_PICTURES_DIR/Screenshots`.
        pub screenshot_directory as ScreenshotDirectory: Option<String> = None,

        /// Folder recordings are saved to. `None` is `$XDG_VIDEOS_DIR/Recordings`.
        pub recording_directory as RecordingDirectory: Option<String> = None,

        pub image_format as Format: ImageFormat = ImageFormat::Png,

        pub video_codec as Codec: VideoCodec = VideoCodec::H264,

        pub video_container as Container: VideoContainer = VideoContainer::Mp4,

        pub frame_rate as FrameRate: u32 = 60,

        pub quality as Quality: RecordingQuality = RecordingQuality::High,

        pub show_cursor as ShowCursor: bool = true,

        pub capture_microphone as CaptureMicrophone: bool = true,

        pub capture_system_audio as CaptureSystemAudio: bool = true,

        /// Seconds to wait before a recording starts.
        pub countdown_seconds as CountdownSeconds: u32 = 0,
    }
}
