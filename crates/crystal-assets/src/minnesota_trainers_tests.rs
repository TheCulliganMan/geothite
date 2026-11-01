use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_core::battle::start::TrainerBattleStartStatus;
use crystal_core::models::Dv;
use crystal_core::state::GameState;

use crate::{
    CompiledGamePack, GameDataSet, build_minnesota_trainers_modpack,
    read_verified_compiled_game_pack,
};

fn packs() -> &'static (CompiledGamePack, CompiledGamePack) {
    static PACKS: OnceLock<(CompiledGamePack, CompiledGamePack)> = OnceLock::new();
    PACKS.get_or_init(|| {
        let path = std::env::var_os("CRYSTAL_MINNESOTA_TRAINER_BASE_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content-packs/minnesota-trainer-regional-base-v01.crystalpack")
            });
        let base =
            read_verified_compiled_game_pack(path.canonicalize().expect("Minnesota parent path"))
                .expect("verified Minnesota parent");
        let child = std::env::var_os("CRYSTAL_MINNESOTA_TRAINER_HISTORY_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content-packs/minnesota-trainers-v03.crystalpack")
            });
        // Read-only compatibility checks for the archived pre-compact pack.
        // New publication must pass the compact geography guard below.
        let pack = read_verified_compiled_game_pack(child.canonicalize().expect("history path"))
            .expect("verified historical trainer pack");
        (base, pack)
    })
}

fn state(data: &GameDataSet, levels: &[u8]) -> GameState {
    let mut state = data
        .start_overworld_session_from_spawn(
            data.runtime_spawn_point(0).unwrap(),
            &data.audio_ids().into_iter().map(str::to_owned).collect(),
        )
        .unwrap()
        .0;
    for (index, level) in levels.iter().enumerate() {
        state.storage.party.pokemon[index] = Some(
            data.create_pokemon("POUCHID", *level, Dv::default())
                .unwrap(),
        );
    }
    state.sync_party_from_storage();
    state
}

#[test]
fn minnesota_trainers_preserve_existing_world_and_cover_every_numbered_route() {
    let (base, pack) = packs();
    assert!(pack.data().minnesota_trainer_scaling);
    assert_eq!(base.compiled_audio(), pack.compiled_audio());
    assert_eq!(base.runtime_files(), pack.runtime_files());
    assert_eq!(base.data().pokemon, pack.data().pokemon);
    assert_eq!(
        base.data().trainers.trainers.len()
            + pack
                .data()
                .trainers
                .trainers
                .keys()
                .filter(|id| id.starts_with("MN_EXTRA_"))
                .count(),
        pack.data().trainers.trainers.len()
    );
    for (id, trainer) in &base.data().trainers.trainers {
        assert_eq!(&pack.data().trainers.trainers[id], trainer);
    }
    for (name, original) in &base.data().maps {
        let changed = &pack.data().maps[name];
        assert_eq!(original.blocks, changed.blocks, "terrain: {name}");
        assert_eq!(
            original.attributes, changed.attributes,
            "connections: {name}"
        );
        assert_eq!(
            original.events, changed.events,
            "doors and story events: {name}"
        );
        assert_eq!(
            &changed.objects[..original.objects.len()],
            &original.objects
        );
        for (label, body) in &original.scripts {
            assert_eq!(&changed.scripts[label], body, "original script: {label}");
        }
    }
    for route in 1..=20 {
        assert!(
            pack.data()
                .trainers
                .trainers
                .keys()
                .any(|id| id.starts_with(&format!("MN_EXTRA_R{route:02}_")))
        );
    }
    assert!(
        pack.data()
            .trainers
            .trainers
            .keys()
            .filter(|id| id.starts_with("MN_EXTRA_"))
            .count()
            >= 100
    );
}

#[test]
fn minnesota_trainers_first_battles_scale_evolve_learn_moves_and_validate_saves() {
    let data = packs().1.data();
    let mut seen = BTreeSet::new();
    let mut evolved = 0;
    let mut learned = 0;
    for level in [5, 25, 50, 100] {
        let state = state(data, &[level, level, level]);
        for map in data.maps.values().filter(|map| map.id.starts_with("Mn")) {
            for request in map
                .trainer_scripts
                .values()
                .chain(
                    map.scripted_trainer_battles
                        .iter()
                        .map(|battle| &battle.request),
                )
                .filter(|request| request.trainer_id.starts_with("MN_"))
            {
                let TrainerBattleStartStatus::Started(start) =
                    data.trainer_battle_start(&state, request.clone()).unwrap()
                else {
                    panic!("fresh first battle must start");
                };
                let trainer = data.trainers.get(&request.trainer_id).unwrap();
                let floor = trainer.party.iter().map(|mon| mon.level).max().unwrap();
                assert_eq!(
                    start.enemy_party.iter().map(|mon| mon.level).max(),
                    Some(level.max(floor))
                );
                for (original, mon) in trainer.party.iter().zip(&start.enemy_party) {
                    assert_eq!(mon.level - original.level, level.max(floor) - floor);
                    evolved += usize::from(original.species != mon.species.id);
                    learned += usize::from(original.moves != mon.moves);
                    assert!(!mon.moves.is_empty() && mon.moves.len() <= 4);
                    assert!(
                        data.evolutions
                            .0
                            .get(&mon.species.id)
                            .is_none_or(|entries| !entries
                                .iter()
                                .any(|entry| entry.method == "LEVEL"
                                    && entry.level.is_some_and(|required| required <= mon.level)))
                    );
                    for mov in &mon.moves {
                        assert_eq!(mov.current_pp, data.moves[&mov.name].pp);
                        assert!(
                            original.moves.iter().any(|old| old.name == mov.name)
                                || data.learnsets[&mon.species.id]
                                    .iter()
                                    .any(|entry| entry.0 <= mon.level && entry.1 == mov.name)
                        );
                    }
                }
                data.validate_saved_trainer_enemy_party_for_script(
                    trainer,
                    &start.enemy_party,
                    &start.enemy_pokemon,
                    &start.source_script,
                )
                .unwrap();
                let mut forged = start.enemy_party.clone();
                forged[0].species = data.pokemon["MEW"].clone();
                assert!(
                    data.validate_saved_trainer_enemy_party_for_script(
                        trainer,
                        &forged,
                        &forged[0],
                        &start.source_script
                    )
                    .is_err()
                );
                seen.insert(trainer.trainer_id.clone());
            }
        }
    }
    assert!(
        seen.len() > 300,
        "all Minnesota battles must be covered: {}",
        seen.len()
    );
    assert!(evolved > 100 && learned > 100);
}

#[test]
fn minnesota_trainers_respect_defeat_flags_and_ignore_eggs_lead_order_and_fainting() {
    let data = packs().1.data();
    let request = data
        .maps
        .values()
        .flat_map(|map| map.trainer_scripts.values())
        .find(|request| request.trainer_id.starts_with("MN_EXTRA_"))
        .unwrap();
    let mut a = state(data, &[5, 30, 40, 50]);
    let TrainerBattleStartStatus::Started(start) =
        data.trainer_battle_start(&a, request.clone()).unwrap()
    else {
        panic!("fresh battle");
    };
    let mut egg = data.create_pokemon("POUCHID", 100, Dv::default()).unwrap();
    egg.is_egg = true;
    a.storage.party.pokemon[4] = Some(egg);
    a.storage.party.pokemon.swap(0, 3);
    for mon in a.storage.party.pokemon.iter_mut().flatten() {
        mon.hp = 0;
    }
    let TrainerBattleStartStatus::Started(other) =
        data.trainer_battle_start(&a, request.clone()).unwrap()
    else {
        panic!("fresh battle");
    };
    assert_eq!(start.enemy_party, other.enemy_party);
    a.flags.set_event_flag(&request.event_flag, true).unwrap();
    assert!(matches!(
        data.trainer_battle_start(&a, request.clone()).unwrap(),
        TrainerBattleStartStatus::AlreadyDefeated { .. }
    ));
}

#[test]
fn minnesota_trainers_evolve_at_native_thresholds_and_learn_only_available_moves() {
    let data = packs().1.data();
    let template = data
        .trainers
        .trainers
        .values()
        .find(|trainer| trainer.trainer_id.starts_with("MN_EXTRA_"))
        .unwrap();
    let mut thresholds = 0;
    for (species, entries) in &data.evolutions.0 {
        if !data
            .pokemon
            .get(species)
            .is_some_and(|species| species.int_id > 251)
        {
            continue;
        }
        for evolution in entries.iter().filter(|entry| entry.method == "LEVEL") {
            let threshold = evolution.level.unwrap();
            let mut trainer = template.clone();
            trainer.party.truncate(1);
            let original = &mut trainer.party[0];
            original.species.clone_from(species);
            original.level = 1;
            original.moves = data.create_pokemon(species, 1, original.dvs).unwrap().moves;
            let before = data
                .rematch_trainer_at_ace(&trainer, threshold.saturating_sub(1).max(1))
                .unwrap();
            let after = data.rematch_trainer_at_ace(&trainer, threshold).unwrap();
            assert_eq!(
                &before.party[0].species, species,
                "before native evolution threshold"
            );
            assert_eq!(
                after.party[0].species, evolution.species,
                "at native evolution threshold"
            );
            for mov in &after.party[0].moves {
                assert!(
                    trainer.party[0]
                        .moves
                        .iter()
                        .any(|original| original.name == mov.name)
                        || data.learnsets[&evolution.species]
                            .iter()
                            .any(|entry| entry.0 <= threshold && entry.1 == mov.name)
                );
            }
            thresholds += 1;
        }
    }
    assert!(
        thresholds > 100,
        "native evolution families must be covered: {thresholds}"
    );
}

#[test]
fn minnesota_trainers_follow_current_collection_instead_of_active_team() {
    use crystal_core::models::PcBox;

    let data = packs().1.data();
    let request = &data.maps["MnR01C00R00"].trainer_scripts["MN_EXTRA_R01_01"];
    let mut state = state(data, &[50, 40, 30]);
    let battle = |state: &GameState| {
        let TrainerBattleStartStatus::Started(start) =
            data.trainer_battle_start(state, request.clone()).unwrap()
        else {
            panic!("fresh trainer battle");
        };
        start
    };
    let original = battle(&state);
    assert_eq!(
        original.enemy_party.iter().map(|mon| mon.level).max(),
        Some(50)
    );
    state.storage.pc_boxes = (0..14).map(PcBox::new).collect();
    for (party_slot, box_index) in [(0, 0), (1, 13), (2, 13)] {
        let pokemon = state.storage.party.pokemon[party_slot].take().unwrap();
        assert!(state.storage.pc_boxes[box_index].add_pokemon(pokemon));
        state.storage.party.pokemon[party_slot] =
            Some(data.create_pokemon("POUCHID", 5, Dv::default()).unwrap());
    }
    state.sync_party_from_storage();
    state.storage.validate_metadata().unwrap();
    assert_eq!(
        battle(&state).enemy_party,
        original.enemy_party,
        "depositing the strong team must not lower opponents"
    );
    for level in 1..=10 {
        assert!(
            state.storage.pc_boxes[1].add_pokemon(
                data.create_pokemon("POUCHID", level, Dv::default())
                    .unwrap()
            )
        );
    }
    let mut egg = data.create_pokemon("POUCHID", 100, Dv::default()).unwrap();
    egg.is_egg = true;
    assert!(state.storage.pc_boxes[12].add_pokemon(egg));
    assert_eq!(
        battle(&state).enemy_party,
        original.enemy_party,
        "weak catches and stored eggs do not distort difficulty"
    );
    state.storage.pc_boxes[0].remove_pokemon(0).unwrap();
    assert_eq!(
        battle(&state).enemy_party.iter().map(|mon| mon.level).max(),
        Some(40),
        "released Pokemon no longer contribute to the current collection"
    );
    assert!(
        state.storage.pc_boxes[0]
            .add_pokemon(data.create_pokemon("BOREALYNX", 55, Dv::default()).unwrap())
    );
    assert_eq!(
        battle(&state).enemy_party.iter().map(|mon| mon.level).max(),
        Some(55),
        "a stronger caught Pokemon raises the collection baseline"
    );
}

#[test]
fn minnesota_trainers_weak_first_catches_never_reduce_difficulty() {
    use crystal_core::models::PcBox;

    let data = packs().1.data();
    let request = &data.maps["MnR01C00R00"].trainer_scripts["MN_EXTRA_R01_01"];
    let mut game = state(data, &[50]);
    game.storage.pc_boxes = (0..14).map(PcBox::new).collect();
    let battle = |game: &GameState| {
        let TrainerBattleStartStatus::Started(start) =
            data.trainer_battle_start(game, request.clone()).unwrap()
        else {
            panic!("fresh trainer battle");
        };
        start.enemy_party
    };
    let original = battle(&game);
    assert_eq!(original.iter().map(|mon| mon.level).max(), Some(50));
    for level in 1..=5 {
        assert!(
            game.storage.pc_boxes[13].add_pokemon(
                data.create_pokemon("POUCHID", level, Dv::default())
                    .unwrap()
            )
        );
        assert_eq!(
            battle(&game),
            original,
            "weak catches must not lower the baseline even with fewer than three strong Pokemon"
        );
    }
}

#[test]
fn minnesota_trainers_reject_stale_geography_before_publication() {
    let (base, _) = packs();
    let error = build_minnesota_trainers_modpack(base, "{}").expect_err("stale parent rejected");
    assert!(
        error
            .to_string()
            .contains("compact Superior parent required")
    );
}

#[test]
fn minnesota_trainers_accept_compact_geometry_and_reject_retired_maps() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../content-packs/minnesota-superior-compact-v03.crystalpack");
    let mut pack = read_verified_compiled_game_pack(path.canonicalize().unwrap()).unwrap();
    super::geography::require_compact_superior(&pack).unwrap();
    let original = pack.data.maps["MnR12C00R00"].clone();
    pack.data.maps.insert("MnR12C01R00".to_owned(), original);
    let error = super::geography::require_compact_superior(&pack).unwrap_err();
    assert!(error.to_string().contains("retired exterior"));
}

#[test]
#[ignore = "requires CRYSTAL_CYCLISTS_PACK and CRYSTAL_CYCLISTS_BASE"]
fn minneapolis_cyclists_preserve_world_and_start_all_battles() {
    let load = |name| {
        read_verified_compiled_game_pack(std::env::var_os(name).expect("fixture pack"))
            .expect("verified cyclists fixture")
    };
    let base = load("CRYSTAL_CYCLISTS_BASE");
    let pack = load("CRYSTAL_CYCLISTS_PACK");
    assert_eq!(base.runtime_files(), pack.runtime_files());
    assert_eq!(base.compiled_audio(), pack.compiled_audio());
    assert_eq!(base.data().pokemon, pack.data().pokemon);
    for (id, map) in &base.data().maps {
        if id != "MnMinneapolis" {
            assert_eq!(map, &pack.data().maps[id]);
        }
    }
    let original = &base.data().maps["MnMinneapolis"];
    let city = &pack.data().maps["MnMinneapolis"];
    assert_eq!(original.blocks, city.blocks);
    assert_eq!(original.events.warps, city.events.warps);
    assert_eq!(&city.objects[..original.objects.len()], original.objects);
    assert_eq!(city.objects.len(), original.objects.len() + 18);
    for object in &city.objects[original.objects.len()..] {
        assert!(matches!(
            object.sprite.as_str(),
            "SPRITE_CHRIS_BIKE" | "SPRITE_KRIS_BIKE"
        ));
        assert_eq!(object.spritemovedata, "SPRITEMOVEDATA_PATROL_CLOCKWISE");
        assert!(2 * (object.move_range_x + object.move_range_y) >= 32);
        assert_eq!(object.object_type, "OBJECTTYPE_SCRIPT");
        assert_eq!(object.radius, 0);
        let id = object.object_identifier.as_ref().expect("cyclist ID");
        let request = city
            .scripted_trainer_battles
            .iter()
            .find(|b| b.request.trainer_id == *id)
            .expect("optional cyclist battle")
            .request
            .clone();
        let return_index = city.scripts[id]
            .as_array()
            .expect("cyclist program")
            .iter()
            .position(|command| command["command"] == "reloadmapafterbattle")
            .expect("authored return from battle");
        assert_eq!(
            pack.data()
                .script_map_command("MnMinneapolis", id, return_index)
                .expect("cyclist must return through an executable map command")
                .command,
            "reloadmapafterbattle"
        );
        let game = state(pack.data(), &[24]);
        let TrainerBattleStartStatus::Started(start) = pack
            .data()
            .trainer_battle_start(&game, request)
            .expect("native cyclist battle")
        else {
            panic!("fresh cyclist must battle");
        };
        assert_eq!(start.enemy_party.len(), 2);
    }
}
