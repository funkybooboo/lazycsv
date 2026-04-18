use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "LazyCSV: A blazing-fast TUI viewer for CSV, TSV, JSON, Parquet, Excel, and SQLite", long_about = None, disable_help_flag = true)]
pub struct CliArgs {
    /// Path to a data file or directory, optionally followed by a sheet name or
    /// 1-based sheet index for spreadsheet files.
    /// Supported formats: CSV, TSV, CSV.GZ, TSV.GZ, JSON, NDJSON/JSONL, Parquet, Excel (XLSX/XLS), ODS, SQLite (db/sqlite/sqlite3)
    /// Examples: lazycsv data.csv | lazycsv data.parquet | lazycsv file.xlsx 2 | lazycsv db.sqlite
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

    /// Print header row values for each CSV file (non-interactive mode).
    #[arg(short = 'h', long, help = "Print header row values for each CSV file")]
    pub headers: bool,

    /// Print column statistics for each CSV file (non-interactive mode).
    /// With -g: specify data type (customer, sales, marketing, weather, scientific, random).
    #[arg(
        short = 't',
        long = "stats",
        num_args = 0..=1,
        default_missing_value = "",
        require_equals = false,
        help = "Print column statistics; with -g: data type (customer,sales,marketing,weather,scientific,random)"
    )]
    pub stats: Option<String>,

    /// Print row count for each CSV file (non-interactive mode).
    /// With -g: specify number of rows to generate (required).
    #[arg(
        short = 'r',
        long = "rows",
        num_args = 0..=1,
        default_missing_value = "",
        require_equals = false,
        help = "Print row count; with -g: number of rows to generate"
    )]
    pub rows: Option<String>,

    /// Print column count for each CSV file (non-interactive mode).
    /// With -g: specify number of columns to generate (required).
    #[arg(
        short = 'c',
        long = "columns",
        num_args = 0..=1,
        default_missing_value = "",
        require_equals = false,
        help = "Print column count; with -g: number of columns to generate"
    )]
    pub columns: Option<String>,

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

    /// Extract Excel/ODS sheets to CSV files (non-interactive mode).
    /// Uses the file path and optional sheet from positional args.
    /// Examples: lazycsv file.xlsx -x | lazycsv file.xlsx 2 -x | lazycsv file.ods -x
    #[arg(short = 'x', long = "xlsx", help = "Extract Excel/ODS sheets to CSV")]
    pub xlsx: bool,

    /// Output target for non-interactive modes and standalone format conversion.
    /// Standalone: `lazycsv in.csv -o out.json` converts by output extension
    /// (csv, tsv, json, md/markdown, xlsx, parquet).
    /// Also used with -q, -s, -t, -r, -c, -h, -A, -D, -g, -S, -x to redirect output.
    /// Forms: `-o` (stdout), `-o dir/` (directory, for -x/-S), `-o file.ext` (file).
    #[arg(
        short = 'o',
        long = "output",
        num_args = 0..=1,
        default_missing_value = "-",
        help = "Output file/dir; with a CSV input and typed extension (json/tsv/md/xlsx/parquet), converts format"
    )]
    pub output: Option<String>,

    /// Copy CSV output to system clipboard (non-interactive mode).
    /// Works with CSV, XLSX, XLS, ODS files and -q query results.
    #[arg(
        short = 'C',
        long = "cb-copy",
        help = "Copy CSV output to system clipboard"
    )]
    pub clipboard: bool,

    /// Paste CSV data from system clipboard and open in the TUI.
    #[arg(
        short = 'P',
        long = "cb-paste",
        help = "Open CSV data from system clipboard in the TUI"
    )]
    pub paste: bool,

    /// Split a CSV/XLSX/ODS file into multiple CSV files with the specified number of rows each.
    /// Files are numbered sequentially (1.csv, 2.csv, ...).
    /// Output to -o directory if provided, otherwise to the input file's directory.
    #[arg(
        short = 'S',
        long = "split",
        help = "Split file into CSVs of N rows each (e.g., -S 1000)"
    )]
    pub split: Option<usize>,

    /// Add a header row to a CSV file (non-interactive mode).
    /// If a CSV header string is provided, those values are used.
    /// If no value is provided, generates C1, C2, ... headers based on column count.
    #[arg(
        short = 'A',
        long = "add-header",
        num_args = 0..=1,
        default_missing_value = "",
        require_equals = true,
        help = "Add header row to CSV (e.g., -A or -A=\"Name,Age,City\")"
    )]
    pub add_header: Option<String>,

    /// Remove duplicate rows from a CSV file (non-interactive mode).
    /// Optionally specify PK columns by name or 1-based index (comma-separated).
    /// If no columns specified, all columns are used for deduplication.
    /// Examples: -D  |  -D Name  |  -D 1,3  |  -D "Name,Age"
    #[arg(
        short = 'D',
        long = "dedup",
        num_args = 0..=1,
        default_missing_value = "",
        require_equals = true,
        help = "Deduplicate rows, optionally by PK columns (e.g., -D=Name,Age)"
    )]
    pub dedup: Option<String>,

    /// When deduplicating, keep the first occurrence instead of the last.
    #[arg(
        long = "keep-first",
        hide = true,
        help = "Keep first duplicate row instead of last"
    )]
    pub keep_first: bool,

    /// Allow rows where all PK columns are NULL during dedup (ambiguous by default).
    #[arg(
        long = "allow-nulls",
        hide = true,
        help = "Allow all-NULL PK rows during dedup"
    )]
    pub allow_nulls: bool,

    /// Ignore case when comparing VARCHAR values during dedup.
    #[arg(
        long = "ignore-case",
        hide = true,
        help = "Case-insensitive dedup comparison"
    )]
    pub ignore_case: bool,

    /// Report duplicate rows instead of removing them.
    #[arg(
        long = "report-only",
        hide = true,
        help = "Report duplicates instead of removing them"
    )]
    pub report_only: bool,

    /// Generate a CSV file with synthetic data (non-interactive mode).
    /// Requires -r <rows> and -c <columns>. Optionally -t <type> (default: random).
    /// Types: customer, sales, marketing, weather, scientific, random.
    /// Examples: lazycsv -g -r 1000 -c 10 | lazycsv -g -r 500 -c 5 -t sales -o data.csv
    #[arg(
        short = 'g',
        long = "generate",
        help = "Generate a CSV file with synthetic data"
    )]
    pub generate: bool,

    /// Print help (use with -D for dedup-specific options)
    #[arg(long = "help", action = clap::ArgAction::SetTrue, help = "Print help")]
    pub help: bool,
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

    /// Whether -r was used as a boolean flag (no value or empty value, without -g).
    pub fn is_rows_flag(&self) -> bool {
        !self.generate && self.rows.as_deref() == Some("")
    }

    /// Whether -c was used as a boolean flag (no value or empty value, without -g).
    pub fn is_columns_flag(&self) -> bool {
        !self.generate && self.columns.as_deref() == Some("")
    }

    /// Whether -t was used as a boolean flag (no value or empty value, without -g).
    pub fn is_stats_flag(&self) -> bool {
        !self.generate && self.stats.as_deref() == Some("")
    }

    /// Get the generate row count from -r <N> (when -g is present).
    pub fn gen_rows(&self) -> Option<usize> {
        if !self.generate {
            return None;
        }
        self.rows.as_deref().and_then(|s| s.parse().ok())
    }

    /// Get the generate column count from -c <N> (when -g is present).
    pub fn gen_cols(&self) -> Option<usize> {
        if !self.generate {
            return None;
        }
        self.columns.as_deref().and_then(|s| s.parse().ok())
    }

    /// Get the generate data type from -t <type> (when -g is present). Defaults to "random".
    pub fn gen_type(&self) -> &str {
        if !self.generate {
            return "random";
        }
        match self.stats.as_deref() {
            Some(s) if !s.is_empty() => s,
            _ => "random",
        }
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
    match CliArgs::try_parse() {
        Ok(args) => args,
        Err(e) => {
            // If --help is in the args and clap failed (e.g. missing value for -q),
            // show command-specific help instead of the error.
            let raw_args: Vec<String> = std::env::args().collect();
            if raw_args.iter().any(|a| a == "--help") {
                if let Some(cmd) = detect_command_for_help(&raw_args) {
                    print_command_help(cmd);
                    std::process::exit(0);
                }
            }
            e.exit();
        }
    }
}

/// Detect which command flag is present in args for --help routing.
fn detect_command_for_help(args: &[String]) -> Option<&'static str> {
    for arg in args {
        match arg.as_str() {
            "-q" | "--query" => return Some("q"),
            "-s" | "--sort" => return Some("s"),
            "-S" | "--split" => return Some("S"),
            "-x" | "--xlsx" => return Some("x"),
            "-t" | "--stats" => return Some("t"),
            "-h" | "--headers" => return Some("h"),
            "-g" | "--generate" => return Some("g"),
            "-r" | "--rows" | "-c" | "--columns" => return Some("rc"),
            _ if arg.starts_with("-A") || arg.starts_with("--add-header") => return Some("A"),
            _ if arg.starts_with("-D") || arg.starts_with("--dedup") => return Some("D"),
            _ => {}
        }
    }
    None
}

/// Print command-specific help. Called both from parse_args (on clap error)
/// and from main (when --help parses successfully alongside a command flag).
pub fn print_command_help(cmd: &str) {
    match cmd {
        "q" => print!("{}", QUERY_HELP),
        "x" => print!("{}", XLSX_HELP),
        "S" => print!("{}", SPLIT_HELP),
        "t" => print!("{}", STATS_HELP),
        "s" => print!("{}", SORT_HELP),
        "h" => print!("{}", HEADERS_HELP),
        "rc" => print!("{}", COUNT_HELP),
        "A" => print!("{}", ADD_HEADER_HELP),
        "D" => print!("{}", DEDUP_HELP),
        "g" => print!("{}", GENERATE_HELP),
        _ => {}
    }
}

pub const QUERY_HELP: &str = "\
Execute SQL queries against data files (non-interactive mode)

Usage: lazycsv <FILE> -q <QUERY> [OPTIONS]

Supported formats: CSV, TSV, CSV.GZ, TSV.GZ, JSON, NDJSON/JSONL, Parquet, SQLite

The file is loaded as a database table named after the filename (without extension).
All supported files in the same directory are available for JOINs.

Examples:
  lazycsv data.csv -q \"SELECT * FROM data WHERE age > 30\"
  lazycsv data.csv.gz -q \"SELECT count(*) FROM data GROUP BY name\"
  lazycsv data.parquet -q \"SELECT name, sum(amount) FROM data GROUP BY name\"
  lazycsv sqlite.db -q \"SELECT * FROM users LIMIT 10\"
  lazycsv data.json -q \"SELECT id, status FROM data WHERE status = 'active'\"
  lazycsv . -q \"SELECT a.id, b.name FROM users a JOIN emails b ON a.id = b.user_id\"
  cat data.csv | lazycsv -q \"SELECT * FROM stdin ORDER BY name\"
  lazycsv data.csv -q \"SELECT * FROM data\" -o results.csv

Options:
  -o, --output <FILE>  Write results to a file instead of stdout
  -C, --cb-copy        Copy results to system clipboard
  -d, --delimiter      Custom delimiter for input file
  -H, --no-headers     Treat first row as data
  -e, --encoding       File encoding (e.g., 'utf-8', 'latin1')
";

pub const XLSX_HELP: &str = "\
Extract Excel/ODS sheets to CSV files (non-interactive mode)

Usage: lazycsv <FILE> -x [OPTIONS]

Converts spreadsheet sheets to CSV format.

Examples:
  lazycsv file.xlsx -x                   Extract all sheets to file/ directory
  lazycsv file.xlsx 2 -x                 Extract sheet 2 only
  lazycsv file.xlsx \"Sheet Name\" -x      Extract named sheet
  lazycsv file.xlsx -x -o out/           Extract to specific directory
  lazycsv file.xlsx -x -o -              Extract to stdout (single sheet)
  lazycsv file.xlsx -x -o result.csv     Extract to specific file (single sheet)

Options:
  -o, --output [<PATH>]  Output target:
                           -o (stdout), -o dir/ (directory), -o file.csv (file)
                           omitted: output to <filename>/ directory
";

pub const SPLIT_HELP: &str = "\
Split a CSV/XLSX/ODS file into multiple CSV files (non-interactive mode)

Usage: lazycsv <FILE> -S <ROWS> [OPTIONS]

Files are numbered sequentially (1.csv, 2.csv, ...).

Examples:
  lazycsv data.csv -S 1000              Split into files of 1000 rows each
  lazycsv data.csv -S 500 -o out/       Split into out/ directory
  lazycsv file.xlsx -S 1000             Split spreadsheet

Options:
  -o, --output <DIR>  Output directory (default: input file's directory)
";

pub const STATS_HELP: &str = "\
Print per-column statistics for CSV files (non-interactive mode)

Usage: lazycsv <FILE> -t [OPTIONS]

Output columns: col_name, data_type, min, max, min_len, max_len, mean, stddev, median, mode, cardinality

Examples:
  lazycsv data.csv -t                    Print stats to stdout (CSV format)
  lazycsv data.csv -t -o stats.csv       Write stats to file
  lazycsv data.csv -t | lazycsv -q \"SELECT col_name, data_type FROM stdin\"
  cat data.csv | lazycsv -t              Stats from piped input

Options:
  -o, --output <FILE>  Write stats to a file instead of stdout
  -d, --delimiter      Custom delimiter for input file
  -e, --encoding       File encoding
";

pub const SORT_HELP: &str = "\
Sort CSV data by columns and output to stdout (non-interactive mode)

Usage: lazycsv <FILE> -s <SPEC> [OPTIONS]

Columns can be specified by name, 1-based index, or Excel letter (A, B, ...).
Prefix with '!' for descending order. Multiple columns separated by commas.

Examples:
  lazycsv data.csv -s Name               Sort by Name ascending
  lazycsv data.csv -s 1,2                Sort by columns 1 and 2
  lazycsv data.csv -s '!Age'             Sort by Age descending
  lazycsv data.csv -s '!Price,Name'      Sort by Price desc, then Name asc
  lazycsv data.csv -s Name -o sorted.csv Write sorted output to file
  cat data.csv | lazycsv -s 1            Sort piped input

Options:
  -o, --output <FILE>  Write sorted CSV to a file instead of stdout
  -d, --delimiter      Custom delimiter for input file
  -H, --no-headers     Treat first row as data
  -e, --encoding       File encoding
";

pub const HEADERS_HELP: &str = "\
Print header row values for CSV files (non-interactive mode)

Usage: lazycsv <FILE> -h [OPTIONS]

Examples:
  lazycsv data.csv -h                    Print headers
  lazycsv data.csv -h -o headers.txt     Write headers to file
  lazycsv dir/ -h                        Print headers for all CSV files in directory
  cat data.csv | lazycsv -h              Headers from piped input

Options:
  -o, --output <FILE>  Write headers to a file instead of stdout
  -d, --delimiter      Custom delimiter for input file
  -e, --encoding       File encoding
";

pub const COUNT_HELP: &str = "\
Print row and/or column counts for CSV files (non-interactive mode)

Usage: lazycsv <FILE> -r|-c [OPTIONS]

Examples:
  lazycsv data.csv -r                    Print row count
  lazycsv data.csv -c                    Print column count
  lazycsv data.csv -r -c                 Print both
  lazycsv data.csv -r -f                 Row count with thousands separators
  lazycsv dir/ -r                        Row counts for all CSV files
  lazycsv data.csv -r -o counts.txt      Write counts to file
  cat data.csv | lazycsv -r              Count from piped input

Options:
  -o, --output <FILE>  Write counts to a file instead of stdout
  -f, --format         Format numbers with locale-aware thousands separators
  -d, --delimiter      Custom delimiter for input file
  -H, --no-headers     Treat first row as data
  -e, --encoding       File encoding
";

pub const ADD_HEADER_HELP: &str = "\
Add a header row to a CSV file (non-interactive mode)

Usage: lazycsv <FILE> -A[=\"Header1,Header2,...\"] [OPTIONS]

If header values are provided, they are used as the header row.
If omitted, generates C1, C2, C3, ... based on the number of columns.
The number of provided header values must match the number of columns in the file.

By default, the input file is modified in place. Use -o to write to a different file.

Examples:
  lazycsv data.csv -A                           Add auto-generated headers (C1, C2, ...)
  lazycsv data.csv -A=\"Name,Age,City\"            Add custom headers
  lazycsv data.csv -A -o output.csv             Add headers, write to new file
  lazycsv data.csv -A=\"Name,Age\" -o output.csv   Custom headers to new file
  cat data.csv | lazycsv -A                     Add headers to piped input (stdout)

Options:
  -o, --output <FILE>  Write to a file instead of modifying in place
  -d, --delimiter      Custom delimiter for input file
  -e, --encoding       File encoding
";

pub const DEDUP_HELP: &str = "\
Deduplicate rows in a CSV file

Usage: lazycsv <FILE> -D[=<COLUMNS>] [OPTIONS]

COLUMNS can be column names or 1-based indexes, comma-separated.
If omitted, all columns are used for deduplication.

Examples:
  lazycsv data.csv -D                         Dedup by all columns
  lazycsv data.csv -D=Name                    Dedup by Name column
  lazycsv data.csv -D=Name,Age                Dedup by composite key
  lazycsv data.csv -D=1,3                     Dedup by column indexes
  lazycsv data.csv -D=Name --keep-first       Keep first occurrence
  lazycsv data.csv -D=Name --report-only      Report duplicates only
  lazycsv data.csv -D=Name -o out.csv         Write to file

Options:
      --keep-first     Keep the first duplicate row instead of the last (default: last wins)
      --allow-nulls    Allow rows where all PK columns are NULL (errors by default)
      --ignore-case    Case-insensitive comparison for VARCHAR values
      --report-only    Report duplicate rows instead of removing them
                       Output includes: row_number, original columns, dup_count
  -o, --output <FILE>  Write output to a file instead of stdout
";

pub const GENERATE_HELP: &str = "\
Generate a CSV file with synthetic data (non-interactive mode)

Usage: lazycsv -g -r <ROWS> -c <COLUMNS> [-t <TYPE>] [OPTIONS]

ROWS and COLUMNS are required. TYPE defaults to 'random'.

Available types:
  customer    Customer data (ID, Name, Email, Phone, City, State, ...)
  sales       Sales records (OrderID, Date, Product, Quantity, Price, ...)
  marketing   Marketing metrics (CampaignID, Channel, Impressions, Clicks, ...)
  weather     Weather observations (Date, City, TempHigh, TempLow, Humidity, ...)
  scientific  Scientific measurements (ExperimentID, Sensor, Temp, Pressure, ...)
  random      Random mixed-type columns (C1, C2, C3, ...)

Examples:
  lazycsv -g -r 1000 -c 10                Generate 1000 rows, 10 columns (random)
  lazycsv -g -r 500 -c 8 -t sales         Generate 500 rows of sales data
  lazycsv -g -r 10000 -c 5 -t weather     Generate weather data
  lazycsv -g -r 100 -c 6 -o data.csv      Generate and write to file
  lazycsv -g -r 1000 -c 10 -t customer | lazycsv -q \"SELECT * FROM stdin WHERE State = 'CA'\"

Options:
  -o, --output <FILE>  Write to a file instead of stdout
";

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

    // ── Generate flag tests ──────────────────────────────────

    #[test]
    fn test_cli_generate_with_rows_cols() {
        let args = CliArgs::try_parse_from(["lazycsv", "-g", "-r", "1000", "-c", "10"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.generate);
        assert_eq!(args.gen_rows(), Some(1000));
        assert_eq!(args.gen_cols(), Some(10));
        assert_eq!(args.gen_type(), "random"); // default
    }

    #[test]
    fn test_cli_generate_with_type() {
        let args =
            CliArgs::try_parse_from(["lazycsv", "-g", "-r", "500", "-c", "8", "-t", "sales"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.generate);
        assert_eq!(args.gen_rows(), Some(500));
        assert_eq!(args.gen_cols(), Some(8));
        assert_eq!(args.gen_type(), "sales");
    }

    #[test]
    fn test_cli_generate_without_rows_returns_none() {
        let args = CliArgs::try_parse_from(["lazycsv", "-g", "-c", "10"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.generate);
        assert_eq!(args.gen_rows(), None); // -r not provided
    }

    #[test]
    fn test_cli_generate_without_cols_returns_none() {
        let args = CliArgs::try_parse_from(["lazycsv", "-g", "-r", "100"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.generate);
        assert_eq!(args.gen_cols(), None); // -c not provided
    }

    #[test]
    fn test_cli_rows_flag_without_generate() {
        // -r alone (no -g) should act as boolean flag
        let args = CliArgs::try_parse_from(["lazycsv", "data.csv", "-r"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(!args.generate);
        assert!(args.is_rows_flag());
        assert_eq!(args.gen_rows(), None); // not in generate mode
    }

    #[test]
    fn test_cli_stats_flag_without_generate() {
        // -t alone (no -g) should act as stats flag
        let args = CliArgs::try_parse_from(["lazycsv", "data.csv", "-t"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(!args.generate);
        assert!(args.is_stats_flag());
    }

    #[test]
    fn test_cli_generate_type_via_stats_flag() {
        // -t with -g provides the generation type
        let args =
            CliArgs::try_parse_from(["lazycsv", "-g", "-r", "10", "-c", "5", "-t", "weather"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.generate);
        assert!(!args.is_stats_flag()); // -g overrides stats meaning
        assert_eq!(args.gen_type(), "weather");
    }
}
