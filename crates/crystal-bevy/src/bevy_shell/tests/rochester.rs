//! Earned opening/travel/delivery plus a separately identified gym combat fixture.
use std::collections::BTreeMap;

use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::{RuntimeBattleKind, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{RouteGrid, RouteSpec, indoor_walk};
use super::northstar_earned_gyms;
use super::winona::{at, continued, finish, load, opening, press, travel};

const DELIVERED: &str = "EVENT_MN_DELIVERED_ROCHESTER_PARCEL";
fn delivered(c: &VisibleShellController) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(DELIVERED)
        .expect("delivery event")
}

pub(super) fn flee(c: &mut VisibleShellController) {
    for _ in 0..512 {
        let s = c.snapshot().expect("wild encounter");
        if let Some(b) = &s.battle {
            assert!(
                matches!(
                    b.kind,
                    RuntimeBattleKind::Wild { .. } | RuntimeBattleKind::StaticWild { .. }
                ),
                "safe trail planner must not skip a trainer battle"
            );
        } else if s.phase == RuntimeShellPhase::Overworld
            && !s.ui.text_window_open
            && !c.has_pending_script_work()
        {
            return;
        }
        let button = if !c.shell.battle_messages.is_empty()
            || (s.battle.is_none()
                && s.ui.menu.is_none()
                && s.ui.text_window_open
                && s.ui.text.is_some())
        {
            GameButton::A
        } else if let Some(cursor) = &c.shell.battle_faint_prompt_cursor {
            if cursor.option_index > 0 {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_switch_cursor {
            let target = s
                .party
                .slots
                .iter()
                .position(|p| {
                    p.pokemon.hp > 0
                        && Some(p.index)
                            != s.battle.as_ref().and_then(|b| b.active_player_party_index)
                })
                .expect("living partner for native RUN retry");
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_party_action_cursor {
            if cursor.option_index > 0 {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            match cursor.option_index {
                0 | 1 => GameButton::Down,
                2 => GameButton::Right,
                3 => GameButton::A,
                _ => panic!("battle menu"),
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    panic!("native RUN did not release the encounter");
}

pub(super) fn trail(
    c: &mut VisibleShellController,
    r: &crystal_runtime::CrystalRuntime,
    to: (i16, i16),
) {
    trail_with(c, r, to, flee);
}

pub(super) fn trail_with(
    c: &mut VisibleShellController,
    r: &crystal_runtime::CrystalRuntime,
    to: (i16, i16),
    mut encounter: impl FnMut(&mut VisibleShellController),
) {
    assert!(
        try_trail_with(c, r, to, &mut encounter),
        "R01 walk was interrupted by a native map transition"
    );
}

/// A blackout ends this walk at the real recovery map instead of resuming a
/// stale route plan. The caller must heal and plan a fresh departure.
pub(super) fn try_trail_with(
    c: &mut VisibleShellController,
    r: &crystal_runtime::CrystalRuntime,
    to: (i16, i16),
    mut encounter: impl FnMut(&mut VisibleShellController),
) -> bool {
    let s = c.snapshot().expect("route start");
    let start = (s.overworld.tile.x, s.overworld.tile.y);
    let grid = RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix: "MnR01",
            columns: 1,
            rows: 1,
            tileset: "johto",
            start,
            doors: BTreeMap::from([((18, 17), "MnR01Cave1F"), ((49, 15), "MnR01Farmhouse")]),
        },
    );
    for _ in 0..4096 {
        let snapshot = c.snapshot().expect("live trail position");
        let tile = snapshot.overworld.tile;
        let here = (tile.x, tile.y);
        if here == to {
            return true;
        }
        // A native hop or encounter can finish beyond the originally planned
        // quadrant. Replan from the actual position rather than replaying a
        // stale suffix of the route's input list.
        let button = *grid.path(here, to).first().expect("remaining R01 step");
        let old = c.snapshot().expect("before trail tile").overworld.tile;
        for _ in 0..8 {
            press(c, button);
            if c.snapshot().expect("trail battle").battle.is_some() {
                encounter(c);
            }
            let now = c.snapshot().expect("trail tile");
            if now.overworld.map_name != "MnR01C00R00" {
                if now.overworld.map_name.ends_with("Pokecenter1F")
                    && let Some(directory) = std::env::var_os("CRYSTAL_WINONA_SAVE_DIR")
                {
                    let nonce = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("native blackout checkpoint clock")
                        .as_nanos();
                    let path = std::path::PathBuf::from(directory).join(format!(
                        "r01-native-blackout-{}-{nonce}.crystalsave",
                        std::process::id()
                    ));
                    c.save(path).expect("retain actual route blackout recovery");
                }
                return false;
            }
            if now.overworld.tile != old {
                break;
            }
        }
        let after = c.snapshot().expect("native trail step outcome");
        if after.overworld.tile == old {
            if let Some(directory) = std::env::var_os("CRYSTAL_WINONA_SAVE_DIR") {
                let path = std::path::PathBuf::from(directory).join(format!(
                    "r01-blocked-{}-{}-{}.crystalsave",
                    std::process::id(),
                    old.x,
                    old.y
                ));
                c.save(path).expect("retain the actual blocked route state");
            }
            panic!(
                "R01 step {button:?} from {old:?} did not move; ui={:?}, pending={}",
                after.ui,
                c.has_pending_script_work()
            );
        }
    }
    panic!("bounded native R01 walk did not reach {to:?}");
}

pub(super) fn delivery() -> (
    crystal_assets::AssetRoot,
    crystal_runtime::CrystalRuntime,
    VisibleShellController,
) {
    if load()
        .1
        .data()
        .story_event_script_constants
        .global
        .contains_key("EVENT_MN_TAMARACK_INTRO")
    {
        return super::dialogue_overhaul::delivery(1);
    }
    let (root, r, mut c) = opening(1, "CYNDAQUIL");
    travel(&mut c, GameButton::Down, "MnWinona", (7, 8));
    indoor_walk(&mut c, &r, (4, 18));
    travel(&mut c, GameButton::Left, "MnR01C00R00", (63, 24));
    assert!(!delivered(&c));
    c = continued(c, &root, &r, "r01-earned-arrival");
    trail(&mut c, &r, (0, 24));
    travel(&mut c, GameButton::Left, "MnRochester", (39, 18));
    assert!(!delivered(&c));
    c = continued(c, &root, &r, "rochester-earned-arrival");
    // Preview the real room before delivery. The earlier room permits a direct
    // leader refusal; the revised room first presents its closed lane and guide.
    // Its separate parcel guard is retained by the local prerequisite cases.
    indoor_walk(&mut c, &r, (30, 24));
    let selected = northstar_earned_gyms::selected(&r);
    let arrival = if selected {
        northstar_earned_gyms::arrival(&r, 1)
    } else {
        (6, 17)
    };
    travel(&mut c, GameButton::Up, "MnRochesterGym", arrival);
    if selected {
        northstar_earned_gyms::rochester_preview(&mut c, &r);
    } else {
        indoor_walk(&mut c, &r, (6, 4));
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        assert!(c.snapshot().expect("gym refusal").ui.text.is_some());
        finish(&mut c);
    }
    assert!(!delivered(&c));
    assert!(!c.shell.shell.session().state().badges.johto[0]);
    if selected {
        northstar_earned_gyms::leave(&mut c, &r, 1, "MnRochester", (30, 24));
    } else {
        indoor_walk(&mut c, &r, (6, 18));
        travel(&mut c, GameButton::Down, "MnRochester", (30, 24));
    }
    indoor_walk(&mut c, &r, (15, 8));
    travel(&mut c, GameButton::Up, "MnRochesterPokecenter1F", (10, 13));
    indoor_walk(&mut c, &r, (11, 9));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    assert!(!delivered(&c));
    let before = c.snapshot().expect("delivery page");
    let expected = super::northstar_dialogue::canonical_pages("MS01", "MnRCDeliveryText");
    assert_eq!(
        before.ui.text.as_ref().and_then(|t| t.asm_text.as_ref()),
        Some(&expected[0])
    );
    press(&mut c, GameButton::Down);
    at(&mut c, "MnRochesterPokecenter1F", (11, 9));
    assert!(!delivered(&c));
    press(&mut c, GameButton::A);
    assert!(
        !delivered(&c),
        "first acknowledgment cannot finish both pages"
    );
    assert_eq!(
        c.snapshot()
            .expect("second page")
            .ui
            .text
            .and_then(|t| t.asm_text),
        Some(expected[1].clone())
    );
    finish(&mut c);
    assert!(delivered(&c));
    assert!(!c.shell.shell.session().state().badges.johto[0]);
    c = continued(c, &root, &r, "rochester-originals-delivered");
    assert!(delivered(&c));
    let resources = c.snapshot().expect("before repeat");
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    let repeated = c.snapshot().expect("after repeat");
    assert_eq!(resources.party, repeated.party);
    assert_eq!(resources.bag, repeated.bag);
    (root, r, c)
}

#[test]
#[ignore = "requires the connected Rochester pack via CRYSTAL_WINONA_TEST_PACK"]
fn rochester_new_game_walks_r01_delivers_acknowledged_originals_and_returns_home() {
    let (root, r, mut c) = delivery();
    indoor_walk(&mut c, &r, (10, 14));
    travel(&mut c, GameButton::Down, "MnRochester", (15, 8));
    indoor_walk(&mut c, &r, (38, 18));
    travel(&mut c, GameButton::Right, "MnR01C00R00", (0, 24));
    trail(&mut c, &r, (63, 24));
    travel(&mut c, GameButton::Right, "MnWinona", (0, 18));
    travel(&mut c, GameButton::Right, "MnWinona", (4, 18));
    indoor_walk(&mut c, &r, (7, 8));
    travel(&mut c, GameButton::Up, "MnWinonaHome1F", (7, 6));
    assert!(delivered(&c));
    c = continued(c, &root, &r, "delivery-earned-return-home");
    assert!(delivered(&c));
}

#[test]
#[ignore = "requires the connected Rochester pack via CRYSTAL_WINONA_TEST_PACK"]
fn rochester_winona_starter_checkpoint_denies_both_lanes_without_granting_progress() {
    let (root, r) = load();
    let mut c = VisibleShellController::new_game(root, r.clone(), "CHRIS", None).expect("New Game");
    indoor_walk(&mut c, &r, (7, 1));
    travel(&mut c, GameButton::Up, "MnWinonaHome1F", (9, 0));
    finish(&mut c);
    press(&mut c, GameButton::Down);
    indoor_walk(&mut c, &r, (6, 6));
    travel(&mut c, GameButton::Down, "MnWinona", (7, 8));
    for y in [18, 19] {
        indoor_walk(&mut c, &r, (4, y));
        for _ in 0..8 {
            if c.snapshot().expect("checkpoint").ui.text.is_some() {
                break;
            }
            press(&mut c, GameButton::Left);
            assert_eq!(
                c.snapshot().expect("blocked exit").overworld.map_name,
                "MnWinona"
            );
        }
        assert!(c.snapshot().expect("starter refusal").ui.text.is_some());
        finish(&mut c);
        at(&mut c, "MnWinona", (4, y));
        assert!(!delivered(&c));
        assert!(c.snapshot().expect("empty party").party.slots.is_empty());
    }
}

fn gym_battle(c: &mut VisibleShellController, expected: &str) {
    let before = c.snapshot().expect("before gym battle").party;
    let mut saw = false;
    for _ in 0..1024 {
        let s = c.snapshot().expect("gym battle");
        if let Some(b) = &s.battle {
            saw = true;
            assert!(
                matches!(&b.kind,RuntimeBattleKind::Trainer{trainer_id,..} if trainer_id==expected)
            );
        } else if saw {
            break;
        }
        let button = if s.ui.menu.is_none() && s.ui.text.is_some() {
            GameButton::A
        } else if let (Some(cursor), Some(b)) = (&c.shell.battle_move_cursor, &s.battle) {
            let target = b
                .player_moves
                .iter()
                .position(|m| m.name == "EMBER" && m.current_pp > 0)
                .expect("legal Ember with PP");
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            match cursor.option_index {
                0 => GameButton::A,
                1 => GameButton::Left,
                2 | 3 => GameButton::Up,
                _ => panic!("battle cursor"),
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    assert!(saw);
    finish(c);
    let after = c.snapshot().expect("gym outcome");
    assert!(after.battle.is_none());
    assert_eq!(
        after.overworld.map_name, "MnRochesterGym",
        "a loss must not masquerade as victory"
    );
    assert!(after.party.slots[0].pokemon.experience > before.slots[0].pokemon.experience);
    let pp = |p: &crystal_runtime::RuntimePartySnapshot| {
        p.slots[0]
            .pokemon
            .moves
            .iter()
            .find(|m| m.name == "EMBER")
            .expect("Ember")
            .current_pp
    };
    assert!(pp(&after.party) < pp(&before));
}

#[test]
#[ignore = "requires the connected Rochester pack via CRYSTAL_WINONA_TEST_PACK"]
fn rochester_gym_native_trainers_badge_tm_both_city_returns_and_continue() {
    let (root, r) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        r.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: r.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnRochesterGym".into(),
            tile_x: 6,
            tile_y: 17,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local gym placement");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture name");
    // Legal level16 evolved partner and completed delivery are explicit setup.
    // Every qualification, battle, badge, reward and return below uses input.
    let mut p = r
        .data()
        .create_pokemon("QUILAVA", 16, Dv::default())
        .expect("legal fixture partner");
    let trainer = c.snapshot().expect("fixture trainer").trainer;
    p.original_trainer_name = trainer.player_name;
    p.original_trainer_id = trainer.player_id;
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(p));
    state.sync_party_from_storage();
    state
        .flags
        .set_event_flag(DELIVERED, true)
        .expect("local delivery setup");
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    for (tile, id) in [
        ((6, 16), "MN_GYM_1_TRAINER_1"),
        ((6, 8), "MN_GYM_1_TRAINER_2"),
    ] {
        indoor_walk(&mut c, &r, tile);
        press(&mut c, GameButton::Left);
        press(&mut c, GameButton::A);
        gym_battle(&mut c, id);
        assert!(
            c.shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(&format!("EVENT_{id}_DEFEATED"))
                .expect("actual trainer win")
        );
        assert!(!c.shell.shell.session().state().badges.johto[0]);
    }
    indoor_walk(&mut c, &r, (6, 4));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    gym_battle(&mut c, "MN_GYM_1");
    assert!(c.shell.shell.session().state().badges.johto[0]);
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_GYM_1_DEFEATED")
            .expect("leader victory event")
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_HEADBUTT"]),
        1
    );
    c = continued(c, &root, &r, "rochester-gym-local-win");
    assert!(c.shell.shell.session().state().badges.johto[0]);
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_GYM_1_DEFEATED")
            .expect("continued leader victory")
    );
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_HEADBUTT"]),
        1
    );
    for x in [6, 7] {
        indoor_walk(&mut c, &r, (x, 18));
        travel(&mut c, GameButton::Down, "MnRochester", (30, 24));
        if x == 6 {
            travel(&mut c, GameButton::Up, "MnRochesterGym", (6, 17));
        }
    }
}
