# LazyCSV Architecture

System architecture and code organization for LazyCSV.

Before contributing, it's highly recommended to familiarize yourself with the architecture outlined here. This will help you understand where your changes fit into the bigger picture. For the full contribution process, see the [Development Guide](development.md).

## Overview

LazyCSV follows a clean, modular architecture with strong type safety (v0.4.0 Complete):

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

### 2. Domain Types (`domain/` module) **NEW in v0.2.0, Enhanced in v0.2.1**

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
- [ACHIEVED] Compiler prevents swapping row/column parameters
- [ACHIEVED] Self-documenting APIs (clear which parameter is which)
- [ACHIEVED] Zero runtime cost (newtypes are compile-time only)

**Design Decisions (v0.2.1):**

**Why Newtypes Over Type Aliases?**
```rust
// ❌ Type alias - no compile-time safety
type RowIndex = usize;
type ColIndex = usize;
fn get_cell(row: RowIndex, col: ColIndex) { }
get_cell(col, row);  // ❌ Compiles! Bug at runtime!

// [ACHIEVED] Newtype - compile-time safety
struct RowIndex(usize);
struct ColIndex(usize);
fn get_cell(row: RowIndex, col: ColIndex) { }
get_cell(col, row);  // ❌ Compile error! Bug caught at build time!
```

**Why Saturation Arithmetic?**

LazyCSV uses saturation arithmetic for position types instead of wrapping or panicking:

```rust
// Saturation at boundaries
RowIndex::new(5).saturating_sub(10)  // → RowIndex(0), not panic
RowIndex::MAX.saturating_add(1)      // → RowIndex(MAX), not overflow

// Benefit: Navigation commands never panic
// User presses "k" (up) 100 times at row 0 → stays at row 0
// User presses "j" (down) past end → clamps to last row
```

**Rationale:**
1. **Safety:** No panics from user navigation commands
2. **UX:** Intuitive behavior (can't scroll past boundaries)
3. **Simplicity:** No need for bounds checking at every call site
4. **Performance:** Saturation is as fast as wrapping on modern CPUs

**Property-Based Testing (v0.2.1):**

The domain types are verified with 29 property-based tests using `proptest`:

```rust
// Example properties verified:
// 1. Reversibility: from(x).get() == x
// 2. Associativity: (a + b) + c == a + (b + c)
// 3. Identity: x + 0 == x
// 4. Saturation: 0 - 1 == 0, MAX + 1 == MAX
// 5. Ordering: if a < b then a.cmp(b) == Less
```

This provides mathematical proof that the type safety guarantees hold across all possible inputs.

See `src/domain/position_proptests.rs` for the full test suite.

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
-  Old: `handle_key() -> Result<bool>`  (unclear what `true` means)
-  New: `handle_key() -> Result<InputResult>` (semantic, self-documenting)

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
.get_cell(row: RowIndex, col: ColIndex) -> &str  //  Type-safe!
.get_header(col: ColIndex) -> &str                //  Type-safe!

// Future (v0.4.0+)
.set_cell(row: RowIndex, col: ColIndex, value: String)
.save_to_file(path) -> Result<()>
.add_row(at: RowIndex)
.delete_row(at: RowIndex)
.add_column(at: ColIndex, header: String)
.delete_column(at: ColIndex)
```

**Trade-offs:**
-  **Simple & Fast**: The in-memory model is simple to implement and provides very fast O(1) access for navigation.
-  **High Memory Usage**: This approach is not "lazy" and is unsuitable for CSV files that are too large to fit into RAM.
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

### Navigation Pipeline (v0.3.1 - Detailed)

The navigation system is designed for sub-microsecond response times with count prefix support.

```
┌─────────────────────────────────────────────────────────────┐
│ User Input: "5j" (move down 5 rows)                         │
└────────────────────────┬────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ input::handler::handle_normal_mode()                         │
│  • Detects '5' → stores in input_state.command_count        │
│  • Detects 'j' → calls handle_navigation(KeyCode::Char('j'))│
└────────────────────────┬────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ navigation::commands::handle_navigation()                    │
│  • Consumes count prefix: count = input_state.take()        │
│  • Routes to handler based on key code                       │
└────────────────────────┬────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ navigation::commands::handle_directional_movement()          │
│  • Matches KeyCode::Char('j') → move_down_by(app, count)   │
└────────────────────────┬────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ navigation::commands::move_down_by(app, 5)                   │
│  • current = app.view_state.table_state.selected()          │
│  • target = (current + 5).min(max_row)                      │
│  • app.view_state.table_state.select(Some(target))          │
│  • app.view_state.viewport_mode = ViewportMode::Auto        │
│                                                               │
│  Performance: ~1.4 nanoseconds (O(1) regardless of CSV size) │
└────────────────────────┬────────────────────────────────────┘
                         ↓
                   Return to event loop
                         ↓
                   Render updated UI
```

**Navigation Helper Functions (v0.3.1 Refactor):**
- `handle_directional_movement()` - hjkl/arrow keys with count
- `handle_column_boundary()` - 0 and $ keys for first/last column
- `handle_page_navigation()` - PageUp/PageDown
- `handle_row_jump()` - Home/End/G with count support
- `handle_word_motion()` - w/b/e for non-empty cell navigation

**Count Prefix Behavior:**
- Count is consumed once at the top level
- If no count provided, defaults to 1
- Count applies to all compatible navigation commands
- Invalid counts (0) are rejected with error message

**Viewport Coordination:**
After navigation, viewport mode determines scroll behavior:
- `ViewportMode::Auto` - Keep cursor centered when possible
- `ViewportMode::Top` - Selected row at top (zt command)
- `ViewportMode::Center` - Selected row centered (zz command)
- `ViewportMode::Bottom` - Selected row at bottom (zb command)

### Rendering Pipeline (v0.3.1 - Detailed)

The rendering system uses virtual scrolling to maintain constant-time performance.

```
┌─────────────────────────────────────────────────────────────┐
│ main::run() - Event Loop                                     │
│   terminal.draw(|f| ui::render(f, app))                     │
└────────────────────────┬────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ ui::render(frame, app)                                       │
│  • Splits terminal into 3 areas:                             │
│    - Table area (Constraint::Min(0))                         │
│    - File switcher (Constraint::Length(2))                   │
│    - Status bar (Constraint::Length(1))                      │
│  • Calls specialized renderers for each area                 │
└────────────────────────┬────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ ui::table::render_table(frame, app, area)                    │
│  1. Calculate visible viewport dimensions                    │
│     • table_height = area.height - TABLE_HEADER_HEIGHT      │
│     • visible_cols = min(MAX_VISIBLE_COLS, total_cols)      │
│                                                               │
│  2. Calculate scroll offsets                                 │
│     • vertical: calculate_scroll_offset(selected, height)    │
│     • horizontal: based on selected_column and scroll_offset │
│                                                               │
│  3. Calculate column widths                                  │
│     • Measure header lengths for visible columns             │
│     • Terminal width / visible_cols with min/max bounds      │
│                                                               │
│  4. Build visible rows                                       │
│     • Extract only visible rows from document                │
│     • visible_rows = document.rows[offset..offset+height]   │
│                                                               │
│  5. Build row components                                     │
│     • build_column_letters_row() - A, B, C...                │
│     • build_header_row() - Column names                      │
│     • build_data_rows() - Visible data with styling          │
│                                                               │
│  6. Apply cell styling                                       │
│     • Selected cell: white background                        │
│     • Visual selection: dark gray background                 │
│     • Search matches: yellow highlight                       │
│     • Insert mode: show edit buffer with cursor              │
│                                                               │
│  7. Render Table widget to frame                             │
│     • Ratatui Table widget with calculated constraints       │
│     • frame.render_stateful_widget(table, area, state)      │
│                                                               │
│  Performance: ~389 µs for 100K rows (43x faster than 60 FPS) │
└────────────────────────┬────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ ui::status::render_file_switcher(frame, app, area)           │
│  • Display file list with scroll indicators                  │
│  • Show active file with highlight                           │
│  • Calculate scroll to keep active file visible              │
└────────────────────────┬────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ ui::status::render_status_bar(frame, app, area)              │
│  • Left: mode indicator (NORMAL, INSERT, VISUAL, etc.)       │
│  • Center: position (Row 5, Col B)                           │
│  • Right: flags (DIRTY, RO, ...)                             │
└─────────────────────────────────────────────────────────────┘
```

**Key Rendering Optimizations:**

1. **Virtual Scrolling**: Only renders visible cells (~40 rows × 10 cols = 400 cells)
   - Rendering time independent of CSV size
   - Performance: O(visible cells), not O(total cells)

2. **O(1) Cell Access**: Direct Vec indexing via RowIndex/ColIndex
   - Single cell access: ~1.4 ns
   - Sequential access (400 cells): ~360 ns

3. **Minimal Allocations**: Reuses buffers, no unnecessary cloning
   - Cell strings returned as `&str` references
   - Row data accessed via slice views

4. **CPU Cache Friendly**: Sequential memory access patterns
   - Rows stored as Vec<Vec<String>>
   - Visible rows accessed sequentially

5. **Lazy Computation**: Only calculates what's visible
   - Column widths: only for visible columns
   - Styling: only for visible cells
   - Scroll offsets: calculated once per frame

**Performance Characteristics (v0.3.1 Benchmarks):**

| Dataset Size | Render Time | FPS Equivalent | Margin vs 60 FPS |
|--------------|-------------|----------------|------------------|
| 1,000 rows   | 394 µs      | 2,538 FPS      | 42x faster       |
| 10,000 rows  | 397 µs      | 2,518 FPS      | 42x faster       |
| 100,000 rows | 389 µs      | 2,571 FPS      | 43x faster       |

**Why Constant Performance?**
- Virtual scrolling means only ~400 cells rendered regardless of CSV size
- Cell access is O(1) via direct indexing
- No iteration over non-visible data
- Terminal I/O (~390µs) is the bottleneck, not data processing (<1µs)

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
    HeaderEdit,  // Edit column headers (v0.12.0)
    Visual,      // Select rows/cells/blocks (v0.6.0)
    Command,     // Execute commands via `:` prefix
}

pub struct EditBuffer {
    pub content: String,   // Current content being edited
    pub cursor: usize,     // Cursor position within content
    pub original: String,  // Original content for cancel/undo
}
```

## Insert Mode Architecture (v0.4.1)

**Status:** Implemented and refactored in v0.4.1

Insert Mode provides quick, inline cell editing with vim-style keybindings. The implementation is organized into focused modules for maintainability.

### Module Structure

```
src/input/insert_mode/
├── mod.rs              # Main handler (36 lines)
├── commit_cancel.rs    # Enter, Tab, Esc operations (57 lines)
├── text_editing.rs     # Character input, backspace, delete (58 lines)
├── cursor_movement.rs  # Arrow keys, Home, End (39 lines)
└── vim_commands.rs     # Ctrl+h, Ctrl+w, Ctrl+u (73 lines)
```

### Editing Flow

```
User presses 's' in Normal mode
     ↓
Enter Insert mode with EditBuffer
     ↓
EditBuffer {
    content: "current cell value",
    cursor: 0,  // Character position (not bytes!)
    original: "current cell value"  // For cancellation
}
     ↓
User types/edits with:
  • Regular keys → Insert characters
  • Backspace/Delete → Remove characters
  • Ctrl+w → Delete word backward
  • Ctrl+u → Delete to start of line
  • Arrow keys → Move cursor
     ↓
User commits with:
  • Enter → Save and move down
  • Shift+Enter → Save and move up
  • Tab → Save and move right
  • Shift+Tab → Save and move left
     ↓
OR cancels with:
  • Esc → Discard changes, return to Normal mode
     ↓
commit_edit() called:
  • Only marks dirty if content changed
  • Updates document.set_cell(row, col, new_value)
  • Tracks last_edit_position for potential undo
```

### Unicode Handling

Insert Mode correctly handles multi-byte UTF-8 characters:

```rust
// Cursor position is in CHARACTERS, not bytes
buffer.cursor = 5;  // 5th character (could be 15+ bytes)

// Convert char position to byte position for string operations
let byte_pos = buffer.content
    .char_indices()
    .nth(buffer.cursor)
    .map(|(i, _)| i)
    .unwrap_or(buffer.content.len());

buffer.content.insert(byte_pos, new_char);
```

**Why this matters:**
- Emoji like "🚀" is 1 character but 4 bytes
- Japanese characters like "こんにちは" are 5 characters but 15 bytes
- Cursor position must match user's visual perception (characters)
- String mutations must use byte offsets (Rust requirement)

**Tested edge cases:**
- Emoji insertion and deletion
- Multi-byte Unicode (Japanese, accented characters)
- Cursor movement at grapheme boundaries
- Backspace/Delete with combining characters

### Commit Strategies

Insert Mode supports directional commit for efficient data entry:

| Key Combination | Action | Use Case |
|----------------|--------|----------|
| `Enter` | Save + move down | Vertical data entry (column-wise) |
| `Shift+Enter` | Save + move up | Correction workflow |
| `Tab` | Save + move right | Horizontal data entry (row-wise) |
| `Shift+Tab` | Save + move left | Backward correction |
| `Esc` | Cancel (no save) | Discard unwanted changes |

**Design rationale:**
- Matches spreadsheet UX (Excel, Google Sheets)
- Minimizes mode switches for bulk editing
- Directional navigation after commit reduces keystrokes

### Vim-Style Editing Commands

Insert Mode includes vim keybindings for power users:

| Command | Action | Example |
|---------|--------|---------|
| `Ctrl+h` | Backspace (vim) | Delete previous character |
| `Ctrl+w` | Delete word backward | `"hello world"` → `"hello "` |
| `Ctrl+u` | Delete to line start | `"hello world"` → `"world"` (cursor at 11) |

**Ctrl+w behavior:**
1. Delete trailing spaces first
2. Then delete word characters until hitting a space
3. Repeatable (keeps deleting words)

**Performance:**
- All editing operations are O(1) or O(n) where n = string length
- No performance degradation with long cell content
- Character-based cursor tracking: ~1.4ns per operation

### Dirty Tracking

The document tracks which cells have been modified:

```rust
// In commit_edit()
if buffer.content != buffer.original {
    app.document.set_cell(row_idx, col_idx, buffer.content);
    app.last_edit_position = Some((row_idx, col_idx));
    // Document internally sets is_dirty = true
}
```

**Why track dirty state?**
- Enable "unsaved changes" warnings (future: v0.6.0)
- Support undo/redo (future: v1.0.0)
- Optimize saves (only write if changed)
- Track last edit position for potential jump-to-last-edit command

### Refactoring Notes (v0.4.1)

The v0.4.1 refactor reduced `handle_insert_mode` from 183 lines to 36 lines by:

1. **Extracting commit/cancel operations** → `commit_cancel.rs`
   - Clearer separation of "exit insert mode" logic
   - commit_edit() duplicated from handler.rs to keep module self-contained

2. **Extracting text editing** → `text_editing.rs`
   - Character insertion with UTF-8 handling
   - Backspace/Delete operations

3. **Extracting cursor movement** → `cursor_movement.rs`
   - Arrow keys with saturation at boundaries
   - Home/End keys

4. **Extracting vim commands** → `vim_commands.rs`
   - Complex multi-step operations (Ctrl+w, Ctrl+u)
   - Ctrl+h as vim-style backspace

**Benefits:**
- Each module has a single responsibility
- Easier to test individual operations
- Improved readability and maintainability
- Reduced cognitive load when making changes

**Test coverage:**
- 64 tests in `tests/insert_mode_test.rs` (v0.4.0)
- 13 additional edge case tests in `tests/insert_mode_edge_cases.rs` (v0.4.1)
- Tests cover: Unicode, boundaries, vim commands, commit strategies, cancellation

## Visual Mode Architecture (v0.5.1)

### Overview

LazyCSV supports three visual modes for selecting and manipulating data:

- **Block Mode** (`v`): Rectangular selection of cells
- **Line Mode** (`V`): Whole row selection
- **Column Mode** (`,v`): Whole column selection

Each mode supports delete (`d`), yank (`y`), and paste (`p`/`P`) operations using an independent clipboard buffer.

### Triple Clipboard System

```
┌─────────────────────────────────────────────┐
│           Triple Clipboard System           │
├─────────────────┬─────────────┬─────────────┤
│   Row Buffer    │Col Buffer   │Region Buffer│
│                 │             │             │
│  yy/dd/p/P/o/O  │ ,yy/,dd/... │ Visual Block│
│  Visual Line    │Visual Column│   yank/del  │
└─────────────────┴─────────────┴─────────────┘
         ↓                ↓              ↓
    No cross-pasting between buffers
```

**Design principles:**
- **Isolated buffers**: Row, column, and region buffers never cross-contaminate
- **No transpose**: Yanked rows stay as rows, columns stay as columns
- **Mode-specific paste**: Each mode only reads from its corresponding buffer
- **Independent lifecycle**: Yanking in one mode doesn't affect other buffers

**Example:**
```
1. yy (yank row)      → Row buffer: ["A","B","C"]
2. ,yy (yank column)  → Column buffer: ["A","D","G"] (Row buffer unchanged)
3. Visual Block yank  → Region buffer: [["A","B"],["D","E"]] (Others unchanged)
4. p (paste row)      → Uses only Row buffer (ignores Column/Region)
```

### Module Structure

Visual mode operations were refactored in v0.5.1 from a single 331-line block into separate modules:

```
src/input/visual_mode/
├── mod.rs          # Module overview and public exports (25 lines)
├── delete.rs       # Visual delete operations (141 lines)
├── paste.rs        # Visual paste operations (115 lines)
└── yank.rs         # Visual yank operations (113 lines)
```

### Visual Delete Operations

**Flow:**
```
User presses 'd' in visual mode
    ↓
handle_visual_delete(app, clipboard)
    ↓
Match visual mode type:
    ├─ Block → delete_visual_block()
    │           ├─ Store region in region_buffer
    │           ├─ Delete cells in rectangle
    │           └─ Replace with empty strings
    │
    ├─ Line → delete_visual_line()
    │          ├─ Store rows in row_buffer
    │          ├─ Delete entire rows
    │          └─ Update cursor position
    │
    └─ Column → delete_visual_column()
               ├─ Store columns in column_buffer
               ├─ Delete entire columns (including header)
               └─ Adjust cursor if needed
```

**Key functions:**
- `delete_visual_block()`: Deletes rectangular selection, stores in region buffer
- `delete_visual_line()`: Deletes rows, stores in row buffer
- `delete_visual_column()`: Deletes columns, stores in column buffer

### Visual Yank Operations

**Flow:**
```
User presses 'y' in visual mode
    ↓
handle_visual_yank(app, clipboard)
    ↓
Match visual mode type:
    ├─ Block → yank_visual_block()
    │           └─ Copy cells to region_buffer (no modification)
    │
    ├─ Line → yank_visual_line()
    │          └─ Copy rows to row_buffer (no modification)
    │
    └─ Column → yank_visual_column()
               └─ Copy columns to column_buffer (no modification)
```

**Key functions:**
- `yank_visual_block()`: Copies rectangular selection to region buffer
- `yank_visual_line()`: Copies rows to row buffer
- `yank_visual_column()`: Copies columns to column buffer

### Visual Paste Operations

**Flow:**
```
User presses 'p'/'P' in visual mode
    ↓
handle_visual_paste(app, clipboard, key)
    ↓
Match visual mode type:
    ├─ Block → paste_visual_block()
    │           ├─ Replace selection with region_buffer
    │           └─ Expand document if needed
    │
    ├─ Line → paste_visual_line()
    │          ├─ Replace rows with row_buffer
    │          │  (p: after, P: before)
    │          └─ Delete original selection
    │
    └─ Column → paste_visual_column()
               ├─ Replace columns with column_buffer
               │  (p: after, P: before)
               └─ Delete original selection
```

**Key functions:**
- `paste_visual_block()`: Pastes region buffer into rectangular selection
- `paste_visual_line()`: Pastes row buffer, replacing selected rows
- `paste_visual_column()`: Pastes column buffer, replacing selected columns

**P vs p behavior:**
- Block mode: Same behavior (paste into selection)
- Line mode: `P` pastes before selection, `p` pastes after
- Column mode: `P` pastes before selection, `p` pastes after

### Refactoring Results (v0.5.1)

**Before refactoring (v0.5.0):**
```
src/input/handler.rs: 3053 lines
  ├─ handle_visual_delete: 118 lines
  ├─ handle_visual_paste: 115 lines
  └─ handle_visual_yank: 93 lines
Total: 326 lines in handler.rs
```

**After refactoring (v0.5.1):**
```
src/input/handler.rs: 2722 lines (-331 lines, -10.8%)

src/input/visual_mode/
  ├─ delete.rs: 141 lines
  │   ├─ handle_visual_delete: 47 lines (-60%)
  │   ├─ delete_visual_block: 26 lines
  │   ├─ delete_visual_line: 29 lines
  │   └─ delete_visual_column: 28 lines
  │
  ├─ paste.rs: 115 lines
  │   ├─ handle_visual_paste: 36 lines (-69%)
  │   ├─ paste_visual_block: 22 lines
  │   ├─ paste_visual_line: 28 lines
  │   └─ paste_visual_column: 22 lines
  │
  └─ yank.rs: 113 lines
      ├─ handle_visual_yank: 30 lines (-68%)
      ├─ yank_visual_block: 23 lines
      ├─ yank_visual_line: 30 lines
      └─ yank_visual_column: 24 lines

Total: 369 lines in visual_mode/ (+43 lines for improved organization)
```

**Benefits:**
- Main handler functions reduced by 60-69%
- Each operation type isolated in its own module
- Single responsibility per function
- Easier to test and maintain
- Clear separation between Block/Line/Column logic

### Test Coverage

**Existing tests:**
- 32 tests in `tests/visual_mode_test.rs` (v0.5.0)
  - Block, Line, Column mode operations
  - Delete, yank, paste combinations
  - Selection boundaries and edge cases

**New tests (v0.5.1):**
- 11 tests in `tests/clipboard_isolation.rs`
  - Buffer isolation (row/column/region don't cross-contaminate)
  - Yank operations only update their own buffer
  - No transpose operations between modes
  - Multiple operations stay in correct buffer

- 14 tests in `tests/column_reorder_edge_cases.rs`
  - Move single column to beginning/end
  - Move multiple columns forward/backward
  - Move to same position (no-op)
  - Column letter and numeric notation
  - Invalid source/target handling

**Total visual mode test coverage:** 57 tests

---

## Search Architecture (v0.7.1)

### Overview

LazyCSV provides powerful regex-based search with visual highlighting and vim-style navigation. The search system is optimized for large datasets, achieving ~18ms search times on 100K rows.

**Key features:**
- **Regex support**: Full regex pattern matching with case-insensitivity
- **Automatic fallback**: Invalid regex patterns fall back to literal substring search
- **Wrap-around navigation**: `n` and `N` commands wrap at document boundaries
- **Visual highlighting**: Current match highlighted differently from other matches
- **Match counter**: Status bar shows `[current/total]` position

### Search Pipeline

```
User enters search mode (/)
     ↓
Type pattern and press Enter
     ↓
find_matches(document, pattern)
     ↓
Try regex compilation (case-insensitive)
     │
     ├─ Success → Use regex matching
     │
     └─ Failure → Fall back to literal substring
     ↓
Scan all cells (row-major order)
     ↓
Store match positions: Vec<(RowIndex, ColIndex)>
     ↓
Create SearchState {pattern, matches, current_match}
     ↓
User navigates: n (next) / N (prev)
     ↓
jump_to_next() / jump_to_prev()
     ↓
Update cursor position
     ↓
UI highlights matches in render()
```

### Implementation Details

**Module:** `src/search/mod.rs` (398 lines)

**Core types:**
```rust
pub struct SearchState {
    pub pattern: String,
    pub matches: Vec<(RowIndex, ColIndex)>,
    pub current_match: Option<usize>,
}
```

**Key functions:**

1. `find_matches(document, pattern) -> Vec<(RowIndex, ColIndex)>`
   - Time: O(rows × cols × pattern_match_time)
   - Space: O(num_matches)
   - Tries regex first, falls back to literal substring
   - Returns sorted list of match positions

2. `SearchState::jump_to_next(cursor_row, cursor_col) -> Option<((RowIndex, ColIndex), bool)>`
   - Finds next match after cursor position
   - Returns (position, wrapped) tuple
   - Wraps to first match if at end of document

3. `SearchState::jump_to_prev(cursor_row, cursor_col) -> Option<((RowIndex, ColIndex), bool)>`
   - Finds previous match before cursor position
   - Returns (position, wrapped) tuple
   - Wraps to last match if at start of document

4. `SearchState::is_match(row, col) -> bool`
   - Fast O(n) check if cell is any match
   - Used by UI for highlight rendering

5. `SearchState::is_current_match(row, col) -> bool`
   - Fast O(1) check if cell is current match
   - Used by UI for different highlight style

### Search Algorithm

**Pattern Compilation:**
```rust
// Try regex first
if let Ok(re) = RegexBuilder::new(pattern).case_insensitive(true).build() {
    // Use regex matching
} else {
    // Fall back to literal substring (case-insensitive)
    let pattern_lower = pattern.to_lowercase();
    // Search using string.to_lowercase().contains(pattern_lower)
}
```

**Document Traversal:**
- Row-major order: iterate rows then columns
- Check each cell against pattern
- Store (RowIndex, ColIndex) for matches
- Natural sorting from iteration order

**Navigation:**
- Binary search through match list would be O(log n), but linear scan is fast enough
- Use `position()` to find next match after cursor
- Use `rposition()` to find previous match before cursor
- Wrap-around logic handled in jump methods

### UI Integration

**Highlighting in `src/ui/table.rs`:**

```rust
let style = if search_state.map(|s| s.is_current_match(ri, ci)).unwrap_or(false) {
    // Current match: yellow background, black text, bold
    Style::default()
        .bg(Color::Yellow)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD)
} else if search_state.map(|s| s.is_match(ri, ci)).unwrap_or(false) {
    // Other matches: dark gray background, yellow text
    Style::default().bg(Color::DarkGray).fg(Color::Yellow)
} else {
    // Normal cell styling
    Style::default()
};
```

**Status bar in `src/ui/status.rs`:**
- Search mode: Shows `/pattern` on left
- After search: Shows `/pattern [3/10]` with match counter
- Clear search: `:noh` or `Esc` in Normal mode

### Performance Characteristics

**Benchmark results (v0.7.1):**

| Dataset | Literal Search | Regex Search | No Matches | All Match (Worst) |
|---------|---------------|--------------|------------|-------------------|
| 1K rows | 199 µs | 293 µs | 221 µs | 183 µs |
| 10K rows | 1.76 ms | 2.23 ms | 877 µs | 1.72 ms |
| 100K rows | **18.1 ms** | **20.9 ms** | 67.2 ms | 196 ms |

**Analysis:**
- [ACHIEVED] **Target achieved**: 100K row search in ~18ms (well under 100ms target)
- Regex adds ~15% overhead vs literal search
- No matches case faster due to early termination in regex
- All match case (worst) still under 200ms for 100K rows
- **No optimization needed** - performance exceeds requirements

**Special cases:**
- Unicode search (Japanese, emoji): ~1.3ms for 10K rows
- Invalid regex fallback: ~6.7ms for 10K rows (fallback adds minimal overhead)
- Jump navigation: O(n) where n = match count, typically <1µs

### Edge Cases Handled

**Empty and boundary cases:**
- Empty pattern (matches everything or nothing, implementation dependent)
- Empty document (returns empty match list)
- Single cell document (works correctly)
- No matches found (empty match list, navigation returns None)
- All cells match (worst case, still performant)

**Regex edge cases:**
- Invalid regex syntax (unclosed brackets, etc.) → fallback to literal
- Very long regex patterns (>100 chars) → works correctly
- Special regex characters ($, %, (, [, etc.) → fallback to literal
- Unicode in regex (東京, emoji) → works correctly
- Anchor characters (^, $) → works as expected

**Navigation edge cases:**
- Single match (wraps to itself)
- Jump from exact match position (goes to next match)
- Jump prev from first match (wraps to last)
- Jump next from last match (wraps to first)
- No matches (returns None)

### Testing Strategy

**Test coverage (v0.7.1):**
- Module tests: 27 passing (in `src/search/mod.rs`)
- Edge case tests: 22 passing (in `tests/search_edge_cases.rs`)
- **Total: 49 search-specific tests**

**Test categories:**

1. **Basic functionality** (11 tests in module)
   - Pattern matching (literal, substring, case-insensitive)
   - Match counting and display
   - Navigation (next, prev, wrap-around)

2. **Regex patterns** (6 tests in module + 5 in edge cases)
   - Anchor matching (^, $)
   - Complex patterns (\d{1,2}, etc.)
   - Invalid regex fallback
   - Special characters

3. **Edge cases** (22 tests)
   - Empty/boundary conditions (5 tests)
   - Regex edge cases (6 tests)
   - Unicode and special characters (3 tests)
   - Performance stress tests (2 tests)
   - Navigation edge cases (5 tests)
   - Match detection (1 test)

4. **Performance benchmarks** (10 benchmark suites in `benches/search.rs`)
   - Simple literal search (1K, 10K, 100K rows)
   - Regex pattern search (1K, 10K, 100K rows)
   - Case-insensitive search
   - Regex vs literal comparison
   - Jump navigation performance
   - Worst-case scenarios (all match, no match)
   - Invalid regex fallback
   - Unicode content search

### Code Quality

**Function sizes:**
- All functions <50 lines [ACHIEVED]
- Longest function: `find_matches()` at 24 lines
- Average function size: ~15 lines

**Documentation:**
- Comprehensive rustdoc on public API [ACHIEVED]
- Usage examples in module docs
- Performance characteristics documented
- Algorithm details explained

**Clippy warnings:** 0 [ACHIEVED]

**Code organization:**
- Single module (398 lines) - appropriately sized
- No large function refactoring needed
- Clean separation of concerns

### Future Enhancements

**Potential improvements for later versions:**

1. **Incremental search**
   - Update matches on document changes instead of full re-search
   - Track dirty regions and re-scan only affected cells

2. **Parallel search**
   - Use rayon for multi-threaded search on very large datasets (>1M rows)
   - Split document into chunks and search concurrently

3. **Search history** (integrate with v0.10.0 undo/redo)
   - Store recent search patterns for quick re-use
   - Integrate with command history (`.` repeat, etc.)

4. **Lazy matching** (if needed)
   - Search visible viewport first for instant feedback
   - Expand to full document in background
   - Only implement if user testing shows >100ms feels slow

5. **Search indexing**
   - Pre-index frequently searched columns for O(1) lookup
   - Build inverted index for common patterns
   - Useful for very large datasets or repeated searches

**Note:** Current performance (18ms for 100K rows) exceeds requirements by 5.5x, so these optimizations are not currently needed.

---

## SQL Query System (v0.8.0-v0.8.1)

LazyCSV provides a SQL query mode that loads CSV files into an in-memory SQLite database for powerful data analysis and multi-table operations.

### Overview

The SQL query system allows users to:
- Execute SQL queries on CSV files using `:q SELECT...` command
- Perform JOINs across multiple CSV files automatically
- Use GROUP BY, ORDER BY, aggregations, and complex queries
- View query results as regular CSV documents
- Cache loaded data for instant re-execution

**Architecture:**
```
CSV Files → SQLite Tables → SQL Query → Result → Document
   ↓            ↓              ↓           ↓          ↓
sample.csv  → sample       → SELECT    → Rows    → Displayed
orders.csv  → orders       → FROM      → Columns    in UI
customers.csv → customers  → JOIN      → Data
```

### Core Components

#### 1. Query Module (`src/query/mod.rs`)

**Responsibilities:**
- CSV to SQLite loading (`load_csv_into_sqlite()`)
- Query execution (`execute_query_to_document_cancellable()`)
- Table name derivation (`table_name_from_path()`)
- File discovery (`resolve_csv_files()`)
- Error enhancement (`error_enhancer.rs`)

**Key Functions:**
```rust
// Load CSV document into SQLite table
pub fn load_csv_into_sqlite(conn: &Connection, doc: &Document, table_name: &str) -> Result<()>

// Execute query and return result as Document
pub fn execute_query_to_document_cancellable(
    conn: &Connection,
    query: &str,
    output_filename: String,
    cancelled: &AtomicBool,
) -> Result<Document>

// Convert file path to SQLite table name
pub fn table_name_from_path(path: &Path) -> String

// Resolve which CSV files to load (directory or siblings)
pub fn resolve_csv_files(path: &Path) -> Result<Vec<PathBuf>>
```

#### 2. SQL Execution Helpers (`src/app/sql_execution.rs`)

**Responsibilities:**
- Orchestrate SQL query execution with caching
- Load documents from session, cache, or disk
- Manage stale table cleanup
- Handle file configuration (delimiters, headers)

**Key Functions:**
```rust
// Unified entry point for loading files
pub(crate) fn load_session_file(
    cache: &mut SqliteCache,
    session: &Session,
    path: &Path,
    config: &FileLoadConfig,
) -> Result<()>

// Execute query and return Document
pub(crate) fn execute_and_convert_query(
    cache: &SqliteCache,
    query: &str,
    output_name: &str,
    cancelled: &AtomicBool,
) -> (Option<Document>, bool, Option<String>)

// Remove obsolete tables from cache
pub(crate) fn cleanup_stale_tables(cache: &mut SqliteCache, valid_paths: &[PathBuf])
```

#### 3. SQL Cache (`src/app/mod.rs` - SqliteCache)

**Responsibilities:**
- Maintain single in-memory SQLite connection per session
- Track loaded tables and their generation numbers
- Detect when documents need reloading
- Provide access to SQLite connection

**Structure:**
```rust
pub struct SqliteCache {
    conn: Connection,                    // In-memory SQLite database
    loaded_tables: HashMap<PathBuf, GenerationId>,  // Track loaded documents
}

impl SqliteCache {
    pub fn new() -> Result<Self>
    pub fn needs_reload(&self, path: &Path, current_gen: GenerationId) -> bool
    pub fn reload_table(&mut self, path: &Path, gen: GenerationId)
    pub fn remove_table(&mut self, path: &Path)
    pub fn conn(&self) -> &Connection
}
```

#### 4. SQL Editor UI (`src/ui/sql_editor.rs`)

**Responsibilities:**
- Render SQL editor overlay
- Display query text with cursor
- Show error messages
- Display help text

**Helper Functions (in `src/ui/sql_editor_helpers.rs`):**
```rust
// Build text lines with cursor highlighting
pub(crate) fn build_cursor_highlighted_lines(text: &str, cursor: usize) -> Vec<Line>

// Handle multi-line text with cursor
pub(crate) fn build_multiline_with_cursor(text: &str, cursor: usize) -> Vec<Line>

// Create error message line
pub(crate) fn build_error_line(error: &str) -> Line
```

### Data Flow

**1. User Enters SQL Query:**
```
User types: `:q SELECT * FROM sample WHERE price > 100`
    ↓
Input handler captures command
    ↓
App.query_buffer stores query text
    ↓
App.mode = Mode::SqlEditor
```

**2. Query Execution:**
```
User presses Enter in SQL editor
    ↓
execute_sql_query_cancellable() called
    ↓
cleanup_stale_tables() removes obsolete tables
    ↓
For each CSV file in directory:
    - Check if already loaded (SqliteCache.needs_reload())
    - If not: load_session_file() → load_csv_into_sqlite()
    - SQLite table now contains CSV data
    ↓
execute_and_convert_query() runs SQL query
    ↓
Result converted to Document
    ↓
Display result in main table view
```

**3. Table Name Resolution:**
```
File: /path/to/sales_data.csv
    ↓
table_name_from_path()
    ↓
Extract stem: "sales_data"
    ↓
Replace non-alphanumeric: "sales_data" (unchanged)
    ↓
Table name: "sales_data"
```

**Special Cases:**
- `my-file.csv` → table: `my_file`
- `data@2024.csv` → table: `data_2024`
- Spaces, dashes, special chars → all become `_`

### Multi-Table JOINs

LazyCSV automatically loads **all CSV files in the same directory** for JOIN queries:

**Example:**
```sql
-- Directory contains: orders.csv, customers.csv, products.csv
:q SELECT o.order_id, c.name, p.product_name
   FROM orders o
   JOIN customers c ON o.customer_id = c.customer_id
   JOIN products p ON o.product_id = p.product_id
   WHERE o.total > 100
```

**Process:**
1. User opens `orders.csv`
2. LazyCSV scans directory: finds `orders.csv`, `customers.csv`, `products.csv`
3. All files loaded into SQLite as tables: `orders`, `customers`, `products`
4. Query references all three tables → JOIN executes successfully
5. Result displayed as CSV document

**Why All Files?**
- Enables seamless JOINs without manual loading
- User doesn't need to know which files the query will reference
- Matches Excel/spreadsheet mental model (all sheets available)

### Caching Strategy

**Generation Tracking:**
Each Document has a `generation` number (incrementing counter). Cache uses this to detect staleness:

```rust
// Check if table needs reloading
if cache.needs_reload(&path, document.generation) {
    cache.reload_table(&path, document.generation);
    load_csv_into_sqlite(&cache.conn(), &document, &table_name)?;
}
```

**Cache Invalidation:**
- Document edited → generation increments → cache detects reload needed
- File closed → table remains in cache (available for future queries)
- Stale tables (files no longer in session) → removed by `cleanup_stale_tables()`

**Performance Benefits:**
- First query: ~50ms for 100K rows (load + execute)
- Subsequent queries: <5ms (cache hit, just execute query)
- Editing document invalidates cache automatically

### Error Enhancement (v0.8.1)

The `error_enhancer.rs` module transforms cryptic SQLite errors into helpful messages:

**1. Column Name Errors:**
```
Before: "no such column: usrname"
After:  "Column 'usrname' does not exist. Did you mean: username?
         Available columns: orders.order_id, orders.customer_id, orders.total"
```

**2. Table Name Errors:**
```
Before: "no such table: ordrers"
After:  "Table 'ordrers' does not exist. Did you mean: orders?
         Available tables: orders, customers, products"
```

**3. Syntax Errors:**
```
Before: "near SELECT: syntax error"
After:  "Syntax error near 'SELECT' at column 5:
           SLECT * FROM orders
               ^"
```

**How It Works:**
```rust
// Intercept SQLite errors
conn.prepare(query).map_err(|e| enhance_sql_error(e, conn, query))?

// enhance_sql_error() does:
1. Parse error message (extract column/table name)
2. Query SQLite schema (get available tables/columns)
3. Use Levenshtein distance to find similar names (fuzzy matching)
4. Build helpful error message with suggestions
```

### Performance Characteristics

**Benchmarks (v0.8.1):**

| Operation | 1K rows | 10K rows | 100K rows |
|-----------|---------|----------|-----------|
| Load CSV to SQLite | 2ms | 15ms | 150ms |
| Simple SELECT | 0.5ms | 2ms | 18ms |
| WHERE clause | 0.8ms | 3ms | 25ms |
| ORDER BY | 1ms | 5ms | 48ms |
| 2-way JOIN | 1.5ms | 12ms | 120ms |
| 3-way JOIN | 2ms | 18ms | 180ms |
| GROUP BY | 1.2ms | 8ms | 65ms |

**Targets (from roadmap):**
- [ACHIEVED] Simple SELECT <50ms for 100K rows (achieved: 18ms)
- [ACHIEVED] JOIN <200ms for 10K rows (achieved: 12ms for 2-way, 18ms for 3-way)

**Optimization:**
- Single transaction for bulk INSERT (50x faster than row-by-row)
- Prepared statements with parameter binding (no SQL injection + faster)
- `PRAGMA` optimizations for in-memory databases:
  ```sql
  PRAGMA journal_mode=OFF;
  PRAGMA synchronous=OFF;
  PRAGMA temp_store=MEMORY;
  PRAGMA cache_size=-64000;  -- 64MB cache
  ```
- Lazy loading: only load CSVs when first query executed
- Cancellation checks every 1000 rows (responsive Ctrl+C)

### SQLite Schema

All columns are `TEXT` type (SQLite flexible typing):
```sql
CREATE TABLE "sample" (
    "ID" TEXT,
    "Name" TEXT,
    "Price" TEXT,
    "Quantity" TEXT
)
```

**Why TEXT?**
- CSV files are inherently text-based
- SQLite automatic type coercion (`CAST(price AS REAL)` works)
- Avoids parse errors from mixed-type columns
- Preserves original formatting (leading zeros, etc.)

**Type Conversions in Queries:**
```sql
-- String to number
SELECT * FROM orders WHERE CAST(total AS REAL) > 100.0

-- String to integer
SELECT SUM(CAST(quantity AS INTEGER)) FROM orders

-- Numeric comparison (automatic)
SELECT * FROM orders WHERE price > '100'  -- Works, coerces to number
```

### Testing Strategy (v0.8.1)

**Unit Tests:**
- CSV loading (various formats, encodings, edge cases)
- Table name derivation (special characters, Unicode)
- Error enhancement (Levenshtein distance, suggestion ranking)

**Integration Tests (11 original):**
- Simple SELECT queries
- WHERE filtering
- ORDER BY sorting
- Multi-table JOINs (2-way, 3-way)
- GROUP BY aggregations
- Subqueries

**Edge Case Tests (30 new in v0.8.1):**
- **Error Handling (8):**
  - Invalid syntax
  - Misspelled columns
  - Missing tables
  - Type errors (division by zero)
- **Edge Cases (8):**
  - Empty results
  - Large datasets (1000 rows)
  - NULL values
  - Special characters
  - Unicode
  - Long strings
  - Case-insensitive columns
- **Complex Queries (8):**
  - Three-way JOINs
  - Subqueries
  - UNION
  - GROUP BY + HAVING
  - Self-joins
  - Multiple aggregations
- **Additional (6):**
  - LIMIT/OFFSET
  - DISTINCT
  - String functions (UPPER, LOWER, SUBSTR)
  - CASE expressions
  - Date functions (DATE, DATETIME, STRFTIME)
  - CROSS JOIN, LIKE patterns, IN operator

**Benchmarks (13 groups in v0.8.1):**
- CSV loading (single + multiple tables)
- Query operations (SELECT, WHERE, ORDER BY, JOIN, GROUP BY)
- Result size impact (10 rows, 1K, 50K)
- Complex queries (multiple JOINs + aggregations)
- Table name derivation

### Known Limitations

1. **Memory Usage:**
   - All CSVs loaded into SQLite in-memory database
   - Memory usage = (sum of all CSV file sizes) × 1.5
   - Not suitable for >1GB of combined CSV data

2. **SQL Dialect:**
   - SQLite syntax only (not PostgreSQL, MySQL, etc.)
   - Some functions differ (e.g., `SUBSTR()` vs `SUBSTRING()`)
   - No window functions in older SQLite versions

3. **Schema Limitations:**
   - All columns TEXT type (no strong typing)
   - No indexes (not needed for in-memory small datasets)
   - No foreign keys or constraints

4. **No Persistent Database:**
   - SQLite database destroyed on exit
   - Query history not saved (feature for v1.0.0+)
   - No SQL scripts or stored procedures

### Future Enhancements

**v0.11.0 - SQL Editor Vim Editing:**
- Full vim modal editing in SQL editor
- Multi-line query support with proper navigation
- Syntax highlighting (SQL keywords, table names)

**v0.18.0 - SQL IntelliSense:**
- Auto-completion for table names, column names, SQL keywords
- Context-aware suggestions (after FROM → table names)
- Real-time syntax error detection
- Query templates for common patterns

**v1.0.0+:**
- Query history with recall (Up/Down arrows)
- Saved queries library
- Query performance profiling (EXPLAIN QUERY PLAN)
- Export query results to JSON/Markdown

---

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
| Navigate | O(1) | Just update index (1-80 ns) |
| Render | O(v) | v = visible cells (~400) |
| Cell access | O(1) | Direct Vec indexing (~1.4 ns) |
| Search (v0.7.0) | O(n) | Full table scan |
| Sort (future) | O(n log n) | Standard sort |

### Space Complexity

| Structure | Memory | Notes |
|-----------|--------|-------|
| CSV data | O(n) | n = total cells, fully in RAM |
| UI state | O(1) | Fixed size ViewState |
| Render buffer | O(v) | v = visible cells (~400) |
| Navigation state | O(1) | Cursor position, scroll offsets |

### Performance Targets (v0.3.1 Benchmarks)

[ACHIEVED] **All targets exceeded by 40x+**

**Rendering Performance:**
- Target: <16.67 ms per frame (60 FPS)
- Actual: ~389 µs per frame (2,571 FPS)
- **Result: 43x faster than target** [ACHIEVED]

| Dataset Size | Render Time | vs Target | Status |
|--------------|-------------|-----------|--------|
| 1K rows      | 394 µs      | 2.4% used | PASS |
| 10K rows     | 397 µs      | 2.4% used | PASS |
| 100K rows    | 389 µs      | 2.3% used | PASS |

**Navigation Performance:**
- Target: <100 ns per operation
- Actual: 1-80 ns per operation
- **Result: Sub-nanosecond to sub-100ns** [ACHIEVED]

| Operation | Time | Notes |
|-----------|------|-------|
| move_down/up | 1.4 ns | With count prefix |
| move_left/right | 1.4 ns | Horizontal movement |
| goto_first_row | 0.9 ns | Jump to top |
| goto_last_row | 0.9 ns | Jump to bottom |
| goto_line | 30 ns | Jump to specific row |
| next_word | 3.1 ns | Non-empty cell search |
| goto_column | 50 ns | Excel-style column jump |

**Cell Access Performance:**
- Single cell: ~1.4 ns (O(1) Vec indexing)
- Sequential 400 cells: ~360 ns (typical viewport)
- **No performance degradation** with dataset size

**Why So Fast?**
1. **Virtual Scrolling**: Only renders visible cells (~400)
2. **O(1) Indexing**: Direct Vec access, no iteration
3. **Minimal Allocations**: Reuses buffers, returns references
4. **CPU Cache Friendly**: Sequential memory access
5. **Lazy Computation**: Only calculates visible data

**Bottleneck Analysis:**
- Terminal I/O: ~390 µs (ratatui rendering)
- Data processing: <1 µs (navigation + cell access)
- **Conclusion**: Terminal rendering is bottleneck, not data operations

**Performance Margin:**
With 43x performance margin, LazyCSV can:
- Add rich styling/colors without FPS impact
- Implement complex visual modes
- Add search highlighting
- Support larger viewports
- Handle real-time data updates

For detailed benchmark results, see `docs/v0.3.1-benchmarks.md`.

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
-  Simpler (no sync primitives needed)
-  Sufficient for keyboard input (low latency)
-  CSV loading is fast enough
-  Rendering is fast enough

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

**v0.2.1  COMPLETED:**
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

**v0.2.2-v0.2.6  COMPLETED:**
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
