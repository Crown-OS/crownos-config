# glass

`~/.config/crownos/glass.ron`, the `Glass` section. Generated from `src/schema`; do not edit.

The lit edge of the compositor's frosted glass.

Every piece of glass the compositor draws — window frames, a client's own
blurred background, panels, menus — has a bevelled rim that refracts what is
behind it and catches the light. This is how that rim looks, separately for
each kind of surface, so a panel can glow while windows stay quiet.

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `windows` | `EdgeGlow` | see below | Window frames and the blurred backgrounds windows ask for. |
| `panels` | `EdgeGlow` | see below | Layer-shell surfaces: bars, docks, launchers, notifications. |
| `menus` | `EdgeGlow` | see below | Popups and the compositor's own menus. |

Default `windows`:

```ron
(enabled:true,width:6.0,intensity:0.5,light_angle:0.0,adaptive_tint:0.1,inner_shadow:0.0)
```

Default `panels`:

```ron
(enabled:true,width:6.0,intensity:0.6,light_angle:0.0,adaptive_tint:0.1,inner_shadow:0.0)
```

Default `menus`:

```ron
(enabled:true,width:3.0,intensity:0.4,light_angle:0.0,adaptive_tint:0.1,inner_shadow:0.0)
```

## Types

### `EdgeGlow`

The rim of one kind of glass.

| Field | Type | What it does |
|---|---|---|
| `enabled` | `bool` | Off leaves the edge flat: no bevel, no highlight, no inner shadow. |
| `width` | `f64` | Width of the bevel in logical pixels. |
| `intensity` | `f64` | Brightness of the highlight along the edge, 0 to 1. |
| `light_angle` | `f64` | Where the light comes from, in degrees clockwise from straight up. The highlight sits on the two edges facing along this line, and the light it lets in pools as a soft glow inside the far one. |
| `adaptive_tint` | `f64` | How much the highlight takes its colour from what is behind the glass: 0 is white light, 1 is lit entirely in the colours of the window below. |
| `inner_shadow` | `f64` | Depth of the shade just inside the edge, 0 to 1. |

## Example

The file with every key at its default:

```ron
(
    windows: (enabled:true,width:6.0,intensity:0.5,light_angle:0.0,adaptive_tint:0.1,inner_shadow:0.0),
    panels: (enabled:true,width:6.0,intensity:0.6,light_angle:0.0,adaptive_tint:0.1,inner_shadow:0.0),
    menus: (enabled:true,width:3.0,intensity:0.4,light_angle:0.0,adaptive_tint:0.1,inner_shadow:0.0),
)
```
