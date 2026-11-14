# bar

`~/.config/crownos/bar.ron`, the `Bar` section. Generated from `src/schema`; do not edit.

The status bar: where it sits, how it is shaped and what is on it.

Widgets are named by id. Built-ins use their own name (`"clock"`); a
plugin's widget is `"plugin:<plugin id>/<widget id>"`. A built-in missing
from all three lists is hidden.

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `height` | `Option<u32>` | `None` | Overrides `appearance.bar_height`. |
| `position` | `BarPosition` | `Top` |  |
| `floating` | `bool` | `false` | Detach the bar from the screen edge by `margin`. |
| `margin` | `u16` | `0` |  |
| `radius` | `Option<u16>` | `None` |  |
| `left` | `Vec<String>` | `["clock","live"]` |  |
| `center` | `Vec<String>` | `[]` |  |
| `right` | `Vec<String>` | see below |  |

Default `right`:

```ron
["layout","weather","caffeine","volume","brightness","bluetooth","wifi","battery","notifications"]
```

## Types

### `BarPosition`

| Variant | What it means |
|---|---|
| `Top` (default) |  |
| `Bottom` |  |

## Example

The file with every key at its default:

```ron
(
    height: None,
    position: Top,
    floating: false,
    margin: 0,
    radius: None,
    left: ["clock","live"],
    center: [],
    right: ["layout","weather","caffeine","volume","brightness","bluetooth","wifi","battery","notifications"],
)
```
