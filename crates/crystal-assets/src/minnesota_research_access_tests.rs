use super::build_minnesota_research_access;

#[test]
#[ignore = "requires the original Fair Table pack in MN_STARTER_BENCH_PARENT"]
fn winona_benches_follow_each_starter_choice_and_adoption() -> anyhow::Result<()> {
    use crystal_core::state::GameState;
    use crystal_core::world::map::TilePosition;

    use crate::grand_marais_modpack::authoring::command;

    const STATION: &str = "MnWinonaFieldStation";
    let mut pack = crate::read_verified_compiled_game_pack(
        std::env::var_os("MN_STARTER_BENCH_PARENT").expect("external Fair Table pack"),
    )?;
    let before = pack.data.maps[STATION].clone();
    super::refresh_adopted_starter_benches(&mut pack)?;
    let room = &pack.data.maps[STATION];
    let starters = [
        ("POUCHID", "CHIKORITA"),
        ("KINDLIT", "CYNDAQUIL"),
        ("BIRCHIT", "TOTODILE"),
    ];
    for (species, original) in starters {
        let gift = format!("MnEarlyPartner{species}");
        let ball = format!("MN_W_{original}");
        let claim = command(
            "setevent",
            &[&format!("EVENT_MN_{species}_RESEARCH_RECEIVED")],
        );
        let mut expected = before.scripts[&gift]
            .as_array()
            .expect("original gift script")
            .clone();
        let success = expected
            .iter()
            .position(|row| row == &claim)
            .expect("successful adoption");
        expected.insert(success + 1, command("disappear", &[&ball]));
        assert_eq!(
            room.scripts[&gift].as_array(),
            Some(&expected),
            "only successful gifts hide a ball"
        );
        assert_eq!(
            room.scripts[&ball], before.scripts[&ball],
            "preserve first-choice encounters"
        );
    }
    for chosen in [None, Some(0), Some(1), Some(2)] {
        for adoptions in 0..8 {
            let mut state = GameState::reset_wram_for_new_game();
            for (index, (species, original)) in starters.iter().enumerate() {
                state.flags.set_event_flag(
                    &format!("EVENT_MN_WINONA_{original}"),
                    Some(index) == chosen,
                )?;
                state.flags.set_event_flag(
                    &format!("EVENT_MN_{species}_RESEARCH_RECEIVED"),
                    adoptions & (1 << index) != 0,
                )?;
            }
            let mut session = pack
                .data
                .overworld_session(STATION, TilePosition::new(4, 6), 0)?;
            pack.data.apply_map_setup_callbacks(
                &mut state,
                &mut session,
                STATION,
                "MAPSETUP_WARP",
            )?;
            for (index, (_, original)) in starters.iter().enumerate() {
                let actor = format!("MN_W_{original}");
                let ball = session
                    .objects
                    .iter()
                    .find(|obj| obj.object_identifier.as_deref() == Some(actor.as_str()))
                    .expect("native ball");
                let picked = Some(index) == chosen;
                assert_eq!(
                    session.is_object_visible(ball),
                    !picked && adoptions & (1 << index) == 0
                );
                assert_eq!(
                    state
                        .flags
                        .is_event_flag_set(&format!("EVENT_MN_WINONA_{original}"))?,
                    picked
                );
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the external selected Minnesota review pack"]
fn fossil_lab_dialogue_revision_preserves_the_existing_room() {
    let base = crate::read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_FOSSIL_DIALOGUE_PACK").expect("selected pack"),
    )
    .expect("verified selected pack");
    let child = super::revise_minnesota_fossil_lab_dialogue(&base).expect("revise lab dialogue");
    let before = &base.data.maps["MnRochesterFossilLab"];
    let after = &child.data.maps["MnRochesterFossilLab"];
    assert_eq!(before.blocks, after.blocks);
    assert_eq!(before.objects, after.objects);
    let labels = [
        "MnRochesterFossilLabAssistantText",
        "MnLabFossilNeedSpecimenText",
        "MnLabSecondThanksText",
    ];
    for label in labels {
        let body = after.scripts[label].to_string();
        assert!(
            !body.contains("DRIFTLESS") && !body.contains("Route 3"),
            "{label}"
        );
    }
    for (label, script) in &before.scripts {
        if !labels.contains(&label.as_str()) {
            assert_eq!(after.scripts.get(label), Some(script), "script {label}");
        }
    }
    assert_eq!(before.scripts.len(), after.scripts.len());
    for (name, original) in &base.data.maps {
        if name != "MnRochesterFossilLab" {
            assert_eq!(&child.data.maps[name], original, "map {name}");
        }
    }
    let original = serde_json::to_value(base.data()).expect("original game data");
    let mut revised = serde_json::to_value(child.data()).expect("revised game data");
    for label in labels {
        for field in ["scripts", "script_text_bodies"] {
            revised["maps"]["MnRochesterFossilLab"][field][label] =
                original["maps"]["MnRochesterFossilLab"][field][label].clone();
        }
        revised["map_scripts"][label] = original["map_scripts"][label].clone();
    }
    assert_eq!(original, revised, "only the three lab texts may change");
    assert_eq!(base.runtime_files(), child.runtime_files());
    assert!(base.compiled_audio() == child.compiled_audio());
    assert!(super::revise_minnesota_fossil_lab_dialogue(&child).is_err());
}

#[test]
#[ignore = "requires CRYSTAL_RESEARCH_PARENT with installed fossil lab and late starter aide"]
fn early_research_preserves_unrelated_content_and_rejects_stale_scripts() {
    let base = crate::read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RESEARCH_PARENT").expect("external research parent"),
    )
    .expect("verified parent");
    let child = build_minnesota_research_access(&base).expect("complete early access");
    let lab = &child.data.maps["MnRochesterFossilLab"];
    for label in [
        "MnRochesterFossilLabAssistantText",
        "MnLabFossilNeedSpecimenText",
        "MnLabSecondThanksText",
    ] {
        let dialogue = lab.scripts[label].to_string();
        assert!(
            !dialogue.contains("DRIFTLESS"),
            "{label} assumes a trail discovery"
        );
        assert!(
            !dialogue.contains("Route 3"),
            "{label} spoils a later route"
        );
    }
    let before = serde_json::to_value(base.data()).expect("parent data");
    let mut after = serde_json::to_value(child.data()).expect("child data");
    after["maps"] = before["maps"].clone();
    after["map_scripts"] = before["map_scripts"].clone();
    for flag in [
        "EVENT_MN_POUCHID_RESEARCH_RECEIVED",
        "EVENT_MN_BIRCHIT_RESEARCH_RECEIVED",
    ] {
        assert!(
            after["story_event_script_constants"]["global"]
                .as_object_mut()
                .expect("global flags")
                .remove(flag)
                .is_some()
        );
    }
    assert_eq!(
        before, after,
        "all other data and original regions retained exactly"
    );
    assert_eq!(base.runtime_files(), child.runtime_files());
    assert!(base.compiled_audio() == child.compiled_audio());
    for (name, original) in &base.data.maps {
        let revised = &child.data.maps[name];
        if !matches!(
            name.as_str(),
            "MnWinonaFieldStation" | "MnR01C00R00" | "MnRochesterFossilLab"
        ) {
            assert_eq!(original, revised, "unrelated complete map {name}");
            continue;
        }
        let old = serde_json::to_value(original).expect("original map");
        let mut new = serde_json::to_value(revised).expect("revised map");
        for key in [
            "scripts",
            "script_item_grants",
            "gift_pokemon_scripts",
            "script_flag_commands",
            "script_text_commands",
            "script_text_bodies",
            "script_variable_commands",
            "script_control_commands",
            "script_runtime_commands",
            "script_object_commands",
        ] {
            new[key] = old[key].clone();
        }
        assert_eq!(
            old, new,
            "terrain, actors, story, encounters and all unrelated map metadata {name}"
        );
        assert_eq!(
            original
                .script_object_commands
                .iter()
                .filter(|entry| entry.source_script != "MnRochesterFossilLabAssistant")
                .collect::<Vec<_>>(),
            revised
                .script_object_commands
                .iter()
                .filter(|entry| entry.source_script != "MnRochesterFossilLabAssistant")
                .collect::<Vec<_>>(),
            "only the fossil assistant's explicit facing command changes",
        );
        for (label, body) in &original.scripts {
            if matches!(
                label.as_str(),
                "MnKindlitResearchAide"
                    | "MnWTamarackDeliveryReturnText"
                    | "MnFossilExchange"
                    | "MnFossilExchangeText"
                    | "MnFossilDirectionsText"
                    | "MnRochesterFossilLabAssistant"
                    | "MnRochesterFossilLabAssistantText"
                    | "MnLabFossilNeedSpecimenText"
                    | "MnLabFossilRestoredText"
            ) {
                continue;
            }
            assert_eq!(
                Some(body),
                revised.scripts.get(label),
                "unrelated program {name}/{label}"
            );
        }
    }
    assert!(
        build_minnesota_research_access(&child).is_err(),
        "no repeated installation"
    );
    let mut stale = base.clone();
    stale
        .data
        .maps
        .get_mut("MnWinonaFieldStation")
        .expect("station")
        .scripts
        .get_mut("MnKindlitResearchAide")
        .expect("aide")[1]["args"][0] = serde_json::json!("EVENT_MN_STARTER_RECEIVED");
    assert!(
        build_minnesota_research_access(&stale).is_err(),
        "changed prerequisite is rejected"
    );
}
