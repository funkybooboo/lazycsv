use lazycsv::ui::column_index_to_letter;

#[test]
fn test_column_index_to_letter() {
    assert_eq!(column_index_to_letter(0), "A");
    assert_eq!(column_index_to_letter(1), "B");
    assert_eq!(column_index_to_letter(25), "Z");
    assert_eq!(column_index_to_letter(26), "AA");
    assert_eq!(column_index_to_letter(27), "AB");
    assert_eq!(column_index_to_letter(51), "AZ");
    assert_eq!(column_index_to_letter(52), "BA");
}
