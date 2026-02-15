//! Unified clipboard system for LazyCSV.
//!
//! This module implements a smart clipboard that can store different types of data
//! (rows, columns, cells, regions) and adapts paste behavior based on context.
//!
//! ## Design Philosophy
//!
//! - One clipboard for all operations
//! - Type metadata tracks what was yanked
//! - Paste operations adapt intelligently based on clipboard type and context
//! - Support transpose operations (row→column, column→row)
//!
//! ## Examples
//!
//! ```ignore
//! // Yank a row, paste as row
//! yy → p  // Normal row paste
//!
//! // Yank a column, paste as column
//! ,yy → ,p  // Normal column paste
//!
//! // Transpose operations
//! yy → ,p  // Paste row as column (transpose)
//! ,yy → p  // Paste column as row (transpose)
//! ```

/// Type of data stored in the clipboard
#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardType {
    /// Single row of data (Vec<String> for cells)
    Row,
    /// Single column of data (Vec<String> including header)
    Column,
    /// Single cell value
    Cell,
    /// Rectangular region (rows × columns)
    Region { rows: usize, cols: usize },
}

/// Unified clipboard that stores various types of CSV data
#[derive(Debug, Clone)]
pub struct Clipboard {
    /// Type of data stored
    clipboard_type: ClipboardType,
    /// Raw data stored as 2D vector (rows × columns)
    /// For Row: Single row with N cells
    /// For Column: N rows with 1 cell each
    /// For Cell: Single row with 1 cell
    /// For Region: M rows with N cells each
    data: Vec<Vec<String>>,
}

impl Clipboard {
    /// Create a new empty clipboard
    pub fn new() -> Self {
        Self {
            clipboard_type: ClipboardType::Cell,
            data: vec![],
        }
    }

    /// Store a row in the clipboard
    pub fn yank_row(&mut self, row: Vec<String>) {
        self.clipboard_type = ClipboardType::Row;
        self.data = vec![row];
    }

    /// Store a column in the clipboard (includes header at index 0)
    pub fn yank_column(&mut self, column: Vec<String>) {
        self.clipboard_type = ClipboardType::Column;
        // Convert column to 2D format: each cell becomes its own row
        self.data = column.into_iter().map(|cell| vec![cell]).collect();
    }

    /// Store a single cell in the clipboard
    pub fn yank_cell(&mut self, cell: String) {
        self.clipboard_type = ClipboardType::Cell;
        self.data = vec![vec![cell]];
    }

    /// Store a rectangular region in the clipboard
    pub fn yank_region(&mut self, region: Vec<Vec<String>>) {
        let rows = region.len();
        let cols = region.first().map(|r| r.len()).unwrap_or(0);
        self.clipboard_type = ClipboardType::Region { rows, cols };
        self.data = region;
    }

    /// Get the clipboard type
    pub fn clipboard_type(&self) -> &ClipboardType {
        &self.clipboard_type
    }

    /// Check if clipboard is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get data as a row (for paste as row)
    /// Returns None if clipboard is empty
    pub fn as_row(&self) -> Option<Vec<String>> {
        match &self.clipboard_type {
            ClipboardType::Row => self.data.first().cloned(),
            ClipboardType::Column => {
                // Transpose: column becomes row
                Some(self.data.iter().map(|row| row[0].clone()).collect())
            }
            ClipboardType::Cell => self.data.first().cloned(),
            ClipboardType::Region { .. } => {
                // Return first row of region
                self.data.first().cloned()
            }
        }
    }

    /// Get data as a column (for paste as column)
    /// Returns None if clipboard is empty
    pub fn as_column(&self) -> Option<Vec<String>> {
        match &self.clipboard_type {
            ClipboardType::Column => {
                // Normal column paste
                Some(self.data.iter().map(|row| row[0].clone()).collect())
            }
            ClipboardType::Row => {
                // Transpose: row becomes column
                self.data.first().map(|row| row.iter().cloned().collect())
            }
            ClipboardType::Cell => self
                .data
                .first()
                .and_then(|row| row.first())
                .map(|cell| vec![cell.clone()]),
            ClipboardType::Region { .. } => {
                // Return first column of region
                Some(self.data.iter().map(|row| row[0].clone()).collect())
            }
        }
    }

    /// Get data as a rectangular region (for paste as region)
    /// Returns None if clipboard is empty
    pub fn as_region(&self) -> Option<Vec<Vec<String>>> {
        if self.data.is_empty() {
            None
        } else {
            Some(self.data.clone())
        }
    }

    /// Get dimensions of clipboard data (rows, cols)
    pub fn dimensions(&self) -> (usize, usize) {
        let rows = self.data.len();
        let cols = self.data.first().map(|r| r.len()).unwrap_or(0);
        (rows, cols)
    }

    /// Clear the clipboard
    pub fn clear(&mut self) {
        self.clipboard_type = ClipboardType::Cell;
        self.data.clear();
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_new() {
        let clipboard = Clipboard::new();
        assert!(clipboard.is_empty());
        assert_eq!(*clipboard.clipboard_type(), ClipboardType::Cell);
    }

    #[test]
    fn test_yank_row() {
        let mut clipboard = Clipboard::new();
        let row = vec!["A".to_string(), "B".to_string(), "C".to_string()];

        clipboard.yank_row(row.clone());

        assert_eq!(*clipboard.clipboard_type(), ClipboardType::Row);
        assert_eq!(clipboard.as_row(), Some(row));
        assert_eq!(clipboard.dimensions(), (1, 3));
    }

    #[test]
    fn test_yank_column() {
        let mut clipboard = Clipboard::new();
        let column = vec!["Header".to_string(), "A".to_string(), "B".to_string()];

        clipboard.yank_column(column.clone());

        assert_eq!(*clipboard.clipboard_type(), ClipboardType::Column);
        assert_eq!(clipboard.as_column(), Some(column));
        assert_eq!(clipboard.dimensions(), (3, 1));
    }

    #[test]
    fn test_yank_cell() {
        let mut clipboard = Clipboard::new();
        let cell = "Test".to_string();

        clipboard.yank_cell(cell.clone());

        assert_eq!(*clipboard.clipboard_type(), ClipboardType::Cell);
        assert_eq!(clipboard.as_row(), Some(vec![cell.clone()]));
        assert_eq!(clipboard.dimensions(), (1, 1));
    }

    #[test]
    fn test_yank_region() {
        let mut clipboard = Clipboard::new();
        let region = vec![
            vec!["A1".to_string(), "B1".to_string()],
            vec!["A2".to_string(), "B2".to_string()],
            vec!["A3".to_string(), "B3".to_string()],
        ];

        clipboard.yank_region(region.clone());

        assert_eq!(
            *clipboard.clipboard_type(),
            ClipboardType::Region { rows: 3, cols: 2 }
        );
        assert_eq!(clipboard.as_region(), Some(region));
        assert_eq!(clipboard.dimensions(), (3, 2));
    }

    #[test]
    fn test_transpose_row_to_column() {
        let mut clipboard = Clipboard::new();
        let row = vec!["A".to_string(), "B".to_string(), "C".to_string()];

        clipboard.yank_row(row.clone());

        // Paste row as column (transpose)
        let column = clipboard.as_column();
        assert_eq!(column, Some(row));
    }

    #[test]
    fn test_transpose_column_to_row() {
        let mut clipboard = Clipboard::new();
        let column = vec!["A".to_string(), "B".to_string(), "C".to_string()];

        clipboard.yank_column(column.clone());

        // Paste column as row (transpose)
        let row = clipboard.as_row();
        assert_eq!(row, Some(column));
    }

    #[test]
    fn test_clear() {
        let mut clipboard = Clipboard::new();
        clipboard.yank_row(vec!["A".to_string(), "B".to_string()]);

        assert!(!clipboard.is_empty());

        clipboard.clear();

        assert!(clipboard.is_empty());
        assert_eq!(*clipboard.clipboard_type(), ClipboardType::Cell);
    }

    #[test]
    fn test_empty_clipboard_as_row() {
        let clipboard = Clipboard::new();
        assert_eq!(clipboard.as_row(), None);
    }

    #[test]
    fn test_empty_clipboard_as_column() {
        let clipboard = Clipboard::new();
        assert_eq!(clipboard.as_column(), None);
    }

    #[test]
    fn test_region_as_row() {
        let mut clipboard = Clipboard::new();
        let region = vec![
            vec!["A1".to_string(), "B1".to_string()],
            vec!["A2".to_string(), "B2".to_string()],
        ];

        clipboard.yank_region(region.clone());

        // Should return first row
        assert_eq!(
            clipboard.as_row(),
            Some(vec!["A1".to_string(), "B1".to_string()])
        );
    }

    #[test]
    fn test_region_as_column() {
        let mut clipboard = Clipboard::new();
        let region = vec![
            vec!["A1".to_string(), "B1".to_string()],
            vec!["A2".to_string(), "B2".to_string()],
        ];

        clipboard.yank_region(region.clone());

        // Should return first column
        assert_eq!(
            clipboard.as_column(),
            Some(vec!["A1".to_string(), "A2".to_string()])
        );
    }
}
