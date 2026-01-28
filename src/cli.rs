use anyhow::Result;
use std::path::PathBuf;

/// Parse command-line arguments and return a path (file or directory)
pub fn parse_args(args: &[String]) -> Result<PathBuf> {
    // If no arguments provided, use current directory
    let path = if args.len() < 2 {
        PathBuf::from(".")
    } else {
        PathBuf::from(&args[1])
    };

    // Check if path exists
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }

    Ok(path)
}
