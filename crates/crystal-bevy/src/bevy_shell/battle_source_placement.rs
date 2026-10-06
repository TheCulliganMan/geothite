use crystal_render_api::{VisualBattleSourceAssembly, VisualBattleSourcePlacement};

/// Annotate the already-presented object; the source interpreter remains the
/// only owner of callbacks, allocation, framesets, position and retirement.
/// This is bounded callback coverage, not a move-name placement heuristic.
fn immersive_battle_source_placement(
    animation: &VisibleMoveAnimation,
    bundle: &serde_json::Value,
    live: &VisibleBattleObjectFrame,
) -> Option<VisualBattleSourcePlacement> {
    let event = animation.object_events.get(live.event_index)?;
    let VisibleMoveObjectCommand::Spawn {
        object_id, param, ..
    } = &event.command
    else {
        return None;
    };
    let object = bundle.get("objects")?.get(object_id)?;
    if object_id != "BATTLE_ANIM_OBJ_EMBER"
        || object["object_id"].as_str() != Some("BATTLE_ANIM_OBJ_EMBER")
        || object["function"].as_str() != Some("BATTLE_ANIM_FUNC_EMBER")
        || object["frameset"].as_str() != Some("BATTLE_ANIM_FRAMESET_EMBER")
        || object["gfx_id"].as_str() != Some("BATTLE_ANIM_GFX_FIRE")
        || object["palette"].as_str() != Some("PAL_BATTLE_OB_RED")
        || object["flags"].as_u64() != Some(1)
        || object["fix_y"].as_u64() != Some(170)
        || live.spawn_frame != event.frame
        || live.bytes[1] != 1
        || live.bytes[2] != 170
        || battle_program::FUNCTIONS
            .get(usize::from(live.bytes[4]))
            .copied()
            != Some("BATTLE_ANIM_FUNC_EMBER")
        || live.bytes[5] != 4
        || live.bytes[9] != 0
        || live.bytes[10] != 0
        || live.bytes[11] != *param
        || battle_program::FRAMESETS
            .get(usize::from(live.bytes[3]))
            .copied()
            != Some(live.frameset)
        || usize::from(live.bytes[13]) != live.frame
    {
        return None;
    }

    // Oracle cases 130–137 attest both turns. State 0 only selects the high
    // param nibble and returns: its first presented sample is state 1 or 3,
    // at the original coordinates. State 2 may retain this tick's final OAM
    // after deinitialization, so bytes[0] must not be used as a visibility test.
    let stationary = match (*param, live.bytes[14], live.frameset) {
        (0x12..=0x14, 1 | 2, "BATTLE_ANIM_FRAMESET_EMBER") => false,
        (0x30, 3, "BATTLE_ANIM_FRAMESET_EMBER")
        | (0x30, 4, "BATTLE_ANIM_FRAMESET_FLAMETHROWER") => true,
        _ => return None,
    };
    if !immersive_ember_complete_oam(bundle, live) {
        return None;
    }
    let from = if animation.player_move {
        VisualBattleSide::Player
    } else {
        VisualBattleSide::Enemy
    };
    if stationary {
        // Each burst keeps its source-relative location on the same target
        // chart. It is not a fabricated continuation of a retired traveler.
        Some(VisualBattleSourcePlacement::BattlerLocal {
            side: if animation.player_move {
                VisualBattleSide::Enemy
            } else {
                VisualBattleSide::Player
            },
        })
    } else {
        Some(VisualBattleSourcePlacement::IndependentTransport {
            from,
            assembly: VisualBattleSourceAssembly {
                event_index: live.event_index,
                spawn_frame: live.spawn_frame,
            },
            // InitBattleAnimBuffer already applied the turn fix. Register
            // against its unclipped LCD origin, never the cropped center.
            pivot: Vec2::new(
                (live.oam.origin.0 - 8) as f32,
                (live.oam.origin.1 - 16) as f32,
            ),
        })
    }
}

fn immersive_ember_complete_oam(
    bundle: &serde_json::Value,
    live: &VisibleBattleObjectFrame,
) -> bool {
    let frame = &bundle["framesets"][live.frameset][live.frame];
    let (names, offsets): (&[&str], &[(i32, i32)]) = match live.frameset {
        "BATTLE_ANIM_FRAMESET_EMBER" => (
            &["BATTLE_ANIM_OAMSET_0F", "BATTLE_ANIM_OAMSET_10"],
            &[(-4, -4)],
        ),
        "BATTLE_ANIM_FRAMESET_FLAMETHROWER" => (
            &["BATTLE_ANIM_OAMSET_0A", "BATTLE_ANIM_OAMSET_0E"],
            &[(-8, -8), (0, -8), (-8, 0), (0, 0)],
        ),
        _ => return false,
    };
    if frame["command"].as_str() != Some("frame")
        || !frame["oam_set"]
            .as_str()
            .is_some_and(|name| names.contains(&name))
        || frame["xflip"].as_bool() != Some(false)
        || frame["yflip"].as_bool() != Some(false)
        || live.oam.entries.len() != offsets.len()
        || live.oam.rows.len() != offsets.len()
    {
        return false;
    }
    live.oam
        .entries
        .iter()
        .zip(offsets)
        .all(|(entry, &(x, y))| {
            i32::from(entry[1]) == (live.oam.origin.0 + x).rem_euclid(256)
                && i32::from(entry[0]) == (live.oam.origin.1 + y).rem_euclid(256)
        })
}

/// Unsupported pieces could belong to a larger assembly. Keep the complete
/// visible source canvas on its existing projection unless every piece is
/// proven; mixing per-object charts with that fallback would tear the effect.
fn retain_complete_immersive_source_placement(objects: &mut [VisualBattleSourceObject]) {
    if objects.iter().any(|object| object.placement.is_none()) {
        for object in objects {
            object.placement = None;
        }
    }
}

#[cfg(test)]
mod source_placement_tests {
    use super::*;

    fn fixture() -> BevyRuntimeShell {
        let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let asset_root = AssetRoot::new(&repository);
        let pack_path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| repository.join("content-packs/core-modular.crystalpack"));
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack_path)
            .expect("source placement tests require the actual external CRYSTAL_RENDER_TEST_PACK");
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&asset_root, loaded).unwrap();
        let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
        let mut shell = initialize_bevy_runtime_shell(
            asset_root,
            runtime,
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier,
                map_name: "Route36".into(),
                tile_x: 20,
                tile_y: 8,
            },
            BevyShellConfig::default(),
        )
        .unwrap();
        shell
            .shell
            .add_party_pokemon(
                "CYNDAQUIL",
                10,
                None,
                None,
                "SOURCE_PLACEMENT_REGRESSION",
                1,
                Dv::from_non_hp(10, 10, 10, 10),
            )
            .unwrap();
        settle_visible_shell_smoke_until_idle(&mut shell).unwrap();
        shell
            .shell
            .start_scripted_wild_battle("Route36", "WateredWeirdTreeScript", 12)
            .unwrap();
        prepare_visible_battle_entry(&mut shell).unwrap();
        shell.visible_battle_transition = None;
        shell.visible_battle_sliding_intro = None;
        shell.visible_send_out_animation = None;
        shell.battle_entry_messages_remaining = 0;
        shell.battle_player_send_out_pending = false;
        shell.battle_enemy_send_out_pending = false;
        shell.battle_messages.clear();
        shell.battle_message_scenes.clear();
        shell.battle_text_reveal = None;
        shell.visible_move_animations.clear();
        shell
    }

    fn animation(snapshot: &RuntimeShellSnapshot, move_id: &str) -> VisibleMoveAnimation {
        let (animation_label, total_frames, sound_events, cry_events, object_events, bg_events) =
            visible_move_animation_definition(snapshot, move_id, 0).unwrap();
        VisibleMoveAnimation {
            trigger_message: String::new(),
            move_id: move_id.into(),
            animation_label,
            player_move: true,
            started: true,
            waiting_for_hp: false,
            frame: 0,
            total_frames,
            sound_events,
            next_sound_event: 0,
            cry_events,
            next_cry_event: 0,
            object_events,
            bg_events,
            actor_species_override: None,
            actor_shiny_override: None,
        }
    }

    fn spawn(frame: u16, x: i16, y: i16, param: u8) -> VisibleMoveObjectEvent {
        VisibleMoveObjectEvent {
            frame,
            command: VisibleMoveObjectCommand::Spawn {
                object_id: "BATTLE_ANIM_OBJ_EMBER".into(),
                x,
                y,
                param,
            },
        }
    }

    fn expected_pivot(live: &VisibleBattleObjectFrame, player: bool) -> Vec2 {
        let (x, y) = (f32::from(live.bytes[7]), f32::from(live.bytes[8]));
        if player {
            Vec2::new(x - 8.0, y - 16.0)
        } else {
            Vec2::new(172.0 - x, 154.0 - y)
        }
    }

    #[test]
    fn source_placement_ember_matches_pack_oracle_on_both_turns() {
        use crate::battle_anim_machine::oracle_oam;
        let shell = fixture();
        let snapshot = shell.shell.snapshot().unwrap();
        let bundle =
            battle_anim_render_bundle(&mut RenderedTilesetArt::default(), &snapshot).unwrap();
        let mut descriptor = animation(&snapshot, "EMBER");
        let mut count = 0;
        for (case, &(function, object, x, y, param, player)) in oracle_oam::CASES.iter().enumerate()
        {
            if function != 16 {
                continue;
            }
            count += 1;
            assert_eq!(object, 11);
            descriptor.player_move = player;
            descriptor.object_events = vec![spawn(0, i16::from(x), i16::from(y), param)];
            let mut machine = BattleObjectMachine::new(player);
            install_battle_object_data(&mut machine, &bundle).unwrap();
            machine.write(battle_program::W_CUR_ITEM, 5);
            let mut definition = [0; 6];
            for (index, byte) in definition.iter_mut().enumerate().take(5) {
                *byte = machine.read(
                    battle_program::BATTLE_ANIM_OBJECTS + u16::from(object) * 6 + index as u16,
                );
            }
            machine.initialize(0, 1, definition, x, y, param);
            for frame in 0..oracle_oam::FRAMES {
                machine.begin_oam();
                machine.step_object(0).unwrap();
                machine.oam_update(0).unwrap();
                let mut bytes = machine.object(0)[..17].to_vec();
                bytes.extend(
                    [
                        battle_program::W_O_B_P0,
                        battle_program::H_L_C_D_C_POINTER,
                        battle_program::H_L_Y_OVERRIDE_START,
                        battle_program::H_L_Y_OVERRIDE_END,
                    ]
                    .map(|address| machine.read(address)),
                );
                bytes.extend(machine.oam());
                let length = machine.read(battle_program::W_BATTLE_ANIM_O_A_M_POINTER_LO);
                bytes.push(length);
                let hash = bytes
                    .into_iter()
                    .fold(14695981039346656037_u64, |hash, byte| {
                        (hash ^ u64::from(byte)).wrapping_mul(1099511628211)
                    });
                let offset = (case * oracle_oam::FRAMES + frame) * 8;
                assert_eq!(
                    hash.to_le_bytes(),
                    oracle_oam::RECORDS[offset..offset + 8],
                    "case {case} frame {frame}"
                );
                let object_bytes: [u8; 24] = machine.object(0).try_into().unwrap();
                let entries: Vec<[u8; 4]> = machine.oam()[..usize::from(length)]
                    .chunks_exact(4)
                    .map(|entry| entry.try_into().unwrap())
                    .collect();
                let origin = (
                    i32::from(
                        machine
                            .read(battle_program::W_BATTLE_ANIM_TEMP_X_COORD)
                            .wrapping_add(
                                machine.read(battle_program::W_BATTLE_ANIM_TEMP_X_OFFSET),
                            ),
                    ),
                    i32::from(
                        machine
                            .read(battle_program::W_BATTLE_ANIM_TEMP_Y_COORD)
                            .wrapping_add(
                                machine.read(battle_program::W_BATTLE_ANIM_TEMP_Y_OFFSET),
                            ),
                    ),
                );
                let live = VisibleBattleObjectFrame {
                    event_index: 0,
                    spawn_frame: 0,
                    bytes: object_bytes,
                    frameset: battle_program::FRAMESETS[usize::from(object_bytes[3])],
                    frame: usize::from(object_bytes[13]),
                    oam: VisibleBattleObjectOam {
                        rows: vec![[true; 8]; entries.len()],
                        entries,
                        origin,
                    },
                };
                assert!(
                    immersive_battle_source_placement(&descriptor, &bundle, &live).is_some(),
                    "case {case} frame {frame}"
                );
                if frame == 0 {
                    assert_eq!(&live.bytes[7..9], &[x, y], "state 0 must not move");
                    assert_eq!(live.bytes[14], param >> 4);
                    assert_eq!(live.frameset, "BATTLE_ANIM_FRAMESET_EMBER");
                }
                if let Some(VisualBattleSourcePlacement::IndependentTransport { pivot, .. }) =
                    immersive_battle_source_placement(&descriptor, &bundle, &live)
                {
                    assert_eq!(pivot, expected_pivot(&live, player));
                }
            }
        }
        assert_eq!(count, 8, "all audited parameters and both source turns");
    }

    #[test]
    fn source_placement_ember_timeline_preserves_pixels_positions_and_terminal_contacts() {
        let shell = fixture();
        let snapshot = shell.shell.snapshot().unwrap();
        let bundle =
            battle_anim_render_bundle(&mut RenderedTilesetArt::default(), &snapshot).unwrap();
        for player in [true, false] {
            let mut descriptor = animation(&snapshot, "EMBER");
            descriptor.player_move = player;
            assert_eq!(descriptor.total_frames, 62);
            let spawns: Vec<_> = descriptor
                .object_events
                .iter()
                .filter_map(|event| {
                    matches!(event.command, VisibleMoveObjectCommand::Spawn { .. })
                        .then_some(event.frame)
                })
                .collect();
            assert_eq!(spawns, [1, 6, 11, 28, 28, 28]);
            let mut art = RenderedTilesetArt::default();
            let mut classic_art = RenderedTilesetArt::default();
            let mut images = Assets::<Image>::default();
            for tick in [0, 1, 6, 11, 27, 28, 29, 61] {
                descriptor.frame = tick;
                let before = descriptor.clone();
                let baseline = visible_battle_objects(&bundle, &descriptor).unwrap();
                let source = capture_immersive_source_frame(
                    &snapshot,
                    &descriptor,
                    &[None, None],
                    &mut art,
                    &shell.asset_root,
                    &mut images,
                )
                .unwrap();
                assert_eq!(descriptor, before);
                assert_eq!(
                    source.objects.len(),
                    match tick {
                        0 => 0,
                        1 => 1,
                        6 => 2,
                        28 => 6,
                        _ => 3,
                    },
                    "tick {tick}"
                );
                assert!(
                    source
                        .objects
                        .iter()
                        .all(|object| object.placement.is_some())
                );
                let playback = art.battle_object_runtime.as_ref().unwrap();
                for slot in 0..10 {
                    assert_eq!(playback.machine.object(slot), baseline.machine.object(slot));
                }
                assert_eq!(playback.machine.oam(), baseline.machine.oam());
                let instructions = playback.machine.instructions;
                for object in &source.objects {
                    let live = baseline.slots[object.slot].as_ref().unwrap();
                    let metadata = &bundle["objects"][object.object_id.as_ref()];
                    let rendered = battle_anim_rendered_frame_with_battler_palettes(
                        &mut classic_art,
                        &bundle,
                        &shell.asset_root,
                        object.object_id.as_ref(),
                        metadata,
                        live.frameset,
                        live.frame,
                        &bundle["framesets"][live.frameset][live.frame],
                        !player,
                        false,
                        false,
                        Some(battle_object_palette_name(live.bytes[5] & 7).unwrap()),
                        0xe4,
                        0xe4,
                        Some(&live.oam),
                        &[None, None],
                        &mut images,
                    )
                    .unwrap();
                    assert_eq!(
                        images.get(&object.texture).unwrap().data,
                        images.get(&rendered.sprite.handle).unwrap().data,
                        "pixels at tick {tick}"
                    );
                    let size = rendered.sprite.size / (TILE_SIZE / SOURCE_TILE_SIZE as f32);
                    let top_left = Vec2::new(
                        (live.oam.origin.0 - 8 + i32::from(rendered.offset_x)) as f32,
                        (live.oam.origin.1 - 16 + i32::from(rendered.offset_y)) as f32,
                    );
                    let clip = visible_battle_object_clip(top_left, size).unwrap();
                    assert_eq!(object.center, clip.screen.center());
                    assert_eq!(object.size, clip.screen.size());
                    assert_eq!(
                        object.uv_rect,
                        Rect::from_corners(clip.texture.min / size, clip.texture.max / size)
                    );
                    if live.bytes[11] == 0x30 {
                        assert_eq!(
                            object.placement,
                            Some(VisualBattleSourcePlacement::BattlerLocal {
                                side: if player {
                                    VisualBattleSide::Enemy
                                } else {
                                    VisualBattleSide::Player
                                },
                            })
                        );
                        assert_eq!(
                            object.size,
                            Vec2::splat(if tick == 28 { 8.0 } else { 16.0 })
                        );
                    } else {
                        assert_eq!(
                            object.placement,
                            Some(VisualBattleSourcePlacement::IndependentTransport {
                                from: if player {
                                    VisualBattleSide::Player
                                } else {
                                    VisualBattleSide::Enemy
                                },
                                assembly: VisualBattleSourceAssembly {
                                    event_index: live.event_index,
                                    spawn_frame: live.spawn_frame,
                                },
                                pivot: expected_pivot(live, player),
                            })
                        );
                    }
                }
                if tick == 28 {
                    let contacts: Vec<_> = baseline
                        .slots
                        .iter()
                        .flatten()
                        .filter(|live| live.bytes[11] != 0x30)
                        .map(|live| (live.bytes[7], live.bytes[8], live.bytes[0], live.bytes[14]))
                        .collect();
                    assert_eq!(
                        contacts,
                        [(116, 70, 0, 2), (136, 64, 0, 2), (112, 68, 0, 2)]
                    );
                }
                let repeated = capture_immersive_source_frame(
                    &snapshot,
                    &descriptor,
                    &[None, None],
                    &mut art,
                    &shell.asset_root,
                    &mut images,
                )
                .unwrap();
                assert_eq!(repeated, source);
                assert_eq!(
                    art.battle_object_runtime
                        .as_ref()
                        .unwrap()
                        .machine
                        .instructions,
                    instructions
                );
            }
        }
    }

    #[test]
    fn source_placement_clipping_keeps_pivot_and_reused_slot_gets_new_event_identity() {
        let shell = fixture();
        let snapshot = shell.shell.snapshot().unwrap();
        let bundle =
            battle_anim_render_bundle(&mut RenderedTilesetArt::default(), &snapshot).unwrap();
        for player in [true, false] {
            let mut descriptor = animation(&snapshot, "EMBER");
            descriptor.player_move = player;
            descriptor.object_events = vec![
                spawn(1, if player { 10 } else { 170 }, 80, 0x12),
                VisibleMoveObjectEvent {
                    frame: 2,
                    command: VisibleMoveObjectCommand::Clear,
                },
                spawn(3, 64, 96, 0x12),
            ];
            let mut art = RenderedTilesetArt::default();
            let mut images = Assets::<Image>::default();
            for (tick, event_index) in [(1, 0), (3, 2)] {
                descriptor.frame = tick;
                let source = capture_immersive_source_frame(
                    &snapshot,
                    &descriptor,
                    &[None, None],
                    &mut art,
                    &shell.asset_root,
                    &mut images,
                )
                .unwrap();
                assert_eq!(source.objects.len(), 1);
                let object = &source.objects[0];
                assert_eq!(object.slot, 0, "source reuses the retired slot");
                let live = art.battle_object_runtime.as_ref().unwrap().slots[0]
                    .as_ref()
                    .unwrap();
                let Some(VisualBattleSourcePlacement::IndependentTransport {
                    assembly, pivot, ..
                }) = object.placement
                else {
                    panic!("audited traveler");
                };
                assert_eq!(
                    assembly,
                    VisualBattleSourceAssembly {
                        event_index,
                        spawn_frame: tick
                    }
                );
                assert_eq!(pivot, expected_pivot(live, player));
                if tick == 1 {
                    assert_eq!(object.size.x, 6.0);
                    assert_eq!(pivot.x, 2.0);
                    assert_eq!(
                        object.center.x, 3.0,
                        "LCD clipping must not become registration"
                    );
                    assert_eq!(object.uv_rect.min.x, 0.25);
                }
                assert!(immersive_battle_source_placement(&descriptor, &bundle, live).is_some());
            }
        }
    }

    #[test]
    fn source_placement_rejects_unsupported_contracts_and_mixed_visible_sets() {
        let shell = fixture();
        let snapshot = shell.shell.snapshot().unwrap();
        let bundle =
            battle_anim_render_bundle(&mut RenderedTilesetArt::default(), &snapshot).unwrap();
        let mut descriptor = animation(&snapshot, "EMBER");
        descriptor.frame = 1;
        let live = visible_battle_objects(&bundle, &descriptor).unwrap().slots[0]
            .clone()
            .unwrap();
        assert!(immersive_battle_source_placement(&descriptor, &bundle, &live).is_some());
        for (byte, value) in [
            (1, 0),
            (2, 128),
            (4, 7),
            (5, 2),
            (9, 1),
            (10, 1),
            (11, 0x15),
            (14, 0),
            (14, 3),
            (14, 4),
        ] {
            let mut unsupported = live.clone();
            unsupported.bytes[byte] = value;
            assert!(
                immersive_battle_source_placement(&descriptor, &bundle, &unsupported).is_none()
            );
        }
        let mut partial = live.clone();
        partial.oam.entries.push(partial.oam.entries[0]);
        assert!(immersive_battle_source_placement(&descriptor, &bundle, &partial).is_none());
        for (field, value) in [
            ("function", serde_json::json!("BATTLE_ANIM_FUNC_FIRE_SPIN")),
            ("flags", serde_json::json!(0)),
            ("fix_y", serde_json::json!(128)),
            (
                "frameset",
                serde_json::json!("BATTLE_ANIM_FRAMESET_FLAMETHROWER"),
            ),
        ] {
            let mut altered = (*bundle).clone();
            altered["objects"]["BATTLE_ANIM_OBJ_EMBER"][field] = value;
            assert!(immersive_battle_source_placement(&descriptor, &altered, &live).is_none());
        }
        // Same Ember frameset does not prove the different callbacks used by
        // Fire Spin and Flame Wheel, nor permit a mixed source assembly split.
        for move_id in ["FIRE_SPIN", "FLAME_WHEEL"] {
            let mut other = animation(&snapshot, move_id);
            other.frame = other
                .object_events
                .iter()
                .find(|event| matches!(event.command, VisibleMoveObjectCommand::Spawn { .. }))
                .unwrap()
                .frame;
            let source = capture_immersive_source_frame(
                &snapshot,
                &other,
                &[None, None],
                &mut RenderedTilesetArt::default(),
                &shell.asset_root,
                &mut Assets::default(),
            )
            .unwrap();
            assert!(!source.objects.is_empty());
            assert!(
                source
                    .objects
                    .iter()
                    .all(|object| object.placement.is_none())
            );
        }
        let mut mixed = descriptor.clone();
        mixed.object_events = vec![
            spawn(1, 64, 96, 0x12),
            VisibleMoveObjectEvent {
                frame: 1,
                command: VisibleMoveObjectCommand::Spawn {
                    object_id: "BATTLE_ANIM_OBJ_HIT".into(),
                    x: 120,
                    y: 68,
                    param: 0,
                },
            },
        ];
        let source = capture_immersive_source_frame(
            &snapshot,
            &mixed,
            &[None, None],
            &mut RenderedTilesetArt::default(),
            &shell.asset_root,
            &mut Assets::default(),
        )
        .unwrap();
        assert_eq!(source.objects.len(), 2);
        assert!(
            source
                .objects
                .iter()
                .all(|object| object.placement.is_none())
        );
    }

    #[test]
    fn source_placement_retains_tick_61_and_retires_at_exclusive_tick_62() {
        let mut shell = fixture();
        let snapshot = shell.shell.snapshot().unwrap();
        for player in [true, false] {
            let mut descriptor = animation(&snapshot, "EMBER");
            descriptor.player_move = player;
            descriptor.frame = 61;
            descriptor.next_sound_event = descriptor.sound_events.len();
            descriptor.next_cry_event = descriptor.cry_events.len();
            shell.visible_move_animations.push_back(descriptor);
            let mut art = RenderedTilesetArt::default();
            let mut images = Assets::<Image>::default();
            let extract = |shell: &BevyRuntimeShell,
                           art: &mut RenderedTilesetArt,
                           images: &mut Assets<Image>| {
                let mut world = World::new();
                let mut queue = bevy::ecs::world::CommandQueue::default();
                capture_presented_battle(
                    &mut Commands::new(&mut queue, &world),
                    &snapshot,
                    shell,
                    true,
                    art,
                    images,
                )
                .unwrap();
                queue.apply(&mut world);
                world.remove_resource::<VisualBattleFrame>().unwrap()
            };
            let frame = extract(&shell, &mut art, &mut images);
            assert_eq!(frame.source.as_ref().unwrap().frame, 61);
            assert_eq!(frame.source.as_ref().unwrap().objects.len(), 3);
            advance_visible_move_animation(&mut shell).unwrap();
            assert!(shell.visible_move_animations.is_empty());
            assert!(extract(&shell, &mut art, &mut images).source.is_none());
        }
    }
}
