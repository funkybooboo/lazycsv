# Configuration

LazyCSV supports optional configuration via TOML files. It works great out of the box — configuration is only needed if you want to customize behavior.

## Config File Locations

LazyCSV loads configuration from two locations, merged in order:

1. **Global config:** `~/.config/lazycsv/config.toml` (defaults and themes)
2. **Global keymap:** `~/.config/lazycsv/keys.toml` (custom keybindings)
3. **Per-directory config:** `.lazycsv.toml` (overrides for both)

Per-directory settings override global settings. Only specified fields are overridden — omitted fields keep their previous value.

### Platform-specific paths

| Platform | Config Directory |
|----------|------------------|
| macOS | `~/.config/lazycsv/` |
| Linux | `$XDG_CONFIG_HOME/lazycsv/` (or `~/.config/lazycsv/`) |
| Windows | `%APPDATA%\lazycsv\` |

## Customizable Keybindings (`keys.toml`)

LazyCSV's keybindings are data-driven. You can fully remap any action to any key sequence by creating a `keys.toml` file.

### Keymap Schema

The keymap is split into mode-scoped sections: `[normal]`, `[insert]`, `[visual]`, `[command]`, `[search]`, `[magnifier]`, `[file_list]`, `[sql_editor]`, and `[global]`.

```toml
[meta]
# Options: "vim" (default), "none" (start blank)
inherit = "vim"

[normal]
# Bind single keys
"ctrl+s" = "save"
# Bind multi-key chords
"gg" = "goto_first_row"
# Unbind a key
"i" = ""

[insert]
"<enter>" = "insert_commit_down"
```

### Action Catalog

Run the `:keys` command in-app to see a full list of every dispatchable action and its current binding.

See [`keybindings.md`](keybindings.md) for a complete reference of the key sequence syntax and action IDs.

## All Config Options (`config.toml`)

### `[defaults]`

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `delimiter` | string (single char) | auto-detect | CSV delimiter character. Examples: `","`, `";"`, `"\t"`, `"\|"` |
| `encoding` | string | `"utf-8"` | File encoding. Examples: `"utf-8"`, `"latin1"`, `"windows-1252"` |
| `zebra_striping` | bool | `true` | Alternate row background colors for readability |
| `max_column_width` | integer (>= 4) | `100` | Maximum column width in characters |
| `undo_limit` | integer (>= 1) | `1000` | Maximum number of undo steps in the SQL editor |

### Theme sections

LazyCSV's theme is split across nested sections by UI surface. See [`themes.md`](themes.md) for a complete reference and example palettes.

| Section | Purpose |
|---------|---------|
| `[ui]` | Global foreground/background and frame border |
| `[table]` | The main spreadsheet (cursor, selection, search, headers, zebra rows, dirty marker) |
| `[popup]` | All popup/modal dialogs (help, SQL editor frame, magnifier, file prompts, completion menu) |
| `[status]` | Bottom status bar (mode badge, error/success messages) |
| `[file_menu]` | The file browser side panel |
| `[sql]` | SQL editor specifics (line numbers, diagnostic colors) |

### `[sql]`

The `[sql]` section also holds SQL-editor behaviour settings:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `format_uppercase` | bool | `true` | Uppercase SQL keywords when formatting (Ctrl+F) |
| `sql_history_limit` | integer | `15` | Maximum SQL queries kept in history (0 disables) |

## Color Values

Colors can be specified as:

**Named colors:** `"black"`, `"white"`, `"red"`, `"green"`, `"blue"`, `"yellow"`, `"cyan"`, `"magenta"`, `"gray"` / `"grey"`, `"darkgray"` / `"darkgrey"`, `"lightred"`, `"lightgreen"`, `"lightyellow"`, `"lightblue"`, `"lightmagenta"`, `"lightcyan"`

**Hex RGB:** `"#1e1e1e"`, `"#ff0000"`, `"#00ff00"` (6-digit hex with `#` prefix)

Color names are case-insensitive (`"White"`, `"WHITE"`, and `"white"` all work).

## Example Config

```toml
# ~/.config/lazycsv/config.toml

[defaults]
delimiter = ","
zebra_striping = true
max_column_width = 120
undo_limit = 2000

[ui]
border_fg = "gray"

[table]
cursor_bg       = "#4488ff"
cursor_fg       = "white"
selection_bg    = "#555555"
selection_fg    = "yellow"
search_match_bg = "#ffaa00"
search_match_fg = "black"
header_bg       = "#333333"
dirty_fg        = "red"

[popup]
bg        = "#222222"
border_fg = "gray"

[status]
mode_fg = "black"
mode_bg = "green"

[sql]
format_uppercase = true
```

## Per-Directory Override

Create a `.lazycsv.toml` in any directory to override specific settings for files opened from that directory. Only include the settings you want to change:

```toml
# .lazycsv.toml — use semicolons for European CSV files in this project
[defaults]
delimiter = ";"
```

## Error Handling

- **Missing config files:** silently uses defaults (this is normal)
- **Malformed TOML:** the file is skipped and a warning is shown in the status bar
- **Invalid values** (bad color strings, out-of-range numbers): the specific field keeps its default and a warning is shown
- **Unknown keys:** silently ignored (forward-compatible with future versions)
- Config errors never prevent LazyCSV from starting
