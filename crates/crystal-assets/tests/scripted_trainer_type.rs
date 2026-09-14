use base64::Engine;
use crystal_assets::read_verified_compiled_game_pack;

#[test]
#[ignore = "requires the external CRYSTAL_RENDER_TEST_PACK used by the running game"]
fn real_rival_pack_honors_canlose_in_existing_metadata_and_recompilation() {
    let pack = std::env::var("CRYSTAL_RENDER_TEST_PACK").expect("external game pack");
    let mut data = read_verified_compiled_game_pack(pack)
        .expect("load pinned pack")
        .data()
        .clone();
    let module = data
        .maps
        .get("CherrygroveCity")
        .expect("Cherrygrove module")
        .clone();
    let rivals: Vec<_> = module
        .scripted_trainer_battles
        .iter()
        .filter(|b| b.request.trainer_class == "RIVAL1")
        .collect();
    assert_eq!(rivals.len(), 3);
    for battle in rivals {
        let request = data
            .scripted_trainer_battle_request(
                "CherrygroveCity",
                &battle.source_script,
                battle.startbattle_command_index,
            )
            .unwrap();
        let saved_request = data
            .saved_trainer_battle_request(&battle.source_script, &battle.request.trainer_id)
            .expect("resolve saved trainer metadata")
            .expect("saved trainer request");
        assert_eq!(saved_request.battle_type, request.battle_type);
        assert_eq!(
            request.battle_type, "BATTLETYPE_CANLOSE",
            "already compiled branch {}",
            battle.source_script
        );
    }
    // Rebuild the map from its existing authored payloads entirely in memory.
    data.maps.remove("CherrygroveCity");
    data.map_scripts.extend(module.scripts.clone());
    data.map_attributes
        .insert("CherrygroveCity".into(), module.attributes.clone());
    data.npcs.insert(
        "CherrygroveCity".into(),
        serde_json::to_value(&module.objects).unwrap(),
    );
    let bytes: Vec<u8> = module
        .blocks
        .iter()
        .map(|&b| u8::try_from(b).unwrap())
        .collect();
    data.map_blocks.insert(
        module.attributes.blocks_label.clone().unwrap(),
        base64::engine::general_purpose::STANDARD.encode(bytes),
    );
    let rebuilt = data
        .assemble_map_module_from_compiled_payloads("CherrygroveCity")
        .expect("recompile authored map");
    let rebuilt_rivals: Vec<_> = rebuilt
        .scripted_trainer_battles
        .iter()
        .filter(|b| b.request.trainer_class == "RIVAL1")
        .collect();
    assert_eq!(rebuilt_rivals.len(), 3);
    for battle in rebuilt_rivals {
        assert_eq!(
            battle.request.battle_type, "BATTLETYPE_CANLOSE",
            "newly compiled branch {}",
            battle.source_script
        );
    }
}
