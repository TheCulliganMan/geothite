//! One-shot visibility attribution for explicitly requested native reviews.

use std::collections::{BTreeMap, HashMap};

use bevy::app::{App, PostUpdate};
use bevy::camera::primitives::{Aabb, Frustum, Sphere};
use bevy::camera::visibility::{
    InheritedVisibility, NoCpuCulling, ViewVisibility, VisibilityClass, VisibilitySystems,
};
use bevy::camera::{Camera, Camera2d, Camera3d};
use bevy::ecs::archetype::ArchetypeId;
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::Local;
use bevy::ecs::world::{EntityRef, World};
use bevy::mesh::{Mesh2d, Mesh3d};
use bevy::transform::components::GlobalTransform;
use serde::Serialize;

pub(super) fn register(app: &mut App) {
    if std::env::var("CRYSTAL_VISIBILITY_CENSUS").as_deref() == Ok("on") {
        app.add_systems(PostUpdate, report.after(VisibilitySystems::CheckVisibility));
    }
}

#[derive(Default, Serialize)]
struct Counts {
    total: usize,
    inherited_visible: usize,
    view_visible: usize,
    bounded: usize,
    no_cpu_culling: usize,
}

fn report(world: &mut World, mut reported: Local<bool>) {
    if *reported {
        return;
    }
    let Some(status) = world.get_resource::<crystal_voxel_view::VoxelViewStatus>() else {
        return;
    };
    if !status.active
        || status.active_frames < 120
        || status.profiles_pending
        || status.terrain_pending
    {
        return;
    }
    *reported = true;

    let mut counts = BTreeMap::<&str, Counts>::new();
    let mut owners = HashMap::<ArchetypeId, usize>::new();
    // Match the engine visibility query's required components. Classification
    // includes NoCpuCulling entities so their exclusions remain visible here.
    let mut entities = world.query_filtered::<EntityRef, (
        With<InheritedVisibility>,
        With<ViewVisibility>,
        With<GlobalTransform>,
    )>();
    for entity in entities.iter(world) {
        let kind = if entity.contains::<Mesh3d>() {
            "mesh3d"
        } else if entity.contains::<Mesh2d>() {
            "mesh2d"
        } else if entity
            .get::<VisibilityClass>()
            .is_some_and(|class| class.iter().next().is_some())
        {
            "other_render_class"
        } else {
            "non_rendering"
        };
        let no_cpu = entity.contains::<NoCpuCulling>();
        let count = counts.entry(kind).or_default();
        count.total += 1;
        count.inherited_visible += usize::from(
            entity
                .get::<InheritedVisibility>()
                .is_some_and(|visibility| visibility.get()),
        );
        count.view_visible += usize::from(
            entity
                .get::<ViewVisibility>()
                .is_some_and(|visibility| visibility.get()),
        );
        count.bounded += usize::from(entity.contains::<Aabb>() || entity.contains::<Sphere>());
        count.no_cpu_culling += usize::from(no_cpu);
        if kind == "non_rendering" && !no_cpu {
            *owners.entry(entity.archetype().id()).or_default() += 1;
        }
    }
    let mut owners: Vec<_> = owners.into_iter().collect();
    owners.sort_unstable_by_key(|(_, count)| std::cmp::Reverse(*count));
    let owner_archetypes: Vec<_> = owners
        .into_iter()
        .take(12)
        .map(|(id, count)| {
            let components: Vec<_> = world.archetypes()[id]
                .iter_components()
                .filter_map(|component| world.components().get_info(component))
                .map(|info| info.name().to_string())
                .collect();
            serde_json::json!({"count":count, "components":components})
        })
        .collect();
    let mut cameras = world.query::<EntityRef>();
    let cameras: Vec<_> = cameras
        .iter(world)
        .filter_map(|entity| {
            let camera = entity.get::<Camera>()?;
            Some(serde_json::json!({
                "active":camera.is_active,
                "camera2d":entity.contains::<Camera2d>(),
                "camera3d":entity.contains::<Camera3d>(),
                "has_frustum":entity.contains::<Frustum>(),
                "no_cpu_culling":entity.contains::<NoCpuCulling>(),
                "layers":format!("{:?}",entity.get::<bevy::camera::visibility::RenderLayers>()),
            }))
        })
        .collect();
    eprintln!(
        "visibility_census {}",
        serde_json::json!({
            "map":world.get_resource::<crystal_render_api::VisualWorldFrame>().map(|frame|frame.map_id.as_ref()),
            "categories":counts,
            "non_rendering_cpu_archetypes":owner_archetypes,
            "cameras":cameras,
        })
    );
}
