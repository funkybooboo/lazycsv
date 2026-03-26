//! CSV document parsing and representation
//!
//! Handles loading CSV files from disk, parsing with configurable
//! delimiters and encoding, and providing in-memory document access.

pub mod document;
pub mod foreign_formats;
pub mod row_storage;
pub mod writer;
pub mod xlsx;

pub use document::Document;
pub use writer::{write_csv_atomic, write_csv_content};

/// Detect the field delimiter from CSV text by analyzing the first few lines.
/// Checks for tab, pipe, semicolon, and comma. Returns the most consistent delimiter.
pub fn detect_delimiter(text: &str) -> u8 {
    let candidates: &[u8] = b"\t|;,";
    let lines: Vec<&str> = text.lines().take(10).collect();

    if lines.is_empty() {
        return b',';
    }

    let mut best = b',';
    let mut best_score = 0i64;

    for &delim in candidates {
        let counts: Vec<usize> = lines
            .iter()
            .map(|line| line.as_bytes().iter().filter(|&&b| b == delim).count())
            .collect();

        // Skip if delimiter doesn't appear
        if counts.iter().all(|&c| c == 0) {
            continue;
        }

        // Score: consistency (all lines have same count) + frequency
        let first = counts[0];
        let consistent = counts.iter().all(|&c| c == first);
        let total: usize = counts.iter().sum();

        // Prefer: consistent count across lines, then higher frequency
        let score = if consistent && first > 0 {
            (total as i64) * 10 + 100
        } else {
            total as i64
        };

        if score > best_score {
            best_score = score;
            best = delim;
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_delimiter_comma() {
        assert_eq!(detect_delimiter("a,b,c\n1,2,3\n"), b',');
    }

    #[test]
    fn test_detect_delimiter_tab() {
        assert_eq!(detect_delimiter("a\tb\tc\n1\t2\t3\n"), b'\t');
    }

    #[test]
    fn test_detect_delimiter_pipe() {
        assert_eq!(detect_delimiter("a|b|c\n1|2|3\n"), b'|');
    }

    #[test]
    fn test_detect_delimiter_semicolon() {
        assert_eq!(detect_delimiter("a;b;c\n1;2;3\n"), b';');
    }

    #[test]
    fn test_detect_delimiter_empty() {
        assert_eq!(detect_delimiter(""), b',');
    }

    #[test]
    fn test_detect_delimiter_no_delimiter() {
        assert_eq!(detect_delimiter("hello\nworld\n"), b',');
    }

    #[test]
    fn test_detect_delimiter_mixed_prefers_consistent() {
        // Tab is consistent (2 per line), comma appears but inconsistently
        assert_eq!(detect_delimiter("a\tb\tc\n1\t2\t3\n"), b'\t');
    }
}
