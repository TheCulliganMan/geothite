//! Initial local fixtures only; later journeys and captures use production input.
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

const PARENT: &str = "MnR16C00R00";
const FIELD: &str = "MnFormerShores";
const ROOM: &str = "MnInnerBasin";
const ACTOR: &str = "MN_BASIN_STRANDRAK";
fn flag(name: &str) -> String {
    format!("EVENT_MN_BASIN_{name}")
}
fn setup(
    prior: &[&str],
    full: bool,
    all_full: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    let mut c = fixture(&root, &r, PARENT, (42, 46));
    let state = c.shell.shell.session_mut().state_mut();
    let mut partner = r
        .data()
        .create_pokemon("POUCHID", 50, Dv::default())
        .expect("native fixture partner");
    partner.item = Some("SMOKE_BALL".into());
    assert!(state.storage.party.add_pokemon(partner));
    if full {
        for slot in 1..6 {
            state.storage.party.pokemon[slot] = Some(
                r.data()
                    .create_pokemon("POUCHID", 50, Dv::default())
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
                            .create_pokemon("POUCHID", 50, Dv::default())
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
    for name in prior {
        state
            .flags
            .set_event_flag(&flag(name), true)
            .expect("declared initial evidence");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64)
        .expect("initial native route presentation");
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
fn capture(c: &mut VisibleShellController) {
    for _ in 0..256 {
        let snapshot = c.snapshot().expect("native Strandrak capture menus");
        if event(c, &flag("CAUGHT")) && snapshot.battle.is_none() {
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
            || (snapshot.ui.menu.is_none() && snapshot.ui.text.is_some())
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
    panic!("native Strandrak capture did not finish");
}

fn visible(c: &mut VisibleShellController) -> bool {
    c.snapshot()
        .expect("native field actors")
        .visible_objects
        .iter()
        .any(|o| o.object_identifier.as_deref() == Some(ACTOR))
}

fn enter(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (42, 47));
    travel(c, GameButton::Down, FIELD, (14, 0));
}
fn leave(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (14, 0));
    travel(c, GameButton::Up, PARENT, (42, 47));
}
fn descend(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (18, 24));
    travel(c, GameButton::Up, ROOM, (2, 25));
}
fn ascend(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (3, 24));
    travel(c, GameButton::Down, FIELD, (18, 24));
}
fn read(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    point: (i16, i16),
    direction: GameButton,
) {
    walk(c, r, point);
    press(c, direction);
    press(c, GameButton::A);
}
fn ask(c: &mut VisibleShellController, r: &CrystalRuntime, yes: bool) {
    read(c, r, (14, 3), GameButton::Up);
    choice(c, yes);
}
#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native former-shores/inner-basin discovery"]
fn strandrak_controller_terraces_all_dials_held_outlet_native_stairs_and_continue() {
    let (root, r, mut c) = setup(&[], false, false);
    enter(&mut c, &r);
    read(&mut c, &r, (17, 6), GameButton::Up);
    assert!(!event(&c, &flag("SURVEYOR_READ")));
    settle(&mut c, "surveyor account");
    assert!(event(&c, &flag("SURVEYOR_READ")));
    for (point, key) in [
        ((16, 9), "HIGH_SHORE_READ"),
        ((20, 15), "MIDDLE_SHORE_READ"),
        ((12, 21), "LOW_SHORE_READ"),
    ] {
        read(&mut c, &r, point, GameButton::Up);
        assert!(!event(&c, &flag(key)));
        press(&mut c, GameButton::Right);
        at(&mut c, FIELD, point);
        settle(&mut c, "shore mark acknowledged");
        assert!(event(&c, &flag(key)));
    }
    read(&mut c, &r, (17, 6), GameButton::Up);
    assert!(!event(&c, &flag("SHORES_COMPARED")));
    c = continued(
        &mut c,
        &root,
        &r,
        "strandrak-held-shore-comparison.crystalsave",
    );
    settle(&mut c, "three former shores compared");
    assert!(event(&c, &flag("SHORES_COMPARED")));
    descend(&mut c, &r);
    at(&mut c, ROOM, (2, 25));
    c = continued(
        &mut c,
        &root,
        &r,
        "strandrak-passive-cave-arrival.crystalsave",
    );
    ascend(&mut c, &r);
    at(&mut c, FIELD, (18, 24));
    descend(&mut c, &r);
    walk(&mut c, &r, (14, 8));
    press(&mut c, GameButton::Up);
    at(&mut c, ROOM, (14, 8));
    let bag = c.shell.shell.session().state().bag.clone();
    read(&mut c, &r, (15, 20), GameButton::Left);
    settle(&mut c, "untouched dials denial");
    assert!(
        !event(&c, &flag("INNER_OUTLET_ACKNOWLEDGED"))
            && c.shell.shell.session().state().bag == bag
    );
    read(&mut c, &r, (10, 10), GameButton::Left);
    assert!(!event(&c, &flag("UPPER_TOUCHED")));
    settle(&mut c, "upper middle setting");
    assert!(event(&c, &flag("UPPER_MIDDLE")));
    read(&mut c, &r, (10, 10), GameButton::Left);
    settle(&mut c, "upper high setting");
    assert!(event(&c, &flag("UPPER_HIGH")));
    read(&mut c, &r, (17, 14), GameButton::Right);
    settle(&mut c, "center middle setting");
    assert!(event(&c, &flag("CENTER_MIDDLE")));
    read(&mut c, &r, (15, 20), GameButton::Left);
    settle(&mut c, "lower default is still untouched");
    assert!(!event(&c, &flag("LOWER_TOUCHED")) && !event(&c, &flag("INNER_OUTLET_ACKNOWLEDGED")));
    assert!(event(&c, &flag("UPPER_HIGH")) && event(&c, &flag("CENTER_MIDDLE")));
    for _ in 0..3 {
        read(&mut c, &r, (12, 18), GameButton::Left);
        settle(&mut c, "lower indicator acknowledgment");
    }
    assert!(
        event(&c, &flag("LOWER_TOUCHED"))
            && !event(&c, &flag("LOWER_MIDDLE"))
            && !event(&c, &flag("LOWER_HIGH"))
    );
    read(&mut c, &r, (15, 20), GameButton::Left);
    assert!(!event(&c, &flag("INNER_OUTLET_ACKNOWLEDGED")));
    c = continued(&mut c, &root, &r, "strandrak-held-inner-outlet.crystalsave");
    assert!(!event(&c, &flag("INNER_OUTLET_ACKNOWLEDGED")));
    settle(&mut c, "small inner outlet acknowledged");
    assert!(event(&c, &flag("INNER_OUTLET_ACKNOWLEDGED")));
    walk(&mut c, &r, (14, 8));
    travel(&mut c, GameButton::Up, ROOM, (14, 7));
    walk(&mut c, &r, (14, 3));
    assert!(visible(&mut c));
    c = continued(
        &mut c,
        &root,
        &r,
        "strandrak-drained-inner-basin.crystalsave",
    );
    ascend(&mut c, &r);
    leave(&mut c, &r);
    enter(&mut c, &r);
    descend(&mut c, &r);
    walk(&mut c, &r, (14, 3));
    assert!(visible(&mut c) && !event(&c, &flag("CAUGHT")));
}
#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native former-shores/inner-basin discovery"]
fn strandrak_controller_refusal_run_hidden_continue_physical_retry_selected_pc_capture_once() {
    let (root, r, mut c) = setup(&["INNER_OUTLET_ACKNOWLEDGED"], true, false);
    enter(&mut c, &r);
    descend(&mut c, &r);
    ask(&mut c, &r, false);
    settle(&mut c, "optional basin refusal");
    assert!(visible(&mut c));
    ask(&mut c, &r, true);
    super::rochester::flee(&mut c);
    settle(&mut c, "actual Strandrak escape");
    assert!(!event(&c, &flag("CAUGHT")) && !visible(&mut c));
    c = continued(&mut c, &root, &r, "strandrak-withdrawn.crystalsave");
    assert!(!visible(&mut c));
    ascend(&mut c, &r);
    descend(&mut c, &r);
    walk(&mut c, &r, (14, 3));
    assert!(visible(&mut c));
    ask(&mut c, &r, true);
    capture(&mut c);
    assert!(event(&c, &flag("CAUGHT")) && !visible(&mut c));
    let state = c.shell.shell.session().state();
    assert!(state.pokedex.has_caught("STRANDRAK"));
    assert!(
        state.storage.pc_boxes[3]
            .pokemon
            .iter()
            .flatten()
            .any(|p| p.species.id == "STRANDRAK")
    );
    assert_eq!(state.bag.quantity(&r.data().items["MASTER_BALL"]), 0);
    c = continued(&mut c, &root, &r, "strandrak-caught.crystalsave");
    ascend(&mut c, &r);
    descend(&mut c, &r);
    walk(&mut c, &r, (14, 3));
    assert!(event(&c, &flag("CAUGHT")) && !visible(&mut c));
}
#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native Strandrak field branch"]
fn strandrak_controller_full_storage_preserves_ball_and_retry() {
    let (root, r, mut c) = setup(&["INNER_OUTLET_ACKNOWLEDGED"], true, true);
    enter(&mut c, &r);
    descend(&mut c, &r);

    ask(&mut c, &r, true);
    // Battle entry initializes its turn counter. Compare capacity refusal
    // with the actual battle state, without changing any partner or box.
    for _ in 0..128 {
        if c.shell.battle_action_cursor.is_some() {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(
        c.shell.battle_action_cursor.is_some(),
        "native battle action boundary"
    );
    let storage = c.shell.shell.session().state().storage.clone();
    let mut refused = false;
    for _ in 0..192 {
        let snapshot = c.snapshot().expect("native full-box encounter");
        if snapshot
            .ui
            .text
            .as_ref()
            .and_then(|text| text.asm_text.as_deref())
            .is_some_and(|text| text.contains("BOX") && text.contains("full"))
        {
            refused = true;
            break;
        }
        let button = if (snapshot.ui.menu.is_none() && snapshot.ui.text.is_some())
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
    assert!(refused, "native full-storage refusal");
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["MASTER_BALL"]),
        1
    );
    assert!(c.shell.shell.session().state().storage == storage);
    assert!(!event(&c, &flag("CAUGHT")));
    for _ in 0..24 {
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
    settle(&mut c, "full-storage escape");
    c = continued(&mut c, &root, &r, "strandrak-full-storage.crystalsave");
    assert!(!visible(&mut c));
    ascend(&mut c, &r);
    descend(&mut c, &r);
    walk(&mut c, &r, (14, 3));
    assert!(visible(&mut c));
}
