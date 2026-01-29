# LazyCSV Features

Complete feature specification for LazyCSV.

## Philosophy

LazyCSV is designed around these core principles:

1. **Keyboard-first** - All operations accessible via keyboard
2. **Vim-inspired** - Familiar navigation for vim users
3. **Fast** - Handle 10K+ rows at 60 FPS
4. **Simple** - Clean, minimal interface
5. **Powerful** - Complex operations with simple keystrokes

## Current Features (Phase 1 - MVP)

### File Loading
- ✅ Load CSV files from command line: `lazycsv file.csv`
- ✅ Load from directory: `lazycsv .` or `lazycsv /path/to/dir`
- ✅ No arguments defaults to current directory: `lazycsv`
- ✅ Support absolute and relative paths (files and directories)
- ✅ Directory mode: loads first CSV file alphabetically
- ✅ Handle UTF-8 encoding
- ✅ Parse quoted fields and escaped commas
- ✅ Error messages for invalid files or missing directories

**Usage:**
```bash
# Specific file
lazycsv data.csv

# Current directory (loads first CSV alphabetically)
lazycsv
lazycsv .

# Parent directory
lazycsv ..

# Subdirectory
lazycsv ./data
lazycsv data/exports

# Absolute directory path
lazycsv /home/user/csvfiles
```

### Table Display
- ✅ **Row numbers** - Left gutter shows row numbers (1, 2, 3...)
- ✅ **Column letters** - Top row shows column letters (A, B, C...)
- ✅ **Headers** - Bold header row with column names
- ✅ **Data rows** - All CSV data displayed in table format
- ✅ **Current row indicator** - `►` symbol shows selected row
- ✅ **Current cell highlight** - Selected cell shown with reverse video
- ✅ **Text truncation** - Long values truncated with `...` (max 20 chars)
- ✅ **Horizontal scrolling** - Show ~10 columns at a time

**Visual Layout:**
```
┌─ lazycsv: data.csv ────────────────────┐
│     │  A      │ ►B      │  C      │... │ ← Column letters (► shows selected)
├─────┼─────────┼─────────┼─────────┼────┤
│  #  │  Name   │  Email  │  Age    │... │ ← Headers
├─────┼─────────┼─────────┼─────────┼────┤
│  1  │  Alice  │  a@e... │  30     │... │
│►2   │  Bob    │ [b@e...]│  25     │... │ ← Current cell (highlighted)
│  3  │  Charlie│  c@e... │  35     │... │
├─────┴─────────┴─────────┴─────────┴────┤
│ [?] help │ [q] quit │ [ ] files │      │ ← Status bar (left: controls,
│ Row 2/100 │ Col B: Email (2/5) │       │           right: position)
│ Cell: "bob@example.com"                │
├───────────────────────────────────────┤
│ Files (1/2): ► data.csv | other.csv  │ ← File switcher
└───────────────────────────────────────┘
```

### Vim-Style Navigation
All navigation is keyboard-driven with vim-inspired keys:

**Cursor Movement:**
- ✅ `h` / `←` - Move left (previous column)
- ✅ `j` / `↓` - Move down (next row)
- ✅ `k` / `↑` - Move up (previous row)
- ✅ `l` / `→` - Move right (next column)

**Jumps:**
- ✅ `gg` / `Home` - Jump to first row
- ✅ `G` / `End` - Jump to last row
- ✅ `0` - Jump to first column
- ✅ `$` - Jump to last column

**Paging:**
- ✅ `PageUp` / `PageDown` - Page up/down (~20 rows)

### Multi-File Navigation
LazyCSV treats CSV files in the same directory like "worksheets":

- ✅ **Auto-discovery** - Scans directory for all .csv files on startup
- ✅ **Works with files or directories** - Scans parent dir when given a file, or scans the directory when given a dir path
- ✅ **Always-visible switcher** - Bottom panel shows all available files
- ✅ **Quick switching** - Press `[` for previous, `]` for next file
- ✅ **Current file indicator** - `►` shows active file in top bar and file switcher
- ✅ **File count** - Shows "Files (2/5): ► file1.csv | file2.csv | ..."

**Usage:**
```bash
# Open a specific file - automatically finds other CSVs in same directory
lazycsv sales.csv

# Open a directory - loads first CSV alphabetically, finds all others
lazycsv .
lazycsv /path/to/csvfiles

# Now in the app:
# Press ] to switch to next file (customers.csv)
# Press [ to switch back to previous file (sales.csv)
```

### Help System
- ✅ **Toggle help overlay** - Press `?` to show/hide cheatsheet
- ✅ **Organized layout** - Grouped by function (Navigation, Editing, etc.)
- ✅ **Context-aware** - Shows available keys for current phase
- ✅ **Centered overlay** - Doesn't obscure entire table
- ✅ **Close with `?` or `Esc`**

### Status Bar
Always-visible status bar with two sections:

**Left side (controls):**
- ✅ Quick help: `[?] help`
- ✅ Quit hint: `[q] quit`
- ✅ File switching hint: `[ ] files` (when multiple files)

**Right side (position info):**
- ✅ Current row: `Row 5/100`
- ✅ Current column: `Col B: Email (2/5)` (letter, name, and position)
- ✅ Current cell value: `Cell: "value"` (or `<empty>` for empty cells)

**Format:**
```
[?] help │ [q] quit │ [ ] files │ Row 5/100 │ Col B: Email (2/5) │ Cell: "example"
```

### File Information
- ✅ Filename in title bar
- ✅ Dirty indicator `*` when unsaved (Phase 2)
- ✅ Row count and column count in status

## Planned Features

### Phase 2: Cell Editing

**Edit Mode:**
- 📋 Press `i` or `Enter` to edit current cell
- 📋 Select-all text by default (ready to replace)
- 📋 Type to modify value
- 📋 `Enter` to save, `Esc` to cancel
- 📋 Visual indicator (yellow background)
- 📋 Mode indicator shows `[EDIT]`

**File Saving:**
- 📋 `Ctrl+S` to save changes
- 📋 `:w` command to save (vim-style)
- 📋 Atomic write (write to temp, then rename)
- 📋 Success message: "✓ Saved successfully"
- 📋 Error handling for save failures

**Dirty State Tracking:**
- 📋 `*` indicator in title when modified
- 📋 Warning on quit if unsaved changes
- 📋 Vim-style quit behavior:
  - `q` warns and refuses to quit
  - `:q!` forces quit without saving

**Undo/Redo:**
- 📋 `u` to undo last operation
- 📋 `Ctrl+r` to redo
- 📋 History of 100 operations
- 📋 Works for cell edits, row/column ops, sorts
- 📋 Shows what was undone: "Undo: Edit cell A5"

### Phase 3: Row & Column Operations

**Row Operations:**
- 📋 `o` - Add row below current (empty cells)
- 📋 `O` - Add row above current (empty cells)
- 📋 `dd` - Delete current row (no confirmation)
- 📋 `yy` - Copy (yank) current row
- 📋 `p` - Paste row below current
- 📋 `P` - Paste row above current

**Column Operations:**
- 📋 `Ctrl+A` - Add column after current
- 📋 `Ctrl+Shift+A` - Add column before current
- 📋 `D` - Delete current column (no confirmation)
- 📋 Prompt for column header name on add

**Design Decisions:**
- No confirmation for delete operations (rely on undo)
- New rows have empty strings for all cells
- Clipboard persists across operations (can paste multiple times)

### Phase 4: Advanced Features

**Fuzzy Search:**
- 📋 Press `/` to open fuzzy finder overlay
- 📋 Search multiple types:
  - Row numbers: "15" finds row 15
  - Column letters: "C" finds column C
  - Column names: "Email" finds Email column (fuzzy: "eml" → Email)
  - Cell data: "widget" finds cells containing "widget"
- 📋 Live results as you type
- 📋 `j`/`k` to navigate results
- 📋 `Enter` to jump to match
- 📋 `Esc` to cancel without jumping
- 📋 `n`/`N` to cycle through matches after jumping
- 📋 `*` to search current cell value

**Sorting:**
- 📋 `s` - Sort by current column (toggle asc/desc)
- 📋 In-place sort (actually reorders data)
- 📋 Smart sorting (numeric vs. text)
- 📋 Sort indicator in header: ↑ or ↓
- 📋 Undoable
- 📋 Sets dirty flag

**Filtering:**
- 📋 `:filter` command with expressions
- 📋 Syntax: `column operator value`
- 📋 Operators: `=`, `!=`, `>`, `<`, `>=`, `<=`, `contains`, `starts`, `ends`
- 📋 Examples:
  - `:filter Age>30`
  - `:filter Name contains "John"`
- 📋 Multiple filters (AND logic)
- 📋 Status indicator: "Filtered: 45/100 rows"
- 📋 `:nofilter` to clear

**Visual Selection:**
- 📋 `v` - Enter visual mode (cell selection)
- 📋 `V` - Visual line mode (row selection)
- 📋 Extend with `hjkl`
- 📋 Highlighted region (blue tint)
- 📋 Operations on selection:
  - `d` - Delete selected rows
  - `y` - Copy selected rows
- 📋 Show selection count: "5 rows selected"

**Column Statistics:**
- 📋 `:stats` command
- 📋 Show for current column:
  - Count (non-empty cells)
  - Sum (if numeric)
  - Average (if numeric)
  - Min/Max (if numeric)
  - Unique values (if text)
- 📋 Display in overlay panel
- 📋 Close with `Esc`

### Phase 5: Multi-File/Sheet Navigation

**CSV Multi-File:**
- 📋 Already implemented in Phase 1! ✅
- 📋 Scan directory on startup
- 📋 Switch with `[` and `]`
- 📋 Always-visible file list at bottom

**Excel Support:**
- 📋 Detect file type: `.xlsx`, `.xls`, `.xlsm`
- 📋 Load Excel files with calamine crate
- 📋 Extract all sheet names
- 📋 Load first/active sheet by default
- 📋 Convert Excel data types:
  - Numbers → formatted strings
  - Dates → ISO 8601 format
  - Formulas → evaluated values (or formula text)
  - Boolean → "TRUE"/"FALSE"
- 📋 Handle merged cells (take first value)

**Multi-Sheet Navigation:**
- 📋 Show sheet list at bottom (same as file list)
- 📋 Title shows "Sheets" instead of "Files"
- 📋 Current sheet with `►` indicator
- 📋 Press `[`/`]` to switch sheets
- 📋 Show count: "Sheets (2/5)"
- 📋 Consistent UX with CSV multi-file

**Saving:**
- 📋 Save as CSV (convert from Excel)
- 📋 Warning when converting (potential data loss)
- 📋 Future: Save back to Excel (preserve other sheets)

## Performance Requirements

LazyCSV is designed for speed:

| Operation | Target | Status |
|-----------|--------|--------|
| File loading | < 100ms for 10K rows | ✅ Achieved |
| Render frame | < 16ms (60 FPS) | ✅ Achieved |
| Navigation | < 10ms response | ✅ Achieved |
| Search | < 200ms for 10K rows | 📋 Phase 4 |
| Sort | < 500ms for 10K rows | 📋 Phase 4 |
| Save | < 200ms for 10K rows | 📋 Phase 2 |

## Constraints & Limitations

### Current (Phase 1):
- **Read-only** - No editing yet (Phase 2)
- **Memory-bounded**: The entire file is loaded into memory. This is fast for small to medium files (up to 100K rows), but makes it unsuitable for very large datasets that don't fit in RAM. True lazy-loading is a top priority for future development.
- **~10 columns visible** - Horizontal scroll for more
- **20 char cell limit** - Longer text truncated with `...`
- **Monochrome** - No colors (design decision)
- **English only** - No i18n (for now)

### Future Improvements:
- Virtual scrolling for massive files (1M+ rows)
- Dynamic column width calculation
- Cell formatting (numbers, dates, currency)
- Formula evaluation
- Plugin system

## Design Decisions

### Why No Confirmations for Delete?
- Faster workflow for power users
- Undo system provides safety net
- Follows vim philosophy

### Why Select-All in Edit Mode?
- Most edits are replacements, not additions
- Press `End` key to append if needed
- Faster for common case

### Why In-Place Sort?
- Simpler mental model (data actually changes)
- Sets dirty flag appropriately
- Undoable if mistake
- Alternative "view-only" sort adds complexity

### Why Case-Insensitive Search?
- More useful for data exploration
- Can override with flag in future
- Matches most user expectations

### Why No Colors?
- Cleaner, more professional look
- Works on all terminals
- Less visual noise
- May add as option in Phase 6

### Why Multi-File for CSV?
- Provides consistent UX with Excel multi-sheet
- Convenient for related datasets
- Innovative feature not found in other tools
- Simple with `[` and `]` keys

## Use Cases

### Data Exploration
- Quick view of CSV files without opening Excel
- Navigate large datasets efficiently
- Check data before importing

### Data Cleaning
- Find and fix errors in cells
- Delete duplicate or bad rows
- Standardize column formats

### Data Validation
- Check for missing values
- Verify data types
- Count unique values

### Quick Edits
- Fix typos
- Update cell values
- Add/remove rows

### Batch Operations
- Sort by column
- Filter specific rows
- Copy/paste rows between files

## Comparison with Other Tools

| Feature | LazyCSV | Excel | less/cat | visidata |
|---------|---------|-------|----------|----------|
| Keyboard-first | ✅ | ❌ | ✅ | ✅ |
| Vim navigation | ✅ | ❌ | Partial | ✅ |
| Fast (10K+ rows) | ✅ | ❌ | ✅ | ✅ |
| In-place editing | Phase 2 | ✅ | ❌ | ✅ |
| Multi-file nav | ✅ | ❌ | ❌ | ❌ |
| Excel support | Phase 5 | ✅ | ❌ | ✅ |
| Clean UI | ✅ | ❌ | ✅ | ❌ |
| Built-in help | ✅ | ✅ | ❌ | ✅ |

**LazyCSV's Niche:**
- Faster than Excel for viewing
- More intuitive than visidata
- More powerful than less/cat
- Consistent multi-file experience (innovation!)

## Future Ideas

### Phase 6+: Advanced Features
- Configuration file (`~/.config/lazycsv/config.toml`)
- Custom keybindings
- Theme support (colors as option)
- SQL query mode (query CSV like database)
- Export formats (JSON, Markdown, HTML)
- Diff mode (compare two CSVs)
- Formula evaluation (basic spreadsheet functions)
- Clipboard integration (system clipboard)
- Plugin system
- Network file loading (HTTP URLs)

## Feedback & Requests

Have ideas for new features? Open an issue on GitHub!

- What features would make LazyCSV more useful for you?
- What workflows should we optimize?
- What pain points can we solve?

We prioritize features based on:
1. User demand
2. Alignment with keyboard-first philosophy
3. Implementation complexity
4. Performance impact
