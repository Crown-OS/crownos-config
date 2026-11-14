# appearance

`~/.config/crownos/appearance.ron`, the `Appearance` section. Generated from `src/schema`; do not edit.

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `dark_mode` | `bool` | `true` |  |
| `accent` | `AccentColor` | `Purple` |  |
| `transparency` | `f64` | `0.0` |  |
| `wallpaper` | `String` | `""` |  |
| `bar_height` | `u32` | `32` |  |
| `gaps_inner` | `u16` | `8` | Pixels between tiled windows. |
| `gaps_outer` | `u16` | `8` | Pixels between the tiled area and the edge of the output. |
| `border_width` | `u16` | `2` |  |
| `border_radius` | `u16` | `8` |  |
| `animations` | `AnimationProfile` | `Standard` |  |
| `minimize_effect` | `MinimizeAnimation` | `Liquid` |  |
| `close_effect` | `CloseAnimation` | `Dust` |  |
| `titlebar_height` | `u16` | `36` |  |
| `snap` | `bool` | `true` |  |
| `appmenu` | `bool` | `true` |  |
| `blur` | `bool` | `true` |  |
| `blur_passes` | `u16` | `4` |  |
| `blur_size` | `f64` | `1.5` |  |
| `blur_noise` | `f64` | `0.01` |  |
| `cursor_theme` | `String` | `""` | Name of an XCursor theme under `/usr/share/icons`; empty keeps the system default. |
| `icon_theme` | `String` | `""` | Name of a freedesktop icon theme; empty keeps the system default. |

## Types

### `AccentColor`

| Variant | What it means |
|---|---|
| `Purple` (default) |  |
| `Blue` |  |
| `Green` |  |
| `Orange` |  |
| `Pink` |  |

### `AnimationProfile`

| Variant | What it means |
|---|---|
| `None` |  |
| `Snappy` |  |
| `Standard` (default) |  |
| `Smooth` |  |

### `MinimizeAnimation`

How a window travels into its icon and back out of it.

| Variant | What it means |
|---|---|
| `Liquid` (default) | A drop of glass that lifts the icon out of its dock and stretches into the window on springs, leading edge first. |
| `Genie` | Poured through a neck into the icon. |
| `Scale` | Scaled down whole into the icon. |

### `CloseAnimation`

How a closed application window leaves the screen.

| Variant | What it means |
|---|---|
| `Dust` (default) | Crumbles into particles that fall away. |
| `Shrink` | Shrinks into its icon. |

## Example

The file with every key at its default:

```ron
(
    dark_mode: true,
    accent: Purple,
    transparency: 0.0,
    wallpaper: "",
    bar_height: 32,
    gaps_inner: 8,
    gaps_outer: 8,
    border_width: 2,
    border_radius: 8,
    animations: Standard,
    minimize_effect: Liquid,
    close_effect: Dust,
    titlebar_height: 36,
    snap: true,
    appmenu: true,
    blur: true,
    blur_passes: 4,
    blur_size: 1.5,
    blur_noise: 0.01,
    cursor_theme: "",
    icon_theme: "",
)
```
