//! Formula completion popup for Insert Mode
//!
//! When the user types '=' at the start of a cell, a completion popup appears
//! listing all available formula functions (SUM, AVERAGE, etc.). The popup
//! filters as the user types and allows selection with arrow keys.

use crate::app::{App, CompletionItem, CompletionKind, SqlCompletion};

/// All supported formula function names with descriptions shown as completion items.
const FORMULA_FUNCTIONS: &[(&str, &str)] = &[
    ("SUM", "Sum of values"),
    ("AVERAGE", "Arithmetic mean"),
    ("MIN", "Smallest value"),
    ("MAX", "Largest value"),
    ("COUNT", "Count non-empty cells"),
    ("POWER", "Raise to power"),
    ("CEILING", "Round up to multiple"),
    ("FLOOR", "Round down to multiple"),
    ("CONCAT", "Join text strings"),
    ("TRIM", "Remove extra spaces"),
    ("UPPER", "Convert to uppercase"),
    ("LOWER", "Convert to lowercase"),
    ("PROPER", "Capitalize words"),
    ("LEFT", "Extract left chars"),
    ("RIGHT", "Extract right chars"),
    ("MID", "Extract middle chars"),
    ("SUBSTITUTE", "Replace text"),
    ("REPLACE", "Replace by position"),
    ("NOW", "Current date and time"),
    ("TODAY", "Current date"),
    ("DATEDIF", "Date difference"),
    ("VLOOKUP", "Vertical lookup"),
    ("HLOOKUP", "Horizontal lookup"),
    ("IF", "Conditional value"),
];

/// Build the list of formula completion items.
fn formula_items() -> Vec<CompletionItem> {
    FORMULA_FUNCTIONS
        .iter()
        .map(|(name, _)| CompletionItem {
            text: name.to_string(),
            kind: CompletionKind::Function,
        })
        .collect()
}

/// Open the formula completion popup with an optional pre-typed prefix.
pub fn open_formula_completion(app: &mut App, prefix: &str) {
    let items = formula_items();
    app.formula_completion = Some(SqlCompletion::new(items, prefix));
}

/// Close the formula completion popup.
pub fn close_formula_completion(app: &mut App) {
    app.formula_completion = None;
}

/// Accept the currently selected completion item.
/// Replaces the typed filter text in the edit buffer with the selected function name + "(".
pub fn accept_completion(app: &mut App) {
    let (text, chars_to_delete) = {
        let comp = match &app.formula_completion {
            Some(c) => c,
            None => return,
        };
        let item = match comp.selected_item() {
            Some(i) => i,
            None => {
                app.formula_completion = None;
                return;
            }
        };
        // Delete the entire filter (all chars typed since popup opened)
        (item.text.clone(), comp.filter.len())
    };
    app.formula_completion = None;

    if let Some(ref mut buffer) = app.edit_buffer {
        // Delete the characters that were typed as filter
        for _ in 0..chars_to_delete {
            if buffer.cursor > 0 {
                buffer.cursor -= 1;
                let byte_pos = buffer
                    .content
                    .char_indices()
                    .nth(buffer.cursor)
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                buffer.content.remove(byte_pos);
            }
        }

        // Insert the function name + opening paren
        let insert_text = format!("{}(", text);
        let byte_pos = buffer
            .content
            .char_indices()
            .nth(buffer.cursor)
            .map(|(i, _)| i)
            .unwrap_or(buffer.content.len());
        buffer.content.insert_str(byte_pos, &insert_text);
        buffer.cursor += insert_text.chars().count();
    }
}
