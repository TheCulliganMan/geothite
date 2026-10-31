//! Declared local partner/setup; subsequent town play uses production input.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::models::item::ITEM_POCKET_ITEM;
use crystal_runtime::CrystalRuntime;

use super::super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::super::interior_life::{fixture, talk};
use super::super::northstar_ice_lair::walk;
use super::super::winona::{at, finish, load, press, travel};

const TOWN: &str = "MnEly";
const CACHE: &str = "EVENT_MN_ELY_PINE_SUPPLY";

fn setup(full: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    let mut c = fixture(&root, &r, TOWN, (24, 2));
    let state = c.shell.shell.session_mut().state_mut();
    let mut mon = r
        .data()
        .create_pokemon("VOYAGEAR", 60, Default::default())
        .expect("active regional partner");
    mon.hp = 3;
    mon.status = Some("BURN".into());
    mon.moves[0].current_pp = 0;
    assert!(state.storage.party.add_pokemon(mon));
    state.sync_party_from_storage();
    if full {
        assert!(
            state
                .bag
                .add_item(&r.data().items["HYPER_POTION"], 1)
                .expect("declared usable carried medicine")
        );
        for (_, item) in r
            .data()
            .items
            .iter()
            .filter(|(id, item)| {
                item.pocket == ITEM_POCKET_ITEM
                    && !["FULL_RESTORE", "HYPER_POTION"].contains(&id.as_str())
            })
            .take(ITEM_POCKET_CAPACITY - 1)
        {
            assert!(state.bag.add_item(item, 1).expect("declared full pocket"));
        }
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    // Setup ends here. No subsequent position, party, bag, flag or RNG writes.
    (root, r, c)
}

fn claimed(c: &VisibleShellController) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(CACHE)
        .expect("supply flag")
}

fn quantity(c: &VisibleShellController, r: &CrystalRuntime) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items["FULL_RESTORE"])
}

fn save(c: &mut VisibleShellController, name: &str) -> PathBuf {
    let out = PathBuf::from(
        std::env::var_os("CRYSTAL_ELY_PINE_CAMP_REVIEW_DIR").expect("fresh Ely review directory"),
    );
    std::fs::create_dir_all(&out).expect("review directory");
    let p = out.join(format!("{name}.crystalsave"));
    assert!(!p.exists(), "retain every reached state");
    c.save(&p).expect("native save");
    p
}

fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    name: &str,
) -> VisibleShellController {
    let mut next = VisibleShellController::load_save(root.clone(), r.clone(), save(c, name), None)
        .expect("production Continue");
    next.set_runtime_journal_enabled(false);
    next.wait_frames(16).expect("Continue presentation");
    finish(&mut next);
    next
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK and fresh CRYSTAL_ELY_PINE_CAMP_REVIEW_DIR"]
fn ely_pine_camp_controller_loop_residents_supply_real_use_services_and_continue() {
    let (root, r, mut c) = setup(false);
    save(&mut c, "initial");
    for (approach, target, words) in [
        ((21, 4), (21, 5), "pines"),
        ((33, 10), (32, 10), "supply nook"),
        ((29, 17), (29, 16), "FULL HEAL"),
    ] {
        walk(&mut c, &r, approach);
        let pages = talk(&mut c, target);
        assert!(pages.contains(words), "{pages}");
        at(&mut c, TOWN, approach);
    }
    save(&mut c, "sheltered-lodge");
    walk(&mut c, &r, (34, 5));
    save(&mut c, "supply-approach");
    assert!(!claimed(&c));
    assert_eq!(quantity(&c, &r), 0);
    talk(&mut c, (35, 5));
    assert!(claimed(&c));
    assert_eq!(quantity(&c, &r), 1);
    c = continued(&mut c, &root, &r, "supply-claimed");
    assert!(claimed(&c));
    assert_eq!(quantity(&c, &r), 1);
    let before = c.snapshot().expect("depleted partner").party.slots[0]
        .pokemon
        .clone();
    super::ely_league::restore_partner_with(&mut c, &r, "FULL_RESTORE", 0);
    let after = c.snapshot().expect("actual recovery").party.slots[0]
        .pokemon
        .clone();
    assert_eq!(after.hp, after.max_hp);
    assert!(after.status.is_none());
    assert_eq!(
        after.moves, before.moves,
        "Full Restore does not restore PP"
    );
    assert_eq!(quantity(&c, &r), 0);
    for y in [14, 15] {
        walk(&mut c, &r, (39, y));
        travel(&mut c, GameButton::Right, "MnElyShorePass", (0, y));
        travel(&mut c, GameButton::Left, TOWN, (39, y));
    }
    for x in [24, 25] {
        walk(&mut c, &r, (x, 0));
        travel(&mut c, GameButton::Up, "MnR13C00R02", (x + 10, 31));
        travel(&mut c, GameButton::Down, TOWN, (x, 0));
    }
    for (point, map, arrival) in [
        ((11, 22), "MnElyPokecenter1F", (3, 5)),
        ((27, 28), "MnElyMart", (2, 5)),
    ] {
        walk(&mut c, &r, point);
        save(
            &mut c,
            if map == "MnElyPokecenter1F" {
                "clinic-garden"
            } else {
                "mart-windbreak"
            },
        );
        travel(&mut c, GameButton::Up, map, arrival);
        if map == "MnElyPokecenter1F" {
            walk(&mut c, &r, (3, 3));
            assert!(!talk(&mut c, (3, 2)).is_empty());
            let healed = &c.snapshot().expect("original nurse care").party.slots[0].pokemon;
            assert!(
                healed
                    .moves
                    .iter()
                    .all(|m| m.current_pp == r.data().moves[&m.name].pp)
            );
        }
        // Find an original carpet by readonly metadata, then use native input.
        let m = &r.data().maps[map];
        let exit = m
            .events
            .warps
            .iter()
            .find(|w| {
                super::super::interior_life::collision(&r, m, (w.x as i16, w.y as i16))
                    == Some("WARP_CARPET_DOWN")
            })
            .expect("original room exit");
        walk(&mut c, &r, (exit.x as i16, exit.y as i16 - 1));
        travel(&mut c, GameButton::Down, TOWN, point);
    }
    walk(&mut c, &r, (25, 14));
    travel(&mut c, GameButton::Up, "MnLeagueLobby", (1, 4));
    travel(&mut c, GameButton::Left, TOWN, (25, 14));
    walk(&mut c, &r, (34, 5));
    assert!(claimed(&c));
    assert_eq!(quantity(&c, &r), 0);
    c = continued(&mut c, &root, &r, "loop-and-real-returns");
    assert!(claimed(&c));
    assert_eq!(quantity(&c, &r), 0);
}

#[test]
#[ignore = "requires current Ely pack and fresh output; explicit full-pocket setup"]
fn ely_pine_camp_controller_full_pocket_retains_supply_for_native_retry() {
    let (root, r, mut c) = setup(true);
    walk(&mut c, &r, (34, 5));
    talk(&mut c, (35, 5));
    assert!(!claimed(&c));
    assert_eq!(quantity(&c, &r), 0);
    c = continued(&mut c, &root, &r, "full-pocket-denial");
    assert!(!claimed(&c));
    // The full-pocket fixture contains a Hyper Potion; consuming it through
    // the real Pack frees a slot without changing inventory directly.
    super::ely_league::restore_partner_with(&mut c, &r, "HYPER_POTION", 0);
    talk(&mut c, (35, 5));
    assert!(claimed(&c));
    assert_eq!(quantity(&c, &r), 1);
    c = continued(&mut c, &root, &r, "capacity-retry-claimed");
    assert!(claimed(&c));
    assert_eq!(quantity(&c, &r), 1);
    press(&mut c, GameButton::Start);
    press(&mut c, GameButton::B);
}

#[cfg(feature = "location-tester")]
#[test]
#[ignore = "requires reached Ely saves and explicit offscreen native capture"]
fn ely_pine_camp_headless_captures_actual_reached_snowy_town() {
    use super::super::super::{BevyShellConfig, BevyShellStart, run_bevy_shell};
    assert_eq!(
        std::env::var("CRYSTAL_RENDER_TEST_HEADLESS").as_deref(),
        Ok("1")
    );
    assert_eq!(
        std::env::var("CRYSTAL_RENDER_TEST_CAPTURE_TARGET").as_deref(),
        Ok("image")
    );
    let (root, r) = load();
    let saves = PathBuf::from(
        std::env::var_os("CRYSTAL_ELY_PINE_CAMP_REVIEW_DIR").expect("reached native saves"),
    );
    let output = PathBuf::from(
        std::env::var_os("CRYSTAL_ELY_PINE_CAMP_CAPTURE_DIR")
            .expect("fresh native image directory"),
    );
    std::fs::create_dir_all(&output).expect("native capture directory");
    for name in [
        "sheltered-lodge",
        "supply-approach",
        "clinic-garden",
        "mart-windbreak",
        "loop-and-real-returns",
    ] {
        let screenshot = output.join(format!("{name}.png"));
        assert!(!screenshot.exists(), "retain original GPU images");
        run_bevy_shell(
            root.clone(),
            r.clone(),
            BevyShellStart::LoadSave {
                save_path: saves.join(format!("{name}.crystalsave")),
            },
            BevyShellConfig {
                voxel_view_enabled: Some(false),
                render_test_screenshot: Some(screenshot.clone()),
                render_test_window_size: Some((640, 576)),
                ..Default::default()
            },
        )
        .expect("actual reached snowy exterior capture");
        assert!(screenshot.is_file());
    }
}
