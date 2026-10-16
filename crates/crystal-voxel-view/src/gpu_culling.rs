//! Lossless native GPU-culling experiment; never enabled without measurement.
use crate::{VoxelSun, VoxelWorldCamera, sync_voxel_view};
use bevy::{
    core_pipeline::prepass::DepthPrepass, prelude::*, render::occlusion_culling::OcclusionCulling,
    render::{Render, RenderApp, RenderSystems,
        batching::gpu_preprocessing::{GpuPreprocessingMode, GpuPreprocessingSupport},
        renderer::{RenderAdapterInfo, RenderDevice}},
};
use crystal_render_api::VisualWorldFrame;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

mod shadow_bridge;

#[derive(Resource, Clone, Copy)]
struct Mode {
    camera: bool,
    shadows: bool,
    selective_depth: bool,
}

#[derive(Resource, Clone, Default)]
struct DeviceSupport(Arc<AtomicBool>);

#[derive(Resource)]
struct SupportTrace(bool);

pub(super) fn register(app: &mut App) {
    let selective_depth = crate::TREE_DEPTH_PREPASS.get().copied().unwrap_or(false);
    let mode = match std::env::var("CRYSTAL_GPU_CULLING").as_deref() {
        Ok("camera") => Mode {
            camera: true,
            shadows: false,
            selective_depth,
        },
        Ok("shadows") => Mode {
            camera: false,
            shadows: true,
            selective_depth,
        },
        Ok("on") => Mode {
            camera: true,
            shadows: true,
            selective_depth,
        },
        _ => Mode { camera: false, shadows: false, selective_depth },
    };
    let trace = std::env::var("CRYSTAL_GPU_CULLING_TRACE").as_deref() == Ok("on");
    if !mode.camera && !mode.shadows && !trace { return; }
    let support = DeviceSupport::default();
    let Some(render) = app.get_sub_app_mut(RenderApp) else { return; };
    render.insert_resource(support.clone()).insert_resource(SupportTrace(trace))
        .insert_resource(mode)
        .add_systems(Render, record_device_support.in_set(RenderSystems::Prepare));
    app.insert_resource(support);
    app.insert_resource(mode)
        .add_systems(Update, sync.after(sync_voxel_view));
    // Independent shadow HZB needs explicit subview downsampling and late
    // re-tests in this Bevy version. Ordinary play and the camera experiment
    // retain their existing paths; this bridge is only the named native QA.
    if mode.shadows && !mode.camera {
        shadow_bridge::register(app);
    }
}

fn record_device_support(device: Res<RenderDevice>, adapter: Res<RenderAdapterInfo>,
    preprocessing: Option<Res<GpuPreprocessingSupport>>, support: Res<DeviceSupport>,
    trace: Res<SupportTrace>, mode: Res<Mode>, mut recorded: Local<bool>) {
    if *recorded { return; }
    let Some(preprocessing) = preprocessing else { return; };
    let supported = preprocessing.max_supported_mode == GpuPreprocessingMode::Culling;
    support.0.store(supported,Ordering::Release);
    *recorded = true;
    if trace.0 {
        let limits = device.limits();
        eprintln!("gpu_culling_support adapter={:?} backend={:?} supported={} selective_tree_depth={} camera_requested={} shadows_requested={} features={:?} storage_textures={} storage_buffers={} compute_workgroup_storage={} immediate_bytes={}",
            adapter.name,adapter.backend,supported,mode.selective_depth,mode.camera,mode.shadows,
            device.features(),limits.max_storage_textures_per_shader_stage,
            limits.max_storage_buffers_per_shader_stage,limits.max_compute_workgroup_storage_size,
            limits.max_immediate_size);
        drop(crate::loading_trace::span("render.gpu_culling_support", || serde_json::json!({
            "adapter":adapter.name,"backend":format!("{:?}",adapter.backend),"supported":supported,
            "selective_tree_depth":mode.selective_depth,"camera_requested":mode.camera,
            "shadows_requested":mode.shadows,"features":format!("{:?}",device.features()),
            "max_storage_textures_per_shader_stage":limits.max_storage_textures_per_shader_stage,
            "max_storage_buffers_per_shader_stage":limits.max_storage_buffers_per_shader_stage,
            "max_compute_workgroup_storage_size":limits.max_compute_workgroup_storage_size,
            "max_immediate_size":limits.max_immediate_size})));
    }
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
    support: Res<DeviceSupport>,
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
        let enabled = support.0.load(Ordering::Acquire) && supported(&frame.map_id)
            && if camera { mode.camera && mode.selective_depth } else { mode.shadows };
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
            .insert_resource(DeviceSupport(Arc::new(AtomicBool::new(true))))
            .insert_resource(Mode {
                camera: true,
                shadows: true,
                selective_depth: true,
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
                selective_depth: true,
            },
            Mode {
                camera: false,
                shadows: true,
                selective_depth: true,
            },
        ] {
            let mut app = App::new();
            let mut frame = VisualWorldFrame::default();
            frame.map_id = "Route29".into();
            app.insert_resource(frame)
                .insert_resource(DeviceSupport(Arc::new(AtomicBool::new(true))))
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

    #[test]
    fn camera_culling_requires_real_device_support_and_selective_tree_depth() {
        for (hardware, selective_depth) in [(false,false),(false,true),(true,false),(true,true)] {
            let mut app = App::new();
            let mut frame = VisualWorldFrame::default();
            frame.map_id = "Route29".into();
            let support = DeviceSupport(Arc::new(AtomicBool::new(hardware)));
            app.insert_resource(frame).insert_resource(support.clone())
                .insert_resource(Mode { camera: true, shadows: false, selective_depth })
                .add_systems(Update,sync);
            let camera = app.world_mut().spawn(VoxelWorldCamera).id();
            let sun = app.world_mut().spawn(VoxelSun).id();
            app.update();
            assert_eq!(app.world().get::<OcclusionCulling>(camera).is_some(),hardware&&selective_depth);
            assert_eq!(app.world().get::<DepthPrepass>(camera).is_some(),hardware&&selective_depth);
            assert!(app.world().get::<OcclusionCulling>(sun).is_none());
            support.0.store(false,Ordering::Release);
            app.update();
            assert!(app.world().get::<OcclusionCulling>(camera).is_none());
            assert!(app.world().get::<DepthPrepass>(camera).is_none());
        }
    }
}
