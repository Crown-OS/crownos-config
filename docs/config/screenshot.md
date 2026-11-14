# screenshot

`~/.config/crownos/screenshot.ron`, the `Screenshot` section. Generated from `src/schema`; do not edit.

Screenshots and screen recordings: where they land and how they are encoded.

The consumer is crownos-screenshot, which reads this file each time it runs.

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `screenshot_directory` | `Option<String>` | `None` | Folder screenshots are saved to. `None` is `$XDG_PICTURES_DIR/Screenshots`. |
| `recording_directory` | `Option<String>` | `None` | Folder recordings are saved to. `None` is `$XDG_VIDEOS_DIR/Recordings`. |
| `image_format` | `ImageFormat` | `Png` |  |
| `video_codec` | `VideoCodec` | `H264` |  |
| `video_container` | `VideoContainer` | `Mp4` |  |
| `frame_rate` | `u32` | `60` |  |
| `quality` | `RecordingQuality` | `High` |  |
| `show_cursor` | `bool` | `true` |  |
| `capture_microphone` | `bool` | `true` |  |
| `capture_system_audio` | `bool` | `true` |  |
| `countdown_seconds` | `u32` | `0` | Seconds to wait before a recording starts. |

## Types

### `ImageFormat`

| Variant | What it means |
|---|---|
| `Png` (default) |  |
| `Jpeg` |  |

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

## Example

The file with every key at its default:

```ron
(
    screenshot_directory: None,
    recording_directory: None,
    image_format: Png,
    video_codec: H264,
    video_container: Mp4,
    frame_rate: 60,
    quality: High,
    show_cursor: true,
    capture_microphone: true,
    capture_system_audio: true,
    countdown_seconds: 0,
)
```
