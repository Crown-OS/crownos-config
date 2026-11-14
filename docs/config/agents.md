# agents

`~/.config/crownos/agents.ron`, the `Agents` section. Generated from `src/schema`; do not edit.

AI agents on this desktop: which programs may drive it, and which
applications may show their UI to agents.

The consumer is [crownpositor], which reads this file live. It gates the
agent protocols (window control, agent cursors, the context stream) by the
executable behind a client, and answers `crownos_agent_access_v1` requests
from the remembered decisions here, asking on screen for the rest.

[crownpositor]: https://github.com/crown-os/crownpositor

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `trusted_executables` | `Vec<String>` | see below | Programs, by absolute path, that may bind the agent protocols and so see every window, move them and drive them through their own cursors. The compositor checks the executable behind each client when it connects. |
| `ask_for_app_access` | `bool` | `true` | Ask on screen when an application asks to expose its UI to agents and no answer is remembered below. Off, such requests are denied. |
| `app_access` | `BTreeMap<String, AccessDecision>` | `{}` | Remembered answers per application id. |

Default `trusted_executables`:

```ron
["/usr/bin/crownos-computer-use","/usr/bin/crown-contextd","/usr/bin/crownlink-linux","/usr/bin/crownos-remoted"]
```

## Types

### `AccessDecision`

A remembered answer to an application asking to expose its UI to agents.

| Variant | What it means |
|---|---|
| `Allow` |  |
| `Deny` |  |

## Example

The file with every key at its default:

```ron
(
    trusted_executables: ["/usr/bin/crownos-computer-use","/usr/bin/crown-contextd","/usr/bin/crownlink-linux","/usr/bin/crownos-remoted"],
    ask_for_app_access: true,
    app_access: {},
)
```
