# Themes

LazyCSV's appearance is configurable through nested `[ui]`, `[table]`, `[popup]`, `[status]`, `[file_menu]`, and `[sql]` sections in the config file. Every setting is optional — anything you omit falls back to the built-in default.

Each UI surface owns its own section.

## Sections at a Glance

| Section | Covers |
|---------|--------|
| `[ui]` | Global foreground/background and frame border color used everywhere |
| `[table]` | The main spreadsheet (cursor, selection, search, headers, zebra rows, dirty marker) |
| `[popup]` | All popup/modal dialogs — help, SQL editor frame, magnifier, file prompts, completion menus |
| `[status]` | The bottom status bar and its mode badge / error / success colors |
| `[file_menu]` | The file browser side panel |
| `[sql]` | SQL-editor specifics (line numbers, diagnostic squiggles) and SQL-related defaults |

## `[ui]` — Global Surfaces

Base colors that apply throughout the TUI.

| Option | Default | Description |
|--------|---------|-------------|
| `fg` | `"reset"` | Default foreground (`reset` = use terminal default) |
| `bg` | `"reset"` | Default background |
| `border_fg` | `"gray"` | Color of frame borders |

## `[table]` — Main Spreadsheet

| Option | Default | Description |
|--------|---------|-------------|
| `header_fg` | `"reset"` | Foreground of column-letter headers (A, B, C…) |
| `header_bg` | unset | Background of the column-letter header row |
| `header_bold` | `true` | Bold the column-letter headers |
| `zebra_bg` | `"#1e1e1e"` | Background for alternating (even) rows |
| `cursor_fg` | `"black"` | Foreground of the active cell |
| `cursor_bg` | `"white"` | Background of the active cell |
| `selection_fg` | `"yellow"` | Foreground inside a visual selection |
| `selection_bg` | `"darkgray"` | Background of a visual selection |
| `search_match_fg` | `"black"` | Foreground of a search match |
| `search_match_bg` | `"yellow"` | Background of a search match |
| `dirty_fg` | `"red"` | Color of the `*` indicator for unsaved changes |

## `[popup]` — Modal Dialogs

Used by the help screen, SQL editor frame, magnifier, file-operation prompts, and the completion menu.

| Option | Default | Description |
|--------|---------|-------------|
| `bg` | `"darkgray"` | Popup background fill |
| `fg` | `"white"` | Default text color inside popups |
| `border_fg` | `"gray"` | Popup border color |
| `title_fg` | `"white"` | Popup title color |
| `completion_sel_fg` | `"white"` | Completion menu — selected entry foreground |
| `completion_sel_bg` | `"blue"` | Completion menu — selected entry background |

## `[status]` — Bottom Status Bar

| Option | Default | Description |
|--------|---------|-------------|
| `fg` | `"reset"` | Foreground of the status bar |
| `bg` | `"reset"` | Background of the status bar |
| `mode_fg` | `"black"` | Foreground of the mode badge (NORMAL / INSERT / VISUAL …) |
| `mode_bg` | `"green"` | Background of the mode badge |
| `error_fg` | `"red"` | Color used for error messages |
| `success_fg` | `"green"` | Color used for success messages |

## `[file_menu]` — File Browser

| Option | Default | Description |
|--------|---------|-------------|
| `dir_fg` | `"blue"` | Color of directory entries |
| `highlight_fg` | `"black"` | Foreground of the highlighted entry |
| `highlight_bg` | `"white"` | Background of the highlighted entry |
| `separator_fg` | `"gray"` | Color of column separators |
| `status_bg` | `"darkgray"` | Background of the file-menu status line |
| `status_mode_bg` | `"blue"` | Background of the mode segment in the file-menu status line |
| `status_accent_bg` | `"magenta"` | Accent segment background in the file-menu status line |
| `active_indicator_fg` | `"green"` | Marker color for the currently-open file |
| `preview_cols` | `["blue","green","yellow","cyan","magenta","red","lightblue","lightgreen"]` | Eight column-rotation colors used in the CSV preview |

`preview_cols` must be a list of **exactly 8** colors. Lists of any other length are rejected with a warning and the defaults are kept.

## `[sql]` — SQL Editor

The SQL section also doubles as the top-level SQL config (history limit, formatting), so behavioural and visual settings live together here.

| Option | Default | Description |
|--------|---------|-------------|
| `format_uppercase` | `true` | Format SQL keywords as uppercase |
| `sql_history_limit` | `15` | Maximum number of SQL queries kept in history (0 = disabled) |
| `line_number_fg` | `"darkgray"` | Color of line numbers in the SQL editor |
| `diagnostic_error_fg` | `"red"` | Color of error squiggles and error status text |
| `diagnostic_warning_fg` | `"yellow"` | Color of warning squiggles and warning status text |

## Color Format

Colors can be specified as named colors or hex RGB values.

**Named colors:**

`"black"`, `"white"`, `"red"`, `"green"`, `"blue"`, `"yellow"`, `"cyan"`, `"magenta"`, `"gray"` / `"grey"`, `"darkgray"` / `"darkgrey"`, `"lightred"`, `"lightgreen"`, `"lightyellow"`, `"lightblue"`, `"lightmagenta"`, `"lightcyan"`

CSS-style extras are also supported: `"silver"`, `"dimgray"`, `"crimson"`, `"pink"`, `"hotpink"`, `"firebrick"`, `"darkblue"`, `"teal"`, `"lime"`, `"forestgreen"`, `"seagreen"`, `"olive"`, `"gold"`, `"orange"`, `"darkorange"`, `"lemonchiffon"`, `"purple"`, `"rebeccapurple"`, `"indigo"`, `"brown"`, `"maroon"`, `"sandybrown"`, `"beige"`, `"antiquewhite"`, `"aqua"`, `"fuchsia"`.

**Hex RGB:**

Six-digit hex with a `#` prefix: `"#1e1e1e"`, `"#ff0000"`, `"#d79921"`.

**Reset:**

`"reset"` keeps the terminal's default — useful for `bg` so the application is transparent over a custom terminal background.

Color names are case-insensitive (`"White"`, `"WHITE"`, and `"white"` all work).

## Applying a Theme

Place theme settings in your config file:

- **Global:** `~/.config/lazycsv/config.toml`
- **Per-directory:** `.lazycsv.toml` in the current working directory (overrides global)

Only include the settings you want to change. LazyCSV reloads the config automatically when the file is saved, so changes take effect without restarting.

## Breaking Change Notice

Older versions of LazyCSV used a single flat `[theme]` section with prefixed keys (e.g. `cursor_bg`, `file_menu_dir_fg`). That layout has been **removed**. To migrate:

| Old key (`[theme]`) | New location |
|---------------------|--------------|
| `cursor_bg`, `cursor_fg`, `selection_bg`, `selection_fg`, `search_match_bg`, `search_match_fg`, `zebra_bg`, `header_bold`, `header_bg` | `[table]` (same names) |
| `dirty_indicator_fg` | `[table].dirty_fg` |
| `file_menu_dir_fg`, `file_menu_highlight_fg`, `file_menu_highlight_bg`, `file_menu_separator_fg`, `file_menu_status_bg`, `file_menu_status_mode_bg`, `file_menu_status_accent_bg`, `file_menu_active_indicator_fg` | `[file_menu]` (drop the `file_menu_` prefix) |
| `file_menu_preview_col_1` … `file_menu_preview_col_8` | `[file_menu].preview_cols = [..., ..., ..., ..., ..., ..., ..., ...]` (one 8-element array) |

Old keys are silently ignored — you will see no warnings, just defaults.

## Example Themes

### Gruvbox Dark

Warm, retro-style palette with muted earth tones.

```toml
# ~/.config/lazycsv/config.toml

[ui]
border_fg = "#7c6f64"

[table]
zebra_bg        = "#282828"
cursor_bg       = "#d79921"
cursor_fg       = "#282828"
selection_bg    = "#504945"
selection_fg    = "#ebdbb2"
search_match_bg = "#b57614"
search_match_fg = "#fbf1c7"
header_bold     = true
header_bg       = "#3c3836"
dirty_fg        = "#fb4934"

[popup]
bg        = "#3c3836"
fg        = "#ebdbb2"
border_fg = "#7c6f64"
title_fg  = "#fabd2f"

[status]
mode_fg = "#282828"
mode_bg = "#b8bb26"
```

### Solarized Dark

The classic Solarized palette with precise, low-contrast base tones.

```toml
# ~/.config/lazycsv/config.toml

[ui]
border_fg = "#586e75"

[table]
zebra_bg        = "#073642"
cursor_bg       = "#268bd2"
cursor_fg       = "#fdf6e3"
selection_bg    = "#586e75"
selection_fg    = "#eee8d5"
search_match_bg = "#b58900"
search_match_fg = "#002b36"
header_bold     = true
header_bg       = "#002b36"
dirty_fg        = "#dc322f"

[popup]
bg        = "#073642"
fg        = "#eee8d5"
border_fg = "#586e75"
title_fg  = "#b58900"

[status]
mode_fg = "#fdf6e3"
mode_bg = "#268bd2"
```

### Nord

Cool, blue-toned Arctic palette with soft contrast.

```toml
# ~/.config/lazycsv/config.toml

[ui]
border_fg = "#4c566a"

[table]
zebra_bg        = "#2e3440"
cursor_bg       = "#88c0d0"
cursor_fg       = "#2e3440"
selection_bg    = "#4c566a"
selection_fg    = "#eceff4"
search_match_bg = "#ebcb8b"
search_match_fg = "#2e3440"
header_bold     = true
header_bg       = "#3b4252"
dirty_fg        = "#bf616a"

[popup]
bg        = "#3b4252"
fg        = "#eceff4"
border_fg = "#4c566a"
title_fg  = "#88c0d0"

[status]
mode_fg = "#2e3440"
mode_bg = "#a3be8c"
```

## Tips for Custom Themes

- **Start from an example.** Copy one of the themes above into your config and adjust individual values.
- **Cursor contrast is critical.** Make sure `[table].cursor_bg` and `cursor_fg` have strong contrast — the cursor is the primary navigation indicator.
- **`zebra_bg` should be subtle.** A slight tint from the terminal background is enough; too much contrast makes the grid hard to scan.
- **Use `"reset"` for `[ui].bg` / `[status].bg`** if you want LazyCSV to inherit your terminal's transparency / background image.
- **`[table].header_bg` is optional.** Leaving it unset lets the header row inherit the terminal's default background, which blends naturally with many color schemes.
- **Test search matches.** Open a file and run a search (`/`) to confirm `search_match_bg`/`search_match_fg` are readable against zebra and cursor colors.
- **Hex values give precise control.** Named colors map to your terminal's 16-color palette and vary between terminal emulators; hex values are exact and consistent.
