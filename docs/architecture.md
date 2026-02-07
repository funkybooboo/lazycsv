# LazyCSV Architecture

System architecture and code organization for LazyCSV.

Before contributing, it's highly recommended to familiarize yourself with the architecture outlined here. This will help you understand where your changes fit into the bigger picture. For the full contribution process, see the [Development Guide](development.md).

## Overview

LazyCSV follows a clean, modular architecture with strong type safety (v0.3.2 Complete):

```
┌─────────────┐
│   main.rs   │  Entry point, TUI lifecycle
└──────┬──────┘
       │
       ▼
┌──────────────────────────────────────────┐
│  app module  │  Application coordinator  │
└──────┬───────┴──────────────────────────┬┘
       │                                   │
       ├───► domain (RowIndex, ColIndex)   │
       ├───► input (Actions, State)        │
       ├───► csv (Document)                │
       ├───► session (Multi-file)          │
       ├───► navigation (Commands)         │
       ├───► file_system (Discovery)       │
       └───► ui (Rendering, ViewState)     │
```

**Key Changes in v0.2.0:**
- `domain/` - Type-safe position types
- `input/` - Action abstraction layer, InputState
- `csv/` - Document (renamed from csv_data)
- `session/` - Multi-file session management (NEW)
- `navigation/` - Movement commands (extracted)
- `file_system/` - File discovery (extracted)
- `ui/` - ViewState (renamed from UiState)

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

The `app` module is the central coordinator, bringing together all other components. It defines the main `App` struct, which holds the application's state.

### App State Structure (v0.2.0)

The `App` struct has been refactored to be minimal and well-organized:

```rust
// In app/mod.rs (v0.2.0 Complete)
pub struct App {
    /// CSV document data
    pub document: Document,

    /// UI view state (selection, scroll, viewport)
    pub view_state: ViewState,

    /// Input state (pending commands, count prefixes)
    pub input_state: InputState,

    /// Multi-file session management
    pub session: Session,

    /// Whether the application should quit
    pub should_quit: bool,

    /// Optional status message to display
    pub status_message: Option<StatusMessage>,
}
```

#### Document Structure

```rust
pub struct Document {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub filename: String,
    pub is_dirty: bool,
}
```

#### ViewState Structure

```rust
pub struct ViewState {
    pub table_state: TableState,           // Ratatui table state
    pub selected_column: ColIndex,         // Current column
    pub column_scroll_offset: usize,       // Horizontal scroll
    pub help_overlay_visible: bool,        // Help shown?
    pub viewport_mode: ViewportMode,       // Viewport positioning
}
```

#### InputState Structure (NEW in v0.2.0)

```rust
pub struct InputState {
    pending_command: Option<PendingCommand>,  // Multi-key command state
    command_count: Option<NonZeroUsize>,      // Count prefix (e.g., "5" in "5j")
    pending_command_time: Option<Instant>,    // Timeout tracking
}
```

#### Session Structure (NEW in v0.2.0)

```rust
pub struct Session {
    files: Vec<PathBuf>,              // All CSV files in directory
    active_file_index: usize,         // Current file
    config: FileConfig,               // File parsing config
}

pub struct FileConfig {
    pub delimiter: u8,
    pub no_headers: bool,
    pub encoding: Option<String>,
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

### 5. CSV Document (`csv/` module)

**Responsibility**: CSV data structures and file I/O

**Location:** `src/csv/document.rs` (renamed from `csv_data.rs` in v0.2.0)

```rust
pub struct Document {
    pub headers: Vec<String>,      // Column names
    pub rows: Vec<Vec<String>>,    // All data rows
    pub filename: String,          // Original filename
    pub is_dirty: bool,            // Unsaved changes (v0.4.0+)
}
```

**Data Model:**
- **Memory-bounded**: Entire file loaded into memory
- **Simple structure**: `Vec<Vec<String>>` (row-major order)
- **Type-agnostic**: Everything stored as strings (no type inference yet)

**API Design (v0.2.0 - Type-safe):**
```rust
// Loading
Document::from_file(path) -> Result<Document>

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
session.next_file()
    ↓
Returns true (signal to reload)
    ↓
main::run() calls app.reload_current_file()
    ↓
Document::from_file(new_path)
    ↓
app.document = new_document
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
  └─> app::App
       ├─> csv::Document         (CSV data)
       ├─> ui::ViewState         (UI state)
       ├─> input::InputState     (Input state)
       ├─> session::Session      (Multi-file session)
       ├─> input::handler        (Key handling)
       ├─> navigation::commands  (Movement)
       └─> ui::render            (UI rendering)

input::handler
  ├─> input::actions   (UserAction, NavigateAction)
  ├─> input::state     (InputState management)
  └─> app::messages    (User-facing strings)

navigation::commands
  └─> domain::position (RowIndex, ColIndex)

ui::render
  ├─> ui::table        (Table rendering)
  ├─> ui::status       (Status bar)
  ├─> ui::help         (Help overlay)
  └─> ui::view_state   (ViewState)

session::Session
  └─> file_system::discovery (CSV file scanning)

csv::Document
  └─> domain::position (RowIndex, ColIndex)
```

**Key Design Decisions:**
- **App** is a thin coordinator, delegates to specialized modules
- **Clear separation:** Input handling → Actions → State changes
- **Type safety:** RowIndex/ColIndex prevent coordinate bugs at compile-time
- **Single responsibility:** Each module has one clear purpose
- **No circular dependencies:** Clean layering maintained

**Module Structure (v0.2.0):**
```
src/
├── domain/            # Domain types (RowIndex, ColIndex, Position)
├── input/             # Input handling
│   ├── actions.rs     # UserAction, NavigateAction, ViewportAction
│   ├── state.rs       # InputState (pending commands, counts)
│   └── handler.rs     # Input event handling
├── navigation/        # Navigation commands
│   └── commands.rs    # Vim-style movement functions
├── session/           # Multi-file session management
│   └── mod.rs         # Session, FileConfig
├── csv/               # CSV data operations
│   └── document.rs    # Document struct (CSV loading/parsing)
├── file_system/       # File operations
│   └── discovery.rs   # CSV file scanning
├── app/               # Application coordinator
│   ├── mod.rs         # App struct, main loop
│   └── messages.rs    # User-facing message strings
└── ui/                # UI rendering
    ├── mod.rs         # Main render function
    ├── view_state.rs  # ViewState (viewport control)
    ├── table.rs       # Table rendering with virtual scrolling
    ├── status.rs      # Status bar and file switcher
    ├── help.rs        # Help overlay
    └── utils.rs       # Utility functions
```

## v0.2.0 Refactoring Summary

The v0.2.0 release was a major refactor to improve code quality, maintainability, and type safety. The work was completed over several point releases:

**v0.2.1: Type Safety Foundation**
- Introduced RowIndex/ColIndex newtypes to prevent coordinate bugs
- Created UserAction abstraction layer for all input handling
- Eliminated primitive obsession with semantic types (NonZeroUsize, StatusMessage)

**v0.2.2: Separation of Concerns**
- Extracted InputState from App (pending commands, count prefixes)
- Extracted Session management (multi-file, file config)
- Renamed UiState → ViewState for clarity

**v0.2.3: Better Naming & Consistency**
- Renamed csv_data → Document throughout codebase
- Renamed ui → view_state for consistency
- Standardized function naming: get_*, move_*, goto_*
- Centralized user messages in app/messages.rs

**v0.2.4: Code Organization**
- Reorganized modules: csv/, file_system/, session/, navigation/
- Defined clear module boundaries and public APIs
- Reduced App struct from 12 fields to 6 fields

**v0.2.5: Clean Code Improvements**
- Decomposed long functions (render_table: 180 → 74 lines)
- Removed all magic numbers, replaced with named constants
- Added comprehensive module-level documentation
- Removed all commented-out dead code

**v0.2.6: Testing & Validation**
- Expanded test suite from 133 to 257 tests (+124 new tests)
- Added z-command tests, timeout tests, navigation unit tests
- Zero compiler warnings
- Zero clippy warnings
- (v0.3.2 expanded to 344 tests)

**Result:** Clean, maintainable, type-safe architecture ready for future feature development. All internal refactoring with no user-facing changes.

## v0.3.0-v0.3.2 Feature Summary

### v0.3.0: Advanced Navigation
- Row jumping: `gg`, `G`, `<number>G`
- Column jumping: via command mode
- Command mode: `:` prefix
- Count prefixes: `5j` moves down 5 rows
- Word motion: `w`, `b`, `e`
- Viewport control: `zt`, `zz`, `zb`

### v0.3.1: UI/UX Polish
- Mode indicator display
- Transient message system
- File list horizontal scrolling
- Redesigned help overlay

### v0.3.2: Pre-Edit Polish (271+ tests)

**UI Redesign:**
- Minimal borders (horizontal rules only)
- Vim-like status line: `NORMAL 3,C "cell value"`
- Auto-width columns (8-50 char range)
- Current row indicator: `>`

**Command Mode Improvements:**
- `:c` command for column navigation (`:c A`, `:c 5`, `:c AA`)
- Reserved commands: `:q`, `:w`, `:h` take priority
- Out-of-bounds errors (not silent clamping)

**Input Handling:**
- No timeout on pending commands (vim-like)
- Pending command display in status bar

**Mode Preparation:**
```rust
pub enum Mode {
    Normal,      // Default mode for navigation
    Insert,      // Quick single-cell editing (v0.4.0)
    Magnifier,   // Full vim editor for cell (v0.5.0)
    HeaderEdit,  // Edit column headers (v0.9.0)
    Visual,      // Select rows/cells/blocks (v0.6.0)
    Command,     // Execute commands via `:` prefix
}

pub struct EditBuffer {
    pub content: String,   // Current content being edited
    pub cursor: usize,     // Cursor position within content
    pub original: String,  // Original content for cancel/undo
}
```

## Error Handling Strategy

LazyCSV uses `anyhow` for error handling:

```rust
// Propagate errors with ?
let document = Document::from_file(path)?;

// Add context
let document = Document::from_file(path)
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

**v0.2.1 ✅ COMPLETED:**
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

**v0.2.2-v0.2.6 ✅ COMPLETED:**
```rust
// v0.2.2: Separation of Concerns
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
- **v0.2.3**: Better Naming & Consistency (e.g., `Document`, `ViewState`)
- **v0.2.4**: Code Organization (clear module boundaries)
- **v0.2.5**: Clean Code Improvements (long functions decomposed, magic numbers removed)
- **v0.2.6**: Testing & Validation (test suite expanded, zero warnings)

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
