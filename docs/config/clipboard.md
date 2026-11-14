# clipboard

`~/.config/crownos/clipboard.ron`, the `Clipboard` section. Generated from `src/schema`; do not edit.

The clipboard manager: how much of what was copied outlives the app that
copied it.

The consumer is [crownos-clipboard], which reads this file at startup.

[crownos-clipboard]: https://github.com/crown-os/crownos-clipboard

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `history_size` | `u32` | `50` | How many past copies to remember. Zero keeps only the current one. |
| `max_entry_mib` | `u32` | `16` | The largest copy kept, in MiB, counting every format it was offered in. Anything bigger still pastes while its app is running but is neither remembered nor kept alive after it exits. |
| `persist_primary` | `bool` | `false` | Also keep the primary selection (middle-click paste) alive after its app exits. Off by default: it changes on every text highlight. |

## Example

The file with every key at its default:

```ron
(
    history_size: 50,
    max_entry_mib: 16,
    persist_primary: false,
)
```
