//! Process-local display options. These do not alter saved preferences or command authority.
use std::sync::atomic::{AtomicBool, Ordering};

static QUIET: AtomicBool = AtomicBool::new(false);

pub(crate) fn configure(arguments: &mut Vec<String>) {
    let quiet = arguments.iter().any(|argument| argument == "--quiet");
    arguments.retain(|argument| argument != "--quiet");
    QUIET.store(quiet, Ordering::Relaxed);
}

pub(crate) fn quiet() -> bool {
    QUIET.load(Ordering::Relaxed)
}
