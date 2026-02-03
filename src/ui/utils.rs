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

/// Convert Excel column letter(s) to 0-based index
/// "A" -> 0, "B" -> 1, "Z" -> 25, "AA" -> 26, "BC" -> 54
pub fn excel_letter_to_column(letters: &str) -> Result<usize, String> {
    if letters.is_empty() {
        return Err("Empty column name".to_string());
    }

    let letters_upper = letters.to_uppercase();
    if !letters_upper.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(format!("Invalid column: {}", letters));
    }

    let mut result = 0;
    for ch in letters_upper.chars() {
        let digit = (ch as usize) - ('A' as usize) + 1;
        result = result * 26 + digit;
    }

    Ok(result - 1) // Convert to 0-based
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

    #[test]
    fn test_excel_letter_to_column() {
        assert_eq!(excel_letter_to_column("A").unwrap(), 0);
        assert_eq!(excel_letter_to_column("B").unwrap(), 1);
        assert_eq!(excel_letter_to_column("Z").unwrap(), 25);
        assert_eq!(excel_letter_to_column("AA").unwrap(), 26);
        assert_eq!(excel_letter_to_column("AB").unwrap(), 27);
        assert_eq!(excel_letter_to_column("AZ").unwrap(), 51);
        assert_eq!(excel_letter_to_column("BA").unwrap(), 52);
        assert_eq!(excel_letter_to_column("BC").unwrap(), 54);
        assert_eq!(excel_letter_to_column("ZZ").unwrap(), 701);
    }

    #[test]
    fn test_excel_letter_to_column_case_insensitive() {
        assert_eq!(excel_letter_to_column("a").unwrap(), 0);
        assert_eq!(excel_letter_to_column("b").unwrap(), 1);
        assert_eq!(excel_letter_to_column("aa").unwrap(), 26);
        assert_eq!(excel_letter_to_column("Bc").unwrap(), 54);
    }

    #[test]
    fn test_excel_letter_to_column_invalid() {
        assert!(excel_letter_to_column("").is_err());
        assert!(excel_letter_to_column("123").is_err());
        assert!(excel_letter_to_column("A1").is_err());
        assert!(excel_letter_to_column("1A").is_err());
        assert!(excel_letter_to_column("A B").is_err());
    }

    #[test]
    fn test_roundtrip_conversion() {
        // Test that converting to letter and back gives the same result
        for i in 0..100 {
            let letter = column_to_excel_letter(i);
            let index = excel_letter_to_column(&letter).unwrap();
            assert_eq!(index, i, "Roundtrip failed for index {}", i);
        }
    }

    #[test]
    fn test_extended_roundtrip_conversion() {
        // Test more extensive range including 3-letter columns
        for i in 0..1000 {
            let letter = column_to_excel_letter(i);
            let index = excel_letter_to_column(&letter).unwrap();
            assert_eq!(index, i, "Roundtrip failed for index {}", i);
        }
    }

    #[test]
    fn test_three_letter_columns() {
        // Test 3-letter column names
        assert_eq!(column_to_excel_letter(702), "AAA"); // First 3-letter column
        assert_eq!(excel_letter_to_column("AAA").unwrap(), 702);
        assert_eq!(excel_letter_to_column("ABC").unwrap(), 730);
        assert_eq!(excel_letter_to_column("ZZZ").unwrap(), 18277);
    }

    #[test]
    fn test_column_letter_boundary_cases() {
        // Test boundary cases between 1, 2, and 3 letter columns
        assert_eq!(column_to_excel_letter(25), "Z"); // Last 1-letter
        assert_eq!(column_to_excel_letter(26), "AA"); // First 2-letter
        assert_eq!(column_to_excel_letter(701), "ZZ"); // Last 2-letter
        assert_eq!(column_to_excel_letter(702), "AAA"); // First 3-letter
    }

    #[test]
    fn test_column_letter_mixed_case_conversion() {
        // Test various mixed case inputs
        assert_eq!(excel_letter_to_column("Ab").unwrap(), 27);
        assert_eq!(excel_letter_to_column("aB").unwrap(), 27);
        assert_eq!(excel_letter_to_column("AB").unwrap(), 27);
        assert_eq!(excel_letter_to_column("ab").unwrap(), 27);
    }
}
