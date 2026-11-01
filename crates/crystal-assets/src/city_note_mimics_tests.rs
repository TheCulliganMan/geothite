use std::path::PathBuf;

use crystal_core::models::Dv;
use crystal_core::random::ReplayDivider;
use crystal_core::state::GameState;

use super::{Notes, build_city_note_mimics, preserves_authored_level, selected_notes, validate};

const SOURCE: &str = include_str!("../../../planning/minnesota/city-note-mimics.json");

#[test]
fn city_notes_select_a_stable_minority_and_keep_low_level_basic_slots() {
    let source: Notes = serde_json::from_str(SOURCE).expect("checked-in source");
    validate(&source).expect("valid contract");
    for city in &source.cities {
        let picked = selected_notes(&source, city);
        assert_eq!(picked.len(), 2);
        assert_eq!(picked, selected_notes(&source, city));
        assert!(
            city.encounters
                .iter()
                .any(|e| e.species == "JOTLET" && e.level == 6 && !e.scale_level)
        );
    }
    let changed: Notes =
        serde_json::from_str(&SOURCE.replace("20261030", "20261031")).expect("new seed");
    assert!(
        source
            .cities
            .iter()
            .zip(&changed.cities)
            .any(|(a, b)| selected_notes(&source, a) != selected_notes(&changed, b))
    );
    assert!(preserves_authored_level(
        "MnMinneapolis",
        "MnCityNoteMPLS01Native"
    ));
    assert!(!preserves_authored_level(
        "MnMankato",
        "MN_MANKATO_NOTEBOOK"
    ));
    assert!(!preserves_authored_level(
        "MnMinneapolis",
        "MnCityNoteMPLS01Challenge"
    ));
    assert!(!preserves_authored_level(
        "GoldenrodCity",
        "MnCityNoteMPLS01Native"
    ));
}

#[test]
fn city_notes_reject_story_locations_invalid_frequency_and_evolved_refuges() {
    for replacement in [
        SOURCE.replace("MnMinneapolis", "MnMankato"),
        SOURCE.replace("\"mimic_denominator\": 4", "\"mimic_denominator\": 0"),
        SOURCE.replace("\"mimic_numerator\": 1", "\"mimic_numerator\": 2"),
        SOURCE.replace("\"JOTLET\"", "\"MEMOFOLD\""),
        SOURCE.replace("\"JOTLET\"", "\"VOLTORB\""),
    ] {
        let source: Notes = serde_json::from_str(&replacement).expect("valid JSON");
        assert!(validate(&source).is_err());
    }
}

#[test]
#[ignore = "requires CRYSTAL_CITY_NOTE_BASE_PACK: completed Minnesota city pack"]
fn city_notes_overlay_preserves_quests_world_audio_and_scales_basic_form_separately() {
    let input = PathBuf::from(
        std::env::var_os("CRYSTAL_CITY_NOTE_BASE_PACK").expect("explicit prerequisite"),
    );
    let base = crate::read_verified_compiled_game_pack(input).expect("verified parent");
    let pack = build_city_note_mimics(&base, SOURCE).expect("city overlay");
    assert_eq!(base.compiled_audio(), pack.compiled_audio());
    assert_eq!(base.runtime_files(), pack.runtime_files());
    assert_eq!(base.data.pokemon, pack.data.pokemon);
    assert_eq!(base.data.trainers, pack.data.trainers);
    assert_eq!(base.data.wild_encounters, pack.data.wild_encounters);
    assert_eq!(base.data.field_encounters, pack.data.field_encounters);
    assert_eq!(
        base.data.runtime_spawn_points,
        pack.data.runtime_spawn_points
    );
    for (name, map) in &base.data.maps {
        let after = &pack.data.maps[name];
        if !["MnMinneapolis", "MnStPaul", "MnDuluth"].contains(&name.as_str()) {
            assert_eq!(map, after, "unrelated map {name}");
            continue;
        }
        assert_eq!(map.blocks, after.blocks);
        assert_eq!(map.attributes, after.attributes);
        assert_eq!(map.events, after.events);
        assert_eq!(after.objects.len(), map.objects.len() + 8);
        assert_eq!(&map.objects, &after.objects[..map.objects.len()]);
        for (label, body) in &map.scripts {
            assert_eq!(body, &after.scripts[label]);
        }
        let mut state = GameState::default();
        state.storage.party.add_pokemon(
            pack.data
                .create_pokemon("POUCHID", 80, Dv::default())
                .expect("late party"),
        );
        state.sync_party_from_storage();
        for battle in after
            .scripted_wild_battles
            .iter()
            .filter(|b| b.source_script.starts_with("MnCityNote"))
        {
            let mut state = state.clone();
            let mut divider = ReplayDivider::new([0; 256]);
            let actual = pack
                .data
                .start_scripted_wild_battle(
                    &mut state,
                    name,
                    name,
                    &battle.source_script,
                    battle.startbattle_command_index,
                    &mut divider,
                )
                .expect("production wild entry");
            assert_eq!(
                actual.enemy_pokemon.species.id, battle.request.species,
                "level does not evolve wild species"
            );
            if battle.source_script.ends_with("Native") {
                assert_eq!((actual.species.as_str(), actual.level), ("JOTLET", 6));
            } else {
                assert!(actual.level >= 79, "distinct challenge slot scales");
            }
            assert!(pack.data.saved_static_wild_battle_origin_exists(
                name,
                &battle.source_script,
                battle.startbattle_command_index,
                battle.startbattle_command_index + 1,
                &actual.battle_type,
                &actual.species,
                actual.level
            ));
        }
    }
    assert!(build_city_note_mimics(&pack, SOURCE).is_err());
    let mut occupied: serde_json::Value = serde_json::from_str(SOURCE).expect("source");
    occupied["cities"][0]["notes"][0] = serde_json::json!([63, 25]);
    assert!(
        build_city_note_mimics(&base, &occupied.to_string()).is_err(),
        "cannot replace an entrance"
    );
}
