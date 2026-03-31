//! Per-file view settings persistence.
//!
//! Saves and loads column widths, column types, bg/fg colors, and frozen
//! columns/rows to `~/.config/lazycsv/views.json`.

use crate::ui::conditional::{
    self, ColorRule, SerializableColorRule, SerializableRowConditionalRule,
};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Serializable view settings for a single file.
/// Uses String keys for JSON compatibility (JSON keys are always strings).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileView {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub column_widths: HashMap<String, u16>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub column_types: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub column_bg_colors: HashMap<String, Vec<SerializableColorRule>>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub column_fg_colors: HashMap<String, Vec<SerializableColorRule>>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub row_bg_colors: HashMap<String, Vec<SerializableColorRule>>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub row_fg_colors: HashMap<String, Vec<SerializableColorRule>>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub row_cond_bg: Vec<SerializableRowConditionalRule>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub row_cond_fg: Vec<SerializableRowConditionalRule>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frozen_columns: Vec<usize>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frozen_rows: Vec<usize>,
}

impl FileView {
    /// Returns true if all fields are empty (nothing to save).
    pub fn is_empty(&self) -> bool {
        self.column_widths.is_empty()
            && self.column_types.is_empty()
            && self.column_bg_colors.is_empty()
            && self.column_fg_colors.is_empty()
            && self.row_bg_colors.is_empty()
            && self.row_fg_colors.is_empty()
            && self.row_cond_bg.is_empty()
            && self.row_cond_fg.is_empty()
            && self.frozen_columns.is_empty()
            && self.frozen_rows.is_empty()
    }
}

/// All saved views, keyed by canonical file path.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViewStore {
    #[serde(flatten)]
    pub files: HashMap<String, FileView>,
}

/// Get the views file path (~/.config/lazycsv/views.json).
fn views_path() -> Option<PathBuf> {
    super::dirs_path().map(|p| p.join("views.json"))
}

/// Load views from disk.
pub fn load_views() -> ViewStore {
    let path = match views_path() {
        Some(p) => p,
        None => return ViewStore::default(),
    };

    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return ViewStore::default(),
    };

    serde_json::from_str(&data).unwrap_or_default()
}

/// Save views to disk.
pub fn save_views(store: &ViewStore) {
    let path = match views_path() {
        Some(p) => p,
        None => return,
    };

    // Ensure config directory exists
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    if let Ok(data) = serde_json::to_string_pretty(store) {
        let _ = std::fs::write(&path, data);
    }
}

/// Canonicalize a path to use as a stable views.json key.
/// Falls back to the original path string if canonicalization fails.
pub fn canonical_key(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string()
}

/// Save view settings for the active file in the given app.
/// Called on exit and after view-changing commands.
pub fn save_current_views(app: &crate::App) {
    let mut store = load_views();
    let active_path = app
        .session
        .files()
        .get(app.session.active_file_index())
        .cloned();
    if let Some(ref path) = active_path {
        let fv = collect_file_view(path, &app.session, &app.view_state);
        let key = canonical_key(path);
        if fv.is_empty() {
            store.files.remove(&key);
        } else {
            store.files.insert(key, fv);
        }
        save_views(&store);
    }
}

/// Convert a Color to a string for serialization.
pub fn color_to_string(color: Color) -> String {
    match color {
        Color::Black => "black".to_string(),
        Color::Red => "red".to_string(),
        Color::Green => "green".to_string(),
        Color::Yellow => "yellow".to_string(),
        Color::Blue => "blue".to_string(),
        Color::Magenta => "magenta".to_string(),
        Color::Cyan => "cyan".to_string(),
        Color::Gray => "gray".to_string(),
        Color::DarkGray => "darkgray".to_string(),
        Color::LightRed => "lightred".to_string(),
        Color::LightGreen => "lightgreen".to_string(),
        Color::LightYellow => "lightyellow".to_string(),
        Color::LightBlue => "lightblue".to_string(),
        Color::LightMagenta => "lightmagenta".to_string(),
        Color::LightCyan => "lightcyan".to_string(),
        Color::White => "white".to_string(),
        Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
        _ => "white".to_string(),
    }
}

/// Collect the current view state for a file into a FileView.
pub fn collect_file_view(
    file_path: &Path,
    session: &crate::session::Session,
    view_state: &crate::ui::view_state::ViewState,
) -> FileView {
    let mut fv = FileView::default();

    // Column widths from session (keyed by file)
    if let Some(widths) = session.column_widths_for(file_path) {
        fv.column_widths = widths.iter().map(|(&k, &v)| (k.to_string(), v)).collect();
    }

    // Column types from session
    if let Some(types) = session.column_types_for(file_path) {
        fv.column_types = types
            .iter()
            .map(|(&k, v)| (k.to_string(), v.display_name().to_string()))
            .collect();
    }

    // Column color rules from view state
    if !view_state.column_bg_colors.is_empty() {
        fv.column_bg_colors = view_state
            .column_bg_colors
            .iter()
            .map(|(&k, rules)| {
                (
                    k.to_string(),
                    rules.iter().map(conditional::serialize_rule).collect(),
                )
            })
            .collect();
    }
    if !view_state.column_fg_colors.is_empty() {
        fv.column_fg_colors = view_state
            .column_fg_colors
            .iter()
            .map(|(&k, rules)| {
                (
                    k.to_string(),
                    rules.iter().map(conditional::serialize_rule).collect(),
                )
            })
            .collect();
    }

    // Row color rules from view state
    if !view_state.row_bg_colors.is_empty() {
        fv.row_bg_colors = view_state
            .row_bg_colors
            .iter()
            .map(|(&k, rules)| {
                (
                    k.to_string(),
                    rules.iter().map(conditional::serialize_rule).collect(),
                )
            })
            .collect();
    }
    if !view_state.row_fg_colors.is_empty() {
        fv.row_fg_colors = view_state
            .row_fg_colors
            .iter()
            .map(|(&k, rules)| {
                (
                    k.to_string(),
                    rules.iter().map(conditional::serialize_rule).collect(),
                )
            })
            .collect();
    }

    // Row conditional rules
    if !view_state.row_cond_bg.is_empty() {
        fv.row_cond_bg = view_state
            .row_cond_bg
            .iter()
            .map(conditional::serialize_row_rule)
            .collect();
    }
    if !view_state.row_cond_fg.is_empty() {
        fv.row_cond_fg = view_state
            .row_cond_fg
            .iter()
            .map(conditional::serialize_row_rule)
            .collect();
    }

    // Frozen columns/rows from session
    let frozen_cols = session.frozen_columns_for(file_path);
    if !frozen_cols.is_empty() {
        fv.frozen_columns = frozen_cols.to_vec();
    }
    let frozen_rows = session.frozen_rows_for(file_path);
    if !frozen_rows.is_empty() {
        fv.frozen_rows = frozen_rows.to_vec();
    }

    fv
}

/// Apply a saved FileView to restore session and view state for a file.
pub fn apply_file_view(
    file_path: &Path,
    fv: &FileView,
    session: &mut crate::session::Session,
    view_state: &mut crate::ui::view_state::ViewState,
) {
    // Restore column widths
    for (col_str, &width) in &fv.column_widths {
        if let Ok(col) = col_str.parse::<usize>() {
            session.set_column_width_for(file_path, col, width);
        }
    }

    // Restore column types
    for (col_str, type_name) in &fv.column_types {
        if let Ok(col) = col_str.parse::<usize>() {
            if let Some(ct) = crate::column::metadata::ColumnType::from_name(type_name) {
                session.set_column_type_for(file_path, col, ct);
            }
        }
    }

    // Restore column color rules
    view_state.column_bg_colors.clear();
    for (col_str, rules) in &fv.column_bg_colors {
        if let Ok(col) = col_str.parse::<usize>() {
            let parsed: Vec<ColorRule> = rules
                .iter()
                .filter_map(conditional::deserialize_rule)
                .collect();
            if !parsed.is_empty() {
                view_state.column_bg_colors.insert(col, parsed);
            }
        }
    }
    view_state.column_fg_colors.clear();
    for (col_str, rules) in &fv.column_fg_colors {
        if let Ok(col) = col_str.parse::<usize>() {
            let parsed: Vec<ColorRule> = rules
                .iter()
                .filter_map(conditional::deserialize_rule)
                .collect();
            if !parsed.is_empty() {
                view_state.column_fg_colors.insert(col, parsed);
            }
        }
    }

    // Restore row color rules
    view_state.row_bg_colors.clear();
    for (row_str, rules) in &fv.row_bg_colors {
        if let Ok(row) = row_str.parse::<usize>() {
            let parsed: Vec<ColorRule> = rules
                .iter()
                .filter_map(conditional::deserialize_rule)
                .collect();
            if !parsed.is_empty() {
                view_state.row_bg_colors.insert(row, parsed);
            }
        }
    }
    view_state.row_fg_colors.clear();
    for (row_str, rules) in &fv.row_fg_colors {
        if let Ok(row) = row_str.parse::<usize>() {
            let parsed: Vec<ColorRule> = rules
                .iter()
                .filter_map(conditional::deserialize_rule)
                .collect();
            if !parsed.is_empty() {
                view_state.row_fg_colors.insert(row, parsed);
            }
        }
    }

    // Restore row conditional rules
    view_state.row_cond_bg = fv
        .row_cond_bg
        .iter()
        .filter_map(conditional::deserialize_row_rule)
        .collect();
    view_state.row_cond_fg = fv
        .row_cond_fg
        .iter()
        .filter_map(conditional::deserialize_row_rule)
        .collect();

    // Restore frozen columns/rows
    if !fv.frozen_columns.is_empty() {
        session.set_frozen_columns_for(file_path, fv.frozen_columns.clone());
    }
    if !fv.frozen_rows.is_empty() {
        session.set_frozen_rows_for(file_path, fv.frozen_rows.clone());
    }
}
