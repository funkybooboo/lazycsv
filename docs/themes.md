# Themes

LazyCSV's appearance is controlled by the `[theme]` section of the config file. All theme settings are optional — omitted settings use the built-in defaults.

## Theme Settings

| Option | Default | Description |
|--------|---------|-------------|
| `zebra_bg` | `"#1e1e1e"` | Background for alternating (even) rows |
| `cursor_bg` | `"white"` | Background of the selected cell |
| `cursor_fg` | `"black"` | Foreground of the selected cell |
| `selection_bg` | `"darkgray"` | Background for visual selections |
| `selection_fg` | `"yellow"` | Foreground for visual selections |
| `search_match_bg` | `"yellow"` | Background for search match highlights |
| `search_match_fg` | `"black"` | Foreground for search match highlights |
| `header_bold` | `true` | Bold text in column letter headers (A, B, C...) |
| `header_bg` | none | Background for the column letter header row (unset = terminal default) |
| `dirty_indicator_fg` | `"red"` | Color of the `*` indicator for unsaved changes |

## Color Format

Colors can be specified as named colors or hex RGB values.

**Named colors:**

`"black"`, `"white"`, `"red"`, `"green"`, `"blue"`, `"yellow"`, `"cyan"`, `"magenta"`, `"gray"` / `"grey"`, `"darkgray"` / `"darkgrey"`, `"lightred"`, `"lightgreen"`, `"lightyellow"`, `"lightblue"`, `"lightmagenta"`, `"lightcyan"`

**Hex RGB:**

Six-digit hex with a `#` prefix: `"#1e1e1e"`, `"#ff0000"`, `"#d79921"`

Color names are case-insensitive (`"White"`, `"WHITE"`, and `"white"` all work).

## Applying a Theme

Place theme settings in your config file under the `[theme]` section:

- **Global:** `~/.config/lazycsv/config.toml`
- **Per-directory:** `.lazycsv.toml` in the current working directory

You only need to include the settings you want to change. LazyCSV reloads the config automatically when the file is saved, so changes take effect without restarting.

## Example Themes

### Gruvbox Dark

Warm, retro-style palette with muted earth tones.

```toml
# ~/.config/lazycsv/config.toml

[theme]
zebra_bg         = "#282828"
cursor_bg        = "#d79921"
cursor_fg        = "#282828"
selection_bg     = "#504945"
selection_fg     = "#ebdbb2"
search_match_bg  = "#b57614"
search_match_fg  = "#fbf1c7"
header_bold      = true
header_bg        = "#3c3836"
dirty_indicator_fg = "#fb4934"
```

### Solarized Dark

The classic Solarized palette with precise, low-contrast base tones.

```toml
# ~/.config/lazycsv/config.toml

[theme]
zebra_bg         = "#073642"
cursor_bg        = "#268bd2"
cursor_fg        = "#fdf6e3"
selection_bg     = "#586e75"
selection_fg     = "#eee8d5"
search_match_bg  = "#b58900"
search_match_fg  = "#002b36"
header_bold      = true
header_bg        = "#002b36"
dirty_indicator_fg = "#dc322f"
```

### Nord

Cool, blue-toned Arctic palette with soft contrast.

```toml
# ~/.config/lazycsv/config.toml

[theme]
zebra_bg         = "#2e3440"
cursor_bg        = "#88c0d0"
cursor_fg        = "#2e3440"
selection_bg     = "#4c566a"
selection_fg     = "#eceff4"
search_match_bg  = "#ebcb8b"
search_match_fg  = "#2e3440"
header_bold      = true
header_bg        = "#3b4252"
dirty_indicator_fg = "#bf616a"
```

## Tips for Custom Themes

- **Start from an example.** Copy one of the themes above into your config and adjust individual values.
- **Cursor contrast is critical.** Make sure `cursor_bg` and `cursor_fg` have strong contrast — the cursor is the primary navigation indicator.
- **`zebra_bg` should be subtle.** A slight tint from the terminal background is enough; too much contrast makes the grid hard to scan.
- **`header_bg` is optional.** Leaving it unset lets the header row inherit the terminal's default background, which blends naturally with many color schemes.
- **Test search matches.** Open a file and run a search (`/`) to confirm `search_match_bg` and `search_match_fg` are readable against your zebra and cursor colors.
- **Hex values give precise control.** Named colors map to your terminal's palette and vary between terminal emulators; hex values are exact and consistent.
