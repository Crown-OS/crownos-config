# display

`~/.config/crownos/display.ron`, the `Display` section. Generated from `src/schema`; do not edit.

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `brightness` | `f64` | `80.0` | Deprecated: per-output brightness replaces this value. Kept so old files load. |
| `night_light` | `bool` | `false` |  |
| `night_light_warmth` | `f64` | `50.0` |  |

## Example

The file with every key at its default:

```ron
(
    brightness: 80.0,
    night_light: false,
    night_light_warmth: 50.0,
)
```
