//! App constructor and file resolution methods.

use super::{messages, App};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

impl App {
    /// Resolve file paths from CLI arguments (fast, no file loading).
    /// Returns (file_path, csv_files, current_file_index, file_config).
    pub fn resolve_files(
        cli_args: &crate::cli::CliArgs,
    ) -> Result<(PathBuf, Vec<PathBuf>, usize, crate::session::FileConfig)> {
        let explicit_path = cli_args.file_path();
        let path = explicit_path.clone().unwrap_or_else(|| PathBuf::from("."));

        // Determine the CSV file to load and scan directory for others
        let (file_path, csv_files, current_file_index) = if path.is_file() {
            let csv_files = crate::file_system::scan_directory_for_csvs(&path)?;
            let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());
            let current_file_index = csv_files
                .iter()
                .position(|p| p.canonicalize().unwrap_or_else(|_| p.clone()) == canonical_path)
                .unwrap_or(0);
            (path, csv_files, current_file_index)
        } else if path.is_dir() {
            let csv_files = crate::file_system::scan_directory(&path)?;
            if csv_files.is_empty() {
                if explicit_path.is_none() {
                    anyhow::bail!(
                        "No file provided and no CSV files found in directory: {}",
                        path.display()
                    );
                }
                anyhow::bail!("{}", messages::no_csv_files_found(&path));
            }
            let file_path = csv_files[0].clone();
            (file_path, csv_files, 0)
        } else {
            anyhow::bail!("{}", messages::invalid_path(&path));
        };

        // Create file configuration
        let file_config = crate::session::FileConfig::with_options(
            cli_args.delimiter,
            cli_args.no_headers,
            cli_args.encoding.clone(),
        );

        Ok((file_path, csv_files, current_file_index, file_config))
    }

    /// Load a CSV file and create an App instance.
    /// Call after `resolve_files` to actually load the document.
    pub fn load_file(
        file_path: &Path,
        csv_files: Vec<PathBuf>,
        current_file_index: usize,
        file_config: crate::session::FileConfig,
        cli_args: &crate::cli::CliArgs,
    ) -> Result<Self> {
        // Apply config defaults for the initial file load
        let config = crate::config::load_config();
        let delimiter = cli_args
            .delimiter
            .or(config.defaults.delimiter.map(|d| d as u8));
        let encoding = cli_args
            .encoding
            .clone()
            .or(config.defaults.encoding.clone());

        let csv_data =
            crate::csv::Document::from_file(file_path, delimiter, cli_args.no_headers, encoding)
                .context(messages::failed_to_load_csv(file_path))?;

        let mut app = Self::new(csv_data, csv_files, current_file_index, file_config);
        app.session.record_file_mtime(file_path);
        Ok(app)
    }

    /// Create a new `App` instance from CLI arguments.
    /// This function handles file scanning, initial data loading, and App creation.
    pub fn from_cli(cli_args: crate::cli::CliArgs) -> Result<Self> {
        let (file_path, csv_files, current_file_index, file_config) =
            Self::resolve_files(&cli_args)?;
        Self::load_file(
            &file_path,
            csv_files,
            current_file_index,
            file_config,
            &cli_args,
        )
    }
}
