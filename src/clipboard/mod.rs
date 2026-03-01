//! Dual clipboard system for LazyCSV.
//!
//! Two independent buffers — a row buffer and a column buffer — with no
//! cross-buffer pasting. Row operations (yy/dd/p/P/o/O) use the row buffer;
//! column operations (,yy/,dd/,p/,P/,o/,O) use the column buffer.

/// Internal buffer shared by both row and column clipboards
#[derive(Debug, Clone)]
struct ClipboardBuffer {
    data: Vec<Vec<String>>,
}

impl ClipboardBuffer {
    fn new() -> Self {
        Self { data: vec![] }
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn store(&mut self, data: Vec<Vec<String>>) {
        self.data = data;
    }

    fn get(&self) -> Option<&Vec<Vec<String>>> {
        if self.data.is_empty() {
            None
        } else {
            Some(&self.data)
        }
    }

    fn clear(&mut self) {
        self.data.clear();
    }
}

/// Dual clipboard with independent row and column buffers
#[derive(Debug, Clone)]
pub struct DualClipboard {
    row_buffer: ClipboardBuffer,
    column_buffer: ClipboardBuffer,
}

impl DualClipboard {
    /// Create a new empty dual clipboard
    pub fn new() -> Self {
        Self {
            row_buffer: ClipboardBuffer::new(),
            column_buffer: ClipboardBuffer::new(),
        }
    }

    // ── Row buffer methods ──

    /// Store a single row in the row buffer
    pub fn yank_row(&mut self, row: Vec<String>) {
        self.row_buffer.store(vec![row]);
    }

    /// Store a single cell in the row buffer (treated as a 1-cell row)
    pub fn yank_cell(&mut self, cell: String) {
        self.row_buffer.store(vec![vec![cell]]);
    }

    /// Store a rectangular region in the row buffer
    pub fn yank_region(&mut self, region: Vec<Vec<String>>) {
        self.row_buffer.store(region);
    }

    /// Get the first row from the row buffer
    pub fn as_row(&self) -> Option<Vec<String>> {
        self.row_buffer.get().and_then(|d| d.first().cloned())
    }

    /// Get all rows from the row buffer (for region paste)
    pub fn as_region(&self) -> Option<Vec<Vec<String>>> {
        self.row_buffer.get().cloned()
    }

    /// Check if the row buffer is empty
    pub fn row_buffer_empty(&self) -> bool {
        self.row_buffer.is_empty()
    }

    // ── Column buffer methods ──

    /// Store a single column in the column buffer
    pub fn yank_column(&mut self, column: Vec<String>) {
        self.column_buffer.store(vec![column]);
    }

    /// Store multiple columns in the column buffer
    pub fn yank_columns(&mut self, columns: Vec<Vec<String>>) {
        self.column_buffer.store(columns);
    }

    /// Get the first column from the column buffer
    pub fn as_column(&self) -> Option<Vec<String>> {
        self.column_buffer.get().and_then(|d| d.first().cloned())
    }

    /// Get all columns from the column buffer
    pub fn as_columns(&self) -> Option<Vec<Vec<String>>> {
        self.column_buffer.get().cloned()
    }

    /// Check if the column buffer is empty
    pub fn column_buffer_empty(&self) -> bool {
        self.column_buffer.is_empty()
    }

    // ── General methods ──

    /// Check if both buffers are empty
    pub fn is_empty(&self) -> bool {
        self.row_buffer.is_empty() && self.column_buffer.is_empty()
    }

    /// Clear both buffers
    pub fn clear(&mut self) {
        self.row_buffer.clear();
        self.column_buffer.clear();
    }
}

impl Default for DualClipboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_clipboard_is_empty() {
        let cb = DualClipboard::new();
        assert!(cb.is_empty());
        assert!(cb.row_buffer_empty());
        assert!(cb.column_buffer_empty());
    }

    #[test]
    fn test_yank_row() {
        let mut cb = DualClipboard::new();
        cb.yank_row(vec!["A".into(), "B".into(), "C".into()]);

        assert!(!cb.row_buffer_empty());
        assert_eq!(cb.as_row(), Some(vec!["A".into(), "B".into(), "C".into()]));
        // Column buffer untouched
        assert!(cb.column_buffer_empty());
        assert_eq!(cb.as_column(), None);
    }

    #[test]
    fn test_yank_column() {
        let mut cb = DualClipboard::new();
        cb.yank_column(vec!["Header".into(), "1".into(), "2".into()]);

        assert!(!cb.column_buffer_empty());
        assert_eq!(
            cb.as_column(),
            Some(vec!["Header".into(), "1".into(), "2".into()])
        );
        // Row buffer untouched
        assert!(cb.row_buffer_empty());
        assert_eq!(cb.as_row(), None);
    }

    #[test]
    fn test_buffers_are_independent() {
        let mut cb = DualClipboard::new();
        cb.yank_row(vec!["row".into()]);
        cb.yank_column(vec!["col".into()]);

        assert_eq!(cb.as_row(), Some(vec!["row".into()]));
        assert_eq!(cb.as_column(), Some(vec!["col".into()]));
        assert!(!cb.is_empty());
    }

    #[test]
    fn test_yank_cell() {
        let mut cb = DualClipboard::new();
        cb.yank_cell("hello".into());

        assert_eq!(cb.as_row(), Some(vec!["hello".into()]));
        assert!(cb.column_buffer_empty());
    }

    #[test]
    fn test_yank_region() {
        let mut cb = DualClipboard::new();
        let region = vec![
            vec!["A1".into(), "B1".into()],
            vec!["A2".into(), "B2".into()],
        ];
        cb.yank_region(region.clone());

        assert_eq!(cb.as_region(), Some(region));
        assert!(cb.column_buffer_empty());
    }

    #[test]
    fn test_yank_columns() {
        let mut cb = DualClipboard::new();
        let cols = vec![vec!["H1".into(), "1".into()], vec!["H2".into(), "2".into()]];
        cb.yank_columns(cols.clone());

        assert_eq!(cb.as_columns(), Some(cols));
        assert!(cb.row_buffer_empty());
    }

    #[test]
    fn test_overwrite_row_buffer() {
        let mut cb = DualClipboard::new();
        cb.yank_row(vec!["first".into()]);
        cb.yank_row(vec!["second".into()]);

        assert_eq!(cb.as_row(), Some(vec!["second".into()]));
    }

    #[test]
    fn test_overwrite_column_buffer() {
        let mut cb = DualClipboard::new();
        cb.yank_column(vec!["first".into()]);
        cb.yank_column(vec!["second".into()]);

        assert_eq!(cb.as_column(), Some(vec!["second".into()]));
    }

    #[test]
    fn test_clear() {
        let mut cb = DualClipboard::new();
        cb.yank_row(vec!["row".into()]);
        cb.yank_column(vec!["col".into()]);

        cb.clear();
        assert!(cb.is_empty());
        assert!(cb.row_buffer_empty());
        assert!(cb.column_buffer_empty());
    }

    #[test]
    fn test_empty_row_buffer_returns_none() {
        let cb = DualClipboard::new();
        assert_eq!(cb.as_row(), None);
        assert_eq!(cb.as_region(), None);
    }

    #[test]
    fn test_empty_column_buffer_returns_none() {
        let cb = DualClipboard::new();
        assert_eq!(cb.as_column(), None);
        assert_eq!(cb.as_columns(), None);
    }
}
