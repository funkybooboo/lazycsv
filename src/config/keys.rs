//! Customizable keybindings.
//!
//! This module owns the parsing and lookup machinery for user-remappable
//! keybindings. The high-level pieces are:
//!
//! - [`KeySequence`]: a normalized sequence of `(KeyCode, KeyModifiers)` pairs.
//!   `"j"`, `"ctrl+s"`, `"<esc>"`, `"gg"`, `",dd"` all parse via [`parse_sequence`].
//! - [`Keymap`] (TODO, comes in step #13): a per-mode lookup table built from
//!   a TOML file at `~/.config/lazycsv/keys.toml` (or per-directory
//!   `.lazycsv.toml`).
//!
//! ## Sequence syntax
//!
//! Sequences are concatenations of *atoms*. An atom is either a single
//! character (`j`), a bracketed reserved key (`<esc>`), or a modifier-prefixed
//! atom (`ctrl+s`, `ctrl+shift+f`). Atoms can be glued together to form
//! chord sequences:
//!
//! - `j` — one keypress, lowercase j with no modifiers
//! - `J` — one keypress, **shift+J** (uppercase letters auto-lift `Shift`)
//! - `gg` — two-key chord, `g` then `g`
//! - `,dd` — three-key chord, `,` then `d` then `d`
//! - `ctrl+s` — one keypress, Ctrl-s
//! - `<esc>` — Esc key
//! - `<f2>` — F2 function key
//! - `<space>j` — chord: spacebar, then j
//! - `g~` — chord: g then tilde
//! - `ctrl+x ctrl+s` — multi-atom chord; whitespace separates atoms
//!
//! Modifier names: `ctrl`, `shift`, `alt`, `super` (case-insensitive).
//!
//! Reserved key names (inside `<…>`, case-insensitive):
//! `esc`, `enter`, `tab`, `bs`/`backspace`, `space`, `del`/`delete`,
//! `up`, `down`, `left`, `right`, `home`, `end`, `pgup`/`pageup`,
//! `pgdn`/`pagedown`, `f1`..`f12`.
//!
//! ## Examples
//!
//! ```
//! use lazycsv::config::keys::parse_sequence;
//! let seq = parse_sequence("ctrl+s").expect("valid");
//! assert_eq!(seq.len(), 1);
//!
//! let chord = parse_sequence("gg").expect("valid chord");
//! assert_eq!(chord.len(), 2);
//! ```

use crate::input::keymap_actions::Action;
use crossterm::event::{KeyCode, KeyModifiers};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Modes a binding can target. Matches `crate::app::types::Mode` plus a
/// virtual `Global` bucket for bindings that should resolve regardless of
/// the active mode (e.g. `?` for help). Distinct enum so changes to the
/// app's `Mode` don't ripple out — we just map through [`KeymapScope::for_mode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeymapScope {
    Normal,
    Insert,
    Visual,
    Command,
    Search,
    Magnifier,
    FileList,
    SqlEditor,
    FileOperation,
    /// Bindings that fire in any mode if the more specific mode lookup misses.
    Global,
}

impl KeymapScope {
    /// Map a runtime [`crate::app::Mode`] to the scope used for keymap lookup.
    /// VisualBlock/Line/Column collapse onto the shared `Visual` scope since
    /// they share their key handler today.
    pub fn for_mode(mode: crate::app::Mode) -> Self {
        use crate::app::Mode::*;
        match mode {
            Normal => Self::Normal,
            Insert => Self::Insert,
            VisualBlock | VisualLine | VisualColumn => Self::Visual,
            Command => Self::Command,
            Search => Self::Search,
            Magnifier => Self::Magnifier,
            FileList => Self::FileList,
            SqlEditor => Self::SqlEditor,
            FileOperationPrompt => Self::FileOperation,
        }
    }

    /// TOML section header that maps to this scope (`[normal]`, `[insert]`, …).
    fn toml_key(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Insert => "insert",
            Self::Visual => "visual",
            Self::Command => "command",
            Self::Search => "search",
            Self::Magnifier => "magnifier",
            Self::FileList => "file_list",
            Self::SqlEditor => "sql_editor",
            Self::FileOperation => "file_operation",
            Self::Global => "global",
        }
    }
}

/// A normalized sequence of keypresses (one element per atom).
///
/// Internally a `Vec<KeyAtom>` so chord sequences (`gg`, `,dd`,
/// `ctrl+x ctrl+s`) are first-class.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeySequence(pub Vec<KeyAtom>);

/// One key event within a sequence: code + active modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyAtom {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyAtom {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    /// Build an atom from a live `crossterm::event::KeyEvent`.
    ///
    /// We strip `KEYPAD` and `KEYPAD_BEGIN` style flags that the platform
    /// might inject so user-typed atoms compare cleanly against parsed atoms.
    pub fn from_event(ev: crossterm::event::KeyEvent) -> Self {
        Self {
            code: ev.code,
            modifiers: normalize_modifiers(ev.modifiers, ev.code),
        }
    }
}

impl KeySequence {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Errors returned from [`parse_sequence`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyParseError {
    /// The input was empty.
    Empty,
    /// A modifier prefix was given but no key followed (e.g. `ctrl+`).
    DanglingModifier,
    /// A `<...>` form did not close (e.g. `<esc`).
    UnclosedBracket,
    /// A `<name>` reserved key was unrecognised.
    UnknownReservedKey(String),
    /// A modifier name (in `name+key`) was unrecognised.
    UnknownModifier(String),
    /// The resulting sequence had zero atoms.
    NoAtoms,
}

impl std::fmt::Display for KeyParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyParseError::Empty => write!(f, "empty key sequence"),
            KeyParseError::DanglingModifier => write!(f, "modifier with no key"),
            KeyParseError::UnclosedBracket => write!(f, "unclosed `<…>` reserved key"),
            KeyParseError::UnknownReservedKey(name) => {
                write!(f, "unknown reserved key `<{}>`", name)
            }
            KeyParseError::UnknownModifier(name) => write!(f, "unknown modifier `{}`", name),
            KeyParseError::NoAtoms => write!(f, "key sequence produced no atoms"),
        }
    }
}

impl std::error::Error for KeyParseError {}

/// Parse a key-sequence string into a [`KeySequence`].
///
/// See the module docs for the full syntax.
pub fn parse_sequence(s: &str) -> Result<KeySequence, KeyParseError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(KeyParseError::Empty);
    }

    // Whitespace separates atom-groups (so `ctrl+x ctrl+s` works). Within a
    // group, a modifier prefix consumes the rest of the group; otherwise
    // each char becomes its own atom (chord like `gg` or `,dd`).
    let mut atoms: Vec<KeyAtom> = Vec::new();
    for word in s.split_whitespace() {
        parse_word(word, &mut atoms)?;
    }
    if atoms.is_empty() {
        return Err(KeyParseError::NoAtoms);
    }
    Ok(KeySequence(atoms))
}

/// Parse a whitespace-free fragment, appending one or more atoms. The
/// fragment may be:
///
/// - A bracketed reserved form: `<esc>`, `<f2>`
/// - A modifier-prefixed atom: `ctrl+s`, `ctrl+shift+f`, `ctrl+<enter>`
/// - A chord of single chars / reserveds: `gg`, `,dd`, `<space>j`, `g~`
fn parse_word(word: &str, atoms: &mut Vec<KeyAtom>) -> Result<(), KeyParseError> {
    if word.is_empty() {
        return Err(KeyParseError::Empty);
    }
    let bytes = word.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '<' {
            // Treat `<` as opening a reserved-key form only if the next
            // char is a name char (ASCII letter). For other cases the `<`
            // is a literal — covers `<<` chords and the standalone `<` key.
            //
            // If the next char looks name-like, a closing `>` is required;
            // otherwise we emit an UnclosedBracket error so typos of
            // `<esc` get caught.
            let looks_like_name_start = bytes
                .get(i + 1)
                .map(|b| (*b as char).is_ascii_alphabetic())
                .unwrap_or(false);
            if looks_like_name_start {
                let close_off = word[i + 1..]
                    .find('>')
                    .ok_or(KeyParseError::UnclosedBracket)?;
                let name = &word[i + 1..i + 1 + close_off];
                let code = parse_reserved_key(name)
                    .ok_or_else(|| KeyParseError::UnknownReservedKey(name.to_string()))?;
                atoms.push(KeyAtom::new(code, KeyModifiers::NONE));
                i += 1 + close_off + 1;
                continue;
            }
            // Literal `<` — fall through to single-char emission below.
        }
        // Look ahead for a modifier prefix: a run of ASCII letters followed
        // by `+`. If the run is a known modifier name, consume the rest of
        // the word as a single modifier atom. If the run is followed by `+`
        // but isn't a known modifier, error — the user almost certainly
        // meant a modifier (writing `gg+x` as a chord is ill-formed).
        if c.is_ascii_alphabetic() {
            let mut j = i;
            while j < bytes.len() && (bytes[j] as char).is_ascii_alphabetic() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] as char == '+' {
                let prefix = &word[i..j];
                if modifier_from_name(prefix).is_some() {
                    let atom = parse_modifier_atom(&word[i..])?;
                    atoms.push(atom);
                    return Ok(());
                }
                return Err(KeyParseError::UnknownModifier(prefix.to_string()));
            }
        }
        // Single char — emit as its own atom.
        atoms.push(parse_single_char_atom(c)?);
        i += c.len_utf8();
    }
    Ok(())
}

/// Build a single-char atom (auto-lifts `Shift` for ASCII uppercase).
fn parse_single_char_atom(c: char) -> Result<KeyAtom, KeyParseError> {
    let mut modifiers = KeyModifiers::NONE;
    if c.is_ascii_uppercase() {
        modifiers |= KeyModifiers::SHIFT;
    }
    Ok(KeyAtom::new(KeyCode::Char(c), modifiers))
}

/// Map a modifier-name token (`ctrl`, `shift`, …) to its bitflag.
fn modifier_from_name(name: &str) -> Option<KeyModifiers> {
    Some(match name.to_ascii_lowercase().as_str() {
        "ctrl" | "control" => KeyModifiers::CONTROL,
        "shift" => KeyModifiers::SHIFT,
        "alt" | "meta" | "option" => KeyModifiers::ALT,
        "super" | "cmd" | "command" | "win" => KeyModifiers::SUPER,
        _ => return None,
    })
}

/// Parse a `mod+key` (or `mod+mod+key`) atom.
fn parse_modifier_atom(s: &str) -> Result<KeyAtom, KeyParseError> {
    let parts: Vec<&str> = s.split('+').collect();
    if parts.len() < 2 {
        return Err(KeyParseError::DanglingModifier);
    }
    let key_part = parts.last().expect("len ≥ 2");
    if key_part.is_empty() {
        return Err(KeyParseError::DanglingModifier);
    }
    let mut modifiers = KeyModifiers::NONE;
    for part in &parts[..parts.len() - 1] {
        modifiers |= modifier_from_name(part)
            .ok_or_else(|| KeyParseError::UnknownModifier(part.to_string()))?;
    }
    // Resolve the key part: either a reserved <name> form or a single char.
    let code = if key_part.starts_with('<') && key_part.ends_with('>') && key_part.len() >= 3 {
        let inner = &key_part[1..key_part.len() - 1];
        parse_reserved_key(inner)
            .ok_or_else(|| KeyParseError::UnknownReservedKey(inner.to_string()))?
    } else if key_part.chars().count() == 1 {
        // With explicit modifiers, the key letter is taken literally (case
        // insensitive). `Ctrl+S` and `ctrl+s` mean the same thing; for
        // `Ctrl+Shift+S` the user must spell `shift+` explicitly.
        let c = key_part.chars().next().unwrap();
        KeyCode::Char(c.to_ascii_lowercase())
    } else {
        // Multi-char without `<…>` — try lowercase reserved name (e.g.
        // `ctrl+esc`).
        parse_reserved_key(key_part)
            .ok_or_else(|| KeyParseError::UnknownReservedKey(key_part.to_string()))?
    };
    Ok(KeyAtom::new(code, modifiers))
}

/// Map a reserved-key name (the contents of `<…>`) to its [`KeyCode`].
/// Returns `None` for unknown names.
pub fn parse_reserved_key(name: &str) -> Option<KeyCode> {
    let n = name.to_ascii_lowercase();
    match n.as_str() {
        "esc" | "escape" => Some(KeyCode::Esc),
        "enter" | "return" | "ret" => Some(KeyCode::Enter),
        "tab" => Some(KeyCode::Tab),
        "bs" | "backspace" => Some(KeyCode::Backspace),
        "space" | "spc" => Some(KeyCode::Char(' ')),
        "del" | "delete" => Some(KeyCode::Delete),
        "ins" | "insert" => Some(KeyCode::Insert),
        "up" | "arrow_up" => Some(KeyCode::Up),
        "down" | "arrow_down" => Some(KeyCode::Down),
        "left" | "arrow_left" => Some(KeyCode::Left),
        "right" | "arrow_right" => Some(KeyCode::Right),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "pgup" | "pageup" | "page_up" => Some(KeyCode::PageUp),
        "pgdn" | "pgdown" | "pagedown" | "page_down" => Some(KeyCode::PageDown),
        "lt" => Some(KeyCode::Char('<')),
        "gt" => Some(KeyCode::Char('>')),
        s if s.starts_with('f') => {
            let n = s[1..].parse::<u8>().ok()?;
            if (1..=12).contains(&n) {
                Some(KeyCode::F(n))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Crossterm sometimes reports SHIFT for printable uppercase chars — keep
/// our atoms consistent so `K` (parsed) and a real keypress of Shift+K
/// compare equal.
fn normalize_modifiers(mods: KeyModifiers, code: KeyCode) -> KeyModifiers {
    let mut m = mods;
    if let KeyCode::Char(c) = code {
        if c.is_ascii_uppercase() {
            m |= KeyModifiers::SHIFT;
        }
    }
    m
}

// ─── Keymap loader ───────────────────────────────────────────────────────

/// Compiled per-scope keymap: a [`KeySequence`] resolves to an [`Action`].
///
/// Built from a [`KeymapToml`] (or directly via [`Keymap::default`] which
/// loads the baked-in vim preset).
///
/// In addition to bindings, we track *explicit unbinds* — keys the user
/// set to the empty string (`"i" = ""`). These distinguish "never bound"
/// from "actively suppressed" so the dispatcher knows to skip the legacy
/// fallback when an unbind is in effect.
#[derive(Debug, Clone, Default)]
pub struct Keymap {
    bindings: HashMap<KeymapScope, HashMap<KeySequence, Action>>,
    unbinds: HashMap<KeymapScope, std::collections::HashSet<KeySequence>>,
}

/// Result of looking up a key sequence in the active scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookupResult {
    /// The sequence resolves to a complete action.
    Action(Action),
    /// No exact match, but at least one binding starts with this prefix
    /// (e.g. user pressed `g` and `gg`/`gj`/`gk` all start that way).
    /// Caller should keep accumulating keys until the chord completes.
    PartialChord,
    /// User explicitly unbound this sequence in their `keys.toml`. The
    /// dispatcher should NOT fall through to the legacy match arm — the
    /// user's intent is "this key does nothing".
    ExplicitlyUnbound,
    /// No match and no longer chord starts with this sequence.
    Unbound,
}

impl Keymap {
    /// Construct a keymap from a parsed TOML document plus the baked-in vim
    /// fallback (used when `[meta] inherit` is unset or set to `"vim"`).
    pub fn from_toml(toml: &KeymapToml, warnings: &mut Vec<String>) -> Self {
        let mut map = Self::default();
        // 1. Layer the inheritance base first.
        let inherit = toml
            .meta
            .as_ref()
            .and_then(|m| m.inherit.as_deref())
            .unwrap_or("vim");
        if inherit != "none" {
            map.merge(&Self::baked_in_preset(inherit, warnings));
        }
        // 2. Apply user overrides on top.
        map.merge_toml(toml, warnings);
        map
    }

    /// Look up a sequence in the given scope, falling back to `Global`.
    ///
    /// Returns:
    /// - `Action(a)` — the sequence is bound to action `a`
    /// - `PartialChord` — a longer chord starts with these atoms
    /// - `ExplicitlyUnbound` — the user set this sequence to `""`, so the
    ///   dispatcher must NOT fall through to the legacy handler
    /// - `Unbound` — no binding and no chord prefix; caller may fall back
    pub fn lookup(&self, scope: KeymapScope, seq: &KeySequence) -> LookupResult {
        if let Some(scoped) = self.bindings.get(&scope) {
            if let Some(action) = scoped.get(seq) {
                return LookupResult::Action(*action);
            }
            if scoped.keys().any(|k| starts_with(k, seq)) {
                return LookupResult::PartialChord;
            }
        }
        if scope != KeymapScope::Global {
            if let Some(global) = self.bindings.get(&KeymapScope::Global) {
                if let Some(action) = global.get(seq) {
                    return LookupResult::Action(*action);
                }
                if global.keys().any(|k| starts_with(k, seq)) {
                    return LookupResult::PartialChord;
                }
            }
        }
        // Check explicit unbinds *after* normal lookups so a binding
        // re-added after an unbind still wins.
        if let Some(set) = self.unbinds.get(&scope) {
            if set.contains(seq) {
                return LookupResult::ExplicitlyUnbound;
            }
        }
        if scope != KeymapScope::Global {
            if let Some(set) = self.unbinds.get(&KeymapScope::Global) {
                if set.contains(seq) {
                    return LookupResult::ExplicitlyUnbound;
                }
            }
        }
        LookupResult::Unbound
    }

    /// Insert a single binding (programmatic API used by tests + presets).
    pub fn bind(&mut self, scope: KeymapScope, seq: KeySequence, action: Action) {
        self.bindings.entry(scope).or_default().insert(seq, action);
    }

    /// Remove a binding (returns whether anything was removed).
    pub fn unbind(&mut self, scope: KeymapScope, seq: &KeySequence) -> bool {
        self.bindings
            .get_mut(&scope)
            .map(|m| m.remove(seq).is_some())
            .unwrap_or(false)
    }

    /// Iterate over every (scope, sequence, action) — handy for `:keys`.
    pub fn iter(&self) -> impl Iterator<Item = (KeymapScope, &KeySequence, Action)> {
        self.bindings
            .iter()
            .flat_map(|(scope, map)| map.iter().map(move |(k, a)| (*scope, k, *a)))
    }

    /// The default vim keymap, baked into the binary.
    pub fn vim_default() -> Self {
        let mut warnings = Vec::new();
        let toml: KeymapToml = toml::from_str(VIM_DEFAULT_TOML).expect("baked vim.toml must parse");
        let mut map = Self::default();
        map.merge_toml(&toml, &mut warnings);
        debug_assert!(
            warnings.is_empty(),
            "baked vim.toml produced warnings: {:?}",
            warnings
        );
        map
    }

    /// Look up a baked-in preset by name. Currently only `vim` is baked.
    fn baked_in_preset(name: &str, warnings: &mut Vec<String>) -> Self {
        match name {
            "vim" => Self::vim_default(),
            other => {
                warnings.push(format!(
                    "[meta].inherit = {:?} is unknown — falling back to vim",
                    other
                ));
                Self::vim_default()
            }
        }
    }

    fn merge(&mut self, other: &Self) {
        for (scope, map) in &other.bindings {
            let dest = self.bindings.entry(*scope).or_default();
            for (k, v) in map {
                dest.insert(k.clone(), *v);
            }
        }
    }

    fn merge_toml(&mut self, toml: &KeymapToml, warnings: &mut Vec<String>) {
        for (scope, table) in toml.scopes() {
            for (key_str, action_str) in table {
                if action_str.is_empty() {
                    // empty value → explicit unbind. Remove any inherited
                    // binding, AND record the unbind so the dispatcher
                    // knows to suppress the legacy fallback.
                    if let Ok(seq) = parse_sequence(key_str) {
                        self.unbind(scope, &seq);
                        self.unbinds.entry(scope).or_default().insert(seq);
                    }
                    continue;
                }
                let seq = match parse_sequence(key_str) {
                    Ok(s) => s,
                    Err(e) => {
                        warnings.push(format!("[{}] {}: {}", scope.toml_key(), key_str, e));
                        continue;
                    }
                };
                let action = match Action::from_name(action_str) {
                    Some(a) => a,
                    None => {
                        warnings.push(format!(
                            "[{}] {}: unknown action {:?}",
                            scope.toml_key(),
                            key_str,
                            action_str
                        ));
                        continue;
                    }
                };
                self.bind(scope, seq, action);
            }
        }
    }
}

fn starts_with(longer: &KeySequence, prefix: &KeySequence) -> bool {
    if prefix.len() > longer.len() || prefix.len() == longer.len() {
        return false;
    }
    longer.0[..prefix.len()] == prefix.0[..]
}

/// TOML schema for `keys.toml`.
///
/// Each section is a string-to-string table: keys are sequence syntax,
/// values are action IDs. An empty-string value unbinds the key.
#[derive(Debug, Default, Deserialize)]
pub struct KeymapToml {
    #[serde(default)]
    pub meta: Option<KeymapMeta>,
    #[serde(default)]
    pub global: HashMap<String, String>,
    #[serde(default)]
    pub normal: HashMap<String, String>,
    #[serde(default)]
    pub insert: HashMap<String, String>,
    #[serde(default)]
    pub visual: HashMap<String, String>,
    #[serde(default)]
    pub command: HashMap<String, String>,
    #[serde(default)]
    pub search: HashMap<String, String>,
    #[serde(default)]
    pub magnifier: HashMap<String, String>,
    #[serde(default)]
    pub file_list: HashMap<String, String>,
    #[serde(default)]
    pub sql_editor: HashMap<String, String>,
    #[serde(default)]
    pub file_operation: HashMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct KeymapMeta {
    /// Name of the baked-in preset to inherit from. `"vim"` by default;
    /// `"none"` skips inheritance entirely.
    pub inherit: Option<String>,
}

impl KeymapToml {
    fn scopes(&self) -> Vec<(KeymapScope, &HashMap<String, String>)> {
        vec![
            (KeymapScope::Global, &self.global),
            (KeymapScope::Normal, &self.normal),
            (KeymapScope::Insert, &self.insert),
            (KeymapScope::Visual, &self.visual),
            (KeymapScope::Command, &self.command),
            (KeymapScope::Search, &self.search),
            (KeymapScope::Magnifier, &self.magnifier),
            (KeymapScope::FileList, &self.file_list),
            (KeymapScope::SqlEditor, &self.sql_editor),
            (KeymapScope::FileOperation, &self.file_operation),
        ]
    }
}

/// Load a keymap from `path` (returns an empty toml if the file is missing).
pub fn load_toml_file(path: &Path) -> Result<Option<KeymapToml>, String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("could not read {}: {}", path.display(), e)),
    };
    toml::from_str(&content)
        .map(Some)
        .map_err(|e| format!("invalid TOML in {}: {}", path.display(), e.message()))
}

/// The baked-in vim keymap. Defines every default binding lazycsv ships
/// with. Edit `keymaps/vim.toml` (the source of truth) and rerun
/// `cargo build` — `include_str!` re-bakes it.
pub const VIM_DEFAULT_TOML: &str = include_str!("../../keymaps/vim.toml");

/// Display path for `keys.toml` (used in `:keys` status output and docs).
/// Returns `~/.config/lazycsv/keys.toml`-style display strings.
pub fn shell_keys_hint() -> Option<String> {
    super::dirs_path().map(|p| {
        let path = p.join("keys.toml");
        // Substitute $HOME with `~` for friendlier display.
        if let Ok(home) = std::env::var("HOME") {
            if let Ok(stripped) = path.strip_prefix(&home) {
                return format!("~/{}", stripped.display());
            }
        }
        path.display().to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom_char(c: char) -> KeyAtom {
        KeyAtom::new(KeyCode::Char(c), KeyModifiers::NONE)
    }

    fn atom_char_mod(c: char, m: KeyModifiers) -> KeyAtom {
        KeyAtom::new(KeyCode::Char(c), m)
    }

    #[test]
    fn empty_input_errors() {
        assert!(matches!(parse_sequence(""), Err(KeyParseError::Empty)));
        assert!(matches!(parse_sequence("   "), Err(KeyParseError::Empty)));
    }

    #[test]
    fn single_letter() {
        let seq = parse_sequence("j").unwrap();
        assert_eq!(seq.0, vec![atom_char('j')]);
    }

    #[test]
    fn uppercase_lifts_shift() {
        let seq = parse_sequence("J").unwrap();
        assert_eq!(seq.0, vec![atom_char_mod('J', KeyModifiers::SHIFT)]);
    }

    #[test]
    fn chord_gg() {
        let seq = parse_sequence("gg").unwrap();
        assert_eq!(seq.0, vec![atom_char('g'), atom_char('g')]);
    }

    #[test]
    fn chord_with_punctuation() {
        let seq = parse_sequence(",dd").unwrap();
        assert_eq!(seq.0, vec![atom_char(','), atom_char('d'), atom_char('d')]);
    }

    #[test]
    fn ctrl_modifier() {
        let seq = parse_sequence("ctrl+s").unwrap();
        assert_eq!(seq.0, vec![atom_char_mod('s', KeyModifiers::CONTROL)]);
    }

    #[test]
    fn ctrl_shift_modifier() {
        let seq = parse_sequence("ctrl+shift+f").unwrap();
        assert_eq!(
            seq.0,
            vec![atom_char_mod(
                'f',
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )]
        );
    }

    #[test]
    fn reserved_esc() {
        let seq = parse_sequence("<esc>").unwrap();
        assert_eq!(seq.0, vec![KeyAtom::new(KeyCode::Esc, KeyModifiers::NONE)]);
    }

    #[test]
    fn reserved_f2() {
        let seq = parse_sequence("<f2>").unwrap();
        assert_eq!(seq.0, vec![KeyAtom::new(KeyCode::F(2), KeyModifiers::NONE)]);
    }

    #[test]
    fn modifier_with_reserved() {
        let seq = parse_sequence("ctrl+<enter>").unwrap();
        assert_eq!(
            seq.0,
            vec![KeyAtom::new(KeyCode::Enter, KeyModifiers::CONTROL)]
        );
    }

    #[test]
    fn whitespace_separated_chord() {
        let seq = parse_sequence("ctrl+x ctrl+s").unwrap();
        assert_eq!(
            seq.0,
            vec![
                atom_char_mod('x', KeyModifiers::CONTROL),
                atom_char_mod('s', KeyModifiers::CONTROL),
            ]
        );
    }

    #[test]
    fn reserved_then_letter() {
        let seq = parse_sequence("<space>j").unwrap();
        assert_eq!(
            seq.0,
            vec![
                KeyAtom::new(KeyCode::Char(' '), KeyModifiers::NONE),
                atom_char('j'),
            ]
        );
    }

    #[test]
    fn dangling_modifier_errors() {
        assert!(matches!(
            parse_sequence("ctrl+"),
            Err(KeyParseError::DanglingModifier)
        ));
    }

    #[test]
    fn unclosed_bracket_errors() {
        assert!(matches!(
            parse_sequence("<esc"),
            Err(KeyParseError::UnclosedBracket)
        ));
    }

    #[test]
    fn unknown_reserved_errors() {
        assert!(matches!(
            parse_sequence("<wat>"),
            Err(KeyParseError::UnknownReservedKey(_))
        ));
    }

    #[test]
    fn unknown_modifier_errors() {
        assert!(matches!(
            parse_sequence("hyper+s"),
            Err(KeyParseError::UnknownModifier(_))
        ));
    }

    #[test]
    fn from_event_normalizes_uppercase() {
        let ev = crossterm::event::KeyEvent::new(KeyCode::Char('K'), KeyModifiers::NONE);
        let atom = KeyAtom::from_event(ev);
        assert!(atom.modifiers.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn case_insensitive_reserved_names() {
        assert_eq!(
            parse_sequence("<ESC>").unwrap(),
            parse_sequence("<esc>").unwrap()
        );
    }

    #[test]
    fn case_insensitive_modifier_names() {
        assert_eq!(
            parse_sequence("Ctrl+S").unwrap(),
            parse_sequence("ctrl+s").unwrap()
        );
    }

    #[test]
    fn comma_p_chord() {
        let seq = parse_sequence(",P").unwrap();
        assert_eq!(
            seq.0,
            vec![atom_char(','), atom_char_mod('P', KeyModifiers::SHIFT)]
        );
    }

    // ── Keymap tests ──────────────────────────────────────────────

    #[test]
    fn vim_default_loads_without_warnings() {
        // Just constructing the default exercises the bake-in parser.
        let _km = Keymap::vim_default();
    }

    #[test]
    fn vim_default_binds_basic_normal_keys() {
        let km = Keymap::vim_default();
        let lookup = |k: &str| km.lookup(KeymapScope::Normal, &parse_sequence(k).unwrap());
        assert_eq!(lookup("j"), LookupResult::Action(Action::CursorDown));
        assert_eq!(lookup("k"), LookupResult::Action(Action::CursorUp));
        assert_eq!(lookup("h"), LookupResult::Action(Action::CursorLeft));
        assert_eq!(lookup("l"), LookupResult::Action(Action::CursorRight));
        assert_eq!(lookup("dd"), LookupResult::Action(Action::RowDelete));
        assert_eq!(lookup(",dd"), LookupResult::Action(Action::ColDelete));
    }

    #[test]
    fn partial_chord_returns_partial() {
        let km = Keymap::vim_default();
        // `g` alone is a prefix of `gg`, `gj`, `gk`, `g~`, `g.` — so a lone
        // `g` press should report PartialChord.
        let lookup = km.lookup(KeymapScope::Normal, &parse_sequence("g").unwrap());
        assert_eq!(lookup, LookupResult::PartialChord);
    }

    #[test]
    fn unbound_key_reports_unbound() {
        let km = Keymap::vim_default();
        let lookup = km.lookup(KeymapScope::Normal, &parse_sequence("Z").unwrap());
        assert_eq!(lookup, LookupResult::Unbound);
    }

    #[test]
    fn user_override_replaces_inherited_binding() {
        let toml: KeymapToml = toml::from_str(
            r#"
            [meta]
            inherit = "vim"

            [normal]
            "j" = "cursor_up"
        "#,
        )
        .unwrap();
        let mut warnings = Vec::new();
        let km = Keymap::from_toml(&toml, &mut warnings);
        assert!(warnings.is_empty(), "warnings: {:?}", warnings);
        let lookup = km.lookup(KeymapScope::Normal, &parse_sequence("j").unwrap());
        assert_eq!(lookup, LookupResult::Action(Action::CursorUp));
        // k stays inherited
        let lookup_k = km.lookup(KeymapScope::Normal, &parse_sequence("k").unwrap());
        assert_eq!(lookup_k, LookupResult::Action(Action::CursorUp));
    }

    #[test]
    fn empty_value_unbinds_inherited_binding() {
        let toml: KeymapToml = toml::from_str(
            r#"
            [normal]
            "dd" = ""
        "#,
        )
        .unwrap();
        let mut warnings = Vec::new();
        let km = Keymap::from_toml(&toml, &mut warnings);
        assert!(warnings.is_empty());
        let lookup = km.lookup(KeymapScope::Normal, &parse_sequence("dd").unwrap());
        assert_eq!(
            lookup,
            LookupResult::ExplicitlyUnbound,
            "empty value should be tracked as an explicit unbind"
        );
    }

    #[test]
    fn unknown_action_warns_but_continues() {
        let toml: KeymapToml = toml::from_str(
            r#"
            [meta]
            inherit = "none"

            [normal]
            "x" = "this_action_does_not_exist"
            "j" = "cursor_down"
        "#,
        )
        .unwrap();
        let mut warnings = Vec::new();
        let km = Keymap::from_toml(&toml, &mut warnings);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("unknown action"));
        // Other binding still applied
        let lookup = km.lookup(KeymapScope::Normal, &parse_sequence("j").unwrap());
        assert_eq!(lookup, LookupResult::Action(Action::CursorDown));
    }

    #[test]
    fn malformed_key_warns_with_section_qualifier() {
        let toml: KeymapToml = toml::from_str(
            r#"
            [meta]
            inherit = "none"

            [normal]
            "ctrl+" = "cursor_down"
        "#,
        )
        .unwrap();
        let mut warnings = Vec::new();
        let _km = Keymap::from_toml(&toml, &mut warnings);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("[normal]"));
        assert!(warnings[0].contains("ctrl+"));
    }

    #[test]
    fn global_scope_falls_back_when_mode_specific_missing() {
        let toml: KeymapToml = toml::from_str(
            r#"
            [meta]
            inherit = "none"

            [global]
            "?" = "toggle_help"
        "#,
        )
        .unwrap();
        let mut warnings = Vec::new();
        let km = Keymap::from_toml(&toml, &mut warnings);
        assert!(warnings.is_empty());
        let lookup = km.lookup(KeymapScope::Insert, &parse_sequence("?").unwrap());
        assert_eq!(lookup, LookupResult::Action(Action::ToggleHelp));
    }

    #[test]
    fn meta_inherit_none_skips_baked_keymap() {
        let toml: KeymapToml = toml::from_str(
            r#"
            [meta]
            inherit = "none"
        "#,
        )
        .unwrap();
        let mut warnings = Vec::new();
        let km = Keymap::from_toml(&toml, &mut warnings);
        assert!(warnings.is_empty());
        // No vim defaults loaded — `j` should be unbound.
        let lookup = km.lookup(KeymapScope::Normal, &parse_sequence("j").unwrap());
        assert_eq!(lookup, LookupResult::Unbound);
    }

    #[test]
    fn unknown_inherit_warns_and_falls_back_to_vim() {
        let toml: KeymapToml = toml::from_str(
            r#"
            [meta]
            inherit = "definitely-not-a-preset"
        "#,
        )
        .unwrap();
        let mut warnings = Vec::new();
        let km = Keymap::from_toml(&toml, &mut warnings);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("inherit"));
        // vim defaults still apply
        let lookup = km.lookup(KeymapScope::Normal, &parse_sequence("j").unwrap());
        assert_eq!(lookup, LookupResult::Action(Action::CursorDown));
    }

    // ── Shipped preset round-trip tests ──────────────────────────────

    fn load_preset(content: &str) -> (Keymap, Vec<String>) {
        let toml: KeymapToml = toml::from_str(content).expect("preset must parse");
        let mut warnings = Vec::new();
        let km = Keymap::from_toml(&toml, &mut warnings);
        (km, warnings)
    }

    #[test]
    fn emacs_preset_loads_without_warnings() {
        let content = include_str!("../../keymaps/emacs.toml");
        let (_km, warnings) = load_preset(content);
        assert!(
            warnings.is_empty(),
            "emacs.toml produced warnings: {:?}",
            warnings
        );
    }

    #[test]
    fn excel_preset_loads_without_warnings() {
        let content = include_str!("../../keymaps/excel.toml");
        let (_km, warnings) = load_preset(content);
        assert!(
            warnings.is_empty(),
            "excel.toml produced warnings: {:?}",
            warnings
        );
    }

    #[test]
    fn emacs_preset_binds_ctrl_n_to_cursor_down() {
        let content = include_str!("../../keymaps/emacs.toml");
        let (km, _) = load_preset(content);
        let lookup = km.lookup(KeymapScope::Normal, &parse_sequence("ctrl+n").unwrap());
        assert_eq!(lookup, LookupResult::Action(Action::CursorDown));
    }

    #[test]
    fn excel_preset_binds_arrows_and_f2() {
        let content = include_str!("../../keymaps/excel.toml");
        let (km, _) = load_preset(content);
        assert_eq!(
            km.lookup(KeymapScope::Normal, &parse_sequence("<down>").unwrap()),
            LookupResult::Action(Action::CursorDown)
        );
        assert_eq!(
            km.lookup(KeymapScope::Normal, &parse_sequence("<f2>").unwrap()),
            LookupResult::Action(Action::CellReplaceF2)
        );
        assert_eq!(
            km.lookup(KeymapScope::Normal, &parse_sequence("ctrl+s").unwrap()),
            LookupResult::Action(Action::Save)
        );
    }

    #[test]
    fn excel_preset_unbinds_vim_letter_modes() {
        let content = include_str!("../../keymaps/excel.toml");
        let (km, _) = load_preset(content);
        // `i`, `a`, `v` are unbound (empty value) in excel.toml so the
        // user can type them as cell content.
        assert_eq!(
            km.lookup(KeymapScope::Normal, &parse_sequence("i").unwrap()),
            LookupResult::ExplicitlyUnbound
        );
        assert_eq!(
            km.lookup(KeymapScope::Normal, &parse_sequence("a").unwrap()),
            LookupResult::ExplicitlyUnbound
        );
        assert_eq!(
            km.lookup(KeymapScope::Normal, &parse_sequence("v").unwrap()),
            LookupResult::ExplicitlyUnbound
        );
    }
}
