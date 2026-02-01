# LazyCSV Development Roadmap

A versioned checklist for building the LazyCSV TUI. Each version represents a deliverable milestone.

## Version Milestones

- **v0.1.0** - Foundation ✅ (Complete)
- **v0.2.0** - Type Safety Refactor
- **v0.3.0** - Advanced Navigation
- **v0.4.0** - Quick Editing
- **v0.5.0** - Vim Magnifier
- **v0.6.0** - Save/Quit Guards
- **v0.7.0** - Row Operations
- **v0.8.0** - Column Operations
- **v0.9.0** - Header Management
- **v1.0.0** - Undo/Redo System
- **v1.1.0** - Search & Visual
- **v1.2.0** - Sorting & Filtering
- **v1.3.0** - Multi-File Guards
- **v1.4.0** - Advanced Viewing
- **v1.5.0** - Data Analysis
- **v1.6.0** - Final Polish

---

## Guiding Principles

- **Vim-like Modal Editing:** Core navigation and commands should feel familiar to Vim users.
- **Ephemeral Edits:** No changes are saved to the file until the user explicitly commands it with `:w` or `:wq`. All cell edits update an in-memory representation first.
- **Intuitive UX:** While inspired by Vim, the UX should be clean, clear, and intuitive.
- **In-Memory Only:** All CSV files are loaded entirely into RAM for maximum performance.
- **Robust Error Handling:** Handle errors gracefully with clear user feedback.

---

## v0.1.0 - Foundation ✅

*Core viewing with vim navigation (COMPLETE)*

- ✅ Vim navigation (hjkl, arrows)
- ✅ Multi-file switching ([, ])
- ✅ Basic UI with status bar
- ✅ Help overlay (?)
- ✅ File scanning and loading
- ✅ Row/column numbering (A, B, C...)

---

## v0.1.1 - Foundation Cleanup (Patch Release) ✅

### Version 0.1.1: Code Quality & Safety Fixes ✅

*Address audit findings before v0.2.0 refactor*

#### Critical Fixes (Must Do) ✅
- [x] **Fix Clippy Error 1:** `csv_data.rs:89` - Change `rows.get(0)` to `rows.first()`
- [x] **Fix Clippy Error 2:** `csv_data.rs:95` - Change `.unwrap_or_else(Vec::new)` to `.unwrap_or_default()`
- [x] **Fix Dangerous unwrap():** `app/mod.rs:101` - Replace `cli_args.path.unwrap()` with proper error handling using `.context("No path provided")?`

#### Code Cleanup (Should Do) ✅
- [x] **Update Phase Comments:** Replace all "// Phase X" comments with version numbers
  - `csv_data.rs` lines 125, 129
  - `app/mod.rs` lines 15-17, 93
  - `app/input.rs` line 13
- [x] **Update Mode Comments:** Clean up commented future modes in `Mode` enum
- [x] **Verify Clippy Clean:** Run `cargo clippy -- -D warnings` and ensure zero errors ✅ (0 warnings)

#### Test File Fixes ✅
- [x] **Fix Test Warnings:** `file_scanner_test.rs` - Changed `len() >= 1` to `!is_empty()` (4 occurrences)
- [x] **Fix Test Warnings:** `csv_edge_cases_test.rs` - Removed empty string from `writeln!()`
- [x] **Fix Test Warnings:** `cli_test.rs` & `cli_integration_test.rs` - Removed needless borrows `&[...]` → `[...]` (16 occurrences)

#### Verification ✅
- [x] **Run Tests:** Confirm all tests pass after fixes ✅ (51 tests passing)
- [x] **Build Check:** Verify release build succeeds ✅ (clean build)
- [x] **Clippy Check:** Full check with `--all-targets --all-features` ✅ (0 warnings across all targets)

---

## v0.1.2 - Test Coverage Expansion ✅ COMPLETE

### Version 0.1.2: Comprehensive Test Suite ✅

*Fill critical test gaps before v0.2.0 refactor*

#### Critical Test Gaps (P0 - Must Have) ✅ COMPLETE
- [x] **Multi-Key Command Tests:**
  - [x] `test_multi_key_gg_goes_to_first_row()` - Test `gg` command ✅
  - [x] `test_multi_key_G_goes_to_last_row()` - Test `G` command ✅
  - [x] `test_multi_key_2G_goes_to_row_2()` - Test count + G ✅
  - [x] `test_count_prefix_2j_moves_down_2_rows()` - Count prefix ✅
  - [x] `test_count_prefix_0_goes_to_first_column()` - 0 behavior ✅
  - [x] `test_count_prefix_clears_after_use()` - State cleanup ✅

- [x] **Count Prefix Tests (COMPLETE):**
  - [x] `test_count_prefix_2j_moves_down_2_rows()` - Count prefix ✅
  - [x] `test_count_prefix_2l_moves_right_2_columns()` - Count + l ✅
  - [x] `test_count_prefix_0_goes_to_first_column()` - 0 behavior ✅
  - [x] `test_count_prefix_clears_after_use()` - State cleanup ✅

#### Important Test Gaps (P1 - Should Have) ✅ COMPLETE
- [x] **Error Handling Tests:**
  - [x] `test_error_file_not_found_shows_message()` - File not found ✅
  - [x] `test_error_malformed_csv_recovered()` - Invalid CSV ✅
  - [x] `test_scan_empty_directory_no_csvs()` - Empty dir ✅

- [x] **File Switching Edge Cases:**
  - [x] `test_file_switch_single_file_no_op()` - Only 1 file ✅
  - [x] `test_file_switch_at_first_boundary()` - First file wrap ✅
  - [x] `test_file_switch_at_last_boundary()` - Last file wrap ✅
  - [x] `test_file_switch_preserves_position()` - Position ✅

- [x] **State Consistency Tests:**
  - [x] `test_state_after_help_toggle()` - Help + navigation ✅
  - [x] `test_state_comprehensive_after_file_switch()` - File switch state ✅
  - [x] `test_dirty_flag_behavior()` - Dirty flag ✅

- [x] **Input Edge Cases:**
  - [x] `test_special_keys_ignored_in_normal_mode()` - Special keys ✅

#### Target Metrics ✅ ALL ACHIEVED
- [x] **Reach 70+ tests** - **ACHIEVED: 136 tests!** ✅
- [x] **Multi-key command coverage** - 100% ✅
- [x] **Count prefix coverage** - 100% ✅
- [x] **Error handling coverage** - Core cases covered ✅
- [x] **All tests passing** - 136/136 passing ✅
- [x] **90%+ code coverage** for input handling
- [x] **100% coverage** for multi-key commands
- [x] **All P0 tests passing** before v0.2.0

---

## v0.1.3 - Rust Idioms & Code Quality

### Version 0.1.3: Idiomatic Rust Refactoring

*Address code quality issues and make code more idiomatic before v0.2.0*

#### Critical Issues Found

**1. Test Organization (Major Issue)**
- [ ] **Move unit tests inline:** All tests are in separate `tests/` directory
  - Should use `#[cfg(test)] mod tests { }` in source files
  - Keep integration tests in `tests/`, move unit tests inline
  - Benefits: Tests are with code they test, better visibility, faster compilation
  - Files to refactor: `app/mod.rs`, `app/input.rs`, `app/navigation.rs`, `csv_data.rs`, `ui/table.rs`, `ui/status.rs`

**2. String Allocations (19 found)**
- [ ] **Reduce `.to_string()` calls:** Many could use `&'static str` or `Cow<str>`
  - `input.rs`: 8 calls for status messages
  - `mod.rs`: 3 calls for cloning
  - `csv_data.rs`: 2 calls for headers
  - `ui/*.rs`: 6 calls for UI strings
  - Replace with: `&'static str` for constants, `Cow<str>` for flexibility

**3. Clone Usage (Potential optimization)**
- [ ] **Audit clone calls:** 19 `.clone()`, `.to_string()`, `.to_owned()` calls
  - Some clones may be unnecessary with better lifetime management
  - Use references where possible (`&str` instead of `String`)
  - Pass by reference instead of cloning

**4. Function Signatures**
- [ ] **Use `&[T]` instead of `&Vec<T>`:** More flexible, accepts slices
- [ ] **Return `&str` instead of `String`:** When possible, avoid allocations
  - `column_index_to_letter()` returns `String`, could return `&'static str`

**5. Iterator Usage**
- [ ] **Replace manual index loops:** 3 found with `for _ in 0..count`
  - Use iterator methods like `.take()`, `.skip()`, `.step_by()`
  - More idiomatic and often more efficient

**6. Error Handling**
- [ ] **Use `?` operator more:** Some places manually match on Result
- [ ] **Use `.context()` from anyhow:** Better error messages
- [ ] **Consider `thiserror` for custom errors:** More structured than anyhow for library code

**7. Traits Implementation**
- [ ] **Implement `Default` trait:** For `App`, `UiState`, `CsvData`
  - Remove manual `Default::default()` calls
  - Use `#[derive(Default)]` where possible
- [ ] **Use `From`/`Into` traits:** For conversions instead of helper functions

**8. Module Organization**
- [ ] **Re-export commonly used items:** Reduce deep imports
- [ ] **Use `use crate::` consistently:** Some mix of relative and absolute paths

#### Code Smells to Fix
- [ ] **Magic numbers:** Replace hardcoded values (20, 1000, etc.) with constants
- [ ] **String literals in code:** Move to constants or config
- [ ] **Large functions:** Break up functions over 50 lines
  - `input.rs:handle_key()` - 60+ lines
  - `navigation.rs:handle_navigation()` - 80+ lines

#### Documentation
- [ ] **Add `#![warn(missing_docs)]`:** Enforce documentation
- [ ] **Document all public APIs:** Add missing doc comments
- [ ] **Add examples in doc comments:** Show usage

---

## v0.2.0 - Type Safety Refactor

### Version 0.2.0: Type System & State Refactoring

*Improve code clarity, safety, and maintainability*

#### Command & Action Abstraction
- [ ] **Introduce `UserAction` Enum:** Create comprehensive enum for all user actions (Navigate, Edit, ToggleHelp, Quit, etc.)
- [ ] **Refactor Input Handling:** Modify `app/input.rs` to parse KeyEvents into UserActions
- [ ] **Define Helper Enums:** Create `Direction`, `Location`, `FileDirection` enums

#### Newtype Wrappers for Indices
- [ ] **Introduce `RowIndex` and `ColIndex`:** Newtype wrappers around `usize` for row/column indices
- [ ] **Update Function Signatures:** Refactor all functions to use newtypes

#### State Management Refinement
- [ ] **`UiState` Refactoring:** Confirm clean separation of UI-related state
- [ ] **`InputState` Struct:** Create struct to hold `pending_key`, `pending_key_time`, `command_count`

---

## v0.3.0 - Advanced Navigation

### Version 0.3.0: Enhanced Navigation

*Vim-style navigation enhancements*

- [ ] **Row Jumping:** Implement `gg`, `G`, `<number>G` (e.g., `15G`)
  - `gg` jumps to first row
  - `G` jumps to last row
  - `15G` jumps to row 15
  - Buffer number keys before G
- [ ] **Column Jumping:** Implement `g<letter(s)>` for column navigation
  - `ga` or `gA` jumps to column A (first column)
  - `gBC` jumps to column 55 (Excel-style letters)
  - Base-26 conversion: A=1, B=2, ..., Z=26, AA=27, AB=28
  - Buffer letter keys after `g`
- [ ] **Command-line Jumps:** Implement `:<number>` and `:<column>`
  - `:15` jumps to row 15
  - `:B` jumps to column B
  - `:BC` jumps to column 55
- [ ] **Count Prefixes:** Support vim-style count prefixes for all navigation
  - `5j` moves down 5 rows
  - `3h` moves left 3 columns
- [ ] **Enter Key:** In Normal mode, `Enter` moves cursor down one row (like vim)
- [ ] **Word Motion:** Add vim-style word navigation for sparse data
  - `w` jumps to next non-empty cell in row
  - `b` jumps to previous non-empty cell in row
  - `e` jumps to last non-empty cell in row
- [ ] **Error Handling:**
  - Out-of-bounds jumps clamp to valid range
  - Invalid column letters show "Invalid column" error
  - `99999G` on 100-row file goes to last row

### Version 0.3.1: UI/UX Polish

*Polish the user interface and feedback systems*

- [ ] **Intuitive Bottom Bar:**
  - Redesign status bar with clear mode indicators
  - Prominent file status (dirty `*`, filename, read-only)
- [ ] **Transient Message System:**
  - Non-critical feedback ("File Saved", "Copied 1 row", "Invalid key")
  - Messages persist until next keypress, then clear
  - Invalid multi-key sequences show feedback (e.g., after `g`, invalid key → "Invalid column")
- [ ] **Scrolling File Viewer:**
  - Horizontal scroll for file list if wider than terminal
  - Track horizontal scroll offset
- [ ] **Clean Help Menu:**
  - Redesign `?` help overlay for clarity
  - Group keybindings logically (Navigation, Editing, Global)
  - Easy to read at a glance

---

## v0.4.0 - Quick Editing

### Version 0.4.0: Quick Edit Mode

*Fast, intuitive in-place editing of cell values*

- [ ] **Create `Insert` Mode:** New mode distinct from Magnifier
- [ ] **Vim-style Triggers:**
  - `i`: Enter Insert mode at current position
  - `a`: Enter Insert mode with cursor after current position
  - `A`: Enter Insert mode at end of cell content
  - `I`: Enter Insert mode at beginning of cell content
  - `gi`: Go to last edited cell and enter Insert mode
- [ ] **Edit Buffer State:**
  - Add `edit_buffer: String` and `cursor_position: usize` to App
  - Populate `edit_buffer` with current cell content on entry
- [ ] **Save/Cancel Flow:**
  - `Enter`: Commits change to in-memory `CsvData`, sets `is_dirty = true`, returns to Normal mode
  - `Esc`: Discards change, returns to Normal mode
- [ ] **Text Handling:**
  - Handle printable characters, `Backspace`, `Delete`
  - Arrow keys: `Left`, `Right` for cursor movement
  - `Home` (start of cell), `End` (end of cell)
  - Vim-style: `Ctrl+h` (backspace), `Ctrl+w` (delete word), `Ctrl+u` (delete to start)
- [ ] **Visual Feedback:**
  - Status bar shows `-- INSERT --`
  - Visually highlight edited cell (distinct border)
  - Render text cursor within cell at `cursor_position`
- [ ] **Scrolling:**
  - Horizontal scrolling within cell if content exceeds width
- [ ] **Error Handling:** Prevent cursor from going out of bounds

---

## v0.5.0 - Vim Magnifier

### Version 0.5.0: Cell Magnifier Mode

*Power editing with embedded vim-like editor*

- [ ] **Create `Magnifier` Mode:** New mode in the `Mode` enum
- [ ] **Trigger:** Pressing `Enter` in Normal mode opens Magnifier for current cell
  - (Note: `Enter` also moves down one row if not opening magnifier - context-aware)
- [ ] **Vim Experience:**
  - Embed full vim-like editor within modal (research `ratatui-vim` or build custom)
  - Support full vim Normal and Insert modes
  - Standard vim keys: `i`, `a`, `A`, `I`, `o`, `O`, `dd`, `yy`, `p`, `hjkl`, `w`, `b`, etc.
- [ ] **Saving (to memory):**
  - `:w` saves buffer content to in-memory `CsvData` for that cell
  - Sets `is_dirty = true`
  - Does NOT write to file
- [ ] **Exiting:**
  - `:q` closes Magnifier, discards unsaved changes
  - `:wq` or `ZZ` saves to memory, then closes
  - `:q!` forces close without saving
- [ ] **UI:**
  - Modal popup at 80% terminal width and height, centered
  - Display cell content within vim editor
  - Text wrapping and scrolling for multi-line content
  - Show mode indicator within magnifier (`-- INSERT --`, `-- NORMAL --`)
- [ ] **Navigation Between Cells:**
  - `Ctrl-h/j/k/l` moves magnifier to adjacent cell
  - If unsaved changes exist, prompt: "Save changes? (y/n/cancel)"
  - At edge cells, Ctrl-hjkl is blocked (no wrapping)
- [ ] **Error Handling:**
  - If vim editor fails to initialize, fall back to simple text area
  - Handle empty cells gracefully

---

## v0.6.0 - Save/Quit Guards

### Version 0.6.0: Persistence & Guards

*File saving and quit protection*

- [ ] **Ephemeral By Default:** All edits only update in-memory `CsvData` and set `is_dirty`
- [ ] **Command Mode Logic:**
  - `:w` - Serialize in-memory `CsvData` and overwrite original file
  - `:wq` - Write and quit
  - After successful save, clear `is_dirty` flag
- [ ] **Quitting Guards:**
  - `:q` fails if `is_dirty` is true with error: "No write since last change (add ! to override)"
  - `:q!` quits without saving, discarding all changes
- [ ] **Keybinding:** `Ctrl+S` shortcut for `:w` command
- [ ] **Error Handling:** Handle file write errors (permissions, disk full) with clear feedback

---

## v0.7.0 - Row Operations

### Version 0.7.0: Row Manipulation

*Add, delete, copy, and paste rows*

- [ ] **Add Row:**
  - `o`: Add row below current, automatically enter Insert mode for first cell of new row
  - `O`: Add row above current, automatically enter Insert mode for first cell
  - Support count prefixes: `2o` adds 2 rows
- [ ] **Delete Row:**
  - `dd`: Delete current row
  - Support count prefixes: `3dd` deletes 3 rows
- [ ] **New Row Behavior:**
  - All cells start as empty strings
  - After creating row, cursor moves to new row's first cell
  - User can press Esc to exit Insert mode without editing
- [ ] **Copy/Paste:**
  - `yy`: Copy (yank) current row
  - `5yy`: Copy 5 rows
  - `p`: Paste row below current
  - `P`: Paste row above current
- [ ] **Cursor Positioning:** After operations, cursor moves appropriately
- [ ] **Error Handling:**
  - Allow deleting last row (file can have zero data rows)
  - Handle deleting more rows than available

---

## v0.8.0 - Column Operations

### Version 0.8.0: Column Manipulation

*Vim-style column operators (more intuitive than commands)*

- [ ] **Column Operators:**
  - `dc`: Delete current column (like `dd` for rows)
  - `yc`: Yank (copy) current column (like `yy` for rows)
  - `pc`: Paste column after current (like `p` for rows)
  - `Pc`: Paste column before current (like `P` for rows)
- [ ] **Add Column (Header Context):**
  - When in header row, `o` adds column after and enters HeaderEdit mode
  - When in header row, `O` adds column before and enters HeaderEdit mode
- [ ] **New Column Behavior:**
  - All cells start as empty strings
  - After adding column with `o`/`O`, automatically enter HeaderEdit mode
- [ ] **Cursor Positioning:** After adding, cursor moves to header cell of new column
- [ ] **Error Handling:**
  - Allow deleting last column
  - No confirmation needed (undo with `u`)
  - Clear feedback on operations

---

## v0.9.0 - Header Management

### Version 0.9.0: Header Editing

*Edit column headers and header row management*

- [ ] **Edit Header Names:**
  - `gh` in Normal mode: Enter HeaderEdit mode for current column header
  - `:rename <new_name>`: Rename current column header directly
  - Create `HeaderEdit` mode distinct from cell Insert mode
  - `header_edit_buffer: String` and `header_cursor_position: usize`
  - Full text editing: printable chars, Backspace, Delete, arrows, Home, End
  - `Enter`: Commit header change, set `is_dirty = true`, return to Normal
  - `Esc`: Discard changes, return to Normal
  - Status bar shows `-- HEADER EDIT --`
  - Visually highlight header cell being edited
- [ ] **Duplicate Name Validation:**
  - Check for duplicate column names on commit
  - Show error: "Duplicate column name: <name>"
  - Keep user in HeaderEdit mode to correct
  - Allow forcing duplicate with second Enter press
- [ ] **No-Headers Mode:**
  - When loaded with `--no-headers`, create empty header strings internally
  - Track `has_user_defined_headers: bool` (starts false for `--no-headers`)
  - On first header edit, set `has_user_defined_headers = true`
  - Only write header row on save if `has_user_defined_headers` is true
- [ ] **Toggle Headers Command:**
  - `:headers` command toggles header row on/off
  - Toggle On: Promotes first data row to headers, reduces row count by 1
  - Toggle Off: Demotes headers to first data row, increases row count by 1
  - Sets `is_dirty = true` and updates flag
- [ ] **Magnifier Restriction:** Disable Magnifier mode on header cells (use `gh` instead)
- [ ] **Undo Integration:** Header edits tracked in command history

---

## v1.0.0 - Undo/Redo System

### Version 1.0.0: Command History

*Undo and redo for all mutations*

- [ ] **Create Command History Stack:** Track all mutations (edits, row/col ops, header edits, toggle)
- [ ] **Keybindings:**
  - `u`: Undo last operation
  - `Ctrl+r`: Redo
  - `.` (dot command): Repeat last edit operation
- [ ] **Status Feedback:** Show "Undo: Edit cell A5" or similar
- [ ] **History Limits:** Up to 100 operations
- [ ] **Error Handling:** Ensure undo/redo operations are robust

### Version 1.0.1: Marks System

*Jump between marked cells*

- [ ] **Set Marks:** `m[a-z]` sets mark at current cell (e.g., `ma` sets mark 'a)
- [ ] **Jump to Marks:**
  - `'[a-z]` jumps to mark (beginning of cell)
  - `` `[a-z] `` jumps to mark (exact position)
- [ ] **Navigation:**
  - `''` or `` `` `` jumps back to previous position
  - `'.` jumps to last edited cell
- [ ] **Integration:** Marks work across file switches (persist during session)

---

## v1.1.0 - Search & Visual

### Version 1.1.0: Fuzzy Search

*Find rows, columns, and cell data*

- [ ] **Keybindings:**
  - `/`: Open fuzzy finder
  - `n`: Next match
  - `N`: Previous match
  - `*`: Search for current cell value
- [ ] **Search Overlay:**
  - Centered, live results as you type
  - Shows match type: [Row], [Col], [Cell]
  - `j`/`k` to navigate results
  - `Enter` to jump, `Esc` to cancel
- [ ] **Fuzzy Matching:** Scoring-based fuzzy matching
- [ ] **Error Handling:** Handle no matches found, invalid search patterns

### Version 1.1.1: Visual Selection

*Select and operate on ranges*

- [ ] **Keybindings:**
  - `v`: Enter visual mode (cell selection)
  - `V`: Enter visual line mode (row selection)
  - `Ctrl+v`: Enter visual block mode (rectangle selection)
  - `d`: Delete selection
  - `y`: Yank (copy) selection
  - `o`: Move cursor to other end of selection
  - `Esc`: Exit visual mode
- [ ] **Visual Indicators:** Use `══` markers on selected rows
- [ ] **Visual Block:** Rectangle selection for copying/pasting cell blocks
- [ ] **Selection Logic:** Robust selection handling

---

## v1.2.0 - Sorting & Filtering

### Version 1.2.0: Sorting

*Sort data by columns*

- [ ] **Keybinding:** `s` - Sort by current column (toggle asc/desc)
- [ ] **Commands:**
  - `:sort`: Sort ascending
  - `:sort!`: Sort descending
- [ ] **Smart Sorting:** Numeric sort for numbers, text sort for strings
- [ ] **Header Indicator:** Show ↑ or ↓ in header
- [ ] **Undoable:** Sort operations tracked in undo history
- [ ] **Error Handling:** Handle sorting on mixed-type columns

### Version 1.2.1: Filtering

*Filter rows by criteria*

- [ ] **Command:** `:filter <expr>` (e.g., `:filter Age>30`, `:filter Name contains "John"`)
- [ ] **Filter Operators:**
  - `=`: Equals
  - `!=`: Not equals
  - `>`, `<`, `>=`, `<=`: Comparisons (numeric)
  - `contains`: Contains substring
  - `starts`: Starts with
  - `ends`: Ends with
- [ ] **Clear Filter:** `:nofilter` or `:nof`
- [ ] **Error Handling:** Validate filter syntax, handle invalid column names

---

## v1.3.0 - Multi-File Guards

### Version 1.3.0: File Switching with Guards

*Safe navigation between files*

- [ ] **Unsaved Changes Guard:**
  - `[` / `]` block if current file `is_dirty`
  - Show status error: "No write since last change"
- [ ] **Force Commands:** (Future) `:next!`, `:prev!` to force switch
- [ ] **Error Handling:** Handle inaccessible files during switching

---

## v1.4.0 - Advanced Viewing

### Version 1.4.0: Column Management

*Freeze columns and adjust widths*

- [ ] **Column Freezing:**
  - `:freeze` command locks current column and all to its left
  - Frozen columns remain visible during horizontal scrolling
  - Visual indicator on frozen column headers
- [ ] **Column Sizing:**
  - Manual: `Ctrl+Left/Right` to resize
  - Auto: `:autowidth` resizes current column to fit longest visible data
- [ ] **Error Handling:** Ensure resizing/freezing works with horizontal scrolling

### Version 1.4.1: Session Persistence

*Save and restore view state*

- [ ] **Save View State:**
  - On quit, save cursor position, scroll offsets, sort order, filters, frozen columns
  - Store in `~/.cache/lazycsv/`
- [ ] **Restore View State:**
  - On startup, restore to previous state if session file exists
- [ ] **Error Handling:** Handle corrupted or outdated session files gracefully

### Version 1.4.2: Theming

*Custom color themes*

- [ ] **Configuration:** Allow custom colors in `config.toml`
- [ ] **Themeable Elements:** Headers, selected cell, dirty indicator, status bar
- [ ] **Fallback:** Monochrome theme if config is invalid

---

## v1.5.0 - Data Analysis

### Version 1.5.0: Data Transformation

*Advanced data manipulation*

- [ ] **Regex Search & Replace:**
  - `:s/pattern/replacement/g` command
  - Apply to column or whole file
- [ ] **Transpose View:**
  - `:transpose` command toggles between normal and transposed view
- [ ] **Advanced Sorting:**
  - Multi-column sort: `:sort State, City`
  - Natural sorting for alphanumeric (e.g., `file1`, `file2`, `file10`)
- [ ] **Undoable:** All transformations tracked in history

### Version 1.5.1: Statistics & Plotting

*In-app data analysis*

- [ ] **Enhanced Statistics:**
  - `:stats` command shows rich popup panel
  - Numeric columns: count, mean, median, mode, stddev, text-based histogram
  - Text columns: unique count, frequency distribution
- [ ] **Terminal Plotting:**
  - `:plot` command generates text-based charts
  - Bar charts and scatter plots (using `textplots` crate)
- [ ] **Error Handling:** Handle analysis on unsuitable data types

---

## v1.6.0 - Final Polish

### Version 1.6.0: Code Quality

*Final cleanup pass*

- [ ] **Naming Consistency:** Audit all function/variable names
- [ ] **Module Organization:** Review `src/` structure
- [ ] **Linting:** Run `cargo clippy`, address all warnings
- [ ] **Dead Code Removal:** Identify and remove unused code

### Version 1.6.1: Test Suite

*Comprehensive testing*

- [ ] **Coverage Analysis:** Use `cargo-tarpaulin` to measure coverage
- [ ] **Unit Tests:** Add tests for low-coverage functions
- [ ] **Integration Tests:** Cover complex module interactions
- [ ] **Snapshot Testing:** Use `insta` crate for UI rendering tests
- [ ] **Property-Based Testing:** Use `proptest` for pure functions
- [ ] **Edge Cases:** Empty files, single row/column, invalid data

### Version 1.6.2: Documentation & Distribution

*Release preparation*

- [ ] **README:** Update with GIF demo, clear features list
- [ ] **Keybindings Reference:** Complete and accurate
- [ ] **Architecture Docs:** Update to match final code
- [ ] **crates.io:** Publish to Rust package registry
- [ ] **Package Managers:** Homebrew, AUR, etc.

---

## Future Ideas (Post v1.6.0)

*These may become future versions if prioritized*

- [ ] Network file loading (HTTP/HTTPS URLs)
- [ ] System clipboard integration
- [ ] SQL query mode (query CSV like a database)
- [ ] Export to other formats (JSON, Markdown)
- [ ] Formula evaluation (basic spreadsheet functions)
- [ ] Diff mode (compare two CSV files)
- [ ] Merge/join operations
- [ ] Pivot table support
