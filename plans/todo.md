# LazyCSV Development Todo

A phased checklist for building the LazyCSV TUI. Check off items as they're completed.

## Guiding Principles
- **Vim-like Modal Editing:** Core navigation and commands should feel familiar to Vim users.
- **Ephemeral Edits:** No changes are saved to the file until the user explicitly commands it with `:w` or `:wq`. All cell edits update an in-memory representation of the data first.
- **Intuitive UX:** While inspired by Vim, the UX should be clean, clear, and intuitive, with good feedback for the user.
- **Robust Error Handling & Guards:** The application must handle errors gracefully, provide clear user feedback, and prevent accidental data loss.

---

## Phase 1: Core Viewing (MVP) 🎯

*(This phase is complete, and its items remain as a record of initial setup)*

---

## Phase 1.5: UI/UX Enhancements & Large File Support 🚀

*This phase addresses new requirements for better usability and performance with large datasets.*

### Data Layer: Smart Loading Strategy
- [ ] **Implement Hybrid Loading:** Refactor `CsvData` to support two modes of operation.
    - [ ] **In-Memory Mode:** For small files, load the entire CSV into memory for maximum performance.
    - [ ] **Lazy-Loading Mode:** For large files, use a paging/virtual scrolling mechanism that only holds a portion of the file in memory.
- [ ] **Define "Large File" Threshold:**
    - [ ] Establish a threshold to determine which mode to use. This could be based on file size (e.g., > 50MB) and/or row/column count.
    - [ ] The application will check the file against this threshold before loading.
- [ ] **Architectural Consideration:**
    - [ ] This could be implemented with an enum `DataSource { InMemory(Vec<Vec<String>>), Lazy(LazyLoader) }` or a similar trait-based approach to keep the rest of the app agnostic to the loading mode.
- [ ] Ensure all data access methods (`get_cell`, `row_count`, etc.) work seamlessly with both modes.
- [ ] **Error Handling:** Implement robust error handling for file I/O, partial reads, and data inconsistencies, especially in lazy-loading mode.

### Navigation Enhancements
- [ ] **Row Jumping:** Implement `g<number>` key sequence. For example, `g15` should jump to row 15. The app should buffer the number keys after `g` is pressed.
- [ ] **Column Jumping:** Implement `g<letter(s)>` key sequence. For example, `ga` or `gA` should jump to column A. `gBC` should jump to column 55. The app should buffer the letter keys after `g` is pressed.
- [ ] **Error Handling:** Gracefully handle invalid row/column inputs (e.g., `g99999` on a 100-row file, or `gZz`).

### UI Feature: Cell Magnifier (Power Edit Mode)
- [ ] Create a new `Magnifier` mode in the `Mode` enum.
- [ ] On `Enter` in `Normal` mode, switch to `Magnifier` mode for the current cell.
- [ ] **Vim Experience:**
    - [ ] Embed a full, self-contained `ratatui-vim` or similar `vim` component within the modal.
    - [ ] `Enter`, `Esc`, and all other keys should function as they do in standard Vim.
- [ ] **Saving (to memory):**
    - [ ] Inside the Magnifier, `:w` will save the buffer's content to the in-memory `CsvData` for that cell and set the file's `is_dirty` flag to `true`. It will NOT write to the file.
- [ ] **Exiting:**
    - [ ] `:q` will close the Magnifier and discard any unsaved changes in the vim buffer.
    - [ ] `:wq` will first save the changes to memory (same as `:w`), then close the Magnifier.
- [ ] **UI:**
    - [ ] Render a modal/popup window centered on the screen.
    - [ ] The modal should display the cell content within the embedded Vim editor.
    - [ ] Implement text wrapping within the modal.
- [ ] **Navigation:**
    - [ ] In Magnifier mode, handle `Ctrl-h/j/k/l` to move the magnifier to the adjacent cell, effectively closing and re-opening the magnifier on the new cell.
- [ ] **Error Handling:** Gracefully handle cases where the vim editor fails to initialize or content fails to be updated.

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
    - [ ] Provide feedback for invalid multi-key sequences. For example, after `g`, if an invalid key is pressed, show a message like "Invalid target for g".
    - [ ] Single invalid keys in Normal mode should be ignored silently to avoid noise.
- [ ] **General Polish:**
    - [x] Remove the `w` and `b` key handling from the codebase and `docs/keybindings.md`.

---

## Phase 2: Cell Editing (Quick Edit Mode) ✏️

*This phase focuses on fast, intuitive in-place editing of cell values.*

### Edit Mode State & Logic
- [ ] Create a new `Insert` mode in the `Mode` enum (distinct from `Magnifier`).
- [ ] **Trigger:** Pressing `i` in `Normal` mode enters `Insert` mode for the current cell.
- [ ] Add `edit_buffer: String` and `cursor_position: usize` to the App struct.
- [ ] **Save/Cancel Flow:**
    - [ ] `Enter`: Commits the change from `edit_buffer` to the in-memory `CsvData`, sets `is_dirty = true`, and returns to `Normal` mode.
    - [ ] `Esc`: Discards the change in `edit_buffer` and returns to `Normal` mode.
- [ ] **Text Handling:**
    - [ ] Handle printable characters, `Backspace`, `Delete`.
    - [ ] Handle cursor movement (`Left`, `Right`, `Home`, `End`).
- [ ] **Error Handling:** Prevent cursor from going out of bounds.

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
- [ ] Keybindings: `o`, `O`, `dd`.
- [ ] **Error Handling:** Handle edge cases like deleting the last row.

### Column Operations
- [ ] Implement `add_column()`, `delete_column()`.
- [ ] Keybindings: `Ctrl+A`, `D`.
- [ ] **Error Handling:** Handle deleting the last column.

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
