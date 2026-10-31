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

const PARENT: &str = "MnR16C01R00";
const FIELD: &str = "MnFieldMargin";
const ACTOR: &str = "MN_MARGIN_KENSIGARD";
fn flag(name: &str) -> String {
    format!("EVENT_MN_KENSIGARD_{name}")
}
fn setup(
    prior: &[&str],
    full: bool,
    all_full: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    let mut c = fixture(&root, &r, PARENT, (23, 6));
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
        let snapshot = c.snapshot().expect("native Kensigard capture menus");
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
    panic!("native Kensigard capture did not finish");
}

fn visible(c: &mut VisibleShellController) -> bool {
    c.snapshot()
        .expect("native field actors")
        .visible_objects
        .iter()
        .any(|o| o.object_identifier.as_deref() == Some(ACTOR))
}
fn enter(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (22, 0));
    travel(c, GameButton::Up, FIELD, (16, 31));
}
fn leave(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (16, 31));
    travel(c, GameButton::Down, PARENT, (22, 0));
}
fn read(c: &mut VisibleShellController, r: &CrystalRuntime, at: (i16, i16), direction: GameButton) {
    walk(c, r, at);
    press(c, direction);
    press(c, GameButton::A);
}
fn ask(c: &mut VisibleShellController, yes: bool) {
    press(c, GameButton::Up);
    press(c, GameButton::A);
    choice(c, yes);
}
fn reveal(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (13, 7));
    travel(c, GameButton::Up, FIELD, (13, 6));
    assert!(!event(c, &flag("REVEALED")));
    settle(c, "moving boundary-stone reveal");
    assert!(event(c, &flag("REVEALED")));
    assert!(visible(c));
}
#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native Kensigard field branch"]
fn kensigard_controller_physical_revisit_report_and_reveal_acknowledgments() {
    let (root, r, mut c) = setup(&[], false, false);
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    assert!(!event(&c, &flag("CARETAKER_READ")));
    c = continued(&mut c, &root, &r, "kensigard-held-caretaker.crystalsave");
    settle(&mut c, "caretaker account");
    assert!(event(&c, &flag("CARETAKER_READ")));
    enter(&mut c, &r);
    assert!(!visible(&mut c));
    for (point, direction, key) in [
        ((11, 8), GameButton::Right, "SOIL_READ"),
        ((8, 8), GameButton::Up, "BURIED_EDGE_READ"),
        ((13, 22), GameButton::Right, "GROOVES_FIRST_READ"),
    ] {
        read(&mut c, &r, point, direction);
        assert!(!event(&c, &flag(key)));
        press(&mut c, GameButton::Down);
        at(&mut c, FIELD, point);
        settle(&mut c, "acknowledged field evidence");
        assert!(event(&c, &flag(key)));
    }
    c = continued(&mut c, &root, &r, "kensigard-first-grooves.crystalsave");
    assert!(!event(&c, &flag("GROOVES_REVISIT_READY")));
    read(&mut c, &r, (13, 22), GameButton::Right);
    settle(&mut c, "same-visit grooves");
    assert!(!event(&c, &flag("CHANGING_GROOVES_READ")));
    leave(&mut c, &r);
    enter(&mut c, &r);
    assert!(event(&c, &flag("GROOVES_REVISIT_READY")));
    read(&mut c, &r, (13, 22), GameButton::Right);
    assert!(!event(&c, &flag("CHANGING_GROOVES_READ")));
    settle(&mut c, "changed grooves after physical visit");
    assert!(event(&c, &flag("CHANGING_GROOVES_READ")));
    leave(&mut c, &r);
    read(&mut c, &r, (23, 6), GameButton::Up);
    assert!(!event(&c, &flag("DIRECTIONS_ACKNOWLEDGED")));
    settle(&mut c, "caretaker comparison");
    assert!(event(&c, &flag("DIRECTIONS_ACKNOWLEDGED")));
    enter(&mut c, &r);
    read(&mut c, &r, (10, 10), GameButton::Left);
    assert!(!event(&c, &flag("BOUNDARY_LOCATED")));
    settle(&mut c, "old boundary marker");
    leave(&mut c, &r);
    read(&mut c, &r, (23, 6), GameButton::Up);
    assert!(!event(&c, &flag("NOTICE_CORRECTED")));
    settle(&mut c, "reported boundary correction");
    assert!(event(&c, &flag("BOUNDARY_REPORTED")) && event(&c, &flag("NOTICE_CORRECTED")));
    enter(&mut c, &r);
    reveal(&mut c, &r);
    c = continued(&mut c, &root, &r, "kensigard-earned-reveal.crystalsave");
    assert!(event(&c, &flag("REVEALED")) && visible(&mut c));
}
#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native Kensigard field branch"]
fn kensigard_controller_refusal_run_hidden_continue_retry_selected_pc_capture_once() {
    let (root, r, mut c) = setup(&["NOTICE_CORRECTED"], true, false);
    enter(&mut c, &r);
    reveal(&mut c, &r);
    let bag = c.shell.shell.session().state().bag.clone();
    ask(&mut c, false);
    settle(&mut c, "voluntary field refusal");
    assert!(visible(&mut c));
    assert!(c.shell.shell.session().state().bag == bag);
    ask(&mut c, true);
    super::rochester::flee(&mut c);
    settle(&mut c, "actual field escape");
    assert!(!event(&c, &flag("CAUGHT")) && !visible(&mut c));
    c = continued(&mut c, &root, &r, "kensigard-withdrawn.crystalsave");
    assert!(!visible(&mut c));
    leave(&mut c, &r);
    enter(&mut c, &r);
    walk(&mut c, &r, (13, 6));
    assert!(visible(&mut c));
    ask(&mut c, true);
    capture(&mut c);
    assert!(event(&c, &flag("CAUGHT")) && !visible(&mut c));
    let state = c.shell.shell.session().state();
    assert!(state.pokedex.has_caught("KENSIGARD"));
    assert!(
        state.storage.pc_boxes[3]
            .pokemon
            .iter()
            .flatten()
            .any(|p| p.species.id == "KENSIGARD")
    );
    assert_eq!(state.bag.quantity(&r.data().items["MASTER_BALL"]), 0);
    c = continued(&mut c, &root, &r, "kensigard-caught.crystalsave");
    leave(&mut c, &r);
    enter(&mut c, &r);
    assert!(event(&c, &flag("CAUGHT")) && !visible(&mut c));
}
#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with native Kensigard field branch"]
fn kensigard_controller_full_storage_preserves_ball_and_retry() {
    let (root, r, mut c) = setup(&["NOTICE_CORRECTED"], true, true);
    enter(&mut c, &r);
    reveal(&mut c, &r);

    ask(&mut c, true);
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
    c = continued(&mut c, &root, &r, "kensigard-full-storage.crystalsave");
    assert!(!visible(&mut c));
    leave(&mut c, &r);
    enter(&mut c, &r);
    walk(&mut c, &r, (13, 6));
    assert!(visible(&mut c));
}
