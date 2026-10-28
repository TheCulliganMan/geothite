//! Publication checks against completed external city packs, never copied assets.
use std::path::PathBuf;

use crystal_assets::{build_minnesota_city_life_modpack, read_verified_compiled_game_pack};
use serde_json::Value;

fn has_grass_species(value: &Value, species: &str) -> bool {
    match value {
        Value::Object(fields) => {
            fields.get("species").and_then(Value::as_str) == Some(species)
                || fields
                    .values()
                    .any(|value| has_grass_species(value, species))
        }
        Value::Array(values) => values.iter().any(|value| has_grass_species(value, species)),
        _ => false,
    }
}

#[test]
#[ignore = "requires CRYSTAL_CITY_LIFE_BASE and CRYSTAL_CITY_LIFE_PACK external packs"]
fn city_life_preserves_content_and_installs_every_resident_and_trade() {
    let base = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_CITY_LIFE_BASE").expect("population parent"),
    ))
    .expect("verified parent");
    let pack = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_CITY_LIFE_PACK").expect("published population"),
    ))
    .expect("verified population");
    let source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/city-life.json"),
    )
    .expect("population source");
    let contract: Value = serde_json::from_str(&source).expect("population contract");
    let before = base.data();
    let after = pack.data();
    assert_eq!(before.maps.len(), after.maps.len());
    assert_eq!(base.runtime_files(), pack.runtime_files());
    assert_eq!(base.compiled_audio(), pack.compiled_audio());
    assert_eq!(before.pokemon, after.pokemon);
    assert_eq!(before.moves, after.moves);
    assert_eq!(before.items, after.items);
    assert_eq!(before.wild_encounters, after.wild_encounters);
    assert_eq!(before.trainers, after.trainers);
    assert_eq!(before.runtime_title_screen, after.runtime_title_screen);
    assert_eq!(before.runtime_spawn_points, after.runtime_spawn_points);
    assert_eq!(before.tilesets, after.tilesets);
    let mut count = 0;
    for (name, old) in &before.maps {
        let Some(city) = contract["cities"]
            .as_array()
            .expect("cities")
            .iter()
            .find(|c| c["map"] == *name)
        else {
            assert_eq!(old, &after.maps[name], "unrelated map {name}");
            continue;
        };
        count += 1;
        let new = &after.maps[name];
        let cast = city["actors"].as_array().expect("local cast");
        assert_eq!(new.objects.len(), old.objects.len() + cast.len());
        assert_eq!(&new.objects[..old.objects.len()], old.objects);
        assert_eq!(old.blocks, new.blocks);
        assert_eq!(old.attributes, new.attributes);
        assert_eq!(old.events, new.events);
        for (label, script) in &old.scripts {
            assert_eq!(Some(script), new.scripts.get(label));
        }
        for (label, text) in &old.script_text_bodies {
            assert_eq!(Some(text), new.script_text_bodies.get(label));
        }
        // Remove just the new actor scripts and records. All pre-existing
        // command tables, event ordering and story metadata must be identical.
        let mut retained = new.clone();
        retained.objects.truncate(old.objects.len());
        retained
            .scripts
            .retain(|label, _| old.scripts.contains_key(label));
        retained
            .script_text_bodies
            .retain(|label, _| old.script_text_bodies.contains_key(label));
        retained
            .script_text_commands
            .truncate(old.script_text_commands.len());
        retained
            .script_object_commands
            .truncate(old.script_object_commands.len());
        retained
            .script_control_commands
            .truncate(old.script_control_commands.len());
        retained
            .script_runtime_commands
            .truncate(old.script_runtime_commands.len());
        retained.map_event_section_commands = old.map_event_section_commands.clone();
        assert_eq!(&retained, old, "original compiled programs {name}");
        let requested = city["trade"]["requested"].as_str().expect("request");
        let encounters = serde_json::to_value(&before.wild_encounters).expect("encounter view");
        assert!(
            encounters
                .as_object()
                .expect("encounter maps")
                .iter()
                .any(|(map, value)| {
                    map.starts_with("Mn")
                        && (has_grass_species(&value["grass"], requested)
                            || value["zones"].as_array().is_some_and(|zones| {
                                zones
                                    .iter()
                                    .any(|zone| has_grass_species(&zone["grass"], requested))
                            }))
                }),
            "requested {requested} needs an actual regional grass slot"
        );
        let trade_id = format!("NPC_TRADE_{}_LOCAL", name.to_ascii_uppercase());
        let rule = &after.npc_trades[&trade_id];
        assert_eq!(
            rule.requested_species,
            city["trade"]["requested"].as_str().expect("request")
        );
        assert_eq!(
            rule.offered_species,
            city["trade"]["offered"].as_str().expect("offer")
        );
        let trades = new
            .script_runtime_commands
            .iter()
            .filter(|c| c.command == "trade")
            .collect::<Vec<_>>();
        assert_eq!(trades.len(), 1, "exactly one trader in {name}");
        assert_eq!(trades[0].args, vec![trade_id]);
        for actor in cast {
            let id = actor["id"].as_str().expect("resident id");
            assert!(new.objects.iter().any(|o| o.script == id));
            assert!(new.script_text_bodies.contains_key(&format!("{id}Text")));
        }
    }
    assert!(count >= 14, "all existing Minnesota towns covered");
    assert_eq!(after.npc_trades.len(), before.npc_trades.len() + count);
    for (id, rule) in &before.npc_trades {
        assert_eq!(Some(rule), after.npc_trades.get(id));
    }
    let rebuilt = build_minnesota_city_life_modpack(&base, &source).expect("repeatable population");
    assert_eq!(
        pack.identity().expect("identity"),
        rebuilt.identity().expect("rebuilt identity")
    );
    assert!(
        build_minnesota_city_life_modpack(&pack, &source).is_err(),
        "no duplicate traders"
    );
    let mut invalid = contract.clone();
    invalid["cities"][0]["placements"][0]["tiles"][0] = serde_json::json!([0, 0]);
    assert!(
        build_minnesota_city_life_modpack(&base, &invalid.to_string()).is_err(),
        "blocked actor rejected"
    );
    invalid = contract.clone();
    invalid["cities"].as_array_mut().expect("cities").pop();
    assert!(
        build_minnesota_city_life_modpack(&base, &invalid.to_string()).is_err(),
        "missing city rejected"
    );
}
