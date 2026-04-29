# lazycsv

A blazingly fast terminal UI for CSV files. Navigate huge datasets with vim keys, switch between files instantly, and never touch your mouse.

Inspired by [lazygit](https://github.com/jesseduffield/lazygit), [lazydocker](https://github.com/jesseduffield/lazydocker), and [lazysql](https://github.com/jorgerojas26/lazysql).

## Why LazyCSV?

- **Fast** - 100K+ rows at 60 FPS (in-memory)
- **Vim keys** - hjkl your way through data, full vim emulation planned
- **Multi-file** - switch between CSVs like Excel sheets (press `[` `]`)
- **Simple** - no config needed, just works
- **Clean** - minimal vim-like UI, zero clutter

**Note:** LazyCSV loads the entire CSV file into memory for maximum performance. This design choice prioritizes speed and simplicity over handling files larger than available RAM.

## Install

### Pre-built binaries (recommended)

Download the latest release for your platform from [GitHub Releases](https://github.com/funkybooboo/lazycsv/releases/latest).

**macOS (Apple Silicon)**
```bash
curl -L https://github.com/funkybooboo/lazycsv/releases/latest/download/lazycsv-v0.24.4-aarch64-apple-darwin.tar.gz | tar xz
sudo mv lazycsv /usr/local/bin/
```

**macOS (Intel)**
```bash
curl -L https://github.com/funkybooboo/lazycsv/releases/latest/download/lazycsv-v0.24.4-x86_64-apple-darwin.tar.gz | tar xz
sudo mv lazycsv /usr/local/bin/
```

**Linux (x86_64)**
```bash
curl -L https://github.com/funkybooboo/lazycsv/releases/latest/download/lazycsv-v0.24.4-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv lazycsv /usr/local/bin/
```

**Windows**

Download `lazycsv-v0.24.4-x86_64-pc-windows-msvc.zip` from the [releases page](https://github.com/funkybooboo/lazycsv/releases/latest), extract, and add to your PATH.

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

**v0.24.3 Complete** (April 2026) - Polish on v0.24.2: popup defaults are now `Color::Reset` so the no-config look is uniformly transparent (popups don't paint a contrasting bg over the table view). Dependency bumps: lru, criterion, comfy-table.

**v0.24.2** - Customizable keybindings + shell command in file browser. Data-driven keymaps via `~/.config/lazycsv/keys.toml` with three shipped presets (vim default, emacs, excel). `:` inside the file menu (`<space>f`) opens a "Shell (block):" prompt that runs commands in the current directory with `$CWD` / `$FILE` / `$NAME` / `$EXT` substitution. Builds on v0.24.0's TUI theming foundation.

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
- **v0.31.0** - Documentation & Maintainability
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

## What's New in v0.24.3

**Default-theme consistency + dep bumps (April 2026):**
- **Popup defaults are now `Color::Reset`** instead of `DarkGray`. With no config file, the SQL editor, help overlay, file menu, etc. inherit the terminal default rather than painting a hardcoded gray panel over an otherwise-transparent table. The 11 shipped theme presets still set their own popup colors explicitly, so users with a theme installed see no change.
- **`ui.border_fg` default also `Color::Reset`** — same rationale.
- **Dependency upgrades:** `lru` 0.17 → 0.18, `criterion` 0.5 → 0.8 (dev), `comfy-table` 7.1.4 → 7.2.2 (transitive). The criterion bump required swapping every `criterion::black_box` import for `std::hint::black_box` across the bench files (criterion deprecated its own version in favour of the std one).
- **CI test fixes:** `:sort`, `:wq`, and other ex-commands no longer get their `InputResult` swallowed by the keymap dispatcher. `gg` in the file menu jumps to the top again.

## What's New in v0.24.2

**Customizable keybindings (April 2026):**
- **Data-driven keymap** at `~/.config/lazycsv/keys.toml` (or per-directory `.lazycsv.toml`). Schema sections: `[normal]`, `[insert]`, `[visual]`, `[command]`, `[search]`, `[magnifier]`, `[file_list]`, `[sql_editor]`, `[file_operation]`, `[global]`. `[meta] inherit = "vim"` layers your overrides on top of the default; `inherit = "none"` starts from a blank slate.
- **3 presets ship under `keymaps/`:** `vim.toml` (default, baked into the binary), `emacs.toml` (readline-style: `Ctrl-f/b/n/p`, `Ctrl-a/e`, `Alt-x`, …), `excel.toml` (arrow-key navigation, `F2` to edit, `Ctrl-S/Z/Y`, `Tab/Enter` for data entry).
- **199 named actions** in the `Action` registry; every user-facing behaviour has a stable `snake_case` ID. Run `:keys` to see how many bindings are active.
- **Multi-key chords** (vim's `gg`, `dd`, `,yy`, `<space>q`) flow through the keymap. Parametric chords (`g{letter}` for column jump, `q{a-z}` for macro registers) still work — when the keymap can't resolve a chord, buffered keys are replayed through the legacy path.
- **Explicit unbinds** with `"i" = ""` actually silence a key (no fall-through to legacy hardcoded behaviour). Used in `excel.toml` so `i`/`a`/`v` can be typed as cell content.
- **Hot-reload** via the existing config watcher — edit `keys.toml` and the new bindings apply on save.

**Shell command in file browser (April 2026):**
- Press `:` inside the file menu (`<space>f`) to open a themed "Shell (block):" prompt. Whatever you type is executed via `$SHELL -c` in the file menu's current directory.
- **Variable substitution** before exec: `$CWD`, `$FILE`, `$NAME`, `$EXT` (all shell-quoted). Literal `$` escapable as `\$`; unknown `$<name>` tokens pass through.
- **TUI suspended** for the duration; aggressive screen clear on resume so terminal state never leaks into the table view.
- **Stdout discarded; stderr captured** (≤ 64 KiB). Exit-code outcomes: silent success / cream toast for warnings / red error toast on non-zero. Multi-line stderr opens a scrollable popup (`j/k` to scroll, `Esc` to dismiss).
- **Persistent shell history** at `~/.config/lazycsv/shell_history`. Up/Down walks past entries; configurable `[defaults] shell_history_limit` (default 50, 0 disables).

## What's New in v0.24.0

**TUI Theming (April 2026):**
- **Nested theme schema:** `[ui]` / `[table]` / `[popup]` / `[status]` / `[file_menu]` / `[sql]` sections in `~/.config/lazycsv/config.toml` (or per-directory `.lazycsv.toml`). Hard break — the old flat `[theme]` block is gone; see [`docs/themes.md`](docs/themes.md) for the migration map.
- **11 presets shipped under `themes/`:** Gruvbox dark/light, Dracula, Nord, Catppuccin Mocha/Macchiato/Frappé/Latte, Solarized dark/light, Tokyo Night. Each uses authentic upstream palette hex codes.
- **Full coverage:** every popup (help, file menu, SQL completion/history, formula completion, file-op prompt, stats overlay, context menu, magnifier), the status bar, title bar, and table chrome are theme-aware. A base-canvas pass paints the entire frame with `[ui].bg` so transparent terminals are fully covered.
- **Hot-reload:** save your config and changes apply instantly — the existing config watcher detects mtime changes and reloads.

## What's New in v0.23.0

**Performance Benchmarking & Tuning (April 2026):**
- **Benchmark suite:** Criterion benches for navigation, rendering, search, magnifier, and SQL paths under `benches/` — guards against regressions on the critical paths.
- **Performance work delivered earlier:** mmap-backed lazy loading for large CSVs, DuckDB-backed query engine, parallelized search and `:s` substitute via rayon, buffered + parallel sort, COPY+mmap fast path, contiguous-byte writes for unedited rows.

## What's New in v0.22.0

**Macros & Command History (April 2026):**
- **Macro recording:** `qa` to record into register `a`, `q` to stop, `@a` to replay, `@@` to repeat the last macro. 26 registers (a–z), with replay-depth and length guards.
- **Command history:** Persistent `:` command history at `~/.config/lazycsv/command_history`. Up/Down arrows in command mode walk through past entries; typed text is restored when you pass the newest entry.
- **`:history`:** Lists the most recent commands in the status bar.
- **Configurable:** New `[defaults] command_history_limit` (default 50; 0 disables).
- **Testing:** 30 new integration tests + 9 unit tests, all green.

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
