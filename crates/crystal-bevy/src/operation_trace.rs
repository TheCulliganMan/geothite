//! One native capture owns logging from pack loading through renderer shutdown.
use std::sync::atomic::{AtomicBool, Ordering};

use bevy::{app::App, log::LogPlugin};

static STARTED: AtomicBool = AtomicBool::new(false);

/// Keep the returned owner alive until the end of the process's capture.
/// Nested callers share the already-installed subscriber. Dropping the owner
/// flushes Bevy's Chrome writer, including the closing JSON delimiter.
/// Like Bevy's LogPlugin, this supports one capture per process.
pub fn start() -> Option<App> {
    if STARTED.swap(true, Ordering::SeqCst) {
        return None;
    }
    let mut owner = App::new();
    owner.add_plugins(LogPlugin::default());
    Some(owner)
}
