# Configuration

LazyCSV supports optional configuration via TOML files. It works great out of the box — configuration is only needed if you want to customize behavior.

## Config File Locations

LazyCSV loads configuration from two locations, merged in order:

1. **Global config:** `~/.config/lazycsv/config.toml`
2. **Per-directory config:** `.lazycsv.toml` in the current working directory

Per-directory settings override global settings. Only specified fields are overridden — omitted fields keep their previous value.

### Platform-specific paths

| Platform | Global config path |
|----------|-------------------|
| macOS | `~/.config/lazycsv/config.toml` |
| Linux | `$XDG_CONFIG_HOME/lazycsv/config.toml` (or `~/.config/lazycsv/config.toml`) |
| Windows | `%APPDATA%\lazycsv\config.toml` |

## All Config Options

### `[defaults]`

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `delimiter` | string (single char) | auto-detect | CSV delimiter character. Examples: `","`, `";"`, `"\t"`, `"\|"` |
| `encoding` | string | `"utf-8"` | File encoding. Examples: `"utf-8"`, `"latin1"`, `"windows-1252"` |
| `zebra_striping` | bool | `true` | Alternate row background colors for readability |
| `max_column_width` | integer (>= 4) | `100` | Maximum column width in characters |
| `undo_limit` | integer (>= 1) | `1000` | Maximum number of undo steps in the SQL editor |

### `[theme]`

Colors accept named colors or hex RGB values.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `zebra_bg` | color | `"#1e1e1e"` | Background for alternating (even) rows |
| `cursor_bg` | color | `"white"` | Background of the selected cell |
| `cursor_fg` | color | `"black"` | Foreground of the selected cell |
| `selection_bg` | color | `"darkgray"` | Background for visual selections |
| `selection_fg` | color | `"yellow"` | Foreground for visual selections |
| `search_match_bg` | color | `"yellow"` | Background for search matches |
| `search_match_fg` | color | `"black"` | Foreground for search matches |
| `header_bold` | bool | `true` | Bold text in column headers |
| `header_bg` | color | none | Background for the column letters row (A, B, C...) |
| `dirty_indicator_fg` | color | `"red"` | Color of the `*` dirty file indicator |

### `[sql]`

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `format_uppercase` | bool | `true` | Uppercase SQL keywords when formatting (Ctrl+F) |

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

[theme]
cursor_bg = "#4488ff"
cursor_fg = "white"
selection_bg = "#555555"
selection_fg = "yellow"
search_match_bg = "#ffaa00"
search_match_fg = "black"
header_bg = "#333333"
dirty_indicator_fg = "red"

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
