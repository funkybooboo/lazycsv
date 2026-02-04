# Development Guide

Guide for contributing to LazyCSV.

## Getting Started

### Prerequisites

- **Rust**: 1.70+ (install from https://rustup.rs/)
- **Task**: Optional but recommended (install from https://taskfile.dev/)
- **Git**: For version control

### Clone and Build

```bash
# Clone repository
git clone https://github.com/funkybooboo/lazycsv.git
cd lazycsv

# Build
task build
# Or: cargo build

# Run
task run
# Or: cargo run -- sample.csv

# Run tests
task test
# Or: cargo test
```

## Project Structure

```
lazycsv/
├── src/                    - Rust source code
│   ├── main.rs             - Entry point & main loop
│   ├── lib.rs              - Library exports
│   ├── cli.rs              - CLI argument parsing
│   ├── domain/             - Domain types (v0.2.0)
│   │   ├── mod.rs          - Module exports
│   │   └── position.rs     - RowIndex, ColIndex, Position types
│   ├── input/              - Input handling (v0.2.0)
│   │   ├── actions.rs      - UserAction, InputResult enums
│   │   ├── state.rs        - InputState (pending commands, counts)
│   │   └── handler.rs      - Keyboard event handling
│   ├── navigation/         - Navigation commands (v0.2.0)
│   │   └── commands.rs     - Vim-style movement functions
│   ├── session/            - Multi-file sessions (v0.2.0)
│   │   └── mod.rs          - Session, FileConfig structures
│   ├── csv/                - CSV operations (v0.2.0)
│   │   └── document.rs     - Document struct (loading, parsing)
│   ├── file_system/        - File operations (v0.2.0)
│   │   └── discovery.rs    - CSV file scanning
│   ├── app/                - Application coordinator (v0.2.0)
│   │   ├── mod.rs          - App struct (6 fields)
│   │   └── messages.rs     - User-facing message strings
│   └── ui/                 - User interface (v0.2.0)
│       ├── mod.rs          - Main render function
│       ├── view_state.rs   - ViewState (viewport control)
│       ├── table.rs        - Table rendering (virtual scrolling)
│       ├── status.rs       - Status bar & file switcher
│       ├── help.rs         - Help overlay
│       └── utils.rs        - Utility functions
│
├── tests/                  - Test suite
│   ├── README.md           - Test documentation
│   ├── cli_integration_test.rs - CLI argument testing (7 tests)
│   └── integration_workflows_test.rs - End-to-end workflows (21 tests)
│   ├── csv_data_test.rs    - CSV loading tests
│   ├── csv_edge_cases_test.rs - Edge case tests
│   ├── file_scanner_test.rs - File discovery tests
│   ├── integration_workflows_test.rs - End-to-end tests
│   ├── navigation_workflows_test.rs - Navigation tests
│   ├── ui_rendering_test.rs - TUI rendering tests
│   ├── ui_state_test.rs    - UI state tests
│   └── ui_test.rs          - UI utility tests
│
├── docs/                   - Documentation
│   ├── README.md           - Docs index
│   ├── features.md         - Feature specs
│   ├── design.md           - UI/UX design
│   ├── architecture.md     - System architecture
│   ├── keybindings.md      - Keyboard shortcuts
│   └── development.md      - This file
│
├── plans/            - Planning documents
│   ├── README.md     - Plans index
│   └── roadmap.md       - Development checklist
│
├── Cargo.toml        - Dependencies
├── Taskfile.yml      - Task runner config
└── README.md         - Project readme
```

### Module Structure (v0.2.0)

**Current organization after v0.2.0 refactor:**

```
src/
├── domain/            # Domain types (RowIndex, ColIndex, Position)
├── input/             # Input handling
│   ├── actions.rs     # UserAction, NavigateAction
│   ├── state.rs       # InputState (pending commands, counts)
│   └── handler.rs     # Keyboard event handling
├── navigation/        # Navigation commands
│   └── commands.rs    # Vim-style movement functions
├── session/           # Multi-file session management
│   └── mod.rs         # Session, FileConfig structures
├── csv/               # CSV operations
│   └── document.rs    # Document struct (loading, parsing, get_cell)
├── file_system/       # File system operations
│   └── discovery.rs   # CSV file scanning
├── app/               # Application coordinator (thin layer)
│   ├── mod.rs         # App struct, main loop
│   └── messages.rs    # User-facing message strings
└── ui/                # UI rendering
    ├── view_state.rs  # ViewState (viewport, selection)
    ├── table.rs       # Table rendering
    ├── status.rs      # Status bar
    ├── help.rs        # Help overlay
    └── utils.rs       # Utility functions

tests/
├── cli_integration_test.rs      # CLI argument testing (7 tests)
└── integration_workflows_test.rs # End-to-end workflows (21 tests)
```

**Important Terminology (v0.2.0):**
- Use `Document` not `CsvData` or `csv_data`
- Use `ViewState` not `UiState` or `ui_state`
- Use `view_state` field not `ui` field
- `App` struct has 6 fields: document, view_state, input_state, session, should_quit, status_message

## Development Philosophy: The TDD Loop

LazyCSV follows a Test-Driven Development (TDD) approach to ensure code quality, correctness, and maintainability. The development process is an iterative loop: **Test -> Write Code -> Test -> Write Docs -> Repeat**.

This loop ensures that every piece of functionality is:
1.  **Correctly specified** by a test before it's written.
2.  **Verified** by that test after it's written.
3.  **Well-documented** for future users and developers.

The ideal workflow looks like this:

1.  **Write a Failing Test**: Before writing any implementation code, write a unit or integration test that describes the desired functionality and fails because the functionality doesn't exist yet. This is the "red" phase.
2.  **Write Code to Pass the Test**: Write the simplest, most straightforward code possible to make the test pass. This is the "green" phase.
3.  **Refactor**: With the safety of a passing test suite, refactor the code for clarity, efficiency, and to ensure it aligns with the project's architecture.
4.  **Document**: If the change is user-facing, update the relevant documentation. This could be adding a new keybinding to `keybindings.md`, explaining a new feature in `features.md`, or updating screenshots in `design.md`.
5.  **Commit**: Commit the changes with a clear, conventional commit message.
6.  **Repeat**: Move to the next task.

## Development Workflow

This section puts the TDD philosophy into a concrete, step-by-step process for contributing to LazyCSV.

### 1. Understand the Task

Before you write a single line of code, make sure you understand what you're building.
- **For the big picture**, check the **[Project Roadmap](../plans/roadmap.md)** to see the versioned feature list and where your contribution fits in.
- **For new features**, start with the **[features.md](features.md)** document to understand the requirements.
- **For UI/UX changes**, consult the **[design.md](design.md)** document to see how it should look and behave.
- **To understand the existing codebase**, refer to the **[architecture.md](architecture.md)** document.

### 2. Find or Create an Issue

All work should be tied to a GitHub issue. Find an existing issue or create a new one that describes the feature or bug you're working on.

### 3. Create a Branch

Create a new branch from `main` for your changes:
```bash
git checkout -b feature/your-feature-name
# Or: fix/bug-name
```

### 4. Write a Failing Test

This is the first step of the TDD loop. Add a new test to the test suite in the `tests/` directory or within the relevant module. Run `cargo test` and watch it fail. This is expected and good!

### 5. Implement the Feature

Now, write the code to make your new test pass.
- Run `cargo test` frequently to check your progress.
- Use `cargo build` and `cargo run` to manually verify your changes as you go.

### 6. Refactor and Document

Once your test is passing and the feature is working:
- **Refactor**: Clean up your code. Make it more readable, efficient, and aligned with the project's style. Ensure `cargo test` still passes.
- **Document**: Update any and all relevant documentation. Did you add a keybinding? Update `keybindings.md`. Did you change the UI? Update `design.md`.

### 7. Format and Lint

Ensure your code adheres to our quality standards by running:
```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings
```
Our pre-commit hooks should handle this, but it's good practice to run it manually as well.

### 8. Commit and Push

Commit your changes using the Conventional Commits format (e.g., `feat(search): add fuzzy matching`).
```bash
# Add your changes
git add .

# Commit
git commit -m "feat: your descriptive message"

# Push to your fork
git push origin feature/your-feature-name
```

### 9. Submit a Pull Request

Open a pull request on the main LazyCSV repository. In the description, link to the issue you're resolving and provide a clear summary of the changes.

## Coding Standards


### Rust Style

Follow Rust conventions:

```rust
// Good: Clear names, proper error handling
pub fn load_file(path: &Path) -> Result<Document> {
    let document = Document::from_file(path)
        .context("Failed to load CSV")?;
    Ok(document)
}

// Bad: Unwrap, abbreviations
pub fn ld_f(p: &Path) -> Document {
    Document::from_file(p).unwrap()
}
```

### Formatting

Use `rustfmt`:

```bash
task fmt
# Or: cargo fmt
```

Configuration in `rustfmt.toml` (if needed):
```toml
edition = "2021"
max_width = 100
```

### Linting

Use `clippy`:

```bash
task clippy
# Or: cargo clippy -- -D warnings
```

Fix all warnings before submitting PR.

### Documentation

Document public APIs:

```rust
/// Load CSV file from path.
///
/// # Arguments
/// * `path` - Path to CSV file
///
/// # Returns
/// * `Ok(Document)` - Loaded data
/// * `Err` - File not found or invalid CSV
///
/// # Example
/// ```
/// let document = Document::from_file(Path::new("data.csv"))?;
/// ```
pub fn from_file(path: &Path) -> Result<Document> {
    // ...
}
```

## Testing

### Unit Tests

Write tests for new functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::position::{RowIndex, ColIndex};

    #[test]
    fn test_cell_access() {
        let document = Document {
            headers: vec!["Name".to_string()],
            rows: vec![vec!["Alice".to_string()]],
            filename: "test.csv".to_string(),
            is_dirty: false,
        };

        // v0.2.0: Type-safe access with RowIndex and ColIndex
        assert_eq!(data.get_cell(RowIndex::new(0), ColIndex::new(0)), "Alice");
        assert_eq!(data.get_cell(RowIndex::new(0), ColIndex::new(1)), ""); // Out of bounds
    }
}
```

Run tests:
```bash
task test
task test-verbose   # With output
```

### Integration Tests

Create integration tests in `tests/`:

```rust
// tests/integration_test.rs
use lazycsv::*;

#[test]
fn test_load_and_navigate() {
    // Test end-to-end functionality
}
```

### Manual Testing

Create test CSV files:

```bash
# Generate sample files
task sample

# Test with custom data
echo "A,B,C\n1,2,3" > test.csv
cargo run -- test.csv
```

## Common Tasks

### Add a New Feature

1. Check [features.md](features.md) for spec
2. Update [roadmap.md](../plans/roadmap.md) with tasks
3. Implement in appropriate module
4. Add tests
5. Update documentation
6. Submit PR

### Fix a Bug

1. Create failing test that reproduces bug
2. Fix bug
3. Verify test passes
4. Add regression test
5. Submit PR

### Improve Performance

1. Profile with `cargo flamegraph`
2. Identify bottleneck
3. Optimize (with benchmarks)
4. Verify no regressions
5. Document improvements

### Update Dependencies

```bash
# Check outdated
cargo outdated

# Update Cargo.lock
cargo update

# Update Cargo.toml (manually)
# Then: cargo build
```

## Debugging

### Print Debugging

```rust
// Use eprintln! (goes to stderr, not terminal)
eprintln!("Debug: selected_row = {:?}", app.selected_row());
```

### Logging

```rust
// Add tracing (future)
use tracing::{info, debug, error};

info!("Loading file: {}", path.display());
debug!("Current state: {:?}", app);
```

### GDB/LLDB

```bash
# Build with debug symbols
cargo build

# Run in debugger
rust-gdb target/debug/lazycsv
# Or: rust-lldb
```

## Performance Profiling

### Flamegraph

```bash
# Install
cargo install flamegraph

# Profile
cargo flamegraph -- sample.csv

# Opens flamegraph in browser
```

### Benchmarking

```bash
# Install criterion (add to Cargo.toml)
[dev-dependencies]
criterion = "0.5"

# Write benchmarks in benches/
# Run
cargo bench
```

## Documentation

### Generate Docs

```bash
# Generate and open docs
cargo doc --open

# Or
task docs
```

### Update Docs

When adding features:
1. Update [features.md](features.md) with spec
2. Update [keybindings.md](keybindings.md) with new keys
3. Update [design.md](design.md) if UI changes
4. Update [README.md](../README.md) with user-facing info

## Release Process

### Version Bumping

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Commit: `git commit -m "chore: bump version to 0.2.0"`
4. Tag: `git tag v0.2.0`
5. Push: `git push && git push --tags`

### Building Release

```bash
# Build optimized binary
task build-release

# Strip debug symbols
strip target/release/lazycsv

# Test release build
./target/release/lazycsv sample.csv
```

### Publishing to Crates.io

```bash
# Login (one time)
cargo login

# Publish
cargo publish

# Or dry run first
cargo publish --dry-run
```

## CI/CD

### GitHub Actions

`.github/workflows/ci.yml`:
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
      - run: cargo build --release
```

## Common Issues

### Build Fails

```bash
# Clean and rebuild
cargo clean
cargo build
```

### Tests Fail

```bash
# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Terminal Messed Up

```bash
# If app crashes and terminal is broken
reset
# Or: stty sane
```

## Getting Help

### Documentation
- [Architecture](architecture.md) - System design
- [Features](features.md) - Feature specs
- [Design](design.md) - UI/UX design

### Community
- **Issues**: https://github.com/funkybooboo/lazycsv/issues
- **Discussions**: https://github.com/funkybooboo/lazycsv/discussions
- **Discord**: (future)

### Questions
- Check existing issues first
- Search discussions
- Ask in new discussion
- Tag maintainers if urgent

## Contributing Guidelines

### Code of Conduct

Be respectful and inclusive:
- Welcome newcomers
- Be patient with questions
- Give constructive feedback
- Follow code of conduct

### PR Guidelines

Good PRs:
- ✅ Focus on one thing
- ✅ Include tests
- ✅ Update documentation
- ✅ Pass all checks (fmt, clippy, test)
- ✅ Have clear description

Bad PRs:
- ❌ Mix unrelated changes
- ❌ No tests
- ❌ Don't update docs
- ❌ Have warnings/errors
- ❌ No description

### Commit Messages

Follow conventional commits:

```
feat: add fuzzy search
fix: resolve crash on empty file
docs: update keybindings guide
chore: bump dependencies
test: add tests for cell editing
refactor: simplify navigation logic
```

### Review Process

1. Maintainer reviews PR
2. Feedback provided
3. Author addresses feedback
4. Maintainer approves
5. PR merged

Typical turnaround: 1-3 days

## Development Tools

### Recommended VSCode Extensions
- rust-analyzer
- CodeLLDB (debugging)
- Even Better TOML
- Error Lens

### Recommended Cargo Tools
```bash
cargo install cargo-watch    # Auto-rebuild
cargo install cargo-outdated # Check deps
cargo install flamegraph     # Profiling
```

### Useful Aliases

Add to `~/.bashrc` or `~/.zshrc`:
```bash
alias cb='cargo build'
alias ct='cargo test'
alias cr='cargo run --'
alias cf='cargo fmt && cargo clippy'
```

## License

GPL License - see [LICENSE](../LICENSE) for details.

## Acknowledgments

Thanks to all contributors!

See [README.md](../README.md) for inspiration and acknowledgments.
