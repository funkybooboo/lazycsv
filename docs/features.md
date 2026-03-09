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

## Implemented Features (v0.1.0 - v0.8.1)

This section details all features currently available in the application.

### File Loading & Handling
-  Load CSV files from the command line (`lazycsv file.csv`).
-  Discover and load files from a directory (`lazycsv .`).
-  Support for custom delimiters, encodings, and files with no headers.
-  Non-interactive query mode (`lazycsv --query "SELECT * FROM data"`) for piping/automation.
-  Locale-aware number formatting with `--format` flag.
-  Cancellable file loading with `Esc` key.
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
-  **File Picker**: `:files` command opens an interactive file selection dialog.
-  **External Modification Detection**: Polls files every 2 seconds, prompts to reload if changed externally.

### Application Features
-  **Help System**: A toggleable overlay (`?`) shows available keybindings.
-  **Status Bar**: Provides contextual information about the file, position, and mode.
-  **Quit Protection**: Warns on quit if there are unsaved changes.

### v0.4.0: Insert Mode & Row Operations

**Cell Editing:**
-  Multiple entry modes: `i` (insert at cursor), `a` / `A` (append), `I` (insert at start), `s` (substitute)
-  `F2` for Excel-style editing
-  Navigation within cell: arrow keys, Home/End
-  Vim-style editing: `Ctrl+h` (delete char), `Ctrl+w` (delete word), `Ctrl+u` (delete line)
-  Smart save-and-move: `Enter` (down), `Shift+Enter` (up), `Tab` (right), `Shift+Tab` (left)
-  `Esc` to cancel changes

**Row Operations:**
-  `o` / `O` - Add new row below/above and enter Insert mode
-  `dd` - Delete current row (stored in clipboard)
-  `yy` - Copy (yank) current row
-  `p` / `P` - Paste row below/above
-  `Delete` - Clear current cell content

### v0.4.1: Persistence & Multi-File Workflow

**File Operations:**
-  `:w` - Save current file
-  `:W` - Save all dirty files
-  `:wq` - Save and quit (checks other files for dirty state)
-  `:q` - Quit (blocks if any file has unsaved changes)
-  `:q!` - Force quit, discard all changes

**Multi-File Dirty Tracking:**
-  Session caches dirty documents
-  File switcher shows `*` for unsaved files
-  Switching preserves edits in cache
-  After `:w`, file removed from cache

### v0.5.0: Column Operations & Visual Mode

**Comma Leader for Column Ops:**
-  `,o` / `,O` - Insert column right/left (enters HeaderEdit mode)
-  `,dd` - Delete column
-  `,yy` - Yank column (includes header)
-  `,p` / `,P` - Paste column right/left

**Visual Selection:**
-  `v` - Cell-by-cell visual selection (free movement)
-  `V` - Row visual selection (whole rows)
-  `,v` - Column cell visual (free movement, column intent)
-  Operations: `d` (delete/clear), `y` (yank), `c` (change), `p` (paste)

**Count Prefixes:**
-  `5dd` - Delete 5 rows
-  `5yy` - Yank 5 rows
-  `cc` - Clear row and enter Insert mode

### v0.6.0: Vim Magnifier

**Full Vim Editor for Cells:**
-  `Enter` - Open Magnifier for current cell
-  Full vim editing (multi-line, word motion, operators, visual mode)
-  `:w` - Save cell content
-  `:wq` or `ZZ` - Save and close
-  `:q!` - Close without saving
-  `Alt+hjkl` or `Alt+arrows` - Navigate to adjacent cells (prompts if dirty)
-  Unlimited undo/redo history
-  Search with `/`, `n`, `N`, `*`
-  All vim motions: `w`, `b`, `e`, `f`, `t`, `0`, `$`, `^`, `gg`, `G`
-  All vim operators: `d`, `c`, `y`, `p`, `r`, `x`, `>>`, `<<`, `J`

**Use Cases:**
- Editing JSON data in cells
- Multi-line descriptions or notes
- Complex text that needs vim power
- Large cell content (>100 chars)

### v0.7.0: Search

**Regex Search:**
-  `/` - Open search overlay
-  `n` / `N` - Next/previous match
-  `*` - Search current cell content
-  `:noh` - Clear highlighting
-  Regex pattern matching with automatic fallback to literal substring search
-  Case-insensitive by default
-  Live results as you type

### v0.8.0: SQL Query Mode

**SQL Editor:**
-  `:sql` - Open SQL editor
-  Write SQL queries against current CSV data
-  Auto-complete for table name, column names, and SQL keywords
-  Syntax highlighting for SQL
-  `Ctrl+Enter` - Execute query
-  `Esc` - Cancel long-running queries
-  Results displayed in new virtual table view
-  Error messages with fuzzy column name suggestions (Levenshtein distance)
-  Query history and navigation

**Supported SQL Features:**
-  `SELECT` with column expressions
-  `WHERE` clause with comparison operators
-  `ORDER BY` with ASC/DESC
-  `LIMIT` and `OFFSET`
-  Aggregate functions: COUNT, SUM, AVG, MIN, MAX
-  `GROUP BY` with aggregates
-  String functions: UPPER, LOWER, LENGTH, TRIM, SUBSTR
-  Math operators: +, -, *, /, %
-  Logical operators: AND, OR, NOT

**Column Sort Commands:**
-  `:sort <col,...>` - Sort by column(s) ascending (e.g., `:sort Name`, `:sort Dept,Name`)
-  `:sort! <col,...>` - Sort by column(s) descending
-  Supports multiple columns for stable sorting
-  In-place modification (sets dirty flag, undoable in v0.10.0+)

### v0.8.1: SQL & Data Operations Polish

**Refactoring & Code Quality:**
-  Refactored SQL execution code (164 → 53 lines, 67.7% reduction)
-  Refactored SQL editor rendering (118 → 35 lines, 70% reduction)
-  Created `src/app/sql_execution.rs` helper module (239 lines)
-  Created `src/ui/sql_editor_helpers.rs` rendering helpers (99 lines)
-  Enhanced error messages with fuzzy matching suggestions
-  30 comprehensive SQL edge case tests
-  13 SQL benchmark groups for performance monitoring

**Test Coverage:**
-  555 total tests (514 lib + 11 SQL integration + 30 edge cases)
-  All tests passing
-  Benchmarks for performance validation

## Planned Features

The following features are on the roadmap and are **not yet implemented**.

### v0.9.0: Configuration System

**Configuration File (~/.config/lazycsv/config.toml):**
- Color scheme customization
- Key binding remapping
- Default behavior settings
- Column width preferences
- Theme selection

### v0.10.0: Undo/Redo & Command Ranges

**History Management:**
- `u` - Undo last operation
- `Ctrl+r` - Redo
- `.` - Repeat last edit (dot command)
- Up to 1000 operations in history
- Works for: cell edits, row/column ops, sorts

**Command Ranges (Planned):**
- Row ranges: `:5,10d`, `:%d`, `:.d`, `:$d`
- Column ranges: `:B,D`, `:B,Dd`, `:B,Dy`
- Combined ranges: `:B,D@5,10d` (rectangular regions)

### v0.11.0: SQL Editor Vim Editing

**Full Vim Integration in SQL Editor:**
- Complete vim motions (hjkl, w/b/e, f/t, gg/G, etc.)
- Visual mode for query text selection
- Operators: d, c, y, p for query manipulation
- Text objects: iw, aw, i", a", etc.
- Search within query: /, n, N
- Undo/redo in editor: u, Ctrl+r
- Line operations: dd, yy, cc, o, O

### v0.14.0: Cell Transforms & Advanced Data Operations

**Cell Transforms:**
- `~` - Toggle case (UPPER ↔ lower)
- `gU` - Uppercase cell
- `gu` - Lowercase cell
- `g~` - Title Case cell
- `g.` - Toggle boolean (yes↔no, true↔false, 1↔0)

**Row Movement:**
- `gj` - Swap row with row below
- `gk` - Swap row with row above

**Advanced Filtering:**
- `:filter <expr>` - Filter rows (e.g., `:filter Age>30`)
- `:nof` - Clear all filters

### v1.0.0: First Stable Release

**Polish & Performance:**
- All core features working
- Stable command interface
- 500+ tests passing
- Complete documentation
- Performance targets met

## Performance Requirements

LazyCSV is designed for speed:

| Operation | Target | Status |
|-----------|--------|--------|
| File loading | < 100ms for 10K rows | Achieved |
| Render frame | < 16ms (60 FPS) | Achieved |
| Navigation | < 10ms response | Achieved |
| SQL queries | < 100ms for 10K rows | Achieved (v0.8.0) |
| Search | < 200ms for 10K rows | Achieved (v0.7.0) |
| Sort | < 500ms for 10K rows | Achieved (v0.8.0) |
| Save | < 200ms for 10K rows | Achieved (v0.4.1) |

## Constraints & Limitations

### Current (v0.8.1):
- **Memory-bounded**: The entire file is loaded into memory. This is fast for small to medium files (up to 100K rows), but makes it unsuitable for very large datasets that don't fit in RAM. True lazy-loading is a top priority for future development.
- **~10 columns visible** - Horizontal scroll for more
- **20 char cell limit** - Longer text truncated with `...` (use Magnifier for full editing)
- **Monochrome** - No colors (design decision)
- **English only** - No i18n (for now)
- **SQL read-only** - SQL queries create virtual views, don't modify data

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
- `,v` provides column-specific selection

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
| In-place editing |  |  |  |  |
| Multi-file nav |  |  |  |  |
| Clean UI |  |  |  |  |
| Built-in help |  |  |  |  |
| SQL queries |  |  |  |  |

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
