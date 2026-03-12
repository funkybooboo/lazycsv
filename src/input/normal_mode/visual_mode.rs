//! Visual mode entry operations for Normal mode

use crate::app::{App, Mode, VisualMode, VisualSelection};
use crate::domain::position::RowIndex;

/// Enter Visual Block mode (v)
pub fn enter_block_mode(app: &mut App) {
    let row = app.selected_row().unwrap_or(RowIndex::new(0));
    let col = app.view_state.selected_column;
    app.visual_selection = Some(VisualSelection::new(row, col, VisualMode::Block));
    app.mode = Mode::VisualBlock;
}

/// Enter Visual Line mode (V)
pub fn enter_line_mode(app: &mut App) {
    let row = app.selected_row().unwrap_or(RowIndex::new(0));
    let col = app.view_state.selected_column;
    app.visual_selection = Some(VisualSelection::new(row, col, VisualMode::Line));
    app.mode = Mode::VisualLine;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv::Document;
    use crate::domain::position::{ColIndex, RowIndex};
    use crate::session::FileConfig;
    use std::path::PathBuf;

    fn create_test_app() -> App {
        let document = Document::new(
            vec!["A".to_string(), "B".to_string()],
            vec![
                vec!["a1".to_string(), "b1".to_string()],
                vec!["a2".to_string(), "b2".to_string()],
            ],
            "test.csv".to_string(),
        );
        App::new(
            document,
            vec![PathBuf::from("test.csv")],
            0,
            FileConfig::new(),
        )
    }

    #[test]
    fn test_enter_block_mode() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(0));
        enter_block_mode(&mut app);

        assert_eq!(app.mode, Mode::VisualBlock);
        assert!(app.visual_selection.is_some());
        let sel = app.visual_selection.unwrap();
        assert_eq!(sel.mode, VisualMode::Block);
    }

    #[test]
    fn test_enter_line_mode() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(1));
        enter_line_mode(&mut app);

        assert_eq!(app.mode, Mode::VisualLine);
        assert!(app.visual_selection.is_some());
        let sel = app.visual_selection.unwrap();
        assert_eq!(sel.mode, VisualMode::Line);
    }

    #[test]
    fn test_visual_modes_preserve_cursor_position() {
        let mut app = create_test_app();
        app.view_state.table_state.select(Some(1));
        app.view_state.selected_column = ColIndex::new(1);

        enter_block_mode(&mut app);
        let sel = app.visual_selection.unwrap();
        assert_eq!(sel.anchor.0, RowIndex::new(1));
        assert_eq!(sel.anchor.1, ColIndex::new(1));
    }
}
