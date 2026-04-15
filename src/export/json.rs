//! JSON export — array of objects.

use anyhow::Result;
use std::io::Write;

/// Write data as a JSON array of objects.
/// Each row becomes `{"header1": "value1", "header2": "value2", ...}`.
pub fn write_json<W: Write>(
    writer: &mut W,
    headers: &[String],
    rows: &[Vec<String>],
) -> Result<()> {
    let mut array = Vec::with_capacity(rows.len());

    for row in rows {
        let mut map = serde_json::Map::new();
        for (i, header) in headers.iter().enumerate() {
            let value = row.get(i).map(|s| s.as_str()).unwrap_or("");
            // Try to preserve types: numbers and booleans
            if let Ok(n) = value.parse::<f64>() {
                if let Some(num) = serde_json::Number::from_f64(n) {
                    map.insert(header.clone(), serde_json::Value::Number(num));
                } else {
                    map.insert(header.clone(), serde_json::Value::String(value.to_string()));
                }
            } else if value.eq_ignore_ascii_case("true") {
                map.insert(header.clone(), serde_json::Value::Bool(true));
            } else if value.eq_ignore_ascii_case("false") {
                map.insert(header.clone(), serde_json::Value::Bool(false));
            } else if value.is_empty() {
                map.insert(header.clone(), serde_json::Value::Null);
            } else {
                map.insert(header.clone(), serde_json::Value::String(value.to_string()));
            }
        }
        array.push(serde_json::Value::Object(map));
    }

    serde_json::to_writer_pretty(&mut *writer, &array)?;
    writer.write_all(b"\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_json_basic() {
        let headers = vec!["Name".into(), "Age".into(), "Active".into()];
        let rows = vec![
            vec!["Alice".into(), "30".into(), "true".into()],
            vec!["Bob".into(), "25".into(), "false".into()],
        ];
        let mut buf = Vec::new();
        write_json(&mut buf, &headers, &rows).unwrap();
        let output: Vec<serde_json::Value> = serde_json::from_slice(&buf).unwrap();
        assert_eq!(output.len(), 2);
        assert_eq!(output[0]["Name"], "Alice");
        assert_eq!(output[0]["Age"], 30.0);
        assert_eq!(output[0]["Active"], true);
        assert_eq!(output[1]["Name"], "Bob");
    }

    #[test]
    fn test_write_json_empty_cells() {
        let headers = vec!["A".into(), "B".into()];
        let rows = vec![vec!["val".into(), "".into()]];
        let mut buf = Vec::new();
        write_json(&mut buf, &headers, &rows).unwrap();
        let output: Vec<serde_json::Value> = serde_json::from_slice(&buf).unwrap();
        assert!(output[0]["B"].is_null());
    }

    #[test]
    fn test_write_json_empty_rows() {
        let headers = vec!["A".into()];
        let rows: Vec<Vec<String>> = vec![];
        let mut buf = Vec::new();
        write_json(&mut buf, &headers, &rows).unwrap();
        let output: Vec<serde_json::Value> = serde_json::from_slice(&buf).unwrap();
        assert_eq!(output.len(), 0);
    }
}
