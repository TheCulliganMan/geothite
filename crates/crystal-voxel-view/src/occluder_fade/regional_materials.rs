//! Keep dormant regional foliage from allocating a private GPU material.

use std::sync::OnceLock;

/// Startup-only native control preserves the old allocation policy for QA.
pub(super) fn deferred() -> bool {
    static DEFERRED: OnceLock<bool> = OnceLock::new();
    *DEFERRED.get_or_init(|| {
        cfg!(target_arch = "wasm32")
            || std::env::var("CRYSTAL_REGIONAL_MATERIALS").as_deref() != Ok("eager")
    })
}
