//! External-pack migration fixtures. No user saves are read or written.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::state::OverworldMemory;
use crystal_core::world::map::TilePosition;

use super::{GYMS, SHOP, migrate_gym_review_save};
use crate::{CrystalRuntime, RuntimeGameShell};

fn runtimes() -> (AssetRoot, CrystalRuntime, CrystalRuntime) {
    static PACKS: OnceLock<(AssetRoot, CrystalRuntime, CrystalRuntime)> = OnceLock::new();
    PACKS
        .get_or_init(|| {
            let source = PathBuf::from(
                std::env::var_os("CRYSTAL_GYM_MIGRATION_SOURCE").expect("source pack"),
            );
            let target = PathBuf::from(
                std::env::var_os("CRYSTAL_GYM_MIGRATION_TARGET").expect("target pack"),
            );
            let root = AssetRoot::new(source.parent().expect("pack directory"));
            let load = |path| {
                CrystalRuntime::from_loaded_compiled_pack(
                    &root,
                    read_loaded_verified_compiled_game_pack(path).expect("verified pack"),
                )
                .expect("runtime")
            };
            let old = load(source);
            let new = load(target);
            (root, old, new)
        })
        .clone()
}

fn fixture(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    map: &str,
    x: u16,
    y: u16,
) -> RuntimeGameShell {
    RuntimeGameShell::new_game_at_runtime_tile(
        root.clone(),
        runtime.clone(),
        runtime.title_new_game_spawn_identifier().expect("spawn"),
        map,
        i16::try_from(x).expect("x"),
        i16::try_from(y).expect("y"),
    )
    .expect("source position fixture")
}

#[test]
#[ignore = "requires CRYSTAL_GYM_MIGRATION_SOURCE and CRYSTAL_GYM_MIGRATION_TARGET"]
fn gym_migration_rebuilds_eight_rooms_and_preserves_progress() {
    let (root, source, target) = runtimes();
    for (index, &name) in GYMS.iter().enumerate() {
        let old = &source.data().maps[name];
        let arrival = old
            .events
            .warps
            .iter()
            .find(|w| w.index == 3)
            .expect("old arrival");
        let mut game = fixture(&root, &source, name, arrival.x, arrival.y);
        game.session_mut().state_mut().money = 4321;
        for trainer in 1..=3 {
            let flag = format!("EVENT_MN_GYM_{}_TRAINER_{}_DEFEATED", index + 1, trainer);
            if source
                .data()
                .story_event_script_constants
                .global
                .contains_key(&flag)
            {
                game.session_mut()
                    .state_mut()
                    .flags
                    .set_event_flag(&flag, true)
                    .expect("trainer fixture");
            }
        }
        if index == 6 {
            for flag in [
                "EVENT_MN_GYM_7_CUT_SOUTH_HEDGE_PRUNED",
                "EVENT_MN_GYM_7_CUT_NORTH_HEDGE_PRUNED",
            ] {
                game.session_mut()
                    .state_mut()
                    .flags
                    .set_event_flag(flag, true)
                    .expect("pruned fixture");
            }
        }
        game.apply_current_map_setup_callbacks("MAPSETUP_SUBMENU")
            .expect("old gate callbacks");
        let original = game.session().state().clone();
        let migrated = migrate_gym_review_save(&source, &target, &root, &original)
            .unwrap_or_else(|error| panic!("{name}: {error:#}"));
        assert_eq!(migrated.money, original.money);
        assert_eq!(migrated.flags, original.flags);
        assert_eq!(migrated.storage, original.storage);
        assert_eq!(migrated.bag, original.bag);
        assert!(!migrated.map_block_overrides.contains_key(name));
        assert_eq!(
            game.session().state(),
            &original,
            "source remains unchanged"
        );
        let next = &target.data().maps[name];
        let arrival = next
            .events
            .warps
            .iter()
            .find(|w| w.index == 3)
            .expect("new arrival");
        assert!(
            matches!(&migrated.overworld, OverworldMemory::Active { map_name, tile, .. }
            if map_name == name && *tile == TilePosition::new(arrival.x as i16, arrival.y as i16))
        );
        let resumed = target
            .resume_overworld_session(&root, migrated)
            .expect("target Continue");
        assert_eq!(resumed.overworld().map.name, name);
        for (actor, expected) in resumed.overworld().objects.iter().zip(&next.objects) {
            assert_eq!((actor.x, actor.y), (expected.x, expected.y), "{name} actor");
        }
        if index == 6 {
            let tileset = &target.data().tilesets[&next.attributes.tileset_name];
            for (x, y) in [(3_usize, 12_usize), (7, 6)] {
                let block = resumed.overworld().map.metatile_ids
                    [(y / 2) * usize::from(next.attributes.width) + x / 2];
                assert_eq!(
                    tileset.collision[&format!("{block:02x}")][(y % 2) * 2 + x % 2],
                    "FLOOR",
                    "pruned {name} tree"
                );
            }
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_GYM_MIGRATION_SOURCE and CRYSTAL_GYM_MIGRATION_TARGET"]
fn gym_migration_handles_canoe_shop_and_rejects_unknown_changes() {
    let (root, source, target) = runtimes();
    let outside = fixture(&root, &source, "MnStPaul", 15, 63);
    let original = outside.session().state().clone();
    let moved = migrate_gym_review_save(&source, &target, &root, &original).expect("sign tile");
    assert!(
        matches!(moved.overworld, OverworldMemory::Active { ref map_name, tile, .. }
        if map_name == "MnStPaul" && tile == TilePosition::new(15,64))
    );
    target
        .resume_overworld_session(&root, moved)
        .expect("safe sidewalk");

    let shop = fixture(&root, &source, SHOP, 11, 5);
    let original = shop.session().state().clone();
    let migrated =
        migrate_gym_review_save(&source, &target, &root, &original).expect("canoe display");
    assert!(
        matches!(migrated.overworld, OverworldMemory::Active { ref map_name, tile, .. }
        if map_name == SHOP && tile == TilePosition::new(11,6))
    );
    let resumed = target
        .resume_overworld_session(&root, migrated)
        .expect("shop Continue");
    assert!(
        resumed.overworld().object_has_loaded_struct(2),
        "new canoe display is loaded"
    );
    assert_eq!(
        shop.session().state(),
        &original,
        "source remains unchanged"
    );

    let gym = fixture(&root, &source, "MnRochesterGym", 6, 17);
    let mut unknown = gym.session().state().clone();
    let old = &source.data().maps["MnRochesterGym"];
    unknown
        .map_block_overrides
        .entry("MnRochesterGym".into())
        .or_default()
        .insert((0, 0), old.blocks[0]);
    assert!(migrate_gym_review_save(&source, &target, &root, &unknown).is_err());
    let mut pending = gym.session().state().clone();
    pending.script_runtime.player_input_locked = true;
    assert!(migrate_gym_review_save(&source, &target, &root, &pending).is_err());
    assert!(migrate_gym_review_save(&target, &source, &root, gym.session().state()).is_err());
}
