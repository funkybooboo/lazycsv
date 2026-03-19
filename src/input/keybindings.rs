//! Central keybinding registry for LazyCSV
//!
//! This module provides a single source of truth for all keybindings in the application.
//! All input handlers query this registry to determine which action to take for a given key press.

use crate::app::Mode;
use crossterm::event::{KeyCode, KeyModifiers};

/// All possible input actions in LazyCSV
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    // ========================================================================
    // NAVIGATION
    // ========================================================================
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    NavigateToFirstRow,
    NavigateToLastRow,
    NavigateToFirstColumn,
    NavigateToLastColumn,
    NavigateNextWord,
    NavigatePrevWord,
    NavigateEndWord,
    NavigatePageDown,
    NavigatePageUp,
    NavigateToLine(usize), // For :5g style commands
    NavigateToColumn,      // For :cA style commands

    // ========================================================================
    // VIEWPORT
    // ========================================================================
    ViewportTop,
    ViewportCenter,
    ViewportBottom,

    // ========================================================================
    // EDITING
    // ========================================================================
    EnterInsertMode,
    EnterInsertModeStart,
    EnterInsertModeEnd,
    EnterInsertModeReplace,
    DeleteCell,
    DeleteRow,
    YankRow,
    PasteRow,
    PasteRowAbove,
    InsertRowBelow,
    InsertRowAbove,
    CommitEdit,
    CancelEdit,

    // ========================================================================
    // COLUMN OPERATIONS
    // ========================================================================
    DeleteColumn,
    YankColumn,
    PasteColumn,
    PasteColumnLeft,
    InsertColumnRight,
    InsertColumnLeft,

    // ========================================================================
    // MODE TRANSITIONS
    // ========================================================================
    EnterVisualMode,
    EnterVisualLineMode,
    EnterVisualColumnMode,
    ExitVisualMode,
    EnterCommandMode,
    EnterSearchMode,
    EnterMagnifierMode,
    EnterSqlEditorMode,
    EnterFileListMode,
    ExitToNormalMode,

    // ========================================================================
    // FILE OPERATIONS
    // ========================================================================
    NextFile,
    PrevFile,
    SaveFile,
    SaveAndQuit,
    Quit,
    ForceQuit,

    // ========================================================================
    // SPECIAL
    // ========================================================================
    ToggleHelp,
    ClearSearch,
    SearchNext,
    SearchPrev,
    SearchCurrentCell,

    // ========================================================================
    // MAGNIFIER SPECIFIC
    // ========================================================================
    MagnifierNavigateUp,
    MagnifierNavigateDown,
    MagnifierNavigateLeft,
    MagnifierNavigateRight,

    // ========================================================================
    // FILE BROWSER SPECIFIC
    // ========================================================================
    FileBrowserParent,
    FileBrowserEnter,
    FileBrowserFilter,
    FileBrowserRename,
    FileBrowserDelete,
    FileBrowserMove,
    FileBrowserCopy,
    FileBrowserCreate,
}

/// A keybinding maps a key combination to an action in specific modes
#[derive(Debug, Clone)]
pub struct Keybinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
    pub action: InputAction,
    pub modes: &'static [Mode],
    pub description: &'static str,
}

/// Central registry of all keybindings
pub const KEYBINDINGS: &[Keybinding] = &[
    // ========================================================================
    // NAVIGATION - Works in Normal, Visual modes
    // ========================================================================
    Keybinding {
        key: KeyCode::Char('h'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::NavigateLeft,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Move left",
    },
    Keybinding {
        key: KeyCode::Left,
        modifiers: KeyModifiers::NONE,
        action: InputAction::NavigateLeft,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Move left",
    },
    Keybinding {
        key: KeyCode::Char('j'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::NavigateDown,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Move down",
    },
    Keybinding {
        key: KeyCode::Down,
        modifiers: KeyModifiers::NONE,
        action: InputAction::NavigateDown,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Move down",
    },
    Keybinding {
        key: KeyCode::Char('k'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::NavigateUp,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Move up",
    },
    Keybinding {
        key: KeyCode::Up,
        modifiers: KeyModifiers::NONE,
        action: InputAction::NavigateUp,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Move up",
    },
    Keybinding {
        key: KeyCode::Char('l'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::NavigateRight,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Move right",
    },
    Keybinding {
        key: KeyCode::Right,
        modifiers: KeyModifiers::NONE,
        action: InputAction::NavigateRight,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Move right",
    },
    Keybinding {
        key: KeyCode::Char('w'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::NavigateNextWord,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Jump to next non-empty cell",
    },
    Keybinding {
        key: KeyCode::Char('b'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::NavigatePrevWord,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Jump to previous non-empty cell",
    },
    Keybinding {
        key: KeyCode::Char('e'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::NavigateEndWord,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Jump to last non-empty cell",
    },
    Keybinding {
        key: KeyCode::Char('0'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::NavigateToFirstColumn,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Go to first column",
    },
    Keybinding {
        key: KeyCode::Char('$'),
        modifiers: KeyModifiers::SHIFT,
        action: InputAction::NavigateToLastColumn,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Go to last column",
    },
    Keybinding {
        key: KeyCode::Char('G'),
        modifiers: KeyModifiers::SHIFT,
        action: InputAction::NavigateToLastRow,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Go to last row",
    },
    Keybinding {
        key: KeyCode::Char('d'),
        modifiers: KeyModifiers::CONTROL,
        action: InputAction::NavigatePageDown,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Page down",
    },
    Keybinding {
        key: KeyCode::Char('u'),
        modifiers: KeyModifiers::CONTROL,
        action: InputAction::NavigatePageUp,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Page up",
    },
    Keybinding {
        key: KeyCode::Home,
        modifiers: KeyModifiers::NONE,
        action: InputAction::NavigateToFirstColumn,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Go to first column",
    },
    Keybinding {
        key: KeyCode::End,
        modifiers: KeyModifiers::NONE,
        action: InputAction::NavigateToLastColumn,
        modes: &[
            Mode::Normal,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
        ],
        description: "Go to last column",
    },
    // ========================================================================
    // VIEWPORT COMMANDS
    // Note: zt, zz, zb are multi-key commands handled by pending command system
    // ========================================================================
    // EDITING - Normal mode
    // ========================================================================
    Keybinding {
        key: KeyCode::Char('i'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::EnterInsertMode,
        modes: &[Mode::Normal],
        description: "Enter insert mode",
    },
    Keybinding {
        key: KeyCode::Char('a'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::EnterInsertMode,
        modes: &[Mode::Normal],
        description: "Enter insert mode (append)",
    },
    Keybinding {
        key: KeyCode::Char('I'),
        modifiers: KeyModifiers::SHIFT,
        action: InputAction::EnterInsertModeStart,
        modes: &[Mode::Normal],
        description: "Enter insert mode at start",
    },
    Keybinding {
        key: KeyCode::Char('A'),
        modifiers: KeyModifiers::SHIFT,
        action: InputAction::EnterInsertModeEnd,
        modes: &[Mode::Normal],
        description: "Enter insert mode at end",
    },
    Keybinding {
        key: KeyCode::Char('s'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::EnterInsertModeReplace,
        modes: &[Mode::Normal],
        description: "Replace cell (clear and edit)",
    },
    Keybinding {
        key: KeyCode::F(2),
        modifiers: KeyModifiers::NONE,
        action: InputAction::EnterInsertMode,
        modes: &[Mode::Normal],
        description: "Enter insert mode (F2)",
    },
    Keybinding {
        key: KeyCode::Delete,
        modifiers: KeyModifiers::NONE,
        action: InputAction::DeleteCell,
        modes: &[Mode::Normal],
        description: "Clear cell",
    },
    Keybinding {
        key: KeyCode::Char('o'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::InsertRowBelow,
        modes: &[Mode::Normal],
        description: "Insert row below",
    },
    Keybinding {
        key: KeyCode::Char('O'),
        modifiers: KeyModifiers::SHIFT,
        action: InputAction::InsertRowAbove,
        modes: &[Mode::Normal],
        description: "Insert row above",
    },
    Keybinding {
        key: KeyCode::Char('p'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::PasteRow,
        modes: &[Mode::Normal],
        description: "Paste row below",
    },
    Keybinding {
        key: KeyCode::Char('P'),
        modifiers: KeyModifiers::SHIFT,
        action: InputAction::PasteRowAbove,
        modes: &[Mode::Normal],
        description: "Paste row above",
    },
    // ========================================================================
    // MODE TRANSITIONS
    // ========================================================================
    Keybinding {
        key: KeyCode::Esc,
        modifiers: KeyModifiers::NONE,
        action: InputAction::ExitToNormalMode,
        modes: &[
            Mode::Insert,
            Mode::VisualBlock,
            Mode::VisualLine,
            Mode::VisualColumn,
            Mode::Command,
            Mode::Search,
        ],
        description: "Return to normal mode",
    },
    Keybinding {
        key: KeyCode::Char(':'),
        modifiers: KeyModifiers::SHIFT,
        action: InputAction::EnterCommandMode,
        modes: &[Mode::Normal],
        description: "Enter command mode",
    },
    Keybinding {
        key: KeyCode::Char('/'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::EnterSearchMode,
        modes: &[Mode::Normal],
        description: "Enter search mode",
    },
    Keybinding {
        key: KeyCode::Char('v'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::EnterVisualMode,
        modes: &[Mode::Normal],
        description: "Enter visual mode",
    },
    Keybinding {
        key: KeyCode::Char('V'),
        modifiers: KeyModifiers::SHIFT,
        action: InputAction::EnterVisualLineMode,
        modes: &[Mode::Normal],
        description: "Enter visual line mode",
    },
    Keybinding {
        key: KeyCode::Char('m'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::EnterMagnifierMode,
        modes: &[Mode::Normal],
        description: "Enter magnifier mode (Space+m)",
    },
    Keybinding {
        key: KeyCode::Char('q'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::EnterSqlEditorMode,
        modes: &[Mode::Normal],
        description: "Enter SQL editor (Space+q)",
    },
    Keybinding {
        key: KeyCode::Char('f'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::EnterFileListMode,
        modes: &[Mode::Normal],
        description: "Enter file browser (Space+f)",
    },
    // ========================================================================
    // HELP & SEARCH
    // ========================================================================
    Keybinding {
        key: KeyCode::Char('?'),
        modifiers: KeyModifiers::SHIFT,
        action: InputAction::ToggleHelp,
        modes: &[Mode::Normal],
        description: "Toggle help",
    },
    Keybinding {
        key: KeyCode::Char('n'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::SearchNext,
        modes: &[Mode::Normal],
        description: "Next search match",
    },
    Keybinding {
        key: KeyCode::Char('N'),
        modifiers: KeyModifiers::SHIFT,
        action: InputAction::SearchPrev,
        modes: &[Mode::Normal],
        description: "Previous search match",
    },
    Keybinding {
        key: KeyCode::Char('*'),
        modifiers: KeyModifiers::SHIFT,
        action: InputAction::SearchCurrentCell,
        modes: &[Mode::Normal],
        description: "Search for current cell content",
    },
    // ========================================================================
    // FILE NAVIGATION
    // ========================================================================
    Keybinding {
        key: KeyCode::Char(']'),
        modifiers: KeyModifiers::NONE,
        action: InputAction::NextFile,
        modes: &[Mode::Normal],
        description: "Next file",
    },
    Keybinding {
        key: KeyCode::Char('['),
        modifiers: KeyModifiers::NONE,
        action: InputAction::PrevFile,
        modes: &[Mode::Normal],
        description: "Previous file",
    },
];

/// Get the action for a given key press in a specific mode
pub fn get_action(key: KeyCode, modifiers: KeyModifiers, mode: Mode) -> Option<InputAction> {
    KEYBINDINGS
        .iter()
        .find(|binding| {
            binding.key == key && binding.modifiers == modifiers && binding.modes.contains(&mode)
        })
        .map(|binding| binding.action)
}

/// Get all keybindings for a specific mode
pub fn get_keybindings_for_mode(mode: Mode) -> Vec<&'static Keybinding> {
    KEYBINDINGS
        .iter()
        .filter(|binding| binding.modes.contains(&mode))
        .collect()
}

/// Generate help text for a specific mode
pub fn get_help_text_for_mode(mode: Mode) -> String {
    let bindings = get_keybindings_for_mode(mode);
    let mut text = format!("Keybindings for {:?} mode:\n\n", mode);

    for binding in bindings {
        let key_str = format!("{:?}", binding.key);
        text.push_str(&format!("  {:<20} {}\n", key_str, binding.description));
    }

    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_action_normal_mode_h() {
        let action = get_action(KeyCode::Char('h'), KeyModifiers::NONE, Mode::Normal);
        assert_eq!(action, Some(InputAction::NavigateLeft));
    }

    #[test]
    fn test_get_action_normal_mode_j() {
        let action = get_action(KeyCode::Char('j'), KeyModifiers::NONE, Mode::Normal);
        assert_eq!(action, Some(InputAction::NavigateDown));
    }

    #[test]
    fn test_get_action_arrows() {
        assert_eq!(
            get_action(KeyCode::Left, KeyModifiers::NONE, Mode::Normal),
            Some(InputAction::NavigateLeft)
        );
        assert_eq!(
            get_action(KeyCode::Down, KeyModifiers::NONE, Mode::Normal),
            Some(InputAction::NavigateDown)
        );
        assert_eq!(
            get_action(KeyCode::Up, KeyModifiers::NONE, Mode::Normal),
            Some(InputAction::NavigateUp)
        );
        assert_eq!(
            get_action(KeyCode::Right, KeyModifiers::NONE, Mode::Normal),
            Some(InputAction::NavigateRight)
        );
    }

    #[test]
    fn test_esc_returns_to_normal() {
        // Verify Esc is bound in all non-normal modes
        assert_eq!(
            get_action(KeyCode::Esc, KeyModifiers::NONE, Mode::Insert),
            Some(InputAction::ExitToNormalMode)
        );
        assert_eq!(
            get_action(KeyCode::Esc, KeyModifiers::NONE, Mode::VisualBlock),
            Some(InputAction::ExitToNormalMode)
        );
        assert_eq!(
            get_action(KeyCode::Esc, KeyModifiers::NONE, Mode::Command),
            Some(InputAction::ExitToNormalMode)
        );
        assert_eq!(
            get_action(KeyCode::Esc, KeyModifiers::NONE, Mode::Search),
            Some(InputAction::ExitToNormalMode)
        );
    }

    #[test]
    fn test_mode_transitions() {
        assert_eq!(
            get_action(KeyCode::Char(':'), KeyModifiers::SHIFT, Mode::Normal),
            Some(InputAction::EnterCommandMode)
        );
        assert_eq!(
            get_action(KeyCode::Char('/'), KeyModifiers::NONE, Mode::Normal),
            Some(InputAction::EnterSearchMode)
        );
        assert_eq!(
            get_action(KeyCode::Char('v'), KeyModifiers::NONE, Mode::Normal),
            Some(InputAction::EnterVisualMode)
        );
    }

    #[test]
    fn test_get_keybindings_for_mode() {
        let normal_bindings = get_keybindings_for_mode(Mode::Normal);
        assert!(!normal_bindings.is_empty());

        // Verify all returned bindings include Normal mode
        for binding in normal_bindings {
            assert!(binding.modes.contains(&Mode::Normal));
        }
    }

    #[test]
    fn test_no_duplicate_keybindings_per_mode() {
        // For each mode, ensure no key+modifier combination is bound twice
        for mode in [
            Mode::Normal,
            Mode::Insert,
            Mode::VisualBlock,
            Mode::Command,
            Mode::Search,
        ] {
            let bindings = get_keybindings_for_mode(mode);
            let mut seen = std::collections::HashSet::new();

            for binding in bindings {
                let key_combo = (binding.key, binding.modifiers);
                assert!(
                    seen.insert(key_combo),
                    "Duplicate keybinding in {:?} mode: {:?} + {:?}",
                    mode,
                    binding.key,
                    binding.modifiers
                );
            }
        }
    }
}
