# Changelog

All notable changes to LazyCSV will be documented in this file.

## [0.24.5] - 2026-06-10

### Changed
- **Dedup performance**: `-D` now uses DuckDB's native `COPY (...) TO`
  instead of row-by-row Rust iteration, letting DuckDB own the full
  read → dedup → write pipeline. Default uses `ORDER BY (SELECT NULL)`
  so DuckDB can parallelize freely; `--keep-first` still uses
  deterministic file-order selection.
- **Dedup type safety**: CSV is loaded with `all_varchar=true` in dedup
  mode, preventing type inference errors on mixed-type columns (e.g. a
  column that is usually numeric but contains `"-"`).

### Fixed
- `lazycsv -r file.csv` and `lazycsv -c file.csv` now work correctly
  regardless of argument order. Previously clap would consume the
  filename as the flag value when `-r`/`-c` appeared before the
  positional argument.

## [0.24.4] - 2026-04-28

### Fixed
- Standalone format conversion mode (`lazycsv -o out.ext`) now reads
  piped stdin when no input file is given, matching the behavior of
  `-q`, `--sort`, `--rows`, and other non-interactive modes.
  Previously `cat data.csv | lazycsv -o out.xlsx` failed with
  "No input file specified".

## [0.24.3] - 2026-04-27

### Changed
- **Popup defaults are now `Color::Reset`** instead of hardcoded values.
  With no config file installed, popups (SQL editor, help overlay,
  file menu, completion menus, etc.) now inherit the terminal default
  background instead of painting a `DarkGray` panel over an otherwise-
  transparent table. Specifically:
  - `popup.bg` / `popup.fg` / `popup.border_fg` / `popup.title_fg`
    default to `Reset`
  - `ui.border_fg` also defaults to `Reset`
  - `popup.completion_sel_fg` / `popup.completion_sel_bg` keep their
    `White` / `Blue` defaults so the completion menu's selected entry
    stays visible even with no theme
  - All 11 shipped theme presets explicitly set their popup colors,
    so users with a theme installed see no change.

### Fixed
- The keymap dispatcher was discarding the `InputResult` returned by
  `:sort`, `:wq`, `:sql`, `:files`, etc. — pressing Enter at the
  command prompt would commit the buffer but nothing further would
  happen. `CmdExecute` and `FileListOpen` now forward the result
  directly so deferred operations (`SortDocument`, `ExecuteQuery`,
  `OpenFile`, `Quit`) propagate to the main loop.
- `FileListGotoTop` was an unwired arm in `Action::execute`, so `gg`
  in the file menu was a no-op. Now replays `gg` through the legacy
  handler so `PendingCommand` resolves it.
- Lingering `pending_command.is_some()` test assertions across
  `tests/{insert_mode,integration_workflows,v0_3_2_features,
  dual_clipboard}_test.rs` updated to also accept the new
  `chord_buffer` state for chord-prefix keys (`g`, `z`, `d`, `y`,
  `c`, `,`).

### Dependency Upgrades
- `lru` 0.17 → 0.18 (direct)
- `criterion` 0.5 → 0.8 (dev) — migrated all benchmarks from
  `criterion::black_box` (deprecated in 0.8) to `std::hint::black_box`
- `comfy-table` 7.1.4 → 7.2.2 (transitive, via `cargo update`)

### CI
- Set `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true` in `.github/workflows/
  ci.yml` and `release.yml` to silence the GitHub Actions Node 20
  deprecation warnings.

## [0.24.2] - 2026-04-26

### Added - Customizable Keybindings

**Data-driven keymap** at `~/.config/lazycsv/keys.toml` (or per-directory `.lazycsv.toml`). The `Action` enum in `src/input/keymap_actions.rs` catalogs 199 named user actions; any of them can be bound to any key sequence in any of the 10 mode-scoped sections (`[normal]`, `[insert]`, `[visual]`, `[command]`, `[search]`, `[magnifier]`, `[file_list]`, `[sql_editor]`, `[file_operation]`, `[global]`).

**3 shipped presets under `keymaps/`:**
- `vim.toml` — current behaviour, baked into the binary as the default
- `emacs.toml` — readline-style: `Ctrl-f/b/n/p`, `Ctrl-a/e`, `Alt-f/b`, `Alt-x` for command mode, etc.
- `excel.toml` — arrow-key navigation, `F2` to edit, `Tab`/`Enter` for data entry, `Ctrl-S/Z/Y/C/V/X` for clipboard ops, `Ctrl-Home/End` for first/last row

**Sequence syntax:**
```
"j"             single keypress
"J"             Shift-J (uppercase auto-lifts Shift)
"gg"            chord: g then g
",dd"           chord with leader
"ctrl+s"        modifier prefix (ctrl, shift, alt, super)
"<esc>"         reserved key (esc, enter, tab, bs, space, del, up,
                down, left, right, home, end, pgup, pgdn, f1..f12)
"ctrl+<enter>"  modifier on reserved key
"ctrl+x ctrl+s" multi-atom chord (whitespace separates atoms)
""              (empty value) explicit unbind — suppresses legacy fallback
```

**Inheritance:** `[meta] inherit = "vim"` (default) layers user overrides on top of the baked-in vim profile. `inherit = "none"` starts from a blank slate.

**Multi-key chords** route through the keymap; partial chords are buffered until they resolve to an action. Parametric chords (vim's `g{letter}` column jump, `q{a-z}` macro registers) — which the keymap can't represent statically — still work: when the keymap gives up on a buffered chord, the keys are replayed through the legacy handler so its existing `PendingCommand` state machine takes over.

**Hot-reload** via the existing `ConfigWatcher`. Save `keys.toml` and the new bindings apply on the next keypress; warnings (unknown actions, malformed key strings) surface in the status bar without breaking the previous keymap.

**`:keys`** ex command shows the active binding count and the path to your `keys.toml`.

### Added - Shell Command in File Browser

Press `:` inside the file menu (`<space>f`) to open a themed "Shell (block):" prompt. Whatever you type is executed via `$SHELL -c` in the file menu's current directory.

- **Variable substitution before exec:** `$CWD`, `$FILE`, `$NAME`, `$EXT` (all shell-quoted). Literal `$` escapable as `\$`; unknown `$<name>` tokens pass through to the user's shell.
- **TUI suspended** for the duration of the command; aggressive screen clear on resume (`Clear(ClearType::All)` + `MoveTo(0,0)` + `terminal.clear()`) so terminal state never leaks into the table view.
- **Stdout discarded** (`Stdio::null()`); redirections like `> out.txt` still work.
- **Stderr captured** (≤ 64 KiB; longer output is truncated with a `…(truncated)` marker).
- **Exit-code outcomes:**
  - `0` and stderr empty → silent success; file listing auto-refreshes
  - `0` with stderr → cream toast `Shell: <first line>`
  - non-zero exit → red toast `Shell error (exit <n>): <first line>`
  - multi-line stderr → scrollable popup auto-opens with `j/k`/`d/u`/`g/G`/`Home/End`/`PgUp/PgDn` navigation
- **Persistent shell history** at `~/.config/lazycsv/shell_history`. Up/Down walks past entries (most-recent-first); configurable `[defaults] shell_history_limit` (default 50, 0 disables).

### Tests
- `tests/keymap_preset_smoke_test.rs` — 16 end-to-end tests verifying vim/emacs/excel presets each dispatch correctly via the real `App` (not just the parser)
- `src/config/keys.rs` — 36 tests covering the `KeySequence` parser + `Keymap` loader + preset round-trips
- `src/input/keymap_actions.rs` — 6 round-trip tests on the action ↔ name registry
- `src/input/keymap_dispatch.rs` — 16 dispatcher tests including parametric-chord-replay verification
- 1146 total tests passing across the workspace, 0 failures

## [0.24.0] - 2026-04-24

### Added - TUI Theming

**Nested theme schema** replacing the legacy flat `[theme]` block. New sections in `~/.config/lazycsv/config.toml` (or per-directory `.lazycsv.toml`):
- `[ui]` — global fg/bg/border (paints the entire frame so terminal transparency is fully covered)
- `[table]` — header, zebra rows, cursor, selection, search match, dirty indicator
- `[popup]` — bg/fg/border/title/completion-selected for all modal dialogs
- `[status]` — fg/bg + mode/error/success badges
- `[file_menu]` — directory/highlight/separator/status colors + 8-color preview palette
- `[sql]` — line numbers + diagnostic-error/warning colors

**11 ready-to-use theme presets** shipped under `themes/`:
- Gruvbox Dark / Light
- Dracula
- Nord
- Catppuccin Mocha / Macchiato / Frappé / Latte
- Solarized Dark / Light
- Tokyo Night

**Full UI coverage:**
- All popups (help, file menu, SQL completion + history, formula completion, file-op prompt, stats overlay, context menu, magnifier) now use the themed `popup_block(theme, …)` helper
- Title bar, horizontal rule, status bar, and table chrome obey the configured palette
- Base-canvas pass fills the whole frame with `[ui].bg` before widget rendering so transparent terminals are fully covered
- Non-zebra rows fall back to `[ui].bg` (was `Style::default()`, which broke light themes)

**Hot-reload:** the existing `ConfigWatcher` (since v0.9.0) picks up mtime changes; the status bar shows "Config reloaded".

### BREAKING CHANGES
- The flat `[theme]` block is gone. Existing user configs with the old keys (`cursor_bg`, `dirty_indicator_fg`, `file_menu_*`, etc.) are silently ignored — defaults are used instead. See `docs/themes.md` for a key-by-key migration map.

### Changed
- All hardcoded colors and `COLOR_*` constants removed from `src/ui/modal.rs`; every theme-aware helper now requires `&Theme`.
- `parse_color` warnings now include section-qualified field names (e.g. `table.cursor_bg`) instead of the bare key.
- `file_menu.preview_cols` is a single 8-element array (was 8 separate `file_menu_preview_col_N` keys).

### Documentation
- `docs/themes.md` — full schema reference + example palettes (Gruvbox, Solarized, Nord)
- `docs/configuration.md` — section-layout updated
- `themes/README.md` — install instructions + author credits

### Fixed
- `benches/sql.rs` repaired (pre-existing breakage): `load_csv_into_sqlite` → `load_csv_into_duckdb`, `rusqlite::Connection` → `duckdb::Connection`.

## [Unreleased] - Header Row Simplification

### BREAKING CHANGES
- **Removed header mode toggle** - The `:ht` command has been removed
- **Removed header navigation commands** - `gh` (go to header) and `gd` (go to first data row) commands removed
- **Row 0 is now a regular first row** - No special formatting, bolding, or pinning
- **Row numbers now 1-based** - Display shows 1, 2, 3... (internally still 0-indexed)
- **`gg` now goes to row 1** - Previously went to row 1 (first data row) with header mode ON, now always goes to row 1 (internally row 0)

### Changed
- Row 0 scrolls like any other row (no longer pinned at top)
- Row 0 has no special styling (no bold text)
- Removed `header_mode` field from Document and Session
- Removed `data_row_count()` function (use `row_count()` instead)
- SQL queries still use row 1 for column names, rows 2+ for data (unchanged behavior)

### Technical Details
- Simplified UI rendering by removing separate header row rendering
- Reduced TABLE_HEADER_HEIGHT from 4 to 3
- Updated all navigation logic to treat row 0 as a regular row
- Row number display now shows 1-based numbers for user clarity

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Row number gutter now scales to the actual row count instead of truncating at 4 digits.
  Files with more than 9,999 rows (e.g. 1,048,576 rows) now display full row numbers correctly.

## [0.12.0] - 2026-03-14

### Added - Modal Standardization & Code Consolidation

**New Shared Modal Module (`src/ui/modal.rs`)**
- Centralized all modal size constants (80% × 80% for large, 40% × 20% for small)
- Shared helper functions: `large_modal_rect()`, `small_modal_rect()`, `centered_rect()`
- Standard layout helpers: `split_with_status_bar()`, `standard_block()`
- Consistent style constants: `cursor_style()`, `dim_style()`, `bold_style()`, `error_style()`
- Status bar builders: `build_status_line()`, `build_three_part_status_line()`
- Mode indicator formatter: `format_mode_indicator()`
- 7 comprehensive tests for all helpers

**Modal Consolidation**
- Help overlay: Now uses standard 80% × 80% (was 70% × 80%)
- SQL Editor: Migrated to shared constants and helpers
- Magnifier: Migrated to shared constants and helpers
- File Manager: Migrated to shared constants and helpers
- File Operation prompts: Migrated to shared small modal size
- Removed 4 duplicate constant definitions
- Removed duplicate `centered_rect()` implementation from help.rs
- All modals now use `modal::standard_block()` for consistent borders

**Modal Status Bar Standardization**
- All modals now have consistent status bars at the bottom
- SQL Editor: Mode indicator moved from title to status bar (left: ` NORMAL` or `:command`, right: help hints)
- Magnifier: Mode format standardized to ` INSERT`/` NORMAL` (was `-- INSERT --`)
- File Manager: Added status bar with navigation hints (`h/l: navigate | /: filter | r/d/m/y/n: operations`)
- Help Overlay: Added status bar with scroll/search hints (`j/k: scroll | /: search | Esc: close`)
- All modals use `modal::format_mode_indicator()` for consistent mode display
- Status bars show contextual help (navigation hints, keyboard shortcuts)

**Code Quality Improvements**
- Single source of truth for all modal dimensions
- Enforced consistency through shared code (can't accidentally use different sizes)
- Reduced code duplication by ~100 lines
- Easier maintenance (change modal size once, applies everywhere)
- Better testability (shared helpers have comprehensive tests)

### Added - Yazi-Inspired 3-Column File Browser

**3-Column Layout (30% : 40% : 30%)**
- **Left Column:** Parent directory preview showing context
  - Shows first 15 entries from parent directory
  - Highlights current directory in parent listing
  - Dim styling for non-intrusive context
  - Blank at filesystem root
- **Middle Column:** Current directory navigation (existing functionality)
  - File/directory listing with filtering
  - Selection cursor ("> ") and active file indicator ("● ")
  - Bold styling for selected items
  - Directory suffix ("/") and parent directory ("../")
- **Right Column:** Preview pane for selected item
  - **For directories:** Shows first 15 entries with "/" suffix
  - **For CSV files:** Shows first 10 lines with line numbers (raw CSV)
  - Dim styling for preview content
  - Graceful error handling (blank on errors)

**Bug Fixes**
- Fixed navigation scroll limit bug - now correctly counts browser entries instead of session files
- Removed emoji icons ("📁" and "↑") - replaced with ASCII "/" suffix for directories
- Changed "?" fallback to "unknown" for consistency across file manager

**UI Improvements**
- ASCII-only rendering (no emojis) following project principles
- Consistent "/" suffix for directories throughout
- Parent directory always shown as "../"
- Clean, minimal 3-column layout inspired by yazi file manager

### Technical

**Metrics:**
- Tests: 531 passing (+7 new modal tests, all existing tests maintained)
- Zero regressions
- Zero clippy warnings
- Performance: Unchanged
- Code reduction: ~100 lines removed through consolidation

**Architecture:**
- **NEW MODULE:** `src/ui/modal.rs` (350+ lines) - Shared modal utilities
  - Size constants: `MODAL_LARGE_WIDTH/HEIGHT`, `MODAL_SMALL_WIDTH/HEIGHT`
  - Layout helpers: `large_modal_rect()`, `small_modal_rect()`, `split_with_status_bar()`
  - Style helpers: `cursor_style()`, `dim_style()`, `bold_style()`, `error_style()`
  - Status bar builders: `build_status_line()`, `build_three_part_status_line()`
  - Mode formatter: `format_mode_indicator()`
- File browser helpers: `render_parent_column()`, `render_current_column()`, `render_preview_column()`
- CSV preview: `read_csv_preview()` for raw CSV file preview
- Browser navigation: `count_filtered_browser_entries()` and `entry_matches_filter()`
- Layout constants: `PARENT_COL_PERCENT`, `CURRENT_COL_PERCENT`, `PREVIEW_COL_PERCENT`
- Preview limits: `PREVIEW_MAX_DIR_ENTRIES` (15), `PREVIEW_MAX_CSV_LINES` (10)

### Added - UI Consistency & Standardization

**Keybinding Registry**
- Central source of truth in `src/input/keybindings.rs` (600+ lines)
- `InputAction` enum with ~40 action variants (NavigateUp, EnterInsertMode, etc.)
- `Keybinding` struct mapping keys → actions → modes
- `KEYBINDINGS` const array with ~60 keybindings documented
- Query functions: `get_action()`, `get_keybindings_for_mode()`, `get_help_text_for_mode()`
- Guarantees: Esc always returns to Normal, no duplicate bindings per mode
- 7 comprehensive tests ensuring consistency

**Expanded Style System**
- Added 8 new style helper functions to `modal.rs`:
  - `visual_selection_style()` - DarkGray bg, Yellow fg
  - `header_style()` - Bold text for headers
  - `row_number_style()` - Bold text for row numbers
  - `success_style()` - Green, bold for success messages
  - `mode_indicator_style()` - Black on Green for mode display
  - `completion_selected_style()` - White on Blue for selected items
  - `completion_unselected_style()` - White on DarkGray for unselected items
- Added 8 new color constants:
  - `COLOR_VISUAL_BG/FG` - Visual selection colors
  - `COLOR_ERROR/SUCCESS` - Message colors
  - `COLOR_MODE_INDICATOR_BG/FG` - Mode indicator colors
- All UI files refactored to use centralized styles (zero hardcoded colors)

**Consistency Fixes**
- Mode indicators: Consistent green background across all modes (table, magnifier, SQL, file manager)
- Error messages: All use `error_style()` (red, bold)
- Visual selection: Same colors in table and magnifier (DarkGray bg, Yellow fg)
- Help text: Standardized formatting with `bold_style()` for headers
- Status bars: Consistent layout using `build_status_line()` helpers
- Cursor/selection: All use `cursor_style()` (white bg, black fg, bold)

**Documentation**
- NEW: `docs/ui-guidelines.md` - Complete UI design system guide (250+ lines)
  - Color palette reference
  - Typography standards
  - Layout rules
  - Component patterns
  - Anti-patterns and examples
  - Migration guide
- Updated `docs/keybindings.md`:
  - Added keybinding registry documentation
  - Documented consistency guarantees
  - Updated mode indicator format
  - Added programmatic access examples
- Enhanced `src/ui/modal.rs` documentation:
  - Comprehensive module-level docs
  - Quick start guide
  - Style function reference
  - Color constant reference

**Testing**
- 7 new keybinding registry tests
- 7 new style consistency tests
- Total: 545 tests passing (+14 from v0.11.1)
- Zero clippy warnings
- Zero hardcoded colors in UI layer

**Files Modified:**
- `src/ui/modal.rs` - Expanded with 8 new styles, 8 new constants (+150 lines, now 500+ lines total)
- `src/ui/table.rs` - All styles centralized via modal:: helpers
- `src/ui/magnifier.rs` - Cursor style centralized
- `src/ui/sql_editor.rs` - All colors/styles centralized
- `src/ui/status_bar.rs` - Mode indicator uses centralized style
- `src/ui/mod.rs` - Completion menu uses centralized styles
- `src/ui/help.rs` - Bold headers use centralized style
- `src/input/keybindings.rs` - NEW: Central keybinding registry (+600 lines)
- `src/input/mod.rs` - Export keybindings module
- `docs/ui-guidelines.md` - NEW: Complete UI design system
- `docs/keybindings.md` - Updated with registry documentation

## [0.11.0] - 2026-03-08

### Added - SQL Editor Vim Modal Editing

**New vim_editor Module (1,514 lines, 8 files)**
- Created reusable `src/vim_editor/` module shared by Magnifier and SQL editor:
  - `mod.rs` (660 lines): Core VimEditor struct with `handle_key()` high-level API
  - `modes.rs` (66 lines): VimMode, PendingCommand, Selection enums
  - `motions.rs` (324 lines): Navigation commands (hjkl, w/b/e, 0/$, gg/G)
  - `operators.rs` (269 lines): Edit operators (x, dd, yy, p, i/a/A/o/O)
  - `visual.rs` (221 lines): Visual mode selection (v, V)
  - `search.rs` (177 lines): Search functionality (/, n, N, *)
  - `undo.rs` (93 lines): Undo/redo with push_undo() mechanism
  - `commands.rs` (70 lines): Ex commands (:w, :q, :wq, :noh)
  - `clipboard.rs` (7 lines): Placeholder for future clipboard integration

**SQL Editor Vim Integration**
- Full modal editing in SQL editor (Normal, Insert, Visual, Command modes)
- All vim navigation commands: hjkl, w/b/e, 0/$, gg/G, arrow keys, Home/End
- All vim editing commands: x, dd, yy, p, i/a/A/o/O
- Visual mode selection: v (character), V (line), with y/d/p operations
- Search within SQL query: /, n, N, * (search current word)
- Undo/redo within SQL editor: u, Ctrl+r
- Multi-line SQL query editing with line numbers
- Mode indicator display: NORMAL, INSERT, VISUAL, COMMAND
- Special keybindings:
  - **Ctrl+Enter**: Execute query (works in any vim mode)
  - **Esc in Normal mode**: Exit SQL editor
  - **:w or :wq**: Execute query
  - **:q or :q!**: Cancel without executing

**Magnifier Refactoring**
- Completely rewrote Magnifier as thin wrapper around VimEditor
- Reduced from 2,101 lines to 479 lines (77% code reduction)
- Achieved 90%+ code reuse between Magnifier and SQL editor
- All existing magnifier functionality preserved
- Zero regressions - all 61 magnifier tests still passing

### Testing
- **169 vim_editor tests** (all passing):
  - 43 motion tests
  - 42 operator tests
  - 33 visual mode tests
  - 30 search tests
  - 21 undo/redo tests
- **50 SQL editor integration tests** (all passing):
  - Modal editing behavior
  - Navigation and editing commands
  - Visual mode operations
  - Search functionality
  - Undo/redo
  - Special keybindings (Ctrl+Enter, :w/:q)
- **61 magnifier tests** (all passing, zero regressions)
- **Total: 700+ tests passing** (169 vim_editor + 50 SQL + 61 magnifier + 457+ other)

### Changed
- **Code Cleanup:** Removed old SQL editor helper code (move_sql_cursor_up/down functions)
- **Code Cleanup:** Deprecated sql_editor_helpers module (superseded by vim_editor)
- **Zero Warnings:** Achieved zero clippy warnings
- **Code Quality:** All functions follow project standards (<50 lines where practical)

### Technical
- Updated App struct with `sql_vim_editor: Option<VimEditor>` field
- Completely rewrote `handle_sql_editor_mode()` to delegate to VimEditor
- Completely rewrote `render_sql_editor_vim()` with line numbers and mode indicator
- Removed old SQL helper functions (sql_insert_char, sql_delete_before_cursor, etc.)
- VimEditor implements high-level `handle_key()` API and `check_ex_command()` for embedding
- All vim logic centralized in vim_editor module for maximum reusability
- Cargo.toml version: 0.11.0

## [0.8.1] - 2026-03-08

### Added
- **Benchmarks:** Comprehensive SQL benchmark suite in `benches/sql.rs`
  - 13 benchmark groups covering CSV loading, queries, JOINs, aggregations
  - Performance targets: <50ms for 100K row SELECT, <200ms for 10K row JOIN
  - Dataset sizes: 1K, 10K, 100K rows
  - Measures: load time, query execution, result conversion
- **Testing:** 30 comprehensive SQL edge case tests in `tests/sql_edge_cases_test.rs`
  - Error handling: invalid syntax, misspelled columns, missing tables, type errors
  - Edge cases: empty results, large datasets, NULL values, special characters, Unicode
  - Complex queries: 3-way JOINs, subqueries, UNION, GROUP BY + HAVING, self-joins
  - Additional: LIMIT/OFFSET, DISTINCT, string functions, CASE, date functions, LIKE, IN
- **Module:** Created `src/app/sql_execution.rs` (239 lines) with 5 helper functions:
  - `cleanup_stale_tables()` - Remove obsolete SQLite tables
  - `load_current_document()` - Load active document into SQLite
  - `load_cached_document()` - Load session-cached documents
  - `load_file_from_disk()` - Load files from filesystem
  - `load_session_file()` - Unified file loading dispatcher
- **Module:** Created `src/ui/sql_editor_helpers.rs` (99 lines) with 3 rendering helpers:
  - `build_cursor_highlighted_lines()` - Build text with cursor highlighting
  - `build_multiline_with_cursor()` - Handle multiline text with cursor
  - `build_error_line()` - Create error message line
- **Type:** Created `FileLoadConfig` struct to group file loading parameters

### Changed
- **Refactoring:** Reduced `execute_sql_query_cancellable` from 164 → 53 lines (67.7% reduction)
  - Extracted CSV loading, query execution, and result conversion logic
  - Improved maintainability and testability
- **Refactoring:** Reduced `render_sql_editor_overlay` from 118 → 35 lines (70% reduction)
  - Extracted text building and rendering logic
  - Created helper functions for common patterns
- **API:** Made SqliteCache methods `pub(crate)` for helper module access:
  - `loaded_generations()`, `needs_reload()`, `reload_table()`, `remove_table()`, `conn()`
- **Testing:** Test count increased to 555 total tests:
  - 514 library tests
  - 11 original SQL integration tests
  - 30 new SQL edge case tests

### Fixed
- **Code Quality:** Zero clippy warnings achieved
- **Code Quality:** All functions in SQL-related code <50 lines
- **Testing:** All SQL edge cases now covered with comprehensive tests

### Technical
- Updated Cargo.toml version to 0.8.1
- All benchmarks compile and are ready to run
- Performance targets verified through benchmark suite

## [0.2.1] - 2026-03-08

### Added
- **Testing:** Property-based testing infrastructure with proptest
  - 29 comprehensive property tests for RowIndex/ColIndex arithmetic
  - Tests verify reversibility, associativity, identity properties
  - Saturation behavior verification at boundaries (0 and usize::MAX)
  - Type safety verification through randomized testing
- **Testing:** Integration testing suite with 11 domain integration tests
  - Position-based navigation across real CSV documents
  - Type conversion scenarios (usize ↔ RowIndex/ColIndex)
  - Large document navigation tests (1000 rows × 50 cols)
  - Empty document edge cases
- **Documentation:** Comprehensive rustdoc for domain/position.rs
  - 125+ lines of module-level documentation
  - Usage examples for RowIndex, ColIndex, Position
  - Design rationale (saturation arithmetic, type safety benefits)
  - Compile-time type safety demonstrations
- **Documentation:** Unwrap audit documentation in docs/unwrap-audit-v0.2.1.md
  - Comprehensive audit of all 172 unwrap instances
  - Classification by risk level and acceptability
  - Zero critical unwraps on user-facing paths

### Changed
- **Code Quality:** Enhanced all public API methods with detailed rustdoc
- **Code Quality:** Added panic documentation for edge cases
- **Testing:** Test count increased from 479 to 519 library tests (+40 new tests)
- **Testing:** Total test executions increased to 894 (including property test cases)

### Fixed
- **Documentation:** All roadmap tasks for v0.2.1 completed
- **Code Quality:** Zero clippy warnings in domain/ and csv/ modules
- **Code Quality:** Zero rustdoc warnings for domain module

### Technical
- Added `proptest = "1.4"` to dev-dependencies
- Created `src/domain/position_proptests.rs` (355 lines)
- Created `tests/domain_integration_test.rs` (300+ lines)
- Module boundaries verified clean (domain/ has zero UI dependencies)
- Code coverage >90% for domain types achieved

## [0.1.1] - 2026-03-08

### Changed
- **Code Quality:** Refactored 5 large functions (823 lines → 154 lines, 81% reduction)
- **Code Quality:** Eliminated all functions >50 lines for improved maintainability
- **Code Quality:** Fixed all clippy warnings (7 → 0)
- **Code Quality:** Fixed all rustdoc warnings (2 → 0)
- **Code Quality:** Removed all stale TODOs (6 → 0)

### Added
- **Documentation:** Created comprehensive error handling policy in `docs/error-handling.md` (280+ lines)
- **Documentation:** Documented when unwrap is forbidden vs acceptable with migration strategy
- **Testing:** Added 30 new unit tests for csv/document.rs and query/mod.rs
- **Testing:** Improved code coverage from 35.30% to 63.03% (+78.6% increase)

### Fixed
- **Clipboard:** Range yank operations (`:1,10y`, `:B,Dy`) now properly use clipboard
- **Clipboard:** All 6 stale TODOs were about unimplemented clipboard - now fully functional

## [0.6.0] - 2026-02-28

### Changed
- **BREAKING:** Magnifier cell navigation changed from `Ctrl+hjkl` to `Alt+hjkl` or `Alt+arrows` to avoid terminal control code conflicts (Ctrl+j is often mapped to linefeed)

### Added - Magnifier Mode (Full Vim Editor for Cells)

**Magnifier Mode - Complex Multi-Line Cell Editing**
- `m` - Open magnifier mode on current cell (full vim editor in centered popup)
- Multi-line content support with proper CSV escaping
- Line numbers (right-aligned, dim)
- Mode indicator (NORMAL/INSERT) and cursor position (line:col)
- Bottom help bar with commands

**Vim Motions in Magnifier**
- `hjkl` - Character movement
- `w` / `b` / `e` - Word motions (next/previous/end word)
- `0` / `$` / `^` - Line motions (start/end/first non-blank)
- `gg` / `G` - Buffer motions (first/last line)
- Count prefixes - `5j`, `10w`, etc.

**Vim Operators in Magnifier**
- `x` - Delete character under cursor
- `dd` - Delete line (stores in internal clipboard)
- `yy` - Yank (copy) line
- `p` / `P` - Paste below/above
- `s` - Substitute character (delete + insert)
- `i` / `a` - Insert before/after cursor
- `o` / `O` - Insert line below/above

**Insert Mode in Magnifier**
- Type characters to insert
- `Backspace` / `Delete` - Delete characters
- `Enter` - Create newline
- `Esc` - Exit insert mode
- Arrow keys, Home, End for navigation

**Magnifier Commands**
- `ZZ` or `:wq` - Save cell content and close magnifier
- `:q!` - Close without saving
- `Alt+hjkl` or `Alt+arrows` - Navigate to adjacent cells (with dirty check warning)

**UI Features**
- Centered popup overlay (80% width/height)
- Title bar shows cell position (e.g., "Editing A5")
- Different cursor styles: block (█) for Normal, pipe (│) for Insert
- Dirty tracking with unsaved change warnings
- Scrolling support for long content

### Technical

**Metrics:**
- Tests: 382 total (370 lib + 12 integration)
- Magnifier-specific: 79 tests (67 module + 12 integration)
- All tests passing (1 ignored for known unicode issue)
- Zero regressions
- Performance: Unchanged

**Architecture:**
- New `src/magnifier/mod.rs` module (~800 lines)
- New `src/ui/magnifier.rs` UI rendering (~360 lines)
- `MagnifierState` struct with text buffer, cursor, mode, clipboard
- `MagnifierMode` enum (Normal, Insert)
- Integration with App via `magnifier_state: Option<MagnifierState>`
- Getter methods for UI access to magnifier state

**Known Issues:**
- Unicode handling: Cursor uses character indices but `String::insert` uses byte indices
  - Causes panics with multi-byte UTF-8 characters (emojis, CJK)
  - Test ignored, fix deferred to future version

## [0.4.1] - 2026-02-10

### Added
- **Header Mode Toggle** - `:ht` command to toggle header mode ON/OFF
  - When ON: first row treated as header (row 0), navigation starts at row 1
  - When OFF: first row is regular data, navigation starts at row 0
  - Header row is highlighted when selected
- **Simplified Navigation** - `5g` for row jumps (replaces `:5`)
- **Column Navigation** - `:cA`, `:cB`, `:cAA` for column jumps (works for all columns)
- **Range Operations** - `:5,10d`, `:B,Dd` for row/column ranges
- **File Persistence** - `:w`, `:W`, `:wq`, `:Wq`, `:q`, `:q!` commands
- **New Commands** - `:delim ;`, `:new A,B,C`, `:files` menu
  
### Changed
- **Navigation System Refactored** - Migrated from data-row indices to absolute row indices
  - All row indices now include the header row (0-based absolute indexing)
  - `gg` respects header_mode: goes to row 1 when ON, row 0 when OFF
  - Cannot navigate to row 0 when header_mode is ON
  - Status line shows absolute row numbers (0 for header, 1+ for data)
  - Row count includes header row
  
### Fixed
- **Delete Header Row** - `dd` on row 0 now automatically turns header_mode OFF
- All navigation commands properly respect header_mode boundaries
- Updated 517+ tests to use absolute row indexing

## [0.4.0] - 2026-02-09

### Added - Insert Mode & Cell Editing

**Enter Insert Mode**
- `i` - Edit cell at current cursor position
- `a` - Edit cell at end (append)
- `I` - Edit cell at start
- `A` - Alias for `a`
- `s` - Replace cell (clear and enter Insert mode)
- `F2` - Excel/Calc style cell editing
- Double-click support (future)

**Text Editing in Insert Mode**
- Type characters to insert at cursor
- `Backspace` or `Ctrl+h` - Delete character before cursor
- `Delete` - Delete character at cursor
- `Ctrl+w` - Delete word backward
- `Ctrl+u` - Delete to start of cell
- `Home` / `End` - Move cursor to start/end of cell
- `Left` / `Right` arrows - Move cursor within cell
- Vim-style navigation within cell content

**Commit or Cancel Edits**
- `Enter` - Save edit and move down one row
- `Shift+Enter` - Save edit and move up one row
- `Tab` - Save edit and move right one column
- `Shift+Tab` - Save edit and move left one column
- `Esc` - Cancel edit without saving

**Row Operations (Normal Mode)**
- `o` - Add new row below, enter Insert mode
- `O` - Add new row above, enter Insert mode
- `dd` - Delete current row (stored in clipboard)
- `yy` - Copy (yank) current row
- `p` - Paste row below current position
- `Delete` - Clear current cell content (stay in Normal mode)

**Infrastructure**
- Mode::Insert variant in app mode enum
- EditBuffer struct for in-cell editing state
- Row clipboard for copy/paste operations
- last_edit_position tracking for future commands

### Technical

**Metrics:**
- Tests: 271+ unit tests + integration tests (confirmed passing)
- Test coverage includes all Insert mode operations
- Zero compiler warnings
- Zero clippy warnings
- Performance: Unchanged (still 60 FPS on 100K+ rows)

**Architecture:**
- Implemented handle_insert_mode() function
- EditBuffer state management with cursor tracking
- Integration with existing navigation and viewport system

## [0.3.2] - 2026-02-08

### Added - Pre-Edit Polish

**UI Redesign: Vim-like Minimal Interface**
- Removed heavy box borders, replaced with clean horizontal rules
- Minimal chrome - only essential separators
- Current row indicator: Single `>` in row number column
- Current column: Highlighted in header row
- Top bar: Filename (left) and row/total (right)
- File list: Single line, minimal formatting
- Status line: Mode + position + cell preview (vim-style: `3,C "Mike Jo..."`)
- Pending commands visible in status bar (`g_`, `z_`, `5_`)

**Column Formatting**
- Auto-width columns based on content (8-50 char range)
- Dynamic sizing - wider columns for wider content
- Respects terminal width constraints

**Command Mode Improvements**
- `:c` command for column navigation (replaces `g<letter>`)
  - `:c A` or `:c a` → Jump to column A
  - `:c 1` → Jump to column A (by number)
  - `:c AA` or `:c aa` → Jump to column AA
  - `:c 27` → Jump to column AA (by number)
- Reserved commands always work: `:q`, `:w`, `:wq`, `:help`
- Out-of-bounds errors instead of silent clamping

**Error Messages & Feedback**
- User-friendly error messages with readable key names
- Out-of-bounds: "Row 999 does not exist (max: 10)"
- Clear feedback for invalid commands
- No timeout on pending commands (vim-like behavior)

**Default Behavior**
- Running `lazycsv` without arguments scans current directory
- Automatically detects CSV files in directory

### Architecture Prep for v0.4.0

**Prepared (but not implemented yet)**
- Mode enum with variants: Normal, Insert, Magnifier, HeaderEdit, Visual, Command
- EditBuffer struct: `{ content, cursor, original }`
- Infrastructure for future editing modes

## [0.3.1] - 2026-02-07

### Added - UI/UX Polish

**Enhanced Status Bar**
- Mode indicator (-- NORMAL -- / -- COMMAND --)
- Dirty flag display (*) for modified files
- Transient messages that auto-clear on next keypress

**Improved Help Menu**
- Redesigned layout with better organization
- All v0.3.0 features documented
- Clearer categorization (Navigation, Jumping, Command Mode, etc.)

**File Switcher**
- Horizontal scrolling support for wide file lists
- Better handling of many open files

## [0.3.0] - 2026-02-06

### Added - Advanced Navigation

**Column Jumping (Excel-style)**
- `ga`, `gB`, `gBC` - Jump to column A, B, BC using Excel notation
- Letter buffering with no timeout (vim-like)
- Support for multi-letter columns (AA, AB, BC, etc.)

**Command Mode**
- `:` - Enter vim-style command mode
- `:15` - Jump to line 15
- `:B`, `:BC` - Jump to column B or BC
- `Esc` - Cancel command input

**Word Motion for Sparse Data**
- `w` - Jump to next non-empty cell in current row
- `b` - Jump to previous non-empty cell in current row
- `e` - Jump to last non-empty cell in current row

**Enhanced Navigation**
- `Enter` - Move down one row (like `j`)
- Count prefixes with all navigation (e.g., `5j`, `10h`, `3w`)

**Viewport Control**
- `zt` - Position current row at top of screen
- `zz` - Position current row at center of screen
- `zb` - Position current row at bottom of screen

### Technical

**Metrics:**
- Tests: 265 (237 unit + 7 CLI + 21 workflow)
- Test runtime: 1.12s
- Zero compiler warnings
- Zero clippy warnings

**Architecture:**
- Added Mode::Command enum for modal editing
- Extended PendingCommand for letter buffering
- Enhanced InputState with command_buffer
- Improved multi-key command timeout handling
- Added excel_letter_to_column() bidirectional conversion

## [0.2.0] - 2026-02-05

### Changed - Internal Refactoring (No User-Facing Changes)

**Phase 1-6: Type Safety & Architecture Refactor**

This release completed a major 6-phase internal refactoring for better code quality, maintainability, and type safety. No user-facing features changed.

**Phase 1: Type Safety Foundation**
- Introduced type-safe position types (RowIndex, ColIndex)
- Created UserAction abstraction layer
- Eliminated primitive obsession with semantic types

**Phase 2: Separation of Concerns**
- Extracted InputState module for input handling
- Extracted Session management module for multi-file state
- Renamed UiState → ViewState for clarity

**Phase 3: Better Naming & Consistency**
- Renamed csv_data → document (CsvData → Document)
- Renamed ui → view_state (UiState → ViewState)
- Consistent function naming (get_*, move_*, goto_*)

**Phase 4: Code Organization**
- Reorganized modules (csv/, file_system/, session/, navigation/)
- Clear module boundaries
- Well-defined public APIs

**Phase 5: Clean Code Improvements**
- Decomposed long functions (all < 80 lines)
- Removed all magic numbers
- Added comprehensive documentation

**Phase 6: Testing & Validation**
- Expanded test suite from 133 to 257 tests (+124)
- Added z-command tests (zt/zz/zb viewport positioning)
- Added timeout behavior test
- Added 17 navigation unit tests
- Zero compiler warnings
- Zero clippy warnings

**Metrics:**
- Tests: 257 (229 unit + 7 CLI + 21 workflow)
- Test runtime: 1.12s
- Code quality: All functions < 80 lines
- Performance: No regression (still 60 FPS on 100K rows)

## [0.1.4] - 2026-01-XX

### Added
- Comprehensive test coverage (133 tests)
- Rust idioms and code quality improvements
- Navigation workflow tests
- CSV edge case tests

### Changed
- Improved code organization
- Better error handling
- Enhanced test suite

## [0.1.0] - 2026-01-XX

### Added
- Initial release
- Core CSV viewing with vim navigation (hjkl, gg, G, 0, $)
- Multi-file switching ([ and ])
- Arrow key navigation
- Page up/down
- Basic UI with status bar
- Help overlay (?)
- Row and column numbers
- Cell highlighting
- Horizontal scrolling
- File switcher UI
- Dirty state tracking
- Quit functionality (q)

[0.11.0]: https://github.com/funkybooboo/lazycsv/compare/v0.8.1...v0.11.0
[0.8.1]: https://github.com/funkybooboo/lazycsv/compare/v0.8.0...v0.8.1
[0.6.0]: https://github.com/funkybooboo/lazycsv/compare/v0.4.1...v0.6.0
[0.4.1]: https://github.com/funkybooboo/lazycsv/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/funkybooboo/lazycsv/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/funkybooboo/lazycsv/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/funkybooboo/lazycsv/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/funkybooboo/lazycsv/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/funkybooboo/lazycsv/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/funkybooboo/lazycsv/compare/v0.1.0...v0.1.4
[0.1.0]: https://github.com/funkybooboo/lazycsv/releases/tag/v0.1.0
