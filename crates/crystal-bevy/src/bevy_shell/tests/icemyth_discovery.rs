//! Local native party/position fixtures; all later quest and storage actions use input.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, MAX_BOX_MONS, MAX_PC_BOXES, PcBox};
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::authored_route_fixture::{cleanup_reached_save, reached_save_path};
use super::interior_life::fixture;
use super::northstar_ice_lair::{choice, event, walk};
use super::r10_gallery::settle;
use super::winona::{at, load, press, travel};

const HALL: &str = "MnStPaulFairExhibitionHall";
const ROOM: &str = "MnIcemythServicePassage";
const ACTOR: &str = "MN_EXHIBIT_ICEMYTH";
const FROZEN: &str = "MN_EXHIBIT_FROZEN_CASE";

fn flag(name: &str) -> String {
    format!("EVENT_MN_ICEMYTH_{name}")
}
fn setup(full: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    setup_storage(full, false)
}
fn setup_storage(
    full: bool,
    all_full: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    setup_team(full, all_full, "POUCHID", 50)
}
fn setup_team(
    full: bool,
    all_full: bool,
    species: &str,
    level: u8,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    let mut c = fixture(&root, &r, HALL, (3, 9));
    let state = c.shell.shell.session_mut().state_mut();
    let mut partner = r
        .data()
        .create_pokemon(species, level, Dv::default())
        .expect("native fixture partner");
    partner.item = Some("SMOKE_BALL".into());
    assert!(state.storage.party.add_pokemon(partner));
    if full {
        for slot in 1..6 {
            state.storage.party.pokemon[slot] = Some(
                r.data()
                    .create_pokemon(species, level, Dv::default())
                    .expect("declared full native party"),
            );
        }
        state.current_pc_box = 3;
        while state.storage.pc_boxes.len() <= 3 {
            state
                .storage
                .pc_boxes
                .push(PcBox::new(state.storage.pc_boxes.len()));
        }
        assert_eq!(
            state.storage.pc_boxes[3].pokemon.iter().flatten().count(),
            0
        );
    }
    if all_full {
        assert!(full, "full-storage fixture must also fill the party");
        while state.storage.pc_boxes.len() < MAX_PC_BOXES {
            state
                .storage
                .pc_boxes
                .push(PcBox::new(state.storage.pc_boxes.len()));
        }
        for pc_box in &mut state.storage.pc_boxes {
            for _ in 0..MAX_BOX_MONS {
                assert!(
                    pc_box.add_pokemon(
                        r.data()
                            .create_pokemon(species, level, Dv::default())
                            .expect("declared full native storage")
                    )
                );
            }
        }
    }
    state.sync_party_from_storage();
    assert!(
        state
            .bag
            .add_item(&r.data().items["MASTER_BALL"], 1)
            .expect("declared capture resource")
    );
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64)
        .expect("initial native exhibit presentation");
    // No later party, PC, bag, flag, position, HP, PP, mode or RNG writes.
    (root, r, c)
}
fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    name: &str,
) -> VisibleShellController {
    let path = reached_save_path(name);
    c.save(&path).expect("production exhibit save");
    let loaded = VisibleShellController::load_save(root.clone(), r.clone(), path.clone(), None)
        .expect("same-pack exhibit Continue");
    cleanup_reached_save(&path);
    loaded
}
fn visible(c: &mut VisibleShellController, id: &str) -> bool {
    c.snapshot()
        .expect("native exhibit actors")
        .visible_objects
        .iter()
        .any(|o| o.object_identifier.as_deref() == Some(id))
}
fn open(c: &mut VisibleShellController, r: &CrystalRuntime, to: (i16, i16), facing: GameButton) {
    walk(c, r, to);
    press(c, facing);
    press(c, GameButton::A);
}
fn read(c: &mut VisibleShellController, r: &CrystalRuntime, to: (i16, i16), facing: GameButton) {
    open(c, r, to, facing);
    settle(c, "acknowledged exhibit observation");
}
fn enter(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (13, 8));
    travel(c, GameButton::Left, ROOM, (3, 11));
}
fn leave(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (2, 11));
    travel(c, GameButton::Up, HALL, (13, 8));
}
fn discover(c: &mut VisibleShellController, root: &AssetRoot, r: &CrystalRuntime, stage: &str) {
    walk(c, r, (10, 9));
    assert!(visible(c, FROZEN));
    open(c, r, (10, 9), GameButton::Up);
    assert!(
        c.snapshot()
            .expect("held attendant account")
            .ui
            .text
            .is_some()
    );
    assert!(!event(c, &flag("ATTENDANT_ACCOUNT")));
    press(c, GameButton::Right);
    at(c, HALL, (10, 9));
    *c = continued(
        c,
        root,
        r,
        &format!("icemyth-{stage}-held-introduction.crystalsave"),
    );
    assert!(!event(c, &flag("ATTENDANT_ACCOUNT")));
    settle(c, "acknowledged attendant account");
    assert!(event(c, &flag("ATTENDANT_ACCOUNT")));
    read(c, r, (7, 9), GameButton::Up);
    assert!(event(c, &flag("DELIVERY_ACCOUNT")));
    read(c, r, (4, 9), GameButton::Up);
    assert!(event(c, &flag("VISITOR_ACCOUNT")));
    open(c, r, (9, 7), GameButton::Down);
    assert!(!event(c, &flag("LATCH_READ")));
    *c = continued(
        c,
        root,
        r,
        &format!("icemyth-{stage}-held-latch.crystalsave"),
    );
    settle(c, "acknowledged intact latch");
    assert!(event(c, &flag("LATCH_READ")));
    read(c, r, (13, 8), GameButton::Up);
    assert!(event(c, &flag("TRAY_READ")));
    assert!(!event(c, &flag("LABEL_CORRECTED")));
    assert!(visible(c, FROZEN));
    open(c, r, (10, 9), GameButton::Up);
    assert!(!event(c, &flag("LABEL_CORRECTED")));
    *c = continued(
        c,
        root,
        r,
        &format!("icemyth-{stage}-held-comparison.crystalsave"),
    );
    settle(c, "acknowledged corrected exhibit account");
    assert!(event(c, &flag("LABEL_CORRECTED")));
    assert!(!visible(c, FROZEN));
    enter(c, r);
    assert!(!visible(c, ACTOR));
    open(c, r, (6, 4), GameButton::Left);
    assert!(!event(c, &flag("MOVED_ICE_FOUND")));
    *c = continued(
        c,
        root,
        r,
        &format!("icemyth-{stage}-held-empty-case.crystalsave"),
    );
    settle(c, "acknowledged empty case and local reveal");
    assert!(event(c, &flag("MOVED_ICE_FOUND")));
    assert!(visible(c, ACTOR));
    assert!(!event(c, &flag("CAUGHT")));
}
fn ask(c: &mut VisibleShellController, r: &CrystalRuntime, yes: bool) {
    open(c, r, (9, 6), GameButton::Up);
    choice(c, yes);
}
fn battle(c: &mut VisibleShellController) {
    for _ in 0..128 {
        if c.shell.battle_action_cursor.is_some() {
            assert!(
                c.snapshot()
                    .expect("actual Icemyth battle")
                    .battle
                    .is_some()
            );
            return;
        }
        press(c, GameButton::A);
    }
    panic!("native optional Icemyth battle did not start");
}
fn capture(c: &mut VisibleShellController) {
    for _ in 0..256 {
        let s = c.snapshot().expect("native capture menus");
        if event(c, &flag("CAUGHT")) && s.battle.is_none() {
            settle(c, "capture continuation");
            return;
        }
        if let Some(choice) = &c.shell.pending_name_choice {
            if choice.nickname_pages.len() > 1 {
                press(c, GameButton::A);
            } else {
                press(c, GameButton::Down);
                press(c, GameButton::A);
            }
            continue;
        }
        let button = if c.shell.pokedex_menu_open
            || (s.ui.menu.is_none() && s.ui.text.is_some())
            || c.shell.field_pack_action_cursor.is_some()
            || c.shell.ball_cursor.is_some()
        {
            GameButton::A
        } else if c.shell.bag_cursor.is_some() {
            GameButton::Right
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            match cursor.option_index {
                0 | 1 => GameButton::Down,
                3 => GameButton::Left,
                _ => GameButton::A,
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    panic!("native Icemyth capture did not finish");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native Icemyth exhibit/service room"]
fn icemyth_controller_stairs_early_case_town_door_and_continue() {
    let (root, r, mut c) = setup(false);
    enter(&mut c, &r);
    c = continued(
        &mut c,
        &root,
        &r,
        "icemyth-early-service-arrival.crystalsave",
    );
    at(&mut c, ROOM, (3, 11));
    read(&mut c, &r, (6, 4), GameButton::Left);
    assert!(!event(&c, &flag("MOVED_ICE_FOUND")));
    assert!(!visible(&mut c, ACTOR));
    leave(&mut c, &r);
    c = continued(
        &mut c,
        &root,
        &r,
        "icemyth-hall-passive-arrival.crystalsave",
    );
    at(&mut c, HALL, (13, 8));
    travel(&mut c, GameButton::Down, HALL, (13, 9));
    walk(&mut c, &r, (3, 10));
    for _ in 0..8 {
        press(&mut c, GameButton::Down);
        if c.snapshot().expect("ordinary hall exit").overworld.map_name == "MnStPaul" {
            break;
        }
    }
    assert_eq!(
        c.snapshot().expect("ordinary hall exit").overworld.map_name,
        "MnStPaul"
    );
    let door = r.data().maps["MnStPaul"]
        .events
        .warps
        .iter()
        .find(|w| {
            Some(w.target_map_constant.as_str())
                == r.data().maps[HALL].attributes.map_constant.as_deref()
                && w.target_warp_id == 3
        })
        .expect("original exhibition hall street door");
    assert_eq!(
        door.target_map_constant,
        r.data().maps[HALL]
            .attributes
            .map_constant
            .as_deref()
            .expect("hall constant")
    );
    let target = (
        i16::try_from(door.x).expect("door x"),
        i16::try_from(door.y).expect("door y") + 1,
    );
    walk(&mut c, &r, target);
    travel(&mut c, GameButton::Up, HALL, (3, 9));
    walk(&mut c, &r, (10, 9));
    assert!(visible(&mut c, FROZEN));
    assert!(!event(&c, &flag("LABEL_CORRECTED")));
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native Icemyth exhibit/service room"]
fn icemyth_controller_five_accounts_acknowledgment_reveal_refusal_and_continue() {
    let (root, r, mut c) = setup(false);
    let bag = c.shell.shell.session().state().bag.clone();
    discover(&mut c, &root, &r, "observations");
    ask(&mut c, &r, false);
    settle(&mut c, "voluntary Icemyth refusal");
    assert!(visible(&mut c, ACTOR));
    assert!(!event(&c, &flag("CAUGHT")));
    assert!(c.shell.shell.session().state().bag == bag);
    c = continued(&mut c, &root, &r, "icemyth-revealed-refusal.crystalsave");
    assert!(visible(&mut c, ACTOR));
    leave(&mut c, &r);
    assert!(!visible(&mut c, FROZEN));
    enter(&mut c, &r);
    assert!(visible(&mut c, ACTOR));
    assert!(event(&c, &flag("LABEL_CORRECTED")) && event(&c, &flag("MOVED_ICE_FOUND")));
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native Icemyth exhibit/service room"]
fn icemyth_controller_run_hidden_continue_physical_retry_selected_pc_capture_once() {
    let (root, r, mut c) = setup(true);
    let original = c.shell.shell.session().state().storage.clone();
    discover(&mut c, &root, &r, "full-party");
    ask(&mut c, &r, true);
    battle(&mut c);
    super::rochester::flee(&mut c);
    settle(&mut c, "actual Icemyth RUN");
    assert!(!visible(&mut c, ACTOR));
    assert!(!event(&c, &flag("CAUGHT")));
    c = continued(&mut c, &root, &r, "icemyth-withdrawn-visit.crystalsave");
    assert!(!visible(&mut c, ACTOR));
    read(&mut c, &r, (6, 4), GameButton::Left);
    assert!(
        !visible(&mut c, ACTOR),
        "rechecking the case must not bypass physical reentry"
    );
    leave(&mut c, &r);
    enter(&mut c, &r);
    assert!(visible(&mut c, ACTOR));
    ask(&mut c, &r, true);
    capture(&mut c);
    assert!(event(&c, &flag("CAUGHT")));
    assert!(!visible(&mut c, ACTOR));
    let state = c.shell.shell.session().state();
    assert_eq!(state.current_pc_box, 3);
    assert_eq!(state.storage.party, original.party);
    for (index, before) in original.pc_boxes.iter().enumerate() {
        if index != 3 {
            assert_eq!(&state.storage.pc_boxes[index], before);
        }
    }
    assert_eq!(
        state.storage.pc_boxes[3].pokemon.iter().flatten().count(),
        1
    );
    assert_eq!(
        state.storage.pc_boxes[3].pokemon[0]
            .as_ref()
            .expect("selected PC capture")
            .species
            .id,
        "ICEMYTH"
    );
    assert!(state.pokedex.has_caught("ICEMYTH"));
    assert_eq!(state.bag.quantity(&r.data().items["MASTER_BALL"]), 0);
    let captured = state.storage.clone();
    c = continued(&mut c, &root, &r, "icemyth-selected-pc-capture.crystalsave");
    assert!(event(&c, &flag("CAUGHT")));
    assert!(!visible(&mut c, ACTOR));
    assert_eq!(c.shell.shell.session().state().storage, captured);
    leave(&mut c, &r);
    enter(&mut c, &r);
    assert!(!visible(&mut c, ACTOR));
    assert_eq!(c.shell.shell.session().state().storage, captured);
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native Icemyth exhibit/service room"]
fn icemyth_controller_full_storage_refusal_retains_ball_caught_state_and_retry() {
    let (root, r, mut c) = setup_storage(true, true);
    let original_storage = c.shell.shell.session().state().storage.clone();
    let original_bag = c.shell.shell.session().state().bag.clone();
    discover(&mut c, &root, &r, "all-storage-full");
    ask(&mut c, &r, true);
    battle(&mut c);
    // Native battle entry initializes turns_in_battle. Compare the refused
    // transaction with that reached state, preserving every stored partner.
    let battle_storage = c.shell.shell.session().state().storage.clone();
    let mut refused = false;
    for _ in 0..128 {
        let s = c.snapshot().expect("native full-box capture attempt");
        assert!(
            s.battle.is_some(),
            "capacity refusal must retain the battle"
        );
        if s.ui
            .text
            .as_ref()
            .and_then(|text| text.asm_text.as_deref())
            .is_some_and(|text| text.contains("BOX") && text.contains("full"))
        {
            refused = true;
            break;
        }
        let button = if (s.ui.menu.is_none() && s.ui.text.is_some())
            || c.shell.field_pack_action_cursor.is_some()
            || c.shell.ball_cursor.is_some()
        {
            GameButton::A
        } else if c.shell.bag_cursor.is_some() {
            GameButton::Right
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            match cursor.option_index {
                0 | 1 => GameButton::Down,
                3 => GameButton::Left,
                _ => GameButton::A,
            }
        } else {
            GameButton::A
        };
        press(&mut c, button);
    }
    assert!(refused, "native capture must show the full-box refusal");
    let state = c.shell.shell.session().state();
    assert_eq!(
        state.storage.pc_boxes, original_storage.pc_boxes,
        "full native PC boxes"
    );
    for (index, (actual, expected)) in state
        .storage
        .party
        .pokemon
        .iter()
        .zip(&battle_storage.party.pokemon)
        .enumerate()
    {
        assert_eq!(
            actual, expected,
            "party slot {index} after native capacity refusal"
        );
    }
    assert!(state.bag == original_bag);
    assert!(!state.pokedex.has_caught("ICEMYTH"));
    assert!(!event(&c, &flag("CAUGHT")));
    for _ in 0..16 {
        if c.shell.battle_action_cursor.is_some()
            && c.shell.bag_cursor.is_none()
            && c.shell.ball_cursor.is_none()
            && c.shell.field_pack_action_cursor.is_none()
        {
            break;
        }
        press(&mut c, GameButton::B);
    }
    super::rochester::flee(&mut c);
    settle(&mut c, "native full-storage RUN");
    c = continued(
        &mut c,
        &root,
        &r,
        "icemyth-full-storage-refusal.crystalsave",
    );
    assert!(!visible(&mut c, ACTOR));
    assert!(!event(&c, &flag("CAUGHT")));
    leave(&mut c, &r);
    enter(&mut c, &r);
    assert!(visible(&mut c, ACTOR));
    ask(&mut c, &r, true);
    battle(&mut c);
    super::rochester::flee(&mut c);
    settle(&mut c, "full-storage retry escape");
    let state = c.shell.shell.session().state();
    assert!(state.storage == original_storage && state.bag == original_bag);
    assert!(!state.pokedex.has_caught("ICEMYTH"));
    assert!(!event(&c, &flag("CAUGHT")));
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native Icemyth exhibit/service room"]
fn icemyth_controller_actual_knockout_spends_pp_and_retries_after_physical_reentry() {
    let (root, r, mut c) = setup_team(true, false, "BOREALYNX", 50);
    let before = c.shell.shell.session().state().storage.party.clone();
    discover(&mut c, &root, &r, "knockout");
    ask(&mut c, &r, true);
    battle(&mut c);
    assert!(
        super::rochester_earned::try_battle(&mut c, &r, "WILD", ""),
        "declared native counters must win through actual combat"
    );
    settle(&mut c, "actual Icemyth knockout continuation");
    let after = &c.shell.shell.session().state().storage.party;
    assert!(
        before
            .pokemon
            .iter()
            .zip(&after.pokemon)
            .any(|(before, after)| {
                match (before, after) {
                    (Some(before), Some(after)) => {
                        before
                            .moves
                            .iter()
                            .zip(&after.moves)
                            .any(|(before, after)| {
                                before.name == after.name && after.current_pp < before.current_pp
                            })
                    }
                    _ => false,
                }
            }),
        "native combat must spend move PP"
    );
    assert!(!event(&c, &flag("CAUGHT")));
    assert!(!visible(&mut c, ACTOR));
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .pokedex
            .has_caught("ICEMYTH")
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["MASTER_BALL"]),
        1
    );
    c = continued(&mut c, &root, &r, "icemyth-knockout-withdrawn.crystalsave");
    assert!(!visible(&mut c, ACTOR));
    leave(&mut c, &r);
    enter(&mut c, &r);
    assert!(visible(&mut c, ACTOR));
    ask(&mut c, &r, true);
    battle(&mut c);
    super::rochester::flee(&mut c);
    settle(&mut c, "knockout retry escape");
    assert!(!event(&c, &flag("CAUGHT")));
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .pokedex
            .has_caught("ICEMYTH")
    );
}
