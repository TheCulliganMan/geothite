//! Real input, battles and persistence for the missing rival and climax scenes.
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::northstar_ice_lair::{choice, event, page, save, talk, walk};
use super::winona::{at, finish, load, press, travel};

fn setup(
    map: &str,
    tile: (i16, i16),
    party: &[(&str, u8)],
    flags: &[&str],
    badges: usize,
) -> (
    crystal_assets::AssetRoot,
    CrystalRuntime,
    VisibleShellController,
) {
    let (root, r) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        r.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: r.title_new_game_spawn_identifier().expect("HOME"),
            map_name: map.into(),
            tile_x: tile.0,
            tile_y: tile.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("declared chapter setup");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    let state = c.shell.shell.session_mut().state_mut();
    for &(species, level) in party {
        assert!(
            state.storage.party.add_pokemon(
                r.data()
                    .create_pokemon(species, level, Dv::from_non_hp(8, 9, 8, 9))
                    .expect("regional partner")
            )
        );
    }
    // Late chapter fixtures declare the reusable Surf HM already taught to
    // compatible Water partners, as a normal six-badge party can do.
    if badges >= 6 {
        for partner in state.storage.party.pokemon.iter_mut().flatten() {
            if partner.level >= 40 && partner.species.tmhm_learnset.iter().any(|m| m == "SURF") {
                partner.moves[0] = LearnedMove {
                    name: "SURF".into(),
                    current_pp: r.data().moves["SURF"].pp,
                    pp_ups: 0,
                };
            }
        }
    }
    state.sync_party_from_storage();
    for i in 0..badges {
        state.badges.johto[i] = true;
        state
            .flags
            .set_engine_flag(
                crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[i],
                true,
            )
            .expect("prior badge");
    }
    for flag in flags {
        state
            .flags
            .set_event_flag(flag, true)
            .expect("declared prior chapter");
    }
    state
        .bag
        .add_item(&r.data().items["SUPER_POTION"], 6)
        .expect("ordinary carried supplies");
    state.last_spawn_map_constant = Some(
        if map == "MnStPaulUnionDepot" {
            "MN_ST_PAUL_POKECENTER_1F"
        } else {
            "MN_ROCHESTER_POKECENTER_1F"
        }
        .into(),
    );
    // Declared prior progress must also initialize the map-entry object roster.
    // Continue intentionally preserves that roster rather than replaying OBJECTS.
    let (state, overworld) = c.shell.shell.session_mut().state_and_overworld_mut();
    r.data()
        .apply_map_setup_callbacks(state, overworld, map, "MAPSETUP_WARP")
        .expect("entry callbacks for declared chapter fixture");
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64).expect("settled fixture");
    // No position, party, HP/PP, RNG or progression writes after setup.
    // Continue preserves the roster initialized above from declared prior flags.
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    c = save(&mut c, &root, &r, &format!("arc-fixture-{map}-{nonce}"));
    (root, r, c)
}
fn won(c: &mut VisibleShellController, r: &CrystalRuntime, id: &str) {
    let flag = if id == "MN_LODE_CHIEF" {
        "EVENT_MN_SURVEY_CHIEF_DEFEATED".into()
    } else {
        format!("EVENT_{id}_DEFEATED")
    };
    assert!(
        super::rochester_earned::try_battle(c, r, id, &flag),
        "native rival course lost {id}"
    );
    finish(c);
}

#[test]
#[ignore = "requires the final regional rival arc"]
fn rival_arc_rochester_decline_retry_battle_and_continue_three_starters() {
    for starter in ["POUCHID", "KINDLIT", "BIRCHIT"] {
        let (root, r, mut c) = setup(
            "MnRochesterPokecenter1F",
            (8, 10),
            &[(starter, 10), ("NIBBIN", 8)],
            &[
                "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
                "EVENT_MN_STARTER_RECEIVED",
            ],
            0,
        );
        talk(&mut c, GameButton::Right);
        choice(&mut c, false);
        finish(&mut c);
        assert!(event(&c, "EVENT_MN_REED_ROCHESTER_MET"));
        assert!(!event(&c, "EVENT_MN_REED_ROCHESTER_DEFEATED"));
        c = save(&mut c, &root, &r, &format!("arc-early-declined-{starter}"));
        talk(&mut c, GameButton::Right);
        choice(&mut c, true);
        won(&mut c, &r, "MN_REED_ROCHESTER");
        c = save(&mut c, &root, &r, &format!("arc-early-won-{starter}"));
        talk(&mut c, GameButton::Right);
        page(&mut c, "MnArcRCAfterText");
        finish(&mut c);
        assert!(!event(&c, "EVENT_MN_REED_JOINED_ICE"));
    }
}
#[test]
#[ignore = "requires the final regional rival arc"]
fn rival_arc_depot_win_does_not_accept_minas_request_three_starters() {
    for starter in ["SLIPPERINA", "RIMEPELT", "PORTABEAU"] {
        let (root, r, mut c) = setup(
            "MnStPaulUnionDepot",
            (19, 16),
            &[
                (starter, 27),
                ("GILLPUP", 25),
                ("PELLETOR", 25),
                ("NOTCHBILL", 25),
            ],
            &["EVENT_MN_COURIER_DEFEATED", "EVENT_MN_DISPATCH_NOTE"],
            3,
        );
        talk(&mut c, GameButton::Left);
        choice(&mut c, false);
        finish(&mut c);
        assert!(event(&c, "EVENT_MN_REED_STPAUL_MET"));
        assert!(
            !event(&c, "EVENT_MN_REED_STPAUL_DEFEATED")
                && !event(&c, "EVENT_MN_DEEP_BORE_MANIFEST_READ")
        );
        c = save(&mut c, &root, &r, &format!("arc-depot-declined-{starter}"));
        for attempt in 0..3 {
            if attempt > 0 {
                at(&mut c, "MnStPaulPokecenter1F", (3, 5));
                walk(&mut c, &r, (3, 6));
                let clinic_exit = r.data().maps["MnStPaulPokecenter1F"]
                    .events
                    .warps
                    .iter()
                    .find(|w| w.x == 3 && w.y == 7)
                    .expect("native clinic exit");
                let town = &r.data().maps["MnStPaul"];
                let arrival = town
                    .events
                    .warps
                    .iter()
                    .find(|w| i16::try_from(w.index).ok() == Some(clinic_exit.target_warp_id))
                    .expect("registered current clinic return");
                travel(
                    &mut c,
                    GameButton::Down,
                    "MnStPaul",
                    (arrival.x as i16, arrival.y as i16),
                );
                let depot = town
                    .events
                    .warps
                    .iter()
                    .find(|w| w.target_map_constant == "MN_ST_PAUL_UNION_DEPOT")
                    .expect("registered current depot facade");
                walk(&mut c, &r, (depot.x as i16, depot.y as i16 + 1));
                travel(&mut c, GameButton::Up, "MnStPaulUnionDepot", (15, 21));
                walk(&mut c, &r, (19, 16));
            }
            talk(&mut c, GameButton::Left);
            choice(&mut c, true);
            if super::rochester_earned::try_battle(
                &mut c,
                &r,
                "MN_REED_STPAUL",
                "EVENT_MN_REED_STPAUL_DEFEATED",
            ) {
                break;
            }
            assert!(
                !event(&c, "EVENT_MN_REED_STPAUL_DEFEATED")
                    && !event(&c, "EVENT_MN_DEEP_BORE_MANIFEST_READ")
            );
        }
        assert!(event(&c, "EVENT_MN_REED_STPAUL_DEFEATED"));
        finish(&mut c);
        assert!(!event(&c, "EVENT_MN_DEEP_BORE_MANIFEST_READ"));
        talk(&mut c, GameButton::Left);
        choice(&mut c, false);
        finish(&mut c);
        talk(&mut c, GameButton::Left);
        choice(&mut c, true);
        page(&mut c, "MnNSTPMinaQuestText");
        assert!(!event(&c, "EVENT_MN_DEEP_BORE_MANIFEST_READ"));
        finish(&mut c);
        assert!(event(&c, "EVENT_MN_DEEP_BORE_MANIFEST_READ"));
        let _ = save(&mut c, &root, &r, &format!("arc-depot-complete-{starter}"));
    }
}
#[test]
#[ignore = "requires the final regional rival arc"]
fn rival_arc_stcloud_both_lanes_refusal_worker_movement_and_continue() {
    for x in [24, 25] {
        let (root, r, mut c) = setup(
            "MnStCloud",
            (x, 7),
            &[("RIMEPELT", 30)],
            &["EVENT_MN_REED_STPAUL_DEFEATED"],
            4,
        );
        for _ in 0..3 {
            press(&mut c, GameButton::Up);
            if c.has_pending_script_work() {
                break;
            }
        }
        choice(&mut c, false);
        finish(&mut c);
        assert!(!event(&c, "EVENT_MN_REED_WORKER_RELEASED"));
        at(&mut c, "MnStCloud", (24, 7));
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::Up);
        choice(&mut c, true);
        page(&mut c, "MnArcSCReleasedText");
        assert!(!event(&c, "EVENT_MN_REED_WORKER_RELEASED"));
        finish(&mut c);
        assert!(event(&c, "EVENT_MN_REED_WORKER_RELEASED"));
        c = save(&mut c, &root, &r, &format!("arc-worker-released-{x}"));
        assert!(
            !c.snapshot()
                .expect("Reed gone")
                .visible_objects
                .iter()
                .any(|o| o.object_identifier.as_deref() == Some("MN_ARC_SC_REED"))
        );
        assert!(!c.shell.shell.session().state().badges.johto[4]);
        let sera = c
            .shell
            .shell
            .session()
            .overworld()
            .object_runtime_tile_by_id("MN_ARC_SC_SERA")
            .expect("Sera remains in town");
        assert_eq!((sera.x, sera.y), (24, 9));
        assert!(
            c.snapshot()
                .expect("Sera visible after Continue")
                .visible_objects
                .iter()
                .any(|o| o.object_identifier.as_deref() == Some("MN_ARC_SC_SERA"))
        );
        walk(&mut c, &r, (24, 8));
        talk(&mut c, GameButton::Down);
        page(&mut c, "MnArcSCAfterText");
        finish(&mut c);
    }
}
#[test]
#[ignore = "requires the final regional rival arc"]
fn rival_arc_chief_link_rival_local_release_creature_crew_and_continue() {
    let (root, r, mut c) = setup(
        "MnBoundaryWatersSurveyShelter",
        (8, 9),
        &[
            ("BOREALYNX", 47),
            ("MUDMENDER", 46),
            ("PORTABEAU", 46),
            ("GOOSNIDE", 46),
            ("MESABLAST", 46),
            ("LEECHURGE", 45),
        ],
        &["EVENT_MN_LIGHTHOUSE_SIGNAL"],
        8,
    );
    assert!(
        super::northstar_ice_lair::path(&mut c, &r, (13, 8)).is_none(),
        "intake shutter is physically closed"
    );
    walk(&mut c, &r, (8, 5));
    talk(&mut c, GameButton::Right);
    won(&mut c, &r, "MN_LODE_CHIEF");
    assert!(!event(&c, "EVENT_MN_ARC_LINK_SENT") && !event(&c, "EVENT_MN_SURVEY_RESTORED"));
    walk(&mut c, &r, (12, 4));
    talk(&mut c, GameButton::Up);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_ARC_LINK_SENT"));
    talk(&mut c, GameButton::Up);
    choice(&mut c, true);
    page(&mut c, "MnArcLinkText");
    assert!(!event(&c, "EVENT_MN_ARC_LINK_SENT"));
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_ARC_LINK_SENT") && !event(&c, "EVENT_MN_SURVEY_RESTORED"));
    c = save(&mut c, &root, &r, "arc-link-sent");
    walk(&mut c, &r, (4, 7));
    talk(&mut c, GameButton::Left);
    finish(&mut c);
    walk(&mut c, &r, (13, 7));
    travel(&mut c, GameButton::Down, "MnBoundaryIntake", (4, 19));
    for _ in 0..8 {
        press(&mut c, GameButton::Up);
        if c.has_pending_script_work() {
            break;
        }
    }
    page(&mut c, "MnArcIntakeSeenText");
    at(&mut c, "MnBoundaryIntake", (18, 9));
    if let Some(dir) = std::env::var_os("CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR") {
        c.save(std::path::PathBuf::from(dir).join("arc-intake-seen-frame.crystalsave"))
            .expect("held cage witness");
    }
    finish(&mut c);
    assert!(
        r.data().maps["MnBoundaryIntake"]
            .objects
            .iter()
            .any(|o| o.sprite == "SPRITE_KETTLURK")
    );
    walk(&mut c, &r, (17, 12));
    talk(&mut c, GameButton::Up);
    page(&mut c, "MnArcIntakeDeniedText");
    finish(&mut c);
    walk(&mut c, &r, (14, 9));
    talk(&mut c, GameButton::Right);
    choice(&mut c, false);
    finish(&mut c);
    talk(&mut c, GameButton::Right);
    choice(&mut c, true);
    won(&mut c, &r, "MN_REED_BOUNDARY");
    assert!(!event(&c, "EVENT_MN_KETTLURK_RELEASED") && !event(&c, "EVENT_MN_SURVEY_RESTORED"));
    walk(&mut c, &r, (17, 12));
    talk(&mut c, GameButton::Up);
    choice(&mut c, false);
    finish(&mut c);
    c = save(&mut c, &root, &r, "arc-intake-declined");
    talk(&mut c, GameButton::Up);
    choice(&mut c, true);
    page(&mut c, "MnArcIntakeOpenText");
    assert!(!event(&c, "EVENT_MN_KETTLURK_RELEASED"));
    let m = &c.shell.shell.session().overworld().map;
    assert_eq!(m.metatile_ids[4 * m.width as usize + 10], 7);
    press(&mut c, GameButton::Down);
    at(&mut c, "MnBoundaryIntake", (18, 12));
    if let Some(dir) = std::env::var_os("CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR") {
        c.save(std::path::PathBuf::from(dir).join("arc-intake-open-frame.crystalsave"))
            .expect("actual held frame");
    }
    page(&mut c, "MnArcIntakeEscapeText");
    let live = c
        .shell
        .shell
        .session()
        .overworld()
        .object_runtime_tile_by_id("MN_ARC_KETTLURK")
        .expect("visible escaped creature");
    assert_eq!((live.x, live.y), (22, 13));
    assert!(
        c.snapshot()
            .expect("escaped creature remains in view")
            .visible_objects
            .iter()
            .any(|o| o.object_identifier.as_deref() == Some("MN_ARC_KETTLURK"))
    );
    if let Some(dir) = std::env::var_os("CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR") {
        c.save(std::path::PathBuf::from(dir).join("arc-kettlurk-current-frame.crystalsave"))
            .expect("actual creature in water");
    }
    page(&mut c, "MnArcIntakeCrewText");
    at(&mut c, "MnBoundaryIntake", (13, 15));
    if let Some(dir) = std::env::var_os("CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR") {
        c.save(std::path::PathBuf::from(dir).join("arc-intake-crew-frame.crystalsave"))
            .expect("held crew clearance");
    }
    for id in ["MN_ARC_INTAKE_IVY", "MN_ARC_INTAKE_FINN"] {
        assert!(
            c.snapshot()
                .expect("crew clearance in view")
                .visible_objects
                .iter()
                .any(|o| o.object_identifier.as_deref() == Some(id))
        );
    }
    assert!(event(&c, "EVENT_MN_KETTLURK_RELEASED"));
    assert!(!event(&c, "EVENT_MN_SURVEY_RESTORED"));
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_SURVEY_RESTORED"));
    c = save(&mut c, &root, &r, "arc-intake-complete");
    for id in ["MN_ARC_KETTLURK", "MN_ARC_INTAKE_REED"] {
        assert!(
            !c.snapshot()
                .expect("released scene")
                .visible_objects
                .iter()
                .any(|o| o.object_identifier.as_deref() == Some(id))
        );
    }
    walk(&mut c, &r, (4, 18));
    travel(
        &mut c,
        GameButton::Right,
        "MnBoundaryWatersSurveyShelter",
        (12, 7),
    );
    walk(&mut c, &r, (12, 4));
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_SURVEY_RESTORED"));
}

#[test]
#[ignore = "requires the final regional rival arc; native loss must retain only witnessed choices"]
fn rival_arc_losses_do_not_grant_rival_victory_or_release() {
    for (map, tile, flags, badges, facing, id, won_flag) in [
        (
            "MnRochesterPokecenter1F",
            (8, 10),
            vec!["EVENT_MN_DELIVERED_ROCHESTER_PARCEL"],
            0,
            GameButton::Right,
            "MN_REED_ROCHESTER",
            "EVENT_MN_REED_ROCHESTER_DEFEATED",
        ),
        (
            "MnStPaulUnionDepot",
            (19, 16),
            vec!["EVENT_MN_COURIER_DEFEATED", "EVENT_MN_DISPATCH_NOTE"],
            3,
            GameButton::Left,
            "MN_REED_STPAUL",
            "EVENT_MN_REED_STPAUL_DEFEATED",
        ),
        (
            "MnBoundaryIntake",
            (14, 9),
            vec!["EVENT_MN_ARC_LINK_SENT"],
            8,
            GameButton::Right,
            "MN_REED_BOUNDARY",
            "EVENT_MN_REED_BOUNDARY_DEFEATED",
        ),
    ] {
        let (_, r, mut c) = setup(map, tile, &[("CROOKIT", 1)], &flags, badges);
        talk(&mut c, facing);
        choice(&mut c, true);
        assert!(!super::rochester_earned::try_battle(
            &mut c, &r, id, won_flag
        ));
        assert!(!event(&c, won_flag));
        assert!(!event(&c, "EVENT_MN_KETTLURK_RELEASED") && !event(&c, "EVENT_MN_SURVEY_RESTORED"));
    }
}

#[test]
#[ignore = "requires the final regional rival arc"]
fn rival_arc_optional_station_aftermath_requires_restoration_and_persists() {
    for restored in [false, true] {
        let flags = if restored {
            vec!["EVENT_MN_SURVEY_RESTORED"]
        } else {
            vec![]
        };
        let (root, r, mut c) = setup("MnStCloudStation", (14, 16), &[("RIMEPELT", 27)], &flags, 5);
        let visible = c
            .snapshot()
            .expect("station arrival")
            .visible_objects
            .iter()
            .any(|o| o.object_identifier.as_deref() == Some("MN_ARC_AFTERMATH_REED"));
        assert_eq!(visible, restored);
        if restored {
            talk(&mut c, GameButton::Right);
            page(&mut c, "MnArcAftermathText");
            finish(&mut c);
            c = save(&mut c, &root, &r, "arc-station-aftermath");
            talk(&mut c, GameButton::Right);
            page(&mut c, "MnArcAftermathText");
            finish(&mut c);
            assert!(event(&c, "EVENT_MN_SURVEY_RESTORED"));
        }
    }
}

/// Settle only the new authored scenes while an earned-route walker owns input.
/// The early challenge can be postponed until native training prepares a team.
pub(super) fn settle_walk_scene(c: &mut VisibleShellController, r: &CrystalRuntime) {
    for _ in 0..256 {
        let s = c.snapshot().expect("authored arc input");
        if let Some(battle) = &s.battle {
            let crystal_runtime::RuntimeBattleKind::Trainer { trainer_id, .. } = &battle.kind
            else {
                panic!("unexpected wild scene");
            };
            assert!(trainer_id.starts_with("MN_REED_"));
            let flag = format!("EVENT_{trainer_id}_DEFEATED");
            assert!(super::rochester_earned::try_battle(c, r, trainer_id, &flag));
            continue;
        }
        let arc_text =
            s.ui.text
                .as_ref()
                .is_some_and(|t| t.label.starts_with("MnArc"));
        if !arc_text {
            return;
        }
        if s.ui.pending_yes_no.is_some() {
            let early =
                s.ui.text
                    .as_ref()
                    .is_some_and(|t| t.label.starts_with("MnArcRC"));
            choice(c, !early);
        } else {
            press(c, GameButton::A);
        }
        if !c.has_pending_script_work() && !c.snapshot().expect("settled scene").ui.text_window_open
        {
            return;
        }
    }
    panic!("authored arc did not return input");
}

#[test]
#[ignore = "requires the complete regional rival pack and isolated chapter saves"]
fn rival_arc_all_twenty_four_grunts_battle_and_persist_with_chapter_parties() {
    let source: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../planning/minnesota/story-arc-v3.json"),
        )
        .expect("authored source"),
    )
    .expect("authored grunt coverage");
    let guards = source["grunts"].as_array().expect("guards");
    assert_eq!(guards.len(), 24);
    for guard in guards {
        let map = guard["map"].as_str().expect("floor");
        let id = guard["id"].as_str().expect("grunt");
        let x = guard["at"][0].as_i64().expect("x") as i16;
        let y = guard["at"][1].as_i64().expect("y") as i16;
        let (tile, toward) = match guard["facing"].as_str().expect("native facing") {
            "UP" => ((x, y - 1), GameButton::Down),
            "DOWN" => ((x, y + 1), GameButton::Up),
            "LEFT" => ((x - 1, y), GameButton::Right),
            "RIGHT" => ((x + 1, y), GameButton::Left),
            _ => panic!("native facing"),
        };
        let (party, badges): (&[(&str, u8)], usize) = if map.starts_with("MnIce") {
            (
                &[
                    ("RIMEPELT", 25),
                    ("GILLPUP", 24),
                    ("NOTCHBILL", 24),
                    ("PELLETOR", 24),
                ],
                2,
            )
        } else if map == "MnBoundaryIntake" {
            (
                &[
                    ("BOREALYNX", 45),
                    ("MUDMENDER", 44),
                    ("NOTCHBILL", 44),
                    ("MESABLAST", 44),
                ],
                8,
            )
        } else {
            (
                &[
                    ("BOREALYNX", 40),
                    ("MUDMENDER", 40),
                    ("NOTCHBILL", 39),
                    ("MESABLAST", 40),
                ],
                7,
            )
        };
        let (root, r, mut c) = setup(map, tile, party, &["EVENT_MN_INTAKE_FIRST_SEEN"], badges);
        let flag = format!("EVENT_{id}_DEFEATED");
        assert!(!event(&c, &flag));
        if c.snapshot().expect("grunt encounter").battle.is_none() {
            press(&mut c, toward);
            press(&mut c, GameButton::A);
        }
        finish(&mut c);
        assert!(event(&c, &flag), "actual grunt victory {id}");
        c = save(&mut c, &root, &r, &format!("grunt-{id}"));
        assert!(event(&c, &flag), "same-pack Continue {id}");
        assert!(!event(&c, "EVENT_MN_SURVEY_RESTORED"));
    }
}
