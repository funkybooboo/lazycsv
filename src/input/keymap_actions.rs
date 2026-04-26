//! Catalog of keymap-dispatchable actions.
//!
//! Every user-facing behaviour that a keypress can trigger is named here.
//! The `Action` enum + its `name() / from_name()` registry are the public
//! API surface for `keys.toml` files: any string ID listed here can appear
//! as the value of a binding entry.
//!
//! # Adding a new action
//!
//! 1. Add the `(VariantName, "snake_case_id")` row to the [`define_actions!`]
//!    invocation below.
//! 2. Wire it into the dispatcher (e.g. `src/input/normal_mode/handler.rs`)
//!    so the key actually does something.
//! 3. (Optional) Add the binding to `keymaps/vim.toml` if it should be on
//!    by default.
//!
//! Action IDs are part of the public API once shipped — renaming requires
//! a deprecation alias.
//!
//! # Excluded by design
//!
//! - Plain text-input keystrokes (typing into a buffer in command/search/
//!   insert/file-operation modes). Those are not key-mappable; the keymap
//!   lookup falls through and the handler's text-input path takes over.
//! - Internal pending-key state (e.g. waiting for the second `g` of `gg`).
//!   The keymap matches on the *resolved* multi-key sequence directly, so
//!   `gg → GotoFirstRow` rather than two intermediate states.

#![allow(dead_code)]

/// A keymap-dispatchable user action.
///
/// Variants intentionally avoid carrying parameters; counts, register
/// letters, and similar runtime state come from the input state, not from
/// the action itself. This keeps the enum simple to round-trip through TOML.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    // ── Cross-mode ───────────────────────────────────────────────────
    Quit,
    QuitForce,
    Save,
    SaveQuit,
    ReloadFile,
    ToggleHelp,

    // ── Normal mode: cursor / navigation ─────────────────────────────
    CursorLeft,
    CursorRight,
    CursorDown,
    CursorUp,
    CursorWordForward,
    CursorWordBackward,
    CursorWordEnd,
    GotoFirstRow,
    GotoLastRow,
    GotoLine,
    GotoFirstColumn,
    GotoLastColumn,
    GotoColumn,
    PageDown,
    PageUp,
    HalfPageDown,
    HalfPageUp,
    ViewportTop,
    ViewportCenter,
    ViewportBottom,

    // ── Normal mode: cell editing ────────────────────────────────────
    CellEditAtEnd,
    CellEditAtStart,
    CellEditAtLineEnd,
    CellReplace,
    CellReplaceF2,
    CellClear,
    ToggleCase,
    TitleCase,
    ToggleBoolean,
    Undo,
    Redo,
    RepeatLastEdit,

    // ── Normal mode: row operations ──────────────────────────────────
    RowInsertBelow,
    RowInsertAbove,
    RowDelete,
    RowYank,
    RowYankCellWord,
    RowChange,
    RowPasteBelow,
    RowPasteAbove,
    RowSwapDown,
    RowSwapUp,

    // ── Normal mode: column operations ───────────────────────────────
    ColDelete,
    ColYank,
    ColPasteRight,
    ColPasteLeft,
    ColInsertRight,
    ColInsertLeft,
    ColWidthIncrease,
    ColWidthDecrease,

    // ── Normal mode: search ──────────────────────────────────────────
    SearchEnter,
    SearchNext,
    SearchPrev,
    SearchCurrentCell,
    ClearSearch,

    // ── Normal mode: mode entry ──────────────────────────────────────
    EnterCommandMode,
    EnterVisualBlock,
    EnterVisualLine,
    EnterVisualColumn,
    EnterSqlEditor,
    EnterMagnifier,
    EnterFileList,
    ReselectVisual,

    // ── Normal mode: macros ──────────────────────────────────────────
    MacroRecordToggle,
    MacroReplayPrompt,
    MacroReplayLast,

    // ── Insert mode ──────────────────────────────────────────────────
    InsertCommitDown,
    InsertCommitUp,
    InsertCommitLeft,
    InsertCommitRight,
    InsertCommitTab,
    InsertCommitBackTab,
    InsertCancel,
    InsertCursorLeft,
    InsertCursorRight,
    InsertCursorHome,
    InsertCursorEnd,
    InsertDeleteBackward,
    InsertDeleteForward,
    InsertDeleteWord,
    InsertDeleteLine,
    InsertDeleteCharBefore,

    // ── Visual modes (block/line/column share these) ─────────────────
    VisualExit,
    VisualCursorLeft,
    VisualCursorRight,
    VisualCursorDown,
    VisualCursorUp,
    VisualGotoFirstRow,
    VisualGotoLastRow,
    VisualDelete,
    VisualYank,
    VisualYankSystem,
    VisualPaste,
    VisualStats,

    // ── Command mode (`:` line) ──────────────────────────────────────
    CmdExecute,
    CmdCancel,
    CmdDeleteCharBack,
    CmdDeleteCharForward,
    CmdCursorLeft,
    CmdCursorRight,
    CmdCursorHome,
    CmdCursorEnd,
    CmdHistoryPrev,
    CmdHistoryNext,

    // ── Search mode (`/` line) ───────────────────────────────────────
    SearchSubmit,
    SearchCancel,
    SearchDeleteChar,

    // ── Magnifier mode ───────────────────────────────────────────────
    MagExit,
    MagNavigateLeft,
    MagNavigateRight,
    MagNavigateUp,
    MagNavigateDown,
    MagRedo,
    MagGotoFirstLine,
    MagDeleteLine,
    MagYankLine,
    MagChangeLine,
    MagSaveAndClose,
    MagIndentRight,
    MagIndentLeft,
    MagFindForward,
    MagFindBackward,
    MagTillForward,
    MagTillBackward,
    MagReplaceChar,
    MagEnterInsert,
    MagEnterCommand,
    MagEnterVisual,
    MagSearch,
    MagInsertExit,
    MagInsertBackspace,
    MagInsertDelete,
    MagInsertNewline,
    MagInsertCursorLeft,
    MagInsertCursorRight,
    MagInsertCursorUp,
    MagInsertCursorDown,
    MagInsertCursorHome,
    MagInsertCursorEnd,
    MagCmdExit,
    MagCmdExecute,
    MagVisualExit,
    MagVisualMoveLeft,
    MagVisualMoveRight,
    MagVisualMoveUp,
    MagVisualMoveDown,

    // ── File list mode ───────────────────────────────────────────────
    FileListExit,
    FileListSearchEnter,
    FileListShellPrompt,
    FileListUp,
    FileListDown,
    FileListGotoTop,
    FileListGotoBottom,
    FileListOpen,
    FileListParent,
    FileListToggleHidden,
    FileListToggleSpot,
    FileListRename,
    FileListDelete,
    FileListMove,
    FileListCopy,
    FileListCreate,
    FileSearchExit,
    FileSearchApply,
    FileSearchBackspace,
    ShellCancel,
    ShellExecute,
    ShellHistoryPrev,
    ShellHistoryNext,
    ShellBackspace,
    ShellCursorLeft,
    ShellCursorRight,
    ShellCursorHome,
    ShellCursorEnd,

    // ── SQL editor mode ──────────────────────────────────────────────
    SqlExit,
    SqlExecute,
    SqlFormat,
    SqlContextCompletion,
    SqlHistoryPopupOpen,
    SqlHistoryPopupUp,
    SqlHistoryPopupDown,
    SqlHistoryPopupSelect,
    SqlHistoryPopupDelete,
    SqlHistoryPopupClose,
    SqlCompletionNext,
    SqlCompletionPrev,
    SqlCompletionAccept,
    SqlCompletionDismiss,
    SqlHelp,

    // ── File operation prompts ───────────────────────────────────────
    FileOpCancel,
    FileOpExecute,
    FileOpBackspace,
}

impl Action {
    /// The stable string identifier used in `keys.toml`. Returning
    /// `&'static str` keeps the registry zero-allocation.
    pub fn name(&self) -> &'static str {
        action_name(self)
    }

    /// Look up an action by its string identifier. Returns `None` for
    /// unknown names so keymap loaders can warn-and-skip rather than fail.
    pub fn from_name(s: &str) -> Option<Action> {
        action_from_name(s)
    }

    /// Slice over every variant — useful for `:keys` listings and tests.
    pub fn all() -> &'static [Action] {
        ALL_ACTIONS
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// The canonical (variant ↔ name) table. Keep alphabetically grouped within
/// each mode block above so adding entries is mechanical.
macro_rules! action_table {
    ( $( ($variant:ident, $name:literal) ),* $(,)? ) => {
        const ALL_ACTIONS: &[Action] = &[ $( Action::$variant ),* ];

        fn action_name(a: &Action) -> &'static str {
            match a { $( Action::$variant => $name ),* }
        }

        fn action_from_name(s: &str) -> Option<Action> {
            match s {
                $( $name => Some(Action::$variant), )*
                _ => None,
            }
        }
    };
}

action_table! {
    (Quit, "quit"),
    (QuitForce, "quit_force"),
    (Save, "save"),
    (SaveQuit, "save_quit"),
    (ReloadFile, "reload_file"),
    (ToggleHelp, "toggle_help"),

    (CursorLeft, "cursor_left"),
    (CursorRight, "cursor_right"),
    (CursorDown, "cursor_down"),
    (CursorUp, "cursor_up"),
    (CursorWordForward, "cursor_word_forward"),
    (CursorWordBackward, "cursor_word_backward"),
    (CursorWordEnd, "cursor_word_end"),
    (GotoFirstRow, "goto_first_row"),
    (GotoLastRow, "goto_last_row"),
    (GotoLine, "goto_line"),
    (GotoFirstColumn, "goto_first_column"),
    (GotoLastColumn, "goto_last_column"),
    (GotoColumn, "goto_column"),
    (PageDown, "page_down"),
    (PageUp, "page_up"),
    (HalfPageDown, "half_page_down"),
    (HalfPageUp, "half_page_up"),
    (ViewportTop, "viewport_top"),
    (ViewportCenter, "viewport_center"),
    (ViewportBottom, "viewport_bottom"),

    (CellEditAtEnd, "cell_edit_at_end"),
    (CellEditAtStart, "cell_edit_at_start"),
    (CellEditAtLineEnd, "cell_edit_at_line_end"),
    (CellReplace, "cell_replace"),
    (CellReplaceF2, "cell_replace_f2"),
    (CellClear, "cell_clear"),
    (ToggleCase, "toggle_case"),
    (TitleCase, "title_case"),
    (ToggleBoolean, "toggle_boolean"),
    (Undo, "undo"),
    (Redo, "redo"),
    (RepeatLastEdit, "repeat_last_edit"),

    (RowInsertBelow, "row_insert_below"),
    (RowInsertAbove, "row_insert_above"),
    (RowDelete, "row_delete"),
    (RowYank, "row_yank"),
    (RowYankCellWord, "row_yank_cell_word"),
    (RowChange, "row_change"),
    (RowPasteBelow, "row_paste_below"),
    (RowPasteAbove, "row_paste_above"),
    (RowSwapDown, "row_swap_down"),
    (RowSwapUp, "row_swap_up"),

    (ColDelete, "col_delete"),
    (ColYank, "col_yank"),
    (ColPasteRight, "col_paste_right"),
    (ColPasteLeft, "col_paste_left"),
    (ColInsertRight, "col_insert_right"),
    (ColInsertLeft, "col_insert_left"),
    (ColWidthIncrease, "col_width_increase"),
    (ColWidthDecrease, "col_width_decrease"),

    (SearchEnter, "search_enter"),
    (SearchNext, "search_next"),
    (SearchPrev, "search_prev"),
    (SearchCurrentCell, "search_current_cell"),
    (ClearSearch, "clear_search"),

    (EnterCommandMode, "enter_command_mode"),
    (EnterVisualBlock, "enter_visual_block"),
    (EnterVisualLine, "enter_visual_line"),
    (EnterVisualColumn, "enter_visual_column"),
    (EnterSqlEditor, "enter_sql_editor"),
    (EnterMagnifier, "enter_magnifier"),
    (EnterFileList, "enter_file_list"),
    (ReselectVisual, "reselect_visual"),

    (MacroRecordToggle, "macro_record_toggle"),
    (MacroReplayPrompt, "macro_replay_prompt"),
    (MacroReplayLast, "macro_replay_last"),

    (InsertCommitDown, "insert_commit_down"),
    (InsertCommitUp, "insert_commit_up"),
    (InsertCommitLeft, "insert_commit_left"),
    (InsertCommitRight, "insert_commit_right"),
    (InsertCommitTab, "insert_commit_tab"),
    (InsertCommitBackTab, "insert_commit_back_tab"),
    (InsertCancel, "insert_cancel"),
    (InsertCursorLeft, "insert_cursor_left"),
    (InsertCursorRight, "insert_cursor_right"),
    (InsertCursorHome, "insert_cursor_home"),
    (InsertCursorEnd, "insert_cursor_end"),
    (InsertDeleteBackward, "insert_delete_backward"),
    (InsertDeleteForward, "insert_delete_forward"),
    (InsertDeleteWord, "insert_delete_word"),
    (InsertDeleteLine, "insert_delete_line"),
    (InsertDeleteCharBefore, "insert_delete_char_before"),

    (VisualExit, "visual_exit"),
    (VisualCursorLeft, "visual_cursor_left"),
    (VisualCursorRight, "visual_cursor_right"),
    (VisualCursorDown, "visual_cursor_down"),
    (VisualCursorUp, "visual_cursor_up"),
    (VisualGotoFirstRow, "visual_goto_first_row"),
    (VisualGotoLastRow, "visual_goto_last_row"),
    (VisualDelete, "visual_delete"),
    (VisualYank, "visual_yank"),
    (VisualYankSystem, "visual_yank_system"),
    (VisualPaste, "visual_paste"),
    (VisualStats, "visual_stats"),

    (CmdExecute, "cmd_execute"),
    (CmdCancel, "cmd_cancel"),
    (CmdDeleteCharBack, "cmd_delete_char_back"),
    (CmdDeleteCharForward, "cmd_delete_char_forward"),
    (CmdCursorLeft, "cmd_cursor_left"),
    (CmdCursorRight, "cmd_cursor_right"),
    (CmdCursorHome, "cmd_cursor_home"),
    (CmdCursorEnd, "cmd_cursor_end"),
    (CmdHistoryPrev, "cmd_history_prev"),
    (CmdHistoryNext, "cmd_history_next"),

    (SearchSubmit, "search_submit"),
    (SearchCancel, "search_cancel"),
    (SearchDeleteChar, "search_delete_char"),

    (MagExit, "mag_exit"),
    (MagNavigateLeft, "mag_navigate_left"),
    (MagNavigateRight, "mag_navigate_right"),
    (MagNavigateUp, "mag_navigate_up"),
    (MagNavigateDown, "mag_navigate_down"),
    (MagRedo, "mag_redo"),
    (MagGotoFirstLine, "mag_goto_first_line"),
    (MagDeleteLine, "mag_delete_line"),
    (MagYankLine, "mag_yank_line"),
    (MagChangeLine, "mag_change_line"),
    (MagSaveAndClose, "mag_save_and_close"),
    (MagIndentRight, "mag_indent_right"),
    (MagIndentLeft, "mag_indent_left"),
    (MagFindForward, "mag_find_forward"),
    (MagFindBackward, "mag_find_backward"),
    (MagTillForward, "mag_till_forward"),
    (MagTillBackward, "mag_till_backward"),
    (MagReplaceChar, "mag_replace_char"),
    (MagEnterInsert, "mag_enter_insert"),
    (MagEnterCommand, "mag_enter_command"),
    (MagEnterVisual, "mag_enter_visual"),
    (MagSearch, "mag_search"),
    (MagInsertExit, "mag_insert_exit"),
    (MagInsertBackspace, "mag_insert_backspace"),
    (MagInsertDelete, "mag_insert_delete"),
    (MagInsertNewline, "mag_insert_newline"),
    (MagInsertCursorLeft, "mag_insert_cursor_left"),
    (MagInsertCursorRight, "mag_insert_cursor_right"),
    (MagInsertCursorUp, "mag_insert_cursor_up"),
    (MagInsertCursorDown, "mag_insert_cursor_down"),
    (MagInsertCursorHome, "mag_insert_cursor_home"),
    (MagInsertCursorEnd, "mag_insert_cursor_end"),
    (MagCmdExit, "mag_cmd_exit"),
    (MagCmdExecute, "mag_cmd_execute"),
    (MagVisualExit, "mag_visual_exit"),
    (MagVisualMoveLeft, "mag_visual_move_left"),
    (MagVisualMoveRight, "mag_visual_move_right"),
    (MagVisualMoveUp, "mag_visual_move_up"),
    (MagVisualMoveDown, "mag_visual_move_down"),

    (FileListExit, "file_list_exit"),
    (FileListSearchEnter, "file_list_search_enter"),
    (FileListShellPrompt, "file_list_shell_prompt"),
    (FileListUp, "file_list_up"),
    (FileListDown, "file_list_down"),
    (FileListGotoTop, "file_list_goto_top"),
    (FileListGotoBottom, "file_list_goto_bottom"),
    (FileListOpen, "file_list_open"),
    (FileListParent, "file_list_parent"),
    (FileListToggleHidden, "file_list_toggle_hidden"),
    (FileListToggleSpot, "file_list_toggle_spot"),
    (FileListRename, "file_list_rename"),
    (FileListDelete, "file_list_delete"),
    (FileListMove, "file_list_move"),
    (FileListCopy, "file_list_copy"),
    (FileListCreate, "file_list_create"),
    (FileSearchExit, "file_search_exit"),
    (FileSearchApply, "file_search_apply"),
    (FileSearchBackspace, "file_search_backspace"),
    (ShellCancel, "shell_cancel"),
    (ShellExecute, "shell_execute"),
    (ShellHistoryPrev, "shell_history_prev"),
    (ShellHistoryNext, "shell_history_next"),
    (ShellBackspace, "shell_backspace"),
    (ShellCursorLeft, "shell_cursor_left"),
    (ShellCursorRight, "shell_cursor_right"),
    (ShellCursorHome, "shell_cursor_home"),
    (ShellCursorEnd, "shell_cursor_end"),

    (SqlExit, "sql_exit"),
    (SqlExecute, "sql_execute"),
    (SqlFormat, "sql_format"),
    (SqlContextCompletion, "sql_context_completion"),
    (SqlHistoryPopupOpen, "sql_history_popup_open"),
    (SqlHistoryPopupUp, "sql_history_popup_up"),
    (SqlHistoryPopupDown, "sql_history_popup_down"),
    (SqlHistoryPopupSelect, "sql_history_popup_select"),
    (SqlHistoryPopupDelete, "sql_history_popup_delete"),
    (SqlHistoryPopupClose, "sql_history_popup_close"),
    (SqlCompletionNext, "sql_completion_next"),
    (SqlCompletionPrev, "sql_completion_prev"),
    (SqlCompletionAccept, "sql_completion_accept"),
    (SqlCompletionDismiss, "sql_completion_dismiss"),
    (SqlHelp, "sql_help"),

    (FileOpCancel, "file_op_cancel"),
    (FileOpExecute, "file_op_execute"),
    (FileOpBackspace, "file_op_backspace"),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_round_trip() {
        for action in Action::all() {
            let name = action.name();
            let resolved = Action::from_name(name)
                .unwrap_or_else(|| panic!("from_name({:?}) returned None", name));
            assert_eq!(*action, resolved, "round trip failed for {:?}", name);
        }
    }

    #[test]
    fn names_are_snake_case() {
        for action in Action::all() {
            let name = action.name();
            assert!(!name.is_empty(), "action {:?} has empty name", action);
            assert!(
                name.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "action {:?} has non-snake_case name {:?}",
                action,
                name
            );
        }
    }

    #[test]
    fn names_are_unique() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for action in Action::all() {
            let name = action.name();
            assert!(seen.insert(name), "duplicate action name {:?}", name);
        }
    }

    #[test]
    fn unknown_name_returns_none() {
        assert!(Action::from_name("definitely_not_an_action").is_none());
        assert!(Action::from_name("").is_none());
    }

    #[test]
    fn display_matches_name() {
        for action in Action::all() {
            assert_eq!(format!("{}", action), action.name());
        }
    }

    #[test]
    fn catalog_size_matches_phase_plan() {
        // Sanity check: the enum should at least cover the catalog.
        // 175+ actions is roughly what we expect after pruning text-input
        // pseudo-actions. Loose lower bound; tighten if it ever shrinks.
        assert!(
            Action::all().len() >= 150,
            "catalog has shrunk: {} actions",
            Action::all().len()
        );
    }
}
