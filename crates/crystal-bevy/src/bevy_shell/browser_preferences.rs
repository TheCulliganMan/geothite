// Only checkpoint atomic overworld states; battles and scripts retain the last
// safe checkpoint rather than saving presentation state the loader cannot restore.
fn save_browser_checkpoint(runtime: &mut BevyRuntimeShell) -> Result<bool> {
    if save_manager_is_open() { return Ok(false); }
    let Some(path) = runtime.quick_save_path.as_ref() else { return Ok(false) };
    let snapshot = runtime.shell.snapshot()?;
    if snapshot.trainer.player_name.is_empty()
        || !visible_quick_save_blockers(runtime, &snapshot, false, false, false).is_empty()
    {
        return Ok(false);
    }
    runtime.shell.save(path)?;
    Ok(true)
}

#[cfg(target_arch = "wasm32")]
fn autosave_browser_progress(mut runtime: ResMut<BevyRuntimeShell>, time: Res<Time>, mut elapsed: Local<f32>) {
    *elapsed += time.delta_secs();
    if *elapsed < 1.0 { return; }
    *elapsed = 0.0;
    if let Err(error) = save_browser_checkpoint(&mut runtime) {
        bevy::log::error!("browser autosave failed: {error:#}");
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static BROWSER_AUDIO_CONTEXT: std::cell::RefCell<Option<web_sys::AudioContext>> = const { std::cell::RefCell::new(None) };
    static BROWSER_PREPARED_AUDIO_CONTEXT: std::cell::RefCell<Option<web_sys::AudioContext>> = const { std::cell::RefCell::new(None) };
    static BROWSER_MUTED: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    static BROWSER_MASTER_GAIN: std::cell::RefCell<Option<web_sys::GainNode>> = const { std::cell::RefCell::new(None) };
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn crystal_set_muted(muted: bool) {
    BROWSER_MUTED.with(|value| value.set(muted));
    BROWSER_MASTER_GAIN.with(|gain| {
        if let Some(gain) = gain.borrow().as_ref() {
            gain.gain().set_value(if muted { 0.0 } else { 1.0 });
        }
    });
}

// Queue browser presentation preferences for the ECS update; never mutate the
// simulation or synthesize a gameplay keypress from a display control.
#[cfg(all(target_arch = "wasm32", feature = "voxel-view"))]
thread_local! {
    static BROWSER_VOXEL_CAMERA: std::cell::Cell<Option<crystal_voxel_view::VoxelCameraControls>> = const { std::cell::Cell::new(None) };
    static BROWSER_VOXEL_CAMERA_STATE: std::cell::Cell<Option<crystal_voxel_view::VoxelCameraControls>> = const { std::cell::Cell::new(None) };
    static BROWSER_VOXEL_CAMERA_FACING: std::cell::Cell<Vec2> = const { std::cell::Cell::new(Vec2::NEG_Y) };
    static BROWSER_VOXEL_CAMERA_ACTIVE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static BROWSER_VOXEL_VIEW: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

#[cfg(all(target_arch = "wasm32", feature = "voxel-view"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn crystal_set_voxel_view(enabled: bool) {
    let enabled = enabled && renderer_policy::crystal_voxel_view_allowed();
    BROWSER_VOXEL_VIEW.with(|value| value.set(Some(enabled)));
}

#[cfg(all(target_arch = "wasm32", feature = "voxel-view"))]
fn apply_browser_voxel_view(
    mut settings: ResMut<crystal_voxel_view::VoxelViewSettings>,
    frame: Option<Res<crystal_render_api::VisualWorldFrame>>,
    mut shell: Option<ResMut<BevyRuntimeShell>>,
) {
    BROWSER_VOXEL_CAMERA.with(|value| {
        if let Some(camera) = value.take() {
            settings.camera = camera;
        }
    });
    BROWSER_VOXEL_VIEW.with(|value| {
        if let Some(enabled) = value.take() {
            settings.enabled = enabled;
        }
    });
    BROWSER_VOXEL_CAMERA_STATE.with(|value| value.set(Some(settings.camera)));
    BROWSER_VOXEL_CAMERA_ACTIVE.with(|value| value.set(settings.enabled
        && frame.as_ref().is_some_and(|frame| frame.active)
        && !shell.as_mut().is_some_and(|shell| has_visible_shell_direction_action(shell))));
    if let Some(facing) = frame.as_ref().and_then(|frame| frame.actors.iter()
        .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
        .and_then(|actor| actor.facing)) {
        BROWSER_VOXEL_CAMERA_FACING.with(|value| value.set(facing));
    }
}

#[cfg(all(target_arch = "wasm32", feature = "voxel-view"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn crystal_set_voxel_camera(zoom_step: f32, rotation_step: f32) {
    BROWSER_VOXEL_CAMERA.with(|value| {
        value.set(Some(crystal_voxel_view::VoxelCameraControls::new(zoom_step, rotation_step)));
    });
}

/// Set presentation only; camera buttons never synthesize gameplay input.
#[cfg(all(target_arch = "wasm32", feature = "voxel-view"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn crystal_set_voxel_camera_pose(zoom: f32, rotation: f32, pitch: f32, first_person: bool) {
    BROWSER_VOXEL_CAMERA.with(|value| {
        value.set(Some(crystal_voxel_view::VoxelCameraControls::with_pose(
            zoom, rotation, pitch, first_person,
        )));
    });
}

/// Browser gamepad Select uses the same Rust presets and visible trainer facing.
#[cfg(all(target_arch = "wasm32", feature = "voxel-view"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn crystal_cycle_voxel_camera() -> String {
    if !BROWSER_VOXEL_CAMERA_ACTIVE.with(|value| value.get()) { return String::new(); }
    let Some(mut camera) = BROWSER_VOXEL_CAMERA.with(|value| value.get())
        .or_else(|| BROWSER_VOXEL_CAMERA_STATE.with(|value| value.get())) else { return String::new(); };
    camera.cycle_view(BROWSER_VOXEL_CAMERA_FACING.with(|value| value.get()));
    BROWSER_VOXEL_CAMERA.with(|value| value.set(Some(camera)));
    serde_json::json!({"zoom":camera.zoom_step,"rotation":camera.rotation_step,
        "pitch":camera.pitch_degrees,"firstPerson":camera.first_person}).to_string()
}

/// Prepare the browser's audio device during loading without exposing it to
/// playback. Returns a JavaScript error if context creation is rejected.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn crystal_prepare_audio_context() -> Result<(), wasm_bindgen::JsValue> {
    if BROWSER_AUDIO_CONTEXT.with(|slot| slot.borrow().is_some()) {
        return Ok(());
    }
    BROWSER_PREPARED_AUDIO_CONTEXT.with(|slot| {
        let mut prepared = slot.borrow_mut();
        if prepared.is_none() {
            *prepared = Some(web_sys::AudioContext::new()?);
        }
        Ok(())
    })
}

// Called synchronously by trusted DOM gestures, outside the ECS frame loop.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn crystal_resume_audio() -> Result<js_sys::Promise, wasm_bindgen::JsValue> {
    BROWSER_AUDIO_CONTEXT.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            let prepared = BROWSER_PREPARED_AUDIO_CONTEXT.with(|slot| slot.borrow_mut().take());
            *slot = Some(match prepared {
                Some(context) => context,
                None => web_sys::AudioContext::new()?,
            });
        }
        slot.as_ref().expect("browser audio context initialized").resume()
    })
}
