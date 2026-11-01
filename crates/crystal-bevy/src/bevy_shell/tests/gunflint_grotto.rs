//! Explicit local setup; all grotto traversal, discovery and care use input.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::LearnedMove;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::models::item::ITEM_POCKET_ITEM;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::super::northstar_ice_lair::{talk, walk};
use super::super::pepin_discovery::paddle;
use super::super::winona::{at, press, travel};
use super::{event, finish, itemfinder};

#[path = "gunflint_grotto_pack.rs"]
mod pack;

const ROOM: &str = "MnR12Cave1F";
const ETHER: &str = "EVENT_MN_R12_CAVE_ETHER";
const REVIVE: &str = "EVENT_MN_R12_GROTTO_REVIVE";

fn setup(
    map: &str,
    point: (i16, i16),
    surf: bool,
    badge: bool,
    full: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = super::load();
    let mut c = super::fixture(&root, &r, map, point);
    let state = c.shell.shell.session_mut().state_mut();
    for species in ["VOYAGEAR", "CINDORE"] {
        let mut mon = r
            .data()
            .create_pokemon(species, 60, Default::default())
            .expect("active Minnesota partner");
        mon.moves.retain(|m| m.name != "SURF");
        if species == "VOYAGEAR" {
            assert!(mon.species.tmhm_learnset.iter().any(|m| m == "SURF"));
            mon.moves.truncate(3);
            for m in &mut mon.moves {
                m.current_pp = 0;
            }
            if surf {
                mon.moves.push(LearnedMove {
                    name: "SURF".into(),
                    current_pp: r.data().moves["SURF"].pp,
                    pp_ups: 0,
                });
            }
            mon.item = Some("SMOKE_BALL".into());
        } else {
            mon.hp = 0;
        }
        assert!(state.storage.party.add_pokemon(mon));
    }
    state.sync_party_from_storage();
    state.badges.johto[3] = badge;
    state
        .flags
        .set_engine_flag(BADGE_ENGINE_FLAGS[3], badge)
        .expect("declared prior Surf badge");
    assert!(
        state
            .bag
            .add_item(&r.data().items["ITEMFINDER"], 1)
            .expect("detector")
    );
    if full {
        for (_, item) in r
            .data()
            .items
            .iter()
            .filter(|(id, item)| {
                item.pocket == ITEM_POCKET_ITEM && !["REVIVE", "MAX_ETHER"].contains(&id.as_str())
            })
            .take(ITEM_POCKET_CAPACITY)
        {
            assert!(state.bag.add_item(item, 1).expect("declared capacity"));
        }
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    assert!(!event(&c, ETHER) && !event(&c, REVIVE));
    assert_eq!(
        [
            r.data().maps[ROOM].attributes.width,
            r.data().maps[ROOM].attributes.height
        ],
        [20, 14]
    );
    // Setup ends here: no subsequent state, position, flag, party or bag writes.
    (root, r, c)
}

fn checkpoint(c: &mut VisibleShellController, name: &str) -> PathBuf {
    let out = PathBuf::from(
        std::env::var_os("CRYSTAL_GUNFLINT_GROTTO_REVIEW_DIR").expect("fresh grotto output"),
    );
    std::fs::create_dir_all(&out).expect("grotto output");
    let path = out.join(format!("{name}.crystalsave"));
    assert!(!path.exists(), "retain every reached grotto state");
    c.save(&path).expect("actual production checkpoint");
    path
}

fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    name: &str,
) -> VisibleShellController {
    assert!(!c.has_pending_script_work());
    let mut next =
        VisibleShellController::load_save(root.clone(), r.clone(), checkpoint(c, name), None)
            .expect("production Continue");
    next.set_runtime_journal_enabled(false);
    press(&mut next, GameButton::A);
    finish(&mut next);
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

fn course(c: &mut VisibleShellController, r: &CrystalRuntime, name: &str, reverse: bool) {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/caves/gunflint-grotto-v01/plan.json");
    let plan: Value =
        serde_json::from_str(&std::fs::read_to_string(source).expect("course contract"))
            .expect("contract JSON");
    let mut path = plan["routes"][name]["path"]
        .as_array()
        .expect("explicit authored course")
        .iter()
        .map(|p| {
            (
                p[0].as_i64().expect("x") as i16,
                p[1].as_i64().expect("y") as i16,
            )
        })
        .collect::<Vec<_>>();
    if reverse {
        path.reverse();
    }
    walk(c, r, path[0]);
    for pair in path.windows(2) {
        let button = match (pair[1].0 - pair[0].0, pair[1].1 - pair[0].1) {
            (1, 0) => GameButton::Right,
            (-1, 0) => GameButton::Left,
            (0, 1) => GameButton::Down,
            (0, -1) => GameButton::Up,
            delta => panic!("nonadjacent authored course {delta:?}"),
        };
        travel(c, button, ROOM, pair[1]);
        assert_eq!(
            c.snapshot().expect("dry course").overworld.mode,
            MovementMode::Normal
        );
    }
}

fn mount(c: &mut VisibleShellController) {
    super::super::minneapolis_falls::mount(c, GameButton::Right, ROOM, (22, 19));
    assert_eq!(
        c.snapshot().expect("real spring launch").overworld.mode,
        MovementMode::Surf
    );
}

fn search(c: &mut VisibleShellController) {
    talk(c, GameButton::Right);
    finish(c);
    at(c, ROOM, (27, 16));
}

#[test]
#[ignore = "requires written grotto pack, CRYSTAL_WINONA_TEST_PACK and fresh CRYSTAL_GUNFLINT_GROTTO_REVIEW_DIR"]
fn gunflint_grotto_controller_dry_circuits_stairs_cache_native_ether_and_real_return() {
    let (root, r, mut c) = setup("MnR12C00R00", (40, 16), false, false, false);
    checkpoint(&mut c, "dry-initial-exterior");
    travel(&mut c, GameButton::Up, ROOM, (17, 23));
    travel(&mut c, GameButton::Up, ROOM, (17, 22));
    for (name, reverse) in [
        ("west-fin", false),
        ("east-fin", true),
        ("north-spring", false),
        ("south-spring", true),
    ] {
        course(&mut c, &r, name, reverse);
        checkpoint(&mut c, name);
    }
    walk(&mut c, &r, (6, 10));
    for y in (5..=9).rev() {
        travel(&mut c, GameButton::Up, ROOM, (6, y));
    }
    for y in 6..=10 {
        travel(&mut c, GameButton::Down, ROOM, (6, y));
    }
    walk(&mut c, &r, (34, 10));
    travel(&mut c, GameButton::Up, ROOM, (34, 9));
    press(&mut c, GameButton::Right);
    at(&mut c, ROOM, (34, 9));
    travel(&mut c, GameButton::Up, ROOM, (34, 8));
    travel(&mut c, GameButton::Down, ROOM, (34, 9));
    travel(&mut c, GameButton::Down, ROOM, (34, 10));
    walk(&mut c, &r, (34, 3));
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert!(event(&c, ETHER));
    assert_eq!(quantity(&c, &r, "MAX_ETHER"), 1);
    assert!(
        !c.snapshot()
            .expect("claimed cache")
            .visible_objects
            .iter()
            .any(|o| o.script == "MN_R12_CAVE_CACHE")
    );
    c = continued(&mut c, &root, &r, "dry-cache-claimed");
    assert!(event(&c, ETHER));
    assert_eq!(quantity(&c, &r, "MAX_ETHER"), 1);
    pack::restore_move(&mut c, &r);
    assert_eq!(quantity(&c, &r, "MAX_ETHER"), 0);
    assert!(!event(&c, REVIVE));
    walk(&mut c, &r, (17, 22));
    for _ in 0..16 {
        if c.snapshot().expect("mouth return").overworld.map_name != ROOM {
            break;
        }
        press(&mut c, GameButton::Down);
    }
    at(&mut c, "MnR12C00R00", (40, 16));
    travel(&mut c, GameButton::Up, ROOM, (17, 23));
    travel(&mut c, GameButton::Up, ROOM, (17, 22));
    c = continued(&mut c, &root, &r, "dry-exterior-return-and-reentry");
    assert!(event(&c, ETHER) && !event(&c, REVIVE));
    assert_eq!(
        c.snapshot()
            .expect("no Surf throughout dry trip")
            .overworld
            .mode,
        MovementMode::Normal
    );
}

#[test]
#[ignore = "requires current grotto pack and fresh review output; initial Surf/badge denial fixtures"]
fn gunflint_grotto_controller_spring_denies_missing_move_or_badge() {
    for (surf, badge, name) in [(false, true, "denied-move"), (true, false, "denied-badge")] {
        let (_, _, mut c) = setup(ROOM, (21, 19), surf, badge, false);
        checkpoint(&mut c, &format!("{name}-initial"));
        talk(&mut c, GameButton::Right);
        finish(&mut c);
        at(&mut c, ROOM, (21, 19));
        assert_eq!(
            c.snapshot().expect("denied spring access").overworld.mode,
            MovementMode::Normal
        );
        assert!(!event(&c, REVIVE));
        checkpoint(&mut c, name);
    }
}

#[test]
#[ignore = "requires current grotto pack and fresh review output; declared water partner and fainted companion"]
fn gunflint_grotto_controller_surfs_searches_pillar_uses_revive_and_continues() {
    let (root, r, mut c) = setup(ROOM, (21, 19), true, true, false);
    checkpoint(&mut c, "water-initial-bank");
    mount(&mut c);
    paddle(&mut c, &r, (27, 16));
    itemfinder(&mut c, true);
    checkpoint(&mut c, "water-pillar-before");
    search(&mut c);
    assert!(event(&c, REVIVE));
    assert_eq!(quantity(&c, &r, "REVIVE"), 1);
    c = continued(&mut c, &root, &r, "water-pillar-claimed");
    assert_eq!(
        c.snapshot().expect("continued spring").overworld.mode,
        MovementMode::Surf
    );
    assert!(event(&c, REVIVE));
    search(&mut c);
    assert_eq!(quantity(&c, &r, "REVIVE"), 1);
    itemfinder(&mut c, false);
    pack::revive(&mut c, &r);
    assert_eq!(quantity(&c, &r, "REVIVE"), 0);
    paddle(&mut c, &r, (22, 19));
    travel(&mut c, GameButton::Left, ROOM, (21, 19));
    assert_eq!(
        c.snapshot().expect("dry return").overworld.mode,
        MovementMode::Normal
    );
    c = continued(&mut c, &root, &r, "water-used-and-landed");
    assert!(event(&c, REVIVE));
    assert!(
        c.snapshot().expect("revived after Continue").party.slots[1]
            .pokemon
            .hp
            > 0
    );
}

#[test]
#[ignore = "requires current grotto pack and fresh review output; declared full Items pocket"]
fn gunflint_grotto_controller_both_caches_full_pocket_native_toss_retry_once() {
    let (root, r, mut c) = setup(ROOM, (34, 3), true, true, true);
    checkpoint(&mut c, "full-initial-cache");
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert!(!event(&c, ETHER));
    assert_eq!(quantity(&c, &r, "MAX_ETHER"), 0);
    c = continued(&mut c, &root, &r, "full-ether-denied");
    pack::toss(&mut c);
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert!(event(&c, ETHER));
    assert_eq!(quantity(&c, &r, "MAX_ETHER"), 1);
    walk(&mut c, &r, (21, 19));
    mount(&mut c);
    paddle(&mut c, &r, (27, 16));
    search(&mut c);
    assert!(!event(&c, REVIVE));
    assert_eq!(quantity(&c, &r, "REVIVE"), 0);
    c = continued(&mut c, &root, &r, "full-pillar-denied");
    pack::toss(&mut c);
    search(&mut c);
    assert!(event(&c, REVIVE));
    assert_eq!(quantity(&c, &r, "REVIVE"), 1);
    search(&mut c);
    assert_eq!(quantity(&c, &r, "REVIVE"), 1);
    checkpoint(&mut c, "full-native-retries-claimed");
}
