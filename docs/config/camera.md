# camera

`~/.config/crownos/camera.ron`, the `Camera` section. Generated from `src/schema`; do not edit.

The camera: which device, how it records, and which AI effects are on.

The consumer is crownos-camera. Effects are preferences: a machine without
the backend for one shows it as unavailable and leaves this file alone.

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `device` | `Option<String>` | `None` | The PipeWire node name of the camera. `None` is the first one found. |
| `mode` | `CaptureMode` | `Photo` |  |
| `width` | `u32` | `1920` |  |
| `height` | `u32` | `1080` |  |
| `frame_rate` | `u32` | `30` |  |
| `mirror_preview` | `bool` | `true` | Show the preview as a mirror image. Saved photos and videos are not mirrored. |
| `photo_directory` | `Option<String>` | `None` | Folder photos are saved to. `None` is `$XDG_PICTURES_DIR/Camera`. |
| `video_directory` | `Option<String>` | `None` | Folder videos are saved to. `None` is `$XDG_VIDEOS_DIR/Camera`. |
| `video_codec` | `VideoCodec` | `H264` |  |
| `video_container` | `VideoContainer` | `Mp4` |  |
| `quality` | `RecordingQuality` | `High` |  |
| `countdown_seconds` | `u32` | `0` | Seconds to wait before a photo is taken or a recording starts. |
| `background` | `BackgroundEffect` | `r#None` |  |
| `blur_strength` | `f32` | `0.6` | How strongly the background is blurred, from 0 to 1. |
| `background_image` | `Option<String>` | `None` | The image shown behind the subject when the background is replaced. |
| `enhance` | `bool` | `false` | Denoise, remove compression artifacts and correct low light. |
| `super_resolution` | `bool` | `false` |  |
| `auto_frame` | `bool` | `false` | Keep the subject centred by cropping and following their face. |
| `noise_cancellation` | `bool` | `true` |  |

## Types

### `CaptureMode`

| Variant | What it means |
|---|---|
| `Photo` (default) |  |
| `Video` |  |

### `VideoCodec`

| Variant | What it means |
|---|---|
| `H264` (default) |  |
| `Hevc` |  |
| `Av1` |  |

### `VideoContainer`

| Variant | What it means |
|---|---|
| `Mp4` (default) |  |
| `Mkv` |  |

### `RecordingQuality`

| Variant | What it means |
|---|---|
| `Low` |  |
| `Medium` |  |
| `High` (default) |  |
| `Lossless` |  |

### `BackgroundEffect`

| Variant | What it means |
|---|---|
| `None` (default) |  |
| `Blur` |  |
| `Replace` |  |

## Example

The file with every key at its default:

```ron
(
    device: None,
    mode: Photo,
    width: 1920,
    height: 1080,
    frame_rate: 30,
    mirror_preview: true,
    photo_directory: None,
    video_directory: None,
    video_codec: H264,
    video_container: Mp4,
    quality: High,
    countdown_seconds: 0,
    background: r#None,
    blur_strength: 0.6,
    background_image: None,
    enhance: false,
    super_resolution: false,
    auto_frame: false,
    noise_cancellation: true,
)
```
