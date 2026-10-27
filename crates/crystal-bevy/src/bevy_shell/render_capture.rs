//! Automated captures share a GPU session and finish on readback, not a timer.
use super::*;
// Location-test startup is also available in local browser builds.
use bevy::platform::time::Instant;
use std::sync::{Arc, Mutex};

fn image_capture_target(value: Option<&str>, live: bool) -> Result<bool> {
    match value {
        None => Ok(!live && !cfg!(target_arch = "wasm32")),
        Some("image") => Ok(true),
        Some("window") => Ok(false),
        _ => anyhow::bail!("CRYSTAL_RENDER_TEST_CAPTURE_TARGET must be window or image"),
    }
}

pub(super) fn requested_image_target(live: bool) -> Result<bool> {
    match std::env::var("CRYSTAL_RENDER_TEST_CAPTURE_TARGET") {
        Ok(value) => image_capture_target(Some(&value), live),
        Err(std::env::VarError::NotPresent) => image_capture_target(None, live),
        Err(_) => anyhow::bail!("CRYSTAL_RENDER_TEST_CAPTURE_TARGET must be window or image"),
    }
}

type Completion = Arc<Mutex<Option<Result<(), String>>>>;

// Only passive, paired visual verification holds simulation. Live reviews,
// movement and same-view temporal checks keep the production clock running.
#[derive(Resource, Default)]
pub(super) struct PairedCaptureHold {
    pub(super) frozen: bool,
}

#[derive(Resource)]
struct EntryFrameHold { after: u16, held: bool }

fn requested_entry_after(value: Option<&str>) -> Result<Option<u16>> {
    value.map(|value| {
        let frame: u16 = value.parse().context("entry hold must be an integer source frame")?;
        anyhow::ensure!((2..=180).contains(&frame), "entry hold frame must be 2–180");
        Ok(frame)
    }).transpose()
}

fn install_entry_hold(app: &mut App, after: Option<u16>) {
    if let Some(after) = after {
        app.insert_resource(EntryFrameHold { after, held: false })
            .add_systems(PreUpdate, hold_presented_entry
                .after(bevy::input::InputSystems).before(super::render_walk::drive));
    }
}

fn hold_presented_entry(
    mut request: ResMut<EntryFrameHold>,
    runtime: Res<BevyRuntimeShell>,
    mut walk: Option<ResMut<super::render_walk::RenderWalk>>,
    mut hold: ResMut<PairedCaptureHold>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut exit: MessageWriter<AppExit>,
) {
    if request.held { return; }
    let Some(walk) = walk.as_mut() else {
        eprintln!("entry capture requires --walk and ordinary production input");
        exit.write(AppExit::error()); return;
    };
    let Some(transition) = runtime.visible_battle_transition.filter(|_| walk.started()) else { return; };
    if request.after >= visible_battle_transition_total_frames(&transition) {
        eprintln!("entry hold lies beyond this source transition");
        exit.write(AppExit::error()); return;
    }
    if transition.frame < request.after { return; }
    // Pin the actually observed frame; never manufacture or rewind the clock.
    if let Err(error) = walk.hold_for_source_capture() {
        eprintln!("entry capture could not pin its source frame: {error:#}");
        exit.write(AppExit::error()); return;
    }
    request.held = true;
    keys.reset_all(); hold.frozen = true; virtual_time.pause();
}

#[derive(Resource)]
struct SendOutFrameHold {
    target: u8,
    held: Option<HeldSendOut>,
}

struct HeldSendOut {
    animation: VisibleSendOutAnimation,
    presented: crystal_render_api::VisualBattleFrame,
    checksum: StateChecksum,
    lcd_frame: u64,
}

fn requested_send_out_frame(value: Option<&str>) -> Result<Option<u8>> {
    match value {
        None => Ok(None),
        Some("4") => Ok(Some(4)),
        Some("5") => Ok(Some(5)),
        _ => anyhow::bail!("CRYSTAL_RENDER_TEST_HOLD_SEND_OUT_FRAME must be 4 or 5"),
    }
}

fn presented_send_out_matches(target: u8, animation: Option<&VisibleSendOutAnimation>,
    frame: &crystal_render_api::VisualBattleFrame) -> bool {
    let Some(animation) = animation else { return false; };
    let side = if animation.side == crate::core::battle::turn::BattleSide::Player {
        VisualBattleSide::Player
    } else { VisualBattleSide::Enemy };
    animation.frame == target && frame.active && !frame.use_source_scene && frame.validate().is_ok()
        && frame.source.as_ref().is_some_and(|source| source.frame == u16::from(target))
        && frame.cues.iter().any(|cue| cue.kind == VisualBattleCueKind::SendOut
            && cue.side == side && cue.move_id.as_ref() == "SEND_OUT")
}

fn install_send_out_hold(app: &mut App, target: Option<u8>) {
    if let Some(target) = target {
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0)))
            .insert_resource(SendOutFrameHold { target, held: None })
            .add_systems(PreUpdate, hold_presented_send_out
                .after(bevy::input::InputSystems).before(super::render_walk::drive));
    }
}

fn hold_presented_send_out(
    mut request: ResMut<SendOutFrameHold>,
    runtime: Res<BevyRuntimeShell>,
    frame: Res<crystal_render_api::VisualBattleFrame>,
    mut walk: Option<ResMut<super::render_walk::RenderWalk>>,
    mut hold: ResMut<PairedCaptureHold>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut exit: MessageWriter<AppExit>,
) {
    if request.held.is_some() { return; }
    let Some(walk) = walk.as_mut() else {
        eprintln!("send-out capture requires --walk and an ordinary input route");
        exit.write(AppExit::error());
        return;
    };
    if !walk.started() || !presented_send_out_matches(request.target,
        runtime.visible_send_out_animation.as_ref(), &frame) { return; }
    let pinned = (|| -> Result<HeldSendOut> {
        walk.hold_for_source_capture()?;
        Ok(HeldSendOut {
            animation: runtime.visible_send_out_animation.as_ref().unwrap().clone(),
            presented: frame.clone(),
            checksum: runtime.shell.snapshot()?.state_checksum,
            lcd_frame: runtime.lcd_animation_frame,
        })
    })();
    match pinned {
        Ok(pinned) => {
            request.held = Some(pinned);
            keys.reset_all();
            hold.frozen = true;
            virtual_time.pause();
        }
        Err(error) => {
            eprintln!("send-out capture could not pin its presented state: {error:#}");
            exit.write(AppExit::error());
        }
    }
}

#[derive(PartialEq)]
pub(super) struct PairedState {
    checksum: StateChecksum,
    lcd_frame: u64,
    actors: Vec<crystal_render_api::VisualActor>,
}

#[derive(Default)]
struct StablePairedState {
    last: Option<PairedState>,
    repetitions: u8,
}
impl StablePairedState {
    fn observe(&mut self,state:PairedState)->Option<PairedState> {
        self.repetitions=if self.last.as_ref()==Some(&state) {self.repetitions.saturating_add(1)}else{1};
        if self.repetitions>=3 {self.last=None;Some(state)}else{self.last=Some(state);None}
    }
}

pub(super) fn paired_state(runtime: &BevyRuntimeShell, frame: &crystal_render_api::VisualWorldFrame) -> Result<PairedState> {
    Ok(PairedState {
        checksum: runtime.shell.snapshot()?.state_checksum,
        lcd_frame: runtime.lcd_animation_frame,
        // Compare the source actors shared by both views. The optional 3D
        // companion has no 2D entity or authoritative state to compare.
        actors: frame.actors.iter()
            .filter(|actor| !matches!(actor.id, crystal_render_api::VisualActorId::PartyFollower
                | crystal_render_api::VisualActorId::SurfMount))
            .cloned().collect(),
    })
}

// Startup and post-route readback have independent bounded waits. A finite
// ordinary input route can include long battles/credits and owns its duration.
fn capture_timeout_elapsed(
    startup: Instant,
    readback_started: &mut Option<Instant>,
    route: Option<(bool, bool)>,
    now: Instant,
) -> Option<std::time::Duration> {
    match route {
        Some((true, false)) => None,
        Some((true, true)) => Some(now.duration_since(*readback_started.get_or_insert(now))),
        _ => Some(now.duration_since(startup)),
    }
}

#[test]
fn capture_deadline_gives_a_long_completed_route_its_full_readback_window() {
    let startup=Instant::now();
    let later=startup+std::time::Duration::from_secs(500);
    let mut readback=None;
    assert_eq!(capture_timeout_elapsed(startup,&mut readback,Some((true,false)),later),None);
    assert_eq!(capture_timeout_elapsed(startup,&mut readback,Some((true,true)),later),
        Some(std::time::Duration::ZERO));
    assert_eq!(capture_timeout_elapsed(startup,&mut readback,Some((true,true)),
        later+std::time::Duration::from_secs(61)),Some(std::time::Duration::from_secs(61)));
}

#[test]
fn capture_deadline_still_bounds_static_and_unstarted_route_startup() {
    let startup=Instant::now();
    let later=startup+std::time::Duration::from_secs(61);
    for route in [None,Some((false,false))] {
        let mut readback=None;
        assert_eq!(capture_timeout_elapsed(startup,&mut readback,route,later),
            Some(std::time::Duration::from_secs(61)));
        assert!(readback.is_none());
    }
}

#[derive(Resource)]
pub(super) struct Capture {
    path: PathBuf,
    second: Option<PathBuf>,
    same_second_view: bool,
    frame: u32,
    requested: bool,
    completion: Completion,
    started: Instant,
    timeout_seconds: u64,
    route_readback_started: Option<Instant>,
    live: bool,
    idle: bool,
    first_path: PathBuf,
    second_path: Option<PathBuf>,
    request_path: PathBuf,
    previous: Option<(String, image::RgbImage)>,
    identity: String,
    image_target_mode: bool,
    recording_target_only: bool,
    preserve_alpha: bool,
    image_target: Option<bevy::camera::ImageRenderTarget>,
    image_size: UVec2,
    paired_state: Option<PairedState>,
    // Followers are presentation-only and excluded from paired-state equality.
    // Retain the actual 3D actor list so a loaded map cannot masquerade as a
    // successful species review when the requested companion is absent.
    world_actor_sources: Option<serde_json::Value>,
    paired_stability: StablePairedState,
}

pub(super) fn install(
    app: &mut App,
    path: PathBuf,
    second: Option<PathBuf>,
    live: bool,
) -> Result<()> {
    if let Ok(value) = std::env::var("CRYSTAL_RENDER_TEST_CAMERA") {
        let camera = parse_camera_request(&value)?;
        app.add_systems(Startup, move |mut settings: ResMut<crystal_voxel_view::VoxelViewSettings>| {
            settings.camera = camera;
        });
    }
    if let Ok(value) = std::env::var("CRYSTAL_RENDER_TEST_FOV_DEGREES") {
        let degrees=value.parse::<f32>().context("capture field of view must be a number")?;
        anyhow::ensure!(degrees.is_finite() && (30. ..=120.).contains(&degrees),
            "capture field of view must be between 30 and 120 degrees");
        // Tall landmark QA can widen only the active 3D capture camera.
        // Production controls, the paired 2D camera and game state stay intact.
        app.add_systems(Update,(move |mut cameras:Query<(&Camera,&mut Projection),With<Camera3d>>| {
            for (camera,mut projection) in &mut cameras {
                if camera.is_active && let Projection::Perspective(p)=&mut *projection {
                    p.fov=degrees.to_radians();
                }
            }
        }).after(crystal_render_api::WorldRenderSet::RenderSync).before(capture));
    }
    let timeout_seconds = match std::env::var("CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS") {
        Err(std::env::VarError::NotPresent) => 60,
        Ok(value) => {
            let seconds = value.parse::<u64>().map_err(|_| anyhow::anyhow!(
                "CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS must be an integer between 60 and 300"))?;
            anyhow::ensure!((60..=300).contains(&seconds),
                "CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS must be between 60 and 300");
            seconds
        }
        Err(_) => anyhow::bail!(
            "CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS must be an integer between 60 and 300"),
    };
    let same_second_view = match std::env::var("CRYSTAL_RENDER_TEST_SECOND_VIEW") {
        Err(std::env::VarError::NotPresent) => false,
        Ok(value) if value == "toggle" => false,
        Ok(value) if value == "same" => true,
        _ => anyhow::bail!("CRYSTAL_RENDER_TEST_SECOND_VIEW must be toggle or same"),
    };
    anyhow::ensure!(
        !same_second_view || !live,
        "same-view temporal captures require one-shot verification mode"
    );
    let image_target_mode = requested_image_target(live)?;
    anyhow::ensure!(
        !image_target_mode || !live,
        "image-target capture is a one-shot verification mode; live review requires window capture"
    );
    let preserve_alpha = match std::env::var("CRYSTAL_RENDER_TEST_CAPTURE_CHANNELS") {
        Err(std::env::VarError::NotPresent) => false,
        Ok(value) if value == "rgb" => false,
        Ok(value) if value == "rgba" => true,
        _ => anyhow::bail!("CRYSTAL_RENDER_TEST_CAPTURE_CHANNELS must be rgb or rgba"),
    };
    #[cfg(not(target_arch = "wasm32"))]
    let send_out_frame = match std::env::var("CRYSTAL_RENDER_TEST_HOLD_SEND_OUT_FRAME") {
        Ok(value) => requested_send_out_frame(Some(&value))?,
        Err(std::env::VarError::NotPresent) => None,
        Err(error) => return Err(error).context("read requested send-out frame"),
    };
    #[cfg(target_arch = "wasm32")]
    let send_out_frame = None;
    if send_out_frame.is_some() {
        anyhow::ensure!(!live && !same_second_view && second.is_some(),
            "send-out frame verification requires a one-shot matched 3D/2D capture");
        anyhow::ensure!(app.world().resource::<BevyRuntimeShell>().quick_save_path.is_none(),
            "send-out frame verification requires a disposable session without a save path");
    }
    let state_receipt = path.with_extension("paired-state.txt");
    for path in std::iter::once(&path).chain(second.iter()).chain(std::iter::once(&state_receipt)) {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        // A failed capture must never validate a previous run's image.
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    let request_path = path.with_extension("request");
    if live {
        println!(
            "live review: F6 captures again; external trigger: {} (empty, or zoom and orbit numbers)",
            request_path.display()
        );
    }
    app.init_resource::<PairedCaptureHold>().insert_resource(Capture {
        live,
        idle: false,
        first_path: path.clone(),
        second_path: second.clone(),
        request_path,
        previous: None,
        identity: String::new(),
        image_target_mode,
        recording_target_only: false,
        preserve_alpha,
        image_target: None,
        image_size: UVec2::ZERO,
        paired_state: None,
        world_actor_sources: None,
        paired_stability: StablePairedState::default(),
        path,
        second,
        same_second_view,
        frame: 0,
        requested: false,
        completion: default(),
        started: Instant::now(),
        timeout_seconds,
        route_readback_started: None,
    })
    .add_systems(Update, (prepare_image_target, capture).chain()
        .after(crystal_render_api::WorldRenderSet::RenderSync));
    let entry_after = requested_entry_after(std::env::var("CRYSTAL_RENDER_TEST_HOLD_ENTRY_AFTER").ok().as_deref())?;
    anyhow::ensure!(entry_after.is_none() || send_out_frame.is_none(), "choose entry or send-out source hold");
    install_send_out_hold(app, send_out_frame);
    install_entry_hold(app, entry_after);
    Ok(())
}

// A recording needs the explicit GPU image route, but must not start a
// one-shot capture that freezes the move or waits for the input route to end.
pub(super) fn install_recording_target(app: &mut App, path: PathBuf) -> Result<()> {
    install(app, path, None, false)?;
    app.world_mut().resource_mut::<Capture>().recording_target_only = true;
    Ok(())
}

// Moving captures must use the same explicitly selected target and scale as
// the final readback. An unprepared image target never falls back to a window.
pub(super) fn moving_screenshot(capture: Option<&Capture>, window: Entity) -> Option<Screenshot> {
    if let Some(capture) = capture.filter(|capture| capture.image_target_mode) {
        return capture.image_target.as_ref().map(|target|
            Screenshot(bevy::camera::RenderTarget::Image(target.clone())));
    }
    Some(Screenshot::window(window))
}

// Explicit verification target, never a retry after a failed window capture.
// Keep production cameras, render order, viewport and projection. Other image
// targets (including battle rows) retain their existing ownership.
fn prepare_image_target(
    mut capture: ResMut<Capture>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut targets: Query<(Entity, &mut bevy::camera::RenderTarget, Has<Camera2d>), With<Camera>>,
    mut commands: Commands,
) {
    if !capture.image_target_mode {
        return;
    }
    let Capture { image_target, image_size, .. } = &mut *capture;
    if prepare_image_target_for_cameras(
        image_target, image_size, &windows, &mut images, &mut targets, &mut commands,
    ) {
        capture.frame = 0;
    }
}

// Timing validation shares the exact camera routing and scale used by authored
// captures. This always renders the production cameras on the GPU.
pub(super) fn prepare_image_target_for_cameras(
    image_target: &mut Option<bevy::camera::ImageRenderTarget>,
    image_size: &mut UVec2,
    windows: &Query<(Entity, &Window), With<PrimaryWindow>>,
    images: &mut Assets<Image>,
    targets: &mut Query<(Entity, &mut bevy::camera::RenderTarget, Has<Camera2d>), With<Camera>>,
    commands: &mut Commands,
) -> bool {
    let Ok((window_entity, window)) = windows.single() else {
        return false;
    };
    let size = UVec2::new(window.physical_width(), window.physical_height());
    if size.x == 0 || size.y == 0 {
        return false;
    }
    let scale_factor = window.resolution.scale_factor();
    let previous = image_target.clone();
    let resized = image_target.is_none()
        || *image_size != size
        || image_target
            .as_ref()
            .is_some_and(|target| target.scale_factor != scale_factor);
    if resized {
        let handle = images.add(Image::new_target_texture(
            size.x,
            size.y,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            None,
        ));
        *image_target = Some(bevy::camera::ImageRenderTarget {
            handle,
            scale_factor,
        });
        *image_size = size;
        println!(
            "render-test explicit image target: {}x{} scale {}",
            size.x, size.y, scale_factor
        );
    }
    let Some(image) = image_target.as_ref() else {
        return false;
    };
    for (entity, mut target, is_hud_camera) in targets.iter_mut() {
        let selected = match &*target {
            bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Primary) => true,
            bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(entity)) => {
                *entity == window_entity
            }
            bevy::camera::RenderTarget::Image(current) => previous
                .as_ref()
                .is_some_and(|old| old.handle == current.handle),
            _ => false,
        };
        if selected
            && !matches!(&*target,
            bevy::camera::RenderTarget::Image(current) if current == image)
        {
            *target = bevy::camera::RenderTarget::Image(image.clone());
            if is_hud_camera {
                commands
                    .entity(entity)
                    .insert(crystal_voxel_view::BattleCompositeImageCamera);
            }
        }
    }
    resized
}

fn capture(
    mut capture: ResMut<Capture>,
    mut hold: ResMut<PairedCaptureHold>,
    mut virtual_time: ResMut<Time<Virtual>>,
    runtime: Res<BevyRuntimeShell>,
    walk: Option<Res<render_walk::RenderWalk>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    frame: Res<crystal_render_api::VisualWorldFrame>,
    status: Res<crystal_voxel_view::VoxelViewStatus>,
    battle_status: Res<crystal_voxel_view::BattleViewStatus>,
    battle_frame: Res<crystal_render_api::VisualBattleFrame>,
    review_proofs: (Option<Res<SendOutFrameHold>>,Option<Res<VerifiedPcLcdCapture>>),
    mut settings: ResMut<crystal_voxel_view::VoxelViewSettings>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    move_menu: Query<(), With<CompetitiveMoveMenuMarker>>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    let (send_out_hold,verified_pc_lcd)=review_proofs;
    if let Some(error) = runtime.render_test_error.as_ref() {
        eprintln!("capture rejected after runtime error: {error}");
        exit.write(AppExit::error());
        return;
    }
    if let Some(request) = &send_out_hold {
        if capture.started.elapsed().as_secs() > capture.timeout_seconds {
            eprintln!("send-out frame {} capture timed out; map: {:?}",
                request.target, battle_status.map_scenery_fallback_reason);
            exit.write(AppExit::error());
            return;
        }
        let Some(pinned) = &request.held else { return; };
        let preserved = runtime.visible_send_out_animation.as_ref() == Some(&pinned.animation)
            && *battle_frame == pinned.presented
            && runtime.lcd_animation_frame == pinned.lcd_frame
            && runtime.shell.snapshot().is_ok_and(|snapshot| snapshot.state_checksum == pinned.checksum);
        if !preserved {
            eprintln!("send-out capture rejected: pinned source or authoritative state changed");
            exit.write(AppExit::error());
            return;
        }
    }
    // Check throughout both readbacks, including their completion update.
    // A late mutation must not be accepted just because requests matched.
    if let Some(first) = &capture.paired_state {
        match paired_state(&runtime, &frame) {
            Ok(current) if first == &current => {}
            Ok(current) => {
                eprintln!("paired capture rejected: state changed during GPU readback; checksum {:?} -> {:?}; lcd {} -> {}; actors_equal={}, actor_count {} -> {}",
                    first.checksum, current.checksum, first.lcd_frame, current.lcd_frame,
                    first.actors == current.actors, first.actors.len(), current.actors.len());
                for before in &first.actors {
                    let after=current.actors.iter().find(|actor|actor.id==before.id);
                    if after!=Some(before) {
                        eprintln!("paired capture actor changed: before={before:?}; after={after:?}");
                    }
                }
                exit.write(AppExit::error());
                return;
            }
            Err(error) => {
                eprintln!("paired capture rejected: state unavailable during GPU readback: {error}");
                exit.write(AppExit::error());
                return;
            }
        }
    }
    if capture.recording_target_only { return; }
    if capture.idle {
        let external = match std::fs::read_to_string(&capture.request_path) {
            Ok(value) => {
                let _ = std::fs::remove_file(&capture.request_path);
                Some(value)
            }
            Err(_) => None,
        };
        if !keyboard.just_pressed(KeyCode::F6) && external.is_none() {
            return;
        }
        if let Some(value) = external.filter(|value| !value.trim().is_empty()) {
            match parse_camera_request(&value) {
                Ok(camera) => settings.camera = camera,
                Err(error) => {
                    eprintln!("review request: {error}");
                    return;
                }
            }
        }
        capture.path = capture.first_path.clone();
        capture.second = capture.second_path.clone();
        capture.requested = false;
        capture.frame = 0;
        capture.started = Instant::now();
        capture.route_readback_started = None;
        capture.identity.clear();
        capture.idle = false;
        for path in std::iter::once(&capture.first_path).chain(capture.second_path.iter()) {
            if let Err(error) = std::fs::remove_file(path) {
                if error.kind() != std::io::ErrorKind::NotFound {
                    eprintln!("cannot clear previous capture: {error}");
                    exit.write(AppExit::error());
                    return;
                }
            }
        }
        settings.enabled = false;
    }
    let completed = capture.completion.lock().unwrap().take();
    if let Some(result) = completed {
        if let Err(error) = result {
            eprintln!("capture failed: {error}");
            exit.write(AppExit::error());
            return;
        }
        println!(
            "wrote {} in {:.2}s",
            capture.path.display(),
            capture.started.elapsed().as_secs_f64()
        );
        if let Some(second) = capture.second.take() {
            capture.path = second;
            capture.frame = 0;
            capture.requested = false;
            if !capture.same_second_view {
                settings.enabled = !settings.enabled;
            }
        } else if capture.live {
            if let Err(error) = live_comparison(&mut capture) {
                eprintln!("review failed: {error:#}");
                exit.write(AppExit::error());
                return;
            }
            capture.idle = true;
            println!(
                "review ready in {:.2}s; session stays open — F6 to capture again",
                capture.started.elapsed().as_secs_f64()
            );
        } else {
            if let Some(state) = &capture.paired_state {
                let actor_sources=state.actors.iter().map(|actor|serde_json::json!({
                    "id":format!("{:?}",actor.id),
                    "source":actor.source_id.as_ref(),
                    "center":[actor.center.x,actor.center.y],
                    "size":[actor.size.x,actor.size.y],
                    "facing":actor.facing.map(|f|[f.x,f.y]),
                })).collect::<Vec<_>>();
                let mut receipt = format!(
                    "state_preserved=true\nchecksum={:?}\nlcd_animation_frame={}\nactors={}\nvirtual_seconds={}\nactor_sources={}\n",
                    state.checksum, state.lcd_frame, state.actors.len(), virtual_time.elapsed_secs_f64(),
                    serde_json::Value::Array(actor_sources),
                );
                receipt.push_str(&format!("presentation={}\n", capture.identity));
                if let Some(actors) = &capture.world_actor_sources {
                    receipt.push_str(&format!("world_actor_sources={actors}\n"));
                }
                if let Some(pinned) = send_out_hold.as_ref().and_then(|request| request.held.as_ref()) {
                    receipt.push_str(&format!(
                        "source_state_preserved=true\nsource_clock_hz=60\nsend_out_side={:?}\nsend_out_frame={}\nsource_frame={}\n",
                        pinned.animation.side, pinned.animation.frame,
                        pinned.presented.source.as_ref().unwrap().frame));
                }
                if let Err(error) = std::fs::write(capture.first_path.with_extension("paired-state.txt"), receipt) {
                    eprintln!("paired capture receipt failed: {error}");
                    exit.write(AppExit::error());
                    return;
                }
            }
            exit.write(AppExit::Success);
        }
        return;
    }
    // Bound startup/readback failures; ordinary walking has its own duration.
    let startup=capture.started;
    let elapsed=capture_timeout_elapsed(startup,&mut capture.route_readback_started,
        walk.as_ref().map(|walk|(walk.started(),walk.settled())),Instant::now());
    if elapsed.is_some_and(|elapsed|elapsed.as_secs()>capture.timeout_seconds) {
        eprintln!("capture timed out: {:?}", status.inactive_reason);
        exit.write(AppExit::error());
        return;
    }
    capture.frame = capture.frame.saturating_add(1);
    if capture.frame % 300 == 0
        && std::env::var_os("CRYSTAL_RENDER_TEST_TRACE_FRAME").is_some()
    {
        eprintln!("capture readiness: battle={battle_status:?}; menu={}; script={}; movement={}; fade={}; hold={}; bgp={:?}",
            !move_menu.is_empty(), runtime.pending_scene_script.is_some(),
            runtime.visible_script_movement.is_some(), runtime.screen_fade.is_some(),
            hold.frozen, battle_frame.source.as_ref().map(|source| source.bgp));
    }
    // Preserve the established 30 active-frame GPU warmup, but drop the
    // unconditional 90-frame startup and 60-frame post-save waits.
    // UI-only reviews can explicitly inspect the renderer's honest source-art
    // fallback. Model/scenery reviews continue requiring an active 3D scene.
    let source_fallback = std::env::var("CRYSTAL_RENDER_TEST_CAPTURE_SOURCE_FALLBACK").as_deref() == Ok("1")
        && battle_frame.active && !move_menu.is_empty()
        && battle_status.map_scenery_fallback_reason.is_some_and(|reason|
            reason != "actual battle groves are warming");
    // PC carried-Pokemon and Mail readers intentionally own the original LCD.
    // A named UI review may capture its actual fullscreen owner after its
    // ordinary inputs have frozen. Scenery reviews still require native world
    // or battle frames; this is not admission of a scenery-art fallback.
    let pc_lcd_review = hold.frozen
        && std::env::var("CRYSTAL_RENDER_TEST_HOLD_MENU_ID").ok()
            .is_some_and(|id|render_walk::pc_lcd_review_ready(&runtime,&id))
        && retained_field_fullscreen_active(&runtime)
        && walk.as_ref().is_some_and(|walk|walk.settled());
    let settled = if settings.enabled {
        (status.active && !status.profiles_pending && status.active_frames >= 30)
            || (battle_status.active && battle_status.active_frames >= 30)
            || source_fallback
            || pc_lcd_review
    } else {
        status.inactive_reason.as_deref() == Some("disabled")
    };
    if capture.requested
        // Tower source frames advance every 16 ticks. Same-view readbacks
        // need a longer gap than the ordinary paired-view GPU warmup so they
        // can observe an actual pose change rather than two identical poses.
        || capture.frame < if capture.same_second_view && capture.second.is_none() {43} else {12}
        || !settled
        // Existing source groves stream asynchronously. An active battle
        // alone does not mean its retained location has finished loading.
        || (settings.enabled && battle_status.active
            && battle_status.map_scenery_fallback_reason == Some("actual battle groves are warming"))
        || walk.as_ref().is_some_and(|walk| !walk.settled())
    {
        return;
    }
    // A uniform original background palette is a transient battle flash,
    // unsuitable for comparing scenery. Wait for the next visible source
    // frame before freezing; never advance it after taking the baseline.
    if capture.second.is_some() && battle_frame.active && battle_frame.source.as_ref().is_some_and(|source| {
        let tone = source.bgp & 3;
        (0..4).all(|index| (source.bgp >> (index * 2)) & 3 == tone)
    }) { return; }
    // Original item screens legitimately use only three displayed colors.
    // Keep the general scenery check unchanged; certify this narrow exception
    // only from the live, frozen production LCD owner, never from a file/env claim.
    if pc_lcd_review && std::env::var("CRYSTAL_RENDER_TEST_HOLD_MENU_ID").ok()
        .is_some_and(|id|matches!(id.as_str(),"pc:items"|"pc:quantity"|"pc:toss-confirmation"|"pc:toss-closing"))
        &&let Some(proof)=verified_pc_lcd.as_ref() {
        proof.0.store(true,std::sync::atomic::Ordering::Relaxed);
    }
    let screenshot = if capture.image_target_mode {
        let Some(target) = capture.image_target.as_ref() else {
            return;
        };
        // The screenshot's normalized target must also retain the scale.
        // Screenshot::image(handle) creates a distinct scale-1 target.
        Screenshot(bevy::camera::RenderTarget::Image(target.clone()))
    } else {
        let Ok(window) = primary_window.single() else {
            return;
        };
        Screenshot::window(window)
    };
    if !capture.live && !capture.same_second_view
        && walk.as_ref().is_none_or(|walk| walk.settled())
        && (capture.second.is_some() || capture.paired_state.is_some())
    {
        // Do not strand entry callbacks or a startup transition. Hold only
        // after the presentation has settled, immediately before readback.
        if runtime.pending_scene_script.is_some() || runtime.visible_script_movement.is_some()
            || runtime.screen_fade.is_some() || runtime.title_menu.is_some()
            || runtime.intro_screen.is_some()
        {
            return;
        }
        if !hold.frozen {
            // Actor extraction is normally disabled in 2D. Enable the passive
            // hold first, then let the next Update publish the complete actor
            // list before taking the comparison baseline or requesting readback.
            hold.frozen = true;
            virtual_time.pause();
            return;
        }
        let state = match paired_state(&runtime, &frame) {
            Ok(state) => state,
            Err(error) => {
                eprintln!("paired capture state unavailable: {error}");
                exit.write(AppExit::error());
                return;
            }
        };
        if let Some(first) = &capture.paired_state {
            if first != &state {
                eprintln!("paired capture rejected: authoritative state or actor presentation changed");
                exit.write(AppExit::error());
                return;
            }
        } else {
            // Enabling read-only actor extraction can reconcile newly spawned
            // NPC sprites one update later at the frozen movement subframe.
            // Wait for that presentation to stabilize before either readback.
            let Some(state)=capture.paired_stability.observe(state) else{return;};
            capture.paired_state = Some(state);
            hold.frozen = true;
            virtual_time.pause();
        }
    }
    if capture.identity.is_empty() {
        capture.identity = if battle_frame.active {
            format!(
                "battle|{}|modeled={:?}|source={:?}|fallback={:?}",
                battle_frame.map_id,
                battle_status.modeled_species,
                battle_status.source_art_species,
                battle_status.map_scenery_fallback_reason
            )
        } else {
            format!("{}|{:?}|{:?}", frame.map_id, frame.center, settings.camera)
        };
        if let Some(transition) = runtime.visible_battle_transition {
            capture.identity.push_str(&format!("|entry_frame={}", transition.frame));
        }
    }
    if std::env::var_os("CRYSTAL_RENDER_TEST_TRACE_FRAME").is_some() {
        println!("capture frame: map={} center={:?} origin={:?} grid={:?} tile={:?} viewport={:?} camera={:?}",
            frame.map_id, frame.center, frame.grid_origin, frame.grid_size,
            frame.tile_size, frame.viewport_size, settings.camera);
    }
    let path = capture.path.clone();
    if settings.enabled && capture.world_actor_sources.is_none() {
        capture.world_actor_sources = Some(serde_json::Value::Array(frame.actors.iter().map(|actor|
            serde_json::json!({
                "id":format!("{:?}", actor.id), "source":actor.source_id.as_ref(),
                "center":[actor.center.x,actor.center.y], "size":[actor.size.x,actor.size.y],
                "facing":actor.facing.map(|f|[f.x,f.y]),
            })
        ).collect()));
    }
    let preserve_alpha = capture.preserve_alpha;
    let completion = capture.completion.clone();
    commands
        .spawn(screenshot)
        .observe(move |event: On<ScreenshotCaptured>| {
            let result = (|| -> anyhow::Result<()> {
                crate::capture_archive::archive_existing(&path)?;
                crate::capture_archive::archive_gpu_readback(&path, &event.image)?;
                let image = event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .map_err(|error| anyhow::anyhow!("decode GPU image: {error}"))?;
                crate::capture_archive::archive_capture_image(&path, "original", &image)?;
                // Retain the existing RGB/RGBA latest-image contract while the
                // complete native readback and original alpha remain archived.
                let output = if preserve_alpha {
                    image::DynamicImage::ImageRgba8(image.to_rgba8())
                } else {
                    image::DynamicImage::ImageRgb8(image.to_rgb8())
                };
                crate::capture_archive::write_capture_png(&path, &output)?;
                Ok(())
            })()
            .map_err(|error| format!("{error:#}"));
            *completion.lock().unwrap() = Some(result);
        });
    capture.requested = true;
}

fn parse_camera_request(value: &str) -> Result<crystal_voxel_view::VoxelCameraControls> {
    let values = value
        .split_whitespace()
        .map(str::parse::<f32>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    anyhow::ensure!(
        (2..=4).contains(&values.len()) && values.iter().all(|value| value.is_finite()),
        "expected finite zoom, orbit, optional pitch and first-person (0/1), e.g. 1.0 0.5 5.0 0"
    );
    let mut camera = crystal_voxel_view::VoxelCameraControls::new(values[0], values[1]);
    if let Some(first_person) = values.get(3) {
        anyhow::ensure!(*first_person == 0.0 || *first_person == 1.0, "first-person must be 0 or 1");
        camera.first_person = *first_person == 1.0;
    }
    if let Some(pitch) = values.get(2) {
        camera.pitch_degrees = pitch.clamp(if camera.first_person { -85.0 } else { 5.0 }, 85.0);
    }
    Ok(camera)
}

fn live_comparison(capture: &mut Capture) -> Result<()> {
    let path = capture.first_path.with_extension("compare.png");
    crate::capture_archive::archive_existing(&path)?;
    crate::capture_archive::archive_existing(&capture.first_path)?;
    validate_render_test_screenshot(&capture.first_path)?;
    let second = capture
        .second_path
        .as_ref()
        .context("live review needs paired views")?;
    crate::capture_archive::archive_existing(second)?;
    validate_render_test_screenshot(second)?;
    let current_native = image::open(second)?;
    let reference_native = image::open(&capture.first_path)?;
    crate::capture_archive::archive_capture_image(&path, "current-original", &current_native)?;
    crate::capture_archive::archive_capture_image(&path, "reference-original", &reference_native)?;
    let current = current_native.to_rgb8();
    let reference = reference_native.to_rgb8();
    let previous = capture
        .previous
        .as_ref()
        .filter(|(identity, _)| identity == &capture.identity);
    let mut panels = vec![&reference];
    if let Some((_, previous)) = previous {
        panels.push(previous);
    }
    panels.push(&current);
    let mut sheet = image::RgbImage::new(640 * panels.len() as u32, 576);
    for (index, panel) in panels.iter().enumerate() {
        crate::capture_archive::archive_capture_image(
            &path,
            &format!("panel-{index}-rgb-original"),
            &image::DynamicImage::ImageRgb8((**panel).clone()),
        )?;
        let panel = image::imageops::resize(*panel, 640, 576, image::imageops::FilterType::Nearest);
        crate::capture_archive::archive_capture_image(
            &path,
            &format!("panel-{index}-resized"),
            &image::DynamicImage::ImageRgb8(panel.clone()),
        )?;
        image::imageops::replace(&mut sheet, &panel, index as i64 * 640, 0);
    }
    crate::capture_archive::write_capture_png(&path, &image::DynamicImage::ImageRgb8(sheet))?;
    println!(
        "review {} — 2D reference / {}current 2.5D",
        path.display(),
        if previous.is_some() {
            "previous 2.5D / "
        } else {
            ""
        }
    );
    capture.previous = Some((capture.identity.clone(), current));
    Ok(())
}

#[cfg(test)]
mod image_target_tests {
    use super::*;
    #[test]
    fn automated_native_captures_default_to_offscreen_images() {
        assert_eq!(image_capture_target(None, false).unwrap(), !cfg!(target_arch = "wasm32"));
        assert!(!image_capture_target(None, true).unwrap());
        assert!(image_capture_target(Some("image"), false).unwrap());
        assert!(!image_capture_target(Some("window"), false).unwrap());
        assert!(image_capture_target(Some("invalid"), false).is_err());
    }
    use bevy::{
        camera::RenderTarget,
        window::{WindowRef, WindowResolution},
    };

    #[test]
    fn source_frame_hold_is_opt_in_and_only_accepts_reveal_ticks() {
        assert_eq!(requested_send_out_frame(None).unwrap(), None);
        for tick in ["4", "5"] {
            assert_eq!(requested_send_out_frame(Some(tick)).unwrap(), tick.parse::<u8>().ok());
        }
        for invalid in ["", "0", "3", "6", "40", "4.0"] {
            assert!(requested_send_out_frame(Some(invalid)).is_err());
        }
        let mut app = App::new();
        install_send_out_hold(&mut app, None);
        assert!(!app.world().contains_resource::<bevy::time::TimeUpdateStrategy>());
        assert!(!app.world().contains_resource::<SendOutFrameHold>());
        install_send_out_hold(&mut app, Some(4));
        assert!(matches!(app.world().resource::<bevy::time::TimeUpdateStrategy>(),
            bevy::time::TimeUpdateStrategy::ManualDuration(step)
            if *step == std::time::Duration::from_secs_f64(1.0 / 60.0)));
        assert!(app.world().resource::<SendOutFrameHold>().held.is_none());
    }

    #[test]
    fn entry_hold_is_opt_in_and_does_not_replace_the_game_clock() {
        assert_eq!(requested_entry_after(None).unwrap(), None);
        assert_eq!(requested_entry_after(Some("110")).unwrap(), Some(110));
        for invalid in ["0", "1", "181", "-1", "no"] { assert!(requested_entry_after(Some(invalid)).is_err()); }
        let mut app = App::new();
        install_entry_hold(&mut app, None);
        assert!(!app.world().contains_resource::<EntryFrameHold>());
        install_entry_hold(&mut app, Some(110));
        assert_eq!(app.world().resource::<EntryFrameHold>().after, 110);
        assert!(!app.world().resource::<EntryFrameHold>().held);
        assert!(!app.world().contains_resource::<bevy::time::TimeUpdateStrategy>());
    }

    #[test]
    fn source_frame_hold_requires_the_actual_presented_send_out_without_advancing_it() {
        let animation = VisibleSendOutAnimation {
            side: crate::core::battle::turn::BattleSide::Player, frame: 4, shiny: false,
        };
        let mut frame = crystal_render_api::VisualBattleFrame {
            active: true, map_id: Arc::from("Route29"),
            source: Some(VisualBattleSourceFrame {
                frame: 4, bgp: 0xe4, battler_bgps: [0xe4; 2],
                battler_palettes: [[[1.0; 4]; 4]; 2],
                battler_textures: [Handle::default(), Handle::default()],
                battler_offsets: [Vec2::ZERO; 2], screen_offset: Vec2::ZERO,
                line_x_offsets: None, line_y_offsets: None,
                objects: Vec::new(), battler_rows: [None; 2],
            }),
            cues: vec![VisualBattleCue { kind: VisualBattleCueKind::SendOut,
                side: VisualBattleSide::Player, move_id: Arc::from("SEND_OUT"),
                element: Arc::from("NORMAL"), progress: 4.0 / 36.0, damaging: false }],
            ..default()
        };
        let before = frame.clone();
        assert!(presented_send_out_matches(4, Some(&animation), &frame));
        assert_eq!(frame, before);
        assert_eq!(animation.frame, 4);
        assert!(!presented_send_out_matches(5, Some(&animation), &frame));
        assert!(!presented_send_out_matches(4, None, &frame));
        frame.source.as_mut().unwrap().frame = 5;
        assert!(!presented_send_out_matches(4, Some(&animation), &frame));
        frame.source.as_mut().unwrap().frame = 4;
        frame.cues[0].side = VisualBattleSide::Enemy;
        assert!(!presented_send_out_matches(4, Some(&animation), &frame));
        frame.cues[0].side = VisualBattleSide::Player;
        frame.use_source_scene = true;
        assert!(!presented_send_out_matches(4, Some(&animation), &frame));
    }

    #[test]
    fn moving_capture_retains_explicit_image_target_scale_without_window_fallback() {
        let (mut app, window, _, _, _, _) = fixture(true);
        assert!(moving_screenshot(Some(app.world().resource::<Capture>()), window).is_none());
        app.update();
        let capture = app.world().resource::<Capture>();
        let expected = capture.image_target.as_ref().unwrap();
        let screenshot = moving_screenshot(Some(capture), window).unwrap();
        assert!(matches!(screenshot.0, RenderTarget::Image(ref target) if target == expected));
        assert_eq!(expected.scale_factor, 2.0);
        let (mut app, window, _, _, _, _) = fixture(false);
        app.update();
        assert!(matches!(moving_screenshot(Some(app.world().resource::<Capture>()), window)
            .unwrap().0, RenderTarget::Window(WindowRef::Entity(entity)) if entity == window));
    }

    #[test]
    fn paired_baseline_waits_for_npc_interpolation_reconciliation() {
        let state=|x|PairedState {
            checksum:StateChecksum::new(12,3691618926),lcd_frame:16,
            actors:vec![crystal_render_api::VisualActor {
                id:crystal_render_api::VisualActorId::Object(2),source_id:Arc::from("cooltrainer_f"),
                model_palette:None,texture:default(),center:Vec2::new(x,128.),size:Vec2::splat(64.),
                flip_x:false,above_priority:false,facing:Some(Vec2::X),
            }],
        };
        let mut stable=StablePairedState::default();
        assert!(stable.observe(state(160.)).is_none());
        // Same authoritative state, but the newly reconciled source sprite
        // moves to its correct frozen interpolation point before capture.
        assert!(stable.observe(state(157.84944)).is_none());
        assert!(stable.observe(state(157.84944)).is_none());
        let baseline=stable.observe(state(157.84944)).unwrap();
        assert!(baseline==state(157.84944));assert!(baseline!=state(160.));
    }

    fn fixture(enabled: bool) -> (App, Entity, Entity, Entity, Entity, Handle<Image>) {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>()
            .insert_resource(Capture {
                path: PathBuf::new(),
                second: None,
                same_second_view: false,
                frame: 0,
                requested: false,
                completion: default(),
                started: Instant::now(),
                timeout_seconds: 60,
                route_readback_started: None,
                live: false,
                idle: false,
                first_path: PathBuf::new(),
                second_path: None,
                request_path: PathBuf::new(),
                previous: None,
                identity: String::new(),
                image_target_mode: enabled,
                recording_target_only: false,
                preserve_alpha: false,
                image_target: None,
                image_size: UVec2::ZERO,
                paired_state: None,
                world_actor_sources: None,
        paired_stability: StablePairedState::default(),
            })
            .add_systems(Update, prepare_image_target);
        let window = app
            .world_mut()
            .spawn((
                Window {
                    resolution: WindowResolution::new(640, 576).with_scale_factor_override(2.),
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        let secondary = app.world_mut().spawn(Window::default()).id();
        let primary_camera = app
            .world_mut()
            .spawn((
                Camera2d,
                Camera {
                    order: 3,
                    viewport: Some(bevy::camera::Viewport {
                        physical_position: UVec2::new(16, 24),
                        physical_size: UVec2::new(320, 288),
                        ..default()
                    }),
                    ..default()
                },
            ))
            .id();
        let secondary_camera = app
            .world_mut()
            .spawn((Camera2d, RenderTarget::Window(WindowRef::Entity(secondary))))
            .id();
        let owned_image =
            app.world_mut()
                .resource_mut::<Assets<Image>>()
                .add(Image::new_target_texture(
                    32,
                    32,
                    bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                    None,
                ));
        let image_camera = app
            .world_mut()
            .spawn((Camera2d, RenderTarget::Image(owned_image.clone().into())))
            .id();
        (
            app,
            window,
            primary_camera,
            secondary_camera,
            image_camera,
            owned_image,
        )
    }

    #[test]
    fn explicit_image_capture_preserves_viewport_scale_and_other_target_ownership() {
        let (mut app, _, camera, secondary, image_camera, owned) = fixture(true);
        app.update();
        let capture = app.world().resource::<Capture>();
        let target = capture.image_target.as_ref().unwrap();
        assert_eq!(target.scale_factor, 2.);
        assert_eq!(capture.image_size, UVec2::new(640, 576));
        assert!(matches!(app.world().get::<RenderTarget>(camera),
            Some(RenderTarget::Image(image)) if image == target));
        assert!(
            app.world()
                .get::<crystal_voxel_view::BattleCompositeImageCamera>(camera)
                .is_some()
        );
        assert!(
            app.world()
                .get::<crystal_voxel_view::BattleCompositeImageCamera>(secondary)
                .is_none()
        );
        assert!(
            app.world()
                .get::<crystal_voxel_view::BattleCompositeImageCamera>(image_camera)
                .is_none()
        );
        let camera = app.world().get::<Camera>(camera).unwrap();
        assert_eq!(camera.order, 3);
        assert_eq!(
            camera.viewport.as_ref().unwrap().physical_position,
            UVec2::new(16, 24)
        );
        assert_eq!(
            camera.viewport.as_ref().unwrap().physical_size,
            UVec2::new(320, 288)
        );
        assert!(matches!(
            app.world().get::<RenderTarget>(secondary),
            Some(RenderTarget::Window(WindowRef::Entity(_)))
        ));
        assert!(matches!(app.world().get::<RenderTarget>(image_camera),
            Some(RenderTarget::Image(image)) if image.handle == owned && image.scale_factor == 1.));
    }

    #[test]
    fn explicit_image_capture_resize_updates_only_its_owned_target() {
        let (mut app, window, camera, _, image_camera, owned) = fixture(true);
        app.update();
        let old = app
            .world()
            .resource::<Capture>()
            .image_target
            .as_ref()
            .unwrap()
            .handle
            .clone();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set_physical_resolution(800, 600);
        app.update();
        let capture = app.world().resource::<Capture>();
        let target = capture.image_target.as_ref().unwrap();
        assert_ne!(target.handle, old);
        assert_eq!(capture.image_size, UVec2::new(800, 600));
        assert!(matches!(app.world().get::<RenderTarget>(camera),
            Some(RenderTarget::Image(image)) if image == target));
        assert!(matches!(app.world().get::<RenderTarget>(image_camera),
            Some(RenderTarget::Image(image)) if image.handle == owned));
    }

    #[test]
    fn ordinary_window_capture_does_not_create_or_redirect_an_image_target() {
        let (mut app, _, camera, _, _, _) = fixture(false);
        app.update();
        assert!(app.world().resource::<Capture>().image_target.is_none());
        assert!(matches!(
            app.world().get::<RenderTarget>(camera),
            Some(RenderTarget::Window(WindowRef::Primary))
        ));
    }
}
