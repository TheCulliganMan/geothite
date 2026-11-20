//! Search every installed Minnesota BGEVENT_ITEM from its adjacent public approach.
use std::collections::BTreeSet;

use crystal_assets::{AssetRoot, MapModule};
use crystal_core::input::GameButton;
use crystal_core::map::BackgroundEvent;
use crystal_core::models::item::{ITEM_POCKET_BALL, ITEM_POCKET_ITEM};
use crystal_core::models::{
    BALL_POCKET_CAPACITY, Dv, ITEM_POCKET_CAPACITY, LearnedMove, MAX_ITEM_STACK,
};
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_core::world::collision::PlayerTraversalState;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::super::super::{
    BevyShellConfig, BevyShellStart, VisibleFieldItemPresentation, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::super::badge_trails::load;
use super::super::interior_life::collision;
use super::super::minnesota_field_use::clear_whirlpool;
use super::super::r10_gallery::{continue_save, press, settle};
use super::super::winona::travel;

const EDDY: &str = "MN_LEECH_EDDY_CACHE";
const LAKE: &str = "MnLeechLake";

fn approach_eddy(c: &mut VisibleShellController) {
    travel(c, GameButton::Right, LAKE, (59, 20));
    let world = c.shell.shell.session().overworld();
    assert_eq!(
        world.player.facing,
        Direction::Right,
        "successful eastward paddle faces the eddy"
    );
    let target = c
        .shell
        .shell
        .runtime()
        .data()
        .field_block_target_metatile_in_front(world)
        .expect("eddy metatile in front");
    assert_eq!(target, (30, 10));
    let index = world
        .map
        .metatile_index(target.0 as i16, target.1 as i16)
        .expect("eddy block");
    assert_eq!(
        world.map.metatile_ids[index], 0x07,
        "the undispersed current must face the paddler"
    );
    clear_whirlpool(c);
    travel(c, GameButton::Right, LAKE, (60, 20));
    assert_eq!(
        c.shell.shell.session().overworld().player.mode,
        MovementMode::Surf
    );
}

// Continue reloads temporary field-move blocks. Save off the current and disperse it again.
fn continue_at_site(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    c: &mut VisibleShellController,
    eddy: bool,
) -> VisibleShellController {
    if eddy {
        travel(c, GameButton::Left, LAKE, (59, 20));
        travel(c, GameButton::Left, LAKE, (58, 20));
    }
    let mut loaded = continue_save(root.clone(), runtime.clone(), c);
    if eddy {
        approach_eddy(&mut loaded);
    }
    loaded
}

fn fixture(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    room: &MapModule,
    event: &BackgroundEvent,
) -> VisibleShellController {
    use Direction::{Down, Left, Right, Up};
    let target = TilePosition::new(event.x as i16, event.y as i16);
    let (at, facing, role) = [
        "FLOOR",
        "TALL_GRASS",
        "LONG_GRASS",
        "BRIDGE_EW_DECK",
        "BRIDGE_NS_DECK",
        "WATER",
        "BRIDGE_EW_WATER",
        "BRIDGE_NS_WATER",
        "WHIRLPOOL",
    ]
    .into_iter()
    .find_map(|role| {
        [(Up, Down), (Down, Up), (Left, Right), (Right, Left)]
            .into_iter()
            .find_map(|(side, facing)| {
                let at = target.moved(side);
                let empty = !room.objects.iter().any(|object| {
                    (i16::try_from(object.x).ok(), i16::try_from(object.y).ok())
                        == (Some(at.x), Some(at.y))
                });
                (empty && collision(runtime, room, (at.x, at.y)) == Some(role))
                    .then_some((at, facing, role))
            })
    })
    .unwrap_or_else(|| panic!("no search position beside {}", event.script));

    let launch = if role == "WHIRLPOOL" {
        assert_eq!(
            (&*room.id, event.script.as_str(), at),
            (LAKE, EDDY, TilePosition::new(60, 20))
        );
        let launch = TilePosition::new(58, 20);
        assert_eq!(
            collision(runtime, room, (launch.x, launch.y)),
            Some("WATER")
        );
        launch
    } else if role == "WATER" {
        // New-game initialization always validates the spawn on foot. Start on the
        // surveyed dry bank before installing the fixture's checked Surf position.
        let bank = match event.script.as_str() {
            "MN_R12_GROTTO_REVIVE" => TilePosition::new(28, 11),
            "MN_CAVE_R20_SEEP_CACHE" => TilePosition::new(11, 19),
            other => panic!("unsurveyed Surf position for {other}"),
        };
        assert_eq!(collision(runtime, room, (bank.x, bank.y)), Some("FLOOR"));
        bank
    } else {
        at
    };
    let mut c = VisibleShellController {
        shell: initialize_bevy_runtime_shell(
            root.clone(),
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime
                    .title_new_game_spawn_identifier()
                    .expect("home spawn"),
                map_name: room.id.clone(),
                tile_x: launch.x,
                tile_y: launch.y,
            },
            BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()),
                ..Default::default()
            },
        )
        .expect("explicit search position"),
    };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    c.set_runtime_journal_enabled(false);
    let session = c.shell.shell.session_mut();
    if role == "WATER" {
        let afloat = runtime
            .data()
            .overworld_session_for_traversal(&room.id, at, 0, PlayerTraversalState::Surf)
            .expect("native occupancy accepts the surveyed Surf approach");
        session.overworld_mut().player.tile = afloat.player.tile;
    }
    session.overworld_mut().set_player_facing(facing);
    if role.ends_with("WATER") || role == "WHIRLPOOL" {
        session.overworld_mut().player.mode = MovementMode::Surf;
    }
    if role == "WHIRLPOOL" {
        let mut partner = runtime
            .data()
            .create_pokemon("REEDRAFT", 40, Dv::default())
            .expect("active Minnesota water partner");
        for name in ["SURF", "WHIRLPOOL"] {
            assert!(partner.species.tmhm_learnset.iter().any(|m| m == name));
        }
        partner.moves = ["SURF", "WHIRLPOOL"]
            .into_iter()
            .map(|name| LearnedMove {
                name: name.into(),
                current_pp: runtime.data().moves[name].pp,
                pp_ups: 0,
            })
            .collect();
        let state = session.state_mut();
        state.storage.party.pokemon[0] = Some(partner);
        state.sync_party_from_storage();
        for (index, flag) in BADGE_ENGINE_FLAGS.iter().take(7).enumerate() {
            state.badges.johto[index] = true;
            state
                .flags
                .set_engine_flag(flag, true)
                .expect("prior field badge");
        }
        state
            .flags
            .set_engine_flag("STATUSFLAGS_NO_WILD_ENCOUNTERS_F", true)
            .expect("isolate reward while paddling");
        mark_runtime_snapshot_dirty(&mut c.shell);
        approach_eddy(&mut c);
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    c
}

#[derive(Debug, PartialEq)]
enum Search {
    Full,
    Collected,
    Spent,
}

fn search(c: &mut VisibleShellController, event: &BackgroundEvent, expected: Search) {
    let current = c
        .shell
        .shell
        .current_overworld_interaction_checked()
        .expect("A-button target");
    if expected == Search::Spent {
        assert!(
            current.is_none(),
            "collected site still answers A: {}",
            event.script
        );
    } else {
        let hit = current.unwrap_or_else(|| {
            panic!(
                "no A-button target at {} ({}, {})",
                event.script, event.x, event.y
            )
        });
        assert_eq!(hit.script, event.script);
        assert_eq!(
            hit.target_tile,
            TilePosition::new(event.x as i16, event.y as i16)
        );
    }
    press(c, GameButton::A);
    let outcome = match c
        .shell
        .visible_field_item_notice
        .as_ref()
        .map(|notice| &notice.presentation)
    {
        Some(VisibleFieldItemPresentation::HiddenItem { sound_id: None }) => Search::Full,
        Some(VisibleFieldItemPresentation::HiddenItem { sound_id: Some(_) }) => Search::Collected,
        None => Search::Spent,
        other => panic!("wrong pickup presentation for {}: {other:?}", event.script),
    };
    assert_eq!(
        outcome,
        expected,
        "pickup feedback for {}; status={:?}, field={:?}, events={:?}, tile={:?}, facing={:?}, mode={:?}",
        event.script,
        c.shell.last_action_status,
        c.shell.field_notice,
        c.shell.last_audio_events,
        c.shell.shell.session().overworld().player.tile,
        c.shell.shell.session().overworld().player.facing,
        c.shell.shell.session().overworld().player.mode
    );
    settle(c, &event.script);
}

#[test]
#[ignore = "requires an Ely Challenge archive in MN_BADGE_TRAILS_PACK"]
fn ely_hidden_items_survive_full_pockets_and_continue() {
    let (root, runtime) = load();
    let data = runtime.data();
    let mut rooms = BTreeSet::new();
    let mut claims = BTreeSet::new();
    let mut balls = 0;
    let mut maps: Vec<_> = data
        .maps
        .values()
        .filter(|r| r.id.starts_with("Mn"))
        .collect();
    maps.sort_by_key(|room| match room.id.as_str() {
        "MnR12Cave1F" => 0,
        "MnR20Cave1F" => 1,
        LAKE => 2,
        _ => 3,
    });
    for room in maps {
        for event in room
            .events
            .bg_events
            .iter()
            .filter(|e| e.event_type == "BGEVENT_ITEM")
        {
            let pickup = room
                .script_field_pickups
                .iter()
                .find(|row| row.command == "hiddenitem" && row.source_script == event.script)
                .expect("hidden item payload");
            let item = &data.items[pickup.item_id.as_deref().expect("reward")];
            let flag = pickup.event_flag.as_deref().expect("receipt");
            assert!(claims.insert(flag), "shared receipt {flag}");
            rooms.insert(&room.id);
            assert_eq!(pickup.quantity, 1);
            let (filler, capacity) = match item.pocket.as_str() {
                ITEM_POCKET_ITEM => ("AWAKENING", ITEM_POCKET_CAPACITY),
                ITEM_POCKET_BALL => {
                    balls += 1;
                    ("FAST_BALL", BALL_POCKET_CAPACITY)
                }
                p => panic!("unsupported hidden reward pocket {p}"),
            };
            let filler = &data.items[filler];
            assert_eq!(item.pocket, filler.pocket);
            assert_ne!(item.script_name, filler.script_name);
            let mut c = fixture(&root, &runtime, room, event);
            let state = c.shell.shell.session_mut().state_mut();
            assert!(
                !state
                    .flags
                    .is_event_flag_set(flag)
                    .expect("unclaimed receipt")
            );
            for _ in 0..capacity {
                assert!(
                    state
                        .bag
                        .add_item(filler, MAX_ITEM_STACK)
                        .expect("fill pocket")
                );
            }
            let mut expected = state.bag.clone();
            mark_runtime_snapshot_dirty(&mut c.shell);
            search(&mut c, event, Search::Full);
            let state = c.shell.shell.session().state();
            assert_eq!(
                state.bag, expected,
                "full pocket consumed or granted {}",
                event.script
            );
            assert!(!state.flags.is_event_flag_set(flag).expect("failed claim"));
            let mut c = continue_at_site(&root, &runtime, &mut c, event.script == EDDY);
            let state = c.shell.shell.session_mut().state_mut();
            assert!(
                state
                    .bag
                    .remove_item(filler, MAX_ITEM_STACK)
                    .expect("free one slot")
            );
            assert!(
                expected
                    .remove_item(filler, MAX_ITEM_STACK)
                    .expect("expected free slot")
            );
            assert!(expected.add_item(item, 1).expect("expected reward"));
            mark_runtime_snapshot_dirty(&mut c.shell);
            search(&mut c, event, Search::Collected);
            let state = c.shell.shell.session().state();
            assert_eq!(state.bag, expected, "wrong reward at {}", event.script);
            assert!(
                state
                    .flags
                    .is_event_flag_set(flag)
                    .expect("successful receipt")
            );
            let mut c = continue_at_site(&root, &runtime, &mut c, event.script == EDDY);
            search(&mut c, event, Search::Spent);
            let state = c.shell.shell.session().state();
            assert_eq!(state.bag, expected, "duplicate reward at {}", event.script);
            assert!(
                state
                    .flags
                    .is_event_flag_set(flag)
                    .expect("persisted receipt")
            );
            eprintln!(
                "hidden reward verified: {} ({},{}) {} {}",
                room.id, event.x, event.y, item.script_name, flag
            );
        }
    }
    assert_eq!((claims.len(), rooms.len(), balls), (59, 49, 11));
}
