// Opt-in production menu review using accepted, real six-pose species banks.
#[test]
fn party_menus_use_native_species_sprites_and_preserve_shiny_eggs_and_overlays() {
    let Some(input) = std::env::var_os("NORTHWOODS_PARTY_SPRITES_DIR").map(PathBuf::from) else {
        return;
    };
    assert!(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_some());
    let output = std::env::var_os("NORTHWOODS_PARTY_RENDER_DIR").map(PathBuf::from);
    let mut shell = route36_overworld_shell_for_battle_render_regression();
    for token in ["pouchid", "slipperina", "voyagear"] {
        let destination = shell
            .asset_root
            .runtime_assets()
            .join("gfx/pokemon")
            .join(token);
        std::fs::create_dir_all(&destination).expect("owned fixture species folder");
        for name in [
            "overworld.2bpp",
            "overworld.gbcpal",
            "overworld-shiny.gbcpal",
        ] {
            std::fs::copy(
                input.join("gfx/pokemon").join(token).join(name),
                destination.join(name),
            )
            .expect("real accepted species art");
        }
    }
    for (species, dvs) in [
        ("POUCHID", Dv::from_non_hp(9, 9, 9, 9)),
        ("SLIPPERINA", Dv::from_non_hp(9, 9, 9, 9)),
        ("POUCHID", Dv::from_non_hp(2, 10, 10, 10)),
        ("POUCHID", Dv::from_non_hp(9, 9, 9, 9)),
        ("VOYAGEAR", Dv::from_non_hp(9, 9, 9, 9)),
    ] {
        shell
            .shell
            .add_party_pokemon(species, 30, None, None, "CHRIS", 1, dvs)
            .expect("real party species fixture");
    }
    {
        let state = shell.shell.session_mut().state_mut();
        state.storage.party.pokemon[1]
            .as_mut()
            .expect("normal member")
            .item = Some("POTION".into());
        state.storage.party.pokemon[3]
            .as_mut()
            .expect("shiny member")
            .item = Some("FLOWER_MAIL".into());
        state.storage.party.pokemon[3]
            .as_mut()
            .expect("shiny member")
            .mail = Some(crate::core::models::pokemon::MailData {
            message: "NORTHWOODS".into(),
            author: "CHRIS".into(),
            nationality: 0,
            author_id: 1,
            species: "POUCHID".into(),
            mail_type: "FLOWER_MAIL".into(),
        });
        state.storage.party.pokemon[4]
            .as_mut()
            .expect("egg member")
            .is_egg = true;
        state.storage.party.pokemon[4]
            .as_mut()
            .expect("egg member")
            .nickname = "EGG".into();
        state.sync_party_from_storage();
    }
    let mut controller = VisibleShellController { shell };
    controller
        .press(GameButton::Start)
        .expect("production Start menu");
    for _ in 0..10 {
        if visible_start_menu_entries(&controller.shell)
            .expect("Start entries")
            .iter()
            .any(|entry| entry == ">#MON")
        {
            break;
        }
        controller.press(GameButton::Down).expect("select POKEMON");
    }
    assert!(
        visible_start_menu_entries(&controller.shell)
            .expect("Start selection")
            .iter()
            .any(|entry| entry == ">#MON")
    );
    controller
        .press(GameButton::A)
        .expect("production POKEMON menu");
    assert!(controller.shell.party_menu_open);
    controller.shell.lcd_animation_frame = 0;
    let mut app = battle_render_regression_app(controller.shell);
    for battle_surface in [false, true] {
        if battle_surface {
            let shell = app
                .world_mut()
                .remove_resource::<BevyRuntimeShell>()
                .expect("field shell");
            let mut controller = VisibleShellController { shell };
            controller.press(GameButton::B).expect("close field party");
            assert!(!controller.shell.party_menu_open);
            controller
                .shell
                .shell
                .start_scripted_wild_battle("Route36", "WateredWeirdTreeScript", 12)
                .expect("battle menu fixture");
            mark_runtime_snapshot_dirty(&mut controller.shell);
            prepare_visible_battle_entry(&mut controller.shell).expect("prepare battle fixture");
            // Fixture begins at the command boundary, before any turn is taken.
            controller.shell.visible_battle_transition = None;
            controller.shell.visible_battle_sliding_intro = None;
            controller.shell.visible_send_out_animation = None;
            controller.shell.battle_entry_messages_remaining = 0;
            controller.shell.battle_enemy_send_out_pending = false;
            controller.shell.battle_player_send_out_pending = false;
            controller.shell.battle_messages.clear();
            controller.shell.battle_message_scenes.clear();
            controller.shell.battle_text_reveal = None;
            sync_visible_battle_action_cursor(&mut controller.shell);
            controller
                .press(GameButton::Right)
                .expect("select battle PKMN");
            controller
                .press(GameButton::A)
                .expect("open production battle PKMN");
            assert!(controller.shell.battle_switch_cursor.is_some());
            controller.shell.lcd_animation_frame = 0;
            mark_runtime_presentation_dirty(&mut controller.shell);
            app = battle_render_regression_app(controller.shell);
        }
        let mut handles = Vec::new();
        for phase in 0..2 {
            if phase == 1 {
                // This render-only app has no real-time input-clock system.
                // Advance its LCD phase and invalidate presentation exactly as
                // the production clock does, without changing gameplay state.
                let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
                shell.lcd_animation_frame += 8;
                mark_runtime_presentation_dirty(&mut shell);
                assert!(if battle_surface {
                    shell.battle_switch_cursor.is_some()
                } else {
                    shell.party_menu_open
                });
            }
            app.update();
            let world = app.world_mut();
            assert!(
                world.resource::<BevyRuntimeShell>().last_error.is_none(),
                "party render error"
            );
            let snapshot = world
                .resource::<BevyRuntimeShell>()
                .shell
                .snapshot()
                .expect("party rows");
            assert_eq!(snapshot.party.slots.len(), 6);
            let keys = [
                snapshot.presentation.menu_icons["CYNDAQUIL"].as_str(),
                "pokemon_overworld:pouchid:normal",
                "pokemon_overworld:slipperina:normal",
                "pokemon_overworld:pouchid:shiny",
                snapshot.presentation.menu_icons["EGG"].as_str(),
                "pokemon_overworld:voyagear:normal",
            ];
            let cached = &world.resource::<RenderedTilesetArt>().party_icon_cache;
            let overlay_handles = world
                .resource::<RenderedTilesetArt>()
                .party_icon_overlay_cache
                .values()
                .map(|frame| frame.handle.clone())
                .collect::<Vec<_>>();
            assert_eq!(overlay_handles.len(), 2, "item and mail textures");
            assert_eq!(
                cached.len(),
                6,
                "cache keeps species and shiny identities separate"
            );
            let expected_handles = keys.map(|key| cached[key][phase].handle.clone());
            let mut icons = world
                .query::<(
                    &Sprite,
                    &Transform,
                    Option<&FieldCommandMarker>,
                    Option<&BattleCommandMarker>,
                )>()
                .iter(world)
                .filter(|(_, transform, field, battle)| {
                    transform.translation.z == 3.85
                        && if battle_surface {
                            battle.is_some()
                        } else {
                            field.is_some()
                        }
                })
                .map(|(sprite, transform, _, _)| (sprite.clone(), *transform))
                .collect::<Vec<_>>();
            icons.sort_by(|a, b| b.1.translation.y.total_cmp(&a.1.translation.y));
            assert_eq!(icons.len(), 6, "battle={battle_surface} phase={phase}");
            let images = world
                .remove_resource::<Assets<Image>>()
                .expect("menu textures");
            for (row, (sprite, _)) in icons.iter().enumerate() {
                assert_eq!(
                    sprite.image, expected_handles[row],
                    "row {row} phase {phase}"
                );
                assert_eq!(sprite.custom_size, Some(Vec2::splat(TILE_SIZE * 2.0)));
                let texture = images.get(&sprite.image).expect("party texture");
                assert_eq!((texture.width(), texture.height()), (16, 16));
                if !keys[row].starts_with("pokemon_overworld:") {
                    continue;
                }
                let slot = &snapshot.party.slots[row];
                let folder = input
                    .join("gfx/pokemon")
                    .join(slot.pokemon.species.id.to_ascii_lowercase());
                let bank = std::fs::read(folder.join("overworld.2bpp")).expect("reference bank");
                let palette = pokemon_overworld_batch_palette(
                    &std::fs::read(folder.join(if visible_pokemon_is_shiny(&slot.pokemon) {
                        "overworld-shiny.gbcpal"
                    } else {
                        "overworld.gbcpal"
                    }))
                    .expect("reference palette"),
                );
                assert_eq!(
                    texture.data.as_deref().expect("native RGBA"),
                    pokemon_overworld_batch_expected(
                        &bank,
                        &palette,
                        if phase == 0 { 0 } else { 3 },
                        false
                    )
                );
            }
            assert_eq!(
                world
                    .query::<(
                        &Sprite,
                        &Transform,
                        Option<&FieldCommandMarker>,
                        Option<&BattleCommandMarker>
                    )>()
                    .iter(world)
                    .filter(
                        |(sprite, transform, field, battle)| transform.translation.z == 3.9
                            && overlay_handles.contains(&sprite.image)
                            && if battle_surface {
                                battle.is_some()
                            } else {
                                field.is_some()
                            }
                    )
                    .count(),
                2,
                "held item and mail overlays remain layered above species icons"
            );
            if let Some(output) = &output {
                let surface = if battle_surface { "battle" } else { "field" };
                std::fs::create_dir_all(output).expect("party review folder");
                let canvas = pokemon_overworld_context_canvas(world, &images);
                assert_eq!(
                    canvas.get_pixel(0, 0).0,
                    [255; 4],
                    "party background covers the retained world and battlers"
                );
                image::imageops::resize(&canvas, 160, 144, image::imageops::FilterType::Nearest)
                    .save(output.join(format!("party-{surface}-phase-{phase}.png")))
                    .expect("native party capture");
                canvas
                    .save(output.join(format!("party-{surface}-phase-{phase}.4x.png")))
                    .expect("nearest party capture");
            }
            handles.push(expected_handles);
            world.insert_resource(images);
        }
        for row in 0..6 {
            assert_ne!(handles[0][row], handles[1][row], "two authored menu poses");
        }
    }
}
