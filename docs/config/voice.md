# voice

`~/.config/crownos/voice.ron`, the `Voice` section. Generated from `src/schema`; do not edit.

Voice enhancement for every app's microphone.

The consumers are crown-voiced, which follows this file live, and the
settings panel's Voice page, which writes it.

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `enhance_all_apps` | `bool` | `false` | Makes an enhanced copy of the default microphone the system default source, so apps that open the default input are enhanced too. |
| `all_apps_enhancement` | `VoiceEnhancement` | `NoiseSuppression` | The enhancement the system-wide microphone runs. |
| `backend` | `VoiceBackend` | `Automatic` |  |

## Types

### `VoiceEnhancement`

| Variant | What it means |
|---|---|
| `Off` |  |
| `Light` |  |
| `NoiseSuppression` (default) |  |
| `StudioVoice` |  |

### `VoiceBackend`

Which hardware crown-voiced may run enhancement on.

| Variant | What it means |
|---|---|
| `Automatic` (default) | The best backend the installer set up and this machine can run. |
| `Cpu` | The CPU only, leaving GPUs and NPUs idle. |

## Example

The file with every key at its default:

```ron
(
    enhance_all_apps: false,
    all_apps_enhancement: NoiseSuppression,
    backend: Automatic,
)
```
