#[test]
fn renderer_neutral_empty_party_field_previews_walk_without_invalid_wild_battles() {
    use crate::core::world::session::{
        coord_event_tile_position_checked, object_tile_position_checked, warp_tile_position_checked,
    };
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let asset_root = AssetRoot::new(repository);
    let runtime = workspace_desktop_runtime(&asset_root);
    let data = runtime.data();
    for map_name in ["IlexForest", "DarkCaveVioletEntrance"] {
        let module = &data.maps[map_name];
        let land_encounters = matches!(
            data.runtime_map_metadata_for_name(map_name)
                .unwrap()
                .environment
                .as_str(),
            "CAVE" | "DUNGEON"
        );
        let map = data.overworld_map(map_name).unwrap();
        let (width, height) = map.tile_bounds();
        let stride = crate::core::world::movement::StepOptions::default().stride_tiles;
        let safe = |tile: TilePosition| {
            !module.objects.iter().any(|object| {
                object_tile_position_checked(object).is_some_and(|other| {
                    (i32::from(other.x) - i32::from(tile.x)).abs()
                        + (i32::from(other.y) - i32::from(tile.y)).abs()
                        <= 8
                })
            }) && !module
                .events
                .warps
                .iter()
                .any(|event| warp_tile_position_checked(event) == Some(tile))
                && !module
                    .events
                    .coord_events
                    .iter()
                    .any(|event| coord_event_tile_position_checked(event) == Some(tile))
        };
        let mut pair = None;
        'find: for y in 0..height {
            for x in 0..width {
                let from = TilePosition::new(x as i16, y as i16);
                let Some(to) = crate::core::world::movement::checked_move_by_stride(
                    from,
                    Direction::Right,
                    stride,
                ) else {
                    continue;
                };
                if !safe(from) || !safe(to) {
                    continue;
                }
                if [from, to].into_iter().all(|tile| {
                    data.overworld_session(map_name, tile, 0)
                        .is_ok_and(|session| {
                            session
                                .current_encounter_surface_checked_with_land_encounters(
                                    land_encounters,
                                )
                                .is_ok_and(|surface| surface.is_some())
                        })
                }) {
                    pair = Some((from, to));
                    break 'find;
                }
            }
        }
        let (from, to) = pair.expect("authored unobstructed walking encounter tiles");
        for party in ["empty", "fainted", "starter"] {
            let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
            let shell = initialize_bevy_runtime_shell(
                asset_root.clone(),
                runtime.clone(),
                BevyShellStart::NewGameAtRuntimeTile {
                    spawn_identifier,
                    map_name: map_name.into(),
                    tile_x: from.x,
                    tile_y: from.y,
                },
                BevyShellConfig {
                    smoke_player_name: Some("CHRIS".into()),
                    quick_save_path: None,
                    ..default()
                },
            )
            .unwrap();
            let mut controller = VisibleShellController { shell };
            complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
                .unwrap();
            let initial = controller.snapshot().unwrap();
            assert!(initial.party.slots.is_empty());
            if party != "empty" {
                controller
                    .shell
                    .shell
                    .add_party_pokemon(
                        "CHIKORITA",
                        5,
                        None,
                        None,
                        &initial.trainer.player_name,
                        initial.trainer.player_id,
                        Dv::default(),
                    )
                    .unwrap();
                if party == "fainted" {
                    let state = controller.shell.shell.session_mut().state_mut();
                    state.storage.party.pokemon[0].as_mut().unwrap().hp = 0;
                    state.sync_party_from_storage();
                }
            }
            controller
                .shell
                .shell
                .session_mut()
                .state_mut()
                .random_state = Default::default();
            *controller.shell.shell.session_mut().divider_mut_for_tests() =
                crate::core::random::RuntimeDividerSource::replay(vec![0; 32768]);
            reset_visible_navigation_state(&mut controller.shell);
            mark_runtime_snapshot_dirty(&mut controller.shell);
            settle_visible_shell_smoke_until_idle(&mut controller.shell).unwrap();
            let before = controller.snapshot().unwrap();
            let mut moved = false;
            let mut entered = false;
            for _ in 0..32 {
                let tile = controller.snapshot().unwrap().overworld.tile;
                assert!([from, to].contains(&tile));
                controller
                    .press(if tile == from {
                        GameButton::Right
                    } else {
                        GameButton::Left
                    })
                    .expect("ordinary authored walking must not create an invalid battle");
                let next = controller.snapshot().unwrap();
                moved |= next.overworld.tile != tile;
                assert!(controller.shell.last_error.is_none());
                if let Some(battle) = next.battle {
                    assert_eq!(party, "starter", "{map_name}");
                    assert!(matches!(
                        controller.shell.shell.session().state().battle,
                        crate::core::state::BattleMemory::Wild { .. }
                    ));
                    assert_eq!(
                        controller
                            .shell
                            .shell
                            .session()
                            .state()
                            .battle_active_party_index,
                        Some(0)
                    );
                    assert!(!battle.commands.player_move_slots.is_empty());
                    entered = true;
                    break;
                }
                assert_eq!(next.party, before.party);
                assert_eq!(
                    controller
                        .shell
                        .shell
                        .session()
                        .state()
                        .repel_steps_remaining,
                    0
                );
            }
            assert!(moved, "{map_name} {party}: preserve walking");
            assert_eq!(entered, party == "starter", "{map_name} {party}");
        }
    }
}
