//! Explicit prior-access fixture, then three real rooms and native drilling.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::northstar_ice_lair::{choice, event, page, path, save, talk, walk};
use super::winona::{at, finish, load, press, travel};

const MINE: &str = "MnR09Cave1F";
const APPROACH: &str = "MnIronDrillApproach";
const GALLERY: &str = "MnIronPumpGallery";
const CORE: &str = "MnIronDrillCore";
const FOREMAN: &str = "EVENT_MN_DEPOT_FOREMAN_DEFEATED";
const SEEN: &str = "EVENT_MN_DRILL_TEST_WITNESSED";
const FEED: &str = "EVENT_MN_DRILL_FEED_ISOLATED";
const WON: &str = "EVENT_MN_REED_MINE_DEFEATED";
const OFF: &str = "EVENT_MN_DRILL_STOPPED";
const RECOVERED: &str = "EVENT_MN_MINE_INSTRUMENT_RECOVERED";

#[test]
#[ignore = "requires the published regional drilling chapter and isolated saves"]
fn drill_lair_native_maze_witnessed_piston_rival_shutdown_crew_quest_and_continue() {
    let (root, r) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        r.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: r.title_new_game_spawn_identifier().expect("HOME"),
            map_name: MINE.into(),
            tile_x: 13,
            tile_y: 18,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit prior-access fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    let state = c.shell.shell.session_mut().state_mut();
    // This course verifies scripts and geometry, not intended-level balance.
    // Use a mixed party so two consecutive four-member teams cannot leave the
    // whole scene dependent on the Fire/Ice lead surviving Rock/Water coverage.
    for (species, level) in [
        ("BOREALYNX", 48),
        ("MUDMENDER", 45),
        ("PORTABEAU", 44),
        ("GOOSNIDE", 44),
    ] {
        assert!(
            state.storage.party.add_pokemon(
                r.data()
                    .create_pokemon(species, level, Dv::from_non_hp(8, 9, 8, 9))
                    .expect("regional geometry partner")
            )
        );
    }
    state.sync_party_from_storage();
    assert!(
        state
            .bag
            .add_item(&r.data().items["SUPER_POTION"], 6)
            .expect("explicit geometry-course supplies")
    );
    state.badges.johto = [true, true, true, true, true, true, true, false];
    for flag in &crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[..7] {
        state
            .flags
            .set_engine_flag(flag, true)
            .expect("prior badge fixture");
    }
    state
        .flags
        .set_event_flag("EVENT_MN_HEADWATERS_REPORT", true)
        .expect("prior quest access");
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64).expect("fixture ready");
    let _ = play_chapter(c, &root, &r);
}

pub(super) fn play_chapter(
    mut c: VisibleShellController,
    root: &crystal_assets::AssetRoot,
    r: &crystal_runtime::CrystalRuntime,
) -> VisibleShellController {
    // No progression, position, party, move, HP or RNG writes after this line.
    for flag in [FOREMAN, FEED, SEEN, WON, OFF, RECOVERED] {
        assert!(!event(&c, flag));
    }
    walk(&mut c, &r, (19, 8));
    talk(&mut c, GameButton::Up);
    page(&mut c, "MnNDrillScoutQuestText");
    finish(&mut c);
    walk(&mut c, &r, (7, 4));
    talk(&mut c, GameButton::Down);
    page(&mut c, "MnNDrillMeterDeniedText");
    finish(&mut c);
    assert!(!event(&c, RECOVERED));
    assert!(
        path(&mut c, &r, (19, 11)).is_none(),
        "lower workings sealed before foreman win"
    );
    walk(&mut c, &r, (9, 4));
    talk(&mut c, GameButton::Down);
    assert!(super::rochester_earned::try_battle(
        &mut c,
        &r,
        "MN_LODE_FOREMAN",
        FOREMAN
    ));
    finish(&mut c);
    assert!(!event(&c, OFF) && !event(&c, RECOVERED));
    walk(&mut c, &r, (19, 10));
    travel(&mut c, GameButton::Down, APPROACH, (6, 21));
    let expanded = r.data().maps.contains_key("MnIronHaulage");
    if expanded {
        super::northstar_iron_workings::enter_gallery(&mut c, &r, &root);
    } else {
        walk(&mut c, &r, (25, 6));
        travel(&mut c, GameButton::Up, GALLERY, (4, 19));
    }
    assert!(
        path(&mut c, &r, (25, 4)).is_none(),
        "core stair physically sealed"
    );
    let long = path(&mut c, &r, (25, 18)).expect("real pump maze").len();
    assert!(long > 25, "feed must require winding through pump banks");
    walk(&mut c, &r, (25, 18));
    talk(&mut c, GameButton::Up);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, FEED));
    c = save(&mut c, &root, &r, "pump-feed-declined");
    talk(&mut c, GameButton::Up);
    choice(&mut c, true);
    finish(&mut c);
    assert!(event(&c, FEED) && !event(&c, SEEN) && !event(&c, OFF));
    assert!(path(&mut c, &r, (4, 19)).expect("service return").len() < long);
    c = save(&mut c, &root, &r, "pump-feed-isolated");
    walk(&mut c, &r, (25, 5));
    travel(&mut c, GameButton::Up, CORE, (4, 15));
    walk(&mut c, &r, (4, 13));
    press(&mut c, GameButton::Up);
    for (label, expected) in [
        ("MnNDrillCrewCallText", 0x16),
        ("MnNDrillForemanText", 0x12),
    ] {
        page(&mut c, label);
        assert!(!event(&c, SEEN));
        press(&mut c, GameButton::Left);
        at(&mut c, CORE, (9, 9));
        let map = &c.shell.shell.session().overworld().map;
        let block = map.metatile_ids[4 * map.width as usize + 5];
        assert_eq!(
            block, expected,
            "the visible stroke must change as the native scene advances"
        );
        let directory = std::path::PathBuf::from(
            std::env::var_os("CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR")
                .expect("actual scene capture folder"),
        );
        std::fs::create_dir_all(&directory).expect("capture folder");
        // These actual acted frames are for render review, not Continue.
        c.save(directory.join(format!("drill-piston-{block:02x}.crystalsave")))
            .expect("actual native piston frame");
    }
    assert!(
        c.shell
            .last_audio_events
            .iter()
            .any(|e| e.contains("drained runtime earthquakes")),
        "the production controller must finish the actual shaking presentation"
    );
    finish(&mut c);
    assert!(event(&c, SEEN) && !event(&c, WON) && !event(&c, OFF));
    c = save(&mut c, &root, &r, "drill-witnessed");
    walk(&mut c, &r, (16, 3));
    talk(&mut c, GameButton::Up);
    page(&mut c, "MnNDrillReedBlocksText");
    finish(&mut c);
    walk(&mut c, &r, (8, 9));
    talk(&mut c, GameButton::Up);
    assert!(super::rochester_earned::try_battle(
        &mut c,
        &r,
        "MN_REED_MINE",
        WON
    ));
    finish(&mut c);
    assert!(!event(&c, OFF) && !event(&c, RECOVERED));
    c = save(&mut c, &root, &r, "mine-rival-won");
    walk(&mut c, &r, (16, 3));
    talk(&mut c, GameButton::Up);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, OFF));
    talk(&mut c, GameButton::Up);
    choice(&mut c, true);
    finish(&mut c);
    assert!(event(&c, OFF) && !event(&c, RECOVERED));
    c = save(&mut c, &root, &r, "drill-stopped");
    assert!(
        c.snapshot()
            .expect("rescued crew")
            .visible_objects
            .iter()
            .any(|o| o.object_identifier.as_deref() == Some("MN_NDRILL_FINN"))
    );
    // Snapshot object events retain authored coordinates; movement and
    // Continue restore the authoritative object-struct position separately.
    assert_eq!(
        c.shell
            .shell
            .session()
            .overworld()
            .object_runtime_tile_by_id("MN_NDRILL_FINN")
            .expect("rescued crew position"),
        crystal_core::world::map::TilePosition::new(8, 7)
    );
    walk(&mut c, &r, (8, 6));
    talk(&mut c, GameButton::Down);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, RECOVERED));
    talk(&mut c, GameButton::Down);
    choice(&mut c, true);
    page(&mut c, "MnNDrillMeterQuestText");
    assert!(!event(&c, RECOVERED));
    finish(&mut c);
    assert!(event(&c, RECOVERED));
    c = save(&mut c, &root, &r, "crew-meter-quest-acknowledged");
    talk(&mut c, GameButton::Down);
    page(&mut c, "MnNDrillMeterAfterText");
    finish(&mut c);
    leave_chapter(c, r)
}

fn leave_up(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    approach: (i16, i16),
    target_map: &str,
    target: (i16, i16),
) {
    let source = c.snapshot().expect("mine exit").overworld.map_name;
    walk(c, r, approach);
    for _ in 0..64 {
        let snapshot = c.snapshot().expect("native mine exit");
        if snapshot.overworld.map_name == target_map {
            at(c, target_map, target);
            return;
        }
        if let Some(battle) = snapshot.battle {
            let crystal_runtime::RuntimeBattleKind::Trainer { trainer_id, .. } = battle.kind else {
                panic!("mine exit only has authored trainers");
            };
            assert!(super::rochester_earned::try_battle(
                c,
                r,
                &trainer_id,
                &format!("EVENT_{trainer_id}_DEFEATED")
            ));
            if c.snapshot().expect("post-trainer exit").overworld.map_name == source {
                walk(c, r, approach);
            }
        } else if snapshot.ui.text.is_some() || c.has_pending_script_work() {
            press(c, GameButton::A);
        } else {
            press(c, GameButton::Up);
        }
    }
    panic!("mine exit did not reach {target_map}");
}

/// Walk the completed crew handoff back out through its actual native ladders.
pub(super) fn leave_chapter(
    mut c: VisibleShellController,
    r: &crystal_runtime::CrystalRuntime,
) -> VisibleShellController {
    assert!(event(&c, RECOVERED) && event(&c, OFF));
    let expanded = r.data().maps.contains_key("MnIronHaulage");
    leave_up(&mut c, r, (5, 15), GALLERY, (24, 5));
    if expanded {
        super::northstar_iron_workings::leave_gallery(&mut c, r);
    } else {
        leave_up(&mut c, r, (5, 19), APPROACH, (24, 5));
    }
    walk(&mut c, r, (6, 21));
    travel(&mut c, GameButton::Right, MINE, (19, 9));
    walk(&mut c, r, (19, 8));
    talk(&mut c, GameButton::Up);
    page(&mut c, "MnNDrillPublicAfterText");
    finish(&mut c);
    c
}
