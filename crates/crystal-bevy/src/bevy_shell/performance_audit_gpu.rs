//! Encoder completion timestamps and native Metal correlation; never wait for the GPU.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::{Context, Result};
use bevy::app::App;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Res, ResMut};
use bevy::render::renderer::{
    PendingCommandBuffers, RenderAdapterInfo, RenderDevice, RenderGraph, RenderGraphSystems,
    RenderQueue,
};
use bevy::render::{RenderApp, RenderStartup};
use serde::Serialize;
use wgpu::{
    Backend, Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Features, MapMode,
    PollType, QuerySet, QuerySetDescriptor, QueryType,
};

use super::FrameClock;
#[path = "audit_gpu_marker.rs"]
mod audit_gpu_marker;
use audit_gpu_marker::GpuMarker;

#[derive(Clone, Default, Resource)]
pub(super) struct Shared(Arc<Mutex<Results>>);

#[derive(Default, Clone, Serialize)]
pub(super) struct Results {
    supported: bool,
    reason: String,
    skipped: u64,
    errors: u64,
    samples: Vec<Sample>,
    pipeline_pending: usize,
    completed_frame: u64,
    pub(super) adapter: String,
    pub(super) features: String,
    pipeline_errors: usize,
}

#[derive(Clone, Serialize)]
struct Sample {
    frame: u64,
    start_ns: f64,
    end_ns: f64,
    elapsed_ms: f64,
    completion_latency_ms: f64,
    marker_gpu_ms: f64,
}

impl Shared {
    pub(super) fn completed_frame(&self) -> u64 {
        self.0.lock().expect("GPU result lock").completed_frame
    }

    pub(super) fn pipeline_ready(&self) -> bool {
        let results = self.0.lock().expect("GPU result lock");
        results.pipeline_pending == 0 && results.pipeline_errors == 0
    }

    pub(super) fn snapshot(&self) -> Results {
        self.0.lock().expect("GPU result lock").clone()
    }
}

struct Slot {
    query: QuerySet,
    resolve: Buffer,
    read: Buffer,
    busy: Arc<AtomicBool>,
    sampling_done: Arc<AtomicBool>,
    pending: Option<(u64, Instant)>,
}

#[derive(Resource)]
struct Recorder {
    slots: Vec<Slot>,
    active: Option<(usize, u64, Instant)>,
    timestamp_period: f64,
    marker: Option<GpuMarker>,
    metal_trace: bool,
}

pub(super) fn install(
    app: &mut App,
    shared: Shared,
    enabled: bool,
    metal_trace: bool,
) -> Result<()> {
    app.get_sub_app_mut(RenderApp)
        .context("audit requires RenderApp")?
        .insert_resource(shared)
        .insert_resource(Enabled(enabled, metal_trace))
        .add_systems(RenderStartup, initialize)
        .add_systems(
            RenderGraph,
            (
                begin.before(RenderGraphSystems::Render),
                end.after(RenderGraphSystems::Render)
                    .before(RenderGraphSystems::Submit),
                readback
                    .after(RenderGraphSystems::Submit)
                    .before(RenderGraphSystems::Finish),
            ),
        );
    Ok(())
}

#[derive(Resource)]
struct Enabled(bool, bool);

fn initialize(
    mut commands: bevy::ecs::system::Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    shared: Res<Shared>,
    enabled: Res<Enabled>,
    adapter: Res<RenderAdapterInfo>,
) {
    // Independent compute markers can overlap Metal graphics work. Native
    // Metal captures use CPU submission boundaries and actual GPU intervals.
    let metal = adapter.backend == Backend::Metal;
    let metal_trace = enabled.0 && enabled.1 && metal;
    let supported = enabled.0
        && !metal
        && device
            .features()
            .contains(Features::TIMESTAMP_QUERY | Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);
    {
        let mut results = shared.0.lock().expect("GPU result lock");
        results.supported = supported;
        results.adapter = format!("{:?}", **adapter);
        results.features = format!("{:?}", device.features());
        results.reason = if supported {
            "encoder completion timestamp envelope; separate GPU clock"
        } else if !enabled.0 {
            "GPU timing disabled for instrumentation control"
        } else if metal_trace {
            "native Metal GPU intervals must be imported from Metal System Trace"
        } else if metal {
            "Metal markers do not bound graphics; use the runner's native Metal backend"
        } else {
            "adapter/device lacks encoder timestamp queries; GPU time unavailable"
        }
        .into();
    }
    let mut slots = Vec::new();
    if supported {
        for _ in 0..8 {
            let query = device.wgpu_device().create_query_set(&QuerySetDescriptor {
                label: Some("audit render envelope"),
                ty: QueryType::Timestamp,
                count: 2,
            });
            let resolve = device.wgpu_device().create_buffer(&BufferDescriptor {
                label: Some("audit timestamp resolve"),
                size: 16,
                usage: BufferUsages::QUERY_RESOLVE | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let read = device.wgpu_device().create_buffer(&BufferDescriptor {
                label: Some("audit timestamp readback"),
                size: 16,
                usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            slots.push(Slot {
                query,
                resolve,
                read,
                busy: Arc::new(AtomicBool::new(false)),
                sampling_done: Arc::new(AtomicBool::new(false)),
                pending: None,
            });
        }
    }
    commands.insert_resource(Recorder {
        slots,
        active: None,
        timestamp_period: f64::from(queue.get_timestamp_period()),
        marker: metal_trace.then(|| GpuMarker::new(device.wgpu_device())),
        metal_trace,
    });
}

fn begin(
    clock: Res<FrameClock>,
    mut recorder: ResMut<Recorder>,
    device: Res<RenderDevice>,
    mut pending: ResMut<PendingCommandBuffers>,
    shared: Res<Shared>,
) {
    if !clock.capture_gpu {
        return;
    }
    if recorder.metal_trace {
        record_metal_boundary(&recorder, &device, &mut pending, clock.id, "begin");
        return;
    }
    if recorder.slots.is_empty() {
        return;
    }
    let slot = recorder
        .slots
        .iter()
        .position(|slot| !slot.busy.swap(true, Ordering::AcqRel));
    let Some(index) = slot else {
        shared.0.lock().expect("GPU result lock").skipped += 1;
        return;
    };
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("audit begin marker"),
    });
    // Encoder timestamps record queue completion boundaries. Unlike an
    // independent compute dispatch, these do not pretend to measure graphics
    // pass activity. Metal uses the native pass trace instead.
    encoder.write_timestamp(&recorder.slots[index].query, 0);
    pending.push_encoder(encoder, "audit_begin");
    recorder.active = Some((index, clock.id, Instant::now()));
}

fn end(
    clock: Res<FrameClock>,
    recorder: Res<Recorder>,
    device: Res<RenderDevice>,
    mut pending: ResMut<PendingCommandBuffers>,
) {
    if recorder.metal_trace && clock.capture_gpu {
        record_metal_boundary(&recorder, &device, &mut pending, clock.id, "end");
        return;
    }
    let Some((index, _, _)) = recorder.active else {
        return;
    };
    let slot = &recorder.slots[index];
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("audit end marker"),
    });
    encoder.write_timestamp(&slot.query, 1);
    pending.push_encoder(encoder, "audit_end");
}

fn record_metal_boundary(
    recorder: &Recorder,
    device: &RenderDevice,
    pending: &mut PendingCommandBuffers,
    frame: u64,
    boundary: &str,
) {
    let label = format!("geothite_audit_frame_{frame}_{boundary}");
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some(&label),
    });
    recorder
        .marker
        .as_ref()
        .expect("native boundary marker")
        .record(&mut encoder, None, None, None, &label);
    pending.push_encoder(encoder, label);
}

fn readback(
    mut recorder: ResMut<Recorder>,
    device: Res<RenderDevice>,
    shared: Res<Shared>,
    pipelines: Res<bevy::render::render_resource::PipelineCache>,
    clock: Res<FrameClock>,
    queue: Res<RenderQueue>,
) {
    {
        let mut output = shared.0.lock().expect("GPU result lock");
        output.pipeline_pending = pipelines.waiting_pipelines().count();
        output.pipeline_errors = pipelines
            .pipelines()
            .filter(|pipeline| {
                matches!(
                    pipeline.state,
                    bevy::render::render_resource::CachedPipelineState::Err(_)
                )
            })
            .count();
    }
    let completion = shared.clone();
    let frame = clock.id;
    let sampling_done = recorder.active.take().map(|(index, frame, started)| {
        let slot = &mut recorder.slots[index];
        slot.pending = Some((frame, started));
        Arc::clone(&slot.sampling_done)
    });
    queue.on_submitted_work_done(move || {
        let mut results = completion.0.lock().expect("GPU result lock");
        results.completed_frame = results.completed_frame.max(frame);
        if let Some(sampling_done) = sampling_done {
            sampling_done.store(true, Ordering::Release);
        }
    });
    // Poll only. On Metal, resolving in the sampling submission can return
    // unwritten zeros even after its readback fence completes. Submit resolves
    // only after the sampling callback, while the renderer keeps running.
    if device.poll(PollType::Poll).is_err() {
        shared.0.lock().expect("GPU result lock").errors += 1;
    }
    let period = recorder.timestamp_period;
    for slot in &mut recorder.slots {
        if !slot.sampling_done.swap(false, Ordering::AcqRel) {
            continue;
        }
        let (frame, started) = slot.pending.take().expect("completed sampling slot");
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("audit deferred timestamp resolve"),
        });
        encoder.resolve_query_set(&slot.query, 0..2, &slot.resolve, 0);
        encoder.copy_buffer_to_buffer(&slot.resolve, 0, &slot.read, 0, 16);
        queue.submit([encoder.finish()]);
        let buffer = slot.read.clone();
        let busy = slot.busy.clone();
        let output = shared.clone();
        slot.read.slice(..).map_async(MapMode::Read, move |result| {
            let mut output = output.0.lock().expect("GPU result lock");
            if result.is_ok() {
                let Ok(data) = buffer.slice(..).get_mapped_range() else {
                    output.errors += 1;
                    buffer.unmap();
                    busy.store(false, Ordering::Release);
                    return;
                };
                let start = u64::from_le_bytes(data[0..8].try_into().expect("timestamp width"));
                let end = u64::from_le_bytes(data[8..16].try_into().expect("timestamp width"));
                if start > 0 && end > start && output.samples.len() < 100_000 {
                    output.samples.push(Sample {
                        frame,
                        start_ns: start as f64 * period,
                        end_ns: end as f64 * period,
                        elapsed_ms: (end - start) as f64 * period / 1e6,
                        completion_latency_ms: started.elapsed().as_secs_f64() * 1000.0,
                        marker_gpu_ms: 0.0,
                    });
                } else {
                    output.errors += 1;
                }
                drop(data);
                buffer.unmap();
            } else {
                output.errors += 1;
            }
            busy.store(false, Ordering::Release);
        });
    }
}
