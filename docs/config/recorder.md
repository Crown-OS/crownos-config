# recorder

`~/.config/crownos/recorder.ron`, the `Recorder` section. Generated from `src/schema`; do not edit.

The voice recorder: how new recordings are captured and saved.

The consumer is crownos-sound-recorder, which writes this file whenever one
of its pickers changes.

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `format` | `RecorderFormat` | `Opus` |  |
| `enhancement` | `VoiceEnhancement` | `NoiseSuppression` | The live enhancement new recordings start with, and the default of "Enhance". |
| `input` | `Option<String>` | `None` | The PipeWire node name of the microphone. `None` is the system default. |

## Types

### `RecorderFormat`

| Variant | What it means |
|---|---|
| `Opus` (default) | Opus in Ogg at voice bit rate. |
| `Flac` | Lossless 24-bit FLAC. |

### `VoiceEnhancement`

| Variant | What it means |
|---|---|
| `Off` |  |
| `Light` |  |
| `NoiseSuppression` (default) |  |
| `StudioVoice` |  |

## Example

The file with every key at its default:

```ron
(
    format: Opus,
    enhancement: NoiseSuppression,
    input: None,
)
```
