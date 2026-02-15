//! CSV document parsing and representation
//!
//! Handles loading CSV files from disk, parsing with configurable
//! delimiters and encoding, and providing in-memory document access.

pub mod document;
pub mod writer;

pub use document::Document;
pub use writer::write_csv_atomic;
