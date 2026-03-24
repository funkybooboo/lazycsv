# Error Handling Policy

**Version:** v0.16.1
**Last Updated:** 2026-03-24

## Overview

This document defines when `unwrap()` and `expect()` are acceptable in the LazyCSV codebase versus when proper error handling is required.

**Current State:** 14 unwrap/expect calls in production code, all safe (as of v0.16.1)
**Goal:** Eliminate unwraps on critical paths while maintaining pragmatic approach for internal code

## Core Principle

**Unwrap only when a panic is theoretically impossible or acceptable.**

- **User-facing code:** Must handle errors gracefully with `Result<T, E>` or `Option<T>`
- **Internal code:** Can use unwrap/expect if panic is truly impossible or documents a logic error
- **Tests:** Unwrap is acceptable for clarity and fast failure on unexpected conditions

## When Unwrap Is FORBIDDEN

### 1. File I/O Operations
All file operations can fail due to permissions, disk space, network issues, etc.

**BAD:**
```rust
let content = fs::read_to_string(path).unwrap();
let file = File::open(path).expect("failed to open file");
```

**GOOD:**
```rust
let content = fs::read_to_string(path)
    .map_err(|e| Error::FileRead { path: path.clone(), source: e })?;

let file = File::open(path)
    .map_err(|e| Error::FileOpen { path: path.clone(), source: e })?;
```

### 2. Parsing User Input
CSV parsing, SQL queries, command parsing - all can fail with malformed input.

**BAD:**
```rust
let row: Vec<String> = line.split(',').map(|s| s.to_string()).collect();
let value = row[column_index].unwrap(); // index could be out of bounds
```

**GOOD:**
```rust
let row: Vec<String> = line.split(',').map(|s| s.to_string()).collect();
let value = row.get(column_index)
    .ok_or(Error::ColumnIndexOutOfBounds { index: column_index, max: row.len() })?;
```

### 3. External API Calls
HTTP requests, clipboard operations, terminal interactions - all fallible.

**BAD:**
```rust
let clipboard = Clipboard::new().unwrap();
clipboard.set_contents(text).unwrap();
```

**GOOD:**
```rust
let mut clipboard = Clipboard::new()
    .map_err(|e| Error::ClipboardInit(e))?;
clipboard.set_contents(text)
    .map_err(|e| Error::ClipboardWrite(e))?;
```

### 4. Type Conversions
String to number, encoding conversions, etc.

**BAD:**
```rust
let num = s.parse::<i64>().unwrap();
```

**GOOD:**
```rust
let num = s.parse::<i64>()
    .map_err(|_| Error::InvalidNumber { value: s.to_string() })?;
```

### 5. Array/Vec Indexing
Assume bounds could be violated unless proven otherwise.

**BAD:**
```rust
let first_row = rows[0].unwrap();
let cell = row[column].clone();
```

**GOOD:**
```rust
let first_row = rows.first()
    .ok_or(Error::EmptyDocument)?;
let cell = row.get(column)
    .ok_or(Error::ColumnOutOfBounds { column, max: row.len() })?
    .clone();
```

## When Unwrap Is ACCEPTABLE

### 1. Tests
Tests should fail fast and loudly on unexpected conditions.

```rust
#[test]
fn test_parse_valid_csv() {
    let doc = CsvDocument::from_str("a,b,c\n1,2,3").unwrap();
    assert_eq!(doc.rows().len(), 1);
    assert_eq!(doc.rows()[0][0], "1"); // Acceptable in tests
}
```

### 2. Static/Const Initialization
When values are compile-time validated.

```rust
static REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d+$").expect("regex pattern is valid")
});
```

Use `expect()` with clear message explaining why it's safe.

### 3. Post-Validation Logic
After explicitly checking invariants.

```rust
if rows.is_empty() {
    return Err(Error::EmptyDocument);
}
let first_row = rows[0].clone(); // Safe: we just checked is_empty()
```

**Better alternative:** Use pattern matching to make safety obvious:
```rust
match rows.first() {
    None => return Err(Error::EmptyDocument),
    Some(row) => {
        // Use row here
    }
}
```

### 4. Internal Data Structures
When internal invariants guarantee success.

```rust
// If Session guarantees documents Vec is never empty after initialization
let current_doc = &self.documents[self.current_index]; // Document invariant
```

**Must be documented:**
```rust
/// Returns reference to current document.
///
/// # Panics
/// Panics if documents Vec is empty. This is a logic error - Session should
/// never exist with empty documents after initialization.
fn current_document(&self) -> &CsvDocument {
    &self.documents[self.current_index]
}
```

### 5. Lock Poisoning
For mutexes/locks where poisoning indicates unrecoverable state.

```rust
let guard = mutex.lock().expect("lock poisoned - unrecoverable");
```

## Migration Strategy

### Phase 1: Audit Critical Paths (v0.1.1)
1. **File I/O:** `src/csv/document.rs`, `src/session/mod.rs`, `src/main.rs`
2. **Parsing:** `src/csv/parser.rs`, `src/query/mod.rs`, `src/input/handler.rs`
3. **User Commands:** All command handlers in `src/input/handler.rs`

### Phase 2: Replace with Proper Error Types
Define comprehensive error types in each module:

```rust
#[derive(Debug, thiserror::Error)]
pub enum CsvError {
    #[error("Failed to read file {path}: {source}")]
    FileRead { path: PathBuf, source: io::Error },
    
    #[error("Failed to parse CSV: {0}")]
    ParseError(#[from] csv::Error),
    
    #[error("Column index {index} out of bounds (max: {max})")]
    ColumnOutOfBounds { index: usize, max: usize },
    
    #[error("Document is empty")]
    EmptyDocument,
}
```

### Phase 3: Propagate Errors
Use `?` operator to propagate errors up to UI layer:

```rust
// Before
pub fn load_file(path: &Path) -> CsvDocument {
    let content = fs::read_to_string(path).unwrap();
    CsvDocument::from_str(&content).unwrap()
}

// After
pub fn load_file(path: &Path) -> Result<CsvDocument, CsvError> {
    let content = fs::read_to_string(path)
        .map_err(|e| CsvError::FileRead { path: path.to_path_buf(), source: e })?;
    CsvDocument::from_str(&content)
}
```

### Phase 4: Handle at UI Boundary
Display user-friendly errors in status bar:

```rust
match load_file(path) {
    Ok(doc) => {
        self.document = doc;
        self.status = Status::success("File loaded");
    }
    Err(e) => {
        self.status = Status::error(format!("Failed to load file: {}", e));
    }
}
```

## Audit Checklist

Use this grep command to find unwrap/expect calls:
```bash
rg -n "\.unwrap\(\)|\.expect\(" --type rust src/
```

For each occurrence, ask:
1. **Can this fail?** (If yes, requires proper error handling)
2. **Is this user-facing?** (If yes, requires proper error handling)
3. **Is this a test?** (If yes, unwrap is acceptable)
4. **Is the invariant documented?** (If no, add doc comment explaining panic condition)

## Expected Outcomes

- **Reduced crashes:** User never sees panic messages
- **Better error messages:** Users understand what went wrong and how to fix it
- **Maintainability:** Clear error types document failure modes
- **Debuggability:** Error context (file paths, line numbers) preserved in error types

## References

- [Rust Error Handling Survey](https://blog.burntsushi.net/rust-error-handling/)
- [thiserror crate](https://docs.rs/thiserror/) - Recommended for error types
- [anyhow crate](https://docs.rs/anyhow/) - Consider for application-level errors

## Progress Tracking

**v0.1.1 Goals:**
- [x] Document error handling policy
- [ ] Audit critical paths (file I/O, parsing, user commands)
- [ ] Define module-level error types
- [ ] Replace unwraps in critical paths
- [ ] Add integration tests for error conditions

**Metrics:**
- Starting: 593 unwrap/expect calls
- Target: <100 unwrap/expect calls in src/ (excluding tests)
- Critical paths: 0 unwraps
