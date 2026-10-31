//! Explicit original-partner fixtures; subsequent slides and rewards use production input.
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crystal_assets::{AssetRoot, MapModule, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::world::collision::{
    is_direction_blocked, is_direction_blocked_leaving, permissions,
};
use crystal_core::world::map::Direction;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, neighbors};
use super::r10_gallery::{map, press, settle, tile};

const UPPER: &str = "MnVoyageursIce1F";
const LOWER: &str = "MnVoyageursIceB1F";

fn kind<'a>(runtime: &'a CrystalRuntime, m: &MapModule, at: Point) -> &'a str {
    if at.0 < 0
        || at.1 < 0
        || at.0 >= m.attributes.width as i16 * 2
        || at.1 >= m.attributes.height as i16 * 2
    {
        return "WALL";
    }
    let (x, y) = (at.0 as usize, at.1 as usize);
    let block = m.blocks[y / 2 * usize::from(m.attributes.width) + x / 2];
    &runtime.data().tilesets[&m.attributes.tileset_name].collision[&format!("{block:02x}")]
        [y % 2 * 2 + x % 2]
}

fn route(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    to: Point,
) -> Vec<(Point, GameButton)> {
    let m = &runtime.data().maps[&map(c)];
    let state = c.shell.shell.session().state();
    let bodies = m
        .objects
        .iter()
        .filter(|o| {
            o.event_flag == "-1"
                || !state
                    .flags
                    .is_event_flag_set(&o.event_flag)
                    .expect("registered reward flag")
        })
        .map(|o| (o.x as i16, o.y as i16))
        .collect::<BTreeSet<_>>();
    let mut parents = BTreeMap::from([(tile(c), None)]);
    let mut queue = VecDeque::from([tile(c)]);
    while let Some(at) = queue.pop_front() {
        if at == to {
            break;
        }
        for (next, button) in neighbors(at) {
            let d = (next.0 - at.0, next.1 - at.1);
            let facing = match button {
                GameButton::Left => Direction::Left,
                GameButton::Right => Direction::Right,
                GameButton::Up => Direction::Up,
                GameButton::Down => Direction::Down,
                _ => unreachable!("neighbor planner yields only directional inputs"),
            };
            let mut stop = at;
            let mut next = next;
            loop {
                let k = kind(runtime, m, next);
                if (kind(runtime, m, stop) == "UP_WALL"
                    && is_direction_blocked_leaving(permissions::UP_WALL, facing))
                    || (k == "UP_WALL" && is_direction_blocked(permissions::UP_WALL, facing))
                {
                    break;
                }
                if bodies.contains(&next)
                    || !(matches!(k, "FLOOR" | "ICE" | "ICE_2B" | "UP_WALL")
                        || (next == to && k == "LADDER"))
                {
                    break;
                }
                stop = next;
                if !matches!(k, "ICE" | "ICE_2B") {
                    break;
                }
                next = (next.0 + d.0, next.1 + d.1);
            }
            if let Entry::Vacant(entry) = parents.entry(stop) {
                entry.insert(Some((at, button)));
                queue.push_back(stop);
            }
        }
    }
    assert!(
        parents.contains_key(&to),
        "{}: no slide course {:?} -> {to:?}",
        m.id,
        tile(c)
    );
    let mut result = Vec::new();
    let mut at = to;
    while let Some((before, button)) = parents[&at] {
        result.push((at, button));
        at = before;
    }
    result.reverse();
    result
}

fn travel(c: &mut VisibleShellController, runtime: &CrystalRuntime, to: Point, destination: &str) {
    if c.shell.shell.session().state().repel_steps_remaining == 0
        && quantity(c, runtime, "MAX_REPEL") > 0
    {
        super::minneapolis_falls::grand_marais_harbor::use_max_repel(c, runtime);
    }
    let course = route(c, runtime, to);
    for (expected, button) in course {
        for _ in 0..4 {
            press(c, button);
            c.wait_frames(80).expect("native slide clock");
            if tile(c) == expected || map(c) == destination {
                break;
            }
        }
        if map(c) == destination && !destination.is_empty() {
            return;
        }
        assert_eq!(tile(c), expected, "ice stops at native rock/dry ground");
        assert!(c.snapshot().expect("ice course").battle.is_none());
    }
    if !destination.is_empty() {
        assert_eq!(map(c), destination);
    }
}

fn collect(c: &mut VisibleShellController, runtime: &CrystalRuntime, id: &str, item_id: &str) {
    let m = &runtime.data().maps[&map(c)];
    let object = m
        .objects
        .iter()
        .find(|o| o.script == id)
        .expect("authored reward");
    let at = (object.x as i16, object.y as i16);
    let (approach, button) = neighbors(at)
        .into_iter()
        .filter(|(p, _)| kind(runtime, m, *p) == "FLOOR")
        .map(|(p, _)| {
            let toward = neighbors(p)
                .into_iter()
                .find(|(q, _)| *q == at)
                .expect("adjacent")
                .1;
            (p, toward)
        })
        .min_by_key(|(p, _)| route(c, runtime, *p).len())
        .expect("dry reward approach");
    travel(c, runtime, approach, "");
    let item = &runtime.data().items[item_id];
    let before = c.shell.shell.session().state().bag.quantity(item);
    press(c, button);
    press(c, GameButton::A);
    settle(c, "ice reward notice");
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set(&object.event_flag)
            .expect("reward flag")
    );
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        before + 1
    );
    press(c, GameButton::A);
    settle(c, "collected reward");
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        before + 1
    );
}

fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path =
                PathBuf::from(std::env::var_os("CRYSTAL_ICE_TURNS_PACK").expect("ice child pack"))
                    .canonicalize()
                    .expect("pack path");
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("verified child"),
            )
            .expect("runtime");
            (root, runtime)
        })
        .clone()
}

fn setup(name: &str, at: Point) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        r.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: r.title_new_game_spawn_identifier().expect("HOME retained"),
            map_name: name.into(),
            tile_x: at.0,
            tile_y: at.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("local fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture name");
    let state = c.shell.shell.session_mut().state_mut();
    for species in ["LUTEBRISK", "CINDORE"] {
        assert!(
            state.storage.party.add_pokemon(
                r.data()
                    .create_pokemon(species, 48, Dv::from_non_hp(8, 9, 8, 9))
                    .expect("original Minnesota partner")
            )
        );
    }
    state.sync_party_from_storage();
    state.badges.johto = [true; 8];
    for flag in crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS {
        state
            .flags
            .set_engine_flag(flag, true)
            .expect("declared prior badges");
    }
    state
        .flags
        .set_event_flag("EVENT_MN_LIGHTHOUSE_SIGNAL", true)
        .expect("declared prior route access");
    state.repel_steps_remaining = 250;
    state.active_repel_item = Some("MAX_REPEL".into());
    assert!(
        state
            .bag
            .add_item(&r.data().items["MAX_REPEL"], 10)
            .expect("initial supplies")
    );
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    (root, r, c)
}

fn out(name: &str) -> PathBuf {
    let root = PathBuf::from(
        std::env::var_os("CRYSTAL_ICE_TURNS_REVIEW_DIR").expect("fresh review output"),
    );
    let p = root.join(name);
    assert!(!p.exists(), "preserve prior evidence");
    std::fs::create_dir_all(&p).expect("review folder");
    p
}
fn continued(
    root: &AssetRoot,
    r: &CrystalRuntime,
    c: &mut VisibleShellController,
    path: &Path,
) -> VisibleShellController {
    assert!(!path.exists());
    c.save(path).expect("actual reached save");
    let mut next =
        VisibleShellController::load_save(root.clone(), r.clone(), path.to_path_buf(), None)
            .expect("native Continue");
    next.set_runtime_journal_enabled(false);
    next
}
fn flag(c: &VisibleShellController, id: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(id)
        .expect("registered event")
}
fn quantity(c: &VisibleShellController, r: &CrystalRuntime, id: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items[id])
}

#[test]
#[ignore = "requires CRYSTAL_ICE_TURNS_PACK and fresh CRYSTAL_ICE_TURNS_REVIEW_DIR"]
fn ice_turns_controller_both_floors_forced_direction_all_rewards_and_real_return() {
    let out = out("full-journey");
    let (root, r, mut c) = setup("MnR20C01R00", (6, 34));
    c.save(out.join("initial-outside.crystalsave"))
        .expect("declared initial fixture");
    for _ in 0..4 {
        press(&mut c, GameButton::Up);
        if map(&mut c) == UPPER {
            break;
        }
    }
    assert_eq!(map(&mut c), UPPER);
    assert_eq!(tile(&mut c), (5, 19));
    travel(&mut c, &r, (4, 16), "");
    press(&mut c, GameButton::Left);
    press(&mut c, GameButton::A);
    assert!(c.snapshot().expect("retained guide").ui.text_window_open);
    press(&mut c, GameButton::Down);
    assert_eq!(tile(&mut c), (4, 16));
    settle(&mut c, "guide releases movement");
    collect(&mut c, &r, "MnVoyageursIceHeal", "ICE_HEAL");
    travel(&mut c, &r, (23, 5), LOWER);
    assert_eq!(tile(&mut c), (5, 19));
    travel(&mut c, &r, (4, 20), "");
    press(&mut c, GameButton::Left);
    press(&mut c, GameButton::A);
    assert!(c.snapshot().expect("retained surveyor").ui.text_window_open);
    press(&mut c, GameButton::Up);
    assert_eq!(tile(&mut c), (4, 20));
    settle(&mut c, "surveyor releases movement");
    travel(&mut c, &r, (5, 18), "");
    c.press(GameButton::Up).expect("start forced north slide");
    assert_eq!(tile(&mut c).0, 5);
    let y = tile(&mut c).1;
    c.press(GameButton::Right)
        .expect("perpendicular input while sliding");
    assert_eq!(tile(&mut c).0, 5);
    assert!(tile(&mut c).1 < y);
    c.wait_frames(80).expect("settle at entrance tooth");
    assert_eq!(tile(&mut c), (5, 11));
    collect(&mut c, &r, "MnVoyageursIceRevive", "MAX_REVIVE");
    collect(&mut c, &r, "MnVoyageursIceNeverMelt", "NEVERMELTICE");
    travel(&mut c, &r, (13, 15), "");
    assert_eq!(kind(&r, &r.data().maps[LOWER], (13, 15)), "ICE");
    c = continued(
        &root,
        &r,
        &mut c,
        &out.join("collected-ice-stop.crystalsave"),
    );
    c.wait_frames(80).expect("continued stopped ice");
    assert_eq!(tile(&mut c), (13, 15));
    for (event, item) in [
        ("EVENT_MN_VOYAGEURS_ICE_HEAL", "ICE_HEAL"),
        ("EVENT_MN_VOYAGEURS_ICE_NEVERMELT", "NEVERMELTICE"),
        ("EVENT_MN_VOYAGEURS_ICE_REVIVE", "MAX_REVIVE"),
    ] {
        assert!(flag(&c, event));
        assert_eq!(quantity(&c, &r, item), 1);
    }

    assert_eq!(quantity(&c, &r, "NEVERMELTICE"), 1);
    assert_eq!(quantity(&c, &r, "MAX_REVIVE"), 1);
    travel(&mut c, &r, (5, 21), UPPER);
    assert_eq!(tile(&mut c), (23, 6));
    travel(&mut c, &r, (5, 21), "MnR20C01R00");
    assert_eq!(tile(&mut c), (6, 34));
    c = continued(
        &root,
        &r,
        &mut c,
        &out.join("same-mouth-outside.crystalsave"),
    );
    for _ in 0..4 {
        press(&mut c, GameButton::Up);
        if map(&mut c) == UPPER {
            break;
        }
    }
    assert_eq!(tile(&mut c), (5, 19));
    assert!(flag(&c, "EVENT_MN_VOYAGEURS_ICE_NEVERMELT"));
    assert_eq!(quantity(&c, &r, "NEVERMELTICE"), 1);
}

#[test]
#[ignore = "requires CRYSTAL_ICE_TURNS_PACK and fresh CRYSTAL_ICE_TURNS_REVIEW_DIR"]
fn ice_turns_controller_full_pocket_preserves_body_toss_retry_and_partial_pickup_return() {
    let out = out("full-pocket");
    let (root, r, mut c) = setup(LOWER, (19, 4));
    let bag = &mut c.shell.shell.session_mut().state_mut().bag;
    bag.items.clear();
    assert!(
        bag.add_item(&r.data().items["POTION"], 1)
            .expect("initial tossable supply")
    );
    while bag.items.len() < crystal_core::models::bag::ITEM_POCKET_CAPACITY {
        assert!(
            bag.add_item(&r.data().items["BERRY"], 99)
                .expect("physical full pocket")
        );
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    // The complete local fixture ends here. No later inventory/flag/position writes.
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    settle(&mut c, "full-pocket denial");
    assert!(!flag(&c, "EVENT_MN_VOYAGEURS_ICE_NEVERMELT"));
    assert_eq!(quantity(&c, &r, "NEVERMELTICE"), 0);
    press(&mut c, GameButton::Up);
    assert_eq!(tile(&mut c), (19, 4));
    c = continued(&root, &r, &mut c, &out.join("denied.crystalsave"));
    super::detroit_lakes::toss_potion(&mut c);
    press(&mut c, GameButton::A);
    settle(&mut c, "retry pickup");
    assert!(flag(&c, "EVENT_MN_VOYAGEURS_ICE_NEVERMELT"));
    assert_eq!(quantity(&c, &r, "NEVERMELTICE"), 1);
    c = continued(
        &root,
        &r,
        &mut c,
        &out.join("one-pickup-cleared.crystalsave"),
    );
    travel(&mut c, &r, (19, 3), "");
    travel(&mut c, &r, (5, 19), "");
    assert!(!flag(&c, "EVENT_MN_VOYAGEURS_ICE_REVIVE"));
    travel(&mut c, &r, (5, 21), UPPER);
    assert_eq!(tile(&mut c), (23, 6));
    travel(&mut c, &r, (5, 21), "MnR20C01R00");
    assert_eq!(tile(&mut c), (6, 34));
    c.press(GameButton::Start).expect("Start after return");
    assert!(c.snapshot().expect("Start").ui.menu.is_some());
}
