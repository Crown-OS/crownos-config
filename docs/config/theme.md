# theme

`~/.config/crownos/theme.ron`, the `Theme` section. Generated from `src/schema`; do not edit.

Semantic design tokens shared by every CrownOS surface.

The palettes themselves ship with the toolkits; this section only holds
sparse edits to them, keyed by dotted slot names such as `surface.bg`, plus
the shape and type tokens a theme can retune. A slot left out keeps the
toolkit's built-in color, so an empty file is the stock look.

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `accent` | `Option<AccentGradient>` | `None` | Replaces the accent chosen in `appearance.ron`. |
| `light` | `BTreeMap<String, String>` | `{}` | Slot edits applied only to the light palette. |
| `dark` | `BTreeMap<String, String>` | `{}` | Slot edits applied only to the dark palette. |
| `radii` | `Radii` | `(sm:6.0,md:10.0,lg:16.0,xl:24.0)` |  |
| `font` | `Font` | `(family:"Inter",mono:"JetBrains Mono",scale:1.0)` |  |

## Types

### `AccentGradient`

A custom accent gradient, as `#rrggbb` or `#rrggbbaa` strings.

| Field | Type | What it does |
|---|---|---|
| `start` | `String` | The lighter stop, painted toward the top or left of an accent surface. |
| `end` | `String` | The deeper stop. |

### `Radii`

Corner radii in logical pixels, from small controls to whole panels.

| Field | Type | What it does |
|---|---|---|
| `sm` | `f32` |  |
| `md` | `f32` |  |
| `lg` | `f32` |  |
| `xl` | `f32` |  |

### `Font`

| Field | Type | What it does |
|---|---|---|
| `family` | `String` |  |
| `mono` | `String` |  |
| `scale` | `f32` | Multiplies every text size. |

## Example

The file with every key at its default:

```ron
(
    accent: None,
    light: {},
    dark: {},
    radii: (sm:6.0,md:10.0,lg:16.0,xl:24.0),
    font: (family:"Inter",mono:"JetBrains Mono",scale:1.0),
)
```
