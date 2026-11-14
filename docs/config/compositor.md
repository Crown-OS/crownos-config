# compositor

`~/.config/crownos/compositor.ron`, the `Compositor` section. Generated from `src/schema`; do not edit.

Window management: tiling, keybindings, window rules, outputs.

How windows are *arranged*, not how they look — gaps, borders and animation
are in `appearance`(crate::schema::appearance), because a user changing them
is changing the theme rather than the tiling.

The consumer is [crownpositor], which reads this file live — a rebind takes
effect without a restart. It keeps its own compiled form of these values
(regexes, chords, geometry in signed pixels); this is only the on-disk
vocabulary, so the file stays a stable contract while the compositor's
internal types move around.

[crownpositor]: https://github.com/crown-os/crownpositor

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `layout` | `WorkspaceMode` | `Tiling` |  |
| `focus_follows_mouse` | `bool` | `false` |  |
| `allow_overlapping_outputs` | `bool` | `false` | Kanshi-style configurations sometimes overlap outputs by a pixel on purpose. The compositor's model cannot represent that, so it is refused unless this says otherwise. |
| `window_rules` | `Vec<WindowRule>` | see below |  |
| `outputs` | `Vec<OutputSetting>` | `[]` |  |
| `output_layouts` | `Vec<OutputLayout>` | `[]` |  |
| `startup` | `Vec<String>` | `[]` |  |
| `gaming` | `GamingOptions` | `(allow_tearing:true)` |  |

Default `window_rules`:

```ron
[(app_id:Some("^steam_app_"),tearing:Some(true),vrr:Some(true))]
```

## Types

### `WorkspaceMode`

How a workspace arranges the windows on it.

| Variant | What it means |
|---|---|
| `Tiling` (default) | The compositor owns every window's geometry. |
| `Floating` | Windows keep the size and position they were given, and wear a titlebar. |

### `WindowRule`

One row of the keybinding table. Both fields are strings so the file stays
hand-editable; a bad row is logged and skipped rather than failing the load.

Deliberately not a `Keybind`(crate::Keybind): the compositor's actions are
its own vocabulary, and the chord spellings it accepts (`"Super+Shift+Q"`)
are a superset of what the settings panel's single-shortcut fields need.
Matched at a window's first buffer commit, the earliest point `app_id`,
`title` and the size hints exist.

| Field | Type | What it does |
|---|---|---|
| `app_id` | `Option<String>` | Regex, unanchored: `"blender"` matches `"org.blender.Blender"`. |
| `title` | `Option<String>` | Regex. |
| `floating` | `Option<bool>` |  |
| `fullscreen` | `Option<bool>` |  |
| `maximized` | `Option<bool>` |  |
| `workspace` | `Option<u16>` | Zero-based workspace index on the target output. |
| `output` | `Option<String>` | Connector name or `"MAKE MODEL SERIAL"`. |
| `focus` | `Option<bool>` | `false` opens the window without stealing focus. |
| `opacity` | `Option<f32>` |  |
| `corner_radius` | `Option<u16>` |  |
| `tearing` | `Option<bool>` | Present this window's frames without waiting for vblank while it is fullscreen. For X11 games, which cannot ask through `wp_tearing_control_v1`. Needs `GamingOptions::allow_tearing`. |
| `vrr` | `Option<bool>` | Treat this window as a game for `Vrr::OnDemand`, fullscreen or not. |

### `OutputSetting`

| Field | Type | What it does |
|---|---|---|
| `name` | `String` | Connector name (`"eDP-1"`) or `"MAKE MODEL SERIAL"`. |
| `enabled` | `Option<bool>` |  |
| `mode` | `Option<String>` | `"2560x1440@144.000"`. |
| `scale` | `Option<f64>` |  |
| `transform` | `Option<OutputTransform>` |  |
| `position` | `Option<(i32, i32)>` | Fallback position, used when no `OutputLayout` matches the monitors that are actually plugged in. |
| `vrr` | `Option<Vrr>` |  |
| `mirror_of` | `Option<String>` | Show this output's contents on another one. Names it the same way `Self::name` does. |
| `max_bpc` | `Option<u8>` | DRM `max bpc`. Set 8 for a panel that flickers at 10 or 12. |
| `hdr` | `Option<bool>` | Drive the panel in BT.2100. Refused when its EDID claims no HDR. |
| `sdr_reference_luminance` | `Option<u32>` | cd/m² that SDR white maps to in HDR mode. BT.2408 says 203. |
| `icc_profile` | `Option<String>` | Path to this output's ICC calibration profile. |
| `focus_at_startup` | `Option<bool>` | Take focus at startup instead of the first output. |
| `layout` | `Option<WorkspaceMode>` | Overrides the global default for workspaces created on this output. |

### `OutputLayout`

Where the monitors sit, remembered per *set* of monitors.

Position is the one setting that depends on which other displays are
present — a laptop docked to two screens wants a different arrangement from
the same laptop on its own, while its scale and mode do not change. So
positions live here, keyed by the set, and everything else stays in
`OutputSetting` where it can be hand-edited once.

A `Vec` rather than a map keyed by the set, because RON cannot express a
map with a composite key readably, and these files are meant to be edited
by hand.

| Field | Type | What it does |
|---|---|---|
| `heads` | `Vec<String>` | The outputs present, named as in `OutputSetting::name`. Matched as a set, so order does not matter. |
| `positions` | `Vec<(String, i32, i32)>` | `(name, x, y)` for each output enabled in this arrangement. |
| `disabled` | `Vec<String>` | Outputs switched off in this arrangement. |

### `GamingOptions`

Latency trade-offs for games.

| Field | Type | What it does |
|---|---|---|
| `allow_tearing` | `bool` | Let a fullscreen window that asks for it, or that a rule names, tear instead of waiting for vblank. |

### `OutputTransform`

| Variant | What it means |
|---|---|
| `Normal` (default) |  |
| `R90` |  |
| `R180` |  |
| `R270` |  |
| `Flipped` |  |
| `Flipped90` |  |
| `Flipped180` |  |
| `Flipped270` |  |

### `Vrr`

When a display runs at a variable refresh rate.

Tri-state rather than a bool because "always on" and "on when it helps" are
genuinely different: some panels flicker at low refresh on a static
desktop, so `OnDemand` turns it on only while a fullscreen window is the
only thing on the output, or a game is visible.

| Variant | What it means |
|---|---|
| `Off` |  |
| `On` |  |
| `OnDemand` (default) |  |

## Example

The file with every key at its default:

```ron
(
    layout: Tiling,
    focus_follows_mouse: false,
    allow_overlapping_outputs: false,
    window_rules: [(app_id:Some("^steam_app_"),tearing:Some(true),vrr:Some(true))],
    outputs: [],
    output_layouts: [],
    startup: [],
    gaming: (allow_tearing:true),
)
```
