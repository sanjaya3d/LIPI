//! Settings and messages shared between the keyboard hook thread and the rest of the program.
//! Part 4 (the window) uses the same switches and reads the same log.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static ENABLED: AtomicBool = AtomicBool::new(true);
static LEGACY: AtomicBool = AtomicBool::new(false);
static LOGS: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

/// How many log lines are kept before the oldest are dropped.
const MAX_LOGS: usize = 500;

/// Is Sinhala typing switched on?
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

/// Is the output legacy (FM Abhaya style) instead of Unicode?
pub fn legacy() -> bool {
    LEGACY.load(Ordering::Relaxed)
}

pub fn set_legacy(on: bool) {
    LEGACY.store(on, Ordering::Relaxed);
}

pub fn log(line: impl Into<String>) {
    if let Ok(mut logs) = LOGS.lock() {
        if logs.len() >= MAX_LOGS {
            logs.pop_front();
        }
        logs.push_back(line.into());
    }
}

/// Takes all log lines written since the last call.
pub fn take_logs() -> Vec<String> {
    match LOGS.lock() {
        Ok(mut logs) => logs.drain(..).collect(),
        Err(_) => Vec::new(),
    }
}
