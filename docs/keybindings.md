# LazyCSV Keybindings Reference

Complete keyboard shortcuts reference for LazyCSV.

Press `?` in the app to see the built-in cheatsheet.

## Philosophy

LazyCSV keybindings follow vim conventions:

- **Mnemonic**: Keys chosen for easy memory (o=add, d=delete, y=yank/copy, `;`=column ops)
- **Modal**: Different modes (Normal, Insert, Command)
- **Efficient**: Common actions are single keystrokes
- **Consistent**: Same patterns across operations (cell → row → column)
- **Vim-First**: Every action accessible via vim-style keys
- **Three-tier scope**: Cell (`x`) → Row (`dd`) → Column (`;dd`)

## Mode Indicators

The current mode is always shown in the status bar:
- `-- NORMAL --` - Navigation mode
- `-- INSERT --` - Quick cell editing
- `-- MAGNIFIER --` - Vim editor for power editing (multi-line cells)
- `-- HEADER EDIT --` - Editing column header names
- `-- VISUAL --` / `-- VISUAL LINE --` / `-- COLUMN VISUAL --` / `-- COLUMN VISUAL LINE --` - Selection modes
- `-- COMMAND --` - Command input mode

---

## v0.1.0 - Foundation

### Basic Navigation

| Key | Action |
|-----|--------|
| `h` or `←` | Move left (previous column) |
| `j` or `↓` | Move down (next row) |
| `k` or `↑` | Move up (previous row) |
| `l` or `→` | Move right (next column) |
| `Enter` | Move down one row (vim-style) |
| `w` | Jump to next non-empty cell in row |
| `b` | Jump to previous non-empty cell in row |
| `e` | Jump to last non-empty cell in row |

### File Navigation

| Key | Action |
|-----|--------|
| `[` | Previous CSV file in directory |
| `]` | Next CSV file in directory |

### Help & System

| Key | Action |
|-----|--------|
| `?` | Toggle help/cheatsheet |
| `Esc` | Close help / Cancel current operation |
| `q` | Quit |

---

## v0.2.0 - Type Safety Refactor

*Internal improvements - no new user-facing keybindings*

---

## v0.3.0 - Advanced Navigation ( Complete)

### Enhanced Movement

| Key | Action |
|-----|--------|
| `gg` | Jump to first row |
| `G` | Jump to last row |
| `<number>G` | Jump to specific row (e.g., `15G`) |
| `0` | Jump to first column |
| `$` | Jump to last column |
| `PageUp` | Page up (~20 rows) |
| `PageDown` | Page down (~20 rows) |
| `Enter` | Move down one row (like `j`) |

### Column Jumping (Excel-style)

**Note:** Column jumping uses direct command syntax (`:B`, `:AA`) for better vim compatibility.

| Key | Action |
|-----|--------|
| `:B` | Jump to column B |
| `:AA` | Jump to column AA (multi-letter) |
| `:A5` | Jump to cell A5 (column + row) |
| `:5` | Jump to row 5 |

**Column Letter System:** A=1, B=2, ..., Z=26, AA=27, AB=28, etc.

### Count Prefixes

| Pattern | Action |
|---------|--------|
| `5j` | Move down 5 rows |
| `3h` | Move left 3 columns |
| `10l` | Move right 10 columns |
| `3w` | Jump to 3rd next non-empty cell |

### Command Mode

| Command | Action |
|---------|--------|
| `:` | Enter command mode |
| `:15` | Jump to row 15 |
| `:B` | Jump to column B |
| `:BC` | Jump to column BC (55) |
| `:A5` | Jump to cell A5 |
| `Esc` | Cancel command input |

### Viewport Control

| Key | Action |
|-----|--------|
| `zt` | Position current row at top of screen |
| `zz` | Position current row at center of screen |
| `zb` | Position current row at bottom of screen |

---

## v0.3.1 - UI/UX Polish ( Complete)

*User interface improvements and command mode enhancements*

**UI Features:**
- Mode indicator (-- NORMAL -- / -- COMMAND --)
- Dirty flag display (*)
- Transient messages that auto-clear on keypress
- Enhanced help menu with better organization
- File list horizontal scrolling
- Minimal borders (horizontal rules replace heavy box borders)
- Vim-like status line: `NORMAL 3,C "cell value"` format
- Auto-width columns (sized to content, 8-50 char range)
- Row indicator: `>` marks current row

### Column Navigation

| Command | Action |
|---------|--------|
| `:B` or `:b` | Jump to column B (case-insensitive) |
| `:A5` | Jump to cell A5 |
| `:AA` or `:aa` | Jump to column AA (multi-letter) |

**Column Letter System:** A=1, B=2, ..., Z=26, AA=27, AB=28, etc.

### Reserved Commands (Priority)

These commands always take priority over navigation:

| Command | Action |
|---------|--------|
| `:q` | Quit |
| `:w` | Save (future) |
| `:wq` | Save and quit (future) |
| `:h` or `:help` | Show help |

### Pending Command Display

Multi-key commands now show in the status bar:
- `g` shows `g` while waiting for second key
- `z` shows `z` while waiting for second key
- `5` shows `5` while typing count prefix

No timeout - pending commands wait indefinitely (vim-like).

### Out-of-Bounds Handling

Commands show clear error messages instead of silently clamping:
- `:999` on 10-row file → "Row 999 does not exist (max: 10)"
- `:c Z` on 5-column file → "Column Z does not exist (max: E)"

---

## v0.4.0 - Insert Mode & Row Operations ( Complete)

### Entering Insert Mode

| Key | Action |
|-----|--------|
| `i` | Enter Insert mode at current cursor position |
| `a` | Enter Insert mode at end of cell (append) |
| `A` | Enter Insert mode at end of cell (same as `a`) |
| `I` | Enter Insert mode at beginning of cell |
| `s` | Clear cell and enter Insert mode |
| `F2` | Enter Insert mode at end (Excel/Calc style) |

### In Insert Mode

| Key | Action |
|-----|--------|
| Type characters | Insert text at cursor |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character at cursor |
| `←` `→` | Move cursor within cell |
| `Home` | Move to start of cell |
| `End` | Move to end of cell |
| `Ctrl+h` | Delete character before cursor (vim-style) |
| `Ctrl+w` | Delete word before cursor (vim-style) |
| `Ctrl+u` | Delete to start of cell (vim-style) |
| `Enter` | Save changes and move down one row |
| `Shift+Enter` | Save changes and move up one row |
| `Tab` | Save changes and move right one column |
| `Shift+Tab` | Save changes and move left one column |
| `Esc` | Cancel changes (discard edits) |

### Row Operations (Normal Mode)

| Key | Action |
|-----|--------|
| `o` | Add new row below, enter Insert mode |
| `O` | Add new row above, enter Insert mode |
| `dd` | Delete current row (stored in clipboard) |
| `yy` | Copy (yank) current row |
| `p` | Paste row below current position |
| `P` | Paste row above current position |
| `Delete` | Clear current cell content (stay in Normal mode)

---

## v0.4.1 - Persistence & Multi-File Workflow ( Complete)

### File Operations

| Command | Action |
|---------|--------|
| `:w` | Save current file |
| `:W` | Save all dirty files |
| `:wq` | Save current file and quit (blocks if other files dirty) |
| `:q` | Quit (blocks if ANY file has unsaved changes) |
| `:q!` | Force quit (discard all unsaved changes) |

### Command Ranges

**Row ranges (vim-style):**
| Range | Action |
|-------|--------|
| `:5d` | Delete row 5 |
| `:5,10d` | Delete rows 5-10 |
| `:5,10y` | Yank rows 5-10 |
| `:%d` | Delete all rows |
| `:.d` | Delete current row |
| `:.,+5d` | Delete current row and next 5 |
| `:$d` | Delete last row |

**Column ranges:**
| Range | Action |
|-------|--------|
| `:B,D` | Operate on columns B through D |
| `:B,Dd` | Delete columns B through D |
| `:B,Dy` | Yank columns B through D |

**Combined ranges (row AND column):**
| Range | Action |
|-------|--------|
| `:B,D@5,10d` | Delete cells in rows 5-10, columns B-D |
| `:B,D@5,10y` | Yank cells in rows 5-10, columns B-D |

### Multi-File Dirty Tracking

| Indicator | Meaning |
|-----------|---------|
| `filename.csv*` | File has unsaved changes (shown in file switcher) |
| No `*` | File is clean (no unsaved changes) |

**Notes:**
- `:w` saves only the current file
- `:W` saves all files with unsaved changes
- `:q` checks ALL files (current + others) and blocks if any are dirty
- File switcher shows `*` after filenames with unsaved changes
- Switching files preserves edits in session cache
- Command ranges use comma for ranges (`:5,10d`, `:B,D`)
- Combined ranges use `@` to separate columns from rows (`:B,D@5,10`)

---

## v0.5.0 - Column Operations & Visual Mode ( Complete)

### Semicolon Leader for Column Operations

The semicolon `;` key acts as a leader for all column-level operations, following the three-tier operator system:
- **Cell scope**: `x` (delete cell content)
- **Row scope**: `dd` (delete row)
- **Column scope**: `;dd` (delete column)

| Key | Action |
|-----|--------|
| `;o` | Insert new column to the right (enters HeaderEdit mode) |
| `;O` | Insert new column to the left (enters HeaderEdit mode) |
| `;dd` | Delete current column (including header) |
| `;yy` | Yank (copy) current column (including header) |
| `;p` | Paste column to the right of current |
| `;P` | Paste column to the left of current |

**Notes:**
- Semicolon is a silent leader (no visual feedback, vim standard)
- After paste, cursor moves to the new column
- Yanked columns include the header row
- Column operations work on entire columns (all rows + header)
- `;o` and `;O` automatically enter HeaderEdit mode to name the new column

### Visual Selection

| Key | Mode | Selection |
|-----|------|-----------|
| `v` | Visual | Cell-by-cell selection (free movement) |
| `V` | Visual Line | Whole rows |
| `;v` | Column Visual | Cell-by-cell selection (free movement, column intent) |
| `;V` | Column Visual Line | Whole columns |

**Operations in Visual mode:**
| Key | Action |
|-----|--------|
| `d` | Delete selection (clears cells for regions, removes rows/cols entirely) |
| `y` | Yank (copy) selection |
| `c` | Change selection (clear + enter Insert) |
| `p` | Paste selection (overwrites existing, adds rows/cols if needed) |
| `o` | Toggle cursor to opposite corner of selection |
| `Esc` | Exit Visual mode |
| `gv` | Re-select last visual selection |

**In Visual Mode:**
- `hjkl` extends selection
- Visual indicators show selected region
- Cell region delete clears cells, preserves structure
- Row/column delete removes rows/columns entirely
- All vim selection patterns work

**Notes:**
- `Ctrl+v` is NOT implemented (redundant with `v` for rectangular selection)
- `;v` has same behavior as `v` but signals column intent

### HeaderEdit Mode

| Key | Action |
|-----|--------|
| `gh` | Enter HeaderEdit mode for current column header |

**In HeaderEdit Mode:**
| Key | Action |
|-----|--------|
| Type | Edit header text |
| `Backspace` / `Delete` | Delete characters |
| `←` `→` | Move cursor |
| `Home` / `End` | Jump to start/end |
| `Enter` | Save header and return to Normal |
| `Esc` | Cancel changes and return to Normal |

### Count Prefixes

| Key | Action |
|-----|--------|
| `5dd` | Delete 5 rows |
| `5yy` | Yank 5 rows |
| `P` | Paste row above current |
| `cc` | Clear row and enter Insert mode |

---

## v0.6.0 - Vim Magnifier ( Complete)

### Opening Magnifier

| Key | Action |
|-----|--------|
| `Enter` | Open Magnifier for current cell (multi-line editing) |

### In Magnifier Mode (Comprehensive Vim Editor)

The Magnifier embeds a comprehensive vim editor for editing cells with complex content (JSON, multi-line text, etc.):

**Motions:**
| Key | Action |
|-----|--------|
| `hjkl`, arrows | Character movement |
| `w`, `b`, `e` | Word navigation (next, back, end) |
| `0`, `$` | Line start/end |
| `^` | First non-blank character |
| `gg`, `G` | First/last line |
| `f{char}`, `F{char}` | Find character forward/backward |
| `t{char}`, `T{char}` | Till character forward/backward |
| `;`, `,` | Repeat find forward/backward |

**Operators:**
| Key | Action |
|-----|--------|
| `x` | Delete character |
| `dd` | Delete line |
| `yy` | Yank (copy) line |
| `p`, `P` | Paste below/above |
| `cc` | Change line (delete and enter insert) |
| `C` | Change to end of line |
| `c{motion}` | Change operator (e.g., `cw` change word) |
| `r{char}` | Replace single character |
| `J` | Join current line with next |
| `>>`, `<<` | Indent/dedent line |

**Insert Mode Entry:**
| Key | Action |
|-----|--------|
| `i`, `a` | Insert before/after cursor |
| `I`, `A` | Insert at line start/end |
| `o`, `O` | Open line below/above |
| `s` | Substitute character (delete and insert) |

**Visual Mode:**
| Key | Action |
|-----|--------|
| `v` | Character-wise visual selection |
| `V` | Line-wise visual selection |
| `d` | Delete selection |
| `y` | Yank selection |
| `c` | Change selection |
| `Esc` | Exit visual mode |

**Search:**
| Key | Action |
|-----|--------|
| `/pattern` | Search forward (case-sensitive) |
| `n` | Jump to next match |
| `N` | Jump to previous match |
| `*` | Search for word under cursor |
| `:noh` | Clear search highlighting |

**Undo/Redo:**
| Key | Action |
|-----|--------|
| `u` | Undo (unlimited history) |
| `Ctrl+r` | Redo |

**Ex Commands:**
| Command | Action |
|---------|--------|
| `:w` | Save to cell (updates in-memory document) |
| `:q` | Quit (warns if unsaved changes) |
| `:wq` or `ZZ` | Save to cell and close |
| `:q!` | Force quit without saving |
| `:noh` | Clear search highlighting |

**Cell Navigation (while in Magnifier):**
| Key | Action |
|-----|--------|
| `Alt+h` or `Alt+Left` | Move to cell left (prompts to save if dirty) |
| `Alt+j` or `Alt+Down` | Move to cell below (prompts to save if dirty) |
| `Alt+k` or `Alt+Up` | Move to cell above (prompts to save if dirty) |
| `Alt+l` or `Alt+Right` | Move to cell right (prompts to save if dirty) |

**Important Notes:**
- Magnifier vim commands are **separate** from table vim commands
- `:w` saves to in-memory document, NOT to file (use table `:w` to save file)
- Search is case-sensitive by default
- Undo history is unlimited
- Visual mode supports both character-wise and line-wise selection

**Use Cases:**
- Editing JSON data in cells
- Multi-line text fields (descriptions, notes)
- Complex cell content that needs full vim power
- Large text content (>100 characters)

---

## v0.7.0 - Search ( Complete)

### Fuzzy Search

| Key | Action |
|-----|--------|
| `/` | Open fuzzy search overlay |
| `n` | Jump to next match |
| `N` | Jump to previous match |
| `Esc` | Close search overlay |

**In Search Mode:**
| Key | Action |
|-----|--------|
| Type | Enter search query (fuzzy matching) |
| `Enter` | Jump to first match |
| `Esc` | Cancel search |

**What search finds:**
- Cell data (fuzzy match on content)
- Column names (fuzzy match on headers)

---

## v0.8.0 - SQL Query Mode

### SQL Editor

| Command | Action |
|---------|--------|
| `:sql` | Open SQL editor |

**In SQL Editor:**
| Key | Action |
|-----|--------|
| Type | Write SQL query |
| `Ctrl+Enter` | Execute query and display results |
| `Esc` | Close SQL editor without executing |
| `Tab` | Auto-complete (table, columns, keywords) |
| `Up` / `Down` | Navigate query history |

**Supported SQL Features:**
- `SELECT` with column expressions
- `WHERE` clause with comparison operators  
- `ORDER BY` with ASC/DESC
- `LIMIT` and `OFFSET`
- Aggregate functions: COUNT, SUM, AVG, MIN, MAX
- `GROUP BY` with aggregates
- String functions: UPPER, LOWER, LENGTH, TRIM, SUBSTR
- Math operators: +, -, *, /, %
- Logical operators: AND, OR, NOT

**Notes:**
- Query results displayed in new virtual table view
- Error messages include fuzzy column name suggestions
- Syntax highlighting for SQL keywords
- Auto-complete for table name and column names

---

## v0.8.1 - SQL & Data Operations Polish

*Refactoring and code quality improvements - no new user-facing keybindings*

**Improvements:**
- Enhanced error messages with fuzzy matching suggestions (Levenshtein distance)
- Refactored SQL execution code (67.7% reduction)
- Refactored SQL editor rendering (70% reduction)
- 30 comprehensive SQL edge case tests
- 13 SQL benchmark groups
- 555 total tests passing

---

## v0.9.0 - Configuration System

### Configuration File

Configuration support (~/.config/lazycsv/config.toml):
- Color scheme customization
- Key binding remapping
- Default behavior settings

---

## v0.10.0 - Undo/Redo

### History Management

| Key | Action |
|-----|--------|
| `u` | Undo last operation |
| `Ctrl+r` | Redo |
| `.` | Repeat last edit (dot command) |

**What can be undone:**
- Cell edits (Insert mode and Magnifier)
- Row operations (add, delete, paste)
- Column operations (delete, yank, paste)
- Sorts and filters
- Up to 1000 operations in history

---

## v0.11.0 - SQL Editor Vim Editing

### Full Vim Modal Editing in SQL Editor

**Normal Mode:**
- All standard vim motions (hjkl, w/b/e, f/t, gg/G, etc.)
- Operators: d, c, y, p
- Visual mode: v (character), V (line)
- Search: /, n, N

**Insert Mode:**
- Enter with i, a, I, A, o, O
- Exit with Esc

**Special:**
- `Ctrl+Enter` - Execute query (works in any mode)

---

## v0.14.0 - Cell Transforms

### Data Operations

| Command | Action |
|---------|--------|
| `:sort` | Sort by current column (ascending) |
| `:sort!` | Sort by current column (descending) |
| `:filter <expr>` | Filter rows (e.g., `:filter Age>30`) |
| `:nof` | Clear all filters |

**Filter Operators:**
| Operator | Example |
|----------|---------|
| `=` | `:filter Status=active` |
| `!=` | `:filter Type!=deleted` |
| `>` | `:filter Age>30` |
| `<` | `:filter Score<100` |
| `contains` | `:filter Name contains "John"` |

---

## v1.0.0 - First Stable Release

**Polish, performance, and documentation improvements. No new keybindings.**

---

## Removed Features

The following features were removed from the roadmap to maintain simplicity:

**Removed from original plan:**
- Text objects (`ic`, `ac`, `ir`, `ar`) - Insert/Magnifier modes are sufficient
- Marks (`m`, `'`) - Basic navigation and search are sufficient
- Named registers (`"a`, `"b`) - System clipboard only
- Smart column navigation (`{`, `}`, `[[`, `]]`) - Use search instead
- Excel aliases (F4, Ctrl+-) - Vim-first only
- Old `:c` command - Replaced with direct `:B`, `:A5` syntax

**Kept (Essential Features):**
-  **Visual mode** (`v`, `V`, `;v`, `;V`) - Essential for selecting regions to copy/paste/delete
-  **HeaderEdit mode** (`gh`) - Essential for editing column header names
-  **Magnifier mode** (`Enter`) - Essential for multi-line cell editing (JSON, descriptions)
-  **Command ranges** (`:5,10d`, `:B,D`, `:B,D@5,10`) - Essential for batch operations

**Note:** `Ctrl+v` (block visual) was skipped as redundant - regular `v` already provides rectangular cell selection.

---

## Customization (v1.4.0+)

Custom keybindings via config file:

```toml
# ~/.config/lazycsv/config.toml
[keybindings.normal]
quit = "q"
save = "<C-s>"
help = "?"

[theme]
header = "cyan"
selected_cell = "blue"
status_bar = "blue"
```

**Key notation:**
- `<C-x>` = Ctrl+X
- `<S-x>` = Shift+X
- `<M-x>` = Alt+X (Meta)
- `<Enter>` = Enter
- `<Esc>` = Escape
- `<Space>` = Space

---

## Version Roadmap

| Version | Features Added |
|---------|----------------|
| v0.1.0 |  Foundation - viewing, basic navigation, file switching |
| v0.2.0 |  Type safety refactor (internal) |
| v0.3.0 |  Advanced navigation (column jumps, command mode, word motion, viewport control) |
| v0.3.1 |  UI/UX polish (mode indicator, transient messages, help redesign, minimal UI, command improvements) |
| v0.4.0 |  Insert mode & row operations (cell editing, add/delete/copy/paste rows) |
| v0.4.1 |  Persistence & multi-file workflow (save, command ranges, dirty tracking) |
| v0.5.0 |  Column operations & visual mode (column add/delete/yank/paste, visual selection) |
| v0.6.0 |  Vim Magnifier (full vim editor for cells, multi-line editing) |
| v0.7.0 |  Search (fuzzy search overlay, match navigation) |
| v0.8.0 |  SQL Query Mode (SQL editor, query execution, multi-file queries) |
| v0.8.1 |  SQL & Data Operations Polish (refactoring, error improvements, tests) |
| v0.9.0 | Configuration System (config file, themes, keybindings) |
| v0.10.0 | Undo/Redo (history management, dot command) |
| v0.11.0 | SQL Editor Vim Editing (full vim modal editing in SQL editor) |
| v0.12.0 | UI Consistency & HeaderEdit Mode (standardized UI, gh command for header editing) |
| v0.14.0 | Cell Transforms (case toggle, row swap, sort, filter) |
| v0.18.0 | SQL IntelliSense (auto-completion, context-aware suggestions) |
| v0.22.0 | Macros (command recording and replay) |
| v1.0.0 | First Stable Release (polish, performance, documentation) |

---

## Getting Help

- **In app**: Press `?`
- **Full reference**: This document
- **Issues**: [GitHub Issues](https://github.com/funkybooboo/lazycsv/issues)
