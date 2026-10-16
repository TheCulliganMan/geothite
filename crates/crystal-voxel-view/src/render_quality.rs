//! Camera effects belong to the world camera, never the classic UI compositor.
use bevy::{
    anti_alias::smaa::{Smaa, SmaaPreset},
    camera::Hdr,
    light::{CascadeShadowConfig, CascadeShadowConfigBuilder, ShadowFilteringMethod},
    post_process::{
        bloom::{Bloom, BloomCompositeMode, BloomPrefilter},
        dof::{DepthOfField, DepthOfFieldMode},
    },
    prelude::*,
    render::{
        renderer::{RenderAdapterInfo, RenderDevice},
        view::{ColorGrading, DebandDither, Tonemapping},
    },
};
use std::sync::OnceLock;

#[derive(Component)]
struct WorldOverlayHdr;

pub(super) fn register(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (
            sync_overlay_format,
            sync_tilt_shift.after(bevy::camera::CameraUpdateSystems),
        ),
    );
}

fn tilt_shift_enabled(quality: Quality, supported: bool, first_person: bool) -> bool {
    quality.smaa() && supported && !first_person
}

fn tilt_shift_requested() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("CRYSTAL_TILT_SHIFT").as_deref() != Ok("off"))
}

/// The world is measured in source pixels rather than meters. Scale the lens
/// with focus distance so zoom and viewport scale retain the same gentle blur.
/// At the pitched orbit angle, this produces a sharp strip through the trainer
/// and softer scenery in front of and behind it, using actual scene depth.
fn tilt_shift_lens(focal_distance: f32, framebuffer_height: f32) -> DepthOfField {
    DepthOfField {
        mode: DepthOfFieldMode::Gaussian,
        focal_distance,
        sensor_height: focal_distance * 0.04,
        aperture_f_stops: 0.8,
        max_circle_of_confusion_diameter: (framebuffer_height * 0.018).clamp(3.0, 28.0),
        max_depth: focal_distance * 2.5,
    }
}

fn sync_tilt_shift(
    mut commands: Commands,
    settings: Option<Res<crate::VoxelViewSettings>>,
    frame: Option<Res<crystal_render_api::VisualWorldFrame>>,
    terrain: Option<Res<crate::TerrainRevisionCache>>,
    traversal: Option<Res<crystal_render_api::VisualPlayerTraversal>>,
    adapter: Option<Res<RenderAdapterInfo>>,
    mut cameras: Query<
        (
            Entity,
            &Camera,
            &Projection,
            &Transform,
            Option<&mut DepthOfField>,
        ),
        With<crate::VoxelWorldCamera>,
    >,
) {
    let (Some(settings), Some(frame)) = (settings, frame) else {
        return;
    };
    // WebGL cannot sample the depth texture required by Bevy's DOF. Also
    // check the selected backend when a WebGPU build falls back to WebGL.
    let supported = bevy::core_pipeline::core_3d::DEPTH_PREPASS_TEXTURE_SUPPORTED
        && !(cfg!(target_arch = "wasm32")
            && adapter.as_ref().is_some_and(|a| a.backend.to_str() == "gl"));
    let enabled = tilt_shift_enabled(
        effective(adapter.as_deref()),
        supported,
        settings.camera.first_person,
    ) && tilt_shift_requested();
    let heights = terrain
        .as_ref()
        .map_or(&[][..], |t| t.footing_heights.as_slice());
    let focus = frame
        .actors
        .iter()
        .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
        .map(|actor| {
            let foot = crate::footing::actor_ground_point(&frame, actor);
            let ground =
                crate::footing::traversal_height(&frame, actor, heights, traversal.as_deref())
                    .map(|(height, _)| height)
                    .or_else(|| crate::footing::footing_height(&frame, foot))
                    .unwrap_or(0.0);
            // Focus on the body, between the feet and head, rather than the
            // ground or the camera rig's scenery-height composition target.
            crate::footing::visual_point_to_voxel(foot, ground + frame.tile_size.y)
        });
    for (entity, camera, projection, transform, current) in &mut cameras {
        let active = enabled
            && camera.is_active
            && frame.active
            && settings.enabled
            && matches!(projection, Projection::Perspective(_));
        let distance = focus.map(|focus| (focus - transform.translation).dot(*transform.forward()));
        let Some(distance) = distance.filter(|d| active && d.is_finite() && *d > 0.0) else {
            if current.is_some() {
                commands.entity(entity).remove::<DepthOfField>();
            }
            continue;
        };
        let height = camera
            .physical_viewport_size()
            .map_or(frame.viewport_size.y, |s| s.y as f32);
        let lens = tilt_shift_lens(distance, height);
        if let Some(mut current) = current {
            // Idle frames must not dirty the camera's extracted settings.
            if current.focal_distance != lens.focal_distance
                || current.sensor_height != lens.sensor_height
                || current.max_circle_of_confusion_diameter != lens.max_circle_of_confusion_diameter
                || current.max_depth != lens.max_depth
            {
                *current = lens;
            }
        } else {
            commands.entity(entity).insert(lens);
        }
    }
}

// Stacked cameras share a main buffer only when format and MSAA agree. A
// non-HDR UI camera would otherwise composite an empty LDR buffer over the
// completed HDR world. This changes storage only: no UI bloom, AO or tone curve.
fn sync_overlay_format(
    mut commands: Commands,
    world: Query<(&Camera, Has<Hdr>), With<crate::VoxelWorldCamera>>,
    overlays: Query<(Entity, Has<Hdr>, Has<WorldOverlayHdr>), With<crate::WorldOverlayCamera>>,
) {
    let hdr = world.iter().any(|(camera, hdr)| camera.is_active && hdr);
    for (entity, has_hdr, owned) in &overlays {
        if hdr && !has_hdr {
            commands.entity(entity).insert((Hdr, WorldOverlayHdr));
        } else if !hdr && owned {
            commands.entity(entity).remove::<(Hdr, WorldOverlayHdr)>();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Quality {
    Legacy,
    Low,
    Balanced,
    High,
}
impl Quality {
    fn parse(value: Option<&str>) -> Self {
        match value {
            Some("legacy") => Self::Legacy,
            Some("low") => Self::Low,
            Some("balanced") => Self::Balanced,
            Some("high") => Self::High,
            _ => {
                if cfg!(target_arch = "wasm32") {
                    Self::Balanced
                } else {
                    Self::High
                }
            }
        }
    }
    pub fn materials(self) -> bool {
        self != Self::Legacy
    }
    pub fn smaa(self) -> bool {
        matches!(self, Self::Balanced | Self::High)
    }
}
pub(super) fn requested() -> Quality {
    static QUALITY: OnceLock<Quality> = OnceLock::new();
    *QUALITY.get_or_init(|| Quality::parse(std::env::var("CRYSTAL_RENDER_QUALITY").ok().as_deref()))
}
pub(super) fn effective(adapter: Option<&RenderAdapterInfo>) -> Quality {
    let quality = requested();
    if quality != Quality::Legacy && adapter.is_some_and(|a| crate::software_renderer(&a.name)) {
        Quality::Low
    } else {
        quality
    }
}

pub(super) fn configure_camera(
    commands: &mut Commands,
    camera: Entity,
    adapter: Option<&RenderAdapterInfo>,
    device: Option<&RenderDevice>,
) {
    let quality = effective(adapter);
    commands
        .entity(camera)
        .insert(ShadowFilteringMethod::Gaussian);
    if quality.smaa() {
        commands.entity(camera).insert(Smaa {
            preset: SmaaPreset::High,
        });
    }
    if quality == Quality::High {
        // Neutral's shoulder retains authored midtones while allowing window
        // emission and specular glints to carry energy above display white.
        commands.entity(camera).insert((
            Hdr,
            Tonemapping::KhronosPbrNeutral,
            DebandDither::Enabled,
            ColorGrading::default(),
            Bloom {
                intensity: 0.035,
                low_frequency_boost: 0.15,
                prefilter: BloomPrefilter {
                    threshold: 1.0,
                    threshold_softness: 0.25,
                },
                composite_mode: BloomCompositeMode::Additive,
                max_mip_dimension: 256,
                ..Bloom::NATURAL
            },
        ));
    }
    // Bevy 0.20 SSAO is native-only. Check the actual device limits before
    // enabling its compute passes; software adapters retain baked grounding.
    #[cfg(not(target_arch = "wasm32"))]
    {
        let supported = adapter.is_some_and(|a| !crate::software_renderer(&a.name))
            && device.is_some_and(|d| d.limits().max_storage_textures_per_shader_stage >= 5);
        let requested_ao = match std::env::var("CRYSTAL_MODELED_OCCLUSION").as_deref() {
            Ok("off") => false,
            Ok("on") => true,
            _ => quality == Quality::High,
        };
        if supported && requested_ao {
            commands
                .entity(camera)
                .insert(bevy::pbr::ScreenSpaceAmbientOcclusion {
                    quality_level: bevy::pbr::ScreenSpaceAmbientOcclusionQualityLevel::High,
                    radius: 2.5,
                    constant_object_thickness: 0.8,
                });
        }
    }
    #[cfg(target_arch = "wasm32")]
    let _ = device;
}

/// Stable depth bins allocate cascades to the visible miniature rather than a
/// fixed 4096-unit halo. Use one cascade on the browser's WebGL-compatible path.
pub(super) fn shadow_config(
    viewport: Vec2,
    controls: crate::VoxelCameraControls,
    quality: Quality,
) -> CascadeShadowConfig {
    let pose = controls.pose(viewport);
    let distance = pose.eye.distance(pose.target);
    let scale = (viewport.y / 144.0).max(0.25);
    let far = ((distance + viewport.max_element() * 2.0 + 128.0 * scale) / (64.0 * scale)).ceil()
        * 64.0
        * scale;
    let far = far.clamp(256.0 * scale, 4096.0 * scale);
    CascadeShadowConfigBuilder {
        num_cascades: if cfg!(target_arch = "wasm32") {
            1
        } else {
            match quality {
                Quality::Low => 1,
                Quality::Balanced => 2,
                _ => 3,
            }
        },
        maximum_distance: far,
        first_cascade_far_bound: (distance + 64.0 * scale).min(far * 0.55),
        ..default()
    }
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blur_diameter(lens: &DepthOfField, fov: f32, depth: f32, height: f32) -> f32 {
        let focal_length = bevy::post_process::dof::calculate_focal_length(lens.sensor_height, fov);
        let scale = focal_length * focal_length / (lens.sensor_height * lens.aperture_f_stops);
        let depth = depth.min(lens.max_depth);
        (scale * (depth - lens.focal_distance).abs()
            / (depth * (lens.focal_distance - focal_length))
            * height)
            .min(lens.max_circle_of_confusion_diameter)
    }

    #[test]
    fn miniature_focus_keeps_trainer_sharp_and_scenery_soft_with_bounded_cost() {
        let pose = crate::camera_pose(Vec2::new(640.0, 576.0));
        let focus = pose.eye.distance(pose.target);
        let lens = tilt_shift_lens(focus, 720.0);
        let coc = |depth| blur_diameter(&lens, pose.vertical_fov_radians, depth, 720.0);
        assert_eq!(coc(focus), 0.0);
        for depth in [focus * 0.96, focus * 1.04] {
            assert!(coc(depth) < 1.75, "the trainer's body must stay readable");
        }
        for depth in [focus * 0.65, focus * 1.8] {
            assert!(
                coc(depth) > 10.0,
                "foreground/background softness should be visible"
            );
            assert!(coc(depth) <= 12.96);
        }
        assert!(coc(1.0) <= 12.96);
        assert!(coc(f32::INFINITY).is_finite());
        assert_eq!(
            tilt_shift_lens(focus, 4320.0).max_circle_of_confusion_diameter,
            28.0
        );
    }

    #[test]
    fn miniature_lens_preserves_softness_under_world_scaling_and_orbit_zoom() {
        let fov = crate::camera_pose(Vec2::new(640.0, 576.0)).vertical_fov_radians;
        for focus in [144.0, 288.0, 576.0, 1152.0] {
            for ratio in [0.65, 0.96, 1.04, 1.8] {
                let base = blur_diameter(&tilt_shift_lens(576.0, 720.0), fov, 576.0 * ratio, 720.0);
                let scaled =
                    blur_diameter(&tilt_shift_lens(focus, 720.0), fov, focus * ratio, 720.0);
                assert!((base - scaled).abs() < 0.00001);
                let retina =
                    blur_diameter(&tilt_shift_lens(focus, 1440.0), fov, focus * ratio, 1440.0);
                assert!((retina - scaled * 2.0).abs() < 0.00001);
            }
        }
        assert!(!tilt_shift_enabled(Quality::Low, true, false));
        assert!(!tilt_shift_enabled(Quality::Legacy, true, false));
        assert!(!tilt_shift_enabled(Quality::Balanced, false, false));
        assert!(!tilt_shift_enabled(Quality::High, true, true));
        assert!(tilt_shift_enabled(Quality::Balanced, true, false));
        assert!(tilt_shift_enabled(Quality::High, true, false));
    }

    #[test]
    fn miniature_focus_follows_actual_camera_and_player_height_and_leaves_ui_sharp() {
        use crystal_render_api::{VisualActor, VisualActorId, VisualWorldFrame};
        let mut app = App::new();
        let viewport = Vec2::new(640.0, 576.0);
        let frame = VisualWorldFrame {
            active: true,
            viewport_size: viewport,
            tile_size: Vec2::splat(32.0),
            grid_size: UVec2::new(20, 18),
            actors: vec![VisualActor {
                id: VisualActorId::Player,
                center: Vec2::ZERO,
                size: Vec2::splat(64.0),
                source_id: "player".into(),
                model_palette: None,
                texture: default(),
                facing: Some(Vec2::Y),
                flip_x: false,
                above_priority: false,
            }],
            ..default()
        };
        app.insert_resource(frame)
            .insert_resource(crate::VoxelViewSettings {
                enabled: true,
                ..default()
            })
            .insert_resource(crate::TerrainRevisionCache {
                footing_heights: vec![7.0; 360],
                ..default()
            })
            .add_systems(PostUpdate, sync_tilt_shift);
        let camera = app
            .world_mut()
            .spawn((
                crate::VoxelWorldCamera,
                Camera3d::default(),
                Projection::Perspective(default()),
                Transform::default(),
            ))
            .id();
        let ui = app
            .world_mut()
            .spawn((crate::WorldOverlayCamera, Camera2d))
            .id();
        let capture = app.world_mut().spawn(Camera3d::default()).id();
        for zoom in [0.0, 1.0, 5.0] {
            for yaw in [0.0, 1.5, 4.0, 6.0] {
                let pose = crate::VoxelCameraControls::new(zoom, yaw).pose(viewport);
                let transform = pose.transform();
                app.world_mut().entity_mut(camera).insert(transform);
                app.update();
                if requested().smaa() {
                    let lens = app.world().get::<DepthOfField>(camera).unwrap();
                    let expected = (Vec3::new(0.0, 39.0, 0.0) - pose.eye).dot(*transform.forward());
                    assert!((lens.focal_distance - expected).abs() < 0.001);
                }
                assert!(app.world().get::<DepthOfField>(ui).is_none());
                assert!(app.world().get::<DepthOfField>(capture).is_none());
            }
        }
        app.world_mut()
            .resource_mut::<crate::VoxelViewSettings>()
            .camera
            .first_person = true;
        app.update();
        assert!(app.world().get::<DepthOfField>(camera).is_none());
        app.world_mut()
            .resource_mut::<crate::VoxelViewSettings>()
            .camera
            .first_person = false;
        app.update();
        assert_eq!(
            app.world().get::<DepthOfField>(camera).is_some(),
            requested().smaa()
        );
        app.world_mut().get_mut::<Camera>(camera).unwrap().is_active = false;
        app.update();
        assert!(app.world().get::<DepthOfField>(camera).is_none());
    }

    #[test]
    fn overlay_shares_world_storage_and_restores_exact_source_camera_on_f3() {
        let mut app = App::new();
        let world = app
            .world_mut()
            .spawn((crate::VoxelWorldCamera, Camera::default(), Hdr))
            .id();
        let overlay = app
            .world_mut()
            .spawn((crate::WorldOverlayCamera, Camera2d, Tonemapping::None))
            .id();
        let unrelated = app.world_mut().spawn(Camera2d).id();
        register(&mut app);
        app.update();
        assert!(app.world().get::<Hdr>(overlay).is_some());
        assert!(app.world().get::<Hdr>(unrelated).is_none());
        assert!(app.world().get::<Bloom>(overlay).is_none());
        assert!(app.world().get::<Smaa>(overlay).is_none());
        assert_eq!(
            app.world().get::<Tonemapping>(overlay),
            Some(&Tonemapping::None)
        );
        app.world_mut().get_mut::<Camera>(world).unwrap().is_active = false;
        app.update();
        assert!(app.world().get::<Hdr>(overlay).is_none());
        assert!(app.world().get::<WorldOverlayHdr>(overlay).is_none());
        assert_eq!(
            app.world().get::<Tonemapping>(overlay),
            Some(&Tonemapping::None)
        );
        app.world_mut().get_mut::<Camera>(world).unwrap().is_active = true;
        app.update();
        assert!(app.world().get::<Hdr>(overlay).is_some());
    }
    #[test]
    fn effects_are_confined_to_selected_world_camera() {
        let mut app = App::new();
        let world_camera = app.world_mut().spawn_empty().id();
        let ui_camera = app.world_mut().spawn_empty().id();
        app.add_systems(Update, move |mut commands: Commands| {
            configure_camera(&mut commands, world_camera, None, None);
        });
        app.update();
        assert!(app.world().get::<Hdr>(ui_camera).is_none());
        assert!(app.world().get::<Smaa>(ui_camera).is_none());
        assert!(app.world().get::<Bloom>(ui_camera).is_none());
        assert!(
            app.world()
                .get::<bevy::pbr::ScreenSpaceAmbientOcclusion>(world_camera)
                .is_none()
        );
        assert_eq!(
            app.world().get::<Smaa>(world_camera).is_some(),
            requested().smaa()
        );
    }
    #[test]
    fn shadow_coverage_tracks_zoom_and_world_scale_without_map_switches() {
        let viewport = Vec2::new(160.0, 144.0);
        let wide = shadow_config(
            viewport,
            crate::VoxelCameraControls::new(0.0, 0.0),
            Quality::High,
        );
        let close = shadow_config(
            viewport,
            crate::VoxelCameraControls::new(5.0, 0.0),
            Quality::High,
        );
        let scaled = shadow_config(
            viewport * 2.0,
            crate::VoxelCameraControls::new(5.0, 0.0),
            Quality::High,
        );
        assert!(wide.bounds.last().unwrap() > close.bounds.last().unwrap());
        assert!(*close.bounds.last().unwrap() < 1024.0);
        for (a, b) in close.bounds.iter().zip(&scaled.bounds) {
            assert!((a * 2.0 - b).abs() < 0.01);
        }
    }
    #[test]
    fn presets_keep_browser_and_low_cost_fallbacks() {
        assert!(!Quality::parse(Some("legacy")).materials());
        assert!(Quality::parse(Some("low")).materials());
        assert!(!Quality::parse(Some("low")).smaa());
        assert!(Quality::parse(Some("balanced")).smaa());
        assert!(Quality::parse(Some("high")).smaa());
    }
}
