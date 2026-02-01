# LazyCSV Development Roadmap

A versioned checklist for building the LazyCSV TUI. Each version represents a deliverable milestone.

## Version Milestones

- **v0.1.0** - Foundation ✅ (Complete)
- **v0.1.1** - Foundation Cleanup ✅ (Complete)
- **v0.1.2** - Test Coverage Expansion ✅ (Complete)
- **v0.1.3** - Rust Idioms & Code Quality ✅ (Complete)
- **v0.1.4** - Comprehensive Test Coverage ✅ (Complete)
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
- **CSV Only:** No Excel (.xlsx) support - CSV files only for simplicity.
- **Robust Error Handling:** Handle errors gracefully with clear user feedback.

## CLI Options

*These foundational options are implemented in early versions*

- **`--delimiter <CHAR>`**: Specify custom CSV delimiter (`,`, `;`, `\t`, etc.) - Default: `,`
- **`--no-headers`**: Indicate file has no header row - Default: headers present
- **`--encoding <ENCODING>`**: Specify file encoding (e.g., `utf-8`, `latin1`, `iso-8859-1`)
  - Fallback: If specified encoding fails, automatically fall back to UTF-8 with warning
  - Default: UTF-8

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

## v0.2.0 - Type Safety Refactor

### Version 0.2.0: Type System & Architecture Refactoring

*Comprehensive refactoring to improve type safety, code organization, naming, and separation of concerns*

---

#### Phase 1: Type Safety Foundation ✅ COMPLETED

**1.1 Position Types (Newtype Pattern)** ✅
- [x] Create `src/domain/position.rs` module
- [x] Define `RowIndex(usize)` newtype with:
  - [x] `new()`, `get()`, `saturating_add()`, `saturating_sub()` methods
  - [x] `From<usize>` and `Into<usize>` implementations
  - [x] Proper `Debug`, `Clone`, `Copy`, `PartialEq` derives
- [x] Define `ColIndex(usize)` newtype with similar API
- [x] Define `Position { row: RowIndex, col: ColIndex }` struct
- [x] Update all `usize` row/column parameters to use `RowIndex`/`ColIndex`
- [x] Update `UiState::selected_col` from `usize` to `ColIndex`
- [x] Update `CsvData::get_cell()` to take `(RowIndex, ColIndex)`
- [x] Update `App::selected_row()` to return `Option<RowIndex>`

**1.2 Action Abstraction Layer** ✅
- [x] Create `src/input/actions.rs` module
- [x] Define `UserAction` enum with Navigate, ViewportControl, ToggleHelp, Quit, SwitchFile
- [x] Define `NavigateAction` enum (Up, Down, Left, Right, FirstRow, LastRow, etc.)
- [x] Define `ViewportAction` enum (Top, Center, Bottom, Auto)
- [x] Define `FileDirection` enum (Next, Previous)
- [x] Replace `handle_key() -> Result<bool>` with `handle_key() -> Result<InputResult>`
- [x] Define `InputResult` enum (Continue, ReloadFile, Quit)
- [x] Update input.rs to use new action types

**1.3 Remove Primitive Obsession** ✅
- [x] Replace `command_count: Option<String>` with `Option<NonZeroUsize>`
- [x] Create `PendingCommand` enum instead of `Option<KeyCode>`
- [x] Create `StatusMessage` newtype instead of `Cow<'static, str>`

**Phase 1 Results:**
- ✅ All 219 tests passing
- ✅ Zero compilation warnings
- ✅ Type safety for row/column positions prevents entire class of bugs
- ✅ Semantic action types improve code clarity
- ✅ ~600 lines of type-safe code added

---

#### Phase 2: Separation of Concerns

**2.1 Extract InputState**
- [ ] Create `src/input/state.rs`
- [ ] Define `InputState` struct:
  ```rust
  pub struct InputState {
      pending_command: Option<PendingCommand>,
      command_count: Option<NonZeroUsize>,
      pending_command_time: Option<Instant>,
  }
  ```
- [ ] Move `pending_key`, `pending_key_time`, `command_count` from `App` to `InputState`
- [ ] Add `input_state: InputState` field to `App`
- [ ] Update all references to use `app.input_state.*`

**2.2 Extract Session Management**
- [ ] Create `src/session/mod.rs` module
- [ ] Define `Session` struct:
  ```rust
  pub struct Session {
      files: Vec<PathBuf>,
      current_file_index: usize,
      config: FileConfig,
  }
  ```
- [ ] Define `FileConfig` struct for `delimiter`, `no_headers`, `encoding`
- [ ] Move file-related fields from `App` to `Session`
- [ ] Add `session: Session` field to `App`
- [ ] Implement `Session::next_file()`, `Session::prev_file()` methods

**2.3 Reorganize State in App**
- [ ] Slim down `App` struct to core responsibilities:
  ```rust
  pub struct App {
      document: CsvData,        // Renamed from csv_data
      view_state: ViewState,    // Renamed from ui
      mode: Mode,
      session: Session,
      input_state: InputState,
      status_message: Option<StatusMessage>,
  }
  ```
- [ ] Move `UiState` from `src/app/mod.rs` to `src/ui/view_state.rs`
- [ ] Rename `UiState` to `ViewState` for clarity

**2.4 Domain Layer Creation**
- [ ] Create `src/domain/` directory
- [ ] Move or create domain types:
  - [ ] `position.rs` - RowIndex, ColIndex, Position
  - [ ] `viewport.rs` - Viewport calculations and logic
  - [ ] `document.rs` - Consider extracting CSV operations from CsvData
- [ ] Keep `csv_data.rs` focused on I/O and parsing only

---

#### Phase 3: Better Naming & Consistency

**3.1 Struct/Field Naming**
- [ ] Rename `csv_data` → `document` throughout
- [ ] Rename `ui` → `view_state` throughout
- [ ] Rename `show_cheatsheet` → `help_overlay_visible`
- [ ] Rename `selected_col` → `selected_column` (match `column_count()`)
- [ ] Rename `horizontal_offset` → `column_scroll_offset`
- [ ] Rename `current_file_index` → `active_file_index`

**3.2 Function Naming**
- [ ] Rename `selected_row()` → `get_selected_row()`
- [ ] Rename `current_file()` → `get_current_file()`
- [ ] Rename `column_index_to_letter()` → `column_to_excel_letter()`
- [ ] Make navigation functions consistent: `move_*`, `goto_*`, `jump_to_*`

**3.3 Module Naming**
- [ ] Rename `file_scanner.rs` → `file_discovery.rs` (more accurate)
- [ ] Consider `app/constants.rs` → `app/messages.rs` (more specific)

**3.4 Terminology Consistency**
- [ ] Standardize on "help overlay" (not "cheatsheet" or "help")
- [ ] Standardize on "file switcher" (not "sheet switcher")
- [ ] Standardize on row index variables: always `row_idx` (never `i`, `row`, `r`)
- [ ] Standardize on column index variables: always `col_idx` (never `i`, `col`, `c`)

---

#### Phase 4: Code Organization & Structure

**4.1 Directory Restructuring**
- [ ] Create new module structure:
  ```
  src/
    app/           # Application coordinator (keep minimal)
    domain/        # Domain types (Position, Viewport, etc.)
    input/         # Input handling (actions, state, handlers)
    navigation/    # Navigation commands (extracted from app)
    session/       # Multi-file session management
    ui/            # UI rendering (table, status, help, view_state)
    file_system/   # File operations (rename from file_scanner)
    csv/           # CSV data operations (move csv_data.rs here)
  ```
- [ ] Move `app/navigation.rs` → `navigation/commands.rs`
- [ ] Move `app/input.rs` → `input/handler.rs`
- [ ] Create `input/actions.rs` for action enums
- [ ] Create `input/state.rs` for InputState

**4.2 Module Boundaries**
- [ ] Define clear public APIs for each module
- [ ] Ensure `App` only depends on public APIs, not internal details
- [ ] Move `MAX_VISIBLE_COLS` from `ui/mod.rs` to `ui/constants.rs`
- [ ] Move `MAX_CELL_WIDTH` to `ui/constants.rs`
- [ ] Create `navigation/constants.rs` for `PAGE_SIZE`

**4.3 Constants Organization**
- [ ] Create `src/config.rs` for configurable values:
  - [ ] `DEFAULT_MAX_VISIBLE_COLS = 10`
  - [ ] `DEFAULT_CELL_WIDTH = 20`
  - [ ] `DEFAULT_PAGE_SIZE = 20`
  - [ ] `MULTI_KEY_TIMEOUT_MS = 1000`
- [ ] Move all message strings to `app/messages.rs`
- [ ] Ensure all constants have clear documentation explaining their purpose

---

#### Phase 5: Clean Code Improvements

**5.1 Extract Long Functions**
- [ ] Decompose `render_table()` (172 lines):
  - [ ] Extract `calculate_visible_columns()`
  - [ ] Extract `calculate_scroll_offset()`
  - [ ] Extract `build_column_header_row()`
  - [ ] Extract `build_data_rows()`
- [ ] Decompose `handle_normal_mode()` (111 lines):
  - [ ] Extract multi-key command parsing
  - [ ] Extract count prefix handling
  - [ ] Use action dispatch pattern

**5.2 Remove Magic Numbers**
- [ ] Document or extract all magic numbers to constants
- [ ] Replace `4` (borders + headers) with named constant
- [ ] Replace `6` (status + switcher) with named constant
- [ ] Replace `5` (row number width) with named constant
- [ ] Replace `3` (truncation suffix) with named constant
- [ ] Replace `27` and `30` (cell value display) with named constants

**5.3 Improve Error Handling**
- [ ] Add context to all `anyhow::Context` calls
- [ ] Create custom error types for domain operations
- [ ] Define `CsvError`, `NavigationError`, `InputError` types
- [ ] Replace generic `Result<()>` with specific error types

**5.4 Remove Dead Code**
- [ ] Remove all commented-out "future" code (v0.4.0, etc.)
- [ ] Remove unused constants from `app/constants.rs`
- [ ] Audit all `#[allow(dead_code)]` attributes

**5.5 Improve Code Clarity**
- [ ] Add doc comments to all public types and functions
- [ ] Add module-level documentation explaining purpose
- [ ] Replace unclear variable names (`i`, `idx`, `s`) with descriptive names
- [ ] Extract complex boolean expressions to named variables

---

#### Phase 6: Testing & Validation

**6.1 Update Tests**
- [ ] Update all tests to use new type-safe APIs
- [ ] Fix tests to use `RowIndex`/`ColIndex` constructors
- [ ] Update tests to use new action-based API
- [ ] Ensure all 100+ tests still pass

**6.2 Add New Tests**
- [ ] Test `RowIndex`/`ColIndex` type safety (can't mix them)
- [ ] Test `UserAction` parsing and dispatch
- [ ] Test `InputState` multi-key command timeout
- [ ] Test `Session` file switching logic

**6.3 Integration Testing**
- [ ] Verify UI renders correctly with new types
- [ ] Verify navigation works with new action system
- [ ] Verify file switching works with Session
- [ ] Run full test suite: `cargo test`
- [ ] Run with sample files to ensure no regressions

---

#### Success Criteria

- [ ] **Zero raw `usize` for positions** in public APIs (all wrapped in RowIndex/ColIndex)
- [ ] **All user input becomes UserAction** before state changes
- [ ] **App struct has ≤ 6 fields** (document, view_state, mode, session, input_state, status_message)
- [ ] **Clear module boundaries** - each module has single responsibility
- [ ] **No magic numbers** - all explained with constants or comments
- [ ] **No function > 80 lines** (decomposed for clarity)
- [ ] **Consistent naming** - no mixed terminology
- [ ] **All tests pass** - 100+ tests validate refactoring
- [ ] **No performance regression** - still 60 FPS on 100K rows

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
  - `Ctrl-h/j/k/l` moves magnifier to adjacent cell (left/down/up/right)
  - If unsaved changes exist in current cell, show prompt: "Save changes? (y/n/cancel)"
    - Press `y`: Save to memory (`:w`), then move to adjacent cell
    - Press `n`: Discard changes, move to adjacent cell
    - Press ESC or `cancel`: Stay in current cell, do not move
  - At edge cells (first/last row or column), Ctrl-hjkl is blocked (no wrapping, no-op)
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

*Add and delete rows*

- [ ] **Add Row:**
  - `o`: Add row below current, automatically enter Insert mode for first cell of new row
  - `O`: Add row above current, automatically enter Insert mode for first cell
  - Support count prefixes: `2o` adds 2 rows (creates 2 rows, enters Insert on first)
  - Cursor moves to first cell of new row
- [ ] **Delete Row:**
  - `dd`: Delete current row
  - Support count prefixes: `3dd` deletes 3 rows starting from current
  - Cursor stays in same position (or moves up if at end)
- [ ] **New Row Behavior:**
  - All cells start as empty strings
  - After creating row, automatically enter Insert mode for first cell (leftmost column)
  - User can press Esc to exit Insert mode without editing, returns to Normal mode
- [ ] **Cursor Positioning:** After operations, cursor moves to appropriate position
- [ ] **Error Handling:**
  - Allow deleting last row (file can have zero data rows, only headers)
  - If deleting more rows than available (e.g., `5dd` with only 2 rows left), delete what's available
  - Show status message: "Deleted 2 rows" (actual count)

---

### Version 0.7.1: Copy/Paste System

*Comprehensive yank and paste operations*

- [ ] **Internal Clipboard:** App-internal clipboard for yank/paste operations
- [ ] **Yank Operations (Copy):**
  - `yy`: Yank (copy) current row
  - `5yy`: Yank 5 rows starting from current
  - `yw`: Yank current cell (word-level, single cell)
  - `yc`: Yank entire column (all rows in current column)
  - `Y`: Yank from current cell to end of row (like vim's Y)
- [ ] **Paste Operations:**
  - `p`: Paste below current row (for rows), or after current cell (for cells)
  - `P`: Paste above current row (for rows), or before current cell (for cells)
  - Pasting rows: Inserts yanked rows at cursor position
  - Pasting cells: Replaces current cell with yanked content
  - Pasting columns: Inserts yanked column after current column
- [ ] **Clipboard State:**
  - Track clipboard type (row, cell, column) to determine paste behavior
  - Show clipboard status in status bar: "Yanked 3 rows" or "Yanked cell B5"
  - Clipboard persists across operations until overwritten
- [ ] **System Clipboard Integration:**
  - `Ctrl+c`: Copy current cell to system clipboard (OS clipboard)
  - `Ctrl+v`: Paste from system clipboard into current cell
  - System clipboard operates independently from internal yank/paste
- [ ] **Error Handling:**
  - Cannot paste if clipboard is empty (show message: "Nothing in register")
  - Handle clipboard type mismatches gracefully
  - Ensure paste operations maintain data integrity

---

## v0.8.0 - Column Operations

### Version 0.8.0: Column Manipulation

*Vim-style column operators*

- [ ] **Column Operators:**
  - `dc`: Delete current column (like `dd` for rows)
  - `yc`: Yank (copy) current column (already in v0.7.1, this handles paste integration)
  - `pc`: Paste column after current (like `p` for rows)
  - `Pc`: Paste column before current (like `P` for rows)
- [ ] **Add Column Commands:**
  - `:addcol`: Add column after current column, enter HeaderEdit mode for new column
  - `:addcol before`: Add column before current column, enter HeaderEdit mode
  - `:addcol <name>`: Add column with specified name after current
- [ ] **New Column Behavior:**
  - All cells in new column start as empty strings
  - After adding column, automatically enter HeaderEdit mode (`gh`) for new column's header
  - User provides header name (or leaves empty) and presses Enter to continue
  - Cursor moves to header cell of new column in HeaderEdit mode
- [ ] **Delete Column:**
  - `dc` command deletes current column
  - Support count prefix: `3dc` deletes 3 columns starting from current
  - Sets `is_dirty = true`
- [ ] **Cursor Positioning:** After column operations, cursor moves to appropriate cell
- [ ] **Error Handling:**
  - Allow deleting last column (file can have zero columns - edge case)
  - If deleting more columns than available, delete what's available
  - Show status message: "Deleted 2 columns" or "Added column C"

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

- [ ] **Create Command History Stack:** Track all mutations (edits, row/col ops, header edits, toggle, sort, filter)
- [ ] **Keybindings:**
  - `u`: Undo last operation
  - `Ctrl+r`: Redo
  - `.` (dot command): Repeat last edit operation (vim-style)
- [ ] **Status Feedback:** Show "Undo: Edit cell A5" or similar in status bar
- [ ] **History Limits:** Unlimited undo depth (keep all changes in memory until app closes)
- [ ] **Undo Scope:** All undoable operations:
  - Cell edits (Insert mode, Magnifier)
  - Row operations (add, delete, paste)
  - Column operations (add, delete, paste)
  - Header edits
  - Toggle headers
  - Sort operations
  - Filter operations
- [ ] **Error Handling:** Ensure undo/redo operations are robust, handle undo stack at boundaries

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

### Version 1.1.0: Search

*Find cell data with literal text search*

- [ ] **Keybindings:**
  - `/`: Enter search mode (status bar input)
  - `n`: Jump to next match
  - `N`: Jump to previous match
  - `*`: Search for exact content of current cell (like vim)
  - `#`: Search backwards for current cell content (like vim)
- [ ] **Search Mode:**
  - `/` opens search input in status bar (like vim command mode)
  - Type search pattern, press Enter to find first match
  - `Esc` cancels search input
  - Search is case-insensitive by default
  - Search is literal text matching (not regex, not fuzzy)
- [ ] **Search Behavior:**
  - Highlights all matches in the visible area
  - `n` jumps cursor to next match (wraps to top)
  - `N` jumps cursor to previous match (wraps to bottom)
  - Show "Pattern not found" if no matches
  - Show match count in status bar: "Match 3 of 15"
- [ ] **Search State:**
  - Last search pattern persists (can use `n`/`N` without re-searching)
  - Clear search highlights with `:noh` command (like vim)
- [ ] **Error Handling:** Handle no matches found, empty search pattern


---

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
- [ ] **Visual Indicators:** Use highlighting or `══` markers on selected rows
- [ ] **Visual Block:** Rectangle selection for copying/pasting cell blocks
- [ ] **Selection Logic:** Robust selection handling
- [ ] **Integration:** Visual selection works with yank/delete operations from v0.7.1

---

## v1.2.0 - Sorting & Filtering

### Version 1.2.0: Sorting

*Sort data by columns*

- [ ] **Commands:**
  - `:sort`: Sort current column ascending (toggle to descending on repeat)
  - `:sort!`: Force sort descending
  - `:sort <column>`: Sort by specified column name or letter (e.g., `:sort Age` or `:sort B`)
- [ ] **Smart Sorting:** Numeric sort for numbers, text sort for strings, auto-detect type
- [ ] **Header Indicator:** Show ↑ or ↓ in header to indicate sort column and direction
- [ ] **Header Row:** Header row always stays at top, never gets sorted with data
- [ ] **Undoable:** Sort operations tracked in undo history
- [ ] **Error Handling:** Handle sorting on mixed-type columns (warn user, treat as text)

### Version 1.2.1: Filtering

*Filter rows by criteria*

- [ ] **Command:** `:filter <expr>` (e.g., `:filter Age > 30`, `:filter Name contains "John"`)
- [ ] **Column Specification:**
  - Support both column names and letters: `:filter Age > 30` or `:filter B > 30`
  - Case-insensitive column name matching
- [ ] **Filter Operators:**
  - `=`: Equals (case-insensitive for text)
  - `!=`: Not equals (case-insensitive for text)
  - `>`, `<`, `>=`, `<=`: Comparisons (numeric)
  - `contains`: Contains substring (case-insensitive)
  - `starts`: Starts with (case-insensitive)
  - `ends`: Ends with (case-insensitive)
- [ ] **Multiple Conditions:**
  - Support AND logic: `:filter Age > 30 AND City = NYC`
  - All conditions must match for row to be included
  - Syntax: `condition1 AND condition2 AND condition3 ...`
- [ ] **Filter Behavior:**
  - Filtered rows are hidden from view (not deleted)
  - Row numbers update to show only visible rows
  - Status bar shows: "Filtered: 45 of 100 rows"
  - Editing, navigation, and operations only affect visible (filtered) rows
- [ ] **Clear Filter:**
  - `:nofilter` or `:nof` command removes all filters
  - `:filter` with no arguments also clears filter
- [ ] **Error Handling:**
  - Validate filter syntax, show clear error for invalid syntax
  - Handle invalid column names: "Column 'Xyz' not found"
  - Handle type mismatches: warn if comparing text column with numeric operator

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
  - `:freeze` command locks current column and all columns to its left
  - `:freeze <column>` freezes specified column and all to its left (e.g., `:freeze C`)
  - Frozen columns remain visible at left during horizontal scrolling
  - Visual indicator on frozen column headers (e.g., lock icon or different color)
  - `:unfreeze` or `:freeze off` command removes all frozen columns
- [ ] **Column Sizing:**
  - **Manual Resizing:** `Ctrl+Left/Right` adjusts current column width
  - **Auto-sizing Commands:**
    - `:autowidth`: Resize current column to fit longest visible data in that column
    - `:autowidth <column>`: Resize specified column (e.g., `:autowidth B`)
    - `:autowidth *`: Auto-resize ALL columns to fit their content
  - Width is constrained by config.toml min/max values
- [ ] **Persistence:** Column widths and freeze state saved in session (v1.4.1)
- [ ] **Error Handling:**
  - Ensure resizing/freezing works correctly with horizontal scrolling
  - Handle edge cases: frozen columns wider than terminal, all columns frozen

### Version 1.4.1: Session Persistence

*Save and restore view state*

- [ ] **Save View State:**
  - **When:** Only on quit (`:q`, `:wq`, or normal exit)
  - **What to Save:**
    - Cursor position (row and column)
    - Scroll offsets (horizontal and vertical)
    - Sort state (column, direction)
    - Active filters
    - Frozen columns
    - Column widths (if manually adjusted)
  - **Where:** Store in `~/.cache/lazycsv/sessions/` (or `$XDG_CACHE_HOME/lazycsv/sessions/`)
- [ ] **File Identification:**
  - Use absolute file path + last modified timestamp as session key
  - Format: Hash of `"{path}:{mtime}"` as session filename
  - If file is moved or modified, session won't restore (intentional - stale state)
- [ ] **Restore View State:**
  - On startup, check if session file exists for current CSV
  - If found and file timestamp matches, restore state
  - If timestamp differs, discard session (file was modified)
  - Silently skip restoration on any error
- [ ] **Session Management:**
  - Automatically clean up old session files (>30 days unused)
  - Option to disable: `persist_session = false` in config.toml
- [ ] **Error Handling:**
  - Handle corrupted session files gracefully (discard and continue)
  - Handle outdated session format (from older lazycsv version)

### Version 1.4.2: Configuration System

*Customization via config.toml*

- [ ] **Config File Location:** `~/.config/lazycsv/config.toml` (or `$XDG_CONFIG_HOME/lazycsv/config.toml`)
- [ ] **Default Delimiter:**
  - `default_delimiter = ","` - Set default CSV delimiter
  - Overridden by `--delimiter` CLI flag
- [ ] **Color Theme:**
  - Customize colors for UI elements
  - **Themeable Elements:**
    - Header row background/foreground
    - Selected cell highlight
    - Dirty indicator (`*`)
    - Status bar background/foreground
    - Normal text, cursor
  - **Color Format:** RGB hex (`#RRGGBB`) or named colors (`"red"`, `"blue"`, etc.)
  - **Fallback:** If theme config is invalid, fall back to default monochrome theme
- [ ] **Key Bindings:**
  - Allow remapping keys to custom bindings
  - Example: Remap `hjkl` to arrow keys, or `dd` to different key
  - Validation: Prevent conflicting bindings
- [ ] **Column Width Defaults:**
  - `default_column_width = 20` - Default width for columns
  - `min_column_width = 5` - Minimum allowed width
  - `max_column_width = 100` - Maximum allowed width
- [ ] **Error Handling:**
  - Validate config on load
  - Show clear errors for invalid config entries
  - Fall back to defaults for invalid values

---

## v1.5.0 - Data Analysis

### Version 1.5.0: Data Transformation

*Advanced data manipulation*

- [ ] **Regex Search & Replace:**
  - **Syntax:** `:s/pattern/replacement/flags scope`
  - **Scope (required):**
    - `column <name>`: Apply to all rows in specified column (e.g., `:s/foo/bar/g column Age`)
    - `column <letter>`: Apply to column by letter (e.g., `:s/foo/bar/g column B`)
    - `all`: Apply to entire file, all cells (e.g., `:s/foo/bar/g all`)
  - **Flags:**
    - `g`: Global replace all matches in each cell (default: replace first match only)
    - `i`: Case-insensitive matching (default: case-sensitive)
  - **Examples:**
    - `:s/john/Jane/gi column Name` - Replace all 'john' (case-insensitive) with 'Jane' in Name column
    - `:s/^\s+//g all` - Remove leading whitespace from all cells
  - Sets `is_dirty = true`, undoable
- [ ] **Transpose View:**
  - `:transpose` command toggles between normal and transposed view
  - Rows become columns, columns become rows
  - Transposed view is **fully editable** - all edit operations work normally
  - Cell edits, row/column operations, all apply in transposed space
  - Toggling back to normal view preserves all edits
  - Visual indicator in status bar: "-- TRANSPOSED --"
  - Undoable (transpose itself is an undoable operation)
- [ ] **Advanced Sorting:**
  - **Multi-column sort:** `:sort <col1>, <col2>` (e.g., `:sort State, City`)
    - Sorts by first column, then by second column for ties, etc.
  - **Natural sorting:** Auto-detect and use natural sort for alphanumeric data
    - Example: `file1`, `file2`, `file10` instead of `file1`, `file10`, `file2`
  - Undoable
- [ ] **Error Handling:**
  - Validate regex patterns, show clear error for invalid regex
  - Handle invalid column names in scope
  - Ensure all transformations can be undone

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
- [ ] SQL query mode (query CSV like a database)
- [ ] Export to other formats (JSON, Markdown, Excel)
- [ ] Formula evaluation (basic spreadsheet functions)
- [ ] Diff mode (compare two CSV files)
- [ ] Merge/join operations
- [ ] Pivot table support
- [ ] **Statistics & Plotting:**
  - `:stats` command with rich popup panel
  - Numeric columns: count, mean, median, mode, stddev, histogram
  - Text columns: unique count, frequency distribution
  - `:plot` command for text-based bar/scatter plots
- [ ] Excel file support (.xlsx, .xls)
