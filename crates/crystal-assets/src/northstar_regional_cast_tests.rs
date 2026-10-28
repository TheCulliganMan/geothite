//! Regional appearances must be complete without altering the source regions.
use std::collections::BTreeSet;
use std::path::PathBuf;

use serde_json::Value;

use crate::northstar_regional_roster::RegionalRoster;

fn source(name: &str) -> String {
    std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota")
            .join(name),
    )
    .expect("canonical regional source")
}

fn inputs() -> [String; 4] {
    [
        source("story-pokemon.json"),
        source("pokemon/roster.json"),
        source("pokemon/roster-curation.json"),
        source("pokemon/species-profiles.json"),
    ]
}

#[test]
fn regional_story_rejects_stale_curation_and_foreign_bindings() {
    let [policy, roster, curation, profiles] = inputs();
    RegionalRoster::parse(&policy, &roster, &curation, &profiles).expect("active roster");
    let mut invalid: Value = serde_json::from_str(&curation).expect("curation");
    invalid["active_species_count"] =
        Value::from(invalid["active_species_count"].as_u64().expect("count") + 1);
    assert!(RegionalRoster::parse(&policy, &roster, &invalid.to_string(), &profiles).is_err());
    let mut invalid: Value = serde_json::from_str(&policy).expect("policy");
    invalid["legacy_species_bindings"]["ABRA"] = "PIKACHU".into();
    assert!(RegionalRoster::parse(&invalid.to_string(), &roster, &curation, &profiles).is_err());
}

#[test]
#[ignore = "requires the external ICE story pack; audits all data and audio"]
fn regional_story_preserves_source_regions_audio_and_native_party_contracts() {
    let input = std::env::var_os("CRYSTAL_REGIONAL_CAST_PARENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/minnesota-northstar-ice-lair-v4.crystalpack")
        })
        .canonicalize()
        .expect("external parent");
    let base = crate::read_verified_compiled_game_pack(input).expect("verified parent");
    let [policy, roster, curation, profiles] = inputs();
    let regional = RegionalRoster::parse(&policy, &roster, &curation, &profiles).expect("roster");
    let result =
        crate::build_northstar_regional_cast(&base, &policy, &roster, &curation, &profiles)
            .expect("complete regional binding and appearance audit");
    assert!(base.compiled_audio() == result.compiled_audio());
    assert!(base.runtime_files() == result.runtime_files());
    assert!(base.audio_manifest().expect("audio") == result.audio_manifest().expect("audio"));
    let before = serde_json::to_value(base.data()).expect("parent data");
    let after = serde_json::to_value(result.data()).expect("regional data");
    let changed = BTreeSet::from([
        "maps",
        "map_scripts",
        "npcs",
        "map_attributes",
        "map_dimensions",
        "map_blocks",
        "runtime_map_metadata",
        "wild_encounters",
        "field_encounters",
        "fishing",
        "trainers",
        "runtime_title_screen",
    ]);
    for (field, value) in before.as_object().expect("data") {
        if !changed.contains(field.as_str()) {
            assert!(value == &after[field], "unrelated dataset {field}");
        }
    }
    let mut regional_labels = BTreeSet::new();
    for (id, map) in &base.data().maps {
        if id.starts_with("Mn") {
            regional_labels.extend(map.scripts.keys().map(String::as_str));
            let installed = &result.data().maps[id];
            assert_eq!(
                installed.script_audio_commands,
                crate::parse_script_audio_commands(id, &installed.scripts)
                    .expect("native audio indices"),
                "regional pictures/cries must use the actual new script species {id}"
            );
        } else {
            assert!(before["maps"][id] == after["maps"][id], "source map {id}");
            for field in ["npcs", "map_attributes", "map_dimensions", "map_blocks"] {
                assert!(before[field][id] == after[field][id], "source {field}/{id}");
            }
        }
    }
    for (id, value) in before["map_scripts"].as_object().expect("scripts") {
        if !regional_labels.contains(id.as_str()) {
            assert!(value == &after["map_scripts"][id], "source script {id}");
        }
    }
    for (id, trainer) in &base.data().trainers.trainers {
        let replacement = &result.data().trainers.trainers[id];
        if !id.starts_with("MN_") {
            assert!(before["trainers"]["trainers"][id] == after["trainers"]["trainers"][id]);
            continue;
        }
        let authored: Value = serde_json::from_str(&policy).expect("authored team policy");
        if let Some(team) = authored["faction_teams"].get(id) {
            let team = team.as_array().expect("authored party");
            assert_eq!(
                replacement.party.len(),
                team.len(),
                "authored faction party {id}"
            );
            for (member, intended) in replacement.party.iter().zip(team) {
                assert_eq!(
                    member.species,
                    intended["species"].as_str().expect("species")
                );
                assert_eq!(
                    u64::from(member.level),
                    intended["level"].as_u64().expect("level")
                );
                assert!(regional.allowed.contains(&member.species));
                let native = &result.data().learnsets[&member.species];
                let machines = &result.data().pokemon[&member.species].tmhm_learnset;
                assert!(
                    member
                        .moves
                        .iter()
                        .any(|m| result.data().moves[&m.name].power > 0),
                    "every faction member needs an actual damaging move"
                );
                for m in &member.moves {
                    assert!(
                        native
                            .iter()
                            .any(|entry| entry.0 <= member.level && entry.1 == m.name)
                            || machines.contains(&m.name),
                        "native legal move {id}/{}",
                        m.name
                    );
                    assert_eq!(m.current_pp, result.data().moves[&m.name].pp);
                }
            }
            continue;
        }
        assert_eq!(
            trainer.party.len(),
            replacement.party.len(),
            "party size {id}"
        );
        for (old, new) in trainer.party.iter().zip(&replacement.party) {
            assert_eq!(old.level, new.level, "level {id}");
            assert_eq!(old.item, new.item, "held item {id}");
            assert_eq!(old.dvs, new.dvs, "DVs {id}");
            assert!(
                regional.allowed.contains(&new.species),
                "regional team {id}"
            );
            assert_eq!(
                new.moves,
                result
                    .data()
                    .create_pokemon(&new.species, new.level, new.dvs)
                    .expect("actual learned moves")
                    .moves,
                "native learnset {id}",
            );
        }
    }
    for field in ["wild_encounters", "field_encounters"] {
        for (id, value) in before[field].as_object().expect("encounters") {
            if !id.starts_with("Mn") {
                assert!(value == &after[field][id], "source encounter {id}");
            }
        }
    }
    for field in ["groups", "time_groups", "swarm_rules"] {
        for (id, value) in before["fishing"][field].as_object().expect("fishing") {
            assert!(
                value == &after["fishing"][field][id],
                "source fishing {field}/{id}"
            );
        }
    }
    assert_eq!(
        regional
            .target(&base, "FURRET", Some(20))
            .expect("legal family"),
        "CROOKIT"
    );
    assert_eq!(
        regional
            .target(&base, "FURRET", Some(30))
            .expect("legal evolution"),
        "CROOKVEIL"
    );
    assert!(
        regional.target(&base, "PIKACHU", Some(5)).is_err(),
        "no legacy fallback"
    );
    assert_eq!(
        regional
            .target(&base, "MOSQUITAN", Some(54))
            .expect("legal earlier stage"),
        "PUPUMP"
    );
    assert_eq!(
        regional
            .target(&base, "MOSQUITAN", Some(55))
            .expect("legal final stage"),
        "MOSQUITAN"
    );
    let mut invalid: Value = serde_json::from_str(&policy).expect("policy");
    invalid["faction_teams"]["MN_ICE_ENFORCER"][0]["moves"] = serde_json::json!(["FISSURE"]);
    assert!(
        crate::northstar_faction_teams::install(&mut base.clone(), &regional, &invalid).is_err(),
        "coverage cannot invent an illegal move"
    );
    invalid = serde_json::from_str(&policy).expect("policy");
    invalid["faction_teams"]["MN_ICE_ENFORCER"][0]["species"] = "MOSQUITAN".into();
    assert!(
        crate::northstar_faction_teams::install(&mut base.clone(), &regional, &invalid).is_err(),
        "a boss cannot use an underlevel final mosquito"
    );
    let mut missing = base.clone();
    missing.data.pokemon.remove("GILLPUP");
    assert!(
        regional.target(&missing, "GILLPUP", Some(5)).is_err(),
        "no missing-species fallback"
    );
    assert!(
        crate::build_northstar_regional_cast(&result, &policy, &roster, &curation, &profiles,)
            .is_err(),
        "do not clone fishing groups or bind an installed cast twice"
    );
}
