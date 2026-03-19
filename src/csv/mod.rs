//! CSV document parsing and representation
//!
//! Handles loading CSV files from disk, parsing with configurable
//! delimiters and encoding, and providing in-memory document access.

pub mod document;
pub mod row_storage;
pub mod writer;
pub mod xlsx;

pub use document::Document;
pub use writer::{write_csv_atomic, write_csv_content};
