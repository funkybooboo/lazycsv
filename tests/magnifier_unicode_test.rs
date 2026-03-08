//! Magnifier Unicode and emoji handling tests
//!
//! Tests magnifier behavior with multi-byte characters, emojis, combining chars, etc.

use lazycsv::{
    domain::position::{ColIndex, RowIndex},
    magnifier::MagnifierState,
};

#[test]
fn test_magnifier_emoji_content() {
    let content = "Hello 👋 World 🌍!";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mag = MagnifierState::new(content.to_string(), position);

    assert_eq!(mag.lines().len(), 1);
    assert!(mag.lines()[0].contains("👋"));
    assert!(mag.lines()[0].contains("🌍"));
}

#[test]
fn test_magnifier_emoji_navigation() {
    let content = "Before👋After";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    // Start at beginning
    assert_eq!(mag.cursor(), (0, 0));

    // Move right through emoji
    mag.move_right();
    mag.move_right();
    mag.move_right();
    mag.move_right();
    mag.move_right();
    mag.move_right();

    // Should be past "Before"
    assert!(mag.cursor().1 >= 6);
}

#[test]
fn test_magnifier_emoji_deletion() {
    let content = "👋🌍🎉";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    mag.push_undo();
    mag.delete_char();

    // Should have deleted first emoji
    assert!(mag.lines()[0].contains("🌍"));

    // Undo should restore
    mag.undo();
    assert!(mag.lines()[0].contains("👋"));
}

#[test]
fn test_magnifier_japanese_text() {
    let content = "こんにちは世界\nさようなら";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mag = MagnifierState::new(content.to_string(), position);

    assert_eq!(mag.lines().len(), 2);
    assert_eq!(mag.lines()[0], "こんにちは世界");
    assert_eq!(mag.lines()[1], "さようなら");
}

#[test]
fn test_magnifier_arabic_text() {
    let content = "مرحبا بالعالم";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mag = MagnifierState::new(content.to_string(), position);

    assert_eq!(mag.lines().len(), 1);
    assert!(mag.lines()[0].contains("مرحبا"));
}

#[test]
fn test_magnifier_mixed_scripts() {
    let content = "Hello こんにちは مرحبا 🌍";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mag = MagnifierState::new(content.to_string(), position);

    assert_eq!(mag.lines().len(), 1);
    assert!(mag.lines()[0].contains("Hello"));
    assert!(mag.lines()[0].contains("こんにちは"));
    assert!(mag.lines()[0].contains("مرحبا"));
    assert!(mag.lines()[0].contains("🌍"));
}

#[test]
fn test_magnifier_accented_characters() {
    let content = "Café résumé naïve";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mag = MagnifierState::new(content.to_string(), position);

    assert_eq!(mag.lines()[0], "Café résumé naïve");
}

#[test]
fn test_magnifier_combining_characters() {
    // "e" + combining acute accent
    let content = "e\u{0301}";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mag = MagnifierState::new(content.to_string(), position);

    assert_eq!(mag.lines().len(), 1);
    // The combining character should be part of the content
    assert!(mag.lines()[0].len() > 1);
}

#[test]
fn test_magnifier_zero_width_characters() {
    // Zero-width space (U+200B)
    let content = "word1\u{200B}word2";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mag = MagnifierState::new(content.to_string(), position);

    assert_eq!(mag.lines().len(), 1);
    assert!(mag.lines()[0].contains("word1"));
    assert!(mag.lines()[0].contains("word2"));
}

#[test]
fn test_magnifier_emoji_search() {
    let content = "Start wave middle earth end";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    // Search for word
    mag.search_forward("wave".to_string());

    let matches = mag.search_matches();
    assert_eq!(matches.len(), 1);

    mag.jump_to_next_match();
    // Cursor should be at the match
    assert!(mag.cursor().1 > 0);
}

// Fixed in v0.6.1: Emoji search now works correctly with char-based indexing
#[test]
fn test_magnifier_emoji_search_actual() {
    let content = "Start 👋 middle 🌍 end";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    // Search for emoji - should not crash
    mag.search_forward("👋".to_string());

    // Should find the match
    let matches = mag.search_matches();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].0, 0); // First line

    // Try another emoji search
    mag.search_forward("🌍".to_string());
    let matches = mag.search_matches();
    assert_eq!(matches.len(), 1);
}

#[test]
fn test_magnifier_unicode_word_motion() {
    let content = "Hello 世界 test こんにちは end";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content.to_string(), position);

    // Move through words
    mag.move_next_word();
    mag.move_next_word();

    // Should have moved past at least "Hello"
    assert!(mag.cursor().1 > 5);
}

#[test]
fn test_magnifier_rtl_text_basic() {
    // Right-to-left text (Arabic/Hebrew)
    let content = "العربية עברית";
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mag = MagnifierState::new(content.to_string(), position);

    assert_eq!(mag.lines().len(), 1);
    assert!(!mag.lines()[0].is_empty());
}
