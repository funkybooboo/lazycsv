//! Row storage backends for CSV documents.
//!
//! Provides two storage modes:
//! - `InMemory`: all rows stored as `Vec<Vec<String>>` (small files, SQL results)
//! - `Lazy`: memory-mapped file with row-offset index, parsing rows on demand

use anyhow::{Context, Result};
use lru::LruCache;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::num::NonZeroUsize;
use std::path::Path;

/// Threshold in bytes above which files use lazy loading.
const LAZY_THRESHOLD_BYTES: u64 = 10 * 1024 * 1024; // 10 MB

/// Number of parsed rows to keep in the LRU cache.
const ROW_CACHE_SIZE: usize = 2048;

/// Returns true if the file at `path` should use lazy loading.
pub fn should_use_lazy(path: &Path) -> bool {
    path.metadata()
        .map(|m| m.len() >= LAZY_THRESHOLD_BYTES)
        .unwrap_or(false)
}

/// Dual-mode row storage.
pub enum RowStorage {
    /// Fully materialized rows (existing behavior).
    InMemory { rows: Vec<Vec<String>> },
    /// Lazy disk-backed storage with row-offset index.
    Lazy(Box<LazyStorage>),
}

/// Lazy storage internals, boxed to keep the enum small.
pub struct LazyStorage {
    /// Memory-mapped file bytes.
    mmap: memmap2::Mmap,
    /// Byte offset of the start of each CSV record (monotonically increasing).
    /// Index 0 = first row, index 1 = second row, etc.
    /// These stay in original file order so get_row_bytes can use offsets[i+1] as end boundary.
    row_offsets: Vec<u64>,
    /// Optional sort indirection: logical row index → physical row_offsets index.
    /// When None, logical == physical. When Some, row access goes through this mapping.
    sort_order: Option<Vec<usize>>,
    /// Parsed first row (always materialized, typically contains column names).
    header: Vec<String>,
    /// Number of columns (from header).
    col_count: usize,
    /// Delimiter byte.
    delimiter: u8,
    /// LRU cache of recently parsed rows.
    row_cache: RefCell<LruCache<usize, Vec<String>>>,
    /// Edit overlay: logical row_index -> edited row.
    /// Takes priority over mmap data.
    edits: HashMap<usize, Vec<String>>,
}

impl RowStorage {
    /// Create in-memory storage from existing rows.
    pub fn in_memory(rows: Vec<Vec<String>>) -> Self {
        RowStorage::InMemory { rows }
    }

    /// Create lazy storage by memory-mapping a file and building a row-offset index.
    pub fn lazy_from_file(path: &Path, delimiter: Option<u8>, no_headers: bool) -> Result<Self> {
        let delim = delimiter.unwrap_or(b',');

        // Build row-offset index using buffered I/O (much faster than mmap on macOS)
        let row_offsets = build_row_offset_index_buffered(path, delim)?;

        // Now mmap the file for random-access row lookups
        let file = std::fs::File::open(path)
            .context(format!("Failed to open file: {}", path.display()))?;
        let mmap = unsafe { memmap2::MmapOptions::new().map(&file)? };

        // Parse first row
        let header = if row_offsets.is_empty() {
            vec![]
        } else {
            let header_bytes = get_row_bytes(&mmap, &row_offsets, 0);
            let parsed = parse_single_row(header_bytes, delim);
            if no_headers {
                // Generate synthetic headers, first row is data
                (1..=parsed.len())
                    .map(|i| format!("Column {}", i))
                    .collect()
            } else {
                parsed
            }
        };

        let col_count = header.len();

        let cache = LruCache::new(NonZeroUsize::new(ROW_CACHE_SIZE).unwrap());

        Ok(RowStorage::Lazy(Box::new(LazyStorage {
            mmap,
            row_offsets,
            sort_order: None,
            header,
            col_count,
            delimiter: delim,
            row_cache: RefCell::new(cache),
            edits: HashMap::new(),
        })))
    }

    /// Create lazy storage with cancellation support.
    pub fn lazy_from_file_cancellable(
        path: &Path,
        delimiter: Option<u8>,
        no_headers: bool,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<Self> {
        let delim = delimiter.unwrap_or(b',');

        // Build row-offset index using buffered I/O (much faster than mmap on macOS)
        let row_offsets = build_row_offset_index_buffered_cancellable(path, delim, cancelled)?;

        // Now mmap the file for random-access row lookups
        let file = std::fs::File::open(path)
            .context(format!("Failed to open file: {}", path.display()))?;
        let mmap = unsafe { memmap2::MmapOptions::new().map(&file)? };

        let header = if row_offsets.is_empty() {
            vec![]
        } else {
            let header_bytes = get_row_bytes(&mmap, &row_offsets, 0);
            let parsed = parse_single_row(header_bytes, delim);
            if no_headers {
                (1..=parsed.len())
                    .map(|i| format!("Column {}", i))
                    .collect()
            } else {
                parsed
            }
        };

        let col_count = header.len();
        let cache = LruCache::new(NonZeroUsize::new(ROW_CACHE_SIZE).unwrap());

        Ok(RowStorage::Lazy(Box::new(LazyStorage {
            mmap,
            row_offsets,
            sort_order: None,
            header,
            col_count,
            delimiter: delim,
            row_cache: RefCell::new(cache),
            edits: HashMap::new(),
        })))
    }

    // ── Accessors ──────────────────────────────────────────────

    /// Total number of rows including header.
    pub fn row_count(&self) -> usize {
        match self {
            RowStorage::InMemory { rows } => rows.len(),
            RowStorage::Lazy(s) => s.row_offsets.len(),
        }
    }

    /// Number of columns.
    pub fn col_count(&self) -> usize {
        match self {
            RowStorage::InMemory { rows } => rows.first().map(|r| r.len()).unwrap_or(0),
            RowStorage::Lazy(s) => s.col_count,
        }
    }

    /// Get a single row by index. Returns an owned `Vec<String>`.
    /// For InMemory, clones the row. For Lazy, checks edits, then cache, then parses.
    pub fn get_row(&self, idx: usize) -> Vec<String> {
        match self {
            RowStorage::InMemory { rows } => rows.get(idx).cloned().unwrap_or_default(),
            RowStorage::Lazy(s) => s.get_row(idx),
        }
    }

    /// Get a cell value. Returns "" if out of bounds.
    pub fn get_cell(&self, row_idx: usize, col_idx: usize) -> String {
        match self {
            RowStorage::InMemory { rows } => rows
                .get(row_idx)
                .and_then(|r| r.get(col_idx))
                .cloned()
                .unwrap_or_default(),
            RowStorage::Lazy(s) => {
                if row_idx == 0 {
                    return s.header.get(col_idx).cloned().unwrap_or_default();
                }
                if let Some(row) = s.edits.get(&row_idx) {
                    return row.get(col_idx).cloned().unwrap_or_default();
                }
                let row = s.parse_and_cache_row(idx_to_row(row_idx));
                row.get(col_idx).cloned().unwrap_or_default()
            }
        }
    }

    /// Get the first row (row 0).
    pub fn header_row(&self) -> &[String] {
        match self {
            RowStorage::InMemory { rows } => rows.first().map(|r| r.as_slice()).unwrap_or(&[]),
            RowStorage::Lazy(s) => &s.header,
        }
    }

    /// Get a range of rows [start..end) as owned Vecs.
    pub fn get_rows_range(&self, start: usize, end: usize) -> Vec<Vec<String>> {
        let end = end.min(self.row_count());
        if start >= end {
            return vec![];
        }
        match self {
            RowStorage::InMemory { rows } => rows[start..end].to_vec(),
            RowStorage::Lazy(s) => (start..end)
                .map(|i| {
                    if i == 0 {
                        s.header.clone()
                    } else {
                        s.get_row(i)
                    }
                })
                .collect(),
        }
    }

    /// Get a reference to a slice of rows (only works for InMemory).
    /// For Lazy, returns None.
    pub fn rows_slice(&self, start: usize, end: usize) -> Option<&[Vec<String>]> {
        match self {
            RowStorage::InMemory { rows } => {
                let end = end.min(rows.len());
                if start >= end {
                    Some(&[])
                } else {
                    Some(&rows[start..end])
                }
            }
            RowStorage::Lazy(_) => None,
        }
    }

    // ── Mutation ───────────────────────────────────────────────

    /// Set a cell value. For Lazy, materializes the row into the edit overlay.
    pub fn set_cell(&mut self, row_idx: usize, col_idx: usize, value: String) -> Option<String> {
        match self {
            RowStorage::InMemory { rows } => {
                if let Some(row) = rows.get_mut(row_idx) {
                    if let Some(cell) = row.get_mut(col_idx) {
                        return Some(std::mem::replace(cell, value));
                    }
                }
                None
            }
            RowStorage::Lazy(s) => {
                if row_idx == 0 {
                    // Edit header directly
                    if let Some(cell) = s.header.get_mut(col_idx) {
                        return Some(std::mem::replace(cell, value));
                    }
                    return None;
                }
                // Materialize row into edit overlay if not already there
                if !s.edits.contains_key(&row_idx) {
                    let parsed = s.parse_row(row_idx);
                    s.edits.insert(row_idx, parsed);
                }
                let row = s.edits.get_mut(&row_idx)?;
                if let Some(cell) = row.get_mut(col_idx) {
                    Some(std::mem::replace(cell, value))
                } else {
                    None
                }
            }
        }
    }

    /// Materialize all lazy rows into InMemory mode.
    /// No-op if already InMemory.
    pub fn materialize(&mut self) {
        let old = std::mem::replace(self, RowStorage::InMemory { rows: vec![] });
        match old {
            RowStorage::InMemory { rows } => {
                *self = RowStorage::InMemory { rows };
            }
            RowStorage::Lazy(s) => {
                let count = s.row_offsets.len();
                let mut rows = Vec::with_capacity(count);
                rows.push(s.header.clone());
                for i in 1..count {
                    rows.push(s.get_row(i));
                }
                *self = RowStorage::InMemory { rows };
            }
        }
    }

    /// Returns `true` if this storage is lazy (disk-backed).
    pub fn is_lazy(&self) -> bool {
        matches!(self, RowStorage::Lazy(_))
    }

    /// Get mutable access to in-memory rows, materializing if necessary.
    /// After this call, storage is guaranteed to be InMemory.
    pub fn rows_mut(&mut self) -> &mut Vec<Vec<String>> {
        self.materialize();
        match self {
            RowStorage::InMemory { rows } => rows,
            RowStorage::Lazy(_) => unreachable!(),
        }
    }

    /// Take ownership of rows, leaving empty storage behind.
    /// For Lazy, this drops the mmap without materializing (used for cleanup).
    pub fn take(self) -> Vec<Vec<String>> {
        match self {
            RowStorage::InMemory { rows } => rows,
            RowStorage::Lazy(_) => vec![], // mmap gets dropped
        }
    }

    /// Iterate over all rows (including header at index 0).
    /// For InMemory, iterates directly. For Lazy, parses on demand.
    pub fn iter_rows(&self) -> RowIter<'_> {
        RowIter {
            storage: self,
            idx: 0,
            count: self.row_count(),
        }
    }

    /// Check if the edit overlay has changes (Lazy mode only).
    pub fn has_edits(&self) -> bool {
        match self {
            RowStorage::InMemory { .. } => false,
            RowStorage::Lazy(s) => !s.edits.is_empty(),
        }
    }

    /// Get access to lazy storage internals (for optimized search).
    /// Returns None for InMemory storage.
    pub fn lazy_storage(&self) -> Option<&LazyStorage> {
        match self {
            RowStorage::InMemory { .. } => None,
            RowStorage::Lazy(s) => Some(s),
        }
    }

    /// Sort data rows by column indices using parallel sort with cancellation.
    /// For Lazy storage: extracts sort keys and reorders row_offsets (no materialization).
    /// For InMemory storage: uses parallel sort directly on the rows.
    /// Returns `true` if sort completed, `false` if cancelled.
    pub fn sort_by_columns(
        &mut self,
        col_indices: &[usize],
        ascending: bool,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> bool {
        use rayon::prelude::*;
        use std::sync::atomic::Ordering;

        match self {
            RowStorage::InMemory { rows } => {
                if rows.len() <= 2 {
                    return true;
                }
                let data = &mut rows[1..];
                data.par_sort_by(|a, b| {
                    if cancelled.load(Ordering::Relaxed) {
                        return std::cmp::Ordering::Equal;
                    }
                    compare_rows(a, b, col_indices, ascending)
                });
                !cancelled.load(Ordering::Relaxed)
            }
            RowStorage::Lazy(s) => {
                let count = s.row_offsets.len();
                if count <= 2 {
                    return true;
                }

                // Phase 1: Extract sort keys in parallel (cancellable)
                let keys: Vec<Option<Vec<SortKey>>> = (1..count)
                    .into_par_iter()
                    .map(|i| {
                        if cancelled.load(Ordering::Relaxed) {
                            return None;
                        }
                        let row_bytes = get_row_bytes(&s.mmap, &s.row_offsets, i);
                        Some(extract_sort_keys(row_bytes, s.delimiter, col_indices))
                    })
                    .collect();

                if cancelled.load(Ordering::Relaxed) {
                    return false;
                }

                // Unwrap keys (all Some since we weren't cancelled)
                let keys: Vec<Vec<SortKey>> =
                    keys.into_iter().map(|k| k.unwrap_or_default()).collect();

                // Phase 2: Parallel sort indices by keys (cancellable)
                let mut indices: Vec<usize> = (0..keys.len()).collect();

                indices.par_sort_by(|&ai, &bi| {
                    if cancelled.load(Ordering::Relaxed) {
                        return std::cmp::Ordering::Equal;
                    }
                    let ka = &keys[ai];
                    let kb = &keys[bi];
                    for i in 0..ka.len() {
                        let ord = ka[i].cmp(&kb[i]);
                        let ord = if ascending { ord } else { ord.reverse() };
                        if ord != std::cmp::Ordering::Equal {
                            return ord;
                        }
                    }
                    std::cmp::Ordering::Equal
                });

                if cancelled.load(Ordering::Relaxed) {
                    return false;
                }

                // Phase 3: Set sort_order indirection (row_offsets stay in original file order)
                // indices[new_data_pos] = old_data_idx (0-based)
                // sort_order[new_data_pos] = physical row_offsets index (1-based)
                let order: Vec<usize> = indices.iter().map(|&i| i + 1).collect();

                // Reorder edits map to match new logical positions
                if !s.edits.is_empty() {
                    let old_edits = std::mem::take(&mut s.edits);
                    // Build reverse mapping: old_logical_idx -> new_logical_idx
                    // old_logical_idx = old_data_idx + 1, new_logical_idx = new_data_pos + 1
                    let mut reverse_map: HashMap<usize, usize> = HashMap::new();
                    for (new_pos, &old_data_idx) in indices.iter().enumerate() {
                        reverse_map.insert(old_data_idx + 1, new_pos + 1);
                    }
                    for (old_row_idx, row_data) in old_edits {
                        if let Some(&new_row_idx) = reverse_map.get(&old_row_idx) {
                            s.edits.insert(new_row_idx, row_data);
                        }
                    }
                }

                s.sort_order = Some(order);

                // Clear row cache since logical indices have changed
                s.row_cache.borrow_mut().clear();
                true
            }
        }
    }
}

// Helper to avoid confusion — in lazy mode, row 0 is the header
// and is stored separately. Row indices 1..N map to row_offsets 1..N.
fn idx_to_row(idx: usize) -> usize {
    idx
}

impl LazyStorage {
    /// Map a logical row index to a physical row_offsets index.
    /// When sorted, logical indices go through the sort_order indirection.
    /// Header (index 0) always maps to physical 0.
    fn physical_idx(&self, logical_idx: usize) -> usize {
        if logical_idx == 0 {
            return 0;
        }
        match &self.sort_order {
            Some(order) => {
                let data_idx = logical_idx - 1;
                if data_idx < order.len() {
                    order[data_idx]
                } else {
                    logical_idx
                }
            }
            None => logical_idx,
        }
    }

    /// Raw mmap bytes for byte-level search.
    pub fn raw_bytes(&self) -> &[u8] {
        &self.mmap
    }

    /// Row offset table.
    pub fn row_offsets(&self) -> &[u64] {
        &self.row_offsets
    }

    /// Sort order indirection (if sorted).
    pub fn sort_order(&self) -> Option<&[usize]> {
        self.sort_order.as_deref()
    }

    /// Delimiter byte.
    pub fn delimiter(&self) -> u8 {
        self.delimiter
    }

    /// First row (typically contains column names).
    pub fn header(&self) -> &[String] {
        &self.header
    }

    /// Edited rows overlay.
    pub fn edits(&self) -> &HashMap<usize, Vec<String>> {
        &self.edits
    }

    /// Get the raw bytes for a specific logical row index.
    pub fn row_bytes(&self, idx: usize) -> &[u8] {
        let phys = self.physical_idx(idx);
        get_row_bytes(&self.mmap, &self.row_offsets, phys)
    }

    /// Parse a single row by logical index (public for search).
    pub fn parse_row_public(&self, idx: usize) -> Vec<String> {
        self.get_row(idx)
    }

    /// Get a row, checking edits first, then cache, then parsing from mmap.
    fn get_row(&self, idx: usize) -> Vec<String> {
        if idx == 0 {
            return self.header.clone();
        }
        if let Some(row) = self.edits.get(&idx) {
            return row.clone();
        }
        self.parse_and_cache_row(idx)
    }

    /// Parse a row from mmap and insert into cache.
    fn parse_and_cache_row(&self, idx: usize) -> Vec<String> {
        let mut cache = self.row_cache.borrow_mut();
        if let Some(row) = cache.get(&idx) {
            return row.clone();
        }
        let row = self.parse_row(idx);
        cache.put(idx, row.clone());
        row
    }

    /// Parse a row directly from the memory-mapped bytes using physical index.
    fn parse_row(&self, logical_idx: usize) -> Vec<String> {
        let phys = self.physical_idx(logical_idx);
        if phys >= self.row_offsets.len() {
            return vec![];
        }
        let bytes = get_row_bytes(&self.mmap, &self.row_offsets, phys);
        parse_single_row(bytes, self.delimiter)
    }
}

/// Iterator over all rows in any storage mode.
pub struct RowIter<'a> {
    storage: &'a RowStorage,
    idx: usize,
    count: usize,
}

impl<'a> Iterator for RowIter<'a> {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.count {
            return None;
        }
        let row = self.storage.get_row(self.idx);
        self.idx += 1;
        Some(row)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.count - self.idx;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for RowIter<'_> {}

// ── Index Building ─────────────────────────────────────────────

/// Read buffer size for index building (1 MB).
const INDEX_BUF_SIZE: usize = 1024 * 1024;

/// Fast row count using buffered I/O and memchr newline scanning.
/// Counts data rows (excludes header when `no_headers` is false).
/// This is much faster than full CSV parsing since it only scans for
/// newlines and quotes without parsing individual fields.
///
/// For files larger than 10 MB, uses parallel chunk processing with rayon.
pub fn count_rows_fast(path: &Path, no_headers: bool) -> Result<usize> {
    let file =
        std::fs::File::open(path).context(format!("Failed to open file: {}", path.display()))?;
    let file_len = file.metadata()?.len() as usize;

    if file_len == 0 {
        return Ok(0);
    }

    const PARALLEL_THRESHOLD: usize = 10 * 1024 * 1024; // 10 MB
    let row_count = if file_len >= PARALLEL_THRESHOLD {
        count_rows_parallel(path, file_len)?
    } else {
        count_rows_sequential(file, file_len)?
    };

    // Subtract header row if headers are present
    if !no_headers && row_count > 0 {
        Ok(row_count - 1)
    } else {
        Ok(row_count)
    }
}

/// Single-threaded row count for small files.
fn count_rows_sequential(file: std::fs::File, _file_len: usize) -> Result<usize> {
    use std::io::Seek;

    let mut reader = std::io::BufReader::with_capacity(INDEX_BUF_SIZE, file);
    let mut buf = vec![0u8; INDEX_BUF_SIZE];
    let mut row_count: usize = 1; // First row starts at byte 0
    let mut in_quotes = false;
    let mut skip_next_quote = false;

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let chunk = &buf[..n];
        let finder = memchr::memchr2_iter(b'\n', b'"', chunk);

        for local_pos in finder {
            let b = chunk[local_pos];
            if b == b'"' {
                if skip_next_quote {
                    skip_next_quote = false;
                    continue;
                }
                if in_quotes {
                    let next_is_quote = if local_pos + 1 < n {
                        chunk[local_pos + 1] == b'"'
                    } else {
                        false
                    };
                    if next_is_quote {
                        skip_next_quote = true;
                        continue;
                    }
                    in_quotes = false;
                } else {
                    in_quotes = true;
                }
            } else {
                // b == b'\n'
                skip_next_quote = false;
                if !in_quotes {
                    row_count += 1;
                }
            }
        }
    }

    // Check if file ends with newline — don't count trailing empty row
    let mut f = reader.into_inner();
    f.seek(std::io::SeekFrom::End(-1))?;
    let mut b = [0u8; 1];
    f.read_exact(&mut b)?;
    if b[0] == b'\n' {
        row_count -= 1;
    }

    Ok(row_count)
}

/// Result of scanning a chunk, computed for both possible starting quote states.
struct ChunkScanResult {
    /// Newlines counted assuming we enter this chunk NOT in quotes.
    newlines_if_unquoted: usize,
    /// Newlines counted assuming we enter this chunk IN quotes.
    newlines_if_quoted: usize,
    /// Whether quote state is flipped at end of chunk (odd unescaped quotes).
    /// If true, the exit quote state is the opposite of the entry state.
    flips_quote_state: bool,
}

/// Scan a byte slice, counting newlines for both possible entry quote states.
/// Also tracks whether the chunk has an odd number of unescaped quotes (flips state).
fn scan_chunk(chunk: &[u8]) -> ChunkScanResult {
    let mut newlines_unquoted = 0usize; // newlines outside quotes, assuming start unquoted
    let mut newlines_quoted = 0usize; // newlines outside quotes, assuming start quoted
    let mut in_quotes_if_started_unquoted = false;
    let mut in_quotes_if_started_quoted = true;
    let mut skip_next_unquoted = false;
    let mut skip_next_quoted = false;
    let n = chunk.len();

    let finder = memchr::memchr2_iter(b'\n', b'"', chunk);
    for pos in finder {
        let b = chunk[pos];
        if b == b'"' {
            // Path 1: started unquoted
            if skip_next_unquoted {
                skip_next_unquoted = false;
            } else if in_quotes_if_started_unquoted {
                let next_is_quote = pos + 1 < n && chunk[pos + 1] == b'"';
                if next_is_quote {
                    skip_next_unquoted = true;
                } else {
                    in_quotes_if_started_unquoted = false;
                }
            } else {
                in_quotes_if_started_unquoted = true;
            }

            // Path 2: started quoted
            if skip_next_quoted {
                skip_next_quoted = false;
            } else if in_quotes_if_started_quoted {
                let next_is_quote = pos + 1 < n && chunk[pos + 1] == b'"';
                if next_is_quote {
                    skip_next_quoted = true;
                } else {
                    in_quotes_if_started_quoted = false;
                }
            } else {
                in_quotes_if_started_quoted = true;
            }
        } else {
            // b == b'\n'
            skip_next_unquoted = false;
            skip_next_quoted = false;
            if !in_quotes_if_started_unquoted {
                newlines_unquoted += 1;
            }
            if !in_quotes_if_started_quoted {
                newlines_quoted += 1;
            }
        }
    }

    ChunkScanResult {
        newlines_if_unquoted: newlines_unquoted,
        newlines_if_quoted: newlines_quoted,
        flips_quote_state: in_quotes_if_started_unquoted, // started false, ended true = flipped
    }
}

/// Parallel row count using rayon. Splits file into chunks processed concurrently.
fn count_rows_parallel(path: &Path, file_len: usize) -> Result<usize> {
    use rayon::prelude::*;
    use std::io::Seek;

    let num_threads = rayon::current_num_threads().max(1);
    let chunk_size = (file_len / num_threads).max(INDEX_BUF_SIZE);
    let num_chunks = (file_len + chunk_size - 1) / chunk_size;

    // Each thread reads its chunk incrementally in INDEX_BUF_SIZE sub-buffers
    // to avoid allocating the entire chunk in memory at once.
    let chunk_results: Vec<ChunkScanResult> = (0..num_chunks)
        .into_par_iter()
        .map(|i| {
            let start = i * chunk_size;
            let end = (start + chunk_size).min(file_len);
            let remaining = end - start;

            let mut file =
                std::fs::File::open(path).expect("Failed to open file for parallel read");
            file.seek(std::io::SeekFrom::Start(start as u64))
                .expect("Failed to seek");

            let mut buf = vec![0u8; INDEX_BUF_SIZE];
            let mut accum = ChunkScanResult {
                newlines_if_unquoted: 0,
                newlines_if_quoted: 0,
                flips_quote_state: false,
            };
            let mut bytes_left = remaining;

            while bytes_left > 0 {
                let to_read = bytes_left.min(INDEX_BUF_SIZE);
                let mut total_read = 0;
                while total_read < to_read {
                    let n = file
                        .read(&mut buf[total_read..to_read])
                        .expect("Failed to read chunk");
                    if n == 0 {
                        break;
                    }
                    total_read += n;
                }
                if total_read == 0 {
                    break;
                }

                let sub = scan_chunk(&buf[..total_read]);

                // Merge sub-result into accumulator.
                // The accumulated flips_quote_state tells us the entry state for this sub-chunk:
                // if flipped, the sub-chunk starts in the opposite state of the chunk's entry.
                if accum.flips_quote_state {
                    // Sub-chunk enters with flipped state relative to chunk entry
                    accum.newlines_if_unquoted += sub.newlines_if_quoted;
                    accum.newlines_if_quoted += sub.newlines_if_unquoted;
                } else {
                    accum.newlines_if_unquoted += sub.newlines_if_unquoted;
                    accum.newlines_if_quoted += sub.newlines_if_quoted;
                }
                if sub.flips_quote_state {
                    accum.flips_quote_state = !accum.flips_quote_state;
                }

                bytes_left -= total_read;
            }

            accum
        })
        .collect();

    // Sequential reconciliation: walk chunks, pick correct count based on actual quote state
    let mut row_count: usize = 1; // First row starts at byte 0
    let mut in_quotes = false;

    for result in &chunk_results {
        if in_quotes {
            row_count += result.newlines_if_quoted;
        } else {
            row_count += result.newlines_if_unquoted;
        }
        if result.flips_quote_state {
            in_quotes = !in_quotes;
        }
    }

    // Check trailing newline
    let mut file = std::fs::File::open(path)?;
    file.seek(std::io::SeekFrom::End(-1))?;
    let mut b = [0u8; 1];
    file.read_exact(&mut b)?;
    if b[0] == b'\n' {
        row_count -= 1;
    }

    Ok(row_count)
}

/// Build row-offset index using buffered I/O.
/// This avoids mmap page-fault overhead on macOS, where sequential mmap
/// access is significantly slower than buffered reads.
fn build_row_offset_index_buffered(path: &Path, _delimiter: u8) -> Result<Vec<u64>> {
    let file =
        std::fs::File::open(path).context(format!("Failed to open file: {}", path.display()))?;
    let file_len = file.metadata()?.len() as usize;

    if file_len == 0 {
        return Ok(Vec::new());
    }

    let estimated_rows = file_len / 50;
    let mut offsets = Vec::with_capacity(estimated_rows);
    offsets.push(0);

    let mut reader = std::io::BufReader::with_capacity(INDEX_BUF_SIZE, file);
    let mut buf = vec![0u8; INDEX_BUF_SIZE];
    let mut global_pos: u64 = 0;
    let mut in_quotes = false;
    let mut skip_next_quote = false;
    // Carry over one byte from previous chunk to detect escaped quotes at boundaries
    let mut prev_last_byte: Option<u8> = None;

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let chunk = &buf[..n];

        let finder = memchr::memchr2_iter(b'\n', b'"', chunk);

        for local_pos in finder {
            let b = chunk[local_pos];
            let abs_pos = global_pos + local_pos as u64;

            if b == b'"' {
                if skip_next_quote {
                    skip_next_quote = false;
                    continue;
                }
                if in_quotes {
                    // Check if next byte is also a quote (escaped "")
                    let next_is_quote = if local_pos + 1 < n {
                        chunk[local_pos + 1] == b'"'
                    } else {
                        false // Will be handled in next chunk via prev_last_byte
                    };
                    if next_is_quote {
                        skip_next_quote = true;
                        continue;
                    }
                    in_quotes = false;
                } else {
                    in_quotes = true;
                }
            } else {
                // b == b'\n'
                skip_next_quote = false;
                if !in_quotes {
                    let row_start = abs_pos + 1;
                    if row_start < file_len as u64 {
                        offsets.push(row_start);
                    }
                }
            }
        }

        // Handle escaped quote split across chunk boundary:
        // If chunk ended with '"' and we're in_quotes, check next chunk's first byte
        if n > 0 {
            prev_last_byte = Some(chunk[n - 1]);
        }

        global_pos += n as u64;
    }
    let _ = prev_last_byte; // reserved for future boundary handling

    Ok(offsets)
}

/// Build row-offset index using buffered I/O with cancellation support.
fn build_row_offset_index_buffered_cancellable(
    path: &Path,
    _delimiter: u8,
    cancelled: &std::sync::atomic::AtomicBool,
) -> Result<Vec<u64>> {
    use crate::cancel::{self, CancelledError};

    let file =
        std::fs::File::open(path).context(format!("Failed to open file: {}", path.display()))?;
    let file_len = file.metadata()?.len() as usize;

    if file_len == 0 {
        return Ok(Vec::new());
    }

    let estimated_rows = file_len / 50;
    let mut offsets = Vec::with_capacity(estimated_rows);
    offsets.push(0);

    let mut reader = std::io::BufReader::with_capacity(INDEX_BUF_SIZE, file);
    let mut buf = vec![0u8; INDEX_BUF_SIZE];
    let mut global_pos: u64 = 0;
    let mut in_quotes = false;
    let mut skip_next_quote = false;
    let mut bytes_since_check: u64 = 0;
    let check_interval: u64 = 10_000_000;

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let chunk = &buf[..n];

        bytes_since_check += n as u64;
        if bytes_since_check >= check_interval {
            if cancel::check_esc(cancelled) {
                anyhow::bail!(CancelledError);
            }
            bytes_since_check = 0;
        }

        let finder = memchr::memchr2_iter(b'\n', b'"', chunk);

        for local_pos in finder {
            let b = chunk[local_pos];
            let abs_pos = global_pos + local_pos as u64;

            if b == b'"' {
                if skip_next_quote {
                    skip_next_quote = false;
                    continue;
                }
                if in_quotes {
                    let next_is_quote = if local_pos + 1 < n {
                        chunk[local_pos + 1] == b'"'
                    } else {
                        false
                    };
                    if next_is_quote {
                        skip_next_quote = true;
                        continue;
                    }
                    in_quotes = false;
                } else {
                    in_quotes = true;
                }
            } else {
                // b == b'\n'
                skip_next_quote = false;
                if !in_quotes {
                    let row_start = abs_pos + 1;
                    if row_start < file_len as u64 {
                        offsets.push(row_start);
                    }
                }
            }
        }

        global_pos += n as u64;
    }

    Ok(offsets)
}

/// Build row-offset index from an in-memory slice (used by tests).
#[cfg(test)]
fn build_row_offset_index(data: &[u8], _delimiter: u8) -> Vec<u64> {
    if data.is_empty() {
        return Vec::new();
    }

    let estimated_rows = data.len() / 50;
    let mut offsets = Vec::with_capacity(estimated_rows);
    offsets.push(0);

    let finder = memchr::memchr2_iter(b'\n', b'"', data);
    let len = data.len();
    let mut in_quotes = false;
    let mut skip_next_quote = false;

    for pos in finder {
        let b = data[pos];
        if b == b'"' {
            if skip_next_quote {
                skip_next_quote = false;
                continue;
            }
            if in_quotes {
                if pos + 1 < len && data[pos + 1] == b'"' {
                    skip_next_quote = true;
                    continue;
                }
                in_quotes = false;
            } else {
                in_quotes = true;
            }
        } else {
            skip_next_quote = false;
            if !in_quotes {
                let row_start = pos + 1;
                if row_start < len {
                    offsets.push(row_start as u64);
                }
            }
        }
    }

    offsets
}

// ── Sort Key Helpers ───────────────────────────────────────────

/// Pre-parsed sort key for efficient comparison.
#[derive(Clone, PartialEq)]
enum SortKey {
    Numeric(f64),
    Text(String),
}

impl Eq for SortKey {}

impl PartialOrd for SortKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SortKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (SortKey::Numeric(a), SortKey::Numeric(b)) => {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            }
            (SortKey::Text(a), SortKey::Text(b)) => a.cmp(b),
            // Numeric sorts before text
            (SortKey::Numeric(_), SortKey::Text(_)) => std::cmp::Ordering::Less,
            (SortKey::Text(_), SortKey::Numeric(_)) => std::cmp::Ordering::Greater,
        }
    }
}

/// Extract sort keys from raw row bytes for the given column indices.
fn extract_sort_keys(row_bytes: &[u8], delimiter: u8, col_indices: &[usize]) -> Vec<SortKey> {
    let row = parse_single_row(row_bytes, delimiter);
    col_indices
        .iter()
        .map(|&col| {
            let val = row.get(col).map(|s| s.as_str()).unwrap_or("");
            match val.parse::<f64>() {
                Ok(n) => SortKey::Numeric(n),
                Err(_) => SortKey::Text(val.to_owned()),
            }
        })
        .collect()
}

/// Compare two in-memory rows by column indices.
fn compare_rows(
    a: &[String],
    b: &[String],
    col_indices: &[usize],
    ascending: bool,
) -> std::cmp::Ordering {
    for &col in col_indices {
        let va = a.get(col).map(|s| s.as_str()).unwrap_or("");
        let vb = b.get(col).map(|s| s.as_str()).unwrap_or("");
        let ord = match (va.parse::<f64>(), vb.parse::<f64>()) {
            (Ok(na), Ok(nb)) => na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal),
            _ => va.cmp(vb),
        };
        let ord = if ascending { ord } else { ord.reverse() };
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }
    }
    std::cmp::Ordering::Equal
}

/// Extract the raw bytes for a given row index.
fn get_row_bytes<'a>(data: &'a [u8], offsets: &[u64], idx: usize) -> &'a [u8] {
    let start = offsets[idx] as usize;
    let end = if idx + 1 < offsets.len() {
        offsets[idx + 1] as usize
    } else {
        data.len()
    };
    // Trim trailing \r\n or \n
    let mut end = end;
    while end > start && (data[end - 1] == b'\n' || data[end - 1] == b'\r') {
        end -= 1;
    }
    &data[start..end]
}

/// Parse a single CSV row from raw bytes using the csv crate.
fn parse_single_row(bytes: &[u8], delimiter: u8) -> Vec<String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimiter)
        .from_reader(bytes);

    let mut record = csv::ByteRecord::new();
    if reader.read_byte_record(&mut record).unwrap_or(false) {
        record
            .iter()
            .map(|field| match std::str::from_utf8(field) {
                Ok(s) => s.to_owned(),
                Err(_) => String::from_utf8_lossy(field).into_owned(),
            })
            .collect()
    } else {
        vec![]
    }
}

// ── Debug / Clone ──────────────────────────────────────────────

impl std::fmt::Debug for RowStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RowStorage::InMemory { rows } => f
                .debug_struct("InMemory")
                .field("row_count", &rows.len())
                .finish(),
            RowStorage::Lazy(s) => f
                .debug_struct("Lazy")
                .field("row_count", &s.row_offsets.len())
                .field("col_count", &s.col_count)
                .field("edits", &s.edits.len())
                .finish(),
        }
    }
}

impl Clone for RowStorage {
    fn clone(&self) -> Self {
        match self {
            RowStorage::InMemory { rows } => RowStorage::InMemory { rows: rows.clone() },
            RowStorage::Lazy(s) => {
                // Materialize on clone — can't clone an mmap safely
                let count = s.row_offsets.len();
                let mut rows = Vec::with_capacity(count);
                rows.push(s.header.clone());
                for i in 1..count {
                    rows.push(s.get_row(i));
                }
                RowStorage::InMemory { rows }
            }
        }
    }
}

impl PartialEq for RowStorage {
    fn eq(&self, other: &Self) -> bool {
        // Compare by content — both must produce the same rows
        if self.row_count() != other.row_count() || self.col_count() != other.col_count() {
            return false;
        }
        for i in 0..self.row_count() {
            if self.get_row(i) != other.get_row(i) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_basic() {
        let rows = vec![
            vec!["Name".into(), "Age".into()],
            vec!["Alice".into(), "30".into()],
            vec!["Bob".into(), "25".into()],
        ];
        let storage = RowStorage::in_memory(rows);

        assert_eq!(storage.row_count(), 3);
        assert_eq!(storage.col_count(), 2);
        assert_eq!(storage.get_cell(0, 0), "Name");
        assert_eq!(storage.get_cell(1, 0), "Alice");
        assert_eq!(storage.get_cell(2, 1), "25");
        assert_eq!(storage.header_row(), &["Name", "Age"]);
    }

    #[test]
    fn test_in_memory_set_cell() {
        let rows = vec![vec!["A".into()], vec!["1".into()]];
        let mut storage = RowStorage::in_memory(rows);

        let old = storage.set_cell(1, 0, "2".into());
        assert_eq!(old, Some("1".into()));
        assert_eq!(storage.get_cell(1, 0), "2");
    }

    #[test]
    fn test_in_memory_get_rows_range() {
        let rows = vec![
            vec!["H".into()],
            vec!["1".into()],
            vec!["2".into()],
            vec!["3".into()],
        ];
        let storage = RowStorage::in_memory(rows);

        let range = storage.get_rows_range(1, 3);
        assert_eq!(range.len(), 2);
        assert_eq!(range[0], vec!["1".to_string()]);
        assert_eq!(range[1], vec!["2".to_string()]);
    }

    #[test]
    fn test_in_memory_iter_rows() {
        let rows = vec![vec!["H".into()], vec!["1".into()], vec!["2".into()]];
        let storage = RowStorage::in_memory(rows);

        let collected: Vec<_> = storage.iter_rows().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], vec!["H".to_string()]);
    }

    #[test]
    fn test_build_row_offset_index_simple() {
        let data = b"Name,Age\nAlice,30\nBob,25\n";
        let offsets = build_row_offset_index(data, b',');
        assert_eq!(offsets, vec![0, 9, 18]);
    }

    #[test]
    fn test_build_row_offset_index_crlf() {
        let data = b"A,B\r\n1,2\r\n3,4\r\n";
        let offsets = build_row_offset_index(data, b',');
        assert_eq!(offsets, vec![0, 5, 10]);
    }

    #[test]
    fn test_build_row_offset_index_quoted_newline() {
        let data = b"A,B\n\"hello\nworld\",2\n3,4\n";
        let offsets = build_row_offset_index(data, b',');
        // Row 0 starts at 0, row 1 at 4 (after "A,B\n"),
        // the newline inside quotes is NOT a row boundary,
        // row 2 starts at 20 (after the quoted field row)
        assert_eq!(offsets, vec![0, 4, 20]);
    }

    #[test]
    fn test_build_row_offset_index_escaped_quotes() {
        let data = b"A\n\"he said \"\"hi\"\"\",2\n3,4\n";
        let offsets = build_row_offset_index(data, b',');
        assert_eq!(offsets, vec![0, 2, 21]);
    }

    #[test]
    fn test_parse_single_row() {
        let row = parse_single_row(b"Alice,30,NYC", b',');
        assert_eq!(row, vec!["Alice", "30", "NYC"]);
    }

    #[test]
    fn test_parse_single_row_quoted() {
        let row = parse_single_row(b"\"Last, First\",30", b',');
        assert_eq!(row, vec!["Last, First", "30"]);
    }

    #[test]
    fn test_get_row_bytes() {
        let data = b"A,B\nAlice,30\nBob,25\n";
        let offsets = vec![0, 4, 13];

        let row0 = get_row_bytes(data, &offsets, 0);
        assert_eq!(row0, b"A,B");

        let row1 = get_row_bytes(data, &offsets, 1);
        assert_eq!(row1, b"Alice,30");

        let row2 = get_row_bytes(data, &offsets, 2);
        assert_eq!(row2, b"Bob,25");
    }

    #[test]
    fn test_lazy_from_file() {
        use std::io::Write;
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "Name,Age").unwrap();
        writeln!(file, "Alice,30").unwrap();
        writeln!(file, "Bob,25").unwrap();

        let storage = RowStorage::lazy_from_file(file.path(), None, false).unwrap();

        assert_eq!(storage.row_count(), 3);
        assert_eq!(storage.col_count(), 2);
        assert_eq!(storage.header_row(), &["Name", "Age"]);
        assert_eq!(storage.get_row(1), vec!["Alice", "30"]);
        assert_eq!(storage.get_row(2), vec!["Bob", "25"]);
        assert_eq!(storage.get_cell(1, 0), "Alice");
    }

    #[test]
    fn test_lazy_edit_overlay() {
        use std::io::Write;
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "Name,Age").unwrap();
        writeln!(file, "Alice,30").unwrap();

        let mut storage = RowStorage::lazy_from_file(file.path(), None, false).unwrap();

        let old = storage.set_cell(1, 1, "31".into());
        assert_eq!(old, Some("30".into()));
        assert_eq!(storage.get_cell(1, 1), "31");
        assert!(storage.has_edits());
    }

    #[test]
    fn test_lazy_materialize() {
        use std::io::Write;
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "A,B").unwrap();
        writeln!(file, "1,2").unwrap();
        writeln!(file, "3,4").unwrap();

        let mut storage = RowStorage::lazy_from_file(file.path(), None, false).unwrap();
        assert!(storage.is_lazy());

        storage.materialize();
        assert!(!storage.is_lazy());
        assert_eq!(storage.row_count(), 3);
        assert_eq!(storage.get_cell(1, 0), "1");
        assert_eq!(storage.get_cell(2, 1), "4");
    }

    #[test]
    fn test_lazy_no_headers() {
        use std::io::Write;
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "Alice,30").unwrap();
        writeln!(file, "Bob,25").unwrap();

        let storage = RowStorage::lazy_from_file(file.path(), None, true).unwrap();

        assert_eq!(storage.header_row(), &["Column 1", "Column 2"]);
        // Row 0 in offsets is "Alice,30" which is data, not header
        // But we treat row_offsets[0] as the header offset...
        // With no_headers, the first row is both "header" (synthetic) and data
        assert_eq!(storage.row_count(), 2);
    }

    #[test]
    fn test_rows_mut_materializes() {
        use std::io::Write;
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "A").unwrap();
        writeln!(file, "1").unwrap();

        let mut storage = RowStorage::lazy_from_file(file.path(), None, false).unwrap();
        assert!(storage.is_lazy());

        let rows = storage.rows_mut();
        rows.push(vec!["2".into()]);
        assert!(!storage.is_lazy());
        assert_eq!(storage.row_count(), 3);
    }
}
