//! Edge case tests for search functionality
//!
//! Comprehensive test coverage for boundary conditions, unicode handling,
//! regex edge cases, and performance stress tests.

use lazycsv::csv::Document;
use lazycsv::search::{find_matches, SearchState};
use lazycsv::{ColIndex, RowIndex};

fn make_doc(rows: Vec<Vec<&str>>) -> Document {
    let string_rows: Vec<Vec<String>> = rows
        .into_iter()
        .map(|r| r.into_iter().map(|s| s.to_string()).collect())
        .collect();
    let headers = string_rows[0].clone();
    let data = string_rows[1..].to_vec();
    Document::new(headers, data, "test.csv".to_string())
}

// ============================================================================
// Empty and Boundary Cases
// ============================================================================

#[test]
fn test_search_empty_pattern() {
    let doc = make_doc(vec![vec!["Name", "City"], vec!["Alice", "Portland"]]);
    let matches = find_matches(&doc, "");
    // Empty pattern should match nothing (or everything - depends on impl)
    // Current impl: regex "" matches everything at every position
    // This is acceptable behavior
    assert!(!matches.is_empty());
}

#[test]
fn test_search_empty_document() {
    let doc = Document::new(vec![], vec![], "empty.csv".to_string());
    let matches = find_matches(&doc, "test");
    assert_eq!(matches.len(), 0);
}

#[test]
fn test_search_single_cell_document() {
    let headers = vec!["Col".to_string()];
    let data = vec![];
    let doc = Document::new(headers, data, "single.csv".to_string());

    // Search in header
    let matches = find_matches(&doc, "Col");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0], (RowIndex::new(0), ColIndex::new(0)));
}

#[test]
fn test_search_no_matches() {
    let doc = make_doc(vec![
        vec!["Name", "City"],
        vec!["Alice", "Portland"],
        vec!["Bob", "Boston"],
    ]);
    let matches = find_matches(&doc, "xyz_not_found_123");
    assert_eq!(matches.len(), 0);
}

#[test]
fn test_search_all_cells_match() {
    let doc = make_doc(vec![
        vec!["test", "test"],
        vec!["test", "test"],
        vec!["test", "test"],
    ]);
    let matches = find_matches(&doc, "test");
    assert_eq!(matches.len(), 6); // All 6 cells match
}

// ============================================================================
// Regex Edge Cases
// ============================================================================

#[test]
fn test_search_invalid_regex_fallback() {
    let doc = make_doc(vec![
        vec!["Name", "Value"],
        vec!["test[", "other"],
        vec!["normal", "bracket["],
    ]);
    // "[" is invalid regex — should fall back to literal substring match
    let matches = find_matches(&doc, "[");
    assert_eq!(matches.len(), 2);
}

#[test]
fn test_search_very_long_regex() {
    let doc = make_doc(vec![vec!["Name", "Value"], vec!["A123456789", "other"]]);
    // Very long regex pattern
    let pattern = r"^A\d{9}$";
    let matches = find_matches(&doc, pattern);
    assert_eq!(matches.len(), 1);
}

#[test]
fn test_search_special_regex_chars() {
    let doc = make_doc(vec![
        vec!["Name", "Value"],
        vec!["$100", "10%"],
        vec!["(test)", "[item]"],
    ]);

    // Escape special regex chars - should fallback to literal
    let matches = find_matches(&doc, "$");
    assert!(!matches.is_empty()); // At least one match

    let matches = find_matches(&doc, "%");
    assert!(!matches.is_empty());

    let matches = find_matches(&doc, "(");
    assert!(!matches.is_empty());
}

#[test]
fn test_search_unicode_in_regex() {
    let doc = make_doc(vec![
        vec!["Name", "City"],
        vec!["Alice", "東京"],
        vec!["Bob", "北京"],
    ]);
    let matches = find_matches(&doc, "東京");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0], (RowIndex::new(1), ColIndex::new(1)));
}

#[test]
fn test_search_regex_anchors() {
    let doc = make_doc(vec![
        vec!["Name", "Value"],
        vec!["test", "testing"],
        vec!["contest", "test"],
    ]);

    // ^test$ should match only cells that are exactly "test"
    let matches = find_matches(&doc, "^test$");
    assert_eq!(matches.len(), 2); // "test" appears exactly twice
}

// ============================================================================
// Unicode and Special Characters
// ============================================================================

#[test]
fn test_search_emoji() {
    let doc = make_doc(vec![
        vec!["Name", "Status"],
        vec!["Alice", "✅ Done"],
        vec!["Bob", "❌ Failed"],
        vec!["Charlie", "✅ Done"],
    ]);
    let matches = find_matches(&doc, "✅");
    assert_eq!(matches.len(), 2);
}

#[test]
fn test_search_mixed_unicode() {
    let doc = make_doc(vec![
        vec!["Name", "Description"],
        vec!["Tokyo", "東京 is the capital"],
        vec!["Beijing", "北京 is large"],
    ]);
    let matches = find_matches(&doc, "東京");
    assert_eq!(matches.len(), 1);

    let matches = find_matches(&doc, "is");
    assert_eq!(matches.len(), 2);
}

#[test]
fn test_search_accented_characters() {
    let doc = make_doc(vec![
        vec!["Name", "City"],
        vec!["José", "São Paulo"],
        vec!["François", "Montréal"],
    ]);
    let matches = find_matches(&doc, "José");
    assert_eq!(matches.len(), 1);

    let matches = find_matches(&doc, "São");
    assert_eq!(matches.len(), 1);
}

// ============================================================================
// Performance and Stress Tests
// ============================================================================

#[test]
fn test_search_very_long_cell_content() {
    let long_content = "a".repeat(10000);
    let headers = vec!["Col".to_string()];
    let data = vec![vec![long_content.clone()]];
    let doc = Document::new(headers, data, "long.csv".to_string());

    let matches = find_matches(&doc, "a");
    assert_eq!(matches.len(), 1); // Matches the cell with 10K a's
}

#[test]
fn test_search_many_small_matches() {
    // Create document with many small cells that match
    let headers: Vec<String> = (0..10).map(|i| format!("Col{}", i)).collect();
    let data: Vec<Vec<String>> = (0..1000)
        .map(|_| (0..10).map(|_| "match".to_string()).collect())
        .collect();
    let doc = Document::new(headers, data, "many.csv".to_string());

    let matches = find_matches(&doc, "match");
    assert_eq!(matches.len(), 10_000); // 1000 rows × 10 cols
}

// ============================================================================
// Navigation Edge Cases
// ============================================================================

#[test]
fn test_jump_with_single_match() {
    let matches = vec![(RowIndex::new(5), ColIndex::new(3))];
    let mut state = SearchState::new("test".to_string(), matches);

    // From before the match - should jump to it
    let result = state.jump_to_next(RowIndex::new(0), ColIndex::new(0));
    assert_eq!(result, Some(((RowIndex::new(5), ColIndex::new(3)), false)));

    // From after the match - should wrap around
    let result = state.jump_to_next(RowIndex::new(10), ColIndex::new(0));
    assert_eq!(result, Some(((RowIndex::new(5), ColIndex::new(3)), true)));
}

#[test]
fn test_jump_from_exact_match_position() {
    let matches = vec![
        (RowIndex::new(1), ColIndex::new(0)),
        (RowIndex::new(3), ColIndex::new(0)),
        (RowIndex::new(5), ColIndex::new(0)),
    ];
    let mut state = SearchState::new("test".to_string(), matches);

    // Jumping from exact match position should go to next match
    let result = state.jump_to_next(RowIndex::new(1), ColIndex::new(0));
    assert_eq!(result, Some(((RowIndex::new(3), ColIndex::new(0)), false)));
}

#[test]
fn test_jump_prev_from_first_match() {
    let matches = vec![
        (RowIndex::new(1), ColIndex::new(0)),
        (RowIndex::new(3), ColIndex::new(0)),
    ];
    let mut state = SearchState::new("test".to_string(), matches);

    // Jumping prev from first match should wrap to last
    let result = state.jump_to_prev(RowIndex::new(1), ColIndex::new(0));
    assert_eq!(result, Some(((RowIndex::new(3), ColIndex::new(0)), true)));
}

#[test]
fn test_jump_with_no_matches() {
    let mut state = SearchState::new("test".to_string(), vec![]);

    let result = state.jump_to_next(RowIndex::new(0), ColIndex::new(0));
    assert_eq!(result, None);

    let result = state.jump_to_prev(RowIndex::new(0), ColIndex::new(0));
    assert_eq!(result, None);
}

// ============================================================================
// Match Detection and Display
// ============================================================================

#[test]
fn test_is_match_boundary_conditions() {
    let matches = vec![
        (RowIndex::new(0), ColIndex::new(0)),
        (RowIndex::new(999), ColIndex::new(99)),
    ];
    let state = SearchState::new("test".to_string(), matches);

    // First cell
    assert!(state.is_match(RowIndex::new(0), ColIndex::new(0)));
    // Last cell
    assert!(state.is_match(RowIndex::new(999), ColIndex::new(99)));
    // Non-match
    assert!(!state.is_match(RowIndex::new(5), ColIndex::new(5)));
}

#[test]
fn test_display_position_with_many_matches() {
    let matches: Vec<_> = (0..1000)
        .map(|i| (RowIndex::new(i), ColIndex::new(0)))
        .collect();
    let mut state = SearchState::new("test".to_string(), matches);

    assert_eq!(state.display_position(), "[0/1000]");

    // Jump from before first match - should go to row 0 (first match)
    state.jump_to_next(RowIndex::new(0), ColIndex::new(0));
    // But row 0 col 0 is already a match, so it finds the NEXT one at row 1
    assert_eq!(state.display_position(), "[2/1000]");

    state.jump_to_next(RowIndex::new(500), ColIndex::new(0));
    assert_eq!(state.display_position(), "[502/1000]");
}

#[test]
fn test_match_count_accuracy() {
    let doc = make_doc(vec![
        vec!["A", "B", "C"],
        vec!["test", "other", "test"],
        vec!["test", "test", "other"],
    ]);
    let matches = find_matches(&doc, "test");
    let state = SearchState::new("test".to_string(), matches);

    assert_eq!(state.match_count(), 4); // 4 cells contain "test"
}
