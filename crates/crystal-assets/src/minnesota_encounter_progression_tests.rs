use super::{Group, build_minnesota_encounter_progression, validate_group};

#[test]
fn progression_rejects_opening_levels_and_wrong_badge_gate_in_grand_marais() {
    let mut group: Group = serde_json::from_value(serde_json::json!({
        "id":"north-shore-town", "minimum_badges":8, "access":"Harbor signal",
        "maps":["MnGrandMarais"], "grass":[39,40,41,40,41,42,43], "surf":[39,41,43]
    }))
    .expect("authored town group");
    validate_group(&group).expect("late town band");
    group.grass[0] = 3;
    assert!(validate_group(&group).is_err());
    group.grass = [2, 2, 3, 3, 3, 4, 4];
    group.surf = [20, 22, 24];
    group.minimum_badges = 0;
    assert!(
        validate_group(&group).is_err(),
        "changing the stage cannot hide the wrong gate"
    );
}

#[test]
#[ignore = "requires CRYSTAL_PROGRESSION_PARENT and CRYSTAL_PROGRESSION_SOURCE"]
fn progression_complete_habitats_preservation_and_rejected_incomplete_plan() {
    let base = crate::read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_PROGRESSION_PARENT").expect("external reviewed ecology parent"),
    )
    .expect("parent pack");
    let source = std::fs::read_to_string(
        std::env::var_os("CRYSTAL_PROGRESSION_SOURCE").expect("authored progression plan"),
    )
    .expect("progression JSON");
    let child =
        build_minnesota_encounter_progression(&base, &source).expect("full encounter progression");
    let before = serde_json::to_value(base.data()).expect("parent data");
    let mut after = serde_json::to_value(child.data()).expect("child data");
    for key in ["wild_encounters", "field_encounters", "trainers"] {
        after[key] = before[key].clone();
    }
    assert_eq!(
        before, after,
        "all geography, programs, gifts, fishing, assets and catalog exact"
    );
    for (id, old) in &base.data.trainers.trainers {
        let new = &child.data.trainers.trainers[id];
        if !id.starts_with("MN_") {
            assert_eq!(old, new);
        }
        assert_eq!(
            (&old.name, &old.trainer_class, old.party.len()),
            (&new.name, &new.trainer_class, new.party.len())
        );
        for (a, b) in old.party.iter().zip(&new.party) {
            assert!(b.level >= a.level);
            assert_eq!((&a.item, a.dvs), (&b.item, b.dvs));
        }
    }
    assert_eq!(base.runtime_files(), child.runtime_files());
    assert!(base.compiled_audio() == child.compiled_audio());
    for (name, old) in &base.data.wild_encounters {
        let new = &child.data.wild_encounters[name];
        if !name.starts_with("Mn") {
            assert_eq!(old, new, "original region untouched: {name}");
        }
        assert_eq!(old.grass_rates, new.grass_rates);
        assert_eq!(old.water_rate, new.water_rate);
        assert_eq!(
            old.zones
                .iter()
                .map(|z| (&z.id, [z.min_x, z.min_y, z.max_x, z.max_y], &z.grass_rates))
                .collect::<Vec<_>>(),
            new.zones[..old.zones.len()]
                .iter()
                .map(|z| (&z.id, [z.min_x, z.min_y, z.max_x, z.max_y], &z.grass_rates))
                .collect::<Vec<_>>()
        );
    }
    let town = child.data.wild_encounters["MnGrandMarais"]
        .grass
        .as_ref()
        .expect("late town grass");
    assert!(
        town.morning
            .iter()
            .chain(&town.day)
            .chain(&town.night)
            .all(|s| (39..=43).contains(&s.level))
    );
    assert!(town.day.iter().any(|s| s.species == "GOUGECREST"));
    let pines = child.data.wild_encounters["MnR11C00R00"]
        .grass
        .as_ref()
        .expect("pine grass");
    let basalt = child.data.wild_encounters["MnR11C00R01"]
        .grass
        .as_ref()
        .expect("basalt grass");
    let coast = child.data.wild_encounters["MnR11C00R02"]
        .grass
        .as_ref()
        .expect("coastal grass");
    for (a, b, c) in [
        (&pines.morning, &basalt.morning, &coast.morning),
        (&pines.day, &basalt.day, &coast.day),
        (&pines.night, &basalt.night, &coast.night),
    ] {
        assert_eq!(a[1].species, "LATCHQUILL");
        assert_eq!(a[4].species, "PITCHWICK");
        assert_eq!(b[1].species, "BANDRAKE");
        assert_eq!(b[2].species, "AGATOT");
        assert_eq!(a[0], c[0], "shared common resident remains");
        assert_eq!(&a[5..], &c[5..], "rarities retain their slots");
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_eq!(
            a.iter().map(|s| s.level).collect::<Vec<_>>(),
            c.iter().map(|s| s.level).collect::<Vec<_>>()
        );
    }
    let young = child.data.wild_encounters["MnGrandMarais"]
        .zones
        .iter()
        .find(|z| z.id == "young-cedar-margin")
        .expect("repeatable western cedar pocket");
    assert_eq!(
        [young.min_x, young.min_y, young.max_x, young.max_y],
        [4, 10, 5, 13]
    );
    assert!(young.grass.day.iter().all(|s| (3..=7).contains(&s.level)));
    assert!(young.grass.night.iter().any(|s| s.species == "PINIT"));
    let data = child.data();
    let mut state = crystal_core::state::GameState::default();
    state.storage.party.add_pokemon(
        data.create_pokemon("POUCHID", 80, Default::default())
            .expect("late party"),
    );
    state.sync_party_from_storage();
    state.badges.johto = [true; 8];
    for flag in &crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[..8] {
        state
            .flags
            .set_engine_flag(flag, true)
            .expect("consistent late-game badge flags");
    }
    for slot in young
        .grass
        .morning
        .iter()
        .chain(&young.grass.day)
        .chain(&young.grass.night)
    {
        let native = data.ordinary_challenge_level(&state, "MnGrandMarais", slot.level, 2, 0);
        let scaled = data.ordinary_challenge_level(&state, "MnGrandMarais", slot.level, 2, 1);
        assert_eq!(native, slot.level, "eight badges retain genuine low levels");
        assert!(scaled >= 77, "the challenge component remains available");
        for level in [native, scaled] {
            let mon = data
                .create_pokemon(&slot.species, level, Default::default())
                .expect("resident");
            assert_eq!(
                mon.species.id, slot.species,
                "scaling never forces evolution"
            );
            assert!(data.saved_wild_encounter_exists("MnGrandMarais", &slot.species, level));
        }
    }
    let opening = child.data.wild_encounters["MnR01C00R00"]
        .grass
        .as_ref()
        .expect("opening woods");
    assert!(
        opening
            .day
            .iter()
            .any(|s| s.species == "PINIT" && s.level == 4)
    );
    assert!(
        opening
            .morning
            .iter()
            .any(|s| s.species == "PODLET" && s.level == 3)
    );
    let farm = &child.data.wild_encounters["MnR02C00R00"];
    assert!(
        farm.zones
            .iter()
            .flat_map(|z| z.grass.night.iter())
            .all(|s| s.species != "CROWSCARE"),
        "the 510BST scarecrow first appears on the later prairie loop"
    );
    let fair = &child.data.wild_encounters["MnStateFairgrounds"];
    let fair_species = fair
        .grass
        .iter()
        .chain(fair.zones.iter().map(|z| &z.grass))
        .flat_map(|t| t.morning.iter().chain(&t.day).chain(&t.night))
        .map(|s| s.species.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        !fair_species.contains("PAILBARON"),
        "level38 final form is earned from the available cookie family"
    );
    assert!(fair_species.contains("CURDLET") && fair_species.contains("SQUEAKURD"));
    let mut invalid: serde_json::Value = serde_json::from_str(&source).expect("plan JSON");
    let grand = invalid["groups"]
        .as_array_mut()
        .expect("groups")
        .iter_mut()
        .find(|g| g["id"] == "grand-marais")
        .expect("town group");
    grand["young_grass_zones"][0]["zone"]["minX"] = 34.into();
    grand["young_grass_zones"][0]["zone"]["maxX"] = 35.into();
    assert!(
        build_minnesota_encounter_progression(&base, &invalid.to_string()).is_err(),
        "a zone painted over inaccessible ground cannot claim a repeatable habitat"
    );
    let mut plan: serde_json::Value = serde_json::from_str(&source).expect("plan JSON");
    plan["groups"].as_array_mut().expect("groups").pop();
    assert!(
        build_minnesota_encounter_progression(&base, &plan.to_string()).is_err(),
        "no habitat can silently keep stale levels"
    );
}
