//! Explicit local setup; garden traversal, the gift and every retry use input.
#[path = "minnesota_fishing_walk.rs"]
mod dry_walk;

use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::models::item::ITEM_POCKET_ITEM;
use crystal_runtime::CrystalRuntime;

use super::super::{
    FieldPackAction, FieldPackPocket, StartMenuOption, VisibleShellController,
    mark_runtime_snapshot_dirty, visible_selected_pack_item_actions, visible_start_menu_options,
};
use super::authored_route_fixture::{RouteGrid, RouteSpec, escape_wild, position};
use super::interior_life::fixture;
use super::northstar_ice_lair::event;
use super::winona::{at, finish, load, press, travel};
use dry_walk::path;

const MAP: &str = "MnR02C00R00";
const FLAG: &str = "EVENT_MN_R02_GARDEN_BERRY";

fn setup(full: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    let mut c = fixture(&root, &r, MAP, (10, 19));
    let state = c.shell.shell.session_mut().state_mut();
    if full {
        for (_, item) in r
            .data()
            .items
            .iter()
            .filter(|(id, item)| item.pocket == ITEM_POCKET_ITEM && id.as_str() != "BERRY")
            .take(ITEM_POCKET_CAPACITY)
        {
            assert!(
                state
                    .bag
                    .add_item(item, 1)
                    .expect("declared capacity fixture")
            );
        }
    } else {
        let mut partner = r
            .data()
            .create_pokemon("BIRCHIT", 10, Default::default())
            .expect("original Minnesota local garden companion");
        partner.hp = partner
            .max_hp
            .checked_sub(12)
            .expect("declared injured partner");
        partner.item = Some("SMOKE_BALL".into());
        assert!(state.storage.party.add_pokemon(partner));
        state.sync_party_from_storage();
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    // Setup ends here. No subsequent party, position, progress, bag or RNG writes.
    assert_eq!(quantity(&c, &r), 0);
    assert!(!event(&c, FLAG));
    (root, r, c)
}

fn quantity(c: &VisibleShellController, r: &CrystalRuntime) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items["BERRY"])
}

fn checkpoint(c: &mut VisibleShellController, label: &str) -> PathBuf {
    let dir = PathBuf::from(
        std::env::var_os("CRYSTAL_WORKING_FARMYARD_REVIEW_DIR")
            .expect("fresh ignored farmyard saves"),
    );
    std::fs::create_dir_all(&dir).expect("retained farmyard saves");
    let save = dir.join(format!("{label}.crystalsave"));
    assert!(!save.exists(), "retain every played farmyard checkpoint");
    c.save(&save).expect("save actual garden progress");
    save
}

fn continued(c: &mut VisibleShellController, root: &AssetRoot, r: &CrystalRuntime, label: &str) {
    let before = c.snapshot().expect("before garden Continue");
    let state = c.shell.shell.session().state().clone();
    *c = VisibleShellController::load_save(root.clone(), r.clone(), checkpoint(c, label), None)
        .expect("actual garden Continue");
    c.set_runtime_journal_enabled(false);
    let after = c.snapshot().expect("after garden Continue");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(state.flags, c.shell.shell.session().state().flags);
    assert_eq!(state.badges, c.shell.shell.session().state().badges);
    assert_eq!(state.money, c.shell.shell.session().state().money);
}

fn speak(c: &mut VisibleShellController) {
    at(c, MAP, (10, 19));
    press(c, GameButton::Left);
    at(c, MAP, (10, 19));
    press(c, GameButton::A);
}

fn walk(c: &mut VisibleShellController, r: &CrystalRuntime, to: (i16, i16)) -> usize {
    let mut encounters = 0;
    for _ in 0..256 {
        if position(c) == to {
            return encounters;
        }
        let before = position(c);
        let button = *path(c, r, to)
            .expect("ordinary live farmyard path")
            .first()
            .expect("remaining garden input");
        for _ in 0..8 {
            press(c, button);
            if c.snapshot().expect("garden encounter").battle.is_some() {
                escape_wild(c);
                encounters += 1;
            }
            if position(c) != before {
                break;
            }
        }
        assert_ne!(position(c), before, "blocked garden path");
    }
    panic!("bounded garden traversal");
}

fn use_berry(c: &mut VisibleShellController) {
    press(c, GameButton::Start);
    let s = c.snapshot().expect("production Start");
    let target = visible_start_menu_options(&c.shell, &s)
        .iter()
        .position(|o| *o == StartMenuOption::Pack)
        .expect("PACK");
    for _ in 0..16 {
        let selected = c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if selected == target {
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    press(c, GameButton::A);
    for _ in 0..4 {
        if c.shell.field_pack_pocket == Some(FieldPackPocket::Items) {
            break;
        }
        press(c, GameButton::Right);
    }
    assert_eq!(
        c.snapshot().expect("awarded Items").bag.items[0].item_id,
        "BERRY"
    );
    press(c, GameButton::A);
    let s = c.snapshot().expect("Berry actions");
    let actions = visible_selected_pack_item_actions(&s, &c.shell, &FieldPackPocket::Items, false)
        .expect("native Berry actions");
    assert_eq!(actions[0], FieldPackAction::Use);
    press(c, GameButton::A);
    assert_eq!(c.shell.party_cursor, 0);
    press(c, GameButton::A);
    finish(c);
    for _ in 0..8 {
        press(c, GameButton::B);
    }
}

#[test]
#[ignore = "requires farmyard candidate and fresh CRYSTAL_WORKING_FARMYARD_REVIEW_DIR"]
fn working_farmyard_controller_held_gift_native_berry_use_repeat_and_continue() {
    let (root, r, mut c) = setup(false);
    checkpoint(&mut c, "initial-garden-approach");
    speak(&mut c);
    let text = c
        .snapshot()
        .expect("grower offer")
        .ui
        .text
        .expect("printed offer");
    assert_eq!(text.label, "MnR02GardenOfferText");
    press(&mut c, GameButton::Right);
    at(&mut c, MAP, (10, 19));
    assert!(!event(&c, FLAG));
    assert_eq!(quantity(&c, &r), 0);
    continued(&mut c, &root, &r, "held-grower-offer");
    finish(&mut c);
    assert!(event(&c, FLAG));
    assert_eq!(quantity(&c, &r), 1);
    let before = c
        .snapshot()
        .expect("injured partner before awarded Berry")
        .party
        .slots[0]
        .pokemon
        .clone();
    use_berry(&mut c);
    let after = c
        .snapshot()
        .expect("native awarded Berry healing")
        .party
        .slots[0]
        .pokemon
        .clone();
    assert_eq!(after.hp, before.hp + 10);
    assert_eq!(after.moves, before.moves);
    assert_eq!(after.item, before.item);
    assert_eq!(quantity(&c, &r), 0);
    speak(&mut c);
    finish(&mut c);
    assert_eq!(quantity(&c, &r), 0);
    continued(&mut c, &root, &r, "berry-used-and-repeat");
    speak(&mut c);
    finish(&mut c);
    assert!(event(&c, FLAG));
    assert_eq!(quantity(&c, &r), 0);
    assert_eq!(c.shell.shell.session().state().badges.johto, [false; 8]);
}

#[test]
#[ignore = "requires farmyard candidate and fresh review output; declared full Items pocket"]
fn working_farmyard_controller_full_pocket_continue_native_toss_retry_once() {
    let (root, r, mut c) = setup(true);
    checkpoint(&mut c, "initial-full-garden-pocket");
    let before = c.snapshot().expect("full garden pocket").bag;
    assert_eq!(before.items.len(), ITEM_POCKET_CAPACITY);
    speak(&mut c);
    finish(&mut c);
    assert!(!event(&c, FLAG));
    assert_eq!(quantity(&c, &r), 0);
    assert_eq!(
        c.snapshot().expect("refused gift preserves pocket").bag,
        before
    );
    continued(&mut c, &root, &r, "garden-full-pocket-refused");
    super::northstar_progression_rewards::native_toss_first_pocket_item(
        &mut c,
        &FieldPackPocket::Items,
    );
    for _ in 0..8 {
        press(&mut c, GameButton::B);
    }
    assert_eq!(
        c.snapshot()
            .expect("native Toss frees one garden slot")
            .bag
            .items
            .len(),
        ITEM_POCKET_CAPACITY - 1
    );
    speak(&mut c);
    finish(&mut c);
    assert!(event(&c, FLAG));
    assert_eq!(quantity(&c, &r), 1);
    continued(&mut c, &root, &r, "garden-full-pocket-retry");
    speak(&mut c);
    finish(&mut c);
    assert_eq!(quantity(&c, &r), 1);
}

#[test]
#[ignore = "requires farmyard candidate and fresh review output; local original partner"]
fn working_farmyard_controller_both_stile_lanes_new_crops_wild_run_and_continue() {
    let (root, r, mut c) = setup(false);
    for x in [12, 13] {
        for y in 12..=19 {
            walk(&mut c, &r, (x, y));
        }
        for y in (12..=19).rev() {
            walk(&mut c, &r, (x, y));
        }
    }
    for x in [10, 14] {
        walk(&mut c, &r, (x, 16));
        press(&mut c, GameButton::Down);
        at(&mut c, MAP, (x, 16));
    }
    let mut encounters = 0;
    for _ in 0..64 {
        encounters += walk(&mut c, &r, (10, 20));
        encounters += walk(&mut c, &r, (10, 21));
        if encounters > 0 {
            break;
        }
    }
    assert!(
        encounters > 0,
        "new crops never started a native wild encounter"
    );
    walk(&mut c, &r, (10, 19));
    continued(&mut c, &root, &r, "working-crops-native-run");
}

#[test]
#[ignore = "requires farmyard candidate and fresh review output; explicit prior Care Badge fixture"]
fn working_farmyard_controller_both_lanes_both_real_town_seams_and_continue() {
    let (root, r) = load();
    let mut c = fixture(&root, &r, MAP, (58, 14));
    let state = c.shell.shell.session_mut().state_mut();
    state.badges.johto[0] = true;
    state
        .flags
        .set_engine_flag("ENGINE_ZEPHYRBADGE", true)
        .expect("declared prior Care Badge for the original Rochester departure gate");
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    // Empty-party local navigation only; no later state writes.
    let grid = RouteGrid::new_with_grass(
        &r,
        RouteSpec {
            prefix: "MnR02",
            columns: 1,
            rows: 1,
            tileset: "mn_prairie_kanto",
            start: (58, 14),
            doors: std::collections::BTreeMap::from([
                ((11, 11), "MnR02Farmhouse"),
                ((52, 33), "MnR02ToolShed"),
            ]),
        },
    );
    checkpoint(&mut c, "farm-road-initial-care-ready");
    for y in [14, 15] {
        grid.walk(&mut c, (59, y));
        travel(&mut c, GameButton::Right, "MnRochester", (0, y + 4));
        c.wait_frames(64).expect("stable Rochester inbound arrival");
        at(&mut c, "MnRochester", (0, y + 4));
        travel(&mut c, GameButton::Left, MAP, (59, y));
    }
    for y in [30, 31] {
        grid.walk(&mut c, (0, y));
        travel(&mut c, GameButton::Left, "MnMankato", (47, y - 12));
        c.wait_frames(64).expect("stable Mankato inbound arrival");
        at(&mut c, "MnMankato", (47, y - 12));
        travel(&mut c, GameButton::Right, MAP, (0, y));
    }
    grid.walk(&mut c, (10, 19));
    assert!(!event(&c, FLAG));
    continued(&mut c, &root, &r, "both-real-town-seams");
}
