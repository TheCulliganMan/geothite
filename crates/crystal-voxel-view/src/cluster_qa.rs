//! Bevy's supported CPU clustering backend avoids empty GPU clustering passes
//! in the native game's predominantly sunlit scenes. The GPU backend remains
//! available for same-binary comparisons with CRYSTAL_CLUSTERING=gpu.
use bevy::{light::cluster::GlobalClusterSettings, prelude::*};

pub(super) fn register(app: &mut App) {
    match std::env::var("CRYSTAL_CLUSTERING").as_deref() {
        Ok("cpu") | Err(std::env::VarError::NotPresent) => {},
        _ => return,
    }
    // PbrPlugin creates this resource in finish(). Startup precedes the first
    // light assignment and render extraction, retaining all device limits and
    // support flags while selecting the engine's existing CPU implementation.
    app.add_systems(Startup, select_cpu_backend);
}

fn select_cpu_backend(mut settings: ResMut<GlobalClusterSettings>) {
    let was_gpu = settings.gpu_clustering.is_some();
    settings.gpu_clustering = None;
    drop(crate::loading_trace::span("render.cluster_backend", || serde_json::json!({
        "backend":"cpu", "was_gpu":was_gpu,
        "storage_buffers":settings.supports_storage_buffers,
        "clustered_decals":settings.clustered_decals_are_usable,
    })));
}
