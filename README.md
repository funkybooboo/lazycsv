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

- ⚡ **Fast** - 100K+ rows at 60 FPS
- ⌨️ **Vim keys** - hjkl your way through data
- 📁 **Multi-file** - switch between CSVs like Excel sheets (press `[` `]`)
- 🎯 **Simple** - no config needed, just works
- 🎨 **Clean** - minimalist design, zero clutter

**Note:** The "lazy" in LazyCSV is currently aspirational. The app loads the entire CSV file into memory. True lazy-loading for large files is a top priority for future versions!

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

# In the app:
# hjkl or arrows  → navigate
# [ or ]          → switch between CSV files
# ?               → show help
# q               → quit
```

That's it! Press `?` in the app for full keybindings.

## Essential Keys

| Key | Action |
|-----|--------|
| `hjkl` or arrows | Move around |
| `gg` / `G` | Jump to top/bottom |
| `[` / `]` | Switch CSV files |
| `?` | Show help |
| `q` | Quit |

**Vim users:** All your favorite motions work (`0`, `$`, etc.)

## Innovation: Multi-File Navigation

LazyCSV treats CSV files in the same directory like Excel sheets. Open one file, instantly switch between all of them with `[` and `]` keys. No more `cd` and reopening!

## Coming Soon

- 💾 **True lazy-loading** for huge files
- ✏️ Cell editing & saving
- ➕ Add/delete rows and columns
- 🔍 Fuzzy search & filtering
- 📊 Column sorting
- 📑 Excel file support

See [plans/todo.md](plans/todo.md) for the full roadmap.

## Documentation

- **[Features](docs/features.md)** - What it can do (and will do)
- **[Keybindings](docs/keybindings.md)** - Every keyboard shortcut
- **[Design](docs/design.md)** - How it looks and feels
- **[Architecture](docs/architecture.md)** - How it works
- **[Development](docs/development.md)** - How to contribute

## Development

```bash
# Using Task (recommended)
task run        # run with sample.csv
task test       # run tests (133 tests)
task all        # format, lint, test

# Or with Cargo
cargo run -- sample.csv
cargo test
```

**Test Suite:** 133 comprehensive tests covering all Phase 1 features including directory handling. See [tests/README.md](tests/README.md) for details.

See [docs/development.md](docs/development.md) for contributing guidelines.

## Status

🎉 **Phase 1 MVP Complete!** LazyCSV is ready to use for viewing CSV files.

- ✅ Fast CSV viewer with vim navigation
- ✅ Multi-file switching
- ✅ Row/column numbers (A, B, C...)
- ✅ Comprehensive test suite (133 tests)
- 📋 Cell editing coming in Phase 2

**Version:** 0.1.0 | **Tests:** 99 passing | **Performance:** 60 FPS on 100K+ rows

## Philosophy

LazyCSV follows the "lazy tools" design:
1. **Keyboard first** - mouse optional
2. **Fast** - instant response
3. **Simple** - no configuration required
4. **Powerful** - vim-style efficiency

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
