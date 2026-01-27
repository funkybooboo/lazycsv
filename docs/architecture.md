# LazyCSV Architecture

System architecture and code organization for LazyCSV.

## Overview

LazyCSV follows a clean, modular architecture:

```
┌─────────────┐
│   main.rs   │  Entry point, CLI, terminal lifecycle
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   app.rs    │  Application state, keyboard handling
└──────┬──────┘
       │
       ├───────► csv_data.rs   (CSV data structures)
       │
       └───────► ui.rs          (Rendering with ratatui)
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

### 2. Application State (`app.rs`)

**Responsibility**: Manage all mutable application state

```rust
pub struct App {
    // Data
    csv_data: CsvData,              // Loaded CSV
    csv_files: Vec<PathBuf>,        // Available files
    current_file_index: usize,      // Active file

    // UI State
    table_state: TableState,        // Row selection (from ratatui)
    selected_col: usize,            // Column selection
    horizontal_offset: usize,       // Scroll position
    show_cheatsheet: bool,          // Help visibility
    mode: Mode,                     // Current mode

    // Feedback
    status_message: Option<String>, // Temp status message
    should_quit: bool,              // Exit flag
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
- ✅ Simple: Easy to understand and debug
- ✅ Fast: O(1) random access
- ❌ Memory: Not suitable for massive files (100K+ rows)
- ❌ Types: No numeric operations (yet)

**Future Optimizations** (if needed):
- Virtual scrolling (load only visible rows)
- Memory-mapped files
- Chunked loading
- Type inference for columns

### 4. UI Rendering (`ui.rs`)

**Responsibility**: Render all UI elements with ratatui

```rust
pub fn render(frame: &mut Frame, app: &mut App) {
    // Layout
    let chunks = Layout::vertical([
        Min(0),         // Table
        Length(3),      // File switcher
        Length(3),      // Status bar
    ]).split(frame.area());

    // Render components
    render_table(frame, app, chunks[0]);
    render_sheet_switcher(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);

    // Conditional overlays
    if app.show_cheatsheet {
        render_cheatsheet(frame);
    }
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
  ├─ uses: app, csv_data, ui
  ├─ depends on: crossterm, ratatui, anyhow
  └─ exports: none (binary crate)

app.rs
  ├─ uses: csv_data
  ├─ depends on: crossterm, ratatui, anyhow
  └─ exports: App, Mode

csv_data.rs
  ├─ uses: none
  ├─ depends on: csv, anyhow
  └─ exports: CsvData

ui.rs
  ├─ uses: app
  ├─ depends on: ratatui
  └─ exports: render()
```

**Dependency Graph:**
```
         main.rs
        /   |   \
       /    |    \
   app.rs  ui.rs  csv_data.rs
      |      |
      |      |
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
