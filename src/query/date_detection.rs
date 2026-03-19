//! Date format detection and ISO normalization for CSV columns.
//!
//! Samples column values to detect date patterns, determines the format
//! from system locale (with data-based disambiguation), and normalizes
//! non-ISO dates to ISO 8601 format for correct SQLite comparison/ordering.

/// Detected date format for a column.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DateFormat {
    /// YYYY-MM-DD with optional time — already ISO, no date conversion needed
    Iso,
    /// DD/MM/YYYY with optional time (day-first slash)
    SlashDayFirst,
    /// MM/DD/YYYY with optional time (month-first slash)
    SlashMonthFirst,
    /// DD.MM.YYYY with optional time (dot separator, always day-first)
    Dot,
}

/// Column type detected from sampling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnType {
    /// Use NUMERIC affinity — numbers compare correctly, text stays text
    Numeric,
    /// Date column — values will be normalized to ISO format, stored as TEXT
    Date(DateFormat),
}

/// Maximum number of data rows to sample for type detection.
const SAMPLE_SIZE: usize = 100;

/// Fraction of non-empty sampled values that must match a date pattern.
const DATE_THRESHOLD: f64 = 0.9;

/// Detect column types by sampling document data rows (row 0 excluded).
///
/// `data_rows` should NOT include row 0 (which typically contains column names).
/// Returns a `ColumnType` for each column.
pub fn detect_column_types(data_rows: &[Vec<String>], col_count: usize) -> Vec<ColumnType> {
    if data_rows.is_empty() || col_count == 0 {
        return vec![ColumnType::Numeric; col_count];
    }

    let sample = &data_rows[..data_rows.len().min(SAMPLE_SIZE)];
    let locale_month_first = locale_uses_month_first();

    (0..col_count)
        .map(|col_idx| detect_single_column(sample, col_idx, locale_month_first))
        .collect()
}

/// Detect column types from string slices (for the streaming CSV loader).
///
/// Each row is a slice of `&str` cell values. Header excluded.
#[allow(dead_code)]
pub fn detect_column_types_from_strs(rows: &[Vec<&str>], col_count: usize) -> Vec<ColumnType> {
    if rows.is_empty() || col_count == 0 {
        return vec![ColumnType::Numeric; col_count];
    }

    let sample = &rows[..rows.len().min(SAMPLE_SIZE)];
    let locale_month_first = locale_uses_month_first();

    (0..col_count)
        .map(|col_idx| {
            let values: Vec<&str> = sample
                .iter()
                .filter_map(|row| row.get(col_idx).copied())
                .filter(|s| !s.is_empty())
                .collect();
            classify_values(&values, locale_month_first)
        })
        .collect()
}

/// Build the SQLite affinity string for a detected column type.
pub fn sqlite_affinity(col_type: &ColumnType) -> &'static str {
    match col_type {
        ColumnType::Numeric => "NUMERIC",
        ColumnType::Date(_) => "TEXT",
    }
}

/// Normalize a cell value to ISO format based on detected date format.
///
/// Returns the original value unchanged if parsing fails (graceful degradation).
pub fn normalize_to_iso(value: &str, format: DateFormat) -> String {
    if value.is_empty() {
        return String::new();
    }
    match format {
        DateFormat::Iso => value.to_string(),
        DateFormat::SlashDayFirst => normalize_slash(value, true),
        DateFormat::SlashMonthFirst => normalize_slash(value, false),
        DateFormat::Dot => normalize_dot(value),
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Check system locale to determine if month comes first in dates.
/// Returns true for US-style MM/DD/YYYY locales.
fn locale_uses_month_first() -> bool {
    let locale = std::env::var("LC_TIME")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();

    // Extract country code: "en_US.UTF-8" -> "US"
    let country = locale
        .split('.')
        .next()
        .and_then(|s| s.split('_').nth(1))
        .unwrap_or("");

    // Countries using MM/DD/YYYY
    matches!(country, "US" | "PH" | "FM" | "MH" | "PW")
}

fn detect_single_column(
    sample: &[Vec<String>],
    col_idx: usize,
    locale_month_first: bool,
) -> ColumnType {
    let values: Vec<&str> = sample
        .iter()
        .filter_map(|row| row.get(col_idx).map(|s| s.as_str()))
        .filter(|s| !s.is_empty())
        .collect();

    classify_values(&values, locale_month_first)
}

fn classify_values(values: &[&str], locale_month_first: bool) -> ColumnType {
    if values.len() < 3 {
        return ColumnType::Numeric;
    }

    // Try ISO first (unambiguous)
    let iso_count = values.iter().filter(|v| is_iso_date(v)).count();
    if iso_count as f64 / values.len() as f64 >= DATE_THRESHOLD {
        return ColumnType::Date(DateFormat::Iso);
    }

    // Try dot-separated DD.MM.YYYY (always day-first)
    let dot_count = values.iter().filter(|v| is_dot_date(v)).count();
    if dot_count as f64 / values.len() as f64 >= DATE_THRESHOLD {
        return ColumnType::Date(DateFormat::Dot);
    }

    // Try slash-separated (DD/MM/YYYY or MM/DD/YYYY)
    let slash_count = values.iter().filter(|v| is_slash_date(v)).count();
    if slash_count as f64 / values.len() as f64 >= DATE_THRESHOLD {
        let day_first = disambiguate_slash(values, locale_month_first);
        return if day_first {
            ColumnType::Date(DateFormat::SlashDayFirst)
        } else {
            ColumnType::Date(DateFormat::SlashMonthFirst)
        };
    }

    ColumnType::Numeric
}

// --- Pattern matchers ---

/// Match YYYY-MM-DD with optional time
fn is_iso_date(s: &str) -> bool {
    if s.len() < 10 {
        return false;
    }
    let b = s.as_bytes();
    if b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let year = parse_u32(&s[0..4]);
    let month = parse_u32(&s[5..7]);
    let day = parse_u32(&s[8..10]);

    let (year, month, day) = match (year, month, day) {
        (Some(y), Some(m), Some(d)) => (y, m, d),
        _ => return false,
    };

    if year < 1000 || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return false;
    }

    if s.len() == 10 {
        return true;
    }

    let sep = b[10];
    if sep != b'T' && sep != b' ' {
        return false;
    }

    is_valid_time(&s[11..])
}

/// Match D{1,2}/D{1,2}/D{4} with optional time
fn is_slash_date(s: &str) -> bool {
    let (date_str, time_str) = split_date_time(s);
    let parts: Vec<&str> = date_str.split('/').collect();
    if parts.len() != 3 {
        return false;
    }

    let a = parse_u32(parts[0]);
    let b = parse_u32(parts[1]);
    let year = parse_u32(parts[2]);

    match (a, b, year) {
        (Some(a), Some(b), Some(y)) => {
            if !(1000..=9999).contains(&y) {
                return false;
            }
            // At least one interpretation (DD/MM or MM/DD) must be valid
            let valid_dmy = (1..=31).contains(&a) && (1..=12).contains(&b);
            let valid_mdy = (1..=12).contains(&a) && (1..=31).contains(&b);
            if !valid_dmy && !valid_mdy {
                return false;
            }
        }
        _ => return false,
    }

    if let Some(t) = time_str {
        return is_valid_time(t);
    }
    true
}

/// Match D{1,2}.D{1,2}.D{4} with optional time
fn is_dot_date(s: &str) -> bool {
    let (date_str, time_str) = split_date_time(s);
    let parts: Vec<&str> = date_str.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    match (
        parse_u32(parts[0]),
        parse_u32(parts[1]),
        parse_u32(parts[2]),
    ) {
        (Some(day), Some(month), Some(year)) => {
            if !(1000..=9999).contains(&year)
                || !(1..=12).contains(&month)
                || !(1..=31).contains(&day)
            {
                return false;
            }
        }
        _ => return false,
    }

    if let Some(t) = time_str {
        return is_valid_time(t);
    }
    true
}

// --- Time validation ---

/// Validate a time string (24h or 12h with AM/PM)
fn is_valid_time(s: &str) -> bool {
    is_valid_time_24h(s) || is_valid_time_12h(s)
}

fn is_valid_time_24h(s: &str) -> bool {
    let s = s.trim_end_matches('Z');
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return false;
    }

    let hour = match parse_u32(parts[0]) {
        Some(h) if h <= 23 => h,
        _ => return false,
    };
    let _ = hour;

    if parse_u32(parts[1]).is_none_or(|m| m > 59) {
        return false;
    }

    if parts.len() == 3 {
        // Handle fractional seconds: "30.123"
        let sec_str = parts[2].split('.').next().unwrap_or("");
        if parse_u32(sec_str).is_none_or(|s| s > 59) {
            return false;
        }
    }

    true
}

fn is_valid_time_12h(s: &str) -> bool {
    let s = s.trim();
    let (time_part, _has_ampm) = if let Some(stripped) = s
        .strip_suffix("AM")
        .or_else(|| s.strip_suffix("am"))
        .or_else(|| s.strip_suffix("Am"))
        .or_else(|| s.strip_suffix("PM"))
        .or_else(|| s.strip_suffix("pm"))
        .or_else(|| s.strip_suffix("Pm"))
    {
        (stripped.trim(), true)
    } else {
        return false;
    };

    let parts: Vec<&str> = time_part.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return false;
    }

    if parse_u32(parts[0]).is_none_or(|h| !(1..=12).contains(&h)) {
        return false;
    }
    if parse_u32(parts[1]).is_none_or(|m| m > 59) {
        return false;
    }
    if parts.len() == 3 && parse_u32(parts[2]).is_none_or(|s| s > 59) {
        return false;
    }

    true
}

// --- Disambiguation ---

/// Disambiguate slash dates using data, falling back to locale.
/// Returns true if day comes first.
fn disambiguate_slash(values: &[&str], locale_month_first: bool) -> bool {
    let mut must_be_day_first = false;
    let mut must_be_month_first = false;

    for v in values {
        let date_str = split_date_time(v).0;
        let parts: Vec<&str> = date_str.split('/').collect();
        if parts.len() != 3 {
            continue;
        }

        let a = match parse_u32(parts[0]) {
            Some(n) => n,
            None => continue,
        };
        let b = match parse_u32(parts[1]) {
            Some(n) => n,
            None => continue,
        };

        // If first number > 12, it can't be a month → must be day-first
        if a > 12 {
            must_be_day_first = true;
        }
        // If second number > 12, it can't be a month → must be month-first
        if b > 12 {
            must_be_month_first = true;
        }
    }

    if must_be_day_first && !must_be_month_first {
        true
    } else if must_be_month_first && !must_be_day_first {
        false
    } else {
        // Ambiguous or contradictory — use locale
        !locale_month_first
    }
}

// --- Normalization ---

fn normalize_slash(value: &str, day_first: bool) -> String {
    let (date_str, time_str) = split_date_time(value);
    let parts: Vec<&str> = date_str.split('/').collect();
    if parts.len() != 3 {
        return value.to_string();
    }

    let (day_str, month_str) = if day_first {
        (parts[0], parts[1])
    } else {
        (parts[1], parts[0])
    };
    let year_str = parts[2];

    let day: u32 = match day_str.parse() {
        Ok(d) => d,
        Err(_) => return value.to_string(),
    };
    let month: u32 = match month_str.parse() {
        Ok(m) => m,
        Err(_) => return value.to_string(),
    };

    let mut result = format!("{}-{:02}-{:02}", year_str, month, day);

    if let Some(time) = time_str {
        if let Some(normalized) = normalize_time(time) {
            result.push(' ');
            result.push_str(&normalized);
        }
    }

    result
}

fn normalize_dot(value: &str) -> String {
    let (date_str, time_str) = split_date_time(value);
    let parts: Vec<&str> = date_str.split('.').collect();
    if parts.len() != 3 {
        return value.to_string();
    }

    let day: u32 = match parts[0].parse() {
        Ok(d) => d,
        Err(_) => return value.to_string(),
    };
    let month: u32 = match parts[1].parse() {
        Ok(m) => m,
        Err(_) => return value.to_string(),
    };
    let year_str = parts[2];

    let mut result = format!("{}-{:02}-{:02}", year_str, month, day);

    if let Some(time) = time_str {
        if let Some(normalized) = normalize_time(time) {
            result.push(' ');
            result.push_str(&normalized);
        }
    }

    result
}

/// Normalize a time string to 24h HH:MM:SS format.
fn normalize_time(s: &str) -> Option<String> {
    let s = s.trim();

    // Check for AM/PM
    let (time_part, ampm) = if let Some(stripped) = s
        .strip_suffix("AM")
        .or_else(|| s.strip_suffix("am"))
        .or_else(|| s.strip_suffix("Am"))
    {
        (stripped.trim(), Some(false))
    } else if let Some(stripped) = s
        .strip_suffix("PM")
        .or_else(|| s.strip_suffix("pm"))
        .or_else(|| s.strip_suffix("Pm"))
    {
        (stripped.trim(), Some(true))
    } else {
        (s.trim_end_matches('Z'), None)
    };

    let parts: Vec<&str> = time_part.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let mut hour: u32 = parts[0].parse().ok()?;
    let min: u32 = parts[1].parse().ok()?;
    let sec: u32 = if parts.len() >= 3 {
        parts[2].split('.').next()?.parse().ok()?
    } else {
        0
    };

    if let Some(is_pm) = ampm {
        if is_pm && hour != 12 {
            hour += 12;
        }
        if !is_pm && hour == 12 {
            hour = 0;
        }
    }

    Some(format!("{:02}:{:02}:{:02}", hour, min, sec))
}

// --- Utility ---

/// Split "date time" or "dateT..." into (date_part, Option<time_part>).
fn split_date_time(s: &str) -> (&str, Option<&str>) {
    // Don't split on 'T' for non-ISO formats — only split on space
    if let Some(pos) = s.find(' ') {
        (&s[..pos], Some(s[pos + 1..].trim()))
    } else {
        (s, None)
    }
}

/// Parse a string as u32, returning None on failure.
fn parse_u32(s: &str) -> Option<u32> {
    s.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Pattern matching tests ---

    #[test]
    fn test_iso_date() {
        assert!(is_iso_date("2024-04-20"));
        assert!(is_iso_date("2024-04-20 14:30:00"));
        assert!(is_iso_date("2024-04-20T14:30:00"));
        assert!(is_iso_date("2024-04-20T14:30:00Z"));
        assert!(is_iso_date("2024-04-20 14:30:00.123"));
        assert!(!is_iso_date("20-04-2024"));
        assert!(!is_iso_date("hello"));
        assert!(!is_iso_date("2024-13-01")); // invalid month
        assert!(!is_iso_date("2024-04-32")); // invalid day
    }

    #[test]
    fn test_slash_date() {
        assert!(is_slash_date("10/03/2026"));
        assert!(is_slash_date("3/10/2026"));
        assert!(is_slash_date("03/10/2026 14:30:00"));
        assert!(is_slash_date("03/10/2026 2:30 PM"));
        assert!(!is_slash_date("2024-04-20"));
        assert!(!is_slash_date("hello"));
        assert!(!is_slash_date("13/13/2024")); // both > 12 invalid for either interpretation
    }

    #[test]
    fn test_dot_date() {
        assert!(is_dot_date("10.03.2026"));
        assert!(is_dot_date("1.1.2024"));
        assert!(is_dot_date("10.03.2026 14:30:00"));
        assert!(!is_dot_date("3.14")); // only 2 parts
        assert!(!is_dot_date("192.168.1.1")); // 4 parts
        assert!(!is_dot_date("hello"));
    }

    #[test]
    fn test_time_validation() {
        assert!(is_valid_time_24h("14:30:00"));
        assert!(is_valid_time_24h("00:00:00"));
        assert!(is_valid_time_24h("23:59:59"));
        assert!(is_valid_time_24h("14:30"));
        assert!(!is_valid_time_24h("25:00:00"));
        assert!(!is_valid_time_24h("14:60:00"));

        assert!(is_valid_time_12h("2:30 PM"));
        assert!(is_valid_time_12h("12:00 AM"));
        assert!(is_valid_time_12h("12:59:59PM"));
        assert!(!is_valid_time_12h("13:00 PM"));
        assert!(!is_valid_time_12h("0:00 AM"));
        assert!(!is_valid_time_12h("14:30")); // no AM/PM
    }

    // --- Disambiguation tests ---

    #[test]
    fn test_disambiguate_day_first_from_data() {
        // Day 25 > 12, must be day-first
        let values = vec!["25/03/2024", "10/06/2024", "01/01/2024"];
        assert!(disambiguate_slash(&values, false)); // even with US locale, data wins
    }

    #[test]
    fn test_disambiguate_month_first_from_data() {
        // Second number 25 > 12, must be month-first
        let values = vec!["03/25/2024", "06/10/2024", "01/01/2024"];
        assert!(!disambiguate_slash(&values, true)); // even with non-US locale, data wins
    }

    #[test]
    fn test_disambiguate_falls_back_to_locale() {
        // All values ambiguous (both <= 12)
        let values = vec!["01/02/2024", "03/04/2024", "05/06/2024"];
        assert!(disambiguate_slash(&values, false)); // non-US locale → day-first
        assert!(!disambiguate_slash(&values, true)); // US locale → month-first
    }

    // --- Normalization tests ---

    #[test]
    fn test_normalize_iso_passthrough() {
        assert_eq!(
            normalize_to_iso("2024-04-20", DateFormat::Iso),
            "2024-04-20"
        );
        assert_eq!(
            normalize_to_iso("2024-04-20 14:30:00", DateFormat::Iso),
            "2024-04-20 14:30:00"
        );
    }

    #[test]
    fn test_normalize_slash_day_first() {
        assert_eq!(
            normalize_to_iso("25/03/2024", DateFormat::SlashDayFirst),
            "2024-03-25"
        );
        assert_eq!(
            normalize_to_iso("1/6/2024", DateFormat::SlashDayFirst),
            "2024-06-01"
        );
        assert_eq!(
            normalize_to_iso("25/03/2024 14:30:00", DateFormat::SlashDayFirst),
            "2024-03-25 14:30:00"
        );
    }

    #[test]
    fn test_normalize_slash_month_first() {
        assert_eq!(
            normalize_to_iso("03/25/2024", DateFormat::SlashMonthFirst),
            "2024-03-25"
        );
        assert_eq!(
            normalize_to_iso("03/25/2024 2:30 PM", DateFormat::SlashMonthFirst),
            "2024-03-25 14:30:00"
        );
    }

    #[test]
    fn test_normalize_dot() {
        assert_eq!(
            normalize_to_iso("25.03.2024", DateFormat::Dot),
            "2024-03-25"
        );
        assert_eq!(
            normalize_to_iso("1.6.2024 08:00:00", DateFormat::Dot),
            "2024-06-01 08:00:00"
        );
    }

    #[test]
    fn test_normalize_12h_to_24h() {
        assert_eq!(
            normalize_to_iso("03/25/2024 12:00 AM", DateFormat::SlashMonthFirst),
            "2024-03-25 00:00:00"
        );
        assert_eq!(
            normalize_to_iso("03/25/2024 12:30 PM", DateFormat::SlashMonthFirst),
            "2024-03-25 12:30:00"
        );
        assert_eq!(
            normalize_to_iso("03/25/2024 1:45 PM", DateFormat::SlashMonthFirst),
            "2024-03-25 13:45:00"
        );
    }

    #[test]
    fn test_normalize_graceful_degradation() {
        // Unparseable values return as-is
        assert_eq!(
            normalize_to_iso("not-a-date", DateFormat::SlashDayFirst),
            "not-a-date"
        );
        assert_eq!(normalize_to_iso("", DateFormat::Dot), "");
    }

    // --- Column detection tests ---

    #[test]
    fn test_detect_iso_column() {
        let rows: Vec<Vec<String>> = (0..10)
            .map(|i| vec![format!("2024-{:02}-{:02}", (i % 12) + 1, (i % 28) + 1)])
            .collect();
        let types = detect_column_types(&rows, 1);
        assert_eq!(types, vec![ColumnType::Date(DateFormat::Iso)]);
    }

    #[test]
    fn test_detect_numeric_column() {
        let rows: Vec<Vec<String>> = (0..10).map(|i| vec![format!("{}", i * 100)]).collect();
        let types = detect_column_types(&rows, 1);
        assert_eq!(types, vec![ColumnType::Numeric]);
    }

    #[test]
    fn test_detect_mixed_columns() {
        let rows: Vec<Vec<String>> = (0..10)
            .map(|i| {
                vec![
                    format!("Name{}", i),
                    format!("{}", 50 + i),
                    format!("2024-01-{:02}", i + 1),
                ]
            })
            .collect();
        let types = detect_column_types(&rows, 3);
        assert_eq!(types[0], ColumnType::Numeric); // names — not dates, stays numeric
        assert_eq!(types[1], ColumnType::Numeric); // numbers
        assert_eq!(types[2], ColumnType::Date(DateFormat::Iso)); // dates
    }

    #[test]
    fn test_detect_too_few_samples() {
        let rows: Vec<Vec<String>> = vec![vec!["2024-01-01".into()]];
        let types = detect_column_types(&rows, 1);
        // Only 1 sample, below threshold of 3 → Numeric
        assert_eq!(types, vec![ColumnType::Numeric]);
    }

    #[test]
    fn test_sqlite_affinity() {
        assert_eq!(sqlite_affinity(&ColumnType::Numeric), "NUMERIC");
        assert_eq!(sqlite_affinity(&ColumnType::Date(DateFormat::Iso)), "TEXT");
    }
}
