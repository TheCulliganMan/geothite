//! Offscreen scenarios observe real controller outcomes and renderer readiness.
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Result;
use bevy::app::{App, AppExit, First, Last, PostUpdate, Update};
use bevy::diagnostic::DiagnosticsStore;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, Local, Res, ResMut};
use bevy::ecs::world::World;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::Resource;
use bevy::render::diagnostic::RenderDiagnosticsPlugin;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::renderer::{RenderAdapterInfo, RenderDevice};
use bevy::render::view::screenshot::ScreenshotCaptured;
use bevy::window::PrimaryWindow;
use crystal_render_api::{VisualBattleFrame, WorldRenderSet};
use crystal_voxel_view::{BattleViewStatus, VoxelViewStatus};
use serde::Serialize;

use super::handheld_input::HandheldInput;
use super::{BevyRuntimeShell, apply_keyboard_input, handheld_input, render_capture};
use crate::performance_audit::{AuditExpectation, AuditRequest};

#[path = "performance_audit_gpu.rs"]
mod gpu;

#[derive(Clone, Default, ExtractResource, Resource)]
#[extract_app(bevy::render::RenderApp)]
struct FrameClock {
    id: u64,
    measured: bool,
}

#[derive(Serialize)]
struct FrameSample {
    id: u64,
    start_ms: f64,
    interval_ms: f64,
    main_cpu_ms: f64,
    measured: bool,
    stage: String,
    map: String,
    tile: [i16; 2],
    mode: String,
    battle: bool,
    pp: u32,
    dialogue: bool,
    terrain_pending: bool,
    terrain_builds: u64,
    visible_ready: bool,
    warp_phase: String,
    natural_offset: [f32; 2],
}

#[derive(Serialize)]
struct Checkpoint {
    step: usize,
    preparation: bool,
    key: String,
    latency_ms: f64,
    controller_ms: f64,
    render_settle_ms: f64,
    frame: u64,
    map: String,
    tile: [i16; 2],
}

#[derive(Resource)]
struct Audit {
    request: AuditRequest,
    origin: Instant,
    frame_start: Instant,
    previous_start: Instant,
    interval_ms: f64,
    frames: Vec<FrameSample>,
    checkpoints: Vec<Checkpoint>,
    preparing: bool,
    step: usize,
    step_start: Instant,
    measurement_start: Option<Instant>,
    initial_pp: Option<u32>,
    warmup: usize,
    validation: Arc<Mutex<Option<std::result::Result<(), String>>>>,
    validation_requested: bool,
    validation_accepted: bool,
    drain: usize,
    failure: Option<String>,
    finish_requested: Option<Instant>,
    render_diagnostics: Vec<serde_json::Value>,
    diagnostic_times: BTreeMap<String, bevy::platform::time::Instant>,
    gpu: gpu::Shared,
    stable_ready: usize,
    ready_signature: Option<(String, [i16; 2], bool)>,
    ready_frame: u64,
    committed_ms: Option<f64>,
}

pub(super) fn install(app: &mut App, request: AuditRequest) -> Result<()> {
    anyhow::ensure!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .quick_save_path
            .is_none(),
        "audit cannot write a user save"
    );
    request.scenario.validate()?;
    render_capture::install_recording_target(app, request.directory.join("warmup.png"))?;
    let shared = gpu::Shared::default();
    let now = Instant::now();
    app.insert_resource(Audit {
        preparing: !request.scenario.prepare.is_empty(),
        request,
        origin: now,
        frame_start: now,
        previous_start: now,
        interval_ms: 0.0,
        frames: Vec::new(),
        checkpoints: Vec::new(),
        step: 0,
        step_start: now,
        measurement_start: None,
        initial_pp: None,
        warmup: 0,
        validation: Arc::default(),
        validation_requested: false,
        validation_accepted: false,
        drain: 0,
        failure: None,
        finish_requested: None,
        render_diagnostics: Vec::new(),
        diagnostic_times: BTreeMap::new(),
        gpu: shared.clone(),
        stable_ready: 0,
        ready_signature: None,
        ready_frame: 0,
        committed_ms: None,
    })
    .init_resource::<FrameClock>()
    .add_plugins(ExtractResourcePlugin::<FrameClock>::default())
    .add_systems(First, begin_frame)
    .add_systems(
        Update,
        drive
            .after(handheld_input::gather_input)
            .before(apply_keyboard_input),
    )
    .add_systems(PostUpdate, observe.after(WorldRenderSet::RenderSync))
    .add_systems(Last, end_frame);
    let gpu_enabled = app.world().resource::<Audit>().request.gpu;
    if gpu_enabled {
        app.add_plugins(RenderDiagnosticsPlugin);
    }
    gpu::install(app, shared, gpu_enabled)?;
    Ok(())
}

fn begin_frame(mut audit: ResMut<Audit>, mut clock: ResMut<FrameClock>) {
    let now = Instant::now();
    audit.interval_ms = (now - audit.previous_start).as_secs_f64() * 1000.0;
    audit.previous_start = now;
    audit.frame_start = now;
    clock.id += 1;
    clock.measured = audit.measurement_start.is_some() && audit.finish_requested.is_none();
    #[cfg(feature = "operation-trace")]
    let _marker = bevy::log::info_span!(
        "audit_frame_marker",
        frame_id = clock.id,
        measured = clock.measured
    )
    .entered();
}

fn key_code(key: &str) -> Option<KeyCode> {
    match key {
        "right" => Some(KeyCode::ArrowRight),
        "left" => Some(KeyCode::ArrowLeft),
        "up" => Some(KeyCode::ArrowUp),
        "down" => Some(KeyCode::ArrowDown),
        "a" => Some(KeyCode::KeyZ),
        "b" => Some(KeyCode::KeyX),
        "start" => Some(KeyCode::Enter),
        _ => None,
    }
}

fn drive(
    audit: Res<Audit>,
    mut input: ResMut<HandheldInput>,
    mut previous: Local<ButtonInput<KeyCode>>,
) {
    let steps = if audit.preparing {
        &audit.request.scenario.prepare
    } else {
        &audit.request.scenario.steps
    };
    let desired = if audit.finish_requested.is_none()
        && audit.failure.is_none()
        && (audit.preparing || audit.measurement_start.is_some())
        && audit.warmup >= 90
    {
        steps
            .get(audit.step)
            .filter(|step| {
                !audit.frames.last().is_some_and(|frame| {
                    expectation_matches(
                        &step.until,
                        frame,
                        audit.initial_pp.unwrap_or(frame.pp),
                        audit.step_start.elapsed().as_secs_f64(),
                    )
                })
            })
            .and_then(|step| key_code(&step.key))
            .filter(|key| {
                matches!(
                    key,
                    KeyCode::ArrowRight
                        | KeyCode::ArrowLeft
                        | KeyCode::ArrowUp
                        | KeyCode::ArrowDown
                ) || (audit.step_start.elapsed().as_secs_f64() * 5.0).fract() < 0.5
            })
    } else {
        None
    };
    previous.clear();
    let held: Vec<_> = previous.get_pressed().copied().collect();
    for key in held {
        if Some(key) != desired {
            previous.release(key);
        }
    }
    if let Some(key) = desired {
        previous.press(key);
    }
    input.keys = previous.clone();
}

fn expectation_matches(
    expected: &AuditExpectation,
    frame: &FrameSample,
    initial_pp: u32,
    elapsed: f64,
) -> bool {
    expected.map.as_ref().is_none_or(|map| *map == frame.map)
        && expected.tile.is_none_or(|tile| tile == frame.tile)
        && expected
            .mode
            .as_ref()
            .is_none_or(|mode| *mode == frame.mode)
        && expected.battle.is_none_or(|battle| battle == frame.battle)
        && (!expected.pp_consumed || frame.pp < initial_pp)
        && elapsed >= f64::from(expected.seconds)
}

#[derive(bevy::ecs::system::SystemParam)]
struct Scene<'w, 's> {
    runtime: Res<'w, BevyRuntimeShell>,
    status: Res<'w, VoxelViewStatus>,
    battle_status: Res<'w, BattleViewStatus>,
    battle: Res<'w, VisualBattleFrame>,
    windows: bevy::ecs::system::Query<
        'w,
        's,
        bevy::ecs::entity::Entity,
        bevy::ecs::query::With<PrimaryWindow>,
    >,
    capture: Res<'w, render_capture::Capture>,
}

fn observe(mut audit: ResMut<Audit>, clock: Res<FrameClock>, scene: Scene, mut commands: Commands) {
    if audit.finish_requested.is_some() {
        return;
    }
    if audit.origin.elapsed() > Duration::from_secs(300) || audit.frames.len() >= 100_000 {
        audit.failure = Some("scenario exceeded its bounded capture limit".into());
    }
    if let Some(error) = scene.runtime.render_test_error.as_ref() {
        audit.failure = Some(format!("production controller: {error}"));
    }
    let snapshot = match scene.runtime.shell.presentation_snapshot() {
        Ok(snapshot) => snapshot,
        Err(error) => {
            audit.failure = Some(error.to_string());
            audit.finish_requested = Some(Instant::now());
            return;
        }
    };
    let pp = snapshot
        .party
        .slots
        .iter()
        .flat_map(|slot| &slot.pokemon.moves)
        .map(|m| u32::from(m.current_pp))
        .sum();
    let pp = snapshot.battle.as_ref().map_or(pp, |battle| {
        battle
            .player_moves
            .iter()
            .map(|m| u32::from(m.current_pp))
            .sum()
    });
    let initial_pp = *audit.initial_pp.get_or_insert(pp);
    let battle = snapshot.battle.is_some();
    let scene_ready = if battle {
        scene.battle.active && !scene.battle.preparing && scene.battle_status.active
    } else {
        scene.status.active
            && !scene.status.terrain_pending
            && !scene.status.profiles_pending
            && scene.status.terrain_map.as_deref() == Some(snapshot.overworld.map_name.as_str())
            && scene.runtime.visible_walk_warp_phase.is_none()
            && scene.runtime.player_walk_frame_ticks == 0
    };
    let signature = (
        snapshot.overworld.map_name.clone(),
        [snapshot.overworld.tile.x, snapshot.overworld.tile.y],
        battle,
    );
    if audit.ready_signature.as_ref() != Some(&signature) || !scene_ready {
        audit.ready_signature = Some(signature);
        audit.ready_frame = clock.id;
        audit.stable_ready = 0;
    }
    let pipeline_ready =
        audit.gpu.pipeline_ready() && audit.gpu.completed_frame() >= audit.ready_frame;
    audit.stable_ready = if scene_ready && pipeline_ready {
        audit.stable_ready + 1
    } else {
        0
    };
    let ready = audit.stable_ready >= 3;
    let frame = FrameSample {
        id: clock.id,
        start_ms: (audit.frame_start - audit.origin).as_secs_f64() * 1000.0,
        interval_ms: audit.interval_ms,
        main_cpu_ms: 0.0,
        measured: clock.measured,
        stage: if audit.preparing {
            "prepare"
        } else if audit.measurement_start.is_some() {
            "measure"
        } else {
            "warmup"
        }
        .into(),
        map: snapshot.overworld.map_name.clone(),
        tile: [snapshot.overworld.tile.x, snapshot.overworld.tile.y],
        mode: format!("{:?}", snapshot.overworld.mode),
        battle,
        pp,
        dialogue: !scene.runtime.battle_messages.is_empty() || snapshot.ui.text_window_open,
        terrain_pending: scene.status.terrain_pending,
        terrain_builds: scene.status.terrain_builds,
        visible_ready: ready,
        warp_phase: format!("{:?}", scene.runtime.visible_walk_warp_phase),
        natural_offset: scene
            .runtime
            .natural_movement
            .displacement(&snapshot.overworld.map_name, snapshot.overworld.tile)
            .to_array(),
    };
    audit.warmup += 1;
    if audit.warmup == 90 {
        audit.step_start = Instant::now();
    }
    if audit.warmup >= 90 && (audit.preparing || audit.measurement_start.is_some()) {
        let steps = if audit.preparing {
            &audit.request.scenario.prepare
        } else {
            &audit.request.scenario.steps
        };
        if let Some(step) = steps.get(audit.step).cloned() {
            let elapsed = audit.step_start.elapsed().as_secs_f64();
            let matched = expectation_matches(&step.until, &frame, initial_pp, elapsed);
            if matched && audit.committed_ms.is_none() {
                audit.committed_ms = Some(elapsed * 1000.0);
            }
            if ready && matched {
                let checkpoint = Checkpoint {
                    step: audit.step,
                    preparation: audit.preparing,
                    key: step.key.clone(),
                    latency_ms: elapsed * 1000.0,
                    controller_ms: audit.committed_ms.unwrap_or(elapsed * 1000.0),
                    render_settle_ms: elapsed * 1000.0
                        - audit.committed_ms.unwrap_or(elapsed * 1000.0),
                    frame: clock.id,
                    map: frame.map.clone(),
                    tile: frame.tile,
                };
                audit.checkpoints.push(checkpoint);
                audit.step += 1;
                audit.stable_ready = 0;
                audit.committed_ms = None;
                audit.step_start = Instant::now();
            } else if elapsed > f64::from(step.timeout_seconds) {
                audit.failure = Some(format!(
                    "step {} ({}) timed out at {} {:?}, mode {}, battle {}, PP {}; expected {:?}",
                    audit.step,
                    step.key,
                    frame.map,
                    frame.tile,
                    frame.mode,
                    frame.battle,
                    frame.pp,
                    step.until
                ));
            }
        } else if audit.preparing {
            audit.preparing = false;
            audit.step = 0;
            audit.warmup = 0;
        }
    }
    if !audit.preparing && audit.measurement_start.is_none() && audit.warmup >= 120 && ready {
        let completion = audit
            .validation
            .lock()
            .expect("warmup completion lock")
            .take();
        if let Some(result) = completion {
            match result {
                Ok(()) => audit.validation_accepted = true,
                Err(error) => audit.failure = Some(format!("warmup readback failed: {error}")),
            }
        }
        if !audit.validation_requested {
            if let Ok(window) = scene.windows.single()
                && let Some(screenshot) =
                    render_capture::moving_screenshot(Some(&scene.capture), window)
            {
                let completion = audit.validation.clone();
                let path = audit.request.directory.join("warmup.png");
                commands.spawn(screenshot).observe(
                    move |event: bevy::ecs::observer::On<ScreenshotCaptured>| {
                        let result = (|| -> Result<()> {
                            event
                                .image
                                .clone()
                                .try_into_dynamic()?
                                .to_rgb8()
                                .save(&path)?;
                            super::validate_render_test_screenshot(&path)?;
                            Ok(())
                        })()
                        .map_err(|error| error.to_string());
                        *completion.lock().expect("warmup completion lock") = Some(result);
                    },
                );
                audit.validation_requested = true;
            }
        } else if audit.validation_accepted {
            audit.drain += 1;
            if audit.drain >= 30 {
                audit.measurement_start = Some(Instant::now());
                audit.step_start = Instant::now();
            }
        }
    }
    audit.frames.push(frame);
    let done = audit.measurement_start.is_some_and(|start| {
        start.elapsed().as_secs_f64() >= f64::from(audit.request.scenario.seconds)
    }) && audit.step == audit.request.scenario.steps.len();
    if done || audit.failure.is_some() {
        audit.finish_requested = Some(Instant::now());
    }
}

fn end_frame(world: &mut World) {
    let clock = world.resource::<FrameClock>().clone();
    let observations = if let Some(store) = world.get_resource::<DiagnosticsStore>() {
        store
            .iter()
            .filter_map(|diagnostic| {
                diagnostic
                    .measurement()
                    .map(|m| (diagnostic.path().to_string(), m.time, m.value))
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let adapter = world
        .get_resource::<RenderAdapterInfo>()
        .map(|info| format!("{:?}", **info));
    let device_features = world
        .get_resource::<RenderDevice>()
        .map(|device| format!("{:?}", device.features()));
    let mut audit = world.resource_mut::<Audit>();
    let elapsed = audit.frame_start.elapsed().as_secs_f64() * 1000.0;
    if let Some(frame) = audit.frames.last_mut().filter(|frame| frame.id == clock.id) {
        frame.main_cpu_ms = elapsed;
    }
    if clock.measured {
        for (path, time, value) in observations {
            if !path.starts_with("render/")
                || audit.diagnostic_times.get(&path) == Some(&time)
                || (cfg!(target_os = "macos") && path.ends_with("/elapsed_gpu"))
            {
                continue;
            }
            audit.diagnostic_times.insert(path.clone(), time);
            // Bevy returns delayed durations without a source frame identifier.
            // Keep receipt frame explicit; never pretend these align with it.
            audit
                .render_diagnostics
                .push(serde_json::json!({"receipt_frame":clock.id,"path":path,"value":value}));
        }
    }
    let Some(finished) = audit.finish_requested else {
        return;
    };
    // Nonblocking drain permits final timestamp callbacks to arrive.
    if finished.elapsed() < Duration::from_millis(500) {
        return;
    }
    let gpu = audit.gpu.snapshot();
    let adapter = adapter.or_else(|| Some(gpu.adapter.clone()));
    let device_features = device_features.or_else(|| Some(gpu.features.clone()));
    let report = serde_json::json!({"schema":1,"success":audit.failure.is_none(),"failure":audit.failure,
        "scenario":audit.request.scenario,"adapter":adapter,"features":device_features,
        "frames":audit.frames,"checkpoints":audit.checkpoints,"gpu":gpu,
        "render_diagnostics":audit.render_diagnostics,
        "semantics":{"main_cpu_ms":"First to Last main-app wall time; includes blocking; excludes render sub-app",
        "interval_ms":"consecutive main-app starts; offscreen throughput, not presented FPS",
        "gpu":"timestamp envelope around render graph command buffers; independent GPU clock; source frame IDs",
        "render_diagnostics":"Bevy pass durations; receipt frame only; nested spans overlap"}});
    let result = std::fs::write(
        audit.request.directory.join("audit.json"),
        serde_json::to_vec(&report).expect("serialize audit"),
    );
    let failed = audit.failure.is_some() || result.is_err();
    drop(audit);
    world.write_message(if failed {
        AppExit::error()
    } else {
        AppExit::Success
    });
}

#[cfg(test)]
mod tests {
    use super::{AuditExpectation, FrameSample, expectation_matches};

    #[test]
    fn blocked_movement_and_unexecuted_battle_cannot_pass() {
        let mut frame = FrameSample {
            id: 1,
            start_ms: 0.,
            interval_ms: 1.,
            main_cpu_ms: 1.,
            measured: true,
            stage: "measure".into(),
            map: "Route44".into(),
            tile: [38, 3],
            mode: "Normal".into(),
            battle: true,
            pp: 30,
            dialogue: false,
            terrain_pending: false,
            terrain_builds: 1,
            visible_ready: true,
            warp_phase: "None".into(),
            natural_offset: [0.; 2],
        };
        let expected = AuditExpectation {
            tile: Some([38, 4]),
            mode: Some("Surf".into()),
            pp_consumed: true,
            ..Default::default()
        };
        assert!(!expectation_matches(&expected, &frame, 30, 1.));
        frame.tile = [38, 4];
        frame.mode = "Surf".into();
        assert!(!expectation_matches(&expected, &frame, 30, 1.));
        frame.pp = 29;
        assert!(expectation_matches(&expected, &frame, 30, 1.));
    }
}
