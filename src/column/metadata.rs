/// Column data type annotation for validation and type-aware sorting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    Text,
    Number,
    Date,
    Boolean,
}

impl ColumnType {
    /// Parse a type name string (case-insensitive).
    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "text" | "string" | "str" => Some(Self::Text),
            "number" | "num" | "numeric" | "int" | "float" | "decimal" => Some(Self::Number),
            "date" => Some(Self::Date),
            "boolean" | "bool" => Some(Self::Boolean),
            _ => None,
        }
    }

    /// Display name for status messages and column indicators.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Number => "number",
            Self::Date => "date",
            Self::Boolean => "boolean",
        }
    }

    /// Short indicator for the column header (e.g., "#" for number).
    pub fn indicator(&self) -> &'static str {
        match self {
            Self::Text => "",
            Self::Number => "#",
            Self::Date => "D",
            Self::Boolean => "?",
        }
    }

    /// Validate a cell value against this type. Empty values are always valid.
    pub fn validate(&self, value: &str) -> Result<(), String> {
        if value.is_empty() {
            return Ok(());
        }
        match self {
            Self::Text => Ok(()),
            Self::Number => value
                .replace(',', "")
                .parse::<f64>()
                .map(|_| ())
                .map_err(|_| format!("'{}' is not a valid number", value)),
            Self::Date => parse_date(value)
                .map(|_| ())
                .ok_or_else(|| format!("'{}' is not a valid date (expected YYYY-MM-DD)", value)),
            Self::Boolean => {
                let lower = value.to_lowercase();
                if matches!(
                    lower.as_str(),
                    "true" | "false" | "yes" | "no" | "1" | "0" | "y" | "n" | "t" | "f"
                ) {
                    Ok(())
                } else {
                    Err(format!(
                        "'{}' is not a valid boolean (expected true/false/yes/no/1/0)",
                        value
                    ))
                }
            }
        }
    }

    /// Compare two cell values using this type's ordering.
    /// Returns None if comparison is not possible (parse failures fall back to text).
    pub fn compare(&self, a: &str, b: &str) -> std::cmp::Ordering {
        match self {
            Self::Number => {
                let pa = a.replace(',', "").parse::<f64>().ok();
                let pb = b.replace(',', "").parse::<f64>().ok();
                match (pa, pb) {
                    (Some(va), Some(vb)) => {
                        va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.cmp(b),
                }
            }
            Self::Date => {
                let da = parse_date(a);
                let db = parse_date(b);
                match (da, db) {
                    (Some(va), Some(vb)) => va.cmp(&vb),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.cmp(b),
                }
            }
            Self::Boolean => {
                let ba = bool_ord(a);
                let bb = bool_ord(b);
                ba.cmp(&bb)
            }
            Self::Text => a.to_lowercase().cmp(&b.to_lowercase()),
        }
    }
}

/// Parse a date string in common formats. Returns (year, month, day) for ordering.
fn parse_date(s: &str) -> Option<(i32, u32, u32)> {
    // ISO 8601: YYYY-MM-DD
    if let Some((y, rest)) = s.split_once('-') {
        if let Some((m, d)) = rest.split_once('-') {
            if let (Ok(y), Ok(m), Ok(d)) = (y.parse::<i32>(), m.parse::<u32>(), d.parse::<u32>()) {
                if (1..=12).contains(&m) && (1..=31).contains(&d) {
                    return Some((y, m, d));
                }
            }
        }
    }
    // US format: MM/DD/YYYY
    if let Some((m, rest)) = s.split_once('/') {
        if let Some((d, y)) = rest.split_once('/') {
            if let (Ok(m), Ok(d), Ok(y)) = (m.parse::<u32>(), d.parse::<u32>(), y.parse::<i32>()) {
                if (1..=12).contains(&m) && (1..=31).contains(&d) {
                    return Some((y, m, d));
                }
            }
        }
    }
    None
}

/// Map boolean-like values to a sort order (false < true).
fn bool_ord(s: &str) -> u8 {
    match s.to_lowercase().as_str() {
        "true" | "yes" | "1" | "y" | "t" => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_name_valid() {
        assert_eq!(ColumnType::from_name("number"), Some(ColumnType::Number));
        assert_eq!(ColumnType::from_name("NUM"), Some(ColumnType::Number));
        assert_eq!(ColumnType::from_name("date"), Some(ColumnType::Date));
        assert_eq!(ColumnType::from_name("boolean"), Some(ColumnType::Boolean));
        assert_eq!(ColumnType::from_name("bool"), Some(ColumnType::Boolean));
        assert_eq!(ColumnType::from_name("text"), Some(ColumnType::Text));
        assert_eq!(ColumnType::from_name("string"), Some(ColumnType::Text));
    }

    #[test]
    fn test_from_name_invalid() {
        assert_eq!(ColumnType::from_name("invalid"), None);
        assert_eq!(ColumnType::from_name(""), None);
    }

    #[test]
    fn test_validate_number() {
        let t = ColumnType::Number;
        assert!(t.validate("42").is_ok());
        assert!(t.validate("3.14").is_ok());
        assert!(t.validate("-100").is_ok());
        assert!(t.validate("1,234.56").is_ok());
        assert!(t.validate("").is_ok()); // empty always valid
        assert!(t.validate("abc").is_err());
        assert!(t.validate("12abc").is_err());
    }

    #[test]
    fn test_validate_date() {
        let t = ColumnType::Date;
        assert!(t.validate("2024-01-15").is_ok());
        assert!(t.validate("1/15/2024").is_ok());
        assert!(t.validate("").is_ok());
        assert!(t.validate("not-a-date").is_err());
        assert!(t.validate("2024-13-01").is_err()); // month 13
    }

    #[test]
    fn test_validate_boolean() {
        let t = ColumnType::Boolean;
        for v in &["true", "false", "yes", "no", "1", "0", "y", "n", "t", "f"] {
            assert!(t.validate(v).is_ok(), "expected '{}' to be valid", v);
        }
        assert!(t.validate("TRUE").is_ok());
        assert!(t.validate("").is_ok());
        assert!(t.validate("maybe").is_err());
    }

    #[test]
    fn test_validate_text() {
        let t = ColumnType::Text;
        assert!(t.validate("anything").is_ok());
        assert!(t.validate("").is_ok());
    }

    #[test]
    fn test_compare_number() {
        let t = ColumnType::Number;
        assert_eq!(t.compare("10", "2"), std::cmp::Ordering::Greater);
        assert_eq!(t.compare("2", "10"), std::cmp::Ordering::Less);
        assert_eq!(t.compare("5", "5"), std::cmp::Ordering::Equal);
        assert_eq!(t.compare("1,000", "999"), std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_compare_date() {
        let t = ColumnType::Date;
        assert_eq!(
            t.compare("2024-01-15", "2024-01-14"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            t.compare("2023-12-31", "2024-01-01"),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn test_compare_boolean() {
        let t = ColumnType::Boolean;
        assert_eq!(t.compare("true", "false"), std::cmp::Ordering::Greater);
        assert_eq!(t.compare("no", "yes"), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_compare_text() {
        let t = ColumnType::Text;
        assert_eq!(t.compare("apple", "Banana"), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_indicator() {
        assert_eq!(ColumnType::Number.indicator(), "#");
        assert_eq!(ColumnType::Date.indicator(), "D");
        assert_eq!(ColumnType::Boolean.indicator(), "?");
        assert_eq!(ColumnType::Text.indicator(), "");
    }
}
