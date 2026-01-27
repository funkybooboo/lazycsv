# Plans & Design Documents

This directory contains planning documents, design specifications, and development tracking for LazyCSV.

## Files

### `todo.md`
**Comprehensive development checklist** with checkboxes for tracking progress.

Organized by phase:
- ✅ Phase 1: Core Viewing (MVP) - Complete!
- 📋 Phase 2: Cell Editing
- 📋 Phase 3: Row/Column Operations
- 📋 Phase 4: Advanced Features (Fuzzy Search, Sort, Filter)
- 📋 Phase 5: Multi-File/Sheet Navigation & Excel Support

**How to use:**
- Check off items as you complete them
- Add new items as needed
- Review before starting a new phase
- Track blockers and dependencies

### Implementation Plan (in git history)
The original comprehensive implementation plan is stored in:
- `/home/nate/.claude/plans/piped-toasting-milner.md`

This plan includes:
- Complete technical specifications
- File-by-file implementation details
- Keybindings design
- UI mockups
- Feature specifications
- Testing strategy

## Development Process

### Phase Workflow

1. **Planning**
   - Review phase requirements in `todo.md`
   - Read relevant sections in `docs/`
   - Identify dependencies and blockers

2. **Implementation**
   - Work through checklist items
   - Check off completed items
   - Add tests as you go
   - Update documentation

3. **Testing**
   - Run `task test`
   - Test manually with sample CSVs
   - Verify edge cases
   - Performance check

4. **Documentation**
   - Update `docs/features.md` with new features
   - Update keybindings in `docs/keybindings.md`
   - Update README.md if needed
   - Add examples

5. **Review**
   - Run `task all` (format + lint + test)
   - Review checklist completeness
   - Plan next phase

### Current Status

**Phase 1 (MVP): ✅ Complete**

Implemented:
- CSV loading and display
- Vim-style navigation (hjkl, gg, G, etc.)
- Row numbers and column letters (A, B, C...)
- Multi-file navigation ([/] keys)
- Always-visible file switcher
- Help overlay (? key)
- Status bar with context
- Horizontal scrolling

Next up:
- Phase 2: Cell editing, saving, undo/redo

### Tracking Progress

Use these tools to track development:

```bash
# View todo list
cat plans/todo.md

# Check what's done in current phase
grep "^\- \[x\]" plans/todo.md

# Check what's pending in current phase
grep "^\- \[ \]" plans/todo.md

# Run tests to verify implementation
task test

# Build and run
task run
```

## Design Decisions

Key decisions documented in this directory:

### Phase 1 Decisions
- ✅ **No colors** - Monochrome design for now
- ✅ **Row/column numbers** - Excel-style (A, B, C... and 1, 2, 3...)
- ✅ **File switcher at bottom** - Always visible, doesn't block data
- ✅ **Multi-file navigation** - Innovation! CSV files like Excel sheets
- ✅ **Help as overlay** - Press ?, doesn't lose context
- ✅ **~10 columns visible** - Horizontal scroll for wide tables
- ✅ **Truncate at 20 chars** - Longer text shows with ...

### Phase 2 Decisions (Planned)
- 📋 **Select-all in edit** - Most edits replace, not append
- 📋 **No delete confirmation** - Undo provides safety
- 📋 **Vim-style quit** - q warns, :q! forces
- 📋 **Atomic save** - Write to temp, rename on success

### Phase 4 Decisions (Planned)
- 📋 **Fuzzy search** - Not just substring, score-based
- 📋 **In-place sort** - Actually reorder data (undoable)
- 📋 **Case-insensitive search** - More useful for data

## Future Planning

### Phase 6+ Ideas
- Configuration file (`~/.config/lazycsv/config.toml`)
- Custom keybindings
- Optional color themes
- SQL query mode
- Export formats (JSON, Markdown, HTML)
- Diff mode (compare CSVs)
- Plugin system

### Long-term Vision
- Fastest CSV viewer/editor in terminal
- Most intuitive keybindings
- Best multi-file experience
- Excel integration without compromise
- Extensible for custom workflows

## Contributing to Plans

When adding features or phases:

1. **Update todo.md**
   - Add new checklist items
   - Organize by phase
   - Mark dependencies

2. **Update docs/**
   - Add feature specs to `docs/features.md`
   - Add keybindings to `docs/keybindings.md`
   - Update design in `docs/design.md`

3. **Document decisions**
   - Explain rationale
   - Consider trade-offs
   - Link to discussions

## Questions?

- **Features**: See `docs/features.md`
- **Design**: See `docs/design.md`
- **Development**: See `docs/development.md`
- **Architecture**: See `docs/architecture.md`

## License

GPL License - see [LICENSE](../LICENSE) for details.
