# lazycsv

A blazingly fast terminal UI for CSV files. Open a 10GB file instantly. Query it with SQL. Convert between any format. Edit cells with a full vim editor inside. All from your terminal, zero config needed.

Inspired by [lazygit](https://github.com/jesseduffield/lazygit), [lazydocker](https://github.com/jesseduffield/lazydocker), and [lazysql](https://github.com/jorgerojas26/lazysql).

## Why LazyCSV?

- **Billions of rows, instant open** - memory-mapped lazy loading. No loading screen, no waiting. Scroll through files larger than RAM like they're empty
- **SQL on CSV** - `:sql` opens a full query editor backed by DuckDB. JOIN across files, aggregate, filter, INSERT, UPDATE, DELETE — your data, your queries
- **Edit like vim** - insert mode for quick fixes, magnifier mode for a full vim editor inside any cell. Macros, visual mode, undo/redo. It's vim, in your data
- **Your keys, your way** - ships with vim, emacs, and excel presets. Customize every binding to match your workflow
- **File explorer** - browse all CSVs in a directory with a 3-column file picker. Switch files like Excel sheets, no `cd` required
- **Multi-format powerhouse** - open CSV, TSV, XLSX, ODS, JSON, Parquet. Export to any of them. Convert from the CLI: `lazycsv data.xlsx -o out.json`
- **CLI superpowers** - sort, dedup, generate synthetic data, query non-interactively, grab stats — all without opening the TUI
- **Highly configurable, great defaults** - works out of the box. 11 theme presets (Gruvbox, Dracula, Nord, Catppuccin, Tokyo Night...), 3 keybinding presets, and deep config when you want control. If you don't — it just works

## Features

### Performance

- **Lazy loading** - files under 10MB load into memory for instant access. Larger files use memory-mapped I/O with an LRU row cache, so you can open files bigger than your RAM
- **60 FPS rendering** - virtual scrolling only draws visible cells (O(1) per frame). Files with 100K+ rows render at 2500+ FPS
- **Parallel search** - regex search across 100K rows in ~20ms using rayon multi-core
- **DuckDB-backed queries** - SELECT on 100K rows in under 50ms, JOINs in under 200ms

### SQL Query Mode

Open a full SQL editor inside the TUI with `:sql`:

- **Full DuckDB SQL** - SELECT, WHERE, ORDER BY, GROUP BY, HAVING, JOINs, aggregates, string functions, math operators
- **DML support** - INSERT, UPDATE, DELETE, ALTER, DROP, CREATE
- **Multi-file JOINs** - all supported files in the same directory are auto-loaded as tables
- **SQL IntelliSense** - context-aware auto-completion for table names, column names, and SQL keywords
- **Fuzzy error suggestions** - mistyped column name? Levenshtein distance suggests the right one
- **Query history** - persisted to `~/.config/lazycsv/sql_history`, browse with Up/Down
- **Cancellable** - hit Esc to kill long-running queries
- **Results as virtual table** - query output appears as a new tab in the file switcher

### Editing

**Insert mode** (`i`, `a`, `s`) - quick single-line cell editing for fixes and updates:
- `Tab` saves and moves right, `Enter` saves and moves down, `Esc` cancels
- `Ctrl+w` deletes word, `Ctrl+u` deletes to start, `Ctrl+h` backspaces

**Magnifier mode** (`Enter`) - a full vim editor embedded inside any cell:
- All vim motions: `hjkl`, `w/b/e`, `0/$`, `gg/G`, `f/F/t/T`
- All vim operators: `d`, `c`, `y`, `p`, `r`, `x`, `J`, `>>/<<`
- Visual mode, search (`/`, `n`, `N`), undo/redo (`u`, `Ctrl+r`)
- `Alt+hjkl` to navigate to adjacent cells without closing
- Perfect for JSON, multi-line text, or any cell that needs real editing power

**Row and column operations:**
- `dd` delete row, `yy` yank row, `p`/`P` paste row, `5dd` delete 5 rows
- `,dd` delete column, `,yy` yank column, `,p`/`,P` paste column
- `o`/`O` insert row below/above, `,o`/`,O` insert column right/left
- `cc` clear row, `gj`/`gk` swap rows, drag with mouse to reorder

**Cell transforms:**
- `~` toggle case, `gU` uppercase, `gu` lowercase, `g~` title case
- `g.` toggle boolean values (yes/no, true/false, 1/0, on/off)
- `s/old/new/g` find and replace — in a cell, a range, or across all data

### Visual Mode

- `v` cell-by-cell selection, `V` row selection, `,v` column selection
- `d` delete, `y` yank, `c` change, `p` paste
- `o` toggle selection corner, `gv` re-select last visual range
- `gs` show statistics for selected cells

### Search

- `/` opens regex search with live results as you type
- `n`/`N` next/previous match, `*` search for current cell value
- Match counter shows `[current/total]` in status bar
- Case-insensitive by default with automatic regex fallback

### Undo, Redo, and Macros

- `u` undo, `Ctrl+r` redo — up to 1000 steps, configurable
- `.` repeats the last edit operation
- `qa`...`q` records a macro into register a-z, `@a` replays, `@@` repeats last
- Per-file undo history preserved across file switches

### File Explorer

`:files` opens a yazi-inspired 3-column file browser:

- **Parent | Current | Preview** layout
- Type to filter, `j/k` to navigate, `Enter` to open
- File operations: rename (`r`), delete (`d`), copy (`y`), move (`m`), new file (`n`)
- `:` opens a shell prompt with `$CWD`, `$FILE`, `$NAME`, `$EXT` variable substitution
- Navigate directories, spot-preview files before opening
- Quick switching between files with `[` and `]`

### Multi-Format Support

**Read:** CSV, TSV, CSV.GZ, TSV.GZ, XLSX, XLS, ODS, JSON, NDJSON/JSONL, Parquet, SQLite

**Write/Export:** CSV, TSV, JSON, Markdown, XLSX, ODS, Parquet

**From the TUI:** `:export json` or `:export data.xlsx`

**From the CLI:** `lazycsv data.xlsx -o out.json`, `lazycsv data.csv -o out.parquet`

**Pipe support:** `cat data.csv | lazycsv` or `lazycsv -q "SELECT * FROM stdin"`

### CLI Superpowers

Everything the TUI can do, you can do from the command line:

```bash
# SQL query non-interactively
lazycsv data.csv -q "SELECT * FROM data WHERE amount > 100"

# Sort by column
lazycsv data.csv --sort Name

# Print stats (type, min, max, mean, stddev, median, mode, cardinality)
lazycsv data.csv --stats

# Print headers, row count, or column count
lazycsv data.csv --headers
lazycsv data.csv --rows
lazycsv data.csv --columns

# Deduplicate with primary key support
lazycsv data.csv --dedup --keep-first

# Split a large file into smaller ones
lazycsv data.csv --split 10000

# Generate realistic synthetic data
lazycsv --generate customer --rows 10000 -o customers.csv

# Convert formats
lazycsv spreadsheet.xlsx -o data.json
lazycsv data.csv -o out.parquet

# Extract Excel sheets
lazycsv spreadsheet.xlsx -x

# Add headers to a headerless file
lazycsv data.csv --add-header "Name,Age,City"

# Clipboard integration
lazycsv data.csv -C          # copy output to clipboard
lazycsv -P                    # paste from clipboard into lazycsv

# Locale-aware number formatting
lazycsv data.csv --format
```

### Formulas

Excel-compatible formula engine for spreadsheet power users:

- **Aggregates:** `=SUM(A1:A5)`, `=AVERAGE(A1:A5)`, `=MIN(...)`, `=MAX(...)`, `=COUNT(...)`
- **Math:** `=POWER()`, `=CEILING()`, `=FLOOR()`
- **Text:** `=CONCAT()`, `=TRIM()`, `=UPPER()`, `=LOWER()`, `=PROPER()`, `=LEFT()`, `=RIGHT()`, `=MID()`, `=SUBSTITUTE()`
- **Logic:** `=IF(condition, true_val, false_val)` with comparison operators
- **Lookup:** `=VLOOKUP()`, `=HLOOKUP()`
- **Date:** `=NOW()`, `=TODAY()`, `=DATEDIF()`
- Formula bar displays the formula when cursor is on a formula cell
- Auto-completion popup for function names and cell references
- Imports formulas from XLSX spreadsheets

### Conditional Formatting

Color columns based on conditions:

```
:bgcolor Amount red ">1000"       # red background where Amount > 1000
:bgcolor Status green "=active"    # green where Status equals "active"
:fgcolor Name blue "~^admin"      # blue text where Name matches regex
:bgcolor # green "and(>0,<=100)"   # compound conditions
:clearview                         # remove all custom colors
```

Colors persist across sessions in `~/.config/lazycsv/views.json`.

### Sort, Stats, and Substitute

- `:sort Name` sort ascending by column, `:sort! Name` descending, multi-column: `:sort Dept,Name`
- `:stats` show column statistics overlay (count, sum, avg, min, max, distinct), `:stats A` for one column
- `:sum`, `:avg`, `:count`, `:distinct` for quick single-stat lookups
- `s/old/new/` replace in current cell, `%s/old/new/g` replace everywhere, `5,10s/old/new/g` in a row range, `B,Ds/old/new/g` in a column range

### Session and Persistence

- `:w` save, `:W` save all, `:wq` save and quit, `:q!` force quit
- Dirty file indicator (`*` after filename)
- External modification detection — polls every 2 seconds, prompts to reload
- View state (column widths, types, colors, frozen columns) saved per file across sessions
- `:freeze A,B` pin columns, `:freeze 1,3` pin rows, `:unfreeze` to remove
- `:type A number` annotate column types for proper formatting

### Configuration

**Default config location:** `~/.config/lazycsv/config.toml`  
**Per-directory overrides:** `.lazycsv.toml` in any directory

```toml
[defaults]
delimiter = ","
encoding = "utf-8"
zebra_striping = true
max_column_width = 50
undo_limit = 1000

# Full theme customization with 11 presets
[ui]
bg = "#282c34"
fg = "#abb2bf"

# Every keybinding remappable, 199 named actions
# Three presets: vim (default), emacs, excel
```

**Hot-reload** — save your config and changes apply instantly. No restart needed.

**11 built-in themes:** Gruvbox (dark/light), Dracula, Nord, Catppuccin (Mocha/Macchiato/Frappe/Latte), Solarized (dark/light), Tokyo Night

**3 keybinding presets:** `inherit = "vim"` (default), `inherit = "emacs"`, `inherit = "excel"`, or `inherit = "none"` for a blank slate. Every key in every mode is remappable.

### Mouse Support

- Click to select cells, click headers to select columns, click row gutters to select rows
- Double-click enters insert mode
- Right-click context menu (cut, copy, paste, clear, insert, delete)
- Drag column headers to reorder columns
- Drag row gutters to reorder rows
- Mouse wheel for scrolling

## Install

### Pre-built binaries (recommended)

Download the latest release for your platform from [GitHub Releases](https://github.com/funkybooboo/lazycsv/releases/latest).

**macOS (Apple Silicon)**
```bash
curl -L https://github.com/funkybooboo/lazycsv/releases/latest/download/lazycsv-v0.24.5-aarch64-apple-darwin.tar.gz | tar xz
sudo mv lazycsv /usr/local/bin/
```

**macOS (Intel)**
```bash
curl -L https://github.com/funkybooboo/lazycsv/releases/latest/download/lazycsv-v0.24.5-x86_64-apple-darwin.tar.gz | tar xz
sudo mv lazycsv /usr/local/bin/
```

**Linux (x86_64)**
```bash
curl -L https://github.com/funkybooboo/lazycsv/releases/latest/download/lazycsv-v0.24.5-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv lazycsv /usr/local/bin/
```

**Windows**

Download `lazycsv-v0.24.5-x86_64-pc-windows-msvc.zip` from the [releases page](https://github.com/funkybooboo/lazycsv/releases/latest), extract, and add to your PATH.

### Build from source

Requires the [Rust](https://rustup.rs/) toolchain. Install it first if you don't have it:

**macOS**
```bash
brew install rust
```
or
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Linux (Debian/Ubuntu)**
```bash
sudo apt install cargo
```
or for the latest Rust version:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Windows**

Download and run [rustup-init.exe](https://win.rustup.rs/) from the official site.

Then build lazycsv:

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
```

## Essential Keys

| Key | Action |
|-----|--------|
| `hjkl` or arrows | Move around (with count: `5j`, `10h`) |
| `gg` / `G` | Jump to first / last row |
| `5G` / `:5` | Jump to row 5 |
| `:B` / `:A5` | Jump to column B or cell A5 |
| `w` / `b` / `e` | Next/prev/last non-empty cell |
| `i` / `a` / `s` | Quick edit cell (Insert mode) |
| `Enter` | Magnifier — full vim editor inside the cell |
| `o` / `O` | Insert row below/above |
| `dd` / `yy` / `p` | Delete/yank/paste row |
| `,dd` / `,yy` / `,p` | Delete/yank/paste column |
| `v` / `V` / `,v` | Visual select (cell/row/column) |
| `/` | Search (regex, live results) |
| `n` / `N` | Next/previous match |
| `u` / `Ctrl+r` | Undo / Redo |
| `.` | Repeat last edit |
| `~` / `gU` / `gu` / `g~` | Toggle/upper/lower/title case |
| `g.` | Toggle boolean (yes/no, true/false) |
| `gj` / `gk` | Swap row down/up |
| `:sql` | SQL query mode (DuckDB-backed) |
| `:sort` / `:sort!` | Sort ascending/descending |
| `:stats` | Column statistics overlay |
| `:files` | File explorer (browse, open, manage files) |
| `:export` | Export to JSON, TSV, Markdown, XLSX, ODS, Parquet |
| `s/old/new/g` | Find and replace in cells |
| `qa`...`q` / `@a` | Record / replay macro (26 registers) |
| `Esc` | Cancel loading/queries |
| `zt` / `zz` / `zb` | Scroll row to top/center/bottom |
| `?` | Show help |
| `:w` / `:q` | Save / Quit |

Press `?` in the app for the full keybinding reference.

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

## Philosophy

LazyCSV follows the "lazy tools" design:

1. **Keyboard first** - mouse optional, every action has a key
2. **Fast** - instant response, lazy-loaded for infinite scale
3. **Simple** - no configuration required, sensible defaults
4. **Powerful** - vim-style efficiency, SQL queries, format conversion
5. **Vim-first** - if it works in vim, it should work here

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Credits

Built with:
- [ratatui](https://ratatui.rs/) - TUI framework
- [DuckDB](https://duckdb.org/) - SQL query engine
- [csv](https://docs.rs/csv/) - CSV parsing by BurntSushi
- Rust

Inspired by the excellent "lazy" tools:
[lazygit](https://github.com/jesseduffield/lazygit) |
[lazydocker](https://github.com/jesseduffield/lazydocker) |
[lazysql](https://github.com/jorgerojas26/lazysql) |
[lazyssh](https://github.com/anidude/lazyssh)

---

**Have fun exploring your data!**