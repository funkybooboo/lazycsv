//! Magnifier large text handling edge case tests
//!
//! Tests magnifier behavior with very long lines, many lines, and combinations.

use lazycsv::{
    domain::position::{ColIndex, RowIndex},
    magnifier::MagnifierState,
};

#[test]
fn test_magnifier_very_long_line_1000_chars() {
    // Create content with a single 1000-character line
    let long_line = "a".repeat(1000);
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mag = MagnifierState::new(long_line.clone(), position);

    assert_eq!(mag.lines().len(), 1);
    assert_eq!(mag.lines()[0].len(), 1000);
    assert_eq!(mag.cursor(), (0, 0));
}

#[test]
fn test_magnifier_navigate_long_line() {
    let long_line = "x".repeat(500);
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(long_line, position);

    // Move to end with $
    mag.move_to_line_end();
    assert_eq!(mag.cursor(), (0, 499)); // 0-indexed, so last char is at 499

    // Move back to start with 0
    mag.move_to_line_start();
    assert_eq!(mag.cursor(), (0, 0));
}

#[test]
fn test_magnifier_many_lines_10000() {
    // Create 10,000 lines
    let lines: Vec<String> = (1..=10000).map(|i| format!("Line {}", i)).collect();
    let content = lines.join("\n");
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mag = MagnifierState::new(content, position);

    assert_eq!(mag.lines().len(), 10000);
    assert_eq!(mag.lines()[0], "Line 1");
    assert_eq!(mag.lines()[9999], "Line 10000");
}

#[test]
fn test_magnifier_navigate_many_lines() {
    let lines: Vec<String> = (1..=1000).map(|i| format!("Line {}", i)).collect();
    let content = lines.join("\n");
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content, position);

    // Jump to last line with G
    mag.move_to_last_line();
    assert_eq!(mag.cursor().0, 999);

    // Jump to first line with gg
    mag.move_to_first_line();
    assert_eq!(mag.cursor().0, 0);

    // Navigate with count prefix (would need to set count in real usage)
    mag.move_down(); // Move down a few times
    mag.move_down();
    mag.move_down();
    assert_eq!(mag.cursor().0, 3);
}

#[test]
fn test_magnifier_many_long_lines() {
    // Create 1000 lines, each 200 chars
    let lines: Vec<String> = (1..=1000)
        .map(|i| format!("Line {:05} {}", i, "x".repeat(190)))
        .collect();
    let content = lines.join("\n");
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mag = MagnifierState::new(content, position);

    assert_eq!(mag.lines().len(), 1000);
    assert!(mag.lines()[0].len() >= 200);
    assert!(mag.lines()[999].len() >= 200);
}

#[test]
fn test_magnifier_delete_in_long_line() {
    let long_line = "a".repeat(100);
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(long_line, position);

    // Delete character at start
    mag.push_undo();
    mag.delete_char();

    assert_eq!(mag.lines()[0].len(), 99);

    // Undo
    mag.undo();
    assert_eq!(mag.lines()[0].len(), 100);
}

#[test]
fn test_magnifier_search_in_large_document() {
    // Create 500 lines with "target" word scattered throughout
    let lines: Vec<String> = (1..=500)
        .map(|i| {
            if i % 50 == 0 {
                format!("Line {} contains target word", i)
            } else {
                format!("Line {} normal text", i)
            }
        })
        .collect();
    let content = lines.join("\n");
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content, position);

    // Search for "target"
    mag.search_forward("target".to_string());

    let matches = mag.search_matches();
    assert_eq!(matches.len(), 10); // Should find 10 matches (500/50)

    // Jump to next match
    mag.jump_to_next_match();
    let cursor = mag.cursor();
    assert!(mag.lines()[cursor.0].contains("target"));
}

#[test]
fn test_magnifier_undo_redo_large_operations() {
    let lines: Vec<String> = (1..=100).map(|i| format!("Line {}", i)).collect();
    let content = lines.join("\n");
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content, position);

    // Perform multiple delete operations
    for _ in 0..10 {
        mag.push_undo();
        mag.delete_line();
    }

    assert_eq!(mag.lines().len(), 90); // 100 - 10

    // Undo all operations
    for _ in 0..10 {
        mag.undo();
    }

    assert_eq!(mag.lines().len(), 100);

    // Redo all operations
    for _ in 0..10 {
        mag.redo();
    }

    assert_eq!(mag.lines().len(), 90);
}

#[test]
fn test_magnifier_word_motion_long_line() {
    let long_line = "word1 word2 word3 ".repeat(50); // 150 words
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(long_line, position);

    // Move through words
    mag.move_next_word();
    assert!(mag.cursor().1 > 0);

    mag.move_next_word();
    assert!(mag.cursor().1 > 5);

    // Move backwards
    mag.move_prev_word();
    mag.move_prev_word();
    assert_eq!(mag.cursor().1, 0);
}

#[test]
fn test_magnifier_join_many_lines() {
    let lines = ["line1", "line2", "line3", "line4", "line5"];
    let content = lines.join("\n");
    let position = (RowIndex::new(1), ColIndex::new(0));
    let mut mag = MagnifierState::new(content, position);

    assert_eq!(mag.lines().len(), 5);

    // Join first two lines
    mag.join_lines();
    assert_eq!(mag.lines().len(), 4);
    assert!(mag.lines()[0].contains("line1"));
    assert!(mag.lines()[0].contains("line2"));
}
