//! Vim-style macro recording and playback.
//!
//! Stores raw `KeyEvent`s per register (a–z). Replay feeds events back through
//! the normal input dispatch. The recording hook in `App::handle_key` skips
//! events while a replay is in progress so macros never re-record themselves.

use crossterm::event::KeyEvent;
use std::collections::HashMap;

/// Hard cap on keys recorded into a single macro to prevent runaway loops.
const MAX_MACRO_LEN: usize = 10_000;

/// Hard cap on replay nesting depth (one macro calling another).
const MAX_REPLAY_DEPTH: usize = 8;

/// Per-app macro state: registers, current recording target, replay guard.
#[derive(Debug, Default)]
pub struct MacroState {
    /// Stored macros keyed by register name (a–z).
    registers: HashMap<char, Vec<KeyEvent>>,

    /// If recording, the register being recorded into.
    recording: Option<char>,

    /// Live buffer for the current recording.
    current: Vec<KeyEvent>,

    /// Last register played back (for `@@`).
    last_played: Option<char>,

    /// Depth counter — non-zero means we are currently replaying.
    replay_depth: usize,
}

impl MacroState {
    pub fn new() -> Self {
        Self::default()
    }

    /// True while a `qa` recording is in progress.
    pub fn is_recording(&self) -> bool {
        self.recording.is_some()
    }

    /// True while a macro is being replayed (recording is suppressed).
    pub fn is_replaying(&self) -> bool {
        self.replay_depth > 0
    }

    /// The register currently being recorded into, if any.
    pub fn recording_register(&self) -> Option<char> {
        self.recording
    }

    /// Begin recording into `register`. If already recording, the call is ignored.
    pub fn start_recording(&mut self, register: char) {
        if self.recording.is_some() {
            return;
        }
        self.recording = Some(register);
        self.current.clear();
    }

    /// Stop recording and store the buffer. No-op if not recording.
    /// Returns the register that was just stored.
    ///
    /// The trailing key (the `q` keypress that triggered the stop) is dropped:
    /// the recording hook in `App::handle_key` records every keypress *before*
    /// dispatch, so the stop key is always present in the buffer at this point.
    pub fn stop_recording(&mut self) -> Option<char> {
        let register = self.recording.take()?;
        let mut buffer = std::mem::take(&mut self.current);
        buffer.pop();
        self.registers.insert(register, buffer);
        Some(register)
    }

    /// Record a key. Caller must check `is_recording() && !is_replaying()`.
    pub fn record_key(&mut self, key: KeyEvent) {
        if self.recording.is_none() || self.is_replaying() {
            return;
        }
        if self.current.len() >= MAX_MACRO_LEN {
            return;
        }
        self.current.push(key);
    }

    /// Borrow a stored macro, if any.
    pub fn get(&self, register: char) -> Option<&[KeyEvent]> {
        self.registers.get(&register).map(|v| v.as_slice())
    }

    /// Borrow the macro from a previous `@<reg>` call (for `@@`).
    pub fn last_played(&self) -> Option<char> {
        self.last_played
    }

    /// Mark `register` as last played.
    pub fn set_last_played(&mut self, register: char) {
        self.last_played = Some(register);
    }

    /// Enter a replay scope. Returns false if max depth exceeded — caller should abort.
    pub fn begin_replay(&mut self) -> bool {
        if self.replay_depth >= MAX_REPLAY_DEPTH {
            return false;
        }
        self.replay_depth += 1;
        true
    }

    /// Exit a replay scope. Saturates at zero.
    pub fn end_replay(&mut self) {
        self.replay_depth = self.replay_depth.saturating_sub(1);
    }
}

/// Validate that `c` is a usable macro register (a–z).
pub fn is_valid_register(c: char) -> bool {
    c.is_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    fn k(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
    }

    #[test]
    fn record_and_retrieve() {
        // stop_recording pops the trailing key (the stop-q itself), so we record
        // three keys and expect two stored.
        let mut m = MacroState::new();
        m.start_recording('a');
        m.record_key(k('j'));
        m.record_key(k('j'));
        m.record_key(k('q'));
        m.stop_recording();
        assert_eq!(m.get('a').map(|s| s.len()), Some(2));
    }

    #[test]
    fn second_start_recording_ignored() {
        let mut m = MacroState::new();
        m.start_recording('a');
        m.record_key(k('j'));
        m.start_recording('b'); // should be a no-op while recording
        assert_eq!(m.recording_register(), Some('a'));
    }

    #[test]
    fn stop_without_start_returns_none() {
        let mut m = MacroState::new();
        assert!(m.stop_recording().is_none());
    }

    #[test]
    fn record_suppressed_during_replay() {
        let mut m = MacroState::new();
        m.start_recording('a');
        assert!(m.begin_replay());
        m.record_key(k('j')); // should be skipped
        m.end_replay();
        m.stop_recording();
        assert_eq!(m.get('a').map(|s| s.len()), Some(0));
    }

    #[test]
    fn replay_depth_limit() {
        let mut m = MacroState::new();
        for _ in 0..MAX_REPLAY_DEPTH {
            assert!(m.begin_replay());
        }
        assert!(!m.begin_replay());
    }

    #[test]
    fn last_played_tracking() {
        let mut m = MacroState::new();
        assert!(m.last_played().is_none());
        m.set_last_played('q');
        assert_eq!(m.last_played(), Some('q'));
    }

    #[test]
    fn macro_length_capped() {
        let mut m = MacroState::new();
        m.start_recording('a');
        for _ in 0..(MAX_MACRO_LEN + 50) {
            m.record_key(k('j'));
        }
        m.stop_recording(); // pops one trailing key
        assert_eq!(m.get('a').map(|s| s.len()), Some(MAX_MACRO_LEN - 1));
    }

    #[test]
    fn valid_register_check() {
        assert!(is_valid_register('a'));
        assert!(is_valid_register('z'));
        assert!(!is_valid_register('A'));
        assert!(!is_valid_register('1'));
        assert!(!is_valid_register('@'));
    }

    #[test]
    fn separate_registers_isolated() {
        let mut m = MacroState::new();
        m.start_recording('a');
        m.record_key(k('j'));
        m.record_key(k('q')); // stop key
        m.stop_recording();
        m.start_recording('b');
        m.record_key(k('k'));
        m.record_key(k('k'));
        m.record_key(k('q')); // stop key
        m.stop_recording();
        assert_eq!(m.get('a').map(|s| s.len()), Some(1));
        assert_eq!(m.get('b').map(|s| s.len()), Some(2));
    }
}
