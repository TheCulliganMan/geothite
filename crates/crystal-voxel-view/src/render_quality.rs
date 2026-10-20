//! Camera effects belong to the world camera, never the classic UI compositor.
use bevy::{
    anti_alias::smaa::{Smaa, SmaaPreset},
    camera::Hdr,
    light::{CascadeShadowConfig, CascadeShadowConfigBuilder, ShadowFilteringMethod},
    post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter},
    prelude::*,
    render::{
        renderer::{RenderAdapterInfo, RenderDevice},
        view::{ColorGrading, DebandDither, Tonemapping},
    },
};
use std::sync::OnceLock;

#[path = "tilt_shift.rs"]
mod tilt_shift;
use tilt_shift::TiltShift;

#[derive(Component)]
struct WorldOverlayHdr;

pub(super) fn register(app: &mut App) {
    tilt_shift::register(app);
    app.add_systems(Update, sync_cave_palette_tonemapping.after(crate::sync_voxel_view));
    app.add_systems(
        PostUpdate,
        (
            sync_overlay_format,
            sync_tilt_shift.after(bevy::camera::CameraUpdateSystems),
        ),
    );
}

/// The LDR cave palette already encodes dark mineral pigment. PBR Neutral's
/// low-end subtraction removes its weak green channel and repaints it purple.
/// Keep HDR/bloom/dithering and display source pigment with a linear transfer.
/// Restore each camera's exact prior curve on exit, F3 or a hidden world.
fn sync_cave_palette_tonemapping(
    frame: Option<Res<crystal_render_api::VisualWorldFrame>>,
    settings: Option<Res<crate::VoxelViewSettings>>,
    mut cameras: Query<(Entity, &mut Tonemapping), With<crate::VoxelWorldCamera>>,
    mut previous: Local<std::collections::HashMap<Entity, Tonemapping>>,
) {
    let source_palette = frame.as_deref().zip(settings.as_deref()).is_some_and(|(frame, settings)|
        frame.active && settings.enabled && crate::cave_water::native_cave_source(frame));
    for (entity, mut tone) in &mut cameras {
        if source_palette {
            if *tone != Tonemapping::Linear {
                previous.insert(entity, *tone);
                *tone = Tonemapping::Linear;
            }
        } else if let Some(original) = previous.remove(&entity) {
            if *tone != original { *tone = original; }
        }
    }
    previous.retain(|entity, _| cameras.contains(*entity));
}

fn tilt_shift_enabled(quality: Quality, first_person: bool) -> bool {
    quality.smaa() && !first_person
}

fn tilt_shift_requested() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("CRYSTAL_TILT_SHIFT").as_deref() != Ok("off"))
}

fn exploration_focus(frame: &crystal_render_api::VisualWorldFrame) -> bool {
    crate::cave_water::outdoor(&frame.map_id) && frame.tiles.first().is_some_and(|tile|
        matches!(tile.source.tileset_id.as_ref(), "johto" | "johto_modern" | "kanto"))
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
            Option<&mut TiltShift>,
        ),
        With<crate::VoxelWorldCamera>,
    >,
) {
    let (Some(settings), Some(frame)) = (settings, frame) else {
        return;
    };
    let enabled = tilt_shift_enabled(effective(adapter.as_deref()), settings.camera.first_person)
        && tilt_shift_requested();
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
        let height = camera
            .physical_viewport_size()
            .map_or(frame.viewport_size.y, |s| s.y as f32);
        let band = focus.filter(|_| active).and_then(|focus| {
            TiltShift::for_trainer(projection, transform, focus, frame.tile_size.y, height)
                .map(|band|if exploration_focus(&frame) {band.readable_exterior()} else {band})
        });
        let Some(band) = band else {
            if current.is_some() {
                commands.entity(entity).remove::<TiltShift>();
            }
            continue;
        };
        if let Some(mut current) = current {
            // Idle frames must not dirty the camera's extracted settings.
            if *current != band {
                *current = band;
            }
        } else {
            commands.entity(entity).insert(band);
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

fn smaa_preset(quality: Quality, override_value: Option<&str>) -> Option<SmaaPreset> {
    if !quality.smaa() || override_value == Some("off") {
        return None;
    }
    Some(match override_value {
        Some("smaa-high") => SmaaPreset::High,
        Some("smaa-ultra") => SmaaPreset::Ultra,
        _ if quality == Quality::High => SmaaPreset::Ultra,
        _ => SmaaPreset::High,
    })
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
    if let Some(preset) = smaa_preset(quality,
        std::env::var("CRYSTAL_ANTI_ALIASING").ok().as_deref()) {
        commands.entity(camera).insert(Smaa { preset });
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
                    // Eight spatial samples retain Bevy's depth-aware denoiser
                    // and contact radius without the 18-sample full-screen cost.
                    quality_level: bevy::pbr::ScreenSpaceAmbientOcclusionQualityLevel::Medium,
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
                // Keep the nearest cascade's established range/resolution.
                // A single outer cascade avoids a third full geometry pass.
                _ => 2,
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

    #[test]
    fn cave_palette_preserves_hdr_and_restores_the_exact_camera_curve() {
        use crystal_render_api::{VisualWorldFrame,VisualTile,VisualTileSource};
        let mut app=App::new();
        app.init_resource::<VisualWorldFrame>().init_resource::<crate::VoxelViewSettings>()
            .add_systems(Update,sync_cave_palette_tonemapping);
        app.world_mut().resource_mut::<crate::VoxelViewSettings>().enabled=true;
        let source=VisualWorldFrame {active:true,map_id:"DarkCaveVioletEntrance".into(),
            tiles:vec![VisualTile {column:0,row:0,texture:Default::default(),priority:false,
                animation_frames:None,source:VisualTileSource {tileset_id:"dark_cave".into(),
                    metatile_id:2,tile_index:0x16,subtile_column:0,subtile_row:0}}],..default()};
        let camera=app.world_mut().spawn((crate::VoxelWorldCamera,Hdr,Bloom::default(),DebandDither::Enabled,
            Tonemapping::KhronosPbrNeutral)).id();
        let other=app.world_mut().spawn(Tonemapping::KhronosPbrNeutral).id();
        for original in [Tonemapping::None,Tonemapping::Linear,Tonemapping::KhronosPbrNeutral,Tonemapping::Reinhard] {
            *app.world_mut().get_mut::<Tonemapping>(camera).unwrap()=original;
            *app.world_mut().resource_mut::<VisualWorldFrame>()=source.clone();
            app.update();
            assert_eq!(app.world().get::<Tonemapping>(camera),Some(&Tonemapping::Linear));
            assert!(app.world().get::<Hdr>(camera).is_some());
            assert!(app.world().get::<Bloom>(camera).is_some());
            assert_eq!(app.world().get::<DebandDither>(camera),Some(&DebandDither::Enabled));
            assert_eq!(*app.world().resource::<VisualWorldFrame>(),source);
            assert_eq!(app.world().get::<Tonemapping>(other),Some(&Tonemapping::KhronosPbrNeutral));
            app.world_mut().resource_mut::<VisualWorldFrame>().active=false;
            app.update();
            assert_eq!(app.world().get::<Tonemapping>(camera),Some(&original));
            app.world_mut().resource_mut::<VisualWorldFrame>().active=true;
            app.update();
            app.world_mut().resource_mut::<crate::VoxelViewSettings>().enabled=false;
            app.update();
            assert_eq!(app.world().get::<Tonemapping>(camera),Some(&original));
            app.world_mut().resource_mut::<crate::VoxelViewSettings>().enabled=true;
            app.update();
            app.world_mut().resource_mut::<VisualWorldFrame>().map_id="PlayersHouse1F".into();
            app.world_mut().resource_mut::<VisualWorldFrame>().tiles[0].source.tileset_id="house".into();
            app.update();
            assert_eq!(app.world().get::<Tonemapping>(camera),Some(&original));
        }
        *app.world_mut().resource_mut::<VisualWorldFrame>()=source;
        app.update();
        assert_eq!(app.world().get::<Tonemapping>(camera),Some(&Tonemapping::Linear));
        app.world_mut().remove_resource::<VisualWorldFrame>();
        app.update();
        assert_eq!(app.world().get::<Tonemapping>(camera),Some(&Tonemapping::Reinhard));
    }

    #[test]
    fn antialiasing_presets_keep_fallbacks_and_allow_a_same_binary_comparison() {
        assert!(smaa_preset(Quality::High, None) == Some(SmaaPreset::Ultra));
        assert!(smaa_preset(Quality::Balanced, None) == Some(SmaaPreset::High));
        assert!(smaa_preset(Quality::High, Some("smaa-high")) == Some(SmaaPreset::High));
        assert!(smaa_preset(Quality::Balanced, Some("smaa-ultra")) == Some(SmaaPreset::Ultra));
        assert!(smaa_preset(Quality::High, Some("off")) == None);
        assert!(smaa_preset(Quality::High, Some("unknown")) == Some(SmaaPreset::Ultra));
        for quality in [Quality::Low, Quality::Legacy] {
            assert!(smaa_preset(quality, Some("smaa-ultra")) == None);
        }
    }

    #[test]
    fn exploration_focus_preserves_nearby_scenery_and_excludes_rooms_and_forests() {
        use crystal_render_api::{VisualWorldFrame,VisualTile,VisualTileSource};
        let pose=crate::camera_pose(Vec2::new(640.,576.));
        let projection=Projection::Perspective(PerspectiveProjection {
            fov:pose.vertical_fov_radians,..default()
        });
        let original=TiltShift::for_trainer(&projection,&pose.transform(),
            Vec3::Y*32.,32.,720.).unwrap();
        let readable=original.readable_exterior();
        assert_eq!(readable.band.x,original.band.x);
        assert_eq!(readable.band.z,original.band.z);
        assert!(readable.band.y>=0.20);
        for offset in [-0.20,-0.14,0.,0.14,0.20] {
            assert!(readable.blur_diameter(readable.band.x+offset)<0.00001);
        }
        for y in [0.,0.1,0.25,0.75,0.9,1.] {
            assert!(readable.blur_diameter(y)<=original.blur_diameter(y)+0.00001);
        }
        assert!(readable.band.w>6. && readable.band.w<8.,"gentle edge softness remains");
        for (map,tileset,expected) in [
            ("MahoganyTown","johto",true),("GoldenrodCity","johto_modern",true),
            ("Route26","kanto",true),("CinnabarIsland","kanto",true),
            ("IlexForest","forest",false),("DragonsDenB1F","johto",false),
            ("Route35NationalParkGate","gate",false),("PlayersHouse1F","house",false),
            ("MahoganyTown","unknown",false),
        ] {
            let frame=VisualWorldFrame {map_id:map.into(),tiles:vec![VisualTile {
                column:0,row:0,source:VisualTileSource {tileset_id:tileset.into(),
                    metatile_id:0,subtile_column:0,subtile_row:0,tile_index:0},
                texture:default(),animation_frames:None,priority:false,
            }],..default()};
            assert_eq!(exploration_focus(&frame),expected,"{map}/{tileset}");
        }
        assert!(!exploration_focus(&VisualWorldFrame {map_id:"MahoganyTown".into(),..default()}));
    }

    #[test]
    fn focus_band_preserves_the_trainer_and_fades_smoothly_into_visible_blur() {
        let pose = crate::camera_pose(Vec2::new(640.0, 576.0));
        let projection = Projection::Perspective(PerspectiveProjection {
            fov: pose.vertical_fov_radians,
            ..default()
        });
        let torso = Vec3::new(0.0, 32.0, 0.0);
        let band =
            TiltShift::for_trainer(&projection, &pose.transform(), torso, 32.0, 720.0).unwrap();
        let center = band.band.x;
        assert_eq!(band.blur_diameter(center), 0.0);
        assert!(band.blur_diameter(center - 0.14) < 0.00001);
        assert!(band.blur_diameter(center + 0.14) < 0.00001);
        assert!(band.blur_diameter(center + band.band.y) < 0.00001);
        assert!(band.blur_diameter(center + band.band.y + 0.001) < 0.001);
        // A shifted focus must retain the gradient to BOTH screen edges,
        // rather than reaching maximum blur early on its longer side.
        for center in [center, 0.22, 0.78] {
            let mut band = band;
            band.band.x = center;
            for edge in [0.0, 1.0] {
                let direction = (edge - center).signum();
                let start = center + direction * band.band.y;
                let row = |t: f32| start + (edge - start) * t;
                let mut previous = 0.0;
                for step in 0..=100 {
                    let blur = band.blur_diameter(row(step as f32 / 100.0));
                    assert!(blur >= previous - 0.00001, "monotonic focus falloff");
                    assert!(blur <= band.band.w + 0.00001);
                    previous = blur;
                }
                assert!(
                    band.blur_diameter(row(0.5)) < band.band.w * 0.20,
                    "nearby scenery must retain detail"
                );
                let outer = [0.75, 0.9, 1.0].map(|t| band.blur_diameter(row(t)));
                assert!(outer[1] - outer[0] > band.band.w * 0.15);
                assert!(
                    outer[2] - outer[1] > band.band.w * 0.15,
                    "the outer edge must still have a visible gradient"
                );
                assert!((outer[2] - band.band.w).abs() < 0.0001);
            }
        }
        assert!((band.band.w - 12.96).abs() < 0.001);
        assert_eq!(
            TiltShift::for_trainer(&projection, &pose.transform(), torso, 32.0, 4320.0)
                .unwrap()
                .band
                .w,
            36.0
        );
        assert!(
            TiltShift::for_trainer(
                &projection,
                &pose.transform(),
                pose.eye - *pose.transform().forward(),
                32.0,
                720.0
            )
            .is_none()
        );
    }

    #[test]
    fn focus_band_protects_the_full_trainer_at_every_zoom_and_scales_with_resolution() {
        let viewport = Vec2::new(640.0, 576.0);
        let torso = Vec3::new(0.0, 32.0, 0.0);
        for zoom in [0.0, 1.0, 5.0] {
            let pose = crate::VoxelCameraControls::new(zoom, 0.0).pose(viewport);
            let projection = Projection::Perspective(PerspectiveProjection {
                fov: pose.vertical_fov_radians,
                ..default()
            });
            let band =
                TiltShift::for_trainer(&projection, &pose.transform(), torso, 32.0, 720.0).unwrap();
            let retina =
                TiltShift::for_trainer(&projection, &pose.transform(), torso, 32.0, 1440.0)
                    .unwrap();
            assert_eq!(retina.band.w, band.band.w * 2.0);
            let matrix = projection.get_clip_from_view() * pose.transform().to_matrix().inverse();
            for point in [Vec3::ZERO, torso, Vec3::Y * 64.0] {
                let clip = matrix * point.extend(1.0);
                assert_eq!(band.blur_diameter(0.5 - clip.y / clip.w * 0.5), 0.0);
            }
        }
        assert!(!tilt_shift_enabled(Quality::Low, false));
        assert!(!tilt_shift_enabled(Quality::Legacy, false));
        assert!(!tilt_shift_enabled(Quality::High, true));
        assert!(tilt_shift_enabled(Quality::Balanced, false));
        assert!(tilt_shift_enabled(Quality::High, false));
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
                    let band = app.world().get::<TiltShift>(camera).unwrap();
                    let projection = app.world().get::<Projection>(camera).unwrap();
                    let clip = projection.get_clip_from_view()
                        * transform.to_matrix().inverse()
                        * Vec3::new(0.0, 39.0, 0.0).extend(1.0);
                    let expected = 0.5 - clip.y / clip.w * 0.5;
                    assert!((band.band.x - expected).abs() < 0.001);
                }
                assert!(app.world().get::<TiltShift>(ui).is_none());
                assert!(app.world().get::<TiltShift>(capture).is_none());
            }
        }
        app.world_mut()
            .resource_mut::<crate::VoxelViewSettings>()
            .camera
            .first_person = true;
        app.update();
        assert!(app.world().get::<TiltShift>(camera).is_none());
        app.world_mut()
            .resource_mut::<crate::VoxelViewSettings>()
            .camera
            .first_person = false;
        app.update();
        assert_eq!(
            app.world().get::<TiltShift>(camera).is_some(),
            requested().smaa()
        );
        app.world_mut().get_mut::<Camera>(camera).unwrap().is_active = false;
        app.update();
        assert!(app.world().get::<TiltShift>(camera).is_none());
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
            smaa_preset(requested(), std::env::var("CRYSTAL_ANTI_ALIASING").ok().as_deref()).is_some()
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
