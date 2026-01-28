/// Convert column index to letter (0 -> A, 1 -> B, ..., 26 -> AA, etc.)
pub fn column_index_to_letter(index: usize) -> String {
    let mut result = String::new();
    let mut num = index + 1; // 1-based

    while num > 0 {
        let remainder = (num - 1) % 26;
        result.insert(0, (b'A' + remainder as u8) as char);
        num = (num - 1) / 26;
    }

    result
}
