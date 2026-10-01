//! Opt-in actual-game capture and no-readback timing. Never drives game input.
//! Timestamps retain wall time; no frame interpolation or retiming is performed.
use super::*;
use image::ImageEncoder;
use std::sync::{Arc, Mutex};

type Completions = Arc<Mutex<Vec<Result<(), String>>>>;
const MAX_READBACKS: usize = 8;
const DEFAULT_CAPTURE_HZ: u32 = 30;
const ARM_TIMEOUT_SECONDS: f64 = 180.0;
const CAPTURE_TIME_EPSILON: f64 = 1.0e-9;

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

#[derive(Resource)]
struct Recording {
    directory: PathBuf,
    seconds: f64,
    capture_images: bool,
    on_move: bool,
    armed_at: Option<f64>,
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
}

pub(super) fn install(
    app: &mut App,
    directory: &Path,
    seconds: u32,
    capture_images: bool,
    on_move: bool,
) -> Result<()> {
    anyhow::ensure!(
        (1..=300).contains(&seconds),
        "measurement must be 1–300 seconds"
    );
    let capture_hz = requested_capture_hz(std::env::var("CRYSTAL_CAPTURE_FPS").ok().as_deref())?;
    std::fs::create_dir_all(directory)?;
    anyhow::ensure!(
        std::fs::read_dir(directory)?.next().is_none(),
        "recording output directory must be empty"
    );
    let header = "frame,seconds,map,origin_x,origin_y,player_x,player_y\n";
    app.insert_resource(Recording {
        directory: directory.to_owned(),
        seconds: f64::from(seconds),
        capture_images,
        on_move,
        armed_at: None,
        trigger: String::new(),
        start: None,
        presented_battle_frames: 0,
        cadence: CaptureCadence::new(capture_hz),
        frames: vec![],
        trace: header.into(),
        update_trace: header.into(),
        battle_trace: "frame,seconds,map,player_species,enemy_species,cues,modeled,source_art,source_frame,bgp,source_objects,flash_mode,image_assets,mesh_assets,material_assets,lighting,quality,software_renderer,scene_width,scene_height,window_width,window_height,requested_capture_hz,view_mode,source_line_x,source_line_y,player_size_m,enemy_size_m,row_capture_ready,row_capture_pending\n"
            .into(),
        update_times: vec![],
        outstanding: 0,
        completions: default(),
    })
    .add_systems(
        Update,
        record.after(crystal_render_api::WorldRenderSet::RenderSync),
    );
    Ok(())
}

fn row(index: usize, elapsed: f64, frame: &crystal_render_api::VisualWorldFrame) -> String {
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
        "{index},{elapsed:.6},{},{},{},{:.3},{:.3}\n",
        frame.map_id, frame.grid_origin.x, frame.grid_origin.y, player.x, player.y
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
    let completed = std::mem::take(&mut *recording.completions.lock().unwrap());
    for result in completed {
        if let Err(error) = result {
            eprintln!("capture failed: {error}");
            exit.send(AppExit::error());
            return;
        }
        recording.outstanding = recording.outstanding.saturating_sub(1);
    }
    recording.presented_battle_frames = if battle_frame.active {
        recording.presented_battle_frames.saturating_add(1)
    } else {
        0
    };
    if recording.start.is_none() {
        let now = time.elapsed_seconds_f64();
        let armed_at = *recording.armed_at.get_or_insert(now);
        let world_ready = status.active && status.active_frames >= 30 && !status.profiles_pending;
        let battle_ready = recording_battle_ready(
            battle_frame.active,
            recording.presented_battle_frames,
            battle_status.active,
            battle_status.active_frames,
        );
        if !recording_start_ready(recording.on_move, world_ready, battle_ready, &battle_frame) {
            if recording.on_move && now - armed_at >= ARM_TIMEOUT_SECONDS {
                eprintln!(
                    "capture cancelled: no presented battle move observed within {ARM_TIMEOUT_SECONDS}s"
                );
                exit.send(AppExit::error());
            }
            return;
        }
        if recording.on_move {
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
        }
        recording.start = Some(now);
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
        if recording.outstanding > 0 {
            if elapsed > recording.seconds + 20.0 {
                eprintln!("capture timed out waiting for GPU readback");
                exit.send(AppExit::error());
            }
            return;
        }
        let result = finish(&recording, elapsed);
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
    recording
        .update_trace
        .push_str(&row(index, elapsed, &frame));
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
            "{index},{elapsed:.6},{},{},{},{},{},{},{},{},{},{:?},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
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
        ));
    }
    if !recording.capture_images || !recording.cadence.due(elapsed, recording.outstanding) {
        return;
    }
    let Ok((window, _)) = windows.get_single() else {
        return;
    };
    let index = recording.frames.len();
    let path = recording.directory.join(format!("frame-{index:05}.png"));
    let completions = recording.completions.clone();
    // Screenshot callbacks already run on Bevy's async compute pool. Allow a
    // bounded pipeline rather than waiting 2–3 render frames after each one.
    if screenshots
        .take_screenshot(window, move |image| {
            let result = (|| {
                let rgb = image
                    .try_into_dynamic()
                    .map_err(|error| format!("decode GPU frame: {error}"))?
                    .to_rgb8();
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
            completions.lock().unwrap().push(result);
        })
        .is_ok()
    {
        recording.frames.push(elapsed);
        recording.trace.push_str(&row(index, elapsed, &frame));
        recording.cadence.admitted(elapsed);
        recording.outstanding += 1;
    }
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

fn recording_start_ready(
    on_move: bool,
    world_ready: bool,
    battle_ready: bool,
    frame: &crystal_render_api::VisualBattleFrame,
) -> bool {
    if !on_move {
        return world_ready || battle_ready;
    }
    battle_ready
        && frame.active
        && frame
            .cues
            .iter()
            .any(|cue| cue.kind == crystal_render_api::VisualBattleCueKind::Move)
}

fn finish(recording: &Recording, elapsed: f64) -> std::io::Result<()> {
    std::fs::write(
        recording.directory.join("capture-settings.txt"),
        format!(
            "capture_images={}\nrequested_capture_hz={}\nactual timestamps; missed deadlines are dropped\n",
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
    if recording.capture_images {
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
        std::fs::write(recording.directory.join("timestamps.csv"), &recording.trace)?;
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
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(!recording_start_ready(true, false, true, &frame));
        frame.cues.push(VisualBattleCue {
            kind: VisualBattleCueKind::Move,
            side: VisualBattleSide::Player,
            move_id: "SURF".into(),
            element: "WATER".into(),
            progress: 0.0,
            damaging: true,
        });
        assert!(recording_start_ready(true, false, true, &frame));
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
        assert!(
            captures
                .windows(2)
                .all(|times| (times[1] - times[0] - 0.025).abs() < 1.0e-8)
        );
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
        assert!(!recording_start_ready(true, true, true, &frame));
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
                recording_start_ready(true, true, true, &frame),
                kind == VisualBattleCueKind::Move
            );
            assert!(!recording_start_ready(true, true, false, &frame));
        }
        frame.active = false;
        assert!(!recording_start_ready(true, true, true, &frame));
    }
    #[test]
    fn immersive_battle_default_record_and_measure_readiness_is_unchanged() {
        let frame = VisualBattleFrame::default();
        assert!(recording_start_ready(false, true, false, &frame));
        assert!(recording_start_ready(false, false, true, &frame));
        assert!(!recording_start_ready(false, false, false, &frame));
    }
}
