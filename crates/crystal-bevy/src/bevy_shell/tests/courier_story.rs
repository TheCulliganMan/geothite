//! Explicit local setup; subsequent movement, battles and reading use real input.
use std::collections::{BTreeMap, BTreeSet};

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{RouteGrid, RouteSpec, indoor_walk};
use super::rochester::flee;
use super::winona::{at, continued, finish, load, press, travel};

const DEPOT: &str = "MnStPaulUnionDepot";
const WIN: &str = "EVENT_MN_COURIER_DEFEATED";
const READ: &str = "EVENT_MN_DEEP_BORE_MANIFEST_READ";
const NOTE: &str = "EVENT_MN_DISPATCH_NOTE";

enum Case {
    Ready,
    MissingBadge,
    MissingNote,
    Losing,
    EmptyRoad,
}

fn fixture(
    map: &str,
    tile: (i16, i16),
    case: Case,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
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
    .expect("explicit local courier setup");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture name");
    let current = !matches!(case, Case::MissingBadge);
    let note = !matches!(case, Case::MissingNote | Case::EmptyRoad);
    let losing = matches!(case, Case::Losing);
    let state = c.shell.shell.session_mut().state_mut();
    state.badges.johto = [true, true, current, false, false, false, false, false];
    for (i, flag) in BADGE_ENGINE_FLAGS[..8].iter().enumerate() {
        state
            .flags
            .set_engine_flag(flag, state.badges.johto[i])
            .expect("fixture badge");
    }
    state
        .flags
        .set_event_flag(NOTE, note)
        .expect("fixture dispatch");
    state.money = 10000;
    if !matches!(case, Case::EmptyRoad) {
        let mut p = r
            .data()
            .create_pokemon(
                if losing { "MAGIKARP" } else { "MEW" },
                if losing { 5 } else { 45 },
                Dv::default(),
            )
            .expect("legal local partner");
        p.moves = vec![LearnedMove {
            name: if losing { "SPLASH" } else { "PSYCHIC_M" }.into(),
            current_pp: if losing { 40 } else { 10 },
            pp_ups: 0,
        }];
        assert!(state.storage.party.add_pokemon(p));
        state.sync_party_from_storage();
    }
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    (root, r, c)
}
fn event(c: &VisibleShellController, flag: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(flag)
        .expect("chapter event")
}
fn talk(c: &mut VisibleShellController, direction: GameButton) {
    press(c, direction);
    press(c, GameButton::A);
}

fn road(c: &mut VisibleShellController, r: &CrystalRuntime, target: (i16, i16)) {
    let tile = c.snapshot().expect("road").overworld.tile;
    let start = (tile.x, tile.y);
    let grid = RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix: "MnR04",
            columns: 1,
            rows: 1,
            tileset: "mn_river_park",
            start,
            doors: BTreeMap::new(),
        },
    );
    for button in grid.path(start, target) {
        let old = c.snapshot().expect("road position").overworld.tile;
        for _ in 0..8 {
            press(c, button);
            if c.snapshot().expect("road encounter").battle.is_some() {
                flee(c);
            }
            if c.snapshot().expect("settled step").overworld.tile != old {
                break;
            }
        }
        assert_ne!(c.snapshot().expect("moved").overworld.tile, old);
    }
    at(c, "MnR04C00R00", target);
}

fn fight(c: &mut VisibleShellController, losing: bool) {
    let before = c.snapshot().expect("battle resources");
    let mut saw = false;
    let mut attacks = 0;
    let mut foes = BTreeSet::new();
    for _ in 0..1024 {
        let s = c.snapshot().expect("courier battle");
        let Some(b) = s.battle else {
            if saw {
                break;
            }
            press(c, GameButton::A);
            continue;
        };
        saw = true;
        assert!(
            matches!(b.kind,RuntimeBattleKind::Trainer{ref trainer_id,..} if trainer_id=="MN_LODE_COURIER")
        );
        foes.insert(b.enemy_pokemon.species.id.clone());
        assert!(!event(c, WIN) && !event(c, READ));
        let button = if s.ui.menu.is_none() && s.ui.text.is_some() {
            GameButton::A
        } else if let Some(cursor) = &c.shell.battle_move_cursor {
            let name = if losing { "SPLASH" } else { "PSYCHIC_M" };
            let target = b
                .player_moves
                .iter()
                .position(|m| m.name == name && m.current_pp > 0)
                .expect("native move");
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                attacks += 1;
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
    assert!(
        saw && attacks > 0,
        "an actual selected battle turn is required"
    );
    finish(c);
    let after = c.snapshot().expect("battle exit");
    assert!(after.battle.is_none());
    if !losing {
        assert_eq!(foes, BTreeSet::from(["MAGNEMITE".into(), "DROWZEE".into()]));
        assert!(event(c, WIN));
        assert!(!event(c, READ));
        assert!(after.party.slots[0].pokemon.experience > before.party.slots[0].pokemon.experience);
        assert!(
            after.party.slots[0].pokemon.moves[0].current_pp
                < before.party.slots[0].pokemon.moves[0].current_pp
        );
    }
}

#[test]
#[ignore = "requires courier story via CRYSTAL_WINONA_TEST_PACK"]
fn courier_story_gate_denies_both_lanes_and_inbound_care_is_free() {
    for (y, road_y) in [(24, 18), (25, 19)] {
        let (_, _, mut c) = fixture("MnMinneapolis", (59, y), Case::MissingBadge);
        for _ in 0..8 {
            if c.snapshot().expect("refusal").ui.text.is_some() {
                break;
            }
            press(&mut c, GameButton::Right);
        }
        assert!(
            c.snapshot()
                .expect("badge requirement")
                .ui
                .text
                .and_then(|t| t.asm_text)
                .is_some_and(|s| s.contains("AVRIN"))
        );
        press(&mut c, GameButton::Right);
        at(&mut c, "MnMinneapolis", (60, y));
        finish(&mut c);
        at(&mut c, "MnMinneapolis", (59, y));
        assert!(!event(&c, WIN) && !event(&c, READ));
        let (root, r, mut c) = fixture("MnR04C00R00", (0, road_y), Case::MissingBadge);
        travel(&mut c, GameButton::Left, "MnMinneapolis", (63, y));
        c = continued(c, &root, &r, "courier-local-inbound");
        travel(&mut c, GameButton::Left, "MnMinneapolis", (59, y));
        indoor_walk(&mut c, &r, (45, 12));
        travel(
            &mut c,
            GameButton::Up,
            "MnMinneapolisPokecenter1F",
            (10, 13),
        );
        assert!(!c.shell.shell.session().state().badges.johto[2]);
    }
}

#[test]
#[ignore = "requires courier story via CRYSTAL_WINONA_TEST_PACK"]
fn courier_story_both_battle_prerequisites_and_locked_record_preserve_progress() {
    for (case, text) in [
        (Case::MissingBadge, "CURRENT"),
        (Case::MissingNote, "dispatch"),
    ] {
        let (_, _, mut c) = fixture(DEPOT, (19, 16), case);
        let before = c.snapshot().expect("before refusal");
        talk(&mut c, GameButton::Right);
        assert!(
            c.snapshot()
                .expect("courier refusal")
                .ui
                .text
                .and_then(|t| t.asm_text)
                .is_some_and(|s| s.contains(text))
        );
        press(&mut c, GameButton::Down);
        at(&mut c, DEPOT, (19, 16));
        finish(&mut c);
        talk(&mut c, GameButton::Left);
        assert!(
            c.snapshot()
                .expect("record held")
                .ui
                .text
                .and_then(|t| t.asm_text)
                .is_some_and(|s| s.contains("courier"))
        );
        finish(&mut c);
        let after = c.snapshot().expect("no free progress");
        assert_eq!(before.party, after.party);
        assert_eq!(before.bag, after.bag);
        assert_eq!(c.shell.shell.session().state().money, 10000);
        assert!(!event(&c, WIN) && !event(&c, READ));
    }
}

#[test]
#[ignore = "requires courier story via CRYSTAL_WINONA_TEST_PACK"]
fn courier_story_current_alone_reaches_depot_and_fourth_gym_then_returns() {
    let (root, r, mut c) = fixture("MnMinneapolis", (59, 24), Case::EmptyRoad);
    c = continued(c, &root, &r, "courier-local-road-ready");
    travel(&mut c, GameButton::Right, "MnR04C00R00", (0, 18));
    road(&mut c, &r, (55, 18));
    travel(&mut c, GameButton::Right, "MnStPaul", (0, 24));
    travel(&mut c, GameButton::Right, "MnStPaul", (1, 24));
    indoor_walk(&mut c, &r, (54, 16));
    travel(&mut c, GameButton::Up, "MnStPaulGym", (2, 17));
    travel(&mut c, GameButton::Down, "MnStPaul", (54, 16));
    indoor_walk(&mut c, &r, (44, 38));
    travel(&mut c, GameButton::Up, DEPOT, (15, 21));
    indoor_walk(&mut c, &r, (19, 16));
    talk(&mut c, GameButton::Right);
    assert!(
        c.snapshot()
            .expect("dispatch direction")
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|s| s.contains("dispatch"))
    );
    finish(&mut c);
    indoor_walk(&mut c, &r, (15, 21));
    travel(&mut c, GameButton::Down, "MnStPaul", (44, 38));
    indoor_walk(&mut c, &r, (1, 24));
    travel(&mut c, GameButton::Left, "MnR04C00R00", (55, 18));
    road(&mut c, &r, (0, 18));
    travel(&mut c, GameButton::Left, "MnMinneapolis", (63, 24));
    travel(&mut c, GameButton::Left, "MnMinneapolis", (59, 24));
    assert!(!event(&c, NOTE) && !event(&c, WIN) && !event(&c, READ));
    assert!(!event(&c, "EVENT_MN_GYM_4_DEFEATED"));
    let _ = continued(c, &root, &r, "courier-local-road-return");
}

pub(super) fn acknowledge_manifest(
    root: &AssetRoot,
    r: &CrystalRuntime,
    mut c: VisibleShellController,
) -> VisibleShellController {
    assert!(event(&c, WIN) && !event(&c, READ));
    let before = c.snapshot().expect("after win");
    let money = c.shell.shell.session().state().money;
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().money, money);
    talk(&mut c, GameButton::Left);
    super::northstar_dialogue::acknowledge(&mut c, "MS04_MANIFEST", "MnSPManifestText", READ);
    finish(&mut c);
    assert!(event(&c, READ));
    let after = c.snapshot().expect("copied record");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(c.shell.shell.session().state().money, money);
    c = continued(c, root, r, "courier-local-record-read");
    talk(&mut c, GameButton::Left);
    finish(&mut c);
    assert!(event(&c, WIN) && event(&c, READ));
    assert!(!event(&c, "EVENT_MN_DEPOT_CLEARED"));
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, true, false, false, false, false, false]
    );
    c
}

#[test]
#[ignore = "requires courier story via CRYSTAL_WINONA_TEST_PACK"]
fn courier_story_actual_win_separate_manifest_pages_repeat_and_continue() {
    let (root, r, mut c) = fixture("MnMinneapolis", (59, 24), Case::Ready);
    travel(&mut c, GameButton::Right, "MnR04C00R00", (0, 18));
    road(&mut c, &r, (55, 18));
    travel(&mut c, GameButton::Right, "MnStPaul", (0, 24));
    travel(&mut c, GameButton::Right, "MnStPaul", (1, 24));
    indoor_walk(&mut c, &r, (44, 38));
    travel(&mut c, GameButton::Up, DEPOT, (15, 21));
    indoor_walk(&mut c, &r, (19, 16));
    talk(&mut c, GameButton::Right);
    super::northstar_dialogue::acknowledge(&mut c, "MS04", "MnSPCourierSeenText", WIN);
    fight(&mut c, false);
    c = continued(c, &root, &r, "courier-local-earned-win");
    assert!(c.shell.shell.session().state().money > 10000);
    c = acknowledge_manifest(&root, &r, c);
    indoor_walk(&mut c, &r, (15, 21));
    travel(&mut c, GameButton::Down, "MnStPaul", (44, 38));
    indoor_walk(&mut c, &r, (1, 24));
    travel(&mut c, GameButton::Left, "MnR04C00R00", (55, 18));
    road(&mut c, &r, (0, 18));
    travel(&mut c, GameButton::Left, "MnMinneapolis", (63, 24));
    travel(&mut c, GameButton::Left, "MnMinneapolis", (59, 24));
    assert!(event(&c, WIN) && event(&c, READ));
    let _ = continued(c, &root, &r, "courier-local-manifest-return");
}

#[test]
#[ignore = "requires courier story via CRYSTAL_WINONA_TEST_PACK"]
fn courier_story_played_loss_returns_to_clinic_without_defeat_or_record() {
    let (root, r, mut c) = fixture("MnStPaulPokecenter1F", (3, 3), Case::Losing);
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    indoor_walk(&mut c, &r, (3, 5));
    travel(&mut c, GameButton::Down, "MnStPaul", (7, 16));
    indoor_walk(&mut c, &r, (44, 38));
    travel(&mut c, GameButton::Up, DEPOT, (15, 21));
    indoor_walk(&mut c, &r, (19, 16));
    talk(&mut c, GameButton::Right);
    fight(&mut c, true);
    assert_eq!(
        c.snapshot().expect("blackout").overworld.map_name,
        "MnStPaulPokecenter1F"
    );
    assert!(!event(&c, WIN) && !event(&c, READ));
    assert!(c.shell.shell.session().state().money < 10000);
    let _ = continued(c, &root, &r, "courier-local-played-loss");
}
