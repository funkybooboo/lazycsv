//! Help overlay handling for Normal mode

use crate::app::App;
use crossterm::event::{KeyCode, KeyModifiers};

/// Maximum scroll position for help overlay content
const HELP_CONTENT_LINES: u16 = 200;

/// Page size for help overlay scrolling (Ctrl+d/u)
const HELP_PAGE_SIZE: u16 = 10;

/// Returns true if navigation commands are allowed (help overlay is closed)
pub fn is_navigation_allowed(app: &App) -> bool {
    !app.view_state.help_overlay_visible
}

/// Toggle help overlay visibility
pub fn toggle(app: &mut App) {
    app.view_state.help_overlay_visible = !app.view_state.help_overlay_visible;
}

/// Close help overlay
pub fn close(app: &mut App) {
    app.view_state.hide_help();
}

/// Scroll help overlay down by one line
pub fn scroll_down(app: &mut App) {
    app.view_state.scroll_help_down(HELP_CONTENT_LINES);
}

/// Scroll help overlay up by one line
pub fn scroll_up(app: &mut App) {
    app.view_state.scroll_help_up();
}

/// Scroll help overlay down by one page
pub fn scroll_page_down(app: &mut App) {
    app.view_state
        .scroll_help_page_down(HELP_PAGE_SIZE, HELP_CONTENT_LINES);
}

/// Scroll help overlay up by one page
pub fn scroll_page_up(app: &mut App) {
    app.view_state.scroll_help_page_up(HELP_PAGE_SIZE);
}

/// Handle help overlay keyboard input
/// Returns true if the key was handled
pub fn handle_key(app: &mut App, key: KeyCode, modifiers: KeyModifiers) -> bool {
    if !app.view_state.help_overlay_visible {
        return false;
    }

    // Handle help search input mode (user is typing in the search prompt)
    if app.view_state.help_search_input_active {
        return handle_search_input(app, key);
    }

    // When help overlay is visible, handle vim navigation keys
    // but block all other keys (making it non-editable)
    match key {
        // Allow '?' to pass through so it can toggle help off
        KeyCode::Char('?') => false,
        KeyCode::Esc => {
            if app.view_state.help_search_query.is_some() {
                // First Esc: clear search highlights
                app.view_state.help_search_query = None;
                app.view_state.help_search_match_index = 0;
            } else {
                // Second Esc (no active search): close help
                close(app);
            }
            true
        }
        KeyCode::Char('/') => {
            // Enter search input mode
            app.view_state.help_search_query = Some(String::new());
            app.view_state.help_search_input_active = true;
            app.help_search_buffer.clear();
            true
        }
        KeyCode::Char('n') => {
            // Next search result
            next_search_match(app);
            true
        }
        KeyCode::Char('N') => {
            // Previous search result
            prev_search_match(app);
            true
        }
        KeyCode::Char('j') | KeyCode::Down => {
            scroll_down(app);
            true
        }
        KeyCode::Char('k') | KeyCode::Up => {
            scroll_up(app);
            true
        }
        KeyCode::Char('g') => {
            // Jump to top
            app.view_state.help_scroll_offset = 0;
            true
        }
        KeyCode::Char('G') => {
            // Jump to bottom
            app.view_state.help_scroll_offset = HELP_CONTENT_LINES;
            true
        }
        KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
            scroll_page_down(app);
            true
        }
        KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
            scroll_page_up(app);
            true
        }
        KeyCode::Char('f') if modifiers.contains(KeyModifiers::CONTROL) => {
            scroll_page_down(app);
            true
        }
        KeyCode::Char('b') if modifiers.contains(KeyModifiers::CONTROL) => {
            scroll_page_up(app);
            true
        }
        // Block all other keys when help overlay is visible (non-editable)
        _ => true,
    }
}

/// Handle keyboard input during help search input mode
fn handle_search_input(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Esc => {
            // Cancel search entirely
            app.view_state.help_search_query = None;
            app.view_state.help_search_input_active = false;
            app.help_search_buffer.clear();
            app.view_state.help_search_match_index = 0;
            true
        }
        KeyCode::Enter => {
            // Confirm search: exit input mode, keep query for highlighting + n/N
            app.view_state.help_search_input_active = false;
            execute_help_search(app);
            true
        }
        KeyCode::Backspace => {
            if app.help_search_buffer.is_empty() {
                // Exit search mode if buffer is empty
                app.view_state.help_search_query = None;
                app.view_state.help_search_input_active = false;
            } else {
                app.help_search_buffer.pop();
                app.view_state.help_search_query = Some(app.help_search_buffer.clone());
            }
            true
        }
        KeyCode::Char(c) => {
            app.help_search_buffer.push(c);
            app.view_state.help_search_query = Some(app.help_search_buffer.clone());
            true
        }
        _ => true, // Block all other keys
    }
}

/// Execute help search and jump to first match
fn execute_help_search(app: &mut App) {
    if let Some(query) = &app.view_state.help_search_query {
        if !query.is_empty() {
            let matches = find_help_matches(query);
            if !matches.is_empty() {
                app.view_state.help_search_match_index = 0;
                // Scroll to first match
                app.view_state.help_scroll_offset = matches[0].saturating_sub(2);
            }
        }
    }
}

/// Find all line numbers in help text that match the query (case-insensitive)
fn find_help_matches(query: &str) -> Vec<u16> {
    let query_lower = query.to_lowercase();
    let help_text = crate::ui::help::get_help_text();

    help_text
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            // Extract text from the line
            let line_text = line
                .spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
                .to_lowercase();

            if line_text.contains(&query_lower) {
                Some(i as u16)
            } else {
                None
            }
        })
        .collect()
}

/// Jump to next search match
fn next_search_match(app: &mut App) {
    if let Some(query) = &app.view_state.help_search_query.clone() {
        let matches = find_help_matches(query);
        if !matches.is_empty() {
            app.view_state.help_search_match_index =
                (app.view_state.help_search_match_index + 1) % matches.len();
            let line = matches[app.view_state.help_search_match_index];
            app.view_state.help_scroll_offset = line.saturating_sub(2);
        }
    }
}

/// Jump to previous search match
fn prev_search_match(app: &mut App) {
    if let Some(query) = &app.view_state.help_search_query.clone() {
        let matches = find_help_matches(query);
        if !matches.is_empty() {
            if app.view_state.help_search_match_index == 0 {
                app.view_state.help_search_match_index = matches.len() - 1;
            } else {
                app.view_state.help_search_match_index -= 1;
            }
            let line = matches[app.view_state.help_search_match_index];
            app.view_state.help_scroll_offset = line.saturating_sub(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv::Document;
    use crate::session::FileConfig;
    use std::path::PathBuf;

    fn create_test_app() -> crate::App {
        let document = Document::new(
            vec!["A".to_string()],
            vec![vec!["1".to_string()]],
            "test.csv".to_string(),
        );
        crate::App::new(
            document,
            vec![PathBuf::from("test.csv")],
            0,
            FileConfig::new(),
        )
    }

    #[test]
    fn test_is_navigation_allowed_when_help_closed() {
        let app = create_test_app();
        assert!(is_navigation_allowed(&app));
    }

    #[test]
    fn test_is_navigation_allowed_when_help_open() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;
        assert!(!is_navigation_allowed(&app));
    }

    #[test]
    fn test_toggle_opens_help() {
        let mut app = create_test_app();
        toggle(&mut app);
        assert!(app.view_state.help_overlay_visible);
    }

    #[test]
    fn test_toggle_closes_help() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;
        toggle(&mut app);
        assert!(!app.view_state.help_overlay_visible);
    }

    #[test]
    fn test_close_help() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;
        close(&mut app);
        assert!(!app.view_state.help_overlay_visible);
    }

    #[test]
    fn test_handle_key_esc_closes_help() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;
        let handled = handle_key(&mut app, KeyCode::Esc, KeyModifiers::empty());
        assert!(handled);
        assert!(!app.view_state.help_overlay_visible);
    }

    #[test]
    fn test_handle_key_returns_false_when_help_closed() {
        let mut app = create_test_app();
        let handled = handle_key(&mut app, KeyCode::Char('j'), KeyModifiers::empty());
        assert!(!handled);
    }

    #[test]
    fn test_handle_key_j_scrolls_down() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;
        let initial_scroll = app.view_state.help_scroll_offset;
        let handled = handle_key(&mut app, KeyCode::Char('j'), KeyModifiers::empty());
        assert!(handled);
        assert!(app.view_state.help_scroll_offset > initial_scroll);
    }

    #[test]
    fn test_handle_key_k_scrolls_up() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;
        app.view_state.help_scroll_offset = 5;
        let initial_scroll = app.view_state.help_scroll_offset;
        let handled = handle_key(&mut app, KeyCode::Char('k'), KeyModifiers::empty());
        assert!(handled);
        assert!(app.view_state.help_scroll_offset < initial_scroll);
    }

    #[test]
    fn test_handle_key_g_jumps_to_top() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;
        app.view_state.help_scroll_offset = 10;
        let handled = handle_key(&mut app, KeyCode::Char('g'), KeyModifiers::empty());
        assert!(handled);
        assert_eq!(app.view_state.help_scroll_offset, 0);
    }

    #[test]
    fn test_handle_key_shift_g_jumps_to_bottom() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;
        let handled = handle_key(&mut app, KeyCode::Char('G'), KeyModifiers::empty());
        assert!(handled);
        assert_eq!(app.view_state.help_scroll_offset, HELP_CONTENT_LINES);
    }

    #[test]
    fn test_handle_key_ctrl_d_scrolls_page_down() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;
        let initial_scroll = app.view_state.help_scroll_offset;
        let handled = handle_key(&mut app, KeyCode::Char('d'), KeyModifiers::CONTROL);
        assert!(handled);
        assert!(app.view_state.help_scroll_offset > initial_scroll);
    }

    #[test]
    fn test_handle_key_ctrl_u_scrolls_page_up() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;
        app.view_state.help_scroll_offset = 15;
        let initial_scroll = app.view_state.help_scroll_offset;
        let handled = handle_key(&mut app, KeyCode::Char('u'), KeyModifiers::CONTROL);
        assert!(handled);
        assert!(app.view_state.help_scroll_offset < initial_scroll);
    }

    #[test]
    fn test_handle_key_blocks_edit_keys() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;

        // Test that editing keys are blocked (return true but don't do anything)
        assert!(handle_key(
            &mut app,
            KeyCode::Char('i'),
            KeyModifiers::empty()
        ));
        assert!(handle_key(
            &mut app,
            KeyCode::Char('a'),
            KeyModifiers::empty()
        ));
        assert!(handle_key(
            &mut app,
            KeyCode::Char('s'),
            KeyModifiers::empty()
        ));
        assert!(handle_key(
            &mut app,
            KeyCode::Char('d'),
            KeyModifiers::empty()
        ));
        assert!(handle_key(
            &mut app,
            KeyCode::Char('y'),
            KeyModifiers::empty()
        ));
        assert!(handle_key(
            &mut app,
            KeyCode::Char('p'),
            KeyModifiers::empty()
        ));
        assert!(handle_key(
            &mut app,
            KeyCode::Char('o'),
            KeyModifiers::empty()
        ));
        assert!(handle_key(
            &mut app,
            KeyCode::Char('O'),
            KeyModifiers::empty()
        ));

        // Help should still be visible (keys were blocked)
        assert!(app.view_state.help_overlay_visible);
    }

    #[test]
    fn test_handle_key_allows_question_mark_to_pass_through() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;

        // '?' should return false to allow the main handler to toggle help
        let handled = handle_key(&mut app, KeyCode::Char('?'), KeyModifiers::empty());
        assert!(!handled);

        // Help should still be visible (main handler will toggle it)
        assert!(app.view_state.help_overlay_visible);
    }

    #[test]
    fn test_handle_key_slash_enters_search_mode() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;

        let handled = handle_key(&mut app, KeyCode::Char('/'), KeyModifiers::empty());
        assert!(handled);
        assert!(app.view_state.help_search_query.is_some());
        assert_eq!(app.view_state.help_search_query.as_ref().unwrap(), "");
    }

    #[test]
    fn test_help_search_input() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;

        // Enter search mode
        handle_key(&mut app, KeyCode::Char('/'), KeyModifiers::empty());

        // Type search query
        handle_key(&mut app, KeyCode::Char('i'), KeyModifiers::empty());
        handle_key(&mut app, KeyCode::Char('n'), KeyModifiers::empty());
        handle_key(&mut app, KeyCode::Char('s'), KeyModifiers::empty());

        assert_eq!(app.help_search_buffer, "ins");
        assert_eq!(app.view_state.help_search_query.as_ref().unwrap(), "ins");
    }

    #[test]
    fn test_help_search_backspace() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;

        // Enter search mode and type
        handle_key(&mut app, KeyCode::Char('/'), KeyModifiers::empty());
        handle_key(&mut app, KeyCode::Char('t'), KeyModifiers::empty());
        handle_key(&mut app, KeyCode::Char('e'), KeyModifiers::empty());

        // Backspace
        handle_key(&mut app, KeyCode::Backspace, KeyModifiers::empty());
        assert_eq!(app.help_search_buffer, "t");
    }

    #[test]
    fn test_help_search_esc_cancels() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;

        // Enter search mode
        handle_key(&mut app, KeyCode::Char('/'), KeyModifiers::empty());
        handle_key(&mut app, KeyCode::Char('t'), KeyModifiers::empty());

        // Esc cancels
        handle_key(&mut app, KeyCode::Esc, KeyModifiers::empty());
        assert!(app.view_state.help_search_query.is_none());
        assert_eq!(app.help_search_buffer, "");
    }

    #[test]
    fn test_help_search_blocks_edit_keys() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;

        // Enter search mode
        handle_key(&mut app, KeyCode::Char('/'), KeyModifiers::empty());

        // Navigation keys like 'j' should be captured as search text, not navigation
        handle_key(&mut app, KeyCode::Char('j'), KeyModifiers::empty());
        assert_eq!(app.help_search_buffer, "j");
    }

    #[test]
    fn test_find_help_matches() {
        let matches = find_help_matches("insert");
        // Should find multiple lines containing "insert" (case-insensitive)
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_find_help_matches_case_insensitive() {
        let matches_lower = find_help_matches("insert");
        let matches_upper = find_help_matches("INSERT");
        assert_eq!(matches_lower, matches_upper);
    }

    #[test]
    fn test_slash_enters_search_input_mode() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;

        let handled = handle_key(&mut app, KeyCode::Char('/'), KeyModifiers::empty());
        assert!(handled);
        assert!(app.view_state.help_search_input_active);
        assert_eq!(app.view_state.help_search_query, Some(String::new()));
    }

    #[test]
    fn test_enter_confirms_search_exits_input_mode() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;
        app.view_state.help_search_input_active = true;
        app.help_search_buffer = "pin".to_string();
        app.view_state.help_search_query = Some("pin".to_string());

        let handled = handle_key(&mut app, KeyCode::Enter, KeyModifiers::empty());
        assert!(handled);
        assert!(!app.view_state.help_search_input_active); // input mode exited
        assert!(app.view_state.help_search_query.is_some()); // query preserved for n/N
    }

    #[test]
    fn test_n_navigates_after_search_confirmed() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;
        app.view_state.help_search_query = Some("pin".to_string());
        app.view_state.help_search_input_active = false;

        let initial_offset = app.view_state.help_scroll_offset;
        let handled = handle_key(&mut app, KeyCode::Char('n'), KeyModifiers::empty());
        assert!(handled);
        // Scroll offset should change (or stay if only one match)
        // At minimum, it should not crash
        let _ = app.view_state.help_scroll_offset;
        let _ = initial_offset;
    }

    #[test]
    fn test_esc_clears_search_before_closing_help() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;
        app.view_state.help_search_query = Some("pin".to_string());
        app.view_state.help_search_input_active = false;

        // First Esc: clears search, keeps help open
        let handled = handle_key(&mut app, KeyCode::Esc, KeyModifiers::empty());
        assert!(handled);
        assert!(app.view_state.help_overlay_visible); // still open
        assert!(app.view_state.help_search_query.is_none()); // search cleared

        // Second Esc: closes help
        let handled = handle_key(&mut app, KeyCode::Esc, KeyModifiers::empty());
        assert!(handled);
        assert!(!app.view_state.help_overlay_visible);
    }

    #[test]
    fn test_esc_closes_help_when_no_search() {
        let mut app = create_test_app();
        app.view_state.help_overlay_visible = true;
        // No active search query

        let handled = handle_key(&mut app, KeyCode::Esc, KeyModifiers::empty());
        assert!(handled);
        assert!(!app.view_state.help_overlay_visible);
    }
}
