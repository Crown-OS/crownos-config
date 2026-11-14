# input

`~/.config/crownos/input.ron`, the `Input` section. Generated from `src/schema`; do not edit.

Text input: for now, dictation.

The section is called `input` rather than `dictation` because it is the file
behind the settings panel's *Input* page, and that page is the home for
anything that turns a person into text.
Dictation is simply the only thing in it today,
which is why every field is prefixed: a later `input.ron` gaining key-repeat
or an input-method setting should not have to rename what is already there.

The consumer is [crowndictator], which reads this file at startup and
follows it live — turning the feature off releases its keyboard grab, and
changing the shortcut re-arms it without a restart.

[crowndictator]: https://github.com/crown-os/crowndictator

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `dictation_enabled` | `bool` | `true` | Whether push-to-talk dictation runs at all. Off means the daemon stays resident but the shortcut does nothing and no microphone is ever opened. The shortcut and the device below are remembered rather than forgotten, so turning it back on restores the setup that was already there. |
| `dictation_microphone` | `Option<String>` | `None` | Which microphone to record from, by its PipeWire node name (such as `alsa_input.pci-0000_00_1f.3.analog-stereo`, as `audio::sources()` lists it), or `None` for whatever the system's default input is. `None` is the honest default: the system default follows the user's headset in and out, and pinning a name is the exception rather than the rule. A name that no longer matches any device falls back to the default rather than failing to record. |
| `dictation_hotkey` | `Keybind` | `"Super+Space"` | The push-to-talk shortcut: held to record, released to transcribe. Held rather than struck, which is why a modifier-only chord is a reasonable thing to put here — see `Keybind`. |
| `dictation_gpu` | `bool` | `true` | Whether to run speech recognition on the GPU when one is available. The same switch as crowndictator's `--cpu` flag, from the other direction. Falling back to the CPU is automatic when there is no usable GPU, so this only matters on a machine that has one and wants it left alone. |
| `keyboard_repeat_rate` | `u16` | `200` |  |

## Example

The file with every key at its default:

```ron
(
    dictation_enabled: true,
    dictation_microphone: None,
    dictation_hotkey: "Super+Space",
    dictation_gpu: true,
    keyboard_repeat_rate: 200,
)
```
