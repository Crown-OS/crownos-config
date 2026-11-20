# lock

`~/.config/crownos/lock.ron`, the `Lock` section. Generated from `src/schema`; do not edit.

The look and behaviour of the lock screen, drawn by the compositor.

The compositor reads this section live. `crownplugind` adds what only it
knows — a theme's layout tree and plugin slots, with their images — over
`crownos_plugin_host_v1`.

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `clock` | `ClockLook` | see below |  |
| `time_format` | `String` | `"%H:%M"` | strftime pattern. |
| `date_format` | `String` | `"%a, %b %-d"` |  |
| `background` | `LockBackground` | `Desktop` |  |
| `dim` | `f32` | `0.0` | Darkening over the background, `0.0..=1.0`. |
| `auth_field` | `Placement` | `(anchor:Center,offset:(0.0,0.0))` |  |
| `slots` | `Vec<SlotPlacement>` | `[]` |  |
| `show_user` | `bool` | `true` | Shows the user's name above the password field. |
| `prompt_timeout_secs` | `u32` | `20` | Seconds without input before the prompt gives way to the clock again. |
| `use_on_login_screen` | `bool` | `true` |  |
| `show_notifications` | `bool` | `false` |  |
| `idle_lock_after_secs` | `u32` | `300` | Seconds of inactivity before locking; 0 means never. |
| `lock_on_suspend` | `bool` | `true` |  |

Default `clock`:

```ron
(layout:Centered,font:Sans,weight:700.0,stretch:3.5,whiteness:0.5,colon:true,shadow:0.0,frame:(offset:(0.0,0.0),scale:1.0,warp:((0.0,0.0),(0.0,0.0),(0.0,0.0),(0.0,0.0)),hours:(0.0,0.0),minutes:(0.0,0.0)),depth:true)
```

## Types

### `ClockLook`

| Field | Type | What it does |
|---|---|---|
| `layout` | `ClockLayout` |  |
| `font` | `ClockFont` |  |
| `weight` | `f32` | From hairline, 100, to heavy, 900. |
| `stretch` | `f32` | How much taller than the font draws them the digits stand: stems lengthen while bowls stay round and strokes keep their width. |
| `whiteness` | `f32` | From frosted glass showing the background through the digits, 0, to solid white, 1. |
| `colon` | `bool` | Draws the separator between side-by-side hours and minutes. |
| `shadow` | `f32` | Strength of the shadow under the digits, `0.0..=1.0`. |
| `frame` | `ClockFrame` |  |
| `depth` | `bool` | Stands the background's subject, cut out of it on the fly, in front of the clock. |

### `LockBackground`

| Variant | What it means |
|---|---|
| `Desktop` (default) | The user's desktop wallpaper. |
| `Image(String)` | Any image the user can read. |
| `Token(String)` | A flat theme color token. |

### `Placement`

An anchor plus an offset from it in logical pixels.

| Field | Type | What it does |
|---|---|---|
| `anchor` | `Anchor` |  |
| `offset` | `[f32; 2]` |  |

### `SlotPlacement`

The user's placement of one plugin slot, overriding the plugin's default.

| Field | Type | What it does |
|---|---|---|
| `widget` | `String` | `plugin:<id>/<widget>`. |
| `position` | `LockSlotPosition` |  |
| `offset` | `[f32; 2]` |  |

### `ClockLayout`

Where the hours, the minutes and the date sit against each other.

| Variant | What it means |
|---|---|
| `Centered` (default) | Hours and minutes side by side across the middle, under the date. |
| `Left` | Hours and minutes side by side at the left, over the date. |
| `Stacked` | Hours over minutes at the left, the date beside the hours. |

### `ClockFont`

The typeface the clock's digits are drawn in.

| Variant | What it means |
|---|---|
| `Sans` (default) | Condensed monoline digits with round bowls. |
| `Grotesk` | Monoline digits with a grotesque's squarer bowls. |
| `Serif` | High-contrast digits: heavy stems and hairline curves. |
| `Higher` | Condensed square monoline digits, their bars set low, after Marisa Passos's Higher. |
| `Apache` | Heavy block digits with chamfered corners, after Aldo the Apache. |
| `Boston` | After Eotype's Boston. |
| `Tribo` | Condensed monoline digits with chamfered corners, after Tribo, which draws no digits of its own. |

### `ClockFrame`

Where the clock sits and how it is bent, as the Settings handles leave it.

| Field | Type | What it does |
|---|---|---|
| `offset` | `[f32; 2]` | Logical pixels from where the layout puts the clock. |
| `scale` | `f32` |  |
| `warp` | `[[f32; 2]; 4]` | How far each corner of the clock's box moves — top left, top right, bottom right, bottom left — as a fraction of the box's size. |
| `hours` | `[f32; 2]` | Logical pixels the hours move on their own. |
| `minutes` | `[f32; 2]` | Logical pixels the minutes move on their own. |

### `Anchor`

| Variant | What it means |
|---|---|
| `TopLeft` |  |
| `Top` |  |
| `TopRight` |  |
| `Left` |  |
| `Center` (default) |  |
| `Right` |  |
| `BottomLeft` |  |
| `Bottom` |  |
| `BottomRight` |  |

### `LockSlotPosition`

Where a plugin's lock-screen slot sits.

| Variant | What it means |
|---|---|
| `Clock` | Replaces the clock. |
| `AboveClock` |  |
| `BelowClock` (default) |  |
| `Bottom` |  |

## Example

The file with every key at its default:

```ron
(
    clock: (layout:Centered,font:Sans,weight:700.0,stretch:3.5,whiteness:0.5,colon:true,shadow:0.0,frame:(offset:(0.0,0.0),scale:1.0,warp:((0.0,0.0),(0.0,0.0),(0.0,0.0),(0.0,0.0)),hours:(0.0,0.0),minutes:(0.0,0.0)),depth:true),
    time_format: "%H:%M",
    date_format: "%a, %b %-d",
    background: Desktop,
    dim: 0.0,
    auth_field: (anchor:Center,offset:(0.0,0.0)),
    slots: [],
    show_user: true,
    prompt_timeout_secs: 20,
    use_on_login_screen: true,
    show_notifications: false,
    idle_lock_after_secs: 300,
    lock_on_suspend: true,
)
```
