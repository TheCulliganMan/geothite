//! Skip unchanged lighting publication without hiding material asset edits.

use bevy::asset::Assets;
use bevy::ecs::change_detection::DetectChanges;
use bevy::ecs::system::ResMut;
use bevy::math::Vec4;
use std::sync::OnceLock;

use super::MAX_LIGHTS;
use crate::VoxelMaterial;

static ALWAYS_SCAN: OnceLock<bool> = OnceLock::new();

#[derive(PartialEq)]
pub(crate) struct Lighting {
    pub night: Vec4,
    pub panorama_night: Vec4,
    pub positions: [Vec4; MAX_LIGHTS],
    pub directions: [Vec4; MAX_LIGHTS],
}

pub(super) fn sync(
    materials: &mut ResMut<Assets<VoxelMaterial>>,
    previous: &mut Option<Lighting>,
    next: Lighting,
) {
    // Frame/status changes alone do not change shader inputs. Asset changes
    // invalidate the whole scan, including new clones and same-ID replacement.
    // Reading the asset collection here must not mark it changed.
    // Startup-only native QA control for comparisons in the same executable.
    let always_scan = *ALWAYS_SCAN.get_or_init(|| {
        cfg!(not(target_arch = "wasm32"))
            && std::env::var("CRYSTAL_CITY_LIGHTING_SCAN").as_deref() == Ok("always")
    });
    let skip = !always_scan && !materials.is_changed() && previous.as_ref() == Some(&next);
    let _trace = crate::loading_trace::span("frame.city_lighting_materials", || {
        serde_json::json!({
            "skipped": skip,
            "assets_changed": materials.is_changed(),
            "materials": materials.len(),
        })
    });
    if skip {
        return;
    }
    let no_windows = [Vec4::ZERO; MAX_LIGHTS];
    let changed: Vec<_> = materials
        .iter()
        .filter_map(|(id, material)| {
            let panorama = material.extension.cave_reveal.z > 1.5;
            let (night, windows, planes) = if panorama {
                (next.panorama_night, &no_windows, &no_windows)
            } else {
                (next.night, &next.positions, &next.directions)
            };
            (material.extension.city_night != night
                || material.extension.city_windows != *windows
                || material.extension.city_window_planes != *planes)
                .then_some((id, panorama))
        })
        .collect();
    for (id, panorama) in changed {
        if let Some(mut material) = materials.get_mut(id) {
            material.extension.city_night = if panorama {
                next.panorama_night
            } else {
                next.night
            };
            material.extension.city_windows = if panorama { no_windows } else { next.positions };
            material.extension.city_window_planes = if panorama {
                no_windows
            } else {
                next.directions
            };
        }
    }
    *previous = Some(next);
}

#[cfg(test)]
mod tests {
    use bevy::app::{App, Update};
    use bevy::asset::{AssetApp, AssetEvent, AssetPlugin, Assets};
    use bevy::color::Color;
    use bevy::ecs::message::Messages;
    use bevy::math::{Vec2, Vec4};
    use bevy::prelude::MinimalPlugins;
    use crystal_render_api::{VisualWorldClock, VisualWorldFrame};

    use crate::{TerrainRevisionCache, VoxelMaterial, VoxelViewStatus};

    #[test]
    fn unchanged_lighting_still_repairs_asset_edits_and_same_id_replacement() {
        let frame = VisualWorldFrame {
            active: true,
            map_id: "GoldenrodCity".into(),
            tile_size: Vec2::splat(32.0),
            ..Default::default()
        };
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<VoxelMaterial>()
            .insert_resource(frame.clone())
            .insert_resource(VisualWorldClock {
                minute_of_day: Some(22 * 60),
                second_of_minute: 0,
            })
            .insert_resource(VoxelViewStatus {
                active: true,
                ..Default::default()
            })
            .insert_resource(TerrainRevisionCache {
                built_frame: Some(frame),
                city_emitters: vec![super::super::Emitter {
                    position: Vec4::new(1.0, 16.0, 2.0, 64.0),
                    direction: Vec4::Z,
                }],
                ..Default::default()
            })
            .add_systems(Update, super::super::sync);
        let handle = app
            .world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .add(crate::solid_terrain_material());
        for _ in 0..4 {
            app.update();
        }
        let expected = Vec4::new(1.0, 1.0, 0.0, 0.0);
        assert_eq!(
            app.world()
                .resource::<Assets<VoxelMaterial>>()
                .get(&handle)
                .expect("live material")
                .extension
                .city_night,
            expected
        );

        app.world_mut()
            .resource_mut::<Messages<AssetEvent<VoxelMaterial>>>()
            .clear();
        // Ordinary presentation changes must not cause an idle GPU re-upload.
        app.world_mut()
            .resource_mut::<VoxelViewStatus>()
            .active_frames += 1;
        app.world_mut().resource_mut::<VisualWorldFrame>().center.x += 1.0;
        app.update();
        assert!(
            !app.world_mut()
                .resource_mut::<Messages<AssetEvent<VoxelMaterial>>>()
                .drain()
                .any(|event| matches!(event, AssetEvent::Modified { .. }))
        );

        // Replacing an asset at the same ID must not reuse old publication state.
        let mut replacement = crate::solid_terrain_material();
        replacement.base.base_color = Color::srgb(0.2, 0.4, 0.6);
        replacement.extension.cutaway.fade.x = 0.25;
        app.world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .insert(handle.id(), replacement)
            .expect("replace live material");
        app.update();
        {
            let materials = app.world().resource::<Assets<VoxelMaterial>>();
            let material = materials.get(&handle).expect("replaced material");
            assert_eq!(material.extension.city_night, expected);
            assert_eq!(material.extension.city_windows[0].w, 64.0);
            assert_eq!(material.extension.cutaway.fade.x, 0.25);
            assert_eq!(material.base.base_color, Color::srgb(0.2, 0.4, 0.6));
        }
        // An edit can also change which of the two lighting outputs is required.
        {
            let mut materials = app.world_mut().resource_mut::<Assets<VoxelMaterial>>();
            let mut material = materials.get_mut(&handle).expect("editable material");
            material.extension.cave_reveal.z = 2.0;
            material.extension.city_night = Vec4::splat(9.0);
        }
        app.update();
        let materials = app.world().resource::<Assets<VoxelMaterial>>();
        let material = materials.get(&handle).expect("reclassified material");
        // Goldenrod has no isolated panorama: the material must be cleared now.
        assert_eq!(material.extension.city_night, Vec4::ZERO);
        assert_eq!(
            material.extension.city_windows,
            [Vec4::ZERO; super::MAX_LIGHTS]
        );
        assert_eq!(
            material.extension.city_window_planes,
            [Vec4::ZERO; super::MAX_LIGHTS]
        );
    }
}
