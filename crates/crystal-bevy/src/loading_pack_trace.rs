/// Adapt clock-free asset loader stages to the shared loading waterfall.
pub fn observe_stage(name: &'static str, begin: bool) {
    thread_local! {
        static SPANS: std::cell::RefCell<Vec<(&'static str, Option<crystal_voxel_view::loading_trace::Span>)>> = const { std::cell::RefCell::new(Vec::new()) };
    }
    if begin {
        let span = crystal_voxel_view::loading_trace::span(name, || serde_json::json!({}));
        SPANS.with_borrow_mut(|spans| spans.push((name, span)));
    } else {
        let completed = SPANS.with_borrow_mut(Vec::pop);
        debug_assert_eq!(completed.as_ref().map(|(name, _)| *name), Some(name));
        // Drop outside the RefCell borrow; observers may reenter safely.
        drop(completed);
    }
}
