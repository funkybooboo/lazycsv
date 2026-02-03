# LazyCSV Design Document

Complete UI/UX design specification for LazyCSV.

This document translates the functionality described in the [Features Guide](features.md) into a concrete user interface and experience. The implementation of this design is guided by the project's [Architecture](architecture.md).

## Design Principles

### 1. Vim-First Philosophy
- Every action must be keyboard accessible
- Prefer vim motions (hjkl) over arrows
- Support vim-style operators: `dd`, `yy`, `p`, etc.
- Modal editing: Normal/Edit/Visual/Command modes

### 2. Lazy Tools Aesthetic
- Clean, organized panels
- Persistent context hints
- Context-aware help
- Smooth transitions and feedback
- No clutter, maximum information density

### 3. Information Hierarchy
- **Most important**: Current cell/row
- **Secondary**: Column headers and context
- **Tertiary**: Status information
- **Always visible**: Current mode and basic help

## Screen Layouts

### Default View (v0.1.0)

```
┌─ lazycsv: sales_data.csv ────────────────────────────┐
│     │  A          │ ►B         │  C           │  D    │ ← Column letters (► shows selected)
├─────┼─────────────┼────────────┼──────────────┼───────┤
│  #  │  ID         │  Date      │  Product     │  Qty  │ ← Headers
├─────┼─────────────┼────────────┼──────────────┼───────┤
│  1  │  001        │ 2024-01-15 │ Widget A     │  100  │
│  2  │  002        │ 2024-01-16 │ Gadget B     │   50  │
│►3   │  003        │[2024-01-17]│ Doohickey C  │   75  │ ← Current cell
│  4  │  004        │ 2024-01-18 │ Thingamajig  │  200  │
│  5  │  005        │ 2024-01-19 │ Whatchama... │  125  │
│ ... │                                                  │
├──────────────────────────────────────────────────────┤
│ ? help │ q quit │ [ ] files │                        │ ← Status (left: controls,
│ Row 3/1,234 │ Col B: Date (2/4) │                    │           right: info)
│ Cell: "2024-01-17"                                   │
├──────────────────────────────────────────────────────┤
│ Files (1/2): ► sales_data.csv | customers.csv       │ ← File switcher
└──────────────────────────────────────────────────────┘
```

**Legend:**
- `►` - Row indicator (left gutter) AND column indicator (top row)
- `[text]` - Current cell (reversed video / highlighted)
- `...` - Truncated text (cell width limit ~20 chars)
- Column letters: A, B, C, D... (like Excel)
- Row numbers: 1, 2, 3... (not 0-indexed for user)
- Status bar shows current cell value on bottom line

### With Help Overlay (Press ?)

```
┌─ lazycsv: sales_data.csv ────────────────────────────┐
│     │  A     │  B     ╔═══ Help ════════════════╗    │
├─────┼────────┼────────║ Navigation   │ Editing  ║────┤
│  #  │  ID    │  Date  ║  hjkl/arrows │ i Edit   ║    │
├─────┼────────┼────────║  gg/G First/ │ Esc Cncl ║────┤
│  3  │  003   │  2024..║  0/$ First/l │          ║    │
│  3  │  003   │  2024..║  PageUp/Down │ Rows/Col ║    │
│  4  │  004   │  2024..║              │ o Add ro ║    │
│ ... │                 ║ File Switch  │ O Add ro ║    │
├─────────────────────  ║  [/] Switch  │ dd Delet ║────┤
│ Row 2/1,234 │ Col 2/5║              │          ║    │
├─────────────────────  ║ Other        │          ║────┤
│ Files (1/2): ► sales.║  ?   Help    │          ║    │
└─────────────────────  ║  q   Quit    │          ║    │
                        ║  Esc to close           ║
                        ╚═════════════════════════╝
```

**Help Overlay:**
- Centered (60% width, 70% height)
- Two-column layout (Navigation | Editing)
- Organized by function
- Dimmed background (table still visible)
- Close with `?` or `Esc`

### Edit Mode (v0.4.0)

```
┌─ lazycsv: sales_data.csv [EDIT] * ────────────────────┐
│     │  A          │  B         │  C           │  D    │
├─────┼─────────────┼────────────┼──────────────┼───────┤
│  #  │  ID         │  Date      │  Product     │  Qty  │
├─────┼─────────────┼────────────┼──────────────┼───────┤
│  1  │  001        │ 2024-01-15 │ Widget A     │  100  │
│  2  │  002        │ 2024-01-16 │ Gadget B     │   50  │
│►3   │  003        │ 2024-01-17 │►Doohickey C█ │   75  │ ← Editing
│  4  │  004        │ 2024-01-18 │ Thingamajig  │  200  │
│  5  │  005        │ 2024-01-19 │ Whatchama... │  125  │
├──────────────────────────────────────────────────────┤
│ EDIT: "Doohickey C" │ Enter save │ Esc cancel       │
├──────────────────────────────────────────────────────┤
│ Files (1/2): ► sales_data.csv | customers.csv       │
└──────────────────────────────────────────────────────┘
```

**Edit Mode Indicators:**
- `[EDIT]` mode in title bar
- `*` dirty indicator (unsaved changes)
- `►` edit cursor before text
- `█` blinking cursor at end
- Status bar shows edit instructions
- All text selected by default (type to replace)

### Visual Selection Mode (v1.1.0)

```
┌─ lazycsv: sales_data.csv [VISUAL] ─ 3 rows selected ─┐
│     │  A          │  B         │  C           │  D    │
├─────┼─────────────┼────────────┼──────────────┼───────┤
│  #  │  ID         │  Date      │  Product     │  Qty  │
├─────┼─────────────┼────────────┼──────────────┼───────┤
│  1  │  001        │ 2024-01-15 │ Widget A     │  100  │
│══2  │  002        │ 2024-01-16 │ Gadget B     │   50  │ ← Start
│══3  │  003        │ 2024-01-17 │ Doohickey C  │   75  │ ← Selected
│►4   │  004        │ 2024-01-18 │ Thingamajig  │  200  │ ← Current
│  5  │  005        │ 2024-01-19 │ Whatchama... │  125  │
├──────────────────────────────────────────────────────┤
│ VISUAL: 3 rows │ d delete │ y copy │ Esc cancel    │
├──────────────────────────────────────────────────────┤
│ Files (1/2): ► sales_data.csv | customers.csv       │
└──────────────────────────────────────────────────────┘
```

**Visual Mode Indicators:**
- `[VISUAL]` mode in title bar
- `══` selection markers on selected rows
- `►` current cursor position
- Row count in status: "3 rows selected"
- Available operations in status

### Fuzzy Search Overlay (v1.1.0)

```
┌─ lazycsv: sales_data.csv ────────────────────────────┐
│     │  A     │  B     ╔═══ Search ══════════════╗    │
├─────┼────────┼────────║ Query: eml█             ║────┤
│  #  │  ID    │  Date  ║                         ║    │
├─────┼────────┼────────║ Results (3):            ║────┤
│  1  │  001   │  2024..║ ► [Col C: Email]        ║    │
│►2   │  002   │  2024..║   [Cell B5: emily@...]  ║    │
│  3  │  003   │  2024..║   [Cell B7: emma@...]   ║    │
│  4  │  004   │  2024..║                         ║    │
│ ... │                 ║ j/k navigate            ║    │
├─────────────────────  ║ Enter jump              ║────┤
│ Row 2/1,234 │ Col 2/5║ Esc cancel              ║    │
├─────────────────────  ╚═════════════════════════╝────┤
│ Files (1/2): ► sales_data.csv | customers.csv       │
└──────────────────────────────────────────────────────┘
```

**Fuzzy Search:**
- Centered overlay (50% width)
- Live results as you type
- Shows match type: [Row], [Col], [Cell]
- `j`/`k` to navigate results
- `Enter` to jump, `Esc` to cancel
- Fuzzy matching with scoring

## Visual Design Specification

### Typography
- **Monospace font** - Terminal default
- **Header row** - Bold
- **Current cell** - Reversed (black bg, white fg or vice versa)
- **Column letters** - Dimmed

### Layout Constraints
- **Minimum width**: 40 columns (warning if smaller)
- **Minimum height**: 10 rows
- **Column width**: Dynamic, ~20 chars max
- **Visible columns**: ~10 at a time
- **Row numbers width**: 5 chars
- **Status bar height**: 3 rows
- **File switcher height**: 3 rows

### Color Scheme (Monochrome)

**Current Design Decision: No colors for now**

Using only terminal defaults and text attributes:
- **Normal text**: Default terminal colors
- **Bold**: Headers, mode indicators
- **Dim**: Column letters, hints
- **Reversed**: Current cell highlight
- **Underline**: Not used (may use for links later)

**Rationale:**
- Works on all terminals
- Professional, clean look
- Less visual noise
- Faster rendering
- Accessibility (no color-blind issues)

**Future: Optional Color Theme (v1.4.0)**

If colors are added as an option:
- **Headers**: Cyan + Bold
- **Current row**: Dark gray background
- **Current cell**: Blue background + Bold
- **Edit mode**: Yellow background + Black foreground
- **Visual selection**: Dark blue background
- **Status bar**: Blue background + White foreground
- **Success message**: Green
- **Error message**: Red
- **Warning**: Orange

### Spacing & Borders
- **Table borders**: Single-line box drawing characters (─│┌┐└┘├┤┬┴┼)
- **Cell padding**: 1 space on each side
- **Header separator**: Double line or thicker border
- **Panel separation**: Clear borders between sections

### Text Truncation
- **Max cell width**: 20 characters (configurable later)
- **Truncation indicator**: `...` at end
- **Full text available**: In edit mode or on hover (future)
- **Column names**: Never truncated in column letters row

## Interaction Patterns

### Cell Selection
```
1. Start: No cell selected (first run)
2. Default: Select row 1, column A
3. Navigate: Move with hjkl or arrows
4. Feedback: Cell immediately highlighted
5. Auto-scroll: Keep cell in view
```

### Horizontal Scrolling
```
1. Default view: Show columns 0-9 (A-J)
2. Move right past column J
3. Scroll: Shift view to show columns 1-10 (B-K)
4. Move left to column A
5. Scroll: Shift view back to show columns 0-9 (A-J)
```

### File Switching
```
1. Start: Load file from CLI (e.g., sales.csv)
2. Scan: Find other CSVs in same directory
3. Display: Show all files in bottom panel
4. Press ]: Switch to next file (customers.csv)
5. Load: Brief loading indicator
6. Display: New file with cursor at row 1, col A
7. Message: "Loaded: customers.csv" in status bar
```

### Edit Mode Flow (v0.4.0)
```
1. Navigate to cell
2. Press i or Enter
3. Mode changes to [EDIT]
4. All text selected (ready to replace)
5. Type: Replace entire value
   OR Press End: Move to end to append
6. Edit value
7. Press Enter: Save and return to Normal mode
   OR Press Esc: Cancel and restore original
8. Show feedback: Cell updated, * in title
```

### Row Operations (v0.7.0)
```
Add row below (o):
  1. Navigate to row
  2. Press o
  3. New blank row inserted below
  4. Cursor moves to new row, column A
  5. Message: "Row added below"
  6. * dirty indicator appears

Delete row (dd):
  1. Navigate to row
  2. Press dd (double-tap)
  3. Row immediately deleted
  4. Cursor moves to next row (or previous if last)
  5. Message: "Row deleted"
  6. * dirty indicator appears
```

### Copy/Paste (v0.7.0)
```
1. Navigate to row
2. Press yy (yank)
3. Message: "Row copied"
4. Navigate to target location
5. Press p (paste)
6. Row inserted below current
7. Message: "Row pasted"
8. Can paste multiple times
```

### Undo (v1.0.0)
```
1. Perform operation (edit, delete, sort, etc.)
2. Press u
3. Operation reversed
4. Message: "Undo: Edit cell A5" (or whatever was undone)
5. Can undo multiple times (up to 100 operations)
6. Press Ctrl+r to redo
```

## Responsive Design

### Small Terminal (< 80 cols)
- Reduce max visible columns (show 5-6 instead of 10)
- Truncate cell text more aggressively (15 chars instead of 20)
- Abbreviate status messages
- Help overlay becomes full-screen

### Very Small Terminal (< 40 cols)
- Show single column at a time
- Minimal status: "R3/100 C2/5"
- Warning: "Terminal too small"
- Recommend minimum 40 columns

### Large Terminal (> 120 cols)
- Show more columns (15-20 instead of 10)
- Less aggressive truncation (30 chars instead of 20)
- More detailed status messages
- Wider help panel

## Animation & Timing

**Design Decision: Minimal animation**

LazyCSV prioritizes instant response over smooth animation:

- **Cell selection**: Instant (0ms)
- **Mode changes**: Instant (0ms)
- **Cursor movement**: Instant (0ms)
- **Scrolling**: Immediate, no smooth scroll
- **Text editing**: Instant character appearance

**Rationale:**
- Faster feels more responsive
- Reduces input lag
- Simpler implementation
- Works on slow terminals

**Only animations:**
- Cursor blinking (terminal default)
- Status messages fade after 2 seconds

## Accessibility

### Visual
- High contrast (reversed video for selection)
- No color-only information (uses symbols too: ►, ══, *)
- Clear focus indicators (► and reversed cell)
- Readable font sizes (terminal default)

### Motor
- All features keyboard accessible
- No double-key-press timing requirements
- No mouse required
- Undo available for all destructive operations

### Cognitive
- Persistent help available (press ?)
- Context-aware status messages
- Clear mode indicators (always visible)
- Consistent navigation (vim-style)
- Discoverable (hints in status bar)

## Error States

### File Not Found
```
┌──────────────────────────────────────────┐
│ lazycsv: ERROR                           │
├──────────────────────────────────────────┤
│                                          │
│          ⚠ Failed to load file          │
│                                          │
│          File: sales_data.csv            │
│          Error: File not found           │
│                                          │
│          Press q to quit                 │
│                                          │
├──────────────────────────────────────────┤
│ q quit                                   │
└──────────────────────────────────────────┘
```

### Empty CSV
```
┌──────────────────────────────────────────┐
│ lazycsv: empty.csv                       │
├──────────────────────────────────────────┤
│     │  A      │  B      │  C      │      │
├─────┼─────────┼─────────┼─────────┼──────┤
│  #  │  Name   │  Email  │  Age    │      │
├─────┼─────────┼─────────┼─────────┼──────┤
│                                          │
│         (No data rows)                   │
│                                          │
├──────────────────────────────────────────┤
│ 0 rows │ 3 columns │ ? help │ q quit    │
└──────────────────────────────────────────┘
```

### Unsaved Changes Warning (v0.6.0)
```
┌──────────────────────────────────────────┐
│          ⚠ Unsaved Changes               │
│                                          │
│  You have unsaved changes.               │
│  Press :q! to quit without saving        │
│                                          │
│  Or press Ctrl+S to save first           │
│                                          │
└──────────────────────────────────────────┘
```

## Status Messages

### Format
```
<file info> │ <position> │ <context> │ <hints>
```

### Examples
```
Normal mode:
  sales_data.csv │ Row 5/100 │ Col 2/5 Email │ ? help │ q quit

Multiple files:
  sales_data.csv │ Row 5/100 │ Col 2/5 Email │ [/]] files │ ? help

Edit mode:
  EDIT: "Widget A" │ Enter save │ Esc cancel

After save (v0.6.0):
  ✓ Saved successfully │ Row 10/100 │ Col 2/5

After operation:
  Row added below │ Row 11/101 │ Col 1/5

Error:
  Error: Permission denied │ Press q to quit
```

## Design Philosophy Summary

LazyCSV follows these core UX principles:

1. **Keyboard Efficiency**: Every action is 1-2 keystrokes maximum
2. **Visual Clarity**: Always know where you are (mode, position, state)
3. **Immediate Feedback**: Every action gets instant visual confirmation
4. **Forgiving**: Undo available, clear warnings
5. **Discoverable**: Help always available (?), hints in status bar
6. **Consistent**: Same patterns across modes and operations
7. **Beautiful**: Clean monochrome design, professional look
8. **Fast**: 60 FPS rendering, instant response to input
9. **Simple**: No clutter, maximum information density

## Inspiration

LazyCSV's design draws from:

- **lazygit**: Clean panels, persistent context, keyboard-first
- **lazydocker**: Organized layout, clear mode indicators
- **vim**: Modal editing, mnemonic keys, powerful with minimal UI
- **Excel**: Familiar spreadsheet layout, row/column numbering
- **Spreadsheet simplicity**: Just data, no formulas visible (yet)

## Design Decisions Log

| Decision | Rationale | Date |
|----------|-----------|------|
| No colors | Cleaner, works everywhere, faster | 2026-01 |
| Monospace only | Terminal standard, clear alignment | 2026-01 |
| Row/column numbers | Familiar (Excel-like), easy reference | 2026-01 |
| File switcher at bottom | Always visible, doesn't block data | 2026-01 |
| Select-all in edit | Most edits are replacements | 2026-01 |
| No delete confirmation | Undo provides safety, faster workflow | 2026-01 |
| Truncate at 20 chars | Readable, fits ~10 columns on screen | 2026-01 |
| Help as overlay | Doesn't lose context, easy to reference | 2026-01 |
| Multi-file navigation | Innovation! Consistent with Excel sheets | 2026-01 |

## Future Design Considerations

### v1.4.0+
- Optional color themes (configurable)
- Mouse support (optional, keyboard still primary)
- Column width adjustment (drag or command)
- Row/column freeze (like Excel)
- Split view (compare two files side-by-side)
- Mini-map (overview of entire file)
- Cell formatting preview (dates, numbers)
- Formula bar (like Excel)
