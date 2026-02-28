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
- **v0.4.1** - Persistence & Edge Cases
- **v0.5.0** - Column Operations & Visual Mode
- **v0.6.0** - Magnifier Mode
- **v0.7.0** - Search
- **v0.8.0** - Undo/Redo
- **v0.9.0** - Transforms & Polish

**v1.0.0 - First Stable Release**

**Post-1.0: Future Enhancements**
- **v1.1.0** - Bulk Operations & Advanced Filtering
- **v1.2.0** - Advanced Column Operations
- **v1.3.0** - Data Analysis & Export

---

## Guiding Principles

- **Vim-First Philosophy:** Navigation and commands should feel native to vim users. Composable commands (operator + motion). No timeouts on pending commands. Clean status line.
- **Truly Hybrid:** Balance vim power with spreadsheet familiarity. Support both vim keys (hjkl) and arrow keys, vim commands and spreadsheet-like operations.
- **Three-Tier Operator System:** Cell (`x`) → Row (`dd`) → Column (`,dd`). Comma as leader for CSV-specific column operations.
- **Simplified Navigation:** Use `g` suffix for jumps: `5g` (row 5), `Bg` (column B), `A4g` (cell A4). Reserve `:` for operations only.
- **Header Toggle System:** Header row is always row 0. Toggle header mode with `:ht` to freeze/style row 0. When ON, `gg` goes to row 1 (first data row).
- **Command Ranges:** Standardized ranges: `:5,10d` for rows, `:B,Dd` for columns. Don't overcomplicate.
- **Dual Clipboard Buffers:** Separate row and column buffers. `yy`+`p` for rows, `,yy`+`,p` for columns. No cross-buffer pasting — mismatched paste shows "Nothing to paste".
- **Ephemeral Edits:** No changes saved to file until explicit `:w`. All edits update in-memory representation first.
- **Minimal UI Chrome:** No heavy borders. Use subtle separators. Maximum content, minimum decoration. Status line shows mode + row + column only.
- **In-Memory Only:** All CSV files loaded entirely into RAM for maximum performance.
- **CSV Only:** No Excel (.xlsx) support - CSV files only for simplicity.
- **Zero Configuration:** Works great out of the box. Optional `~/.config/lazycsv/config.toml` for power users with full customization.
- **Robust Error Handling:** Handle errors gracefully with clear, user-friendly feedback.

---

## Modal Editing Reference

LazyCSV uses vim-style modal editing with these modes:

| Mode | Indicator | Purpose | Entry | Exit |
|------|-----------|---------|-------|------|
| Normal | `NORMAL` | Navigation, commands | Default / `Esc` | N/A |
| Insert | `INSERT` | Quick single-cell editing | `i`, `a`, `A`, `I`, `s` | `Enter` (save), `Esc` (cancel) |
| Magnifier | `MAGNIFIER` | Full vim editor for cell | `m` on cell | `:wq`, `:q`, `ZZ` |
| Visual | `VISUAL` | Rectangular selection | `v` | `Esc`, or after operation |
| Visual Line | `VISUAL LINE` | Whole row selection | `V` | `Esc`, or after operation |
| Visual Column | `VISUAL COLUMN` | Whole column selection | `,v` | `Esc`, or after operation |
| Command | `:` prompt | Execute commands | `:` | `Enter` (execute), `Esc` (cancel) |

**Mode hierarchy:** Normal is the "home" mode. All other modes return to Normal.

**Insert vs Magnifier:**
- `i` - Quick edits (single-line, simple text)
- `m` - Complex edits (multi-line, full vim power)
- Default to Insert mode, manually upgrade to Magnifier when needed

---

## Header Row System

**Revolutionary simplification:** Header row is always row 0 (no special storage). Toggle header mode ON/OFF with `:ht`.

### Header Mode: ON (Default)
- Row 0 is styled/frozen as header row
- `gg` goes to row 1 (first data row)
- `k` from row 1 goes to row 0 (header row)
- Header mode remembered per-file in session (not persisted to disk)
- Deleting row 0 with `dd` auto-toggles header mode OFF
- Edit header cells with `i` (just like any other cell - no special keybinding needed)

### Header Mode: OFF
- Row 0 treated as normal data
- `gg` goes to row 0
- No frozen/styled header row
- Edit row 0 like any other row

### Behavior
- Default: Header mode ON for all files in directory
- `:ht` toggles current file's header mode
- Setting persists in RAM during session only
- Visual indicator: Row 0 styling (bold/frozen) when header mode ON
- No status line indicator needed

---

## Command Mode Reference

### Reserved Commands (Priority)
These commands always take priority:

| Command | Action |
|---------|--------|
| `:q` | Quit (checks all files for unsaved changes) |
| `:q!` | Force quit (discard all changes) |
| `:w` | Write all dirty files |
| `:Wq` | Write all dirty files and quit |
| `:wq` | Alias for `:Wq` |
| `:h` `:help` | Show full scrollable help buffer |
| `:noh` | Clear search highlighting |
| `:ht` | Toggle header mode for current file |
| `:delim X` | Set delimiter for current file (session-only, e.g., `:delim ;`) |
| `:new Name,Age,City` | Create new CSV with specified headers (0 rows) |
| `:new` | Create new CSV with 1 column "Column 1" (0 rows) |
| `:files` | Show file menu with numbers to select |
| `:c<column>` | Jump to column (e.g., `:cA`, `:cB`, `:cAA`, `:c1`) |

### Navigation Commands

**Column Navigation (Dual Approach):**

**Method 1: `g` suffix (fast, vim-like, works for most columns):**
- `5g` → row 5
- `Bg` → column B
- `B4g` → cell B4
- ⚠️ **Limitation:** Doesn't work for columns A, I, O, G (keys reserved for other commands)

**Method 2: `:c` command (explicit, works for all columns):**
- `:cA` → column A (reliable for reserved letters)
- `:cB` → column B
- `:cAA` → column AA
- `:c1` → column 1 (numeric: 1=A, 2=B, 27=AA)
- Case-insensitive: `:ca`, `:cA`, `:CA` all work

**Recommended usage:**
- Row jumps: Always use `5g`, `gg`, `G`
- Column jumps (B-Y): Use `Bg` for speed
- Column jumps (A, I, O, G): Use `:cA`, `:cI`, `:cO`, `:cG`

**Rationale:** Reserve `:` for operations/commands, but provide `:c` as a reliable escape hatch for columns whose letters conflict with other commands.

### Range Operations

**Row ranges (vim-style):**
- `:5,10d` - delete rows 5-10
- `:5,10y` - yank rows 5-10
- `:%d` - delete all rows
- `:.d` - delete current row
- `:.,+5d` - delete current row and next 5
- `:$d` - delete last row

**Column ranges (comma separator):**
- `:B,Dd` - delete columns B through D
- `:B,Dy` - yank columns B through D
- `:B,D` alone - ERROR: "Incomplete command. Use :B,Dd to delete"

**Important:** Don't overcomplicate. Stick to these two patterns.

---

## Vim Keybinding Reference

### Motions (Navigation)

| Key | Action |
|-----|--------|
| `h` `j` `k` `l` | Move left/down/up/right |
| `Arrow keys` | Move left/down/up/right |
| `gg` | First non-header row (row 1 if header ON, row 0 if header OFF) |
| `G` | Last row |
| `5g` | Go to row 5 (NEW: replaces old `:5`) |
| `Bg` | Go to column B (NEW: replaces old `:B`) |
| `A4g` | Go to cell A4 (NEW: replaces old `:A5`) |
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

**Note:** `g` is reserved for goto/movement operations ONLY. No other uses.

### Cell Editing

| Key | Action |
|-----|--------|
| `i` | Enter Insert mode (quick edit, single-line) |
| `a` | Enter Insert mode at end of cell |
| `I` | Enter Insert mode at start of cell |
| `A` | Enter Insert mode at end of cell (same as `a`) |
| `s` | Substitute cell (clear + edit) |
| `m` | Enter Magnifier mode (full vim editor, multi-line) |
| `x` | Delete cell content |
| `Delete` | Clear cell content (stay in Normal mode) |

**In Insert Mode:**
- Type characters - insert at cursor
- `Backspace` / `Ctrl+h` - delete before cursor
- `Delete` - delete at cursor
- `Ctrl+w` - delete word backward
- `Ctrl+u` - delete to start of cell
- `Home` / `End` - move to start/end
- `Left` / `Right` - move cursor
- `Enter` - commit edit, move down
- `Shift+Enter` - commit edit, move up
- `Tab` - commit edit, move right
- `Shift+Tab` - commit edit, move left
- `Esc` - cancel edit, stay in place

**In Magnifier Mode:**
- Full vim editing (multi-line, word motion, operators)
- `:w` - save cell content
- `:wq` or `ZZ` - save and close
- `:q!` - close without saving
- `Ctrl+h/j/k/l` - navigate to adjacent cells (prompts to save if dirty)

### Row Operators

| Key | Action |
|-----|--------|
| `o` | Insert row below, enter Insert mode |
| `O` | Insert row above, enter Insert mode |
| `dd` | Delete row (stores in row buffer) |
| `yy` | Yank row (stores in row buffer) |
| `p` | Paste row below |
| `P` | Paste row above |
| `cc` | Clear row and enter Insert mode |
| `5dd` | Delete 5 rows (count prefix) |
| `5yy` | Yank 5 rows (count prefix) |

### Column Operators (Comma Leader)

**Three-tier system:** Cell (`x`) → Row (`dd`) → Column (`,dd`)

| Key | Action |
|-----|--------|
| `,o` | Insert column right |
| `,O` | Insert column left |
| `,dd` | Delete column (stores in column buffer) |
| `,yy` | Yank column (includes header, stores in column buffer) |
| `5,dd` | Delete 5 columns (count prefix) |
| `5,yy` | Yank 5 columns (count prefix) |
| `,p` | Paste column(s) right (cursor moves to new column) |
| `,P` | Paste column(s) left (cursor moves to new column) |

**Behavior:**
- `,yy` yanks entire column including header (row 0)
- `,p` pastes column(s), cursor moves to new column
- `,o`/`,O` creates column with generic header (e.g., "Column D")
- Comma leader waits silently for next key (standard vim behavior)

**No `,h` for header editing:** Header row is just row 0. Navigate to row 0 with `k` from row 1 (or `gg` when header mode OFF) and use `i` to edit like any other cell.

### Dual Clipboard Buffers

Two separate buffers — one for rows, one for columns. Each buffer only responds to its own paste command:

| Operation | Buffer Used | Result |
|-----------|-------------|--------|
| `yy` then `p` | Row buffer | Paste row below |
| `,yy` then `,p` | Column buffer | Paste column |
| `yy` then `,p` | Column buffer (empty) | Message: "Nothing to paste" |
| `,yy` then `p` | Row buffer (empty) | Message: "Nothing to paste" |
| Visual selection `y` then `p` | Row buffer | Paste rectangular region |

**No cross-buffer pasting:** `p` always uses the row buffer, `,p` always uses the column buffer. If the target buffer is empty, a transient message "Nothing to paste" is shown.

### Visual Mode

| Key | Mode | Selection |
|-----|------|-----------|
| `v` | Visual Block | Rectangular region (bounding box) |
| `V` | Visual Line | Whole rows |
| `,v` | Visual Column | Whole columns |

**In Visual Mode:**
- `h` `j` `k` `l` - expand/contract selection
- `d` - delete selection (clears cells for regions, removes rows/columns for line modes)
- `y` - yank selection (stores in row buffer for `V`/`v`, column buffer for `,v`)
- `c` - change selection (clear + insert)
- `p` - paste over selection (overwrites existing, adds rows/cols if needed)
- `Esc` - exit visual mode
- `gv` - re-select last selection

**Notes:**
- Visual Block (`v`): Always creates rectangular bounding box
- No `Ctrl+v` (redundant with `v`)
- Delete cell region clears cells, preserves structure
- Delete whole rows/columns removes them entirely

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
| `"+,yy` | Yank column to system clipboard |
| `"+p` | Paste from system clipboard |

### Undo/Redo

| Key | Action |
|-----|--------|
| `u` | Undo last operation |
| `Ctrl+r` | Redo last undone operation |
| `.` | Repeat last change |

**Undo behavior:**
- Per-file undo history (preserved across file switches)
- History maintained in RAM during session
- `:w` does NOT clear undo history
- Max undo levels: configurable (default 1000)

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
| `:files` | Show file menu with numbers to select |

### Help

| Key | Action |
|-----|--------|
| `?` | Quick reference overlay (summary of common commands) |
| `:help` | Full scrollable help buffer (comprehensive, searchable with `/`) |

### Other

| Key | Action |
|-----|--------|
| `Esc` | Cancel/return to Normal mode |
| `:ht` | Toggle header mode for current file |

---

## Configuration System

**Philosophy:** Zero configuration by default. Optional config for power users.

### Config File Locations

1. **Global config:** `~/.config/lazycsv/config.toml`
2. **Per-directory config:** `./.lazycsv/config.toml` (overrides global)

### Config File Format (TOML)

```toml
# ~/.config/lazycsv/config.toml example

[defaults]
delimiter = ","           # Default delimiter for new files
header_mode = true        # Default header mode (ON/OFF)
undo_limit = 1000         # Max undo history per file

[colors]
header_bg = "blue"        # Header row background
cursor_fg = "yellow"      # Cursor foreground
dirty_indicator = "red"   # Dirty file indicator color

[keybindings]
# Full keybind remapping support (advanced users)
quit = ":q"
save_all = ":w"
# ... etc
```

### Customization Levels

- **None:** Works perfectly out of the box
- **Basic:** Colors, default delimiter
- **Advanced:** Full keybind remapping, undo limits, UI tweaks

---

## CLI Options

**Simplified:** No `--delimiter` or `--no-headers` flags. Use commands instead.

### Usage

```bash
lazycsv [FILES...]
```

### Examples

```bash
# Open current directory (scans for CSV files)
lazycsv

# Open specific file
lazycsv data.csv

# Open multiple files
lazycsv customers.csv orders.csv

# Open a directory
lazycsv ./data/
```

### Delimiter Handling

- **Default:** Comma `,`
- **Change:** Use `:delim X` command in-app (e.g., `:delim ;` for semicolon)
- **Persistence:** Session-only (not saved to disk)
- **Per-file:** Each file remembers its delimiter during session

### Header Handling

- **Default:** Header mode ON (row 0 is header)
- **Toggle:** Use `:ht` command in-app
- **Persistence:** Session-only (not saved to disk)
- **Per-file:** Each file remembers its header mode during session

**Rationale:** Simpler CLI, more discoverable in-app commands.

---

## Status Line Design

**Minimalist approach:** Mode + row + column only. No cell preview.

### Format

```
MODE                                                               ROW,COL
```

### Examples

```
NORMAL                                                             5,C
INSERT                                                             12,AA
MAGNIFIER                                                          3,B
VISUAL                                                             1-5,A-C
```

### Empty File Display

- **0 rows, 0 columns:** `NORMAL                                                             0,0`
- **0 rows, N columns:** `NORMAL                                                             0,B`
- **Header-only (header mode ON):** Cursor on row 0, shows `0,A` (row 0 = header row)

**Note:** No cell preview. Users can see cell content directly in the table.

---

## Empty Document Handling

### Completely Empty File (0 bytes)

**Opening empty.csv:**
- Status: `NORMAL                                                             0,0`
- Message: "Empty file (0 rows, 0 columns). Press 'o' to insert first row, ':new' to initialize, 'q' to quit"

**User presses `o`:**
1. Creates 1 column with header "Column 1"
2. Header mode auto-enabled
3. Inserts row 1 (first data row)
4. Cursor at row 1, col 0
5. Enters Insert mode automatically

**User uses `:new Name,Email,Phone`:**
1. Creates 3 columns with those headers
2. Header mode auto-enabled
3. 0 data rows
4. Cursor on row 0 (header row)
5. Returns to Normal mode
6. Can press `i` to edit headers or `o` to add first row

### Header-Only File (0 rows, N columns)

**Opening headers.csv (headers: Name,Age,City | 0 data rows):**
- Header mode ON by default
- Cursor on row 0 (header row)
- Status: `NORMAL                                                             0,A`

**Available operations:**
- ✅ `h`, `l`, `0`, `$` - navigate between columns (on header row)
- ✅ `i`, `a`, `I`, `A`, `s` - edit header cells (header is just row 0)
- ✅ `,dd` - delete column
- ✅ `,o`, `,O` - insert new column
- ✅ `o`, `O` - insert first data row (cursor moves to row 1)
- ✅ `dd` - delete header row (auto-toggles header mode OFF)
- ❌ `j`, `k`, `gg`, `G` - no-op or show message "No data rows"

### Delete Last Row Workflow

**File has 1 data row, delete with `dd`:**
1. Row deleted
2. Cursor automatically moves to row 0 (header row)
3. Header mode remains ON
4. Status shows `NORMAL                                                             0,A`

**From header row:**
- `h`/`l`/`0`/`$` - navigate columns
- `i`/`a`/`I`/`A`/`s` - edit header cells (header is just row 0, no special keybinding)
- `,dd` - delete column (if only 1 column, creates empty 0x0 file)
- `dd` - delete header row (auto-toggles header mode OFF, row 1 becomes new row 0)
- `o` or `O` - insert first data row (cursor moves to row 1)
- `j` - move to row 1 (if it exists), otherwise no-op
- `k` - no-op (can't go above header)

### Delete Last Column Workflow

**Allow 0-column files (full support, no warning):**
- Deleting last column with `,dd` is allowed
- Creates 0-column document
- Can use `,o` to add new column back
- Status shows `NORMAL                                                             1,0`

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
NORMAL                                                          3,C
```

**Completed changes:**
- [x] No box borders - just horizontal rules to separate sections
- [x] Current row indicator: Single `>` in row number column
- [x] Current column: Highlighted letter in header row
- [x] Top bar: Filename left, row/total right
- [x] File list: Single line, minimal chrome
- [x] Status line: Mode + position (simplified, no cell preview)
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

- [x] Mode enum variants: `Normal, Insert, Magnifier, Visual, Command`
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
- 427 tests passing (271 lib + 156 integration)
- 64 comprehensive Insert mode tests
- Zero clippy warnings
- Zero compiler warnings

---

## v0.4.1 - Persistence & Edge Cases

*Save files, fix edge cases, header toggle system, simplified navigation*

### Critical Edge Case Fixes ✅ COMPLETE

**Empty Document Handling:** ✅ COMPLETE
- [x] `App::new` handles documents with 0 rows gracefully ✅
- [x] Navigation (`gg`, `G`, `j`, `k`) handles 0-row documents ✅
- [x] Status line shows correct position for empty documents ✅
- [x] `o` command works on empty documents ✅
- [x] 0-column documents supported ✅
- [x] 0-row documents supported (delete last row → moves to header) ✅
- [x] Comprehensive edge case test suite (7 tests in empty_document_test.rs) ✅

**Header Row as Row 0:**
- [x] Refactor `Document` - header row is `rows[0]`, not separate field
- [x] Add `header_mode: bool` flag to track header toggle state
- [x] Add `delimiter: char` field to Document
- [x] Update all Document methods to handle +1 offset for data rows
- [x] Add `Document::new()` helper for tests
- [x] Fix all test files to use new Document structure
- [x] Fix `yy` command bug (was yanking header instead of data row)
- [x] Session tracks header mode per-file (in RAM only) via `HashMap<PathBuf, bool>`
- [x] Session tracks delimiter per-file via `HashMap<PathBuf, char>`
- [x] Add `get_header_mode()`, `set_header_mode()`, `get_delimiter()`, `set_delimiter()` to Session
- [x] Implement `:ht` command to toggle header mode
- [x] When header mode ON: style row 0, `gg` goes to row 1
- [x] When header mode OFF: no special styling, `gg` goes to row 0
- [x] Deleting row 0 with `dd` auto-toggles header mode OFF
- [x] Default all files to header mode ON
- [x] Refactor all navigation to use absolute row indices (0-based including header)
- [x] Cannot navigate to row 0 when header_mode ON (k from row 1 stops at row 1)
- [x] Status line shows absolute row numbers (0 for header, 1+ for data)
- [x] Fix all 427 tests to work with absolute row indexing

### Simplified Navigation ✅ COMPLETE (`:c` command only)

**Row Navigation - ✅ COMPLETE:**
- [x] Remove `:5` (row jump) - replaced with `5g` ✅
- [x] Add `5g` → jump to row 5 ✅
- [x] `gg` → first non-header row ✅
- [x] `G` → last row ✅
- [x] 9 tests in `simplified_navigation_test.rs` ✅

**Column Navigation - ✅ COMPLETE (`:c` command only):**
- [x] `:c<column>` command for all column jumps ✅
- [x] `:cA` → jump to column A ✅
- [x] `:cB` → jump to column B ✅
- [x] `:cAA` → jump to column AA (multi-letter columns) ✅
- [x] `:c1` → jump to column 1 (numeric alternative: 1=A, 2=B, 27=AA) ✅
- [x] Case-insensitive: `:ca`, `:cA`, `:CA` all work ✅
- [x] Proper error messages for invalid columns ✅
- [x] 8 tests for `:c` command ✅
- [x] **REMOVED** `<LETTER>g` syntax (Bg, Cg, etc.) - too confusing with reserved keys ✅

**Navigation Summary:**
- **Row jumps:** Use `5g`, `gg`, `G` (vim-like, fast, no conflicts)
- **Column jumps:** Use `:cA`, `:cB`, `:cZ`, `:cAA` (explicit, consistent, works for all columns)
- No more dual approach - one clear way to jump to columns
- 497 tests passing → 507 tests passing ✅

### File Persistence ✅

**Commands Implemented:**
| Command | Action |
|---------|--------|
| `:w` | Write current file only ✅ |
| `:W` | Write all dirty files ✅ |
| `:wq` | Write current file and quit ✅ |
| `:Wq` | Write all dirty files and quit ✅ |
| `:q` | Quit (fails if current file dirty) ✅ |
| `:q!` | Force quit (discard all changes) ✅ |

**Multi-File Dirty Tracking:**
- [x] Session tracks dirty files in `HashSet<PathBuf>` ✅
- [x] Session caches dirty `Document` instances in `HashMap<PathBuf, Document>` ✅
- [x] When switching files: use cache if dirty, reload from disk if clean ✅
- [x] After `:w` / `:W`: remove from cache (reload fresh next time) ✅
- [x] `:q` checks current file for dirty state, blocks if unsaved ✅
- [x] File switcher shows `*` next to dirty files: `customers.csv* | orders.csv` ✅

### Range Operations ✅ ROWS COMPLETE

**Row range syntax - ✅ COMPLETE:**
- [x] `:5,10d` - delete rows 5-10 ✅
- [x] `:5,10y` - yank rows 5-10 ✅
- [x] `:%d` - delete all data rows ✅
- [x] `:%y` - yank all data rows ✅
- [x] `:.d` - delete current row ✅
- [x] `:.y` - yank current row ✅
- [x] `:$d` - delete last row ✅
- [x] `:$y` - yank last row ✅
- [x] 10 comprehensive tests in `range_operations_test.rs` ✅

**Column range syntax - ✅ COMPLETE:**
- [x] `:B,Dd` - delete columns B through D ✅
- [x] `:B,Dy` - yank columns B through D ✅
- [x] `:C,Cd` - delete single column C ✅
- [x] `:B,D` alone - ERROR: "Incomplete command. Use :B,Dd to delete" ✅
- [x] 10 comprehensive tests in `column_range_operations_test.rs` ✅
- [x] 517 tests passing (507 + 10 new) ✅

### New Commands

**`:delim X` - Set Delimiter:** ✅ COMPLETE
- [x] `:delim ;` sets delimiter to semicolon for current file ✅
- [x] Reloads file with new delimiter automatically ✅
- [x] Setting is session-only (not persisted to disk) ✅
- [x] Default delimiter is `,` (comma) ✅
- [x] Each file remembers its delimiter during session ✅

**`:new` - Create New CSV:** ✅ COMPLETE
- [x] `:new Name,Age,City` creates CSV with those headers (0 data rows) ✅
- [x] `:new` creates CSV with 1 column "Column 1" (0 data rows) ✅
- [x] Header mode auto-enabled ✅
- [x] File marked as dirty (unsaved) ✅
- [x] Preserves current delimiter setting ✅

**`:files` - File Menu:** ✅ COMPLETE
- [x] Cursor-based navigation with j/k or arrow keys ✅
- [x] Type to filter file list (case-insensitive) ✅
- [x] Enter to select file ✅
- [x] Visual cursor indicator `>` ✅
- [x] Shows dirty indicator `*` ✅

### Implementation Steps

**File: `src/csv/writer.rs` (new file)** ✅ COMPLETE
- [x] Create CSV writer module ✅
- [x] Implement `write_csv_atomic(document: &Document, path: &Path, delimiter: char) -> Result<()>` ✅
  - Write to temp file first ✅
  - Atomically rename to target path ✅
  - Handle CSV escaping (quotes, commas, newlines) ✅
  - Preserves original on write failure ✅

**File: `src/csv/document.rs`** ✅ COMPLETE
- [x] Refactor: header row is `rows[0]`, not separate field ✅
- [x] Add `header_mode: bool` field ✅
- [x] Add `delimiter: char` field (default: `,`) ✅
- [x] Update all methods to handle header row as row 0 ✅
- [x] Add `Document::new()` helper for tests ✅
- [x] Add `toggle_header_mode(&mut self)` method ✅
- [x] Add `delete_last_row_moves_to_header()` logic ✅

**File: `src/session/mod.rs`** ✅ COMPLETE
- [x] Add `header_modes: HashMap<PathBuf, bool>` field (track per-file) ✅
- [x] Add `delimiters: HashMap<PathBuf, char>` field (track per-file) ✅
- [x] Add `get_header_mode(&self) -> bool` method (default: true) ✅
- [x] Add `set_header_mode(&mut self, mode: bool)` method ✅
- [x] Add `get_delimiter(&self, file: &PathBuf) -> char` method (default: ',') ✅
- [x] Add `set_delimiter(&mut self, file: PathBuf, delimiter: char)` method ✅
- [x] Add `dirty_files: HashSet<PathBuf>` field ✅
- [x] Add `document_cache: HashMap<PathBuf, Document>` field ✅
- [x] Add `mark_dirty(&mut self, path: &Path)` method ✅
- [x] Add `mark_clean(&mut self, path: &Path)` method ✅
- [x] Add `is_dirty(&self, path: &Path) -> bool` method ✅
- [x] Add `cache_document(&mut self, path: PathBuf, doc: Document)` method ✅
- [x] Add `get_cached_document(&self, path: &Path) -> Option<&Document>` method ✅
- [x] Add `remove_from_cache(&mut self, path: &Path)` method ✅
- [x] Add `is_current_file_dirty()`, `has_any_dirty_files()`, `get_dirty_files()` methods ✅
- [x] Add `clear_cache()` method ✅

**File: `src/app/mod.rs`** ✅ COMPLETE
- [x] Add `save_current_file(&mut self) -> Result<PathBuf>` method (saves current file) ✅
- [x] Add `save_all_files(&mut self) -> Result<Vec<PathBuf>>` method (saves all dirty files) ✅
- [x] Cursor positioning for empty documents handled gracefully ✅
- [x] Handle `gg` differently based on header mode ✅

**File: `src/input/handler.rs`** ✅ COMPLETE  
- [x] Fixed case-sensitive command matching (`:W` vs `:w`) ✅
- [x] Add `:w` command handler (saves current file only) ✅
- [x] Add `:W` command handler (saves all dirty files) ✅
- [x] Add `:wq` command handler (save current and quit) ✅
- [x] Add `:Wq` command handler (save all and quit) ✅
- [x] Modify `:q` handler (check current file for dirty state) ✅
- [x] Add `:q!` handler (quit immediately, clear cache) ✅
- [x] Add `:ht` command handler (toggle header mode) ✅
- [x] Add `:delim X` command handler (change delimiter with auto-reload) ✅
- [x] Add `:new [headers]` command handler (create new CSV) ✅
- [x] Add `:files` command handler (file picker with cursor navigation) ✅
- [x] `:c` command works for column navigation (no old version to remove) ✅
- [x] Old `:5`, `:B`, `:A5` navigation removed ✅
- [x] `5g` row jump implemented ✅
- [x] `Bg` column jump **removed** (conflicts with reserved keys, use `:cB` instead) ✅
- [x] Range operation handlers implemented (`:5,10d`, `:B,Dd`) ✅

**File: `src/ui/status.rs`** ✅ COMPLETE
- [x] Status line shows mode + row + column ✅
- [x] `render_file_switcher()` shows `*` for dirty files ✅

**File: `src/ui/table.rs`** ✅ COMPLETE
- [x] Header row styling applied when `header_mode == true` ✅
- [x] Row 0 visually distinct when header mode ON ✅

### Tests to Add

**Edge Cases (`tests/empty_document_test.rs`):** ✅ COMPLETE (7 tests)
- [x] `test_empty_file_0_bytes` ✅
- [x] `test_header_only_file_no_data` ✅
- [x] `test_app_new_with_empty_document_0_cols` ✅
- [x] `test_app_new_with_header_only_document` ✅
- [x] `test_single_row_single_column` ✅
- [x] `test_navigation_with_header_only_file` ✅
- [x] `test_delete_last_data_row_moves_to_header` ✅

**Header Toggle:** ✅ TESTED THROUGHOUT
- [x] Header mode toggle tested in `empty_document_test.rs` ✅
- [x] Header mode behavior tested in `new_command_test.rs` ✅
- [x] `gg` behavior with header mode tested in multiple files ✅
- [x] `dd` on row 0 behavior tested implicitly ✅
- [x] Header mode defaults tested throughout ✅

**Navigation (`tests/simplified_navigation_test.rs`):** ✅ COMPLETE (17 tests)
- [x] `test_5g_jumps_to_row_5` ✅
- [x] `test_bg_removed_use_c_command_instead` (tests old `Bg` syntax removed) ✅
- [x] `test_cell_jump_removed_use_c_command_and_row_jump` ✅
- [x] `test_old_colon_number_navigation_removed` (tests `:5` removed) ✅
- [x] Multiple `:c` command tests (10+ tests) ✅

**Persistence (`tests/persistence_test.rs`):** ✅ COMPLETE (8 tests)
- [x] `test_w_saves_current_file` ✅
- [x] `test_W_saves_all_dirty_files` ✅
- [x] `test_wq_saves_and_quits` ✅
- [x] `test_q_blocks_if_dirty` ✅
- [x] `test_q_succeeds_if_clean` ✅
- [x] `test_q_bang_discards_changes` ✅
- [x] `test_csv_writer_escapes_quotes` ✅
- [x] `test_csv_writer_escapes_commas` ✅
- [x] Dirty indicator tested via integration (file switcher shows `*`) ✅
- [x] File switch preserves edits (via document cache implementation) ✅
- [x] Save removes from cache (via cache management implementation) ✅
- [x] CSV writer atomic write (via temp file implementation) ✅

**Range Operations (`tests/range_operations_test.rs`):** ✅ COMPLETE
- [x] `test_delete_row_range_5_to_10` ✅
- [x] `test_delete_all_rows_percent_d` ✅
- [x] `test_delete_current_row_dot_d` ✅
- [x] `test_delete_last_row_dollar_d` ✅
- [x] `test_yank_row_range_5_to_10` ✅
- [x] `test_yank_all_rows_percent_y` ✅
- [x] `test_yank_current_row_dot_y` ✅
- [x] `test_invalid_range_start_greater_than_end` ✅
- [x] `test_delete_range_with_row_zero` ✅
- [x] `test_delete_out_of_bounds_range` ✅
- [x] `test_delete_column_range_b_to_d` ✅
- [x] `test_yank_column_range_b_to_d` ✅
- [x] `test_delete_single_column_c` ✅
- [x] `test_delete_all_columns_a_to_e` ✅
- [x] `test_column_range_invalid_start_after_end` ✅
- [x] `test_column_range_out_of_bounds` ✅
- [x] `test_column_range_both_out_of_bounds` ✅
- [x] `test_column_range_multi_letter_columns` ✅
- [x] `test_column_range_cursor_adjustment_after_delete` ✅
- [x] `test_incomplete_column_range_shows_error` ✅

**Commands (`tests/commands_test.rs`):** ✅ COMPLETE
- [x] `test_delim_command_changes_delimiter` ✅ (6 tests in delimiter_test.rs)
- [x] `test_new_command_with_headers` ✅ (8 tests in new_command_test.rs)
- [x] `test_new_command_default` ✅
- [x] `test_files_command_shows_menu` ✅ (12 tests in files_command_test.rs)

### Acceptance Criteria

**File Persistence:** ✅ COMPLETE
- [x] `:w` saves current file only ✅
- [x] `:W` saves all dirty files ✅
- [x] `:wq` saves current file and quits ✅
- [x] `:Wq` saves all dirty files and quits ✅
- [x] `:q` blocks if current file dirty ✅
- [x] `:q!` quits without saving ✅
- [x] File switcher shows `*` next to dirty files ✅
- [x] Switching files preserves unsaved edits (via cache) ✅
- [x] CSV output properly escapes special characters (quotes, commas, newlines) ✅
- [x] Atomic writes (temp file → rename) ✅

**Header System:** ✅ COMPLETE
- [x] `:ht` toggles header mode for current file ✅
- [x] Header mode ON: row 0 styled/frozen, `gg` → row 1 ✅
- [x] Header mode OFF: row 0 normal, `gg` → row 0 ✅
- [x] `dd` on row 0 toggles header mode OFF ✅
- [x] All 489 tests passing ✅
- [x] No clippy warnings ✅

**Simplified Navigation:** ✅ COMPLETE
- [x] `5g` jumps to row 5 (old `:5` removed) ✅
- [x] `:c<column>` command for column jumps ✅
- [x] **REMOVED** `<LETTER>g` syntax (Bg, Cg, etc.) ✅
- [x] 497 tests passing → 507 tests passing ✅
- [x] No clippy warnings ✅

**Range Operations:** ✅ COMPLETE (ALL OPERATIONS)
- [x] `:5,10d` deletes rows 5-10 ✅
- [x] `:5,10y` yanks rows 5-10 ✅
- [x] `:%d` deletes all data rows ✅
- [x] `:%y` yanks all data rows ✅
- [x] `:.d` deletes current row ✅
- [x] `:.y` yanks current row ✅
- [x] `:$d` deletes last row ✅
- [x] `:$y` yanks last row ✅
- [x] `:B,Dd` deletes columns B through D ✅
- [x] `:B,Dy` yanks columns B through D ✅
- [x] 517 tests passing (20 range operation tests) ✅
- [x] Zero clippy warnings ✅

**New Commands:** ✅ COMPLETE
- [x] `:delim ;` changes delimiter to semicolon ✅
- [x] `:new Name,Age` creates new CSV with those headers ✅
- [x] `:files` shows file menu with cursor navigation ✅

**Ready for release:**
- ✅ 517 tests passing
- ✅ Zero compiler warnings  
- ✅ 1 pre-existing clippy warning (unrelated)
- ✅ All acceptance criteria met

---

## v0.5.0 - Column Operations & Visual Mode

*Full column manipulation with comma leader, visual selections, dual clipboard buffers*

### Column Operations (Comma Leader)

| Key | Action |
|-----|--------|
| `,o` | Insert column right |
| `,O` | Insert column left |
| `,dd` | Delete column (stores in column buffer) |
| `,yy` | Yank column (includes header, stores in column buffer) |
| `5,dd` | Delete 5 columns (count prefix) |
| `5,yy` | Yank 5 columns (count prefix) |
| `,p` | Paste column(s) right (cursor moves to new column) |
| `,P` | Paste column(s) left (cursor moves to new column) |

**Behavior:**
- `,yy` yanks entire column including header (row 0)
- `,p` pastes column(s), cursor moves to new column
- `,o`/`,O` creates column with generic header (e.g., "Column D")
- Comma leader is silent (no visual feedback, standard vim behavior)
- No `,h` for header editing - just navigate to row 0 and use `i`

### Visual Mode (Simplified to 3 Modes)

| Key | Mode | Selection |
|-----|------|-----------|
| `v` | Visual Block | Rectangular region (bounding box) |
| `V` | Visual Line | Whole rows |
| `,v` | Visual Column | Whole columns |

**Operations in Visual mode:**
- `d` - delete selection (clears cells for regions, removes rows/columns for line modes)
- `y` - yank selection (stores in row buffer for `V`, column buffer for `,v`, row buffer for `v`)
- `c` - change selection (clear + insert)
- `p` - paste over selection (overwrites existing, adds rows/cols if needed)
- `Esc` - exit Visual mode
- `gv` - re-select last selection

**Notes:**
- Visual Block (`v`): Always creates rectangular bounding box (no S-shape free selection)
- No `Ctrl+v` (redundant with `v`)
- Delete cell region clears cells, preserves structure
- Delete whole rows/columns removes them entirely

### Dual Clipboard Buffers

**Two separate buffers — row buffer and column buffer:**
- `yy`, `dd`, `V`+`y`, `v`+`y` store in the **row buffer**
- `,yy`, `,dd`, `,v`+`y` store in the **column buffer**
- `p`/`P` always paste from the **row buffer** — if empty, shows "Nothing to paste"
- `,p`/`,P` always paste from the **column buffer** — if empty, shows "Nothing to paste"
- No cross-buffer pasting or transposing

### Count Prefixes

| Key | Action |
|-----|--------|
| `5dd` | Delete 5 rows |
| `5yy` | Yank 5 rows |
| `5,dd` | Delete 5 columns |
| `5,yy` | Yank 5 columns |
| `P` | Paste above current row |
| `cc` | Clear row and enter Insert mode |

### Column Reordering

**Command approach:**
- `:B,D m A` - move columns B-D to after column A
- `:C m $` - move column C to end
- `:F m 0` - move column F to beginning

### Implementation Steps

**File: `src/clipboard/mod.rs` (new file)**
- [x] Create clipboard module with dual buffers
- [x] Define `RowBuffer` struct (stores row data: `Vec<Vec<String>>`)
- [x] Define `ColumnBuffer` struct (stores column data including header: `Vec<Vec<String>>`)
- [x] Define `DualClipboard` struct containing both buffers
- [x] Implement `has_row_data()` and `has_column_data()` methods
- [x] No transpose or cross-buffer logic

**File: `src/input/actions.rs`**
- [x] Add `LeaderCommand` enum for comma sequences
- [x] Track comma leader state in InputState
- [x] Remove semicolon leader references

**File: `src/input/handler.rs`**
- [x] Add comma (`,`) handler to enter leader mode
- [x] Add leader command handlers: `,o`, `,O`, `,dd`, `,yy`, `,p`, `,P`
- [x] Add count prefix support for `dd`, `yy`, `,dd`, and `,yy`
- [ ] Add `V` handler to enter Visual Line mode
- [ ] Add `v` handler to enter Visual Block mode
- [ ] Add `,v` handler to enter Visual Column mode
- [ ] Add `handle_visual_mode()` function
- [x] Add `P` handler for paste above
- [x] Add `cc` handler
- [ ] Add `gv` handler for re-select
- [x] Update `yy`, `dd` to store in row buffer
- [x] Update `,yy`, `,dd` to store in column buffer
- [x] `p`/`P` paste from row buffer only (show "Nothing to paste" if empty)
- [x] `,p`/`,P` paste from column buffer only (show "Nothing to paste" if empty)

**File: `src/csv/document.rs`**
- [x] Add `insert_column(&mut self, at: ColIndex, header: String)` method
- [x] Add `delete_column(&mut self, at: ColIndex) -> Vec<String>` method
- [x] Add `get_column(&self, col: ColIndex) -> Vec<String>` method (includes row 0 header)
- [ ] Add `move_columns(&mut self, from: ColIndex, to: ColIndex, count: usize)` method
- [x] Add `delete_rows(&mut self, start: RowIndex, end: RowIndex)` method
- [x] Add `get_rows(&self, start: RowIndex, end: RowIndex)` method

**File: `src/app/mod.rs`**
- [x] Replace `row_clipboard` with `DualClipboard` (row buffer + column buffer)
- [ ] Add `visual_anchor: Option<(RowIndex, ColIndex)>` field
- [ ] Add `visual_mode: Option<VisualMode>` field (Block, Line, Column)
- [ ] Add `last_visual_selection: Option<VisualSelection>` field

**File: `src/ui/table.rs`**
- [ ] Highlight visual selections (different styles for Block/Line/Column)
- [ ] Different style for visual selection vs cursor

**File: `src/ui/status.rs`**
- [ ] Show `VISUAL`, `VISUAL LINE`, `VISUAL COLUMN` mode indicators
- [ ] Keep simplified format: mode + row + column

### Tests to Add

**Comma Leader (`tests/dual_clipboard_test.rs`):**
- [x] `test_comma_leader_detection` (via `test_comma_cancelled_with_esc`)
- [x] `test_comma_o_inserts_column_right`
- [x] `test_comma_O_inserts_column_left`
- [x] `test_comma_dd_deletes_column`
- [x] `test_comma_yy_yanks_column_with_header`
- [x] `test_comma_p_pastes_column_right`
- [x] `test_comma_P_pastes_column_left`
- [ ] `test_column_reorder_command`
- [ ] `test_move_column_to_beginning`
- [ ] `test_move_column_to_end`
- [ ] `test_no_comma_h_keybinding` (verify ,h doesn't exist)

**Dual Clipboard (`tests/dual_clipboard_test.rs`):**
- [x] `test_yy_then_p_pastes_row_from_row_buffer` (via existing insert_mode_test + `test_dd_then_capital_p_round_trip`)
- [x] `test_comma_yy_then_comma_p_pastes_column_from_column_buffer` (via `test_comma_dd_then_comma_p_round_trip`)
- [x] `test_yy_then_comma_p_shows_nothing_to_paste` (column buffer empty)
- [x] `test_comma_yy_then_p_shows_nothing_to_paste` (row buffer empty)
- [x] `test_row_and_column_buffers_independent` (both can hold data simultaneously)
- [x] `test_dd_stores_in_row_buffer`
- [x] `test_comma_dd_stores_in_column_buffer`

**Visual Mode (`tests/visual_mode_test.rs`):**
- [ ] `test_v_enters_visual_block`
- [ ] `test_V_enters_visual_line`
- [ ] `test_comma_v_enters_visual_column`
- [ ] `test_visual_block_rectangular_bounding_box`
- [ ] `test_visual_d_deletes_selection`
- [ ] `test_visual_d_clears_cells_preserves_structure`
- [ ] `test_visual_y_yanks_selection`
- [ ] `test_visual_p_overwrites_and_adds_if_needed`
- [ ] `test_gv_reselects`

**Count Prefixes (`tests/dual_clipboard_test.rs`):**
- [x] `test_5dd_deletes_5_rows`
- [x] `test_5yy_yanks_5_rows`
- [x] `test_P_pastes_above` (via `test_capital_p_pastes_row_above`)
- [x] `test_cc_clears_row_enters_insert` (via `test_cc_clears_row_enters_insert`)

### Acceptance Criteria

- [ ] Comma leader works for column operations (not semicolon)
- [ ] `,dd` deletes column
- [ ] `,yy` yanks column including header (row 0)
- [ ] `,p` pastes column, cursor moves to new column
- [ ] `,o` inserts column with generic header
- [ ] No `,h` keybinding exists (headers edited via row 0 with `i`)
- [x] `5dd` deletes exactly 5 rows
- [ ] Visual modes work (`v`, `V`, `,v`) - 3 modes only
- [ ] Visual block creates rectangular bounding box
- [ ] Visual cell delete clears cells, preserves structure
- [ ] Visual row/column delete removes rows/columns entirely
- [ ] Visual operations work (`d`, `y`, `c`, `p`)
- [x] `P` pastes above
- [x] `cc` clears row, enters Insert
- [ ] `gv` re-selects
- [ ] Unified clipboard handles row/column/region
- [ ] Transpose operations work (`yy`+`,p`, `,yy`+`p`)
- [ ] Column reordering works (`:B,D m A`)
- [ ] All existing tests pass
- [ ] No clippy warnings

---

## v0.6.0 - Magnifier Mode

*Full vim editor for complex cell editing*

### Keybindings to Implement

| Key | Action |
|-----|--------|
| `m` | Open Magnifier on current cell |

**In Magnifier Mode:**
- Full vim editing (multi-line, word motion, operators)
- `:w` - Save cell content (update in-memory document)
- `:wq` or `ZZ` - Save and close Magnifier
- `:q!` - Close without saving
- `Ctrl+h/j/k/l` - Navigate to adjacent cells (prompts to save if dirty)

### Use Cases
- Editing JSON data in cells
- Multi-line descriptions or notes
- Complex text that needs vim power
- Large cell content (>100 chars)

### Insert vs Magnifier Decision Tree

| Scenario | Use Insert (`i`) | Use Magnifier (`m`) |
|----------|------------------|---------------------|
| Fix a typo | ✅ | Overkill |
| Replace entire cell | ✅ | Works |
| Multi-line content | ❌ (single line only) | ✅ |
| Long text (>50 chars) | Awkward | ✅ |
| Complex vim editing (operators, macros) | ❌ | ✅ |

**Default behavior:** Always start with Insert mode. Manually upgrade to Magnifier when needed.

### Implementation Steps

**File: `src/magnifier/mod.rs` (new file)**
- [ ] Create magnifier module
- [ ] Implement vim buffer state
- [ ] Implement vim mode switching (Normal/Insert within magnifier)
- [ ] Implement vim motions: `h/j/k/l`, `w/b/e`, `0/$`, `gg/G`
- [ ] Implement vim operators: `dd`, `yy`, `p`, `i/a/o/O`, `x`, `s`
- [ ] Implement line-based editing
- [ ] Implement multi-line support

**File: `src/app/mod.rs`**
- [ ] Add `magnifier_state: Option<MagnifierState>` field
- [ ] Implement `open_magnifier(&mut self)` method
- [ ] Implement `close_magnifier(&mut self, save: bool)` method
- [ ] Handle `Ctrl+h/j/k/l` for cell navigation

**File: `src/input/handler.rs`**
- [ ] Add `m` handler to open magnifier (not `Enter`)
- [ ] Add `handle_magnifier_mode()` function
- [ ] Handle Ctrl+h/j/k/l for cell navigation in magnifier
- [ ] Handle `:w`, `:wq`, `:q!` commands in magnifier
- [ ] Prompt to save if navigating with dirty buffer

**File: `src/ui/magnifier.rs` (new file)**
- [ ] Render magnifier overlay (centered, 80% width/height)
- [ ] Show vim mode indicator (NORMAL/INSERT within magnifier)
- [ ] Show cursor position
- [ ] Show line numbers
- [ ] Syntax highlighting for common formats (future enhancement)

### Tests to Add

- [ ] `test_m_opens_magnifier`
- [ ] `test_magnifier_vim_motions`
- [ ] `test_magnifier_w_saves_cell_content`
- [ ] `test_magnifier_wq_saves_and_closes`
- [ ] `test_magnifier_q_bang_discards`
- [ ] `test_magnifier_ctrl_hjkl_navigates_cells`
- [ ] `test_magnifier_multiline_editing`
- [ ] `test_magnifier_vim_operators`
- [ ] `test_magnifier_dirty_prompt_on_navigation`

### Acceptance Criteria

- [ ] `m` opens magnifier for current cell (not `Enter`)
- [ ] Vim motions work in magnifier (`hjkl`, `w/b/e`, `0/$`, `gg/G`)
- [ ] Vim operators work (`dd`, `yy`, `p`, `x`, `s`, `i/a/o/O`)
- [ ] `:w` saves cell content
- [ ] `:wq` saves and closes
- [ ] `:q!` discards changes
- [ ] `Ctrl+h/j/k/l` navigate to adjacent cells
- [ ] Prompt to save if navigating with dirty buffer
- [ ] Multi-line editing works
- [ ] Line numbers shown
- [ ] Mode indicator shows NORMAL/INSERT within magnifier
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
- [ ] Support regex patterns

**File: `src/app/mod.rs`**
- [ ] Add `search_state: Option<SearchState>` field
- [ ] Add `search_pattern: Option<String>` field
- [ ] Add `search_matches: Vec<(RowIndex, ColIndex)>` field
- [ ] Add `current_match_index: usize` field

**File: `src/input/handler.rs`**
- [ ] Add `/` handler to enter Search mode
- [ ] Add `n`, `N`, `*` handlers
- [ ] Add `:noh` command handler

**File: `src/ui/table.rs`**
- [ ] Highlight matching cells
- [ ] Different style for current match vs other matches

### Tests to Add

- [ ] `test_slash_enters_search_mode`
- [ ] `test_n_moves_to_next_match`
- [ ] `test_N_moves_to_previous_match`
- [ ] `test_search_wraps_around`
- [ ] `test_asterisk_searches_current_cell`
- [ ] `test_noh_clears_highlighting`
- [ ] `test_search_case_insensitive`
- [ ] `test_search_regex_patterns`

### Acceptance Criteria

- [ ] `/` enters search mode
- [ ] `n` moves to next match with wrap-around
- [ ] `N` moves to previous match with wrap-around
- [ ] `*` searches current cell content
- [ ] `:noh` clears highlighting
- [ ] Search is case-insensitive by default
- [ ] Regex patterns supported
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

### Undo/Redo Behavior

**Granularity:**
- Single operations: edit cell (`i`), delete row (`dd`), insert row (`o`) = 1 undo step each
- Compound operations: `5dd` (delete 5 rows) = 1 undo step (NOT 5 separate steps)
- Visual mode operations: delete selection = 1 undo step
- Range operations: `:5,10d` = 1 undo step

**History Management:**
- Max undo levels: 1000 per file (configurable in `~/.config/lazycsv/config.toml`)
- `:w` saves file but PRESERVES undo history (don't clear on save)
- File switching preserves undo history per file (stored in session)
- Undo/redo only works within current file (can't undo across files)

**Limitations:**
- Cannot undo file switch
- Cannot undo `:w` (file write)
- Cannot undo `:q` (quit)

### Implementation Steps

**File: `src/history/mod.rs` (new file)**
- [ ] Create history module
- [ ] Define `EditCommand` enum (variants for all mutation types)
- [ ] Define `History` struct with undo/redo stacks
- [ ] Implement `push_command()` method
- [ ] Implement `undo()` method
- [ ] Implement `redo()` method
- [ ] Implement `clear_redo_stack()` on new command
- [ ] Respect max undo limit

**File: `src/app/mod.rs`**
- [ ] Add `history: History` field
- [ ] Record all mutations to history
- [ ] Add `last_edit_command: Option<EditCommand>` for dot command

**File: `src/session/mod.rs`**
- [ ] Store per-file history in `HashMap<PathBuf, History>`
- [ ] Preserve history across file switches

**File: `src/input/handler.rs`**
- [ ] Add `u` handler
- [ ] Add `Ctrl+r` handler
- [ ] Add `.` handler (repeat last edit)

### Tests to Add

- [ ] `test_u_undoes_cell_edit`
- [ ] `test_u_undoes_row_delete`
- [ ] `test_u_undoes_column_delete`
- [ ] `test_5dd_creates_single_undo_step`
- [ ] `test_ctrl_r_redoes`
- [ ] `test_dot_repeats_last_edit`
- [ ] `test_undo_limit_respected`
- [ ] `test_w_preserves_undo_history`
- [ ] `test_file_switch_preserves_history`
- [ ] `test_new_command_clears_redo_stack`

### Acceptance Criteria

- [ ] `u` undoes last operation
- [ ] `Ctrl+r` redoes last undone operation
- [ ] `.` repeats last edit
- [ ] Compound operations (`5dd`, visual mode, ranges) = single undo step
- [ ] History respects max size (1000 default, configurable)
- [ ] `:w` preserves undo history
- [ ] File switching preserves per-file history
- [ ] All existing tests pass
- [ ] No clippy warnings

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

### System Clipboard

| Key | Action |
|-----|--------|
| `"+yy` | Yank row to system clipboard |
| `"+,yy` | Yank column to system clipboard |
| `"+p` | Paste from system clipboard |

### Implementation Steps

**File: `src/transforms/mod.rs` (new file)**
- [ ] Create transforms module
- [ ] Implement `toggle_case()` function
- [ ] Implement `uppercase()` function
- [ ] Implement `lowercase()` function
- [ ] Implement `title_case()` function
- [ ] Implement `toggle_boolean()` function (yes↔no, true↔false, 1↔0)

**File: `src/csv/document.rs`**
- [ ] Add `swap_rows(&mut self, a: RowIndex, b: RowIndex)` method
- [ ] Add `apply_transform(&mut self, transform: TransformFn, pos: Position)` method

**File: `src/clipboard/mod.rs`**
- [ ] Add system clipboard integration
- [ ] Implement `copy_to_system()` method
- [ ] Implement `paste_from_system()` method

**File: `src/input/handler.rs`**
- [ ] Add `~` handler
- [ ] Add `gU`, `gu`, `g~`, `g.` handlers
- [ ] Add `gj`, `gk` handlers
- [ ] Add `"+yy`, `"+,yy`, `"+p` handlers

### Tests to Add

- [ ] `test_tilde_toggles_case`
- [ ] `test_gU_uppercases_cell`
- [ ] `test_gu_lowercases_cell`
- [ ] `test_g_tilde_title_cases_cell`
- [ ] `test_g_dot_toggles_boolean`
- [ ] `test_gj_swaps_row_below`
- [ ] `test_gk_swaps_row_above`
- [ ] `test_system_clipboard_yank_row`
- [ ] `test_system_clipboard_yank_column`
- [ ] `test_system_clipboard_paste`

### Acceptance Criteria

- [ ] All case transforms work (`~`, `gU`, `gu`, `g~`)
- [ ] Boolean toggle works (`g.`)
- [ ] Row swapping works (`gj`, `gk`)
- [ ] System clipboard works (`"+yy`, `"+,yy`, `"+p`)
- [ ] All existing tests pass
- [ ] No clippy warnings

---

## v1.0.0 - First Stable Release

*All core features working, stable command interface*

### Pre-Release Checklist

**Feature Verification:**
- [ ] All navigation features work (hjkl, gg, G, 5g, Bg, A4g, w/b/e, zt/zz/zb)
- [ ] All editing features work (Insert mode, Magnifier mode)
- [x] All column operations work (,dd, ,yy, ,p, ,o, ,O)
- [ ] All visual mode features work (v, V, ,v)
- [ ] Search works (/pattern, n, N, *, :noh)
- [ ] Undo/redo works (u, Ctrl+r, .)
- [ ] Transforms work (~, gU, gu, g~, g., gj, gk)
- [ ] Multi-file workflow works ([, ], :files)
- [ ] Save/quit protection works (:w, :Wq, :q, :q!)
- [ ] Header toggle system works (:ht)
- [x] Dual clipboard buffers work (yy+p for rows, ,yy+,p for columns, cross-buffer shows "Nothing to paste")
- [ ] Range operations work (:5,10d, :B,Dd)
- [ ] System clipboard works ("+yy, "+,yy, "+p)

**Code Quality:**
- [ ] All tests passing (target: 500+ tests)
- [ ] Zero clippy warnings
- [ ] Zero compiler warnings
- [ ] Code coverage > 80%
- [ ] Documentation complete and accurate

**User Experience:**
- [ ] Help system complete (? for quick ref, :help for full)
- [ ] Error messages clear and helpful
- [ ] Empty document handling graceful
- [ ] Edge cases covered (0 rows, 0 columns, single row/column)

**Performance:**
- [ ] Opens 100K row file in < 2 seconds
- [ ] Smooth scrolling at 60fps
- [ ] No memory leaks

**Documentation:**
- [ ] README.md complete and accurate
- [ ] Keybinding reference up to date
- [ ] Configuration guide complete
- [ ] Examples and tutorials available
- [ ] `--help` output accurate

### Acceptance Criteria

- [ ] All v0.4.1 - v0.9.0 features complete
- [ ] No known critical bugs
- [ ] Documentation matches implementation
- [ ] Performance meets targets
- [ ] Ready for public announcement
- [ ] Version 1.0.0 tagged and released

---

## Post-1.0 Roadmap

Future enhancements for post-1.0 releases:

### v1.1.0 - Bulk Operations & Advanced Filtering

**Bulk Operations:**
- Fill down: `:.,$ fd` (fill current cell down to end)
- Auto-number: `:@A enum` (auto-number column A)
- Delete empty rows: `:g/^$/d`
- Delete rows matching pattern: `:g/pattern/d`
- Deduplication: `:dedup` (remove duplicate rows)

**Advanced Filtering:**
- `:filter column=value` - show only matching rows
- `:sort column` - sort by column
- `:sort -r column` - reverse sort

**Deferred because:** Complex features, less critical for v1.0. Better to stabilize core first.

### v1.2.0 - Advanced Column Operations

**Features:**
- Column resize (manual width adjustment)
- Column freeze (keep columns visible when scrolling)
- Column hide/unhide
- Column reorder via drag (visual mode enhancement)

**Deferred because:** Nice-to-have features, core functionality sufficient for v1.0.

### v1.3.0 - Data Analysis & Export

**Features:**
- Formulas (basic calculations)
- Aggregations (sum, avg, count)
- Export formats (JSON, Markdown, HTML)
- Data validation
- Conditional formatting

**Deferred because:** Advanced features beyond core CSV editing. Post-stable enhancements.

---

## Design Decisions Summary

This roadmap reflects extensive design refinement. Key decisions:

1. **Header Row Toggle System** - Revolutionary simplification. Header is row 0, toggleable with `:ht`.
2. **Simplified Navigation** - `5g`, `Bg`, `A4g` instead of `:5`, `:B`, `:A5`. Reserve `:` for operations.
3. **Comma Leader** - Use `,` for column operations (not `;`). More vim-like, doesn't conflict with vim's `;`.
4. **3 Visual Modes** - Simplified from 4 to 3: `v` (block), `V` (line), `,v` (column). Cleaner, less complex.
5. **Dual Clipboard Buffers** - Separate row and column buffers. `p` uses row buffer, `,p` uses column buffer. No cross-buffer pasting.
6. **Magnifier via `m`** - Use `m` for Magnifier (not `Enter`). Clear distinction from Insert mode.
7. **Truly Hybrid** - Balance vim power with spreadsheet familiarity. Support both paradigms.
8. **Zero Config Default** - Works perfectly out of the box. Optional `~/.config/lazycsv/config.toml` for power users.
9. **Simplified Commands** - `:w` saves all (not just current). `:Wq` for write all and quit. Fewer variants.
10. **Status Line** - Mode + row + column only. No cell preview. Minimalist.
11. **No CLI Flags** - No `--delimiter` or `--no-headers`. Use `:delim` and `:ht` commands instead. More discoverable.
12. **Full Edge Case Support** - 0 rows, 0 columns, header-only, empty files. All gracefully handled.

**Philosophy:** Maximum power, minimum complexity. Vim-first, but truly hybrid. Zero configuration, full customization.
