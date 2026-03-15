# LazyCSV UI Guidelines

**Version:** 0.12.0  
**Last Updated:** 2026-03-14

## Philosophy

LazyCSV's UI follows these core principles:

1. **Single Source of Truth** - All styling comes from `src/ui/modal.rs`
2. **Consistency** - Same look and feel across all modes and panels
3. **Vim-Inspired** - Minimal, keyboard-driven, efficient
4. **ASCII-Only** - No emojis or special Unicode characters (except in data)
5. **Accessibility** - Clear visual hierarchy, readable colors

## Color Palette

All colors are defined in `src/ui/modal.rs` as constants. **Never hardcode colors** in UI code.

### Primary Colors

| Constant | Value | Usage |
|----------|-------|-------|
| `COLOR_POPUP_BG` | `DarkGray` | Completion menus, popup backgrounds |
| `COLOR_LINE_NUMBER` | `DarkGray` | Line numbers in editors |
| `COLOR_VISUAL_BG` | `DarkGray` | Visual selection background |
| `COLOR_VISUAL_FG` | `Yellow` | Visual selection foreground |
| `COLOR_ERROR` | `Red` | Error messages |
| `COLOR_SUCCESS` | `Green` | Success messages, confirmations |
| `COLOR_MODE_INDICATOR_BG` | `Green` | Mode indicator background |
| `COLOR_MODE_INDICATOR_FG` | `Black` | Mode indicator foreground |

### Color Usage Examples

```rust
// ✅ GOOD - Use constants
let style = Style::default().fg(modal::COLOR_ERROR);
let bg = modal::COLOR_POPUP_BG;

// ❌ BAD - Never hardcode
let style = Style::default().fg(Color::Red);
let bg = Color::DarkGray;
```

## Typography

LazyCSV uses three text styles, all defined in `modal.rs`:

### Style Functions

| Function | Modifiers | Usage |
|----------|-----------|-------|
| `bold_style()` | `BOLD` | Headers, row numbers, selected items |
| `dim_style()` | `DIM` | Preview content, inactive items, hints |
| `error_style()` | `RED + BOLD` | Error messages |
| `success_style()` | `GREEN + BOLD` | Success messages |

### Typography Examples

```rust
// ✅ GOOD - Use style helpers
let header = Span::styled("NAVIGATION", modal::bold_style());
let hint = Span::styled("(optional)", modal::dim_style());
let error = Span::styled("File not found", modal::error_style());

// ❌ BAD - Don't create inline styles
let header = Span::styled("NAVIGATION", Style::default().add_modifier(Modifier::BOLD));
```

## Layout Standards

### Modal Sizes

All modals use standardized sizes from `modal.rs`:

| Size | Width | Height | Usage |
|------|-------|--------|-------|
| Large | 80% | 80% | Help, SQL editor, Magnifier, File browser |
| Small | 40% | 20% | File operation prompts, confirmations |

```rust
// ✅ GOOD - Use size helpers
let area = modal::large_modal_rect(frame.area());
let prompt_area = modal::small_modal_rect(frame.area());

// ❌ BAD - Don't calculate manually
let area = centered_rect(80, 80, frame.area());
```

### Status Bars

All modals have a 1-line status bar at the bottom:

```rust
// Split layout for content + status bar
let (content_area, status_area) = modal::split_with_status_bar(inner);

// Build status line
let status = modal::build_status_line(
    " NORMAL",           // Left side (mode)
    "? for help",        // Right side (hints)
    area.width as usize  // Total width
);
```

### Border Styles

**All modals use `Borders::ALL`** - no exceptions.

```rust
// ✅ GOOD - Standard border
let block = modal::standard_block(" Title ");

// ❌ BAD - Custom borders
let block = Block::default().borders(Borders::TOP | Borders::BOTTOM);
```

## Component Patterns

### Cursor/Selection

Use `cursor_style()` for all cursor and selection highlighting:

```rust
// ✅ GOOD
let cursor = Span::styled(" ", modal::cursor_style());

// ❌ BAD
let cursor = Span::styled(" ", Style::default().bg(Color::White).fg(Color::Black));
```

**Cursor Style:**
- Background: White
- Foreground: Black
- Modifier: Bold

### Visual Selection

Use `visual_selection_style()` for visual mode selections:

```rust
// ✅ GOOD
let cell_style = modal::visual_selection_style();

// ❌ BAD
let cell_style = Style::default().bg(Color::DarkGray).fg(Color::Yellow);
```

### Mode Indicators

Use `mode_indicator_style()` for all mode displays:

```rust
// ✅ GOOD
let mode_style = modal::mode_indicator_style();
let mode_text = modal::format_mode_indicator("Normal", None); // " NORMAL"

// ❌ BAD
let mode_style = Style::default().fg(Color::Black).bg(Color::Green);
let mode_text = "-- NORMAL --";
```

**Mode Indicator Format:**
- Space-padded uppercase: ` NORMAL`, ` INSERT`, ` VISUAL`
- Command mode shows buffer: `:wq`, `:sort`
- Always use `format_mode_indicator()` helper

### Completion Menus

Use dedicated completion styles:

```rust
// ✅ GOOD
let selected_style = modal::completion_selected_style();
let unselected_style = modal::completion_unselected_style();

// ❌ BAD
let selected = Style::default().fg(Color::White).bg(Color::Blue);
let unselected = Style::default().fg(Color::White).bg(Color::DarkGray);
```

### Error Messages

All errors use `error_style()`:

```rust
// ✅ GOOD
let error_span = Span::styled("Error: File not found", modal::error_style());

// ❌ BAD
let error_span = Span::styled(
    "Error: File not found",
    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
);
```

**Error Placement:**
- Transient errors: Status bar
- Critical errors: Modal overlay
- Always use `error_style()` for color

## Style Module Usage

### Importing

```rust
use crate::ui::modal;

// Or for specific items
use crate::ui::modal::{cursor_style, bold_style, COLOR_ERROR};
```

### Available Style Functions

```rust
// Basic styles
modal::cursor_style()           // White bg, black fg, bold
modal::bold_style()             // Bold modifier
modal::dim_style()              // Dim modifier
modal::error_style()            // Red, bold
modal::success_style()          // Green, bold

// Specialized styles
modal::visual_selection_style() // DarkGray bg, yellow fg
modal::header_style()           // Bold (for table headers)
modal::row_number_style()       // Bold (for row numbers)
modal::mode_indicator_style()   // Black on green
modal::search_match_style()     // Yellow bg, black fg

// Completion menu
modal::completion_selected_style()   // White on blue
modal::completion_unselected_style() // White on dark gray
```

### Available Layout Functions

```rust
// Modal sizing
modal::large_modal_rect(area)   // 80% × 80%
modal::small_modal_rect(area)   // 40% × 20%
modal::centered_rect(w, h, area) // Custom size

// Layout helpers
modal::split_with_status_bar(area)  // Returns (content, status)
modal::standard_block(title)         // Block with borders + title

// Status bar builders
modal::build_status_line(left, right, width)
modal::build_three_part_status_line(left, center, right, width)
modal::format_mode_indicator(mode, cmd_buffer)
```

## Anti-Patterns

### ❌ Don't Do This

```rust
// Hardcoded colors
Style::default().fg(Color::Red)
Style::default().bg(Color::DarkGray)

// Hardcoded sizes
let width = (area.width * 80) / 100;
let height = (area.height * 80) / 100;

// Inline style creation
Style::default().add_modifier(Modifier::BOLD)
Style::default().bg(Color::White).fg(Color::Black)

// Custom mode formats
format!("-- {} --", mode)
format!("[{}]", mode)

// Custom borders
Block::default().borders(Borders::TOP | Borders::BOTTOM)
```

### ✅ Do This Instead

```rust
// Use color constants
Style::default().fg(modal::COLOR_ERROR)
Style::default().bg(modal::COLOR_POPUP_BG)

// Use size helpers
let area = modal::large_modal_rect(frame.area());

// Use style helpers
modal::bold_style()
modal::cursor_style()

// Use mode formatter
modal::format_mode_indicator("Normal", None)

// Use standard block
modal::standard_block(" Title ")
```

## Testing Your UI Code

### Visual Consistency Checklist

Before committing UI changes, verify:

- [ ] No hardcoded `Color::` constants (except in `modal.rs`)
- [ ] No inline `Style::default()` with modifiers (use helpers)
- [ ] All modals use `large_modal_rect()` or `small_modal_rect()`
- [ ] All modals have status bars via `split_with_status_bar()`
- [ ] All borders use `Borders::ALL` via `standard_block()`
- [ ] Mode indicators use `format_mode_indicator()`
- [ ] Error messages use `error_style()`
- [ ] Cursor/selection uses `cursor_style()`

### Running Tests

```bash
# Run all tests
cargo test

# Run UI-specific tests
cargo test --lib ui::

# Check for style consistency
cargo clippy --all-targets
```

## Migration Guide

### Updating Existing UI Code

If you find UI code that doesn't follow these guidelines:

1. **Identify hardcoded colors**
   ```bash
   grep -r "Color::" src/ui/*.rs | grep -v modal.rs
   ```

2. **Replace with constants**
   ```rust
   // Before
   Style::default().fg(Color::Red)
   
   // After
   modal::error_style()
   ```

3. **Identify inline styles**
   ```bash
   grep -r "Style::default()" src/ui/*.rs | grep -v modal.rs
   ```

4. **Replace with helpers**
   ```rust
   // Before
   Style::default().add_modifier(Modifier::BOLD)
   
   // After
   modal::bold_style()
   ```

5. **Run tests**
   ```bash
   cargo test --lib
   cargo clippy
   ```

## Version History

### v0.12.0 (2026-03-14)
- Created `src/ui/modal.rs` with centralized styles
- Added 8 style helper functions
- Added 8 color constants
- Refactored all UI files to use centralized styles
- Established single source of truth for all styling
- Zero hardcoded colors in UI layer

### Future Enhancements

Planned for v0.12.1+:
- Component library (`src/ui/components/`)
- Theme system (light/dark modes)
- User-configurable color schemes
- Style presets

---

**Questions?** See `src/ui/modal.rs` for implementation details.
