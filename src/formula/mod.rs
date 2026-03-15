//! Cell formula engine for Excel-like functions.
//!
//! Formulas are stored separately from the document — the document always holds
//! the computed value. This means CSV saves write the result, and formulas are
//! a TUI-only feature that is lost on file re-open.
//!
//! Supported functions:
//! - Aggregate: SUM, AVERAGE/AVG, MIN, MAX, COUNT
//! - Math: POWER, CEILING, FLOOR
//! - Text: CONCAT, TRIM, UPPER, LOWER, PROPER, LEFT, RIGHT, MID, SUBSTITUTE, REPLACE
//! - Date: NOW, TODAY, DATEDIF
//! - Lookup: VLOOKUP, HLOOKUP
//! - Logic: IF

use crate::input::command_mode::stats::{format_number, parse_numeric};
use crate::ui::utils::excel_letter_to_column;
use chrono::{Datelike, Local};
use std::collections::HashMap;

/// A reference to a single cell (0-based row and column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellRef {
    pub row: usize,
    pub col: usize,
}

/// A formula argument — either a cell reference, a range expanded to refs, or a literal value.
#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    /// A single cell reference
    Cell(CellRef),
    /// A range of cell references (e.g., A1:A5)
    Range(Vec<CellRef>),
    /// A literal number
    Number(f64),
    /// A literal string (quoted in the formula)
    Text(String),
    /// A boolean literal
    Bool(bool),
}

impl Arg {
    /// Collect all cell references from this argument.
    fn cell_refs(&self) -> Vec<CellRef> {
        match self {
            Arg::Cell(r) => vec![*r],
            Arg::Range(refs) => refs.clone(),
            _ => vec![],
        }
    }

    /// Resolve to a list of string values using the cell getter.
    fn resolve_values(&self, get_cell: &dyn Fn(usize, usize) -> String) -> Vec<String> {
        match self {
            Arg::Cell(r) => vec![get_cell(r.row, r.col)],
            Arg::Range(refs) => refs.iter().map(|r| get_cell(r.row, r.col)).collect(),
            Arg::Number(n) => vec![format_number(*n)],
            Arg::Text(s) => vec![s.clone()],
            Arg::Bool(b) => vec![if *b {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }],
        }
    }

    /// Resolve to a single string value.
    fn resolve_single(&self, get_cell: &dyn Fn(usize, usize) -> String) -> String {
        match self {
            Arg::Cell(r) => get_cell(r.row, r.col),
            Arg::Number(n) => format_number(*n),
            Arg::Text(s) => s.clone(),
            Arg::Bool(b) => {
                if *b {
                    "TRUE".to_string()
                } else {
                    "FALSE".to_string()
                }
            }
            Arg::Range(refs) => {
                if let Some(first) = refs.first() {
                    get_cell(first.row, first.col)
                } else {
                    String::new()
                }
            }
        }
    }

    /// Resolve to a single numeric value.
    fn resolve_number(&self, get_cell: &dyn Fn(usize, usize) -> String) -> Option<f64> {
        match self {
            Arg::Number(n) => Some(*n),
            _ => parse_numeric(&self.resolve_single(get_cell)),
        }
    }
}

/// A parsed formula with function name and arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct Formula {
    pub func: FormulaFunc,
    pub args: Vec<Arg>,
}

/// Supported formula functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormulaFunc {
    // Aggregate
    Sum,
    Average,
    Min,
    Max,
    Count,
    // Math
    Power,
    Ceiling,
    Floor,
    // Text
    Concat,
    Trim,
    Upper,
    Lower,
    Proper,
    Left,
    Right,
    Mid,
    Substitute,
    Replace,
    // Date
    Now,
    Today,
    DateDif,
    // Lookup
    VLookup,
    HLookup,
    // Logic
    If,
}

impl Formula {
    /// Get all cell references this formula depends on.
    pub fn references(&self) -> Vec<CellRef> {
        self.args.iter().flat_map(|a| a.cell_refs()).collect()
    }

    /// Evaluate the formula using a closure to resolve cell values.
    pub fn evaluate(&self, get_cell: &dyn Fn(usize, usize) -> String) -> String {
        match self.func {
            FormulaFunc::Sum => self.eval_aggregate(get_cell, |nums| nums.iter().sum()),
            FormulaFunc::Average => {
                let nums = self.collect_numbers(get_cell);
                if nums.is_empty() {
                    "#DIV/0!".to_string()
                } else {
                    format_number(nums.iter().sum::<f64>() / nums.len() as f64)
                }
            }
            FormulaFunc::Min => self.eval_aggregate(get_cell, |nums| {
                nums.iter().cloned().fold(f64::INFINITY, f64::min)
            }),
            FormulaFunc::Max => self.eval_aggregate(get_cell, |nums| {
                nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
            }),
            FormulaFunc::Count => {
                let values = self.collect_all_values(get_cell);
                let count = values.iter().filter(|v| !v.trim().is_empty()).count();
                format_number(count as f64)
            }
            FormulaFunc::Power => self.eval_power(get_cell),
            FormulaFunc::Ceiling => self.eval_ceiling_floor(get_cell, true),
            FormulaFunc::Floor => self.eval_ceiling_floor(get_cell, false),
            FormulaFunc::Concat => self.eval_concat(get_cell),
            FormulaFunc::Trim => self.eval_trim(get_cell),
            FormulaFunc::Upper => self.eval_case(get_cell, CaseOp::Upper),
            FormulaFunc::Lower => self.eval_case(get_cell, CaseOp::Lower),
            FormulaFunc::Proper => self.eval_case(get_cell, CaseOp::Proper),
            FormulaFunc::Left => self.eval_left_right_mid(get_cell, SubstrOp::Left),
            FormulaFunc::Right => self.eval_left_right_mid(get_cell, SubstrOp::Right),
            FormulaFunc::Mid => self.eval_left_right_mid(get_cell, SubstrOp::Mid),
            FormulaFunc::Substitute => self.eval_substitute(get_cell),
            FormulaFunc::Replace => self.eval_replace(get_cell),
            FormulaFunc::Now => Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            FormulaFunc::Today => Local::now().format("%Y-%m-%d").to_string(),
            FormulaFunc::DateDif => self.eval_datedif(get_cell),
            FormulaFunc::VLookup => self.eval_vlookup(get_cell),
            FormulaFunc::HLookup => self.eval_hlookup(get_cell),
            FormulaFunc::If => self.eval_if(get_cell),
        }
    }

    // ---- Aggregate helpers ----

    fn collect_numbers(&self, get_cell: &dyn Fn(usize, usize) -> String) -> Vec<f64> {
        self.collect_all_values(get_cell)
            .iter()
            .filter_map(|v| parse_numeric(v))
            .collect()
    }

    fn collect_all_values(&self, get_cell: &dyn Fn(usize, usize) -> String) -> Vec<String> {
        self.args
            .iter()
            .flat_map(|a| a.resolve_values(get_cell))
            .collect()
    }

    fn eval_aggregate(
        &self,
        get_cell: &dyn Fn(usize, usize) -> String,
        op: impl Fn(&[f64]) -> f64,
    ) -> String {
        let nums = self.collect_numbers(get_cell);
        if nums.is_empty() {
            "0".to_string()
        } else {
            format_number(op(&nums))
        }
    }

    // ---- Math ----

    fn eval_power(&self, get_cell: &dyn Fn(usize, usize) -> String) -> String {
        if self.args.len() < 2 {
            return "#VALUE!".to_string();
        }
        let base = match self.args[0].resolve_number(get_cell) {
            Some(n) => n,
            None => return "#VALUE!".to_string(),
        };
        let exp = match self.args[1].resolve_number(get_cell) {
            Some(n) => n,
            None => return "#VALUE!".to_string(),
        };
        format_number(base.powf(exp))
    }

    fn eval_ceiling_floor(
        &self,
        get_cell: &dyn Fn(usize, usize) -> String,
        is_ceiling: bool,
    ) -> String {
        if self.args.is_empty() {
            return "#VALUE!".to_string();
        }
        let number = match self.args[0].resolve_number(get_cell) {
            Some(n) => n,
            None => return "#VALUE!".to_string(),
        };
        let significance = if self.args.len() >= 2 {
            match self.args[1].resolve_number(get_cell) {
                Some(n) if n != 0.0 => n,
                _ => return "#VALUE!".to_string(),
            }
        } else {
            1.0
        };
        let result = if is_ceiling {
            (number / significance).ceil() * significance
        } else {
            (number / significance).floor() * significance
        };
        format_number(result)
    }

    // ---- Text ----

    fn eval_concat(&self, get_cell: &dyn Fn(usize, usize) -> String) -> String {
        self.args
            .iter()
            .flat_map(|a| a.resolve_values(get_cell))
            .collect::<Vec<_>>()
            .join("")
    }

    fn eval_trim(&self, get_cell: &dyn Fn(usize, usize) -> String) -> String {
        if self.args.is_empty() {
            return String::new();
        }
        let s = self.args[0].resolve_single(get_cell);
        // Excel TRIM: collapse multiple spaces to single, trim leading/trailing
        let mut result = String::new();
        let mut prev_space = true; // treat start as after a space to trim leading
        for c in s.chars() {
            if c == ' ' {
                if !prev_space {
                    result.push(' ');
                }
                prev_space = true;
            } else {
                result.push(c);
                prev_space = false;
            }
        }
        // Trim trailing space
        if result.ends_with(' ') {
            result.pop();
        }
        result
    }

    fn eval_case(&self, get_cell: &dyn Fn(usize, usize) -> String, op: CaseOp) -> String {
        if self.args.is_empty() {
            return String::new();
        }
        let s = self.args[0].resolve_single(get_cell);
        match op {
            CaseOp::Upper => s.to_uppercase(),
            CaseOp::Lower => s.to_lowercase(),
            CaseOp::Proper => {
                let mut result = String::new();
                let mut capitalize_next = true;
                for c in s.chars() {
                    if c.is_whitespace() || c == '-' || c == '\'' {
                        result.push(c);
                        capitalize_next = true;
                    } else if capitalize_next {
                        result.extend(c.to_uppercase());
                        capitalize_next = false;
                    } else {
                        result.extend(c.to_lowercase());
                    }
                }
                result
            }
        }
    }

    fn eval_left_right_mid(
        &self,
        get_cell: &dyn Fn(usize, usize) -> String,
        op: SubstrOp,
    ) -> String {
        if self.args.is_empty() {
            return String::new();
        }
        let s = self.args[0].resolve_single(get_cell);
        let chars: Vec<char> = s.chars().collect();

        match op {
            SubstrOp::Left => {
                let n = self
                    .args
                    .get(1)
                    .and_then(|a| a.resolve_number(get_cell))
                    .unwrap_or(1.0) as usize;
                chars.iter().take(n).collect()
            }
            SubstrOp::Right => {
                let n = self
                    .args
                    .get(1)
                    .and_then(|a| a.resolve_number(get_cell))
                    .unwrap_or(1.0) as usize;
                let start = chars.len().saturating_sub(n);
                chars[start..].iter().collect()
            }
            SubstrOp::Mid => {
                // MID(text, start_num, num_chars) — start_num is 1-based
                let start = self
                    .args
                    .get(1)
                    .and_then(|a| a.resolve_number(get_cell))
                    .unwrap_or(1.0) as usize;
                let n = self
                    .args
                    .get(2)
                    .and_then(|a| a.resolve_number(get_cell))
                    .unwrap_or(1.0) as usize;
                let start_idx = start.saturating_sub(1); // convert to 0-based
                chars.iter().skip(start_idx).take(n).collect()
            }
        }
    }

    fn eval_substitute(&self, get_cell: &dyn Fn(usize, usize) -> String) -> String {
        // SUBSTITUTE(text, old_text, new_text)
        if self.args.len() < 3 {
            return "#VALUE!".to_string();
        }
        let text = self.args[0].resolve_single(get_cell);
        let old_text = self.args[1].resolve_single(get_cell);
        let new_text = self.args[2].resolve_single(get_cell);
        text.replace(&old_text, &new_text)
    }

    fn eval_replace(&self, get_cell: &dyn Fn(usize, usize) -> String) -> String {
        // REPLACE(old_text, start_num, num_chars, new_text) — start_num is 1-based
        if self.args.len() < 4 {
            return "#VALUE!".to_string();
        }
        let text = self.args[0].resolve_single(get_cell);
        let start = self.args[1].resolve_number(get_cell).unwrap_or(1.0) as usize;
        let num_chars = self.args[2].resolve_number(get_cell).unwrap_or(0.0) as usize;
        let new_text = self.args[3].resolve_single(get_cell);

        let chars: Vec<char> = text.chars().collect();
        let start_idx = start.saturating_sub(1);
        let end_idx = (start_idx + num_chars).min(chars.len());

        let mut result: String = chars[..start_idx].iter().collect();
        result.push_str(&new_text);
        result.extend(chars[end_idx..].iter());
        result
    }

    // ---- Date ----

    fn eval_datedif(&self, get_cell: &dyn Fn(usize, usize) -> String) -> String {
        // DATEDIF(start_date, end_date, unit)
        // unit: "d" (days), "m" (months), "y" (years)
        if self.args.len() < 3 {
            return "#VALUE!".to_string();
        }
        let start_str = self.args[0].resolve_single(get_cell);
        let end_str = self.args[1].resolve_single(get_cell);
        let unit = self.args[2].resolve_single(get_cell).to_lowercase();

        let start = match chrono::NaiveDate::parse_from_str(start_str.trim(), "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => return "#VALUE!".to_string(),
        };
        let end = match chrono::NaiveDate::parse_from_str(end_str.trim(), "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => return "#VALUE!".to_string(),
        };

        match unit.as_str() {
            "d" => format_number((end - start).num_days() as f64),
            "m" => {
                let months =
                    (end.year() - start.year()) * 12 + (end.month() as i32 - start.month() as i32);
                format_number(months as f64)
            }
            "y" => format_number((end.year() - start.year()) as f64),
            _ => "#VALUE!".to_string(),
        }
    }

    // ---- Lookup ----

    fn eval_vlookup(&self, get_cell: &dyn Fn(usize, usize) -> String) -> String {
        // VLOOKUP(lookup_value, table_range, col_index, [exact_match])
        if self.args.len() < 3 {
            return "#VALUE!".to_string();
        }
        let lookup_val = self.args[0].resolve_single(get_cell);
        let table_refs = match &self.args[1] {
            Arg::Range(refs) => refs,
            _ => return "#VALUE!".to_string(),
        };
        let col_index = match self.args[2].resolve_number(get_cell) {
            Some(n) => n as usize,
            None => return "#VALUE!".to_string(),
        };
        let exact = self
            .args
            .get(3)
            .map(|a| {
                let s = a.resolve_single(get_cell).to_uppercase();
                s == "FALSE" || s == "0"
            })
            .unwrap_or(true); // default to exact match

        // Determine table dimensions from range
        let (min_row, max_row, min_col, max_col) = range_bounds(table_refs);
        let table_cols = max_col - min_col + 1;

        if col_index == 0 || col_index > table_cols {
            return "#REF!".to_string();
        }

        // Search first column for lookup value
        for r in min_row..=max_row {
            let cell_val = get_cell(r, min_col);
            let matches = if exact {
                cell_val.trim() == lookup_val.trim()
            } else {
                // Approximate: find last value <= lookup_val (assumes sorted)
                cell_val.trim() == lookup_val.trim()
            };
            if matches {
                let result_col = min_col + col_index - 1;
                return get_cell(r, result_col);
            }
        }
        "#N/A".to_string()
    }

    fn eval_hlookup(&self, get_cell: &dyn Fn(usize, usize) -> String) -> String {
        // HLOOKUP(lookup_value, table_range, row_index, [exact_match])
        if self.args.len() < 3 {
            return "#VALUE!".to_string();
        }
        let lookup_val = self.args[0].resolve_single(get_cell);
        let table_refs = match &self.args[1] {
            Arg::Range(refs) => refs,
            _ => return "#VALUE!".to_string(),
        };
        let row_index = match self.args[2].resolve_number(get_cell) {
            Some(n) => n as usize,
            None => return "#VALUE!".to_string(),
        };
        let _exact = self
            .args
            .get(3)
            .map(|a| {
                let s = a.resolve_single(get_cell).to_uppercase();
                s == "FALSE" || s == "0"
            })
            .unwrap_or(true);

        let (min_row, max_row, min_col, max_col) = range_bounds(table_refs);
        let table_rows = max_row - min_row + 1;

        if row_index == 0 || row_index > table_rows {
            return "#REF!".to_string();
        }

        // Search first row for lookup value
        for c in min_col..=max_col {
            let cell_val = get_cell(min_row, c);
            let matches = cell_val.trim() == lookup_val.trim();
            if matches {
                let result_row = min_row + row_index - 1;
                return get_cell(result_row, c);
            }
        }
        "#N/A".to_string()
    }

    // ---- Logic ----

    fn eval_if(&self, get_cell: &dyn Fn(usize, usize) -> String) -> String {
        // IF(condition, value_if_true, value_if_false)
        if self.args.len() < 2 {
            return "#VALUE!".to_string();
        }
        let condition = evaluate_condition(&self.args[0], get_cell);
        if condition {
            self.args
                .get(1)
                .map(|a| a.resolve_single(get_cell))
                .unwrap_or_default()
        } else {
            self.args
                .get(2)
                .map(|a| a.resolve_single(get_cell))
                .unwrap_or_else(|| "FALSE".to_string())
        }
    }
}

#[derive(Clone, Copy)]
enum CaseOp {
    Upper,
    Lower,
    Proper,
}

#[derive(Clone, Copy)]
enum SubstrOp {
    Left,
    Right,
    Mid,
}

/// Get the bounding box of a set of cell references.
fn range_bounds(refs: &[CellRef]) -> (usize, usize, usize, usize) {
    let mut min_row = usize::MAX;
    let mut max_row = 0;
    let mut min_col = usize::MAX;
    let mut max_col = 0;
    for r in refs {
        min_row = min_row.min(r.row);
        max_row = max_row.max(r.row);
        min_col = min_col.min(r.col);
        max_col = max_col.max(r.col);
    }
    (min_row, max_row, min_col, max_col)
}

// ============================================================================
// Parsing
// ============================================================================

/// Parse a cell reference like "A1", "B10", "AA3" into a CellRef.
/// Row numbers in formulas map directly to document row indices (matching the TUI gutter).
/// Row 0 = header row, Row 1 = first data row.
fn parse_cell_ref(s: &str) -> Option<CellRef> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let letter_end = s
        .char_indices()
        .find(|(_, c)| !c.is_ascii_alphabetic())
        .map(|(i, _)| i)
        .unwrap_or(s.len());

    if letter_end == 0 || letter_end == s.len() {
        return None;
    }

    let col_str = &s[..letter_end];
    let row_str = &s[letter_end..];

    let col = excel_letter_to_column(col_str).ok()?;
    let row: usize = row_str.parse().ok()?;

    Some(CellRef { row, col })
}

/// Expand a range "A1:B3" into a list of CellRefs.
fn expand_range(start: CellRef, end: CellRef) -> Vec<CellRef> {
    let r_start = start.row.min(end.row);
    let r_end = start.row.max(end.row);
    let c_start = start.col.min(end.col);
    let c_end = start.col.max(end.col);

    let mut refs = Vec::new();
    for r in r_start..=r_end {
        for c in c_start..=c_end {
            refs.push(CellRef { row: r, col: c });
        }
    }
    refs
}

/// Split a formula arguments string into individual argument tokens,
/// respecting quoted strings and nested parentheses.
fn split_args(args: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = '"';
    let mut paren_depth = 0;

    for c in args.chars() {
        if in_quote {
            current.push(c);
            if c == quote_char {
                in_quote = false;
            }
        } else if c == '"' || c == '\'' {
            in_quote = true;
            quote_char = c;
            current.push(c);
        } else if c == '(' {
            paren_depth += 1;
            current.push(c);
        } else if c == ')' {
            paren_depth -= 1;
            current.push(c);
        } else if c == ',' && paren_depth == 0 {
            result.push(current.trim().to_string());
            current = String::new();
        } else {
            current.push(c);
        }
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        result.push(trimmed);
    }
    result
}

/// Parse a single argument token into an Arg.
fn parse_arg(token: &str) -> Option<Arg> {
    let token = token.trim();
    if token.is_empty() {
        return None;
    }

    // Check for quoted string
    if (token.starts_with('"') && token.ends_with('"'))
        || (token.starts_with('\'') && token.ends_with('\''))
    {
        let inner = &token[1..token.len() - 1];
        return Some(Arg::Text(inner.to_string()));
    }

    // Check for boolean
    match token.to_uppercase().as_str() {
        "TRUE" => return Some(Arg::Bool(true)),
        "FALSE" => return Some(Arg::Bool(false)),
        _ => {}
    }

    // Check for range (A1:B3)
    if token.contains(':') {
        let parts: Vec<&str> = token.splitn(2, ':').collect();
        if parts.len() == 2 {
            if let (Some(start), Some(end)) = (parse_cell_ref(parts[0]), parse_cell_ref(parts[1])) {
                return Some(Arg::Range(expand_range(start, end)));
            }
        }
    }

    // Check for comparison expression (for IF conditions like A1>10)
    // We parse this as a special Bool arg evaluated at parse time... but we can't
    // evaluate at parse time since we don't have cell values. Instead, we'll store
    // comparisons as a special Arg type. For simplicity, we'll handle this in the
    // IF evaluator by checking the raw condition.
    // Actually, let's handle comparisons as a text arg that IF evaluates specially.
    for op in &[">=", "<=", "!=", "<>", ">", "<", "="] {
        if token.contains(op) {
            return Some(Arg::Text(format!("__CMP__{}", token)));
        }
    }

    // Check for cell reference
    if let Some(cell_ref) = parse_cell_ref(token) {
        return Some(Arg::Cell(cell_ref));
    }

    // Check for number
    if let Some(n) = parse_numeric(token) {
        return Some(Arg::Number(n));
    }

    // Treat as text literal (unquoted)
    Some(Arg::Text(token.to_string()))
}

/// Try to parse a string as a formula.
pub fn parse_formula(input: &str) -> Option<Formula> {
    let input = input.trim();
    if !input.starts_with('=') {
        return None;
    }

    let rest = &input[1..];

    let paren_pos = rest.find('(')?;
    let func_name = rest[..paren_pos].trim().to_uppercase();

    // Find matching closing paren (handle nested parens)
    let after_open = &rest[paren_pos + 1..];
    let mut depth = 1;
    let mut close_offset = None;
    for (i, c) in after_open.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close_offset = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close_offset = close_offset?;
    let args_str = &after_open[..close_offset];

    let func = match func_name.as_str() {
        "SUM" => FormulaFunc::Sum,
        "AVERAGE" | "AVG" => FormulaFunc::Average,
        "MIN" => FormulaFunc::Min,
        "MAX" => FormulaFunc::Max,
        "COUNT" => FormulaFunc::Count,
        "POWER" => FormulaFunc::Power,
        "CEILING" => FormulaFunc::Ceiling,
        "FLOOR" => FormulaFunc::Floor,
        "CONCAT" | "CONCATENATE" => FormulaFunc::Concat,
        "TRIM" => FormulaFunc::Trim,
        "UPPER" => FormulaFunc::Upper,
        "LOWER" => FormulaFunc::Lower,
        "PROPER" => FormulaFunc::Proper,
        "LEFT" => FormulaFunc::Left,
        "RIGHT" => FormulaFunc::Right,
        "MID" => FormulaFunc::Mid,
        "SUBSTITUTE" => FormulaFunc::Substitute,
        "REPLACE" => FormulaFunc::Replace,
        "NOW" => FormulaFunc::Now,
        "TODAY" => FormulaFunc::Today,
        "DATEDIF" => FormulaFunc::DateDif,
        "VLOOKUP" => FormulaFunc::VLookup,
        "HLOOKUP" => FormulaFunc::HLookup,
        "IF" => FormulaFunc::If,
        _ => return None,
    };

    // NOW() and TODAY() take no arguments
    if matches!(func, FormulaFunc::Now | FormulaFunc::Today) {
        return Some(Formula { func, args: vec![] });
    }

    let arg_tokens = split_args(args_str);
    let args: Vec<Arg> = arg_tokens.iter().filter_map(|t| parse_arg(t)).collect();

    if args.is_empty() {
        return None;
    }

    Some(Formula { func, args })
}

// ============================================================================
// Enhanced IF condition evaluation
// ============================================================================

/// Re-implement evaluate_condition to handle comparison expressions stored as __CMP__ text.
fn evaluate_condition(arg: &Arg, get_cell: &dyn Fn(usize, usize) -> String) -> bool {
    match arg {
        Arg::Bool(b) => *b,
        Arg::Number(n) => *n != 0.0,
        Arg::Text(s) if s.starts_with("__CMP__") => {
            let expr = &s[7..]; // strip __CMP__
            eval_comparison(expr, get_cell)
        }
        Arg::Text(s) => !s.is_empty() && s.to_uppercase() != "FALSE",
        Arg::Cell(r) => {
            let val = get_cell(r.row, r.col);
            let trimmed = val.trim();
            if trimmed.is_empty() {
                return false;
            }
            if let Some(n) = parse_numeric(trimmed) {
                n != 0.0
            } else {
                trimmed.to_uppercase() != "FALSE"
            }
        }
        Arg::Range(_) => true,
    }
}

/// Evaluate a comparison expression like "A1>10", "B2>=5", "A1<>B1".
fn eval_comparison(expr: &str, get_cell: &dyn Fn(usize, usize) -> String) -> bool {
    // Try each operator (longest first to avoid prefix matching)
    type CmpOperators<'a> = &'a [(&'a str, fn(&str, &str) -> bool)];
    let operators: CmpOperators = &[
        (">=", cmp_gte),
        ("<=", cmp_lte),
        ("<>", cmp_ne),
        ("!=", cmp_ne),
        (">", cmp_gt),
        ("<", cmp_lt),
        ("=", cmp_eq),
    ];

    for &(op_str, cmp_fn) in operators {
        if let Some(pos) = expr.find(op_str) {
            let left_str = expr[..pos].trim();
            let right_str = expr[pos + op_str.len()..].trim();

            let left_val = resolve_comparison_operand(left_str, get_cell);
            let right_val = resolve_comparison_operand(right_str, get_cell);

            return cmp_fn(&left_val, &right_val);
        }
    }
    false
}

fn resolve_comparison_operand(s: &str, get_cell: &dyn Fn(usize, usize) -> String) -> String {
    // Try as cell ref first
    if let Some(cell_ref) = parse_cell_ref(s) {
        return get_cell(cell_ref.row, cell_ref.col);
    }
    // Try unquoting
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        return s[1..s.len() - 1].to_string();
    }
    s.to_string()
}

// Comparison function helpers — use numeric comparison when both are numeric
fn cmp_gt(a: &str, b: &str) -> bool {
    if let (Some(na), Some(nb)) = (parse_numeric(a), parse_numeric(b)) {
        na > nb
    } else {
        a > b
    }
}
fn cmp_lt(a: &str, b: &str) -> bool {
    if let (Some(na), Some(nb)) = (parse_numeric(a), parse_numeric(b)) {
        na < nb
    } else {
        a < b
    }
}
fn cmp_gte(a: &str, b: &str) -> bool {
    if let (Some(na), Some(nb)) = (parse_numeric(a), parse_numeric(b)) {
        na >= nb
    } else {
        a >= b
    }
}
fn cmp_lte(a: &str, b: &str) -> bool {
    if let (Some(na), Some(nb)) = (parse_numeric(a), parse_numeric(b)) {
        na <= nb
    } else {
        a <= b
    }
}
fn cmp_eq(a: &str, b: &str) -> bool {
    a == b
}
fn cmp_ne(a: &str, b: &str) -> bool {
    a != b
}

// ============================================================================
// FormulaStore
// ============================================================================

/// Stores formulas for cells, keyed by (row, col).
#[derive(Debug, Default)]
pub struct FormulaStore {
    formulas: HashMap<(usize, usize), (String, Formula)>,
}

impl FormulaStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, row: usize, col: usize, raw: String, formula: Formula) {
        self.formulas.insert((row, col), (raw, formula));
    }

    pub fn remove(&mut self, row: usize, col: usize) {
        self.formulas.remove(&(row, col));
    }

    pub fn get_raw(&self, row: usize, col: usize) -> Option<&str> {
        self.formulas.get(&(row, col)).map(|(raw, _)| raw.as_str())
    }

    pub fn get_formula(&self, row: usize, col: usize) -> Option<&Formula> {
        self.formulas.get(&(row, col)).map(|(_, f)| f)
    }

    pub fn cells_referencing(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
        let target = CellRef { row, col };
        self.formulas
            .iter()
            .filter(|(_, (_, formula))| formula.references().contains(&target))
            .map(|(&pos, _)| pos)
            .collect()
    }

    pub fn clear(&mut self) {
        self.formulas.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.formulas.is_empty()
    }

    pub fn clear_on_structural_change(&mut self) {
        self.formulas.clear();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cell_ref() {
        assert_eq!(parse_cell_ref("A1"), Some(CellRef { row: 1, col: 0 }));
        assert_eq!(parse_cell_ref("B3"), Some(CellRef { row: 3, col: 1 }));
        assert_eq!(parse_cell_ref("AA1"), Some(CellRef { row: 1, col: 26 }));
        assert_eq!(parse_cell_ref("A0"), Some(CellRef { row: 0, col: 0 })); // header row
        assert_eq!(parse_cell_ref("1A"), None);
        assert_eq!(parse_cell_ref("A"), None);
        assert_eq!(parse_cell_ref("1"), None);
    }

    #[test]
    fn test_parse_formula_case_insensitive() {
        assert!(parse_formula("=sum(A1:A3)").is_some());
        assert!(parse_formula("=Sum(A1:A3)").is_some());
        assert!(parse_formula("=AVERAGE(A1:A3)").is_some());
        assert!(parse_formula("=avg(A1:A3)").is_some());
        assert!(parse_formula("=min(A1:A3)").is_some());
        assert!(parse_formula("=MAX(A1:A3)").is_some());
        assert!(parse_formula("=count(A1:A3)").is_some());
        assert!(parse_formula("=Power(A1, 2)").is_some());
        assert!(parse_formula("=trim(A1)").is_some());
        assert!(parse_formula("=UPPER(A1)").is_some());
        assert!(parse_formula("=today()").is_some());
    }

    #[test]
    fn test_parse_formula_invalid() {
        assert!(parse_formula("SUM(A1:A3)").is_none());
        assert!(parse_formula("=UNKNOWN(A1:A3)").is_none());
        assert!(parse_formula("=SUM()").is_none());
        assert!(parse_formula("hello").is_none());
    }

    // ---- Aggregate ----

    #[test]
    fn test_evaluate_sum() {
        let f = parse_formula("=SUM(A1:A3)").unwrap();
        let result = f.evaluate(&|row, _col| match row {
            1 => "10".to_string(),
            2 => "20".to_string(),
            3 => "30".to_string(),
            _ => String::new(),
        });
        assert_eq!(result, "60");
    }

    #[test]
    fn test_evaluate_sum_skips_non_numeric() {
        let f = parse_formula("=SUM(A1:A3)").unwrap();
        let result = f.evaluate(&|row, _col| match row {
            1 => "10".to_string(),
            2 => "hello".to_string(),
            3 => "30".to_string(),
            _ => String::new(),
        });
        assert_eq!(result, "40");
    }

    #[test]
    fn test_evaluate_average() {
        let f = parse_formula("=AVERAGE(A1:A3)").unwrap();
        let result = f.evaluate(&|row, _col| match row {
            1 => "10".to_string(),
            2 => "20".to_string(),
            3 => "30".to_string(),
            _ => String::new(),
        });
        assert_eq!(result, "20");
    }

    #[test]
    fn test_evaluate_min_max() {
        let f_min = parse_formula("=MIN(A1:A3)").unwrap();
        let f_max = parse_formula("=MAX(A1:A3)").unwrap();
        let get = |row: usize, _col: usize| match row {
            1 => "10".to_string(),
            2 => "5".to_string(),
            3 => "30".to_string(),
            _ => String::new(),
        };
        assert_eq!(f_min.evaluate(&get), "5");
        assert_eq!(f_max.evaluate(&get), "30");
    }

    #[test]
    fn test_evaluate_count() {
        let f = parse_formula("=COUNT(A1:A5)").unwrap();
        let result = f.evaluate(&|row, _col| match row {
            1 => "10".to_string(),
            2 => "".to_string(),
            3 => "hello".to_string(),
            4 => "  ".to_string(),
            5 => "5".to_string(),
            _ => String::new(),
        });
        assert_eq!(result, "3");
    }

    // ---- Math ----

    #[test]
    fn test_evaluate_power() {
        let f = parse_formula("=POWER(A1, 2)").unwrap();
        let result = f.evaluate(&|row, _col| {
            if row == 1 {
                "3".to_string()
            } else {
                String::new()
            }
        });
        assert_eq!(result, "9");
    }

    #[test]
    fn test_evaluate_ceiling() {
        let f = parse_formula("=CEILING(A1, 5)").unwrap();
        let result = f.evaluate(&|row, _col| {
            if row == 1 {
                "23".to_string()
            } else {
                "5".to_string()
            }
        });
        assert_eq!(result, "25");
    }

    #[test]
    fn test_evaluate_floor() {
        let f = parse_formula("=FLOOR(A1, 5)").unwrap();
        let result = f.evaluate(&|row, _col| {
            if row == 1 {
                "23".to_string()
            } else {
                "5".to_string()
            }
        });
        assert_eq!(result, "20");
    }

    // ---- Text ----

    #[test]
    fn test_evaluate_concat() {
        let f = parse_formula("=CONCAT(A1, \" \", B1)").unwrap();
        let result = f.evaluate(&|row, col| {
            if row == 1 && col == 0 {
                "Hello".to_string()
            } else if row == 1 && col == 1 {
                "World".to_string()
            } else {
                String::new()
            }
        });
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_evaluate_trim() {
        let f = parse_formula("=TRIM(A1)").unwrap();
        let result = f.evaluate(&|row, _| {
            if row == 1 {
                "  hello   world  ".to_string()
            } else {
                String::new()
            }
        });
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_evaluate_upper_lower_proper() {
        let f_upper = parse_formula("=UPPER(A1)").unwrap();
        let f_lower = parse_formula("=LOWER(A1)").unwrap();
        let f_proper = parse_formula("=PROPER(A1)").unwrap();
        let get = |row: usize, _: usize| {
            if row == 1 {
                "hello world".to_string()
            } else {
                String::new()
            }
        };
        assert_eq!(f_upper.evaluate(&get), "HELLO WORLD");
        assert_eq!(f_lower.evaluate(&get), "hello world");
        assert_eq!(f_proper.evaluate(&get), "Hello World");
    }

    #[test]
    fn test_evaluate_left_right_mid() {
        let f_left = parse_formula("=LEFT(A1, 3)").unwrap();
        let f_right = parse_formula("=RIGHT(A1, 3)").unwrap();
        let f_mid = parse_formula("=MID(A1, 2, 3)").unwrap();
        let get = |row: usize, _: usize| {
            if row == 1 {
                "Hello".to_string()
            } else {
                String::new()
            }
        };
        assert_eq!(f_left.evaluate(&get), "Hel");
        assert_eq!(f_right.evaluate(&get), "llo");
        assert_eq!(f_mid.evaluate(&get), "ell");
    }

    #[test]
    fn test_evaluate_substitute() {
        let f = parse_formula("=SUBSTITUTE(A1, \"Old\", \"New\")").unwrap();
        let result = f.evaluate(&|row, _| {
            if row == 1 {
                "Old text with Old parts".to_string()
            } else {
                String::new()
            }
        });
        assert_eq!(result, "New text with New parts");
    }

    #[test]
    fn test_evaluate_replace() {
        let f = parse_formula("=REPLACE(A1, 2, 3, \"XYZ\")").unwrap();
        let result = f.evaluate(&|row, _| {
            if row == 1 {
                "Hello".to_string()
            } else {
                String::new()
            }
        });
        assert_eq!(result, "HXYZo");
    }

    // ---- Date ----

    #[test]
    fn test_evaluate_today() {
        let f = parse_formula("=TODAY()").unwrap();
        let result = f.evaluate(&|_, _| String::new());
        let today = Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(result, today);
    }

    #[test]
    fn test_evaluate_datedif_days() {
        let f = parse_formula("=DATEDIF(A1, B1, \"d\")").unwrap();
        let result = f.evaluate(&|row, col| {
            if row == 1 && col == 0 {
                "2024-01-01".to_string()
            } else if row == 1 && col == 1 {
                "2024-01-31".to_string()
            } else {
                String::new()
            }
        });
        assert_eq!(result, "30");
    }

    #[test]
    fn test_evaluate_datedif_months() {
        let f = parse_formula("=DATEDIF(A1, B1, \"m\")").unwrap();
        let result = f.evaluate(&|row, col| {
            if row == 1 && col == 0 {
                "2024-01-15".to_string()
            } else if row == 1 && col == 1 {
                "2024-06-15".to_string()
            } else {
                String::new()
            }
        });
        assert_eq!(result, "5");
    }

    // ---- Lookup ----

    #[test]
    fn test_evaluate_vlookup() {
        // Table in A1:C3 (rows 1-3):
        let f = parse_formula("=VLOOKUP(\"Banana\", A1:C3, 3, FALSE)").unwrap();
        let result = f.evaluate(&|row, col| match (row, col) {
            (1, 0) => "Apple".to_string(),
            (1, 1) => "Red".to_string(),
            (1, 2) => "1".to_string(),
            (2, 0) => "Banana".to_string(),
            (2, 1) => "Yellow".to_string(),
            (2, 2) => "2".to_string(),
            (3, 0) => "Cherry".to_string(),
            (3, 1) => "Red".to_string(),
            (3, 2) => "3".to_string(),
            _ => String::new(),
        });
        assert_eq!(result, "2");
    }

    #[test]
    fn test_evaluate_vlookup_not_found() {
        let f = parse_formula("=VLOOKUP(\"Grape\", A1:B2, 2, FALSE)").unwrap();
        let result = f.evaluate(&|row, col| match (row, col) {
            (1, 0) => "Apple".to_string(),
            (1, 1) => "1".to_string(),
            (2, 0) => "Banana".to_string(),
            (2, 1) => "2".to_string(),
            _ => String::new(),
        });
        assert_eq!(result, "#N/A");
    }

    // ---- Logic ----

    #[test]
    fn test_evaluate_if_true() {
        let f = parse_formula("=IF(A1>10, \"High\", \"Low\")").unwrap();
        let result = f.evaluate(&|row, _| {
            if row == 1 {
                "15".to_string()
            } else {
                String::new()
            }
        });
        assert_eq!(result, "High");
    }

    #[test]
    fn test_evaluate_if_false() {
        let f = parse_formula("=IF(A1>10, \"High\", \"Low\")").unwrap();
        let result = f.evaluate(&|row, _| {
            if row == 1 {
                "5".to_string()
            } else {
                String::new()
            }
        });
        assert_eq!(result, "Low");
    }

    // ---- Formula Store ----

    #[test]
    fn test_formula_store_basic() {
        let mut store = FormulaStore::new();
        let f = parse_formula("=SUM(A1:A3)").unwrap();
        store.set(5, 0, "=SUM(A1:A3)".to_string(), f);

        assert_eq!(store.get_raw(5, 0), Some("=SUM(A1:A3)"));
        assert!(store.get_formula(5, 0).is_some());
        assert!(store.get_raw(0, 0).is_none());

        // A2 (row 2, col 0) is referenced by the formula
        let deps = store.cells_referencing(2, 0);
        assert_eq!(deps, vec![(5, 0)]);

        let deps = store.cells_referencing(5, 5);
        assert!(deps.is_empty());
    }

    #[test]
    fn test_formula_store_remove() {
        let mut store = FormulaStore::new();
        let f = parse_formula("=SUM(A1:A3)").unwrap();
        store.set(5, 0, "=SUM(A1:A3)".to_string(), f);
        store.remove(5, 0);
        assert!(store.get_raw(5, 0).is_none());
        assert!(store.is_empty());
    }

    #[test]
    fn test_parse_formula_with_spaces() {
        let f = parse_formula("= SUM( A1 : A3 )").unwrap();
        assert!(matches!(f.func, FormulaFunc::Sum));
    }

    #[test]
    fn test_parse_formula_comma_list() {
        let f = parse_formula("=SUM(A1,A3,B2)").unwrap();
        assert!(matches!(f.func, FormulaFunc::Sum));
        assert_eq!(f.references().len(), 3);
    }

    #[test]
    fn test_split_args_with_quotes() {
        let args = split_args("A1, \" hello, world \", B1");
        assert_eq!(args.len(), 3);
        assert_eq!(args[0], "A1");
        assert_eq!(args[1], "\" hello, world \"");
        assert_eq!(args[2], "B1");
    }

    #[test]
    fn test_evaluate_concatenate_alias() {
        let f = parse_formula("=CONCATENATE(A1, B1)").unwrap();
        assert!(matches!(f.func, FormulaFunc::Concat));
    }

    #[test]
    fn test_evaluate_hlookup() {
        // Table:
        // A1=Name  B1=Age   C1=City
        // A2=Alice B2=30    C2=NYC
        let f = parse_formula("=HLOOKUP(\"Age\", A1:C2, 2, FALSE)").unwrap();
        let result = f.evaluate(&|row, col| match (row, col) {
            (1, 0) => "Name".to_string(),
            (1, 1) => "Age".to_string(),
            (1, 2) => "City".to_string(),
            (2, 0) => "Alice".to_string(),
            (2, 1) => "30".to_string(),
            (2, 2) => "NYC".to_string(),
            _ => String::new(),
        });
        assert_eq!(result, "30");
    }
}
