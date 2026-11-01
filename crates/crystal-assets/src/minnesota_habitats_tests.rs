use super::*;

#[test]
#[ignore = "requires CRYSTAL_HABITAT_BASE_PACK"]
fn habitats_preserve_native_world_and_reject_foreign_walls_and_missing_roots() {
    let base = crate::read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_HABITAT_BASE_PACK").expect("base pack"),
    )
    .expect("verified parent");
    let source = include_str!("../../../planning/minnesota/pokemon/acquisition-v01/habitats.json");
    let result = build_minnesota_habitats(&base, source).expect("authored habitats");
    for (name, before) in &base.data.maps {
        let after = &result.data.maps[name];
        let mut expected = before.clone();
        expected
            .attributes
            .fishing_group
            .clone_from(&after.attributes.fishing_group);
        assert_eq!(
            &expected, after,
            "only local fishing headers may change: {name}"
        );
    }
    assert_eq!(base.data.trainers, result.data.trainers);
    for (id, group) in &base.data.fishing.groups {
        assert_eq!(result.data.fishing.groups.get(id), Some(group));
    }
    assert_eq!(
        base.data.fishing.time_groups,
        result.data.fishing.time_groups
    );
    for (id, rule) in &base.data.fishing.swarm_rules {
        assert_eq!(result.data.fishing.swarm_rules.get(id), Some(rule));
    }
    assert_eq!(base.data.field_encounters, result.data.field_encounters);
    assert_eq!(base.data.pokemon, result.data.pokemon);
    assert_eq!(base.runtime_files(), result.runtime_files());
    assert!(base.compiled_audio() == result.compiled_audio());
    for (name, before) in &base.data.wild_encounters {
        let after = &result.data.wild_encounters[name];
        assert_eq!(before.grass, after.grass, "fallback grass {name}");
        assert_eq!(before.grass_rates, after.grass_rates);
        assert_eq!(before.water_rate, after.water_rate);
        assert_eq!(before.swarm_overrides, after.swarm_overrides);
        assert!(after.zones.starts_with(&before.zones));
        assert!(species(before).is_subset(&species(after)));
    }
    assert!(build_minnesota_habitats(&result, source).is_err());
    for bad in 0..4 {
        let mut value: Value = serde_json::from_str(source).expect("source");
        match bad {
            0 => value["grass"][0]["residents"]["morning"] = serde_json::json!(["PIKACHU"]),
            1 => value["grass"][0]["rect"] = serde_json::json!([0, 0, 1, 1]),
            2 => {
                value["grass"][1]["map"] = value["grass"][0]["map"].clone();
                value["grass"][1]["rect"] = value["grass"][0]["rect"].clone();
                value["grass"][1]["anchor"] = value["grass"][0]["anchor"].clone();
            }
            _ => {
                value["grass"][0]["residents"]["morning"] = serde_json::json!(["DRIPTIP"]);
                value["grass"][0]["residents"]["night"] = serde_json::json!(["DRIPTIP"]);
            }
        }
        assert!(
            build_minnesota_habitats(&base, &value.to_string()).is_err(),
            "mutation {bad}"
        );
    }
}
