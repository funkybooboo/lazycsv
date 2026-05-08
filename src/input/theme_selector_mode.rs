//! Theme selector mode — keyboard input handling.
//!
//! Delegates rendering to `ui::theme_selector` and config persistence to
//! `config::toml_parsing`, keeping this module focused on input dispatch.

use crate::app::Mode;
use crate::input::InputResult;
use crate::App;

/// Handle a key event while in `Mode::ThemeSelector`.
pub fn handle(app: &mut App, key: crossterm::event::KeyEvent) -> InputResult {
    use crossterm::event::KeyCode;

    let max_index = app.theme_list.len().saturating_sub(1);

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if app.theme_selector_index < max_index {
                app.theme_selector_index += 1;
            }
            InputResult::Continue
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.theme_selector_index > 0 {
                app.theme_selector_index -= 1;
            }
            InputResult::Continue
        }
        KeyCode::Char('g') | KeyCode::Home => {
            app.theme_selector_index = 0;
            InputResult::Continue
        }
        KeyCode::Char('G') | KeyCode::End => {
            app.theme_selector_index = max_index;
            InputResult::Continue
        }

        KeyCode::Enter => {
            apply_selected_theme(app);
            app.mode = Mode::Normal;
            InputResult::Continue
        }

        KeyCode::Esc => {
            app.mode = Mode::Normal;
            InputResult::Continue
        }

        KeyCode::PageDown => {
            app.theme_selector_index = (app.theme_selector_index + 10).min(max_index);
            InputResult::Continue
        }
        KeyCode::PageUp => {
            app.theme_selector_index = app.theme_selector_index.saturating_sub(10);
            InputResult::Continue
        }

        _ => InputResult::Continue,
    }
}

/// Move the selection index by `delta` (negative = up, positive = down).
/// Called by the mouse scroll handler to avoid duplicating bounds logic.
pub fn scroll_selection(app: &mut App, delta: i32) {
    let max_index = app.theme_list.len().saturating_sub(1);
    if delta < 0 {
        app.theme_selector_index = app.theme_selector_index.saturating_sub((-delta) as usize);
    } else {
        app.theme_selector_index = (app.theme_selector_index + delta as usize).min(max_index);
    }
}

/// Apply the currently selected theme and persist it to config.
pub(crate) fn apply_selected_theme(app: &mut App) {
    if app.theme_list.is_empty() {
        return;
    }

    let (_name, path) = app.theme_list[app.theme_selector_index].clone();

    let mut warnings = Vec::new();
    match crate::config::apply_theme_from_file(&mut app.config, &path, &mut warnings) {
        Ok(Some(())) => {
            if let Err(e) = save_config(&app.config) {
                app.status_message = Some(crate::input::StatusMessage::from(format!(
                    "Failed to save config: {}",
                    e
                )));
                return;
            }

            let warning_msg = if warnings.is_empty() {
                String::new()
            } else {
                format!(" (warnings: {})", warnings.join("; "))
            };

            let name = &app.theme_list[app.theme_selector_index].0;
            app.status_message = Some(crate::input::StatusMessage::from(format!(
                "Applied theme: {}{}",
                name, warning_msg
            )));
        }
        Ok(None) => {
            app.status_message = Some(crate::input::StatusMessage::from(format!(
                "Theme file not found: {}",
                path.display()
            )));
        }
        Err(e) => {
            app.status_message = Some(crate::input::StatusMessage::from(format!(
                "Failed to load theme: {}",
                e
            )));
        }
    }
}

/// Persist the current config to `~/.config/lazycsv/config.toml`.
fn save_config(config: &crate::config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = crate::config::dirs_path()
        .map(|p| p.join("config.toml"))
        .ok_or("Could not find config directory")?;

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let toml_string = crate::config::config_to_toml_string(config)?;
    std::fs::write(&config_path, toml_string)?;

    Ok(())
}
