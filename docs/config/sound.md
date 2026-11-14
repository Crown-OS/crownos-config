# sound

`~/.config/crownos/sound.ron`, the `Sound` section. Generated from `src/schema`; do not edit.

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `output_volume` | `f64` | `50.0` |  |
| `input_volume` | `f64` | `50.0` |  |
| `muted` | `bool` | `false` |  |
| `output_device` | `Option<String>` | `None` |  |

## Example

The file with every key at its default:

```ron
(
    output_volume: 50.0,
    input_volume: 50.0,
    muted: false,
    output_device: None,
)
```
