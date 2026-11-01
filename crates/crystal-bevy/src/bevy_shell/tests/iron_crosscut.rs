//! Declared initial setup; every crosscut movement, fight and item use is native.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::models::item::ITEM_POCKET_ITEM;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};

use super::super::{FieldPackPocket, VisibleShellController, mark_runtime_snapshot_dirty};
use super::northstar_ice_lair::{event, path, talk};
use super::winona::{at, finish, load, press, travel};

#[path = "iron_crosscut_pack.rs"]
mod pack;

const ROOM: &str = "MnIronDrillApproach";
const CACHE: &str = "EVENT_MN_IRON_APPROACH_ETHER";
const ACCESS: &str = "EVENT_MN_LODE_ARC_ACCESS_DEFEATED";
const SPOIL: &str = "EVENT_MN_LODE_ARC_SPOIL_DEFEATED";

fn setup(
    quiet: bool,
    full: bool,
    depleted: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    let mut c = super::interior_life::fixture(&root, &r, ROOM, (6, 21));
    let state = c.shell.shell.session_mut().state_mut();
    for species in ["VOYAGEAR", "FORTIGARD", "CINDORE"] {
        let mut mon = r
            .data()
            .create_pokemon(species, 60, Default::default())
            .expect("active regional partner");
        mon.item = Some("SMOKE_BALL".into());
        if depleted {
            for m in &mut mon.moves {
                m.current_pp = 0;
            }
        }
        assert!(state.storage.party.add_pokemon(mon));
    }
    state.sync_party_from_storage();
    state
        .flags
        .set_event_flag("EVENT_MN_DEPOT_FOREMAN_DEFEATED", true)
        .expect("declared existing public-shaft access");
    if quiet {
        for flag in [ACCESS, SPOIL] {
            state
                .flags
                .set_event_flag(flag, true)
                .expect("declared cleared-guard geometry fixture");
        }
    }
    if full {
        for (_, item) in r
            .data()
            .items
            .iter()
            .filter(|(id, item)| item.pocket == ITEM_POCKET_ITEM && id.as_str() != "ETHER")
            .take(ITEM_POCKET_CAPACITY)
        {
            assert!(state.bag.add_item(item, 1).expect("declared full pocket"));
        }
    } else {
        assert!(
            state
                .bag
                .add_item(&r.data().items["HYPER_POTION"], 15)
                .expect("declared carried supplies")
        );
    }
    state.money = 1000;
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    assert!(!event(&c, CACHE));
    // Setup ends here: no later position, party, flag, bag, HP, PP or RNG writes.
    (root, r, c)
}

fn checkpoint(c: &mut VisibleShellController, name: &str) -> PathBuf {
    let folder = PathBuf::from(
        std::env::var_os("CRYSTAL_IRON_CROSSCUT_REVIEW_DIR").expect("fresh crosscut output"),
    );
    std::fs::create_dir_all(&folder).expect("review directory");
    let p = folder.join(format!("{name}.crystalsave"));
    assert!(!p.exists(), "retain every reached state");
    c.save(&p).expect("native reached checkpoint");
    p
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

fn settle(c: &mut VisibleShellController) {
    let s = c.snapshot().expect("live crosscut state");
    if s.battle
        .as_ref()
        .is_some_and(|b| matches!(b.kind, RuntimeBattleKind::Wild { .. }))
    {
        super::rochester::flee(c);
    } else if s.battle.is_some() || s.ui.text_window_open || c.has_pending_script_work() {
        finish(c);
    }
}

fn go(c: &mut VisibleShellController, r: &CrystalRuntime, target: (i16, i16)) {
    for _ in 0..512 {
        settle(c);
        let s = c.snapshot().expect("crosscut travel");
        if (s.overworld.tile.x, s.overworld.tile.y) == target {
            return;
        }
        let button = *path(c, r, target)
            .expect("ordinary dry walking course")
            .first()
            .expect("remaining step");
        press(c, button);
    }
    panic!("crosscut did not reach {target:?}");
}

fn step(c: &mut VisibleShellController, button: GameButton, target: (i16, i16)) {
    for _ in 0..16 {
        settle(c);
        let s = c.snapshot().expect("crosscut step");
        if (s.overworld.tile.x, s.overworld.tile.y) == target {
            return;
        }
        press(c, button);
        assert_ne!(
            (
                c.snapshot()
                    .expect("never occupy jumped wall")
                    .overworld
                    .tile
                    .x,
                c.snapshot().expect("jump course").overworld.tile.y
            ),
            (12, target.1)
        );
    }
    at(c, ROOM, target);
}

fn quantity(c: &VisibleShellController, r: &CrystalRuntime) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items["ETHER"])
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK and fresh CRYSTAL_IRON_CROSSCUT_REVIEW_DIR"]
fn iron_crosscut_controller_stair_two_westward_lanes_reverse_denial_and_both_real_returns() {
    let (root, r, mut c) = setup(true, false, false);
    checkpoint(&mut c, "geometry-initial");
    for target in [
        (9, 13),
        (4, 5),
        (6, 10),
        (15, 10),
        (19, 13),
        (25, 18),
        (14, 18),
    ] {
        go(&mut c, &r, target);
    }
    for y in [18, 19] {
        go(&mut c, &r, (14, y));
        step(&mut c, GameButton::Left, (13, y));
        checkpoint(&mut c, &format!("lip-{y}"));
        step(&mut c, GameButton::Left, (11, y));
        press(&mut c, GameButton::Right);
        at(&mut c, ROOM, (11, y));
        c = continued(&mut c, &root, &r, &format!("landing-{y}"));
        press(&mut c, GameButton::Right);
        at(&mut c, ROOM, (11, y));
        go(&mut c, &r, (14, y));
    }
    go(&mut c, &r, (6, 10));
    for y in (7..=9).rev() {
        step(&mut c, GameButton::Up, (6, y));
    }
    for y in 8..=10 {
        step(&mut c, GameButton::Down, (6, y));
    }
    go(&mut c, &r, (24, 5));
    checkpoint(&mut c, "haulage-approach");
    travel(&mut c, GameButton::Right, "MnIronHaulage", (4, 23));
    checkpoint(&mut c, "actual-haulage-arrival");
    travel(&mut c, GameButton::Right, "MnIronHaulage", (5, 23));
    travel(&mut c, GameButton::Up, ROOM, (24, 5));
    go(&mut c, &r, (6, 21));
    travel(&mut c, GameButton::Right, "MnR09Cave1F", (19, 9));
    checkpoint(&mut c, "actual-public-mine-return");
    travel(&mut c, GameButton::Down, "MnR09Cave1F", (19, 10));
    travel(&mut c, GameButton::Down, ROOM, (6, 21));
    c = continued(&mut c, &root, &r, "paired-returns-and-reentry");
    at(&mut c, ROOM, (6, 21));
    assert!(!event(&c, CACHE));
}

#[test]
#[ignore = "requires the current crosscut pack and fresh output; declared depleted PP before input"]
fn iron_crosscut_controller_original_ether_native_use_once_and_continue() {
    let (root, r, mut c) = setup(true, false, true);
    checkpoint(&mut c, "ether-initial-depleted");
    go(&mut c, &r, (4, 5));
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert!(event(&c, CACHE));
    assert_eq!(quantity(&c, &r), 1);
    c = continued(&mut c, &root, &r, "ether-claimed");
    assert!(event(&c, CACHE));
    assert_eq!(quantity(&c, &r), 1);
    pack::restore_move(&mut c, &r);
    assert_eq!(quantity(&c, &r), 0);
    c = continued(&mut c, &root, &r, "ether-used");
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(event(&c, CACHE));
    assert_eq!(quantity(&c, &r), 0);
    assert!(
        !c.snapshot()
            .expect("persistent pickup visibility")
            .visible_objects
            .iter()
            .any(|o| o.script == "MN_IRON_APPROACH_ETHER")
    );
}

#[test]
#[ignore = "requires the current crosscut pack and fresh output; initial full Items pocket"]
fn iron_crosscut_controller_full_pocket_toss_retry_and_native_cache_persistence() {
    let (root, r, mut c) = setup(true, true, false);
    checkpoint(&mut c, "capacity-initial");
    go(&mut c, &r, (4, 5));
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert!(!event(&c, CACHE));
    assert_eq!(quantity(&c, &r), 0);
    c = continued(&mut c, &root, &r, "capacity-refused");
    super::northstar_progression_rewards::native_toss_first_pocket_item(
        &mut c,
        &FieldPackPocket::Items,
    );
    for _ in 0..8 {
        press(&mut c, GameButton::B);
    }
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert!(event(&c, CACHE));
    assert_eq!(quantity(&c, &r), 1);
    c = continued(&mut c, &root, &r, "capacity-retry-collected");
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(quantity(&c, &r), 1);
}

#[test]
#[ignore = "requires the current crosscut pack and fresh output; healthy local party and public-mine access only"]
fn iron_crosscut_controller_both_original_encounters_win_spend_pp_and_persist() {
    let (root, r, mut c) = setup(false, false, false);
    checkpoint(&mut c, "battle-initial-unbeaten");
    assert!(!event(&c, ACCESS) && !event(&c, SPOIL));
    let before = c.snapshot().expect("before encounters").party;
    go(&mut c, &r, (9, 13));
    step(&mut c, GameButton::Up, (9, 12));
    finish(&mut c);
    assert!(event(&c, ACCESS));
    checkpoint(&mut c, "access-victory");
    go(&mut c, &r, (25, 18));
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert!(event(&c, SPOIL));
    assert!(!event(&c, CACHE));
    let after = c.snapshot().expect("actual native victories").party;
    assert!(
        after
            .slots
            .iter()
            .any(|p| before.slots.iter().any(|old| old.index == p.index
                && p.pokemon.moves.iter().any(|m| old
                    .pokemon
                    .moves
                    .iter()
                    .any(|n| n.name == m.name && m.current_pp < n.current_pp))))
    );
    assert!(c.shell.shell.session().state().money > 1000);
    c = continued(&mut c, &root, &r, "both-native-victories");
    assert!(event(&c, ACCESS) && event(&c, SPOIL));
}
