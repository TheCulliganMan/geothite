//! Opt-in native diagnostics. GPU queries report elapsed work, never displayed FPS.
use bevy::{
    diagnostic::{DiagnosticsStore, FrameCount},
    platform::time::Instant,
    prelude::*,
    render::{
        Extract, ExtractSchedule, RenderApp,
        diagnostic::{
            DiagnosticsRecorder, RecordDiagnostics, RenderDiagnosticsPlugin, resolve_encoder,
        },
        render_resource::{
            Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, WgpuFeatures,
        },
        renderer::{
            PendingCommandBuffers, RenderAdapterInfo, RenderDevice, RenderGraph,
            RenderGraphSystems, RenderQueue,
        },
    },
};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

const FRAME_MARKER: &str = "crystal_frame_id";
const FRAME_MARKER_PATH: &str = "render/crystal_frame_id";
const SAMPLE_COUNT: usize = 30;
const MAX_PENDING_IDENTITIES: usize = 512;

pub(super) fn register(app: &mut App) {
    let cpu_log = std::env::var("CRYSTAL_RENDER_PASS_TIMING").as_deref() == Ok("on");
    let output = std::env::var_os("CRYSTAL_RENDER_GPU_TRACE")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from);
    if !cpu_log && output.is_none() {
        return;
    }
    app.add_plugins(GatedRenderDiagnostics);
    if cpu_log {
        app.add_systems(Update, report);
    }
    let Some(output) = output else {
        return;
    };
    let after_seconds = std::env::var("CRYSTAL_RENDER_GPU_TRACE_AFTER_SECONDS")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(5.0);
    let shared = SharedTiming::default();
    app.insert_resource(shared.clone());
    app.insert_resource(CaptureState {
        output,
        after_seconds,
        collector: ConsecutiveFrames::default(),
        last_marker_time: None,
        exported: false,
    });
    app.add_systems(Update, capture_completed);
    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app
            .insert_resource(shared)
            .init_resource::<ExtractedFrame>()
            .add_systems(ExtractSchedule, extract_identity);
    }
}

fn supports_pass_diagnostics(features: WgpuFeatures) -> bool {
    features.contains(WgpuFeatures::TIMESTAMP_QUERY
        | WgpuFeatures::TIMESTAMP_QUERY_INSIDE_ENCODERS
        | WgpuFeatures::TIMESTAMP_QUERY_INSIDE_PASSES)
}

fn recorder_is_safe(features: WgpuFeatures, cpu_requested: bool) -> bool {
    supports_pass_diagnostics(features)
        || (cpu_requested && !features.contains(WgpuFeatures::TIMESTAMP_QUERY))
}

/// RenderDevice is installed by RenderPlugin::finish, after the build phase.
/// Forward recorder initialization only after proving its timestamp support.
/// Bevy allocates query sets even on boundary-only devices where its interior
/// timestamps cannot work; leaving that recorder active can exhaust Metal's
/// counter buffers despite our report already saying timings are unavailable.
struct GatedRenderDiagnostics;
impl Plugin for GatedRenderDiagnostics {
    fn build(&self, _: &mut App) {}
    fn finish(&self, app: &mut App) {
        let Some(render) = app.get_sub_app_mut(RenderApp) else { return; };
        let world = render.world();
        let Some(device) = world.get_resource::<RenderDevice>() else { return; };
        let features = device.features();
        if let Some(shared) = world.get_resource::<SharedTiming>()
            && let (Some(adapter), Some(queue)) = (world.get_resource::<RenderAdapterInfo>(), world.get_resource::<RenderQueue>())
            && let Ok(mut data) = shared.data.lock()
        {
            data.adapter = Some(AdapterTiming {
                name: adapter.name.clone(), backend: format!("{:?}",adapter.backend),
                timestamp_query_enabled: features.contains(WgpuFeatures::TIMESTAMP_QUERY),
                timestamp_inside_encoders_enabled: features.contains(WgpuFeatures::TIMESTAMP_QUERY_INSIDE_ENCODERS),
                timestamp_inside_passes_enabled: features.contains(WgpuFeatures::TIMESTAMP_QUERY_INSIDE_PASSES),
                timestamp_period_ns: queue.get_timestamp_period(),
            });
        }
        let cpu_requested = std::env::var("CRYSTAL_RENDER_PASS_TIMING").as_deref() == Ok("on");
        if !recorder_is_safe(features, cpu_requested) {
            if cpu_requested {
                bevy::log::warn!("Per-pass timing is unavailable for this device; use CRYSTAL_LOADING_TRACE for CPU schedule timings");
            }
            return;
        }
        // Compose the underlying plugin here rather than adding it to the
        // registry during finish (new registry entries need not be finished).
        if !app.is_plugin_added::<RenderDiagnosticsPlugin>() {
            RenderDiagnosticsPlugin.build(app);
            RenderDiagnosticsPlugin.finish(app);
        }
        if let Some(render) = app.get_sub_app_mut(RenderApp)
            && supports_pass_diagnostics(features)
            && render.world().contains_resource::<SharedTiming>()
        {
            render.add_systems(RenderGraph, mark_frame
                .after(RenderGraphSystems::Render).before(resolve_encoder)
                .before(RenderGraphSystems::Submit));
        }
    }
}

#[derive(Clone, Default, Resource)]
struct SharedTiming {
    data: Arc<Mutex<RenderIdentityData>>,
    finished: Arc<AtomicBool>,
}
#[derive(Default)]
struct RenderIdentityData {
    adapter: Option<AdapterTiming>,
    identities: BTreeMap<u32, ExtractedFrame>,
    identity_evictions: u64,
}
#[derive(Clone, Serialize)]
struct AdapterTiming {
    name: String,
    backend: String,
    timestamp_query_enabled: bool,
    timestamp_inside_encoders_enabled: bool,
    timestamp_inside_passes_enabled: bool,
    timestamp_period_ns: f32,
}
#[derive(Clone, Default, Resource, Serialize)]
struct ExtractedFrame {
    main_frame_id: u32,
    main_interval_ms: f64,
    app_elapsed_seconds: f64,
    world_3d_ready: bool,
    active_3d_frames: u32,
    map_id: String,
    terrain_map: Option<String>,
}
fn extract_identity(
    mut output: ResMut<ExtractedFrame>,
    frame: Extract<Res<FrameCount>>,
    time: Extract<Res<Time<Real>>>,
    settings: Extract<Res<crate::VoxelViewSettings>>,
    status: Extract<Res<crate::VoxelViewStatus>>,
    world: Extract<Res<crystal_render_api::VisualWorldFrame>>,
) {
    *output = ExtractedFrame {
        main_frame_id: frame.0,
        main_interval_ms: time.delta_secs_f64() * 1000.0,
        app_elapsed_seconds: time.elapsed_secs_f64(),
        world_3d_ready: settings.enabled
            && status.active
            && status.active_frames >= 90
            && world.active
            && !status.terrain_pending
            && !status.profiles_pending
            && status.terrain_map.as_deref() == Some(world.map_id.as_ref()),
        active_3d_frames: status.active_frames,
        map_id: world.map_id.to_string(),
        terrain_map: status.terrain_map.clone(),
    };
}
fn mark_frame(
    recorder: Res<DiagnosticsRecorder>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    adapter: Res<RenderAdapterInfo>,
    identity: Res<ExtractedFrame>,
    shared: Res<SharedTiming>,
    mut pending: ResMut<PendingCommandBuffers>,
    mut next: Local<u32>,
    mut buffer: Local<Option<Buffer>>,
) {
    if shared.finished.load(Ordering::Relaxed) {
        return;
    }
    let supported = device.features().contains(WgpuFeatures::TIMESTAMP_QUERY);
    let Ok(mut data) = shared.data.lock() else {
        return;
    };
    if data.adapter.is_none() {
        data.adapter = Some(AdapterTiming {
            name: adapter.name.clone(),
            backend: format!("{:?}", adapter.backend),
            timestamp_query_enabled: supported,
            timestamp_inside_encoders_enabled: device
                .features()
                .contains(WgpuFeatures::TIMESTAMP_QUERY_INSIDE_ENCODERS),
            timestamp_inside_passes_enabled: device
                .features()
                .contains(WgpuFeatures::TIMESTAMP_QUERY_INSIDE_PASSES),
            timestamp_period_ns: queue.get_timestamp_period(),
        });
    }
    // Bevy records pass spans with interior timestamps, rather than the pass
    // descriptor boundary timestamps that Metal also exposes. A device with
    // boundary-only queries cannot produce these diagnostics.
    if !supported
        || !device.features().contains(
            WgpuFeatures::TIMESTAMP_QUERY_INSIDE_ENCODERS
                | WgpuFeatures::TIMESTAMP_QUERY_INSIDE_PASSES,
        )
    {
        return;
    }
    *next = next.wrapping_add(1);
    let id = *next;
    data.identities.insert(id, identity.clone());
    if data.identities.len() > MAX_PENDING_IDENTITIES {
        data.identities.pop_first();
        data.identity_evictions += 1;
    }
    drop(data);
    let source = buffer.get_or_insert_with(|| {
        device.create_buffer(&BufferDescriptor {
            label: Some("crystal_render_diagnostic_frame_id"),
            size: 4,
            usage: BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    });
    queue.write_buffer(source, 0, &id.to_le_bytes());
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("crystal_render_diagnostic_identity"),
    });
    // The identity travels in the same DiagnosticsRecorder frame as its pass
    // timestamps, including delayed GPU readback. No CPU-frame guess is used.
    recorder.record_u32(&mut encoder, &source.slice(..), FRAME_MARKER);
    pending.push_encoder(encoder, "crystal_render_diagnostic_identity");
}

#[derive(Clone, Serialize)]
struct CompletedFrame {
    render_frame_id: u32,
    source: ExtractedFrame,
    observed_main_frame_id: u32,
    cpu_pass_ms: BTreeMap<String, f64>,
    gpu_pass_ms: BTreeMap<String, f64>,
}
#[derive(Default)]
struct ConsecutiveFrames {
    frames: Vec<CompletedFrame>,
    last_id: Option<u32>,
    observed_unique: usize,
    gaps: usize,
    duplicates: usize,
    discarded_runs: usize,
    missing_gpu_frames: usize,
}
impl ConsecutiveFrames {
    fn accept(&mut self, frame: CompletedFrame) -> bool {
        if self.last_id == Some(frame.render_frame_id) {
            self.duplicates += 1;
            return false;
        }
        self.observed_unique += 1;
        if self
            .last_id
            .is_some_and(|last| frame.render_frame_id != last.wrapping_add(1))
        {
            self.gaps += 1;
            if !self.frames.is_empty() {
                self.discarded_runs += 1;
            }
            self.frames.clear();
        }
        self.last_id = Some(frame.render_frame_id);
        if frame.gpu_pass_ms.is_empty() {
            self.missing_gpu_frames += 1;
            if !self.frames.is_empty() {
                self.discarded_runs += 1;
            }
            self.frames.clear();
            return false;
        }
        self.frames.push(frame);
        self.frames.len() == SAMPLE_COUNT
    }
}
#[derive(Resource)]
struct CaptureState {
    output: PathBuf,
    after_seconds: f64,
    collector: ConsecutiveFrames,
    last_marker_time: Option<Instant>,
    exported: bool,
}
fn capture_completed(
    time: Res<Time<Real>>,
    main_frame: Res<FrameCount>,
    diagnostics: Res<DiagnosticsStore>,
    shared: Res<SharedTiming>,
    mut state: ResMut<CaptureState>,
) {
    if state.exported {
        return;
    }
    let Ok(mut data) = shared.data.lock() else {
        return;
    };
    let Some(adapter) = data.adapter.clone() else {
        return;
    };
    let mut status = None;
    if !adapter.timestamp_query_enabled
        || !adapter.timestamp_inside_encoders_enabled
        || !adapter.timestamp_inside_passes_enabled
    {
        status = Some("bevy_gpu_timestamps_unavailable_for_device");
    } else if let Some(marker) = diagnostics
        .iter()
        .find(|d| d.path().as_str() == FRAME_MARKER_PATH)
        .and_then(|d| d.measurement())
    {
        if state.last_marker_time != Some(marker.time) {
            state.last_marker_time = Some(marker.time);
            if marker.value.is_finite()
                && marker.value >= 0.0
                && marker.value <= u32::MAX as f64
                && marker.value.fract() == 0.0
            {
                let id = marker.value as u32;
                if let Some(source) = data.identities.remove(&id) {
                    if source.app_elapsed_seconds >= state.after_seconds && source.world_3d_ready {
                        let (cpu_pass_ms, gpu_pass_ms) =
                            fresh_pass_values(&diagnostics, marker.time, cfg!(target_os = "macos"));
                        if state.collector.accept(CompletedFrame {
                            render_frame_id: id,
                            source,
                            observed_main_frame_id: main_frame.0,
                            cpu_pass_ms,
                            gpu_pass_ms,
                        }) {
                            status = Some("complete");
                        }
                    }
                }
            }
        }
    }
    if status.is_none() && time.elapsed_secs_f64() > state.after_seconds + 60.0 {
        status = Some("incomplete_no_consecutive_30_frame_run");
    }
    let Some(status) = status else {
        return;
    };
    shared.finished.store(true, Ordering::Relaxed);
    let evictions = data.identity_evictions;
    drop(data);
    state.exported = true;
    let report = make_report(
        status,
        &adapter,
        &state.collector,
        state.after_seconds,
        evictions,
    );
    let result = serde_json::to_vec_pretty(&report)
        .map_err(|e| e.to_string())
        .and_then(|bytes| {
            if let Some(parent) = state.output.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(&state.output, bytes).map_err(|e| e.to_string())
        });
    match result {
        Ok(()) => eprintln!(
            "render_gpu_trace status={status} samples={} path={}",
            state.collector.frames.len(),
            state.output.display()
        ),
        Err(error) => eprintln!(
            "render_gpu_trace export_failed path={} error={error}",
            state.output.display()
        ),
    }
}
fn fresh_pass_values(
    diagnostics: &DiagnosticsStore,
    completed_at: Instant,
    macos: bool,
) -> (BTreeMap<String, f64>, BTreeMap<String, f64>) {
    let mut cpu = BTreeMap::new();
    let mut gpu = BTreeMap::new();
    for diagnostic in diagnostics.iter() {
        let path = diagnostic.path().as_str();
        let Some(value) = diagnostic
            .measurement()
            .filter(|v| v.time == completed_at && v.value.is_finite() && v.value >= 0.0)
        else {
            continue;
        };
        if !path.starts_with("render/") {
            continue;
        }
        if path.ends_with("/elapsed_gpu") && (!macos || verified_pass_gpu_path(path)) {
            gpu.insert(path.to_owned(), value.value);
        } else if path.ends_with("/elapsed_cpu") {
            cpu.insert(path.to_owned(), value.value);
        }
    }
    (cpu, gpu)
}
// Audited against installed Bevy 0.20 pass_span calls. Unknown labels fail
// closed on macOS: encoder scopes can expose zero or recycled unwritten slots.
fn verified_pass_gpu_path(path: &str) -> bool {
    let Some(pass) = path
        .strip_suffix("/elapsed_gpu")
        .and_then(|p| p.rsplit('/').next())
    else {
        return false;
    };
    matches!(
        pass,
        "main_opaque_pass_3d"
            | "main_transparent_pass_3d"
            | "main_opaque_pass_2d"
            | "main_transparent_pass_2d"
            | "main_transmissive_pass_3d"
            | "early prepass"
            | "late prepass"
            | "uniform_allocation"
            | "bin_unpacking"
            | "early_mesh_preprocessing"
            | "late_mesh_preprocessing"
            | "mip generation pass 1"
            | "mip generation pass 2"
            | "ui"
            | "postprocessing"
            | "fxaa"
            | "taa"
            | "contrast_adaptive_sharpening"
            | "motion_blur"
            | "oit_resolve"
            | "deferred_lighting"
            | "copy_deferred_lighting_id"
            | "atmosphere_luts"
            | "render_sky"
            | "ssr"
    )
}
fn make_report(
    status: &str,
    adapter: &AdapterTiming,
    capture: &ConsecutiveFrames,
    after_seconds: f64,
    evictions: u64,
) -> serde_json::Value {
    let mut aggregates: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for frame in &capture.frames {
        for (path, &value) in frame.cpu_pass_ms.iter().chain(frame.gpu_pass_ms.iter()) {
            aggregates.entry(path.clone()).or_default().push(value);
        }
    }
    let means: BTreeMap<_, _> = aggregates.into_iter().map(|(path, values)| {
        let n = values.len();
        (path, serde_json::json!({"sample_count": n, "mean_ms": values.iter().sum::<f64>() / n as f64}))
    }).collect();
    serde_json::json!({
        "schema": "crystal-native-render-diagnostics-v1", "status": status,
        "requested_consecutive_frames": SAMPLE_COUNT, "captured_frames": capture.frames.len(),
        "capture_after_app_seconds": after_seconds, "requires_ready_3d_world_frames": 90, "adapter": adapter,
        "observed_unique_completions": capture.observed_unique, "identity_gaps_or_reordering": capture.gaps,
        "duplicate_render_ids_rejected": capture.duplicates, "discarded_incomplete_runs": capture.discarded_runs,
        "completed_frames_without_gpu_timings": capture.missing_gpu_frames, "pending_identity_evictions": evictions,
        "frames": capture.frames, "pass_means": means,
        "clock_limitations": [
            "GPU values are timestamp-query elapsed pass durations, not displayed FPS or presentation intervals.",
            "On macOS Bevy 0.20 suppresses CommandEncoder timestamp writes. Only audited RenderPass/ComputePass diagnostic labels are retained; encoder/node/shadow aggregate GPU diagnostics are omitted even when they contain nonzero recycled query values.",
            "CPU pass durations and GPU pass durations carry the same GPU-readback render-frame marker; main_interval_ms is the paired extracted app wall interval, not CPU busy time.",
            "Diagnostic completion is delayed; observed_main_frame_id identifies observation, not the source render frame.",
            "Only measurements bearing the exact completion timestamp of their frame marker are included; stale diagnostic values are excluded.",
            "Bevy completion synchronization can overwrite results before main-world observation. Any missing/reordered render ID resets the consecutive capture.",
            "Bevy exposes elapsed GPU durations without absolute GPU begin/end times. Nested or overlapping passes cannot be summed into a total GPU span or reconstructed as a timestamp waterfall.",
            "Timestamp queries and the four-byte identity readback add profiling overhead. Export file I/O occurs after collection.",
            "If multiple view/pass occurrences share one diagnostic path, DiagnosticsStore exposes its last occurrence rather than a sum; this report preserves that Bevy path granularity."
        ]
    })
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

#[cfg(test)]
mod tests {
    #[test]
    fn boundary_only_timestamp_devices_do_not_install_a_recorder() {
        assert!(super::recorder_is_safe(bevy::render::render_resource::WgpuFeatures::empty(), true));
        assert!(!super::recorder_is_safe(bevy::render::render_resource::WgpuFeatures::empty(), false));
        assert!(!super::recorder_is_safe(bevy::render::render_resource::WgpuFeatures::TIMESTAMP_QUERY, true));
        assert!(!super::supports_pass_diagnostics(bevy::render::render_resource::WgpuFeatures::TIMESTAMP_QUERY));
        let all = bevy::render::render_resource::WgpuFeatures::TIMESTAMP_QUERY
            | bevy::render::render_resource::WgpuFeatures::TIMESTAMP_QUERY_INSIDE_ENCODERS
            | bevy::render::render_resource::WgpuFeatures::TIMESTAMP_QUERY_INSIDE_PASSES;
        assert!(super::supports_pass_diagnostics(all));
        for missing in [bevy::render::render_resource::WgpuFeatures::TIMESTAMP_QUERY,
            bevy::render::render_resource::WgpuFeatures::TIMESTAMP_QUERY_INSIDE_ENCODERS,
            bevy::render::render_resource::WgpuFeatures::TIMESTAMP_QUERY_INSIDE_PASSES] {
            assert!(!super::supports_pass_diagnostics(all - missing));
        }
        let mut app = bevy::prelude::App::new();
        app.add_plugins(super::GatedRenderDiagnostics);
        app.finish();
        assert!(!app.is_plugin_added::<bevy::render::diagnostic::RenderDiagnosticsPlugin>());
    }
    use super::*;
    use bevy::diagnostic::{Diagnostic, DiagnosticMeasurement, DiagnosticPath};
    fn frame(id: u32) -> CompletedFrame {
        CompletedFrame {
            render_frame_id: id,
            source: ExtractedFrame {
                main_frame_id: id + 100,
                main_interval_ms: 16.0,
                app_elapsed_seconds: 10.0,
                world_3d_ready: true,
                active_3d_frames: 100,
                map_id: "Route29".into(),
                terrain_map: Some("Route29".into()),
            },
            observed_main_frame_id: id + 103,
            cpu_pass_ms: BTreeMap::from([("render/opaque/elapsed_cpu".into(), 1.0)]),
            gpu_pass_ms: BTreeMap::from([("render/opaque/elapsed_gpu".into(), f64::from(id))]),
        }
    }
    #[test]
    fn native_gpu_capture_requires_thirty_unique_consecutive_completions() {
        let mut capture = ConsecutiveFrames::default();
        assert!(!capture.accept(frame(1)));
        assert!(!capture.accept(frame(1)));
        assert_eq!(capture.frames.len(), 1);
        assert!(!capture.accept(frame(3)));
        assert_eq!(capture.frames.len(), 1);
        assert_eq!(capture.duplicates, 1);
        assert_eq!(capture.gaps, 1);
        for id in 4..32 {
            assert!(!capture.accept(frame(id)));
        }
        assert!(capture.accept(frame(32)));
        assert_eq!(capture.frames.len(), 30);
        assert_eq!(capture.frames.first().unwrap().render_frame_id, 3);
        assert_eq!(capture.frames.last().unwrap().render_frame_id, 32);
        assert_eq!(capture.frames[0].source.main_frame_id, 103);
        assert_eq!(capture.frames[0].observed_main_frame_id, 106);
    }
    #[test]
    fn native_gpu_capture_excludes_stale_diagnostic_measurements() {
        let completed = Instant::now();
        let stale = completed - std::time::Duration::from_millis(1);
        let mut store = DiagnosticsStore::default();
        for (path, time, value) in [
            ("render/opaque/elapsed_cpu", completed, 2.0),
            ("render/opaque/elapsed_gpu", completed, 5.0),
            ("render/shadow/elapsed_gpu", stale, 9.0),
            ("render/invalid/elapsed_gpu", completed, f64::NAN),
            ("render/negative/elapsed_gpu", completed, -1.0),
        ] {
            let mut diagnostic = Diagnostic::new(DiagnosticPath::new(path));
            diagnostic.add_measurement(DiagnosticMeasurement { time, value });
            store.add(diagnostic);
        }
        let (cpu, gpu) = fresh_pass_values(&store, completed, false);
        assert_eq!(cpu.len(), 1);
        assert_eq!(
            gpu,
            BTreeMap::from([("render/opaque/elapsed_gpu".into(), 5.0)])
        );
        assert!(
            fresh_pass_values(
                &store,
                completed + std::time::Duration::from_nanos(1),
                false
            )
            .1
            .is_empty()
        );
    }
    #[test]
    fn macos_gpu_capture_rejects_unwritten_encoder_slots_even_when_nonzero() {
        let completed = Instant::now();
        let mut store = DiagnosticsStore::default();
        for (path, value) in [
            ("render/view/main_opaque_pass_3d/elapsed_gpu", 5.0),
            ("render/view/early_mesh_preprocessing/elapsed_gpu", 0.5),
            ("render/view/per_view_shadow_pass/elapsed_gpu", 9.0),
            ("render/view/encoder_node/elapsed_gpu", 0.0),
        ] {
            let mut diagnostic = Diagnostic::new(DiagnosticPath::new(path));
            diagnostic.add_measurement(DiagnosticMeasurement {
                time: completed,
                value,
            });
            store.add(diagnostic);
        }
        let (_, gpu) = fresh_pass_values(&store, completed, true);
        assert_eq!(gpu.len(), 2);
        assert_eq!(gpu["render/view/main_opaque_pass_3d/elapsed_gpu"], 5.0);
        assert_eq!(gpu["render/view/early_mesh_preprocessing/elapsed_gpu"], 0.5);
        assert!(!verified_pass_gpu_path(
            "render/view/per_view_shadow_pass/elapsed_gpu"
        ));
    }
    #[test]
    fn missing_gpu_frames_reset_run_and_means_report_actual_pass_counts() {
        let mut capture = ConsecutiveFrames::default();
        capture.accept(frame(1));
        let mut missing = frame(2);
        missing.gpu_pass_ms.clear();
        assert!(!capture.accept(missing));
        assert!(capture.frames.is_empty());
        assert_eq!(capture.missing_gpu_frames, 1);
        capture.accept(frame(3));
        let mut next = frame(4);
        next.gpu_pass_ms
            .insert("render/shadow/elapsed_gpu".into(), 8.0);
        capture.accept(next);
        let report = make_report(
            "incomplete",
            &AdapterTiming {
                name: "test".into(),
                backend: "test".into(),
                timestamp_query_enabled: true,
                timestamp_inside_encoders_enabled: true,
                timestamp_inside_passes_enabled: true,
                timestamp_period_ns: 1.0,
            },
            &capture,
            5.0,
            0,
        );
        assert_eq!(report["captured_frames"], 2);
        assert_eq!(
            report["pass_means"]["render/opaque/elapsed_gpu"]["sample_count"],
            2
        );
        assert_eq!(
            report["pass_means"]["render/opaque/elapsed_gpu"]["mean_ms"],
            3.5
        );
        assert_eq!(
            report["pass_means"]["render/shadow/elapsed_gpu"]["sample_count"],
            1
        );
    }
}
