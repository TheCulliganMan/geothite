//! Shared distant pine crowns affect rendering only, never collision or reveal.
use super::{FadeMeshes, OccluderGroup, PreparedGroup, PreparedLeaf, SharedLeaf};
use crate::{TerrainRevisionCache, VoxelMaterial, VoxelWorldCamera, mesh::SurfaceMeshData};
use bevy::camera::primitives::Aabb;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::{
    App, Assets, Camera, ChildOf, Component, GlobalTransform, Handle, IntoScheduleConfigs, Mat4,
    Mesh, Mesh3d, PostUpdate, Projection, Query, Res, ResMut, Vec3, ViewVisibility, With,
};
use std::sync::{Arc, Mutex};

// Fraction of viewport height, independent of render resolution. Hysteresis
// prevents repeated mesh changes when walking along the detail boundary.
const ENTER_DISTANT: f32 = 0.25;
const RETURN_FULL: f32 = 0.30;

#[derive(Component)]
struct TreeDetail {
    full: Handle<Mesh>,
    distant: Arc<Mutex<SharedLeaf>>,
    handle: Option<Handle<Mesh>>,
    simplified: bool,
}

pub(super) fn register(app: &mut App) {
    app.add_systems(
        PostUpdate,
        sync.after(FadeMeshes)
            .after(bevy::transform::TransformSystems::Propagate)
            .after(bevy::camera::CameraUpdateSystems)
            .before(bevy::asset::AssetEventSystems)
            .before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate),
    );
}

/// Attach a shared render alternative without replacing any collision surface.
pub(crate) fn link(full: &[PreparedGroup], distant: &[PreparedGroup]) {
    assert_eq!(full.len(), 1, "one pine prototype owns one reveal group");
    assert_eq!(
        distant.len(),
        1,
        "one distant crown matches one pine prototype"
    );
    let full = full[0]
        .opaque_leaf
        .as_ref()
        .and_then(|leaf| leaf.shared.as_ref())
        .expect("pine opaque geometry is shared");
    let distant = distant[0]
        .opaque_leaf
        .as_ref()
        .and_then(|leaf| leaf.shared.as_ref())
        .expect("distant pine opaque geometry is shared");
    full.lock()
        .expect("pine geometry lock is not poisoned")
        .distant = Some(distant.clone());
}

pub(super) fn insert(entity: &mut EntityCommands, leaf: &PreparedLeaf, full: Handle<Mesh>) {
    if !enabled() {
        return;
    }
    let distant = leaf.shared.as_ref().and_then(|shared| {
        shared
            .lock()
            .expect("pine geometry lock is not poisoned")
            .distant
            .clone()
    });
    if let Some(distant) = distant {
        entity.insert(TreeDetail {
            full,
            distant,
            handle: None,
            simplified: false,
        });
    }
}

/// Fit once at prototype construction. Both levels keep identical culling bounds
/// and placement scale, even if the outermost needle differs between recipes.
pub(crate) fn fit_bounds(mesh: &mut SurfaceMeshData, (low, high): (Vec3, Vec3)) {
    let (from_low, from_high) = mesh.positions.iter().map(|p| Vec3::from_array(*p)).fold(
        (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
        |(low, high), p| (low.min(p), high.max(p)),
    );
    let scale = (high - low) / (from_high - from_low);
    assert!(
        scale.is_finite() && scale.min_element() > 0.,
        "pine bounds have volume"
    );
    for (position, normal) in mesh.positions.iter_mut().zip(&mut mesh.normals) {
        *position = (low + (Vec3::from_array(*position) - from_low) * scale).to_array();
        *normal = (Vec3::from_array(*normal) / scale).normalize().to_array();
    }
}

fn distant(simplified: bool, projected_height: f32, protected: bool) -> bool {
    !protected
        && projected_height.is_finite()
        && projected_height >= 0.
        && projected_height
            < if simplified {
                RETURN_FULL
            } else {
                ENTER_DISTANT
            }
}

fn projected_height(bounds: &Aabb, transform: &GlobalTransform, clip: Mat4) -> Option<f32> {
    let center: Vec3 = bounds.center.into();
    let half: Vec3 = bounds.half_extents.into();
    let matrix = clip * transform.to_matrix();
    let mut low = f32::INFINITY;
    let mut high = f32::NEG_INFINITY;
    for corner in 0..8 {
        let point = center
            + half
                * Vec3::new(
                    if corner & 1 == 0 { -1. } else { 1. },
                    if corner & 2 == 0 { -1. } else { 1. },
                    if corner & 4 == 0 { -1. } else { 1. },
                );
        let projected = matrix * point.extend(1.);
        if !projected.is_finite() || projected.w <= 0.001 {
            return None;
        }
        let y = projected.y / projected.w;
        low = low.min(y);
        high = high.max(y);
    }
    Some((high - low) * 0.5)
}

fn sync(
    cameras: Query<(&Camera, &Projection, &GlobalTransform), With<VoxelWorldCamera>>,
    cache: Res<TerrainRevisionCache>,
    materials: Res<Assets<VoxelMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    groups: Query<&OccluderGroup>,
    mut trees: Query<(
        &mut Mesh3d,
        &mut TreeDetail,
        &Aabb,
        &GlobalTransform,
        &ChildOf,
        &ViewVisibility,
    )>,
) {
    if !enabled() {
        return;
    }
    let _trace = crate::loading_trace::span("frame.tree_lod_sync", || serde_json::json!({}));
    let Ok((camera, projection, camera_transform)) = cameras.single() else {
        return;
    };
    if !camera.is_active {
        return;
    }
    let clip = projection.get_clip_from_view() * camera_transform.to_matrix().inverse();
    let player = cache
        .solid_material
        .as_ref()
        .and_then(|handle| materials.get(handle))
        .map(|material| material.extension.cutaway.bottom_radius)
        .filter(|player| player.w > 0.);
    let mut changed = 0;
    let mut simplified = 0;
    let mut evaluated = 0;
    for (mut mesh, mut detail, bounds, transform, parent, visible) in &mut trees {
        // Before visibility propagation this is the previous completed cull.
        // Hidden trees keep the full mesh ready for their first visible frame;
        // the expensive screen-size test only follows a camera/shadow view.
        if !visible.get() {
            if detail.simplified {
                mesh.0 = detail.full.clone();
                detail.simplified = false;
                changed += 1;
            }
            continue;
        }
        evaluated += 1;
        let protected = groups
            .get(parent.parent())
            .is_ok_and(|group| group.blocked || group.blending || group.fade.opacity < 1.);
        let near_player = player.is_some_and(|player| {
            let root = transform.translation();
            let delta = root - player.truncate();
            delta.length_squared() < (player.w * (6. / 0.65)).powi(2)
        });
        let height = projected_height(bounds, transform, clip).unwrap_or(f32::INFINITY);
        let use_distant = distant(detail.simplified, height, protected || near_player);
        simplified += usize::from(use_distant);
        if detail.simplified == use_distant {
            continue;
        }
        let handle = if use_distant {
            if detail.handle.is_none() {
                let mut shared = detail
                    .distant
                    .lock()
                    .expect("pine geometry lock is not poisoned");
                let handle = if let Some(handle) = &shared.uploaded {
                    handle.clone()
                } else {
                    let mesh = shared.mesh.take().expect("distant mesh uploads once");
                    let handle = meshes.add(mesh);
                    shared.uploaded = Some(handle.clone());
                    handle
                };
                drop(shared);
                detail.handle = Some(handle);
            }
            detail
                .handle
                .as_ref()
                .expect("distant mesh was uploaded")
                .clone()
        } else {
            detail.full.clone()
        };
        mesh.0 = handle;
        detail.simplified = use_distant;
        changed += 1;
    }
    let _trace = crate::loading_trace::span("frame.tree_lod", || {
        serde_json::json!({
            "trees": trees.iter().count(), "evaluated": evaluated,
            "distant": simplified, "changed": changed
        })
    });
}

fn enabled() -> bool {
    if cfg!(feature = "scenery-full-detail") {
        return false;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ENABLED.get_or_init(|| std::env::var("CRYSTAL_TREE_LOD").as_deref() != Ok("off"))
    }
    #[cfg(target_arch = "wasm32")]
    {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::PerspectiveProjection;
    #[cfg(not(feature = "scenery-full-detail"))]
    use bevy::prelude::{Camera3d, MinimalPlugins, Mut, Transform, Vec4, Visibility};

    #[test]
    fn detail_boundary_has_hysteresis_and_reveal_always_restores_full_geometry() {
        assert!(distant(false, 0.24, false));
        assert!(!distant(false, 0.27, false));
        assert!(distant(true, 0.27, false));
        assert!(!distant(true, 0.31, false));
        for height in [0.01, 0.24, 0.27, 0.31, f32::NAN, f32::INFINITY] {
            assert!(!distant(true, height, true));
        }
        assert!(!distant(true, f32::NAN, false));
    }

    #[test]
    fn zoom_and_near_plane_crossings_keep_large_trees_detailed() {
        let bounds = Aabb::from_min_max(Vec3::splat(-1.), Vec3::splat(1.));
        let clip = Projection::Perspective(PerspectiveProjection {
            fov: 1.,
            aspect_ratio: 1.6,
            near: 0.1,
            far: 100.,
            ..Default::default()
        })
        .get_clip_from_view();
        let near = GlobalTransform::from_translation(Vec3::new(0., 0., -3.));
        let far = GlobalTransform::from_translation(Vec3::new(0., 0., -30.));
        assert!(!distant(
            false,
            projected_height(&bounds, &near, clip).expect("in front"),
            false
        ));
        assert!(distant(
            false,
            projected_height(&bounds, &far, clip).expect("in front"),
            false
        ));
        let crossing = GlobalTransform::IDENTITY;
        assert!(projected_height(&bounds, &crossing, clip).is_none());
    }

    #[test]
    #[cfg(not(feature = "scenery-full-detail"))]
    fn shared_pines_swap_only_render_meshes_and_reuse_the_distant_upload() {
        use super::super::spawn_groups;
        use crate::foliage::GroveFamily;

        let library = crate::forest_instances::Library::default();
        let groups = library.prepare(
            [0., 32., 0., 32.],
            0.,
            16.,
            [4, 8],
            [true; 4],
            GroveFamily::JohtoPine,
        );
        let surface = groups[0].surface.clone();
        let full_shared = groups[0]
            .opaque_leaf
            .as_ref()
            .expect("opaque pine")
            .shared
            .as_ref()
            .expect("shared pine")
            .clone();
        let full_count = full_shared
            .lock()
            .expect("geometry lock")
            .mesh
            .as_ref()
            .expect("CPU mesh")
            .indices()
            .expect("indexed mesh")
            .len();
        let distant = full_shared
            .lock()
            .expect("geometry lock")
            .distant
            .clone()
            .expect("LOD");
        let distant_count = distant
            .lock()
            .expect("geometry lock")
            .mesh
            .as_ref()
            .expect("CPU mesh")
            .indices()
            .expect("indexed mesh")
            .len();
        assert!(
            distant_count < full_count * 3 / 4,
            "a distant pine saves at least 25% of triangles"
        );

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::transform::TransformPlugin))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<VoxelMaterial>>()
            .init_resource::<TerrainRevisionCache>();
        register(&mut app);
        let camera = app
            .world_mut()
            .spawn((
                Camera3d::default(),
                Transform::from_xyz(16., 100., 3000.)
                    .looking_at(Vec3::new(16., 100., 16.), Vec3::Y),
                VoxelWorldCamera,
            ))
            .id();
        let root = app
            .world_mut()
            .spawn((Transform::default(), Visibility::Inherited))
            .id();
        let material = app
            .world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .add(crate::solid_terrain_material());
        for _ in 0..2 {
            app.world_mut()
                .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                    world.resource_scope(|world, mut materials: Mut<Assets<VoxelMaterial>>| {
                        spawn_groups(
                            &mut world.commands(),
                            &mut meshes,
                            &mut materials,
                            root,
                            groups.clone(),
                            material.clone(),
                            None,
                        );
                    });
                });
        }
        app.world_mut().flush();
        {
            let world = app.world_mut();
            let mut visibility = world.query_filtered::<&mut ViewVisibility, With<TreeDetail>>();
            for mut visible in visibility.iter_mut(world) {
                *visible = ViewVisibility::VISIBLE;
            }
        }
        app.update();
        let mut trees = app.world_mut().query::<(&TreeDetail, &Mesh3d, &ChildOf)>();
        let initial: Vec<_> = trees
            .iter(app.world())
            .map(|(detail, mesh, parent)| {
                assert!(detail.simplified);
                let group = app
                    .world()
                    .get::<OccluderGroup>(parent.parent())
                    .expect("owner");
                assert!(
                    Arc::ptr_eq(&surface, &group.surface),
                    "LOD never replaces collision"
                );
                (mesh.0.clone(), parent.parent())
            })
            .collect();
        assert_eq!(initial.len(), 2);
        assert_eq!(
            initial[0].0, initial[1].0,
            "both instances reuse one LOD upload"
        );
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 2);

        {
            let world = app.world_mut();
            let mut visibility = world.query_filtered::<&mut ViewVisibility, With<TreeDetail>>();
            for mut visible in visibility.iter_mut(world) {
                *visible = ViewVisibility::HIDDEN;
            }
        }
        app.update();
        for (detail, mesh, _) in trees.iter(app.world()) {
            assert!(
                !detail.simplified,
                "hidden trees keep full detail ready for reentry"
            );
            assert_eq!(mesh.0, detail.full);
        }
        {
            let world = app.world_mut();
            let mut visibility = world.query_filtered::<&mut ViewVisibility, With<TreeDetail>>();
            for mut visible in visibility.iter_mut(world) {
                *visible = ViewVisibility::VISIBLE;
            }
        }
        app.update();

        app.world_mut()
            .get_mut::<OccluderGroup>(initial[0].1)
            .expect("owner")
            .blocked = true;
        app.update();
        for (detail, mesh, parent) in trees.iter(app.world()) {
            if parent.parent() == initial[0].1 {
                assert!(
                    !detail.simplified,
                    "player reveal restores full detail immediately"
                );
                assert_eq!(mesh.0, detail.full);
            } else {
                assert!(detail.simplified);
            }
        }
        app.world_mut()
            .get_mut::<OccluderGroup>(initial[0].1)
            .expect("owner")
            .blocked = false;
        app.world_mut()
            .get_mut::<Transform>(camera)
            .expect("camera")
            .translation
            .z = 150.;
        app.update();
        for (detail, mesh, _) in trees.iter(app.world()) {
            assert!(
                !detail.simplified,
                "approaching the tree restores the full mesh"
            );
            assert_eq!(mesh.0, detail.full);
        }
        app.world_mut()
            .get_mut::<Transform>(camera)
            .expect("camera")
            .translation
            .z = 3000.;
        app.update();
        for (detail, mesh, _) in trees.iter(app.world()) {
            assert!(detail.simplified);
            assert_eq!(
                mesh.0, initial[0].0,
                "returning to distance reuses the same upload"
            );
        }
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 2);

        app.world_mut()
            .resource_mut::<TerrainRevisionCache>()
            .solid_material = Some(material.clone());
        app.world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .get_mut(&material)
            .expect("pine material")
            .extension
            .cutaway
            .bottom_radius = Vec4::new(16., 0., 16., 10.4);
        app.update();
        for (detail, mesh, _) in trees.iter(app.world()) {
            assert!(
                !detail.simplified,
                "nearby small trees retain full detail even when zoomed out"
            );
            assert_eq!(mesh.0, detail.full);
        }
        app.world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .get_mut(&material)
            .expect("pine material")
            .extension
            .cutaway
            .bottom_radius
            .y = 1000.;
        app.update();
        for (detail, _, _) in trees.iter(app.world()) {
            assert!(
                detail.simplified,
                "a rooftop does not protect distant trees far below it"
            );
        }
    }
}
