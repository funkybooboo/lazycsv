# Source Code

This directory contains the Rust source code for LazyCSV.

## Structure

```
src/
├── main.rs       - Entry point and event loop
├── app.rs        - Application state and keyboard handling
├── csv_data.rs   - CSV data structures and file I/O
└── ui.rs         - UI rendering with ratatui
```

## Modules

### `main.rs`
**Purpose**: Entry point, CLI argument parsing, terminal lifecycle

**Key Functions:**
- `main()` - Entry point, initializes terminal and runs app
- `parse_args()` - Parse and validate CLI arguments
- `scan_directory_for_csvs()` - Find other CSV files in same directory
- `run()` - Main event loop

**Responsibilities:**
- Parse CLI arguments
- Initialize terminal with ratatui
- Load initial CSV file
- Run event loop (draw → poll → handle → repeat)
- Ensure terminal cleanup on exit

### `csv_data.rs`
**Purpose**: CSV data structures and file I/O

**Key Structures:**
- `CsvData` - Holds parsed CSV data (headers, rows, filename, dirty flag)

**Key Methods:**
- `from_file()` - Load CSV from file path
- `row_count()` / `column_count()` - Get dimensions
- `get_cell()` / `get_header()` - Access data
- Phase 2: `set_cell()`, `save_to_file()`
- Phase 3: `add_row()`, `delete_row()`, `add_column()`, `delete_column()`

**Responsibilities:**
- Load and parse CSV files with `csv` crate
- Store data in memory (Vec<Vec<String>>)
- Provide safe access to cells and headers
- Handle errors with anyhow::Context

### `app.rs`
**Purpose**: Application state and keyboard event handling

**Key Structures:**
- `Mode` - Application mode (Normal, Edit, Visual, Command)
- `App` - Main application state

**Key Fields:**
- `csv_data` - Loaded CSV data
- `table_state` - Current table selection (row)
- `selected_col` - Current column
- `horizontal_offset` - Horizontal scroll position
- `show_cheatsheet` - Help overlay visibility
- `csv_files` - List of CSV files in directory
- `current_file_index` - Active file index

**Key Methods:**
- `new()` - Create app with initial state
- `handle_key()` - Dispatch keyboard events by mode
- `handle_normal_mode()` - Handle navigation and commands
- `handle_navigation()` - Move cursor, scroll, jump
- `reload_current_file()` - Load different CSV file

**Responsibilities:**
- Track all mutable application state
- Handle keyboard input
- Implement navigation logic
- Manage file switching
- Phase 2: Edit mode, undo/redo

### `ui.rs`
**Purpose**: UI rendering with ratatui

**Key Functions:**
- `render()` - Main render function (called each frame)
- `render_table()` - Draw CSV table with row/column numbers
- `render_status_bar()` - Draw status bar at bottom
- `render_sheet_switcher()` - Draw file list at bottom
- `render_cheatsheet()` - Draw help overlay when active

**Helper Functions:**
- `centered_rect()` - Calculate centered area for overlays
- `column_index_to_letter()` - Convert 0→A, 1→B, 26→AA, etc.

**Responsibilities:**
- Render all UI elements using ratatui widgets
- Apply styling (bold, dim, reversed)
- Handle text truncation (max 20 chars)
- Display row numbers and column letters
- Show current cell highlight
- Layout panels (table, status, switcher)

## Code Style

### Rust Conventions
- Format with `rustfmt` (run `task fmt`)
- Lint with `clippy` (run `task clippy`)
- Document public APIs with `///` doc comments
- Use `anyhow::Result` for error handling
- Clear variable names (no abbreviations)

### Error Handling
- Use `?` operator for propagation
- Add context with `.context("description")`
- User-facing errors go to status bar (Phase 2)
- Never unwrap() in production code

### Module Organization
- Each file has single, clear responsibility
- Public API surface is minimal
- Prefer composition over inheritance

## Testing

Run tests with:
```bash
task test          # All tests
task test-verbose  # With output
```

### Test Coverage
- `csv_data.rs`: Tests for loading, cell access, edge cases
- `ui.rs`: Tests for column letter conversion
- More tests needed (contributions welcome!)

## Dependencies

See [Cargo.toml](../Cargo.toml) for full list.

**Core:**
- `ratatui 0.29` - TUI framework
- `crossterm 0.29` - Terminal control
- `csv 1.3` - CSV parsing
- `anyhow 1.0` - Error handling
- `fuzzy-matcher 0.3` - Fuzzy search (Phase 4)

## Building

```bash
task build           # Debug build
task build-release   # Optimized build
task check           # Check without building
```

## Performance

Target performance (Phase 1):
- Load 10K rows: < 100ms ✅
- Render frame: < 16ms (60 FPS) ✅
- Navigation: < 10ms response ✅

## Future Modules

As the project grows, we may add:

### Phase 2+
- `edit.rs` - Edit mode logic
- `undo.rs` - Undo/redo system

### Phase 3+
- `operations.rs` - Row/column operations
- `clipboard.rs` - Copy/paste logic

### Phase 4+
- `search.rs` - Fuzzy search implementation
- `filter.rs` - Row filtering
- `sort.rs` - Column sorting

### Phase 5+
- `excel.rs` - Excel file support with calamine
- `worksheet.rs` - Unified CSV/Excel abstraction

## Contributing

See [docs/development.md](../docs/development.md) for contribution guidelines.

**Before submitting:**
1. Run `task all` (format + lint + test)
2. Add tests for new features
3. Update documentation
4. Keep modules focused and small
