//! Cell value transforms for data cleanup.
//!
//! Pure functions that transform cell values:
//! - Case transforms: toggle, upper, lower, title
//! - Boolean toggle: true/false, yes/no, 1/0
//! - Whitespace: trim

/// Toggle the case of a string (upper → lower, lower → upper, mixed → lower).
pub fn toggle_case(s: &str) -> String {
    if s.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()) {
        s.to_lowercase()
    } else {
        s.to_uppercase()
    }
}

/// Convert to uppercase.
pub fn to_upper(s: &str) -> String {
    s.to_uppercase()
}

/// Convert to lowercase.
pub fn to_lower(s: &str) -> String {
    s.to_lowercase()
}

/// Convert to title case (first letter of each word uppercased).
pub fn to_title(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    let rest: String = chars.as_str().to_lowercase();
                    format!("{}{}", upper, rest)
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Toggle boolean values. Returns None if the value isn't recognized as boolean.
pub fn toggle_boolean(s: &str) -> Option<String> {
    match s.trim().to_lowercase().as_str() {
        "true" => Some("false".into()),
        "false" => Some("true".into()),
        "yes" => Some("no".into()),
        "no" => Some("yes".into()),
        "1" => Some("0".into()),
        "0" => Some("1".into()),
        "on" => Some("off".into()),
        "off" => Some("on".into()),
        "y" => Some("n".into()),
        "n" => Some("y".into()),
        _ => None,
    }
}

/// Trim leading and trailing whitespace.
pub fn trim(s: &str) -> String {
    s.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_case_lower_to_upper() {
        assert_eq!(toggle_case("hello"), "HELLO");
    }

    #[test]
    fn test_toggle_case_upper_to_lower() {
        assert_eq!(toggle_case("HELLO"), "hello");
    }

    #[test]
    fn test_toggle_case_mixed_to_upper() {
        assert_eq!(toggle_case("Hello World"), "HELLO WORLD");
    }

    #[test]
    fn test_toggle_case_empty() {
        assert_eq!(toggle_case(""), "");
    }

    #[test]
    fn test_toggle_case_numbers() {
        assert_eq!(toggle_case("123"), "123");
    }

    #[test]
    fn test_to_upper() {
        assert_eq!(to_upper("hello world"), "HELLO WORLD");
    }

    #[test]
    fn test_to_lower() {
        assert_eq!(to_lower("HELLO WORLD"), "hello world");
    }

    #[test]
    fn test_to_title() {
        assert_eq!(to_title("hello world"), "Hello World");
    }

    #[test]
    fn test_to_title_already_titled() {
        assert_eq!(to_title("Hello World"), "Hello World");
    }

    #[test]
    fn test_to_title_all_caps() {
        assert_eq!(to_title("HELLO WORLD"), "Hello World");
    }

    #[test]
    fn test_to_title_single_word() {
        assert_eq!(to_title("hello"), "Hello");
    }

    #[test]
    fn test_to_title_empty() {
        assert_eq!(to_title(""), "");
    }

    #[test]
    fn test_toggle_boolean_true_false() {
        assert_eq!(toggle_boolean("true"), Some("false".into()));
        assert_eq!(toggle_boolean("false"), Some("true".into()));
    }

    #[test]
    fn test_toggle_boolean_yes_no() {
        assert_eq!(toggle_boolean("yes"), Some("no".into()));
        assert_eq!(toggle_boolean("no"), Some("yes".into()));
    }

    #[test]
    fn test_toggle_boolean_1_0() {
        assert_eq!(toggle_boolean("1"), Some("0".into()));
        assert_eq!(toggle_boolean("0"), Some("1".into()));
    }

    #[test]
    fn test_toggle_boolean_on_off() {
        assert_eq!(toggle_boolean("on"), Some("off".into()));
        assert_eq!(toggle_boolean("off"), Some("on".into()));
    }

    #[test]
    fn test_toggle_boolean_y_n() {
        assert_eq!(toggle_boolean("y"), Some("n".into()));
        assert_eq!(toggle_boolean("n"), Some("y".into()));
    }

    #[test]
    fn test_toggle_boolean_case_insensitive() {
        assert_eq!(toggle_boolean("TRUE"), Some("false".into()));
        assert_eq!(toggle_boolean("False"), Some("true".into()));
        assert_eq!(toggle_boolean("YES"), Some("no".into()));
    }

    #[test]
    fn test_toggle_boolean_not_boolean() {
        assert_eq!(toggle_boolean("hello"), None);
        assert_eq!(toggle_boolean("42"), None);
        assert_eq!(toggle_boolean(""), None);
    }

    #[test]
    fn test_toggle_boolean_with_whitespace() {
        assert_eq!(toggle_boolean(" true "), Some("false".into()));
    }

    #[test]
    fn test_trim() {
        assert_eq!(trim("  hello  "), "hello");
        assert_eq!(trim("hello"), "hello");
        assert_eq!(trim("  "), "");
        assert_eq!(trim("\thello\n"), "hello");
    }
}
