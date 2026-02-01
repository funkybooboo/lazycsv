# LazyCSV Architecture

System architecture and code organization for LazyCSV.

## Overview

LazyCSV follows a clean, modular architecture with strong type safety (v0.2.0 Phase 1):

```
┌─────────────┐
│   main.rs   │  Entry point, TUI lifecycle
└──────┬──────┘
       │
       ▼
┌──────────────────────────────────────────┐
│  app module  │  Application state & logic│
└──────┬───────┴──────────────────────────┬┘
       │                                   │
       ├───► domain (RowIndex, ColIndex)   │
       ├───► input  (UserAction, Result)   │
       ├───► csv_data (Type-safe access)   │
       └───► ui     (Rendering)            │
```

**New in v0.2.0 Phase 1:**
- `domain/` - Position types for type safety
- `input/` - Action enums and semantic types
- Type-safe APIs throughout (no raw usize for positions)

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

### 2. Domain Types (`domain/` module) **NEW in v0.2.0**

**Responsibility**: Core domain types for type safety

The `domain` module provides newtype wrappers to prevent type confusion:

```rust
// In domain/position.rs
pub struct RowIndex(usize);      // Can't confuse with ColIndex
pub struct ColIndex(usize);       // Can't confuse with RowIndex
pub struct Position { row: RowIndex, col: ColIndex }
```

**Key Methods:**
```rust
RowIndex::new(5)                  // Create from usize
row.get()                         // Extract usize
row.saturating_add(3)             // Safe arithmetic
row.to_line_number()              // Convert to 1-based NonZeroUsize
```

**Type Safety Benefits:**
- ✅ Compiler prevents swapping row/column parameters
- ✅ Self-documenting APIs (clear which parameter is which)
- ✅ Zero runtime cost (newtypes are compile-time only)

### 3. Input Actions (`input/` module) **NEW in v0.2.0**

**Responsibility**: Type-safe action representation

The `input` module defines semantic action types:

```rust
// In input/actions.rs
pub enum InputResult {
    Continue,       // Normal operation
    ReloadFile,     // Switch to different file
    Quit,           // Exit application
}

pub enum UserAction {
    Navigate(NavigateAction),
    ViewportControl(ViewportAction),
    ToggleHelp,
    Quit { force: bool },
    SwitchFile(FileDirection),
}

pub enum PendingCommand {
    G,  // Waiting for 'gg'
    Z,  // Waiting for 'zt', 'zz', 'zb'
}
```

**Improvements Over Old Design:**
- ❌ Old: `handle_key() -> Result<bool>`  (unclear what `true` means)
- ✅ New: `handle_key() -> Result<InputResult>` (semantic, self-documenting)

### 4. Application State (`app/` module)

**Responsibility**: Manage all mutable application state and handle user input.

The `app` module is split into three parts:
- **`mod.rs`**: Defines the central `App` struct (updated with type-safe fields)
- **`input.rs`**: Handles keyboard input using new action types
- **`navigation.rs`**: Navigation logic with type-safe position handling

```rust
// In app/mod.rs (v0.2.0 Phase 1)
pub struct App {
    // Data
    csv_data: CsvData,
    csv_files: Vec<PathBuf>,
    current_file_index: usize,

    // UI State (type-safe)
    ui: UiState,
    mode: Mode,

    // Input state (improved types)
    pending_key: Option<PendingCommand>,        // Was: Option<KeyCode>
    command_count: Option<NonZeroUsize>,         // Was: Option<String>
    status_message: Option<StatusMessage>,       // Was: Option<Cow<'static, str>>

    // Config
    delimiter: Option<u8>,
    no_headers: bool,
    encoding: Option<String>,
}

pub struct UiState {
    table_state: TableState,
    selected_col: ColIndex,                      // Was: usize
    horizontal_offset: usize,
    show_cheatsheet: bool,
    viewport_mode: ViewportMode,
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

### 5. CSV Data (`csv_data.rs`)

**Responsibility**: CSV data structures and file I/O

```rust
pub struct CsvData {
    headers: Vec<String>,      // Column names
    rows: Vec<Vec<String>>,    // All data rows
    filename: String,          // Original filename
    is_dirty: bool,            // Unsaved changes (v0.4.0+)
}
```

**Data Model:**
- **Memory-bounded**: Entire file loaded into memory
- **Simple structure**: `Vec<Vec<String>>` (row-major order)
- **Type-agnostic**: Everything stored as strings (no type inference yet)

**API Design (v0.2.0 - Type-safe):**
```rust
// Loading
CsvData::from_file(path) -> Result<CsvData>

// Querying (type-safe in v0.2.0)
.row_count() -> usize
.column_count() -> usize
.get_cell(row: RowIndex, col: ColIndex) -> &str  // ✅ Type-safe!
.get_header(col: ColIndex) -> &str                // ✅ Type-safe!

// Future (v0.4.0+)
.set_cell(row: RowIndex, col: ColIndex, value: String)
.save_to_file(path) -> Result<()>
.add_row(at: RowIndex)
.delete_row(at: RowIndex)
.add_column(at: ColIndex, header: String)
.delete_column(at: ColIndex)
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

### 6. UI Rendering (`ui/` module)

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

## Module Dependencies (v0.2.0)

```
main.rs
  ├─ uses: app, csv_data, ui, cli, file_scanner, domain, input
  ├─ depends on: crossterm, ratatui, anyhow
  └─ exports: none (binary crate)

domain/
  ├─ uses: none (pure types)
  ├─ depends on: std only
  └─ exports: RowIndex, ColIndex, Position

input/
  ├─ uses: domain (for action types)
  ├─ depends on: crossterm, std
  └─ exports: UserAction, InputResult, PendingCommand, etc.

app/
  ├─ uses: csv_data, domain, input
  ├─ depends on: crossterm, ratatui, anyhow
  └─ exports: App, Mode, UiState

csv_data.rs
  ├─ uses: domain (RowIndex, ColIndex)
  ├─ depends on: csv, anyhow, encoding_rs
  └─ exports: CsvData

ui/
  ├─ uses: app, domain
  ├─ depends on: ratatui
  └─ exports: render()
```

**Dependency Graph (v0.2.0):**
```
         main.rs
        /   |   \
       /    |    \
   app/   ui/   csv_data
    /  \   |     /
   /    \ |    /
domain  input
  \      /
   \    /
  (shared)
```

**Key Observations**:
- ✅ No circular dependencies
- ✅ Clear layering: domain → input → app → main
- ✅ `domain` is pure (no dependencies)
- ✅ `input` uses domain types
- ✅ Everything uses domain for type safety

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
| Search (v1.1.0) | O(n) | Full table scan |
| Sort (v1.2.0) | O(n log n) | Standard sort |

### Space Complexity

| Structure | Memory | Notes |
|-----------|--------|-------|
| CSV data | O(n) | n = total cells |
| UI state | O(1) | Fixed size |
| Render buffer | O(v) | v = visible cells |

### Performance Targets

✅ Achieved in v0.1.0:
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
- Parallel sort (v1.2.0)

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

## Future Architecture (v0.2.0 - v1.6.2)

### Version 0.2.0: Type System & State Refactoring

**Phase 1 ✅ COMPLETED:**
```rust
// Type-safe position types
pub struct RowIndex(usize);
pub struct ColIndex(usize);
pub struct Position { row: RowIndex, col: ColIndex }

// Semantic action types
pub enum InputResult { Continue, ReloadFile, Quit }
pub enum UserAction {
    Navigate(NavigateAction),
    ViewportControl(ViewportAction),
    ToggleHelp,
    Quit { force: bool },
    SwitchFile(FileDirection),
}
pub enum PendingCommand { G, Z }

// Improved primitives
command_count: Option<NonZeroUsize>  // Was: Option<String>
status_message: Option<StatusMessage> // Was: Option<Cow<'static, str>>
```

**Phases 2-6 (TODO):**
```rust
// Phase 2: Separation of Concerns
pub struct InputState {
    pending_command: Option<PendingCommand>,
    command_count: Option<NonZeroUsize>,
    pending_command_time: Option<Instant>,
}

pub struct Session {
    files: Vec<PathBuf>,
    current_file_index: usize,
    config: FileConfig,
}
```

### Version 0.4.0: Quick Edit Mode
```rust
// Enhanced Mode enum
pub enum Mode {
    Normal,
    Insert { buffer: String, cursor: usize },
}

// App additions
pub struct App {
    edit_buffer: String,
    cursor_position: usize,
}
```

### Version 0.5.0: Vim Magnifier
```rust
// New mode for power editing
pub enum Mode {
    Normal,
    Insert { ... },
    Magnifier { vim_buffer: VimBuffer },  // Embedded vim editor
}

// Potential integration with ratatui-vim or custom implementation
pub struct VimBuffer {
    content: String,
    vim_state: VimState,  // Normal/Insert mode within magnifier
}
```

### Version 0.6.0: Persistence & Guards
```rust
// Commands
pub enum Command {
    Save,
    Quit { force: bool },
    SaveAndQuit,
}

// Dirty tracking (already in CsvData)
pub struct CsvData {
    is_dirty: bool,
    // ...
}
```

### Version 0.7.0-0.9.0: Row, Column, Header Operations
```rust
// Command pattern for undo/redo
trait Operation {
    fn execute(&mut self, data: &mut CsvData) -> Result<()>;
    fn undo(&mut self, data: &mut CsvData) -> Result<()>;
}

pub enum OperationType {
    EditCell { row: RowIndex, col: ColIndex, old: String, new: String },
    AddRow { at: RowIndex },
    DeleteRow { at: RowIndex, data: Vec<String> },
    AddColumn { at: ColIndex, header: String },
    DeleteColumn { at: ColIndex, header: String, data: Vec<String> },
    EditHeader { col: ColIndex, old: String, new: String },
    ToggleHeaders { had_headers: bool },
}
```

### Version 1.0.0-1.0.1: Command History & Marks
```rust
pub struct CommandHistory {
    operations: Vec<Box<dyn Operation>>,
    current: usize,  // Position in history
    max_size: usize, // 100 operations
}

impl CommandHistory {
    fn push(&mut self, op: Box<dyn Operation>);
    fn undo(&mut self, data: &mut CsvData) -> Option<String>; // Returns description
    fn redo(&mut self, data: &mut CsvData) -> Option<String>;
}
```

### Version 1.1.0-1.1.1: Search & Visual
```rust
// New modules
mod search;   // Fuzzy search with fuzzy-matcher

pub struct SearchState {
    query: String,
    matches: Vec<Match>,
    current: usize,
}

pub struct VisualSelection {
    start: CellPosition,
    end: CellPosition,
    mode: VisualMode,  // Cell or Line
}
```

### Version 1.2.0-1.2.1: Sorting & Filtering
```rust
mod filter;
mod sort;

pub struct Filter {
    column: ColIndex,
    operator: FilterOperator,
    value: String,
}

pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
    StartsWith,
    EndsWith,
}
```

### Version 1.4.1: Session Persistence
```rust
pub struct SessionState {
    cursor_position: CellPosition,
    scroll_offset: (usize, usize),
    sort_order: Option<(ColIndex, SortDirection)>,
    filters: Vec<Filter>,
    frozen_columns: usize,
}

// Saved to ~/.cache/lazycsv/<file_hash>.session
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
