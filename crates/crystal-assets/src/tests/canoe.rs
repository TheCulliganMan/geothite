fn canoe_fixture(
    map_name: &str,
    owns: bool,
    obstacle: bool,
) -> (GameDataSet, GameState, OverworldSession) {
    let mut map = test_map_module(map_name, "CANOE_TEST", None);
    map.attributes.width = 3;
    map.blocks = vec![0, 1, 0];
    if obstacle {
        map.objects.push(test_object("CANOE_TEST_NPC", "-1", 2, 0));
    }
    let mut tileset = test_tileset_definition();
    tileset
        .collision
        .insert("1".into(), vec!["WATER".into(); 4]);
    let mut item = test_item("CANOE");
    item.pocket = "KEY_ITEM".into();
    let data = GameDataSet {
        maps: map_payload(vec![map]),
        tilesets: BTreeMap::from([("johto".into(), tileset)]),
        items: BTreeMap::from([("CANOE".into(), item)]),
        runtime_map_metadata: BTreeMap::from([(
            "CANOE_TEST".into(),
            RuntimeMapMetadata {
                constant: "CANOE_TEST".into(),
                name: map_name.into(),
                group_name: "GROUP_CANOE_TEST".into(),
                group_id: 1,
                map_id: 1,
                width: 3,
                height: 1,
                environment: "ROUTE".into(),
                phone_service: 0,
            },
        )]),
        ..GameDataSet::default()
    };
    let mut state = GameState::default();
    if owns {
        assert!(
            state
                .bag
                .add_item(&data.items["CANOE"], 1)
                .expect("canoe fixture")
        );
    }
    let mut session = data
        .overworld_session(map_name, TilePosition::new(1, 0), 0)
        .expect("shore");
    session.player.facing = Direction::Right;
    (data, state, session)
}

fn canoe_input(
    data: &GameDataSet,
    state: &mut GameState,
    session: &mut OverworldSession,
    button: GameButton,
) -> OverworldInputFrame {
    data.apply_overworld_input(
        state,
        session,
        [button],
        &BTreeSet::new(),
        &mut ReplayDivider::new([0; 256]),
    )
    .expect("normal movement transaction")
}

#[test]
fn canoe_launches_without_party_or_pp_and_portages_back_to_land() {
    use crystal_core::systems::field_moves::CANOE_ACTIVE_MEMORY;
    let (data, mut state, mut session) = canoe_fixture("MnCanoeTest", true, false);
    let bag = state.bag.clone();
    let storage = state.storage.clone();
    let frame = canoe_input(&data, &mut state, &mut session, GameButton::Right);
    assert!(matches!(frame.movement, Some(StepOutcome::Moved { .. })));
    assert_eq!(session.player.tile, TilePosition::new(2, 0));
    assert_eq!(session.player.mode, MovementMode::Surf);
    assert_eq!(state.script_runtime.memory[CANOE_ACTIVE_MEMORY], "1");
    // Turning on water is still an ordinary turn; the following input lands.
    canoe_input(&data, &mut state, &mut session, GameButton::Left);
    canoe_input(&data, &mut state, &mut session, GameButton::Left);
    assert_eq!(session.player.tile, TilePosition::new(1, 0));
    assert_eq!(session.player.mode, MovementMode::Normal);
    assert!(
        !state
            .script_runtime
            .memory
            .contains_key(CANOE_ACTIVE_MEMORY)
    );
    assert_eq!(state.bag, bag);
    assert_eq!(state.storage, storage);
}

#[test]
fn canoe_requires_ownership_and_preserves_johto_and_occupied_shores() {
    use crystal_core::systems::field_moves::CANOE_ACTIVE_MEMORY;
    for (map, owns, obstacle) in [
        ("MnCanoeTest", false, false),
        ("Route29", true, false),
        ("MnCanoeTest", true, true),
    ] {
        let (data, mut state, mut session) = canoe_fixture(map, owns, obstacle);
        let frame = canoe_input(&data, &mut state, &mut session, GameButton::Right);
        assert!(
            matches!(
                frame.movement,
                Some(StepOutcome::Blocked { .. } | StepOutcome::BlockedByObject { .. })
            ),
            "{map}/{owns}/{obstacle}: {:?}",
            frame.movement
        );
        assert_eq!(session.player.tile, TilePosition::new(1, 0));
        assert_eq!(session.player.mode, MovementMode::Normal);
        assert!(
            !state
                .script_runtime
                .memory
                .contains_key(CANOE_ACTIVE_MEMORY)
        );
    }
}

#[test]
fn canoe_does_not_bypass_dialogue_or_directional_shore_barriers() {
    let (data, mut state, mut session) = canoe_fixture("MnCanoeTest", true, false);
    state.script_runtime.next_script = Some(ScriptLocation {
        origin_map_name: "MnCanoeTest".into(),
        script: "StillTalking".into(),
    });
    let frame = canoe_input(&data, &mut state, &mut session, GameButton::Right);
    assert!(frame.movement.is_none());
    assert_eq!(session.player.mode, MovementMode::Normal);
    state.script_runtime.next_script = None;
    session.tileset.metatiles[0].collision = [permissions::RIGHT_WALL; 4];
    let frame = canoe_input(&data, &mut state, &mut session, GameButton::Right);
    // Wall origins are not traversable launch pads.
    assert!(!matches!(frame.movement, Some(StepOutcome::Moved { .. })));
    assert_eq!(session.player.mode, MovementMode::Normal);
}

#[test]
#[ignore = "requires CRYSTAL_CANOE_BASE_PACK"]
fn canoe_pack_preserves_audio_roster_existing_art_and_unrelated_maps() {
    let base = read_verified_compiled_game_pack(std::path::PathBuf::from(
        std::env::var_os("CRYSTAL_CANOE_BASE_PACK").expect("actual parent pack"),
    ))
    .expect("verified parent");
    let result = crate::build_canoe_modpack(&base).expect("canoe publication");
    assert_eq!(result.compiled_audio, base.compiled_audio);
    assert_eq!(result.audio_manifest, base.audio_manifest);
    assert_eq!(result.data.pokemon, base.data.pokemon);
    assert_eq!(result.data.field_moves, base.data.field_moves);
    for (name, group) in &base.data.fishing.groups {
        if !name.starts_with("FISHGROUP_MN_") {
            assert_eq!(result.data.fishing.groups.get(name), Some(group));
        }
    }
    for (id, bytes) in &base.runtime_files {
        assert_eq!(
            result.runtime_files.get(id),
            Some(bytes),
            "existing art {id}"
        );
    }
    assert_eq!(result.runtime_files.len(), base.runtime_files.len() + 4);
    for (id, old) in &base.data.maps {
        let new = &result.data.maps[id];
        if ![
            "MnStPaul",
            "MnStPaulAnglerHouse",
            "MnBemidji",
            "MnBoundarySafariC00R01",
            "MnBoundaryWaters",
            "MnBrainerdOutfitter",
            "MnDuluth",
            "MnGrandMaraisFisherHouse",
            "MnMinneapolis",
            "MnMississippiLowerWoodsC01R01",
            "MnMoorhead",
            "MnR07C00R00",
            "MnR10C01R00",
            "MnR18C00R00",
            "MnStPaulGym",
        ]
        .contains(&id.as_str())
        {
            assert_eq!(new, old, "unrelated map {id}");
            assert_eq!(
                result.data.wild_encounters.get(id),
                base.data.wild_encounters.get(id)
            );
            assert_eq!(
                result.data.field_encounters.get(id),
                base.data.field_encounters.get(id)
            );
        } else {
            if id != "MnStPaul" {
                assert_eq!(new.blocks, old.blocks, "unchanged terrain {id}");
            }
            assert_eq!(new.attributes, old.attributes, "connections {id}");
            assert_eq!(new.events.warps, old.events.warps, "warps {id}");
            if id != "MnStPaulAnglerHouse" {
                assert_eq!(new.objects, old.objects, "actors {id}");
            }
        }
    }
    let item = &result.data.items["CANOE"];
    assert_eq!(item.pocket, "KEY_ITEM");
    assert!(!item.consumable);
    assert_eq!(item.property, "CANT_SELECT | CANT_TOSS");
    assert!(crate::build_canoe_modpack(&result).is_err());
}

#[test]
fn canoe_cannot_launch_from_a_bridge_deck_or_into_a_special_water_obstacle() {
    let (data, mut state, mut session) = canoe_fixture("MnCanoeTest", true, false);
    session.tileset.metatiles[0].collision = [crystal_core::world::bridge::NORTH_SOUTH_DECK; 4];
    let frame = canoe_input(&data, &mut state, &mut session, GameButton::Right);
    assert!(!matches!(frame.movement, Some(StepOutcome::Moved { .. })));
    assert_eq!(session.player.mode, MovementMode::Normal);
    let (mut data, mut state, mut session) = canoe_fixture("MnCanoeTest", true, false);
    data.field_moves
        .surf
        .blocked_collisions
        .push(permissions::WATERFALL);
    session.tileset.metatiles[1].collision = [permissions::WATERFALL; 4];
    let frame = canoe_input(&data, &mut state, &mut session, GameButton::Right);
    assert!(!matches!(frame.movement, Some(StepOutcome::Moved { .. })));
    assert_eq!(session.player.mode, MovementMode::Normal);
}

#[test]
fn canoe_fishing_requires_a_mounted_canoe_and_water_ahead() {
    use crystal_core::systems::field_moves::CANOE_ACTIVE_MEMORY;
    use crystal_core::world::fishing::FishingError;

    let (data, mut state, mut session) = canoe_fixture("MnCanoeTest", true, false);
    session.player.mode = MovementMode::Surf;
    let before = state.clone();
    let mut divider = ReplayDivider::new([0; 8]);
    let error = data
        .cast_fishing_rod_in_session_with_divider(&mut state, &session, "GOOD_ROD", &mut divider)
        .expect_err("owning a canoe does not allow fishing from a Surf Pokémon");
    assert!(matches!(
        error.downcast_ref::<FishingError>(),
        Some(FishingError::CannotFishWhileSurfing)
    ));
    assert_eq!(state, before);

    state
        .script_runtime
        .memory
        .insert(CANOE_ACTIVE_MEMORY.into(), "1".into());
    session.player.tile = TilePosition::new(3, 0);
    let before = state.clone();
    let error = data
        .cast_fishing_rod_in_session_with_divider(&mut state, &session, "GOOD_ROD", &mut divider)
        .expect_err("canoe must face water");
    assert!(matches!(
        error.downcast_ref::<FishingError>(),
        Some(FishingError::FacingTileIsNotWater)
    ));
    assert_eq!(state, before);
}

#[test]
#[ignore = "requires CRYSTAL_CANOE_TEST_PACK"]
fn canoe_shop_grant_rejects_a_full_key_pocket_and_can_retry() {
    use crystal_core::models::bag::KEY_ITEM_POCKET_CAPACITY;
    use crystal_core::systems::script_items::ScriptItemGrantOutcome;

    const SHOP: &str = "MnStPaulAnglerHouse";
    const OWNER: &str = "MnCanoeShopOwner";
    let pack = read_verified_compiled_game_pack(std::path::PathBuf::from(
        std::env::var_os("CRYSTAL_CANOE_TEST_PACK").expect("canoe pack"),
    ))
    .expect("verified canoe pack");
    let mut data = pack.data;
    let grant = data.maps[SHOP]
        .script_item_grants
        .iter()
        .find(|grant| grant.source_script == OWNER && grant.item_id == "CANOE")
        .expect("compiled canoe grant");
    let command = grant.command_index;
    // Add one fixture-only key item to exercise a rejection the shipped
    // catalog cannot normally reach before the player owns the canoe.
    let mut extra = test_item("CANOE_TEST_KEY_ITEM");
    extra.pocket = "KEY_ITEM".into();
    data.items.insert(extra.script_name.clone(), extra.clone());
    let mut state = GameState::default();
    for item in data
        .items
        .values()
        .filter(|item| item.pocket == "KEY_ITEM" && item.script_name != "CANOE")
        .take(KEY_ITEM_POCKET_CAPACITY)
    {
        assert!(state.bag.add_item(item, 1).expect("fill key pocket"));
    }
    assert_eq!(state.bag.key_items.len(), KEY_ITEM_POCKET_CAPACITY);
    let before = state.bag.clone();
    assert!(matches!(
        data.grant_script_item(&mut state, SHOP, SHOP, OWNER, command)
            .expect("attempt actual shop grant"),
        ScriptItemGrantOutcome::BagFull { .. }
    ));
    assert_eq!(state.bag, before);
    assert!(state.bag.remove_item(&extra, 1).expect("free a slot"));
    assert!(matches!(
        data.grant_script_item(&mut state, SHOP, SHOP, OWNER, command)
            .expect("retry actual shop grant"),
        ScriptItemGrantOutcome::Granted { .. }
    ));
    assert_eq!(state.bag.quantity(&data.items["CANOE"]), 1);
}
