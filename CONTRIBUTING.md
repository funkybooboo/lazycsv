# Contributing to LazyCSV

First off, thank you for considering contributing to LazyCSV! It's people like you that make open source software such a great community. We welcome any and all contributions.

This document provides guidelines for contributing to the project. Please read it carefully to ensure a smooth and effective contribution process.

## Code of Conduct

All contributors are expected to follow our [Code of Conduct](CODE_OF_CONDUCT.md). Please make sure you are familiar with its contents.

## How Can I Contribute?

There are many ways to contribute, from writing code to improving documentation. Before you start, we recommend reading through our documentation in the `docs/` directory to get a good understanding of the project's goals, design, and architecture.

### Reporting Bugs

If you find a bug, please open an issue on our [GitHub issue tracker](https://github.com/funkybooboo/lazycsv/issues).

Please include the following in your bug report:
- Your operating system and terminal.
- The version of LazyCSV you are using.
- A clear and concise description of the bug.
- Steps to reproduce the bug.
- Any relevant error messages or screenshots.

### Suggesting Enhancements

If you have an idea for a new feature or an improvement to an existing one, please open an issue on our [GitHub issue tracker](https://github.com/funkybooboo/lazycsv/issues).

Before submitting, you may want to consult our project documentation to see how your idea fits in:
- **[Features Guide](docs/features.md)**: See what features are already planned.
- **[Design Document](docs/design.md)**: Understand the project's UI/UX philosophy.

Please provide a clear and detailed explanation of the feature, including:
- What the feature is and why it's needed.
- A description of the proposed user experience.
- Any potential drawbacks or trade-offs.

### Pull Requests

We welcome pull requests! If you'd like to contribute code, please follow these steps:

1.  **Understand the project**. Before you start coding, please read our **[Development Guide](docs/development.md)** and familiarize yourself with the project's structure as described in the **[Architecture Document](docs/architecture.md)**.
2.  **Fork the repository** and clone it to your local machine.
3.  **Create a new branch** for your changes: `git checkout -b feature/your-feature-name` or `fix/bug-name`.
4.  **Make your changes**, adhering to the coding standards outlined below and our [Test-Driven Development](docs/development.md#development-philosophy-the-tdd-loop) philosophy.
5.  **Add tests** for your changes.
6.  **Ensure all tests pass** by running `cargo test`.
7.  **Format and lint** your code with `cargo fmt` and `cargo clippy -- -D warnings`.
8.  **Commit your changes** with a descriptive commit message (see "Commit Messages" below).
9.  **Push your branch** to your fork and open a pull request against the `main` branch of the original repository.

## Getting Started

To get started with development, you'll need to have Rust 1.70+ installed.

```bash
# Clone the repository
git clone https://github.com/funkybooboo/lazycsv.git
cd lazycsv

# Build the project
cargo build

# Run the application
cargo run -- test_data/sample.csv

# Run the test suite
cargo test
```

For a more detailed guide on the project structure and development workflow, please see the [Development Guide](docs/development.md).

## Coding Standards

### Commit Messages

We use the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification. This helps us to automate changelog generation and versioning.

Your commit message should be structured as follows:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Example:**

```
feat: add fuzzy search for column names

Implements fuzzy matching for column names in the search overlay.
Adds tests for search scoring and result navigation.

Closes #42
```

Common types include: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`.

### Rust Style

-   **Formatting**: All code must be formatted with `rustfmt`. You can run this with `cargo fmt`.
-   **Linting**: We use `clippy` to enforce code quality. Please ensure your code has no warnings by running `cargo clippy -- -D warnings`.
-   **Documentation**: All public functions, structs, and enums should have clear and concise documentation comments.

Our CI pipeline will check for formatting and linting errors, so please run these checks locally before submitting a pull request.

## Testing

All new features and bug fixes should be accompanied by tests.

-   **Unit tests** should be placed in a `#[cfg(test)]` module at the bottom of the file they are testing.
-   **Integration tests** are located in the `tests/` directory.

You can run all tests with `cargo test`.

## License

By contributing to LazyCSV, you agree that your contributions will be licensed under the [GPL-3.0-or-later License](LICENSE).
