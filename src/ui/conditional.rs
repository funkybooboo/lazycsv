//! Conditional formatting rules for per-cell color styling.
//!
//! Rules are attached to columns and evaluated against cell values during rendering.
//! Multiple rules per column are supported; first match wins.

use ratatui::style::Color;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// The condition that determines whether a rule matches a cell value.
#[derive(Debug, Clone)]
pub enum ConditionType {
    /// No condition — applies to all cells in the column.
    Always,
    /// Exact string equality.
    Equals(String),
    /// String inequality.
    NotEquals(String),
    /// Numeric greater than.
    GreaterThan(f64),
    /// Numeric less than.
    LessThan(f64),
    /// Numeric greater than or equal.
    GreaterThanOrEqual(f64),
    /// Numeric less than or equal.
    LessThanOrEqual(f64),
    /// Regex pattern match.
    RegexMatch(Regex),
    /// All sub-conditions must match.
    And(Vec<ConditionType>),
    /// Any sub-condition must match.
    Or(Vec<ConditionType>),
}

impl ConditionType {
    /// Test whether a cell value matches this condition.
    pub fn matches(&self, value: &str) -> bool {
        match self {
            ConditionType::Always => true,
            ConditionType::Equals(s) => value == s,
            ConditionType::NotEquals(s) => value != s,
            ConditionType::GreaterThan(n) => parse_numeric(value).is_some_and(|v| v > *n),
            ConditionType::LessThan(n) => parse_numeric(value).is_some_and(|v| v < *n),
            ConditionType::GreaterThanOrEqual(n) => parse_numeric(value).is_some_and(|v| v >= *n),
            ConditionType::LessThanOrEqual(n) => parse_numeric(value).is_some_and(|v| v <= *n),
            ConditionType::RegexMatch(re) => re.is_match(value),
            ConditionType::And(conditions) => conditions.iter().all(|c| c.matches(value)),
            ConditionType::Or(conditions) => conditions.iter().any(|c| c.matches(value)),
        }
    }
}

/// A color rule: a condition plus a color.
#[derive(Debug, Clone)]
pub struct ColorRule {
    pub condition: ConditionType,
    pub color: Color,
}

/// Serializable form of a color rule for views.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableColorRule {
    pub color: String,
    /// Condition operator: "always", "=", "!=", ">", "<", ">=", "<=", "~", "and", "or"
    #[serde(default = "default_operator")]
    pub op: String,
    /// The value/pattern for the condition (empty for "always").
    #[serde(default)]
    pub value: String,
    /// Sub-conditions for "and"/"or" compound conditions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<SerializableCondition>,
}

/// A single serializable condition (used inside compound and/or).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableCondition {
    pub op: String,
    pub value: String,
}

fn default_operator() -> String {
    "always".to_string()
}

/// Serialize a single ConditionType to (op, value) pair.
fn serialize_condition(cond: &ConditionType) -> (String, String) {
    match cond {
        ConditionType::Always => ("always".to_string(), String::new()),
        ConditionType::Equals(s) => ("=".to_string(), s.clone()),
        ConditionType::NotEquals(s) => ("!=".to_string(), s.clone()),
        ConditionType::GreaterThan(n) => (">".to_string(), n.to_string()),
        ConditionType::LessThan(n) => ("<".to_string(), n.to_string()),
        ConditionType::GreaterThanOrEqual(n) => (">=".to_string(), n.to_string()),
        ConditionType::LessThanOrEqual(n) => ("<=".to_string(), n.to_string()),
        ConditionType::RegexMatch(re) => ("~".to_string(), re.as_str().to_string()),
        // Compound types shouldn't appear as sub-conditions, but handle gracefully
        ConditionType::And(_) | ConditionType::Or(_) => ("always".to_string(), String::new()),
    }
}

/// Deserialize a single (op, value) pair to a ConditionType.
fn deserialize_condition(op: &str, value: &str) -> Option<ConditionType> {
    match op {
        "always" | "" => Some(ConditionType::Always),
        "=" => Some(ConditionType::Equals(value.to_string())),
        "!=" => Some(ConditionType::NotEquals(value.to_string())),
        ">" => value.parse().ok().map(ConditionType::GreaterThan),
        "<" => value.parse().ok().map(ConditionType::LessThan),
        ">=" => value.parse().ok().map(ConditionType::GreaterThanOrEqual),
        "<=" => value.parse().ok().map(ConditionType::LessThanOrEqual),
        "~" => Regex::new(value).ok().map(ConditionType::RegexMatch),
        _ => None,
    }
}

/// Convert a ColorRule to its serializable form.
pub fn serialize_rule(rule: &ColorRule) -> SerializableColorRule {
    let color_str = crate::config::views::color_to_string(rule.color);
    match &rule.condition {
        ConditionType::And(subs) => SerializableColorRule {
            color: color_str,
            op: "and".to_string(),
            value: String::new(),
            conditions: subs
                .iter()
                .map(|c| {
                    let (op, value) = serialize_condition(c);
                    SerializableCondition { op, value }
                })
                .collect(),
        },
        ConditionType::Or(subs) => SerializableColorRule {
            color: color_str,
            op: "or".to_string(),
            value: String::new(),
            conditions: subs
                .iter()
                .map(|c| {
                    let (op, value) = serialize_condition(c);
                    SerializableCondition { op, value }
                })
                .collect(),
        },
        other => {
            let (op, value) = serialize_condition(other);
            SerializableColorRule {
                color: color_str,
                op,
                value,
                conditions: Vec::new(),
            }
        }
    }
}

/// Convert a serializable rule back to a ColorRule. Returns None if color or regex is invalid.
pub fn deserialize_rule(sr: &SerializableColorRule) -> Option<ColorRule> {
    let color = crate::config::parse_color(&sr.color)?;
    let condition = match sr.op.as_str() {
        "and" => {
            let subs: Option<Vec<ConditionType>> = sr
                .conditions
                .iter()
                .map(|c| deserialize_condition(&c.op, &c.value))
                .collect();
            ConditionType::And(subs?)
        }
        "or" => {
            let subs: Option<Vec<ConditionType>> = sr
                .conditions
                .iter()
                .map(|c| deserialize_condition(&c.op, &c.value))
                .collect();
            ConditionType::Or(subs?)
        }
        _ => deserialize_condition(&sr.op, &sr.value)?,
    };
    Some(ColorRule { condition, color })
}

/// Evaluate a list of rules against a cell value, returning the first matching color.
pub fn evaluate_rules(rules: &[ColorRule], value: &str) -> Option<Color> {
    rules
        .iter()
        .find(|rule| rule.condition.matches(value))
        .map(|rule| rule.color)
}

/// Format a condition for display (used by :bgcolor list).
pub fn format_condition(cond: &ConditionType) -> String {
    match cond {
        ConditionType::Always => "always".to_string(),
        ConditionType::Equals(s) => format!("= \"{}\"", s),
        ConditionType::NotEquals(s) => format!("!= \"{}\"", s),
        ConditionType::GreaterThan(n) => format!("> {}", n),
        ConditionType::LessThan(n) => format!("< {}", n),
        ConditionType::GreaterThanOrEqual(n) => format!(">= {}", n),
        ConditionType::LessThanOrEqual(n) => format!("<= {}", n),
        ConditionType::RegexMatch(re) => format!("~ \"{}\"", re.as_str()),
        ConditionType::And(subs) => subs
            .iter()
            .map(format_condition)
            .collect::<Vec<_>>()
            .join(" && "),
        ConditionType::Or(subs) => subs
            .iter()
            .map(format_condition)
            .collect::<Vec<_>>()
            .join(" || "),
    }
}

// ── Row conditional rules ─────────────────────────────────────

/// A column-qualified condition: check a specific column's value.
#[derive(Debug, Clone)]
pub struct ColumnCondition {
    pub col_index: usize,
    pub condition: ConditionType,
}

/// Compound column conditions with AND/OR.
#[derive(Debug, Clone)]
pub enum RowCondition {
    Single(ColumnCondition),
    And(Vec<RowCondition>),
    Or(Vec<RowCondition>),
}

impl RowCondition {
    /// Evaluate this condition against a row's column values.
    /// `get_value` returns the cell value for a given column index.
    pub fn matches(&self, get_value: &dyn Fn(usize) -> String) -> bool {
        match self {
            RowCondition::Single(cc) => {
                let val = get_value(cc.col_index);
                cc.condition.matches(&val)
            }
            RowCondition::And(subs) => subs.iter().all(|s| s.matches(get_value)),
            RowCondition::Or(subs) => subs.iter().any(|s| s.matches(get_value)),
        }
    }
}

/// A row conditional rule: color + condition referencing column values.
#[derive(Debug, Clone)]
pub struct RowConditionalRule {
    pub color: Color,
    pub condition: RowCondition,
}

/// Serializable column condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableColumnCondition {
    pub col: usize,
    pub op: String,
    pub value: String,
}

/// Serializable row conditional rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableRowConditionalRule {
    pub color: String,
    /// "single", "and", "or"
    pub kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<SerializableColumnCondition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sub_rules: Vec<SerializableRowConditionalRule>,
}

pub fn serialize_row_rule(rule: &RowConditionalRule) -> SerializableRowConditionalRule {
    let color_str = crate::config::views::color_to_string(rule.color);
    serialize_row_condition(&rule.condition, &color_str)
}

fn serialize_row_condition(cond: &RowCondition, color: &str) -> SerializableRowConditionalRule {
    match cond {
        RowCondition::Single(cc) => {
            let (op, value) = serialize_condition(&cc.condition);
            SerializableRowConditionalRule {
                color: color.to_string(),
                kind: "single".to_string(),
                conditions: vec![SerializableColumnCondition {
                    col: cc.col_index,
                    op,
                    value,
                }],
                sub_rules: Vec::new(),
            }
        }
        RowCondition::And(subs) => SerializableRowConditionalRule {
            color: color.to_string(),
            kind: "and".to_string(),
            conditions: Vec::new(),
            sub_rules: subs
                .iter()
                .map(|s| serialize_row_condition(s, color))
                .collect(),
        },
        RowCondition::Or(subs) => SerializableRowConditionalRule {
            color: color.to_string(),
            kind: "or".to_string(),
            conditions: Vec::new(),
            sub_rules: subs
                .iter()
                .map(|s| serialize_row_condition(s, color))
                .collect(),
        },
    }
}

pub fn deserialize_row_rule(sr: &SerializableRowConditionalRule) -> Option<RowConditionalRule> {
    let color = crate::config::parse_color(&sr.color)?;
    let condition = deserialize_row_condition(sr)?;
    Some(RowConditionalRule { color, condition })
}

fn deserialize_row_condition(sr: &SerializableRowConditionalRule) -> Option<RowCondition> {
    match sr.kind.as_str() {
        "single" => {
            let cc = sr.conditions.first()?;
            let cond = deserialize_condition(&cc.op, &cc.value)?;
            Some(RowCondition::Single(ColumnCondition {
                col_index: cc.col,
                condition: cond,
            }))
        }
        "and" => {
            let subs: Option<Vec<RowCondition>> =
                sr.sub_rules.iter().map(deserialize_row_condition).collect();
            Some(RowCondition::And(subs?))
        }
        "or" => {
            let subs: Option<Vec<RowCondition>> =
                sr.sub_rules.iter().map(deserialize_row_condition).collect();
            Some(RowCondition::Or(subs?))
        }
        _ => None,
    }
}

/// Evaluate row conditional rules against a row, returning the first matching color.
pub fn evaluate_row_conditional_rules(
    rules: &[RowConditionalRule],
    get_value: &dyn Fn(usize) -> String,
) -> Option<Color> {
    rules
        .iter()
        .find(|rule| rule.condition.matches(get_value))
        .map(|rule| rule.color)
}

/// Format a row condition for display.
pub fn format_row_condition(cond: &RowCondition, headers: &[String]) -> String {
    match cond {
        RowCondition::Single(cc) => {
            let col_name = headers
                .get(cc.col_index)
                .cloned()
                .unwrap_or_else(|| column_index_to_letter(cc.col_index));
            format!("{} {}", col_name, format_condition(&cc.condition))
        }
        RowCondition::And(subs) => subs
            .iter()
            .map(|s| format_row_condition(s, headers))
            .collect::<Vec<_>>()
            .join(" && "),
        RowCondition::Or(subs) => subs
            .iter()
            .map(|s| format_row_condition(s, headers))
            .collect::<Vec<_>>()
            .join(" || "),
    }
}

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

/// Parse a numeric value from a cell string, stripping common prefixes like '$' and commas.
fn parse_numeric(s: &str) -> Option<f64> {
    let trimmed = s.trim();
    let stripped = trimmed.strip_prefix('$').unwrap_or(trimmed);
    let cleaned: String = stripped.chars().filter(|c| *c != ',').collect();
    cleaned.parse().ok()
}
