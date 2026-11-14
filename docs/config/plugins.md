# plugins

`~/.config/crownos/plugins.ron`, the `Plugins` section. Generated from `src/schema`; do not edit.

Which themes and plugins are active and what each has been allowed to do.

Written only by `crownplugind`; everything else asks it over IPC.

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `active_theme` | `Option<String>` | `None` |  |
| `enabled` | `Vec<String>` | `[]` |  |
| `grants` | `BTreeMap<String, BTreeMap<String, Decision>>` | `{}` | plugin id -> capability name -> what the user decided. |

## Types

### `Decision`

| Variant | What it means |
|---|---|
| `Granted` |  |
| `Denied` |  |

## Example

The file with every key at its default:

```ron
(
    active_theme: None,
    enabled: [],
    grants: {},
)
```
