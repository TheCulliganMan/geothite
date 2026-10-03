//! Native opt-in image bridge for one static enemy capture. No controller,
//! outcome, sound, RNG, presentation timer, or mesh mutation belongs here.
use super::*;
use crystal_render_api::{VisualBattleCapture, VisualCapturePicture};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum CaptureChoice {
    #[default]
    Idle,
    Armed,
    Image,
    Classic(&'static str),
}

#[derive(Clone, PartialEq)]
struct CaptureIdentity {
    map: Arc<str>,
    species: Arc<str>,
    party: Option<usize>,
    ball: Arc<str>,
}

struct EnemyImageLease {
    image: Handle<Image>,
    stamp: row_capture::SettledActorImage,
    layout: BattleSceneLayout,
    source_rect: Rect,
    opaque_rect: Rect,
}

#[derive(Resource)]
pub(super) struct BattleCaptureBridge {
    pub(super) enabled: bool,
    pub(super) choice: CaptureChoice,
    identity: Option<CaptureIdentity>,
    last_frame: u16,
    lease: Option<EnemyImageLease>,
}
impl Default for BattleCaptureBridge {
    fn default() -> Self {
        Self {
            enabled: cfg!(not(target_arch = "wasm32"))
                && std::env::var("CRYSTAL_BATTLE_CAPTURE_PROTOTYPE").as_deref() == Ok("1"),
            choice: CaptureChoice::Idle,
            identity: None,
            last_frame: 0,
            lease: None,
        }
    }
}
impl BattleCaptureBridge {
    pub(super) fn image_active(&self) -> bool {
        self.choice == CaptureChoice::Image && self.lease.is_some()
    }
    pub(super) fn allows_scene(&self) -> bool {
        !matches!(self.choice, CaptureChoice::Classic(_))
    }
    fn retire(&mut self) {
        self.choice = CaptureChoice::Idle;
        self.identity = None;
        self.lease = None;
        self.last_frame = 0;
    }
    fn fallback(&mut self, reason: &'static str) {
        self.choice = CaptureChoice::Classic(reason);
        self.lease = None;
    }
}

fn lease_compatible(
    lease: &EnemyImageLease,
    scene: &BattleScene,
    layout: &BattleSceneLayout,
    frame: &VisualBattleFrame,
    output: UVec2,
    images: &Assets<Image>,
    meshes: &Assets<Mesh>,
    materials: &Assets<StandardMaterial>,
) -> bool {
    let Some(actor) = scene.actors[1].as_ref() else { return false };
    let Some(enemy) = frame.battlers[1].as_ref() else { return false };
    let Some(image) = images.get(&lease.image) else { return false };
    actor.key.modeled && !actor.is_articulated()
        && actor.key.species == enemy.species_id && actor.key.party_index == enemy.party_index
        && actor.entity == lease.stamp.actor
        && actor.mesh.as_ref().is_some_and(|mesh| mesh.id() == lease.stamp.mesh)
        && actor.material.id() == lease.stamp.material
        && meshes.contains(lease.stamp.mesh) && materials.contains(lease.stamp.material)
        && actor.base_pose.compute_matrix() == lease.stamp.pose
        && actor.palette_key.0 == 0xe4
        && enemy.allow_species_model && !enemy.shiny
        && enemy.source_rect == lease.source_rect
        && enemy.source_opaque_rect == lease.opaque_rect
        && layout == &lease.layout
        && layout.camera.compute_matrix() == lease.stamp.camera
        && output.min_element() > 0 && output == lease.stamp.output_size
        && scene.row_targets[1].id() == lease.stamp.target
        && scene.row_targets[0].id() != lease.stamp.target
        && scene.row_sizes[1] == lease.stamp.render_size
        && UVec2::new(image.texture_descriptor.size.width, image.texture_descriptor.size.height)
            == lease.stamp.render_size
        && frame.source.as_ref().is_none_or(|source| {
            source.battler_bgps[1] == 0xe4
                && source.battler_offsets == [Vec2::ZERO; 2]
                && source.screen_offset == Vec2::ZERO
                && source.line_x_offsets.is_none() && source.line_y_offsets.is_none()
                && source.battler_rows == [None; 2]
        })
}

/// This pure latch is deliberately irreversible within one visible capture.
/// Missing GPU readiness never holds the source clock or expires into success.
fn choose_capture(
    current: CaptureChoice,
    presented: bool,
    allowed: bool,
    ready: bool,
    compatible: bool,
) -> CaptureChoice {
    if !presented { return CaptureChoice::Armed; }
    match current {
        CaptureChoice::Idle | CaptureChoice::Armed => {
            if !allowed { CaptureChoice::Classic("capture pilot unsupported") }
            else if ready { CaptureChoice::Image }
            else { CaptureChoice::Classic("enemy image was not ready at capture start") }
        }
        CaptureChoice::Image if !allowed || !compatible =>
            CaptureChoice::Classic("capture image interrupted or incompatible"),
        previous => previous,
    }
}

/// Revalidate the exact isolated view that produced the receipt. Unrelated
/// overworld suns and cameras must never participate in this decision.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct CaptureViewInputs<'w, 's> {
    ambient: Option<Res<'w, AmbientLight>>,
    msaa: Option<Res<'w, Msaa>>,
    lights: Query<'w, 's, (Entity, Ref<'static, DirectionalLight>, &'static Transform,
        &'static Visibility, &'static InheritedVisibility, Option<&'static RenderLayers>, Has<Parent>)>,
    other_lights: Query<'w, 's, (Option<&'static RenderLayers>, Has<bevy::pbr::LightProbe>),
        Or<(With<PointLight>, With<SpotLight>, With<bevy::pbr::LightProbe>)>>,
    views: Query<'w, 's, (Entity, &'static BattleRowCamera, &'static Camera, &'static Transform,
        &'static Projection, &'static bevy::render::camera::CameraRenderGraph,
        Ref<'static, Camera3d>, Ref<'static, bevy::render::camera::Exposure>,
        Ref<'static, Tonemapping>, Ref<'static, bevy::render::view::ColorGrading>,
        Ref<'static, bevy::core_pipeline::tonemapping::DebandDither>, &'static RenderLayers,
        Option<&'static Fxaa>, Option<&'static FogSettings>, Has<Parent>), With<BattleRowCamera>>,
    other_cameras: Query<'w, 's, &'static Camera, Without<BattleRowCamera>>,
    unsupported: Query<'w, 's, (), Or<(
        With<bevy::render::view::GpuCulling>, With<bevy::core_pipeline::Skybox>,
        With<bevy::render::camera::TemporalJitter>,
        With<bevy::core_pipeline::experimental::taa::TemporalAntiAliasSettings>,
        With<bevy::core_pipeline::dof::DepthOfFieldSettings>,
        With<bevy::pbr::ScreenSpaceAmbientOcclusionSettings>, With<bevy::pbr::ScreenSpaceReflectionsSettings>,
        With<bevy::pbr::environment_map::EnvironmentMapLight>,
    )>>,
}
impl CaptureViewInputs<'_, '_> {
    fn compatible(&self, stamp: &row_capture::SettledActorImage) -> bool {
        use bevy::render::render_graph::RenderSubGraph;
        let layers = RenderLayers::layer(BATTLE_ROW_LAYERS[1]);
        let mut lights = self.lights.iter().filter(|(_, _, _, visibility, inherited, light_layers, _)|
            **visibility != Visibility::Hidden && inherited.get()
                && light_layers.cloned().unwrap_or_default().intersects(&layers));
        let Some((entity, light, pose, _, _, _, parent)) = lights.next() else { return false };
        if lights.next().is_some() || parent || light.shadows_enabled
            || (entity, light.last_changed().get(), pose.compute_matrix()) != stamp.light
            || self.ambient.as_ref().map(|light| (light.color, light.brightness)) != stamp.ambient
            || self.msaa.as_ref().map_or(1, |msaa| msaa.samples()) != stamp.msaa_samples
            || self.other_lights.iter().any(|(light_layers, probe)| probe
                || light_layers.cloned().unwrap_or_default().intersects(&layers))
            || self.other_cameras.iter().any(|camera| camera.is_active
                && matches!(&camera.target, RenderTarget::Image(target) if target.id() == stamp.target))
        { return false; }
        if self.views.iter().filter(|(_, row, ..)| row.0 == 1).count() != 1 { return false; }
        let Some((entity, _, camera, pose, projection, graph, camera_3d, exposure, tone, grading,
            dither, camera_layers, fxaa, fog, parent)) = self.views.iter().find(|(_, row, ..)| row.0 == 1)
        else { return false };
        // Check every other row view by entity, not only the expected player
        // index. A second active writer cannot share this held image target.
        if self.views.iter().any(|(other, _, camera, ..)| other != entity && camera.is_active
            && matches!(&camera.target, RenderTarget::Image(target) if target.id() == stamp.target))
        { return false; }
        let Projection::Perspective(projection) = projection else { return false };
        !parent && !self.unsupported.contains(entity)
            && *camera_layers == layers && !camera.hdr && camera.viewport.is_none()
            && fxaa.is_none() && fog.is_none()
            && **graph == bevy::core_pipeline::core_3d::graph::Core3d.intern()
            && matches!(&camera.target, RenderTarget::Image(target) if target.id() == stamp.target)
            && matches!(camera.clear_color, ClearColorConfig::Custom(color) if color == Color::NONE)
            && matches!(camera.output_mode, bevy::render::camera::CameraOutputMode::Write {
                blend_state: Some(bevy::render::render_resource::BlendState::REPLACE),
                clear_color: ClearColorConfig::Custom(color),
            } if color == Color::NONE)
            && pose.compute_matrix() == stamp.camera
            && [projection.fov, projection.aspect_ratio, projection.near, projection.far] == stamp.projection
            && [camera_3d.last_changed().get(), exposure.last_changed().get(), tone.last_changed().get(),
                grading.last_changed().get(), dither.last_changed().get()] == stamp.camera_changes
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn select_capture_bridge(
    frame: Res<VisualBattleFrame>,
    settings: Res<VoxelViewSettings>,
    canvas: Res<VisualBattleCanvas>,
    layout: Res<BattleSceneLayout>,
    scene: Res<BattleScene>,
    captures: Res<row_capture::ActorCaptures>,
    mut bridge: ResMut<BattleCaptureBridge>,
    mut status: ResMut<BattleViewStatus>,
    assets: (Res<Assets<Image>>, Res<Assets<Mesh>>, Res<Assets<StandardMaterial>>),
    mut events: row_capture::CaptureAssetEvents,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    view_inputs: CaptureViewInputs,
) {
    let (images, meshes, materials) = assets;
    status.capture_image_active = false;
    status.capture_fallback_reason = None;
    // Drain every reader on every display sample, including idle/pending
    // capture. Otherwise old initialization events can reject a newer already-
    // submitted image when the first capture starts. Do not short-circuit a
    // reader: later events must not leak into the next sample either.
    // Only the leased actor and target matter. Source OAM intentionally
    // refreshes unrelated materials while the image is held. Asset events
    // flushed in Last invalidate the next presented sample. Supported image
    // writers are disabled while leased, so actor edits cannot overwrite it.
    // Arbitrary same-size CPU Image edits are unsupported: their GPU upload can
    // precede next-sample invalidation and replace the held target's pixels.
    let stamp = bridge.lease.as_ref().map(|lease| lease.stamp.clone())
        .or_else(|| captures.settled_image(1));
    let mesh_dirty = events.meshes.read().fold(false, |dirty, event|
        dirty | stamp.as_ref().is_some_and(|stamp| stamp.mesh == row_capture::changed_asset(event)));
    let material_dirty = events.materials.read().fold(false, |dirty, event|
        dirty | stamp.as_ref().is_some_and(|stamp| stamp.material == row_capture::changed_asset(event)));
    let image_dirty = events.images.read().fold(false, |dirty, event|
        dirty | stamp.as_ref().is_some_and(|stamp| stamp.target == row_capture::changed_asset(event)));
    let shader_dirty = events.shaders.read().count() > 0;
    let changed = mesh_dirty || material_dirty || image_dirty || shader_dirty;
    let Some(capture) = frame.capture.as_ref().filter(|_| frame.active) else {
        bridge.retire();
        return;
    };
    let Some(enemy) = frame.battlers[1].as_ref() else {
        bridge.fallback("capture target unavailable");
        status.capture_fallback_reason = Some("capture target unavailable");
        return;
    };
    let identity = CaptureIdentity {
        map: frame.map_id.clone(), species: enemy.species_id.clone(),
        party: enemy.party_index, ball: capture.ball_id.clone(),
    };
    if bridge.identity.as_ref() != Some(&identity) || capture.frame < bridge.last_frame {
        bridge.retire();
        bridge.identity = Some(identity);
    }
    let output = windows.get_single().map_or(UVec2::ZERO, |window|
        UVec2::new(window.physical_width(), window.physical_height()));
    let view_compatible = stamp.as_ref().is_some_and(|stamp| view_inputs.compatible(stamp));
    let allowed = bridge.enabled && settings.enabled && !frame.use_source_scene
        && frame.validate().is_ok() && output.min_element() > 0;
    if capture.presented && matches!(bridge.choice, CaptureChoice::Idle | CaptureChoice::Armed)
        && allowed && !changed && view_compatible && enemy.visible
        // Starting the renderer in the middle of an already-running capture
        // is not evidence of the original full target image.
        && (bridge.choice == CaptureChoice::Armed || capture.frame == 0)
        && let Some(stamp) = captures.settled_image(1)
    {
        let lease = EnemyImageLease {
            image: scene.row_targets[1].clone(), stamp, layout: (*layout).clone(),
            source_rect: enemy.source_rect, opaque_rect: enemy.source_opaque_rect,
        };
        if lease_compatible(&lease, &scene, &layout, &frame, output, &images, &meshes, &materials) {
            bridge.lease = Some(lease);
        }
    }
    let expected_size = scene.quality.target_size(if canvas.physical_size.min_element() > 0 {
        canvas.physical_size
    } else { output });
    let compatible = !changed && view_compatible && bridge.lease.as_ref().is_some_and(|lease|
        lease.stamp.render_size == expected_size
            && lease_compatible(lease, &scene, &layout, &frame, output, &images, &meshes, &materials));
    bridge.choice = choose_capture(bridge.choice, capture.presented, allowed,
        bridge.lease.is_some() && compatible, compatible);
    if !matches!(bridge.choice, CaptureChoice::Image) { bridge.lease = None; }
    bridge.last_frame = capture.frame;
    status.capture_image_active = bridge.image_active();
    if let CaptureChoice::Classic(reason) = bridge.choice {
        status.capture_fallback_reason = Some(reason);
    }
}

pub(super) fn picture_tiles(capture: &VisualBattleCapture) -> u8 {
    match capture.enemy_picture {
        VisualCapturePicture::Full => 7,
        VisualCapturePicture::Tiles(tiles) => tiles,
        VisualCapturePicture::Hidden => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_is_once_per_capture_and_never_a_timer() {
        let mut choice = choose_capture(CaptureChoice::Idle, false, true, false, false);
        assert_eq!(choice, CaptureChoice::Armed);
        choice = choose_capture(choice, true, true, false, false);
        assert!(matches!(choice, CaptureChoice::Classic(_)));
        for _ in 0..10_000 {
            choice = choose_capture(choice, true, true, true, true);
            assert!(matches!(choice, CaptureChoice::Classic(_)));
        }
        assert_eq!(choose_capture(CaptureChoice::Idle, true, true, true, true), CaptureChoice::Image);
    }

    #[test]
    fn interruption_cannot_resume_an_old_image_inside_the_same_capture() {
        for (allowed, compatible) in [(false, true), (true, false)] {
            let choice = choose_capture(CaptureChoice::Image, true, allowed, true, compatible);
            assert!(matches!(choice, CaptureChoice::Classic(_)));
            assert_eq!(choose_capture(choice, true, true, true, true), choice);
        }
        // A new capture has its own readiness decision after retirement.
        assert_eq!(choose_capture(CaptureChoice::Idle, true, true, true, true), CaptureChoice::Image);
    }

    #[test]
    fn hidden_and_expansion_are_picture_samples_not_new_capture_decisions() {
        let mut sample = VisualBattleCapture { frame: 0, ball_id: Arc::from("POKE_BALL"),
            presented: true, enemy_picture: VisualCapturePicture::Full };
        for (frame, picture, tiles) in [
            (70, VisualCapturePicture::Full, 7), (71, VisualCapturePicture::Tiles(7), 7),
            (75, VisualCapturePicture::Tiles(5), 5), (79, VisualCapturePicture::Tiles(3), 3),
            (83, VisualCapturePicture::Hidden, 0), (437, VisualCapturePicture::Tiles(3), 3),
            (441, VisualCapturePicture::Tiles(5), 5), (445, VisualCapturePicture::Tiles(7), 7),
            (449, VisualCapturePicture::Full, 7),
        ] {
            sample.frame = frame; sample.enemy_picture = picture;
            assert_eq!(picture_tiles(&sample), tiles);
            assert_eq!(choose_capture(CaptureChoice::Image, true, true, false, true), CaptureChoice::Image);
        }
    }
}
