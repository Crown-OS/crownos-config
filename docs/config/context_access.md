# context_access

`~/.config/crownos/context_access.ron`, the `ContextAccess` section. Generated from `src/schema`; do not edit.

Which applications may use screen context, and how much of it.

The consumers are [crownpositor], which gates the context, window control
and agent cursor protocols, and [crown-contextd], which gates its API.
Both read this file live and stop a client at once when its grant goes.

A grant is stored under the text form of an `AppKey`. An application
without a grant gets the default of its kind: `sandboxed_default` for a
sandboxed client, `host_default` for every other client.

[crownpositor]: https://github.com/crown-os/crownpositor
[crown-contextd]: https://github.com/crown-os/crownos-visual-context-engine

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `ask_on_first_use` | `bool` | `true` | Ask on screen the first time an application without a grant asks for context. Off, such an application is refused. |
| `sandboxed_default` | `FirstUsePolicy` | `Deny` | What a sandboxed application without a grant gets. |
| `host_default` | `FirstUsePolicy` | `Ask` | What every other application without a grant gets. |
| `grants` | `BTreeMap<String, Grant>` | `{}` | Grants by application key, such as `exe:/usr/bin/crown-contextd` or `sandbox:flatpak/org.example.Notes`. |
| `denied` | `Vec<String>` | `[]` | Application keys the user refused. A refusal is stronger than a grant. |
| `raw_reauth_minutes` | `u32` | `5` | Minutes after which the user must authenticate again before an application gets screen pixels through the `Raw` scope. It does not apply to the `Engine` scope. |
| `migrated_agents` | `bool` | `false` | Set when the grants of `agents.ron` were copied into this file, so that a grant the user removes later does not come back. |

## Types

### `FirstUsePolicy`

What happens when an application without a grant asks for context.

| Variant | What it means |
|---|---|
| `Deny` | Refuse without a question. |
| `Ask` | Ask the user on screen, if `ask_on_first_use` is on. |

### `Grant`

The scopes the user gave one application.

| Field | Type | What it does |
|---|---|---|
| `scopes` | `Vec<Scope>` | What the application may do. |
| `expires` | `Expiry` | When the grant stops. |

### `Scope`

One thing an application may do with screen context.

| Variant | What it means |
|---|---|
| `Metadata` | Window list, app ids, focus, placement and activity counts. No titles. |
| `Text` | Window titles, selection types and recognized text. |
| `Raw` | Screen pixels. The user authenticates again after `raw_reauth_minutes`. |
| `MemoryWrite` | Add to the context memory. |
| `MemoryDelete` | Delete from the context memory. |
| `PolicyAdmin` | Change exclusions, retention and the grants of other apps. |
| `Pause` | Pause and resume observation, and nothing more. |
| `Engine` | Run the context engine: pause every stream and capture. Includes `Metadata`, `Text`, `Raw` and `Pause`. |
| `WaylandWindowControl` | Move, focus and close the windows of other apps. |
| `WaylandAgentCursor` | Drive an agent cursor. |

### `Expiry`

When a grant stops.

| Variant | What it means |
|---|---|
| `Never` (default) | The grant stays until the user removes it. |
| `Session(String)` | The grant applies only in the session with this identity, from `current_session`. The compositor also calls `ContextAccess::forget_session_grants` when a session starts. |
| `At(u64)` | The grant stops at this time, in seconds since the Unix epoch. |

## Example

The file with every key at its default:

```ron
(
    ask_on_first_use: true,
    sandboxed_default: Deny,
    host_default: Ask,
    grants: {},
    denied: [],
    raw_reauth_minutes: 5,
    migrated_agents: false,
)
```
