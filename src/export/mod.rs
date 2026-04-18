//! Multi-format export — JSON, TSV, Markdown, XLSX, ODS.

pub mod json;
pub mod markdown;
pub mod ods;
pub mod tsv;
pub mod xlsx;

use anyhow::{bail, Result};
use std::path::Path;

/// Supported export formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Tsv,
    Markdown,
    Xlsx,
    Ods,
    Parquet,
    Csv,
}

impl ExportFormat {
    /// Detect format from file extension (case-insensitive).
    pub fn from_extension(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "json" => Some(Self::Json),
            "tsv" => Some(Self::Tsv),
            "md" | "markdown" => Some(Self::Markdown),
            "xlsx" => Some(Self::Xlsx),
            "ods" => Some(Self::Ods),
            "parquet" => Some(Self::Parquet),
            "csv" => Some(Self::Csv),
            _ => None,
        }
    }

    /// Parse format name from user input (`:export json`).
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "json" => Some(Self::Json),
            "tsv" => Some(Self::Tsv),
            "md" | "markdown" => Some(Self::Markdown),
            "xlsx" => Some(Self::Xlsx),
            "ods" => Some(Self::Ods),
            "parquet" => Some(Self::Parquet),
            "csv" => Some(Self::Csv),
            _ => None,
        }
    }

    /// File extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Tsv => "tsv",
            Self::Markdown => "md",
            Self::Xlsx => "xlsx",
            Self::Ods => "ods",
            Self::Parquet => "parquet",
            Self::Csv => "csv",
        }
    }
}

/// Export data to a file in the given format.
pub fn export_to_file(
    format: ExportFormat,
    path: &Path,
    headers: &[String],
    rows: &[Vec<String>],
) -> Result<()> {
    match format {
        ExportFormat::Json => {
            let file = std::fs::File::create(path)?;
            let mut writer = std::io::BufWriter::new(file);
            json::write_json(&mut writer, headers, rows)
        }
        ExportFormat::Tsv => {
            let file = std::fs::File::create(path)?;
            let mut writer = std::io::BufWriter::new(file);
            tsv::write_tsv(&mut writer, headers, rows)
        }
        ExportFormat::Markdown => {
            let file = std::fs::File::create(path)?;
            let mut writer = std::io::BufWriter::new(file);
            markdown::write_markdown(&mut writer, headers, rows)
        }
        ExportFormat::Xlsx => xlsx::write_xlsx(path, headers, rows),
        ExportFormat::Ods => ods::write_ods(path, headers, rows),
        ExportFormat::Parquet => export_via_duckdb(path, headers, rows, "parquet"),
        ExportFormat::Csv => {
            bail!("Use :w for CSV export")
        }
    }
}

/// Export via DuckDB: write data to a temp CSV, then use DuckDB COPY to convert.
/// Supports parquet (and potentially other DuckDB-native formats).
fn export_via_duckdb(
    path: &Path,
    headers: &[String],
    rows: &[Vec<String>],
    format: &str,
) -> Result<()> {
    use anyhow::Context;

    // Write to temp CSV first
    let temp_csv = std::env::temp_dir().join(format!("lazycsv_export_{}.csv", std::process::id()));
    {
        let file = std::fs::File::create(&temp_csv)?;
        let mut writer = std::io::BufWriter::new(file);
        tsv::write_tsv(&mut writer, headers, rows).ok(); // fallback: write as CSV instead
        drop(writer);
    }
    // Actually write proper CSV for DuckDB
    {
        let file = std::fs::File::create(&temp_csv)?;
        let mut wtr = csv::Writer::from_writer(file);
        wtr.write_record(headers)?;
        for row in rows {
            wtr.write_record(row)?;
        }
        wtr.flush()?;
    }

    let conn = duckdb::Connection::open_in_memory().context("Failed to open DuckDB")?;
    let csv_str = temp_csv.display().to_string().replace('\'', "''");
    let out_str = path.display().to_string().replace('\'', "''");
    let copy_sql = format!(
        "COPY (SELECT * FROM read_csv('{}')) TO '{}' (FORMAT '{}')",
        csv_str, out_str, format
    );
    conn.execute_batch(&copy_sql)
        .context(format!("Failed to export as {}", format))?;
    let _ = std::fs::remove_file(&temp_csv);
    Ok(())
}

/// Collect headers and data rows from a Document.
/// Row 0 is treated as headers; rows 1..N are data.
pub fn collect_document_data(doc: &crate::csv::Document) -> (Vec<String>, Vec<Vec<String>>) {
    let headers = doc.storage.header_row().to_vec();
    let rows: Vec<Vec<String>> = (1..doc.row_count())
        .map(|i| {
            (0..doc.column_count())
                .map(|j| doc.storage.get_cell(i, j))
                .collect()
        })
        .collect();
    (headers, rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_format_from_extension() {
        assert_eq!(
            ExportFormat::from_extension(&PathBuf::from("out.json")),
            Some(ExportFormat::Json)
        );
        assert_eq!(
            ExportFormat::from_extension(&PathBuf::from("out.TSV")),
            Some(ExportFormat::Tsv)
        );
        assert_eq!(
            ExportFormat::from_extension(&PathBuf::from("out.md")),
            Some(ExportFormat::Markdown)
        );
        assert_eq!(
            ExportFormat::from_extension(&PathBuf::from("out.xlsx")),
            Some(ExportFormat::Xlsx)
        );
        assert_eq!(
            ExportFormat::from_extension(&PathBuf::from("out.csv")),
            Some(ExportFormat::Csv)
        );
        assert_eq!(
            ExportFormat::from_extension(&PathBuf::from("out.txt")),
            None
        );
    }

    #[test]
    fn test_format_from_name() {
        assert_eq!(ExportFormat::from_name("json"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::from_name("JSON"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::from_name("md"), Some(ExportFormat::Markdown));
        assert_eq!(
            ExportFormat::from_name("markdown"),
            Some(ExportFormat::Markdown)
        );
        assert_eq!(ExportFormat::from_name("unknown"), None);
    }
}
