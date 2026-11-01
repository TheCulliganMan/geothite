use super::build_minnesota_water_cave_habitats;

#[test]
#[ignore = "requires CRYSTAL_WATER_CAVE_PARENT and CRYSTAL_SWEET_SCENT_SOURCE"]
fn water_caves_cover_native_surfaces_preserve_parent_and_reject_bad_contracts() {
    let base = crate::read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_WATER_CAVE_PARENT").expect("external water/cave parent"),
    )
    .expect("verified external parent");
    let source = include_str!(
        "../../../planning/minnesota/pokemon/acquisition-v01/water-caves-v01/habitats.json"
    );
    let native = std::fs::read_to_string(
        std::env::var_os("CRYSTAL_SWEET_SCENT_SOURCE").expect("external native Sweet Scent source"),
    )
    .expect("native source file");
    let child =
        build_minnesota_water_cave_habitats(&base, source, &native).expect("complete habitats");
    let before = serde_json::to_value(base.data()).expect("parent data");
    let mut after = serde_json::to_value(child.data()).expect("child data");
    for name in [
        "maps",
        "map_attributes",
        "map_blocks",
        "map_scripts",
        "wild_encounters",
        "story_events",
        "global_scripts",
    ] {
        after[name] = before[name].clone();
    }
    assert_eq!(
        before, after,
        "only bounded maps/headers and missing tables change"
    );
    // Only the nine pinned field-program definitions and one root are added.
    let mut stories = serde_json::to_value(&child.data.story_events).expect("new stories");
    let old_stories = serde_json::to_value(&base.data.story_events).expect("parent stories");
    for (after, before) in stories
        .as_array_mut()
        .expect("stories")
        .iter_mut()
        .zip(old_stories.as_array().expect("old stories"))
    {
        if let Some(catalog) = after
            .get_mut("StandardScripts")
            .and_then(serde_json::Value::as_object_mut)
        {
            let original = before["StandardScripts"]
                .as_object()
                .expect("original standard scripts");
            for label in crate::minnesota_sweet_scent_source::LABELS {
                if !original.contains_key(label) {
                    assert!(catalog.remove(label).is_some());
                }
            }
            let roots = catalog
                .get_mut("GlobalScriptRoots")
                .and_then(serde_json::Value::as_array_mut)
                .expect("roots");
            assert_eq!(
                roots.pop(),
                Some(serde_json::json!(".SweetScent@SweetScentFromMenu"))
            );
        }
    }
    assert_eq!(
        stories, old_stories,
        "all pre-existing source catalogs exact"
    );
    let original = serde_json::to_value(&base.data.global_scripts).expect("old global module");
    let current = serde_json::to_value(&child.data.global_scripts).expect("new global module");
    for (key, value) in original.as_object().expect("global fields") {
        match value {
            serde_json::Value::Object(entries) => {
                for (label, body) in entries {
                    assert_eq!(&current[key][label], body, "retained global {key}:{label}");
                }
            }
            serde_json::Value::Array(entries) => {
                let after = current[key].as_array().expect("global entries");
                for entry in entries {
                    assert!(after.contains(entry), "retained global {key}");
                }
            }
            _ => assert_eq!(&current[key], value),
        }
    }
    assert!(
        build_minnesota_water_cave_habitats(&base, source, &(native.clone() + "\n;changed"))
            .is_err(),
        "pinned native source preimage"
    );
    assert_eq!(base.runtime_files(), child.runtime_files());
    assert!(base.compiled_audio() == child.compiled_audio());
    for name in [
        "MnR12Cave1F",
        "MnR13Cave1F",
        "MnBoundaryIntake",
        "MnIronSump",
    ] {
        let table = &child.data.wild_encounters[name];
        assert!(table.grass.is_none(), "peaceful land surface {name}");
        assert!(
            table.grass_rates.is_none(),
            "absent land rate/table pair {name}"
        );
    }
    for (name, map) in &base.data.maps {
        let mut expected = map.clone();
        expected
            .attributes
            .fishing_group
            .clone_from(&child.data.maps[name].attributes.fishing_group);
        if name != "MnMoorhead" {
            assert_eq!(
                expected, child.data.maps[name],
                "unrelated complete map {name}"
            );
        } else {
            assert_eq!(map.objects, child.data.maps[name].objects);
            assert_eq!(map.events.warps, child.data.maps[name].events.warps);
            assert_eq!(
                map.attributes.connections,
                child.data.maps[name].attributes.connections
            );
        }
    }
    for (name, table) in &base.data.wild_encounters {
        let mut expected = table.clone();
        if name == "MnLeechLake" {
            assert_eq!(table.water_rate, Some(0));
            expected
                .water
                .clone_from(&child.data.wild_encounters[name].water);
            expected.water_rate = Some(5);
            assert!(!table.zones.is_empty(), "inherited island grass habitat");
        }
        assert_eq!(
            expected, child.data.wild_encounters[name],
            "existing active encounters and all island zones {name}"
        );
    }
    assert!(build_minnesota_water_cave_habitats(&child, source, &native).is_err());
    for (field, value) in [
        ("parent_content_hash", serde_json::json!("stale")),
        ("exceptions", serde_json::json!([])),
        ("fishing_repairs", serde_json::json!([])),
    ] {
        let mut bad: serde_json::Value = serde_json::from_str(source).expect("source");
        bad[field] = value;
        assert!(
            build_minnesota_water_cave_habitats(&base, &bad.to_string(), &native).is_err(),
            "{field}"
        );
    }
    for (field, value) in [
        ("species", serde_json::json!("MAGIKARP")),
        ("level", serde_json::json!(0)),
    ] {
        let mut bad: serde_json::Value = serde_json::from_str(source).expect("source");
        bad["maps"][0]["water"]["table"]["day"][0][field] = value;
        assert!(
            build_minnesota_water_cave_habitats(&base, &bad.to_string(), &native).is_err(),
            "{field}"
        );
    }
    let mut bad: serde_json::Value = serde_json::from_str(source).expect("source");
    bad["maps"][0]["water"]["sample"] = serde_json::json!([0, 0]);
    assert!(
        build_minnesota_water_cave_habitats(&base, &bad.to_string(), &native).is_err(),
        "wall habitat"
    );
    let mut bad: serde_json::Value = serde_json::from_str(source).expect("source");
    bad["moorhead_access"]["cells"][0]["after"] = serde_json::json!("35");
    assert!(
        build_minnesota_water_cave_habitats(&base, &bad.to_string(), &native).is_err(),
        "open river edge"
    );
}
