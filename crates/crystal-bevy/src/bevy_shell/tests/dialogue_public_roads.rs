//! Declared local approach/party fixtures, followed only by production input.
//! Register directly under bevy_shell::tests.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::winona::{at, continue_in_place, finish, load, press};

fn fixture(map: &str, tile: (i16, i16)) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: map.into(),
            tile_x: tile.0,
            tile_y: tile.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local approach fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture name");
    let partner = runtime
        .data()
        .create_pokemon("KINDLIT", 35, Dv::default())
        .expect("healthy regional fixture partner");
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    (root, runtime, c)
}

const ROADS: &[(&str, &str)] = &[
    ("MnBemidji", "MnBemidjiItascaCheck"),
    ("MnBemidji", "MnBemidjiIronCheck"),
    ("MnBemidji", "MnCohesionBemidjiNorth"),
    ("MnBoundaryWaters", "MnBoundaryGunflintCheck"),
    ("MnBoundaryWaters", "MnBoundaryLeagueCheck"),
    ("MnBoundaryWaters", "MnBoundaryVoyageursCheck"),
    ("MnBrainerd", "MnBrainerdLakeCheck"),
    ("MnBrainerd", "MnCohesionBrainerdWest"),
    ("MnBrainerd", "MnCohesionBrainerdEast"),
    ("MnDuluth", "MnDuluthRangeCheck"),
    ("MnDuluth", "MnDuluthShoreCheck"),
    ("MnGrandMarais", "MnCohesionMaraisWest"),
    ("MnGrandMarais", "MnCohesionMaraisEast"),
    ("MnGrandRapids", "MnGrandRapidsWestCheck"),
    ("MnGrandRapids", "MnGrandRapidsEastCheck"),
    ("MnLakeItasca", "MnCohesionItascaWest"),
    ("MnLeechLake", "MnLeechLakeCheck"),
    ("MnMankato", "MnMKRiverCheck"),
    ("MnMinneapolis", "MnMPEastCheck"),
    ("MnMinneapolis", "MnMPNorthCheck"),
    ("MnRochester", "MnRCPrairieCheck"),
    ("MnStCloud", "MnSCWindCheck"),
];

#[test]
#[ignore = "requires final public-roads pack in CRYSTAL_WINONA_TEST_PACK and isolated CRYSTAL_WINONA_SAVE_DIR; local position/party setup is explicit"]
fn public_roads_walk_both_lanes_both_directions_without_badges_and_continue() {
    let (_, runtime) = load();
    for (map_name, gate) in ROADS {
        let map = runtime.data().map_module(map_name).expect("final road map");
        let actors = map
            .objects
            .iter()
            .filter(|o| o.script == *gate)
            .collect::<Vec<_>>();
        assert_eq!(actors.len(), 2, "preserved numeric actor slots");
        let vertical_pair = actors[0].x == actors[1].x;
        for (lane, actor) in actors.iter().enumerate() {
            for sign in [-1_i16, 1] {
                let x = i16::try_from(actor.x).expect("native x");
                let y = i16::try_from(actor.y).expect("native y");
                let (start, target, button) = if vertical_pair {
                    (
                        (x + sign, y),
                        (x - sign, y),
                        if sign < 0 {
                            GameButton::Right
                        } else {
                            GameButton::Left
                        },
                    )
                } else {
                    (
                        (x, y + sign),
                        (x, y - sign),
                        if sign < 0 {
                            GameButton::Down
                        } else {
                            GameButton::Up
                        },
                    )
                };
                let (root, runtime, mut c) = fixture(map_name, start);
                let before = c.shell.shell.session().state().clone();
                assert!(before.badges.johto.iter().all(|b| !b), "no supplied badges");
                for _ in 0..10 {
                    let snapshot = c.snapshot().expect("native road input");
                    assert!(
                        !snapshot.ui.text_window_open,
                        "ordinary travel must not open a permission conversation"
                    );
                    if (snapshot.overworld.tile.x, snapshot.overworld.tile.y) == target {
                        break;
                    }
                    press(&mut c, button);
                }
                at(&mut c, map_name, target);
                let after = c.shell.shell.session().state();
                assert_eq!(after.badges, before.badges);
                assert_eq!(
                    after.flags, before.flags,
                    "opening roads must not grant quest results"
                );
                assert_eq!(after.bag, before.bag);
                if lane == 0 && sign == -1 {
                    continue_in_place(&mut c, &root, &runtime, &format!("public-{gate}"));
                    assert!(!c.snapshot().expect("road Continue").ui.text_window_open);
                    at(&mut c, map_name, target);
                }
            }
        }
    }
}

#[test]
#[ignore = "requires final public-roads pack; partner/position are explicit fixtures, starter receipt remains false"]
fn public_roads_keep_the_initial_partner_safeguard() {
    let (_, runtime) = load();
    let map = runtime
        .data()
        .map_module("MnWinona")
        .expect("current hometown");
    let gate = map
        .objects
        .iter()
        .find(|o| o.script == "MnWTrailCheck")
        .expect("retained partner safeguard");
    let (x, y) = (
        i16::try_from(gate.x).expect("x"),
        i16::try_from(gate.y).expect("y"),
    );
    let (_, _, mut c) = fixture("MnWinona", (x + 1, y));
    for _ in 0..4 {
        if c.snapshot().expect("partner gate").ui.text_window_open {
            break;
        }
        press(&mut c, GameButton::Left);
    }
    assert!(c.snapshot().expect("partner guidance").ui.text_window_open);
    assert!(c.snapshot().expect("stopped before road").overworld.tile.x > x);
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_STARTER_RECEIVED")
            .expect("starter flag")
    );
}

#[test]
#[ignore = "requires final public-roads pack; zero/eight badges are explicit prior setup, restoration is never granted"]
fn public_roads_keep_real_league_registration_requirements() {
    use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
    let (_, runtime) = load();
    let map = runtime
        .data()
        .map_module("MnLeagueLobby")
        .expect("current lobby");
    let actor = map
        .objects
        .iter()
        .find(|o| o.script == "MnElyRegister")
        .expect("real registrar");
    let arrival = map.events.warps.first().expect("public lobby entrance");
    for count in [0_usize, 8] {
        let (_, runtime, mut c) = fixture(
            "MnLeagueLobby",
            (
                i16::try_from(arrival.x).expect("x"),
                i16::try_from(arrival.y).expect("y"),
            ),
        );
        {
            let state = c.shell.shell.session_mut().state_mut();
            for (i, flag) in BADGE_ENGINE_FLAGS[..8].iter().enumerate() {
                state.badges.johto[i] = i < count;
                state
                    .flags
                    .set_engine_flag(flag, i < count)
                    .expect("declared prior badge");
            }
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        let before = c.shell.shell.session().state().clone();
        super::minnesota_gyms::interact(&mut c, &runtime, (actor.x, actor.y));
        assert!(
            c.snapshot()
                .expect("registration refusal")
                .ui
                .text_window_open
        );
        finish(&mut c);
        assert_eq!(
            c.snapshot()
                .expect("still in public lobby")
                .overworld
                .map_name,
            "MnLeagueLobby"
        );
        let after = c.shell.shell.session().state();
        assert_eq!(
            after.flags, before.flags,
            "no registration or restoration granted"
        );
        assert_eq!(after.badges, before.badges);
    }
}
