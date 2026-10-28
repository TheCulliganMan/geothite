//! Bridge overlay regressions. Root executed all five cases successfully; retained
//! drawing and four actual GPU views were independently checked on the river pack.
use std::collections::BTreeMap;
use std::path::PathBuf;

use bevy::prelude::Assets;
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::world::collision::PlayerTraversalState;
use crystal_runtime::{CrystalRuntime, RuntimeTilesetCatalogSnapshot};

use super::{OVERLAY_DEPTH, actor_depth, covers_subtile, overlay_spec};

fn tileset(tokens: [&str; 4]) -> RuntimeTilesetCatalogSnapshot {
    RuntimeTilesetCatalogSnapshot {
        tileset_id: "bridge_test".into(),
        collision: BTreeMap::from([("01".into(), tokens.map(str::to_owned).to_vec())]),
        palette_map: Vec::new(),
    }
}

#[test]
fn bridge_overlay_masks_only_explicit_collision_quadrants() {
    let bank = tileset(["BRIDGE_NS_RAIL", "FLOOR", "WATER", "BRIDGE_EW_DECK"]);
    for y in 0..4 {
        for x in 0..4 {
            assert_eq!(
                covers_subtile(&bank, 1, x, y).expect("quadrant"),
                (x < 2 && y < 2) || (x >= 2 && y >= 2)
            );
        }
    }
    for token in [
        "FLOOR",
        "WALL",
        "WATER",
        "BUOY",
        "TALL_GRASS",
        "BRIDGE_NS_DECK_EXTRA",
    ] {
        let ordinary = tileset([token; 4]);
        assert!(!covers_subtile(&ordinary, 1, 0, 0).expect("ordinary native token"));
    }
    assert!(covers_subtile(&bank, 1, 4, 0).is_err());
    assert!(covers_subtile(&bank, 2, 0, 0).is_err());
    let short = RuntimeTilesetCatalogSnapshot {
        collision: BTreeMap::from([("01".into(), vec!["BRIDGE_NS_DECK".into()])]),
        ..bank
    };
    assert!(covers_subtile(&short, 1, 0, 0).is_err());
}

#[test]
fn bridge_depth_separates_entire_surf_oam_from_opaque_deck_and_foot_oam() {
    // Both mouth approaches are ordinary WATER. Layering must not depend on
    // bridge footing: a Surf avatar's head overlaps the deck before its feet.
    for ordinary_depth in [0.99, 1.0, 1.09, 1.181] {
        let lower = actor_depth(ordinary_depth, true, PlayerTraversalState::Surf);
        let upper = actor_depth(ordinary_depth, true, PlayerTraversalState::Walk);
        assert!(0.0 < lower && lower < OVERLAY_DEPTH);
        assert!(OVERLAY_DEPTH < upper && upper < 2.4);
        assert_eq!(upper, ordinary_depth);
        for traversal in [PlayerTraversalState::Walk, PlayerTraversalState::Surf] {
            assert_eq!(
                actor_depth(ordinary_depth, false, traversal),
                ordinary_depth
            );
        }
    }
    // Native Y/X/object-slot ordering remains intact within either plane.
    assert!(
        actor_depth(1.01, true, PlayerTraversalState::Surf)
            < actor_depth(1.02, true, PlayerTraversalState::Surf)
    );
}

#[test]
#[ignore = "requires a completed external pack; tests actual production source images"]
fn bridge_opaque_overlay_copies_actual_native_rgba_including_palette_zero() {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let root = AssetRoot::new(repository.clone());
    let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| repository.join("content-packs/text-tui.crystalpack"));
    let loaded = read_loaded_verified_compiled_game_pack(path).expect("actual pack");
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).expect("runtime assets");
    let root = runtime
        .materialize_runtime_files()
        .expect("temporary native asset mount");
    let definition = &runtime.data().tilesets["johto"];
    let mut images = Assets::default();
    let art =
        super::super::load_tileset_art(&root, "johto", "day", &definition.palette_map, &mut images)
            .expect("actual Johto tiles");
    let (index, _) = art
        .transition_tiles
        .iter()
        .enumerate()
        .find(|(_, tile)| tile.indices.contains(&0) && tile.indices.iter().any(|i| *i != 0))
        .expect("native glyph with zero and nonzero palette indices");
    let handle = &art.tile_handles[index];
    let source = images
        .get(handle)
        .expect("native source image")
        .data
        .clone()
        .expect("native RGBA");
    let bank = tileset(["BRIDGE_NS_DECK"; 4]);
    let spec = overlay_spec(&bank, 1, 0, 0, handle).expect("source overlay");
    let composed = super::super::compose_priority_tile_grid(&[spec], 1, 1, None, &mut images);
    let actual = images
        .get(&composed)
        .expect("composed overlay")
        .data
        .as_ref()
        .expect("RGBA");
    assert_eq!(
        actual, &source,
        "every native RGB/alpha byte retained, including index zero"
    );
    assert!(
        actual
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| pixel[3] == 255)
    );
    let native_priority = images
        .get(&art.priority_tile_handles[index])
        .expect("native roof priority glyph")
        .data
        .as_ref()
        .expect("priority RGBA");
    assert!(
        native_priority
            .as_chunks::<4>()
            .0
            .iter()
            .any(|pixel| pixel[3] == 0)
    );
}

#[test]
fn bridge_scene_ignores_unused_alias_rows_and_keeps_native_actor_modes() {
    use super::{blocks_have_bridge, object_traversal};
    let mut bank = tileset(["BRIDGE_NS_DECK"; 4]);
    bank.collision.insert("00".into(), vec!["WATER".into(); 4]);
    assert!(!blocks_have_bridge(&bank, &[0, 0]).expect("ordinary installed blocks"));
    assert!(blocks_have_bridge(&bank, &[0, 1, 0]).expect("installed bridge"));
    for token in [
        "BRIDGE_NS_DECK",
        "BRIDGE_NS_RAIL",
        "BRIDGE_EW_DECK",
        "BRIDGE_EW_RAIL",
    ] {
        assert!(covers_subtile(&tileset([token; 4]), 1, 3, 3).expect("explicit bridge"));
    }
    assert!(matches!(
        object_traversal("SPRITEMOVEDATA_SWIM_WANDER"),
        PlayerTraversalState::Surf
    ));
    for movement in [
        "SPRITEMOVEDATA_STILL",
        "SPRITEMOVEDATA_WANDER",
        "SPRITEMOVEDATA_STANDING_DOWN",
    ] {
        assert!(matches!(
            object_traversal(movement),
            PlayerTraversalState::Walk
        ));
    }
}

#[test]
#[ignore = "requires completed external native art; checks original transition palette output"]
fn bridge_battle_transition_keeps_native_rgb_and_opaque_zero_on_only_bridge_cells() {
    use super::super::{
        CLASSIC_SCROLL_HALO_TILES, CLASSIC_SCROLL_TILES_X, CLASSIC_SCROLL_TILES_Y, TILE_SIZE,
        VisibleBattleTransition, prepare_battle_transition_texture,
    };
    use super::transition_overlay;
    use bevy::prelude::Vec2;

    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let root = AssetRoot::new(repository.clone());
    let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| repository.join("content-packs/text-tui.crystalpack"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(path).expect("actual pack"),
    )
    .expect("native assets");
    let root = runtime
        .materialize_runtime_files()
        .expect("temporary native asset mount");
    let mut images = Assets::default();
    let art = super::super::load_tileset_art(
        &root,
        "johto",
        "day",
        &runtime.data().tilesets["johto"].palette_map,
        &mut images,
    )
    .expect("native Johto art");
    let tile = art
        .transition_tiles
        .iter()
        .find(|tile| tile.indices.contains(&0) && tile.indices.iter().any(|index| *index != 0))
        .expect("native tile containing palette zero");
    let columns = CLASSIC_SCROLL_TILES_X as usize;
    let count = columns * CLASSIC_SCROLL_TILES_Y as usize;
    let tiles = vec![tile.clone(); count];
    let mut mask = vec![false; count];
    let halo = CLASSIC_SCROLL_HALO_TILES as usize;
    mask[(halo + 4) * columns + halo + 4] = true;
    for camera in [Vec2::ZERO, Vec2::new(TILE_SIZE * 0.5, 0.0)] {
        for frame in [0, 3, 5, 12, 78] {
            let transition = VisibleBattleTransition {
                frame,
                trainer_battle: false,
                cave_environment: false,
                stronger_enemy: false,
            };
            let base = prepare_battle_transition_texture(
                &root,
                transition,
                &tiles,
                camera,
                false,
                false,
                None,
                &mut images,
            )
            .expect("production native palette frame");
            let before = images
                .get(&base)
                .expect("base image")
                .data
                .clone()
                .expect("RGBA");
            let overlay = transition_overlay(&base, &mask, camera, None, &mut images)
                .expect("bridge transition mask");
            let actual = images
                .get(&overlay)
                .expect("bridge image")
                .data
                .as_ref()
                .expect("RGBA");
            assert_eq!(
                actual
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .filter(|pixel| pixel[3] == 255)
                    .count(),
                64
            );
            for (pixel, native) in actual
                .as_chunks::<4>()
                .0
                .iter()
                .zip(before.as_chunks::<4>().0.iter())
            {
                assert_eq!(pixel[..3], native[..3], "native flash palette RGB retained");
            }
            assert_eq!(
                images
                    .get(&base)
                    .expect("unmodified base")
                    .data
                    .as_ref()
                    .expect("RGBA"),
                &before,
                "bridge masking never mutates the shared transition background"
            );
            assert!(
                transition_overlay(&base, &mask[..count - 1], camera, None, &mut images).is_err()
            );
        }
    }
}
