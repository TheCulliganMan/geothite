//! Local position/party fixtures; every later route, reward and battle uses input.
use std::path::{Path, PathBuf};

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life::{fixture, load};
use super::northstar_ice_lair::{choice, event, path, talk};
use super::r10_gallery::press;
use super::winona::{at, finish, travel};

const ROOM: &str = "MnR10Cave1F";

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
        std::env::var_os("CRYSTAL_R10_EXPLORATION_REVIEW_DIR")
            .expect("fresh ignored review directory"),
    )
    .join(name);
    std::fs::create_dir_all(&path).expect("review output");
    path
}

fn setup(root: &AssetRoot, r: &CrystalRuntime, point: (i16, i16)) -> VisibleShellController {
    assert_eq!(
        [
            r.data().maps[ROOM].attributes.width,
            r.data().maps[ROOM].attributes.height
        ],
        [24, 18]
    );
    let mut c = fixture(root, r, ROOM, point);
    let state = c.shell.shell.session_mut().state_mut();
    for species in ["VOYAGEAR", "FORTIGARD", "MUDMENDER", "HAULGEIST"] {
        assert!(
            state.storage.party.add_pokemon(
                r.data()
                    .create_pokemon(species, 48, Dv::from_non_hp(8, 9, 8, 9))
                    .expect("original Minnesota geometry/battle partner")
            )
        );
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

#[test]
#[ignore = "requires rebuilt R10 in CRYSTAL_INTERIOR_PACK and a fresh CRYSTAL_R10_EXPLORATION_REVIEW_DIR"]
fn r10_exploration_stairs_short_drops_resources_real_mouth_and_continue() {
    let (root, r, _) = load();
    let out = output("routes");
    let mut c = setup(&root, &r, (3, 12));
    // No position, flags, inventory, party or RNG writes after initial setup.
    walk(&mut c, &r, (8, 8));
    for point in [(8, 7), (8, 6), (8, 5)] {
        step(&mut c, GameButton::Up, point);
    }
    talk(&mut c, GameButton::Right);
    assert!(c.snapshot().expect("original reader").ui.text_window_open);
    let before = c.snapshot().expect("reader position").overworld.tile;
    press(&mut c, GameButton::Left);
    assert_eq!(
        c.snapshot().expect("reader input ownership").overworld.tile,
        before
    );
    finish(&mut c);
    for point in [(8, 6), (8, 7), (8, 8)] {
        step(&mut c, GameButton::Down, point);
    }
    walk(&mut c, &r, (20, 9));
    for point in [(20, 8), (20, 7), (20, 6)] {
        step(&mut c, GameButton::Up, point);
    }
    for point in [(20, 7), (20, 8), (20, 9)] {
        step(&mut c, GameButton::Down, point);
    }
    pickup(
        &mut c,
        &r,
        (38, 5),
        GameButton::Right,
        "ESCAPE_ROPE",
        "EVENT_MN_R10_ESCAPE_ROPE",
    );
    for x in [14, 15] {
        walk(&mut c, &r, (x, 15));
        for _ in 0..4 {
            press(&mut c, GameButton::Down);
            let at = c.snapshot().expect("native short drop").overworld.tile;
            if (at.x, at.y) == (x, 18) {
                break;
            }
        }
        at(&mut c, ROOM, (x, 18));
        for _ in 0..3 {
            press(&mut c, GameButton::Up);
            at(&mut c, ROOM, (x, 18));
        }
        walk(&mut c, &r, (x, 15));
    }
    pickup(
        &mut c,
        &r,
        (28, 23),
        GameButton::Right,
        "REVIVE",
        "EVENT_MN_R10_ITEM_04",
    );
    walk(&mut c, &r, (33, 25));
    c = continued(
        &mut c,
        &root,
        &r,
        &out.join("whole-cave-reached.crystalsave"),
    );
    assert_eq!(quantity(&c, &r, "ESCAPE_ROPE"), 1);
    assert_eq!(quantity(&c, &r, "REVIVE"), 1);
    walk(&mut c, &r, (39, 5));
    walk(&mut c, &r, (29, 23));
    walk(&mut c, &r, (3, 12));
    let warp = &r.data().maps[ROOM].events.warps[0];
    let outside = warp.target_map.clone();
    let return_warp = &r.data().maps[&outside].events.warps
        [usize::try_from(warp.target_warp_id - 1).expect("positive return warp id")];
    let expected = (return_warp.x as i16, return_warp.y as i16);
    travel(&mut c, GameButton::Down, &outside, expected);
    travel(&mut c, GameButton::Up, ROOM, (3, 13));
    step(&mut c, GameButton::Up, (3, 12));
    c.press(GameButton::Start)
        .expect("production Start after real return");
    assert!(c.snapshot().expect("Start").ui.menu.is_some());
}

#[test]
#[ignore = "requires rebuilt R10 in CRYSTAL_INTERIOR_PACK and a fresh CRYSTAL_R10_EXPLORATION_REVIEW_DIR"]
fn r10_exploration_optional_both_battles_decline_pp_victory_and_continue() {
    let (root, r, _) = load();
    let out = output("opponents");
    let mut c = setup(&root, &r, (3, 12));
    // Initial party/badges are fixtures. Native choices and battle turns follow.
    for (id, approach, facing) in [
        ("MN_EXTRA_R10_SURVEY", (19, 15), GameButton::Left),
        ("MN_EXTRA_R10_SPRING", (33, 25), GameButton::Up),
    ] {
        let flag = format!("EVENT_{id}");
        walk(&mut c, &r, approach);
        talk(&mut c, facing);
        choice(&mut c, false);
        finish(&mut c);
        assert!(!event(&c, &flag));
        talk(&mut c, facing);
        choice(&mut c, true);
        assert!(
            super::rochester_earned::try_battle(&mut c, &r, id, &flag),
            "actual PP-spending victory"
        );
        finish(&mut c);
        assert!(event(&c, &flag));
        c = continued(
            &mut c,
            &root,
            &r,
            &out.join(format!("{id}-won.crystalsave")),
        );
        assert!(event(&c, &flag));
        talk(&mut c, facing);
        finish(&mut c);
        assert!(
            c.snapshot()
                .expect("won opponent dialogue ends")
                .battle
                .is_none()
        );
    }
    walk(&mut c, &r, (3, 12));
}

#[test]
#[ignore = "requires rebuilt R10 in CRYSTAL_INTERIOR_PACK and a fresh CRYSTAL_R10_EXPLORATION_REVIEW_DIR"]
fn r10_exploration_supply_full_pocket_toss_retry_once_and_continue() {
    let (root, r, _) = load();
    let out = output("full-pocket");
    let mut c = setup(&root, &r, (38, 5));
    {
        let bag = &mut c.shell.shell.session_mut().state_mut().bag;
        bag.items.clear();
        assert!(
            bag.add_item(&r.data().items["POTION"], 1)
                .expect("initial tossable item")
        );
        while bag.items.len() < crystal_core::models::bag::ITEM_POCKET_CAPACITY {
            assert!(
                bag.add_item(&r.data().items["BERRY"], 99)
                    .expect("initial full pocket")
            );
        }
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    // The physical full pocket is initial setup, before this course's input.
    talk(&mut c, GameButton::Right);
    assert!(
        c.snapshot()
            .expect("native full-pocket text")
            .ui
            .text_window_open
    );
    press(&mut c, GameButton::Down);
    at(&mut c, ROOM, (38, 5));
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_R10_ESCAPE_ROPE"));
    assert_eq!(quantity(&c, &r, "ESCAPE_ROPE"), 0);
    press(&mut c, GameButton::Right);
    at(&mut c, ROOM, (38, 5));
    c = continued(
        &mut c,
        &root,
        &r,
        &out.join("full-pocket-denied.crystalsave"),
    );
    super::detroit_lakes::toss_potion(&mut c);
    c.press(GameButton::A).expect("native retry after Toss");
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_R10_ESCAPE_ROPE"));
    assert_eq!(quantity(&c, &r, "ESCAPE_ROPE"), 1);
    c = continued(&mut c, &root, &r, &out.join("supply-collected.crystalsave"));
    press(&mut c, GameButton::Right);
    at(&mut c, ROOM, (39, 5));
    press(&mut c, GameButton::Left);
    press(&mut c, GameButton::Right);
    c.press(GameButton::A)
        .expect("collected floor cannot award twice");
    finish(&mut c);
    assert_eq!(quantity(&c, &r, "ESCAPE_ROPE"), 1);
    c.press(GameButton::Start)
        .expect("Start after collected cache");
    assert!(c.snapshot().expect("Start").ui.menu.is_some());
}
