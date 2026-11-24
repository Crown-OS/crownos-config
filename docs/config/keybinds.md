# keybinds

`~/.config/crownos/keybinds.ron`, the `Keybinds` section. Generated from `src/schema`; do not edit.

Desktop-wide keyboard shortcuts.

Shortcuts that belong to the desktop itself rather than to one feature —
the ones a compositor grabs globally and acts on no matter what has focus.
A shortcut that only makes sense while a particular thing is running stays
with that thing instead: dictation's push-to-talk chord lives in
`input`(crate::schema::input), because it is meaningless with dictation
switched off.

The consumer is the compositor. It is the only process that can honour a
global chord — every other CrownOS app is a Wayland client and only ever
sees keys while it is focused, which is exactly the state a launcher
shortcut has to work from *outside*. It reads this file live, so rebinding
takes effect without a restart or a re-login.

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `launcher` | `Keybind` | `"Super+Space"` |  |
| `dictation` | `Keybind` | `"Super+Ctrl"` |  |
| `custom_keybinds` | `Vec<Binding>` | `[]` |  |
| `app_keybinds` | `Vec<AppKeybind>` | `[]` |  |

## Types

### `Binding`

| Field | Type | What it does |
|---|---|---|
| `keys` | `String` |  |
| `action` | `String` |  |

### `AppKeybind`

The chord the user set for one action that an application registered
through `crownos_keybind_v1`. `keys: "None"` removes the chord of the action.

| Field | Type | What it does |
|---|---|---|
| `app_id` | `String` | The app_id the application registered the action with. |
| `name` | `String` | The name of the action, for example `toggle-recording`. |
| `keys` | `Keybind` | The chord the action gets in place of the one the application asked for. |

## Example

The file with every key at its default:

```ron
(
    launcher: "Super+Space",
    dictation: "Super+Ctrl",
    custom_keybinds: [],
    app_keybinds: [],
)
```
