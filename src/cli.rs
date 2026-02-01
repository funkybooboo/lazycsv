use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "LazyCSV: A blazing-fast CSV TUI viewer", long_about = None)]
pub struct CliArgs {
    /// Path to the CSV file or directory containing CSV files.
    /// If a directory is provided, the first CSV file found will be opened.
    /// If no path is provided, the current directory will be scanned.
    pub path: Option<PathBuf>,

    /// Specify a custom delimiter character for the CSV file.
    #[arg(short, long, value_parser = parse_delimiter, help = "Custom delimiter character (e.g., ',' or ';')")]
    pub delimiter: Option<u8>,

    /// Treat the first row as data rather than a header.
    #[arg(long, help = "Treat the first row as data, not headers.")]
    pub no_headers: bool,

    /// Specify the character encoding of the file.
    #[arg(
        short,
        long,
        help = "File encoding (e.g., 'utf-8', 'latin1', 'utf-16le')"
    )]
    pub encoding: Option<String>,
}

fn parse_delimiter(s: &str) -> Result<u8, String> {
    if s.len() == 1 {
        Ok(s.as_bytes()[0])
    } else {
        Err(format!("Delimiter must be a single character, got '{}'", s))
    }
}

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
        assert_eq!(args.path, None);
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
        assert_eq!(args.path, Some(file_path));
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
        assert_eq!(args.path, Some(file_path));
        assert_eq!(args.delimiter, Some(b','));
        assert!(args.no_headers);
        assert_eq!(args.encoding, Some("utf-8".to_string()));
    }

    #[test]
    fn test_cli_path_not_found_is_ok_for_parser() {
        let args = CliArgs::try_parse_from(["lazycsv", "/non/existent/path.csv"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.path, Some(PathBuf::from("/non/existent/path.csv")));
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
}
