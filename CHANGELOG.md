# Changelog

All notable changes to LazyCSV will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - v0.4.1

### Added
- **Header Mode Toggle** - `:ht` command to toggle header mode ON/OFF
  - When ON: first row treated as header (row 0), navigation starts at row 1
  - When OFF: first row is regular data, navigation starts at row 0
  - Header row is highlighted when selected
  
### Changed
- **Navigation System Refactored** - Migrated from data-row indices to absolute row indices
  - All row indices now include the header row (0-based absolute indexing)
  - `gg` respects header_mode: goes to row 1 when ON, row 0 when OFF
  - Cannot navigate to row 0 when header_mode is ON
  - Status line shows absolute row numbers (0 for header, 1+ for data)
  - Row count includes header row
  
### Fixed
- **Delete Header Row** - `dd` on row 0 now automatically turns header_mode OFF
- All navigation commands properly respect header_mode boundaries
- Updated 400+ tests to use absolute row indexing

## [0.4.0] - 2026-02-09

### Added - Insert Mode & Cell Editing

**Enter Insert Mode**
- `i` - Edit cell at current cursor position
- `a` - Edit cell at end (append)
- `I` - Edit cell at start
- `A` - Alias for `a`
- `s` - Replace cell (clear and enter Insert mode)
- `F2` - Excel/Calc style cell editing
- Double-click support (future)

**Text Editing in Insert Mode**
- Type characters to insert at cursor
- `Backspace` or `Ctrl+h` - Delete character before cursor
- `Delete` - Delete character at cursor
- `Ctrl+w` - Delete word backward
- `Ctrl+u` - Delete to start of cell
- `Home` / `End` - Move cursor to start/end of cell
- `Left` / `Right` arrows - Move cursor within cell
- Vim-style navigation within cell content

**Commit or Cancel Edits**
- `Enter` - Save edit and move down one row
- `Shift+Enter` - Save edit and move up one row
- `Tab` - Save edit and move right one column
- `Shift+Tab` - Save edit and move left one column
- `Esc` - Cancel edit without saving

**Row Operations (Normal Mode)**
- `o` - Add new row below, enter Insert mode
- `O` - Add new row above, enter Insert mode
- `dd` - Delete current row (stored in clipboard)
- `yy` - Copy (yank) current row
- `p` - Paste row below current position
- `Delete` - Clear current cell content (stay in Normal mode)

**Infrastructure**
- Mode::Insert variant in app mode enum
- EditBuffer struct for in-cell editing state
- Row clipboard for copy/paste operations
- last_edit_position tracking for future commands

### Technical

**Metrics:**
- Tests: 271+ unit tests + integration tests (confirmed passing)
- Test coverage includes all Insert mode operations
- Zero compiler warnings
- Zero clippy warnings
- Performance: Unchanged (still 60 FPS on 100K+ rows)

**Architecture:**
- Implemented handle_insert_mode() function
- EditBuffer state management with cursor tracking
- Integration with existing navigation and viewport system

## [0.3.2] - 2026-02-08

### Added - Pre-Edit Polish

**UI Redesign: Vim-like Minimal Interface**
- Removed heavy box borders, replaced with clean horizontal rules
- Minimal chrome - only essential separators
- Current row indicator: Single `>` in row number column
- Current column: Highlighted in header row
- Top bar: Filename (left) and row/total (right)
- File list: Single line, minimal formatting
- Status line: Mode + position + cell preview (vim-style: `3,C "Mike Jo..."`)
- Pending commands visible in status bar (`g_`, `z_`, `5_`)

**Column Formatting**
- Auto-width columns based on content (8-50 char range)
- Dynamic sizing - wider columns for wider content
- Respects terminal width constraints

**Command Mode Improvements**
- `:c` command for column navigation (replaces `g<letter>`)
  - `:c A` or `:c a` → Jump to column A
  - `:c 1` → Jump to column A (by number)
  - `:c AA` or `:c aa` → Jump to column AA
  - `:c 27` → Jump to column AA (by number)
- Reserved commands always work: `:q`, `:w`, `:wq`, `:help`
- Out-of-bounds errors instead of silent clamping

**Error Messages & Feedback**
- User-friendly error messages with readable key names
- Out-of-bounds: "Row 999 does not exist (max: 10)"
- Clear feedback for invalid commands
- No timeout on pending commands (vim-like behavior)

**Default Behavior**
- Running `lazycsv` without arguments scans current directory
- Automatically detects CSV files in directory

### Architecture Prep for v0.4.0

**Prepared (but not implemented yet)**
- Mode enum with variants: Normal, Insert, Magnifier, HeaderEdit, Visual, Command
- EditBuffer struct: `{ content, cursor, original }`
- Infrastructure for future editing modes

## [0.3.1] - 2026-02-07

### Added - UI/UX Polish

**Enhanced Status Bar**
- Mode indicator (-- NORMAL -- / -- COMMAND --)
- Dirty flag display (*) for modified files
- Transient messages that auto-clear on next keypress

**Improved Help Menu**
- Redesigned layout with better organization
- All v0.3.0 features documented
- Clearer categorization (Navigation, Jumping, Command Mode, etc.)

**File Switcher**
- Horizontal scrolling support for wide file lists
- Better handling of many open files

## [0.3.0] - 2026-02-06

### Added - Advanced Navigation

**Column Jumping (Excel-style)**
- `ga`, `gB`, `gBC` - Jump to column A, B, BC using Excel notation
- Letter buffering with no timeout (vim-like)
- Support for multi-letter columns (AA, AB, BC, etc.)

**Command Mode**
- `:` - Enter vim-style command mode
- `:15` - Jump to line 15
- `:B`, `:BC` - Jump to column B or BC
- `Esc` - Cancel command input

**Word Motion for Sparse Data**
- `w` - Jump to next non-empty cell in current row
- `b` - Jump to previous non-empty cell in current row
- `e` - Jump to last non-empty cell in current row

**Enhanced Navigation**
- `Enter` - Move down one row (like `j`)
- Count prefixes with all navigation (e.g., `5j`, `10h`, `3w`)

**Viewport Control**
- `zt` - Position current row at top of screen
- `zz` - Position current row at center of screen
- `zb` - Position current row at bottom of screen

### Technical

**Metrics:**
- Tests: 265 (237 unit + 7 CLI + 21 workflow)
- Test runtime: 1.12s
- Zero compiler warnings
- Zero clippy warnings

**Architecture:**
- Added Mode::Command enum for modal editing
- Extended PendingCommand for letter buffering
- Enhanced InputState with command_buffer
- Improved multi-key command timeout handling
- Added excel_letter_to_column() bidirectional conversion

## [0.2.0] - 2026-02-05

### Changed - Internal Refactoring (No User-Facing Changes)

**Phase 1-6: Type Safety & Architecture Refactor**

This release completed a major 6-phase internal refactoring for better code quality, maintainability, and type safety. No user-facing features changed.

**Phase 1: Type Safety Foundation**
- Introduced type-safe position types (RowIndex, ColIndex)
- Created UserAction abstraction layer
- Eliminated primitive obsession with semantic types

**Phase 2: Separation of Concerns**
- Extracted InputState module for input handling
- Extracted Session management module for multi-file state
- Renamed UiState → ViewState for clarity

**Phase 3: Better Naming & Consistency**
- Renamed csv_data → document (CsvData → Document)
- Renamed ui → view_state (UiState → ViewState)
- Consistent function naming (get_*, move_*, goto_*)

**Phase 4: Code Organization**
- Reorganized modules (csv/, file_system/, session/, navigation/)
- Clear module boundaries
- Well-defined public APIs

**Phase 5: Clean Code Improvements**
- Decomposed long functions (all < 80 lines)
- Removed all magic numbers
- Added comprehensive documentation

**Phase 6: Testing & Validation**
- Expanded test suite from 133 to 257 tests (+124)
- Added z-command tests (zt/zz/zb viewport positioning)
- Added timeout behavior test
- Added 17 navigation unit tests
- Zero compiler warnings
- Zero clippy warnings

**Metrics:**
- Tests: 257 (229 unit + 7 CLI + 21 workflow)
- Test runtime: 1.12s
- Code quality: All functions < 80 lines
- Performance: No regression (still 60 FPS on 100K rows)

## [0.1.4] - 2026-01-XX

### Added
- Comprehensive test coverage (133 tests)
- Rust idioms and code quality improvements
- Navigation workflow tests
- CSV edge case tests

### Changed
- Improved code organization
- Better error handling
- Enhanced test suite

## [0.1.0] - 2026-01-XX

### Added
- Initial release
- Core CSV viewing with vim navigation (hjkl, gg, G, 0, $)
- Multi-file switching ([ and ])
- Arrow key navigation
- Page up/down
- Basic UI with status bar
- Help overlay (?)
- Row and column numbers
- Cell highlighting
- Horizontal scrolling
- File switcher UI
- Dirty state tracking
- Quit functionality (q)

[0.4.0]: https://github.com/funkybooboo/lazycsv/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/funkybooboo/lazycsv/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/funkybooboo/lazycsv/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/funkybooboo/lazycsv/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/funkybooboo/lazycsv/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/funkybooboo/lazycsv/compare/v0.1.0...v0.1.4
[0.1.0]: https://github.com/funkybooboo/lazycsv/releases/tag/v0.1.0
