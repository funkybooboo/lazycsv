# lazycsv

A fast, ergonomic TUI for CSV and Excel files. Browse, edit, and explore tabular data directly from your terminal—no spreadsheet GUI required.

Inspired by [lazygit](https://github.com/jesseduffield/lazygit), [lazydocker](https://github.com/jesseduffield/lazydocker), and [lazysql](https://github.com/jorgerojas26/lazysql).

## Features

### Phase 1: Core Viewing (MVP - In Development)
- 📊 **Beautiful table display** with row/column numbers
- ⌨️ **Vim-style navigation** (hjkl + arrow keys)
- 🎯 **Row highlighting** and current cell indication
- 📜 **Smooth scrolling** for large files (10K+ rows)
- ↔️ **Horizontal scrolling** for wide tables (~10 columns visible)
- 📁 **Multi-file navigation** - switch between CSV files in same directory
- ❓ **Built-in cheatsheet** (press `?`) like lazygit
- 🎨 **Clean, minimal UI** (monochrome design)

### Phase 2: Cell Editing (Planned)
- ✏️ **In-place editing** of cell values
- 💾 **Save changes** with Ctrl+S or `:w`
- 🔄 **Undo/redo** support
- 🚦 **Dirty state tracking** (unsaved changes indicator)

### Phase 3: Row/Column Operations (Planned)
- ➕ **Add rows** (`o`/`O`) and columns (`Ctrl+A`)
- ➖ **Delete rows** (`dd`) and columns (`D`)
- 📋 **Copy/paste rows** (`yy`/`p`)
- 🎯 **Visual selection** mode for bulk operations

### Phase 4: Advanced Features (Planned)
- 🔍 **Fuzzy finder** - search rows, columns, and cell data
- 🔢 **Sort by column** (ascending/descending)
- 🎯 **Filter** rows by criteria
- 📈 **Column statistics** (sum, average, min/max)

### Phase 5: Excel Support (Planned)
- 📑 **Excel workbook support** - read/write .xlsx files
- 🗂️ **Multi-sheet navigation** - switch between worksheets
- 🎨 **Preserve formatting** on save

## Installation

### From Source
```bash
git clone https://github.com/yourusername/lazycsv.git
cd lazycsv
cargo build --release
sudo cp target/release/lazycsv /usr/local/bin/
```

### From Crates.io (Coming Soon)
```bash
cargo install lazycsv
```

## Usage

```bash
# Open a CSV file
lazycsv data.csv

# Future: Open Excel file
lazycsv spreadsheet.xlsx
```

## Keybindings

Press `?` in the app to see the full cheatsheet. Here are the essentials:

### Navigation (Phase 1)
| Key | Action |
|-----|--------|
| `hjkl` or arrows | Move cursor (vim-style) |
| `gg` / `Home` | Jump to first row |
| `G` / `End` | Jump to last row |
| `Ctrl+d` / `PageDown` | Page down |
| `Ctrl+u` / `PageUp` | Page up |
| `w` | Next column |
| `b` | Previous column |
| `[` | Previous file/sheet |
| `]` | Next file/sheet |

### Editing (Phase 2+)
| Key | Action |
|-----|--------|
| `i` or `Enter` | Edit current cell |
| `Esc` | Cancel edit |
| `Ctrl+S` | Save file |
| `o` | Add row below |
| `O` | Add row above |
| `dd` | Delete current row |
| `yy` | Copy row |
| `p` | Paste row |
| `u` | Undo |
| `Ctrl+r` | Redo |

### Search & Sort (Phase 4+)
| Key | Action |
|-----|--------|
| `/` | Open fuzzy finder |
| `n` / `N` | Next/previous match |
| `s` | Sort by column |

### Other
| Key | Action |
|-----|--------|
| `?` | Toggle help/cheatsheet |
| `q` | Quit (warns if unsaved) |
| `:q!` | Force quit |
| `:w` | Save (command mode) |

## Design Philosophy

LazyCSV follows the "lazy tools" philosophy:

1. **Keyboard-first**: Never touch the mouse
2. **Intuitive**: Vim-style keybindings that feel natural
3. **Beautiful**: Clean UI with helpful visual feedback
4. **Powerful**: Complex operations with simple keystrokes
5. **Fast**: Handle large files (10K+ rows) smoothly at 60 FPS
6. **Consistent**: Same UX for CSV files and Excel sheets

## Innovation

**Multi-file navigation**: LazyCSV treats CSV files in the same directory like "worksheets" - providing the same navigation experience as Excel's multi-sheet workbooks. Switch between files with `[` and `]` keys!

## Technology

Built with:
- [ratatui](https://ratatui.rs/) - Terminal UI framework
- [crossterm](https://docs.rs/crossterm/) - Cross-platform terminal control
- [csv](https://docs.rs/csv/) - Fast CSV parsing by BurntSushi
- [fuzzy-matcher](https://docs.rs/fuzzy-matcher/) - Fuzzy search
- [serde](https://serde.rs/) - Serialization framework

## Development

### Running Tests
```bash
cargo test
```

### Running Locally
```bash
cargo run -- sample.csv
```

### Building Release
```bash
cargo build --release
```

## Roadmap

See [plans/todo.md](plans/todo.md) for the detailed development checklist.

- [ ] Phase 1: Core viewing with vim navigation (In Progress)
- [ ] Phase 2: Cell editing and file saving
- [ ] Phase 3: Row/column operations
- [ ] Phase 4: Fuzzy search, filter, sort
- [ ] Phase 5: Excel support

## Contributing

Contributions welcome! Please:
1. Check the [todo list](plans/todo.md) for open tasks
2. Open an issue to discuss major changes
3. Follow Rust conventions (rustfmt, clippy)
4. Add tests for new features

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Inspiration

This project draws inspiration from the excellent "lazy" series:
- [lazygit](https://github.com/jesseduffield/lazygit) - Simple terminal UI for git
- [lazydocker](https://github.com/jesseduffield/lazydocker) - Docker TUI
- [lazysql](https://github.com/jorgerojas26/lazysql) - SQL database TUI
- [lazyssh](https://github.com/anidude/lazyssh) - SSH TUI

## Acknowledgments

Special thanks to:
- [ratatui team](https://ratatui.rs/) for the excellent TUI framework
- [BurntSushi](https://github.com/BurntSushi) for the CSV crate
- The Rust community for making this possible
