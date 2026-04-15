//! Color and conditional formatting commands (:bgcolor, :fgcolor, :clearview).

use crate::app::App;
use crate::config::parse_color;
use crate::config::views;
use crate::input::actions::InputResult;
use crate::input::StatusMessage;
use crate::ui::conditional::{ColumnCondition, ConditionType, RowCondition};
use crate::ui::utils::excel_letter_to_column;
use anyhow::Result;

pub(super) fn execute_column_color(app: &mut App, arg: &str, is_bg: bool) -> Result<InputResult> {
    use crate::ui::conditional::ColorRule;

    let kind = if is_bg { "bgcolor" } else { "fgcolor" };

    let parts: Vec<&str> = arg.splitn(2, ' ').collect();
    if parts.len() < 2 {
        app.status_message = Some(StatusMessage::from(format!(
            "Usage: :{0} <col|row> <color> [condition] (e.g., :{0} C red, :{0} 1 red, :{0} C red > 100)",
            kind
        )));
        return Ok(InputResult::Continue);
    }

    let target_spec = parts[0].trim();
    let rest = parts[1].trim();

    // # = row conditional mode
    if target_spec == "#" {
        return execute_row_conditional_color(app, rest, is_bg);
    }

    // Detect if target is a row number or column spec
    let is_row = target_spec.parse::<usize>().is_ok();

    if is_row {
        return execute_row_color(app, target_spec, rest, is_bg);
    }

    // Column mode
    let col_index = match resolve_column_spec(app, target_spec) {
        Ok(idx) => idx,
        Err(msg) => {
            app.status_message = Some(StatusMessage::from(msg));
            return Ok(InputResult::Continue);
        }
    };

    // Handle "clear" / "none"
    if rest.eq_ignore_ascii_case("clear") || rest.eq_ignore_ascii_case("none") {
        if is_bg {
            app.view_state.column_bg_colors.remove(&col_index);
        } else {
            app.view_state.column_fg_colors.remove(&col_index);
        }
        views::save_current_views(app);
        let letter = column_index_to_letter(col_index);
        app.status_message = Some(StatusMessage::from(format!(
            "Cleared {} for column {}",
            kind, letter
        )));
        return Ok(InputResult::Continue);
    }

    // Handle "list" — show all rules for this column
    if rest.eq_ignore_ascii_case("list") {
        let map = if is_bg {
            &app.view_state.column_bg_colors
        } else {
            &app.view_state.column_fg_colors
        };
        let letter = column_index_to_letter(col_index);
        if let Some(rules) = map.get(&col_index) {
            let lines: Vec<String> = rules
                .iter()
                .enumerate()
                .map(|(i, rule)| {
                    let color_str = crate::config::views::color_to_string(rule.color);
                    let cond_str = crate::ui::conditional::format_condition(&rule.condition);
                    format!("  {}: {} {}", i + 1, color_str, cond_str)
                })
                .collect();
            app.status_message = Some(StatusMessage::from(format!(
                "{} rules for column {}:{}",
                kind,
                letter,
                lines.join(",")
            )));
        } else {
            app.status_message = Some(StatusMessage::from(format!(
                "No {} rules for column {}",
                kind, letter
            )));
        }
        return Ok(InputResult::Continue);
    }

    // Handle "remove N" — remove a specific rule by 1-based index
    if let Some(num_str) = rest
        .strip_prefix("remove ")
        .or_else(|| rest.strip_prefix("rm "))
    {
        let num_str = num_str.trim();
        if let Ok(idx) = num_str.parse::<usize>() {
            let map = if is_bg {
                &mut app.view_state.column_bg_colors
            } else {
                &mut app.view_state.column_fg_colors
            };
            let letter = column_index_to_letter(col_index);
            if let Some(rules) = map.get_mut(&col_index) {
                if idx >= 1 && idx <= rules.len() {
                    rules.remove(idx - 1);
                    if rules.is_empty() {
                        map.remove(&col_index);
                    }
                    views::save_current_views(app);
                    app.status_message = Some(StatusMessage::from(format!(
                        "Removed {} rule {} for column {}",
                        kind, idx, letter
                    )));
                } else {
                    app.status_message = Some(StatusMessage::from(format!(
                        "Rule {} out of range (1-{})",
                        idx,
                        rules.len()
                    )));
                }
            } else {
                app.status_message = Some(StatusMessage::from(format!(
                    "No {} rules for column {}",
                    kind, letter
                )));
            }
            return Ok(InputResult::Continue);
        }
    }

    // Parse: <color> [operator value]
    // Split rest into color and optional condition
    let (color_str, condition_str) = parse_color_and_condition(rest);

    let color = match parse_color(color_str) {
        Some(c) => c,
        None => {
            app.status_message = Some(StatusMessage::from(format!(
                "Unknown color: {:?}. Use a named color (red, blue, etc.) or hex (#ff0000)",
                color_str
            )));
            return Ok(InputResult::Continue);
        }
    };

    // Parse condition if present
    let condition = if let Some(cond) = condition_str {
        match parse_condition(cond) {
            Ok(c) => c,
            Err(msg) => {
                app.status_message = Some(StatusMessage::from(msg));
                return Ok(InputResult::Continue);
            }
        }
    } else {
        ConditionType::Always
    };

    let rule = ColorRule { condition, color };
    let map = if is_bg {
        &mut app.view_state.column_bg_colors
    } else {
        &mut app.view_state.column_fg_colors
    };

    // For unconditional rules, replace all existing rules
    // For conditional rules, append to the list
    if matches!(rule.condition, ConditionType::Always) {
        map.insert(col_index, vec![rule]);
    } else {
        map.entry(col_index).or_default().push(rule);
    }

    views::save_current_views(app);
    let letter = column_index_to_letter(col_index);
    let desc = if condition_str.is_some() {
        format!(
            "Set conditional {} for column {} to {}",
            kind, letter, color_str
        )
    } else {
        format!("Set {} for column {} to {}", kind, letter, color_str)
    };
    app.status_message = Some(StatusMessage::from(desc));

    Ok(InputResult::Continue)
}

/// Execute :bgcolor/:fgcolor for a row target
fn execute_row_color(
    app: &mut App,
    row_spec: &str,
    rest: &str,
    is_bg: bool,
) -> Result<InputResult> {
    use crate::ui::conditional::ColorRule;

    let kind = if is_bg { "bgcolor" } else { "fgcolor" };

    let row_num = row_spec.parse::<usize>().unwrap();
    if row_num == 0 || row_num > app.document.row_count() {
        app.status_message = Some(StatusMessage::from(format!(
            "Row {} out of range (1-{})",
            row_num,
            app.document.row_count()
        )));
        return Ok(InputResult::Continue);
    }
    let row_index = row_num - 1; // Convert to 0-based

    // Handle "clear" / "none"
    if rest.eq_ignore_ascii_case("clear") || rest.eq_ignore_ascii_case("none") {
        if is_bg {
            app.view_state.row_bg_colors.remove(&row_index);
        } else {
            app.view_state.row_fg_colors.remove(&row_index);
        }
        views::save_current_views(app);
        app.status_message = Some(StatusMessage::from(format!(
            "Cleared {} for row {}",
            kind, row_num
        )));
        return Ok(InputResult::Continue);
    }

    // Handle "list"
    if rest.eq_ignore_ascii_case("list") {
        let map = if is_bg {
            &app.view_state.row_bg_colors
        } else {
            &app.view_state.row_fg_colors
        };
        if let Some(rules) = map.get(&row_index) {
            let lines: Vec<String> = rules
                .iter()
                .enumerate()
                .map(|(i, rule)| {
                    let color_str = crate::config::views::color_to_string(rule.color);
                    let cond_str = crate::ui::conditional::format_condition(&rule.condition);
                    format!("  {}: {} {}", i + 1, color_str, cond_str)
                })
                .collect();
            app.status_message = Some(StatusMessage::from(format!(
                "{} rules for row {}:{}",
                kind,
                row_num,
                lines.join(",")
            )));
        } else {
            app.status_message = Some(StatusMessage::from(format!(
                "No {} rules for row {}",
                kind, row_num
            )));
        }
        return Ok(InputResult::Continue);
    }

    // Handle "remove N"
    if let Some(num_str) = rest
        .strip_prefix("remove ")
        .or_else(|| rest.strip_prefix("rm "))
    {
        let num_str = num_str.trim();
        if let Ok(idx) = num_str.parse::<usize>() {
            let map = if is_bg {
                &mut app.view_state.row_bg_colors
            } else {
                &mut app.view_state.row_fg_colors
            };
            if let Some(rules) = map.get_mut(&row_index) {
                if idx >= 1 && idx <= rules.len() {
                    rules.remove(idx - 1);
                    if rules.is_empty() {
                        map.remove(&row_index);
                    }
                    views::save_current_views(app);
                    app.status_message = Some(StatusMessage::from(format!(
                        "Removed {} rule {} for row {}",
                        kind, idx, row_num
                    )));
                } else {
                    app.status_message = Some(StatusMessage::from(format!(
                        "Rule {} out of range (1-{})",
                        idx,
                        rules.len()
                    )));
                }
            } else {
                app.status_message = Some(StatusMessage::from(format!(
                    "No {} rules for row {}",
                    kind, row_num
                )));
            }
            return Ok(InputResult::Continue);
        }
    }

    // Parse color (row colors are always unconditional — "Always")
    let color = match parse_color(rest) {
        Some(c) => c,
        None => {
            app.status_message = Some(StatusMessage::from(format!(
                "Unknown color: {:?}. Use a named color (red, blue, etc.) or hex (#ff0000)",
                rest
            )));
            return Ok(InputResult::Continue);
        }
    };

    let rule = ColorRule {
        condition: ConditionType::Always,
        color,
    };
    let map = if is_bg {
        &mut app.view_state.row_bg_colors
    } else {
        &mut app.view_state.row_fg_colors
    };
    map.insert(row_index, vec![rule]);

    views::save_current_views(app);
    app.status_message = Some(StatusMessage::from(format!(
        "Set {} for row {} to {}",
        kind, row_num, rest
    )));

    Ok(InputResult::Continue)
}

/// Execute :bgcolor # / :fgcolor # — row conditional coloring.
/// Syntax: :bgcolor # red A > 100 && B = "fish"
fn execute_row_conditional_color(app: &mut App, rest: &str, is_bg: bool) -> Result<InputResult> {
    use crate::ui::conditional::RowConditionalRule;

    let kind = if is_bg { "bgcolor" } else { "fgcolor" };

    // Handle "clear"
    if rest.eq_ignore_ascii_case("clear") || rest.eq_ignore_ascii_case("none") {
        if is_bg {
            app.view_state.row_cond_bg.clear();
        } else {
            app.view_state.row_cond_fg.clear();
        }
        views::save_current_views(app);
        app.status_message = Some(StatusMessage::from(format!(
            "Cleared all row conditional {}",
            kind
        )));
        return Ok(InputResult::Continue);
    }

    // Handle "list"
    if rest.eq_ignore_ascii_case("list") {
        let rules = if is_bg {
            &app.view_state.row_cond_bg
        } else {
            &app.view_state.row_cond_fg
        };
        if rules.is_empty() {
            app.status_message = Some(StatusMessage::from(format!(
                "No row conditional {} rules",
                kind
            )));
        } else {
            let headers = app.document.storage.header_row();
            let lines: Vec<String> = rules
                .iter()
                .enumerate()
                .map(|(i, rule)| {
                    let color_str = crate::config::views::color_to_string(rule.color);
                    let cond_str =
                        crate::ui::conditional::format_row_condition(&rule.condition, headers);
                    format!("  {}: {} where {}", i + 1, color_str, cond_str)
                })
                .collect();
            app.status_message = Some(StatusMessage::from(format!(
                "Row {} rules:{}",
                kind,
                lines.join(",")
            )));
        }
        return Ok(InputResult::Continue);
    }

    // Handle "remove N"
    if let Some(num_str) = rest
        .strip_prefix("remove ")
        .or_else(|| rest.strip_prefix("rm "))
    {
        if let Ok(idx) = num_str.trim().parse::<usize>() {
            let rules = if is_bg {
                &mut app.view_state.row_cond_bg
            } else {
                &mut app.view_state.row_cond_fg
            };
            if idx >= 1 && idx <= rules.len() {
                rules.remove(idx - 1);
                views::save_current_views(app);
                app.status_message = Some(StatusMessage::from(format!(
                    "Removed row conditional {} rule {}",
                    kind, idx
                )));
            } else {
                app.status_message = Some(StatusMessage::from(format!(
                    "Rule {} out of range (1-{})",
                    idx,
                    rules.len()
                )));
            }
            return Ok(InputResult::Continue);
        }
    }

    // Parse: <color> <column_condition>
    // Find where the color ends and the column-qualified condition begins
    // The first token that resolves as a column spec marks the start of conditions
    let tokens: Vec<&str> = rest.splitn(2, ' ').collect();
    if tokens.len() < 2 {
        app.status_message = Some(StatusMessage::from(format!(
            "Usage: :{} # <color> <col> <op> <value> [&& <col> <op> <value>]",
            kind
        )));
        return Ok(InputResult::Continue);
    }

    let color_str = tokens[0].trim();
    let cond_str = tokens[1].trim();

    let color = match parse_color(color_str) {
        Some(c) => c,
        None => {
            app.status_message = Some(StatusMessage::from(format!(
                "Unknown color: {:?}",
                color_str
            )));
            return Ok(InputResult::Continue);
        }
    };

    // Parse column-qualified conditions
    match parse_row_condition(app, cond_str) {
        Ok(condition) => {
            let rule = RowConditionalRule { color, condition };
            if is_bg {
                app.view_state.row_cond_bg.push(rule);
            } else {
                app.view_state.row_cond_fg.push(rule);
            }
            views::save_current_views(app);
            app.status_message = Some(StatusMessage::from(format!(
                "Added row conditional {} rule",
                kind
            )));
        }
        Err(msg) => {
            app.status_message = Some(StatusMessage::from(msg));
        }
    }

    Ok(InputResult::Continue)
}

/// Parse a column-qualified row condition like `A > 100 && B = "fish"`.
/// Each sub-condition starts with a column spec followed by an operator and value.
fn parse_row_condition(app: &App, s: &str) -> std::result::Result<RowCondition, String> {
    let s = s.trim();

    // Split on top-level ||
    let or_parts = split_top_level(s, "||");
    if or_parts.len() > 1 {
        let subs: std::result::Result<Vec<RowCondition>, String> = or_parts
            .iter()
            .map(|p| parse_row_condition(app, p.trim()))
            .collect();
        return Ok(RowCondition::Or(subs?));
    }

    // Split on top-level &&
    let and_parts = split_top_level(s, "&&");
    if and_parts.len() > 1 {
        let subs: std::result::Result<Vec<RowCondition>, String> = and_parts
            .iter()
            .map(|p| parse_row_condition(app, p.trim()))
            .collect();
        return Ok(RowCondition::And(subs?));
    }

    // Strip outer parens
    let s = s.trim();
    if s.starts_with('(') && s.ends_with(')') && paren_depth_valid(&s[1..s.len() - 1]) {
        return parse_row_condition(app, &s[1..s.len() - 1]);
    }

    // Single condition: <column> <op> <value>
    // Find where the column spec ends and the operator begins
    parse_single_row_condition(app, s)
}

/// Parse a single column-qualified condition like `A > 100` or `"Cost Margin" = "foo"`.
fn parse_single_row_condition(app: &App, s: &str) -> std::result::Result<RowCondition, String> {
    let s = s.trim();

    // Handle quoted column names: "Col Name" > 100
    let (col_spec, rest) = if let Some(after_quote) = s.strip_prefix('"') {
        // Find closing quote
        if let Some(end) = after_quote.find('"') {
            let col = &s[..end + 2]; // include quotes
            let rest = s[end + 2..].trim();
            (col, rest)
        } else {
            return Err(format!("Unterminated quote in: {:?}", s));
        }
    } else {
        // Column spec is the first token
        let mut parts = s.splitn(2, |c: char| {
            c == ' ' || c == '>' || c == '<' || c == '=' || c == '!' || c == '~'
        });
        let col = parts.next().unwrap_or("").trim();
        // Find where the operator starts in the original string
        let rest = &s[col.len()..].trim();
        (col, *rest)
    };

    if col_spec.is_empty() || rest.is_empty() {
        return Err(format!(
            "Expected <column> <operator> <value>, got: {:?}",
            s
        ));
    }

    let col_index =
        resolve_column_spec(app, col_spec).map_err(|e| format!("In row condition: {}", e))?;

    let condition = parse_single_condition(rest)?;

    Ok(RowCondition::Single(ColumnCondition {
        col_index,
        condition,
    }))
}

/// Split a string like "red > 100" or "red (> 100 && < 200)" into (color, condition).
fn parse_color_and_condition(s: &str) -> (&str, Option<&str>) {
    // Operators/chars that signal start of condition
    let operators = ["!=", ">=", "<=", ">", "<", "==", "=", "~", "("];
    for op in &operators {
        if let Some(pos) = s.find(op) {
            let color_part = s[..pos].trim();
            let cond_part = s[pos..].trim();
            if !color_part.is_empty() && !cond_part.is_empty() {
                return (color_part, Some(cond_part));
            }
        }
    }
    (s, None)
}

/// Parse a single condition (no && or ||).
fn parse_single_condition(s: &str) -> std::result::Result<ConditionType, String> {
    let s = s.trim();

    // Try two-char operators first
    for (prefix, make) in [
        (
            "!=",
            make_not_equals as fn(&str) -> std::result::Result<ConditionType, String>,
        ),
        (
            ">=",
            make_gte as fn(&str) -> std::result::Result<ConditionType, String>,
        ),
        (
            "<=",
            make_lte as fn(&str) -> std::result::Result<ConditionType, String>,
        ),
    ] {
        if let Some(rest) = s.strip_prefix(prefix) {
            return make(rest.trim());
        }
    }

    // Single/double-char operators
    if let Some(rest) = s.strip_prefix("==") {
        let val = unquote(rest.trim());
        return Ok(ConditionType::Equals(val));
    }
    if let Some(rest) = s.strip_prefix('=') {
        let val = unquote(rest.trim());
        return Ok(ConditionType::Equals(val));
    }
    if let Some(rest) = s.strip_prefix('>') {
        let val = rest
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("Expected number after >, got {:?}", rest.trim()))?;
        return Ok(ConditionType::GreaterThan(val));
    }
    if let Some(rest) = s.strip_prefix('<') {
        let val = rest
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("Expected number after <, got {:?}", rest.trim()))?;
        return Ok(ConditionType::LessThan(val));
    }
    if let Some(rest) = s.strip_prefix('~') {
        let pattern = unquote(rest.trim());
        let re = regex::Regex::new(&pattern)
            .map_err(|e| format!("Invalid regex {:?}: {}", pattern, e))?;
        return Ok(ConditionType::RegexMatch(re));
    }

    Err(format!(
        "Unknown condition: {:?}. Use =, !=, >, <, >=, <=, ~, && or ||",
        s
    ))
}

/// Parse a condition string, supporting compound conditions with && and ||,
/// and parenthesized groups like `(> 100 && < 200) || = 0`.
fn parse_condition(s: &str) -> std::result::Result<ConditionType, String> {
    let s = s.trim();

    // Split on top-level || first (lower precedence)
    let or_parts = split_top_level(s, "||");
    if or_parts.len() > 1 {
        let subs: std::result::Result<Vec<ConditionType>, String> =
            or_parts.iter().map(|p| parse_condition(p.trim())).collect();
        return Ok(ConditionType::Or(subs?));
    }

    // Split on top-level &&
    let and_parts = split_top_level(s, "&&");
    if and_parts.len() > 1 {
        let subs: std::result::Result<Vec<ConditionType>, String> = and_parts
            .iter()
            .map(|p| parse_condition(p.trim()))
            .collect();
        return Ok(ConditionType::And(subs?));
    }

    // Strip outer parentheses if present
    let s = s.trim();
    if s.starts_with('(') && s.ends_with(')') {
        let inner = &s[1..s.len() - 1];
        // Verify these are matching parens (not nested)
        if paren_depth_valid(inner) {
            return parse_condition(inner);
        }
    }

    parse_single_condition(s)
}

/// Split a string on a delimiter, but only at the top level (not inside parentheses).
fn split_top_level<'a>(s: &'a str, delim: &str) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut last = 0;
    let bytes = s.as_bytes();
    let delim_bytes = delim.as_bytes();

    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'(' {
            depth += 1;
        } else if bytes[i] == b')' {
            depth = depth.saturating_sub(1);
        } else if depth == 0
            && i + delim_bytes.len() <= bytes.len()
            && &bytes[i..i + delim_bytes.len()] == delim_bytes
        {
            parts.push(&s[last..i]);
            last = i + delim_bytes.len();
            i = last;
            continue;
        }
        i += 1;
    }
    parts.push(&s[last..]);
    parts
}

/// Check that parentheses are balanced inside a string (for stripping outer parens).
fn paren_depth_valid(s: &str) -> bool {
    let mut depth = 0i32;
    for c in s.chars() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

fn make_not_equals(s: &str) -> std::result::Result<ConditionType, String> {
    Ok(ConditionType::NotEquals(unquote(s)))
}
fn make_gte(s: &str) -> std::result::Result<ConditionType, String> {
    let val = s
        .parse::<f64>()
        .map_err(|_| format!("Expected number after >=, got {:?}", s))?;
    Ok(ConditionType::GreaterThanOrEqual(val))
}
fn make_lte(s: &str) -> std::result::Result<ConditionType, String> {
    let val = s
        .parse::<f64>()
        .map_err(|_| format!("Expected number after <=, got {:?}", s))?;
    Ok(ConditionType::LessThanOrEqual(val))
}

/// Strip surrounding quotes from a string if present.
fn unquote(s: &str) -> String {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Resolve a column specifier (letter, number, or header name) to a 0-based index
fn resolve_column_spec(app: &App, spec: &str) -> std::result::Result<usize, String> {
    // Strip surrounding quotes for header names like "Cost Margin"
    let spec = if (spec.starts_with('"') && spec.ends_with('"'))
        || (spec.starts_with('\'') && spec.ends_with('\''))
    {
        &spec[1..spec.len() - 1]
    } else {
        spec
    };

    // Try header name (case-insensitive)
    let header_row = app.document.storage.header_row();
    if let Some(idx) = header_row
        .iter()
        .position(|name| name.eq_ignore_ascii_case(spec))
    {
        return Ok(idx);
    }

    // Try Excel-style column letter
    if spec.chars().all(|c| c.is_ascii_alphabetic()) {
        match excel_letter_to_column(spec) {
            Ok(idx) if idx < app.document.column_count() => return Ok(idx),
            _ => {}
        }
    }

    Err(format!(
        "Unknown column: {:?} (use letter like A or header name)",
        spec
    ))
}

/// Execute :clearview — remove all saved view settings for the current file
pub(super) fn execute_clear_view(app: &mut App) -> Result<InputResult> {
    // Clear in-memory state
    app.session.clear_all_column_widths();
    app.session.unfreeze_all();
    app.view_state.column_bg_colors.clear();
    app.view_state.column_fg_colors.clear();
    app.view_state.row_bg_colors.clear();
    app.view_state.row_fg_colors.clear();
    app.view_state.row_cond_bg.clear();
    app.view_state.row_cond_fg.clear();

    // Clear column types for current file
    let file = app
        .session
        .files()
        .get(app.session.active_file_index())
        .cloned();
    if let Some(ref path) = file {
        // Remove from persisted views
        let mut store = views::load_views();
        let key = views::canonical_key(path);
        store.files.remove(&key);
        views::save_views(&store);
    }

    app.status_message = Some(StatusMessage::from("View settings cleared"));
    Ok(InputResult::Continue)
}

/// Convert a 0-based column index to an Excel-style letter (0=A, 1=B, 25=Z, 26=AA)
fn column_index_to_letter(mut idx: usize) -> String {
    let mut result = String::new();
    loop {
        result.insert(0, (b'A' + (idx % 26) as u8) as char);
        if idx < 26 {
            break;
        }
        idx = idx / 26 - 1;
    }
    result
}
