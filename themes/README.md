# LazyCSV Themes

Drop-in `theme.toml` files for popular color schemes. Pick one, copy it into your config, and restart (or just save — LazyCSV hot-reloads).

## Available Themes

| File | Description |
|------|-------------|
| `gruvbox-dark.toml` | Warm, retro earth tones — dark variant |
| `gruvbox-light.toml` | Same palette on a cream background |
| `dracula.toml` | Vibrant neon accents on deep purple |
| `nord.toml` | Cold, clean slate grays and frosty blues |
| `catppuccin-mocha.toml` | Soothing pastel — darkest Catppuccin flavor |
| `catppuccin-macchiato.toml` | Soothing pastel — middle Catppuccin dark |
| `catppuccin-frappe.toml` | Soothing pastel — softest Catppuccin dark |
| `catppuccin-latte.toml` | Soothing pastel — light Catppuccin |
| `solarized-dark.toml` | Precision-engineered blue-green dark |
| `solarized-light.toml` | Precision-engineered cream light |
| `tokyo-night.toml` | Deep blues with neon highlights |

## How to Install

LazyCSV reads its config from `~/.config/lazycsv/config.toml` (macOS / Linux) or `%APPDATA%\lazycsv\config.toml` (Windows). To use a theme, copy one of these files there:

```sh
# Whole-config replacement
cp themes/dracula.toml ~/.config/lazycsv/config.toml
```

If you already have a `config.toml` with `[defaults]` or `[sql]` settings you want to keep, append the theme sections to it instead — every theme file sets six sections (`[ui]`, `[table]`, `[popup]`, `[status]`, `[file_menu]`, and a partial `[sql]`). The `[sql]` section in each theme only defines visual fields (`line_number_fg`, `diagnostic_*_fg`); behavioural keys like `format_uppercase` are untouched.

You can also drop a theme file alongside a project-specific override in `.lazycsv.toml` to apply it only when working in that directory.

## Customising

These files are starting points. Edit the hex values to taste — see [`docs/themes.md`](../docs/themes.md) for the full list of theme keys and what each one controls.

## Credits

| Theme | Original author / project |
|-------|---------------------------|
| Gruvbox | Pavel Pertsev — <https://github.com/morhetz/gruvbox> |
| Dracula | Zeno Rocha — <https://draculatheme.com/> |
| Nord | Arctic Ice Studio — <https://www.nordtheme.com/> |
| Catppuccin | Catppuccin community — <https://catppuccin.com/> |
| Solarized | Ethan Schoonover — <https://ethanschoonover.com/solarized/> |
| Tokyo Night | Enkia — <https://github.com/enkia/tokyo-night-vscode-theme> |

All palettes are reproduced under their respective open-source licenses (typically MIT). Use freely.
