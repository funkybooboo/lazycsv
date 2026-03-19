# lazycsv

A blazingly fast terminal UI for CSV files. Navigate huge datasets with vim keys, switch between files instantly, and never touch your mouse.

Inspired by [lazygit](https://github.com/jesseduffield/lazygit), [lazydocker](https://github.com/jesseduffield/lazydocker), and [lazysql](https://github.com/jorgerojas26/lazysql).

<!-- Screenshot: Main table view showing CSV data with vim-style navigation -->
![LazyCSV Main View](screenshots/main-view.png)

## Why LazyCSV?

- **Fast** - 100K+ rows at 60 FPS (in-memory)
- **Vim keys** - hjkl your way through data, full vim emulation planned
- **Multi-file** - switch between CSVs like Excel sheets (press `[` `]`)
- **Simple** - no config needed, just works
- **Clean** - minimal vim-like UI, zero clutter

**Note:** LazyCSV loads the entire CSV file into memory for maximum performance. This design choice prioritizes speed and simplicity over handling files larger than available RAM.

## Install

```bash
git clone https://github.com/funkybooboo/lazycsv.git
cd lazycsv
cargo install --path .
```

## Quick Start

```bash
# Open current directory (scans for CSV files)
lazycsv

# Open specific file
lazycsv data.csv

# Open a directory
lazycsv ./data/

# With options
lazycsv data.csv --delimiter ';' --no-headers

# Non-interactive query mode
lazycsv data.csv --query "SELECT * FROM data WHERE amount > 100"

# Locale-aware number formatting
lazycsv data.csv --format

# In the app:
# hjkl or arrows  -> navigate
# [ or ]          -> switch between CSV files
# gg or G         -> jump to top/bottom
# :B or :5        -> jump to column B or row 5
# i or Enter      -> edit cell (quick or magnifier)
# /               -> search
# :sql            -> SQL query mode
# :files          -> file picker (same as [ and ])
# Esc             -> cancel loading/queries
# ?               -> show help
# :q              -> quit
```

That's it! Press `?` in the app for full keybindings.

## Essential Keys

| Key | Action |
|-----|--------|
| `hjkl` or arrows | Move around (with count: `5j`, `10h`) |
| `gg` | Jump to first row (row 0) |
| `G` / `5G` | Jump to last row / row 5 |
| `:B` / `:A5` | Jump to column B or cell A5 |
| `w` / `b` / `e` | Next/prev/last non-empty cell |
| `i` / `a` / `s` | Quick edit cell (Insert mode) |
| `Enter` | Magnifier mode (full vim editor for multi-line) |
| `o` / `O` | Insert row below/above |
| `dd` / `yy` / `p` | Delete/yank/paste row |
| `,dd` / `,yy` / `,p` | Delete/yank/paste column |
| `v` / `V` | Visual selection (cell/row) |
| `/` | Search (regex) |
| `:sql` | SQL query mode |
| `:files` | File picker dialog |

| `Esc` | Cancel loading/queries |
| `zt` / `zz` / `zb` | Position row at top/center/bottom |
| `[` / `]` | Switch CSV files |
| `?` | Show help |
| `:w` / `:q` | Save / Quit |

**Vim users:** All your favorite motions work (`0`, `$`, count prefixes, etc.)

## Editing Modes

### Insert Mode (Quick Edits)
Press `i`, `a`, or `s` for quick single-line cell editing. Perfect for fixing typos or updating values.

```
i     -> Edit cell (cursor at end)
a     -> Edit cell (cursor at end)
I     -> Edit cell (cursor at start)
s     -> Replace cell (clear + edit)
```

Exit with `Enter` (save + move down), `Tab` (save + move right), or `Esc` (cancel).

### Magnifier Mode (Complex Edits)
Press `Enter` to open a full vim editor for complex multi-line cell editing. Perfect for JSON, descriptions, or any content that needs vim power.

```
Enter              -> Open magnifier on current cell
hjkl / w/b/e       -> Vim motions
i/a/o/O            -> Enter insert mode
dd / yy / p        -> Delete/yank/paste lines
x / s              -> Delete/substitute character
:wq or ZZ          -> Save and close
:q!                -> Close without saving
Alt+hjkl/arrows    -> Navigate to adjacent cells
```

**Features:**
- Full vim editing (motions, operators, count prefixes)
- Multi-line content with proper CSV escaping
- Line numbers and mode indicators
- Centered popup overlay
- Dirty tracking with save prompts

## Innovation: Multi-File Navigation

LazyCSV treats CSV files in the same directory like Excel sheets. Open one file, instantly switch between all of them with `[` and `]` keys. No more `cd` and reopening!

## Current Status

**v0.12.0 Complete** (March 2026) - Yazi-inspired 3-column file browser with parent directory preview, current directory navigation, and file/CSV preview.

**Completed Features:**
- Fast CSV viewer/editor with vim navigation
- Multi-file switching (`[` `]` or `:files`)
- Full vim editing (Insert mode + Magnifier mode)
- Row operations (`o`, `O`, `dd`, `yy`, `p`)
- Column operations (`,dd`, `,yy`, `,p`, `,o`)
- Visual mode (`v`, `V`, `,v`)
- Search (`/`, `n`, `N`) with regex support
- SQL query mode (`:sql` with full SELECT/WHERE/JOIN/GROUP BY)
- Non-interactive query mode (`--query` flag)
- Locale-aware formatting (`--format` flag)
- External file modification detection
- Cancellable operations (`Esc` during loading/queries)
- File persistence (`:w`, `:wq`, `:q`)
- 700+ tests passing

**Next Up:**
- **v0.12.1** - UI System Testing
- **v0.13.0** - Repository Organization & Structure
- **v1.0.0** - Stable release

See [plans/roadmap.md](plans/roadmap.md) for the complete detailed roadmap.

## Documentation

- **[Keybindings](docs/keybindings.md)** - Every keyboard shortcut by version
- **[Design](docs/design.md)** - How it looks and feels
- **[Architecture](docs/architecture.md)** - How it works
- **[Development](docs/development.md)** - How to contribute

## Development

```bash
# Using mise (recommended)
mise run run        # run with sample.csv
mise run test       # run tests
mise run ci         # format, lint, test

# Or with Cargo
cargo run -- sample.csv
cargo test
```

See [docs/development.md](docs/development.md) for contributing guidelines.

## Screenshots

### Main View
<!-- Screenshot: Normal mode navigation -->
![Normal Mode](screenshots/normal-mode.png)

### Insert Mode
<!-- Screenshot: Quick cell editing in Insert mode -->
![Insert Mode](screenshots/insert-mode.png)

### Magnifier Mode
<!-- Screenshot: Full vim editor for multi-line cell content -->
![Magnifier Mode](screenshots/magnifier-mode.png)

### Visual Selection
<!-- Screenshot: Visual mode selecting multiple cells/rows -->
![Visual Selection](screenshots/visual-mode.png)

### SQL Query Mode
<!-- Screenshot: SQL editor with query -->
![SQL Query Mode](screenshots/sql-editor.png)

<!-- Screenshot: SQL query results -->
![SQL Results](screenshots/sql-results.png)

### Search
<!-- Screenshot: Regex search overlay -->
![Search](screenshots/search.png)

## What's New in v0.11.0

**Full Vim Modal Editing, Undo/Redo, and Configuration (March 2026):**
- **SQL Editor:** Full vim modal editing in SQL editor (Normal/Insert/Visual modes)
- **Undo/Redo:** Complete undo/redo system (`u`, `Ctrl+r`, `.` repeat)
- **Configuration:** Config file support with themes and customization
- **Testing:** 700+ total tests passing with comprehensive coverage
- **Performance:** Maintained 60 FPS with enhanced editing capabilities

## Philosophy

LazyCSV follows the "lazy tools" design:
1. **Keyboard first** - mouse optional
2. **Fast** - instant response, in-memory for speed
3. **Simple** - no configuration required
4. **Powerful** - vim-style efficiency
5. **Vim-first** - if it works in vim, it should work here

## License

GPL License - see [LICENSE](LICENSE) file for details.

## Credits

Built with:
- [ratatui](https://ratatui.rs/) - TUI framework
- [csv](https://docs.rs/csv/) - CSV parsing by BurntSushi
- Rust

Inspired by the excellent "lazy" tools:
[lazygit](https://github.com/jesseduffield/lazygit) |
[lazydocker](https://github.com/jesseduffield/lazydocker) |
[lazysql](https://github.com/jorgerojas26/lazysql) |
[lazyssh](https://github.com/anidude/lazyssh)

---

**Have fun exploring your data!**
