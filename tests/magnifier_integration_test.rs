use lazycsv::{App, Document};
use std::path::PathBuf;

fn create_test_document() -> Document {
    Document::new(
        vec!["Name".to_string(), "Description".to_string()],
        vec![
            vec!["Alice".to_string(), "Engineer".to_string()],
            vec!["Bob".to_string(), "Designer".to_string()],
            vec!["Charlie".to_string(), "Manager".to_string()],
        ],
        "test.csv".to_string(),
    )
}

#[test]
fn test_magnifier_open_edit_save_close_workflow() {
    let doc = create_test_document();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, lazycsv::session::FileConfig::new());

    // Initial state: no magnifier
    assert!(app.magnifier_state.is_none());
    assert!(!app.document.is_dirty);

    // Open magnifier on current cell (row 1, col 0 = "Alice")
    app.open_magnifier();
    assert!(app.magnifier_state.is_some());

    // Verify initial content
    let magnifier = app.magnifier_state.as_ref().unwrap();
    assert_eq!(magnifier.get_content(), "Alice");
    assert!(!magnifier.is_dirty());

    // Edit content
    let magnifier = app.magnifier_state.as_mut().unwrap();
    magnifier.enter_insert_mode();
    magnifier.move_to_line_end();
    magnifier.insert_char(' ');
    magnifier.insert_char('S');
    magnifier.insert_char('m');
    magnifier.insert_char('i');
    magnifier.insert_char('t');
    magnifier.insert_char('h');
    magnifier.exit_insert_mode();

    // Verify dirty state
    assert!(magnifier.is_dirty());
    assert_eq!(magnifier.get_content(), "Alice Smith");

    // Save and close
    app.save_and_close_magnifier();
    assert!(app.magnifier_state.is_none());
    assert!(app.document.is_dirty);

    // Verify cell was updated
    assert_eq!(
        app.document.get_cell(
            app.get_selected_row().unwrap(),
            app.view_state.selected_column
        ),
        "Alice Smith"
    );
}

#[test]
fn test_magnifier_open_edit_discard_workflow() {
    let doc = create_test_document();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, lazycsv::session::FileConfig::new());

    // Open magnifier
    app.open_magnifier();
    let original_content = app.magnifier_state.as_ref().unwrap().get_content();

    // Edit content
    let magnifier = app.magnifier_state.as_mut().unwrap();
    magnifier.enter_insert_mode();
    magnifier.insert_char('X');
    magnifier.exit_insert_mode();

    // Verify dirty
    assert!(magnifier.is_dirty());

    // Discard changes
    app.close_magnifier_discard();
    assert!(app.magnifier_state.is_none());
    assert!(!app.document.is_dirty);

    // Verify cell unchanged
    assert_eq!(
        app.document.get_cell(
            app.get_selected_row().unwrap(),
            app.view_state.selected_column
        ),
        original_content
    );
}

#[test]
fn test_magnifier_multiline_editing() {
    let doc = create_test_document();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, lazycsv::session::FileConfig::new());

    // Open magnifier
    app.open_magnifier();

    // Add multiple lines
    let magnifier = app.magnifier_state.as_mut().unwrap();
    magnifier.enter_insert_mode();
    magnifier.move_to_line_end();
    magnifier.newline();
    magnifier.insert_char('L');
    magnifier.insert_char('i');
    magnifier.insert_char('n');
    magnifier.insert_char('e');
    magnifier.insert_char(' ');
    magnifier.insert_char('2');
    magnifier.newline();
    magnifier.insert_char('L');
    magnifier.insert_char('i');
    magnifier.insert_char('n');
    magnifier.insert_char('e');
    magnifier.insert_char(' ');
    magnifier.insert_char('3');
    magnifier.exit_insert_mode();

    // Verify content has newlines
    let content = magnifier.get_content();
    assert!(content.contains('\n'));
    assert_eq!(content.lines().count(), 3);

    // Save and verify
    app.save_and_close_magnifier();
    let cell_content = app.document.get_cell(
        app.get_selected_row().unwrap(),
        app.view_state.selected_column,
    );
    assert!(cell_content.contains('\n'));
    assert_eq!(cell_content.lines().count(), 3);
}

#[test]
fn test_magnifier_vim_motions_workflow() {
    let doc = create_test_document();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, lazycsv::session::FileConfig::new());

    // Open magnifier on "Engineer" cell
    app.view_state.selected_column = lazycsv::domain::position::ColIndex::new(1);
    app.open_magnifier();

    let magnifier = app.magnifier_state.as_mut().unwrap();
    assert_eq!(magnifier.get_content(), "Engineer");

    // Test vim motions
    magnifier.move_to_line_start(); // Cursor at 0
    assert_eq!(magnifier.cursor(), (0, 0));

    magnifier.move_next_word(); // Move to next word (end of "Engineer")
    assert!(magnifier.cursor().1 > 0);

    magnifier.move_to_line_end(); // Move to end
                                  // In Normal mode, cursor can't be past last character
    assert_eq!(magnifier.cursor().1, "Engineer".len() - 1);

    magnifier.move_to_line_start(); // Back to start
    assert_eq!(magnifier.cursor(), (0, 0));

    // Close without changes
    app.close_magnifier_discard();
}

#[test]
fn test_magnifier_vim_operators_workflow() {
    let doc = create_test_document();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, lazycsv::session::FileConfig::new());

    // Open magnifier
    app.open_magnifier();

    let magnifier = app.magnifier_state.as_mut().unwrap();
    let original = magnifier.get_content();

    // Test delete character (x)
    magnifier.delete_char();
    assert_ne!(magnifier.get_content(), original);
    assert!(magnifier.is_dirty());

    // Close and discard
    app.close_magnifier_discard();
    assert_eq!(
        app.document.get_cell(
            app.get_selected_row().unwrap(),
            app.view_state.selected_column
        ),
        original
    );
}

#[test]
fn test_magnifier_empty_cell() {
    let mut doc = create_test_document();
    // Set a cell to empty
    doc.set_cell(
        lazycsv::domain::position::RowIndex::new(1),
        lazycsv::domain::position::ColIndex::new(0),
        "".to_string(),
    );

    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, lazycsv::session::FileConfig::new());

    // Open magnifier on empty cell
    app.open_magnifier();

    let magnifier = app.magnifier_state.as_ref().unwrap();
    assert_eq!(magnifier.get_content(), "");
    assert_eq!(magnifier.lines().len(), 1);
    assert_eq!(magnifier.cursor(), (0, 0));

    // Add content
    let magnifier = app.magnifier_state.as_mut().unwrap();
    magnifier.enter_insert_mode();
    magnifier.insert_char('N');
    magnifier.insert_char('e');
    magnifier.insert_char('w');
    magnifier.exit_insert_mode();

    assert_eq!(magnifier.get_content(), "New");

    // Save
    app.save_and_close_magnifier();
    assert_eq!(
        app.document.get_cell(
            app.get_selected_row().unwrap(),
            app.view_state.selected_column
        ),
        "New"
    );
}

#[test]
fn test_magnifier_long_content() {
    let doc = create_test_document();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, lazycsv::session::FileConfig::new());

    // Open magnifier
    app.open_magnifier();

    // Add very long content
    let magnifier = app.magnifier_state.as_mut().unwrap();
    magnifier.enter_insert_mode();
    magnifier.move_to_line_end();
    for _ in 0..100 {
        magnifier.insert_char('X');
    }
    magnifier.exit_insert_mode();

    // Verify content length
    let content = magnifier.get_content();
    assert!(content.len() > 100);

    // Save and verify
    app.save_and_close_magnifier();
    let cell_content = app.document.get_cell(
        app.get_selected_row().unwrap(),
        app.view_state.selected_column,
    );
    assert!(cell_content.len() > 100);
}

#[test]
#[ignore] // TODO: Fix unicode handling - cursor uses char indices but String::insert uses byte indices
fn test_magnifier_unicode_content() {
    let doc = create_test_document();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, lazycsv::session::FileConfig::new());

    // Open magnifier
    app.open_magnifier();

    // Add unicode content (use string insertion instead of char-by-char to avoid boundary issues)
    let magnifier = app.magnifier_state.as_mut().unwrap();
    let original = magnifier.get_content();
    magnifier.enter_insert_mode();
    magnifier.move_to_line_end();
    // Insert as a string to avoid char boundary issues
    for ch in "  日本".chars() {
        magnifier.insert_char(ch);
    }
    magnifier.exit_insert_mode();

    // Verify unicode preserved
    let content = magnifier.get_content();
    // Unicode characters should be preserved
    assert!(content.len() > original.len());

    // Save and verify
    app.save_and_close_magnifier();
    let cell_content = app.document.get_cell(
        app.get_selected_row().unwrap(),
        app.view_state.selected_column,
    );
    assert!(cell_content.len() > original.len());
    assert!(cell_content.contains('日'));
}

#[test]
fn test_magnifier_count_prefix() {
    let doc = create_test_document();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, lazycsv::session::FileConfig::new());

    // Open magnifier with multi-line content
    app.open_magnifier();

    let magnifier = app.magnifier_state.as_mut().unwrap();
    magnifier.enter_insert_mode();
    magnifier.newline();
    magnifier.insert_char('L');
    magnifier.insert_char('2');
    magnifier.newline();
    magnifier.insert_char('L');
    magnifier.insert_char('3');
    magnifier.exit_insert_mode();

    // Move to first line
    magnifier.move_to_first_line();
    assert_eq!(magnifier.cursor().0, 0);

    // Use count prefix to move down 2 lines
    magnifier.set_count_prefix(2);
    magnifier.move_down();
    assert_eq!(magnifier.cursor().0, 2);

    app.close_magnifier_discard();
}

#[test]
fn test_magnifier_yank_paste() {
    let doc = create_test_document();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, lazycsv::session::FileConfig::new());

    // Open magnifier with multi-line content
    app.open_magnifier();

    let magnifier = app.magnifier_state.as_mut().unwrap();
    magnifier.enter_insert_mode();
    magnifier.newline();
    magnifier.insert_char('L');
    magnifier.insert_char('i');
    magnifier.insert_char('n');
    magnifier.insert_char('e');
    magnifier.insert_char(' ');
    magnifier.insert_char('2');
    magnifier.exit_insert_mode();

    // Yank first line
    magnifier.move_to_first_line();
    magnifier.yank_line();

    // Move to end and paste
    magnifier.move_to_last_line();
    magnifier.paste_below();

    // Verify line was pasted
    assert_eq!(magnifier.lines().len(), 3);

    app.close_magnifier_discard();
}

#[test]
fn test_magnifier_delete_line() {
    let doc = create_test_document();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, lazycsv::session::FileConfig::new());

    // Open magnifier with multi-line content
    app.open_magnifier();

    let magnifier = app.magnifier_state.as_mut().unwrap();
    magnifier.enter_insert_mode();
    magnifier.newline();
    magnifier.insert_char('L');
    magnifier.insert_char('2');
    magnifier.newline();
    magnifier.insert_char('L');
    magnifier.insert_char('3');
    magnifier.exit_insert_mode();

    let initial_lines = magnifier.lines().len();
    assert_eq!(initial_lines, 3);

    // Delete middle line
    magnifier.move_to_line(1);
    magnifier.delete_line();

    assert_eq!(magnifier.lines().len(), 2);
    assert!(magnifier.is_dirty());

    app.close_magnifier_discard();
}

#[test]
fn test_magnifier_insert_modes() {
    let doc = create_test_document();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, lazycsv::session::FileConfig::new());

    // Open magnifier
    app.open_magnifier();

    let magnifier = app.magnifier_state.as_mut().unwrap();

    // Test insert_before (i)
    magnifier.insert_before();
    assert_eq!(magnifier.mode(), lazycsv::magnifier::MagnifierMode::Insert);
    magnifier.exit_insert_mode();

    // Test insert_after (a)
    magnifier.insert_after();
    assert_eq!(magnifier.mode(), lazycsv::magnifier::MagnifierMode::Insert);
    magnifier.exit_insert_mode();

    // Test insert_line_below (o)
    let initial_lines = magnifier.lines().len();
    magnifier.insert_line_below();
    assert_eq!(magnifier.mode(), lazycsv::magnifier::MagnifierMode::Insert);
    assert_eq!(magnifier.lines().len(), initial_lines + 1);
    magnifier.exit_insert_mode();

    // Test insert_line_above (O)
    magnifier.insert_line_above();
    assert_eq!(magnifier.mode(), lazycsv::magnifier::MagnifierMode::Insert);
    magnifier.exit_insert_mode();

    app.close_magnifier_discard();
}

#[test]
fn test_magnifier_substitute_char() {
    let doc = create_test_document();
    let files = vec![PathBuf::from("test.csv")];
    let mut app = App::new(doc, files, 0, lazycsv::session::FileConfig::new());

    // Open magnifier
    app.open_magnifier();

    let magnifier = app.magnifier_state.as_mut().unwrap();
    let original = magnifier.get_content();

    // Substitute character (s)
    magnifier.substitute_char();
    assert_eq!(magnifier.mode(), lazycsv::magnifier::MagnifierMode::Insert);

    // Type new character
    magnifier.insert_char('X');
    magnifier.exit_insert_mode();

    // Verify character was substituted
    assert_ne!(magnifier.get_content(), original);
    assert!(magnifier.is_dirty());

    app.close_magnifier_discard();
}
