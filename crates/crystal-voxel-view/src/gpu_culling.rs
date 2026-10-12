//! Lossless native GPU-culling experiment; never enabled without measurement.
use crate::{VoxelSun, VoxelWorldCamera, sync_voxel_view};
use bevy::{
    core_pipeline::prepass::DepthPrepass, prelude::*, render::occlusion_culling::OcclusionCulling,
};
use crystal_render_api::VisualWorldFrame;

#[derive(Resource, Clone, Copy)]
struct Mode {
    camera: bool,
    shadows: bool,
}

pub(super) fn register(app: &mut App) {
    let mode = match std::env::var("CRYSTAL_GPU_CULLING").as_deref() {
        Ok("camera") => Mode {
            camera: true,
            shadows: false,
        },
        Ok("shadows") => Mode {
            camera: false,
            shadows: true,
        },
        Ok("on") => Mode {
            camera: true,
            shadows: true,
        },
        _ => return,
    };
    app.insert_resource(mode)
        .add_systems(Update, sync.after(sync_voxel_view));
}

// Track only components we inserted, so leaving a supported map cannot remove
// a prepass or culling component installed by another renderer feature.
#[derive(Component, Clone, Copy)]
struct Applied {
    depth: bool,
    culling: bool,
}

fn supported(map: &str) -> bool {
    matches!(map, "NewBarkTown" | "Route29")
}

fn sync(
    mut commands: Commands,
    frame: Res<VisualWorldFrame>,
    mode: Res<Mode>,
    views: Query<
        (
            Entity,
            Has<DepthPrepass>,
            Has<OcclusionCulling>,
            Option<&Applied>,
            Has<VoxelWorldCamera>,
        ),
        Or<(With<VoxelWorldCamera>, With<VoxelSun>)>,
    >,
) {
    for (entity, depth, culling, applied, camera) in &views {
        let enabled = supported(&frame.map_id) && if camera { mode.camera } else { mode.shadows };
        if enabled {
            if applied.is_some() {
                continue;
            }
            let owned = Applied {
                depth: camera && !depth,
                culling: !culling,
            };
            let mut view = commands.entity(entity);
            if owned.depth {
                view.insert(DepthPrepass);
            }
            if owned.culling {
                view.insert(OcclusionCulling);
            }
            view.insert(owned);
        } else if let Some(owned) = applied {
            let mut view = commands.entity(entity);
            if owned.depth {
                view.remove::<DepthPrepass>();
            }
            if owned.culling {
                view.remove::<OcclusionCulling>();
            }
            view.remove::<Applied>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_culling_is_restricted_to_verified_outdoors_and_preserves_existing_components() {
        let mut app = App::new();
        let mut frame = VisualWorldFrame::default();
        frame.map_id = "NewBarkTown".into();
        app.insert_resource(frame)
            .insert_resource(Mode {
                camera: true,
                shadows: true,
            })
            .add_systems(Update, sync);
        let camera = app.world_mut().spawn(VoxelWorldCamera).id();
        let sun = app.world_mut().spawn(VoxelSun).id();
        let existing = app
            .world_mut()
            .spawn((VoxelWorldCamera, DepthPrepass, OcclusionCulling))
            .id();
        app.update();
        assert!(app.world().get::<DepthPrepass>(camera).is_some());
        assert!(app.world().get::<OcclusionCulling>(camera).is_some());
        assert!(app.world().get::<OcclusionCulling>(sun).is_some());
        assert!(app.world().get::<DepthPrepass>(sun).is_none());
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id = "IcePathB1F".into();
        app.update();
        assert!(app.world().get::<DepthPrepass>(camera).is_none());
        assert!(app.world().get::<OcclusionCulling>(camera).is_none());
        assert!(app.world().get::<OcclusionCulling>(sun).is_none());
        assert!(app.world().get::<DepthPrepass>(existing).is_some());
        assert!(app.world().get::<OcclusionCulling>(existing).is_some());
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id = "Route29".into();
        app.update();
        assert!(app.world().get::<OcclusionCulling>(sun).is_some());
    }

    #[test]
    fn gpu_camera_and_shadow_culling_can_be_measured_independently() {
        for mode in [
            Mode {
                camera: true,
                shadows: false,
            },
            Mode {
                camera: false,
                shadows: true,
            },
        ] {
            let mut app = App::new();
            let mut frame = VisualWorldFrame::default();
            frame.map_id = "Route29".into();
            app.insert_resource(frame)
                .insert_resource(mode)
                .add_systems(Update, sync);
            let camera = app.world_mut().spawn(VoxelWorldCamera).id();
            let sun = app.world_mut().spawn(VoxelSun).id();
            app.update();
            assert_eq!(
                app.world().get::<OcclusionCulling>(camera).is_some(),
                mode.camera
            );
            assert_eq!(
                app.world().get::<DepthPrepass>(camera).is_some(),
                mode.camera
            );
            assert_eq!(
                app.world().get::<OcclusionCulling>(sun).is_some(),
                mode.shadows
            );
        }
    }
}
