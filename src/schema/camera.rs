//! The camera: which device, how it records, and which AI effects are on.
//!
//! The consumer is crownos-camera. Effects are preferences: a machine without
//! the backend for one shows it as unavailable and leaves this file alone.

use serde::{Deserialize, Serialize};

pub use super::screenshot::{RecordingQuality, VideoCodec, VideoContainer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CaptureMode {
    #[default]
    Photo,
    Video,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BackgroundEffect {
    #[default]
    None,
    Blur,
    Replace,
}

crate::section! {
    pub struct Camera in "camera", keys CameraKey {
        /// The PipeWire node name of the camera. `None` is the first one found.
        pub device as Device: Option<String> = None,

        pub mode as Mode: CaptureMode = CaptureMode::Photo,

        pub width as Width: u32 = 1920,

        pub height as Height: u32 = 1080,

        pub frame_rate as FrameRate: u32 = 30,

        /// Show the preview as a mirror image. Saved photos and videos are not mirrored.
        pub mirror_preview as MirrorPreview: bool = true,

        /// Folder photos are saved to. `None` is `$XDG_PICTURES_DIR/Camera`.
        pub photo_directory as PhotoDirectory: Option<String> = None,

        /// Folder videos are saved to. `None` is `$XDG_VIDEOS_DIR/Camera`.
        pub video_directory as VideoDirectory: Option<String> = None,

        pub video_codec as Codec: VideoCodec = VideoCodec::H264,

        pub video_container as Container: VideoContainer = VideoContainer::Mp4,

        pub quality as Quality: RecordingQuality = RecordingQuality::High,

        /// Seconds to wait before a photo is taken or a recording starts.
        pub countdown_seconds as CountdownSeconds: u32 = 0,

        pub background as Background: BackgroundEffect = BackgroundEffect::None,

        /// How strongly the background is blurred, from 0 to 1.
        pub blur_strength as BlurStrength: f32 = 0.6,

        /// The image shown behind the subject when the background is replaced.
        pub background_image as BackgroundImage: Option<String> = None,

        /// Denoise, remove compression artifacts and correct low light.
        pub enhance as Enhance: bool = false,

        pub super_resolution as SuperResolution: bool = false,

        /// Keep the subject centred by cropping and following their face.
        pub auto_frame as AutoFrame: bool = false,

        pub noise_cancellation as NoiseCancellation: bool = true,
    }
}
