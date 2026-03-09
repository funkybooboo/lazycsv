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
- **[Project Roadmap](../plans/roadmap.md)**: The master plan. This document contains the versioned checklist of features and milestones.

## Quick Links

### For Users
- **Getting Started**: See main [README](../README.md)
- **Keyboard Shortcuts**: Press `?` in the app or see [keybindings.md](keybindings.md)
- **Feature List**: See [features.md](features.md) for current and planned features

### For Developers
- **Contributing**: See [development.md](development.md)
- **Todo List**: See [plans/roadmap.md](../plans/roadmap.md) for development checklist
- **Architecture**: See [architecture.md](architecture.md) for code structure

## Project Status

### v0.1.0 - Foundation  Complete
- Fast CSV loading and display (in-memory)
- Vim-style navigation
- Multi-file switching
- Comprehensive test suite

### v0.2.0 - Type Safety Refactor  Complete (v0.2.1 - v0.2.6)
-  **v0.2.1 - v0.2.6**: Type safety refactor COMPLETE
  - v0.2.1: Type safety foundation
  - v0.2.2: Separation of concerns
  - v0.2.3: Better naming & consistency
  - v0.2.4: Code organization
  - v0.2.5: Clean code improvements
  - v0.2.6: Testing & validation

### Roadmap to v1.0

**Completed (v0.1.0 - v0.8.1):**
- Foundation, navigation, vim editing, row/column operations
- Magnifier mode (full vim editor for cells)
- Search and filtering
- SQL query mode with multi-file support

**Next Major Versions:**
- **v0.9.0** - Configuration system (themes, keybindings)
- **v0.10.0** - Undo/redo system (u, Ctrl+r, .)
- **v0.11.0** - SQL editor vim editing (full modal editing)
- **v0.14.0** - Cell transforms (case toggle, sort, filter)
- **v0.18.0** - SQL IntelliSense (auto-completion)
- **v0.22.0** - Macros (command recording)
- **v1.0.0** - First stable release

See [../plans/roadmap.md](../plans/roadmap.md) for complete details.

## Support

- **Issues**: [GitHub Issues](https://github.com/funkybooboo/lazycsv/issues)
- **Discussions**: [GitHub Discussions](https://github.com/funkybooboo/lazycsv/discussions)
- **Contributing**: See [development.md](development.md)

## License

GPL License - see [LICENSE](../LICENSE) file for details.
