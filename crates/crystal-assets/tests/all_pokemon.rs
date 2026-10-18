use std::path::PathBuf;

use crystal_assets::{
    ALL_251_MANIFEST_ID, EVOLUTION_STONES, LINK_CABLE_ITEM_ID, SOLO_EVOLUTION_MANIFEST_ID,
    STONE_MART_ID, build_all_pokemon_modpack, read_verified_compiled_game_pack,
    verify_all_251_coverage,
};
use crystal_core::{
    models::Dv,
    state::{GameState, OverworldMemory},
    world::{
        encounters::TimeOfDay,
        map::{Direction, TilePosition},
        movement::MovementMode,
    },
};

fn base_path() -> PathBuf {
    std::env::var_os("CRYSTAL_MODPACK_TEST_BASE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/core-modular.browser.crystalpack")
        })
        .canonicalize()
        .expect("resolve local test pack")
}

#[test]
fn all_pokemon_pack_covers_251_preserves_content_and_roundtrips() {
    let base = read_verified_compiled_game_pack(base_path()).unwrap();
    let pack = build_all_pokemon_modpack(&base).unwrap();
    verify_all_251_coverage(&pack).unwrap();
    let id = pack.runtime_modpack_id().unwrap();
    assert!(id.contains(ALL_251_MANIFEST_ID) && id.contains(SOLO_EVOLUTION_MANIFEST_ID));
    assert_eq!(
        build_all_pokemon_modpack(&pack)
            .unwrap()
            .identity()
            .unwrap(),
        pack.identity().unwrap()
    );
    let mut unchanged = pack.data().clone();
    unchanged.wild_encounters = base.data().wild_encounters.clone();
    unchanged.items = base.data().items.clone();
    unchanged.evolutions = base.data().evolutions.clone();
    unchanged.marts = base.data().marts.clone();
    assert_eq!(&unchanged, base.data());
    for (id, item) in &base.data().items {
        let mut expected = item.clone();
        if id == "MOON_STONE" {
            expected.price = 2100;
        }
        assert_eq!(&pack.data().items[id], &expected, "existing item {id}");
    }
    for (source, entries) in &base.data().evolutions.0 {
        for entry in entries {
            assert!(
                pack.data().evolutions.0[source].contains(entry),
                "existing evolution {source}"
            );
        }
    }
    for (id, stock) in &base.data().marts.0 {
        if id != STONE_MART_ID {
            assert_eq!(&pack.data().marts.0[id], stock);
        }
    }
    for (map, table) in &base.data().wild_encounters {
        let changed = &pack.data().wild_encounters[map];
        assert_eq!(table.grass_rates, changed.grass_rates);
        assert_eq!(table.water_rate, changed.water_rate);
        assert_eq!(table.zones, changed.zones);
        assert_eq!(table.swarm_overrides, changed.swarm_overrides);
    }
    assert_eq!(pack.runtime_files(), base.runtime_files());
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    assert_eq!(
        pack.audio_manifest().unwrap(),
        base.audio_manifest().unwrap()
    );
    let output =
        std::env::temp_dir().join(format!("all-pokemon-{}.crystalpack", std::process::id()));
    pack.write_preserving_storage(&output).unwrap();
    let loaded = read_verified_compiled_game_pack(&output).unwrap();
    std::fs::remove_file(output).unwrap();
    assert_eq!(loaded.identity().unwrap(), pack.identity().unwrap());
    assert_eq!(loaded.data(), pack.data());
    verify_all_251_coverage(&loaded).unwrap();
}

#[test]
fn all_pokemon_stone_counter_sells_cables_and_all_ten_trade_evolutions_work_solo() {
    let base = read_verified_compiled_game_pack(base_path()).unwrap();
    let pack = build_all_pokemon_modpack(&base).unwrap();
    let data = pack.data();
    let cable = &data.items[LINK_CABLE_ITEM_ID];
    assert!(cable.field_usable && cable.consumable && !cable.battle_usable);
    assert_eq!(cable.price, 2100);
    let command = data
        .map_module("CeladonDeptStore4F")
        .unwrap()
        .script_shop_commands
        .iter()
        .find(|command| command.mart_id == STONE_MART_ID)
        .unwrap();
    let mut shop = GameState::reset_wram_for_new_game();
    shop.money = 50_000;
    data.open_script_shop(
        &mut shop,
        "CeladonDeptStore4F",
        "CeladonDeptStore4F",
        &command.source_script,
        command.command_index,
    )
    .unwrap();
    for id in EVOLUTION_STONES.iter().copied().chain([LINK_CABLE_ITEM_ID]) {
        assert!(
            crystal_core::systems::shop::max_buy_quantity(&shop, &data.items[id]) > 0,
            "the production quantity selector must offer {id}"
        );
        assert!(data.buy_shop_item(&mut shop, id, 1).unwrap().success);
        assert_eq!(shop.bag.quantity(&data.items[id]), 1, "purchase {id}");
    }
    assert_eq!(
        shop.money,
        50_000
            - EVOLUTION_STONES
                .iter()
                .map(|id| u32::from(data.items[*id].price))
                .sum::<u32>()
            - 2100
    );
    for (source, target, held) in [
        ("KADABRA", "ALAKAZAM", None),
        ("MACHOKE", "MACHAMP", None),
        ("GRAVELER", "GOLEM", None),
        ("HAUNTER", "GENGAR", None),
        ("KADABRA", "ALAKAZAM", Some("BERRY")),
        ("POLIWHIRL", "POLITOED", Some("KINGS_ROCK")),
        ("SLOWPOKE", "SLOWKING", Some("KINGS_ROCK")),
        ("ONIX", "STEELIX", Some("METAL_COAT")),
        ("SCYTHER", "SCIZOR", Some("METAL_COAT")),
        ("SEADRA", "KINGDRA", Some("DRAGON_SCALE")),
        ("PORYGON", "PORYGON2", Some("UP_GRADE")),
    ] {
        let mut state = GameState::reset_wram_for_new_game();
        state.overworld = OverworldMemory::Active {
            map_name: "NewBarkTown".into(),
            tile: TilePosition::new(5, 8),
            facing: Direction::Down,
            mode: MovementMode::Normal,
        };
        let mut pokemon = data
            .create_pokemon(source, 30, Dv::from_non_hp(10, 10, 10, 10))
            .unwrap();
        pokemon.item = held.map(str::to_owned);
        state.storage.party.pokemon[0] = Some(pokemon);
        state.sync_party_from_storage();
        state.bag.add_item(cable, 2).unwrap();
        let (_, result) = data
            .use_bag_item_on_party_pokemon(&mut state, LINK_CABLE_ITEM_ID, 0, TimeOfDay::Day)
            .unwrap();
        assert_eq!(result.evolution_target.as_deref(), Some(target), "{source}");
        assert!(result.consumed && result.evolution_cancel_snapshot.is_none());
        let evolved = state.storage.party.pokemon[0].as_ref().unwrap();
        assert_eq!(evolved.species.id, target);
        assert_eq!(evolved.item.as_deref(), held.filter(|id| *id == "BERRY"));
        assert_eq!(evolved.level, 30);
        assert_eq!(state.bag.quantity(cable), 1);
        assert!(
            data.evolutions.0[source]
                .iter()
                .any(|entry| entry.method == "TRADE" && entry.species == target)
        );
    }
}

#[test]
fn all_pokemon_cable_failures_leave_pokemon_and_inventory_unchanged() {
    let base = read_verified_compiled_game_pack(base_path()).unwrap();
    let pack = build_all_pokemon_modpack(&base).unwrap();
    let data = pack.data();
    for (species, held, egg) in [
        ("ONIX", None, false),
        ("ONIX", Some("KINGS_ROCK"), false),
        ("KADABRA", Some("EVERSTONE"), false),
        ("PIKACHU", None, false),
        ("KADABRA", None, true),
    ] {
        let mut state = GameState::reset_wram_for_new_game();
        state.overworld = OverworldMemory::Active {
            map_name: "NewBarkTown".into(),
            tile: TilePosition::new(5, 8),
            facing: Direction::Down,
            mode: MovementMode::Normal,
        };
        let mut pokemon = data
            .create_pokemon(species, 30, Dv::from_non_hp(10, 10, 10, 10))
            .unwrap();
        pokemon.item = held.map(str::to_owned);
        pokemon.is_egg = egg;
        state.storage.party.pokemon[0] = Some(pokemon.clone());
        state.sync_party_from_storage();
        state
            .bag
            .add_item(&data.items[LINK_CABLE_ITEM_ID], 1)
            .unwrap();
        assert!(
            data.use_bag_item_on_party_pokemon(&mut state, LINK_CABLE_ITEM_ID, 0, TimeOfDay::Day)
                .is_err()
        );
        assert_eq!(state.storage.party.pokemon[0].as_ref().unwrap(), &pokemon);
        assert_eq!(state.bag.quantity(&data.items[LINK_CABLE_ITEM_ID]), 1);
    }
}
