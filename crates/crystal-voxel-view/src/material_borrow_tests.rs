//! Retained scene checks must preserve asset change detection until a write.

use bevy::app::{App, Update};
use bevy::asset::{AssetApp, AssetPlugin, Assets, Handle};
use bevy::camera::visibility::Visibility;
use bevy::ecs::change_detection::DetectChanges;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, Query, Res, ResMut};
use bevy::image::Image;
use bevy::math::{Mat4, UVec2, Vec2};
use bevy::mesh::Mesh;
use bevy::prelude::MinimalPlugins;
use bevy::transform::components::Transform;
use crystal_render_api::{VisualActor, VisualActorId, VisualWorldClock, VisualWorldFrame};

use crate::{
    ActorIdCache, TerrainBuildQueue, TerrainCacheKey, TerrainRevisionCache, TerrainSyncState,
    VoxelMaterial, VoxelTerrainFilter, VoxelViewStatus,
};

#[derive(Resource)]
struct TestActor(VisualActor);

#[derive(Resource, Default)]
struct Changes(Vec<bool>);

#[allow(clippy::too_many_arguments)]
fn synchronize(
    frame: Res<VisualWorldFrame>,
    actor: Res<TestActor>,
    mut commands: Commands,
    mut cache: ResMut<TerrainRevisionCache>,
    mut builds: ResMut<TerrainBuildQueue>,
    mut actors: ResMut<ActorIdCache>,
    mut status: ResMut<VoxelViewStatus>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut entities: Query<(&mut Visibility, &mut Transform), VoxelTerrainFilter>,
) {
    assert_eq!(
        crate::sync_terrain(
            &frame,
            &crate::live_profiles::LiveProfiles::default(),
            &mut commands,
            &mut cache,
            &mut builds,
            &mut images,
            &mut meshes,
            &mut materials,
            &mut entities
        ),
        Ok(TerrainSyncState::Ready)
    );
    assert!(!crate::grove_stream::sync(
        &frame,
        Mat4::IDENTITY,
        None,
        &mut cache,
        &mut commands,
        &mut meshes,
        &mut materials
    ));
    crate::actor_material(&actor.0, &mut actors, &mut materials);
    status.active_frames += 1;
}

fn observe(materials: Res<Assets<VoxelMaterial>>, mut changes: ResMut<Changes>) {
    changes.0.push(materials.is_changed());
}

#[test]
fn retained_terrain_and_actor_checks_preserve_real_material_change_ticks() {
    let frame = VisualWorldFrame {
        active: true,
        map_id: "GoldenrodCity".into(),
        grid_size: UVec2::splat(1),
        tile_size: Vec2::splat(32.0),
        viewport_size: Vec2::splat(64.0),
        ..Default::default()
    };
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .init_asset::<Mesh>()
        .init_asset::<VoxelMaterial>()
        .insert_resource(frame.clone())
        .insert_resource(TerrainRevisionCache {
            key: Some(TerrainCacheKey::from_frame(&frame)),
            built_footing_heights: vec![0.0],
            built_frame: Some(frame),
            ..Default::default()
        })
        .insert_resource(TestActor(VisualActor {
            id: VisualActorId::Player,
            source_id: "player".into(),
            model_palette: None,
            texture: Handle::default(),
            center: Vec2::ZERO,
            size: Vec2::splat(16.0),
            flip_x: false,
            above_priority: false,
            facing: None,
        }))
        .insert_resource(VisualWorldClock {
            minute_of_day: Some(22 * 60),
            second_of_minute: 0,
        })
        .insert_resource(VoxelViewStatus {
            active: true,
            ..Default::default()
        })
        .init_resource::<TerrainBuildQueue>()
        .init_resource::<ActorIdCache>()
        .init_resource::<Changes>()
        .add_systems(
            Update,
            (synchronize, crate::city_lights::sync, observe).chain(),
        );
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(
        app.world().resource::<Changes>().0.last(),
        Some(&false),
        "cached terrain and actor checks must not invalidate stable lighting"
    );
    let original = app
        .world()
        .resource::<ActorIdCache>()
        .textures
        .values()
        .next()
        .expect("cached actor material")
        .material
        .clone();
    let next_texture = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default());
    app.world_mut().resource_mut::<TestActor>().0.texture = next_texture;
    app.update();
    assert_eq!(
        app.world().resource::<Changes>().0.last(),
        Some(&true),
        "a new actor material must still publish an asset change"
    );
    for _ in 0..4 {
        app.update();
    }
    assert_eq!(app.world().resource::<Changes>().0.last(), Some(&false));
    app.world_mut()
        .resource_mut::<Assets<VoxelMaterial>>()
        .get_mut(&original)
        .expect("retained original material")
        .extension
        .city_night
        .x = 0.25;
    app.update();
    assert_eq!(
        app.world().resource::<Changes>().0.last(),
        Some(&true),
        "external material edits must still invalidate lighting"
    );
    assert_eq!(
        app.world()
            .resource::<Assets<VoxelMaterial>>()
            .get(&original)
            .expect("repaired original material")
            .extension
            .city_night
            .x,
        1.0
    );
}
