//! Opt-in CPU pass timings for native rendering; never changes scene quality.
use bevy::{diagnostic::DiagnosticsStore, prelude::*, render::diagnostic::RenderDiagnosticsPlugin};

pub(super) fn register(app: &mut App) {
    if std::env::var("CRYSTAL_RENDER_PASS_TIMING").as_deref() != Ok("on") {
        return;
    }
    if !app.is_plugin_added::<RenderDiagnosticsPlugin>() {
        app.add_plugins(RenderDiagnosticsPlugin);
    }
    app.add_systems(Update, report);
}

fn report(time: Res<Time<Real>>, diagnostics: Res<DiagnosticsStore>, mut next: Local<f64>) {
    let now = time.elapsed_secs_f64();
    if now < *next {
        return;
    }
    *next = now + 2.0;
    let mut passes: Vec<_> = diagnostics
        .iter()
        .filter_map(|diagnostic| {
            let path = diagnostic.path().as_str();
            if path.starts_with("render/") && path.ends_with("/elapsed_cpu") {
                diagnostic.value().map(|ms| (path, ms))
            } else {
                None
            }
        })
        .collect();
    passes.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));
    for (path, ms) in passes.into_iter().take(12) {
        eprintln!("render_pass_cpu path={path} ms={ms:.3}");
    }
}
