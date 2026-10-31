use super::build_minnesota_biome_ecology;

#[test]
#[ignore = "requires CRYSTAL_ECOLOGY_PARENT and CRYSTAL_ECOLOGY_SOURCE"]
fn biome_ecology_preserves_native_content_and_rejects_bad_plans() {
    let base = crate::read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_ECOLOGY_PARENT").expect("external ecology parent"),
    )
    .expect("verified ecology parent");
    let source = std::fs::read_to_string(
        std::env::var_os("CRYSTAL_ECOLOGY_SOURCE").expect("authored ecology source"),
    )
    .expect("ecology plan");
    let plan: serde_json::Value = serde_json::from_str(&source).expect("source JSON");
    let child = build_minnesota_biome_ecology(&base, &source).expect("complete biome corrections");
    let before = serde_json::to_value(base.data()).expect("parent data");
    let mut after = serde_json::to_value(child.data()).expect("child data");
    assert!(
        after["story_event_script_constants"]["global"]
            .as_object_mut()
            .expect("event constants")
            .remove("EVENT_MN_PIPULSE_COMPANION_RECEIVED")
            .is_some()
    );
    for field in [
        "maps",
        "map_attributes",
        "wild_encounters",
        "field_encounters",
        "fishing",
        "map_scripts",
    ] {
        after[field] = before[field].clone();
    }
    assert_eq!(
        before, after,
        "all unrelated content, programs and assets exact"
    );
    assert_eq!(base.runtime_files(), child.runtime_files());
    assert!(base.compiled_audio() == child.compiled_audio());
    let mut changed_headers = 0;
    for (name, old) in &base.data.maps {
        if name == "MnRochesterPatientHouse" {
            let new = &child.data.maps[name];
            assert_eq!(old.attributes, new.attributes);
            assert_eq!(old.blocks, new.blocks);
            assert_eq!(old.objects, new.objects);
            assert_eq!(old.events, new.events);
            for (label, body) in &old.scripts {
                if label != "MN_SMALL_INTERIOR_ROCHESTER_PATIENT_FRIEND" {
                    assert_eq!(
                        body, &new.scripts[label],
                        "original rest-house interaction {label}"
                    );
                }
            }
            assert!(
                new.gift_pokemon_scripts
                    .iter()
                    .any(|g| g.species_id == "PIPULSE")
            );
            continue;
        }
        let mut expected = old.clone();
        if expected.attributes.fishing_group != child.data.maps[name].attributes.fishing_group {
            changed_headers += 1;
        }
        expected
            .attributes
            .fishing_group
            .clone_from(&child.data.maps[name].attributes.fishing_group);
        for label in ["MN_R13_BIRD_WATCHERText", "MN_R17_OLD_PINES_WATCHERText"] {
            if expected.scripts.contains_key(label) {
                expected
                    .scripts
                    .insert(label.into(), child.data.maps[name].scripts[label].clone());
                expected.script_text_bodies.insert(
                    label.into(),
                    child.data.maps[name].script_text_bodies[label].clone(),
                );
            }
        }
        assert_eq!(
            expected, child.data.maps[name],
            "complete native map retained: {name}"
        );
    }
    assert_eq!(changed_headers, 46);
    for (name, old) in &base.data.wild_encounters {
        let new = &child.data.wild_encounters[name];
        let mut zones = old.zones.clone();
        for edit in plan["habitat_review"]["relocations"]
            .as_array()
            .expect("reviewed habitats")
        {
            if edit["map"] != name.as_str() {
                continue;
            }
            let Some(id) = edit["zone"].as_str() else {
                continue;
            };
            let table = &mut zones
                .iter_mut()
                .find(|z| z.id == id)
                .expect("reviewed zone")
                .grass;
            for slot in table
                .morning
                .iter_mut()
                .chain(&mut table.day)
                .chain(&mut table.night)
            {
                if slot.species == edit["before"].as_str().expect("preimage") {
                    slot.species = edit["after"].as_str().expect("local resident").into();
                }
            }
        }
        assert_eq!(
            zones, new.zones,
            "only reviewed zone identities change: {name}"
        );
        assert_eq!(old.swarm_overrides, new.swarm_overrides);
        assert_eq!(old.grass_rates, new.grass_rates);
        if name == "MnR02C00R00" {
            assert!(new.water.is_none() && new.water_rate.is_none());
            assert!(new.grass.is_some(), "farm grass remains populated");
            assert_eq!(
                child.data.maps[name].attributes.fishing_group.as_deref(),
                Some("FISHGROUP_NONE")
            );
        } else {
            assert_eq!(old.water_rate, new.water_rate);
        }
        if !name.starts_with("Mn") {
            assert_eq!(old, new);
        }
    }
    for (name, group) in &base.data.fishing.groups {
        assert_eq!(
            group, &child.data.fishing.groups[name],
            "original rod catalog {name}"
        );
    }
    for (name, group) in &base.data.fishing.time_groups {
        assert_eq!(group, &child.data.fishing.time_groups[name]);
    }
    for (name, rule) in &base.data.fishing.swarm_rules {
        assert_eq!(rule, &child.data.fishing.swarm_rules[name]);
    }
    for (name, map) in &child.data.maps {
        let Some(id) = map
            .attributes
            .fishing_group
            .as_ref()
            .filter(|s| s.starts_with("FISHGROUP_MN_ECO_"))
        else {
            continue;
        };
        let original_id = base.data.maps[name]
            .attributes
            .fishing_group
            .as_ref()
            .expect("original rod header");
        let old = &base.data.fishing.groups[original_id];
        let new = &child.data.fishing.groups[id];
        assert_eq!(old.rod_tables["OLD_ROD"], new.rod_tables["OLD_ROD"]);
        assert_eq!(old.bite_threshold, new.bite_threshold);
        for (rod, table) in &old.rod_tables {
            for (a, b) in table.slots.iter().zip(&new.rod_tables[rod].slots) {
                assert_eq!(
                    (a.threshold, a.level, a.time_group.is_some()),
                    (b.threshold, b.level, b.time_group.is_some())
                );
            }
        }
        let old_rule = base
            .data
            .fishing
            .swarm_rules
            .values()
            .find(|r| &r.base_group == original_id);
        let new_rule = child
            .data
            .fishing
            .swarm_rules
            .values()
            .find(|r| &r.base_group == id);
        assert_eq!(
            old_rule.map(|r| (r.daily_flag_bit, r.swarm)),
            new_rule.map(|r| (r.daily_flag_bit, r.swarm))
        );
        if let Some(rule) = new_rule {
            let state = crystal_core::state::GameState {
                fishing: crystal_core::state::FishingMemory {
                    daily_flags1: 1 << rule.daily_flag_bit,
                    swarm_flag: rule.swarm,
                    ..Default::default()
                },
                ..Default::default()
            };
            assert_eq!(
                crystal_core::world::fishing::resolve_group_token(
                    Some(id),
                    Some(&state),
                    Some(&child.data.fishing)
                ),
                Some(rule.swarm_group.clone()),
                "private header must select its native active swarm",
            );
            assert_eq!(
                crystal_core::world::fishing::resolve_group_token(
                    Some(id),
                    Some(&Default::default()),
                    Some(&child.data.fishing)
                ),
                Some(id.clone()),
                "ordinary private header remains active outside a swarm",
            );
        }
    }
    let farm = &child.data.wild_encounters["MnR02C00R00"];
    assert!(
        farm.grass
            .as_ref()
            .expect("farm grass")
            .day
            .iter()
            .any(|s| s.species == "PIPBOOM")
    );
    assert!(
        !farm
            .grass
            .as_ref()
            .expect("farm grass")
            .morning
            .iter()
            .chain(&farm.grass.as_ref().expect("farm grass").day)
            .chain(&farm.grass.as_ref().expect("farm grass").night)
            .any(|s| matches!(s.species.as_str(), "PINIT" | "PECKIT")),
        "open prairie farm has prairie species, not conifer/woodpecker residents"
    );
    assert!(
        !farm
            .grass
            .as_ref()
            .expect("farm grass")
            .day
            .iter()
            .any(|s| s.species == "JOTLET")
    );
    assert!(
        child.data.wild_encounters["MnR04C00R00"]
            .grass
            .as_ref()
            .expect("city park")
            .day
            .iter()
            .any(|s| s.species == "JOTLET")
    );
    let forest = &child.data.wild_encounters["MnR17C00R00"]
        .grass
        .as_ref()
        .expect("pine forest");
    assert!(forest.day.iter().any(|s| s.species == "NOTCHBILL"));
    assert!(
        !forest
            .morning
            .iter()
            .chain(&forest.day)
            .chain(&forest.night)
            .any(|s| s.species == "BOOMRUFF")
    );
    let coast = &child.data.wild_encounters["MnR11C00R02"]
        .water
        .as_ref()
        .expect("lighthouse coast");
    assert!(
        coast
            .night
            .iter()
            .any(|s| s.species == "WRECKQUIEM" && s.level == 41)
    );
    assert!(!coast.day.iter().any(|s| s.species == "WRECKQUIEM"));
    assert!(
        !child.data.wild_encounters["MnR12C00R01"]
            .water
            .as_ref()
            .expect("inland lake")
            .night
            .iter()
            .any(|s| s.species == "WRECKQUIEM")
    );
    let oak = &child.data.field_encounters["MnR19C00R00"].tables["headbutt"];
    assert!(
        !oak.common
            .iter()
            .chain(&oak.rare)
            .any(|s| matches!(s.species.as_str(), "PIPBOOM" | "CLINKLAW"))
    );
    assert!(oak.rare.iter().any(|s| s.species == "WHIRLNUT"));
    for (field, value) in [
        ("parent_content_hash", serde_json::json!("stale")),
        ("grass", serde_json::json!([])),
        ("fishing", serde_json::json!([])),
        ("retired_water", serde_json::json!([])),
    ] {
        let mut bad = plan.clone();
        bad[field] = value;
        assert!(
            build_minnesota_biome_ecology(&base, &bad.to_string()).is_err(),
            "{field}"
        );
    }
    let mut bad = plan.clone();
    bad["retired_water"][0]["water_cells"] = serde_json::json!(151);
    assert!(build_minnesota_biome_ecology(&base, &bad.to_string()).is_err());
    for (field, value) in [
        ("before", serde_json::json!("wrong")),
        ("after", serde_json::json!("MAGIKARP")),
        ("time", serde_json::json!("dawn")),
        ("level", serde_json::json!(0)),
    ] {
        let mut bad = plan.clone();
        bad["grass"][0]["edits"][0][field] = value;
        assert!(
            build_minnesota_biome_ecology(&base, &bad.to_string()).is_err(),
            "{field}"
        );
    }
    assert!(
        build_minnesota_biome_ecology(&child, &source).is_err(),
        "duplicate publication"
    );
}
