// The fixture supplies a legal party, carried/registered rod, and a real
// compiled shoreline. The cast itself always enters through controller SELECT.
fn fishing_origin_controller_at_shore() -> (VisibleShellController, TilePosition) {
    use crate::core::world::collision::{Terrain, describe_collision, sample_collision};

    let mut shell = route36_overworld_shell_for_battle_render_regression();
    let runtime = shell.shell.runtime().clone();
    let data = runtime.data();
    let map = data.overworld_map("Route32").expect("compiled fishing map");
    let tileset = data
        .tileset_collision(data.map_tileset_name("Route32").unwrap())
        .expect("compiled shoreline collision");
    let (width, height) = map.tile_bounds();
    let mut shoreline = None;
    'tiles: for y in 0..height {
        for x in 0..width {
            let shore = TilePosition::new(x as i16, y as i16);
            if !sample_collision(&map, &tileset, shore).is_some_and(|sample| {
                sample.permission == crate::core::world::collision::permissions::FLOOR
            }) || data.maps["Route32"].objects.iter().any(|object| {
                crate::core::world::session::object_tile_position_checked(object) == Some(shore)
            }) {
                continue;
            }
            for facing in [
                Direction::Right,
                Direction::Down,
                Direction::Left,
                Direction::Up,
            ] {
                let Some(water) = crate::core::world::movement::checked_move_by_stride(
                    shore,
                    facing,
                    crate::core::world::movement::StepOptions::default().stride_tiles,
                ) else {
                    continue;
                };
                if sample_collision(&map, &tileset, water).is_some_and(|sample| {
                    describe_collision(sample.permission).terrain == Terrain::Water
                }) {
                    shoreline = Some((shore, facing, water));
                    break 'tiles;
                }
            }
        }
    }
    let (shore, facing, water) = shoreline.expect("Route32 has an unoccupied land/water edge");
    {
        let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
        data.transition_overworld_session(
            state,
            overworld,
            "Route32",
            shore,
            crate::core::systems::map_context::SpawnMemoryUpdate::Preserve,
            &runtime.music_ids(),
        )
        .expect("place the fixture on the validated shore");
        overworld.set_player_facing(facing);
        state.overworld = crate::core::state::OverworldMemory::from_snapshot(&overworld.snapshot());
    }
    reset_visible_navigation_state(&mut shell);
    mark_runtime_snapshot_dirty(&mut shell);
    settle_visible_shell_smoke_until_idle(&mut shell).expect("settle shoreline arrival");
    shell
        .shell
        .add_bag_item("GOOD_ROD", 1)
        .expect("carry a legal rod");
    shell
        .shell
        .register_key_item("GOOD_ROD")
        .expect("register the carried rod");
    shell.shell.set_runtime_journal_enabled(true);
    let snapshot = shell.shell.snapshot().unwrap();
    assert_eq!(snapshot.overworld.map_name, "Route32");
    assert_eq!(snapshot.overworld.tile, shore);
    assert_eq!(snapshot.overworld.facing, facing);
    assert_eq!(snapshot.overworld.mode, MovementMode::Normal);
    assert_ne!(shore, water);
    assert_eq!(
        snapshot.progression.registered_key_item.as_deref(),
        Some("GOOD_ROD")
    );
    assert_eq!(
        snapshot
            .bag
            .key_items
            .iter()
            .find(|item| item.item_id == "GOOD_ROD")
            .unwrap()
            .quantity,
        1
    );
    assert!(snapshot.battle.is_none());
    (VisibleShellController { shell }, water)
}

fn fishing_origin_set_divider(controller: &mut VisibleShellController, samples: &[u8]) {
    let session = controller.shell.shell.session_mut();
    session.state_mut().random_state = crate::core::random::CrystalRandomState::default();
    *session.divider_mut_for_tests() =
        crate::core::random::RuntimeDividerSource::replay(samples.iter().copied());
}

fn fishing_origin_assert_consumed_divider(controller: &VisibleShellController, samples: usize) {
    let crate::core::random::RuntimeDividerSource::Replay(divider) =
        controller.shell.shell.session().divider_for_tests()
    else {
        panic!("fishing fixture must use its finite source DIV trace");
    };
    assert_eq!(divider.consumed(), samples);
    assert_eq!(divider.remaining(), 0);
}

fn fishing_origin_advance_to_notice(controller: &mut VisibleShellController) {
    // SELECT above is the actual controller transaction. Its renderer-neutral
    // settle loop does not advance fishing, so drive the same bounded clock as
    // native rendering, then return to controller input for the entry boundary.
    for _ in 0..113 {
        if controller
            .shell
            .visible_fishing_animation
            .as_ref()
            .is_some_and(|animation| animation.phase == VisibleFishingPhase::AwaitText)
        {
            break;
        }
        advance_visible_fishing_animation(&mut controller.shell);
    }
    assert_eq!(
        controller
            .shell
            .visible_fishing_animation
            .as_ref()
            .unwrap()
            .phase,
        VisibleFishingPhase::AwaitText,
        "the authored fishing clock must terminate within 113 frames",
    );
    controller
        .wait_frames(1)
        .expect("reveal the authored fishing notice");
}

#[test]
fn battle_origin_registered_fishing_freezes_real_shore_and_water_through_entry() {
    let (mut controller, water) = fishing_origin_controller_at_shore();
    // Select a real non-Magikarp slot to avoid its separate DV/length rejection
    // loop. This reads the compiled table; neither encounter odds nor data change.
    let (slot_roll, expected_species) = {
        let data = controller.shell.shell.runtime().data();
        let group = data
            .map_fishing_group("Route32")
            .unwrap()
            .expect("Route32 fishing group");
        let (_, rod) = data.fishing_rod_item("GOOD_ROD").unwrap();
        let group = &data.fishing.groups[group];
        assert!(group.bite_threshold > 0);
        let mut lower = 0_u8;
        group.rod_tables[rod]
            .slots
            .iter()
            .find_map(|slot| {
                let roll = lower;
                lower = slot.threshold.saturating_add(1);
                slot.species
                    .as_ref()
                    .filter(|species| !matches!(species.as_str(), "MAGIKARP" | "UNOWN"))
                    .filter(|_| roll <= slot.threshold)
                    .map(|species| (roll, species.clone()))
            })
            .expect("GOOD_ROD has a direct species without DV rejection")
    };
    // Random starts at add=sub=0. Two reads yield bite=0; the next two
    // yield the chosen slot; the final six yield no held item and both DVs=0.
    let samples = [
        0,
        0,
        0,
        0_u8.wrapping_sub(slot_roll),
        0,
        slot_roll,
        0,
        0,
        0,
        0,
    ];
    fishing_origin_set_divider(&mut controller, &samples);
    let source = controller.shell.shell.session().snapshot();
    let state_before = controller.shell.shell.session().state().clone();
    let commands_before = controller.shell.shell.retained_runtime_commands().len();
    assert!(controller.battle_presentation_origin().is_none());

    controller
        .press(GameButton::Select)
        .expect("cast the registered GOOD_ROD");

    let commands = &controller.shell.shell.retained_runtime_commands()[commands_before..];
    assert_eq!(
        commands.len(),
        1,
        "SELECT must perform one authoritative bag cast"
    );
    let crystal_assets::RuntimeMutationCommand::UseBagFishingRodInField(command) =
        crystal_assets::decode_runtime_mutation_command_frame(&commands[0], &state_before).unwrap()
    else {
        panic!("SELECT must use the production bag fishing transaction");
    };
    assert_eq!(command.item_id, "GOOD_ROD");
    assert_eq!(command.divider_trace.samples, samples);
    fishing_origin_assert_consumed_divider(&controller, samples.len());
    let battle_snapshot = controller.shell.shell.snapshot().unwrap();
    let battle = battle_snapshot
        .battle
        .as_ref()
        .expect("real fishing battle committed");
    assert_eq!(battle.battle_type, "BATTLETYPE_FISH");
    assert_eq!(battle.enemy_pokemon.species.id, expected_species);
    assert_eq!(
        controller
            .shell
            .visible_fishing_animation
            .as_ref()
            .unwrap()
            .phase,
        VisibleFishingPhase::Cast
    );
    let pending = controller
        .shell
        .battle_origin
        .pending
        .as_ref()
        .expect("origin precedes casting animation")
        .clone();
    assert!(controller.shell.battle_origin.active.is_none());
    assert_eq!(pending.source, source);
    assert_eq!(pending.kind, BattleOriginKind::Wild);
    assert_eq!(pending.battle_type, "BATTLETYPE_FISH");
    assert_eq!(
        pending.contact,
        Some(BattleOriginContact::FishingWaterTarget {
            map_id: source.map_name.clone(),
            tile: water,
        })
    );
    assert_eq!(pending.authoritative_step, None);
    assert_ne!(pending.source.tile, water);

    let session_after_cast = controller.shell.shell.session().clone();
    fishing_origin_advance_to_notice(&mut controller);
    assert_eq!(
        controller.shell.field_notice.as_deref(),
        Some("Oh!\nA bite!")
    );
    assert!(controller.shell.pending_field_battle_entry);
    assert!(Arc::ptr_eq(
        controller.shell.battle_origin.pending.as_ref().unwrap(),
        &pending
    ));
    assert_eq!(
        controller.shell.shell.session(),
        &session_after_cast,
        "presentation must not advance the authoritative pose, state, or DIV trace"
    );

    controller
        .press(GameButton::A)
        .expect("acknowledge the bite and enter battle");

    assert!(controller.shell.visible_fishing_animation.is_none());
    assert!(!controller.shell.pending_field_battle_entry);
    assert!(controller.shell.battle_origin.pending.is_none());
    assert!(Arc::ptr_eq(
        controller.shell.battle_origin.active.as_ref().unwrap(),
        &pending
    ));
    assert_eq!(
        controller.battle_presentation_origin(),
        Some(pending.as_ref())
    );
    assert!(
        controller.shell.battle_message_scene.is_some(),
        "real battle entry must retain its scene"
    );
    assert!(
        !controller.shell.battle_messages.is_empty(),
        "real battle entry must publish narration"
    );
    assert_eq!(controller.shell.last_error, None);
}

#[test]
fn battle_origin_registered_fishing_no_bite_never_publishes_an_origin() {
    let (mut controller, _) = fishing_origin_controller_at_shore();
    // From sub=0 this exact two-read Random returns 255. Verify that the
    // real compiled group permits no bite; do not override its chance/table.
    let runtime = controller.shell.shell.runtime().clone();
    let group = runtime
        .data()
        .map_fishing_group("Route32")
        .unwrap()
        .unwrap();
    assert!(runtime.data().fishing.groups[group].bite_threshold < 255);
    let samples = [0, 1];
    fishing_origin_set_divider(&mut controller, &samples);
    let mut replay = controller.shell.shell.session().clone();
    let state_before = replay.state().clone();
    let commands_before = controller.shell.shell.retained_runtime_commands().len();
    let source = controller.shell.shell.session().snapshot();

    controller
        .press(GameButton::Select)
        .expect("cast a real no-bite attempt");

    let commands = &controller.shell.shell.retained_runtime_commands()[commands_before..];
    assert_eq!(
        commands.len(),
        2,
        "no-bite also drains the ordinary ItemUse event queue"
    );
    let crystal_assets::RuntimeMutationCommand::UseBagFishingRodInField(command) =
        crystal_assets::decode_runtime_mutation_command_frame(&commands[0], &state_before).unwrap()
    else {
        panic!("no-bite SELECT must still execute the bag fishing transaction");
    };
    assert_eq!(command.item_id, "GOOD_ROD");
    assert_eq!(command.divider_trace.samples, samples);
    replay
        .apply_runtime_command_frame(&runtime, &commands[0])
        .unwrap();
    let crystal_assets::RuntimeMutationCommand::DrainScriptEventQueue(drain) =
        crystal_assets::decode_runtime_mutation_command_frame(&commands[1], replay.state())
            .unwrap()
    else {
        panic!("the only extra no-bite command must drain the completed ItemUse events");
    };
    assert_eq!(
        drain.queue,
        crystal_assets::RuntimeScriptEventQueue::ItemUse
    );
    replay
        .apply_runtime_command_frame(&runtime, &commands[1])
        .unwrap();
    assert_eq!(replay.state(), controller.shell.shell.session().state());
    fishing_origin_assert_consumed_divider(&controller, samples.len());
    let animation = controller.shell.visible_fishing_animation.as_ref().unwrap();
    assert!(!animation.bite && !animation.starts_battle);
    assert!(controller.battle_presentation_origin().is_none());
    assert!(controller.shell.shell.snapshot().unwrap().battle.is_none());

    fishing_origin_advance_to_notice(&mut controller);
    assert_eq!(
        controller.shell.field_notice.as_deref(),
        Some("Not even a nibble!")
    );
    assert!(!controller.shell.pending_field_battle_entry);
    assert!(controller.battle_presentation_origin().is_none());
    controller
        .press(GameButton::A)
        .expect("dismiss the no-bite notice");

    assert!(controller.shell.visible_fishing_animation.is_none());
    assert!(controller.shell.field_notice.is_none());
    assert!(controller.shell.battle_origin.pending.is_none());
    assert!(controller.shell.battle_origin.active.is_none());
    assert!(controller.shell.shell.snapshot().unwrap().battle.is_none());
    assert_eq!(controller.shell.shell.session().snapshot(), source);
    assert_eq!(controller.shell.last_error, None);
}
