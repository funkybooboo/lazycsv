# LazyCSV Documentation

Welcome to the LazyCSV documentation! This directory contains comprehensive documentation about LazyCSV's features, design, and development.

## Documentation Index

This documentation is structured to guide you from high-level concepts to low-level implementation details.

### The "What" and "Why"
- **[Features](features.md)**: What LazyCSV does. Start here to understand the intended functionality and user-facing features. This document defines the "what" we are building.

### The "How" - Design and Architecture
- **[Design](design.md)**: How LazyCSV should look and feel. This document covers the UI/UX, visual identity, and interaction design. It translates features into a user experience.
- **[Architecture](architecture.md)**: How LazyCSV is built. This document dives into the code structure, data flow, and core components. It's the blueprint for the implementation.

### The "How-To" - Development and Reference
- **[Development](development.md)**: How to contribute to LazyCSV. This guide outlines our development process, coding standards, and the "test, write, test, docs" workflow.
- **[Keybindings](keybindings.md)**: A comprehensive reference for all keyboard shortcuts. Essential for both users and developers.
- **[Project Roadmap](../plans/road_map.md)**: The master plan. This document contains the versioned checklist of features and milestones.

## Quick Links

### For Users
- **Getting Started**: See main [README](../README.md)
- **Keyboard Shortcuts**: Press `?` in the app or see [keybindings.md](keybindings.md)
- **Feature List**: See [features.md](features.md) for current and planned features

### For Developers
- **Contributing**: See [development.md](development.md)
- **Todo List**: See [plans/road_map.md](../plans/road_map.md) for development checklist
- **Architecture**: See [architecture.md](architecture.md) for code structure

## Project Status

### v0.1.0 - Foundation ✅ Complete
- Fast CSV loading and display (in-memory)
- Vim-style navigation
- Multi-file switching
- Comprehensive test suite

### v0.2.0 - Type Safety Refactor ✅ Complete (v0.2.1 - v0.2.6)
- ✅ **v0.2.1 - v0.2.6**: Type safety refactor COMPLETE
  - v0.2.1: Type safety foundation
  - v0.2.2: Separation of concerns
  - v0.2.3: Better naming & consistency
  - v0.2.4: Code organization
  - v0.2.5: Clean code improvements
  - v0.2.6: Testing & validation

### Roadmap to v1.0
- **v0.2.0** - Type safety refactor ✅ Complete (All 6 phases)
- **v0.3.0** - Advanced navigation (gg, G, counts, column jumps)
- **v0.4.0** - Quick editing (Insert mode)
- **v0.5.0** - **Vim magnifier** (full vim editor embedded)
- **v0.6.0** - Save/quit guards
- **v0.7.0** - Row operations (o, O, dd, yy, p)
- **v0.8.0** - Column operations (:addcol, :delcol)
- **v0.9.0** - Header management (gh to edit headers)
- **v1.0.0** - Undo/redo system (u, Ctrl+r)

## Support

- **Issues**: [GitHub Issues](https://github.com/funkybooboo/lazycsv/issues)
- **Discussions**: [GitHub Discussions](https://github.com/funkybooboo/lazycsv/discussions)
- **Contributing**: See [development.md](development.md)

## License

GPL License - see [LICENSE](../LICENSE) file for details.
