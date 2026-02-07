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
- **v0.5.0** - Selection & Advanced Operators (V, counts, cc)
- **v0.6.0** - Search (/, n, N, *, #)
- **v0.7.0** - Persistence (:w, :q, :wq)
- **v0.8.0** - Undo/Redo (history stack, . dot command)
- **v0.9.0** - Power Features (fill, date/time, Excel shortcuts)

**v1.0.0 - First Stable Release**

**Post-1.0: Enhancements**
- **v1.1.0** - Marks & Registers
- **v1.2.0** - Text Objects (ic, ir, ac, ar)
- **v1.3.0** - Sorting & Filtering
- **v1.4.0** - Column Operations (resize, freeze)
- **v1.5.0** - Advanced Features (tab completion, macros)
- **v1.6.0** - Data Analysis & Export

---

## Guiding Principles

- **Vim-First Philosophy:** Navigation and commands should feel native to vim users. Composable commands (operator + motion). No timeouts on pending commands. Clean status line.
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
| Visual | `-- VISUAL --` | Select rows/cells/blocks | `v`, `V`, `Ctrl+v` | `Esc`, or after operation |
| Command | `:` prompt | Execute commands | `:` | `Enter` (execute), `Esc` (cancel) |

**Mode hierarchy:** Normal is the "home" mode. All other modes return to Normal.

---

## Command Mode Reference

### Reserved Commands (Priority)
These commands always take priority over column/row jumps:

| Command | Action |
|---------|--------|
| `:q` | Quit |
| `:q!` | Force quit (discard changes) |
| `:w` | Save (write to file) |
| `:w!` | Force save |
| `:wq` | Save and quit |
| `:wq!` | Force save and quit |
| `:x` | Save and quit (alias for `:wq`) |
| `:h` `:help` | Show help |
| `:noh` | Clear search highlighting |

### Navigation Commands

| Command | Action |
|---------|--------|
| `:<number>` | Jump to row (e.g., `:15` jumps to row 15) |
| `:c <arg>` | Jump to column |

**`:c` command accepts:**
- Number: `:c 1` → column A, `:c 27` → column AA
- Uppercase: `:c A` → column A, `:c Q` → column Q
- Lowercase: `:c a` → column A (case-insensitive)
- Multi-letter: `:c AA`, `:c bc` → column AA, BC

**Out-of-bounds behavior:**
- `:999` on 10-row file → error: "Row 999 does not exist (max: 10)"
- `:c Z` on 5-column file → error: "Column Z does not exist (max: E)"
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

### Operators (Editing)

| Key | Action |
|-----|--------|
| `d` | Delete |
| `y` | Yank (copy) |
| `c` | Change (delete + insert) |
| `p` | Paste below/after |
| `P` | Paste above/before |

**Operator examples:**
| Command | Action |
|---------|--------|
| `dd` | Delete current row |
| `5dd` | Delete 5 rows |
| `yy` | Yank current row |
| `5yy` | Yank 5 rows |
| `cc` | Change current row |
| `dw` | Delete to next non-empty cell |
| `d$` | Delete to end of row |
| `y$` | Yank to end of row |

### Visual Mode

| Key | Mode | Selection |
|-----|------|-----------|
| `v` | Visual | Cell-by-cell |
| `V` | Visual Line | Whole rows |
| `Ctrl+v` | Visual Block | Rectangle of cells |

### Search

| Key | Action |
|-----|--------|
| `/pattern` | Search forward |
| `?pattern` | Search backward |
| `n` | Next match |
| `N` | Previous match |
| `*` | Search for current cell content |
| `#` | Search backward for current cell |

### Registers

| Key | Action |
|-----|--------|
| `"ayy` | Yank row to register 'a' |
| `"ap` | Paste from register 'a' |
| `"+y` | Yank to system clipboard |
| `"+p` | Paste from system clipboard |

### Marks

| Key | Action |
|-----|--------|
| `ma` | Set mark 'a' at current position |
| `'a` | Jump to mark 'a' |
| `` `a `` | Jump to exact position of mark 'a' |
| `''` | Jump to previous position |

### Undo/Redo

| Key | Action |
|-----|--------|
| `u` | Undo |
| `Ctrl+r` | Redo |
| `.` | Repeat last change |

### CSV-Specific Text Objects

| Object | Meaning |
|--------|---------|
| `ic` | Inner cell (content without delimiters) |
| `ac` | A cell (including context) |
| `ir` | Inner row |
| `ar` | A row |

**Examples:** `dic` = delete inner cell, `yir` = yank inner row, `cic` = change inner cell

### File Navigation

| Key | Action |
|-----|--------|
| `[` | Previous CSV file in directory |
| `]` | Next CSV file in directory |
| `:e <file>` | Open specific file |
| `:files` | List all CSV files |

### Cell Reference Navigation

| Command | Action |
|---------|--------|
| `:A5` | Go to cell A5 (column A, row 5) |
| `:B12` | Go to cell B12 |
| `:AA1` | Go to cell AA1 |

### Quick Cell Transforms

| Key | Action |
|-----|--------|
| `~` | Toggle case (UPPER ↔ lower) |
| `gU` | Uppercase entire cell |
| `gu` | Lowercase entire cell |
| `g~` | Title Case cell |
| `g.` | Toggle boolean (yes↔no, true↔false, 1↔0) |

### Smart Column Navigation

| Key | Action |
|-----|--------|
| `{` | Jump to previous empty cell in column |
| `}` | Jump to next empty cell in column |
| `[[` | Jump to previous different value in column |
| `]]` | Jump to next different value in column |
| `gf` | Go to first non-empty cell in column |
| `gl` | Go to last non-empty cell in column |

### Row/Column Movement

| Key | Action |
|-----|--------|
| `gj` | Swap current row with row below |
| `gk` | Swap current row with row above |
| `g<` | Move current column left |
| `g>` | Move current column right |

### Selection Helpers

| Key | Action |
|-----|--------|
| `g*` | Select all cells with same value as current |
| `gv` | Re-select last selection |
| `gn` | Select next search match |

### Column Operations

| Key | Action |
|-----|--------|
| `yc` | Yank entire column |
| `dc` | Delete entire column |
| `gc` | Duplicate current column |

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
  - `:c` command replaces `g<letter>` for column navigation
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
- **Vim-first**: Vim commands take precedence, Excel enhances where useful
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
- 64 comprehensive Insert mode tests covering:
  - Enter/exit Insert mode (i, a, I, A, s, F2)
  - Text editing (typing, backspace, delete, cursor movement)
  - Vim-style editing (Ctrl+h, Ctrl+w, Ctrl+u)
  - Commit behavior (Enter, Tab, Shift+Enter, Shift+Tab)
  - Cancel behavior (Esc)
  - Row operations (o, O, dd, yy, p, Delete)
  - Unicode/emoji character support
  - Boundary conditions (first/last row/column)
  - Edge cases (empty cells, clipboard state)
- Zero clippy warnings
- Zero compiler warnings

### Quick Edit vs Magnifier

| Scenario | Use Quick Edit (`i`) | Use Magnifier (`Enter`) |
|----------|---------------------|-------------------------|
| Fix a typo | Yes | Overkill |
| Replace entire cell | Yes | Works |
| Multi-line content | No (single line only) | Yes (future) |
| Long text (>50 chars) | Awkward | Yes (future) |
| Complex vim editing | No | Yes (future) |

---

## v0.5.0 - Selection & Advanced Operators

*Extend row operations with counts and visual selection*

**Note:** Basic `dd`, `yy`, `p` are already implemented in v0.4.0. This version adds count prefixes and visual mode.

### Keybindings to Implement

| Key | Action |
|-----|--------|
| `5dd` | Delete 5 rows |
| `5yy` | Yank 5 rows |
| `V` | Enter Visual Line mode (select row) |
| `j`/`k` | Extend selection in Visual mode |
| `d` | Delete selected rows (Visual mode) |
| `y` | Yank selected rows (Visual mode) |
| `Esc` | Exit Visual mode |
| `P` | Paste above current row |
| `cc` | Clear row and enter Insert mode |
| `gv` | Re-select last selection |

### Implementation Steps

**File: `src/app/mod.rs`**
- [ ] Add `visual_anchor: Option<RowIndex>` to `ViewState` for selection start point
- [ ] Add `last_visual_selection: Option<(RowIndex, RowIndex)>` for `gv` command
- [ ] Ensure `Mode::Visual` variant exists (added in v0.3.2)

**File: `src/input/handler.rs`**
- [ ] Add count prefix support for `d` pending command
  - Modify `handle_multi_key_command` to check `app.input_state.command_count`
  - `dd` with count: delete `count` rows starting from current
- [ ] Add count prefix support for `y` pending command
  - `yy` with count: yank `count` rows starting from current
- [ ] Add `V` handler to enter Visual Line mode
  - Set `app.mode = Mode::Visual`
  - Set `visual_anchor` to current row
- [ ] Add `handle_visual_mode()` function:
  - `j`/`k`: Move cursor, selection extends from anchor to cursor
  - `d`: Delete all rows in selection, exit to Normal
  - `y`: Yank all rows in selection, exit to Normal
  - `Esc`: Exit to Normal, clear selection
- [ ] Add `P` handler for paste above
  - Insert row at current position (before, not after)
- [ ] Add `cc` handler
  - Clear all cells in current row
  - Enter Insert mode at first column
- [ ] Add `gv` handler
  - Restore last visual selection
  - Enter Visual mode with saved anchor/cursor

**File: `src/csv/document.rs`**
- [ ] Add `delete_rows(&mut self, start: RowIndex, count: usize)` method
- [ ] Add `get_rows(&self, start: RowIndex, count: usize) -> Vec<Vec<String>>` method
- [ ] Add `insert_rows(&mut self, at: RowIndex, rows: Vec<Vec<String>>)` method

**File: `src/ui/table.rs`**
- [ ] Highlight all rows in visual selection (not just current row)
- [ ] Use different highlight style for visual selection vs cursor

**File: `src/ui/status.rs`**
- [ ] Show `VISUAL LINE` when `app.mode == Mode::Visual`
- [ ] Update row count messages: "Deleted 3 rows" / "Yanked 5 rows"

### Tests to Add (`tests/visual_mode_test.rs`)
- [ ] `test_5dd_deletes_5_rows`
- [ ] `test_5yy_yanks_5_rows`
- [ ] `test_V_enters_visual_mode`
- [ ] `test_visual_mode_j_extends_selection_down`
- [ ] `test_visual_mode_k_extends_selection_up`
- [ ] `test_visual_mode_d_deletes_selection`
- [ ] `test_visual_mode_y_yanks_selection`
- [ ] `test_visual_mode_esc_exits`
- [ ] `test_P_pastes_above_current_row`
- [ ] `test_cc_clears_row_enters_insert`
- [ ] `test_gv_reselects_last_selection`
- [ ] `test_count_prefix_at_end_of_file` (edge case)

### Acceptance Criteria
- [ ] `5dd` deletes exactly 5 rows
- [ ] `5yy` yanks exactly 5 rows to clipboard
- [ ] `V` enters Visual Line mode, status shows `VISUAL LINE`
- [ ] Moving with `j`/`k` in Visual mode highlights multiple rows
- [ ] `d` in Visual mode deletes all highlighted rows
- [ ] `y` in Visual mode yanks all highlighted rows
- [ ] `P` pastes row above (not below) current row
- [ ] `cc` clears row content and enters Insert mode
- [ ] `gv` restores previous visual selection
- [ ] All 383+ existing tests still pass
- [ ] No clippy warnings

---

## v0.6.0 - Search

*Find data in the CSV*

### Keybindings to Implement

| Key | Action |
|-----|--------|
| `/` | Enter search mode (forward) |
| `?` | Enter search mode (backward) |
| `n` | Jump to next match |
| `N` | Jump to previous match |
| `*` | Search forward for current cell content |
| `#` | Search backward for current cell content |
| `:noh` | Clear search highlighting |
| `Esc` | Cancel search input |

### Implementation Steps

**File: `src/app/mod.rs`**
- [ ] Add `Mode::Search` variant to Mode enum
- [ ] Add search state fields to App:
  ```rust
  pub search_pattern: Option<String>,
  pub search_direction: SearchDirection, // Forward/Backward
  pub search_matches: Vec<(RowIndex, ColIndex)>,
  pub current_match_index: Option<usize>,
  pub search_buffer: String, // Current search input
  ```
- [ ] Add `SearchDirection` enum: `Forward`, `Backward`

**File: `src/input/handler.rs`**
- [ ] Add `/` handler to enter Search mode (forward)
  - Set `app.mode = Mode::Search`
  - Set `app.search_direction = Forward`
  - Clear `app.search_buffer`
- [ ] Add `?` handler to enter Search mode (backward)
- [ ] Add `handle_search_mode()` function:
  - Character input: append to `search_buffer`
  - Backspace: remove last character
  - Enter: execute search, jump to first match, return to Normal
  - Esc: cancel search, return to Normal
- [ ] Add `n` handler (Normal mode): jump to next match
- [ ] Add `N` handler (Normal mode): jump to previous match
- [ ] Add `*` handler: search for current cell content forward
- [ ] Add `#` handler: search for current cell content backward
- [ ] Add `:noh` command handler to clear highlighting

**File: `src/search/mod.rs` (new file)**
- [ ] Create search module
- [ ] Implement `find_matches(document: &Document, pattern: &str) -> Vec<(RowIndex, ColIndex)>`
  - Case-insensitive by default
  - Search all cells in document
  - Return list of (row, col) positions
- [ ] Implement `find_next_match(matches: &[...], current: (RowIndex, ColIndex), direction: SearchDirection) -> Option<usize>`
- [ ] Implement wrap-around logic at file boundaries

**File: `src/ui/table.rs`**
- [ ] Highlight cells that match search pattern
  - Use distinct style (e.g., yellow background)
- [ ] Highlight current match differently (e.g., orange background)
- [ ] Only highlight matches in visible area for performance

**File: `src/ui/status.rs`**
- [ ] Show search prompt when in Search mode: `/pattern_`
- [ ] Show match count after search: "Match 3 of 15"
- [ ] Show "Pattern not found" when no matches
- [ ] Show `?pattern_` for backward search

**File: `src/lib.rs`**
- [ ] Add `mod search;` to module list

### Tests to Add (`tests/search_test.rs`)
- [ ] `test_slash_enters_search_mode`
- [ ] `test_search_finds_exact_match`
- [ ] `test_search_case_insensitive`
- [ ] `test_search_no_match_shows_message`
- [ ] `test_n_jumps_to_next_match`
- [ ] `test_N_jumps_to_previous_match`
- [ ] `test_search_wraps_at_end_of_file`
- [ ] `test_search_wraps_at_beginning_of_file`
- [ ] `test_star_searches_current_cell`
- [ ] `test_hash_searches_backward`
- [ ] `test_noh_clears_highlighting`
- [ ] `test_esc_cancels_search`
- [ ] `test_search_empty_pattern_does_nothing`
- [ ] `test_search_highlights_visible_matches`

### Acceptance Criteria
- [ ] `/` enters search mode, shows `/` prompt in status bar
- [ ] Typing shows pattern in status bar
- [ ] Enter executes search, jumps to first match
- [ ] `n` moves to next match, wraps at end
- [ ] `N` moves to previous match, wraps at start
- [ ] `*` searches for current cell content
- [ ] `#` searches backward for current cell content
- [ ] Matching cells are highlighted in visible area
- [ ] Status shows "Match X of Y" after search
- [ ] "Pattern not found" shown when no matches
- [ ] `:noh` clears search highlighting
- [ ] All existing tests pass
- [ ] No clippy warnings

---

## v0.7.0 - Persistence

*File saving and quit protection*

### Commands to Implement

| Command | Action |
|---------|--------|
| `:w` | Write (save) to file |
| `:w!` | Force write (overwrite read-only) |
| `:wq` | Write and quit |
| `:wq!` | Force write and quit |
| `:x` | Write (if modified) and quit |
| `:q` | Quit (fails if dirty) |
| `:q!` | Force quit (discard changes) |
| `Ctrl+S` | Save (alias for `:w`) |
| `ZZ` | Write and quit (alias for `:wq`) |
| `ZQ` | Quit without saving (alias for `:q!`) |

### Implementation Steps

**File: `src/csv/writer.rs` (new file)**
- [ ] Create CSV writer module
- [ ] Implement `write_csv(document: &Document, path: &Path) -> Result<()>`
  - Write headers as first row
  - Write all data rows
  - Handle CSV escaping (quotes, commas, newlines)
  - Use same delimiter as input file
- [ ] Implement `write_csv_atomic(document: &Document, path: &Path) -> Result<()>`
  - Write to temp file first
  - Rename to target path (atomic on most filesystems)
  - Preserves original on write failure

**File: `src/csv/mod.rs`**
- [ ] Add `pub mod writer;`
- [ ] Re-export writer functions

**File: `src/app/mod.rs`**
- [ ] Add `original_path: PathBuf` field to track file path
- [ ] Add `save(&mut self) -> Result<()>` method
  - Call `writer::write_csv_atomic`
  - Clear `is_dirty` on success
  - Return error with context on failure
- [ ] Add `can_quit(&self) -> bool` method
  - Returns `!self.document.is_dirty`

**File: `src/input/handler.rs`**
- [ ] Add `:w` command handler
  - Call `app.save()`
  - Show "Written: filename.csv" on success
  - Show error message on failure
- [ ] Add `:w!` command handler
  - Same as `:w` but with force flag (for future read-only handling)
- [ ] Add `:wq` command handler
  - Call `app.save()`, then quit if successful
- [ ] Add `:x` command handler
  - Only write if `is_dirty`, then quit
- [ ] Modify `:q` handler
  - Check `app.can_quit()`
  - If dirty: show "No write since last change (add ! to override)"
  - If clean: quit
- [ ] Add `:q!` handler
  - Quit immediately, discard changes
- [ ] Add `Ctrl+S` handler in Normal mode
  - Same as `:w`
- [ ] Add `ZZ` handler (two-key command)
  - Add `PendingCommand::Z` handling for `ZZ`
  - Same as `:wq`
- [ ] Add `ZQ` handler
  - Same as `:q!`

**File: `src/input/actions.rs`**
- [ ] Add `PendingCommand::Z` variant (for ZZ, ZQ sequences)

**File: `src/ui/status.rs`**
- [ ] Show write success: "Written: filename.csv (X rows)"
- [ ] Show write error: "Error: Could not write to file: <reason>"
- [ ] Show quit guard message when dirty

### Tests to Add (`tests/persistence_test.rs`)
- [ ] `test_w_saves_file`
- [ ] `test_w_clears_dirty_flag`
- [ ] `test_wq_saves_and_quits`
- [ ] `test_x_saves_only_if_dirty`
- [ ] `test_x_quits_without_save_if_clean`
- [ ] `test_q_fails_if_dirty`
- [ ] `test_q_succeeds_if_clean`
- [ ] `test_q_bang_quits_even_if_dirty`
- [ ] `test_ctrl_s_saves`
- [ ] `test_ZZ_saves_and_quits`
- [ ] `test_ZQ_quits_without_saving`
- [ ] `test_csv_writer_escapes_quotes`
- [ ] `test_csv_writer_escapes_commas`
- [ ] `test_csv_writer_escapes_newlines`
- [ ] `test_csv_writer_preserves_delimiter`
- [ ] `test_save_error_shows_message`

### Acceptance Criteria
- [ ] `:w` saves file to original path
- [ ] `:w` clears `is_dirty` flag
- [ ] `:w` shows success message with filename
- [ ] `:wq` saves and quits in one command
- [ ] `:x` only writes if file was modified
- [ ] `:q` shows error if file is dirty
- [ ] `:q!` quits without saving
- [ ] `Ctrl+S` saves file
- [ ] `ZZ` saves and quits
- [ ] `ZQ` quits without saving
- [ ] CSV output properly escapes special characters
- [ ] Write errors display clear error messages
- [ ] All existing tests pass
- [ ] No clippy warnings

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
- [ ] Define `EditCommand` enum for all undoable operations:
  ```rust
  pub enum EditCommand {
      SetCell { row: RowIndex, col: ColIndex, old: String, new: String },
      InsertRow { at: RowIndex, content: Vec<String> },
      DeleteRow { at: RowIndex, content: Vec<String> },
      InsertRows { at: RowIndex, rows: Vec<Vec<String>> },
      DeleteRows { at: RowIndex, rows: Vec<Vec<String>> },
      SetHeader { col: ColIndex, old: String, new: String },
      // Future: InsertColumn, DeleteColumn, Sort, Filter
  }
  ```
- [ ] Implement `EditCommand::undo(&self, document: &mut Document)`
- [ ] Implement `EditCommand::redo(&self, document: &mut Document)`
- [ ] Define `History` struct:
  ```rust
  pub struct History {
      undo_stack: Vec<EditCommand>,
      redo_stack: Vec<EditCommand>,
      last_edit: Option<EditCommand>, // For dot command
      max_size: usize, // Limit memory usage
  }
  ```
- [ ] Implement `History::push(cmd: EditCommand)` - add to undo stack, clear redo
- [ ] Implement `History::undo() -> Option<EditCommand>` - pop from undo, push to redo
- [ ] Implement `History::redo() -> Option<EditCommand>` - pop from redo, push to undo
- [ ] Implement `History::last_repeatable() -> Option<&EditCommand>` - for dot command

**File: `src/app/mod.rs`**
- [ ] Add `history: History` field to App
- [ ] Modify all mutation methods to record commands:
  - `set_cell` → push `SetCell` command
  - `insert_row` → push `InsertRow` command
  - `delete_row` → push `DeleteRow` command
  - `insert_rows` → push `InsertRows` command
  - `delete_rows` → push `DeleteRows` command
- [ ] Add `undo(&mut self) -> Option<String>` method
  - Pop from history, execute undo, return description
- [ ] Add `redo(&mut self) -> Option<String>` method
  - Pop from redo stack, execute redo, return description
- [ ] Add `repeat_last_edit(&mut self) -> Option<String>` method
  - Get last repeatable command, apply to current position

**File: `src/input/handler.rs`**
- [ ] Add `u` handler in Normal mode
  - Call `app.undo()`
  - Show "Undo: <description>" message
  - Show "Nothing to undo" if stack empty
- [ ] Add `Ctrl+r` handler in Normal mode
  - Call `app.redo()`
  - Show "Redo: <description>" message
  - Show "Nothing to redo" if stack empty
- [ ] Add `.` handler in Normal mode
  - Call `app.repeat_last_edit()`
  - Apply last edit at current cursor position
  - Show "Repeat: <description>" message

**File: `src/lib.rs`**
- [ ] Add `mod history;`

**File: `src/csv/document.rs`**
- [ ] Modify mutation methods to return command info for history
- [ ] Or refactor to use command pattern throughout

### Tests to Add (`tests/history_test.rs`)
- [ ] `test_u_undoes_cell_edit`
- [ ] `test_u_undoes_row_insert`
- [ ] `test_u_undoes_row_delete`
- [ ] `test_ctrl_r_redoes_after_undo`
- [ ] `test_multiple_undo_redo_sequence`
- [ ] `test_undo_clears_redo_on_new_edit`
- [ ] `test_undo_nothing_shows_message`
- [ ] `test_redo_nothing_shows_message`
- [ ] `test_dot_repeats_cell_edit`
- [ ] `test_dot_repeats_at_current_position`
- [ ] `test_dot_after_row_delete`
- [ ] `test_history_respects_max_size`
- [ ] `test_undo_updates_dirty_flag_correctly`

### Acceptance Criteria
- [ ] `u` undoes the last edit operation
- [ ] `u` shows description of what was undone
- [ ] Multiple `u` presses undo multiple operations
- [ ] `Ctrl+r` redoes the last undone operation
- [ ] `Ctrl+r` shows description of what was redone
- [ ] New edit after undo clears redo stack
- [ ] `.` repeats last edit at current cursor position
- [ ] `.` works for cell edits
- [ ] `.` works for row operations
- [ ] "Nothing to undo/redo" shown when stack empty
- [ ] Undo stack has reasonable size limit
- [ ] `is_dirty` correctly reflects undo/redo state
- [ ] All existing tests pass
- [ ] No clippy warnings

---

## v0.9.0 - Power Features

*Fill operations, transforms, and quality of life*

### Keybindings to Implement

**Fill Operations (Insert mode):**
| Key | Action |
|-----|--------|
| `Ctrl+d` | Fill from cell above |
| `Ctrl+r` | Fill from cell to the left |
| `Ctrl+Shift+d` | Auto-increment from cell above |

**Date/Time Entry (Insert mode):**
| Key | Action |
|-----|--------|
| `Ctrl+;` | Insert current date (YYYY-MM-DD) |
| `Ctrl+Shift+;` | Insert current time (HH:MM:SS) |

**Cell Transforms (Normal mode):**
| Key | Action |
|-----|--------|
| `~` | Toggle case (UPPER ↔ lower) |
| `gU` | Uppercase entire cell |
| `gu` | Lowercase entire cell |
| `g~` | Title Case cell |
| `g.` | Toggle boolean (yes↔no, true↔false, 1↔0) |

**Row Operations (Normal mode):**
| Key | Action |
|-----|--------|
| `gj` | Swap current row with row below |
| `gk` | Swap current row with row above |

**Excel Aliases:**
| Key | Action |
|-----|--------|
| `F4` | Repeat last edit (alias for `.`) |
| `Ctrl+-` | Delete current row (alias for `dd`) |

### Implementation Steps

**File: `src/transforms/mod.rs` (new file)**
- [ ] Create transforms module
- [ ] Implement `toggle_case(s: &str) -> String`
  - If all uppercase → lowercase
  - If all lowercase → uppercase
  - If mixed → uppercase
- [ ] Implement `uppercase(s: &str) -> String`
- [ ] Implement `lowercase(s: &str) -> String`
- [ ] Implement `title_case(s: &str) -> String`
  - Capitalize first letter of each word
- [ ] Implement `toggle_boolean(s: &str) -> Option<String>`
  - yes↔no, Yes↔No, YES↔NO
  - true↔false, True↔False, TRUE↔FALSE
  - 1↔0, Y↔N, y↔n
  - Return None if not a recognized boolean

**File: `src/input/handler.rs`**
- [ ] **Insert mode - Fill operations:**
  - `Ctrl+d`: Get cell above, insert into edit buffer
  - `Ctrl+r`: Get cell to the left, insert into edit buffer
  - `Ctrl+Shift+d`: Parse cell above as number, increment, insert
- [ ] **Insert mode - Date/Time:**
  - `Ctrl+;`: Insert `chrono::Local::now().format("%Y-%m-%d")`
  - `Ctrl+Shift+;`: Insert `chrono::Local::now().format("%H:%M:%S")`
- [ ] **Normal mode - Case transforms:**
  - `~`: Apply `toggle_case` to current cell
  - Add `PendingCommand::G` handling for:
    - `gU`: Apply `uppercase` to current cell
    - `gu`: Apply `lowercase` to current cell
    - `g~`: Apply `title_case` to current cell
    - `g.`: Apply `toggle_boolean` to current cell
- [ ] **Normal mode - Row swap:**
  - `gj`: Swap current row with row at index+1
  - `gk`: Swap current row with row at index-1
- [ ] **Excel aliases:**
  - `F4`: Same as `.` (repeat last edit)
  - `Ctrl+-`: Same as `dd` (delete row)

**File: `src/csv/document.rs`**
- [ ] Add `swap_rows(&mut self, a: RowIndex, b: RowIndex) -> bool`
  - Return false if either index out of bounds
  - Swap in-place
  - Set `is_dirty = true`
- [ ] Add `get_cell_above(&self, row: RowIndex, col: ColIndex) -> Option<&str>`
- [ ] Add `get_cell_left(&self, row: RowIndex, col: ColIndex) -> Option<&str>`

**File: `src/input/actions.rs`**
- [ ] Extend `PendingCommand::G` cases for `gU`, `gu`, `g~`, `g.`, `gj`, `gk`

**File: `Cargo.toml`**
- [ ] Add `chrono` dependency for date/time features

**File: `src/lib.rs`**
- [ ] Add `mod transforms;`

### Tests to Add (`tests/power_features_test.rs`)
- [ ] `test_ctrl_d_fills_from_above`
- [ ] `test_ctrl_d_at_first_row_does_nothing`
- [ ] `test_ctrl_r_fills_from_left`
- [ ] `test_ctrl_shift_d_auto_increments`
- [ ] `test_ctrl_semicolon_inserts_date`
- [ ] `test_ctrl_shift_semicolon_inserts_time`
- [ ] `test_tilde_toggles_case_upper_to_lower`
- [ ] `test_tilde_toggles_case_lower_to_upper`
- [ ] `test_gU_uppercases_cell`
- [ ] `test_gu_lowercases_cell`
- [ ] `test_g_tilde_title_cases_cell`
- [ ] `test_g_dot_toggles_boolean_yes_no`
- [ ] `test_g_dot_toggles_boolean_true_false`
- [ ] `test_g_dot_toggles_boolean_1_0`
- [ ] `test_g_dot_non_boolean_shows_error`
- [ ] `test_gj_swaps_row_down`
- [ ] `test_gk_swaps_row_up`
- [ ] `test_gj_at_last_row_does_nothing`
- [ ] `test_gk_at_first_row_does_nothing`
- [ ] `test_F4_repeats_last_edit`
- [ ] `test_ctrl_minus_deletes_row`

### Acceptance Criteria
- [ ] `Ctrl+d` in Insert mode fills from cell above
- [ ] `Ctrl+r` in Insert mode fills from cell to left
- [ ] `Ctrl+Shift+d` auto-increments numeric values
- [ ] `Ctrl+;` inserts current date in YYYY-MM-DD format
- [ ] `Ctrl+Shift+;` inserts current time in HH:MM:SS format
- [ ] `~` toggles case of current cell
- [ ] `gU`, `gu`, `g~` transform cell case appropriately
- [ ] `g.` toggles boolean values (yes/no, true/false, 1/0)
- [ ] `gj` swaps current row with row below
- [ ] `gk` swaps current row with row above
- [ ] `F4` repeats last edit (Excel compatibility)
- [ ] `Ctrl+-` deletes current row (Excel compatibility)
- [ ] All transforms record to history for undo
- [ ] All existing tests pass
- [ ] No clippy warnings

---

## v1.0.0 - First Stable Release

*All core vim features working, stable command interface*

### Pre-Release Checklist

**Feature Verification:**
- [ ] Navigation: `hjkl`, arrows, `gg`, `G`, `5G`, `0`, `$`, `w`, `b`, `e`
- [ ] Viewport: `zt`, `zz`, `zb`
- [ ] Count prefixes: `5j`, `5dd`, `5yy`
- [ ] Insert mode: `i`, `a`, `I`, `A`, `s`, `o`, `O`
- [ ] Row operations: `dd`, `yy`, `p`, `P`, `cc`
- [ ] Visual mode: `V`, `d`, `y` on selection, `gv`
- [ ] Search: `/`, `?`, `n`, `N`, `*`, `#`, `:noh`
- [ ] Persistence: `:w`, `:wq`, `:q`, `:q!`, `ZZ`, `ZQ`
- [ ] Undo/Redo: `u`, `Ctrl+r`, `.`
- [ ] Transforms: `~`, `gU`, `gu`, `g~`, `g.`
- [ ] Fill: `Ctrl+d`, `Ctrl+r`, `Ctrl+;`
- [ ] File switching: `[`, `]`

**Code Quality:**
- [ ] All tests passing (target: 500+ tests)
- [ ] Zero clippy warnings
- [ ] Zero compiler warnings
- [ ] Code coverage > 80%
- [ ] No panics in normal operation

**Documentation:**
- [ ] README.md complete with:
  - [ ] Installation instructions
  - [ ] Quick start guide
  - [ ] Feature overview
  - [ ] Keybinding reference
- [ ] `docs/keybindings.md` up to date
- [ ] `--help` output accurate
- [ ] Man page (optional)

**Distribution:**
- [ ] Cargo.toml metadata complete (description, license, keywords)
- [ ] GitHub releases configured
- [ ] Binary builds for Linux, macOS, Windows (via CI)

**Performance:**
- [ ] Opens 100K row file in < 2 seconds
- [ ] Smooth scrolling at 60fps
- [ ] Memory usage < 2x file size

### Acceptance Criteria
- [ ] All v0.4.0 - v0.9.0 features complete
- [ ] No known critical bugs
- [ ] Documentation matches implementation
- [ ] Ready for public announcement

---

## v1.1.0 - Marks & Registers

*Advanced vim features*

### Keybindings to Implement

**Marks:**
| Key | Action |
|-----|--------|
| `m[a-z]` | Set mark at current cell |
| `'[a-z]` | Jump to row of mark |
| `` `[a-z] `` | Jump to exact cell of mark |
| `''` | Jump to previous position |
| `'.` | Jump to last edited cell |
| `:marks` | List all marks |
| `:delmarks [a-z]` | Delete specific mark |

**Registers:**
| Key | Action |
|-----|--------|
| `"[a-z]yy` | Yank row to named register |
| `"[a-z]p` | Paste from named register |
| `"+y` | Yank to system clipboard |
| `"+p` | Paste from system clipboard |
| `"0p` | Paste from yank register |
| `:reg` | Show register contents |

### Implementation Steps

**File: `src/marks/mod.rs` (new file)**
- [ ] Create marks module
- [ ] Define `Mark` struct: `{ row: RowIndex, col: ColIndex }`
- [ ] Define `Marks` struct with `HashMap<char, Mark>`
- [ ] Implement `set_mark(name: char, row: RowIndex, col: ColIndex)`
- [ ] Implement `get_mark(name: char) -> Option<&Mark>`
- [ ] Implement `delete_mark(name: char)`
- [ ] Implement `list_marks() -> Vec<(char, &Mark)>`
- [ ] Store `previous_position` for `''` command
- [ ] Track `last_edit_position` (already exists in App)

**File: `src/registers/mod.rs` (new file)**
- [ ] Create registers module
- [ ] Define `Register` enum: `{ Cell(String), Row(Vec<String>), Rows(Vec<Vec<String>>) }`
- [ ] Define `Registers` struct with `HashMap<char, Register>`
- [ ] Implement `set_register(name: char, content: Register)`
- [ ] Implement `get_register(name: char) -> Option<&Register>`
- [ ] Implement `"0` register (last yank, auto-updated)
- [ ] Implement `"+` register (system clipboard via `arboard` crate)
- [ ] Implement `list_registers() -> Vec<(char, &Register)>`

**File: `src/app/mod.rs`**
- [ ] Add `marks: Marks` field
- [ ] Add `registers: Registers` field
- [ ] Add `previous_position: Option<(RowIndex, ColIndex)>` field
- [ ] Update position tracking on navigation

**File: `src/input/handler.rs`**
- [ ] Add `m` handler to enter mark-set mode
  - Next character [a-z] sets mark at current position
- [ ] Add `'` handler to jump to mark row
- [ ] Add `` ` `` handler to jump to exact mark position
- [ ] Add `''` handler to jump to previous position
- [ ] Add `'.` handler to jump to last edit position
- [ ] Add `"` handler to enter register mode
  - Next character [a-z0+] selects register
  - Following command uses that register
- [ ] Add `:marks` command handler
- [ ] Add `:delmarks` command handler
- [ ] Add `:reg` command handler

**File: `Cargo.toml`**
- [ ] Add `arboard` dependency for system clipboard

**File: `src/lib.rs`**
- [ ] Add `mod marks;`
- [ ] Add `mod registers;`

### Tests to Add (`tests/marks_registers_test.rs`)
- [ ] `test_m_a_sets_mark_a`
- [ ] `test_apostrophe_a_jumps_to_mark_row`
- [ ] `test_backtick_a_jumps_to_exact_position`
- [ ] `test_apostrophe_apostrophe_jumps_to_previous`
- [ ] `test_apostrophe_dot_jumps_to_last_edit`
- [ ] `test_marks_command_lists_marks`
- [ ] `test_delmarks_removes_mark`
- [ ] `test_quote_a_yy_yanks_to_register_a`
- [ ] `test_quote_a_p_pastes_from_register_a`
- [ ] `test_quote_0_p_pastes_last_yank`
- [ ] `test_quote_plus_y_copies_to_clipboard`
- [ ] `test_quote_plus_p_pastes_from_clipboard`
- [ ] `test_reg_command_shows_registers`

### Acceptance Criteria
- [ ] `ma` sets mark 'a' at current cell
- [ ] `'a` jumps to row of mark 'a'
- [ ] `` `a `` jumps to exact cell of mark 'a'
- [ ] `''` jumps to position before last jump
- [ ] `'.` jumps to last edited cell
- [ ] `:marks` shows all set marks
- [ ] `"ayy` yanks to register 'a'
- [ ] `"ap` pastes from register 'a'
- [ ] `"+y` copies to system clipboard
- [ ] `"+p` pastes from system clipboard
- [ ] `:reg` shows register contents
- [ ] All existing tests pass
- [ ] No clippy warnings

---

## v1.2.0 - Text Objects

*CSV-specific text objects*

### Text Objects to Implement

| Object | Meaning | Use Cases |
|--------|---------|-----------|
| `ic` | Inner cell (content only) | `dic`, `yic`, `cic` |
| `ac` | A cell (including cell) | `dac`, `yac`, `cac` |
| `ir` | Inner row (all cells) | `dir`, `yir`, `cir` |
| `ar` | A row (entire row) | `dar`, `yar`, `car` |

### Implementation Steps

**File: `src/text_objects/mod.rs` (new file)**
- [ ] Create text_objects module
- [ ] Define `TextObject` enum:
  ```rust
  pub enum TextObject {
      InnerCell,   // ic - current cell content
      ACell,       // ac - current cell (same as ic for CSV)
      InnerRow,    // ir - all cells in current row
      ARow,        // ar - entire row (same as ir for CSV)
  }
  ```
- [ ] Implement `get_cell_content(app: &App) -> String`
- [ ] Implement `get_row_content(app: &App) -> Vec<String>`
- [ ] Define how text objects work with operators:
  - `d` + object → delete
  - `y` + object → yank
  - `c` + object → change (delete + insert)

**File: `src/input/handler.rs`**
- [ ] Extend `PendingCommand::D` to handle text objects:
  - After `d`, if next is `i` or `a`, enter text object mode
  - `dic` → delete inner cell (clear cell content)
  - `dac` → delete a cell (same as dic for CSV)
  - `dir` → delete inner row (clear all cells in row)
  - `dar` → delete a row (remove entire row, same as dd)
- [ ] Extend `PendingCommand::Y` to handle text objects:
  - `yic` → yank inner cell to register
  - `yac` → yank a cell
  - `yir` → yank inner row
  - `yar` → yank a row (same as yy)
- [ ] Add `PendingCommand::C` for change operator:
  - `cic` → clear cell, enter insert mode
  - `cac` → clear cell, enter insert mode
  - `cir` → clear row, enter insert mode at first column
  - `car` → delete row, insert new row, enter insert mode

**File: `src/input/actions.rs`**
- [ ] Add `PendingCommand::C` variant
- [ ] Add state tracking for operator + object parsing

**File: `src/lib.rs`**
- [ ] Add `mod text_objects;`

### Tests to Add (`tests/text_objects_test.rs`)
- [ ] `test_dic_clears_cell_content`
- [ ] `test_dac_clears_cell_content`
- [ ] `test_dir_clears_all_cells_in_row`
- [ ] `test_dar_deletes_entire_row`
- [ ] `test_yic_yanks_cell_content`
- [ ] `test_yac_yanks_cell_content`
- [ ] `test_yir_yanks_row_content`
- [ ] `test_yar_yanks_entire_row`
- [ ] `test_cic_clears_and_enters_insert`
- [ ] `test_cac_clears_and_enters_insert`
- [ ] `test_cir_clears_row_enters_insert`
- [ ] `test_car_replaces_row_enters_insert`
- [ ] `test_text_objects_with_registers` (e.g., `"ayic`)
- [ ] `test_invalid_text_object_shows_error`

### Acceptance Criteria
- [ ] `dic` clears current cell content
- [ ] `yic` yanks current cell content
- [ ] `cic` clears cell and enters Insert mode
- [ ] `dir` clears all cells in current row
- [ ] `yir` yanks all cells in current row
- [ ] `cir` clears row and enters Insert mode
- [ ] `dar` deletes entire row (same as `dd`)
- [ ] `yar` yanks entire row (same as `yy`)
- [ ] `car` replaces row and enters Insert mode
- [ ] Text objects work with named registers
- [ ] Invalid text objects show error message
- [ ] All existing tests pass
- [ ] No clippy warnings

---

## v1.3.0 - Sorting & Filtering

*Data organization*

### Commands to Implement

**Sorting:**
| Command | Action |
|---------|--------|
| `:sort` | Sort by current column ascending |
| `:sort!` | Sort by current column descending |
| `:sort <col>` | Sort by specified column |
| `:sort <col1>, <col2>` | Multi-column sort |

**Filtering:**
| Command | Action |
|---------|--------|
| `:filter <expr>` | Filter rows by expression |
| `:nofilter` | Clear all filters |
| `:filters` | Show active filters |

**Data Cleanup:**
| Command | Action |
|---------|--------|
| `:dedup` | Remove duplicate rows |
| `:dedup <col>` | Remove rows with duplicate values in column |
| `:trim` | Trim whitespace from selection/cell |
| `:upper` | Uppercase selection/cell |
| `:lower` | Lowercase selection/cell |
| `:title` | Title case selection/cell |
| `:fill <val>` | Fill selection with value |
| `:filldown` | Fill empty cells from above |
| `:fillseries` | Auto-increment series |
| `:s/pat/rep/` | Find and replace |

### Implementation Steps

**File: `src/sort/mod.rs` (new file)**
- [ ] Create sort module
- [ ] Define `SortOrder` enum: `Ascending`, `Descending`
- [ ] Define `SortKey` struct: `{ column: ColIndex, order: SortOrder }`
- [ ] Implement `sort_rows(rows: &mut Vec<Vec<String>>, keys: &[SortKey])`
  - Detect numeric vs text automatically
  - Support multi-column sorting
- [ ] Implement `parse_sort_command(args: &str) -> Result<Vec<SortKey>>`

**File: `src/filter/mod.rs` (new file)**
- [ ] Create filter module
- [ ] Define `FilterOp` enum: `Eq`, `Ne`, `Gt`, `Lt`, `Ge`, `Le`, `Contains`, `StartsWith`, `EndsWith`
- [ ] Define `FilterExpr` struct: `{ column: ColIndex, op: FilterOp, value: String }`
- [ ] Define `FilterCombinator`: `And`, `Or`
- [ ] Implement `parse_filter_expression(expr: &str) -> Result<Vec<FilterExpr>>`
- [ ] Implement `matches_filter(row: &[String], filters: &[FilterExpr]) -> bool`
- [ ] Store filtered indices rather than modifying data

**File: `src/cleanup/mod.rs` (new file)**
- [ ] Create cleanup module
- [ ] Implement `dedup_rows(rows: &mut Vec<Vec<String>>, column: Option<ColIndex>)`
- [ ] Implement `trim_cells(rows: &mut Vec<Vec<String>>, selection: &Selection)`
- [ ] Implement `transform_cells(rows: &mut ..., selection: &Selection, transform: fn(&str) -> String)`
- [ ] Implement `fill_cells(rows: &mut ..., selection: &Selection, value: &str)`
- [ ] Implement `fill_down(rows: &mut ..., column: ColIndex)`
- [ ] Implement `fill_series(rows: &mut ..., column: ColIndex, start: &str)`
- [ ] Implement `find_replace(rows: &mut ..., pattern: &str, replacement: &str, flags: &str)`

**File: `src/app/mod.rs`**
- [ ] Add `active_filters: Vec<FilterExpr>` field
- [ ] Add `filtered_row_indices: Option<Vec<usize>>` field
- [ ] Add `sort_state: Option<Vec<SortKey>>` field
- [ ] Modify row iteration to respect filters
- [ ] Update row count to show "X of Y rows"

**File: `src/input/handler.rs`**
- [ ] Add `:sort` command handler
- [ ] Add `:sort!` command handler
- [ ] Add `:filter` command handler
- [ ] Add `:nofilter` command handler
- [ ] Add `:filters` command handler
- [ ] Add `:dedup` command handler
- [ ] Add `:trim` command handler
- [ ] Add `:upper`, `:lower`, `:title` command handlers
- [ ] Add `:fill` command handler
- [ ] Add `:filldown` command handler
- [ ] Add `:fillseries` command handler
- [ ] Add `:s/pat/rep/` command handler

**File: `src/ui/table.rs`**
- [ ] Show sort indicator in column header (↑ or ↓)
- [ ] Dim or hide filtered-out rows (or show only matching rows)

**File: `src/ui/status.rs`**
- [ ] Show "Filtered: X of Y rows" when filter active
- [ ] Show sort column indicator

**File: `src/lib.rs`**
- [ ] Add `mod sort;`
- [ ] Add `mod filter;`
- [ ] Add `mod cleanup;`

### Tests to Add (`tests/sort_filter_test.rs`)
- [ ] `test_sort_ascending`
- [ ] `test_sort_descending`
- [ ] `test_sort_numeric`
- [ ] `test_sort_multi_column`
- [ ] `test_filter_equals`
- [ ] `test_filter_greater_than`
- [ ] `test_filter_contains`
- [ ] `test_filter_and_condition`
- [ ] `test_nofilter_clears`
- [ ] `test_dedup_removes_duplicates`
- [ ] `test_dedup_by_column`
- [ ] `test_trim_whitespace`
- [ ] `test_fill_selection`
- [ ] `test_filldown`
- [ ] `test_fillseries_numeric`
- [ ] `test_fillseries_alpha`
- [ ] `test_find_replace`
- [ ] `test_find_replace_regex`

### Acceptance Criteria
- [ ] `:sort` sorts current column ascending
- [ ] `:sort!` sorts current column descending
- [ ] `:sort A` sorts by column A
- [ ] `:sort A, B` sorts by A then B
- [ ] Numeric columns sort numerically
- [ ] Sort indicator shows in header
- [ ] `:filter Age > 30` hides non-matching rows
- [ ] `:filter` supports `=`, `!=`, `>`, `<`, `contains`, `starts`, `ends`
- [ ] `:filter ... AND ...` combines conditions
- [ ] `:nofilter` shows all rows again
- [ ] Status shows "Filtered: X of Y rows"
- [ ] `:dedup` removes exact duplicate rows
- [ ] `:trim` removes leading/trailing whitespace
- [ ] `:fill value` fills selection
- [ ] `:filldown` propagates values down
- [ ] `:s/old/new/g` replaces text
- [ ] All operations are undoable
- [ ] All existing tests pass
- [ ] No clippy warnings

---

## v1.4.0 - Column Operations

*Column management*

### Keybindings to Implement

**Column Operators:**
| Key | Action |
|-----|--------|
| `dc` | Delete current column |
| `yc` | Yank current column |
| `pc` | Paste column after current |
| `Pc` | Paste column before current |
| `gc` | Duplicate current column |

**Column Movement:**
| Key | Action |
|-----|--------|
| `g<` | Move current column left |
| `g>` | Move current column right |

**Header Editing:**
| Key | Action |
|-----|--------|
| `gh` | Enter HeaderEdit mode |

**Column Visibility:**
| Key | Action |
|-----|--------|
| `zc` | Hide current column |
| `zo` | Show all hidden columns |

**Commands:**
| Command | Action |
|---------|--------|
| `:addcol` | Add column after current |
| `:addcol before` | Add column before current |
| `:addcol <name>` | Add column with name |
| `:rename <name>` | Rename current column |
| `:swap <A> <B>` | Swap columns A and B |
| `:hide <col>` | Hide specific column |
| `:show` | Show all hidden columns |
| `:freeze` | Freeze header row |
| `:freeze <n>` | Freeze first n rows |
| `:unfreeze` | Remove freeze |
| `:autowidth` | Auto-size column to content |

### Implementation Steps

**File: `src/csv/document.rs`**
- [ ] Add `delete_column(&mut self, col: ColIndex) -> Option<Vec<String>>`
  - Remove column from headers
  - Remove column from each row
  - Return deleted column data
- [ ] Add `insert_column(&mut self, at: ColIndex, header: String, data: Vec<String>)`
- [ ] Add `move_column(&mut self, from: ColIndex, to: ColIndex)`
- [ ] Add `swap_columns(&mut self, a: ColIndex, b: ColIndex)`
- [ ] Add `duplicate_column(&mut self, col: ColIndex)`
- [ ] Add `set_header(&mut self, col: ColIndex, name: String)`

**File: `src/app/mod.rs`**
- [ ] Add `column_clipboard: Option<Vec<String>>` field
- [ ] Add `hidden_columns: HashSet<ColIndex>` field
- [ ] Add `frozen_rows: usize` field (0 = no freeze)
- [ ] Add `column_widths: HashMap<ColIndex, u16>` field (for manual sizing)
- [ ] Modify column iteration to skip hidden columns

**File: `src/input/handler.rs`**
- [ ] Add `dc` handler (d + c text object already exists, extend for column)
  - Delete column, store in column_clipboard
- [ ] Add `yc` handler
  - Copy column to column_clipboard
- [ ] Add `pc` / `Pc` handlers
  - Insert column from clipboard after/before current
- [ ] Add `gc` handler (g + c sequence)
  - Duplicate current column
- [ ] Add `g<` / `g>` handlers
  - Move column left/right
- [ ] Add `gh` handler
  - Enter HeaderEdit mode
  - Edit buffer initialized with current header
- [ ] Add `handle_header_edit_mode()` function
  - Similar to Insert mode but for header
  - Enter commits, Esc cancels
- [ ] Add `zc` / `zo` handlers
  - Hide/show columns
- [ ] Add command handlers: `:addcol`, `:rename`, `:swap`, `:hide`, `:show`
- [ ] Add command handlers: `:freeze`, `:unfreeze`, `:autowidth`

**File: `src/ui/table.rs`**
- [ ] Skip hidden columns when rendering
- [ ] Show frozen rows at top (separate from scrollable area)
- [ ] Show freeze line indicator
- [ ] Use custom column widths when set

**File: `src/ui/status.rs`**
- [ ] Show quick stats when in Visual mode with selection:
  - `Sum: X | Avg: Y | Count: Z` for numeric columns
  - `Count: Z` for text columns

### Tests to Add (`tests/column_operations_test.rs`)
- [ ] `test_dc_deletes_column`
- [ ] `test_yc_yanks_column`
- [ ] `test_pc_pastes_column_after`
- [ ] `test_Pc_pastes_column_before`
- [ ] `test_gc_duplicates_column`
- [ ] `test_g_less_moves_column_left`
- [ ] `test_g_greater_moves_column_right`
- [ ] `test_gh_enters_header_edit`
- [ ] `test_header_edit_commit`
- [ ] `test_header_edit_cancel`
- [ ] `test_zc_hides_column`
- [ ] `test_zo_shows_all_columns`
- [ ] `test_addcol_adds_column`
- [ ] `test_rename_changes_header`
- [ ] `test_swap_columns`
- [ ] `test_freeze_rows`
- [ ] `test_quick_stats_sum`
- [ ] `test_quick_stats_count`

### Acceptance Criteria
- [ ] `dc` deletes current column
- [ ] `yc` yanks column to clipboard
- [ ] `pc` pastes column after current
- [ ] `Pc` pastes column before current
- [ ] `gc` duplicates current column
- [ ] `g<` moves column left
- [ ] `g>` moves column right
- [ ] `gh` enters header edit mode
- [ ] Header edit works like Insert mode
- [ ] `zc` hides current column
- [ ] `zo` shows all hidden columns
- [ ] `:addcol` adds new column
- [ ] `:rename` renames column header
- [ ] `:swap A B` swaps columns
- [ ] `:freeze` freezes header row
- [ ] Frozen rows stay visible during scroll
- [ ] Quick stats show in status bar for selection
- [ ] All column operations are undoable
- [ ] All existing tests pass
- [ ] No clippy warnings

---

## v1.5.0 - Advanced Features

*Power user features*

### Keybindings to Implement

**Cell Reference Navigation:**
| Command | Action |
|---------|--------|
| `:A5` | Go to cell A5 (column A, row 5) |
| `:B12` | Go to cell B12 |
| `:AA1` | Go to cell AA1 |

**Smart Column Navigation:**
| Key | Action |
|-----|--------|
| `{` | Previous empty cell in column |
| `}` | Next empty cell in column |
| `[[` | Previous different value in column |
| `]]` | Next different value in column |
| `gf` | First non-empty cell in column |
| `gl` | Last non-empty cell in column |

**Selection Helpers:**
| Key | Action |
|-----|--------|
| `g*` | Select all cells with same value |
| `gn` | Select next search match |

**Auto-complete (Insert mode):**
| Key | Action |
|-----|--------|
| `Ctrl+n` | Next suggestion |
| `Ctrl+p` | Previous suggestion |
| `Tab` | Accept suggestion |

**Macros:**
| Key | Action |
|-----|--------|
| `q[a-z]` | Start recording macro |
| `q` | Stop recording |
| `@[a-z]` | Play macro |
| `@@` | Repeat last macro |

### Implementation Steps

**File: `src/navigation/smart.rs` (new file)**
- [ ] Create smart navigation module
- [ ] Implement `find_prev_empty_in_column(doc: &Document, from: RowIndex, col: ColIndex) -> Option<RowIndex>`
- [ ] Implement `find_next_empty_in_column(doc: &Document, from: RowIndex, col: ColIndex) -> Option<RowIndex>`
- [ ] Implement `find_prev_different_in_column(doc: &Document, from: RowIndex, col: ColIndex) -> Option<RowIndex>`
- [ ] Implement `find_next_different_in_column(doc: &Document, from: RowIndex, col: ColIndex) -> Option<RowIndex>`
- [ ] Implement `find_first_nonempty_in_column(doc: &Document, col: ColIndex) -> Option<RowIndex>`
- [ ] Implement `find_last_nonempty_in_column(doc: &Document, col: ColIndex) -> Option<RowIndex>`

**File: `src/autocomplete/mod.rs` (new file)**
- [ ] Create autocomplete module
- [ ] Implement `get_column_values(doc: &Document, col: ColIndex) -> Vec<String>`
  - Get unique values from column
- [ ] Implement `filter_suggestions(values: &[String], prefix: &str) -> Vec<String>`
- [ ] Define `AutocompleteState` struct:
  ```rust
  pub struct AutocompleteState {
      suggestions: Vec<String>,
      current_index: usize,
      prefix: String,
  }
  ```
- [ ] Implement suggestion cycling with Ctrl+n/Ctrl+p

**File: `src/completion/mod.rs` (new file)**
- [ ] Create command completion module
- [ ] Define list of all commands for completion
- [ ] Implement `complete_command(prefix: &str) -> Vec<String>`
- [ ] Implement `complete_column_name(doc: &Document, prefix: &str) -> Vec<String>`

**File: `src/macros/mod.rs` (new file)**
- [ ] Create macros module
- [ ] Define `Macro` struct: `Vec<KeyEvent>`
- [ ] Define `MacroState` struct:
  ```rust
  pub struct MacroState {
      recording: Option<char>,       // Which register we're recording to
      current_macro: Vec<KeyEvent>,  // Keys being recorded
      macros: HashMap<char, Vec<KeyEvent>>,
      last_played: Option<char>,     // For @@ command
  }
  ```
- [ ] Implement `start_recording(register: char)`
- [ ] Implement `stop_recording()`
- [ ] Implement `record_key(key: KeyEvent)`
- [ ] Implement `play_macro(register: char) -> Vec<KeyEvent>`

**File: `src/config/mod.rs` (new file)**
- [ ] Create config module
- [ ] Define `Config` struct:
  ```rust
  pub struct Config {
      keybindings: HashMap<String, String>,
      theme: Theme,
      default_column_width: u16,
      show_line_numbers: bool,
      // ...
  }
  ```
- [ ] Implement `load_config(path: &Path) -> Result<Config>`
- [ ] Implement `default_config() -> Config`
- [ ] Parse TOML config file

**File: `src/app/mod.rs`**
- [ ] Add `macro_state: MacroState` field
- [ ] Add `autocomplete_state: Option<AutocompleteState>` field
- [ ] Add `config: Config` field
- [ ] Load config on startup

**File: `src/input/handler.rs`**
- [ ] Add cell reference parsing (`:A5` format)
- [ ] Add `{` / `}` handlers for empty cell navigation
- [ ] Add `[[` / `]]` handlers for different value navigation
- [ ] Add `gf` / `gl` handlers for first/last non-empty
- [ ] Add `g*` handler for selecting matching cells
- [ ] Add `gn` handler for selecting next search match
- [ ] Add `Ctrl+n` / `Ctrl+p` in Insert mode for autocomplete
- [ ] Add Tab in Insert mode to accept suggestion
- [ ] Add Tab in Command mode for completion
- [ ] Add `q` handlers for macro recording
- [ ] Add `@` handlers for macro playback
- [ ] When recording, add keys to macro buffer

**File: `Cargo.toml`**
- [ ] Add `toml` dependency for config parsing
- [ ] Add `dirs` dependency for config path

**File: `src/lib.rs`**
- [ ] Add `mod autocomplete;`
- [ ] Add `mod completion;`
- [ ] Add `mod macros;`
- [ ] Add `mod config;`

### Tests to Add (`tests/advanced_features_test.rs`)
- [ ] `test_cell_reference_A5`
- [ ] `test_cell_reference_AA1`
- [ ] `test_brace_jumps_to_empty`
- [ ] `test_brackets_jump_to_different`
- [ ] `test_gf_finds_first_nonempty`
- [ ] `test_gl_finds_last_nonempty`
- [ ] `test_g_star_selects_matching`
- [ ] `test_ctrl_n_cycles_suggestions`
- [ ] `test_tab_accepts_suggestion`
- [ ] `test_command_completion`
- [ ] `test_column_name_completion`
- [ ] `test_macro_record_and_play`
- [ ] `test_macro_at_at_repeats`
- [ ] `test_config_loads`
- [ ] `test_config_default_values`

### Acceptance Criteria
- [ ] `:A5` jumps to column A, row 5
- [ ] `{` / `}` navigate to empty cells in column
- [ ] `[[` / `]]` navigate to different values in column
- [ ] `gf` / `gl` find first/last non-empty in column
- [ ] `g*` selects all cells with matching value
- [ ] `Ctrl+n` / `Ctrl+p` cycle autocomplete suggestions
- [ ] Tab accepts autocomplete suggestion
- [ ] Tab completes commands in command mode
- [ ] Tab completes column names after `:c `
- [ ] `qa` starts recording macro to register 'a'
- [ ] `q` stops recording
- [ ] `@a` plays macro from register 'a'
- [ ] `@@` repeats last played macro
- [ ] Config file loads from `~/.config/lazycsv/config.toml`
- [ ] Default config used if file doesn't exist
- [ ] All existing tests pass
- [ ] No clippy warnings

---

## v1.6.0 - Data Analysis & Export

*Advanced data operations*

### Commands to Implement

**Data Analysis:**
| Command | Action |
|---------|--------|
| `:stats` | Show statistics overlay |
| `:stats <col>` | Show stats for specific column |
| `:plot` | Show plot overlay |
| `:plot bar` | Bar chart for categorical |
| `:plot hist` | Histogram for numeric |

**Data Transformation:**
| Command | Action |
|---------|--------|
| `:transpose` | Swap rows and columns |
| `:s/pat/rep/flags` | Regex search and replace |

**Export:**
| Command | Action |
|---------|--------|
| `:export json <file>` | Export as JSON |
| `:export md <file>` | Export as Markdown table |
| `:export html <file>` | Export as HTML table |

**Session:**
| Command | Action |
|---------|--------|
| `:mksession <file>` | Save session state |
| `:source <file>` | Load session state |

### Implementation Steps

**File: `src/stats/mod.rs` (new file)**
- [ ] Create stats module
- [ ] Implement statistical functions:
  - `count(values: &[&str]) -> usize`
  - `sum(values: &[&str]) -> Option<f64>` (returns None if non-numeric)
  - `average(values: &[&str]) -> Option<f64>`
  - `median(values: &[&str]) -> Option<f64>`
  - `min(values: &[&str]) -> Option<f64>`
  - `max(values: &[&str]) -> Option<f64>`
  - `std_dev(values: &[&str]) -> Option<f64>`
  - `unique_count(values: &[&str]) -> usize`
  - `top_n_frequent(values: &[&str], n: usize) -> Vec<(String, usize)>`
- [ ] Implement `column_stats(doc: &Document, col: ColIndex) -> ColumnStats`
- [ ] Define `ColumnStats` struct with all computed values

**File: `src/plot/mod.rs` (new file)**
- [ ] Create plot module
- [ ] Implement `bar_chart(data: &[(String, usize)], width: u16, height: u16) -> Vec<String>`
  - Use Unicode block characters: █ ▇ ▆ ▅ ▄ ▃ ▂ ▁
  - Scale to fit terminal
- [ ] Implement `histogram(values: &[f64], bins: usize, width: u16, height: u16) -> Vec<String>`
- [ ] Implement `sparkline(values: &[f64]) -> String`
  - Single line representation for status bar

**File: `src/export/mod.rs` (new file)**
- [ ] Create export module
- [ ] Implement `export_json(doc: &Document, path: &Path) -> Result<()>`
  - Array of objects with header keys
- [ ] Implement `export_markdown(doc: &Document, path: &Path) -> Result<()>`
  - Proper markdown table with alignment
- [ ] Implement `export_html(doc: &Document, path: &Path) -> Result<()>`
  - HTML table with basic styling

**File: `src/session/mod.rs` (new file)**
- [ ] Create session module
- [ ] Define `SessionState` struct:
  ```rust
  pub struct SessionState {
      files: Vec<PathBuf>,
      active_file_index: usize,
      cursor_positions: HashMap<PathBuf, (RowIndex, ColIndex)>,
      hidden_columns: HashMap<PathBuf, HashSet<ColIndex>>,
      frozen_rows: HashMap<PathBuf, usize>,
      marks: HashMap<PathBuf, HashMap<char, (RowIndex, ColIndex)>>,
      // ...
  }
  ```
- [ ] Implement `save_session(state: &SessionState, path: &Path) -> Result<()>`
- [ ] Implement `load_session(path: &Path) -> Result<SessionState>`
- [ ] Use TOML format for session files

**File: `src/transform/transpose.rs` (new file)**
- [ ] Implement `transpose(doc: &mut Document)`
  - Swap rows and columns
  - First column becomes headers
  - Headers become first column

**File: `src/ui/overlay.rs` (new file)**
- [ ] Create overlay module
- [ ] Define `Overlay` enum: `Stats`, `Plot`, `Help`
- [ ] Implement overlay rendering (centered popup)
- [ ] Handle overlay keyboard: Esc/q to close

**File: `src/app/mod.rs`**
- [ ] Add `overlay: Option<Overlay>` field
- [ ] Add overlay state management

**File: `src/input/handler.rs`**
- [ ] Add `:stats` command handler
  - Show stats overlay for current column or selection
- [ ] Add `:plot` command handler
  - Show plot overlay
- [ ] Add `:transpose` command handler
- [ ] Add `:export json <file>` command handler
- [ ] Add `:export md <file>` command handler
- [ ] Add `:export html <file>` command handler
- [ ] Add `:mksession` command handler
- [ ] Add `:source` command handler
- [ ] Handle overlay mode input (Esc to close)

**File: `src/ui/status.rs`**
- [ ] Show sparkline for numeric column when selected (optional feature)

**File: `src/lib.rs`**
- [ ] Add `mod stats;`
- [ ] Add `mod plot;`
- [ ] Add `mod export;`
- [ ] Add `mod session;`

### Tests to Add (`tests/data_analysis_test.rs`)
- [ ] `test_stats_count`
- [ ] `test_stats_sum`
- [ ] `test_stats_average`
- [ ] `test_stats_median`
- [ ] `test_stats_min_max`
- [ ] `test_stats_std_dev`
- [ ] `test_stats_unique_count`
- [ ] `test_stats_top_n_frequent`
- [ ] `test_bar_chart_renders`
- [ ] `test_histogram_renders`
- [ ] `test_sparkline_renders`
- [ ] `test_export_json`
- [ ] `test_export_markdown`
- [ ] `test_export_html`
- [ ] `test_transpose`
- [ ] `test_save_session`
- [ ] `test_load_session`
- [ ] `test_regex_replace`

### Acceptance Criteria
- [ ] `:stats` shows statistics overlay
- [ ] Stats include: count, sum, avg, median, min, max, std dev
- [ ] Stats show unique count and top 10 frequent values
- [ ] `:plot` shows visualization overlay
- [ ] Bar chart works for categorical data
- [ ] Histogram works for numeric data
- [ ] `:transpose` swaps rows and columns
- [ ] `:export json` creates valid JSON file
- [ ] `:export md` creates valid Markdown table
- [ ] `:export html` creates valid HTML table
- [ ] `:mksession` saves view state
- [ ] `:source` restores view state
- [ ] `:s/pat/rep/g` does regex replacement
- [ ] Overlays close with Esc or q
- [ ] All existing tests pass
- [ ] No clippy warnings

---

## Future Ideas (Post v1.6.0)

*May become future versions*

- [ ] Network file loading (HTTP/HTTPS URLs)
- [ ] SQL query mode (query CSV like a database)
- [ ] Excel file support (.xlsx)
- [ ] Formula evaluation (basic spreadsheet functions)
- [ ] Diff mode (compare two CSV files)
- [ ] Merge/join operations
- [ ] Pivot table support
- [ ] Advanced plotting:
  - Scatter plots (two numeric columns)
  - Time series (date column + value)
  - Box plots for distribution comparison
  - Correlation matrix heatmap
