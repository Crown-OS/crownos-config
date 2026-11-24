# app_actions

`~/.config/crownos/app_actions.ron`, the `AppActions` section. Generated from `src/schema`; do not edit.

The actions that applications registered through `crownos_keybind_v1`, and
the chord each one has.

The writer is [crownpositor]. It writes this file each time an application
registers or removes an action, or a chord changes. To change the chord of
an action, add a row to `keybinds.app_keybinds`, not to this file: the
compositor writes the row of a running action again at the next change.

An action stays in the list after its application stops, so the settings
panel can show it and the user can change its chord at any time. To remove
the action of an application that is not running, delete its row here.

[crownpositor]: https://github.com/crown-os/crownpositor

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `actions` | `Vec<AppAction>` | `[]` | Every action that an application registered, in the order they were first registered. |

## Types

### `AppAction`

One action that an application registered.

| Field | Type | What it does |
|---|---|---|
| `app_id` | `String` | The app_id the application registered the action with. |
| `name` | `String` | The name of the action, for example `toggle-recording`. |
| `description` | `String` | What the action does, in words for the user. |
| `preferred_keys` | `Keybind` | The chord the application asked for. `None` if it cannot be read. |
| `wanted_keys` | `Keybind` | The chord the compositor tried to give the action: the one in `keybinds.app_keybinds`, else the one the application asked for. |
| `assigned_keys` | `Keybind` | The chord the action has now, or had when its application stopped. `None` when the action has no chord. |
| `running` | `bool` | Whether the application has the action registered now. |

## Example

The file with every key at its default:

```ron
(
    actions: [],
)
```
