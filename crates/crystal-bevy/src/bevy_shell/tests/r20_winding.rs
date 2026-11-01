//! Local position/party fixtures; every later route, reward and battle uses input.
use std::path::{Path, PathBuf};

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life::{fixture, load};
use super::northstar_ice_lair::{choice, event, path, talk};
use super::r10_gallery::{press, settle, tile};
use super::winona::{at, finish, travel};

const ROOM: &str = "MnR20Cave1F";

fn step(c: &mut VisibleShellController, button: GameButton, target: (i16, i16)) {
    let start = c
        .snapshot()
        .expect("before native stair step")
        .overworld
        .tile;
    for _ in 0..8 {
        press(c, button);
        let position = c.snapshot().expect("native stair step").overworld.tile;
        if (position.x, position.y) == target {
            return;
        }
        assert_eq!(position, start, "unexpected stair movement");
    }
    at(c, ROOM, target);
}

fn walk(c: &mut VisibleShellController, r: &CrystalRuntime, target: (i16, i16)) {
    for _ in 0..1024 {
        if c.shell.shell.session().state().repel_steps_remaining == 0
            && c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items["MAX_REPEL"])
                > 0
        {
            super::minneapolis_falls::grand_marais_harbor::use_max_repel(c, r);
        }
        let snapshot = c.snapshot().expect("native cave walking");
        if (snapshot.overworld.tile.x, snapshot.overworld.tile.y) == target {
            return;
        }
        let button = *path(c, r, target)
            .expect("ordinary cave path")
            .first()
            .expect("remaining cave step");
        let before = snapshot.overworld.tile;
        for _ in 0..8 {
            press(c, button);
            if c.snapshot().expect("native cave step").overworld.tile != before {
                break;
            }
        }
    }
    at(c, ROOM, target);
}

fn output(name: &str) -> PathBuf {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_R20_WINDING_REVIEW_DIR").expect("fresh ignored review directory"),
    )
    .join(name);
    std::fs::create_dir_all(&path).expect("review output");
    path
}

fn setup(
    root: &AssetRoot,
    r: &CrystalRuntime,
    point: (i16, i16),
    injured: bool,
) -> VisibleShellController {
    assert_eq!(
        [
            r.data().maps[ROOM].attributes.width,
            r.data().maps[ROOM].attributes.height
        ],
        [14, 12]
    );
    let mut c = fixture(root, r, ROOM, point);
    let state = c.shell.shell.session_mut().state_mut();
    let species: &[&str] = if injured {
        &["LUTEBRISK"]
    } else {
        &["VOYAGEAR", "FORTIGARD", "CINDORE"]
    };
    for &species in species {
        let mut pokemon = r
            .data()
            .create_pokemon(
                species,
                if injured { 80 } else { 48 },
                Dv::from_non_hp(8, 9, 8, 9),
            )
            .expect("active original Minnesota fixture partner");
        if injured {
            assert!(pokemon.max_hp > 210);
            pokemon.hp = pokemon.max_hp - 210;
        }
        assert!(state.storage.party.add_pokemon(pokemon));
    }
    state.sync_party_from_storage();
    state.repel_steps_remaining = 250;
    state.active_repel_item = Some("MAX_REPEL".into());
    state.badges.johto = [true, true, true, true, true, true, true, false];
    for flag in &crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[..7] {
        state
            .flags
            .set_engine_flag(flag, true)
            .expect("declared prior badges");
    }
    state
        .flags
        .set_event_flag("EVENT_MN_DEPOT_CLEARED", true)
        .expect("declared prior Iron Range Descent access");
    assert!(
        state
            .bag
            .add_item(&r.data().items["SUPER_POTION"], 6)
            .expect("initial course supplies")
    );
    assert!(
        state
            .bag
            .add_item(&r.data().items["MAX_REPEL"], 10)
            .expect("initial legal Repel supplies")
    );
    mark_runtime_snapshot_dirty(&mut c.shell);
    c
}

fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    path: &Path,
) -> VisibleShellController {
    assert!(!path.exists(), "preserve all reached saves");
    c.save(path).expect("actual reached save");
    let mut next =
        VisibleShellController::load_save(root.clone(), r.clone(), path.to_path_buf(), None)
            .expect("same-pack native Continue");
    next.set_runtime_journal_enabled(false);
    next
}

fn quantity(c: &VisibleShellController, r: &CrystalRuntime, item: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items[item])
}

fn pickup(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    approach: (i16, i16),
    facing: GameButton,
    item: &str,
    flag: &str,
) {
    assert!(!event(c, flag));
    let before = quantity(c, r, item);
    walk(c, r, approach);
    talk(c, facing);
    let start = c.snapshot().expect("pickup dialogue").overworld.tile;
    press(c, GameButton::Down);
    assert_eq!(
        c.snapshot().expect("dialogue owns movement").overworld.tile,
        start
    );
    finish(c);
    assert!(event(c, flag));
    assert_eq!(quantity(c, r, item), before + 1);
}

const ITEM: &str = "HYPER_POTION";
const FLAG: &str = "EVENT_MN_R20_ITEM_01";
fn cursor(
    controller: &mut VisibleShellController,
    target: usize,
    index: impl Fn(&VisibleShellController) -> usize,
) {
    for _ in 0..32 {
        let selected = index(controller);
        if selected == target {
            return;
        }
        press(
            controller,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    panic!("native cursor failed to reach {target}");
}

fn use_hyper_potion(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    settle(controller, "before native Hyper Potion Pack input");
    let before = controller.snapshot().expect("injured party").party;
    let mut expected = before.clone();
    let amount =
        u16::try_from(runtime.data().items[ITEM].parameter).expect("native healing amount");
    assert_eq!(amount, 200, "actual original item potency");
    let partner = &mut expected.slots[0].pokemon;
    partner.hp = partner.hp.saturating_add(amount).min(partner.max_hp);
    assert_ne!(before, expected);
    press(controller, GameButton::Start);
    let snapshot = controller.snapshot().expect("native Start");
    let target = super::super::visible_start_menu_options(&controller.shell, &snapshot)
        .iter()
        .position(|option| *option == super::super::StartMenuOption::Pack)
        .expect("PACK option");
    cursor(controller, target, |c| {
        c.shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index
    });
    press(controller, GameButton::A);
    let target = controller
        .snapshot()
        .expect("carried potion")
        .bag
        .items
        .iter()
        .filter(|item| item.quantity > 0)
        .position(|item| item.item_id == ITEM)
        .expect("actual collected Hyper Potion");
    cursor(controller, target, |c| {
        c.shell
            .bag_cursor
            .as_ref()
            .expect("Pack cursor")
            .option_index
    });
    press(controller, GameButton::A);
    let snapshot = controller.snapshot().expect("item actions");
    let actions = super::super::visible_selected_pack_item_actions(
        &snapshot,
        &controller.shell,
        &super::super::FieldPackPocket::Items,
        false,
    )
    .expect("native potion actions");
    let target = actions
        .iter()
        .position(|action| *action == super::super::FieldPackAction::Use)
        .expect("USE option");
    cursor(controller, target, |c| {
        c.shell
            .field_pack_action_cursor
            .as_ref()
            .expect("action cursor")
            .option_index
    });
    press(controller, GameButton::A);
    assert_eq!(
        controller.shell.field_pack_target_mode,
        Some(super::super::FieldPackTargetMode::PartyPokemon)
    );
    cursor(controller, 0, |c| c.shell.party_cursor);
    press(controller, GameButton::A);
    assert_eq!(
        controller.snapshot().expect("native item effect").party,
        expected,
        "only HP changes through the authoritative item engine"
    );
    assert_eq!(
        quantity(controller, runtime, ITEM),
        0,
        "collected potion consumed once"
    );
    let before_position = tile(controller);
    press(controller, GameButton::Down);
    assert_eq!(
        tile(controller),
        before_position,
        "medicine notice owns movement"
    );
    for _ in 0..64 {
        let snapshot = controller.snapshot().expect("close medicine menu");
        if controller.shell.field_notice.is_some() || snapshot.ui.pending_text_wait.is_some() {
            controller.wait_frames(60).expect("medicine notice frames");
            press(controller, GameButton::A);
        } else if snapshot.ui.menu.is_some()
            || controller.shell.field_pack_target_mode.is_some()
            || controller.shell.field_pack_action_cursor.is_some()
            || controller.shell.bag_cursor.is_some()
            || controller.shell.field_pack_pocket.is_some()
            || controller.shell.start_menu_cursor.is_some()
        {
            press(controller, GameButton::B);
        } else {
            settle(controller, "after native medicine menus close");
            return;
        }
    }
    panic!("native Hyper Potion menus did not release input");
}

#[test]
#[ignore = "requires rebuilt cleft in CRYSTAL_INTERIOR_PACK and fresh CRYSTAL_R20_WINDING_REVIEW_DIR"]
fn r20_winding_two_mouths_stairs_drop_supply_native_healing_and_continue() {
    let (root, r, _) = load();
    let out = output("route-resource");
    let mut c = setup(&root, &r, (5, 20), true);
    // Initial injured partner, prior badges and supplies end here. All later
    // traversal, transactions, health changes and saves are production input.
    travel(&mut c, GameButton::Down, "MnR20C00R00", (60, 34));
    travel(&mut c, GameButton::Up, ROOM, (5, 21));
    step(&mut c, GameButton::Up, (5, 20));
    walk(&mut c, &r, (4, 10));
    for point in [(4, 9), (4, 8), (4, 7)] {
        step(&mut c, GameButton::Up, point);
    }
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    walk(&mut c, &r, (11, 7));
    talk(&mut c, GameButton::Right);
    assert!(c.snapshot().expect("original scout").ui.text_window_open);
    press(&mut c, GameButton::Down);
    at(&mut c, ROOM, (11, 7));
    finish(&mut c);
    walk(&mut c, &r, (4, 7));
    for point in [(4, 8), (4, 9), (4, 10)] {
        step(&mut c, GameButton::Down, point);
    }
    walk(&mut c, &r, (16, 10));
    for point in [(16, 9), (16, 8), (16, 7)] {
        step(&mut c, GameButton::Up, point);
    }
    for point in [(16, 8), (16, 9), (16, 10)] {
        step(&mut c, GameButton::Down, point);
    }
    for y in [12, 13] {
        walk(&mut c, &r, (14, y));
        for _ in 0..8 {
            press(&mut c, GameButton::Left);
            if tile(&mut c) == (11, y) {
                break;
            }
        }
        at(&mut c, ROOM, (11, y));
        for _ in 0..3 {
            press(&mut c, GameButton::Right);
            at(&mut c, ROOM, (11, y));
        }
        walk(&mut c, &r, (14, y));
    }
    pickup(&mut c, &r, (4, 5), GameButton::Right, ITEM, FLAG);
    c = continued(
        &mut c,
        &root,
        &r,
        &out.join("original-potion-reached.crystalsave"),
    );
    assert!(event(&c, FLAG));
    assert_eq!(quantity(&c, &r, ITEM), 1);
    walk(&mut c, &r, (5, 5));
    walk(&mut c, &r, (4, 5));
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert_eq!(quantity(&c, &r, ITEM), 1);
    use_hyper_potion(&mut c, &r);
    let healed = c.snapshot().expect("native healed partner").party;
    assert_eq!(
        healed.slots[0].pokemon.hp,
        healed.slots[0].pokemon.max_hp - 10
    );
    c = continued(&mut c, &root, &r, &out.join("potion-used.crystalsave"));
    assert_eq!(c.snapshot().expect("reached healed Continue").party, healed);
    assert_eq!(quantity(&c, &r, ITEM), 0);
    walk(&mut c, &r, (23, 20));
    travel(&mut c, GameButton::Down, "MnR20C01R00", (12, 32));
    travel(&mut c, GameButton::Up, ROOM, (23, 21));
    step(&mut c, GameButton::Up, (23, 20));
    walk(&mut c, &r, (5, 20));
    travel(&mut c, GameButton::Down, "MnR20C00R00", (60, 34));
    travel(&mut c, GameButton::Up, ROOM, (5, 21));
    step(&mut c, GameButton::Up, (5, 20));
    c.press(GameButton::Start)
        .expect("Start after both real mouths");
    assert!(c.snapshot().expect("Start").ui.menu.is_some());
}

#[test]
#[ignore = "requires rebuilt cleft in CRYSTAL_INTERIOR_PACK and fresh CRYSTAL_R20_WINDING_REVIEW_DIR"]
fn r20_winding_optional_wayfinder_decline_pp_victory_and_continue() {
    let (root, r, _) = load();
    let out = output("wayfinder");
    let mut c = setup(&root, &r, (5, 20), false);
    let id = "MN_EXTRA_R20_WAYFINDER";
    let flag = "EVENT_MN_EXTRA_R20_WAYFINDER";
    walk(&mut c, &r, (21, 18));
    talk(&mut c, GameButton::Up);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, flag));
    talk(&mut c, GameButton::Up);
    choice(&mut c, true);
    assert!(super::rochester_earned::try_battle(&mut c, &r, id, flag));
    finish(&mut c);
    assert!(event(&c, flag));
    c = continued(&mut c, &root, &r, &out.join("wayfinder-won.crystalsave"));
    assert!(event(&c, flag));
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert!(c.snapshot().expect("won opponent").battle.is_none());
    walk(&mut c, &r, (23, 20));
}
