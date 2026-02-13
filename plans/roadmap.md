# LazyCSV Development Roadmap

A versioned checklist for building the LazyCSV TUI. Each version represents a deliverable milestone.

## Version Milestones

**Pre-1.0: Building Core Features**
- **v0.1.0** - Foundation ✅ (Complete)
- **v0.1.1** - Foundation Cleanup ✅ (Complete)
- **v0.1.2** - Test Coverage Expansion ✅ (Complete)
- **v0.1.3** - Rust Idioms & Code Quality ✅ (Complete)
- **v0.1.4** - Comprehensive Test Coverage ✅ (Complete)
- **v0.2.0** - Type Safety Refactor ✅ (Complete)
- **v0.3.0** - Advanced Navigation ✅ (Complete)
- **v0.3.1** - UI/UX Polish ✅ (Complete)
- **v0.3.2** - Pre-Edit Polish ✅ (Complete)
- **v0.4.0** - Insert Mode ✅ (Complete)
- **v0.4.1** - Persistence & Multi-File Workflow
- **v0.5.0** - Column Operations & Visual Mode
- **v0.6.0** - Vim Magnifier
- **v0.7.0** - Search
- **v0.8.0** - Undo/Redo
- **v0.9.0** - Transforms & Polish

**v1.0.0 - First Stable Release**

**Post-1.0: Future Enhancements**
- **v1.1.0** - Sorting & Filtering
- **v1.2.0** - Advanced Column Operations (resize, freeze)
- **v1.3.0** - Data Analysis & Export

---

## Guiding Principles

- **Vim-First Philosophy:** Navigation and commands should feel native to vim users. Composable commands (operator + motion). No timeouts on pending commands. Clean status line.
- **Three-Tier Operator System:** Cell (`x`) → Row (`dd`) → Column (`;dd`). Semicolon as leader for CSV-specific column operations.
- **Command Ranges:** Vim-style ranges for batch operations (`:5,10d`, `:B,D` for columns, `:B,D@5,10` for combined).
- **Ephemeral Edits:** No changes saved to file until explicit `:w` or `:wq`. All edits update in-memory representation first.
- **Minimal UI Chrome:** No heavy borders. Use subtle separators. Maximum content, minimum decoration.
- **In-Memory Only:** All CSV files loaded entirely into RAM for maximum performance.
- **CSV Only:** No Excel (.xlsx) support - CSV files only for simplicity.
- **Robust Error Handling:** Handle errors gracefully with clear, user-friendly feedback.

---

## Modal Editing Reference

LazyCSV uses vim-style modal editing with these modes:

| Mode | Indicator | Purpose | Entry | Exit |
|------|-----------|---------|-------|------|
| Normal | `-- NORMAL --` | Navigation, commands | Default / `Esc` | N/A |
| Insert | `-- INSERT --` | Quick single-cell editing | `i`, `a`, `A`, `I` | `Enter` (save), `Esc` (cancel) |
| Magnifier | `-- MAGNIFIER --` | Full vim editor for cell | `Enter` on cell | `:wq`, `:q`, `ZZ` |
| HeaderEdit | `-- HEADER EDIT --` | Edit column header names | `gh` | `Enter` (save), `Esc` (cancel) |
| Visual | `-- VISUAL --` | Select rows/cells/blocks | `v`, `V`, `;v`, `;V` | `Esc`, or after operation |
| Command | `:` prompt | Execute commands | `:` | `Enter` (execute), `Esc` (cancel) |

**Mode hierarchy:** Normal is the "home" mode. All other modes return to Normal.

---

## Command Mode Reference

### Reserved Commands (Priority)
These commands always take priority over column/row jumps:

| Command | Action |
|---------|--------|
| `:q` | Quit (checks all files for unsaved changes) |
| `:q!` | Force quit (discard all changes) |
| `:w` | Write current file |
| `:W` | Write all dirty files |
| `:wq` | Write current file and quit (checks other files) |
| `:h` `:help` | Show help |
| `:noh` | Clear search highlighting |

### Navigation Commands

| Command | Action |
|---------|--------|
| `:<number>` | Jump to row (e.g., `:15` jumps to row 15) |
| `:<letters>` | Jump to column (e.g., `:B` jumps to column B) |
| `:<letters><number>` | Jump to cell (e.g., `:A5` jumps to cell A5) |

**Examples:**
- `:5` → row 5
- `:B` → column B
- `:AA` → column AA
- `:A5` → cell A5 (column A, row 5)
- `:B12` → cell B12

**Command Detection Logic:**
1. Check reserved commands (`:q`, `:w`, `:wq`, `:W`, `:help`, `:noh`)
2. If pure number → row jump
3. If letters only → column jump
4. If letters + number → cell reference
5. Otherwise → error

**Out-of-bounds behavior:**
- `:999` on 10-row file → error: "Row 999 does not exist (max: 10)"
- `:Z` on 5-column file → error: "Column Z does not exist (max: E)"
- Never silently clamp to valid range

---

## Vim Keybinding Reference

### Motions (Navigation)

| Key | Action |
|-----|--------|
| `h` `j` `k` `l` | Move left/down/up/right |
| `Arrow keys` | Move left/down/up/right |
| `gg` | First row |
| `G` | Last row |
| `5G` | Go to row 5 |
| `0` | First column |
| `$` | Last column |
| `w` | Next non-empty cell |
| `b` | Previous non-empty cell |
| `e` | Last non-empty cell in row |
| `zt` | Scroll current row to top |
| `zz` | Scroll current row to center |
| `zb` | Scroll current row to bottom |
| `5j` | Move down 5 rows (count prefix) |
| `3h` | Move left 3 columns (count prefix) |

### Cell Editing

| Key | Action |
|-----|--------|
| `i`, `a`, `A`, `I` | Enter Insert mode at cell |
| `x` | Delete cell content |
| `s` | Substitute cell (clear + edit) |
| `Enter` | Open cell in Magnifier (full editor) |

### Row Operators

| Key | Action |
|-----|--------|
| `o` | Insert row below |
| `O` | Insert row above |
| `dd` | Delete row |
| `yy` | Yank row |
| `p` | Paste row below |
| `P` | Paste row above |
| `cc` | Clear row and enter Insert mode |
| `5dd` | Delete 5 rows |
| `5yy` | Yank 5 rows |

### Column Operators (Semicolon Leader)

**Three-tier system:** Cell (`x`) → Row (`dd`) → Column (`;dd`)

| Key | Action |
|-----|--------|
| `;o` | Insert column right (enters HeaderEdit mode) |
| `;O` | Insert column left (enters HeaderEdit mode) |
| `;dd` | Delete column |
| `;yy` | Yank column (includes header) |
| `;p` | Paste column right (cursor moves to new column) |
| `;P` | Paste column left (cursor moves to new column) |

**Note:** Semicolon leader waits silently for next key (standard vim behavior).

### Header Editing

| Key | Action |
|-----|--------|
| `gh` | Edit column header name |

### Visual Mode

| Key | Mode | Selection |
|-----|------|-----------|
| `v` | Visual | Cell-by-cell (free movement) |
| `V` | Visual Line | Whole rows |
| `;v` | Column Visual | Cell-by-cell (free movement, column intent) |
| `;V` | Column Visual Line | Whole columns |

Then in visual mode:
- `d` - delete selection (clears cells, preserves structure)
- `y` - yank selection
- `c` - change selection (clear + insert)
- `p` - paste selection (overwrites existing, adds rows/cols if needed)
- `gv` - re-select last selection

### Search

| Key | Action |
|-----|--------|
| `/pattern` | Search forward |
| `n` | Next match |
| `N` | Previous match |
| `*` | Search for current cell content |
| `:noh` | Clear search highlighting |

### Clipboard (System Only)

| Key | Action |
|-----|--------|
| `"+yy` | Yank row to system clipboard |
| `"+;yy` | Yank column to system clipboard |
| `"+p` | Paste from system clipboard |

### Undo/Redo

| Key | Action |
|-----|--------|
| `u` | Undo |
| `Ctrl+r` | Redo |
| `.` | Repeat last change |

### Cell Transforms

| Key | Action |
|-----|--------|
| `~` | Toggle case (UPPER ↔ lower) |
| `gU` | Uppercase entire cell |
| `gu` | Lowercase entire cell |
| `g~` | Title Case cell |
| `g.` | Toggle boolean (yes↔no, true↔false, 1↔0) |

### Row Movement

| Key | Action |
|-----|--------|
| `gj` | Swap current row with row below |
| `gk` | Swap current row with row above |

### File Navigation

| Key | Action |
|-----|--------|
| `[` | Previous CSV file in directory |
| `]` | Next CSV file in directory |

### Cell Reference Navigation

| Command | Action |
|---------|--------|
| `:A5` | Go to cell A5 (column A, row 5) |
| `:B12` | Go to cell B12 |
| `:AA1` | Go to cell AA1 |
| `:B` | Go to column B |
| `:5` | Go to row 5 |

### Other

| Key | Action |
|-----|--------|
| `?` | Help overlay |
| `Esc` | Cancel/return to Normal mode |

---

## CLI Options

*Foundational options implemented in early versions*

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

## v0.2.0 - Type Safety Refactor ✅

*Type safety, separation of concerns, and clean architecture (COMPLETE)*

- ✅ Type-safe position types (RowIndex/ColIndex newtypes)
- ✅ Action abstraction layer (UserAction, NavigateAction, ViewportAction)
- ✅ Separation of concerns (InputState, Session, ViewState)
- ✅ Module reorganization (domain/, input/, navigation/, session/, ui/, csv/, file_system/)
- ✅ Consistent naming (document, view_state, get_*/move_*/goto_*)
- ✅ Clean code (decomposed long functions, removed magic numbers, full docs)
- ✅ Comprehensive tests (257 passing: 229 unit + 7 CLI + 21 workflow)
- ✅ Zero warnings (cargo test ✅ | cargo clippy ✅)

---

## v0.3.0 - Advanced Navigation ✅

*Vim-style navigation enhancements (COMPLETE)*

- ✅ **Row Jumping:** `gg`, `G`, `<number>G` (e.g., `15G`)
- ✅ **Column Jumping:** `g<letter(s)>` for column navigation
- ✅ **Command-line Jumps:** `:<number>` and `:<column>`
- ✅ **Count Prefixes:** `5j` moves down 5 rows
- ✅ **Enter Key:** In Normal mode, `Enter` moves cursor down one row
- ✅ **Word Motion:** `w`, `b`, `e` for sparse data navigation
- ✅ **Error Handling:** Out-of-bounds jumps clamp to valid range

---

## v0.3.1 - UI/UX Polish ✅

*Polish the user interface and feedback systems (COMPLETE)*

- ✅ **Intuitive Bottom Bar:** Status bar with clear mode indicators
- ✅ **Transient Message System:** Non-critical feedback that clears on next keypress
- ✅ **Scrolling File Viewer:** Horizontal scroll for file list
- ✅ **Clean Help Menu:** Redesigned `?` overlay with logical groupings

---

## v0.3.2 - Pre-Edit Polish ✅

*UI redesign, bug fixes, and command mode improvements (COMPLETE)*

### UI Redesign: Vim-like Minimal Interface ✅

**New UI:**
```
 lazycsv: customers.csv                                                    1/5
─────────────────────────────────────────────────────────────────────────────
      A                B                C                D                E
  #   CustomerID       Company          Contact          Country          Phone
  1   101              Acme Corp        John Doe         USA              555-0001
> 3   103              Global Solutions Mike Johnson     UK               555-0003
  4   104              DataDrive LLC    Sarah Wilson     USA              555-0004

─────────────────────────────────────────────────────────────────────────────
customers.csv | sample.csv | test.csv                                    [1/5]
NORMAL                                                          3,C "Mike Jo..."
```

**Completed changes:**
- [x] No box borders - just horizontal rules to separate sections
- [x] Current row indicator: Single `>` in row number column
- [x] Current column: Highlighted letter in header row
- [x] Top bar: Filename left, row/total right
- [x] File list: Single line, minimal chrome
- [x] Status line: Mode + position + cell preview (like vim's `5,12 "text"`)
- [x] Pending commands visible in status line (e.g., `g` when waiting after `g`)
- [x] Auto-width columns based on content (8-50 char range)

### Bug Fixes ✅

- [x] **Bug 1: Default to current directory** - Running `lazycsv` without arguments scans "." for CSV files
- [x] **Bug 2: Notifications inline** - Status bar shows notification with position info
- [x] **Bug 3: User-friendly error messages** - Shows readable key names
- [x] **Bug 4: Jump commands fixed**
  - No timeout on pending commands (vim-like)
  - Pending command shown in status bar
  - `:c` command for column navigation
- [x] **Bug 5: Auto-width columns** - Column widths calculated from content

### Command Mode Improvements ✅

- [x] **Reserved commands take priority:** `:q`, `:w`, `:wq`, `:help` always work
- [x] **`:c` command for column jumps:**
  - `:c A` or `:c a` → column A
  - `:c 1` → column A (by number)
  - `:c AA` or `:c aa` → column AA
  - `:c 27` → column AA (by number)
- [x] **Out-of-bounds errors:** Show error instead of silently clamping

### Architecture Prep for Editing ✅

- [x] Mode enum variants: `Normal, Insert, Magnifier, HeaderEdit, Visual, Command`
- [x] `edit_buffer: Option<EditBuffer>` added to App
- [x] `EditBuffer { content, cursor, original }` defined

### Test Summary
- 344 tests passing at v0.3.2 completion
- Zero clippy warnings
- Zero compiler warnings

---

## v0.4.0 - Insert Mode ✅

*Fast, intuitive in-place editing of cell values (COMPLETE)*

### Design Philosophy
- **Essentials only**: Focus on commands that help users get work done
- **Vim-first**: Vim commands take precedence
- **Zero configuration**: Works great out of the box

### Implemented Keybindings

**Enter Insert Mode:**
| Key | Action | Status |
|-----|--------|--------|
| `i` | Edit cell (cursor at end) | ✅ |
| `a` | Edit cell (cursor at end) | ✅ |
| `I` | Edit cell (cursor at start) | ✅ |
| `A` | Edit cell (cursor at end) | ✅ |
| `s` | Replace cell (clear + edit) | ✅ |
| `F2` | Edit cell (cursor at end) | ✅ |
| `Delete` | Clear cell (stay in Normal mode) | ✅ |

**Exit Insert Mode:**
| Key | Action | Status |
|-----|--------|--------|
| `Enter` | Commit edit, move down | ✅ |
| `Shift+Enter` | Commit edit, move up | ✅ |
| `Tab` | Commit edit, move right | ✅ |
| `Shift+Tab` | Commit edit, move left | ✅ |
| `Esc` | Cancel edit, stay in place | ✅ |

**Row Operations:**
| Key | Action | Status |
|-----|--------|--------|
| `o` | Add row below, enter Insert mode | ✅ |
| `O` | Add row above, enter Insert mode | ✅ |
| `dd` | Delete row (stores in clipboard) | ✅ |
| `yy` | Yank (copy) row | ✅ |
| `p` | Paste row below | ✅ |

**Text Editing (in Insert mode):**
| Key | Action | Status |
|-----|--------|--------|
| Type characters | Insert at cursor | ✅ |
| `Backspace` | Delete character before cursor | ✅ |
| `Delete` | Delete character at cursor | ✅ |
| `Ctrl+h` | Delete character before cursor | ✅ |
| `Ctrl+w` | Delete word backward | ✅ |
| `Ctrl+u` | Delete to start of cell | ✅ |
| `Home` | Move cursor to start | ✅ |
| `End` | Move cursor to end | ✅ |
| `Left`/`Right` | Move cursor | ✅ |

### Implementation Details

- [x] `Mode::Insert` in Mode enum (was prepared in v0.3.2)
- [x] `EditBuffer { content, cursor, original }` for edit state
- [x] `last_edit_position` tracking for future `gi` command
- [x] `row_clipboard` for `yy`/`p` operations
- [x] `set_cell()`, `insert_row()`, `delete_row()` in Document
- [x] Handle all exit keys with appropriate cursor movement
- [x] Set `is_dirty = true` on commit (only if content changed)
- [x] Status bar shows `INSERT` mode indicator
- [x] Edit buffer displayed with visible cursor (`│`)
- [x] Pending `d` and `y` commands shown in status bar

### Test Summary
- 408 tests passing (271 lib + 137 integration)
- 64 comprehensive Insert mode tests
- Zero clippy warnings
- Zero compiler warnings

### Quick Edit vs Magnifier

| Scenario | Use Quick Edit (`i`) | Use Magnifier (`Enter`) |
|----------|---------------------|-------------------------|
| Fix a typo | Yes | Overkill |
| Replace entire cell | Yes | Works |
| Multi-line content | No (single line only) | Yes |
| Long text (>50 chars) | Awkward | Yes |
| Complex vim editing | No | Yes |

---

## v0.4.1 - Persistence & Multi-File Workflow

*Save files without quitting, track unsaved changes across multiple files, command ranges*

### Commands to Implement

| Command | Action |
|---------|--------|
| `:w` | Write current file |
| `:W` | Write all dirty files |
| `:wq` | Write current file and quit (checks other files for dirty) |
| `:q` | Quit (fails if any file dirty) |
| `:q!` | Force quit (discard all changes) |

### Command Ranges

**Row ranges (vim-style):**
- `:5d` - delete row 5
- `:5,10d` - delete rows 5-10
- `:5,10y` - yank rows 5-10
- `:%d` - delete all rows
- `:.d` - delete current row
- `:.,+5d` - delete current row and next 5
- `:$d` - delete last row

**Column ranges:**
- `:B,D` - operate on columns B through D
- `:B,Dd` - delete columns B through D
- `:B,Dy` - yank columns B through D

**Combined ranges (row AND column):**
- `:B,D@5,10` - operate on rows 5-10, columns B-D (rectangular region)
- `:B,D@5,10d` - delete cells in that rectangular region
- `:B,D@5,10y` - yank cells in that rectangular region

### Enhanced Command Mode

**New simplified syntax:**
- `:B` → jump to column B (replaces old `:c B`)
- `:AA` → jump to column AA
- `:A5` → jump to cell A5 (column A, row 5)
- `:5` → jump to row 5

**Command detection:**
1. Reserved commands (`:q`, `:w`, `:wq`, `:W`, `:help`, `:noh`)
2. Range patterns (`:5,10d`, `:B,D`, `:B,D@5,10`)
3. Pure number → row jump
4. Letters only → column jump
5. Letters + number → cell reference
6. Otherwise → error

**Remove old `:c` command entirely.**

### Multi-File Dirty Tracking

**File switcher visual:**
```
customers.csv* | orders.csv | products.csv*                              [1/3]
^dirty                          ^dirty
```

**Behavior:**
- Session tracks dirty files in `HashSet<PathBuf>`
- Session caches dirty `Document` instances in `HashMap<PathBuf, Document>`
- When switching files: use cache if dirty, reload from disk if clean
- After `:w`: remove from cache (reload fresh next time)
- `:q` checks ALL files for dirty state, blocks if any unsaved
- `:W` saves all cached (dirty) documents

### Implementation Steps

**File: `src/csv/writer.rs` (new file)**
- [ ] Create CSV writer module
- [ ] Implement `write_csv_atomic(document: &Document, path: &Path, delimiter: u8) -> Result<()>`
  - Write to temp file first
  - Atomically rename to target path
  - Handle CSV escaping (quotes, commas, newlines)
  - Preserves original on write failure

**File: `src/session/mod.rs`**
- [ ] Add `dirty_files: HashSet<PathBuf>` field
- [ ] Add `document_cache: HashMap<PathBuf, Document>` field
- [ ] Add `mark_dirty(&mut self, path: &Path)` method
- [ ] Add `mark_clean(&mut self, path: &Path)` method
- [ ] Add `is_dirty(&self, path: &Path) -> bool` method
- [ ] Add `cache_document(&mut self, path: PathBuf, doc: Document)` method
- [ ] Add `get_cached(&self, path: &Path) -> Option<&Document>` method
- [ ] Add `remove_from_cache(&mut self, path: &Path)` method

**File: `src/app/mod.rs`**
- [ ] Add `original_path: PathBuf` field
- [ ] Add `save(&mut self) -> Result<()>` method
  - Call `writer::write_csv_atomic`
  - Clear `is_dirty` on success
  - Sync with session dirty tracking
- [ ] Add `save_all(&mut self) -> Result<Vec<PathBuf>>` method
  - Save all dirty files from session cache

**File: `src/input/handler.rs`**
- [ ] Remove `:c` command handling entirely
- [ ] Add `:B`, `:A5` parsing (auto-detect based on pattern)
- [ ] Add `:w` command handler
  - Call `app.save()`
  - Show "Written: filename.csv (X rows)"
- [ ] Add `:W` command handler
  - Call `app.save_all()`
  - Show "Written 3 files: file1.csv, file2.csv, file3.csv"
- [ ] Modify `:q` handler
  - Check if ANY file is dirty (current + session)
  - Block with: "X files have unsaved changes (use :q! to discard)"
- [ ] Add `:q!` handler
  - Quit immediately, clear cache
- [ ] Modify `:wq` handler
  - Save current file
  - Check if other files dirty, block if so
  - Suggest `:Wq` or `:W` then `:q`

**File: `src/ui/status.rs`**
- [ ] Modify `render_file_switcher()` to show `*` for dirty files
- [ ] Add `*` suffix to filename if `session.is_dirty(path)`

### Tests to Add (`tests/persistence_test.rs`)
- [ ] `test_w_saves_current_file`
- [ ] `test_W_saves_all_dirty_files`
- [ ] `test_w_clears_dirty_flag`
- [ ] `test_dirty_indicator_in_file_switcher`
- [ ] `test_q_blocks_if_current_file_dirty`
- [ ] `test_q_blocks_if_other_files_dirty`
- [ ] `test_q_succeeds_if_all_clean`
- [ ] `test_q_bang_discards_all_changes`
- [ ] `test_wq_saves_current_checks_others`
- [ ] `test_file_switch_preserves_edits`
- [ ] `test_save_removes_from_cache`
- [ ] `test_csv_writer_escapes_quotes`
- [ ] `test_csv_writer_escapes_commas`
- [ ] `test_csv_writer_atomic_write`
- [ ] `test_B_command_jumps_to_column`
- [ ] `test_A5_command_jumps_to_cell`
- [ ] `test_old_c_command_removed`
- [ ] `test_row_range_delete`
- [ ] `test_row_range_yank`
- [ ] `test_column_range_delete`
- [ ] `test_column_range_yank`
- [ ] `test_combined_range_delete`
- [ ] `test_combined_range_yank`
- [ ] `test_percent_range_all_rows`
- [ ] `test_dollar_range_last_row`
- [ ] `test_dot_range_current_row`

### Acceptance Criteria
- [ ] `:w` saves current file to original path
- [ ] `:W` saves all dirty files
- [ ] `:q` blocks if any file (current or others) is dirty
- [ ] `:q!` quits without saving, clears cache
- [ ] `:wq` saves current, blocks if others dirty
- [ ] File switcher shows `*` next to dirty files
- [ ] Switching files preserves unsaved edits (via cache)
- [ ] After `:w`, file removed from cache
- [ ] `:B` jumps to column B (old `:c B` removed)
- [ ] `:A5` jumps to cell A5
- [ ] `:5,10d` deletes rows 5-10
- [ ] `:B,D` operates on columns B through D
- [ ] `:B,D@5,10` operates on rectangular region
- [ ] `:%d` deletes all rows
- [ ] `:.d` deletes current row
- [ ] `:$d` deletes last row
- [ ] CSV output properly escapes special characters
- [ ] Write errors display clear error messages
- [ ] All existing tests pass
- [ ] No clippy warnings

---

## v0.5.0 - Column Operations & Visual Mode

*Full column manipulation with semicolon leader, visual selections*

### Column Operations (Semicolon Leader)

| Key | Action |
|-----|--------|
| `;o` | Insert column right (enters HeaderEdit mode) |
| `;O` | Insert column left (enters HeaderEdit mode) |
| `;dd` | Delete column |
| `;yy` | Yank column (includes header) |
| `;p` | Paste column right (cursor moves to new column) |
| `;P` | Paste column left (cursor moves to new column) |

**Behavior:**
- `;yy` yanks entire column including header
- `;p` pastes column, cursor moves to new column
- `;o`/`;O` creates column with generic header (Column letter), enters HeaderEdit mode
- Semicolon leader is silent (no visual feedback, standard vim)

### Visual Mode

| Key | Mode | Selection |
|-----|------|-----------|
| `v` | Visual | Cell-by-cell (free movement) |
| `V` | Visual Line | Whole rows |
| `;v` | Column Visual | Cell-by-cell (free movement, column intent) |
| `;V` | Column Visual Line | Whole columns |

**Operations in Visual mode:**
- `d` - delete selection (clears cells, preserves structure for cell regions)
- `y` - yank selection
- `c` - change selection (clear + insert)
- `p` - paste selection (overwrites existing, adds rows/cols if needed)
- `Esc` - exit Visual mode
- `gv` - re-select last selection

**Notes:**
- `Ctrl+v` is NOT implemented (redundant with `v`)
- Delete cell region clears cells, preserves structure
- Delete whole rows/columns removes them entirely
- Paste overwrites existing cells, adds rows/cols if needed

### Count Prefixes

| Key | Action |
|-----|--------|
| `5dd` | Delete 5 rows |
| `5yy` | Yank 5 rows |
| `P` | Paste above current row |
| `cc` | Clear row and enter Insert mode |

### Implementation Steps

**File: `src/input/actions.rs`**
- [ ] Add `LeaderCommand` enum for semicolon sequences
- [ ] Track semicolon leader state in InputState

**File: `src/input/handler.rs`**
- [ ] Add semicolon (`;`) handler to enter leader mode
- [ ] Add leader command handlers: `;o`, `;O`, `;dd`, `;yy`, `;p`, `;P`
- [ ] Add count prefix support for `dd` and `yy`
- [ ] Add `V` handler to enter Visual Line mode
- [ ] Add `v` handler to enter Visual cell mode
- [ ] Add `;v` handler to enter Column Visual cell mode
- [ ] Add `;V` handler to enter Column Visual Line mode
- [ ] Add `handle_visual_mode()` function
- [ ] Add `P` handler for paste above
- [ ] Add `cc` handler
- [ ] Add `gv` handler for re-select

**File: `src/csv/document.rs`**
- [ ] Add `insert_column(&mut self, at: ColIndex, header: String)` method
- [ ] Add `delete_column(&mut self, at: ColIndex) -> Vec<String>` method
- [ ] Add `get_column(&self, col: ColIndex) -> Vec<String>` method (includes header)
- [ ] Add `delete_rows(&mut self, start: RowIndex, count: usize)` method
- [ ] Add `get_rows(&self, start: RowIndex, count: usize)` method
- [ ] Add column clipboard field

**File: `src/app/mod.rs`**
- [ ] Add `column_clipboard: Option<Vec<String>>` field
- [ ] Add `visual_anchor: Option<(RowIndex, ColIndex)>` field
- [ ] Add `last_visual_selection: Option<...>` field

**File: `src/ui/table.rs`**
- [ ] Highlight visual selections
- [ ] Different style for visual vs cursor

**File: `src/ui/status.rs`**
- [ ] Show `VISUAL`, `VISUAL LINE`, `COLUMN VISUAL`, `COLUMN VISUAL LINE` mode indicators

### Tests to Add
- [ ] `test_semicolon_leader_detection`
- [ ] `test_semicolon_o_inserts_column_right`
- [ ] `test_semicolon_O_inserts_column_left`
- [ ] `test_semicolon_dd_deletes_column`
- [ ] `test_semicolon_yy_yanks_column_with_header`
- [ ] `test_semicolon_p_pastes_column_right`
- [ ] `test_semicolon_P_pastes_column_left`
- [ ] `test_5dd_deletes_5_rows`
- [ ] `test_5yy_yanks_5_rows`
- [ ] `test_V_enters_visual_line`
- [ ] `test_v_enters_visual_cell`
- [ ] `test_semicolon_v_enters_column_visual_cell`
- [ ] `test_semicolon_V_enters_column_visual_line`
- [ ] `test_visual_d_deletes_selection`
- [ ] `test_visual_d_clears_cells_preserves_structure`
- [ ] `test_visual_y_yanks_selection`
- [ ] `test_visual_p_overwrites_and_adds_if_needed`
- [ ] `test_P_pastes_above`
- [ ] `test_cc_clears_row_enters_insert`
- [ ] `test_gv_reselects`

### Acceptance Criteria
- [ ] Semicolon leader works for column operations
- [ ] `;dd` deletes column
- [ ] `;yy` yanks column including header
- [ ] `;p` pastes column, cursor moves to new column
- [ ] `;o` inserts column, enters HeaderEdit
- [ ] `5dd` deletes exactly 5 rows
- [ ] Visual modes work (`v`, `V`, `;v`, `;V`)
- [ ] Visual cell delete clears cells, preserves structure
- [ ] Visual row/column delete removes rows/columns entirely
- [ ] Visual operations work (`d`, `y`, `c`, `p`)
- [ ] `P` pastes above
- [ ] `cc` clears row, enters Insert
- [ ] `gv` re-selects
- [ ] All existing tests pass
- [ ] No clippy warnings

---

## v0.6.0 - Vim Magnifier

*Full vim editor for complex cell editing*

### Keybindings to Implement

| Key | Action |
|-----|--------|
| `Enter` | Open Magnifier on current cell |

**In Magnifier Mode:**
- Full vim editing (multi-line, word motion, etc.)
- `:w` - Save cell content (update in-memory document)
- `:wq` or `ZZ` - Save and close Magnifier
- `:q!` - Close without saving
- `Ctrl+h/j/k/l` - Navigate to adjacent cells (prompts to save if dirty)

### Use Cases
- Editing JSON data in cells
- Multi-line descriptions or notes
- Complex text that needs vim power
- Large cell content (>100 chars)

### Implementation Steps

**File: `src/magnifier/mod.rs` (new file)**
- [ ] Create magnifier module
- [ ] Implement vim buffer state
- [ ] Implement vim mode switching (Normal/Insert within magnifier)
- [ ] Implement vim motions: `h/j/k/l`, `w/b/e`, `0/$`, `gg/G`
- [ ] Implement vim operators: `dd`, `yy`, `p`, `i/a/o/O`
- [ ] Implement line-based editing

**File: `src/app/mod.rs`**
- [ ] Add `magnifier_state: Option<MagnifierState>` field
- [ ] Implement `open_magnifier(&mut self)` method
- [ ] Implement `close_magnifier(&mut self, save: bool)` method

**File: `src/input/handler.rs`**
- [ ] Add `Enter` handler to open magnifier
- [ ] Add `handle_magnifier_mode()` function
- [ ] Handle Ctrl+h/j/k/l for cell navigation in magnifier
- [ ] Handle `:w`, `:wq`, `:q!` commands in magnifier

**File: `src/ui/magnifier.rs` (new file)**
- [ ] Render magnifier overlay (centered, 80% width/height)
- [ ] Show vim mode indicator
- [ ] Show cursor position
- [ ] Syntax highlighting for common formats (future)

### Tests to Add
- [ ] `test_enter_opens_magnifier`
- [ ] `test_magnifier_vim_motions`
- [ ] `test_magnifier_save_updates_cell`
- [ ] `test_magnifier_quit_discards`
- [ ] `test_magnifier_wq_saves_and_closes`
- [ ] `test_magnifier_ctrl_hjkl_navigates_cells`
- [ ] `test_magnifier_multiline_editing`

### Acceptance Criteria
- [ ] `Enter` opens magnifier for current cell
- [ ] Vim motions work in magnifier
- [ ] `:w` saves cell content
- [ ] `:wq` saves and closes
- [ ] `:q!` discards changes
- [ ] Ctrl+h/j/k/l navigate cells
- [ ] Multi-line editing works
- [ ] All existing tests pass
- [ ] No clippy warnings

---

## v0.7.0 - Search

*Find data in the CSV*

### Keybindings to Implement

| Key | Action |
|-----|--------|
| `/pattern` | Search forward |
| `n` | Jump to next match |
| `N` | Jump to previous match |
| `*` | Search forward for current cell content |
| `:noh` | Clear search highlighting |

### Implementation Steps

**File: `src/search/mod.rs` (new file)**
- [ ] Create search module
- [ ] Implement `find_matches(document: &Document, pattern: &str) -> Vec<(RowIndex, ColIndex)>`
- [ ] Implement `find_next_match()` with wrap-around
- [ ] Case-insensitive by default

**File: `src/app/mod.rs`**
- [ ] Add search state fields

**File: `src/input/handler.rs`**
- [ ] Add `/` handler to enter Search mode
- [ ] Add `n`, `N`, `*` handlers
- [ ] Add `:noh` command handler

**File: `src/ui/table.rs`**
- [ ] Highlight matching cells

### Acceptance Criteria
- [ ] `/` enters search mode
- [ ] `n` moves to next match with wrap
- [ ] `*` searches current cell
- [ ] `:noh` clears highlighting
- [ ] All existing tests pass

---

## v0.8.0 - Undo/Redo

*Command history for all mutations*

### Keybindings to Implement

| Key | Action |
|-----|--------|
| `u` | Undo last operation |
| `Ctrl+r` | Redo last undone operation |
| `.` | Repeat last edit (dot command) |

### Implementation Steps

**File: `src/history/mod.rs` (new file)**
- [ ] Create history module
- [ ] Define `EditCommand` enum
- [ ] Define `History` struct with undo/redo stacks
- [ ] Implement undo/redo logic

**File: `src/app/mod.rs`**
- [ ] Add `history: History` field
- [ ] Record all mutations to history

**File: `src/input/handler.rs`**
- [ ] Add `u`, `Ctrl+r`, `.` handlers

### Acceptance Criteria
- [ ] `u` undoes last operation
- [ ] `Ctrl+r` redoes
- [ ] `.` repeats last edit
- [ ] History respects max size
- [ ] All existing tests pass

---

## v0.9.0 - Transforms & Polish

*Data cleanup transformations, final polish*

### Cell Transforms

| Key | Action |
|-----|--------|
| `~` | Toggle case (UPPER ↔ lower) |
| `gU` | Uppercase entire cell |
| `gu` | Lowercase entire cell |
| `g~` | Title Case cell |
| `g.` | Toggle boolean (yes↔no, true↔false, 1↔0) |

### Row Movement

| Key | Action |
|-----|--------|
| `gj` | Swap current row with row below |
| `gk` | Swap current row with row above |

### Implementation Steps

**File: `src/transforms/mod.rs` (new file)**
- [ ] Create transforms module
- [ ] Implement case transforms
- [ ] Implement boolean toggle

**File: `src/csv/document.rs`**
- [ ] Add `swap_rows()` method

**File: `src/input/handler.rs`**
- [ ] Add transform handlers

### Acceptance Criteria
- [ ] All transforms work
- [ ] Row swapping works
- [ ] System clipboard (`"+y`, `"+p`) works
- [ ] All existing tests pass

---

## v1.0.0 - First Stable Release

*All core features working, stable command interface*

### Pre-Release Checklist

**Feature Verification:**
- [ ] All navigation features work
- [ ] All editing features work
- [ ] All column operations work
- [ ] All visual mode features work
- [ ] Search works
- [ ] Undo/redo works
- [ ] Transforms work
- [ ] Multi-file workflow works
- [ ] Save/quit protection works

**Code Quality:**
- [ ] All tests passing (target: 500+ tests)
- [ ] Zero clippy warnings
- [ ] Zero compiler warnings
- [ ] Code coverage > 80%

**Documentation:**
- [ ] README.md complete
- [ ] Keybinding reference up to date
- [ ] `--help` output accurate

**Performance:**
- [ ] Opens 100K row file in < 2 seconds
- [ ] Smooth scrolling at 60fps

### Acceptance Criteria
- [ ] All v0.4.1 - v0.8.0 features complete
- [ ] No known critical bugs
- [ ] Documentation matches implementation
- [ ] Ready for public announcement

---

## Post-1.0 Roadmap

Future enhancements for post-1.0 releases:

- **v1.1.0** - Sorting & Filtering (`:sort`, `:filter`, `:dedup`)
- **v1.2.0** - Advanced Column Operations (resize, freeze, reorder)
- **v1.3.0** - Data Analysis & Export (formulas, aggregations, export formats)
