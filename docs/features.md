# LazyCSV Features

Complete feature specification for LazyCSV.

This document details the functional "what" of LazyCSV. For information on how these features are visually presented and interactable, refer to the [Design Document](design.md) and [Keybindings Reference](keybindings.md).

## Philosophy

LazyCSV is designed around these core principles:

1. **Keyboard-first** - All operations accessible via keyboard
2. **Vim-inspired** - Familiar navigation for vim users
3. **Fast** - Handle 10K+ rows at 60 FPS
4. **Simple** - Clean, minimal interface
5. **Powerful** - Complex operations with simple keystrokes

## Implemented Features (v0.1.0 - v0.3.1)

This section details all features currently available in the application.

### File Loading & Handling
- ✅ Load CSV files from the command line (`lazycsv file.csv`).
- ✅ Discover and load files from a directory (`lazycsv .`).
- ✅ Support for custom delimiters, encodings, and files with no headers.
- ✅ Graceful error handling for invalid files or paths.

### Table Display
- ✅ **Standard View**: Row numbers, column letters (A, B...), and headers.
- ✅ **Highlighting**: The current row and cell are clearly indicated.
- ✅ **Scrolling**: Both vertical and horizontal scrolling are supported.
- ✅ **Text Truncation**: Long cell content is truncated with `...`.

### Vim-Style Navigation
All navigation is keyboard-driven with vim-inspired keys.

**Basic Movement:**
- ✅ `h` / `←` - Move left
- ✅ `j` / `↓` - Move down
- ✅ `k` / `↑` - Move up
- ✅ `l` / `→` - Move right

**Advanced Movement & Jumps:**
- ✅ `gg` / `Home` - Jump to the first row.
- ✅ `G` / `End` - Jump to the last row.
- ✅ `0` - Jump to the first column.
- ✅ `$` - Jump to the last column.
- ✅ `w`, `b`, `e` - Word-style motion to jump between non-empty cells.
- ✅ `PageUp` / `PageDown` - Page up or down.

**Count Prefixes:**
- ✅ Use numbers before commands to repeat them (e.g., `5j` moves down 5 rows).

### Command Mode
- ✅ Press `:` to enter Command mode for direct jumps.
- ✅ Jump to a specific line (e.g., `:15`).
- ✅ Jump to a specific column by letter (e.g., `:B`, `:BC`).

### Viewport Control
- ✅ `zt` - Position the current row at the **t**op of the viewport.
- ✅ `zz` - Position the current row at the **c**enter of the viewport.
- ✅ `zb` - Position the current row at the **b**ottom of the viewport.

### Multi-File Management
- ✅ **Auto-discovery**: Automatically finds all `.csv` files in the same directory.
- ✅ **File Switcher**: A persistent panel at the bottom shows all available files.
- ✅ **Quick Switching**: Use `[` and `]` to cycle between files.

### Application Features
- ✅ **Help System**: A toggleable overlay (`?`) shows available keybindings.
- ✅ **Status Bar**: Provides contextual information about the file, position, and mode.
- ✅ **Quit Protection**: Warns on quit if there are unsaved changes (partial implementation of v0.6.0). Note: Editing is not yet implemented, so the `is_dirty` flag can only be set for testing purposes.

## Planned Features

The following features are on the roadmap and are **not yet implemented**.

### v0.4.0-v0.6.0: Cell Editing & Persistence

**Edit Mode:**
- 📋 Press `i` or `Enter` to edit current cell.
- 📋 Select-all text by default (ready to replace).
- 📋 Type to modify value.
- 📋 `Enter` to save, `Esc` to cancel.
- 📋 Visual indicator (yellow background).
- 📋 Mode indicator shows `[EDIT]`.

**File Saving:**
- 📋 `Ctrl+S` to save changes.
- 📋 `:w` command to save (vim-style).
- 📋 Atomic write (write to temp, then rename).
- 📋 Success message: "✓ Saved successfully".
- 📋 Error handling for save failures.

**Dirty State Tracking:**
- 📋 `*` indicator in title when modified.
- 📋 Vim-style quit behavior:
  - `q` warns and refuses to quit (already implemented).
  - `:q!` forces quit without saving.

**Undo/Redo:**
- 📋 `u` to undo last operation.
- 📋 `Ctrl+r` to redo.
- 📋 History of 100 operations.
- 📋 Works for cell edits, row/column ops, sorts.
- 📋 Shows what was undone: "Undo: Edit cell A5".

### v0.7.0-v0.8.0: Row & Column Operations

**Row Operations:**
- 📋 `o` - Add row below current (empty cells).
- 📋 `O` - Add row above current (empty cells).
- 📋 `dd` - Delete current row (no confirmation).
- 📋 `yy` - Copy (yank) current row.
- 📋 `p` - Paste row below current.
- 📋 `P` - Paste row above current.

**Column Operations:**
- 📋 `Ctrl+A` - Add column after current.
- 📋 `Ctrl+Shift+A` - Add column before current.
- 📋 `D` - Delete current column (no confirmation).
- 📋 Prompt for column header name on add.

### v1.0.0-v1.3.0: Advanced Features

**Fuzzy Search:**
- 📋 Press `/` to open fuzzy finder overlay.
- 📋 Search multiple types: row numbers, column letters/names, cell data.
- 📋 Live results as you type.
- 📋 `n`/`N` to cycle through matches after jumping.
- 📋 `*` to search current cell value.

**Sorting:**
- 📋 `s` - Sort by current column (toggle asc/desc).
- 📋 In-place sort (actually reorders data).
- 📋 Smart sorting (numeric vs. text).
- 📋 Sort indicator in header: ↑ or ↓.
- 📋 Undoable.

**Filtering:**
- 📋 `:filter` command with expressions (e.g., `:filter Age>30`).
- 📋 Support for multiple operators (`=`, `!=`, `>`, `<`, `contains`, etc.).
- 📋 Multiple filters (AND logic).
- 📋 `:nofilter` to clear.

**Visual Selection:**
- 📋 `v` - Enter visual mode (cell selection).
- 📋 `V` - Visual line mode (row selection).
- 📋 Extend with `hjkl`.
- 📋 Operations on selection (`d` to delete, `y` to copy).

**Column Statistics:**
- 📋 `:stats` command to show stats for the current column.
- 📋 Display in overlay panel.

### v1.3.0: Multi-File Guards

**CSV Multi-File:**
- ✅ Already implemented!

**Unsaved Changes Protection:**
- 📋 `[` / `]` blocked if current file has unsaved changes.
- 📋 Status error: "No write since last change".
- 📋 Force switch with `:next!` / `:prev!` (future).
- 📋 Prevents accidental data loss when switching files.

## Performance Requirements

LazyCSV is designed for speed:

| Operation | Target | Status |
|-----------|--------|--------|
| File loading | < 100ms for 10K rows | ✅ Achieved |
| Render frame | < 16ms (60 FPS) | ✅ Achieved |
| Navigation | < 10ms response | ✅ Achieved |
| Search | < 200ms for 10K rows | 📋 v1.1.0 |
| Sort | < 500ms for 10K rows | 📋 v1.2.0 |
| Save | < 200ms for 10K rows | 📋 v0.6.0 |

## Constraints & Limitations

### Current (v0.1.0):
- **Read-only** - No editing yet (v0.4.0)
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
- May add as option in v1.4.0

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
| In-place editing | v0.4.0 | ✅ | ❌ | ✅ |
| Multi-file nav | ✅ | ❌ | ❌ | ❌ |
| Clean UI | ✅ | ❌ | ✅ | ❌ |
| Built-in help | ✅ | ✅ | ❌ | ✅ |

**LazyCSV's Niche:**
- Faster than Excel for viewing
- More intuitive than visidata
- More powerful than less/cat
- Consistent multi-file experience (innovation!)

## Future Ideas

### v1.4.0+: Advanced Features
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
