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
use crystal_render_api::{VisualBattleFrame, VisualWorldFrame, WorldRenderSet};
use crystal_voxel_view::{BattleViewStatus, VoxelViewStatus};
use serde::Serialize;

use super::handheld_input::HandheldInput;
use super::{BevyRuntimeShell, apply_keyboard_input, handheld_input, render_capture};
use crate::performance_audit::{AuditExpectation, AuditFixture, AuditRequest};
use crystal_core::models::Dv;

#[path = "audit_readiness.rs"]
mod audit_readiness;
#[path = "audit_movement_leg.rs"]
mod movement_leg;

#[path = "audit_capture_gate.rs"]
mod audit_capture_gate;

#[path = "audit_motion.rs"]
mod audit_motion;

#[path = "audit_motion_capture.rs"]
mod audit_motion_capture;

#[path = "performance_audit_gpu.rs"]
mod gpu;

#[derive(Clone, Default, ExtractResource, Resource)]
#[extract_app(bevy::render::RenderApp)]
struct FrameClock {
    id: u64,
    measured: bool,
    capture_gpu: bool,
}

#[derive(Default, Serialize)]
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
    battle_commands: bool,
    phase: String,
    player_hp: Option<u16>,
    enemy_hp: Option<u16>,
    pp: u32,
    dialogue: bool,
    terrain_pending: bool,
    terrain_builds: u64,
    visible_ready: bool,
    warp_phase: String,
    natural_offset: [f32; 2],
    simulation_tick: u64,
    walk_remaining: u8,
    motion: audit_motion::MotionSample,
}

#[derive(Serialize)]
struct Checkpoint {
    step: usize,
    preparation: bool,
    key: String,
    latency_ms: f64,
    controller_ms: f64,
    render_settle_ms: f64,
    renderer_waited: bool,
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
    frames: Vec<FrameSample>,
    checkpoints: Vec<Checkpoint>,
    preparing: bool,
    preparation_armed: bool,
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
    readiness: audit_readiness::Readiness,
    committed_ms: Option<f64>,
    matched_last_frame: bool,
    capture_gate: audit_capture_gate::CaptureGate,
    motion: audit_motion::MotionAudit,
    motion_capture: audit_motion_capture::MotionCapture,
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
    let start = app
        .world()
        .resource::<BevyRuntimeShell>()
        .shell
        .presentation_snapshot()?;
    let tile = [start.overworld.tile.x, start.overworld.tile.y];
    anyhow::ensure!(
        start.overworld.map_name == request.scenario.map && tile == request.scenario.tile,
        "audit start differs from recipe: expected {} {:?}, got {} {:?}",
        request.scenario.map,
        request.scenario.tile,
        start.overworld.map_name,
        tile
    );
    match request.scenario.fixture {
        AuditFixture::SurfShore => prepare_repelled_surf(app)?,
        AuditFixture::Battle => prepare_battle_lead(app)?,
        _ => {}
    }
    render_capture::install_recording_target(app, request.directory.join("warmup.png"))?;
    let shared = gpu::Shared::default();
    let now = Instant::now();
    let capture_gate = audit_capture_gate::CaptureGate::new(request.wait_for_metal_capture);
    let motion_capture = audit_motion_capture::MotionCapture::new(&request.directory)?;
    app.insert_resource(Audit {
        preparing: !request.scenario.prepare.is_empty(),
        preparation_armed: false,
        request,
        origin: now,
        frame_start: now,
        previous_start: now,
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
        readiness: Default::default(),
        committed_ms: None,
        matched_last_frame: false,
        capture_gate,
        motion: Default::default(),
        motion_capture,
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
    let metal_trace = app.world().resource::<Audit>().request.metal_trace;
    // Native Metal supplies pass activity without Bevy's counter buffers.
    // Redundant pass queries can exhaust MTLCounterSampleBuffer allocations.
    if gpu_enabled && !metal_trace {
        app.add_plugins(RenderDiagnosticsPlugin);
    }
    gpu::install(app, shared, gpu_enabled, metal_trace)?;
    Ok(())
}

// The field is rendered before ordinary A/Yes enters the actual encounter.
// Use a legal Cyndaquil/Swift setup. The production engine owns
// accuracy, damage and fainting; the audit records the resulting HP and phase.
fn prepare_battle_lead(app: &mut App) -> Result<()> {
    let mut runtime = app.world_mut().resource_mut::<BevyRuntimeShell>();
    let initial = runtime.shell.snapshot()?;
    anyhow::ensure!(
        initial.party.slots.len() == 1 && initial.battle.is_none(),
        "battle audit requires the checked field encounter fixture"
    );
    runtime.shell.add_party_pokemon(
        "CYNDAQUIL",
        30,
        None,
        None,
        &initial.trainer.player_name,
        initial.trainer.player_id,
        Dv::from_non_hp(9, 9, 9, 9),
    )?;
    runtime.shell.add_bag_item("TM_SWIFT", 1)?;
    let replacement =
        (runtime.shell.snapshot()?.party.slots[1].pokemon.moves.len() == 4).then_some(0);
    runtime
        .shell
        .use_bag_tmhm_on_party_pokemon("TM_SWIFT", 1, replacement)?;
    runtime.shell.swap_party_pokemon(0, 1)?;
    super::mark_runtime_snapshot_dirty(&mut runtime);
    Ok(())
}

// Seed only this disposable performance fixture through validated mutations.
// A level-100 lead plus ordinary Max Repel excludes Route44's level-20/25
// encounters without replacing the pack's encounter table or gameplay RNG.
fn prepare_repelled_surf(app: &mut App) -> Result<()> {
    let mut runtime = app.world_mut().resource_mut::<BevyRuntimeShell>();
    let initial = runtime.shell.snapshot()?;
    anyhow::ensure!(
        initial.party.slots.len() == 1,
        "Surf audit requires its checked single-party shore fixture"
    );
    runtime.shell.add_party_pokemon(
        "TOTODILE",
        100,
        None,
        None,
        &initial.trainer.player_name,
        initial.trainer.player_id,
        Dv::from_non_hp(9, 9, 9, 9),
    )?;
    let replacement =
        (runtime.shell.snapshot()?.party.slots[1].pokemon.moves.len() == 4).then_some(0);
    runtime
        .shell
        .use_bag_tmhm_on_party_pokemon("HM_SURF", 1, replacement)?;
    runtime.shell.swap_party_pokemon(0, 1)?;
    runtime.shell.add_bag_item("MAX_REPEL", 1)?;
    runtime.shell.use_bag_repel_in_field("MAX_REPEL")?;
    super::mark_runtime_snapshot_dirty(&mut runtime);
    Ok(())
}

fn begin_frame(mut audit: ResMut<Audit>, mut clock: ResMut<FrameClock>) {
    // Untimed startup otherwise spins through thousands of incomplete renders
    // while shaders compile, flooding native traces. Pace preparation/warmup at
    // normal game cadence; measured frames and their intervals stay unthrottled.
    if audit.measurement_start.is_none() && audit.finish_requested.is_none() {
        let remaining =
            Duration::from_secs_f64(1.0 / 60.0).saturating_sub(audit.previous_start.elapsed());
        std::thread::sleep(remaining);
    }
    let now = Instant::now();
    let interval = (now - audit.previous_start).as_secs_f64() * 1000.0;
    if let Some(frame) = audit.frames.last_mut().filter(|frame| frame.id == clock.id) {
        frame.interval_ms = interval;
    }
    audit.previous_start = now;
    audit.frame_start = now;
    clock.id += 1;
    clock.measured = audit.measurement_start.is_some() && audit.finish_requested.is_none();
    clock.capture_gpu = audit.warmup >= 60 && audit.finish_requested.is_none();
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
        && ((audit.preparing && audit.preparation_armed) || audit.measurement_start.is_some())
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
        && expected
            .battle_commands
            .is_none_or(|commands| commands == frame.battle_commands)
        && (!expected.pp_consumed || (frame.pp < initial_pp && frame.battle && frame.dialogue))
        && elapsed >= f64::from(expected.seconds)
}

#[derive(bevy::ecs::system::SystemParam)]
struct Scene<'w, 's> {
    runtime: Res<'w, BevyRuntimeShell>,
    status: Res<'w, VoxelViewStatus>,
    battle_status: Res<'w, BattleViewStatus>,
    battle: Res<'w, VisualBattleFrame>,
    world: Res<'w, VisualWorldFrame>,
    cameras: bevy::ecs::system::Query<
        'w,
        's,
        (
            &'static bevy::prelude::Camera,
            &'static bevy::prelude::Transform,
        ),
        bevy::ecs::query::With<bevy::prelude::Camera3d>,
    >,
    windows: bevy::ecs::system::Query<
        'w,
        's,
        bevy::ecs::entity::Entity,
        bevy::ecs::query::With<PrimaryWindow>,
    >,
    capture: Res<'w, render_capture::Capture>,
}

fn observe(mut audit: ResMut<Audit>, clock: Res<FrameClock>, scene: Scene, mut commands: Commands) {
    #[cfg(feature = "operation-trace")]
    let _span = bevy::log::info_span!("performance_audit_observer").entered();
    if audit.finish_requested.is_some() {
        return;
    }
    if audit.origin.elapsed() > Duration::from_secs(300) || audit.frames.len() >= 100_000 {
        audit.failure = Some(format!(
            "scenario exceeded capture limit; world={:?}; battle={:?}",
            scene.status.inactive_reason, scene.battle_status.map_scenery_fallback_reason
        ));
    }
    if let Some(error) = scene.runtime.render_test_error.as_ref() {
        audit.failure = Some(format!("production controller: {error}"));
    }
    // Reuse the production snapshot when its authoritative revision matches.
    // Do not rebuild the complete presentation just to measure a frame.
    let retained = scene
        .runtime
        .cached_snapshot
        .as_ref()
        .filter(|(revision, _)| *revision == scene.runtime.snapshot_revision);
    let refreshed;
    let snapshot = if let Some((_, snapshot)) = retained {
        snapshot.as_ref()
    } else {
        refreshed = match scene.runtime.shell.presentation_snapshot() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                audit.failure = Some(error.to_string());
                audit.finish_requested = Some(Instant::now());
                return;
            }
        };
        &refreshed
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
    let battle = snapshot.battle.is_some();
    // Party totals and active-battler PP use different scopes. Entering or
    // leaving battle cannot count as consuming a move.
    if audit
        .frames
        .last()
        .is_some_and(|frame| frame.battle != battle)
    {
        audit.initial_pp = Some(pp);
    }
    let initial_pp = *audit.initial_pp.get_or_insert(pp);
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
    let pipelines_ready = audit.gpu.pipeline_ready();
    let completed_frame = audit.gpu.completed_frame();
    let mut ready = audit.readiness.observe(
        signature,
        scene_ready,
        clock.id,
        pipelines_ready,
        completed_frame,
    );
    let mut frame = FrameSample {
        id: clock.id,
        start_ms: (audit.frame_start - audit.origin).as_secs_f64() * 1000.0,
        interval_ms: 0.0,
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
        battle_commands: scene.runtime.battle_action_cursor.is_some()
            && scene.runtime.battle_messages.is_empty(),
        phase: format!("{:?}", snapshot.phase),
        player_hp: snapshot
            .battle
            .as_ref()
            .and_then(|battle| {
                snapshot
                    .party
                    .slots
                    .get(battle.active_player_party_index.unwrap_or(0))
            })
            .map(|slot| slot.pokemon.hp),
        enemy_hp: snapshot
            .battle
            .as_ref()
            .map(|battle| battle.enemy_pokemon.hp),
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
        motion: audit.motion.observe(
            &scene.world,
            &scene.status,
            scene.cameras.iter().find_map(|(camera, transform)| {
                (camera.is_active && camera.order == 0).then_some(transform)
            }),
        ),
        simulation_tick: scene.runtime.shell.session().state().frame_counter,
        walk_remaining: scene.runtime.player_walk_frame_ticks,
    };
    audit.warmup += 1;
    if audit.preparing && !audit.preparation_armed && audit.warmup >= 120 && ready {
        audit.preparation_armed = true;
        audit.step_start = Instant::now();
    }
    if audit.measurement_start.is_none() && audit.origin.elapsed() > Duration::from_secs(90) {
        audit.failure = Some(format!(
            "warmup readiness timed out; map={}; battle_active={}; scenery={:?}; world={:?}",
            frame.map,
            scene.battle_status.active,
            scene.battle_status.map_scenery_fallback_reason,
            scene.status.inactive_reason
        ));
    }
    if audit.warmup == 90 {
        audit.step_start = Instant::now();
    }
    if audit.warmup >= 90
        && ((audit.preparing && audit.preparation_armed) || audit.measurement_start.is_some())
    {
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
            if matched && !audit.matched_last_frame {
                audit.readiness.require_frame(clock.id);
                ready = false;
                frame.visible_ready = false;
            }
            audit.matched_last_frame = matched;
            let continuous = !audit.preparing && audit.request.scenario.continuous_movement;
            if movement_leg::ready(&frame, continuous, ready) && matched {
                let checkpoint = Checkpoint {
                    step: audit.step,
                    preparation: audit.preparing,
                    key: step.key.clone(),
                    latency_ms: elapsed * 1000.0,
                    controller_ms: audit.committed_ms.unwrap_or(elapsed * 1000.0),
                    render_settle_ms: if continuous { 0. } else {
                        elapsed * 1000.0 - audit.committed_ms.unwrap_or(elapsed * 1000.0)
                    },
                    renderer_waited: !continuous,
                    frame: clock.id,
                    map: frame.map.clone(),
                    tile: frame.tile,
                };
                audit.checkpoints.push(checkpoint);
                audit.step += 1;
                audit.initial_pp = Some(pp);
                audit.readiness.restart_stability();
                audit.committed_ms = None;
                audit.matched_last_frame = false;
                audit.step_start = Instant::now();
            } else if elapsed > f64::from(step.timeout_seconds) {
                audit.failure = Some(format!(
                    "step {} ({}) timed out at {} {:?}, mode {}, battle {}, PP {}; expected {:?}; renderer={:?}; source_scene={}",
                    audit.step,
                    step.key,
                    frame.map,
                    frame.tile,
                    frame.mode,
                    frame.battle,
                    frame.pp,
                    step.until,
                    *scene.battle_status,
                    scene.battle.use_source_scene,
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
                let started = audit.request.directory.join("native-start").is_file();
                match audit.capture_gate.advance(started) {
                    audit_capture_gate::Action::PublishReady => {
                        if let Err(error) = std::fs::write(
                            audit.request.directory.join("native-ready"),
                            b"validated warmup complete",
                        ) {
                            audit.failure =
                                Some(format!("native profiler handshake failed: {error}"));
                        }
                    }
                    audit_capture_gate::Action::Wait => {}
                    audit_capture_gate::Action::Measure => {
                        audit.measurement_start = Some(Instant::now());
                        audit.initial_pp = Some(pp);
                        audit.step_start = Instant::now();
                    }
                }
            }
        }
    }
    if clock.measured && audit.motion_capture.wants_tick(frame.simulation_tick) {
        if let Ok(window) = scene.windows.single() {
            if let Some(screenshot) =
                render_capture::moving_screenshot(Some(&scene.capture), window)
            {
                audit.motion_capture.request(
                    clock.id,
                    frame.simulation_tick,
                    screenshot,
                    &mut commands,
                );
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
            if audit.render_diagnostics.len() >= 1_000_000 {
                audit.failure = Some("render diagnostic limit exceeded".into());
                audit.finish_requested = Some(Instant::now());
                break;
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
    if audit.motion_capture.pending() {
        if finished.elapsed() < Duration::from_secs(10) {
            return;
        }
        audit.failure = Some("motion screenshot readbacks did not drain".into());
    }
    if let Some(error) = audit.motion_capture.error() {
        audit.failure = Some(format!("motion screenshot readback failed: {error}"));
    }
    let gpu = audit.gpu.snapshot();
    let adapter = adapter.or_else(|| Some(gpu.adapter.clone()));
    let device_features = device_features.or_else(|| Some(gpu.features.clone()));
    let report = serde_json::json!({"schema":1,"success":audit.failure.is_none(),"failure":audit.failure,
        "scenario":audit.request.scenario,"adapter":adapter,"features":device_features,
        "frames":audit.frames,"checkpoints":audit.checkpoints,"gpu":gpu,
        "motion_capture":audit.motion_capture.receipt(),
        "render_diagnostics":audit.render_diagnostics,
        "semantics":{"main_cpu_ms":"First to Last main-app wall time; includes blocking; excludes render sub-app",
        "interval_ms":"this frame start to next main-app start; offscreen throughput, not presented FPS",
        "gpu":"native GPU activity or encoder completion timestamps; independent GPU clock; source frame IDs",
        "render_diagnostics":"Bevy pass durations; receipt frame only; nested spans overlap"}});
    let result = std::fs::write(
        audit.request.directory.join("audit.json"),
        serde_json::to_vec(&report).expect("serialize audit"),
    );
    let failed = audit.failure.is_some() || result.is_err();
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
            battle_commands: true,
            phase: "StaticWildBattle".into(),
            player_hp: Some(50),
            enemy_hp: Some(50),
            pp: 30,
            dialogue: false,
            terrain_pending: false,
            terrain_builds: 1,
            visible_ready: true,
            warp_phase: "None".into(),
            natural_offset: [0.; 2],
            motion: Default::default(),
            simulation_tick: 0,
            walk_remaining: 0,
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
        assert!(!expectation_matches(&expected, &frame, 30, 1.));
        frame.dialogue = true;
        assert!(expectation_matches(&expected, &frame, 30, 1.));
    }
}
