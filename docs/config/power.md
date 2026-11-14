# power

`~/.config/crownos/power.ron`, the `Power` section. Generated from `src/schema`; do not edit.

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `screen_off_minutes` | `u32` | `10` |  |
| `sleep_minutes` | `u32` | `30` |  |
| `power_profile` | `PowerProfile` | `Balanced` |  |

## Types

### `PowerProfile`

| Variant | What it means |
|---|---|
| `PowerSaver` |  |
| `Balanced` (default) |  |
| `Performance` |  |

## Example

The file with every key at its default:

```ron
(
    screen_off_minutes: 10,
    sleep_minutes: 30,
    power_profile: Balanced,
)
```
