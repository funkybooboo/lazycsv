# LazyCSV Development Todo

A phased checklist for building the LazyCSV TUI. Check off items as they're completed.

## Phase 1: Core Viewing (MVP) 🎯

### Project Setup
- [ ] Create `Cargo.toml` with dependencies (ratatui, crossterm, csv, serde, anyhow, fuzzy-matcher)
- [ ] Set up project structure (src/ directory)
- [ ] Create `.gitignore` for Rust projects
- [ ] Add release profile optimization to Cargo.toml

### Data Layer (`src/csv_data.rs`)
- [ ] Create `CsvData` struct (headers, rows, filename, is_dirty)
- [ ] Implement `from_file()` - load CSV with csv crate
- [ ] Implement `row_count()` and `column_count()` getters
- [ ] Implement `get_cell()` - safe cell access
- [ ] Add error handling with anyhow::Context
- [ ] Test with sample CSV files (small, large, edge cases)

### Application State (`src/app.rs`)
- [ ] Create `Mode` enum (Normal, Edit, Visual, Command)
- [ ] Create `App` struct with all state fields
- [ ] Implement `new()` constructor
- [ ] Implement `handle_key()` dispatcher
- [ ] Implement `handle_normal_mode()` for navigation
- [ ] Add navigation methods:
  - [ ] `select_next_row()` / `select_previous_row()`
  - [ ] `select_next_col()` / `select_previous_col()`
  - [ ] Page up/down
  - [ ] Home/End (gg/G)
- [ ] Add horizontal scrolling (h/l keys)
- [ ] Add `show_cheatsheet` toggle (? key)
- [ ] Add quit logic (q key with unsaved check)

### UI Rendering (`src/ui.rs`)
- [ ] Create `render()` main function
- [ ] Implement `render_table()`:
  - [ ] Row numbers (left gutter)
  - [ ] Column letters (top row A, B, C...)
  - [ ] Headers row
  - [ ] Data rows from CsvData
  - [ ] Current row highlighting (► indicator)
  - [ ] Current cell border/highlight
  - [ ] Horizontal scroll offset handling
  - [ ] Truncate long text with ...
  - [ ] Table widget with proper layout
- [ ] Implement `render_status_bar()`:
  - [ ] File name (no unsaved indicator for Phase 1)
  - [ ] Row/column position (e.g., "Row 5/100 | Col 2/4")
  - [ ] Column name for current cell
  - [ ] Keybinding hints
- [ ] Implement `render_sheet_switcher()`:
  - [ ] Always visible at bottom (above status bar)
  - [ ] Show list of CSV files in directory
  - [ ] Highlight active file with ►
  - [ ] Show count: "Files (3/3)"
- [ ] Implement `render_cheatsheet()`:
  - [ ] Toggle overlay (press ?)
  - [ ] Organized panel with sections
  - [ ] Navigation keys
  - [ ] Editing keys (for future phases)
  - [ ] Other commands
  - [ ] Two-column layout
  - [ ] Centered on screen
- [ ] Define monochrome styling (no colors for MVP)
- [ ] Test rendering with different terminal sizes

### Main Entry Point (`src/main.rs`)
- [ ] Parse CLI arguments (expect CSV file path)
- [ ] Show usage message if no file provided
- [ ] Validate file exists
- [ ] Scan directory for other CSV files
- [ ] Initialize terminal with `ratatui::init()`
- [ ] Create event loop:
  - [ ] `terminal.draw()` for rendering
  - [ ] `event::poll()` with 100ms timeout
  - [ ] `event::read()` for keyboard input
  - [ ] Filter KeyPress events only
  - [ ] Call `app.handle_key()`
  - [ ] Check `app.should_quit` exit condition
- [ ] Ensure `ratatui::restore()` always called (even on errors)
- [ ] Add proper error handling with anyhow

### Testing & Polish
- [ ] Test with small CSV (10 rows, 5 columns)
- [ ] Test with large CSV (10,000+ rows)
- [ ] Test with wide CSV (50+ columns)
- [ ] Test edge cases:
  - [ ] Empty file
  - [ ] Single row (headers only)
  - [ ] Single column
  - [ ] Unicode characters
  - [ ] Malformed CSV (handle gracefully)
- [ ] Verify smooth scrolling (60 FPS)
- [ ] Verify all vim keys work (hjkl, gg, G, Ctrl+d, Ctrl+u)
- [ ] Verify row/column numbers display correctly
- [ ] Verify sheet switcher shows all files
- [ ] Verify [ and ] switch between files
- [ ] Verify cheatsheet displays correctly (press ?)
- [ ] Verify status bar updates in real-time
- [ ] Test quit functionality (q with unsaved warning)

### Documentation
- [ ] Update README.md with features and usage
- [ ] Add example CSV files to repo
- [ ] Document keybindings in README

---

## Phase 2: Cell Editing ✏️

### Edit Mode State
- [ ] Add `edit_buffer: String` to App struct
- [ ] Add `selected_col: usize` for column tracking
- [ ] Implement `enter_edit_mode()` - select all text in cell
- [ ] Implement `handle_edit_mode()` for text input
- [ ] Handle printable characters (replace buffer)
- [ ] Handle Backspace (remove last char)
- [ ] Handle Delete (remove char at cursor)
- [ ] Handle Enter - save edit
- [ ] Handle Esc - cancel edit
- [ ] Handle Ctrl+C - cancel edit

### Data Modification
- [ ] Implement `set_cell()` in CsvData
- [ ] Set `is_dirty = true` when cell modified
- [ ] Implement `save_to_file()` in CsvData - write CSV back
- [ ] Add save keybinding (Ctrl+S)
- [ ] Add `:w` command mode
- [ ] Show confirmation message: "Saved successfully"
- [ ] Handle save errors gracefully

### UI Updates
- [ ] Render edit mode indicator in cell
- [ ] Show edit buffer text in current cell
- [ ] Update status bar: mode indicator [EDIT]
- [ ] Show dirty indicator (*) next to filename when unsaved
- [ ] Update cheatsheet: show edit mode keys

### Safety & UX
- [ ] Warn on quit if unsaved changes (vim-style: q refuses, :q! forces)
- [ ] Add status message system for user feedback
- [ ] Test edit-save-reload workflow
- [ ] Test cancel edit (no changes)
- [ ] Test multiple edits before save

---

## Phase 3: Row & Column Operations 📊

### Row Operations
- [ ] Implement `add_row()` in CsvData (empty strings)
- [ ] Implement `delete_row()` in CsvData
- [ ] Add keybinding `o` - insert row below current
- [ ] Add keybinding `O` - insert row above current
- [ ] Add keybinding `dd` - delete current row (no confirmation)
- [ ] Update table_state after row operations
- [ ] Show status message: "Row added" / "Row deleted"

### Column Operations
- [ ] Implement `add_column()` in CsvData
- [ ] Implement `delete_column()` in CsvData
- [ ] Add keybinding `Ctrl+A` - insert column after current
- [ ] Add keybinding `Ctrl+Shift+A` - insert column before current
- [ ] Add keybinding `D` - delete current column (no confirmation)
- [ ] Prompt for column header name on add
- [ ] Update horizontal_offset after column operations
- [ ] Show status message: "Column added" / "Column deleted"

### Copy/Paste System
- [ ] Create clipboard struct for row data
- [ ] Add keybinding `yy` - copy current row (yank)
- [ ] Add keybinding `p` - paste row below current
- [ ] Add keybinding `P` - paste row above current
- [ ] Show visual feedback: "Row copied"
- [ ] Support multiple row copy (future: visual mode)

### Undo/Redo System
- [ ] Create command history stack
- [ ] Track all mutations as commands
- [ ] Add keybinding `u` - undo last operation
- [ ] Add keybinding `Ctrl+r` - redo
- [ ] Show status message: "Undo: [operation]"
- [ ] Limit history size (e.g., 100 operations)

### Testing
- [ ] Test add row at various positions
- [ ] Test delete row (first, middle, last)
- [ ] Test add column
- [ ] Test delete column (verify no confirmation)
- [ ] Test copy/paste workflow
- [ ] Test undo/redo
- [ ] Test operations with unsaved edits

---

## Phase 4: Advanced Features 🔍

### Fuzzy Search System
- [ ] Add `fuzzy-matcher` crate (already in dependencies)
- [ ] Add search state to App (query, matches, current_match)
- [ ] Add keybinding `/` - open fuzzy finder overlay
- [ ] Implement search query parsing:
  - [ ] Detect row numbers (e.g., "15")
  - [ ] Detect column letters (e.g., "C", "AB")
  - [ ] Detect column names (e.g., "Email")
  - [ ] Default to cell data search
- [ ] Implement fuzzy matching with scoring
- [ ] Show live results overlay as user types
- [ ] j/k to navigate results in overlay
- [ ] Enter to jump to selected match
- [ ] Esc to cancel without jumping
- [ ] Show match type: "[Row 15]", "[Col C: Email]", "[Cell A5: widget]"
- [ ] Add keybinding `n` - next match (after Enter)
- [ ] Add keybinding `N` - previous match (after Enter)
- [ ] Add keybinding `*` - search current cell value
- [ ] Show match count in status bar: "Match 3/15"

### Sorting
- [ ] Add keybinding `s` - sort by current column (in-place)
- [ ] Toggle ascending/descending on repeated press
- [ ] Implement numeric vs. text sorting
- [ ] Show sort indicator in header (↑/↓)
- [ ] Preserve row selection after sort
- [ ] Mark as dirty after sort
- [ ] Add to undo history

### Filtering
- [ ] Add filter state to App
- [ ] Add keybinding `:f` - enter filter command
- [ ] Parse filter syntax: `column=value` or `column>value`
- [ ] Implement filter matching (case-insensitive)
- [ ] Hide non-matching rows
- [ ] Show filter indicator in status bar: "Filtered: 45/100 rows"
- [ ] Add keybinding `:cf` - clear filter

### Visual Selection Mode
- [ ] Add Visual variant to Mode enum
- [ ] Add keybinding `v` - enter visual mode (cell selection)
- [ ] Add keybinding `V` - visual line mode (row selection)
- [ ] Track selection range (start_row, end_row)
- [ ] Highlight selected region
- [ ] Enable operations on selection:
  - [ ] Delete (`d`)
  - [ ] Copy (`y`)
- [ ] Update status bar: show selection size

### Statistics (Optional)
- [ ] Add stats command (`:stats`)
- [ ] Show for current column:
  - [ ] Count
  - [ ] Sum (if numeric)
  - [ ] Average (if numeric)
  - [ ] Min/Max
  - [ ] Unique values

---

## Phase 5: Multi-File/Sheet Navigation 📈

### CSV Multi-File Support
- [ ] On startup, scan directory for other .csv files
- [ ] Store list of CSV files in App state
- [ ] Update sheet switcher to show all files
- [ ] Implement [ key - load previous file
- [ ] Implement ] key - load next file
- [ ] Show loading indicator when switching
- [ ] Update status bar with current file name
- [ ] Preserve cursor position per file (optional)

### Excel File Loading
- [ ] Add `calamine` dependency to Cargo.toml
- [ ] Detect file extension (.csv vs .xlsx, .xls, .xlsm)
- [ ] Implement Excel file loading with calamine
- [ ] Extract sheet names from workbook
- [ ] Load first/active sheet into CsvData
- [ ] Handle Excel data types:
  - [ ] Numbers → formatted strings
  - [ ] Dates → ISO 8601 strings
  - [ ] Formulas → evaluated values or formula text
  - [ ] Boolean → "TRUE" / "FALSE"
  - [ ] Empty cells → ""

### Excel Multi-Sheet Support
- [ ] Load all sheet names from workbook
- [ ] Update sheet switcher to show sheets (not files) for Excel
- [ ] Change title: "Sheets" instead of "Files"
- [ ] Show count: "Sheets (2/5)"
- [ ] Implement [ / ] to switch sheets
- [ ] Show loading indicator when switching sheets
- [ ] Show sheet name in status bar

### Unified Switcher
- [ ] Create `Worksheet` enum: `Csv(PathBuf)` or `Excel(PathBuf, String)`
- [ ] Abstract switcher rendering over CSV files and Excel sheets
- [ ] Ensure consistent UX between file and sheet switching
- [ ] Test smooth transitions

### Excel Saving (Stretch Goal)
- [ ] Warn when saving Excel as CSV (data loss)
- [ ] Support "Save As CSV" conversion
- [ ] Future: Implement Excel file writing (complex)

---

## Polish & Distribution 🚀

### Configuration System
- [ ] Create config file: `~/.config/lazycsv/config.toml`
- [ ] Support custom keybindings
- [ ] Support default behavior options
- [ ] Load config on startup
- [ ] Validate config with helpful errors

### Documentation
- [ ] Comprehensive README with all features
- [ ] Add demo GIF/video
- [ ] Document all keybindings
- [ ] Add usage examples

### Testing & Quality
- [ ] Add unit tests for core functions
- [ ] Add integration tests for file I/O
- [ ] Test on Windows, macOS, Linux
- [ ] Performance benchmarks (10K, 100K rows)
- [ ] Fix clippy warnings
- [ ] Run rustfmt

### Distribution
- [ ] Create GitHub releases
- [ ] Publish to crates.io
- [ ] Create install script
- [ ] Add to package managers (brew, apt, etc.)

---

## Future Ideas 💡

- [ ] Network file loading (HTTP/HTTPS URLs)
- [ ] Clipboard integration (system clipboard)
- [ ] SQL query mode (query CSV like database)
- [ ] Export formats (JSON, Markdown table, HTML)
- [ ] Column width auto-sizing
- [ ] Cell formatting (numbers, dates, currency)
- [ ] Formula evaluation (basic spreadsheet functions)
- [ ] Plugin system for custom transformations
- [ ] Diff mode (compare two CSV files)
- [ ] Merge/join operations
- [ ] Pivot table support

---

## Notes

- This todo list is based on the approved implementation plan
- Check off items as you complete them
- Phase 1 is the MVP - focus on getting that working first
- Keep code clean and modular (~500-800 lines for Phase 1)
- Test early and often with real CSV files
- No coloring for now - monochrome design
- English only for now
- Target 60 FPS, 10K+ rows smoothly
