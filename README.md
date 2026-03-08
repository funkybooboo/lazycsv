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
# :cB or 5g       -> jump to column B or row 5
# i or m          -> edit cell (quick or magnifier)
# ?               -> show help
# :q              -> quit
```

That's it! Press `?` in the app for full keybindings.

## Essential Keys

| Key | Action |
|-----|--------|
| `hjkl` or arrows | Move around (with count: `5j`, `10h`) |
| `gg` / `G` / `5g` | Jump to first/last/row 5 |
| `:cB` / `:cA` | Jump to column B or A |
| `w` / `b` / `e` | Next/prev/last non-empty cell |
| `i` / `a` / `s` | Quick edit cell (Insert mode) |
| `m` | Magnifier mode (full vim editor) |
| `o` / `O` | Insert row below/above |
| `dd` / `yy` / `p` | Delete/yank/paste row |
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
Press `m` to open a full vim editor for complex multi-line cell editing. Perfect for JSON, descriptions, or any content that needs vim power.

```
m                  -> Open magnifier on current cell
hjkl / w/b/e       -> Vim motions
i/a/o/O            -> Enter insert mode
dd / yy / p        -> Delete/yank/paste lines
x / s              -> Delete/substitute character
ZZ or :wq          -> Save and close
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

## Roadmap to v1.0

| Version | Features |
|---------|----------|
| **v0.1.0** | Foundation - viewing, navigation, multi-file |
| **v0.2.0** | Type safety refactor (internal) |
| **v0.3.0** | Advanced navigation - column jumps, command mode, word motion |
| **v0.4.0** | Cell editing - Insert mode, row operations |
| **v0.5.0** | Column operations - `,o`, `,O`, `,dd`, `,yy`, `,p`, visual mode |
| **v0.6.0** | Magnifier Mode - multi-line cell editing with full vim  |
| **v0.7.0** | Search & Filtering - `/`, `n`, `N`, `*`, `:noh` fuzzy search |
| **v0.8.0** | SQL Query Mode - `:q SELECT...`, JOINs, multi-table queries |
| **v0.8.1** | SQL Polish - refactoring, tests, benchmarks, error messages ✨ **Current** |
| **v0.9.0** | Undo/redo - `u`, `Ctrl+r`, `.` dot command |
| **v1.0.0** | Stable release - polish, performance, docs |

### Post-v1.0 Features

Future considerations (not committed):
- Column resize & freeze
- Advanced features - macros, tab completion
- Data export - JSON, Markdown, HTML

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

## Status

**v0.8.1 Complete!** SQL query mode with comprehensive testing, benchmarks, and polished error handling.

- Fast CSV viewer and editor with vim navigation
- Multi-file switching with `[` `]`
- Full vim editing (Insert mode, Magnifier mode with hjkl, motions, operators)
- Column operations (`,dd`, `,yy`, `,p`, `,o`, `,O`)
- Visual mode (v, V, ,v) for rectangular selections
- Search & filtering (`/`, `n`, `N`, `*`, `:noh`)
- **SQL Query Mode:**
  - `:q SELECT...` to run SQL queries on CSVs
  - Multi-table JOINs (automatically loads sibling CSVs)
  - GROUP BY, ORDER BY, aggregations
  - Query caching for instant re-execution
  - 555+ tests (30 SQL edge cases, comprehensive coverage)
  - Performance benchmarks (SQL operations benchmarked)
- Row operations (`o`, `O`, `dd`, `yy`, `p`)
- Comprehensive test suite (555+ tests passing)

**Current:** v0.8.1 Complete | **Performance:** 60 FPS on 100K+ rows | **Architecture:** Clean, type-safe, well-tested

### What's New in v0.8.1

**v0.8.1 - SQL & Data Operations Polish:**
- **Code Quality:**
  - Refactored `execute_sql_query_cancellable`: 164 → 53 lines (67.7% reduction)
  - Refactored `render_sql_editor_overlay`: 118 → 35 lines (70% reduction)
  - Created `src/app/sql_execution.rs` helper module (239 lines)
  - Created `src/ui/sql_editor_helpers.rs` rendering helpers (99 lines)
  - Zero clippy warnings
- **Testing:**
  - Added 30 comprehensive SQL edge case tests (`tests/sql_edge_cases_test.rs`)
  - Tests cover: error handling, empty results, large datasets, NULLs, complex queries
  - Total: 555 tests passing (514 lib + 11 SQL integration + 30 edge cases)
- **Benchmarks:**
  - Created `benches/sql.rs` with 13 benchmark groups
  - Benchmarks: CSV loading, SELECT, WHERE, ORDER BY, JOIN (2-way, 3-way), GROUP BY
  - Result size impact testing (10 rows, 1K, 50K)
  - Performance targets met: <50ms for 100K row SELECT, <200ms for 10K row JOIN

### What's New in v0.8.0

**v0.8.0 - SQL Query Mode:**
- **Enter Insert mode:** `i`, `a`, `I`, `A`, `s`, or `F2` on any cell
  - `i` - edit at cursor position
  - `a` - edit at end of cell
  - `I` - edit at start of cell
  - `A` - same as `a`
  - `s` - clear cell and enter Insert mode
  - `F2` - edit cell (Excel/Calc style)
- **Text editing in Insert mode:**
  - Type to insert characters
  - `Backspace` or `Ctrl+h` - delete before cursor
  - `Delete` - delete at cursor
  - `Ctrl+w` - delete word backward
  - `Ctrl+u` - delete to start of cell
  - `Home`/`End` - move to start/end
  - `Left`/`Right` arrows - move cursor within cell
- **Commit or cancel:**
  - `Enter` - save and move down
  - `Shift+Enter` - save and move up
  - `Tab` - save and move right
  - `Shift+Tab` - save and move left
  - `Esc` - cancel without saving
- **Row operations in Normal mode:**
  - `o` - add row below, enter Insert mode
  - `O` - add row above, enter Insert mode
  - `dd` - delete current row (stored in clipboard)
  - `yy` - copy current row
  - `p` - paste row below
  - `Delete` - clear current cell content

### Previous Versions

**v0.3.0-v0.3.2 - Navigation & UI Polish:**
- Row jumping: `gg`, `G`, `5G`
- Column jumping: `:c A`, `:c 1`, `:c AA`
- Count prefixes: `5j`, `10h`
- Word motion: `w`, `b`, `e`
- Viewport control: `zt`, `zz`, `zb`
- Status bar with mode indicators
- Help overlay with `?`
- No timeout on pending commands

### What's New in v0.3.0 & v0.3.1

**v0.3.0 - Advanced Navigation:**
- Column jumping with Excel notation (`ga`, `gB`, `gBC`)
- Vim-style command mode (`:15` for line)
- Word motion for sparse data (`w`, `b`, `e`)
- Viewport positioning (`zt`, `zz`, `zb`)
- Count prefixes (5j, 10h, etc.)

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
