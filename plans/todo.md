# LazyCSV Development Todo

A phased checklist for building the LazyCSV TUI. Check off items as they're completed.

## Guiding Principles
- **Vim-like Modal Editing:** Core navigation and commands should feel familiar to Vim users.
- **Ephemeral Edits:** No changes are saved to the file until the user explicitly commands it with `:w` or `:wq`. All cell edits update an in-memory representation of the data first.
- **Intuitive UX:** While inspired by Vim, the UX should be clean, clear, and intuitive, with good feedback for the user.
- **Robust Error Handling & Guards:** The application must handle errors gracefully, provide clear user feedback, and prevent accidental data loss.

---

## Phase 0.5: CLI Enhancements ✨

*This phase focuses on extending the command-line interface with utility functions.*

### CLI Argument Enhancements
- [x] **Introduce `clap` for Argument Parsing:** Integrate `clap` as a dependency for robust command-line argument handling, including automatic `--help` and `--version` support.
- [x] **Define New CLI Options for TUI:**
    - [x] Add `--delimiter <CHAR>` flag to specify a custom CSV delimiter (e.g., `,`, `;`, `\t`).
    - [x] Add `--no-headers` flag to indicate that the CSV file does not have a header row.
    - [ ] Add `--encoding <ENCODING>` flag to specify the file encoding (e.g., `utf-8`, `latin1`).
    - [x] The primary positional argument will remain `<PATH>`, which can be a file or directory.
- [x] **Integrate CLI Options with CSV Loading:** Pass the parsed `--delimiter` and `--no-headers` values to the CSV loading logic to correctly interpret the file.
- [ ] **Integrate Encoding Option:** Use the `encoding_rs` crate to decode the file's contents before parsing if an encoding is specified.
- [ ] **Add Unit and Integration Tests:** Cover the new argument parsing and its effect on CSV loading.
- [x] **Code Organization Refactoring:** Encapsulated application initialization logic into `App::from_cli`, improving separation of concerns.

---

## Phase 1: Core Viewing (MVP) 🎯

*(This phase is complete, and its items remain as a record of initial setup)*

---

## Phase 1.5: UI/UX Enhancements & Large File Support 🚀

*This phase addresses new requirements for better usability and performance with large datasets.*

### Data Layer: In-Memory Strategy
- [ ] **In-Memory Only:** All CSV files are loaded entirely into RAM for maximum performance and simplicity.
- [ ] **No Lazy-Loading:** No paging or virtual scrolling. The application assumes sufficient memory for the dataset.
- [ ] **Error Handling:** Implement robust error handling for:
    - [ ] Files too large to fit in memory (show clear error message suggesting file size reduction).
    - [ ] File I/O errors during initial load.
    - [ ] CSV parsing errors (malformed rows, encoding issues).

### Navigation Enhancements
- [ ] **Core Vim Navigation:** Ensure `hjkl` movement works correctly, along with `w`/`b` removed (not applicable to CSV).
- [ ] **Enter Key:** In Normal mode, `Enter` moves cursor down one row (equivalent to `j`), matching vim behavior.
- [ ] **Row Jumping:** Implement vim-style `<number>G` sequence to jump to row. For example:
    - [ ] `gg` jumps to first row (row 1, or header if present).
    - [ ] `G` jumps to last row.
    - [ ] `15G` jumps to row 15.
    - [ ] Buffer number keys before `G`, matching vim's behavior exactly.
- [ ] **Column Jumping:** Implement `g<letter(s)>` key sequence for column navigation:
    - [ ] `ga` or `gA` jumps to column A (first column).
    - [ ] `gB` jumps to column B (second column).
    - [ ] `gAA` jumps to column 27, `gBC` jumps to column 55 (Excel-style column letters).
    - [ ] Use base-26 conversion: A=1, B=2, ..., Z=26, AA=27, AB=28, etc.
    - [ ] Buffer letter keys after `g` until a non-letter key or timeout (match vim behavior for `g` commands).
- [ ] **Count Prefixes:** Support vim-style count prefixes for all navigation:
    - [ ] `5j` moves down 5 rows, `3h` moves left 3 columns, etc.
- [ ] **Error Handling:**
    - [ ] Gracefully handle invalid row/column inputs (e.g., `99999G` on a 100-row file goes to last row).
    - [ ] Invalid column letters (e.g., `gZZZ`) should show status bar error: "Invalid column".
    - [ ] Out-of-bounds jumps should clamp to valid range (first/last row or column).

### UI Feature: Cell Magnifier (Power Edit Mode)
- [ ] Create a new `Magnifier` mode in the `Mode` enum.
- [ ] **Trigger:** Pressing `m` in `Normal` mode opens Magnifier for the current cell.
- [ ] **Vim Experience:**
    - [ ] Embed a full, self-contained vim-like editor within the modal (investigate `ratatui-vim` or build custom vim emulation).
    - [ ] Support full vim Normal and Insert modes within the magnifier.
    - [ ] All standard vim keys should function: `i`, `a`, `A`, `I`, `o`, `O`, `dd`, `yy`, `p`, `hjkl`, `w`, `b`, etc.
- [ ] **Saving (to memory):**
    - [ ] Inside the Magnifier, `:w` saves the buffer content to the in-memory `CsvData` for that cell and sets `is_dirty = true`.
    - [ ] `:w` does NOT write to the file, only updates in-memory data.
- [ ] **Exiting:**
    - [ ] `:q` closes the Magnifier and discards any unsaved changes in the vim buffer.
    - [ ] `:wq` or `ZZ` saves changes to memory, then closes the Magnifier.
    - [ ] `:q!` forces close without saving (if unsaved changes exist).
- [ ] **UI:**
    - [ ] Render a modal/popup window at 80% of terminal width and height, centered on screen.
    - [ ] Display cell content within the embedded vim editor.
    - [ ] Implement text wrapping and scrolling for multi-line content.
    - [ ] Show mode indicator within magnifier (e.g., `-- INSERT --`, `-- NORMAL --`).
- [ ] **Navigation Between Cells:**
    - [ ] `Ctrl-h/j/k/l` moves the magnifier to the adjacent cell (left/down/up/right).
    - [ ] When moving with Ctrl-hjkl:
        - [ ] If unsaved changes exist, prompt: "Save changes? (y/n/cancel)".
        - [ ] On 'y', save to memory and move. On 'n', discard and move. On cancel or ESC, stay in current cell.
    - [ ] At edge cells (first/last row or column), Ctrl-hjkl is blocked (no wrapping).
- [ ] **Error Handling:**
    - [ ] If vim editor fails to initialize, fall back to simple text area with basic editing.
    - [ ] Handle empty cells gracefully (start in Insert mode if cell is empty?).

### UI Polish & User Feedback
- [ ] **Intuitive Bottom Bar:**
    - [ ] Review and redesign the status bar and file viewer bar.
    - [ ] Ensure mode indicators are clear and follow Vim conventions (e.g., `-- NORMAL --`, `-- INSERT --`).
    - [ ] Make sure file status (dirty `*`, filename, read-only status) is prominent and easy to understand.
- [ ] **Scrolling File Viewer:**
    - [ ] The file viewer bar should scroll horizontally if the list of files is wider than the terminal.
    - [ ] Add state to track the horizontal scroll offset for the file list.
- [ ] **Clean Help Menu:**
    - [ ] Review and redesign the `?` help menu for clarity.
    - [ ] Group keybindings logically (Navigation, Editing, Global, etc.).
    - [ ] Ensure it's easy to read and understand at a glance.
- [ ] **User Feedback & Guards:**
    - [ ] Implement a transient message system in the status bar for non-critical feedback (e.g., "File Saved", "Copied 1 row", "Invalid key").
    - [ ] **Message Persistence:** Messages persist until the next keypress (like vim), then clear automatically.
    - [ ] Provide feedback for invalid multi-key sequences. For example, after `g`, if an invalid key is pressed, show a message like "Invalid column".
    - [ ] Single invalid keys in Normal mode should be ignored silently to avoid noise (vim-style).
- [ ] **General Polish:**
    - [x] Remove the `w` and `b` key handling from the codebase and `docs/keybindings.md`.

---

## Phase 1.8: Type System & State Refactoring 🧠

*This phase focuses on improving code clarity, safety, and maintainability by introducing a richer type system and refactoring state management.*

### Command & Action Abstraction
- [ ] **Introduce `UserAction` Enum:** Create a comprehensive `UserAction` enum to represent all possible actions a user can take (e.g., `Navigate(Direction)`, `GoTo(Location)`, `ToggleHelp`, `Quit`).
- [ ] **Refactor Input Handling:** Modify `app/input.rs` to act as a parser that translates raw `KeyEvent`s into `UserAction`s. The main app logic will then dispatch based on the `UserAction`.
- [ ] **Define Helper Enums:** Create smaller, focused enums like `Direction`, `Location`, and `FileDirection` to be used within `UserAction`.

### Newtype Wrappers for Indices
- [ ] **Introduce `RowIndex` and `ColIndex`:** Create newtype wrappers around `usize` (e.g., `struct RowIndex(pub usize);`) for row and column indices.
- [ ] **Update Function Signatures:** Refactor functions throughout the codebase (`get_cell`, navigation functions, etc.) to use these newtypes, preventing accidental swapping of row and column values.

### State Management Refinement
- [ ] **`UiState` Refactoring:** Confirm that all UI-related state is cleanly separated into the `UiState` struct.
- [ ] **`InputState`:** Consider creating an `InputState` struct to hold `pending_key`, `pending_key_time`, and `command_count`.

---

## Phase 2: Cell Editing (Quick Edit Mode) ✏️

*This phase focuses on fast, intuitive in-place editing of cell values.*

### Edit Mode State & Logic
- [ ] Create a new `Insert` mode in the `Mode` enum (distinct from `Magnifier`).
- [ ] **Triggers (Vim-style):**
    - [ ] `i`: Enter Insert mode with cursor at current position (start of current cell content).
    - [ ] `a`: Enter Insert mode with cursor after current position (append). For cell editing, cursor at end of content.
    - [ ] `A`: Enter Insert mode at end of cell content (append at end of line).
    - [ ] `I`: Enter Insert mode at beginning of cell content (insert at start).
- [ ] **Initial Buffer State:** When entering Insert mode, `edit_buffer` is populated with the current cell content.
- [ ] Add `edit_buffer: String` and `cursor_position: usize` to the App struct.
- [ ] **Save/Cancel Flow:**
    - [ ] `Enter`: Commits the change from `edit_buffer` to the in-memory `CsvData`, sets `is_dirty = true`, and returns to `Normal` mode.
    - [ ] `Esc`: Discards the change in `edit_buffer` and returns to `Normal` mode.
- [ ] **Text Handling:**
    - [ ] Handle printable characters, `Backspace`, `Delete`.
    - [ ] Handle cursor movement with arrow keys: `Left`, `Right`.
    - [ ] Handle `Home` (move to start of cell) and `End` (move to end of cell).
    - [ ] Consider adding Vim-style Insert mode navigation: `Ctrl+h` (backspace), `Ctrl+w` (delete word), `Ctrl+u` (delete to start).
- [ ] **Error Handling:** Prevent cursor from going out of bounds during editing.

### UI Updates for In-Place Editing
- [ ] **Visual Mode Indicator:**
    - [ ] Update status bar to show `-- INSERT --`.
    - [ ] Visually highlight the actively edited cell (e.g., distinct border).
    - [ ] Render a text cursor within the edited cell at `cursor_position`.
- [ ] **Scrolling within Cell:**
    - [ ] If the `edit_buffer` content exceeds the cell's width, implement horizontal scrolling to follow the cursor.

### Global Data Persistence
- [ ] **Ephemeral By Default:** All edits (from Magnifier or In-place) only update the in-memory `CsvData` and set the `is_dirty` flag.
- [ ] **Saving to File:**
    - [ ] Implement command mode logic for `:w` (write) and `:wq` (write & quit).
    - [ ] These commands will serialize the in-memory `CsvData` and overwrite the original file.
    - [ ] After a successful save, the `is_dirty` flag is cleared.
- [ ] **Quitting (Guard):**
    - [ ] `:q` will fail if there are unsaved changes (`is_dirty` is true). Show a status bar error: "No write since last change (add ! to override)".
    - [ ] `:q!` will quit without saving, discarding all in-memory changes.
- [ ] **Keybinding:** `Ctrl+S` should be a shortcut for the `:w` command.
- [ ] **Error Handling:** Handle file write errors (e.g., permissions, disk full) and provide clear user feedback.

---

## Phase 3: Row & Column Operations 📊

*(This phase remains as previously planned, but all operations must adhere to the ephemeral edit principle, setting the `is_dirty` flag)*

### Row Operations
- [ ] Implement `add_row()`, `delete_row()`.
- [ ] **Keybindings (Vim-style):**
    - [ ] `o`: Add row below current row, automatically enter Insert mode for first cell of new row.
    - [ ] `O`: Add row above current row, automatically enter Insert mode for first cell of new row.
    - [ ] `dd`: Delete current row.
    - [ ] Support count prefixes: `3dd` deletes 3 rows starting from current, `2o` adds 2 rows.
- [ ] **New Row Behavior:**
    - [ ] All cells in new row start as empty strings.
    - [ ] After creating row, automatically enter Insert mode for the first cell (leftmost column).
    - [ ] User can press Esc to exit Insert mode and return to Normal mode without editing.
- [ ] **Cursor Positioning:** After adding row, cursor moves to the new row's first cell.
- [ ] **Error Handling:**
    - [ ] Allow deleting the last row (file can have zero data rows, just headers).
    - [ ] Allow deleting header row if user confirms (or treat specially).
    - [ ] If deleting multiple rows with `3dd` and only 2 rows remain, delete available rows and show message.

### Column Operations
- [ ] Implement `add_column()`, `delete_column()`.
- [ ] **Commands (no keybindings to avoid vim conflicts):**
    - [ ] `:addcol`: Add column after current column.
    - [ ] `:addcol before`: Add column before current column.
    - [ ] `:addcol <N>`: Add column at position N (0-indexed or 1-indexed?).
    - [ ] `:delcol`: Delete current column.
    - [ ] `:delcol <N>`: Delete column at position N.
- [ ] **New Column Behavior:**
    - [ ] All cells in new column start as empty strings.
    - [ ] After adding column, automatically enter HeaderEdit mode (`gh`) for the new column's header.
    - [ ] User provides header name (or leaves empty) and presses Enter to continue.
    - [ ] If file has `has_user_defined_headers = false` and user names the new column, set flag to true.
- [ ] **Cursor Positioning:** After adding column, cursor moves to the header cell of new column (in HeaderEdit mode).
- [ ] **Error Handling:**
    - [ ] Allow deleting the last column (file can have zero columns - edge case).
    - [ ] Confirm before deleting column: show message "Delete column '<name>'? Press d again to confirm." (or use `:delcol!`).
    - [ ] Prevent accidental data loss with clear feedback.

### Header Operations
- [ ] **Edit Header Names:** Allow editing of column header values.
    - [ ] **Keybinding:** `gh` in Normal mode to enter header edit mode for the current column header (mnemonic: "go to header").
    - [ ] **Command:** `:rename <new_name>` to rename current column header directly.
    - [ ] **Mode State:** Create a new `HeaderEdit` mode (distinct from `Insert` mode for cells).
    - [ ] **Edit Buffer:** Similar to Insert mode, provide `header_edit_buffer: String` and `header_cursor_position: usize`.
    - [ ] **Text Editing:** Support full text editing capabilities:
        - [ ] Handle printable characters.
        - [ ] Support `Backspace` and `Delete` keys.
        - [ ] Implement cursor movement: `Left`, `Right`, `Home`, `End`.
        - [ ] Support arrow keys like Insert mode.
        - [ ] Implement horizontal scrolling if header text exceeds column width.
    - [ ] **Save/Cancel Flow:**
        - [ ] `Enter`: Commits the header change to in-memory data, sets `is_dirty = true`, returns to Normal mode.
        - [ ] `Esc`: Discards changes in edit buffer and returns to Normal mode.
    - [ ] **Visual Feedback:**
        - [ ] Update status bar to show `-- HEADER EDIT --` (distinct from `-- INSERT --`).
        - [ ] Visually highlight the header cell being edited (distinct border/style).
        - [ ] Render text cursor at `header_cursor_position`.
    - [ ] **Magnifier Restriction:** Disable Magnifier mode (`m`) on header cells. Headers are single-line only, use `gh` instead.
- [ ] **No-Headers Mode Handling:**
    - [ ] When file is loaded with `--no-headers`, internally create empty header strings for each column.
    - [ ] These empty headers are ephemeral and won't be written to the file unless the user edits at least one.
    - [ ] Track state: `has_user_defined_headers: bool` (starts as false for `--no-headers` files).
    - [ ] On first header edit via `H`, set `has_user_defined_headers = true`.
    - [ ] When saving: Only write header row if `has_user_defined_headers` is true.
- [ ] **Toggle Headers Command:**
    - [ ] Implement `:headers` command to toggle header row on/off.
    - [ ] **Toggle On:** Promotes the first data row to become headers. Reduces total row count by 1.
    - [ ] **Toggle Off:** Demotes current headers to become the first data row. Increases total row count by 1.
    - [ ] Updates `has_user_defined_headers` flag accordingly.
    - [ ] Sets `is_dirty = true` when toggled.
- [ ] **New Column Header Handling:**
    - [ ] When a new column is added via `Ctrl+A`, automatically enter `HeaderEdit` mode for that column's header.
    - [ ] User must provide a name or leave it empty, then press Enter or Esc to continue.
    - [ ] If file has `has_user_defined_headers = false` and user names the new column, set `has_user_defined_headers = true`.
- [ ] **Validation & Error Handling:**
    - [ ] **Duplicate Names:** When committing a header edit, check for duplicate column names.
        - [ ] If duplicate detected, show status bar error: "Duplicate column name: <name>".
        - [ ] Keep user in `HeaderEdit` mode to correct the name.
        - [ ] Allow forcing duplicate by pressing Enter a second time (or implement alternate flow).
    - [ ] **Empty Headers:** Allow empty and whitespace-only header names without validation errors.
    - [ ] **Cursor Bounds:** Prevent header cursor from going out of bounds during editing.
- [ ] **Undo/Redo Integration:**
    - [ ] Header edits must be tracked in the command history stack (Phase 3).
    - [ ] Header changes can be undone with `u` and redone with `Ctrl+r`.
    - [ ] Toggling headers on/off must also be undoable.

### Copy/Paste System
- [ ] Keybindings: `yy`, `p`, `P`.
- [ ] **Error Handling:** Ensure clipboard operations don't corrupt data.

### Undo/Redo System
- [ ] Create command history stack for all mutations.
- [ ] Keybindings: `u`, `Ctrl+r`.
- [ ] **Error Handling:** Ensure undo/redo operations are robust.

---

## Phase 4: Advanced Features 🔍

*(This phase remains as previously planned)*

### Fuzzy Search System
- [ ] Keybindings: `/`, `n`, `N`, `*`.
- [ ] **Error Handling:** Handle no matches found, invalid search patterns.

### Sorting
- [ ] Keybinding: `s`. Must set `is_dirty` flag.
- [ ] **Error Handling:** Handle sorting on mixed-type columns.

### Filtering
- [ ] Command: `:filter <expr>`.
- [ ] **Error Handling:** Validate filter syntax, handle invalid column names.

### Visual Selection Mode
- [ ] Keybindings: `v`, `V`.
- [ ] Operations on selection: `d`, `y`.
- [ ] **Error Handling:** Ensure selection logic is robust.

---

## Phase 5: Multi-File/Sheet Navigation 📈

### File & Sheet Support
- [ ] **Unsaved Changes Guard:**
    - [ ] When switching files/sheets (`[`, `]`), if the current file `is_dirty`, block the switch and show a status bar error: "No write since last change".
    - [ ] (Future) Add `!` variants for commands to force actions, e.g., `:next!`.
- [ ] CSV multi-file support (`[`, `]`).
- [ ] Excel file loading (`calamine`).
- [ ] Excel multi-sheet support.
- [ ] **Error Handling:** Handle inaccessible files, corrupted files, and invalid sheet names gracefully during switching.

---

## Phase 6: Advanced Viewing & Usability ✨

*This phase focuses on quality-of-life features that make viewing and interacting with data easier.*

### Column Management
- [ ] **Column Freezing/Pinning:**
    - [ ] Implement a command (`:freeze`) to lock the current column (and all to its left) on the screen.
    - [ ] Frozen columns should remain visible during horizontal scrolling.
    - [ ] Add a visual indicator to frozen column headers.
- [ ] **Column Sizing:**
    - [ ] Manual Resizing: Allow column width adjustment with keybindings (e.g., `Ctrl+Left/Right`).
    - [ ] Auto-Sizing: Add a command (`:autowidth`) to resize the current column to fit the longest visible data.
- [ ] **Error Handling:** Ensure resizing/freezing works correctly with horizontal scrolling and does not corrupt the view.

### Session & View Persistence
- [ ] **Save View State:** On quit, automatically save the view state for each file (cursor position, scroll offsets, sort order, filters, frozen columns) to a local file (e.g., in `~/.cache/lazycsv/`).
- [ ] **Restore View State:** On startup, if a session file exists for a given CSV, restore the view to its previous state.
- [ ] **Error Handling:** Handle corrupted or outdated session files gracefully.

### Theming
- [ ] **Custom Color Themes:** Allow users to define custom colors in `config.toml`.
- [ ] Users should be able to theme the header, selected row/cell, dirty indicator, status bar, etc.
- [ ] **Error Handling:** If the theme config is invalid, fall back to a default monochrome theme.

---

## Phase 7: Data Analysis & Manipulation 🛠️

*This phase introduces powerful, built-in tools for analyzing and transforming data on the fly.*

### Advanced Data Transformation
- [ ] **Regex Search & Replace:**
    - [ ] Implement a command for true regex-based search and replace within a column or across the whole file (e.g., `:s/pattern/replacement/g`).
    - [ ] This should be a powerful tool for data cleaning and adhere to the ephemeral edit model.
- [ ] **Transpose View:**
    - [ ] Add a command (`:transpose`) to toggle between normal view and a transposed view where rows become columns and vice-versa.
- [ ] **Advanced Sorting:**
    - [ ] Allow multi-column sorting (e.g., `:sort State, City`).
    - [ ] Implement "natural sorting" for alphanumeric data (e.g., `file1`, `file2`, `file10`).
- [ ] **Error Handling:** Ensure data transformations are efficient and can be undone.

### In-App Data Analysis
- [ ] **Enhanced Column Statistics:**
    - [ ] Augment the `:stats` command to show a rich popup panel.
    - [ ] For numeric columns: Show count, mean, median, mode, standard deviation, and a text-based histogram.
    - [ ] For text columns: Show unique count and a frequency distribution of the most common values.
- [ ] **Terminal-Based Plotting:**
    - [ ] Implement a `:plot` command.
    - [ ] For numeric columns, generate a simple, text-based bar chart or scatter plot in a popup window (using a crate like `textplots`).
- [ ] **Error Handling:** Handle cases where analysis or plotting is attempted on unsuitable data types.

---

## Phase 8: Code Cleanup & Naming Conventions 🧹

*This phase focuses on a final pass over the codebase to improve clarity, consistency, and organization.*

### Naming Consistency
- [ ] **Review Function and Variable Names:** Audit the entire codebase for consistent and descriptive naming.
    - [ ] Standardize test helper function names (e.g., `setup_test_app`, `create_test_data`).
    - [ ] Ensure function prefixes like `handle_`, `goto_`, `render_` are used consistently.
- [ ] **Review Module and Struct Names:** Ensure all module and struct names are clear and accurately reflect their purpose.

### File & Module Organization
- [ ] **Evaluate Module Structure:** Review the `src/` directory to see if any modules should be combined, split, or moved.
    - [ ] Consider creating a `src/io/` module for file-related operations if more are added.
    - [ ] Consider creating a `src/state/` module to house `App`, `UiState`, etc. if state management becomes more complex.
- [ ] **Review Test File Organization:** Ensure test files are logically named and their contents are focused.

### Code Quality & Readability
- [ ] **Remove Redundant Code:** Identify and remove any dead code, commented-out logic, or redundant helper functions.
- [ ] **Improve Comments:** Ensure all comments are high-level, explaining *why* something is done, not *what*. Remove any trivial comments.
- [ ] **Run `cargo clippy`:** Address all lints and warnings from `clippy` to enforce idiomatic Rust.

---

## Phase 9: Test Suite Audit & Enhancement 🧪

*This phase focuses on systematically improving the quality, coverage, and robustness of the entire test suite.*

### Coverage Analysis
- [ ] **Generate Coverage Report:** Use a code coverage tool (e.g., `cargo-tarpaulin`) to measure the current test coverage percentage for the entire codebase.
- [ ] **Identify Untested Code:** Analyze the report to find functions, modules, and code branches that have low or zero test coverage.

### Test Case Expansion
- [ ] **Add Missing Unit Tests:** Write new unit tests for functions identified as having low coverage.
- [ ] **Add Missing Integration Tests:** Write new integration tests to cover more complex interactions between modules (e.g., error handling between `App::from_cli` and `CsvData`).
- [ ] **Test Edge Cases:** Explicitly add tests for edge cases like empty files, files with only a header, files with a single row/column, and files with invalid data.

### Test Refactoring & Improvement
- [ ] **Refactor Test Helpers:** Create and standardize helper functions to reduce boilerplate code in tests (e.g., functions to create specific `App` states or `CsvData` instances).
- [ ] **Introduce UI Snapshot Testing:** Refactor UI rendering tests in `ui_rendering_test.rs` and `ui_state_test.rs` to use the `insta` crate for snapshot testing. This will provide more robust and maintainable checks for the TUI's appearance.
- [ ] **Explore Property-Based Testing:** Identify pure functions (e.g., `column_index_to_letter`) that would benefit from property-based testing with the `proptest` crate to cover a wider range of inputs automatically.

---

## Polish & Distribution 🚀

*(This phase remains as previously planned)*

- [ ] Configuration System (`config.toml`).
- [ ] Documentation (README, GIF, keybindings).
- [ ] Testing (Unit, Integration, Performance).
- [ ] Distribution (crates.io, package managers).

---

## Future Ideas 💡

*(Items from this list are promoted to official phases as they are prioritized)*

- [ ] Network file loading (HTTP/HTTPS URLs).
- [ ] System clipboard integration.
- [ ] SQL query mode (query CSV like a database).
- [ ] Export to other formats (JSON, Markdown).
- [ ] Formula evaluation (basic spreadsheet functions).
- [ ] Diff mode (compare two CSV files).
- [ ] Merge/join operations.
- [ ] Pivot table support.
