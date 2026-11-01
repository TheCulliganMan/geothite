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

const PARENT: &str = "MnBemidjiNorthwoods";
const FIELD: &str = "MnLayeredFen";
const ACTOR: &str = "MN_FEN_FLARKINDLE";
fn flag(name: &str) -> String {
    format!("EVENT_MN_FEN_{name}")
}
fn setup(
    prior: &[&str],
    full: bool,
    all_full: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    let mut c = fixture(&root, &r, PARENT, (6, 16));
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
        let snapshot = c.snapshot().expect("native Flarkindle capture menus");
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
    panic!("native Flarkindle capture did not finish");
}

fn visible(c: &mut VisibleShellController) -> bool {
    c.snapshot()
        .expect("native field actors")
        .visible_objects
        .iter()
        .any(|o| o.object_identifier.as_deref() == Some(ACTOR))
}

fn enter(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (0, 16));
    travel(c, GameButton::Left, FIELD, (39, 16));
}
fn leave(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (39, 16));
    travel(c, GameButton::Right, PARENT, (0, 16));
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
    read(c, r, (32, 6), GameButton::Up);
    choice(c, yes);
}
#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native layered-fen discovery"]
fn flarkindle_controller_clues_wrong_order_held_solution_and_restored_native_path() {
    let (root, r, mut c) = setup(&[], false, false);
    enter(&mut c, &r);
    read(&mut c, &r, (35, 17), GameButton::Right);
    assert!(!event(&c, &flag("OBSERVER_READ")));
    settle(&mut c, "fen observer account");
    assert!(event(&c, &flag("OBSERVER_READ")));
    walk(&mut c, &r, (25, 8));
    press(&mut c, GameButton::Right);
    at(&mut c, FIELD, (25, 8));
    for (point, direction, key) in [
        ((11, 12), GameButton::Right, "ROOT_READ"),
        ((11, 18), GameButton::Right, "POOL_READ"),
        ((29, 12), GameButton::Right, "RIB_READ"),
    ] {
        read(&mut c, &r, point, direction);
        assert!(!event(&c, &flag(key)));
        press(&mut c, GameButton::Left);
        at(&mut c, FIELD, point);
        settle(&mut c, "acknowledged fen layer");
        assert!(event(&c, &flag(key)));
    }
    read(&mut c, &r, (35, 17), GameButton::Right);
    assert!(!event(&c, &flag("LAYERS_COMPARED")));
    c = continued(&mut c, &root, &r, "flarkindle-held-comparison.crystalsave");
    settle(&mut c, "layer comparison");
    assert!(event(&c, &flag("LAYERS_COMPARED")));
    let bag = c.shell.shell.session().state().bag.clone();
    read(&mut c, &r, (11, 20), GameButton::Right);
    settle(&mut c, "wrong sequence feedback");
    assert!(!event(&c, &flag("SEQUENCE_ROOT")) && !event(&c, &flag("SEQUENCE_POOL")));
    assert!(c.shell.shell.session().state().bag == bag);
    assert!(
        event(&c, &flag("ROOT_READ"))
            && event(&c, &flag("POOL_READ"))
            && event(&c, &flag("RIB_READ"))
    );
    read(&mut c, &r, (11, 15), GameButton::Right);
    assert!(!event(&c, &flag("SEQUENCE_ROOT")));
    settle(&mut c, "older root selected");
    assert!(event(&c, &flag("SEQUENCE_ROOT")));
    read(&mut c, &r, (11, 20), GameButton::Right);
    settle(&mut c, "pool selected");
    assert!(event(&c, &flag("SEQUENCE_POOL")));
    read(&mut c, &r, (27, 14), GameButton::Right);
    assert!(!event(&c, &flag("FOLDED_PATH_REVEALED")));
    c = continued(&mut c, &root, &r, "flarkindle-held-folded-path.crystalsave");
    assert!(!event(&c, &flag("FOLDED_PATH_REVEALED")));
    settle(&mut c, "living peat and folded path acknowledged");
    assert!(event(&c, &flag("FOLDED_PATH_REVEALED")));
    walk(&mut c, &r, (25, 8));
    travel(&mut c, GameButton::Right, FIELD, (26, 8));
    walk(&mut c, &r, (32, 6));
    assert!(visible(&mut c));
    c = continued(&mut c, &root, &r, "flarkindle-open-hollow.crystalsave");
    walk(&mut c, &r, (25, 8));
    leave(&mut c, &r);
    enter(&mut c, &r);
    walk(&mut c, &r, (32, 6));
    assert!(visible(&mut c));
    assert!(!event(&c, &flag("CAUGHT")));
}
#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native layered-fen discovery"]
fn flarkindle_controller_refusal_run_withdrawn_continue_reentry_selected_pc_capture_once() {
    let (root, r, mut c) = setup(&["FOLDED_PATH_REVEALED"], true, false);
    enter(&mut c, &r);
    ask(&mut c, &r, false);
    settle(&mut c, "fen refusal");
    assert!(visible(&mut c));
    ask(&mut c, &r, true);
    super::rochester::flee(&mut c);
    settle(&mut c, "actual Flarkindle escape");
    assert!(!visible(&mut c) && !event(&c, &flag("CAUGHT")));
    c = continued(&mut c, &root, &r, "flarkindle-withdrawn.crystalsave");
    assert!(!visible(&mut c));
    leave(&mut c, &r);
    enter(&mut c, &r);
    walk(&mut c, &r, (32, 6));
    assert!(visible(&mut c));
    ask(&mut c, &r, true);
    capture(&mut c);
    assert!(event(&c, &flag("CAUGHT")) && !visible(&mut c));
    let state = c.shell.shell.session().state();
    assert!(state.pokedex.has_caught("FLARKINDLE"));
    assert!(
        state.storage.pc_boxes[3]
            .pokemon
            .iter()
            .flatten()
            .any(|p| p.species.id == "FLARKINDLE")
    );
    assert_eq!(state.bag.quantity(&r.data().items["MASTER_BALL"]), 0);
    c = continued(&mut c, &root, &r, "flarkindle-caught.crystalsave");
    leave(&mut c, &r);
    enter(&mut c, &r);
    assert!(event(&c, &flag("CAUGHT")) && !visible(&mut c));
    walk(&mut c, &r, (32, 6));
}
#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native Flarkindle field branch"]
fn flarkindle_controller_full_storage_preserves_ball_and_retry() {
    let (root, r, mut c) = setup(&["FOLDED_PATH_REVEALED"], true, true);
    enter(&mut c, &r);

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
    c = continued(&mut c, &root, &r, "flarkindle-full-storage.crystalsave");
    assert!(!visible(&mut c));
    leave(&mut c, &r);
    enter(&mut c, &r);
    walk(&mut c, &r, (32, 6));
    assert!(visible(&mut c));
}
