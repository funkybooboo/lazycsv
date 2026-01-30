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
    #[arg(short, long, help = "File encoding (e.g., 'utf-8', 'latin1', 'utf-16le')")]
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