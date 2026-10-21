//! Portable pass-boundary timestamps, including Metal; never wait for the GPU.
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
    Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, ComputePassDescriptor,
    ComputePassTimestampWrites, Features, MapMode, PollType, QuerySet, QuerySetDescriptor,
    QueryType,
};

use super::FrameClock;

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
}

#[derive(Resource)]
struct Recorder {
    slots: Vec<Slot>,
    active: Option<(usize, u64, Instant)>,
    timestamp_period: f64,
}

pub(super) fn install(app: &mut App, shared: Shared, enabled: bool) -> Result<()> {
    app.get_sub_app_mut(RenderApp)
        .context("audit requires RenderApp")?
        .insert_resource(shared)
        .insert_resource(Enabled(enabled))
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
struct Enabled(bool);

fn initialize(
    mut commands: bevy::ecs::system::Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    shared: Res<Shared>,
    enabled: Res<Enabled>,
    adapter: Res<RenderAdapterInfo>,
) {
    let supported = enabled.0 && device.features().contains(Features::TIMESTAMP_QUERY);
    {
        let mut results = shared.0.lock().expect("GPU result lock");
        results.supported = supported;
        results.adapter = format!("{:?}", **adapter);
        results.features = format!("{:?}", device.features());
        results.reason = if supported {
            "pass-boundary timestamps; separate GPU clock"
        } else if !enabled.0 {
            "GPU timing disabled for instrumentation control"
        } else {
            "adapter/device lacks TIMESTAMP_QUERY; GPU time unavailable"
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
            });
        }
    }
    commands.insert_resource(Recorder {
        slots,
        active: None,
        timestamp_period: f64::from(queue.get_timestamp_period()),
    });
}

fn begin(
    clock: Res<FrameClock>,
    mut recorder: ResMut<Recorder>,
    device: Res<RenderDevice>,
    mut pending: ResMut<PendingCommandBuffers>,
    shared: Res<Shared>,
) {
    if !clock.measured || recorder.slots.is_empty() {
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
    // Pass descriptor writes work on Metal/WebGPU too; encoder.write_timestamp
    // needs a different capability and is deliberately avoided.
    {
        let _pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("audit start"),
            timestamp_writes: Some(ComputePassTimestampWrites {
                query_set: &recorder.slots[index].query,
                beginning_of_pass_write_index: Some(0),
                end_of_pass_write_index: None,
            }),
        });
    }
    pending.push_encoder(encoder, "audit_begin");
    recorder.active = Some((index, clock.id, Instant::now()));
}

fn end(
    recorder: Res<Recorder>,
    device: Res<RenderDevice>,
    mut pending: ResMut<PendingCommandBuffers>,
) {
    let Some((index, _, _)) = recorder.active else {
        return;
    };
    let slot = &recorder.slots[index];
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("audit end marker"),
    });
    {
        let _pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("audit end"),
            timestamp_writes: Some(ComputePassTimestampWrites {
                query_set: &slot.query,
                beginning_of_pass_write_index: None,
                end_of_pass_write_index: Some(1),
            }),
        });
    }
    encoder.resolve_query_set(&slot.query, 0..2, &slot.resolve, 0);
    encoder.copy_buffer_to_buffer(&slot.resolve, 0, &slot.read, 0, 16);
    pending.push_encoder(encoder, "audit_end_resolve");
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
    queue.on_submitted_work_done(move || {
        let mut results = completion.0.lock().expect("GPU result lock");
        results.completed_frame = results.completed_frame.max(frame);
    });
    if let Some((index, frame, started)) = recorder.active.take() {
        let slot = &recorder.slots[index];
        let buffer = slot.read.clone();
        let busy = slot.busy.clone();
        let output = shared.clone();
        let period = recorder.timestamp_period;
        slot.read.slice(..).map_async(MapMode::Read, move |result| {
            let mut output = output.0.lock().expect("GPU result lock");
            if result.is_ok() {
                let data = buffer.slice(..).get_mapped_range();
                let start = u64::from_le_bytes(data[0..8].try_into().expect("timestamp width"));
                let end = u64::from_le_bytes(data[8..16].try_into().expect("timestamp width"));
                if end >= start && output.samples.len() < 100_000 {
                    output.samples.push(Sample {
                        frame,
                        start_ns: start as f64 * period,
                        end_ns: end as f64 * period,
                        elapsed_ms: (end - start) as f64 * period / 1e6,
                        completion_latency_ms: started.elapsed().as_secs_f64() * 1000.0,
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
    // Poll only; no Wait/WaitForSubmissionIndex or synchronous readback.
    if device.poll(PollType::Poll).is_err() {
        shared.0.lock().expect("GPU result lock").errors += 1;
    }
}
