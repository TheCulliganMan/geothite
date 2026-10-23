//! Narrow native diagnostics without forcing the detailed render graph barriers.
#[cfg(not(target_arch = "wasm32"))]
fn prefix() -> Option<&'static str> {
    static PREFIX: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    PREFIX
        .get_or_init(|| {
            std::env::var("CRYSTAL_LOADING_TRACE_PREFIX")
                .ok()
                .filter(|prefix| !prefix.is_empty())
        })
        .as_deref()
}

pub(super) fn allows(name: &str) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        prefix().is_none_or(|prefix| name.starts_with(prefix))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = name;
        true
    }
}

pub(super) fn full_graph() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        prefix().is_none()
    }
    #[cfg(target_arch = "wasm32")]
    {
        true
    }
}
