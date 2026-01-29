# Source Code

This directory contains the Rust source code for LazyCSV.

## Structure

```
src/
├── main.rs         - Entry point & TUI lifecycle
├── lib.rs          - Main library crate
├── cli.rs          - CLI argument parsing
├── file_scanner.rs - Scans for CSV files
├── csv_data.rs     - CSV data loading and storage
│
├── app/
│   ├── mod.rs      - Core application state (`App` struct)
│   ├── input.rs    - Keyboard input handling
│   └── navigation.rs - Vim-style navigation logic
│
└── ui/
    ├── mod.rs      - Main UI rendering entrypoint
    ├── table.rs    - CSV table rendering (with virtual scrolling)
    ├── status.rs   - Status bar and file switcher UI
    ├── help.rs     - Help/cheatsheet overlay
    └── utils.rs    - UI helper functions
```

## Modules

### `main.rs`
**Purpose**: Entry point, TUI initialization, and main event loop.

**Responsibilities:**
- Initialize the terminal and `ratatui` backend.
- Parse CLI arguments using the `cli` module.
- Scan for CSV files using the `file_scanner` module.
- Create the main `App` state.
- Run the main event loop, handling events and drawing the UI.
- Ensure proper terminal cleanup on exit.

### `cli.rs`
**Purpose**: Handles command-line argument parsing.

**Responsibilities:**
- Defines the CLI arguments using `clap`.
- Parses the arguments provided when the application is started.
- Determines the initial file or directory to open.

### `file_scanner.rs`
**Purpose**: Finds CSV files in a given directory.

**Responsibilities:**
- Scans a directory for files with `.csv` or `.CSV` extensions.
- Ignores subdirectories.
- Sorts the found files alphabetically.

### `csv_data.rs`
**Purpose**: Manages loading, storing, and accessing CSV data.

**Key Structures:**
- `CsvData` - Holds the file path, headers, and all rows of a CSV file.

**Responsibilities:**
- Loads a CSV file from a given path using the `csv` crate.
- Stores the entire dataset in memory as a `Vec<Vec<String>>`.
- Provides safe methods to access headers and cell data.
- Tracks whether the data has been modified (the `is_dirty` flag).

### `app/` Module
**Purpose**: Manages the application's state and logic.

#### `app/mod.rs`
**Purpose**: Defines the core application state.
- **`App` struct**: The single source of truth for the application's state. It holds the CSV data, selection state, scroll offsets, current mode, file list, and more.
- **`Mode` enum**: Defines the different application modes (e.g., `Normal`).

#### `app/input.rs`
**Purpose**: Handles all keyboard input.
- It receives key events and translates them into actions.
- It manages multi-key command sequences (e.g., `gg`, `zz`) with timeouts.
- It handles numeric prefixes for commands (e.g., `5j`).
- It dispatches to the `navigation` module for movement commands.

#### `app/navigation.rs`
**Purpose**: Implements all vim-style navigation logic.
- It contains functions for moving the cursor (`move_up_by`, `move_down_by`, etc.).
- It handles jumping to specific lines (`goto_line`, `goto_first_row`).
- It manages the viewport and scrolling.

### `ui/` Module
**Purpose**: Handles all UI rendering using `ratatui`.

#### `ui/mod.rs`
**Purpose**: The main entrypoint for rendering.
- The `render` function sets up the main layout and calls the other UI modules to draw their respective components.

#### `ui/table.rs`
**Purpose**: Renders the main CSV data table.
- It displays the data in a grid with row numbers and column letters.
- **Virtual Scrolling**: For performance, it only renders the rows and columns that are currently visible in the viewport.
- It highlights the selected row and cell.

#### `ui/status.rs`
**Purpose**: Renders the UI components at the bottom of the screen.
- `render_status_bar`: Draws the status bar, which shows information about the current selection, file, and any status messages.
- `render_sheet_switcher`: Draws the list of open CSV files, allowing the user to see which files are open and which is active.

#### `ui/help.rs`
**Purpose**: Renders the help/cheatsheet overlay.
- It displays a centered popup with a list of keybindings.

#### `ui/utils.rs`
**Purpose**: Contains helper functions for the UI module.
- `column_index_to_letter`: Converts a column index (0, 1, 2...) to its spreadsheet-style letter representation (A, B, C...).

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
The project has a comprehensive test suite with 133 tests.
- **Unit Tests**: Found alongside the code in each module.
- **Integration Tests**: Located in the `tests/` directory, covering workflows, UI rendering, and edge cases.
- `cli_test.rs`: Tests command-line argument parsing.
- `csv_data_test.rs` & `csv_edge_cases_test.rs`: Test CSV loading and data handling.
- `app_test.rs` & `navigation_workflows_test.rs`: Test application logic and user workflows.
- `ui_rendering_test.rs` & `ui_state_test.rs`: Test UI components and state changes.

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

## Future Development

The following are key areas for future development:

- **True Lazy Loading**: The highest priority is to re-engineer `csv_data.rs` to not load the entire file into memory. This will likely involve using a streaming parser and an indexing mechanism to fetch rows on demand, fulfilling the "lazy" promise.
- **Cell Editing**: Implementing `edit.rs` and the `Edit` mode in `app/mod.rs` to allow users to modify cell content and save changes back to the file. This will also require an undo/redo system (`undo.rs`).
- **Row/Column Operations**: Adding functionality to add, delete, and reorder rows and columns.
- **Search and Filter**: Implementing fuzzy search (`search.rs`), column sorting, and row filtering.
- **Excel Support**: Adding support for reading `.xlsx` files, likely using a crate like `calamine`.

## Contributing

See [docs/development.md](../docs/development.md) for contribution guidelines.

**Before submitting:**
1. Run `task all` (format + lint + test)
2. Add tests for new features
3. Update documentation
4. Keep modules focused and small
