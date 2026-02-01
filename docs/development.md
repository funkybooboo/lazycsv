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
│   ├── file_scanner.rs     - CSV file discovery
│   ├── csv_data.rs         - CSV data model (type-safe v0.2.0)
│   ├── domain/             - Domain types (NEW in v0.2.0)
│   │   ├── mod.rs          - Module exports
│   │   └── position.rs     - RowIndex, ColIndex, Position types
│   ├── input/              - Input actions (NEW in v0.2.0)
│   │   ├── mod.rs          - Module exports
│   │   └── actions.rs      - UserAction, InputResult enums
│   ├── app/                - Application logic
│   │   ├── mod.rs          - App struct & state (type-safe v0.2.0)
│   │   ├── input.rs        - Keyboard input handling
│   │   ├── navigation.rs   - Navigation methods (type-safe v0.2.0)
│   │   └── constants.rs    - App constants & messages
│   └── ui/                 - User interface
│       ├── mod.rs          - Main render function
│       ├── table.rs        - Table rendering (type-safe v0.2.0)
│       ├── status.rs       - Status bar & file switcher (type-safe v0.2.0)
│       ├── help.rs         - Help overlay
│       └── utils.rs        - Utility functions
│
├── tests/                  - Test suite
│   ├── README.md           - Test documentation
│   ├── app_test.rs         - Application logic tests
│   ├── cli_test.rs         - CLI parsing tests
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
│   └── todo.md       - Development checklist
│
├── Cargo.toml        - Dependencies
├── Taskfile.yml      - Task runner config
└── README.md         - Project readme
```

## Development Workflow

### 1. Pick a Task

Check [plans/todo.md](../plans/todo.md) for open items:

```bash
# View todo list
cat plans/todo.md

# Check current phase
grep "## Phase" plans/todo.md | head -5
```

### 2. Create a Branch

```bash
git checkout -b feature/your-feature-name
# Or: fix/bug-name
```

### 3. Implement

```bash
# Edit code
vim src/app.rs

# Build and test
task build
task test

# Run to verify
task run
```

### 4. Format and Lint

```bash
# Format code
task fmt

# Run clippy
task clippy

# Or run all checks
task all
```

### 5. Commit

```bash
# Add changes
git add .

# Commit with descriptive message
git commit -m "feat: add fuzzy search for column names

- Implement fuzzy matching with fuzzy-matcher crate
- Add search overlay UI
- Support j/k navigation in results
- Add tests for search scoring

Closes #42"
```

### 6. Test

Test your changes thoroughly:

```bash
# Run unit tests
task test

# Run with various CSV files
task run
cargo run -- customers.csv
cargo run -- large_file.csv

# Test edge cases
cargo run -- empty.csv
cargo run -- single_column.csv
cargo run -- wide.csv
```

### 7. Submit PR

```bash
# Push branch
git push origin feature/your-feature-name

# Create PR on GitHub
# - Describe changes
# - Link to issue
# - Add screenshots/demos if UI changes
```

## Coding Standards

### Rust Style

Follow Rust conventions:

```rust
// Good: Clear names, proper error handling
pub fn load_file(path: &Path) -> Result<CsvData> {
    let data = CsvData::from_file(path)
        .context("Failed to load CSV")?;
    Ok(data)
}

// Bad: Unwrap, abbreviations
pub fn ld_f(p: &Path) -> CsvData {
    CsvData::from_file(p).unwrap()
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
/// * `Ok(CsvData)` - Loaded data
/// * `Err` - File not found or invalid CSV
///
/// # Example
/// ```
/// let data = CsvData::from_file(Path::new("data.csv"))?;
/// ```
pub fn from_file(path: &Path) -> Result<CsvData> {
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
        let data = CsvData {
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
2. Update [todo.md](../plans/todo.md) with tasks
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
