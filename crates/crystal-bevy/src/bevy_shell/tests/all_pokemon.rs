#[test]
fn renderer_neutral_controller_uses_link_cable_through_production_pack() {
    let pack = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/text-tui.crystalpack")
        })
        .canonicalize()
        .expect("resolve local test pack");
    let asset_root = AssetRoot::new(pack.parent().unwrap());
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&asset_root, loaded).unwrap();
    assert!(runtime.data().items.contains_key("LINK_CABLE"));
    let mut controller =
        VisibleShellController::new_game(asset_root, runtime, "CHRIS", None).unwrap();
    controller
        .shell
        .shell
        .add_party_pokemon(
            "ONIX",
            30,
            None,
            None,
            "CHRIS",
            1,
            Dv::from_non_hp(10, 10, 10, 10),
        )
        .unwrap();
    controller
        .shell
        .shell
        .add_bag_item("LINK_CABLE", 2)
        .unwrap();
    controller
        .shell
        .shell
        .add_bag_item("METAL_COAT", 1)
        .unwrap();
    controller
        .shell
        .shell
        .give_bag_item_to_party_pokemon("METAL_COAT", 0)
        .unwrap();
    controller.press(GameButton::Start).unwrap();
    for _ in 0..10 {
        if visible_start_menu_entries(&controller.shell)
            .unwrap()
            .iter()
            .any(|line| line == ">PACK")
        {
            break;
        }
        controller.press(GameButton::Down).unwrap();
    }
    assert!(
        visible_start_menu_entries(&controller.shell)
            .unwrap()
            .iter()
            .any(|line| line == ">PACK")
    );
    controller.press(GameButton::A).unwrap(); // PACK
    assert_eq!(
        controller.shell.field_pack_pocket,
        Some(FieldPackPocket::Items)
    );
    controller.press(GameButton::A).unwrap(); // cable action menu
    assert!(controller.shell.field_pack_action_cursor.is_some());
    controller.press(GameButton::A).unwrap(); // USE
    assert_eq!(
        controller.shell.field_pack_target_mode,
        Some(FieldPackTargetMode::PartyPokemon)
    );
    controller.press(GameButton::A).unwrap(); // ONIX
    for _ in 0..20 {
        let snapshot = controller.snapshot().unwrap();
        if snapshot.party.slots[0].pokemon.species.id == "STEELIX" {
            break;
        }
        controller.press(GameButton::A).unwrap();
    }
    let snapshot = controller.snapshot().unwrap();
    assert_eq!(snapshot.party.slots[0].pokemon.species.id, "STEELIX");
    assert_eq!(snapshot.party.slots[0].pokemon.item, None);
    let mut visible_text = String::new();
    for _ in 0..16 {
        let snapshot = controller.snapshot().unwrap();
        if let Some(text) = snapshot.ui.text.and_then(|text| text.asm_text) {
            visible_text.push_str(&text);
            visible_text.push(' ');
        }
        if visible_text.contains("STEELIX") && visible_text.contains("evolved") {
            break;
        }
        controller.press(GameButton::A).unwrap();
    }
    assert!(
        visible_text.contains("STEELIX") && visible_text.contains("evolved"),
        "{visible_text}"
    );
    let state = controller.shell.shell.session().state();
    assert_eq!(
        state
            .bag
            .quantity(&controller.shell.shell.runtime().data().items["LINK_CABLE"]),
        1
    );
}
