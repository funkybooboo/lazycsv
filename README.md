# lazycsv

A blazingly fast terminal UI for CSV files. Navigate huge datasets with vim keys, switch between files instantly, and never touch your mouse.

Inspired by [lazygit](https://github.com/jesseduffield/lazygit), [lazydocker](https://github.com/jesseduffield/lazydocker), and [lazysql](https://github.com/jorgerojas26/lazysql).

```
 lazycsv: sales_data.csv                                              2/100
──────────────────────────────────────────────────────────────────────────
      A           B              C              D           E
  #   ID          Date           Product        Qty         Price
  1   001         2024-01-15     Widget A       100         $25.00
> 2   002         2024-01-16     Gadget B       50          $42.50
  3   003         2024-01-17     Doohickey      75          $18.75
  4   004         2024-01-18     Thingamajig    200         $12.00
  5   005         2024-01-19     Whatchamacal   150         $35.00
──────────────────────────────────────────────────────────────────────────
sales.csv | customers.csv | orders.csv                               [1/3]
NORMAL                                                    2,B "2024-01-16"
```

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

# In the app:
# hjkl or arrows  -> navigate
# [ or ]          -> switch between CSV files
# gg or G         -> jump to top/bottom
# :c A or :c 5    -> jump to column A or column 5
# ?               -> show help
# :q              -> quit
```

That's it! Press `?` in the app for full keybindings.

## Essential Keys

| Key | Action |
|-----|--------|
| `hjkl` or arrows | Move around (with count: `5j`, `10h`) |
| `gg` / `G` / `15G` | Jump to first/last/line 15 |
| `:c A` / `:c 5` | Jump to column A or column 5 |
| `w` / `b` / `e` | Next/prev/last non-empty cell |
| `:15` | Command mode: jump to row 15 |
| `zt` / `zz` / `zb` | Position row at top/center/bottom |
| `[` / `]` | Switch CSV files |
| `?` | Show help |
| `:q` or `q` | Quit |

**Vim users:** All your favorite motions work (`0`, `$`, count prefixes, etc.)

## Innovation: Multi-File Navigation

LazyCSV treats CSV files in the same directory like Excel sheets. Open one file, instantly switch between all of them with `[` and `]` keys. No more `cd` and reopening!

## Roadmap to v1.0

| Version | Features |
|---------|----------|
| **v0.1.0** | Foundation - viewing, navigation, multi-file |
| **v0.2.0** | Type safety refactor (internal) |
| **v0.3.0** | Advanced navigation - column jumps, command mode, word motion |
| **v0.3.1** | UI/UX polish - mode indicator, transient messages, help redesign |
| **v0.3.2** | Pre-edit polish - minimal UI, vim-like status line, `:c` command |
| **v0.4.0** | Insert mode - quick cell editing |
| **v0.5.0** | Operators - `d`, `y`, `c`, `p` for composable editing |
| **v0.6.0** | Visual mode - `v`, `V`, `Ctrl+v` for selection |
| **v0.7.0** | Persistence - `:w`, `:q`, `:wq` save/quit |
| **v0.8.0** | Undo/redo - `u`, `Ctrl+r`, `.` dot command |
| **v0.9.0** | Search - `/`, `?`, `n`, `N`, `*`, `#` |
| **v1.0.0** | First stable release |

### Post-v1.0 Features

| Version | Features |
|---------|----------|
| v1.1.0 | Marks & registers - `m`, `'`, `"` |
| v1.2.0 | Text objects - `ic`, `ir`, `ac`, `ar` for cells/rows |
| v1.3.0 | Sorting & filtering - `:sort`, `:filter` |
| v1.4.0 | Column operations - resize, freeze |
| v1.5.0 | Advanced features - tab completion, macros |
| v1.6.0 | Data analysis - stats, export |

See [plans/roadmap.md](plans/roadmap.md) for the complete detailed roadmap.

## Documentation

- **[Keybindings](docs/keybindings.md)** - Every keyboard shortcut by version
- **[Design](docs/design.md)** - How it looks and feels
- **[Architecture](docs/architecture.md)** - How it works
- **[Development](docs/development.md)** - How to contribute

## Development

```bash
# Using Task (recommended)
task run        # run with sample.csv
task test       # run tests
task all        # format, lint, test

# Or with Cargo
cargo run -- sample.csv
cargo test
```

See [docs/development.md](docs/development.md) for contributing guidelines.

## Status

**v0.3.2 Complete!** Minimal vim-like UI and improved command mode.

- Fast CSV viewer with vim navigation
- Multi-file switching with `[` `]`
- Row/column numbering (A, B, C...)
- Column jumping with `:c` command (`:c A`, `:c 5`, `:c AA`)
- Command mode with reserved commands (`:q`, `:w`, `:h`)
- Word motion (w/b/e for sparse data)
- Viewport control (zt/zz/zb)
- **NEW v0.3.2:** Minimal vim-like UI (no heavy borders)
- **NEW v0.3.2:** Auto-width columns based on content
- **NEW v0.3.2:** Pending command display (shows `g_`, `z_`, `5_`)
- **NEW v0.3.2:** Out-of-bounds errors (not silent clamping)
- **NEW v0.3.2:** Default to current directory when no path provided
- Comprehensive test suite (344 tests passing)
- Cell editing coming in v0.4.0

**Current:** v0.3.2 Complete | **Performance:** 60 FPS on 100K+ rows | **Architecture:** Clean, type-safe, well-tested

### What's New in v0.3.2

**v0.3.2 - Pre-Edit Polish:**
- **Minimal UI:** Removed heavy box borders, replaced with clean horizontal rules
- **Vim-like status line:** `NORMAL  3,C "cell value"` format
- **`:c` command:** Jump to columns with `:c A`, `:c 5`, or `:c AA`
- **Reserved commands:** `:q`, `:w`, `:h` work properly (don't jump to columns)
- **Pending command display:** Shows what you've typed (`g_`, `z_`, `5_`)
- **Auto-width columns:** Columns size to content (8-50 char range)
- **Out-of-bounds errors:** Clear messages like "Row 999 does not exist (max: 10)"
- **Default directory:** Running `lazycsv` without args scans current directory
- **No more timeout:** Pending commands wait indefinitely (vim-like)

### What's New in v0.3.0 & v0.3.1

**v0.3.0 - Advanced Navigation:**
- Column jumping with Excel notation (`ga`, `gB`, `gBC`)
- Vim-style command mode (`:15` for line)
- Word motion for sparse data (`w`, `b`, `e`)
- Viewport positioning (`zt`, `zz`, `zb`)

**v0.3.1 - UI/UX Polish:**
- Mode indicator (-- NORMAL -- / -- COMMAND --)
- Dirty flag display (*)
- Transient messages that auto-clear
- Redesigned help menu with better organization
- File list horizontal scrolling

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
