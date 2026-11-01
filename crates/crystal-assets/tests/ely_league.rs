//! External-pack preservation and source-to-native League publication checks.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, build_ely_league_modpack, read_verified_compiled_game_pack,
};

fn input() -> &'static (CompiledGamePack, CompiledGamePack) {
    static INPUT: OnceLock<(CompiledGamePack, CompiledGamePack)> = OnceLock::new();
    INPUT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let base = read_verified_compiled_game_pack(
            std::env::var_os("CRYSTAL_ELY_BASE")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    root.join("content-packs/minnesota-northstar-north-shore-story.crystalpack")
                }),
        )
        .expect("completed North Shore pack");
        let result = build_ely_league_modpack(
            &base,
            &std::fs::read_to_string(root.join("planning/minnesota/towns/MnEly.json"))
                .expect("Ely source"),
            &std::fs::read_to_string(root.join("planning/minnesota/towns/MnEly.layout.json"))
                .expect("Ely layout"),
            &std::fs::read_to_string(root.join("planning/minnesota/story.json"))
                .expect("League source"),
        )
        .expect("publish Ely");
        let written = read_verified_compiled_game_pack(
            std::env::var_os("CRYSTAL_ELY_PACK")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    root.join("content-packs/minnesota-northstar-ely-league.crystalpack")
                }),
        )
        .expect("written Ely pack");
        assert_eq!(
            result.identity().expect("identity"),
            written.identity().expect("written identity")
        );
        (base, result)
    })
}

#[test]
#[ignore = "requires completed external North Shore and Ely League packs"]
fn ely_league_preserves_old_content_and_original_game_endings() {
    let (base, result) = input();
    let old = base.data();
    let new = result.data();
    assert_eq!(new.maps.len(), old.maps.len() + 10);
    for (name, map) in &old.maps {
        let mut expected = map.clone();
        if name == "MnR13C00R02" {
            expected
                .attributes
                .connections
                .push(crystal_core::map::MapConnection {
                    direction: "south".into(),
                    target_map: "MnEly".into(),
                    offset: 5,
                });
            expected.attributes.connection_flags = Some("NORTH | SOUTH".into());
        }
        assert_eq!(new.maps[name], expected, "old map {name}");
    }
    assert_eq!(old.audio, new.audio);
    assert_eq!(old.tilesets, new.tilesets);
    assert_eq!(old.pokemon, new.pokemon);
    assert_eq!(old.moves, new.moves);
    assert_eq!(old.wild_encounters, new.wild_encounters);
    for (name, value) in &old.story_event_script_constants.global {
        assert_eq!(
            new.story_event_script_constants.global.get(name),
            Some(value),
            "original constant {name}"
        );
    }
    assert_eq!(base.runtime_files(), result.runtime_files());
    assert_eq!(base.compiled_audio(), result.compiled_audio());
    for (name, trainer) in &old.trainers.trainers {
        assert_eq!(
            new.trainers.trainers.get(name),
            Some(trainer),
            "original trainer {name}"
        );
    }
    for (name, stock) in &old.marts.0 {
        assert_eq!(new.marts.0.get(name), Some(stock), "original shop {name}");
    }
    assert_eq!(new.trainers.trainers.len(), old.trainers.trainers.len() + 5);
    assert_eq!(new.marts.0.len(), old.marts.0.len() + 1);
    for (name, spawn) in &old.runtime_spawn_points {
        assert_eq!(new.runtime_spawn_points.get(name), Some(spawn));
    }
    for name in [
        "SPAWN_LANCE",
        "SPAWN_RED",
        "SPAWN_NEW_BARK",
        "SPAWN_MT_SILVER",
    ] {
        assert_eq!(
            old.story_event_script_constants.global[name],
            new.story_event_script_constants.global[name]
        );
    }
    assert_eq!(
        new.runtime_spawn_points.len(),
        old.runtime_spawn_points.len() + 2
    );
    let clinic = new
        .runtime_spawn_points
        .values()
        .find(|s| s.map_name == "MnElyPokecenter1F")
        .expect("Ely recovery");
    assert_eq!((clinic.tile_x, clinic.tile_y), (3, 5));
    assert_eq!(
        new.story_event_script_constants.maps["MnLeagueHallOfFame"]["GEOTHITE_POST_CREDITS_SPAWN"],
        i64::from(clinic.identifier)
    );
}

#[test]
#[ignore = "requires completed external North Shore and Ely League packs"]
fn ely_league_uses_native_rooms_and_reconstructs_script_payloads() {
    let (base, result) = input();
    let old = base.data();
    let new = result.data();
    for (name, template) in [
        ("MnElyPokecenter1F", "GoldenrodPokecenter1F"),
        ("MnElyMart", "CherrygroveMart"),
        ("MnLeagueLobby", "IlexForestAzaleaGate"),
        ("MnLeagueRoom1", "KogasRoom"),
        ("MnLeagueRoom2", "BrunosRoom"),
        ("MnLeagueRoom3", "WillsRoom"),
        ("MnLeagueRoom4", "KarensRoom"),
        ("MnLeagueChampion", "LancesRoom"),
        ("MnLeagueHallOfFame", "HallOfFame"),
    ] {
        let map = &new.maps[name];
        let mut expected = old.maps[template].blocks.clone();
        if name == "MnElyPokecenter1F" {
            expected[15] = 0x04;
        }
        assert_eq!(map.blocks, expected);
        let mut uncached = new.clone();
        uncached.maps.remove(name);
        let rebuilt = uncached
            .assemble_map_module_from_compiled_payloads(name)
            .expect("native payload reconstruction");
        assert_eq!(
            rebuilt.scripted_trainer_battles, map.scripted_trainer_battles,
            "{name}"
        );
        assert_eq!(
            rebuilt.script_map_commands, map.script_map_commands,
            "{name}"
        );
        assert_eq!(
            rebuilt.script_runtime_commands, map.script_runtime_commands,
            "{name}"
        );
        assert_eq!(
            rebuilt.script_variable_commands, map.script_variable_commands,
            "{name}"
        );
        assert_eq!(
            rebuilt.script_block_changes, map.script_block_changes,
            "{name}"
        );
        assert_eq!(rebuilt.events, map.events, "{name}");
    }
    assert_eq!(
        new.maps["MnLeagueChampion"].scripted_trainer_battles.len(),
        1
    );
    assert_eq!(new.trainers.trainers["MN_CHAMPION"].party.len(), 6);
}
