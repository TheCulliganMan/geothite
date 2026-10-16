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

#[derive(Component)]
struct WorldOverlayHdr;

pub(super) fn register(app: &mut App) {
    app.add_systems(PostUpdate, sync_overlay_format);
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
