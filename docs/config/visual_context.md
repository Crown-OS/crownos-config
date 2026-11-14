# visual_context

`~/.config/crownos/visual_context.ron`, the `VisualContext` section. Generated from `src/schema`; do not edit.

The visual context engine: what it may look at and how long it remembers.

Two consumers: [crownpositor] reads the exclusion rules live and enforces
them in the compositor, so a window stays out of captures even when the
engine is not running; [crown-contextd] reads everything else.

[crownpositor]: https://github.com/crown-os/crownpositor
[crown-contextd]: https://github.com/crown-os/crownos-visual-context-engine

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `enabled` | `bool` | `false` | Whether the engine observes at all. |
| `excluded_apps` | `Vec<String>` | see below | Application ids, as globs, whose windows are never captured or described: `org.keepassxc.*`, `*bank*`. |
| `private_titles` | `Vec<String>` | see below | Window titles, as regular expressions, that mark a private browsing window. Matching windows are never captured. |
| `pause_hotkey` | `String` | `"Super+Shift+period"` | The keybinding that pauses and resumes observation. |
| `retention_days` | `u32` | `30` | Days raw text and thumbnails are kept before pruning. |

Default `excluded_apps`:

```ron
["org.keepassxc.KeePassXC","org.gnome.seahorse.Application","1password","bitwarden"]
```

Default `private_titles`:

```ron
["(?i)private browsing","(?i)\\(private\\)$","(?i)incognito","(?i)inprivate","(?i)navigation privée","(?i)privates fenster","(?i)navegación privada","(?i)プライベート"]
```

## Example

The file with every key at its default:

```ron
(
    enabled: false,
    excluded_apps: ["org.keepassxc.KeePassXC","org.gnome.seahorse.Application","1password","bitwarden"],
    private_titles: ["(?i)private browsing","(?i)\\(private\\)$","(?i)incognito","(?i)inprivate","(?i)navigation privée","(?i)privates fenster","(?i)navegación privada","(?i)プライベート"],
    pause_hotkey: "Super+Shift+period",
    retention_days: 30,
)
```
