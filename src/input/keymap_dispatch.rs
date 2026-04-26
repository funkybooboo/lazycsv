//! Bridge between the keymap lookup and the existing per-mode handlers.
//!
//! `execute(app, action)` runs an [`Action`] by calling the same code path
//! the hardcoded key arm would have. The strategy is **synthetic key
//! events**: every action has a canonical default keypress (the one wired
//! up in `keymaps/vim.toml`), and we simply forward that key into the
//! existing handler instead of re-implementing the action.
//!
//! That keeps the action ↔ implementation coupling weak: when a handler
//! changes, the dispatcher doesn't need to follow.
//!
//! # Phase coverage
//!
//! - **Phase 1** (this file): Normal mode + Insert mode actions are wired.
//!   Other modes' actions return `InputResult::Continue` as a no-op so the
//!   user's existing mappings still work via the legacy match arms.
//! - **Phase 2** will extend the dispatcher and migrate the legacy arms.

use crate::app::App;
use crate::config::keys::{KeyAtom, KeySequence, KeymapScope, LookupResult};
use crate::input::keymap_actions::Action;
use crate::input::InputResult;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Try to dispatch a key via the active keymap.
///
/// - On `Action`: dispatch the action and return its result.
/// - On `PartialChord`: buffer the key and return `Continue` (more keys
///   needed to resolve the chord).
/// - On `Unbound`:
///   - If the buffer had previous atoms (we were holding a chord prefix
///     that didn't pan out — e.g. `g` then `X` for a parametric chord
///     like vim's `g{letter}`), **replay** all buffered keys through
///     `handle_raw` so the legacy chord state can rebuild. Returns
///     `Some(Continue)`.
///   - If the buffer was empty (single-key unbound), returns `None` so
///     the caller falls through to its legacy match.
pub fn try_keymap(
    app: &mut App,
    key: KeyEvent,
    scope: KeymapScope,
    handle_raw: fn(&mut App, KeyEvent) -> Result<InputResult>,
) -> Result<Option<InputResult>> {
    let atom = KeyAtom::from_event(key);
    let mut atoms = std::mem::take(&mut app.input_state.chord_buffer);
    atoms.push(atom);
    let seq = KeySequence(atoms);

    match app.keymap.lookup(scope, &seq) {
        LookupResult::Action(action) => execute(app, action),
        LookupResult::PartialChord => {
            app.input_state.chord_buffer = seq.0;
            Ok(Some(InputResult::Continue))
        }
        LookupResult::ExplicitlyUnbound => {
            // User said "this key does nothing" via `""` in keys.toml.
            // Don't fall through to the legacy handler — return Continue
            // so the keypress is consumed silently.
            Ok(Some(InputResult::Continue))
        }
        LookupResult::Unbound if seq.len() > 1 => {
            // The keymap was holding a chord prefix that didn't pan out.
            // Replay everything through the legacy handler so its own
            // PendingCommand state can build up. This preserves
            // parametric chords like `g{letter}`/`q{a-z}` that the
            // keymap can't express directly.
            for a in seq.0.iter() {
                let ev = KeyEvent::new(a.code, a.modifiers);
                handle_raw(app, ev)?;
            }
            Ok(Some(InputResult::Continue))
        }
        LookupResult::Unbound => Ok(None),
    }
}

/// Reset any in-progress chord (called when modes change).
pub fn clear_chord(app: &mut App) {
    app.input_state.chord_buffer.clear();
}

/// Dispatch an `Action` against the running app.
///
/// Returns `Some(result)` if the action was handled, `None` if it isn't
/// wired in this phase — letting the caller fall through to the existing
/// match-based handler.
pub fn execute(app: &mut App, action: Action) -> Result<Option<InputResult>> {
    use Action::*;

    // Each arm produces either a synthesized `KeyEvent` for a routing call,
    // or directly returns the InputResult.
    match action {
        // ── Normal mode: cursor / navigation ─────────────────────────
        CursorLeft => syn_navigate(app, KeyCode::Char('h'))?,
        CursorRight => syn_navigate(app, KeyCode::Char('l'))?,
        CursorDown => syn_navigate(app, KeyCode::Char('j'))?,
        CursorUp => syn_navigate(app, KeyCode::Char('k'))?,
        CursorWordForward => syn_navigate(app, KeyCode::Char('w'))?,
        CursorWordBackward => syn_navigate(app, KeyCode::Char('b'))?,
        CursorWordEnd => syn_navigate(app, KeyCode::Char('e'))?,
        GotoLastRow => crate::navigation::commands::goto_last_row(app),
        GotoFirstColumn => syn_navigate(app, KeyCode::Char('0'))?,
        GotoLastColumn => syn_navigate(app, KeyCode::Char('$'))?,
        PageDown => syn_normal(app, KeyCode::Char('d'), KeyModifiers::CONTROL)?,
        PageUp => syn_normal(app, KeyCode::Char('u'), KeyModifiers::CONTROL)?,
        HalfPageDown => syn_normal(app, KeyCode::Char('d'), KeyModifiers::CONTROL)?,
        HalfPageUp => syn_normal(app, KeyCode::Char('u'), KeyModifiers::CONTROL)?,

        // ── Cell editing ─────────────────────────────────────────────
        CellEditAtEnd => syn_normal(app, KeyCode::Char('i'), KeyModifiers::NONE)?,
        CellEditAtStart => syn_normal(app, KeyCode::Char('I'), KeyModifiers::SHIFT)?,
        CellEditAtLineEnd => syn_normal(app, KeyCode::Char('A'), KeyModifiers::SHIFT)?,
        CellReplace => syn_normal(app, KeyCode::Char('s'), KeyModifiers::NONE)?,
        CellReplaceF2 => syn_normal(app, KeyCode::F(2), KeyModifiers::NONE)?,
        CellClear => syn_normal(app, KeyCode::Delete, KeyModifiers::NONE)?,
        Undo => syn_normal(app, KeyCode::Char('u'), KeyModifiers::NONE)?,
        Redo => syn_normal(app, KeyCode::Char('r'), KeyModifiers::CONTROL)?,
        RepeatLastEdit => syn_normal(app, KeyCode::Char('.'), KeyModifiers::NONE)?,
        ToggleCase => syn_normal(app, KeyCode::Char('~'), KeyModifiers::NONE)?,

        // ── Row operations (single-key only — chord ones in Phase 2) ─
        RowInsertBelow => syn_normal(app, KeyCode::Char('o'), KeyModifiers::NONE)?,
        RowInsertAbove => syn_normal(app, KeyCode::Char('O'), KeyModifiers::SHIFT)?,
        RowPasteBelow => syn_normal(app, KeyCode::Char('p'), KeyModifiers::NONE)?,
        RowPasteAbove => syn_normal(app, KeyCode::Char('P'), KeyModifiers::SHIFT)?,

        // ── Column width ─────────────────────────────────────────────
        ColWidthIncrease => syn_normal(app, KeyCode::Char('+'), KeyModifiers::NONE)?,
        ColWidthDecrease => syn_normal(app, KeyCode::Char('-'), KeyModifiers::NONE)?,

        // ── Search ───────────────────────────────────────────────────
        SearchEnter => syn_normal(app, KeyCode::Char('/'), KeyModifiers::NONE)?,
        SearchNext => syn_normal(app, KeyCode::Char('n'), KeyModifiers::NONE)?,
        SearchPrev => syn_normal(app, KeyCode::Char('N'), KeyModifiers::SHIFT)?,
        SearchCurrentCell => syn_normal(app, KeyCode::Char('*'), KeyModifiers::NONE)?,

        // ── Mode entry ───────────────────────────────────────────────
        EnterCommandMode => syn_normal(app, KeyCode::Char(':'), KeyModifiers::NONE)?,
        EnterVisualBlock => syn_normal(app, KeyCode::Char('v'), KeyModifiers::NONE)?,
        EnterVisualLine => syn_normal(app, KeyCode::Char('V'), KeyModifiers::SHIFT)?,

        // ── Help (cross-mode) ────────────────────────────────────────
        ToggleHelp => syn_normal(app, KeyCode::Char('?'), KeyModifiers::NONE)?,

        // ── File save / quit (no default key in vim, but emacs/excel
        //    presets bind Ctrl-S / Ctrl-Q etc. to these). Calls the same
        //    code path the `:w` / `:q` / `:wq` ex commands use.
        Save => {
            return Ok(Some(crate::input::command_mode::executor::execute_write(
                app,
            )?));
        }
        SaveQuit => {
            return Ok(Some(
                crate::input::command_mode::executor::execute_write_quit(app)?,
            ));
        }
        Quit => {
            return Ok(Some(crate::input::command_mode::executor::execute_quit(
                app,
            )?));
        }
        QuitForce => {
            return Ok(Some(
                crate::input::command_mode::executor::execute_force_quit(app)?,
            ));
        }

        // ── Insert mode ──────────────────────────────────────────────
        InsertCommitDown => syn_insert(app, KeyCode::Enter, KeyModifiers::NONE),
        InsertCommitUp => syn_insert(app, KeyCode::Enter, KeyModifiers::SHIFT),
        InsertCommitLeft => syn_insert(app, KeyCode::Left, KeyModifiers::SHIFT),
        InsertCommitRight => syn_insert(app, KeyCode::Right, KeyModifiers::SHIFT),
        InsertCommitTab => syn_insert(app, KeyCode::Tab, KeyModifiers::NONE),
        InsertCommitBackTab => syn_insert(app, KeyCode::BackTab, KeyModifiers::SHIFT),
        InsertCancel => syn_insert(app, KeyCode::Esc, KeyModifiers::NONE),
        InsertCursorLeft => syn_insert(app, KeyCode::Left, KeyModifiers::NONE),
        InsertCursorRight => syn_insert(app, KeyCode::Right, KeyModifiers::NONE),
        InsertCursorHome => syn_insert(app, KeyCode::Home, KeyModifiers::NONE),
        InsertCursorEnd => syn_insert(app, KeyCode::End, KeyModifiers::NONE),
        InsertDeleteBackward => syn_insert(app, KeyCode::Backspace, KeyModifiers::NONE),
        InsertDeleteForward => syn_insert(app, KeyCode::Delete, KeyModifiers::NONE),
        InsertDeleteWord => syn_insert(app, KeyCode::Char('w'), KeyModifiers::CONTROL),
        InsertDeleteLine => syn_insert(app, KeyCode::Char('u'), KeyModifiers::CONTROL),
        InsertDeleteCharBefore => syn_insert(app, KeyCode::Char('h'), KeyModifiers::CONTROL),

        // ── Chord-resolved normal-mode actions ───────────────────────
        // For chords we synthesize each constituent key through the
        // legacy normal-mode handler so the existing PendingCommand state
        // machine resolves them. Avoids exposing per-action internals.
        GotoFirstRow => syn_normal_chord(app, &[(KeyCode::Char('g'), KeyModifiers::NONE); 2])?,
        ViewportTop => syn_normal_chord(
            app,
            &[
                (KeyCode::Char('z'), KeyModifiers::NONE),
                (KeyCode::Char('t'), KeyModifiers::NONE),
            ],
        )?,
        ViewportCenter => syn_normal_chord(
            app,
            &[
                (KeyCode::Char('z'), KeyModifiers::NONE),
                (KeyCode::Char('z'), KeyModifiers::NONE),
            ],
        )?,
        ViewportBottom => syn_normal_chord(
            app,
            &[
                (KeyCode::Char('z'), KeyModifiers::NONE),
                (KeyCode::Char('b'), KeyModifiers::NONE),
            ],
        )?,
        TitleCase => syn_normal_chord(
            app,
            &[
                (KeyCode::Char('g'), KeyModifiers::NONE),
                (KeyCode::Char('~'), KeyModifiers::NONE),
            ],
        )?,
        ToggleBoolean => syn_normal_chord(
            app,
            &[
                (KeyCode::Char('g'), KeyModifiers::NONE),
                (KeyCode::Char('.'), KeyModifiers::NONE),
            ],
        )?,
        RowDelete => syn_normal_chord(app, &[(KeyCode::Char('d'), KeyModifiers::NONE); 2])?,
        RowYank => syn_normal_chord(app, &[(KeyCode::Char('y'), KeyModifiers::NONE); 2])?,
        RowChange => syn_normal_chord(app, &[(KeyCode::Char('c'), KeyModifiers::NONE); 2])?,
        RowYankCellWord => syn_normal_chord(
            app,
            &[
                (KeyCode::Char('c'), KeyModifiers::NONE),
                (KeyCode::Char('w'), KeyModifiers::NONE),
            ],
        )?,
        RowSwapDown => syn_normal_chord(
            app,
            &[
                (KeyCode::Char('g'), KeyModifiers::NONE),
                (KeyCode::Char('j'), KeyModifiers::NONE),
            ],
        )?,
        RowSwapUp => syn_normal_chord(
            app,
            &[
                (KeyCode::Char('g'), KeyModifiers::NONE),
                (KeyCode::Char('k'), KeyModifiers::NONE),
            ],
        )?,
        ColDelete => syn_normal_chord(
            app,
            &[
                (KeyCode::Char(','), KeyModifiers::NONE),
                (KeyCode::Char('d'), KeyModifiers::NONE),
                (KeyCode::Char('d'), KeyModifiers::NONE),
            ],
        )?,
        ColYank => syn_normal_chord(
            app,
            &[
                (KeyCode::Char(','), KeyModifiers::NONE),
                (KeyCode::Char('y'), KeyModifiers::NONE),
                (KeyCode::Char('y'), KeyModifiers::NONE),
            ],
        )?,
        ColPasteRight => syn_normal_chord(
            app,
            &[
                (KeyCode::Char(','), KeyModifiers::NONE),
                (KeyCode::Char('p'), KeyModifiers::NONE),
            ],
        )?,
        ColPasteLeft => syn_normal_chord(
            app,
            &[
                (KeyCode::Char(','), KeyModifiers::NONE),
                (KeyCode::Char('P'), KeyModifiers::SHIFT),
            ],
        )?,
        ColInsertRight => syn_normal_chord(
            app,
            &[
                (KeyCode::Char(','), KeyModifiers::NONE),
                (KeyCode::Char('o'), KeyModifiers::NONE),
            ],
        )?,
        ColInsertLeft => syn_normal_chord(
            app,
            &[
                (KeyCode::Char(','), KeyModifiers::NONE),
                (KeyCode::Char('O'), KeyModifiers::SHIFT),
            ],
        )?,
        EnterVisualColumn => syn_normal_chord(
            app,
            &[
                (KeyCode::Char(','), KeyModifiers::NONE),
                (KeyCode::Char('v'), KeyModifiers::NONE),
            ],
        )?,
        EnterSqlEditor => syn_normal_chord(
            app,
            &[
                (KeyCode::Char(' '), KeyModifiers::NONE),
                (KeyCode::Char('q'), KeyModifiers::NONE),
            ],
        )?,
        EnterMagnifier => syn_normal_chord(
            app,
            &[
                (KeyCode::Char(' '), KeyModifiers::NONE),
                (KeyCode::Char('m'), KeyModifiers::NONE),
            ],
        )?,
        EnterFileList => syn_normal_chord(
            app,
            &[
                (KeyCode::Char(' '), KeyModifiers::NONE),
                (KeyCode::Char('f'), KeyModifiers::NONE),
            ],
        )?,
        ReselectVisual => syn_normal_chord(
            app,
            &[
                (KeyCode::Char('g'), KeyModifiers::NONE),
                (KeyCode::Char('v'), KeyModifiers::NONE),
            ],
        )?,
        MacroReplayLast => syn_normal_chord(app, &[(KeyCode::Char('@'), KeyModifiers::NONE); 2])?,

        // ── Visual mode (forward synthetic keys to the visual handler) ─
        VisualExit => syn_visual(app, KeyCode::Esc, KeyModifiers::NONE)?,
        VisualCursorLeft => syn_visual(app, KeyCode::Char('h'), KeyModifiers::NONE)?,
        VisualCursorRight => syn_visual(app, KeyCode::Char('l'), KeyModifiers::NONE)?,
        VisualCursorUp => syn_visual(app, KeyCode::Char('k'), KeyModifiers::NONE)?,
        VisualCursorDown => syn_visual(app, KeyCode::Char('j'), KeyModifiers::NONE)?,
        VisualGotoFirstRow => syn_visual_chord(
            app,
            &[
                (KeyCode::Char('g'), KeyModifiers::NONE),
                (KeyCode::Char('g'), KeyModifiers::NONE),
            ],
        )?,
        VisualGotoLastRow => syn_visual(app, KeyCode::Char('G'), KeyModifiers::SHIFT)?,
        VisualDelete => syn_visual(app, KeyCode::Char('d'), KeyModifiers::NONE)?,
        VisualYank => syn_visual(app, KeyCode::Char('y'), KeyModifiers::NONE)?,
        VisualYankSystem => syn_visual(app, KeyCode::Char('Y'), KeyModifiers::SHIFT)?,
        VisualPaste => syn_visual(app, KeyCode::Char('p'), KeyModifiers::NONE)?,
        VisualStats => syn_visual_chord(
            app,
            &[
                (KeyCode::Char('g'), KeyModifiers::NONE),
                (KeyCode::Char('s'), KeyModifiers::NONE),
            ],
        )?,

        // ── Command-mode line editor ─────────────────────────────────
        CmdExecute => syn_command(app, KeyCode::Enter, KeyModifiers::NONE)?,
        CmdCancel => syn_command(app, KeyCode::Esc, KeyModifiers::NONE)?,
        CmdDeleteCharBack => syn_command(app, KeyCode::Backspace, KeyModifiers::NONE)?,
        CmdDeleteCharForward => syn_command(app, KeyCode::Delete, KeyModifiers::NONE)?,
        CmdCursorLeft => syn_command(app, KeyCode::Left, KeyModifiers::NONE)?,
        CmdCursorRight => syn_command(app, KeyCode::Right, KeyModifiers::NONE)?,
        CmdCursorHome => syn_command(app, KeyCode::Home, KeyModifiers::NONE)?,
        CmdCursorEnd => syn_command(app, KeyCode::End, KeyModifiers::NONE)?,
        CmdHistoryPrev => syn_command(app, KeyCode::Up, KeyModifiers::NONE)?,
        CmdHistoryNext => syn_command(app, KeyCode::Down, KeyModifiers::NONE)?,

        // ── Search-mode line editor ──────────────────────────────────
        SearchSubmit => syn_search(app, KeyCode::Enter, KeyModifiers::NONE)?,
        SearchCancel => syn_search(app, KeyCode::Esc, KeyModifiers::NONE)?,
        SearchDeleteChar => syn_search(app, KeyCode::Backspace, KeyModifiers::NONE)?,

        // ── File operation prompt (rename/delete/etc.) ───────────────
        FileOpExecute => syn_file_op(app, KeyCode::Enter, KeyModifiers::NONE)?,
        FileOpCancel => syn_file_op(app, KeyCode::Esc, KeyModifiers::NONE)?,
        FileOpBackspace => syn_file_op(app, KeyCode::Backspace, KeyModifiers::NONE)?,

        // ── File list (`<space>f`) ───────────────────────────────────
        FileListExit => syn_file_list(app, KeyCode::Esc, KeyModifiers::NONE)?,
        FileListSearchEnter => syn_file_list(app, KeyCode::Char('/'), KeyModifiers::NONE)?,
        FileListShellPrompt => syn_file_list(app, KeyCode::Char(':'), KeyModifiers::NONE)?,
        FileListUp => syn_file_list(app, KeyCode::Char('k'), KeyModifiers::NONE)?,
        FileListDown => syn_file_list(app, KeyCode::Char('j'), KeyModifiers::NONE)?,
        FileListGotoBottom => syn_file_list(app, KeyCode::Char('G'), KeyModifiers::SHIFT)?,
        FileListOpen => syn_file_list(app, KeyCode::Enter, KeyModifiers::NONE)?,
        FileListParent => syn_file_list(app, KeyCode::Char('h'), KeyModifiers::NONE)?,
        FileListToggleHidden => syn_file_list(app, KeyCode::Char('.'), KeyModifiers::NONE)?,
        FileListToggleSpot => syn_file_list(app, KeyCode::Tab, KeyModifiers::NONE)?,
        FileListRename => syn_file_list(app, KeyCode::Char('r'), KeyModifiers::NONE)?,
        FileListDelete => syn_file_list(app, KeyCode::Char('d'), KeyModifiers::NONE)?,
        FileListMove => syn_file_list(app, KeyCode::Char('m'), KeyModifiers::NONE)?,
        FileListCopy => syn_file_list(app, KeyCode::Char('y'), KeyModifiers::NONE)?,
        FileListCreate => syn_file_list(app, KeyCode::Char('n'), KeyModifiers::NONE)?,

        // ── Phase 2 stretch: deeper SQL / magnifier / file-list-shell
        //    actions still fall through to the legacy match arms. They're
        //    reachable; remapping them is queued for a follow-up.
        _ => return Ok(None),
    }

    Ok(Some(InputResult::Continue))
}

/// Forward a sequence of synthetic keys through the legacy normal-mode
/// handler. Each key is fed in turn so the existing `PendingCommand`
/// state machine can resolve chord sequences (`gg`, `dd`, `,yy`, …).
fn syn_normal_chord(app: &mut App, keys: &[(KeyCode, KeyModifiers)]) -> Result<()> {
    for &(code, mods) in keys {
        let ev = KeyEvent::new(code, mods);
        let _ = crate::input::normal_mode::handle_raw(app, ev)?;
    }
    Ok(())
}

/// Forward a synthetic key into the visual-mode handler. Uses `_raw` to
/// bypass the keymap pre-pass and avoid recursion.
fn syn_visual(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
    let ev = KeyEvent::new(code, modifiers);
    let _ = crate::input::visual_mode::handler::handle_raw(app, ev)?;
    Ok(())
}

fn syn_visual_chord(app: &mut App, keys: &[(KeyCode, KeyModifiers)]) -> Result<()> {
    for &(code, mods) in keys {
        let ev = KeyEvent::new(code, mods);
        let _ = crate::input::visual_mode::handler::handle_raw(app, ev)?;
    }
    Ok(())
}

fn syn_command(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
    let ev = KeyEvent::new(code, modifiers);
    let _ = crate::input::command_mode::handler::handle_raw(app, ev)?;
    Ok(())
}

fn syn_search(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
    let ev = KeyEvent::new(code, modifiers);
    let _ = crate::input::search_mode::handle_raw(app, ev)?;
    Ok(())
}

fn syn_file_op(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
    let ev = KeyEvent::new(code, modifiers);
    let _ = crate::input::file_operation_mode::handle_raw(app, ev)?;
    Ok(())
}

fn syn_file_list(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
    let ev = KeyEvent::new(code, modifiers);
    let _ = crate::input::file_list_mode::handle_raw(app, ev)?;
    Ok(())
}

/// Run the action by feeding a synthetic key event back into the normal
/// mode handler. Used for actions that have a canonical single-key default.
fn syn_normal(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
    let ev = KeyEvent::new(code, modifiers);
    let _ = crate::input::normal_mode::handle_raw(app, ev)?;
    Ok(())
}

/// Run a navigation key (h/j/k/l, w/b/e, 0/$) by calling the navigation
/// commands directly.
fn syn_navigate(app: &mut App, code: KeyCode) -> Result<()> {
    crate::navigation::commands::handle_navigation(app, code)
}

/// Run the action by feeding a synthetic key event back into the insert
/// mode handler.
fn syn_insert(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    let ev = KeyEvent::new(code, modifiers);
    let _ = crate::input::insert_mode::handle_raw(app, ev);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Mode;

    fn make_test_app() -> App {
        let document = crate::csv::Document::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![
                vec!["a1".to_string(), "b1".to_string(), "c1".to_string()],
                vec!["a2".to_string(), "b2".to_string(), "c2".to_string()],
                vec!["a3".to_string(), "b3".to_string(), "c3".to_string()],
            ],
            "test.csv".to_string(),
        );
        App::new(
            document,
            vec![std::path::PathBuf::from("test.csv")],
            0,
            crate::session::FileConfig::default(),
        )
    }

    #[test]
    fn cursor_down_moves_selection_down() {
        let mut app = make_test_app();
        let start = app.view_state.table_state.selected().unwrap_or(0);
        let result = execute(&mut app, Action::CursorDown).unwrap();
        assert!(matches!(result, Some(InputResult::Continue)));
        let end = app.view_state.table_state.selected().unwrap_or(0);
        assert_eq!(end, start + 1, "CursorDown should move selection down");
    }

    #[test]
    fn cursor_up_moves_selection_up() {
        let mut app = make_test_app();
        // Move down first so we have somewhere to go up from.
        execute(&mut app, Action::CursorDown).unwrap();
        let start = app.view_state.table_state.selected().unwrap_or(0);
        execute(&mut app, Action::CursorUp).unwrap();
        let end = app.view_state.table_state.selected().unwrap_or(0);
        assert_eq!(end, start - 1);
    }

    #[test]
    fn cursor_right_advances_column() {
        let mut app = make_test_app();
        let start = app.view_state.selected_column.get();
        execute(&mut app, Action::CursorRight).unwrap();
        let end = app.view_state.selected_column.get();
        assert_eq!(end, start + 1);
    }

    #[test]
    fn cell_edit_at_end_enters_insert_mode() {
        let mut app = make_test_app();
        assert_eq!(app.mode, Mode::Normal);
        execute(&mut app, Action::CellEditAtEnd).unwrap();
        assert_eq!(app.mode, Mode::Insert);
    }

    #[test]
    fn enter_command_mode_switches_modes() {
        let mut app = make_test_app();
        execute(&mut app, Action::EnterCommandMode).unwrap();
        assert_eq!(app.mode, Mode::Command);
    }

    #[test]
    fn enter_visual_block_switches_modes() {
        let mut app = make_test_app();
        execute(&mut app, Action::EnterVisualBlock).unwrap();
        assert_eq!(app.mode, Mode::VisualBlock);
    }

    #[test]
    fn unwired_action_returns_none() {
        let mut app = make_test_app();
        // Pick an action that's intentionally still unwired in this phase
        // (deep magnifier internals — not reachable from any preset's
        // single-key bindings). Updating this list is fine if/when an
        // action gets wired.
        let result = execute(&mut app, Action::SqlHistoryPopupSelect).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn toggle_help_flips_overlay() {
        let mut app = make_test_app();
        let before = app.view_state.help_overlay_visible;
        execute(&mut app, Action::ToggleHelp).unwrap();
        let after = app.view_state.help_overlay_visible;
        assert_ne!(before, after);
    }

    #[test]
    fn insert_cancel_returns_to_normal() {
        let mut app = make_test_app();
        execute(&mut app, Action::CellEditAtEnd).unwrap();
        assert_eq!(app.mode, Mode::Insert);
        execute(&mut app, Action::InsertCancel).unwrap();
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn page_down_does_not_panic() {
        // PageDown's actual movement depends on the rendered viewport
        // height, which isn't available outside of `terminal.draw(…)`.
        // Just smoke-test that the dispatcher returns a result instead of
        // panicking when the action is invoked headlessly.
        let mut app = make_test_app();
        let result = execute(&mut app, Action::PageDown).unwrap();
        assert!(result.is_some());
    }

    // ── End-to-end keymap → handler integration ──────────────────────

    fn noop_handle_raw(_: &mut App, _: KeyEvent) -> Result<crate::input::InputResult> {
        Ok(crate::input::InputResult::Continue)
    }

    #[test]
    fn vim_default_j_routes_through_keymap() {
        let mut app = make_test_app();
        let start = app.view_state.table_state.selected().unwrap_or(0);
        let result = try_keymap(
            &mut app,
            KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
            crate::config::keys::KeymapScope::Normal,
            noop_handle_raw,
        )
        .unwrap();
        assert!(result.is_some(), "vim's `j` should be bound");
        let end = app.view_state.table_state.selected().unwrap_or(0);
        assert_eq!(end, start + 1);
    }

    #[test]
    fn user_remap_emacs_ctrl_n_to_cursor_down() {
        let mut app = make_test_app();
        let toml: crate::config::keys::KeymapToml = toml::from_str(
            r#"
            [meta]
            inherit = "vim"

            [normal]
            "ctrl+n" = "cursor_down"
            "ctrl+p" = "cursor_up"
        "#,
        )
        .unwrap();
        let mut warnings = Vec::new();
        app.keymap = crate::config::keys::Keymap::from_toml(&toml, &mut warnings);
        assert!(warnings.is_empty(), "warnings: {:?}", warnings);

        let start = app.view_state.table_state.selected().unwrap_or(0);
        let result = try_keymap(
            &mut app,
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
            crate::config::keys::KeymapScope::Normal,
            noop_handle_raw,
        )
        .unwrap();
        assert!(result.is_some(), "Ctrl-N should be bound after override");
        let end = app.view_state.table_state.selected().unwrap_or(0);
        assert_eq!(end, start + 1, "Ctrl-N should move cursor down");
    }

    #[test]
    fn unbound_single_key_returns_none() {
        let mut app = make_test_app();
        let result = try_keymap(
            &mut app,
            KeyEvent::new(KeyCode::Char('Z'), KeyModifiers::SHIFT),
            crate::config::keys::KeymapScope::Normal,
            noop_handle_raw,
        )
        .unwrap();
        assert!(result.is_none(), "unbound single key should fall through");
    }

    #[test]
    fn partial_chord_buffers_and_waits() {
        let mut app = make_test_app();
        let result = try_keymap(
            &mut app,
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE),
            crate::config::keys::KeymapScope::Normal,
            noop_handle_raw,
        )
        .unwrap();
        assert!(result.is_some(), "partial chord returns Some(Continue)");
        assert!(
            !app.input_state.chord_buffer.is_empty(),
            "chord buffer should hold the `g` atom"
        );
    }

    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Replay-counter handler. We can't close over a Vec from a `fn`, so
    /// we use a process-wide counter to verify that the keymap fed buffered
    /// keys back into the legacy handler.
    static REPLAY_COUNT: AtomicUsize = AtomicUsize::new(0);
    fn counting_handle_raw(_: &mut App, _: KeyEvent) -> Result<crate::input::InputResult> {
        REPLAY_COUNT.fetch_add(1, Ordering::SeqCst);
        Ok(crate::input::InputResult::Continue)
    }

    #[test]
    fn parametric_chord_replays_through_legacy_when_keymap_gives_up() {
        // `g{letter}` (column jump) is parametric and not in the keymap.
        // After `g` is buffered then `X` arrives, the keymap returns
        // Unbound for `gX`. Replay logic must feed `g` and `X` into the
        // legacy handler so PendingCommand state can rebuild.
        let mut app = make_test_app();

        let _ = try_keymap(
            &mut app,
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE),
            crate::config::keys::KeymapScope::Normal,
            counting_handle_raw,
        )
        .unwrap();
        assert_eq!(
            app.input_state.chord_buffer.len(),
            1,
            "`g` should be buffered after first keypress"
        );

        // Reset replay counter for the second-key test.
        REPLAY_COUNT.store(0, Ordering::SeqCst);

        // Press `X` — `gX` is not bound, keymap gives up + replays both.
        let result = try_keymap(
            &mut app,
            KeyEvent::new(KeyCode::Char('X'), KeyModifiers::SHIFT),
            crate::config::keys::KeymapScope::Normal,
            counting_handle_raw,
        )
        .unwrap();

        assert!(
            app.input_state.chord_buffer.is_empty(),
            "buffer must be drained on Unbound"
        );
        assert!(result.is_some(), "Unbound replay returns Some(Continue)");
        assert_eq!(
            REPLAY_COUNT.load(Ordering::SeqCst),
            2,
            "both buffered keys should have been replayed via handle_raw"
        );
    }

    #[test]
    fn excel_style_arrow_keys_navigate() {
        let mut app = make_test_app();
        let start = app.view_state.table_state.selected().unwrap_or(0);
        let result = try_keymap(
            &mut app,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            crate::config::keys::KeymapScope::Normal,
            noop_handle_raw,
        )
        .unwrap();
        assert!(result.is_some());
        let end = app.view_state.table_state.selected().unwrap_or(0);
        assert_eq!(end, start + 1);
    }

    #[test]
    fn keymap_respects_scope_when_in_insert_mode() {
        let mut app = make_test_app();
        execute(&mut app, Action::CellEditAtEnd).unwrap();
        assert_eq!(app.mode, crate::app::Mode::Insert);

        let result = try_keymap(
            &mut app,
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            crate::config::keys::KeymapScope::Insert,
            noop_handle_raw,
        )
        .unwrap();
        assert!(result.is_some());
        assert_eq!(app.mode, crate::app::Mode::Normal);
    }
}
