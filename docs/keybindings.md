# LazyCSV Keybindings Reference

Complete keyboard shortcuts reference for LazyCSV.

Press `?` in the app to see the built-in cheatsheet.

## Philosophy

LazyCSV keybindings follow vim conventions:

- **Mnemonic**: Keys chosen for easy memory (o=add, d=delete, y=yank/copy, m=magnify)
- **Modal**: Different modes (Normal, Insert, Magnifier, Visual, Command)
- **Efficient**: Common actions are single keystrokes
- **Consistent**: Same patterns across operations
- **Vim-First**: Every action accessible via vim-style keys

## Mode Indicators

The current mode is always shown in the status bar:
- `-- NORMAL --` - Navigation mode
- `-- INSERT --` - Quick cell editing
- `-- MAGNIFIER --` - Vim editor for power editing
- `-- HEADER EDIT --` - Editing column headers
- `-- VISUAL --` - Selection mode
- `-- COMMAND --` - Command input mode

---

## v0.1.0 - Foundation (Current)

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

## v0.3.0 - Advanced Navigation (✅ Complete)

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

| Key | Action |
|-----|--------|
| `ga` or `gA` | Jump to column A (first column) |
| `gB` | Jump to column B |
| `gBC` | Jump to column 55 (Excel-style letters) |

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
| `:BC` | Jump to column 55 |
| `Esc` | Cancel command input |

### Viewport Control

| Key | Action |
|-----|--------|
| `zt` | Position current row at top of screen |
| `zz` | Position current row at center of screen |
| `zb` | Position current row at bottom of screen |

---

## v0.3.1 - UI/UX Polish (✅ Complete)

*User interface improvements - no new keybindings*

**Features:**
- Mode indicator (-- NORMAL -- / -- COMMAND --)
- Dirty flag display (*)
- Transient messages that auto-clear on keypress
- Enhanced help menu with better organization
- File list horizontal scrolling

---

## v0.4.0 - Quick Editing

### Entering Insert Mode

| Key | Action |
|-----|--------|
| `i` | Enter Insert mode at current position |
| `a` | Enter Insert mode with cursor after current position |
| `A` | Enter Insert mode at end of cell content |
| `I` | Enter Insert mode at beginning of cell content |
| `gi` | Go to last edited cell and enter Insert mode |

### In Insert Mode

| Key | Action |
|-----|--------|
| Type characters | Insert text at cursor |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character at cursor |
| `←` `→` | Move cursor within cell |
| `Home` | Move to start of cell |
| `End` | Move to end of cell |
| `Ctrl+h` | Backspace (vim-style) |
| `Ctrl+w` | Delete word before cursor |
| `Ctrl+u` | Delete to start of line |
| `Enter` | Save changes and exit to Normal mode |
| `Esc` | Cancel changes and exit to Normal mode |

---

## v0.5.0 - Vim Magnifier

### Opening Magnifier

| Key | Action |
|-----|--------|
| `Enter` | Open Magnifier for current cell |

### In Magnifier Mode (Full Vim)

The Magnifier embeds a complete vim-like editor:

**Normal Mode Commands:**
- `i`, `a`, `A`, `I` - Enter Insert mode
- `o`, `O` - Open new line
- `dd` - Delete line
- `yy` - Yank (copy) line
- `p`, `P` - Paste
- `hjkl` - Navigate
- `w`, `b` - Word navigation
- `0`, `$` - Line start/end
- `gg`, `G` - File start/end

**Commands:**
| Command | Action |
|---------|--------|
| `:w` | Save to memory (not to file yet) |
| `:q` | Close magnifier, discard changes |
| `:wq` or `ZZ` | Save to memory and close |
| `:q!` | Force close without saving |

**Cell Navigation:**
| Key | Action |
|-----|--------|
| `Ctrl+h` | Move to cell left (prompts to save if dirty) |
| `Ctrl+j` | Move to cell below (prompts to save if dirty) |
| `Ctrl+k` | Move to cell above (prompts to save if dirty) |
| `Ctrl+l` | Move to cell right (prompts to save if dirty) |

---

## v0.6.0 - Save/Quit Guards

### Saving

| Key/Command | Action |
|-------------|--------|
| `Ctrl+S` | Save file |
| `:w` | Save file |
| `:wq` | Save and quit |
| `:x` | Save and quit (alias) |

### Quitting

| Command | Action |
|---------|--------|
| `:q` | Quit (fails if unsaved changes) |
| `:q!` | Force quit (discard all changes) |

---

## v0.7.0 - Row Operations

### Add/Delete Rows

| Key | Action |
|-----|--------|
| `o` | Add row below, enter Insert mode for first cell |
| `O` | Add row above, enter Insert mode for first cell |
| `dd` | Delete current row |
| `<number>dd` | Delete N rows (e.g., `3dd`) |
| `<number>o` | Add N rows (e.g., `2o`) |

### Copy/Paste Rows

| Key | Action |
|-----|--------|
| `yy` | Copy (yank) current row |
| `<number>yy` | Copy N rows (e.g., `5yy`) |
| `p` | Paste row below current |
| `P` | Paste row above current |

**Notes:**
- New rows are empty (blank cells)
- No confirmation for delete (use `u` to undo if mistake)
- Clipboard persists (can paste multiple times)

---

## v0.8.0 - Column Operations

### Column Operators (Vim-Style)

| Key | Action |
|-----|--------|
| `dc` | Delete current column |
| `yc` | Yank (copy) current column |
| `pc` | Paste column after current |
| `Pc` | Paste column before current |
| `o` | In header row: add column after, enter HeaderEdit mode |
| `O` | In header row: add column before, enter HeaderEdit mode |

**Notes:**
- Column operators work like vim: `d` for delete, `y` for yank, `p` for paste
- `c` suffix targets the column (like `w` targets a word in vim)
- After adding column with `o`/`O`, automatically enter HeaderEdit mode
- No confirmation needed (use `u` to undo)
- All cells in new column start empty

---

## v0.9.0 - Header Management

### Header Editing

| Key/Command | Action |
|-------------|--------|
| `gh` | Enter HeaderEdit mode for current column header |
| `:rename <name>` | Rename current column header |

### In HeaderEdit Mode

| Key | Action |
|-----|--------|
| Type characters | Edit header name |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character at cursor |
| `←` `→` | Move cursor |
| `Home` | Move to start |
| `End` | Move to end |
| `Enter` | Save header change, return to Normal |
| `Esc` | Cancel changes, return to Normal |

### Header Row Toggle

| Command | Action |
|---------|--------|
| `:headers` | Toggle header row on/off |

**Toggle On:** Promotes first data row to headers
**Toggle Off:** Demotes headers to first data row

---

## v1.0.0 - Undo/Redo & Power Commands

### Undo/Redo

| Key | Action |
|-----|--------|
| `u` | Undo last operation |
| `Ctrl+r` | Redo |

### Vim Power Features

| Key | Action |
|-----|--------|
| `.` | Repeat last edit (dot command) |

**What can be undone:**
- Cell edits (quick and magnifier)
- Row operations (add, delete, paste)
- Column operations (delete, yank, paste)
- Header edits and renames
- Header row toggle
- Sort operations
- Up to 100 operations

---

## v1.1.0 - Marks System

### Setting & Jumping to Marks

| Key | Action |
|-----|--------|
| `m[a-z]` | Set mark at current cell (e.g., `ma` sets mark 'a) |
| `'[a-z]` | Jump to mark (beginning of cell) |
| `` `[a-z] `` | Jump to mark (exact position) |
| `''` or `` `` `` | Jump back to previous position |
| `'.` | Jump to last edited cell |

**Examples:**
```
ma          # Set mark 'a at current cell
gg → 0      # Jump to A1
...         # Do some work
'a          # Jump back to mark 'a
```

---

## v1.2.0 - Search & Visual

### Fuzzy Search

| Key | Action |
|-----|--------|
| `/` | Open fuzzy finder |
| `*` | Search for current cell value |

**In Search Mode:**
| Key | Action |
|-----|--------|
| Type | Enter search query |
| `j` or `↓` | Next result |
| `k` or `↑` | Previous result |
| `Enter` | Jump to selected result |
| `Esc` | Cancel search |

**After Jumping:**
| Key | Action |
|-----|--------|
| `n` | Next match |
| `N` | Previous match |

**What fuzzy search finds:**
- **Row numbers**: "15" → row 15
- **Column letters**: "C" → column C
- **Column names**: "Email" or "eml" → Email column
- **Cell data**: "widget" → cells containing "widget"

### Visual Selection

| Key | Action |
|-----|--------|
| `v` | Enter Visual mode (cell selection) |
| `V` | Enter Visual Line mode (row selection) |
| `Ctrl+v` | Enter Visual Block mode (rectangle selection) |
| `d` | Delete selection |
| `y` | Yank (copy) selection |
| `o` | Move cursor to other end of selection |
| `Esc` | Exit Visual mode |

**In Visual Mode:**
- `hjkl` extends selection
- `o` jumps cursor to opposite corner of selection
- Visual indicators (`══`) show selected rows
- Block mode allows selecting rectangular regions

---

## v1.2.0 - Sorting & Filtering

### Sorting

| Key/Command | Action |
|-------------|--------|
| `s` | Sort by current column (toggle asc/desc) |
| `:sort` | Sort ascending |
| `:sort!` | Sort descending |

**Notes:**
- Smart: numeric sort for numbers, text sort for strings
- Header shows ↑ or ↓ indicator
- Undoable

### Filtering

| Command | Action |
|---------|--------|
| `:filter <expr>` | Filter rows (e.g., `:filter Age>30`) |
| `:nofilter` or `:nof` | Clear all filters |

**Filter Operators:**
| Operator | Meaning | Example |
|----------|---------|---------|
| `=` | Equals | `:filter Status=active` |
| `!=` | Not equals | `:filter Type!=deleted` |
| `>` | Greater than | `:filter Age>30` |
| `<` | Less than | `:filter Score<100` |
| `>=` | Greater or equal | `:filter Price>=10` |
| `<=` | Less or equal | `:filter Qty<=50` |
| `contains` | Contains substring | `:filter Name contains "John"` |
| `starts` | Starts with | `:filter Email starts "admin"` |
| `ends` | Ends with | `:filter File ends ".csv"` |

---

## v1.3.0 - Multi-File Guards

| Key | Action |
|-----|--------|
| `[` | Previous file (blocks if unsaved changes) |
| `]` | Next file (blocks if unsaved changes) |

**Error:** "No write since last change" if trying to switch with dirty file

---

## v1.4.0 - Command Ranges

### Range Operations

| Command | Action |
|---------|--------|
| `:1,10d` | Delete rows 1-10 |
| `:1,10y` | Yank (copy) rows 1-10 |
| `:%d` | Delete all data rows |
| `:%y` | Yank all rows |
| `:'a,'bd` | Delete from mark 'a to mark 'b |
| `:'a,$y` | Yank from mark 'a to last row |

**Range Syntax:**
- `<number>` - Specific row (e.g., `5` means row 5)
- `.` - Current row
- `$` - Last row
- `%` - All rows (1,$)
- `'a` - Mark 'a

---

## v1.5.0 - Advanced Viewing

### Column Management

| Command | Action |
|---------|--------|
| `:freeze` | Freeze current column and all to its left |
| `:autowidth` | Auto-size current column to fit content |

| Key | Action |
|-----|--------|
| `Ctrl+Left` | Decrease column width |
| `Ctrl+Right` | Increase column width |

### Statistics & Plotting

| Command | Action |
|---------|--------|
| `:stats` | Show statistics for current column |
| `:plot` | Show text-based plot for numeric column |

### Data Transformation

| Command | Action |
|---------|--------|
| `:s/pattern/replacement/g` | Regex search and replace |
| `:transpose` | Toggle transposed view (rows↔columns) |
| `:sort <col1>,<col2>` | Multi-column sort |

---

## Global Keys

These work in (almost) all modes:

| Key | Action |
|-----|--------|
| `Ctrl+C` | Cancel/escape current operation |
| `Ctrl+L` | Redraw screen |

---

## Quick Reference Card

Print-friendly summary:

```
╔═══════════════════════════════════════════════════════╗
║              LAZYCSV QUICK REFERENCE                  ║
╠═══════════════════════════════════════════════════════╣
║ NAVIGATION     │ EDITING          │ FILES             ║
║  hjkl/arrows   │  i    Quick edit │  [/]   Switch     ║
║  gg/G First/La │  Enter Magnifier │  ?     Help       ║
║  0/$  Col 1/End│  Esc  Cancel     │  q     Quit       ║
║  w/b/e Words   │  gi   Last+Edit  │                   ║
║  gA/gBC Coljmp │  .    Repeat     │                   ║
║  :15/:B  Jump  │  ^S   Save file  │                   ║
╠═══════════════════════════════════════════════════════╣
║ ROWS           │ COLUMNS          │ HEADER            ║
║  o/O  Add      │  o/O  Add col    │  gh     Edit hdr  ║
║  dd   Delete   │  dc   Delete     │  :ren   Rename    ║
║  yy   Copy     │  yc   Copy       │  :headers Toggle  ║
║  p/P  Paste    │  pc/Pc Paste     │                   ║
╠═══════════════════════════════════════════════════════╣
║ SEARCH         │ SORT/FILTER      │ SYSTEM            ║
║  /     Search  │  s      Sort     │  u      Undo      ║
║  *     Find    │  :sort  Sort cmd │  ^r     Redo      ║
║  n/N   Next    │  :filt  Filter   │  :w     Save      ║
║  v/V/^v Visual │  :nof   Clear    │  :q!    Force q   ║
╠═══════════════════════════════════════════════════════╣
║ MARKS          │ RANGES           │                   ║
║  ma    Set     │  :1,10d Delete   │                   ║
║  'a     Jump   │  :%y   Yank all  │                   ║
║  `.     Last   │  :'a,'bd Range   │                   ║
╚═══════════════════════════════════════════════════════╝
```

---

## Vim User Guide

### Direct Mappings (Same as Vim)

**Movement:**
- `hjkl` - Basic navigation
- `gg`, `G` - First/last row
- `0`, `$` - First/last column
- `w`, `b`, `e` - Word motion (next/prev/last non-empty cell)

**Operators:**
- `dd` - Delete row
- `yy` - Yank (copy) row
- `dc` - Delete column
- `yc` - Yank column
- `p`, `P` - Paste
- `pc`, `Pc` - Paste column
- `u` - Undo
- `Ctrl+r` - Redo
- `.` - Repeat last edit

**Visual:**
- `v`, `V` - Visual mode
- `Ctrl+v` - Visual block mode

**Marks:**
- `m[a-z]` - Set mark
- `'[a-z]` - Jump to mark
- `''` - Jump back
- `'.` - Jump to last edit

**Search:**
- `/` - Search
- `n`, `N` - Next/previous match
- `*` - Search for word under cursor

**Commands:**
- `:w` - Write (save)
- `:q` - Quit
- `:wq` - Write and quit
- `:q!` - Force quit
- `:1,10d` - Delete range
- `:%y` - Yank all

### Adapted Mappings (Similar Concept)

| Key | LazyCSV | Vim | Mnemonic |
|-----|---------|-----|----------|
| `o`/`O` | Add row/col | Open line | o = open/add |
| `i` | Edit cell | Insert mode | i = insert |
| `Enter` | Magnifier | - | Enter = enter cell |
| `dc`/`yc` | Delete/yank col | dw/yw (word) | c = column |
| `gh` | Edit header | - | g = go, h = header |
| `gi` | Go to last edit | gi | g = go, i = insert |
| `s` | Sort | Substitute | s = sort |
| `m` | Set mark | m | m = mark |

### Different from Vim

| Key | LazyCSV | Vim |
|-----|---------|-----|
| `[`/`]` | Switch files | Jump to previous/next section |
| `?` | Help | Search backward |
| `Enter` | Magnifier / Down | - |
| `w`/`b`/`e` | Next/prev/last cell | Next/prev/end of word |
| `o`/`O` | Add row/col | Open line |
| `dc`/`yc` | Column delete/yank | Delete/yank word |

---

## Tips & Tricks

### Lightning Fast Navigation

```
gg → 0              # Jump to cell A1 (top-left)
G → $               # Jump to last cell (bottom-right)
15G                 # Jump to row 15
gBC                 # Jump to column 55
:25 → Enter         # Jump to row 25
:C → Enter          # Jump to column C
```

### Efficient Editing Workflows

**Quick Edit & Power Edit:**
```
i → type → Enter    # Quick replace cell value
Enter → edit → :wq  # Full vim magnifier editing
gi → type → Enter   # Jump to last edit, edit, save
.                   # Repeat last edit (dot command!)
```

**Row Manipulation:**
```
yy → jjj → p        # Copy row, move down 3, paste
5dd → p             # Delete 5 rows, paste them elsewhere
o → type → Enter    # Add row and enter data
```

**Column Operations:**
```
gA → dc             # Jump to column A, delete it
yc → 5l → pc        # Copy column, move right 5, paste column
gD → yc → gA → Pc   # Copy column D, paste before column A
o → Name → Enter    # Add column in header, name it
```

**Marks & Navigation:**
```
ma                  # Set mark 'a at current cell
gg → 0              # Jump to A1
...                 # Do some work
'a                  # Jump back to mark 'a
`.                  # Jump to last edited cell
```

**Word Motion (Sparse Data):**
```
w → w → w           # Jump to next non-empty cells
b → b               # Jump to previous non-empty cell
e                   # Jump to last non-empty cell
```

**Visual Block (Rectangle Selection):**
```
Ctrl+v → jj → l → y # Select 3 rows x 2 cols, yank
5j → p              # Move down 5, paste block
```

**Command Ranges:**
```
:1,10d              # Delete rows 1-10
:%y                 # Yank all rows
:'a,'bd             # Delete from mark 'a to 'b
:10,20y → 25 → p    # Copy rows 10-20, jump to row 25, paste
```

### Search Workflows

```
/email → Enter      # Find "email" in columns/cells
*                   # Search for value in current cell
n → n → n           # Jump through matches
:s/widget/gadget/g  # Replace all "widget" with "gadget"
```

### Multi-File Workflow

```
] → ] → [           # Next, next, back one
] → :w → ]          # Save, then switch
```

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
| v0.1.0 | ✅ Foundation - viewing, basic navigation |
| v0.2.0 | ✅ Type safety refactor (internal) |
| v0.3.0 | ✅ Advanced navigation (column jumps, command mode, word motion) |
| v0.3.1 | ✅ UI/UX polish (mode indicator, transient messages, help redesign) |
| v0.4.0 | Quick editing (Insert mode) |
| v0.5.0 | Vim magnifier (power editing) |
| v0.6.0 | Save/quit guards |
| v0.7.0 | Row operations |
| v0.8.0 | Column operations |
| v0.9.0 | Header management |
| v1.0.0 | Undo/redo system |
| v1.1.0 | Search & visual selection |
| v1.2.0 | Sorting & filtering |
| v1.3.0 | Multi-file guards |
| v1.4.0 | Advanced viewing (freeze, themes) |
| v1.5.0 | Data analysis (stats, plotting) |
| v1.6.0 | Final polish |

---

## Getting Help

- **In app**: Press `?`
- **Full reference**: This document
- **Issues**: [GitHub Issues](https://github.com/yourusername/lazycsv/issues)
