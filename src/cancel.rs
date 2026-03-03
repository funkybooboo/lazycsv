//! Cancellation support for long-running TUI operations.
//!
//! A background watcher thread reads raw bytes from stdin (bypassing crossterm)
//! and sets an `AtomicBool` flag when a standalone Esc keypress is detected.
//! Hot loops call `check_esc()` which is a simple flag load — no I/O overhead.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Error type returned when an operation is cancelled by the user.
#[derive(Debug)]
pub struct CancelledError;

impl std::fmt::Display for CancelledError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Operation cancelled")
    }
}

impl std::error::Error for CancelledError {}

/// Check cancellation flag every CHECK_INTERVAL iterations in hot loops.
pub const CHECK_INTERVAL: usize = 10_000;

/// Fast, zero-overhead cancellation check — just loads the atomic flag.
/// The flag is set by an [`EscWatcher`] background thread.
pub fn check_esc(cancelled: &AtomicBool) -> bool {
    cancelled.load(Ordering::Relaxed)
}

/// Background thread that watches stdin for Esc keypresses.
/// Uses raw `libc` fd operations to avoid crossterm threading issues.
pub struct EscWatcher {
    done: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl EscWatcher {
    /// Spawn a background thread that monitors stdin for standalone Esc.
    /// When Esc is detected, `cancelled` is set to `true`.
    pub fn spawn(cancelled: &Arc<AtomicBool>) -> Self {
        let done = Arc::new(AtomicBool::new(false));
        let c = Arc::clone(cancelled);
        let d = Arc::clone(&done);

        let handle = std::thread::spawn(move || {
            esc_watch_loop(&c, &d);
        });

        EscWatcher {
            done,
            handle: Some(handle),
        }
    }

    /// Stop the watcher and wait for the thread to exit.
    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        self.done.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for EscWatcher {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Read raw bytes from stdin, looking for standalone Esc (0x1B).
/// Escape sequences (arrow keys, etc.) start with 0x1B but have
/// trailing bytes within ~20ms; those are drained and ignored.
#[cfg(unix)]
fn esc_watch_loop(cancelled: &AtomicBool, done: &AtomicBool) {
    use std::os::unix::io::AsRawFd;

    let fd = std::io::stdin().as_raw_fd();

    while !done.load(Ordering::Relaxed) {
        // Wait up to 50ms for data on stdin
        let mut pfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let ret = unsafe { libc::poll(&mut pfd, 1, 50) };

        if ret <= 0 || (pfd.revents & libc::POLLIN) == 0 {
            continue;
        }

        // Read one byte
        let mut buf = [0u8; 1];
        let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, 1) };
        if n != 1 {
            continue;
        }

        if buf[0] == 0x1B {
            // Disambiguate: standalone Esc vs multi-byte escape sequence.
            // If more bytes arrive within 20ms, it's an escape sequence — drain them.
            let mut pfd2 = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };
            let ret2 = unsafe { libc::poll(&mut pfd2, 1, 20) };
            if ret2 > 0 && (pfd2.revents & libc::POLLIN) != 0 {
                // Escape sequence — drain remaining bytes
                let mut drain = [0u8; 16];
                loop {
                    let mut pfd3 = libc::pollfd {
                        fd,
                        events: libc::POLLIN,
                        revents: 0,
                    };
                    if unsafe { libc::poll(&mut pfd3, 1, 2) } <= 0 {
                        break;
                    }
                    if unsafe { libc::read(fd, drain.as_mut_ptr() as *mut libc::c_void, 16) } <= 0
                    {
                        break;
                    }
                }
            } else {
                // Standalone Esc — cancel!
                cancelled.store(true, Ordering::Relaxed);
                return;
            }
        }
        // Non-Esc byte — discard and continue
    }
}

/// Fallback for non-Unix: use crossterm's event API from the background thread.
#[cfg(not(unix))]
fn esc_watch_loop(cancelled: &AtomicBool, done: &AtomicBool) {
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use std::time::Duration;

    while !done.load(Ordering::Relaxed) {
        if event::poll(Duration::from_millis(50)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Esc {
                    cancelled.store(true, Ordering::Relaxed);
                    return;
                }
            }
        }
    }
}
