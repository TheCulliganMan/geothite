// Source-backed palette/lifecycle expectations from pret/pokecrystal
// 5beda23ffa505f62e1dad7e3d7c214d1737b3358. No source program or art is embedded.
fn capture_palette_animation(capture: &VisibleCaptureAnimation) -> VisibleMoveAnimation {
    VisibleMoveAnimation {
        trigger_message: String::new(), move_id: format!("THROW_{}", capture.ball_id),
        animation_label: "BattleAnim_ThrowPokeBall".into(), player_move: true,
        started: true, waiting_for_hp: false, frame: capture.object_frame(),
        total_frames: capture.total_frames(), sound_events: Vec::new(), next_sound_event: 0,
        cry_events: Vec::new(), next_cry_event: 0, object_events: capture.object_events(),
        bg_events: Vec::new(), actor_species_override: None, actor_shiny_override: None,
    }
}

#[test]
fn battle_object_palette_ids_are_cartridge_indices() {
    for (id, name) in ["ENEMY", "PLAYER", "GRAY", "YELLOW", "RED", "GREEN", "BLUE", "BROWN"]
        .into_iter().enumerate() {
        let name = format!("PAL_BATTLE_OB_{name}");
        assert_eq!(battle_object_palette_id(&name).unwrap(), id as u8);
        assert_eq!(battle_object_palette_name(id as u8).unwrap(), name);
    }
    assert!(battle_object_palette_name(8).is_err());
    assert!(capture_source_item_byte("UNKNOWN_BALL").is_err());
}

#[test]
fn capture_palettes_use_source_item_identity_at_throw_lid_and_wobble() {
    let bundle = battle_anim_regression_bundle();
    for (ball, item, palette) in [("POKE_BALL", 5, 4), ("GREAT_BALL", 4, 6),
        ("ULTRA_BALL", 2, 3), ("MASTER_BALL", 1, 5), ("PARK_BALL", 0xb1, 2)] {
        let mut capture = capture_clock_fixture(ball, true, 4);
        for tick in [0, capture.opening_lid_frame(), capture.first_shake_check_frame(), capture.return_frame()] {
            capture.frame = tick;
            let animation = capture_palette_animation(&capture);
            let playback = visible_battle_objects(&bundle, &animation).unwrap();
            assert_eq!(playback.machine.read(battle_program::W_CUR_ITEM), item);
            let balls = playback.slots.iter().flatten().filter(|live| matches!(
                &animation.object_events[live.event_index].command,
                VisibleMoveObjectCommand::Spawn { object_id, .. } if object_id == "BATTLE_ANIM_OBJ_POKE_BALL"
            )).collect::<Vec<_>>();
            assert!(!balls.is_empty(), "{ball} tick {tick}");
            if tick == capture.opening_lid_frame() { assert_eq!(balls.len(), 2); }
            for live in balls {
                assert_eq!(live.bytes[5], palette, "{ball} tick {tick}");
                assert!(live.oam.entries.iter().all(|entry| entry[3] & 7 == palette));
            }
        }
    }
    let mut blocked = capture_clock_fixture("GREAT_BALL", false, 0);
    blocked.blocked = true;
    let playback = visible_battle_objects(&bundle, &capture_palette_animation(&blocked)).unwrap();
    assert_eq!(playback.machine.read(battle_program::W_CUR_ITEM), 4);
    assert_eq!(playback.slots[0].as_ref().unwrap().bytes[5], 6);
}

#[test]
fn capture_caught_palette_cleanup_changes_only_final_oam_low_attributes() {
    let bundle = battle_anim_regression_bundle();
    for ball in ["POKE_BALL", "GREAT_BALL", "ULTRA_BALL", "MASTER_BALL"] {
        let mut capture = capture_clock_fixture(ball, true, 4);
        capture.frame = capture.return_frame();
        let animation = capture_palette_animation(&capture);
        let mut playback = visible_battle_objects(&bundle, &animation).unwrap();
        // Exercise bank-bit clearing and preservation of all high attributes.
        for live in playback.slots.iter_mut().flatten() {
            for entry in &mut live.oam.entries { entry[3] |= 0xf8; }
        }
        let before = playback.slots.clone();
        let machine_before = (0..10).map(|slot| playback.machine.object(slot).to_vec()).collect::<Vec<_>>();
        let raw_oam_before = playback.machine.oam().to_vec();
        let before_completion = visible_capture_oam_slots(&playback, Some(&capture));
        capture.complete = true;
        // A rewind while complete is set must still show the pre-return frame.
        let at_return = visible_capture_oam_slots(&playback, Some(&capture));
        capture.frame = capture.total_frames();
        let retained = visible_capture_oam_slots(&playback, Some(&capture));
        for slot in 0..10 {
            let Some(old) = &before[slot] else { assert!(retained[slot].is_none()); continue };
            assert_eq!(before_completion[slot].as_ref().unwrap().oam.entries, old.oam.entries);
            assert_eq!(at_return[slot].as_ref().unwrap().oam.entries, old.oam.entries);
            let kept = retained[slot].as_ref().unwrap();
            assert_eq!(kept.bytes, old.bytes);
            assert_eq!(kept.oam.rows, old.oam.rows);
            assert_eq!(kept.oam.origin, old.oam.origin);
            assert_eq!((kept.frame, kept.frameset, kept.event_index, kept.spawn_frame),
                (old.frame, old.frameset, old.event_index, old.spawn_frame));
            for (actual, expected) in kept.oam.entries.iter().zip(&old.oam.entries) {
                assert_eq!(actual[..3], expected[..3]);
                assert_eq!(actual[3], expected[3] & 0xf0);
            }
        }
        assert_eq!((0..10).map(|slot| playback.machine.object(slot).to_vec()).collect::<Vec<_>>(), machine_before);
        assert_eq!(playback.machine.oam(), raw_oam_before);
        assert!(capture.enemy_hidden());
        capture.sprites_cleared = true;
        assert!(!capture.retained_objects_visible());
        assert!(capture.enemy_hidden());
    }
}

fn palette_raster_fixture() -> (AssetRoot, serde_json::Value, serde_json::Value, serde_json::Value, VisibleBattleObjectOam) {
    let assets = AssetRoot::new_temporary_in(std::env::temp_dir()).unwrap();
    let graphics = assets.runtime_assets().join("gfx/battle_anims");
    std::fs::create_dir_all(&graphics).unwrap();
    // Authored diagnostic pattern: source colors 0,1,2,3,0,1,2,3 per row.
    let tile: Vec<u8> = (0..8).flat_map(|_| [0x55, 0x33]).collect();
    std::fs::write(graphics.join("wind.2bpp"), tile).unwrap();
    std::fs::write(graphics.join("battle_anims.pal"),
        "; gray\nRGB 31,31,31\nRGB 25,25,25\nRGB 13,13,13\nRGB 0,0,0\n").unwrap();
    let object = serde_json::json!({"flags": 0, "gfx_id": "TEST", "palette": "PAL_BATTLE_OB_GRAY"});
    let frame = serde_json::json!({"oam_set": "TEST", "xflip": false, "yflip": false});
    let bundle = serde_json::json!({
        "gfx_table": {"TEST": [1, "TestWind"]},
        "gfx_sources": {"TestWind": "gfx/battle_anims/wind.2bpp"},
        "oam_sets": {"TEST": {"tile_offset": 0, "entries": [{
            "x": 0, "y": 0, "tile_id": 0, "xflip": false, "yflip": false, "obp": 0
        }]}}
    });
    let oam = VisibleBattleObjectOam { entries: vec![[16, 8, 0, 2]], rows: vec![[true; 8]], origin: (8, 16) };
    (assets, bundle, object, frame, oam)
}

#[test]
fn capture_palette_boundary_preserves_exact_gust_gray_pixels_and_obp_remap() {
    let (assets, bundle, object, frame, oam) = palette_raster_fixture();
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    for obp in [0xe4_u8, 0xe0] {
        let rendered = battle_anim_rendered_frame(&mut art, &bundle, &assets,
            "BATTLE_ANIM_OBJ_GUST", &object, "TEST", 0, &frame,
            false, false, false, None, obp, 0xe4, Some(&oam), &mut images).unwrap();
        let data = &images.get(&rendered.sprite.handle).unwrap().data;
        let colors = [[255,255,255,255], [206,206,206,255], [107,107,107,255], [0,0,0,255]];
        let expected = (0..64).flat_map(|pixel| {
            let source = pixel % 4;
            if source == 0 { [0,0,0,0] } else { colors[usize::from((obp >> (source * 2)) & 3)] }
        }).collect::<Vec<_>>();
        assert_eq!(*data, expected);
    }
    let real_bundle = battle_anim_regression_bundle();
    let shell = route36_battle_shell_for_render_regression();
    let snapshot = shell.shell.snapshot().unwrap();
    let (_, total_frames, sound_events, cry_events, object_events, bg_events) =
        visible_move_animation_definition(&snapshot, "GUST", 0).unwrap();
    let mut animation = capture_palette_animation(&capture_clock_fixture("POKE_BALL", true, 4));
    animation.move_id = "GUST".into(); animation.animation_label = "BattleAnim_Gust".into();
    animation.total_frames = total_frames; animation.sound_events = sound_events;
    animation.cry_events = cry_events; animation.object_events = object_events; animation.bg_events = bg_events;
    for tick in [1, 8, 22, 56, 82, 90] {
        animation.frame = tick;
        let playback = visible_battle_objects(&real_bundle, &animation).unwrap();
        for live in playback.slots.iter().flatten() { assert_eq!(live.bytes[5], 2); }
    }
}

#[test]
fn capture_palette_cache_uses_resolved_enemy_colors_and_keeps_transparency() {
    let (assets, bundle, object, frame, mut oam) = palette_raster_fixture();
    oam.entries[0][3] = 0;
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    let mut previous = None;
    for palette in [ [[255,255,255], [30,100,190], [20,40,80], [0,0,0]],
                     [[255,255,255], [190,100,30], [80,40,20], [0,0,0]] ] {
        let rendered = battle_anim_rendered_frame_with_battler_palettes(&mut art, &bundle, &assets,
            "BATTLE_ANIM_OBJ_POKE_BALL", &object, "TEST", 0, &frame,
            false, false, false, None, 0xe4, 0xe4, Some(&oam), &[Some(palette), None], &mut images).unwrap();
        let data = &images.get(&rendered.sprite.handle).unwrap().data;
        for (index, pixel) in data.chunks_exact(4).enumerate() {
            let color = index % 4;
            if color == 0 { assert_eq!(pixel, [0,0,0,0]); }
            else { assert_eq!(pixel, [palette[color][0], palette[color][1], palette[color][2], 255]); }
        }
        if let Some(previous) = previous { assert_ne!(rendered.sprite.handle.id(), previous); }
        previous = Some(rendered.sprite.handle.id());
    }
}

#[test]
fn capture_palette_restores_enemy_normal_and_shiny_colors_in_both_consumers() {
    let shell = route36_battle_shell_for_render_regression();
    let mut snapshot = shell.shell.snapshot().unwrap();
    let bundle = battle_anim_regression_bundle();
    let mut capture = capture_clock_fixture("MASTER_BALL", true, 4);
    capture.complete = true; capture.frame = capture.total_frames();
    let animation = capture_palette_animation(&capture);
    let playback = visible_battle_objects(&bundle, &animation).unwrap();
    let slots = visible_capture_oam_slots(&playback, Some(&capture));
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    let mut previous = None;
    for (dvs, shiny) in [(Dv::from_non_hp(1,1,1,1), false), (Dv::from_non_hp(10,10,10,10), true)] {
        let enemy = snapshot.battle.as_mut().unwrap();
        enemy.enemy_pokemon.species.id = "CYNDAQUIL".into();
        enemy.enemy_pokemon.dvs = dvs;
        enemy.enemy_transformed_species = None; enemy.enemy_transformed_dvs = None;
        let palettes = visible_battle_object_battler_palettes(&snapshot, &shell.asset_root, &animation, &slots).unwrap();
        assert_eq!(palettes[0], Some(load_pokemon_palette(&shell.asset_root, "cyndaquil", PokemonSpriteSide::Front, shiny).unwrap()));
        let live = slots.iter().flatten().next().unwrap();
        let VisibleMoveObjectCommand::Spawn { object_id, .. } = &animation.object_events[live.event_index].command else { panic!() };
        let rendered = battle_anim_rendered_frame_with_battler_palettes(&mut art, &bundle, &shell.asset_root,
            object_id, &bundle["objects"][object_id], live.frameset, live.frame,
            &bundle["framesets"][live.frameset][live.frame], false, false, false,
            Some(battle_object_palette_name(live.bytes[5] & 7).unwrap()), 0xe4, 0xe4,
            Some(&live.oam), &palettes, &mut images).unwrap();
        let source = capture_immersive_source_frame_with_capture(&snapshot, &animation,
            &[None, None], &mut art, &shell.asset_root, &mut images, Some(&capture)).unwrap();
        let source_ball = source.objects.iter().find(|object| object.object_id.as_ref() == object_id.as_str()).unwrap();
        assert_eq!(images.get(&source_ball.texture).unwrap().data, images.get(&rendered.sprite.handle).unwrap().data);
        if let Some(previous) = previous { assert_ne!(rendered.sprite.handle.id(), previous); }
        previous = Some(rendered.sprite.handle.id());
    }
}
