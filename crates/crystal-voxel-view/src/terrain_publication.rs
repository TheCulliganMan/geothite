//! Keep foreground replacements hidden until their staged GPU uploads finish.

use bevy::asset::{AssetId, Assets};
use bevy::prelude::{Commands, Image, Mesh, Visibility};
use crystal_render_api::VisualWorldFrame;

use crate::{TerrainCacheKey, TerrainRevisionCache, destination_preload, terrain_disposal};

type Compatibility = Option<(TerrainCacheKey, AssetId<Image>, bool)>;

pub(super) struct Pending {
    pub(super) cache: TerrainRevisionCache,
    compatibility: Compatibility,
    settled_frames: u8,
}

impl Pending {
    pub(super) fn new(live: &TerrainRevisionCache) -> Self {
        let mut cache = TerrainRevisionCache {
            forest_library: live.forest_library.clone(),
            painted_stone: live.painted_stone.clone(),
            painted_bedrock: live.painted_bedrock.clone(),
            ..Default::default()
        };
        cache.uploads.require_gpu(live.uploads.gpu_required());
        Self {
            cache,
            compatibility: None,
            settled_frames: 0,
        }
    }

    pub(super) fn matches(&mut self, frame: &VisualWorldFrame, revision: u64) -> bool {
        self.cache
            .key
            .as_ref()
            .is_some_and(|key| key.profiles_revision == revision)
            && self.cache.built_frame.as_ref().is_some_and(|source| {
                destination_preload::cached_compatible(source, frame, &mut self.compatibility)
            })
    }
}

pub(super) fn discard(pending: &mut Option<Pending>, commands: &mut Commands) {
    if let Some(pending) = pending.take() {
        for root in destination_preload::roots(&pending.cache) {
            commands.entity(root).despawn();
        }
        terrain_disposal::retire(pending);
    }
}

/// Publish a whole resident scene atomically, preserving live roots and support
/// throughout CPU insertion, GPU preparation and three presentation settle frames.
pub(super) fn advance(
    pending: &mut Option<Pending>,
    live: &mut TerrainRevisionCache,
    meshes: &mut Assets<Mesh>,
    commands: &mut Commands,
) -> bool {
    let Some(replacement) = pending.as_mut() else {
        return false;
    };
    replacement.cache.uploads.advance(meshes);
    destination_preload::hide(&replacement.cache, commands);
    if !replacement.cache.uploads.ready() {
        replacement.settled_frames = 0;
        return false;
    }
    replacement.settled_frames += 1;
    if replacement.settled_frames < 3 {
        return false;
    }
    let replacement = pending.take().expect("resident replacement is present");
    let _publish = crate::loading_trace::span("world.foreground_publish", || {
        serde_json::json!({
            "map": replacement.cache.built_frame.as_ref().map(|frame| frame.map_id.as_ref()),
            "origin": replacement.cache.built_frame.as_ref().map(|frame| frame.grid_origin.to_array()),
            "retired_map": live.built_frame.as_ref().map(|frame| frame.map_id.as_ref()),
        })
    });
    let retired = std::mem::replace(live, replacement.cache);
    for root in destination_preload::roots(&retired) {
        commands.entity(root).despawn();
    }
    for root in destination_preload::roots(live) {
        commands.entity(root).insert(Visibility::Visible);
        #[cfg(not(target_arch = "wasm32"))]
        commands
            .entity(root)
            .remove::<crate::pipeline_preload::HiddenDestination>();
    }
    terrain_disposal::retire(retired);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::RenderAssetUsages;
    use bevy::prelude::*;
    use bevy::render::render_resource::PrimitiveTopology;

    #[derive(Resource)]
    struct Publication(Option<Pending>);

    #[test]
    fn a_source_or_profile_change_rejects_a_hidden_replacement() {
        let (source, _) = crate::world_stream::test_frames::connected_frames(IVec2::new(40, 0));
        let live = TerrainRevisionCache::default();
        let mut replacement = Pending::new(&live);
        let mut key = TerrainCacheKey::from_frame(&source);
        key.profiles_revision = 2;
        replacement.cache.key = Some(key);
        replacement.cache.built_frame = Some(source.clone());
        assert!(replacement.matches(&source, 2));
        assert!(!replacement.matches(&source, 3));
        let mut changed = source.clone();
        changed.terrain_revision += 1;
        changed.tiles[0].source.tile_index += 1;
        assert!(!replacement.matches(&changed, 2));
        let mut distant = source;
        distant.grid_origin.x += 30;
        assert!(!replacement.matches(&distant, 2));
    }

    #[test]
    fn cancelled_upload_despawns_hidden_roots_and_preserves_the_live_scene() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let old_root = app.world_mut().spawn(Visibility::Visible).id();
        let new_root = app.world_mut().spawn(Visibility::Hidden).id();
        let live = TerrainRevisionCache {
            solid_entity: Some(old_root),
            footing_heights: vec![7.],
            ..Default::default()
        };
        let mut replacement = Pending::new(&live);
        replacement.cache.solid_entity = Some(new_root);
        app.insert_resource(live)
            .insert_resource(Publication(Some(replacement)))
            .add_systems(
                Update,
                |mut publication: ResMut<Publication>, mut commands: Commands| {
                    discard(&mut publication.0, &mut commands);
                },
            );
        app.update();
        assert!(app.world().resource::<Publication>().0.is_none());
        assert!(app.world().get_entity(new_root).is_err());
        assert_eq!(
            app.world().get::<Visibility>(old_root),
            Some(&Visibility::Visible)
        );
        assert_eq!(
            app.world()
                .resource::<TerrainRevisionCache>()
                .footing_heights,
            [7.]
        );
    }

    #[test]
    fn recovery_keeps_live_geometry_and_support_until_all_batches_settle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let old_root = app.world_mut().spawn(Visibility::Visible).id();
        let new_root = app.world_mut().spawn(Visibility::Hidden).id();
        let meshes = Assets::<Mesh>::default();
        let mut large = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        large.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.; 3]; 360_000]);
        let small = Mesh::from(Cuboid::default());
        let live = TerrainRevisionCache {
            solid_entity: Some(old_root),
            footing_heights: vec![7.],
            ..Default::default()
        };
        let mut replacement = Pending::new(&live);
        replacement.cache.solid_entity = Some(new_root);
        replacement.cache.footing_heights = vec![19.];
        let first = replacement.cache.uploads.enqueue(&meshes, large);
        let last = replacement.cache.uploads.enqueue(&meshes, small);
        replacement.cache.uploads.seal();
        // Both the queue and the live scene own different roots/support.
        app.insert_resource(live);
        app.insert_resource(meshes)
            .insert_resource(Publication(Some(replacement)))
            .add_systems(
                Update,
                |mut publication: ResMut<Publication>,
                 mut live: ResMut<TerrainRevisionCache>,
                 mut meshes: ResMut<Assets<Mesh>>,
                 mut commands: Commands| {
                    advance(&mut publication.0, &mut live, &mut meshes, &mut commands);
                },
            );
        app.update();
        assert!(app.world().resource::<Assets<Mesh>>().contains(first.id()));
        assert!(!app.world().resource::<Assets<Mesh>>().contains(last.id()));
        for _ in 0..2 {
            assert_eq!(
                app.world().resource::<TerrainRevisionCache>().solid_entity,
                Some(old_root)
            );
            assert_eq!(
                app.world()
                    .resource::<TerrainRevisionCache>()
                    .footing_heights,
                [7.]
            );
            assert_eq!(
                app.world().get::<Visibility>(old_root),
                Some(&Visibility::Visible)
            );
            assert_eq!(
                app.world().get::<Visibility>(new_root),
                Some(&Visibility::Hidden)
            );
            app.update();
        }
        assert!(app.world().resource::<Assets<Mesh>>().contains(last.id()));
        app.update();
        assert!(app.world().resource::<Publication>().0.is_none());
        assert_eq!(
            app.world().resource::<TerrainRevisionCache>().solid_entity,
            Some(new_root)
        );
        assert_eq!(
            app.world()
                .resource::<TerrainRevisionCache>()
                .footing_heights,
            [19.]
        );
        assert!(app.world().get_entity(old_root).is_err());
        assert_eq!(
            app.world().get::<Visibility>(new_root),
            Some(&Visibility::Visible)
        );
    }
}
