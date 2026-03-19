use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "LazyCSV: A blazing-fast CSV TUI viewer", long_about = None)]
pub struct CliArgs {
    /// Path to a CSV/XLSX file or directory, optionally followed by a sheet name or
    /// 1-based sheet index for spreadsheet files.
    /// Examples: lazycsv data.csv | lazycsv file.xlsx | lazycsv file.xlsx 2 | lazycsv file.xlsx "Sheet Name"
    #[arg(num_args = 0..=2)]
    pub path: Vec<String>,

    /// Specify a custom delimiter character for the CSV file.
    #[arg(short, long, value_parser = parse_delimiter, help = "Custom delimiter character (e.g., ',' or ';')")]
    pub delimiter: Option<u8>,

    /// Treat the first row as data rather than a header.
    #[arg(short = 'H', long, help = "Treat the first row as data, not headers.")]
    pub no_headers: bool,

    /// Specify the character encoding of the file.
    #[arg(
        short,
        long,
        help = "File encoding (e.g., 'utf-8', 'latin1', 'utf-16le')"
    )]
    pub encoding: Option<String>,

    /// SQL query to execute against CSV file(s) (non-interactive mode).
    #[arg(
        short = 'q',
        long = "query",
        help = "SQL query to execute against CSV file(s) (non-interactive mode)"
    )]
    pub query: Option<String>,

    /// Print row count for each CSV file (non-interactive mode).
    #[arg(short = 'r', long, help = "Print row count for each CSV file")]
    pub rows: bool,

    /// Print column count for each CSV file (non-interactive mode).
    #[arg(short = 'c', long, help = "Print column count for each CSV file")]
    pub columns: bool,

    /// Format numbers with locale-aware thousands separators.
    #[arg(
        short = 'f',
        long,
        help = "Format numbers with thousands separators (',' or '.' based on locale)"
    )]
    pub format: bool,

    /// Sort data by specified columns on load.
    /// Columns can be specified by name, number (1-indexed), or Excel letter (A, B, ...).
    /// Multiple columns separated by commas. Prefix with '!' for descending order.
    /// Examples: -s Name  -s 1,2  -s '!Age'  -s '!Price,Name'
    #[arg(
        short = 's',
        long = "sort",
        help = "Sort by columns on load (e.g., -s Name, -s 1,2, -s '!Age' for descending)"
    )]
    pub sort: Option<String>,

    /// Extract Excel sheets to CSV files (non-interactive mode).
    /// Uses the file path and optional sheet from positional args.
    /// Examples: lazycsv file.xlsx -x | lazycsv file.xlsx 2 -x
    #[arg(short = 'x', long = "xlsx", help = "Extract Excel sheets to CSV")]
    pub xlsx: bool,

    /// Output directory for xlsx conversion (used with --xlsx).
    /// Defaults to a directory named after the Excel file (without extension).
    #[arg(
        short = 'o',
        long = "output",
        help = "Output directory (default: <excel_filename> directory)"
    )]
    pub output: Option<PathBuf>,
}

impl CliArgs {
    /// Get the file path from positional args.
    pub fn file_path(&self) -> Option<PathBuf> {
        self.path.first().map(PathBuf::from)
    }

    /// Get the sheet specifier from positional args (second positional arg).
    pub fn sheet_from_path(&self) -> Option<&str> {
        self.path.get(1).map(|s| s.as_str())
    }
}

fn parse_delimiter(s: &str) -> Result<u8, String> {
    if s.len() == 1 {
        Ok(s.as_bytes()[0])
    } else {
        Err(format!("Delimiter must be a single character, got '{}'", s))
    }
}

/// Parse command-line arguments using clap
///
/// # Returns
///
/// A `CliArgs` struct containing all parsed CLI arguments
///
/// # Panics
///
/// Exits the program if invalid arguments are provided (handled by clap)
pub fn parse_args() -> CliArgs {
    CliArgs::parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cli_default_args() {
        let args = CliArgs::try_parse_from(["lazycsv"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.file_path().is_none());
        assert_eq!(args.delimiter, None);
        assert!(!args.no_headers);
        assert_eq!(args.encoding, None);
    }

    #[test]
    fn test_cli_with_file_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.csv");
        std::fs::File::create(&file_path).unwrap();

        let args = CliArgs::try_parse_from(["lazycsv", file_path.to_str().unwrap()]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.file_path(), Some(file_path));
    }

    #[test]
    fn test_cli_with_delimiter() {
        let args = CliArgs::try_parse_from(["lazycsv", "--delimiter", ";"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.delimiter, Some(b';'));
    }

    #[test]
    fn test_cli_with_delimiter_short() {
        let args = CliArgs::try_parse_from(["lazycsv", "-d", ";"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.delimiter, Some(b';'));
    }

    #[test]
    fn test_cli_invalid_delimiter() {
        let args = CliArgs::try_parse_from(["lazycsv", "--delimiter", "abc"]);
        assert!(args.is_err());
    }

    #[test]
    fn test_cli_with_no_headers() {
        let args = CliArgs::try_parse_from(["lazycsv", "--no-headers"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.no_headers);
    }

    #[test]
    fn test_cli_all_args_combined() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("data.csv");
        std::fs::File::create(&file_path).unwrap();

        let args = CliArgs::try_parse_from([
            "lazycsv",
            file_path.to_str().unwrap(),
            "--delimiter",
            ",",
            "--no-headers",
            "--encoding",
            "utf-8",
        ]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.file_path(), Some(file_path));
        assert_eq!(args.delimiter, Some(b','));
        assert!(args.no_headers);
        assert_eq!(args.encoding, Some("utf-8".to_string()));
    }

    #[test]
    fn test_cli_path_not_found_is_ok_for_parser() {
        let args = CliArgs::try_parse_from(["lazycsv", "/non/existent/path.csv"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(
            args.file_path(),
            Some(PathBuf::from("/non/existent/path.csv"))
        );
    }

    #[test]
    fn test_cli_with_encoding() {
        let args = CliArgs::try_parse_from(["lazycsv", "--encoding", "utf-16le"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.encoding, Some("utf-16le".to_string()));
    }

    #[test]
    fn test_cli_with_encoding_short() {
        let args = CliArgs::try_parse_from(["lazycsv", "-e", "latin1"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.encoding, Some("latin1".to_string()));
    }

    #[test]
    fn test_cli_with_query_short() {
        let args = CliArgs::try_parse_from(["lazycsv", "data.csv", "-q", "SELECT * FROM data"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.query, Some("SELECT * FROM data".to_string()));
    }

    #[test]
    fn test_cli_with_query_long() {
        let args = CliArgs::try_parse_from(["lazycsv", "--query", "SELECT 1"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.query, Some("SELECT 1".to_string()));
    }

    #[test]
    fn test_cli_default_query_is_none() {
        let args = CliArgs::try_parse_from(["lazycsv"]);
        assert!(args.is_ok());
        assert_eq!(args.unwrap().query, None);
    }

    #[test]
    fn test_cli_with_sort_short() {
        let args = CliArgs::try_parse_from(["lazycsv", "data.csv", "-s", "Name"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.sort, Some("Name".to_string()));
    }

    #[test]
    fn test_cli_with_sort_long() {
        let args = CliArgs::try_parse_from(["lazycsv", "--sort", "1,2"]);
        assert!(args.is_ok());
        assert_eq!(args.unwrap().sort, Some("1,2".to_string()));
    }

    #[test]
    fn test_cli_with_sort_descending() {
        let args = CliArgs::try_parse_from(["lazycsv", "-s", "!Price,Name"]);
        assert!(args.is_ok());
        assert_eq!(args.unwrap().sort, Some("!Price,Name".to_string()));
    }

    #[test]
    fn test_cli_default_sort_is_none() {
        let args = CliArgs::try_parse_from(["lazycsv"]);
        assert!(args.is_ok());
        assert_eq!(args.unwrap().sort, None);
    }

    #[test]
    fn test_cli_query_with_path_and_delimiter() {
        let args =
            CliArgs::try_parse_from(["lazycsv", "data.csv", "-d", ";", "-q", "SELECT * FROM data"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.file_path(), Some(PathBuf::from("data.csv")));
        assert_eq!(args.delimiter, Some(b';'));
        assert_eq!(args.query, Some("SELECT * FROM data".to_string()));
    }
}
