# Changelog

All notable changes to LazyCSV will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2026-02-02

### Added - UI/UX Polish

**Enhanced Status Bar**
- Mode indicator (-- NORMAL -- / -- COMMAND --)
- Dirty flag display ([*]) for modified files
- Transient messages that auto-clear on next keypress

**Improved Help Menu**
- Redesigned layout with better organization
- All v0.3.0 features documented
- Clearer categorization (Navigation, Jumping, Command Mode, etc.)

**File Switcher**
- Horizontal scrolling support for wide file lists
- Better handling of many open files

## [0.3.0] - 2026-02-02

### Added - Advanced Navigation

**Column Jumping (Excel-style)**
- `ga`, `gB`, `gBC` - Jump to column A, B, BC using Excel notation
- Letter buffering with 1-second timeout
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

## [0.2.0] - 2026-02-02

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

[0.3.1]: https://github.com/funkybooboo/lazycsv/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/funkybooboo/lazycsv/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/funkybooboo/lazycsv/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/funkybooboo/lazycsv/compare/v0.1.0...v0.1.4
[0.1.0]: https://github.com/funkybooboo/lazycsv/releases/tag/v0.1.0
