//! UI utility functions for table rendering.
//!
//! Helper functions for column letter conversion (A, B, C... AA, AB)
//! and other table display utilities.

use std::borrow::Cow;

const SINGLE_LETTER_COLS: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

/// Convert column index to letter (0 -> A, 1 -> B, ..., 26 -> AA, etc.)
pub fn column_to_excel_letter(index: usize) -> Cow<'static, str> {
    if let Some(s) = SINGLE_LETTER_COLS.get(index) {
        return Cow::Borrowed(s);
    }

    let mut result = String::new();
    let mut num = index + 1; // 1-based

    while num > 0 {
        let remainder = (num - 1) % 26;
        result.insert(0, (b'A' + remainder as u8) as char);
        num = (num - 1) / 26;
    }

    Cow::Owned(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_to_excel_letter() {
        assert_eq!(column_to_excel_letter(0), "A");
        assert_eq!(column_to_excel_letter(1), "B");
        assert_eq!(column_to_excel_letter(25), "Z");
        assert_eq!(column_to_excel_letter(26), "AA");
        assert_eq!(column_to_excel_letter(27), "AB");
        assert_eq!(column_to_excel_letter(51), "AZ");
        assert_eq!(column_to_excel_letter(52), "BA");
    }
}
