use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazycsv::{App, ColIndex, Document, FileConfig, RowIndex};
use std::path::PathBuf;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Create test app: header + 5 data rows
/// Row 0: Name, Value (header)
/// Row 1: A, 10
/// Row 2: B, 20
/// Row 3: C, 30
/// Row 4: D, 40
/// Row 5: E, 50
/// Row 6: (empty, for formulas)
fn create_test_app() -> App {
    let doc = Document::new(
        vec!["Name".to_string(), "Value".to_string()],
        vec![
            vec!["A".to_string(), "10".to_string()],
            vec!["B".to_string(), "20".to_string()],
            vec!["C".to_string(), "30".to_string()],
            vec!["D".to_string(), "40".to_string()],
            vec!["E".to_string(), "50".to_string()],
            vec![String::new(), String::new()], // row 6 for formulas
        ],
        "test.csv".to_string(),
    );
    let files = vec![PathBuf::from("test.csv")];
    App::new(doc, files, 0, FileConfig::new())
}

// ===== Direct commit_cell_value tests (formula engine integration) =====

#[test]
fn test_formula_sum_computed_value() {
    let mut app = create_test_app();
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=SUM(B1:B5)".to_string());

    let computed = app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string();
    assert_eq!(computed, "150"); // 10+20+30+40+50
}

#[test]
fn test_formula_bar_shows_formula() {
    let mut app = create_test_app();
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=SUM(B1:B5)".to_string());

    // cell_formula_or_value returns formula text, not the computed value
    let bar_text = app.cell_formula_or_value(RowIndex::new(6), ColIndex::new(1));
    assert_eq!(bar_text, "=SUM(B1:B5)");
}

#[test]
fn test_formula_re_evaluates_on_change() {
    let mut app = create_test_app();
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=SUM(B1:B5)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "150");

    // Change B1 from 10 to 100
    app.commit_cell_value(RowIndex::new(1), ColIndex::new(1), "100".to_string());

    // Formula should auto-update
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "240");
}

#[test]
fn test_formula_overwrite_removes_formula() {
    let mut app = create_test_app();
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=SUM(B1:B5)".to_string());

    // Overwrite with plain value
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "999".to_string());

    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "999");
    // cell_formula_or_value should return plain value (formula is gone)
    assert_eq!(app.cell_formula_or_value(RowIndex::new(6), ColIndex::new(1)), "999");
}

#[test]
fn test_formula_average() {
    let mut app = create_test_app();
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=AVERAGE(B1:B5)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "30");
}

#[test]
fn test_formula_min() {
    let mut app = create_test_app();
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=MIN(B1:B5)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "10");
}

#[test]
fn test_formula_max() {
    let mut app = create_test_app();
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=MAX(B1:B5)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "50");
}

#[test]
fn test_formula_count() {
    let mut app = create_test_app();
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=COUNT(B1:B5)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "5");
}

#[test]
fn test_formula_case_insensitive() {
    let mut app = create_test_app();
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=sum(B1:B5)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "150");
}

#[test]
fn test_formula_comma_separated_cells() {
    let mut app = create_test_app();
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=SUM(B1,B3,B5)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "90"); // 10+30+50
}

#[test]
fn test_formula_power() {
    let mut app = create_test_app();
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=POWER(B1, 2)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "100"); // 10^2
}

#[test]
fn test_formula_ceiling() {
    let mut app = create_test_app();
    // B3 = 30, CEILING(30, 7) = 35
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=CEILING(B3, 7)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "35");
}

#[test]
fn test_formula_floor() {
    let mut app = create_test_app();
    // B3 = 30, FLOOR(30, 7) = 28
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=FLOOR(B3, 7)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "28");
}

#[test]
fn test_formula_concat() {
    let mut app = create_test_app();
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=CONCAT(A1, B1)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "A10");
}

#[test]
fn test_formula_concat_with_literal() {
    let mut app = create_test_app();
    app.commit_cell_value(
        RowIndex::new(6),
        ColIndex::new(1),
        "=CONCAT(A1, \" = \", B1)".to_string(),
    );
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "A = 10");
}

#[test]
fn test_formula_trim() {
    let mut app = create_test_app();
    // Set A6 to a string with extra spaces
    app.document.set_cell(RowIndex::new(6), ColIndex::new(0), "  hello   world  ".to_string());
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=TRIM(A6)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "hello world");
}

#[test]
fn test_formula_upper_lower_proper() {
    let mut app = create_test_app();
    app.document.set_cell(RowIndex::new(6), ColIndex::new(0), "hello world".to_string());

    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=UPPER(A6)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "HELLO WORLD");

    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=LOWER(A6)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "hello world");

    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=PROPER(A6)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "Hello World");
}

#[test]
fn test_formula_left_right_mid() {
    let mut app = create_test_app();
    app.document.set_cell(RowIndex::new(6), ColIndex::new(0), "Hello".to_string());

    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=LEFT(A6, 3)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "Hel");

    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=RIGHT(A6, 3)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "llo");

    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=MID(A6, 2, 3)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "ell");
}

#[test]
fn test_formula_substitute() {
    let mut app = create_test_app();
    app.document.set_cell(RowIndex::new(6), ColIndex::new(0), "Old text Old".to_string());
    app.commit_cell_value(
        RowIndex::new(6),
        ColIndex::new(1),
        "=SUBSTITUTE(A6, \"Old\", \"New\")".to_string(),
    );
    assert_eq!(
        app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(),
        "New text New"
    );
}

#[test]
fn test_formula_replace() {
    let mut app = create_test_app();
    app.document.set_cell(RowIndex::new(6), ColIndex::new(0), "Hello".to_string());
    app.commit_cell_value(
        RowIndex::new(6),
        ColIndex::new(1),
        "=REPLACE(A6, 2, 3, \"XYZ\")".to_string(),
    );
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "HXYZo");
}

#[test]
fn test_formula_today() {
    let mut app = create_test_app();
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=TODAY()".to_string());
    let result = app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string();
    // Should be YYYY-MM-DD format
    assert!(result.len() == 10, "TODAY() should return date in YYYY-MM-DD format, got: {}", result);
    assert!(result.contains('-'), "TODAY() should contain dashes");
}

#[test]
fn test_formula_if_true() {
    let mut app = create_test_app();
    // B1 = 10, so 10 > 5 is true
    app.commit_cell_value(
        RowIndex::new(6),
        ColIndex::new(1),
        "=IF(B1>5, \"High\", \"Low\")".to_string(),
    );
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "High");
}

#[test]
fn test_formula_if_false() {
    let mut app = create_test_app();
    // B1 = 10, so 10 > 100 is false
    app.commit_cell_value(
        RowIndex::new(6),
        ColIndex::new(1),
        "=IF(B1>100, \"High\", \"Low\")".to_string(),
    );
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "Low");
}

#[test]
fn test_formula_vlookup() {
    // Use rows 1-3 as a lookup table: A=Name, B=Value
    let mut app = create_test_app();
    app.commit_cell_value(
        RowIndex::new(6),
        ColIndex::new(1),
        "=VLOOKUP(\"C\", A1:B5, 2, FALSE)".to_string(),
    );
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "30");
}

// ===== Keyboard-driven test (full flow) =====

#[test]
fn test_formula_via_keyboard() {
    let mut app = create_test_app();

    // Navigate to B1 (row 1, col 1) — gg goes to first data row (row 1)
    let _ = app.handle_key(key(KeyCode::Char('g')));
    let _ = app.handle_key(key(KeyCode::Char('g')));
    let _ = app.handle_key(key(KeyCode::Char('l'))); // move to col 1

    // Verify we're at B1 which has value "10"
    assert_eq!(
        app.document
            .cell(app.selected_row().unwrap(), app.view_state.selected_column)
            .to_string(),
        "10"
    );

    // Enter substitute mode ('s' = clear + insert), type formula, commit with Enter
    let _ = app.handle_key(key(KeyCode::Char('s')));
    for c in "=SUM(B2:B5)".chars() {
        let _ = app.handle_key(key(KeyCode::Char(c)));
    }
    let _ = app.handle_key(key(KeyCode::Enter)); // commit and move down

    // Check result: SUM(B2:B5) = 20+30+40+50 = 140
    let computed = app.document.cell(RowIndex::new(1), ColIndex::new(1)).to_string();
    assert_eq!(computed, "140");

    // Check formula bar shows formula text
    let bar = app.cell_formula_or_value(RowIndex::new(1), ColIndex::new(1));
    assert_eq!(bar, "=SUM(B2:B5)");

    // Navigate back to B1 and enter insert mode — should show formula
    let _ = app.handle_key(key(KeyCode::Char('k'))); // move back up to row 1
    let _ = app.handle_key(key(KeyCode::Char('i'))); // enter insert mode
    let edit_content = app.edit_buffer.as_ref().unwrap().content.clone();
    assert_eq!(edit_content, "=SUM(B2:B5)");
}

// ===== Multiple formulas with dependencies =====

#[test]
fn test_multiple_formulas_update() {
    let mut app = create_test_app();

    // Put SUM in B6
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(1), "=SUM(B1:B5)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "150");

    // Put AVERAGE in A6
    app.commit_cell_value(RowIndex::new(6), ColIndex::new(0), "=AVERAGE(B1:B5)".to_string());
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(0)).to_string(), "30");

    // Change B2 from 20 to 120
    app.commit_cell_value(RowIndex::new(2), ColIndex::new(1), "120".to_string());

    // Both formulas should update
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "250"); // 10+120+30+40+50
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(0)).to_string(), "50");  // 250/5
}

// ===== Formula completion popup =====

#[test]
fn test_formula_completion_opens_on_equals() {
    let mut app = create_test_app();
    goto(&mut app, 0, 1);

    // Enter substitute mode and type '='
    let _ = app.handle_key(key(KeyCode::Char('s')));
    let _ = app.handle_key(key(KeyCode::Char('=')));

    // Formula completion popup should be open
    assert!(app.formula_completion.is_some(), "Completion popup should open after typing '='");
    let comp = app.formula_completion.as_ref().unwrap();
    assert!(!comp.filtered_items().is_empty(), "Should have formula items");
}

#[test]
fn test_formula_completion_filters_as_you_type() {
    let mut app = create_test_app();
    goto(&mut app, 0, 1);

    let _ = app.handle_key(key(KeyCode::Char('s')));
    let _ = app.handle_key(key(KeyCode::Char('=')));

    // Type 'S' to filter
    let _ = app.handle_key(key(KeyCode::Char('S')));
    let comp = app.formula_completion.as_ref().unwrap();
    let filtered = comp.filtered_items();
    // Should contain SUM, SUBSTITUTE but not AVERAGE, MIN, etc.
    assert!(filtered.iter().any(|i| i.text == "SUM"));
    assert!(filtered.iter().any(|i| i.text == "SUBSTITUTE"));
    assert!(!filtered.iter().any(|i| i.text == "AVERAGE"));

    // Type 'U' to narrow further
    let _ = app.handle_key(key(KeyCode::Char('U')));
    let comp = app.formula_completion.as_ref().unwrap();
    let filtered = comp.filtered_items();
    assert!(filtered.iter().any(|i| i.text == "SUM"));
    assert!(filtered.iter().any(|i| i.text == "SUBSTITUTE"));
}

#[test]
fn test_formula_completion_accept_inserts_function() {
    let mut app = create_test_app();
    goto(&mut app, 0, 1);

    let _ = app.handle_key(key(KeyCode::Char('s')));
    let _ = app.handle_key(key(KeyCode::Char('=')));

    // Type 'SUM' to filter to SUM
    let _ = app.handle_key(key(KeyCode::Char('S')));
    let _ = app.handle_key(key(KeyCode::Char('U')));
    let _ = app.handle_key(key(KeyCode::Char('M')));

    // Accept with Tab
    let _ = app.handle_key(key(KeyCode::Tab));

    // Popup should be closed
    assert!(app.formula_completion.is_none());

    // Edit buffer should contain "=SUM("
    let content = app.edit_buffer.as_ref().unwrap().content.clone();
    assert_eq!(content, "=SUM(");
}

#[test]
fn test_formula_completion_dismiss_on_esc() {
    let mut app = create_test_app();
    goto(&mut app, 0, 1);

    let _ = app.handle_key(key(KeyCode::Char('s')));
    let _ = app.handle_key(key(KeyCode::Char('=')));
    assert!(app.formula_completion.is_some());

    // Press Esc to dismiss popup (but stay in insert mode with "=" in buffer)
    let _ = app.handle_key(key(KeyCode::Esc));
    assert!(app.formula_completion.is_none());
    // Edit buffer should still have "="
    assert!(app.edit_buffer.is_some());
    assert_eq!(app.edit_buffer.as_ref().unwrap().content, "=");
}

#[test]
fn test_formula_completion_dismiss_on_open_paren() {
    let mut app = create_test_app();
    goto(&mut app, 0, 1);

    let _ = app.handle_key(key(KeyCode::Char('s')));
    let _ = app.handle_key(key(KeyCode::Char('=')));
    let _ = app.handle_key(key(KeyCode::Char('S')));
    let _ = app.handle_key(key(KeyCode::Char('U')));
    let _ = app.handle_key(key(KeyCode::Char('M')));
    // Type '(' to dismiss popup and continue manually
    let _ = app.handle_key(key(KeyCode::Char('(')));
    assert!(app.formula_completion.is_none());
    assert_eq!(app.edit_buffer.as_ref().unwrap().content, "=SUM(");
}

/// Navigate to a specific data row/col (0-indexed from first data row)
fn goto(app: &mut App, row: usize, col: usize) {
    let _ = app.handle_key(key(KeyCode::Char('g')));
    let _ = app.handle_key(key(KeyCode::Char('g')));
    for _ in 0..row {
        let _ = app.handle_key(key(KeyCode::Char('j')));
    }
    let _ = app.handle_key(key(KeyCode::Char('g')));
    let _ = app.handle_key(key(KeyCode::Char('h')));
    for _ in 0..col {
        let _ = app.handle_key(key(KeyCode::Char('l')));
    }
}

#[test]
fn test_formula_datedif() {
    let mut app = create_test_app();
    app.document.set_cell(RowIndex::new(6), ColIndex::new(0), "2024-01-01".to_string());
    app.document.set_cell(RowIndex::new(1), ColIndex::new(0), "2024-03-15".to_string());
    app.commit_cell_value(
        RowIndex::new(6),
        ColIndex::new(1),
        "=DATEDIF(A6, A1, \"d\")".to_string(),
    );
    assert_eq!(app.document.cell(RowIndex::new(6), ColIndex::new(1)).to_string(), "74");
}
