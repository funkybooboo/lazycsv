# LazyCSV Keybindings Reference

Complete keyboard shortcuts reference for LazyCSV.

Press `?` in the app to see the built-in cheatsheet.

## Philosophy

LazyCSV keybindings follow vim conventions:

- **Mnemonic**: Keys chosen for easy memory (o=add, d=delete, y=yank/copy)
- **Modal**: Different modes (Normal, Edit, Visual, Command)
- **Efficient**: Common actions are single keystrokes
- **Consistent**: Same patterns across operations

## Mode Indicator

The current mode is always shown in the title bar:
- `lazycsv: file.csv` - Normal mode
- `lazycsv: file.csv [EDIT]` - Edit mode
- `lazycsv: file.csv [VISUAL]` - Visual mode
- `lazycsv: file.csv [COMMAND]` - Command mode

## Phase 1: Navigation (Current)

### Cursor Movement

| Key | Action |
|-----|--------|
| `h` or `←` | Move left (previous column) |
| `j` or `↓` | Move down (next row) |
| `k` or `↑` | Move up (previous row) |
| `l` or `→` | Move right (next column) |
| `w` | Next column (word forward) |
| `b` | Previous column (word backward) |
| `0` | First column |
| `$` | Last column |
| `gg` or `Home` | First row |
| `G` or `End` | Last row |
| `PageUp` | Page up (~20 rows) |
| `PageDown` | Page down (~20 rows) |

**Tips:**
- Vim users: Use `hjkl` for faster navigation (home row)
- Others: Arrow keys work just as well
- `w`/`b` are same as `l`/`h` for now (column = "word")

### File Navigation

| Key | Action |
|-----|--------|
| `[` | Previous CSV file in directory |
| `]` | Next CSV file in directory |

**How it works:**
- LazyCSV scans the directory on startup
- All CSV files are available to switch between
- File list shown at bottom with `►` indicator
- Like Excel sheets, but for CSV files!

### Help & Information

| Key | Action |
|-----|--------|
| `?` | Toggle help/cheatsheet |
| `Esc` | Close help (when help is open) |

### Quit

| Key | Action |
|-----|--------|
| `q` | Quit (warns if unsaved changes in Phase 2) |

## Phase 2: Cell Editing (Planned)

### Entering Edit Mode

| Key | Action |
|-----|--------|
| `i` | Enter edit mode at current cell |
| `Enter` | Enter edit mode at current cell |

**Behavior:**
- All text is selected by default (ready to replace)
- Type to replace entire value
- Press `End` to move to end if you want to append

### While in Edit Mode

| Key | Action |
|-----|--------|
| Type characters | Edit cell value |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character at cursor |
| `←` `→` | Move cursor within cell |
| `Home` | Move to start of cell |
| `End` | Move to end of cell |
| `Enter` | Save changes and exit edit mode |
| `Esc` | Cancel changes and exit edit mode |
| `Ctrl+C` | Cancel changes (alternative) |

### Saving

| Key | Action |
|-----|--------|
| `Ctrl+S` | Save file |
| `:w` | Save file (command mode) |
| `:wq` | Save and quit |

### Undo/Redo

| Key | Action |
|-----|--------|
| `u` | Undo last operation |
| `Ctrl+R` | Redo |

**What can be undone:**
- Cell edits
- Row operations (add, delete, paste)
- Column operations (add, delete)
- Sort operations
- Up to 100 operations

## Phase 3: Row & Column Operations (Planned)

### Row Operations

| Key | Action |
|-----|--------|
| `o` | Add row below current |
| `O` | Add row above current |
| `dd` | Delete current row |
| `5dd` | Delete 5 rows |
| `yy` | Copy (yank) current row |
| `5yy` | Copy 5 rows |
| `p` | Paste row below current |
| `P` | Paste row above current |

**Notes:**
- New rows are empty (blank cells)
- No confirmation for delete (use undo if mistake)
- Clipboard persists (can paste multiple times)

### Column Operations

| Key | Action |
|-----|--------|
| `Ctrl+A` | Add column after current |
| `Ctrl+Shift+A` | Add column before current |
| `D` | Delete current column |

**Notes:**
- Prompted for column header name when adding
- No confirmation for delete (use undo if mistake)

### Visual Selection

| Key | Action |
|-----|--------|
| `v` | Enter visual mode (cell selection) |
| `V` | Enter visual line mode (row selection) |
| `hjkl` | Extend selection |
| `d` | Delete selected rows |
| `y` | Copy selected rows |
| `Esc` | Exit visual mode |

## Phase 4: Advanced Features (Planned)

### Fuzzy Search

| Key | Action |
|-----|--------|
| `/` | Open fuzzy finder |
| Type | Search query (finds rows, columns, cell data) |
| `j` or `↓` | Next result |
| `k` or `↑` | Previous result |
| `Enter` | Jump to selected result |
| `Esc` | Cancel search |
| `*` | Search for current cell value |

**After jumping:**
| Key | Action |
|-----|--------|
| `n` | Next match |
| `N` | Previous match |

**What fuzzy search finds:**
- **Row numbers**: "15" → row 15
- **Column letters**: "C" → column C
- **Column names**: "Email" or "eml" → Email column
- **Cell data**: "widget" → cells containing "widget"

### Sorting

| Key | Action |
|-----|--------|
| `s` | Sort by current column (toggle asc/desc) |
| `:sort` | Sort ascending (command) |
| `:sort!` | Sort descending (command) |

**Notes:**
- In-place sort (actually reorders rows)
- Smart: numeric sort for numbers, text sort for strings
- Indicator in header shows ↑ or ↓
- Undoable

### Filtering

| Command | Action |
|---------|--------|
| `:filter Age>30` | Filter rows where Age > 30 |
| `:filter Name contains "John"` | Filter rows where Name contains John |
| `:nofilter` or `:nof` | Clear all filters |

**Filter operators:**
- `=` - Equals
- `!=` - Not equals
- `>`, `<`, `>=`, `<=` - Comparisons (numeric)
- `contains` - Contains substring
- `starts` - Starts with
- `ends` - Ends with

### Statistics

| Command | Action |
|---------|--------|
| `:stats` | Show statistics for current column |

## Phase 5: Excel Support (Planned)

Same keybindings work for Excel files!

| Key | Action |
|-----|--------|
| `[` | Previous sheet in workbook |
| `]` | Next sheet in workbook |

**Usage:**
```bash
lazycsv workbook.xlsx    # Opens Excel file
# Press ] to switch to next sheet
# Press [ to go back to previous sheet
```

## Command Mode

Press `:` to enter command mode.

### Available Commands

| Command | Action |
|---------|--------|
| `:q` | Quit (warns if unsaved) |
| `:q!` | Force quit (no save) |
| `:w` | Save file |
| `:wq` or `:x` | Save and quit |
| `:w newfile.csv` | Save as newfile.csv |
| `:sort` | Sort by current column (asc) |
| `:sort!` | Sort by current column (desc) |
| `:filter <expr>` | Filter rows |
| `:nofilter` | Clear filter |
| `:stats` | Show column statistics |
| `:help` or `:h` | Show help |

## Global Keys

These work in (almost) all modes:

| Key | Action |
|-----|--------|
| `Ctrl+C` | Cancel/escape current operation |
| `Ctrl+L` | Redraw screen |

## Quick Reference Card

Print-friendly summary:

```
╔═══════════════════════════════════════════════════════╗
║              LAZYCSV QUICK REFERENCE                  ║
╠═══════════════════════════════════════════════════════╣
║ Navigation     │ Editing (Phase 2) │ Files           ║
║  hjkl/arrows   │  i/Enter  Edit    │  [/]   Switch   ║
║  gg/G First/La │  Esc      Cancel  │  ?     Help     ║
║  w/b  Next/Pre │  ^S       Save    │  q     Quit     ║
║  0/$  First/La │  u        Undo    │                 ║
║  PageUp/Down   │  ^R       Redo    │                 ║
╠═══════════════════════════════════════════════════════╣
║ Rows (Phase 3) │ Columns (Phase 3) │ Search (Phase 4)║
║  o    Add belo │  ^A      Add col  │  /     Search   ║
║  O    Add abov │  D       Del col  │  n/N   Next/Pre ║
║  dd   Delete   │  v       Visual   │  s     Sort     ║
║  yy   Copy     │  d       Del sel  │  *     Find     ║
║  p    Paste    │  y       Copy sel │                 ║
╚═══════════════════════════════════════════════════════╝
```

## Vim Users

If you're a vim user, these will feel natural:

### Direct Mappings (Same as Vim)
- Movement: `hjkl`, `gg`, `G`, `w`, `b`, `0`, `$`
- Operators: `dd`, `yy`, `p`, `P`, `u`, `Ctrl+R`
- Visual: `v`, `V`
- Search: `/`, `n`, `N`, `*`
- Commands: `:w`, `:q`, `:wq`, `:q!`

### Adapted (Similar Concept)
- `o`/`O` - Add row (vs. open line) - same mnemonic
- `i` - Edit cell (vs. insert mode) - same mnemonic
- `D` - Delete column (vs. delete to end of line)
- `s` - Sort column (vs. substitute) - different

### Different
- `[`/`]` - File/sheet navigation (vs. jump to section)
- `?` - Help (vs. search backward) - help takes precedence
- `Ctrl+A` - Add column (vs. increment number)

## Tips & Tricks

### Fast Navigation
```
gg → 0        # Jump to cell A1 (top-left)
G → $         # Jump to last cell (bottom-right)
5G            # Jump to row 5
```

### Efficient Editing (Phase 2+)
```
i → type → Enter       # Quick replace
yy → jjj → p          # Copy row, move down 3, paste
dd → p                # Cut and paste row
```

### Multi-File Workflow
```
] → ] → [             # Next, next, back one
# Or: Keep pressing ] to cycle through all files
```

### Search Power (Phase 4)
```
/email → Enter        # Find "Email" column
*                     # Find current cell value
n → n → n            # Jump through matches
```

## Customization (Phase 6+)

Future: Custom keybindings via config file:

```toml
# ~/.config/lazycsv/config.toml
[keybindings.normal]
quit = "q"
save = "<C-s>"
help = "?"
# ... customize any key
```

Key notation:
- `<C-x>` = Ctrl+X
- `<S-x>` = Shift+X
- `<M-x>` = Alt+X (Meta)
- `<Enter>` = Enter
- `<Esc>` = Escape
- `<Tab>` = Tab

## Getting Help

- **In app**: Press `?`
- **Full reference**: This document
- **Issues**: [GitHub Issues](https://github.com/yourusername/lazycsv/issues)
