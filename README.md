# lazycsv

A blazingly fast terminal UI for CSV files. Navigate huge datasets with vim keys, switch between files instantly, and never touch your mouse.

Inspired by [lazygit](https://github.com/jesseduffield/lazygit), [lazydocker](https://github.com/jesseduffield/lazydocker), and [lazysql](https://github.com/jorgerojas26/lazysql).

```
┌─ lazycsv: sales_data.csv ─────────────────────────┐
│     │  A      │ ►B         │  C           │  D    │
├─────┼─────────┼────────────┼──────────────┼───────┤
│  #  │  ID     │  Date      │  Product     │  Qty  │
├─────┼─────────┼────────────┼──────────────┼───────┤
│  1  │  001    │ 2024-01-15 │ Widget A     │  100  │
│►2   │  002    │ [2024...] │ Gadget B     │   50  │
│  3  │  003    │ 2024-01-17 │ Doohickey... │   75  │
├────────────────────────────────────────────────────┤
│ [?] help │ [q] quit │ [ ] files │                 │
│ Row 2/100 │ Col B: Date (2/4) │ Cell: "2024..." │
├────────────────────────────────────────────────────┤
│ Files (1/2): ► sales.csv | customers.csv         │
└────────────────────────────────────────────────────┘
```

## Why LazyCSV?

- ⚡ **Fast** - 100K+ rows at 60 FPS (in-memory)
- ⌨️ **Vim keys** - hjkl your way through data, full vim emulation in magnifier
- 📁 **Multi-file** - switch between CSVs like Excel sheets (press `[` `]`)
- 🎯 **Simple** - no config needed, just works
- 🎨 **Clean** - minimalist design, zero clutter

**Note:** LazyCSV loads the entire CSV file into memory for maximum performance. This design choice prioritizes speed and simplicity over handling files larger than available RAM.

## Install

```bash
git clone https://github.com/funkybooboo/lazycsv.git
cd lazycsv
cargo install --path .
```

## Quick Start

```bash
# Open current directory
lazycsv

# Or open specific file
lazycsv data.csv

# With options
lazycsv data.csv --delimiter ';' --no-headers

# In the app:
# hjkl or arrows  → navigate
# [ or ]          → switch between CSV files
# gg or G         → jump to top/bottom
# ?               → show help
# q               → quit
```

That's it! Press `?` in the app for full keybindings.

## Essential Keys

| Key | Action |
|-----|--------|
| `hjkl` or arrows | Move around (with count: `5j`, `10h`) |
| `gg` / `G` / `15G` | Jump to first/last/line 15 |
| `ga` / `gB` / `gBC` | Jump to column A/B/BC (Excel-style) |
| `w` / `b` / `e` | Next/prev/last non-empty cell |
| `:15` / `:B` | Command mode: jump to line/column |
| `zt` / `zz` / `zb` | Position row at top/center/bottom |
| `[` / `]` | Switch CSV files |
| `?` | Show help |
| `q` | Quit |

**Vim users:** All your favorite motions work (`0`, `$`, count prefixes, etc.)

## Innovation: Multi-File Navigation

LazyCSV treats CSV files in the same directory like Excel sheets. Open one file, instantly switch between all of them with `[` and `]` keys. No more `cd` and reopening!

## Roadmap to v1.0

| Version | Features |
|---------|----------|
| **v0.1.0** | ✅ Foundation - viewing, navigation, multi-file |
| **v0.2.0** | ✅ Type safety refactor (COMPLETE - all 6 phases) |
| **v0.3.0** | ✅ Advanced navigation - column jumps, command mode, word motion |
| **v0.3.1** | ✅ UI/UX polish - mode indicator, transient messages, help redesign |
| **v0.4.0** | Quick editing - Insert mode for fast cell edits |
| **v0.5.0** | **Vim magnifier** - full vim editor embedded in TUI |
| **v0.6.0** | Save/quit guards - `:w`, `:q`, dirty tracking |
| **v0.7.0** | Row operations - `o`, `O`, `dd`, `yy`, `p` |
| **v0.8.0** | Column operations - `:addcol`, `:delcol` |
| **v0.9.0** | Header management - `gh` to edit headers |
| **v1.0.0** | Undo/redo system - `u`, `Ctrl+r` |

### Post-v1.0 Features

| Version | Features |
|---------|----------|
| v1.1.0 | Search & visual selection - `/`, `v`, `V` |
| v1.2.0 | Sorting & filtering - `s`, `:filter` |
| v1.3.0 | Multi-file guards with dirty tracking |
| v1.4.0 | Advanced viewing - column freezing, themes |
| v1.5.0 | Data analysis - stats, plotting, regex replace |
| v1.6.0 | Final polish - comprehensive tests, docs |

See [plans/todo.md](plans/todo.md) for the complete detailed roadmap.

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

🎉 **v0.3.1 Complete!** Advanced navigation and polished UI ready to use.

- ✅ Fast CSV viewer with vim navigation
- ✅ Multi-file switching with `[` `]`
- ✅ Row/column numbering (A, B, C...)
- ✅ **NEW v0.3.0:** Column jumping (ga, gB, gBC)
- ✅ **NEW v0.3.0:** Command mode (:15, :B)
- ✅ **NEW v0.3.0:** Word motion (w/b/e for sparse data)
- ✅ **NEW v0.3.0:** Viewport control (zt/zz/zb)
- ✅ **NEW v0.3.1:** Mode indicator and dirty flag
- ✅ **NEW v0.3.1:** Enhanced help menu
- ✅ Comprehensive test suite (265 tests passing)
- 📋 Cell editing coming in v0.4.0
- 🎯 Target: v1.0.0 with full editing, undo, rows/columns

**Current:** v0.3.1 Complete | **Performance:** 60 FPS on 100K+ rows | **Architecture:** Clean, type-safe, well-tested

### What's New in v0.3.0 & v0.3.1

**v0.3.0 - Advanced Navigation:**
- Column jumping with Excel notation (`ga`, `gB`, `gBC`)
- Vim-style command mode (`:15` for line, `:B` for column)
- Word motion for sparse data (`w`, `b`, `e`)
- Viewport positioning (`zt`, `zz`, `zb`)
- Enter key navigation

**v0.3.1 - UI/UX Polish:**
- Mode indicator (-- NORMAL -- / -- COMMAND --)
- Dirty flag display ([*])
- Transient messages that auto-clear
- Redesigned help menu with better organization
- File list horizontal scrolling

## Philosophy

LazyCSV follows the "lazy tools" design:
1. **Keyboard first** - mouse optional
2. **Fast** - instant response, in-memory for speed
3. **Simple** - no configuration required
4. **Powerful** - vim-style efficiency with full vim emulation
5. **Vim-first** - if it works in vim, it should work here

## License

GPL License - see [LICENSE](LICENSE) file for details.

## Credits

Built with:
- [ratatui](https://ratatui.rs/) - TUI framework
- [csv](https://docs.rs/csv/) - CSV parsing by BurntSushi
- Rust 🦀

Inspired by the excellent "lazy" tools:
[lazygit](https://github.com/jesseduffield/lazygit) •
[lazydocker](https://github.com/jesseduffield/lazydocker) •
[lazysql](https://github.com/jorgerojas26/lazysql) •
[lazyssh](https://github.com/anidude/lazyssh)

---

**Have fun exploring your data!** 📊✨
