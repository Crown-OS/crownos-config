# files

`~/.config/crownos/files.ron`, the `Files` section. Generated from `src/schema`; do not edit.

The file manager: how folders are listed and what happens when a drive is
plugged in.

The consumer is crownos-files, which follows this file while it runs.

## Keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `show_hidden` | `bool` | `false` | List dotfiles and other hidden entries. |
| `sort_by` | `SortKey` | `Name` | What folders are ordered by. |
| `sort_reverse` | `bool` | `false` | Reverse the order `sort_by` gives. |
| `dirs_first` | `bool` | `true` | Keep folders above files whatever the sort key. |
| `view_mode` | `ViewMode` | `Grid` | Icons or rows. |
| `automount` | `bool` | `true` | Mount removable drives as soon as they are plugged in. |
| `automount_open` | `bool` | `false` | Open a window on a drive after mounting it automatically. |
| `confirm_permanent_delete` | `bool` | `true` | Ask before deleting without the trash. |
| `tags` | `Vec<TagColor>` | see below | The tags offered in the tag menu and the color each is drawn in. |
| `pinned` | `Vec<PathBuf>` | `[]` | Folders pinned to the sidebar, in the order they are listed. |

Default `tags`:

```ron
[(name:"Red",rgb:15026253),(name:"Orange",rgb:14251782),(name:"Yellow",rgb:16498468),(name:"Green",rgb:4906624),(name:"Blue",rgb:3900150),(name:"Purple",rgb:9399803),(name:"Gray",rgb:10592682)]
```

## Types

### `SortKey`

| Variant | What it means |
|---|---|
| `Name` (default) |  |
| `Modified` |  |
| `Created` |  |
| `Size` |  |
| `Kind` |  |

### `ViewMode`

| Variant | What it means |
|---|---|
| `Grid` (default) |  |
| `List` |  |

### `TagColor`

| Field | Type | What it does |
|---|---|---|
| `name` | `String` |  |
| `rgb` | `u32` | `0xRRGGBB`. |

## Example

The file with every key at its default:

```ron
(
    show_hidden: false,
    sort_by: Name,
    sort_reverse: false,
    dirs_first: true,
    view_mode: Grid,
    automount: true,
    automount_open: false,
    confirm_permanent_delete: true,
    tags: [(name:"Red",rgb:15026253),(name:"Orange",rgb:14251782),(name:"Yellow",rgb:16498468),(name:"Green",rgb:4906624),(name:"Blue",rgb:3900150),(name:"Purple",rgb:9399803),(name:"Gray",rgb:10592682)],
    pinned: [],
)
```
