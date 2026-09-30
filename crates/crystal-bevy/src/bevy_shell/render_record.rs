//! Opt-in actual-game capture and no-readback timing. Never drives game input.
//! Timestamps retain wall time; no frame interpolation or retiming is performed.
use super::*;
use image::ImageEncoder;
use std::sync::{Arc, Mutex};

type Completions = Arc<Mutex<Vec<Result<(), String>>>>;
const MAX_READBACKS: usize = 8;
const CAPTURE_INTERVAL: f64 = 1.0 / 30.0;
const ARM_TIMEOUT_SECONDS: f64 = 180.0;

#[derive(Resource)]
struct Recording {
    directory: PathBuf,
    seconds: f64,
    capture_images: bool,
    on_move: bool,
    armed_at: Option<f64>,
    trigger: String,
    start: Option<f64>,
    last: f64,
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
        last: -1.0,
        frames: vec![],
        trace: header.into(),
        update_trace: header.into(),
        battle_trace: "frame,seconds,map,player_species,enemy_species,cues,modeled,source_art,source_frame,bgp,source_objects,flash_mode\n"
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
    windows: Query<Entity, With<PrimaryWindow>>,
    mut screenshots: ResMut<ScreenshotManager>,
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
    if recording.start.is_none() {
        let now = time.elapsed_seconds_f64();
        let armed_at = *recording.armed_at.get_or_insert(now);
        let world_ready = status.active && status.active_frames >= 30 && !status.profiles_pending;
        let battle_ready = battle_status.active && battle_status.active_frames >= 30;
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
        recording.battle_trace.push_str(&format!(
            "{index},{elapsed:.6},{},{},{},{},{},{},{},{},{},{:?}\n",
            battle_frame.map_id,
            species(0),
            species(1),
            cues,
            battle_status.modeled_species.join("|"),
            battle_status.source_art_species.join("|"),
            source_frame,
            bgp,
            source_objects,
            *flash_mode
        ));
    }
    if !recording.capture_images
        || recording.outstanding >= MAX_READBACKS
        || elapsed - recording.last < CAPTURE_INTERVAL
    {
        return;
    }
    let Ok(window) = windows.get_single() else {
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
        recording.last = elapsed;
        recording.outstanding += 1;
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
            "{} updates; median {:.2}ms, p95 {:.2}ms; {} captured frames over {:.2}s\n",
            times.len(),
            times[times.len() / 2],
            times[times.len() * 95 / 100],
            recording.frames.len(),
            elapsed
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
