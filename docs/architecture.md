# LazyCSV Architecture

System architecture and code organization for LazyCSV.

## Overview

LazyCSV follows a clean, modular architecture:

```
┌─────────────┐
│   main.rs   │  Entry point, TUI lifecycle
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  app module │  Application state, input, navigation
└──────┬──────┘
       │
       ├───────► csv_data.rs (Data loading/storage)
       │
       └───────► ui module   (Rendering with ratatui)
```

## Core Components

### 1. Main (`main.rs`)

**Responsibility**: Application entry point and lifecycle management

```rust
fn main() -> Result<()>
    ↓
parse_args() - Validate CLI arguments
    ↓
scan_directory_for_csvs() - Find other CSV files
    ↓
CsvData::from_file() - Load initial file
    ↓
ratatui::init() - Initialize terminal
    ↓
run() - Event loop
    ↓
ratatui::restore() - Clean up terminal (always)
```

**Event Loop (in `run()`):**
```rust
loop {
    1. terminal.draw(ui::render)     // Render UI
    2. event::poll(100ms)            // Wait for input
    3. event::read()                 // Read key press
    4. app.handle_key()              // Update state
    5. if should_reload { reload() } // Switch file if needed
    6. if should_quit { break }      // Exit condition
}
```

**Design Decisions:**
- Terminal initialization/cleanup separated from app logic
- Always restore terminal (even on errors)
- 100ms poll timeout (responsive but not CPU-intensive)
- Returns `Result<()>` for error propagation

### 2. Application State (`app/` module)

**Responsibility**: Manage all mutable application state and handle user input.

The `app` module is split into three parts:
- **`mod.rs`**: Defines the central `App` struct, which is the single source of truth for all application state.
- **`input.rs`**: Handles all keyboard input, mapping keypresses to actions and managing multi-key sequences.
- **`navigation.rs`**: Contains the logic for all vim-style navigation, updating the `App` state based on user commands.

```rust
// In app/mod.rs
pub struct App {
    // Data
    csv_data: CsvData,
    csv_files: Vec<PathBuf>,
    current_file_index: usize,

    // UI State
    table_state: TableState,
    selected_col: usize,
    // ... and so on
}
```

**State Flow:**
```
KeyEvent → handle_key()
            ↓
       handle_normal_mode()
            ↓
       handle_navigation()
            ↓
       Update state (table_state, selected_col, etc.)
            ↓
       Return to event loop
            ↓
       Render updated state
```

**Design Patterns:**
- **Single source of truth**: All state in one struct
- **Immutable updates**: Methods take `&mut self`, return nothing
- **Mode dispatch**: `handle_key()` dispatches by current mode
- **Stateful widgets**: Uses ratatui's `TableState` for row tracking

### 3. CSV Data (`csv_data.rs`)

**Responsibility**: CSV data structures and file I/O

```rust
pub struct CsvData {
    headers: Vec<String>,      // Column names
    rows: Vec<Vec<String>>,    // All data rows
    filename: String,          // Original filename
    is_dirty: bool,            // Unsaved changes (Phase 2)
}
```

**Data Model:**
- **Memory-bounded**: Entire file loaded into memory
- **Simple structure**: `Vec<Vec<String>>` (row-major order)
- **Type-agnostic**: Everything stored as strings (no type inference yet)

**API Design:**
```rust
// Loading
CsvData::from_file(path) -> Result<CsvData>

// Querying
.row_count() -> usize
.column_count() -> usize
.get_cell(row, col) -> &str       // Safe, returns "" if out of bounds
.get_header(col) -> &str

// Future (Phase 2+)
.set_cell(row, col, value)
.save_to_file(path) -> Result<()>
.add_row(at: usize)
.delete_row(at: usize)
.add_column(at: usize, header)
.delete_column(at: usize)
```

**Trade-offs:**
- ✅ **Simple & Fast**: The in-memory model is simple to implement and provides very fast O(1) access for navigation.
- ❌ **High Memory Usage**: This approach is not "lazy" and is unsuitable for CSV files that are too large to fit into RAM.
- **Future Work**: A top priority is to refactor this to a true lazy-loading model that reads from disk on demand.

**Future Optimizations** (if needed):
- Virtual scrolling (load only visible rows)
- Memory-mapped files
- Chunked loading
- Type inference for columns

### 4. UI Rendering (`ui/` module)

**Responsibility**: Render all UI elements with `ratatui`.

The `ui` module is composed of several files:
- **`mod.rs`**: The main `render` function that sets up the layout and calls the other rendering modules.
- **`table.rs`**: Renders the main data table, including the virtual scrolling logic.
- **`status.rs`**: Renders the status bar and the file switcher.
- **`help.rs`**: Renders the help overlay.
- **`utils.rs`**: Contains utility functions for the UI, like `column_index_to_letter`.

```rust
// In ui/mod.rs
pub fn render(frame: &mut Frame, app: &mut App) {
    // ... setup layout ...
    table::render_table(frame, app, ...);
    status::render_status_bar(frame, app, ...);
    // ...
}
```

**Component Hierarchy:**
```
Terminal
  └─ Main Layout (Vertical)
      ├─ Table Area
      │   ├─ Border Block
      │   ├─ Column Letters Row (A, B, C...)
      │   ├─ Headers Row (#, Name, Email...)
      │   └─ Data Rows (with row numbers)
      │
      ├─ File Switcher
      │   ├─ Border Block
      │   └─ File List (► indicator)
      │
      └─ Status Bar
          └─ Status Text (position, hints)

Overlays (when active):
  └─ Help Overlay (centered)
      ├─ Border Block
      └─ Help Text (two-column)
```

**Rendering Strategy:**
- **Immediate mode**: Redraw entire UI each frame
- **Diffing**: Ratatui handles terminal diffing (only send changes)
- **Stateful widgets**: `TableState` tracks row selection
- **60 FPS**: Target < 16ms per frame

**Helper Functions:**
```rust
column_index_to_letter(0) -> "A"
column_index_to_letter(26) -> "AA"
centered_rect(60, 70, area) -> Rect    // For overlays
```

## Data Flow

### Startup Flow
```
CLI args → parse_args()
           ↓
        Scan directory
           ↓
        Load CSV file
           ↓
        Create App
           ↓
        Init terminal
           ↓
        Event loop
```

### Navigation Flow
```
User presses 'j' (down)
    ↓
crossterm reads KeyEvent { code: Down, ... }
    ↓
app.handle_key(KeyEvent)
    ↓
app.handle_normal_mode()
    ↓
app.handle_navigation(KeyCode::Down)
    ↓
app.select_next_row()
    ↓
table_state.select(Some(current + 1))
    ↓
Return to event loop
    ↓
ui::render() draws updated selection
```

### File Switching Flow
```
User presses ']' (next file)
    ↓
app.handle_key() detects file switch
    ↓
current_file_index += 1
    ↓
Returns true (signal to reload)
    ↓
main::run() calls app.reload_current_file()
    ↓
CsvData::from_file(new_path)
    ↓
app.csv_data = new_data
    ↓
Reset cursor to (0, 0)
    ↓
Return to event loop
    ↓
ui::render() draws new file
```

## Module Dependencies

```
main.rs
  ├─ uses: app, csv_data, ui, cli, file_scanner
  ├─ depends on: crossterm, ratatui, anyhow
  └─ exports: none (binary crate)

app/
  ├─ uses: csv_data
  ├─ depends on: crossterm, ratatui, anyhow
  └─ exports: App, Mode

csv_data.rs
  ├─ uses: none
  ├─ depends on: csv, anyhow
  └─ exports: CsvData

ui/
  ├─ uses: app
  ├─ depends on: ratatui
  └─ exports: render()
```

**Dependency Graph:**
```
         main.rs
        /   |   \
       /    |    \
   app/    ui/   csv_data.rs
      \    /
       \  /
      csv_data.rs
```

**Key Observation**:
- No circular dependencies
- Clear ownership: `main` owns everything
- `ui` and `app` depend on `csv_data`, but not vice versa

## Error Handling Strategy

LazyCSV uses `anyhow` for error handling:

```rust
// Propagate errors with ?
let csv_data = CsvData::from_file(path)?;

// Add context
let csv_data = CsvData::from_file(path)
    .context(format!("Failed to load {}", path.display()))?;

// Handle errors at top level (main)
fn main() -> Result<()> {
    // ... on error, anyhow displays full error chain
}
```

**Error Flow:**
```
csv::Reader::from_path() fails
    ↓
CsvData::from_file() adds context
    ↓
Propagated with ?
    ↓
main() returns Result
    ↓
anyhow displays: "Failed to load file.csv: No such file or directory"
```

## Performance Characteristics

### Time Complexity

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Load file | O(n) | n = total cells |
| Navigate | O(1) | Just update index |
| Render | O(v) | v = visible cells (~200) |
| Search (Phase 4) | O(n) | Full table scan |
| Sort (Phase 4) | O(n log n) | Standard sort |

### Space Complexity

| Structure | Memory | Notes |
|-----------|--------|-------|
| CSV data | O(n) | n = total cells |
| UI state | O(1) | Fixed size |
| Render buffer | O(v) | v = visible cells |

### Performance Targets

✅ Achieved in Phase 1:
- File loading: < 100ms for 10K rows
- Frame rendering: < 16ms (60 FPS)
- Navigation: < 10ms response

## Thread Model

**Current: Single-threaded**

```
Main Thread:
  ├─ Terminal rendering
  ├─ Event polling (100ms timeout)
  ├─ Keyboard handling
  ├─ State updates
  └─ File I/O (synchronous)
```

**Why single-threaded?**
- ✅ Simpler (no sync primitives needed)
- ✅ Sufficient for keyboard input (low latency)
- ✅ CSV loading is fast enough
- ✅ Rendering is fast enough

**Future: Multi-threaded (if needed)**

Potential uses:
- Background file loading (large files)
- Async search (massive datasets)
- Real-time file watching
- Parallel sort (Phase 4)

## Testing Strategy

### Unit Tests
```rust
// csv_data.rs
#[test]
fn test_load_valid_csv() { ... }
#[test]
fn test_get_cell_out_of_bounds() { ... }

// ui.rs
#[test]
fn test_column_index_to_letter() { ... }
```

### Integration Tests
```rust
// tests/integration_test.rs
#[test]
fn test_load_and_navigate() {
    let csv = CsvData::from_file(path)?;
    let app = App::new(csv, ...);
    app.handle_key(...);
    assert_eq!(app.selected_row(), Some(1));
}
```

### Manual Tests
- Load various CSV files (small, large, wide, malformed)
- Test all navigation keys
- Test file switching
- Test help overlay
- Test edge cases (empty file, single row, single column)

## Future Architecture (Phases 2-5)

### Phase 2: Edit Mode
```rust
// New modules
mod undo;     // Undo/redo system

// Enhanced structures
pub enum Mode {
    Normal,
    Edit { buffer: String, original: String },  // Track edits
}

pub struct UndoStack {
    history: Vec<Command>,
    current: usize,
}
```

### Phase 3: Operations
```rust
// New modules
mod operations;   // Row/column operations
mod clipboard;    // Copy/paste

// Command pattern
trait Command {
    fn execute(&mut self, data: &mut CsvData);
    fn undo(&mut self, data: &mut CsvData);
}
```

### Phase 4: Search & Filter
```rust
// New modules
mod search;   // Fuzzy search with fuzzy-matcher
mod filter;   // Row filtering
mod sort;     // Column sorting

// Search state
pub struct SearchState {
    query: String,
    matches: Vec<Match>,
    current: usize,
}
```

### Phase 5: Excel Support
```rust
// New modules
mod excel;      // Excel file support with calamine
mod worksheet;  // Unified CSV/Excel abstraction

// Worksheet abstraction
pub enum Worksheet {
    Csv(PathBuf),
    Excel(PathBuf, String),  // file + sheet name
}
```

## Code Quality Standards

### Style
- `rustfmt` - Automatic formatting
- `clippy` - Linting (run with `-D warnings`)
- Clear variable names (no abbreviations)
- Document public APIs with `///` doc comments

### Performance
- Profile with `cargo flamegraph`
- Benchmark with `cargo bench`
- Target: 60 FPS (< 16ms per frame)

### Safety
- No `unwrap()` in production code
- Use `?` for error propagation
- Safe cell access (return "" if out of bounds)

## Deployment

### Binary Size
```bash
cargo build --release
strip target/release/lazycsv  # Remove debug symbols
# Result: ~5-8 MB (static binary)
```

### Distribution
- Crates.io: `cargo install lazycsv`
- GitHub Releases: Pre-built binaries
- Package managers: Homebrew, AUR, etc.

## Resources

- **Ratatui**: https://ratatui.rs/
- **Crossterm**: https://docs.rs/crossterm/
- **CSV crate**: https://docs.rs/csv/
- **Anyhow**: https://docs.rs/anyhow/

## Contributing

See [development.md](development.md) for contribution guidelines.

## License

GPL License - see [LICENSE](../LICENSE) for details.
