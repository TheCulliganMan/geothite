use crate::core::systems::shop::ShopResult;

#[test]
fn mart_counter_interaction_opens_source_shop() {
    let asset_root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..").canonicalize().unwrap());
    let runtime = workspace_desktop_runtime(&asset_root);
    let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
    let mut shell = initialize_bevy_runtime_shell(
        asset_root, runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier, map_name: "CherrygroveMart".to_string(), tile_x: 3, tile_y: 3,
        },
        BevyShellConfig { smoke_player_name: Some("TEST".to_string()), ..Default::default() },
    ).unwrap();
    complete_visible_smoke_player_name_if_needed(&mut shell, Some("TEST")).unwrap();
    shell.shell.session_mut().overworld_mut().player.facing = Direction::Left;
    shell.shell.add_bag_item("PARLYZ_HEAL", 2).unwrap();
    assert_eq!(shell.shell.current_overworld_interaction_checked().unwrap().map(|i| i.script),
        Some("CherrygroveMartClerkScript".to_string()));
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        shell.shell.save(PathBuf::from(directory).join("mart-browser.crystalsave")).unwrap();
    }
    let mut app = menu_render_test_app(shell);
    app.update();
    for _ in 0..128 {
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
        if shell.shop_top_cursor.is_some() { break; }
    }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert!(shell.shop_top_cursor.is_some());
    assert_eq!(shell.shell.snapshot().unwrap().pending_shop.unwrap().mart_id, "MART_CHERRYGROVE");
}

#[test]
fn mart_counter_interaction_survives_save_restore() {
    let asset_root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..").canonicalize().unwrap());
    let runtime = workspace_desktop_runtime(&asset_root);
    let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
    let mut shell = initialize_bevy_runtime_shell(
        asset_root, runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier, map_name: "CherrygroveMart".into(), tile_x: 3, tile_y: 3,
        },
        BevyShellConfig { smoke_player_name: Some("TEST".into()), ..Default::default() },
    ).unwrap();
    complete_visible_smoke_player_name_if_needed(&mut shell, Some("TEST")).unwrap();
    let path = std::env::temp_dir().join(format!("mart-restore-{}.crystalsave", std::process::id()));
    shell.shell.save(&path).unwrap();
    load_visible_runtime_save(&mut shell, &path, "title_continue").unwrap();
    std::fs::remove_file(&path).unwrap();
    assert!(shell.shell.session().overworld().object_has_loaded_struct(0),
        "the restored clerk must retain its live object struct");
    shell.shell.session_mut().overworld_mut().player.facing = Direction::Left;
    assert_eq!(shell.shell.current_overworld_interaction_checked().unwrap().map(|i| i.script),
        Some("CherrygroveMartClerkScript".into()));
    let mut app = menu_render_test_app(shell);
    app.update();
    for _ in 0..128 {
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
        if shell.shop_top_cursor.is_some() { return; }
    }
    panic!("restored Mart clerk must open the shop");
}

#[test]
fn mart_rendering_preserves_the_text_contract() {
    let rendering = include_str!("../overworld_rendering.rs");
    let interactions = include_str!("../battle_messages.rs");

    for text in [
        "Welcome! How may I\\nhelp you?",
        "Can I do anything\\nelse for you?",
        "Please come again!",
        "Here you are.\\nThank you!",
        "You don't have\\nenough money.",
        "You can't carry\\nany more items.",
        "You don't have anything to sell.",
        "You don't have any left.",
        "BUY",
        "SELL",
        "QUIT",
    ] {
        assert!(
            rendering.contains(text) || interactions.contains(text),
            "mart UI is missing visible text {text:?}"
        );
    }

    assert!(rendering.contains("format_price(snapshot.trainer.money)"));
    assert!(rendering.contains("format_price(u32::from(item.price))"));
    assert!(rendering.contains("format!(\"×{:02} {}\", quantity.quantity, format_price(total))"));
    assert!(rendering.contains("visible_window_start(selected, item_ids.len(), 4)"));
    assert!(rendering.contains("battle_hud_tile_origin(1.0, 14.0 + index as f32 * 2.0)"));
    assert!(rendering.contains("let row = 4.0 + visible_index as f32 * 2.0;"));
    assert!(rendering.contains("battle_hud_tile_origin(2.0, row)"));
    assert!(rendering.contains("battle_hud_tile_origin(10.0, row + 1.0)"));
    assert!(rendering.contains("battle_hud_tile_origin(8.0, 16.0)"));
    assert!(!rendering.contains("let price = if selling { item.price / 2 } else { item.price };"));
    assert!(!rendering.contains("shop.mart_type, shop.mart_id, snapshot.trainer.money"));
    assert!(!rendering.contains("SELL {} ${}"));
    assert!(interactions.contains("Sorry, we're sold out."));
}

#[test]
fn mart_top_menu_text_stays_inside_its_window() {
    const LONGEST_OPTION_WIDTH: f32 = 5.0; // cursor/space plus "SELL" or "QUIT"
    const FORMATTED_MONEY_WIDTH: f32 = 7.0;

    assert!(SHOP_TOP_MENU_OPTION_LEFT >= SHOP_TOP_MENU_LEFT + 1.0);
    assert!(
        SHOP_TOP_MENU_OPTION_LEFT + LONGEST_OPTION_WIDTH
            <= SHOP_TOP_MENU_LEFT + SHOP_TOP_MENU_WIDTH - 1.0,
        "BUY/SELL/QUIT must render inside the top-menu frame"
    );
    assert!(SHOP_TOP_MENU_LEFT + SHOP_TOP_MENU_WIDTH <= SHOP_MONEY_WINDOW_LEFT);
    assert!(
        SHOP_MONEY_TEXT_LEFT + FORMATTED_MONEY_WIDTH
            <= SHOP_MONEY_WINDOW_LEFT + SHOP_MONEY_WINDOW_WIDTH - 1.0,
        "the six-digit money value must render inside the money frame"
    );
}

#[test]
fn mart_cancel_and_empty_buy_list_preserve_valid_menu_flow() {
    let session = include_str!("../deterministic_session.rs");
    let interactions = include_str!("../battle_messages.rs");

    let cancel_quantity = session
        .find("if runtime_shell.shop_quantity.take().is_some()")
        .expect("shop quantity cancellation branch");
    let next_shop_branch = session[cancel_quantity..]
        .find("if runtime_shell.shop_top_cursor.is_none()")
        .expect("top-menu fallback after quantity cancellation");
    assert!(
        session[cancel_quantity..cancel_quantity + next_shop_branch].contains("return Ok(());"),
        "cancelling a quantity prompt must return to its current item list"
    );
    assert!(interactions.contains("if shop.inventory.is_empty()"));
    assert!(interactions.contains("Sorry, we're sold out."));
}

#[test]
fn mart_cursor_and_sell_inventory_follow_the_reference_boundaries() {
    let navigation = include_str!("../credits.rs");
    let interactions = include_str!("../battle_messages.rs");

    assert!(navigation.contains("fn move_visible_mart_cursor_slot("));
    assert!(navigation.contains("current.saturating_sub(delta.unsigned_abs())"));
    assert!(navigation.contains(".min(option_count - 1)"));
    assert!(interactions.contains("snapshot\n            .bag\n            .tm_hm"));
}

#[test]
fn mart_item_cursors_clamp_at_both_ends() {
    let mut cursor = Some(MenuCursor {
        surface_id: "shop:test".to_string(),
        option_index: 0,
    });
    let mut events = Vec::new();

    move_visible_mart_cursor_slot(&mut cursor, "shop:test".to_string(), 4, -1, &mut events)
        .expect("clamp at first item");
    assert_eq!(cursor.as_ref().map(|cursor| cursor.option_index), Some(0));

    move_visible_mart_cursor_slot(&mut cursor, "shop:test".to_string(), 4, 99, &mut events)
        .expect("clamp at final item");
    assert_eq!(cursor.as_ref().map(|cursor| cursor.option_index), Some(3));

    move_visible_mart_cursor_slot(&mut cursor, "shop:test".to_string(), 4, -2, &mut events)
        .expect("move upward within list");
    assert_eq!(cursor.as_ref().map(|cursor| cursor.option_index), Some(1));
}

#[test]
fn mart_transaction_notices_use_player_facing_copy_and_prices() {
    let bought = ShopResult {
        success: true,
        message: format_price(300),
        credited: 300,
    };
    let sold = ShopResult {
        success: true,
        message: format_price(150),
        credited: 150,
    };
    let no_money = ShopResult {
        success: false,
        message: "You don't have enough money.".to_string(),
        credited: 0,
    };
    let pack_full = ShopResult {
        success: false,
        message: "Your Pack is full.".to_string(),
        credited: 0,
    };

    assert_eq!(format_price(0), "¥000000");
    assert_eq!(format_price(999_999), "¥999999");
    assert_eq!(
        visible_shop_transaction_status("BOUGHT", "POTION", &bought),
        "Here you are.\nThank you!"
    );
    assert_eq!(
        visible_shop_transaction_status("SOLD", "POTION", &sold),
        "Sold for ¥000150!"
    );
    assert_eq!(
        visible_shop_transaction_status("BOUGHT", "POTION", &no_money),
        "You don't have\nenough money."
    );
    assert_eq!(
        visible_shop_transaction_status("BOUGHT", "POTION", &pack_full),
        "You can't carry\nany more items."
    );
}

fn initialized_mart_shell() -> BevyRuntimeShell {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .pending_shop = Some(crate::core::state::ScriptShopRequest {
        mart_type: "MARTTYPE_STANDARD".to_string(),
        mart_id: "MART_CHERRYGROVE".to_string(),
        inventory: vec![
            "POTION".to_string(),
            "ANTIDOTE".to_string(),
            "PARLYZ_HEAL".to_string(),
            "AWAKENING".to_string(),
        ],
        source_script: "CherrygroveMartClerkScript".to_string(),
        command_index: 3,
    });
    shell.shop_welcome_seen = true;
    shell.shop_notice = None;
    shell.shop_top_cursor = Some(MenuCursor {
        surface_id: "shop:top".to_string(),
        option_index: 0,
    });
    shell
}

#[test]
fn mart_frames_and_glyphs_share_the_fullscreen_dialog_layer() {
    let shell = initialized_mart_shell();
    let snapshot = shell.shell.snapshot().expect("snapshot");
    let mut world = World::new();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    spawn_field_shop_screen(
        &mut Commands::new(&mut queue, &world),
        &snapshot,
        &shell,
        snapshot.pending_shop.as_ref().unwrap(),
        &mut art,
        &shell.asset_root,
        &mut images,
    )
    .expect("render mart");
    queue.apply(&mut world);
    let mut query = world.query::<(Entity, Option<&SceneDialogMarker>)>();
    for (entity, marker) in query.iter(&world) {
        assert!(
            marker.is_some(),
            "mart entity {entity:?} would detach from its text in fullscreen"
        );
    }
    assert!(world.entities().len() > 50, "frames and text were spawned");
}

#[test]
fn mart_quantity_buttons_follow_asm_wrap_and_ten_item_steps() {
    let mut shell = initialized_mart_shell();
    let shop = shell.shell.snapshot().unwrap().pending_shop.unwrap();
    shell.shell.session_mut().state_mut().money = 0;
    begin_visible_shop_quantity(&mut shell, &shop, 0, false).unwrap();
    assert_eq!(
        shell.shop_quantity.as_ref().map(|q| q.max_quantity),
        Some(99),
        "ASM checks money after confirmation, not before quantity selection"
    );
    adjust_visible_shop_quantity(&mut shell, -1).unwrap();
    assert_eq!(shell.shop_quantity.as_ref().unwrap().quantity, 99);
    adjust_visible_shop_quantity(&mut shell, 1).unwrap();
    assert_eq!(shell.shop_quantity.as_ref().unwrap().quantity, 1);
    adjust_visible_shop_quantity(&mut shell, 10).unwrap();
    assert_eq!(shell.shop_quantity.as_ref().unwrap().quantity, 11);
    adjust_visible_shop_quantity(&mut shell, -10).unwrap();
    assert_eq!(shell.shop_quantity.as_ref().unwrap().quantity, 1);
}

#[test]
fn mart_purchase_requires_confirmation_and_returns_to_the_item_list() {
    let mut shell = initialized_mart_shell();
    confirm_visible_shop_top_menu(&mut shell).unwrap();
    let shop = shell.shell.snapshot().unwrap().pending_shop.unwrap();
    shell.shell.session_mut().state_mut().money = 1000;
    begin_visible_shop_quantity(&mut shell, &shop, 0, false).unwrap();
    confirm_visible_shop_quantity(&mut shell).unwrap();
    assert_eq!(
        shell.shell.session().state().money,
        1000,
        "choosing a quantity must not purchase before YesNoBox"
    );
    assert!(shell.shop_quantity.is_some());
    confirm_visible_shop_quantity(&mut shell).unwrap();
    assert_eq!(shell.shell.session().state().money, 700);
    assert!(
        !shell.shop_return_to_top_after_notice,
        "BuyMenuLoop stays in the inventory after a transaction"
    );
}

#[test]
fn mart_confirmation_can_be_declined_without_mutating_money() {
    let mut shell = initialized_mart_shell();
    confirm_visible_shop_top_menu(&mut shell).unwrap();
    let shop = shell.shell.snapshot().unwrap().pending_shop.unwrap();
    let money = shell.shell.session().state().money;
    begin_visible_shop_quantity(&mut shell, &shop, 0, false).unwrap();
    confirm_visible_shop_quantity(&mut shell).unwrap();
    adjust_visible_shop_quantity(&mut shell, -1).unwrap();
    confirm_visible_shop_quantity(&mut shell).unwrap();
    assert!(shell.shop_quantity.is_none());
    assert_eq!(shell.shell.session().state().money, money);
    assert!(shell.shop_top_cursor.is_none());
    assert_eq!(shell.menu_cursor.as_ref().unwrap().option_index, 0);
}

#[test]
fn mart_sell_subtotal_halves_the_complete_quantity_price() {
    let quantity = VisibleShopQuantity {
        item_id: "POTION".to_string(),
        selling: true,
        quantity: 2,
        max_quantity: 99,
        unit_price: 301,
        confirmation: None,
    };
    assert_eq!(visible_shop_quantity_total(&quantity), 301);
}

#[test]
fn mart_buy_list_includes_the_asm_cancel_entry() {
    let mut shell = initialized_mart_shell();
    confirm_visible_shop_top_menu(&mut shell).unwrap();
    move_visible_shop_buy_cursor(&mut shell, 99).unwrap();
    assert_eq!(shell.menu_cursor.as_ref().unwrap().option_index, 4);
    buy_visible_shop_cursor_item(&mut shell).unwrap();
    assert!(shell.shop_top_cursor.is_some());
    assert!(shell.shop_quantity.is_none());
}

#[test]
fn mart_long_item_names_render_every_glyph_in_buy_and_sell_rows() {
    for selling in [false, true] {
        let mut shell = initialized_mart_shell();
        confirm_visible_shop_top_menu(&mut shell).unwrap();
        let mut snapshot = shell.shell.snapshot().unwrap();
        // Render the real PARLYZ HEAL entry with its source price and description.
        if selling {
            snapshot.bag.items = vec![crate::RuntimeBagItemSnapshot {
                item_id: "PARLYZ_HEAL".to_string(),
                quantity: 2,
            }];
            shell.sell_cursor = Some(MenuCursor {
                surface_id: "sell:bag".to_string(),
                option_index: 0,
            });
        }
        if !selling { move_visible_shop_buy_cursor(&mut shell, 2).unwrap(); }
        let mut world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        let mut art = RenderedTilesetArt::default();
        let mut images = Assets::<Image>::default();
        spawn_field_shop_screen(
            &mut Commands::new(&mut queue, &world),
            &snapshot,
            &shell,
            snapshot.pending_shop.as_ref().unwrap(),
            &mut art,
            &shell.asset_root,
            &mut images,
        )
        .unwrap();
        queue.apply(&mut world);
        assert!(art.font_error.is_none(), "{:?}", art.font_error);
        let (x, y) = battle_hud_tile_origin(2.0, if selling { 4.0 } else { 8.0 });
        let mut glyphs = world.query::<(&DialogGlyphMarker, &Transform)>();
        let row = glyphs
            .iter(&world)
            .filter(|(_, transform)| transform.translation.y == y)
            .collect::<Vec<_>>();
        assert_eq!(row.len(), if selling { 16 } else { 12 });
        for index in 0..12 {
            assert!(
                row.iter()
                    .any(|(marker, _)| marker.key == dialog_glyph_key(x, y, index))
            );
        }
        assert!(row.iter().all(|(_, transform)| transform.translation.x < battle_hud_tile_origin(19.0, 4.0).0));
        if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
            std::fs::create_dir_all(&directory).unwrap();
            let label = if selling { "mart-sell" } else { "mart-buy" };
            render_pc_audit_canvas(&mut world, &images, label)
                .save(PathBuf::from(directory).join(format!("{label}.png"))).unwrap();
        }
    }
}

#[test]
fn tm_shop_renders_move_description_on_a_cleared_menu_screen() {
    let mut shell = initialized_mart_shell();
    shell.shell.session_mut().state_mut().script_runtime.pending_shop.as_mut().unwrap().inventory =
        vec!["TM_THUNDERPUNCH".into()];
    #[cfg(feature = "fullscreen-scaling")]
    assert!(fullscreen_field_panel_active(&shell));
    confirm_visible_shop_top_menu(&mut shell).unwrap();
    #[cfg(feature = "fullscreen-scaling")]
    assert!(!fullscreen_field_panel_active(&shell));
    let mut app = menu_render_test_app(shell);
    for _ in 0..3 { app.update(); }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
    let text = rendered_mart_text_for_test(&mut app);
    assert!(text.contains("electric punch") && text.contains("paralyze"), "{text}");
    retained_fullscreen_surface(app.world_mut());
    save_live_menu_lcd_for_test(app.world_mut(), "tm-shop-stock.png");
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    for _ in 0..3 { app.update(); }
    let text = rendered_mart_text_for_test(&mut app);
    assert!(text.contains("How many?"), "{text}");
    assert!(!text.contains("electric punch"), "quantity prompt must replace the description: {text}");
    save_live_menu_lcd_for_test(app.world_mut(), "tm-shop-quantity.png");
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyX);
    for _ in 0..3 { app.update(); }
    let text = rendered_mart_text_for_test(&mut app);
    assert!(text.contains("electric punch") && !text.contains("How many?"), "{text}");
}

fn rendered_mart_text_for_test(app: &mut App) -> String {
    let glyphs = app.world().resource::<RenderedTilesetArt>().font_cache.as_ref().unwrap()
        .glyphs.iter().map(|(character, frame)| (*character, frame.handle.clone())).collect::<Vec<_>>();
    let mut letters = app.world_mut().query::<(&Handle<Image>, &Transform, &Visibility)>()
        .iter(app.world()).filter(|(_, _, visibility)| **visibility != Visibility::Hidden)
        .filter_map(|(image, transform, _)| glyphs.iter().find(|(_, handle)| handle == image)
            .map(|(character, _)| (transform.translation.y, transform.translation.x, *character)))
        .collect::<Vec<_>>();
    letters.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.total_cmp(&b.1)));
    letters.iter().map(|(_, _, character)| *character).collect::<String>()
}
