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
-  Load CSV files from the command line (`lazycsv file.csv`).
-  Discover and load files from a directory (`lazycsv .`).
-  Support for custom delimiters, encodings, and files with no headers.
-  Graceful error handling for invalid files or paths.

### Table Display
-  **Standard View**: Row numbers, column letters (A, B...), and headers.
-  **Highlighting**: The current row and cell are clearly indicated.
-  **Scrolling**: Both vertical and horizontal scrolling are supported.
-  **Text Truncation**: Long cell content is truncated with `...`.

### Vim-Style Navigation
All navigation is keyboard-driven with vim-inspired keys.

**Basic Movement:**
-  `h` / `←` - Move left
-  `j` / `↓` - Move down
-  `k` / `↑` - Move up
-  `l` / `→` - Move right

**Advanced Movement & Jumps:**
-  `gg` / `Home` - Jump to the first row.
-  `G` / `End` - Jump to the last row.
-  `0` - Jump to the first column.
-  `$` - Jump to the last column.
-  `w`, `b`, `e` - Word-style motion to jump between non-empty cells.
-  `PageUp` / `PageDown` - Page up or down.

**Count Prefixes:**
-  Use numbers before commands to repeat them (e.g., `5j` moves down 5 rows).

### Command Mode
-  Press `:` to enter Command mode for direct jumps.
-  Jump to a specific line (e.g., `:15`).
-  Jump to a specific column by letter (e.g., `:B`, `:BC`).

### Viewport Control
-  `zt` - Position the current row at the **t**op of the viewport.
-  `zz` - Position the current row at the **c**enter of the viewport.
-  `zb` - Position the current row at the **b**ottom of the viewport.

### Multi-File Management
-  **Auto-discovery**: Automatically finds all `.csv` files in the same directory.
-  **File Switcher**: A persistent panel at the bottom shows all available files.
-  **Quick Switching**: Use `[` and `]` to cycle between files.

### Application Features
-  **Help System**: A toggleable overlay (`?`) shows available keybindings.
-  **Status Bar**: Provides contextual information about the file, position, and mode.
-  **Quit Protection**: Warns on quit if there are unsaved changes (partial implementation of v0.6.0). Note: Editing is not yet implemented, so the `is_dirty` flag can only be set for testing purposes.

## Planned Features

The following features are on the roadmap and are **not yet implemented**.

### v0.4.1: Persistence & Multi-File Workflow

**File Operations:**
- 📋 `:w` - Save current file
- 📋 `:W` - Save all dirty files
- 📋 `:wq` - Save and quit (checks other files for dirty state)
- 📋 `:q` - Quit (blocks if any file has unsaved changes)
- 📋 `:q!` - Force quit, discard all changes

**Command Ranges:**
- 📋 Row ranges: `:5,10d`, `:%d`, `:.d`, `:$d`
- 📋 Column ranges: `:B,D`, `:B,Dd`, `:B,Dy`
- 📋 Combined ranges: `:B,D@5,10d` (rectangular regions)

**Multi-File Dirty Tracking:**
- 📋 Session caches dirty documents
- 📋 File switcher shows `*` for unsaved files
- 📋 Switching preserves edits in cache
- 📋 After `:w`, file removed from cache

### v0.5.0: Column Operations & Visual Mode

**Semicolon Leader for Column Ops:**
- 📋 `;o` / `;O` - Insert column right/left (enters HeaderEdit mode)
- 📋 `;dd` - Delete column
- 📋 `;yy` - Yank column (includes header)
- 📋 `;p` / `;P` - Paste column right/left

**Visual Selection:**
- 📋 `v` - Cell-by-cell visual selection (free movement)
- 📋 `V` - Row visual selection (whole rows)
- 📋 `;v` - Column cell visual (free movement, column intent)
- 📋 `;V` - Column visual line (whole columns)
- 📋 Operations: `d` (delete/clear), `y` (yank), `c` (change), `p` (paste)

**HeaderEdit Mode:**
- 📋 `gh` - Edit column header name
- 📋 Enter to save, Esc to cancel

**Count Prefixes:**
- 📋 `5dd` - Delete 5 rows
- 📋 `5yy` - Yank 5 rows
- 📋 `P` - Paste row above
- 📋 `cc` - Clear row and enter Insert mode

### v0.6.0: Vim Magnifier

**Full Vim Editor for Cells:**
- 📋 `Enter` - Open Magnifier for current cell
- 📋 Full vim editing (multi-line, word motion, etc.)
- 📋 `:w` - Save cell content
- 📋 `:wq` or `ZZ` - Save and close
- 📋 `:q!` - Close without saving
- 📋 `Alt+hjkl` or `Alt+arrows` - Navigate to adjacent cells (prompts if dirty)

**Use Cases:**
- Editing JSON data in cells
- Multi-line descriptions or notes
- Complex text that needs vim power
- Large cell content (>100 chars)

### v0.7.0: Search

**Fuzzy Search:**
- 📋 `/` - Open search
- 📋 `n` / `N` - Next/previous match
- 📋 `*` - Search current cell content
- 📋 `:noh` - Clear highlighting

### v0.8.0: Undo/Redo

**History Management:**
- 📋 `u` - Undo last operation
- 📋 `Ctrl+r` - Redo
- 📋 `.` - Repeat last edit (dot command)
- 📋 Up to 100 operations in history
- 📋 Works for: cell edits, row/column ops, sorts

### v0.9.0: Transforms & Polish

**Cell Transforms:**
- 📋 `~` - Toggle case (UPPER ↔ lower)
- 📋 `gU` - Uppercase cell
- 📋 `gu` - Lowercase cell
- 📋 `g~` - Title Case cell
- 📋 `g.` - Toggle boolean (yes↔no, true↔false, 1↔0)

**Row Movement:**
- 📋 `gj` - Swap row with row below
- 📋 `gk` - Swap row with row above

**Data Operations:**
- 📋 `:sort` - Sort by current column (ascending)
- 📋 `:sort!` - Sort by current column (descending)
- 📋 `:filter <expr>` - Filter rows (e.g., `:filter Age>30`)
- 📋 `:nof` - Clear all filters

### v1.0.0: First Stable Release

**Polish & Performance:**
- 📋 All core features working
- 📋 Stable command interface
- 📋 500+ tests passing
- 📋 Complete documentation
- 📋 Performance targets met

## Performance Requirements

LazyCSV is designed for speed:

| Operation | Target | Status |
|-----------|--------|--------|
| File loading | < 100ms for 10K rows |  Achieved |
| Render frame | < 16ms (60 FPS) |  Achieved |
| Navigation | < 10ms response |  Achieved |
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

### Why Semicolon for Column Operations?
- Right next to comma on keyboard (easy muscle memory)
- Vim's `;` (repeat f/t) is less commonly used, acceptable override
- Visual separator mnemonic (columns separate data)
- Keeps comma free for vim-style range operations (`:5,10d`)

### Why Command Ranges?
- Vim-native syntax (`:5,10d`, `:%y`)
- Powerful batch operations without visual mode
- Essential for productivity with large datasets
- Familiar to vim users

### Why @ for Combined Ranges?
- Clear visual separator between column and row ranges
- `:B,D@5,10` reads as "columns B-D at rows 5-10"
- Unambiguous parsing (commas for ranges, @ for combination)

### Why No Ctrl+v Block Visual?
- Regular `v` already provides rectangular cell selection
- CSV data is naturally rectangular (cells align to grid)
- Reduces complexity, maintains simplicity
- `;v` and `;V` provide column-specific selection

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
| Keyboard-first |  |  |  |  |
| Vim navigation |  |  | Partial |  |
| Fast (10K+ rows) |  |  |  |  |
| In-place editing | v0.4.0 |  |  |  |
| Multi-file nav |  |  |  |  |
| Clean UI |  |  |  |  |
| Built-in help |  |  |  |  |

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
