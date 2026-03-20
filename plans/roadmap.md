# LazyCSV Development Roadmap

A versioned checklist for building the LazyCSV TUI. Each version represents a deliverable milestone.

## Version Overview

| Version | Focus | Status | Tests |
|---------|-------|--------|-------|
| v0.1.0 | Foundation & Core Viewing | [x] | 257 |
| v0.1.1 | Post-Foundation Refactor | [x] | 450 |
| v0.2.0 | Type Safety & Architecture | [x] | 257 |
| v0.2.1 | Type System Cleanup | [x] | 519 |
| v0.3.0 | Advanced Navigation & UI Polish | [x] | 344 |
| v0.3.1 | Navigation Code Quality | [x] | 514 |
| v0.4.0 | Cell Editing & Persistence | [x] | 517 |
| v0.4.1 | Editing System Refactor | [x] | 527 |
| v0.5.0 | Column Operations & Visual Mode | [x] | 515+ |
| v0.5.1 | Column Operations Cleanup | [x] | 967 |
| v0.6.0 | Magnifier Mode (Full Vim Editor) | [x] | 415 |
| v0.6.1 | Magnifier Performance & Quality | [x] | 1,003 |
| v0.7.0 | Search & Filtering | [x] | 27 |
| v0.7.1 | Search System Optimization | [x] | 27 |
| v0.8.0 | SQL Query Mode & Data Operations | [x] | 30 |
| v0.8.1 | SQL & Data Operations Polish | [x] | 555 |
| v0.9.0 | Configuration System | [ ] | TBD |
| v0.9.1 | Configuration Testing & Polish | [ ] | TBD |
| v0.10.0 | Undo/Redo & Command History | [ ] | TBD |
| v0.10.1 | Undo System Testing & Reliability | [ ] | TBD |
| v0.11.0 | SQL Editor Vim Editing | [x] | 700+ |
| v0.11.1 | SQL Editor Refactoring & Quality | [x] | 1,251 |
| v0.12.0 | UI Consistency & Standardization | [x] | 1,403 |
| v0.12.1 | UI System Testing | [ ] | TBD |
| v0.13.0 | Repository Organization & Structure | [ ] | TBD |
| v0.13.1 | Module Organization & Cleanup | [ ] | TBD |
| v0.14.0 | Cell Transforms & Data Cleanup | [ ] | TBD |
| v0.14.1 | Performance Optimization & Profiling | [ ] | TBD |
| v0.15.0 | System Clipboard & External Integration | [ ] | TBD |
| v0.15.1 | Testing & Reliability Improvements | [ ] | TBD |
| v0.16.0 | Bulk Operations & Find/Replace | [ ] | TBD |
| v0.16.1 | Error Handling & Robustness | [ ] | TBD |
| v0.17.0 | Advanced Filtering & Conditional Views | [ ] | TBD |
| v0.17.1 | Module Organization & Cleanup | [ ] | TBD |
| v0.18.0 | SQL IntelliSense & Auto-completion | [x] | TBD |
| v0.18.1 | SQL IntelliSense Polish & Testing | [x] | 606 |
| v0.19.0 | Column Resize & Advanced Column Operations | [ ] | TBD |
| v0.19.1 | Documentation & Maintainability | [ ] | TBD |
| v0.20.0 | Data Analysis, Statistics & Cell Formulas | [~] | 88 |
| v0.20.1 | Technical Debt Reduction | [ ] | TBD |
| v0.21.0 | Export & Import (JSON, Markdown, TSV) | [ ] | TBD |
| v0.21.1 | Code Coverage & Test Quality | [ ] | TBD |
| v0.22.0 | Macros & Command Recording | [ ] | TBD |
| v0.22.1 | Performance Benchmarking & Tuning | [ ] | TBD |
| v0.23.0 | Final Architecture Review | [ ] | TBD |
| v0.23.1 | Final Architecture Polish | [ ] | TBD |
| v0.24.0 | Performance, CLI Pipeline & SQL Type Intelligence | [x] | TBD |
| v0.25.0 | Spreadsheet Support & CLI Tools | [x] | 13 |
| v1.0.0 | Stable Release & Polish | [ ] | - |

**Total Tests Passing:** 1,403 tests (all integration and unit tests across vim_editor, SQL, magnifier, UI, keybindings, and core modules)

---

## Version Summaries

###  Completed Versions

**[v0.1.0](versions/v0.1.0.md) - Foundation & Core Viewing**  
Basic CSV viewing with vim navigation, multi-file support, and zero-config design. Established in-memory architecture for maximum performance.  

**[v0.1.1](versions/v0.1.1.md) - Post-Foundation Refactor**  
Major refactor consolidating AppState, improving error handling, and establishing clean architecture patterns with 450 passing tests.  

**[v0.2.0](versions/v0.2.0.md) - Type Safety & Architecture**  
Enhanced type safety with CellAddress and comprehensive test framework covering all core interactions.  

**[v0.2.1](versions/v0.2.1.md) - Type System Cleanup**  
Extensive refactoring removing redundant types, consolidating patterns, and improving code quality with 519 tests.  

**[v0.3.0](versions/v0.3.0.md) - Advanced Navigation & UI Polish**  
Word-based navigation, scrolling commands, and header toggle system with refined UI polish.  

**[v0.3.1](versions/v0.3.1.md) - Navigation Code Quality**  
Navigation system cleanup with consistent behavior and comprehensive test coverage (514 tests).  

**[v0.4.0](versions/v0.4.0.md) - Cell Editing & Persistence**  
Insert mode, Magnifier mode, and file persistence with dirty tracking and autosave.  

**[v0.4.1](versions/v0.4.1.md) - Editing System Refactor**  
Edit system cleanup with improved mode transitions and error handling (527 tests).  

**[v0.5.0](versions/v0.5.0.md) - Column Operations & Visual Mode**  
Visual mode (block, line, column) and column operations with comma-prefixed commands.  

**[v0.5.1](versions/v0.5.1.md) - Column Operations Cleanup**  
Column operations refactor with comprehensive test coverage achieving 967 passing tests.  

**[v0.6.0](versions/v0.6.0.md) - Magnifier Mode (Full Vim Editor)**  
Full vim editor for cells with multi-line support and embedded helix integration.  

**[v0.6.1](versions/v0.6.1.md) - Magnifier Performance & Quality**  
Magnifier system optimization with improved rendering and reliability (1,003 tests).  

**[v0.7.0](versions/v0.7.0.md) - Search & Filtering**  
Regex search with highlighting, row filtering, and column-specific search capabilities.  

**[v0.7.1](versions/v0.7.1.md) - Search System Optimization**  
Search system cleanup with improved performance and test coverage.  

**[v0.8.0](versions/v0.8.0.md) - SQL Query Mode & Data Operations**  
SQL query execution using DuckDB with rich result views and data transformation capabilities.  

**[v0.8.1](versions/v0.8.1.md)** - SQL & Data Operations Polish**  
SQL system refinement with improved error handling and comprehensive testing (555 tests total).  

**[v0.11.0](versions/v0.11.0.md) - SQL Editor Vim Editing**  
Full vim modal editing in SQL editor with reusable vim_editor module (700+ tests total).  

**[v0.11.1](versions/v0.11.1.md) - SQL Editor Refactoring & Quality**  
Refactored vim_editor with command pattern, removed code duplication, reorganized normal_mode into 11 modules, split UI modules, added unit tests, zero clippy warnings (1,251 tests total).  

**[v0.12.0](versions/v0.12.0.md) - UI Consistency & Standardization** ✅ COMPLETE
Yazi-inspired 3-column file explorer (30%:40%:30%) with parent directory preview, current directory navigation, and file/CSV preview. Bug fixes for navigation scroll, ASCII-only rendering (no emojis), and consistent error messages. All 524 tests passing.

**[v0.24.0](versions/v0.24.0.md) - Performance, CLI Pipeline & SQL Type Intelligence**
Lazy loading with memory-mapped files for large CSVs, CLI sort mode, piped stdin support for all non-interactive modes, and automatic column type detection for correct numeric/date ordering in SQL queries. Buffered I/O indexing (7x faster on macOS), parallel sort with rayon, fast memchr row counting, and raw-byte CSV write for sorted lazy files.

**[v0.25.0](versions/v0.25.0.md) - Spreadsheet Support, CLI Tools & SQL DML**
Native spreadsheet file support (xlsx/xls/ods) via calamine. Formula preservation in formula bar. CLI extraction (`-x`), clipboard copy (`-C`), clipboard paste (`-P` with auto delimiter detection), file splitting (`-S`), and SQL queries (`-q`) all work across CSV and spreadsheet formats. SQL DML (INSERT/UPDATE/DELETE/ALTER) with DML-aware IntelliSense and templates. TUI `:copy`/`:paste` commands with auto delimiter detection. SQL formatting via Ctrl+F. Streaming I/O for large files. Zebra striping. Terminal robustness fixes.

###  Planned Versions

**[v0.9.0](versions/v0.9.0.md) - Configuration System**  
Implement `~/.config/lazycsv/config.toml` with theme customization, keybinding remapping, and default behaviors.  

**[v0.9.1](versions/v0.9.1.md) - Configuration Testing & Polish**  
Configuration system cleanup with validation, error handling, and comprehensive test coverage.  

**[v0.10.0](versions/v0.10.0.md) - Undo/Redo & Command History**  
Undo/redo system with persistent command history and session restoration.  

**[v0.10.1](versions/v0.10.1.md) - Undo System Testing & Reliability**  
Undo system refinement with edge case handling and comprehensive test coverage.  

**[v0.12.1](versions/v0.12.1.md) - UI System Testing**  
UI system testing with visual regression tests and accessibility improvements.  

**[v0.13.0](versions/v0.13.0.md) - Repository Organization & Structure**  
Reorganize codebase with clear module boundaries and comprehensive documentation.  

**[v0.13.1](versions/v0.13.1.md) - Module Organization & Cleanup**  
Module system cleanup with clear dependencies and improved maintainability.  

**[v0.14.0](versions/v0.14.0.md) - Cell Transforms & Data Cleanup**  
Cell transformation commands (trim, uppercase, lowercase, normalize) with preview.  

**[v0.14.1](versions/v0.14.1.md) - Performance Optimization & Profiling**  
Performance profiling and optimization with benchmarking suite.  

**[v0.15.0](versions/v0.15.0.md) - System Clipboard & External Integration**  
System clipboard integration with TSV/CSV paste support and external tool integration.  

**[v0.15.1](versions/v0.15.1.md) - Testing & Reliability Improvements**  
Testing infrastructure improvements with flaky test fixes and CI enhancements.  

**[v0.16.0](versions/v0.16.0.md) - Bulk Operations & Find/Replace**  
Find/replace across cells with regex support and bulk transformation commands.  

**[v0.16.1](versions/v0.16.1.md) - Error Handling & Robustness**  
Comprehensive error handling with graceful degradation and user-friendly messages.  

**[v0.17.0](versions/v0.17.0.md) - Advanced Filtering & Conditional Views**  
Advanced filtering with multiple conditions and saved filter presets.  

**[v0.17.1](versions/v0.17.1.md) - Module Organization & Cleanup**  
Filter system cleanup with clear abstractions and comprehensive testing.  

**[v0.18.0](versions/v0.18.0.md) - SQL IntelliSense & Auto-completion**
SQL IntelliSense with table/column auto-completion and syntax validation.

**[v0.18.1](versions/v0.18.1.md) - SQL IntelliSense Polish & Testing** ✅ COMPLETE
Schema caching, fuzzy matching, Unicode support, auto-quoting for identifiers with spaces, comprehensive error prevention with inline diagnostics, query templates, and 50 integration tests. Fixed multiple Unicode/multi-byte panics across VimEditor and SQL systems.  

**[v0.19.0](versions/v0.19.0.md) - Column Resize & Advanced Column Operations**  
Interactive column resizing with visual feedback and advanced column manipulation.  

**[v0.19.1](versions/v0.19.1.md) - Documentation & Maintainability**  
Documentation improvements with examples, troubleshooting guides, and API docs.  

**[v0.20.0](versions/v0.20.0.md) - Data Analysis, Statistics & Cell Formulas** (IN PROGRESS)
Column statistics commands (:stats, :sum, :avg, :count, :distinct), Excel-like cell formulas (=SUM, =AVERAGE, =IF, =VLOOKUP, and 20 more), formula bar display, auto re-evaluation, and formula completion popup.

**[v0.20.1](versions/v0.20.1.md) - Technical Debt Reduction**  
Technical debt cleanup with code simplification and pattern consolidation.  

**[v0.21.0](versions/v0.21.0.md) - Export & Import (JSON, Markdown, TSV)**  
Multi-format export/import with format detection and conversion.  

**[v0.21.1](versions/v0.21.1.md) - Code Coverage & Test Quality**  
Improve code coverage to 90%+ with comprehensive test suite.  

**[v0.22.0](versions/v0.22.0.md) - Macros & Command Recording**  
Macro recording and playback with saved macro library.  

**[v0.22.1](versions/v0.22.1.md) - Performance Benchmarking & Tuning**  
Performance benchmarking suite with optimization targets.  

**[v0.23.0](versions/v0.23.0.md) - Final Architecture Review**  
Comprehensive architecture review with refactoring for long-term maintainability.  

**[v0.23.1](versions/v0.23.1.md) - Final Architecture Polish**  
Final architecture cleanup with documentation and code quality improvements.  

**[v1.0.0](versions/v1.0.0.md) - Stable Release & Polish**  
Production-ready release with comprehensive documentation and polish.  

---

## Guiding Principles

- **Vim-First Philosophy:** Navigation and commands should feel native to vim users. Composable commands (operator + motion). No timeouts on pending commands. Clean status line.
- **Truly Hybrid:** Balance vim power with spreadsheet familiarity. Support both vim keys (hjkl) and arrow keys, vim commands and spreadsheet-like operations.
- **Three-Tier Operator System:** Cell (`x`) → Row (`dd`) → Column (`,dd`). Comma as leader for CSV-specific column operations.
- **Simplified Navigation:** Use `g` suffix for jumps: `5g` (row 5), `:cB` (column B). Reserve `:` for operations and explicit navigation.
- **Row Numbering:** All rows are numbered starting from 1 (displayed). Internally uses 0-based indexing. Row 1 typically contains column headers for SQL queries, but receives no special UI treatment.
- **Command Ranges:** Standardized ranges: `:5,10d` for rows, `:B,Dd` for columns. Don't overcomplicate.
- **Triple Clipboard:** Three independent buffers: row buffer (yy/p), column buffer (,yy/,p), region buffer (visual y/p). No cross-pasting.
- **Ephemeral Edits:** No changes saved to file until explicit `:w`. All edits update in-memory representation first.
- **Minimal UI Chrome:** No heavy borders. Use subtle separators. Maximum content, minimum decoration. Status line shows mode + row + column only.
- **In-Memory Only:** Small CSV files loaded entirely into RAM; large files use lazy mmap-backed storage.
- **CSV-Centric:** CSV is the primary format. Spreadsheet files (.xlsx/.xls/.ods) are supported for reading and converted to CSV on save.
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

## Related Documentation

- [Architecture Documentation](../docs/architecture.md)
- [Feature Documentation](../docs/features.md)
- [Version Details](versions/v0.24.0.md) - Detailed documentation for each version
