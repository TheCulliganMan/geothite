//! Opt-in actual-game capture and no-readback timing. Never drives game input.
//! Timestamps retain wall time; no frame interpolation or retiming is performed.
//!
//! Native-only `CRYSTAL_CAPTURE_PACING_PROBE=1` requires image recording,
//! `on_move`, and at least 36 seconds. After the ordinary first-move trigger it
//! admits screenshots only in [14, 22), preserving the ordinary PNG pipeline.
//! `capture-pacing.csv` observes every update, including final readback draining.
//! Counts are cumulative and outstanding includes callbacks not yet collected.
//! Analyze baseline rows only when `baseline_eligible=1`: both ends of the
//! interval must be in the same baseline, and both updates must have no
//! outstanding readbacks before completion polling or after admission.
//! `capture-clock.csv` correlates the unchanged Bevy real clock to Unix UTC;
//! callback timestamps use that same monotonic origin. PNG timing includes file
//! creation, encoding, and writing, not an fsync. A timeout is never a drain.
//! Probe PNGs and timestamps are evidence, not a continuous video, so the probe
//! deliberately emits no ffconcat manifest across its nonrecorded windows.
use super::*;
use bevy::utils::Instant as CaptureInstant;
use image::ImageEncoder;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

type Completions = Arc<Mutex<Vec<CaptureCompletion>>>;
const MAX_READBACKS: usize = 8;
const DEFAULT_CAPTURE_HZ: u32 = 30;
const DEFAULT_ARM_TIMEOUT_SECONDS: u32 = 180;
const CAPTURE_TIME_EPSILON: f64 = 1.0e-9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProbePhase {
    Warmup,
    BaselineBefore,
    Capture,
    Drain,
    BaselineAfter,
    Complete,
}

impl ProbePhase {
    fn at(elapsed: f64) -> Self {
        if elapsed < 6.0 {
            Self::Warmup
        } else if elapsed < 14.0 {
            Self::BaselineBefore
        } else if elapsed < 22.0 {
            Self::Capture
        } else if elapsed < 24.0 {
            Self::Drain
        } else if elapsed < 36.0 {
            Self::BaselineAfter
        } else {
            Self::Complete
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Warmup => "warmup",
            Self::BaselineBefore => "baseline_before",
            Self::Capture => "capture",
            Self::Drain => "drain",
            Self::BaselineAfter => "baseline_after",
            Self::Complete => "complete",
        }
    }

    fn is_baseline(self) -> bool {
        matches!(self, Self::BaselineBefore | Self::BaselineAfter)
    }
}

fn capture_phase_enabled(probe: bool, elapsed: f64) -> bool {
    !probe || ProbePhase::at(elapsed) == ProbePhase::Capture
}

#[derive(Debug, PartialEq, Eq)]
enum ReadbackDrain {
    Pending,
    Complete,
    TimedOut,
}

fn readback_drain(outstanding: usize, wait_seconds: f64) -> ReadbackDrain {
    if outstanding == 0 {
        ReadbackDrain::Complete
    } else if wait_seconds > 20.0 {
        ReadbackDrain::TimedOut
    } else {
        ReadbackDrain::Pending
    }
}

struct ProbeClock {
    origin: CaptureInstant,
    bevy_real_start: f64,
    utc_unix_start: f64,
    pairing_span_us: f64,
}

impl ProbeClock {
    fn sample(time: &Time<Real>) -> Self {
        let before = CaptureInstant::now();
        let utc = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("capture clock must be after the Unix epoch")
            .as_secs_f64();
        let after = CaptureInstant::now();
        let origin = time.last_update().unwrap_or(before);
        let span = after.duration_since(before).as_secs_f64();
        Self {
            origin,
            bevy_real_start: time.elapsed_seconds_f64(),
            utc_unix_start: utc - before.duration_since(origin).as_secs_f64() - span / 2.0,
            pairing_span_us: span * 1_000_000.0,
        }
    }
}

struct PacingProbe {
    clock: Option<ProbeClock>,
    trace: String,
    callbacks: String,
    completed: usize,
    updates: usize,
    previous_elapsed: Option<f64>,
    previous_readback_free: bool,
}

impl Default for PacingProbe {
    fn default() -> Self {
        Self {
            clock: None,
            trace: "update,seconds,delta_ms,phase,submitted,completed,outstanding_before_poll,outstanding,baseline_eligible,window_focused\n".into(),
            callbacks: "frame,frame_seconds,request_seconds,callback_seconds,complete_seconds,request_to_callback_ms,rgb_ms,png_write_ms,success\n".into(),
            completed: 0,
            updates: 0,
            previous_elapsed: None,
            previous_readback_free: false,
        }
    }
}

impl PacingProbe {
    fn observe_update(
        &mut self,
        elapsed: f64,
        delta_ms: f64,
        submitted: usize,
        outstanding_before_poll: usize,
        outstanding: usize,
        focused: Option<bool>,
    ) {
        let phase = ProbePhase::at(elapsed);
        let readback_free = outstanding_before_poll == 0 && outstanding == 0;
        let baseline_eligible = phase.is_baseline()
            && self
                .previous_elapsed
                .is_some_and(|previous| ProbePhase::at(previous) == phase)
            && self.previous_readback_free
            && readback_free;
        self.trace.push_str(&format!(
            "{},{elapsed:.6},{delta_ms:.6},{},{submitted},{},{outstanding_before_poll},{outstanding},{},{}\n",
            self.updates, phase.name(), self.completed, u8::from(baseline_eligible),
            focused.map_or_else(String::new, |value| u8::from(value).to_string()),
        ));
        self.updates += 1;
        self.previous_elapsed = Some(elapsed);
        self.previous_readback_free = readback_free;
    }
}

struct CaptureCompletion {
    result: std::result::Result<(), String>,
    timing: Option<String>,
}

#[derive(Clone, Copy)]
struct CaptureTicket {
    index: usize,
    frame_elapsed: f64,
    origin: CaptureInstant,
    requested_at: CaptureInstant,
}

fn save_capture(image: Image, path: PathBuf, ticket: Option<CaptureTicket>) -> CaptureCompletion {
    let callback_at = ticket.map(|_| CaptureInstant::now());
    let mut rgb_finished_at = None;
    let result = (|| {
        let rgb = image
            .try_into_dynamic()
            .map_err(|error| format!("decode GPU frame: {error}"))?
            .to_rgb8();
        if ticket.is_some() {
            rgb_finished_at = Some(CaptureInstant::now());
        }
        let file = std::fs::File::create(path).map_err(|error| error.to_string())?;
        image::codecs::png::PngEncoder::new_with_quality(
            file,
            image::codecs::png::CompressionType::Fast,
            image::codecs::png::FilterType::NoFilter,
        )
        .write_image(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|error| error.to_string())
    })();
    let timing = ticket.zip(callback_at).map(|(ticket, callback_at)| {
        let completed_at = CaptureInstant::now();
        let rgb_ms = rgb_finished_at.map_or_else(String::new, |at| {
            format!(
                "{:.6}",
                at.duration_since(callback_at).as_secs_f64() * 1000.0
            )
        });
        let png_write_ms = rgb_finished_at.map_or_else(String::new, |at| {
            format!(
                "{:.6}",
                completed_at.duration_since(at).as_secs_f64() * 1000.0
            )
        });
        format!(
            "{},{:.6},{:.6},{:.6},{:.6},{:.6},{rgb_ms},{png_write_ms},{}\n",
            ticket.index,
            ticket.frame_elapsed,
            ticket
                .requested_at
                .duration_since(ticket.origin)
                .as_secs_f64(),
            callback_at.duration_since(ticket.origin).as_secs_f64(),
            completed_at.duration_since(ticket.origin).as_secs_f64(),
            callback_at
                .duration_since(ticket.requested_at)
                .as_secs_f64()
                * 1000.0,
            u8::from(result.is_ok()),
        )
    });
    CaptureCompletion { result, timing }
}

struct CaptureCadence {
    next_at: f64,
    requested_hz: u32,
}

impl Default for CaptureCadence {
    fn default() -> Self {
        Self::new(DEFAULT_CAPTURE_HZ)
    }
}

impl CaptureCadence {
    fn new(requested_hz: u32) -> Self {
        debug_assert!(matches!(requested_hz, 30 | 60));
        Self {
            next_at: 0.0,
            requested_hz,
        }
    }

    fn interval(&self) -> f64 {
        1.0 / f64::from(self.requested_hz)
    }

    fn due(&self, elapsed: f64, outstanding: usize) -> bool {
        outstanding < MAX_READBACKS && elapsed + CAPTURE_TIME_EPSILON >= self.next_at
    }

    fn admitted(&mut self, elapsed: f64) {
        // Anchor deadlines to the start of the recording. Setting the next
        // deadline to elapsed + interval would alias a 40Hz renderer to 20Hz.
        // Skip missed deadlines after a stall or readback backpressure; never
        // synthesize duplicate frames or request catch-up captures.
        self.next_at =
            (((elapsed + CAPTURE_TIME_EPSILON) / self.interval()).floor() + 1.0) * self.interval();
    }
}

fn requested_capture_hz(value: Option<&str>) -> Result<u32> {
    match value {
        None | Some("30") => Ok(DEFAULT_CAPTURE_HZ),
        Some("60") => Ok(60),
        Some(value) => anyhow::bail!("CRYSTAL_CAPTURE_FPS must be 30 or 60, found {value:?}"),
    }
}

fn requested_arm_timeout(value: Option<&str>) -> Result<f64> {
    let seconds = value.map_or(Ok(DEFAULT_ARM_TIMEOUT_SECONDS), str::parse::<u32>)?;
    anyhow::ensure!(
        (30..=900).contains(&seconds),
        "CRYSTAL_CAPTURE_ARM_SECONDS must be 30–900"
    );
    Ok(f64::from(seconds))
}

fn requested_pacing_probe(
    value: Option<&str>,
    capture_images: bool,
    on_move: bool,
    seconds: u32,
) -> Result<bool> {
    let enabled = match value {
        None | Some("0") => false,
        Some("1") => true,
        Some(value) => {
            anyhow::bail!("CRYSTAL_CAPTURE_PACING_PROBE must be 0 or 1, found {value:?}")
        }
    };
    anyhow::ensure!(
        !enabled || (capture_images && on_move && seconds >= 36),
        "CRYSTAL_CAPTURE_PACING_PROBE=1 requires image recording, on_move, and at least 36 seconds"
    );
    Ok(enabled)
}

#[derive(Resource)]
struct Recording {
    directory: PathBuf,
    seconds: f64,
    capture_images: bool,
    on_move: bool,
    on_capture: bool,
    armed_at: Option<f64>,
    arm_timeout_seconds: f64,
    trigger: String,
    start: Option<f64>,
    presented_battle_frames: u32,
    cadence: CaptureCadence,
    frames: Vec<f64>,
    trace: String,
    update_trace: String,
    battle_trace: String,
    update_times: Vec<f64>,
    outstanding: usize,
    completions: Completions,
    probe: Option<PacingProbe>,
}

impl Recording {
    fn observe_probe_update(
        &mut self,
        elapsed: f64,
        delta_ms: f64,
        outstanding_before_poll: usize,
        focused: Option<bool>,
    ) {
        if let Some(probe) = self.probe.as_mut() {
            probe.observe_update(
                elapsed,
                delta_ms,
                self.frames.len(),
                outstanding_before_poll,
                self.outstanding,
                focused,
            );
        }
    }
}

pub(super) fn install(
    app: &mut App,
    directory: &Path,
    seconds: u32,
    capture_images: bool,
    on_move: bool,
    on_capture: bool,
) -> Result<()> {
    anyhow::ensure!(
        (1..=300).contains(&seconds),
        "measurement must be 1–300 seconds"
    );
    anyhow::ensure!(
        !(on_move && on_capture),
        "recording triggers are mutually exclusive"
    );
    anyhow::ensure!(
        !on_capture || capture_images,
        "capture trigger requires image recording"
    );
    let pacing_probe = requested_pacing_probe(
        std::env::var("CRYSTAL_CAPTURE_PACING_PROBE")
            .ok()
            .as_deref(),
        capture_images,
        on_move,
        seconds,
    )?;
    let capture_hz = requested_capture_hz(std::env::var("CRYSTAL_CAPTURE_FPS").ok().as_deref())?;
    let arm_timeout_seconds =
        requested_arm_timeout(std::env::var("CRYSTAL_CAPTURE_ARM_SECONDS").ok().as_deref())?;
    std::fs::create_dir_all(directory)?;
    anyhow::ensure!(
        std::fs::read_dir(directory)?.next().is_none(),
        "recording output directory must be empty"
    );
    let header = "frame,seconds,map,origin_x,origin_y,player_x,player_y,capture_ball,capture_frame,capture_caught,capture_blocked,capture_shakes,capture_started,capture_complete\n";
    app.insert_resource(Recording {
        directory: directory.to_owned(),
        seconds: f64::from(seconds),
        capture_images,
        on_move,
        on_capture,
        armed_at: None,
        arm_timeout_seconds,
        trigger: String::new(),
        start: None,
        presented_battle_frames: 0,
        cadence: CaptureCadence::new(capture_hz),
        frames: vec![],
        trace: header.into(),
        update_trace: header.into(),
        battle_trace: "frame,seconds,map,player_species,enemy_species,cues,modeled,source_art,source_frame,bgp,source_objects,flash_mode,image_assets,mesh_assets,material_assets,lighting,quality,software_renderer,scene_width,scene_height,window_width,window_height,requested_capture_hz,view_mode,source_line_x,source_line_y,player_size_m,enemy_size_m,row_capture_ready,row_capture_pending,capture_ball,capture_frame,capture_caught,capture_blocked,capture_shakes,capture_started,capture_complete,capture_image_active,capture_fallback_reason\n"
            .into(),
        update_times: vec![],
        outstanding: 0,
        completions: default(),
        probe: pacing_probe.then(PacingProbe::default),
    })
    .add_systems(
        Update,
        record.after(crystal_render_api::WorldRenderSet::RenderSync),
    );
    Ok(())
}

fn capture_csv_fields(capture: Option<&VisibleCaptureAnimation>) -> String {
    capture.map_or_else(
        || ",,,,,,".into(),
        |capture| {
            format!(
                "{},{},{},{},{},{},{}",
                capture.ball_id,
                capture.frame,
                capture.caught,
                capture.blocked,
                capture.animation_shakes,
                capture.started,
                capture.complete
            )
        },
    )
}

fn row(
    index: usize,
    elapsed: f64,
    frame: &crystal_render_api::VisualWorldFrame,
    capture: Option<&VisibleCaptureAnimation>,
) -> String {
    let player = frame
        .actors
        .iter()
        .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
        .map(|actor| {
            actor.center - frame.center
                + Vec2::new(
                    frame.grid_origin.x as f32 * frame.tile_size.x,
                    -frame.grid_origin.y as f32 * frame.tile_size.y,
                )
        })
        .unwrap_or(Vec2::splat(f32::NAN));
    format!(
        "{index},{elapsed:.6},{},{},{},{:.3},{:.3},{}\n",
        frame.map_id,
        frame.grid_origin.x,
        frame.grid_origin.y,
        player.x,
        player.y,
        capture_csv_fields(capture)
    )
}

fn record(
    mut recording: ResMut<Recording>,
    time: Res<Time<Real>>,
    status: Res<crystal_voxel_view::VoxelViewStatus>,
    battle_status: Res<crystal_voxel_view::BattleViewStatus>,
    battle_frame: Res<crystal_render_api::VisualBattleFrame>,
    flash_mode: Res<crystal_render_api::BattleFlashMode>,
    frame: Res<crystal_render_api::VisualWorldFrame>,
    runtime: Res<BevyRuntimeShell>,
    walk: Option<Res<super::render_walk::RenderWalk>>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut screenshots: ResMut<ScreenshotManager>,
    images: Res<Assets<Image>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    mut exit: EventWriter<AppExit>,
) {
    if let Some(error) = runtime.render_test_error.as_ref() {
        eprintln!("capture rejected after runtime error: {error}");
        exit.send(AppExit::error());
        return;
    }
    let outstanding_before_poll = recording.outstanding;
    let focused = windows.get_single().ok().map(|(_, window)| window.focused);
    let completed = std::mem::take(&mut *recording.completions.lock().unwrap());
    let mut capture_error = None;
    for completion in completed {
        if let Some(probe) = recording.probe.as_mut() {
            probe.completed += 1;
            if let Some(timing) = completion.timing {
                probe.callbacks.push_str(&timing);
            }
        }
        recording.outstanding = recording.outstanding.saturating_sub(1);
        if let Err(error) = completion.result {
            capture_error.get_or_insert(error);
        }
    }
    if let Some(error) = capture_error {
        if recording.probe.is_some() {
            if let Some(start) = recording.start {
                let elapsed = time.elapsed_seconds_f64() - start;
                recording.observe_probe_update(
                    elapsed,
                    time.delta_seconds_f64() * 1000.0,
                    outstanding_before_poll,
                    focused,
                );
                if let Err(error) = finish(&recording, elapsed, "capture_failed") {
                    eprintln!("capture diagnostics failed: {error}");
                }
            }
        }
        eprintln!("capture failed: {error}");
        exit.send(AppExit::error());
        return;
    }
    recording.presented_battle_frames = if battle_frame.active {
        recording.presented_battle_frames.saturating_add(1)
    } else {
        0
    };
    if recording.start.is_none() {
        let now = time.elapsed_seconds_f64();
        let armed_at = *recording.armed_at.get_or_insert(now);
        // A short world-walk clip should begin with its first ordinary input,
        // after the walk helper's shader/profile warmup. Timing-only runs keep
        // their existing warmup. This does not drive or advance gameplay.
        if recording.capture_images && walk.as_ref().is_some_and(|walk| !walk.started()) {
            return;
        }
        let world_ready = status.active && status.active_frames >= 30 && !status.profiles_pending;
        let battle_ready = recording_battle_ready(
            battle_frame.active,
            recording.presented_battle_frames,
            battle_status.active,
            battle_status.active_frames,
        );
        if !recording_start_ready(
            recording.on_move,
            recording.on_capture,
            world_ready,
            battle_ready,
            &battle_frame,
        ) {
            if (recording.on_move || recording.on_capture)
                && now - armed_at >= recording.arm_timeout_seconds
            {
                eprintln!(
                    "capture cancelled: no requested presented battle cue observed within {}s",
                    recording.arm_timeout_seconds
                );
                exit.send(AppExit::error());
            }
            return;
        }
        if recording.on_capture {
            if let Some(cue) = battle_frame
                .cues
                .iter()
                .find(|cue| is_capture_cue(cue.kind))
            {
                recording.trigger = format!(
                    "first presented capture: {:?} {}; capture state {}; flashes {:?}; no synthetic pre-roll\n",
                    cue.kind, cue.move_id,
                    capture_csv_fields(runtime.visible_capture_animation.as_ref()), *flash_mode
                );
            }
        } else if recording.on_move {
            if let Some(cue) = battle_frame
                .cues
                .iter()
                .find(|cue| cue.kind == crystal_render_api::VisualBattleCueKind::Move)
            {
                recording.trigger = format!(
                    "first presented move: {:?} {} at progress {:.6}; source frame {:?}; flashes {:?}; no synthetic pre-roll\n",
                    cue.side,
                    cue.move_id,
                    cue.progress,
                    battle_frame.source.as_ref().map(|source| source.frame),
                    *flash_mode
                );
            }
        } else if recording.capture_images && walk.is_some() {
            recording.trigger = "first scripted world-walk input after warmup; production controller; no synthetic pre-roll\n".into();
        }
        recording.start = Some(now);
        if let Some(probe) = recording.probe.as_mut() {
            probe.clock = Some(ProbeClock::sample(&time));
        }
        println!(
            "{} to {}",
            if recording.capture_images {
                "recording actual game frames"
            } else {
                "measuring without GPU readback"
            },
            recording.directory.display()
        );
    }
    let elapsed = time.elapsed_seconds_f64() - recording.start.unwrap();
    if elapsed >= recording.seconds {
        recording.observe_probe_update(
            elapsed,
            time.delta_seconds_f64() * 1000.0,
            outstanding_before_poll,
            focused,
        );
        match readback_drain(recording.outstanding, elapsed - recording.seconds) {
            ReadbackDrain::Pending => return,
            ReadbackDrain::TimedOut => {
                if recording.probe.is_some() {
                    if let Err(error) = finish(&recording, elapsed, "readback_timeout") {
                        eprintln!("capture diagnostics failed: {error}");
                    }
                }
                eprintln!("capture timed out waiting for GPU readback");
                exit.send(AppExit::error());
                return;
            }
            ReadbackDrain::Complete => {}
        }
        let result = finish(&recording, elapsed, "complete");
        if let Err(error) = result {
            eprintln!("capture manifest failed: {error}");
            exit.send(AppExit::error());
        } else {
            exit.send(AppExit::Success);
        }
        return;
    }
    let index = recording.update_times.len();
    recording
        .update_times
        .push(time.delta_seconds_f64() * 1000.0);
    recording.update_trace.push_str(&row(
        index,
        elapsed,
        &frame,
        runtime.visible_capture_animation.as_ref(),
    ));
    if battle_frame.active {
        let species = |side: usize| {
            battle_frame.battlers[side]
                .as_ref()
                .map_or("", |battler| battler.species_id.as_ref())
        };
        let cues = battle_frame
            .cues
            .iter()
            .map(|cue| {
                format!(
                    "{:?}:{:?}:{}:{:.3}",
                    cue.kind, cue.side, cue.move_id, cue.progress
                )
            })
            .collect::<Vec<_>>()
            .join("|");
        let source_frame = battle_frame
            .source
            .as_ref()
            .map_or(String::new(), |source| source.frame.to_string());
        let bgp = battle_frame
            .source
            .as_ref()
            .map_or(String::new(), |source| format!("{:02x}", source.bgp));
        let source_objects = battle_frame
            .source
            .as_ref()
            .map_or(String::new(), |source| {
                source
                    .objects
                    .iter()
                    .map(|object| {
                        format!(
                            "{}:{}:{:.1}:{:.1}",
                            object.slot, object.object_id, object.center.x, object.center.y
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("|")
            });
        let source_line_offsets = |vertical: bool| {
            battle_frame
                .source
                .as_ref()
                .and_then(|source| {
                    if vertical {
                        source.line_y_offsets.as_ref()
                    } else {
                        source.line_x_offsets.as_ref()
                    }
                })
                .map_or_else(String::new, |offsets| {
                    offsets
                        .iter()
                        .map(i8::to_string)
                        .collect::<Vec<_>>()
                        .join("|")
                })
        };
        let source_line_x = source_line_offsets(false);
        let source_line_y = source_line_offsets(true);
        let capture_hz = recording.cadence.requested_hz;
        let output_size = windows.get_single().map_or(UVec2::ZERO, |(_, window)| {
            UVec2::new(window.physical_width(), window.physical_height())
        });
        let (view_mode, lighting, quality, scene_size) = if battle_status.active {
            (
                "modeled_3d",
                battle_status.lighting,
                battle_status.quality,
                battle_status.render_size,
            )
        } else {
            // The classic renderer draws at the native output size. A dormant
            // 1x1 offscreen target is not its scene resolution.
            ("source_2d", "source", "native", output_size)
        };
        let physical_size = |index: usize| {
            battle_frame.battlers[index]
                .as_ref()
                .and_then(|battler| battler.pokedex_size_m)
                .map_or_else(String::new, |meters| format!("{meters:.4}"))
        };
        let flags = |values: [bool; 2]| format!("{}|{}", u8::from(values[0]), u8::from(values[1]));
        recording.battle_trace.push_str(&format!(
            "{index},{elapsed:.6},{},{},{},{},{},{},{},{},{},{:?},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            battle_frame.map_id,
            species(0),
            species(1),
            cues,
            battle_status.modeled_species.join("|"),
            battle_status.source_art_species.join("|"),
            source_frame,
            bgp,
            source_objects,
            *flash_mode,
            images.len(),
            meshes.len(),
            materials.len(),
            lighting,
            quality,
            battle_status.software_renderer,
            scene_size.x,
            scene_size.y,
            output_size.x,
            output_size.y,
            capture_hz,
            view_mode,
            source_line_x,
            source_line_y,
            physical_size(0),
            physical_size(1),
            flags(battle_status.row_capture_ready),
            flags(battle_status.row_capture_pending),
            capture_csv_fields(runtime.visible_capture_animation.as_ref()),
            battle_status.capture_image_active,
            format!("\"{}\"", battle_status.capture_fallback_reason.unwrap_or("").replace('"', "\"\"")),
        ));
    }
    if !recording.capture_images
        || !capture_phase_enabled(recording.probe.is_some(), elapsed)
        || !recording.cadence.due(elapsed, recording.outstanding)
    {
        recording.observe_probe_update(
            elapsed,
            time.delta_seconds_f64() * 1000.0,
            outstanding_before_poll,
            focused,
        );
        return;
    }
    let Ok((window, _)) = windows.get_single() else {
        recording.observe_probe_update(
            elapsed,
            time.delta_seconds_f64() * 1000.0,
            outstanding_before_poll,
            focused,
        );
        return;
    };
    let index = recording.frames.len();
    let path = recording.directory.join(format!("frame-{index:05}.png"));
    let completions = recording.completions.clone();
    let ticket = recording
        .probe
        .as_ref()
        .and_then(|probe| probe.clock.as_ref())
        .map(|clock| CaptureTicket {
            index,
            frame_elapsed: elapsed,
            origin: clock.origin,
            requested_at: CaptureInstant::now(),
        });
    // Screenshot callbacks already run on Bevy's async compute pool. Allow a
    // bounded pipeline rather than waiting 2–3 render frames after each one.
    if screenshots
        .take_screenshot(window, move |image| {
            let completion = save_capture(image, path, ticket);
            completions.lock().unwrap().push(completion);
        })
        .is_ok()
    {
        recording.frames.push(elapsed);
        recording.trace.push_str(&row(
            index,
            elapsed,
            &frame,
            runtime.visible_capture_animation.as_ref(),
        ));
        recording.cadence.admitted(elapsed);
        recording.outstanding += 1;
    }
    recording.observe_probe_update(
        elapsed,
        time.delta_seconds_f64() * 1000.0,
        outstanding_before_poll,
        focused,
    );
}

fn recording_battle_ready(
    presented: bool,
    presented_frames: u32,
    modeled_active: bool,
    modeled_frames: u32,
) -> bool {
    presented
        && if modeled_active {
            modeled_frames >= 30
        } else {
            presented_frames >= 30
        }
}

fn is_capture_cue(kind: crystal_render_api::VisualBattleCueKind) -> bool {
    matches!(
        kind,
        crystal_render_api::VisualBattleCueKind::Capture
            | crystal_render_api::VisualBattleCueKind::CaptureDeflect
    )
}

fn recording_start_ready(
    on_move: bool,
    on_capture: bool,
    world_ready: bool,
    battle_ready: bool,
    frame: &crystal_render_api::VisualBattleFrame,
) -> bool {
    if !on_move && !on_capture {
        return world_ready || battle_ready;
    }
    if on_capture {
        return battle_ready
            && frame.active
            && frame.cues.iter().any(|cue| is_capture_cue(cue.kind));
    }
    battle_ready
        && frame.active
        && frame
            .cues
            .iter()
            .any(|cue| cue.kind == crystal_render_api::VisualBattleCueKind::Move)
}

fn finish(recording: &Recording, elapsed: f64, outcome: &str) -> std::io::Result<()> {
    let probe_settings = if recording.probe.is_some() {
        "capture_pacing_probe=1\nwarmup=[0,6)\nbaseline_before=[6,14)\ncapture=[14,22)\ndrain=[22,24)\nbaseline_after=[24,36)\ncomplete=[36,infinity)\nphase_clock=seconds since first presented move; Bevy Time<Real>\nbaseline_eligible=both interval endpoints in same baseline with zero outstanding before completion polling and after admission\noutstanding=accepted requests minus collected callback completions; timeout is not drained\ncallback_pipeline=ordinary RGB conversion and PNG Fast/NoFilter encoding to file; png_write_ms includes file creation, encoding, writing, and close, not fsync\nffconcat=omitted; nonrecorded intervals must not become held video frames\n"
    } else {
        ""
    };
    std::fs::write(
        recording.directory.join("capture-settings.txt"),
        format!(
            "capture_images={}\nrequested_capture_hz={}\nactual timestamps; missed deadlines are dropped\n{probe_settings}",
            recording.capture_images, recording.cadence.requested_hz,
        ),
    )?;
    if !recording.trigger.is_empty() {
        std::fs::write(
            recording.directory.join("capture-trigger.txt"),
            &recording.trigger,
        )?;
    }
    std::fs::write(
        recording.directory.join("battle-timing.csv"),
        &recording.battle_trace,
    )?;
    std::fs::write(
        recording.directory.join("render-timing.csv"),
        &recording.update_trace,
    )?;
    if let Some(probe) = recording.probe.as_ref() {
        std::fs::write(recording.directory.join("capture-pacing.csv"), &probe.trace)?;
        std::fs::write(
            recording.directory.join("capture-callbacks.csv"),
            &probe.callbacks,
        )?;
        std::fs::write(recording.directory.join("capture-probe-result.txt"), format!(
            "outcome={outcome}\nelapsed_seconds={elapsed:.6}\nsubmitted={}\ncompleted={}\noutstanding={}\n",
            recording.frames.len(), probe.completed, recording.outstanding,
        ))?;
        if let Some(clock) = probe.clock.as_ref() {
            std::fs::write(recording.directory.join("capture-clock.csv"), format!(
                "bevy_real_start_seconds,utc_unix_start_seconds,pairing_span_us\n{:.9},{:.9},{:.3}\n",
                clock.bevy_real_start, clock.utc_unix_start, clock.pairing_span_us,
            ))?;
        }
    }
    if recording.capture_images {
        std::fs::write(recording.directory.join("timestamps.csv"), &recording.trace)?;
    }
    if recording.capture_images && recording.probe.is_none() {
        let mut concat = String::from("ffconcat version 1.0\n");
        for (index, &timestamp) in recording.frames.iter().enumerate() {
            let next = recording
                .frames
                .get(index + 1)
                .copied()
                .unwrap_or(recording.seconds);
            concat.push_str(&format!(
                "file 'frame-{index:05}.png'\noption framerate 1000\nduration {:.6}\n",
                (next - timestamp).max(0.001)
            ));
        }
        if let Some(last) = recording.frames.len().checked_sub(1) {
            concat.push_str(&format!(
                "file 'frame-{last:05}.png'\noption framerate 1000\n"
            ));
        }
        std::fs::write(recording.directory.join("frames.ffconcat"), concat)?;
    }
    let mut times = recording.update_times.clone();
    times.sort_by(f64::total_cmp);
    if !times.is_empty() {
        let summary = format!(
            "{} updates; median {:.2}ms, p95 {:.2}ms; {} captured frames over {:.2}s; requested capture cap {}Hz\n",
            times.len(),
            times[times.len() / 2],
            times[times.len() * 95 / 100],
            recording.frames.len(),
            elapsed,
            recording.cadence.requested_hz,
        );
        std::fs::write(recording.directory.join("summary.txt"), &summary)?;
        print!("{summary}");
        if recording.probe.is_some() {
            println!(
                "capture pacing probe: {outcome}; {} outstanding; use eligible phase intervals",
                recording.outstanding
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pacing_probe_requires_explicit_opt_in_and_a_complete_move_recording() {
        for value in [None, Some("0")] {
            for (images, on_move, seconds) in [(false, false, 1), (true, true, 36)] {
                assert!(!requested_pacing_probe(value, images, on_move, seconds).unwrap());
            }
        }
        assert!(requested_pacing_probe(Some("1"), true, true, 36).unwrap());
        assert!(requested_pacing_probe(Some("1"), true, true, 300).unwrap());
        for (images, on_move, seconds) in [(false, true, 36), (true, false, 36), (true, true, 35)] {
            assert!(requested_pacing_probe(Some("1"), images, on_move, seconds).is_err());
        }
        for value in ["", "true", "2", "-1"] {
            assert!(requested_pacing_probe(Some(value), true, true, 36).is_err());
        }
    }

    #[test]
    fn pacing_probe_gates_real_samples_without_deadline_catchup_or_post_window_frames() {
        for (boundary, before, after) in [
            (6.0, ProbePhase::Warmup, ProbePhase::BaselineBefore),
            (14.0, ProbePhase::BaselineBefore, ProbePhase::Capture),
            (22.0, ProbePhase::Capture, ProbePhase::Drain),
            (24.0, ProbePhase::Drain, ProbePhase::BaselineAfter),
            (36.0, ProbePhase::BaselineAfter, ProbePhase::Complete),
        ] {
            assert_eq!(ProbePhase::at(boundary - 1.0e-6), before);
            assert_eq!(ProbePhase::at(boundary), after);
        }
        for cap in [30, 60] {
            let mut ordinary = CaptureCadence::new(cap);
            let mut probe = CaptureCadence::new(cap);
            let mut ordinary_samples = Vec::new();
            let mut probe_samples = Vec::new();
            for sample in 0..40 * 40 {
                let elapsed = sample as f64 / 40.0;
                if capture_phase_enabled(false, elapsed) && ordinary.due(elapsed, 0) {
                    ordinary_samples.push(elapsed);
                    ordinary.admitted(elapsed);
                }
                if capture_phase_enabled(true, elapsed) && probe.due(elapsed, 0) {
                    probe_samples.push(elapsed);
                    probe.admitted(elapsed);
                    assert!(!probe.due(elapsed, 0));
                }
            }
            assert_eq!(ordinary_samples, captures_at_rate(40, 40, cap));
            assert_eq!(probe_samples.len(), 8 * (cap as usize).min(40));
            assert_eq!(probe_samples.first(), Some(&14.0));
            assert!(probe_samples
                .iter()
                .all(|sample| (14.0..22.0).contains(sample)));
            assert!(!capture_phase_enabled(true, 300.0));

            let mut stalled = CaptureCadence::new(cap);
            assert!(!stalled.due(14.0, MAX_READBACKS));
            assert!(stalled.due(21.97, 0));
            stalled.admitted(21.97);
            assert!(!stalled.due(21.97, 0));
            assert!(!capture_phase_enabled(true, 22.0));
        }
    }

    #[test]
    fn pacing_probe_baselines_exclude_boundaries_and_any_uncollected_work() {
        let mut probe = PacingProbe::default();
        for (elapsed, before_poll, outstanding) in [
            (5.99, 0, 0),
            (6.0, 0, 0),
            (6.025, 0, 0),
            (21.99, 1, 1),
            (23.99, 1, 1),
            (24.0, 1, 1),
            (24.025, 1, 0),
            (24.05, 0, 0),
            (24.075, 0, 0),
            (36.0, 0, 0),
        ] {
            probe.observe_update(elapsed, 25.0, 1, before_poll, outstanding, Some(true));
        }
        let eligible: Vec<_> = probe
            .trace
            .lines()
            .skip(1)
            .map(|line| line.split(',').nth(8).unwrap())
            .collect();
        assert_eq!(eligible, ["0", "0", "1", "0", "0", "0", "0", "0", "1", "0"]);
        assert!(probe.trace.lines().skip(1).all(|line| line.ends_with(",1")));
        assert_eq!(readback_drain(1, 2.0), ReadbackDrain::Pending);
        assert_eq!(readback_drain(1, 20.0), ReadbackDrain::Pending);
        assert_eq!(readback_drain(1, 20.01), ReadbackDrain::TimedOut);
        assert_eq!(readback_drain(8, 120.0), ReadbackDrain::TimedOut);
        assert_eq!(readback_drain(0, 120.0), ReadbackDrain::Complete);
    }

    #[test]
    fn pacing_probe_outputs_preserve_exact_timestamps_and_never_invent_video_holds() {
        let root = std::env::temp_dir().join(format!(
            "crystal-pacing-probe-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for enabled in [false, true] {
            let directory = root.join(if enabled { "probe" } else { "ordinary" });
            std::fs::create_dir_all(&directory).unwrap();
            let mut recording = Recording {
                directory,
                seconds: 36.0,
                capture_images: true,
                on_move: true,
                on_capture: false,
                armed_at: None,
                arm_timeout_seconds: 180.0,
                trigger: "first presented move\n".into(),
                start: Some(0.0),
                presented_battle_frames: 30,
                cadence: CaptureCadence::default(),
                frames: vec![14.0, 14.075],
                trace: "frame,seconds\n0,14.000000\n1,14.075000\n".into(),
                update_trace: "all update timestamps\n".into(),
                battle_trace: "all battle timestamps\n".into(),
                update_times: vec![25.0],
                outstanding: 0,
                completions: default(),
                probe: enabled.then(PacingProbe::default),
            };
            finish(&recording, 36.0, "complete").unwrap();
            assert_eq!(
                recording.directory.join("frames.ffconcat").exists(),
                !enabled
            );
            assert_eq!(
                std::fs::read_to_string(recording.directory.join("timestamps.csv")).unwrap(),
                recording.trace
            );
            assert_eq!(
                std::fs::read_to_string(recording.directory.join("render-timing.csv")).unwrap(),
                recording.update_trace
            );
            assert_eq!(
                std::fs::read_to_string(recording.directory.join("battle-timing.csv")).unwrap(),
                recording.battle_trace
            );
            if enabled {
                recording.outstanding = 2;
                finish(&recording, 56.01, "readback_timeout").unwrap();
                let result =
                    std::fs::read_to_string(recording.directory.join("capture-probe-result.txt"))
                        .unwrap();
                assert!(result.contains("outcome=readback_timeout\n"));
                assert!(result.contains("outstanding=2\n"));
                assert!(!recording.directory.join("frames.ffconcat").exists());
            } else {
                assert!(!recording.directory.join("capture-pacing.csv").exists());
                assert_eq!(std::fs::read_to_string(recording.directory.join("capture-settings.txt")).unwrap(),
                    "capture_images=true\nrequested_capture_hz=30\nactual timestamps; missed deadlines are dropped\n");
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn capture_arm_wait_has_a_fixed_default_and_bounded_operator_override() {
        assert_eq!(requested_arm_timeout(None).unwrap(), 180.0);
        assert_eq!(requested_arm_timeout(Some("600")).unwrap(), 600.0);
        for invalid in ["0", "29", "901", "-1", "NaN", "inf", "1.5", ""] {
            assert!(requested_arm_timeout(Some(invalid)).is_err());
        }
    }
    use crystal_render_api::{
        VisualBattleCue, VisualBattleCueKind, VisualBattleFrame, VisualBattleSide,
    };

    fn captures_at_rate(hz: usize, seconds: usize, capture_hz: u32) -> Vec<f64> {
        let mut cadence = CaptureCadence::new(capture_hz);
        let mut captured = Vec::new();
        for sample in 0..hz * seconds {
            let elapsed = sample as f64 / hz as f64;
            if cadence.due(elapsed, 0) {
                captured.push(elapsed);
                cadence.admitted(elapsed);
                assert!(!cadence.due(elapsed, 0), "one capture per observed frame");
            }
        }
        captured
    }

    #[test]
    fn capture_trigger_waits_for_capture_or_deflect_and_preserves_move_trigger() {
        let mut frame = VisualBattleFrame {
            active: true,
            ..default()
        };
        assert!(!recording_start_ready(false, true, true, true, &frame));
        for kind in [
            VisualBattleCueKind::Move,
            VisualBattleCueKind::Capture,
            VisualBattleCueKind::CaptureDeflect,
        ] {
            frame.cues = vec![VisualBattleCue {
                kind,
                side: VisualBattleSide::Enemy,
                move_id: "POKE_BALL".into(),
                element: "NORMAL".into(),
                progress: 0.0,
                damaging: false,
            }];
            assert_eq!(
                recording_start_ready(false, true, false, true, &frame),
                is_capture_cue(kind)
            );
            assert_eq!(
                recording_start_ready(true, false, false, true, &frame),
                kind == VisualBattleCueKind::Move
            );
            assert!(!recording_start_ready(false, true, false, false, &frame));
            frame.active = false;
            assert!(!recording_start_ready(false, true, false, true, &frame));
            frame.active = true;
        }
        assert!(recording_start_ready(false, false, true, false, &frame));
        assert_eq!(capture_csv_fields(None), ",,,,,,");
    }

    #[test]
    fn capture_csv_serializes_observed_state_without_reconstructing_ticks() {
        // Pure serialization fixture. Actual controller outcomes are covered by
        // immersive_capture_fixture_*; this test makes no native-play claim.
        let capture = VisibleCaptureAnimation {
            trigger_message: String::new(),
            ball_id: "MASTER_BALL".into(),
            animation_shakes: 4,
            blocked: false,
            caught: true,
            started: true,
            complete: false,
            sprites_cleared: false,
            frame: 79,
        };
        assert_eq!(
            capture_csv_fields(Some(&capture)),
            "MASTER_BALL,79,true,false,4,true,false"
        );
    }

    #[test]
    fn classic_battle_capture_arms_from_real_presented_frames() {
        assert!(!recording_battle_ready(true, 29, false, 0));
        assert!(recording_battle_ready(true, 30, false, 0));
        assert!(!recording_battle_ready(false, 300, false, 0));
        assert!(!recording_battle_ready(true, 300, true, 29));
        assert!(recording_battle_ready(true, 300, true, 30));
        // Source fallback during an effect is already a warmed presentation.
        assert!(recording_battle_ready(true, 301, false, 0));
        let mut frame = VisualBattleFrame {
            active: true,
            ..default()
        };
        assert!(!recording_start_ready(true, false, false, true, &frame));
        frame.cues.push(VisualBattleCue {
            kind: VisualBattleCueKind::Move,
            side: VisualBattleSide::Player,
            move_id: "SURF".into(),
            element: "WATER".into(),
            progress: 0.0,
            damaging: true,
        });
        assert!(recording_start_ready(true, false, false, true, &frame));
    }

    #[test]
    fn capture_deadlines_preserve_requested_cap_and_real_render_samples() {
        for (capture_hz, hz) in [30, 60]
            .into_iter()
            .flat_map(|cap| [30, 40, 60, 120].map(|hz| (cap, hz)))
        {
            let captures = captures_at_rate(hz, 4, capture_hz);
            assert_eq!(
                captures.len(),
                hz.min(capture_hz as usize) * 4,
                "{hz}Hz renderer must retain the requested {capture_hz}Hz cap"
            );
            assert!(captures.windows(2).all(|times| times[0] < times[1]));
            for (deadline, &actual) in captures.iter().enumerate() {
                if hz >= capture_hz as usize {
                    let scheduled = deadline as f64 / f64::from(capture_hz);
                    assert!(actual + CAPTURE_TIME_EPSILON >= scheduled);
                    assert!(actual - scheduled < 1.0 / hz as f64 + CAPTURE_TIME_EPSILON);
                }
                // These are real render sample timestamps, never a retimed
                // 30Hz output grid. 40Hz deliberately has unequal gaps.
                assert!((actual * hz as f64 - (actual * hz as f64).round()).abs() < 1.0e-8);
            }
        }
        let captures = captures_at_rate(40, 1, 30);
        let gaps: Vec<_> = captures
            .windows(2)
            .map(|times| times[1] - times[0])
            .collect();
        assert!(gaps.iter().any(|gap| (*gap - 0.025).abs() < 1.0e-8));
        assert!(gaps.iter().any(|gap| (*gap - 0.050).abs() < 1.0e-8));
        let captures = captures_at_rate(40, 1, 60);
        assert!(captures
            .windows(2)
            .all(|times| (times[1] - times[0] - 0.025).abs() < 1.0e-8));
    }

    #[test]
    fn capture_deadlines_drop_missed_frames_after_stalls_and_backpressure() {
        for cap in [30, 60] {
            let mut cadence = CaptureCadence::new(cap);
            assert!(cadence.due(0.0, 0));
            cadence.admitted(0.0);
            assert!(!cadence.due(0.01, 0));
            assert!(!cadence.due(0.05, MAX_READBACKS));
            assert!(!cadence.due(0.75, MAX_READBACKS));
            assert!(cadence.due(1.01, MAX_READBACKS - 1));
            cadence.admitted(1.01);
            assert!(!cadence.due(1.01, 0));
            assert!(!cadence.due(1.011, 0));
            assert!(cadence.due(1.04, 0));
            cadence.admitted(1.04);
            assert!(!cadence.due(1.04, 0));
            assert!(!cadence.due(1.041, 0));
            assert!(cadence.due(1.07, 0));
        }
    }

    #[test]
    fn capture_cap_defaults_to_30_and_accepts_only_bounded_explicit_rates() {
        assert_eq!(requested_capture_hz(None).unwrap(), 30);
        assert_eq!(CaptureCadence::default().requested_hz, 30);
        assert_eq!(requested_capture_hz(Some("30")).unwrap(), 30);
        assert_eq!(requested_capture_hz(Some("60")).unwrap(), 60);
        for invalid in ["0", "120", "59.94", "unlimited"] {
            assert!(requested_capture_hz(Some(invalid)).is_err());
        }
    }
    #[test]
    fn immersive_battle_record_arm_requires_an_active_presented_move() {
        let mut frame = VisualBattleFrame {
            active: true,
            ..default()
        };
        assert!(!recording_start_ready(true, false, true, true, &frame));
        for kind in [
            VisualBattleCueKind::Impact,
            VisualBattleCueKind::Faint,
            VisualBattleCueKind::SendOut,
            VisualBattleCueKind::Withdraw,
            VisualBattleCueKind::Capture,
            VisualBattleCueKind::CaptureDeflect,
            VisualBattleCueKind::Move,
        ] {
            frame.cues = vec![VisualBattleCue {
                kind,
                side: VisualBattleSide::Player,
                move_id: "TACKLE".into(),
                element: "NORMAL".into(),
                progress: 0.25,
                damaging: true,
            }];
            assert_eq!(
                recording_start_ready(true, false, true, true, &frame),
                kind == VisualBattleCueKind::Move
            );
            assert!(!recording_start_ready(true, false, true, false, &frame));
        }
        frame.active = false;
        assert!(!recording_start_ready(true, false, true, true, &frame));
    }
    #[test]
    fn immersive_battle_default_record_and_measure_readiness_is_unchanged() {
        let frame = VisualBattleFrame::default();
        assert!(recording_start_ready(false, false, true, false, &frame));
        assert!(recording_start_ready(false, false, false, true, &frame));
        assert!(!recording_start_ready(false, false, false, false, &frame));
    }
}
