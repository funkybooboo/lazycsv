//! SQL pre-execution diagnostic markers.

/// Severity level for a SQL diagnostic
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

/// A pre-execution diagnostic marker in the SQL editor
#[derive(Debug, Clone)]
pub struct SqlDiagnostic {
    /// 0-based line number
    pub line: usize,
    /// 0-based start column (inclusive)
    pub col_start: usize,
    /// 0-based end column (exclusive)
    pub col_end: usize,
    /// Human-readable message
    pub message: String,
    /// Severity level
    pub severity: DiagnosticSeverity,
}
