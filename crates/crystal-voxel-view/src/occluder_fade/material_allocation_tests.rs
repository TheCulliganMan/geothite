//! Preserve regional fades while deferring dormant blend materials.

use bevy::app::App;
use bevy::asset::Assets;
use bevy::camera::visibility::Visibility;
use bevy::ecs::change_detection::Mut;
use bevy::mesh::Mesh;
use bevy::transform::components::Transform;

use crate::VoxelMaterial;
use crate::foliage::GroveFamily;
use crate::forest_instances::Library;
use crate::understory::Plot;

fn single_leaf_recipe() -> super::PreparedGroup {
    let library = Library::default();
    for coordinates in [[4, 8], [11, 3], [2, 19]] {
        let groups = library.prepare_with_ground(
            [-16.0, 16.0, -16.0, 16.0],
            0.0,
            16.0,
            coordinates,
            [false; 4],
            GroveFamily::JohtoPine,
            Some(Plot::isolated([-16.0, 16.0, -16.0, 16.0])),
        );
        if let Some(group) = groups.into_iter().find(|group| {
            group.opaque_leaf.is_none() && group.overlay.is_none() && group.leaves.len() == 1
        }) {
            return group;
        }
    }
    panic!("production source plots must include a single-leaf ground plant");
}

#[test]
fn regional_materials_preserve_first_fade_independence_reuse_and_retirement() {
    use bevy::asset::{AssetApp, AssetPlugin, Handle};
    use bevy::color::Color;
    use bevy::ecs::entity::Entity;
    use bevy::ecs::query::With;
    use bevy::math::{Quat, Vec3, Vec4};
    use bevy::mesh::Mesh3d;
    use bevy::pbr::MeshMaterial3d;
    use bevy::prelude::MinimalPlugins;
    use bevy::time::TimeUpdateStrategy;
    use bevy::transform::TransformPlugin;
    use bevy::transform::components::GlobalTransform;

    for runtime_recipe in [true, false] {
        let mut recipe = single_leaf_recipe();
        // Bank-conforming regional groves use the same ownership path without
        // the library's deferred-entity marker. Their blend can also be lazy.
        recipe.runtime_lazy_sorted = runtime_recipe;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin, AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_asset::<VoxelMaterial>()
            .init_resource::<crate::TerrainRevisionCache>()
            .init_resource::<crate::VoxelViewStatus>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_secs_f64(1.0 / 60.0),
            ));
        super::register(&mut app);
        let mut decoy = crate::solid_terrain_material();
        decoy.extension.cutaway.fade = Vec4::splat(0.17);
        app.world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .insert(Handle::<VoxelMaterial>::default().id(), decoy.clone())
            .expect("populate the unrelated default material slot");
        let mut original = crate::solid_terrain_material();
        original.base.base_color = Color::srgb(0.2, 0.6, 0.4);
        original.extension.city_night = Vec4::X;
        original.extension.city_windows[0] = Vec4::new(2.0, 6.0, 9.0, 4.0);
        let opaque = app
            .world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .add(original.clone());
        let root = app
            .world_mut()
            .spawn((
                Transform::from_xyz(80.0, 7.0, -20.0)
                    .with_rotation(Quat::from_rotation_y(0.7))
                    .with_scale(Vec3::new(1.2, 1.5, 0.8)),
                Visibility::Inherited,
            ))
            .id();
        app.world_mut()
            .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                world.resource_scope(|world, mut materials: Mut<Assets<VoxelMaterial>>| {
                    super::spawn_region_groups(
                        &mut world.commands(),
                        &mut meshes,
                        &mut materials,
                        root,
                        vec![recipe.clone(), recipe],
                        opaque.clone(),
                    );
                });
            });
        app.world_mut().flush();
        for _ in 0..3 {
            app.update();
        }
        let mut groups: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<super::OccluderGroup>>()
            .iter(app.world())
            .collect();
        groups.sort_unstable();
        assert_eq!(groups.len(), 2);
        assert_eq!(
            app.world().resource::<Assets<VoxelMaterial>>().len(),
            2,
            "two untouched regional plants must not allocate dormant blends"
        );
        let leaves: Vec<_> = groups
            .iter()
            .map(|&group| {
                assert!(
                    !app.world()
                        .get::<super::OccluderGroup>(group)
                        .expect("regional fade owner")
                        .transparent_ready
                );
                let children = app
                    .world()
                    .get::<bevy::ecs::hierarchy::Children>(group)
                    .expect("single-leaf regional owner");
                assert_eq!(children.len(), 1);
                let child = children[0];
                assert_eq!(
                    app.world()
                        .get::<MeshMaterial3d<VoxelMaterial>>(child)
                        .expect("opaque leaf material")
                        .0,
                    opaque
                );
                (
                    child,
                    app.world()
                        .get::<Mesh3d>(child)
                        .expect("original mesh")
                        .0
                        .clone(),
                    *app.world()
                        .get::<GlobalTransform>(child)
                        .expect("propagated pose"),
                )
            })
            .collect();
        let mut blend_ids = Vec::new();
        for (index, &group) in groups.iter().enumerate() {
            app.world_mut()
                .get_mut::<super::OccluderGroup>(group)
                .expect("fade owner")
                .fade
                .opacity = 0.5 - index as f32 * 0.1;
            app.update();
            let state = app
                .world()
                .get::<super::OccluderGroup>(group)
                .expect("fading owner");
            assert!(state.transparent_ready);
            blend_ids.push(state.transparent.id());
            let materials = app.world().resource::<Assets<VoxelMaterial>>();
            let blend = materials
                .get(&state.transparent)
                .expect("first-frame blend");
            assert_eq!(blend.base.base_color, original.base.base_color);
            assert_eq!(blend.extension.city_night, original.extension.city_night);
            assert_eq!(
                blend.extension.city_windows,
                original.extension.city_windows
            );
            assert_eq!(blend.extension.cutaway.fade.x, 1.0 - state.fade.opacity);
            assert_eq!(
                materials
                    .get(&opaque)
                    .expect("shared opaque")
                    .extension
                    .cutaway,
                original.extension.cutaway
            );
            assert_eq!(
                materials
                    .get(Handle::<VoxelMaterial>::default().id())
                    .expect("unrelated default material")
                    .extension
                    .cutaway,
                decoy.extension.cutaway
            );
            assert_eq!(materials.len(), 3 + index);
            for (child, mesh, pose) in &leaves {
                assert_eq!(
                    &app.world().get::<Mesh3d>(*child).expect("retained mesh").0,
                    mesh
                );
                assert_eq!(app.world().get::<GlobalTransform>(*child), Some(pose));
            }
            if index == 0 {
                assert!(
                    !app.world()
                        .get::<super::OccluderGroup>(groups[1])
                        .expect("independent clear owner")
                        .transparent_ready
                );
                assert_eq!(
                    app.world()
                        .get::<MeshMaterial3d<VoxelMaterial>>(leaves[1].0)
                        .expect("unfaded neighbor")
                        .0,
                    opaque
                );
            }
        }
        assert_ne!(blend_ids[0], blend_ids[1], "opacity belongs to each plant");
        for _ in 0..180 {
            app.update();
        }
        for (index, &group) in groups.iter().enumerate() {
            assert_eq!(
                app.world()
                    .get::<super::OccluderGroup>(group)
                    .expect("restored owner")
                    .fade
                    .opacity,
                1.0
            );
            assert_eq!(
                app.world()
                    .get::<MeshMaterial3d<VoxelMaterial>>(leaves[index].0)
                    .expect("restored opaque leaf")
                    .0,
                opaque
            );
            app.world_mut()
                .get_mut::<super::OccluderGroup>(group)
                .expect("reused owner")
                .fade
                .opacity = 0.5;
        }
        app.update();
        for (index, &group) in groups.iter().enumerate() {
            assert_eq!(
                app.world()
                    .get::<super::OccluderGroup>(group)
                    .expect("second fade")
                    .transparent
                    .id(),
                blend_ids[index]
            );
        }
        assert_eq!(app.world().resource::<Assets<VoxelMaterial>>().len(), 4);
        app.world_mut().entity_mut(root).despawn();
        for _ in 0..3 {
            app.update();
        }
        assert_eq!(
            app.world().resource::<Assets<VoxelMaterial>>().len(),
            2,
            "retired private blends must not remain pinned"
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod gpu_tests;

#[test]
fn regional_materials_do_not_write_the_default_slot_on_a_zero_time_block() {
    use bevy::asset::{AssetApp, AssetPlugin, Handle};
    use bevy::camera::Camera;
    use bevy::math::Vec4;
    use bevy::prelude::MinimalPlugins;
    use bevy::time::TimeUpdateStrategy;
    use bevy::transform::TransformPlugin;
    use std::time::Duration;

    let mut recipe = single_leaf_recipe();
    // A camera-plane blocker can be selected before real time has advanced.
    recipe.camera_wall = Some([1.0, 0.0, 1_000_000.0]);
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<VoxelMaterial>()
        .init_resource::<crate::TerrainRevisionCache>()
        .insert_resource(crate::VoxelViewStatus {
            active: true,
            ..Default::default()
        })
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    super::register(&mut app);
    app.world_mut().spawn((
        Camera::default(),
        crate::VoxelWorldCamera,
        Transform::IDENTITY,
    ));
    let root = app
        .world_mut()
        .spawn((Transform::IDENTITY, Visibility::Inherited))
        .id();
    let mut decoy = crate::solid_terrain_material();
    decoy.extension.cutaway.fade = Vec4::splat(0.17);
    app.world_mut()
        .resource_mut::<Assets<VoxelMaterial>>()
        .insert(Handle::<VoxelMaterial>::default().id(), decoy.clone())
        .expect("unrelated default material");
    let opaque = app
        .world_mut()
        .resource_mut::<Assets<VoxelMaterial>>()
        .add(crate::solid_terrain_material());
    app.world_mut()
        .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(|world, mut materials: Mut<Assets<VoxelMaterial>>| {
                super::spawn_region_groups(
                    &mut world.commands(),
                    &mut meshes,
                    &mut materials,
                    root,
                    vec![recipe],
                    opaque,
                );
            });
        });
    app.world_mut().flush();
    app.update();
    let state = app
        .world_mut()
        .query::<&super::OccluderGroup>()
        .single(app.world())
        .expect("selected camera-plane owner");
    assert!(
        state.blocked,
        "the fixture must exercise an actual blocked update"
    );
    assert_eq!(state.fade.opacity, 1.0);
    assert!(
        !state.transparent_ready,
        "zero time must not allocate a blend"
    );
    assert_eq!(app.world().resource::<Assets<VoxelMaterial>>().len(), 2);
    assert_eq!(
        app.world()
            .resource::<Assets<VoxelMaterial>>()
            .get(Handle::<VoxelMaterial>::default().id())
            .expect("unrelated default slot")
            .extension
            .cutaway,
        decoy.extension.cutaway
    );
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 60.0,
    )));
    app.update();
    let state = app
        .world_mut()
        .query::<&super::OccluderGroup>()
        .single(app.world())
        .expect("first timed fade");
    assert!(state.transparent_ready);
    assert!(state.fade.opacity < 1.0);
    assert_eq!(app.world().resource::<Assets<VoxelMaterial>>().len(), 3);
    assert_eq!(
        app.world()
            .resource::<Assets<VoxelMaterial>>()
            .get(Handle::<VoxelMaterial>::default().id())
            .expect("preserved default slot")
            .extension
            .cutaway,
        decoy.extension.cutaway
    );
}
