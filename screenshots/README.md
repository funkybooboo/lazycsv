# Screenshots Needed for Documentation

This directory should contain screenshots demonstrating LazyCSV features.

## Required Screenshots

### README.md
- `main-view.png` - Main table view showing CSV data with vim-style navigation

### README.md - Screenshots Section
- `normal-mode.png` - Normal mode navigation showing table, status bar, file switcher
- `insert-mode.png` - Quick cell editing in Insert mode
- `magnifier-mode.png` - Full vim editor for multi-line cell content
- `visual-mode.png` - Visual mode selecting multiple cells/rows
- `sql-editor.png` - SQL editor with query being written
- `sql-results.png` - SQL query results displayed as table
- `search.png` - Fuzzy search overlay with results

### docs/design.md - Screen Layouts
- `design-default-view.png` - Default view (v0.1.0) showing normal mode
- `design-help-overlay.png` - Help overlay showing all keybindings
- `design-insert-mode.png` - Insert mode with cell editing
- `design-visual-mode.png` - Visual selection mode
- `design-search-overlay.png` - Fuzzy search overlay
- `design-sql-editor.png` - SQL editor overlay
- `design-sql-results.png` - SQL query results view

### docs/design.md - Error States
- `design-error-file-not-found.png` - Error screen for missing file
- `design-empty-csv.png` - Empty CSV file view
- `design-unsaved-warning.png` - Unsaved changes warning dialog

## How to Capture Screenshots

1. **Use a terminal with good rendering** (recommended: Alacritty, Kitty, or WezTerm)
2. **Set terminal size** to ~120 columns x 30 rows for consistency
3. **Use sample data** that demonstrates the feature clearly
4. **Capture with transparency** if possible for better documentation integration

## Screenshot Guidelines

- **Resolution**: High enough to be readable but not excessive (1920px width max)
- **Format**: PNG with compression
- **Content**: Show realistic CSV data (use included sample files)
- **Consistency**: Same terminal theme/font across all screenshots
- **Timing**: Capture at moments that clearly show the feature

## Sample Data for Screenshots

Use the provided sample CSV files:
- `sample.csv` - General demonstration
- `sales_data.csv` - For SQL query examples
- `customers.csv` - For multi-file navigation
- `products.csv` - For JOIN queries

## Notes

These screenshots will be referenced in:
1. Main README.md (quick overview)
2. docs/design.md (detailed UI/UX specification)
3. Future: Website, GitHub releases, social media

Quality screenshots help users understand LazyCSV's capabilities at a glance!
