# LazyCSV Development Roadmap

A versioned checklist for building the LazyCSV TUI. Each version represents a deliverable milestone.

## Version Overview

| Version | Focus | Status | Tests |
|---------|-------|--------|-------|
| v0.1.0 | Foundation & Core Viewing | [x] | 257 |
| v0.1.1 | Post-Foundation Refactor | [ ] | TBD |
| v0.2.0 | Type Safety & Architecture | [x] | 257 |
| v0.2.1 | Type System Cleanup | [ ] | TBD |
| v0.3.0 | Advanced Navigation & UI Polish | [x] | 344 |
| v0.3.1 | Navigation Code Quality | [ ] | TBD |
| v0.4.0 | Cell Editing & Persistence | [x] | 517 |
| v0.4.1 | Editing System Refactor | [ ] | TBD |
| v0.5.0 | Column Operations & Visual Mode | [x] | 515+ |
| v0.5.1 | Column Operations Cleanup | [ ] | TBD |
| v0.6.0 | Magnifier Mode (Full Vim Editor) | [x] | 415 |
| v0.6.1 | Magnifier Performance & Quality | [ ] | TBD |
| v0.7.0 | Search & Filtering | [x] | 27 |
| v0.7.1 | Search System Optimization | [ ] | TBD |
| v0.8.0 | SQL Query Mode & Data Operations | [x] | 30 |
| v0.8.1 | SQL & Data Operations Polish | [ ] | TBD |
| v0.9.0 | Undo/Redo & Command History | [ ] | - |
| v0.9.1 | Code Quality & Architecture Refactor | [ ] | TBD |
| v0.10.0 | Cell Transforms & Data Cleanup | [ ] | - |
| v0.10.1 | Performance Optimization & Profiling | [ ] | TBD |
| v0.11.0 | System Clipboard & External Integration | [ ] | - |
| v0.11.1 | Testing & Reliability Improvements | [ ] | TBD |
| v0.12.0 | Bulk Operations & Find/Replace | [ ] | - |
| v0.12.1 | Error Handling & Robustness | [ ] | TBD |
| v0.13.0 | Advanced Filtering & Conditional Views | [ ] | - |
| v0.13.1 | Module Organization & Cleanup | [ ] | TBD |
| v0.14.0 | Column Resize & Advanced Column Operations | [ ] | - |
| v0.14.1 | Documentation & Maintainability | [ ] | TBD |
| v0.15.0 | Data Analysis & Statistics | [ ] | - |
| v0.15.1 | Technical Debt Reduction | [ ] | TBD |
| v0.16.0 | Export & Import (JSON, Markdown, TSV) | [ ] | - |
| v0.16.1 | Code Coverage & Test Quality | [ ] | TBD |
| v0.17.0 | Configuration System | [ ] | - |
| v0.17.1 | Performance Benchmarking & Tuning | [ ] | TBD |
| v0.18.0 | Macros & Command Recording | [ ] | - |
| v0.18.1 | Final Architecture Polish | [ ] | TBD |
| v1.0.0 | Stable Release & Polish | [ ] | - |

**Total Tests Passing:** 420+ library tests + integration tests

---

## Guiding Principles

- **Vim-First Philosophy:** Navigation and commands should feel native to vim users. Composable commands (operator + motion). No timeouts on pending commands. Clean status line.
- **Truly Hybrid:** Balance vim power with spreadsheet familiarity. Support both vim keys (hjkl) and arrow keys, vim commands and spreadsheet-like operations.
- **Three-Tier Operator System:** Cell (`x`) → Row (`dd`) → Column (`,dd`). Comma as leader for CSV-specific column operations.
- **Simplified Navigation:** Use `g` suffix for jumps: `5g` (row 5), `:cB` (column B). Reserve `:` for operations and explicit navigation.
- **Header Toggle System:** Header row is always row 0. Toggle header mode with `:ht` to freeze/style row 0. When ON, `gg` goes to row 1 (first data row).
- **Command Ranges:** Standardized ranges: `:5,10d` for rows, `:B,Dd` for columns. Don't overcomplicate.
- **Triple Clipboard:** Three independent buffers: row buffer (yy/p), column buffer (,yy/,p), region buffer (visual y/p). No cross-pasting.
- **Ephemeral Edits:** No changes saved to file until explicit `:w`. All edits update in-memory representation first.
- **Minimal UI Chrome:** No heavy borders. Use subtle separators. Maximum content, minimum decoration. Status line shows mode + row + column only.
- **In-Memory Only:** All CSV files loaded entirely into RAM for maximum performance.
- **CSV Only:** No Excel (.xlsx) support - CSV files only for simplicity.
- **Zero Configuration:** Works great out of the box. Optional `~/.config/lazycsv/config.toml` for power users with full customization.
- **Robust Error Handling:** Handle errors gracefully with clear, user-friendly feedback.

---

## Modal Editing Reference

LazyCSV uses vim-style modal editing with these modes:

| Mode | Indicator | Purpose | Entry | Exit |
|------|-----------|---------|-------|------|
| Normal | `NORMAL` | Navigation, commands | Default / `Esc` | N/A |
| Insert | `INSERT` | Quick single-cell editing | `i`, `a`, `A`, `I`, `s` | `Enter` (save), `Esc` (cancel) |
| Magnifier | `MAGNIFIER` | Full vim editor for cell | `m` on cell | `:wq`, `:q`, `ZZ` |
| Visual | `VISUAL` | Rectangular selection | `v` | `Esc`, or after operation |
| Visual Line | `VISUAL LINE` | Whole row selection | `V` | `Esc`, or after operation |
| Visual Column | `VISUAL COLUMN` | Whole column selection | `,v` | `Esc`, or after operation |
| Command | `:` prompt | Execute commands | `:` | `Enter` (execute), `Esc` (cancel) |
| Search | `/` prompt | Enter search pattern | `/` | `Enter` (search), `Esc` (cancel) |
| SQL Editor | `SQL` | Edit SQL query | `:q SELECT...` | `Enter` (execute), `Esc` (cancel) |

**Mode hierarchy:** Normal is the "home" mode. All other modes return to Normal.

**Insert vs Magnifier:**
- `i` - Quick edits (single-line, simple text)
- `m` - Complex edits (multi-line, full vim power)
- Default to Insert mode, manually upgrade to Magnifier when needed

---

## Quick Command Reference

### Core Commands

| Command | Action |
|---------|--------|
| `:q` | Quit (checks for unsaved changes) |
| `:q!` | Force quit (discard all changes) |
| `:w` | Write all dirty files |
| `:wq` / `:Wq` | Write all and quit |
| `:h` / `:help` | Show full help buffer |
| `:files` | Show file menu |

### Navigation Commands

| Command | Action |
|---------|--------|
| `:cA` / `:cB` / `:cAA` | Jump to column A, B, AA |
| `:c1` / `:c27` | Jump to column by number (1=A, 27=AA) |
| `5g` | Jump to row 5 |
| `gg` / `G` | First/last row |

### Data Commands

| Command | Action |
|---------|--------|
| `:ht` | Toggle header mode for current file |
| `:delim ;` | Set delimiter to semicolon (session-only) |
| `:new A,B,C` | Create new CSV with headers |
| `:sort A,B` | Sort by columns A, B ascending |
| `:sort! A,B` | Sort by columns A, B descending |
| `:q SELECT...` | Execute SQL query |
| `:sql` | Open SQL editor |
| `:noh` | Clear search highlighting |

### Range Operations

| Command | Action |
|---------|--------|
| `:5,10d` | Delete rows 5-10 |
| `:5,10y` | Yank rows 5-10 |
| `:%d` | Delete all rows |
| `:B,Dd` | Delete columns B through D |
| `:B,Dy` | Yank columns B through D |

---

## Essential Keybindings

### Navigation
- `hjkl` / arrows - Move around
- `gg` / `G` - First/last row
- `5g` - Jump to row 5
- `0` / `$` - First/last column
- `w` / `b` / `e` - Next/prev/end non-empty cell
- `zt` / `zz` / `zb` - Scroll position

### Editing
- `i` / `a` / `I` / `A` / `s` - Enter Insert mode
- `m` - Enter Magnifier mode (full vim editor)
- `x` - Delete cell content
- `o` / `O` - Insert row below/above

### Row Operations
- `dd` - Delete row
- `yy` - Yank (copy) row
- `p` / `P` - Paste row below/above
- `5dd` - Delete 5 rows

### Column Operations (Comma Leader)
- `,dd` - Delete column
- `,yy` - Yank column (includes header)
- `,p` / `,P` - Paste column right/left
- `,o` / `,O` - Insert column right/left

### Visual Mode
- `v` - Visual block (rectangular)
- `V` - Visual line (whole rows)
- `,v` - Visual column (whole columns)
- `d` / `y` / `p` - Delete/yank/paste selection
- `gv` - Re-select last selection

### Search
- `/pattern` - Search forward (regex supported)
- `n` / `N` - Next/previous match
- `*` - Search current cell content
- `:noh` - Clear search highlighting

### File Navigation
- `[` / `]` - Previous/next CSV file
- `:files` - Show file menu

### Help
- `?` - Quick reference overlay
- `:help` - Full help buffer

---

## Version Details

### v0.1.0 - Foundation & Core Viewing [x]

**Focus:** Basic CSV viewing with vim navigation and multi-file support  
**Status:** [x]  
**Tests:** 257 passing (229 unit + 7 CLI + 21 workflow)

**Features:**
- [x] CSV file parsing with encoding detection
- [x] Vim navigation (hjkl, arrows, gg, G, 0, $)
- [x] Multi-file switching with [ and ] keys
- [x] Row/column numbering (1, 2, 3... and A, B, C...)
- [x] Basic UI with status bar and file list
- [x] Help overlay with ? key
- [x] Directory scanning for CSV files
- [x] Zero configuration - works out of the box

**CLI Usage:**
```bash
# Open current directory (scans for CSV files)
lazycsv

# Open specific file
lazycsv data.csv

# Open multiple files
lazycsv customers.csv orders.csv

# Open a directory
lazycsv ./data/
```

**Design Philosophy:**
- No CLI flags for delimiter or headers (use in-app commands instead)
- Default delimiter: comma (`,`)
- Default header mode: ON
- Simpler CLI, more discoverable in-app commands

**Architecture Decisions:**
- **In-Memory Design:** All CSV files loaded entirely into RAM for maximum performance
- **Target Performance:** 100K+ rows at 60 FPS
- **Memory Usage:** ~1MB RAM per 10K rows (approximate)
- **Startup Time:** < 100ms for files under 10MB
- **Trade-off:** Prioritize speed over handling files larger than RAM (use DuckDB/ClickHouse for huge files)

**Consolidates:** v0.1.1 (Foundation Cleanup), v0.1.2 (Test Coverage), v0.1.3 (Rust Idioms), v0.1.4 (Comprehensive Tests)

---

### v0.1.1 - Post-Foundation Refactor [ ]

**Focus:** Refactor foundation code for maintainability  
**Status:** [ ]  
**Primary Focus:** Code quality improvements after initial foundation

**Philosophy:**
Clean up technical debt from initial implementation. Focus on improving code organization, test coverage, and documentation based on lessons learned from v0.1.0.

**Audit Findings (Completed):**
- [x] Total codebase: 17,744 lines across 17 modules
- [x] Current tests: 420 passing (100% pass rate)
- [x] Clippy warnings: 9 total (4 lib, 5 test)
- [x] Functions >50 lines: 35 (largest: 491 lines)
- [x] Unwrap/expect calls: 592 instances
- [x] Stale TODOs: 6 (all clipboard-related)
- [x] Largest file: input/handler.rs (3,257 lines = 18% of codebase)

**Tasks:**
- [ ] Install cargo-tarpaulin for coverage measurement
- [ ] Measure baseline code coverage
- [ ] Fix all clippy warnings (run `cargo clippy --fix`)
- [ ] Remove/update 6 stale TODOs in handler.rs
- [ ] Refactor main.rs::run() (295 lines → <50 lines)
- [ ] Refactor ui/status.rs::render_status_bar() (233 lines → <50 lines)
- [ ] Add rustdoc for all public APIs in root modules
- [ ] Document acceptable unwrap() uses vs. ones needing fixes
- [ ] Add tests for core viewing functionality (target: >80% coverage)
- [ ] Update docs/architecture.md with current module structure

**Success Criteria:**
- [x] Zero clippy warnings (currently 9)
- [ ] Code coverage > 80% (baseline unknown)
- [ ] All functions < 50 lines (currently 35 over limit)
- [ ] Performance benchmarks established
- [ ] Module structure documented
- [ ] All tests pass with no panics (currently 420 passing)

**Testing Strategy:**
- Full regression test suite (maintain 420+ passing)
- Increase test coverage for core viewing
- Edge case testing for navigation
- Add tests before refactoring large functions (TDD)

**Documentation Requirements:**
- Architecture documentation for core modules
- Code comments for complex logic
- Rustdoc for public APIs
- Refactoring notes documenting improvements

---

### v0.2.0 - Type Safety & Architecture Refactor [x]

**Focus:** Clean architecture with type-safe position handling  
**Status:** [x]  
**Tests:** 257 passing (zero warnings)

**Features:**
- [x] Type-safe position types (RowIndex/ColIndex newtypes)
- [x] Action abstraction layer (UserAction, NavigateAction, ViewportAction)
- [x] Separation of concerns (InputState, Session, ViewState)
- [x] Module reorganization (domain/, input/, navigation/, session/, ui/, csv/, file_system/)
- [x] Consistent naming conventions (document, view_state, get_*/move_*/goto_*)
- [x] Comprehensive documentation (all public items documented)
- [x] Zero compiler warnings

---

### v0.2.1 - Type System Cleanup [ ]

**Focus:** Refine type safety and architecture improvements  
**Status:** [ ]  
**Primary Focus:** Type system and module organization polish

**Philosophy:**
Build on v0.2.0's architectural improvements by refining type safety, improving module boundaries, and ensuring all abstractions are clean and maintainable.

**Audit Findings:**
- [x] Domain module: 251 lines (well-sized)
- [x] CSV module: 1,548 lines (document.rs: 1,329 lines)
- [x] Newtype implementations exist (RowIndex, ColIndex)
- [x] Action abstractions in place (UserAction, NavigateAction, ViewportAction)
- [x] Clippy warnings: 3 unnecessary clones on Copy types (VisualSelection)

**Tasks:**
- [ ] Fix unnecessary clones in input/handler.rs (lines 646, 648, 1294)
- [ ] Review and refine RowIndex/ColIndex newtype implementations
- [ ] Add property-based tests for position types (use proptest)
- [ ] Refactor csv/document.rs large functions (if any >50 lines)
- [ ] Document acceptable unwrap() uses in CSV parsing
- [ ] Replace critical unwraps in csv/ module with proper error handling
- [ ] Add comprehensive rustdoc for domain types
- [ ] Add code examples to domain type documentation
- [ ] Verify module boundaries (domain/ should have zero UI dependencies)
- [ ] Add integration tests for type conversions

**Success Criteria:**
- [ ] Zero clippy warnings in domain/ and csv/ modules
- [ ] Code coverage > 90% for domain types
- [ ] All functions < 50 lines in domain/ and csv/
- [ ] All public types have rustdoc with examples
- [ ] Property-based tests for position arithmetic
- [ ] All tests pass with no panics

**Testing Strategy:**
- Property-based tests for type safety (proptest)
- Module integration tests
- Boundary condition testing (overflow, underflow)
- Regression test suite

**Documentation Requirements:**
- Architecture documentation for type system
- Module responsibility documentation (domain/ vs csv/ vs ui/)
- Rustdoc for all public types with examples
- Design decision documentation (why newtypes, why these abstractions)

---

### v0.3.0 - Advanced Navigation & UI Polish [x]

**Focus:** Vim-style navigation enhancements and polished UI  
**Status:** [x]  
**Tests:** 344 passing

**Features:**
- [x] Row jumping (gg, G, 5g for row 5)
- [x] Column jumping (:cA, :cB, :cAA)
- [x] Count prefixes (5j moves down 5 rows, 3h moves left 3)
- [x] Word motion (w, b, e for sparse data navigation)
- [x] Viewport control (zt, zz, zb for scroll positioning)
- [x] Minimal vim-like UI (no heavy borders, clean status line)
- [x] Status bar with mode indicators (NORMAL, COMMAND)
- [x] Transient message system (clears on next keypress)
- [x] Pending command display (shows g_, z_, 5_)
- [x] Auto-width columns based on content (8-50 char range)
- [x] Horizontal file list scrolling
- [x] Reserved commands take priority (:q, :w, :wq, :help)
- [x] Out-of-bounds errors instead of silent clamping

**Consolidates:** v0.3.1 (UI/UX Polish), v0.3.2 (Pre-Edit Polish)

---

### v0.3.1 - Navigation Code Quality [ ]

**Focus:** Refine navigation implementation and UI rendering  
**Status:** [ ]  
**Primary Focus:** Navigation performance and code quality

**Philosophy:**
Improve navigation code quality, optimize rendering performance, and ensure all navigation features are maintainable and well-tested.

**Audit Findings:**
- [x] Navigation module: 887 lines (commands.rs: 873 lines)
- [x] UI module: 3,274 lines (table.rs: 723 lines, status.rs: 426 lines)
- [x] Large functions in navigation/commands.rs (need analysis)
- [x] render_status_bar: 233 lines (CRITICAL)
- [x] render_file_switcher: 132 lines
- [x] build_data_rows: 110 lines
- [x] render_table: 108 lines

**Tasks:**
- [ ] Refactor navigation/commands.rs large functions
- [ ] Extract command parsing logic to separate functions
- [ ] Refactor ui/status.rs::render_status_bar (233 lines → <50 lines)
  - Extract mode indicator rendering
  - Extract position display rendering
  - Extract message rendering
- [ ] Refactor ui/status.rs::render_file_switcher (132 lines → <50 lines)
- [ ] Refactor ui/table.rs::build_data_rows (110 lines → <50 lines)
- [ ] Refactor ui/table.rs::render_table (108 lines → <50 lines)
- [ ] Add rendering performance benchmarks (60 FPS at 100K rows)
- [ ] Add viewport calculation tests
- [ ] Document rendering pipeline in docs/architecture.md
- [ ] Add rustdoc for navigation command functions

**Success Criteria:**
- [ ] Zero clippy warnings in navigation/ and ui/
- [ ] Code coverage > 80% for navigation logic
- [ ] All functions < 50 lines
- [ ] Rendering at 60 FPS for 100K rows (verified by benchmarks)
- [ ] Module structure documented
- [ ] All tests pass with no panics

**Testing Strategy:**
- Navigation command regression tests
- Viewport boundary condition tests (edge of data, scrolling)
- UI rendering performance tests (criterion benchmarks)
- Jump command integration tests (:cA, :cB, 5g, gg, G)

**Documentation Requirements:**
- Navigation architecture documentation
- Viewport management documentation (how scrolling works)
- UI rendering pipeline documentation (table → status → help)
- Performance optimization notes

---

### v0.4.0 - Cell Editing & Persistence [x]

**Focus:** Fast cell editing with Insert mode, file persistence, header toggle  
**Status:** [x]  
**Tests:** 517 passing

**Features:**

**Insert Mode:**
- [x] Enter Insert mode (i, a, I, A, s, F2)
- [x] Text editing (type, backspace, delete, Ctrl+w, Ctrl+u)
- [x] Exit options (Enter/Shift+Enter/Tab/Shift+Tab/Esc)
- [x] Cursor movement (Home, End, arrows)
- [x] Edit buffer with visible cursor

**Row Operations:**
- [x] Insert row (o, O)
- [x] Delete row (dd)
- [x] Yank row (yy)
- [x] Paste row (p, P)
- [x] Clear row (cc)
- [x] Count prefixes (5dd, 5yy)

**File Persistence:**
- [x] :w - Write current file
- [x] :W - Write all dirty files
- [x] :wq / :Wq - Write and quit
- [x] :q - Quit (blocks if dirty)
- [x] :q! - Force quit (discard changes)
- [x] Multi-file dirty tracking with * indicator
- [x] Document caching for unsaved edits
- [x] CSV escaping (quotes, commas, newlines)
- [x] Atomic writes (temp file + rename)

**Header Toggle System:**
- [x] :ht command to toggle header mode ON/OFF
- [x] Header row is always row 0 (no special storage)
- [x] When ON: row 0 styled/frozen, gg goes to row 1
- [x] When OFF: row 0 normal data, gg goes to row 0
- [x] Per-file header mode (session-only, not persisted)
- [x] Deleting row 0 auto-toggles header mode OFF

**Other Commands:**
- [x] :delim ; - Set delimiter (session-only)
- [x] :new A,B,C - Create new CSV with headers
- [x] :files - File menu with cursor navigation
- [x] :c<column> - Column navigation (:cA, :cB, :cAA, :c1)

**Range Operations:**
- [x] Row ranges (:5,10d, :5,10y, :%d, :.d, :$d)
- [x] Column ranges (:B,Dd, :B,Dy)
- [x] Column reordering (:D,E m A)

**Edge Cases:**
- [x] Empty document handling (0 rows, 0 columns)
- [x] Header-only files (0 data rows)
- [x] Absolute row indexing (includes header row)
- [x] Simplified navigation (5g for rows, :c for columns)

**Empty Document Handling Details:**

*Completely Empty File (0 bytes):*
- Opening empty.csv shows: "Empty file (0 rows, 0 columns)"
- Press `o` to create first row (auto-creates "Column 1" header)
- Use `:new Name,Email,Phone` to create headers without data

*Header-Only File (0 data rows):*
- Cursor starts on row 0 (header row)
- Can edit headers with `i`, `a`, `I`, `A`, `s`
- Can navigate columns with `h`, `l`, `0`, `$`
- Press `o` or `O` to insert first data row
- Deleting row 0 with `dd` auto-toggles header mode OFF

*Delete Last Row Workflow:*
- After deleting last data row, cursor moves to row 0 (header)
- Header mode remains ON
- From header row: can edit headers, insert columns, or add new data row

*Delete Last Column Workflow:*
- Deleting last column with `,dd` creates 0-column document (fully supported)
- Use `,o` to add column back
- Status shows position as `1,0`

**Consolidates:** v0.4.1 (Persistence & Edge Cases)

---

### v0.4.1 - Editing System Refactor [ ]

**Focus:** Refine cell editing and persistence implementation  
**Status:** [ ]  
**Primary Focus:** Editing reliability and edge case handling

**Philosophy:**
Improve editing system robustness, ensure persistence is reliable, and handle all edge cases gracefully. Focus on data integrity and user experience.

**Audit Findings:**
- [x] Session module: 618 lines (dirty tracking, file management)
- [x] Input/handler.rs::handle_insert_mode: 183 lines (CRITICAL)
- [x] File persistence logic spread across session/ and csv/writer.rs
- [x] Clippy warning: assert_eq with literal bool (session/mod.rs:577)
- [x] Unwrap calls in file I/O operations (need proper error handling)

**Tasks:**
- [ ] Refactor handle_insert_mode (183 lines → <50 lines)
  - Extract text editing operations
  - Extract cursor movement logic
  - Extract commit/cancel logic
- [ ] Fix clippy warning in session/mod.rs:577 (assert_eq with bool)
- [ ] Review and improve dirty state tracking
- [ ] Add error handling for file I/O operations (replace unwraps)
- [ ] Add tests for edge cases:
  - Editing empty cells
  - Editing cells with special characters
  - Editing with header mode ON/OFF
  - Concurrent edits across multiple files
- [ ] Add integration tests for persistence workflow
- [ ] Document header toggle behavior in code comments
- [ ] Add rustdoc for session management functions

**Success Criteria:**
- [ ] Zero clippy warnings in session/ and editing code
- [ ] Code coverage > 85% for editing operations
- [ ] All functions < 50 lines
- [ ] All file I/O has proper error handling
- [ ] Data integrity tests pass (no data loss scenarios)
- [ ] All tests pass with no panics

**Testing Strategy:**
- Insert mode edge case tests (empty, special chars, unicode)
- File persistence integration tests (write, reload, verify)
- Header toggle scenario tests (toggle on/off, edit header row)
- Dirty state tracking tests (multi-file, unsaved changes)
- Data integrity tests (CSV escaping, newlines, quotes)

**Documentation Requirements:**
- Editing system architecture documentation
- Persistence strategy documentation (atomic writes, temp files)
- Header toggle behavior documentation
- Edge case handling documentation (empty docs, header-only files)

---

### v0.5.0 - Column Operations & Visual Mode [x]

**Focus:** Full column manipulation and visual selections  
**Status:** [x]  
**Tests:** 515+ passing (420 lib + 43 dual clipboard + 20 column range + 32 visual)

**Features:**

**Column Operations (Comma Leader):**
- [x] ,o / ,O - Insert column right/left
- [x] ,dd - Delete column
- [x] ,yy - Yank column (includes header)
- [x] ,p / ,P - Paste column right/left
- [x] Count prefixes (3,dd, 3,yy)
- [x] Cursor moves to new column on paste
- [x] Generic headers on insert ("Column D")

**Visual Mode:**
- [x] v - Visual Block (rectangular selection)
- [x] V - Visual Line (whole rows)
- [x] ,v - Visual Column (whole columns)
- [x] Movement keys (hjkl, arrows) extend selection
- [x] d - Delete selection
- [x] y - Yank selection
- [x] p - Paste over selection
- [x] gv - Re-select last selection
- [x] Visual selection highlighting (bg(Color::DarkGray))

**Triple Clipboard System:**
- [x] Row buffer (yy, dd, p, P, o, O, 5dd, 5yy, Visual Line y)
- [x] Column buffer (,yy, ,dd, ,p, ,P, ,o, ,O, Visual Column y)
- [x] Region buffer (Visual Block y, rectangular selections)
- [x] Buffers isolated (no cross-pasting, no transpose)

**Column Reordering:**
- [x] :D,E m A - Move columns D-E after column A
- [x] :C m $ - Move column C to end
- [x] :A m 0 - Move column A to beginning

---

### v0.5.1 - Column Operations Cleanup [ ]

**Focus:** Refine column operations and visual mode implementation  
**Status:** [ ]  
**Primary Focus:** Column operation reliability and visual mode quality

**Philosophy:**
Improve column operation code quality, ensure visual mode is robust, and handle all clipboard operations reliably. Focus on maintaining data integrity during column manipulations.

**Audit Findings:**
- [x] Clipboard module: 291 lines (well-sized)
- [x] Clippy warning: doc list item without indentation (clipboard/mod.rs:7)
- [x] Visual mode handlers in input/handler.rs:
  - handle_visual_delete: 118 lines
  - handle_visual_paste: 115 lines
- [x] Stale TODOs about clipboard (6 instances - clipboard IS implemented!)
- [x] Tests: 43 dual clipboard tests, 32 visual mode tests

**Tasks:**
- [ ] Fix clippy warning in clipboard/mod.rs:7 (doc formatting)
- [ ] Remove/update 6 stale clipboard TODOs in handler.rs:
  - Lines 1549, 1601, 1660, 1760, 1963
  - Update to reflect that clipboard IS implemented
- [ ] Refactor handle_visual_delete (118 lines → <50 lines)
  - Extract region deletion logic
  - Extract row deletion logic
  - Extract column deletion logic
- [ ] Refactor handle_visual_paste (115 lines → <50 lines)
  - Extract region paste logic
  - Extract row paste logic
  - Extract column paste logic
- [ ] Add tests for triple clipboard isolation (no cross-pasting)
- [ ] Add tests for column reordering edge cases
- [ ] Document triple clipboard system in docs/architecture.md
- [ ] Add rustdoc for clipboard operations

**Success Criteria:**
- [ ] Zero clippy warnings in clipboard/ and visual mode code
- [ ] Code coverage > 85% for column operations
- [ ] All functions < 50 lines
- [ ] Zero stale TODOs
- [ ] Triple clipboard isolation verified by tests
- [ ] All tests pass with no panics

**Testing Strategy:**
- Column operation regression tests (,dd, ,yy, ,p)
- Visual mode selection tests (v, V, ,v)
- Column clipboard integration tests (verify isolation)
- Column reordering edge case tests (:D,E m A)
- Multi-column operation tests (3,dd, 5,yy)

**Documentation Requirements:**
- Column operation architecture documentation
- Visual mode behavior documentation (block, line, column)
- Triple clipboard system documentation (row, column, region buffers)
- Operator-motion composition documentation

---

### v0.6.0 - Magnifier Mode (Full Vim Editor) [x]

**Focus:** Comprehensive vim editor for complex cell editing  
**Status:** [x]  
**Tests:** 415 total (370 lib + 12 integration + 33 advanced)

**Features:**

**Vim Motions:**
- [x] hjkl, arrows - Character movement
- [x] w / b / e - Word navigation
- [x] 0 / $ / ^ - Line motions (start/end/first non-blank)
- [x] gg / G - First/last line
- [x] f/F/t/T{char} - Character find
- [x] ; / , - Repeat find forward/backward
- [x] Count prefixes (5j, 10w, etc.)

**Vim Operators:**
- [x] x - Delete character
- [x] dd - Delete line
- [x] yy - Yank line
- [x] p / P - Paste below/above
- [x] cc - Change line
- [x] C - Change to end of line
- [x] c{motion} - Change operator
- [x] r{char} - Replace character
- [x] J - Join lines
- [x] >> / << - Indent/dedent

**Visual Mode in Magnifier:**
- [x] v - Character-wise visual
- [x] V - Line-wise visual
- [x] d / y / c - Operate on selection

**Search in Magnifier:**
- [x] /pattern - Search forward (case-sensitive)
- [x] n / N - Next/previous match
- [x] * - Search word under cursor
- [x] :noh - Clear search

**Undo/Redo in Magnifier:**
- [x] u - Undo (unlimited history)
- [x] Ctrl+r - Redo

**Ex Commands in Magnifier:**
- [x] :w - Save to cell (updates in-memory document)
- [x] :q - Quit (warns if dirty)
- [x] :wq / ZZ - Save and close
- [x] :q! - Force quit without saving

**Insert Mode Entry:**
- [x] i / a / A / I - Various insert positions
- [x] o / O - Open line below/above
- [x] s - Substitute character

**Cell Navigation:**
- [x] Alt+hjkl or Alt+arrows - Navigate to adjacent cells
- [x] Prompts to save if dirty

**UI Features:**
- [x] Centered popup overlay (80% width/height)
- [x] Title bar shows cell position (e.g., "Editing A5")
- [x] Line numbers (right-aligned, dim)
- [x] Mode indicator (NORMAL/INSERT)
- [x] Cursor position (line:col)
- [x] Bottom help bar with commands
- [x] Different cursor styles (block/pipe)
- [x] Dirty tracking with warnings

---

### v0.6.1 - Magnifier Performance & Quality [ ]

**Focus:** Optimize magnifier mode and improve code quality  
**Status:** [ ]  
**Primary Focus:** Magnifier mode performance and maintainability

**Philosophy:**
Improve magnifier mode performance, ensure all vim operations are efficient and correct, and maintain high code quality. Focus on user experience and editor responsiveness.

**Audit Findings:**
- [x] Magnifier module: 2,020 lines (magnifier/mod.rs is the entire module)
- [x] UI magnifier rendering: ui/magnifier.rs (482 lines)
- [x] Large functions:
  - handle_magnifier_normal: 154 lines (in input/handler.rs)
  - render_magnifier: 100 lines (in ui/magnifier.rs)
- [x] Tests: 12 integration tests, 33 advanced tests

**Tasks:**
- [ ] Refactor handle_magnifier_normal (154 lines → <50 lines)
  - Extract vim motion handling
  - Extract vim operator handling
  - Extract search handling
  - Extract mode transition logic
- [ ] Refactor render_magnifier (100 lines → <50 lines)
  - Extract title bar rendering
  - Extract content rendering
  - Extract status bar rendering
  - Extract help bar rendering
- [ ] Review magnifier/mod.rs for large functions
- [ ] Add performance benchmarks for magnifier operations:
  - Large text rendering (1K, 10K lines)
  - Vim motion performance
  - Undo/redo performance
- [ ] Add tests for edge cases:
  - Very long lines (>1000 chars)
  - Many lines (>10K)
  - Unicode and emoji handling
- [ ] Document vim operation implementation
- [ ] Add rustdoc for magnifier public API

**Success Criteria:**
- [ ] Zero clippy warnings in magnifier/ and ui/magnifier.rs
- [ ] Code coverage > 80% for magnifier operations
- [ ] All functions < 50 lines
- [ ] Magnifier responsive for large text (>10K lines)
- [ ] All vim operations work correctly
- [ ] All tests pass with no panics

**Testing Strategy:**
- Vim operation regression tests (motions, operators, visual mode)
- Performance benchmarks for editing large text
- Undo/redo correctness tests (unlimited history)
- Modal state transition tests (normal ↔ insert ↔ visual)
- Large text handling tests (10K+ lines)

**Documentation Requirements:**
- Magnifier architecture documentation (how it integrates with main app)
- Vim operation implementation notes (which vim features supported)
- Performance optimization documentation
- Modal state machine documentation (mode transitions)

---

### v0.7.0 - Search & Filtering [x]

**Focus:** Find and navigate data across the entire CSV  
**Status:** [x]  
**Tests:** 27 passing

**Features:**
- [x] /pattern - Search forward (regex supported, case-insensitive)
- [x] n - Jump to next match with wrap-around
- [x] N - Jump to previous match with wrap-around
- [x] * - Search current cell content
- [x] :noh - Clear search highlighting
- [x] Esc in Normal mode clears search
- [x] Visual match highlighting (current match vs others)
- [x] Match counter in status line [n/total]
- [x] Invalid regex falls back to literal substring search

**Implementation:**
- Module: src/search/mod.rs
- SearchState struct with pattern, matches, current_match
- find_matches() with regex support
- jump_to_next() / jump_to_prev() with wrap-around
- UI highlighting in src/ui/table.rs

---

### v0.7.1 - Search System Optimization [ ]

**Focus:** Optimize search and filtering implementation  
**Status:** [ ]  
**Primary Focus:** Search performance and code quality

**Philosophy:**
Improve search system performance, ensure filtering is efficient for large datasets, and maintain clean, testable code. Focus on search responsiveness and accuracy.

**Audit Findings:**
- [x] Search module: 398 lines (search/mod.rs)
- [x] Tests: 27 passing search tests
- [x] Search features: regex support, case-insensitive, wrap-around
- [x] Highlight rendering integrated in ui/table.rs

**Tasks:**
- [ ] Review search/mod.rs for functions >50 lines
- [ ] Add performance benchmarks for search:
  - Search in small dataset (1K rows)
  - Search in medium dataset (10K rows)
  - Search in large dataset (100K rows)
  - Regex vs literal search performance
- [ ] Optimize search algorithm if needed (consider caching)
- [ ] Add tests for edge cases:
  - Empty search pattern
  - Pattern not found
  - Pattern in every cell
  - Very long regex patterns
  - Invalid regex fallback
- [ ] Add tests for highlight rendering performance
- [ ] Document search algorithm in code comments
- [ ] Add rustdoc for search public API
- [ ] Document regex pattern handling and fallback behavior

**Success Criteria:**
- [ ] Zero clippy warnings in search/
- [ ] Code coverage > 85% for search operations
- [ ] All functions < 50 lines
- [ ] Search responsive for 100K+ rows (<100ms)
- [ ] Regex compilation errors handled gracefully
- [ ] All tests pass with no panics

**Testing Strategy:**
- Search performance benchmarks (1K, 10K, 100K rows)
- Filtering correctness tests (regex, literal, case-insensitive)
- Regex pattern edge case tests (invalid, empty, special chars)
- Large dataset search tests (verify wrap-around, match counter)
- Highlight rendering tests (current match vs other matches)

**Documentation Requirements:**
- Search algorithm documentation (how matches are found and stored)
- Filtering strategy documentation
- Performance characteristics documentation (O(n) search, caching strategy)
- Regex pattern handling documentation (compilation, fallback to literal)

---

### v0.8.0 - SQL Query Mode & Data Operations [x]

**Focus:** Powerful SQL queries across CSV files with data operations  
**Status:** [x]  
**Tests:** 30 passing (19 lib + 11 integration)

**Features:**

**SQL Query Mode:**
- [x] :q <SQL> - Execute SQL query, open editor with query
- [x] :sql - Open empty SQL editor
- [x] Load CSV files into SQLite automatically
- [x] Multi-file JOIN support (all CSVs in directory)
- [x] Query results displayed as read-only CSV view
- [x] Column name normalization (spaces, special chars → underscores)
- [x] Handles missing cells and inconsistent column counts
- [x] Useful error messages for SQL errors (e.g., misspelled columns)
- [x] Esc cancels query execution
- [x] SQLite connection caching for performance

**SQL Editor:**
- [x] Full editing support (type, backspace, arrows)
- [x] Enter - Execute query
- [x] Esc - Cancel/close editor
- [x] Loading indicator during query execution
- [x] Mode indicator "SQL"

**Sort Commands:**
- [x] :sort col1,col2 - Sort by columns ascending
- [x] :sort! col1,col2 - Sort by columns descending
- [x] Works with column names or numbers

**File Management:**
- [x] External file modification detection
- [x] Prompt user to reload when file changes externally
- [x] reload_current_file_cancellable() method

**Implementation:**
- Module: src/query/mod.rs (576 lines)
- UI: src/ui/sql_editor.rs
- table_name_from_path() - Derive table names from file paths
- load_csv_into_sqlite() - Load Document into SQLite table
- execute_query() - Run SQL and convert back to Document
- Session tracking for query output sheets

**Example Queries:**
```sql
SELECT * FROM customers WHERE age > 30
SELECT c.name, o.total FROM customers c JOIN orders o ON c.id = o.customer_id
SELECT country, COUNT(*) as count FROM customers GROUP BY country ORDER BY count DESC
```

---

### v0.8.1 - SQL & Data Operations Polish [ ]

**Focus:** Refine SQL query mode and data operations  
**Status:** [ ]  
**Primary Focus:** SQL query reliability and performance

**Philosophy:**
Improve SQL query mode robustness, ensure data operations are reliable, and maintain high performance for complex queries. Focus on query correctness and error handling.

**Audit Findings:**
- [x] Query module: 576 lines (query/mod.rs)
- [x] App module: 2,653 lines (app/mod.rs contains SQL execution)
- [x] Large functions:
  - execute_sql_query_cancellable: 164 lines (in app/mod.rs)
  - handle_sql_editor_mode: 125 lines (in input/handler.rs)
  - render_sql_editor_overlay: 118 lines (in ui/sql_editor.rs)
- [x] Tests: 19 lib tests, 11 integration tests
- [x] Uses SQLite (not DuckDB as roadmap mentioned)

**Tasks:**
- [ ] Refactor execute_sql_query_cancellable (164 lines → <50 lines)
  - Extract CSV to SQLite loading logic
  - Extract query execution logic
  - Extract result conversion logic
  - Extract error handling logic
- [ ] Refactor handle_sql_editor_mode (125 lines → <50 lines)
  - Extract input handling
  - Extract query execution trigger
  - Extract editor state management
- [ ] Refactor render_sql_editor_overlay (118 lines → <50 lines)
  - Extract editor rendering
  - Extract status rendering
  - Extract help text rendering
- [ ] Add performance benchmarks for SQL operations:
  - Load CSV into SQLite (1K, 10K, 100K rows)
  - Simple SELECT query
  - Complex JOIN query
  - Aggregation query (GROUP BY, COUNT)
- [ ] Add tests for SQL edge cases:
  - Invalid SQL syntax
  - Misspelled column names
  - Empty result sets
  - Very large result sets
- [ ] Improve error messages for SQL errors
- [ ] Document SQLite integration in docs/architecture.md
- [ ] Add rustdoc for query public API

**Success Criteria:**
- [ ] Zero clippy warnings in query/ and SQL-related code
- [ ] Code coverage > 80% for SQL operations
- [ ] All functions < 50 lines
- [ ] SQL queries responsive for 100K+ rows
- [ ] Error messages are user-friendly
- [ ] All tests pass with no panics

**Testing Strategy:**
- SQL query correctness tests (SELECT, JOIN, GROUP BY, ORDER BY)
- Complex query performance benchmarks
- Error handling tests for invalid SQL (syntax errors, missing columns)
- Data operation integration tests (sort, filter via SQL)
- Large dataset query tests (100K+ rows)

**Documentation Requirements:**
- SQL query mode architecture documentation (how CSVs become SQLite tables)
- SQLite integration documentation (connection caching, table naming)
- Query performance optimization notes
- Error handling strategy documentation (user-friendly SQL error messages)

---

### v0.9.0 - Undo/Redo & Command History [ ]

**Focus:** Complete command history for all mutations  
**Status:** [ ]  
**Target Tests:** 50+

**Features:**
- [ ] u - Undo last operation
- [ ] Ctrl+r - Redo last undone operation
- [ ] . - Repeat last edit (dot command)
- [ ] Per-file undo history (preserved across file switches)
- [ ] Max undo levels: 1000 per file (configurable)
- [ ] Undo stack preserved after :w (save doesn't clear history)
- [ ] Single-step granularity (5dd = 1 undo step, not 5)

**Undo Granularity:**
- Single operations: edit cell (i), delete row (dd), insert row (o) = 1 undo step
- Compound operations: 5dd (delete 5 rows) = 1 undo step (NOT 5 separate steps)
- Visual mode operations: delete selection = 1 undo step
- Range operations: :5,10d = 1 undo step

**History Management:**
- `:w` saves file but PRESERVES undo history
- File switching preserves undo history per file (stored in session)
- Undo/redo only works within current file (can't undo across files)

**Limitations:**
- Cannot undo file switch
- Cannot undo :w (file write)
- Cannot undo :q (quit)

**Implementation Plan:**
- File: src/history/mod.rs (new file)
  - [ ] Create history module
  - [ ] Define EditCommand enum (variants for all mutation types)
  - [ ] Define History struct with undo/redo stacks
  - [ ] Implement push_command() method
  - [ ] Implement undo() method
  - [ ] Implement redo() method
  - [ ] Implement clear_redo_stack() on new command
  - [ ] Respect max undo limit

- File: src/app/mod.rs
  - [ ] Add history: History field
  - [ ] Record all mutations to history
  - [ ] Add last_edit_command: Option<EditCommand> for dot command

- File: src/session/mod.rs
  - [ ] Store per-file history in HashMap<PathBuf, History>
  - [ ] Preserve history across file switches

- File: src/input/handler.rs
  - [ ] Add u handler
  - [ ] Add Ctrl+r handler
  - [ ] Add . handler (repeat last edit)

**Tests:**
- [ ] test_u_undoes_cell_edit
- [ ] test_u_undoes_row_delete
- [ ] test_u_undoes_column_delete
- [ ] test_5dd_creates_single_undo_step
- [ ] test_ctrl_r_redoes
- [ ] test_dot_repeats_last_edit
- [ ] test_undo_limit_respected
- [ ] test_w_preserves_undo_history
- [ ] test_file_switch_preserves_history
- [ ] test_new_command_clears_redo_stack

---

### v0.9.1 - Code Quality & Architecture Refactor [ ]

**Focus:** Improve code organization, reduce complexity, and establish quality baselines  
**Status:** Refactoring milestone  
**Type:** Maintenance & Quality

**Audit Phase (Complete First):**
- [ ] Run `cargo clippy --all-targets` and document all warnings
- [ ] Measure current code coverage with `cargo tarpaulin` or `cargo llvm-cov`
- [ ] Identify all functions > 50 lines
- [ ] Profile performance with `cargo flamegraph` or `perf`
- [ ] Review module dependencies and coupling
- [ ] Calculate cyclomatic complexity for complex functions
- [ ] Review error handling patterns (find all `unwrap()`, `expect()`)
- [ ] Document technical debt items

**Success Criteria:**
- [ ] Zero clippy warnings
- [ ] Code coverage > 80%
- [ ] All functions < 50 lines (or documented exceptions)
- [ ] Performance benchmarks established and met
- [ ] Module structure documented in docs/architecture.md
- [ ] Cyclomatic complexity reduced for flagged functions
- [ ] All tests pass with no panics or unwrap failures
- [ ] Error handling follows consistent patterns

**Testing Strategy:**
- [ ] All existing tests pass (regression testing)
- [ ] Add tests to reach coverage target
- [ ] Consider property-based tests for complex logic
- [ ] Create benchmark suite for performance tracking

**Documentation Updates:**
- [ ] Update docs/architecture.md with module structure
- [ ] Add inline comments for complex logic
- [ ] Ensure all public APIs have rustdoc comments
- [ ] Document refactoring decisions and trade-offs
- [ ] Record performance improvements with metrics

**Common Refactoring Patterns:**
- Extract large functions into smaller, testable units
- Reduce code duplication through abstractions
- Simplify complex conditional logic
- Improve naming consistency across modules
- Replace `unwrap()` with proper error handling
- Reduce coupling between modules

---

### v0.10.0 - Cell Transforms & Data Cleanup [ ]

**Focus:** Case transformations, boolean toggles, and row movement  
**Status:** [ ]  
**Target Tests:** 30+

**Features:**

**Case Transforms:**
- [ ] ~ - Toggle case (UPPER <-> lower)
- [ ] gU - Uppercase entire cell
- [ ] gu - Lowercase entire cell
- [ ] g~ - Title Case cell

**Data Transforms:**
- [ ] g. - Toggle boolean (yes<->no, true<->false, 1<->0)

**Row Movement:**
- [ ] gj - Swap current row with row below
- [ ] gk - Swap current row with row above

**Implementation Plan:**
- File: src/transforms/mod.rs (new file)
  - [ ] Create transforms module
  - [ ] Implement toggle_case() function
  - [ ] Implement uppercase() function
  - [ ] Implement lowercase() function
  - [ ] Implement title_case() function
  - [ ] Implement toggle_boolean() function

- File: src/csv/document.rs
  - [ ] Add swap_rows(&mut self, a: RowIndex, b: RowIndex) method
  - [ ] Add apply_transform(&mut self, transform: TransformFn, pos: Position) method

- File: src/input/handler.rs
  - [ ] Add ~ handler
  - [ ] Add gU, gu, g~, g. handlers
  - [ ] Add gj, gk handlers

**Tests:**
- [ ] test_tilde_toggles_case
- [ ] test_gU_uppercases_cell
- [ ] test_gu_lowercases_cell
- [ ] test_g_tilde_title_cases_cell
- [ ] test_g_dot_toggles_boolean
- [ ] test_gj_swaps_row_below
- [ ] test_gk_swaps_row_above

---

### v0.10.1 - Performance Optimization & Profiling [ ]

**Focus:** Profile hot paths, optimize rendering, reduce allocations  
**Status:** Refactoring milestone  
**Type:** Performance & Optimization

**Audit Phase (Complete First):**
- [ ] Profile with `cargo flamegraph` to identify hot paths
- [ ] Run `cargo bench` to establish performance baselines
- [ ] Measure memory usage with `heaptrack` or `valgrind massif`
- [ ] Identify unnecessary allocations and clones
- [ ] Review rendering pipeline efficiency
- [ ] Check for N+1 query patterns in document operations
- [ ] Profile large file loading (100K+ rows)

**Success Criteria:**
- [ ] Zero clippy warnings (performance lints enabled)
- [ ] Maintain 60 FPS for 100K+ row files
- [ ] Reduce memory usage by 10-20% where possible
- [ ] File loading < 100ms for 10MB files
- [ ] No performance regressions in benchmark suite
- [ ] Document performance characteristics in code

**Testing Strategy:**
- [ ] Create comprehensive benchmark suite
- [ ] Add performance regression tests
- [ ] Test with large datasets (100K, 500K, 1M rows)
- [ ] Ensure all existing tests still pass

**Documentation Updates:**
- [ ] Document performance optimization decisions
- [ ] Add performance notes to hot path functions
- [ ] Update architecture docs with performance considerations
- [ ] Record before/after benchmark results

**Optimization Targets:**
- Rendering pipeline (viewport calculations, cell drawing)
- Document operations (insert, delete, search)
- File I/O and parsing
- Memory allocations in tight loops
- String handling and formatting

---

### v0.11.0 - System Clipboard & External Integration [ ]

**Focus:** Integration with system clipboard for copy/paste with external tools  
**Status:** [ ]  
**Target Tests:** 20+

**Features:**

**System Clipboard:**
- [ ] "+yy - Yank row to system clipboard (CSV format)
- [ ] "+,yy - Yank column to system clipboard (CSV format)
- [ ] "+y - Yank visual selection to system clipboard
- [ ] "+p - Paste from system clipboard (auto-detect format)
- [ ] Support for TSV from system clipboard
- [ ] Support for plain text (single cell or column)

**Implementation Plan:**
- File: src/clipboard/mod.rs (new file)
  - [ ] Add system clipboard integration (use `arboard` crate)
  - [ ] Implement copy_to_system() method
  - [ ] Implement paste_from_system() method
  - [ ] Implement format detection (CSV, TSV, plain text)
  - [ ] Handle clipboard errors gracefully

- File: src/input/handler.rs
  - [ ] Add "+yy, "+,yy, "+y handlers
  - [ ] Add "+p handler

**Tests:**
- [ ] test_system_clipboard_yank_row
- [ ] test_system_clipboard_yank_column
- [ ] test_system_clipboard_yank_visual
- [ ] test_system_clipboard_paste_csv
- [ ] test_system_clipboard_paste_tsv
- [ ] test_system_clipboard_paste_plain_text

---

### v0.11.1 - Testing & Reliability Improvements [ ]

**Focus:** Increase test coverage, add property-based tests, improve reliability  
**Status:** Refactoring milestone  
**Type:** Testing & Quality Assurance

**Audit Phase (Complete First):**
- [ ] Run coverage report (`cargo tarpaulin` or `cargo llvm-cov`)
- [ ] Identify untested or under-tested modules
- [ ] Review test quality (unit vs integration balance)
- [ ] Find edge cases that lack tests
- [ ] Review error paths for test coverage
- [ ] Identify areas suitable for property-based testing

**Success Criteria:**
- [ ] Code coverage > 80% (90%+ for critical paths)
- [ ] All error paths have tests
- [ ] Property-based tests for complex algorithms
- [ ] Integration tests cover major workflows
- [ ] No flaky tests in CI
- [ ] All panics and unwraps have test coverage

**Testing Strategy:**
- [ ] Add unit tests for under-tested modules
- [ ] Write integration tests for user workflows
- [ ] Implement property-based tests using `proptest` or `quickcheck`
- [ ] Add fuzzing targets for parsers
- [ ] Test error handling paths explicitly
- [ ] Add regression tests for all discovered bugs

**Documentation Updates:**
- [ ] Document testing strategy in docs/development.md
- [ ] Add examples of property-based tests
- [ ] Document test organization and patterns
- [ ] Update CI/CD documentation

**Focus Areas:**
- CSV parsing edge cases (malformed files, encoding issues)
- Navigation boundary conditions
- Undo/redo state consistency
- Multi-file session management
- Error handling and recovery

---

### v0.12.0 - Bulk Operations & Find/Replace [ ]

**Focus:** Find and replace across cells, rows, and columns  
**Status:** [ ]  
**Target Tests:** 40+

**Features:**

**Find and Replace:**
- [ ] :%s/old/new/ - Replace first occurrence in all cells
- [ ] :%s/old/new/g - Replace all occurrences in all cells
- [ ] :5,10s/old/new/g - Replace in row range
- [ ] :B,Ds/old/new/g - Replace in column range
- [ ] Visual mode selection replace
- [ ] Regex support for find patterns
- [ ] Case-sensitive and case-insensitive options
- [ ] Confirmation prompts (:%s/old/new/gc)

**Bulk Delete/Clear:**
- [ ] :%d - Delete all data rows (already implemented)
- [ ] :B,Dd - Delete column range (already implemented)
- [ ] :%clear - Clear all cell contents (preserve structure)

**Implementation Plan:**
- File: src/find_replace/mod.rs (new file)
  - [ ] Create find_replace module
  - [ ] Implement parse_substitute_command()
  - [ ] Implement apply_substitute() method
  - [ ] Support regex patterns
  - [ ] Handle confirmation mode

- File: src/input/command.rs
  - [ ] Add :s command parser
  - [ ] Handle range parsing (:%, :5,10, :B,D)

**Tests:**
- [ ] test_substitute_all_cells
- [ ] test_substitute_row_range
- [ ] test_substitute_column_range
- [ ] test_substitute_with_regex
- [ ] test_substitute_case_sensitive
- [ ] test_substitute_with_confirmation

---

### v0.12.1 - Error Handling & Robustness [ ]

**Focus:** Consistent error handling, better error messages, resilience  
**Status:** Refactoring milestone  
**Type:** Reliability & Error Handling

**Audit Phase (Complete First):**
- [ ] Find all `unwrap()` and `expect()` calls
- [ ] Review `Result` and `Option` handling patterns
- [ ] Identify error types that could be more descriptive
- [ ] Review error propagation and context
- [ ] Check for silent error swallowing
- [ ] Test error handling paths

**Success Criteria:**
- [ ] No production `unwrap()` calls (test code OK)
- [ ] All errors have helpful messages for users
- [ ] Consistent error types across modules
- [ ] Error context preserved through call stack
- [ ] All error paths tested
- [ ] Graceful degradation where possible

**Testing Strategy:**
- [ ] Add negative tests for all error conditions
- [ ] Test file I/O errors (permissions, disk full, etc.)
- [ ] Test malformed CSV files
- [ ] Test resource exhaustion scenarios
- [ ] Verify error messages are user-friendly

**Documentation Updates:**
- [ ] Document error handling patterns
- [ ] Add error handling guidelines for contributors
- [ ] Document error types and when to use them
- [ ] Update user-facing error messages

**Error Handling Improvements:**
- Replace `unwrap()` with proper error handling
- Add context to errors (which file, which operation)
- Implement custom error types where needed
- Improve error recovery mechanisms
- Add user-friendly error messages

---

### v0.13.0 - Advanced Filtering & Conditional Views [ ]

**Focus:** Show/hide rows based on conditions, highlight cells  
**Status:** [ ]  
**Target Tests:** 35+

**Features:**

**Row Filtering:**
- [ ] :filter col=value - Show only rows where column equals value
- [ ] :filter col>10 - Show rows where column > 10
- [ ] :filter col~pattern - Show rows matching regex pattern
- [ ] :filter! - Clear filter, show all rows
- [ ] Visual indicator for filtered view
- [ ] Filter status in status line

**Conditional Formatting:**
- [ ] Highlight cells based on value (e.g., negative numbers in red)
- [ ] Highlight duplicates
- [ ] Highlight empty cells
- [ ] Custom color rules

**Implementation Plan:**
- File: src/filter/mod.rs (new file)
  - [ ] Create filter module
  - [ ] Define FilterCondition enum
  - [ ] Implement parse_filter_command()
  - [ ] Implement apply_filter() method
  - [ ] Track filtered row indices

- File: src/ui/table.rs
  - [ ] Update render to skip filtered rows
  - [ ] Add conditional formatting logic

**Tests:**
- [ ] test_filter_by_equality
- [ ] test_filter_by_comparison
- [ ] test_filter_by_regex
- [ ] test_clear_filter
- [ ] test_multiple_filters
- [ ] test_conditional_formatting

---

### v0.13.1 - Module Organization & Cleanup [ ]

**Focus:** Improve module structure, reduce coupling, clean interfaces  
**Status:** Refactoring milestone  
**Type:** Architecture & Organization

**Audit Phase (Complete First):**
- [ ] Review module dependencies with `cargo modules` or similar
- [ ] Identify circular dependencies
- [ ] Find modules with too many responsibilities
- [ ] Review public API surface area
- [ ] Check for leaky abstractions
- [ ] Identify overly-coupled modules

**Success Criteria:**
- [ ] Clear module responsibilities documented
- [ ] No circular dependencies
- [ ] Reduced coupling between modules
- [ ] Clean public APIs with minimal surface area
- [ ] Logical module hierarchy
- [ ] Updated architecture documentation

**Testing Strategy:**
- [ ] Ensure all tests pass after reorganization
- [ ] Add tests for new module boundaries
- [ ] Verify integration points work correctly

**Documentation Updates:**
- [ ] Update docs/architecture.md with module structure
- [ ] Document module responsibilities
- [ ] Add module dependency diagram
- [ ] Update API documentation

**Organization Improvements:**
- Split large modules into focused sub-modules
- Extract shared code into utility modules
- Clarify module boundaries and responsibilities
- Reduce public API surface where possible
- Improve naming consistency across modules

---

### v0.14.0 - Column Resize & Advanced Column Operations [ ]

**Focus:** Manual column width control, column pinning, and metadata  
**Status:** [ ]  
**Target Tests:** 30+

**Features:**

**Column Width:**
- [ ] :width A 20 - Set column A width to 20 characters
- [ ] :width B auto - Auto-size column B
- [ ] :width * auto - Auto-size all columns
- [ ] Manual resize with mouse or keybindings
- [ ] Per-file column width memory (session)

**Column Pinning:**
- [ ] :freeze A,B - Freeze columns A and B (always visible)
- [ ] :unfreeze - Unfreeze all columns
- [ ] Visual indicator for frozen columns

**Column Metadata:**
- [ ] :type A number - Mark column A as numeric
- [ ] :type B date - Mark column B as date
- [ ] Type validation on edit
- [ ] Type-aware sorting (numbers sort numerically)

**Implementation Plan:**
- File: src/column/metadata.rs (new file)
  - [ ] Define ColumnMetadata struct
  - [ ] Track column widths, types, frozen state
  - [ ] Per-file metadata storage in Session

- File: src/ui/table.rs
  - [ ] Implement frozen column rendering
  - [ ] Use custom widths instead of auto-sizing

**Tests:**
- [ ] test_set_column_width
- [ ] test_auto_size_column
- [ ] test_freeze_columns
- [ ] test_column_type_validation
- [ ] test_numeric_sort

---

### v0.14.1 - Documentation & Maintainability [ ]

**Focus:** Comprehensive documentation, code comments, maintainability  
**Status:** Refactoring milestone  
**Type:** Documentation & Maintainability

**Audit Phase (Complete First):**
- [ ] Run `cargo doc` and review for missing docs
- [ ] Find complex functions lacking comments
- [ ] Review public API documentation quality
- [ ] Check architecture docs for accuracy
- [ ] Review inline comments for clarity
- [ ] Identify undocumented design decisions

**Success Criteria:**
- [ ] All public items have rustdoc comments
- [ ] Complex algorithms have explanatory comments
- [ ] Architecture docs up-to-date and comprehensive
- [ ] API examples provided where helpful
- [ ] Design decisions documented
- [ ] Contributing guide updated

**Testing Strategy:**
- [ ] Ensure doc tests pass (`cargo test --doc`)
- [ ] Add examples to documentation
- [ ] Verify doc coverage meets standards

**Documentation Updates:**
- [ ] Complete API documentation (rustdoc)
- [ ] Add code examples to docs
- [ ] Update docs/architecture.md comprehensively
- [ ] Document design patterns and idioms used
- [ ] Add inline comments for complex logic
- [ ] Update docs/development.md for contributors

**Maintainability Improvements:**
- Improve function and variable naming
- Add high-level module documentation
- Document non-obvious code patterns
- Add TODO/FIXME tracking
- Clarify ownership and lifetime patterns

---

### v0.15.0 - Data Analysis & Statistics [ ]

**Focus:** Basic statistical analysis and aggregation  
**Status:** [ ]  
**Target Tests:** 25+

**Features:**

**Column Statistics:**
- [ ] :stats A - Show statistics for column A (sum, avg, min, max, count)
- [ ] :sum A - Show sum of column A
- [ ] :avg A - Show average of column A
- [ ] :count A - Count non-empty cells in column A
- [ ] :distinct A - Count distinct values in column A

**Aggregation:**
- [ ] Visual mode selection statistics
- [ ] Footer row showing column totals (optional)
- [ ] Statistics overlay/popup

**Implementation Plan:**
- File: src/stats/mod.rs (new file)
  - [ ] Create statistics module
  - [ ] Implement calculate_stats() function
  - [ ] Handle numeric vs text columns
  - [ ] Parse numeric values safely

- File: src/ui/stats_overlay.rs (new file)
  - [ ] Create statistics display overlay
  - [ ] Format statistics nicely

**Tests:**
- [ ] test_stats_numeric_column
- [ ] test_stats_text_column
- [ ] test_sum_column
- [ ] test_avg_column
- [ ] test_count_non_empty
- [ ] test_distinct_values

---

### v0.15.1 - Technical Debt Reduction [ ]

**Focus:** Address accumulated TODOs, simplify complex code, pay down debt  
**Status:** Refactoring milestone  
**Type:** Technical Debt & Simplification

**Audit Phase (Complete First):**
- [ ] Find all TODO, FIXME, HACK, XXX comments
- [ ] Identify complex functions (high cyclomatic complexity)
- [ ] Review code duplication with tools
- [ ] Find deprecated patterns still in use
- [ ] Identify over-engineered solutions
- [ ] Review dead code and unused features

**Success Criteria:**
- [ ] All critical TODOs addressed
- [ ] Complex functions simplified or documented
- [ ] Code duplication significantly reduced
- [ ] No deprecated patterns in use
- [ ] Dead code removed
- [ ] Consistent coding patterns throughout

**Testing Strategy:**
- [ ] Ensure refactoring doesn't break functionality
- [ ] Add tests for previously untested areas
- [ ] Remove tests for removed functionality

**Documentation Updates:**
- [ ] Document resolved technical debt items
- [ ] Update known issues list
- [ ] Document remaining acceptable debt
- [ ] Add lessons learned

**Debt Reduction Targets:**
- Resolve high-priority TODOs
- Simplify overly complex functions
- Extract duplicated code
- Remove unused code and features
- Standardize inconsistent patterns
- Update deprecated API usage

---

### v0.16.0 - Export & Import (JSON, Markdown, TSV) [ ]

**Focus:** Export CSV to other formats, import from TSV  
**Status:** [ ]  
**Target Tests:** 30+

**Features:**

**Export:**
- [ ] :export json - Export current file to JSON
- [ ] :export markdown - Export to Markdown table
- [ ] :export html - Export to HTML table
- [ ] :export tsv - Export to TSV
- [ ] Visual selection export
- [ ] Output file path prompt

**Import:**
- [ ] :import file.tsv - Import TSV file
- [ ] :import file.json - Import JSON (array of objects)
- [ ] Auto-detect format on file open

**Implementation Plan:**
- File: src/export/mod.rs (new file)
  - [ ] Create export module
  - [ ] Implement export_json()
  - [ ] Implement export_markdown()
  - [ ] Implement export_html()
  - [ ] Implement export_tsv()

- File: src/import/mod.rs (new file)
  - [ ] Create import module
  - [ ] Implement import_tsv()
  - [ ] Implement import_json()

**Tests:**
- [ ] test_export_json
- [ ] test_export_markdown
- [ ] test_export_html
- [ ] test_export_tsv
- [ ] test_import_tsv
- [ ] test_import_json

---

### v0.16.1 - Code Coverage & Test Quality [ ]

**Focus:** Maximize test coverage, improve test quality and organization  
**Status:** Refactoring milestone  
**Type:** Testing Excellence

**Audit Phase (Complete First):**
- [ ] Generate detailed coverage report
- [ ] Identify untested branches and paths
- [ ] Review test organization and naming
- [ ] Find slow or flaky tests
- [ ] Check test duplication
- [ ] Review test assertions quality

**Success Criteria:**
- [ ] Code coverage > 85% (stretch: 90%+)
- [ ] All critical paths have tests
- [ ] Fast test suite (< 30s for full run)
- [ ] No flaky tests
- [ ] Well-organized test modules
- [ ] Meaningful test names and assertions

**Testing Strategy:**
- [ ] Add tests for all uncovered code paths
- [ ] Improve test organization by feature area
- [ ] Add edge case and boundary tests
- [ ] Ensure error paths are tested
- [ ] Add integration tests for workflows
- [ ] Remove or fix flaky tests

**Documentation Updates:**
- [ ] Document test organization patterns
- [ ] Add testing best practices guide
- [ ] Document coverage requirements
- [ ] Update CI test documentation

**Test Quality Improvements:**
- Improve test naming conventions
- Add descriptive assertion messages
- Organize tests by feature/module
- Remove duplicate test code
- Speed up slow tests
- Add missing edge case tests

---

### v0.17.0 - Configuration System [ ]

**Focus:** User customization via config files  
**Status:** [ ]  
**Target Tests:** 20+

**Features:**

**Config File Support:**
- [ ] Load config from ~/.config/lazycsv/config.toml
- [ ] Per-directory config (./.lazycsv/config.toml)
- [ ] Default settings (delimiter, header_mode, undo_limit)
- [ ] Color customization
- [ ] Keybinding remapping (advanced)

**Config Options:**
```toml
[defaults]
delimiter = ","
header_mode = true
undo_limit = 1000

[colors]
header_bg = "blue"
cursor_fg = "yellow"
dirty_indicator = "red"

[keybindings]
quit = ":q"
save_all = ":w"
```

**Implementation Plan:**
- File: src/config/mod.rs (new file)
  - [ ] Create config module
  - [ ] Define Config struct
  - [ ] Implement load_config()
  - [ ] Parse TOML with `toml` crate
  - [ ] Merge global + directory configs

- File: src/app/mod.rs
  - [ ] Load config on startup
  - [ ] Apply config settings

**Tests:**
- [ ] test_load_global_config
- [ ] test_load_directory_config
- [ ] test_config_merge
- [ ] test_invalid_config_handling

---

### v0.17.1 - Performance Benchmarking & Tuning [ ]

**Focus:** Establish benchmarks, tune critical paths, validate performance  
**Status:** Refactoring milestone  
**Type:** Performance Validation

**Audit Phase (Complete First):**
- [ ] Create comprehensive benchmark suite
- [ ] Profile with real-world datasets (10K, 100K, 1M rows)
- [ ] Identify performance regressions since last version
- [ ] Measure memory usage patterns
- [ ] Test with various CSV sizes and complexities
- [ ] Profile startup time and file loading

**Success Criteria:**
- [ ] Comprehensive benchmark suite in place
- [ ] 60 FPS maintained for 100K+ row files
- [ ] File loading < 100ms for 10MB files
- [ ] Memory usage within acceptable bounds
- [ ] No performance regressions
- [ ] Performance characteristics documented

**Testing Strategy:**
- [ ] Add criterion benchmarks for critical paths
- [ ] Test performance with large datasets
- [ ] Add memory usage tests
- [ ] Create performance regression tests
- [ ] Benchmark against previous versions

**Documentation Updates:**
- [ ] Document performance characteristics
- [ ] Add benchmark results to docs
- [ ] Document performance optimization decisions
- [ ] Update performance goals and targets

**Performance Validation:**
- Navigation responsiveness (hjkl, gg, G, searches)
- Rendering performance (viewport updates, redraws)
- File operations (load, save, switch)
- Search and filter operations
- Large dataset handling
- Memory efficiency

---

### v0.18.0 - Macros & Command Recording [ ]

**Focus:** Record and replay command sequences  
**Status:** [ ]  
**Target Tests:** 25+

**Features:**

**Macro Recording:**
- [ ] qa - Start recording macro into register 'a'
- [ ] q - Stop recording
- [ ] @a - Replay macro from register 'a'
- [ ] @@ - Replay last macro
- [ ] Support for multiple registers (a-z)

**Command History:**
- [ ] :history - Show command history
- [ ] Up/Down arrows in command mode to navigate history
- [ ] Persistent command history across sessions

**Implementation Plan:**
- File: src/macro/mod.rs (new file)
  - [ ] Create macro module
  - [ ] Define Macro struct
  - [ ] Implement record_action()
  - [ ] Implement replay_macro()
  - [ ] Store macros in HashMap<char, Vec<Action>>

- File: src/input/handler.rs
  - [ ] Add q handler for recording
  - [ ] Add @ handler for replay

**Tests:**
- [ ] test_record_macro
- [ ] test_replay_macro
- [ ] test_replay_last_macro
- [ ] test_multiple_registers
- [ ] test_command_history

---

### v0.18.1 - Final Architecture Polish [ ]

**Focus:** Pre-release code quality and architecture finalization  
**Status:** [ ]  
**Primary Focus:** Final architecture polish and 1.0 readiness

**Philosophy:**
This is the final refactoring pass before v1.0.0. Focus on polishing the entire codebase to ensure it's maintainable, well-documented, and ready for stable release. Address any remaining architectural concerns and ensure all modules are production-ready.

**Audit Phase:**
Start with comprehensive audit of entire codebase:
- Review all module boundaries and public APIs
- Identify any remaining technical debt
- Check consistency across all features
- Verify documentation completeness
- Ensure performance targets are met
- Review test coverage gaps

**Success Criteria:**
- Zero clippy warnings
- Code coverage > 80%
- All functions < 50 lines
- Performance benchmarks met
- Module structure documented
- Cyclomatic complexity reduced
- All tests pass with no panics
- All public APIs have rustdoc
- Architecture docs complete
- Ready for 1.0 release

**Testing Strategy:**
- Full regression test suite
- Integration tests for all features
- Performance benchmark validation
- Edge case verification
- User workflow testing
- Documentation accuracy checks

**Documentation Requirements:**
- Complete rustdoc for all public APIs
- Architecture documentation finalized
- User guide comprehensive
- Developer documentation complete
- Migration guide if needed
- Performance characteristics documented

---

### v1.0.0 - Stable Release & Polish [ ]

**Focus:** All core features working, stable command interface, comprehensive documentation  
**Status:** [ ]

**Pre-Release Checklist:**

**Feature Verification:**
- [ ] All navigation features work (hjkl, gg, G, 5g, :cB, w/b/e, zt/zz/zb)
- [ ] All editing features work (Insert mode, Magnifier mode)
- [ ] All column operations work (,dd, ,yy, ,p, ,o, ,O)
- [ ] All visual mode features work (v, V, ,v)
- [ ] Search works (/pattern, n, N, *, :noh)
- [ ] SQL query mode works (:q SELECT..., :sql, :sort)
- [ ] Undo/redo works (u, Ctrl+r, .)
- [ ] Cell transforms work (~, gU, gu, g~, g.)
- [ ] Row movement works (gj, gk)
- [ ] System clipboard works ("+yy, "+,yy, "+p)
- [ ] Find/replace works (:%s/old/new/g)
- [ ] Filtering works (:filter, :filter!)
- [ ] Column operations work (:width, :freeze, :type)
- [ ] Statistics work (:stats, :sum, :avg)
- [ ] Export/import works (JSON, Markdown, TSV, HTML)
- [ ] Configuration system works (config.toml)
- [ ] Macros work (qa, @a, @@)
- [ ] Multi-file workflow works ([, ], :files)
- [ ] Save/quit protection works (:w, :Wq, :q, :q!)
- [ ] Header toggle system works (:ht)
- [ ] Triple clipboard works (row, column, region buffers)
- [ ] Range operations work (:5,10d, :B,Dd)

**Code Quality:**
- [ ] All tests pass (target: 700+ tests)
- [ ] Zero clippy warnings
- [ ] Zero compiler warnings
- [ ] All public APIs documented
- [ ] Code coverage > 80%

**Documentation:**
- [ ] README.md complete and accurate
- [ ] CHANGELOG.md up to date
- [ ] docs/keybindings.md comprehensive
- [ ] docs/architecture.md accurate
- [ ] docs/development.md for contributors
- [ ] Example CSV files in test_data/

**Performance:**
- [ ] 100K+ rows at 60 FPS
- [ ] Instant file switching
- [ ] No memory leaks (valgrind clean)
- [ ] Startup time < 100ms for 10MB files

**UX Polish:**
- [ ] All error messages clear and actionable
- [ ] Help system (? and :help) comprehensive
- [ ] No confusing edge cases
- [ ] Vim users feel at home immediately

**Release Prep:**
- [ ] Version bumped to 1.0.0
- [ ] Git tag created
- [ ] Crates.io metadata accurate
- [ ] LICENSE file up to date
- [ ] GitHub release notes written

---

**End of Roadmap**
