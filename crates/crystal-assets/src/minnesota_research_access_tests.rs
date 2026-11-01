use super::build_minnesota_research_access;

#[test]
#[ignore = "requires CRYSTAL_RESEARCH_PARENT with installed fossil lab and late starter aide"]
fn early_research_preserves_unrelated_content_and_rejects_stale_scripts() {
    let base = crate::read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RESEARCH_PARENT").expect("external research parent"),
    )
    .expect("verified parent");
    let child = build_minnesota_research_access(&base).expect("complete early access");
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
