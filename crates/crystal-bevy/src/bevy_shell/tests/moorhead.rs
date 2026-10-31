//! Explicit setup fixtures; every journey, service, marker and gift uses native input.
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life::{fixture, load, talk, travel, walk};
use super::r10_gallery::{continue_save, press};

const CITY: &str = "MnMoorhead";
const CLINIC: &str = "MnMoorheadPokecenter1F";
const READ: &str = "EVENT_MN_MOORHEAD_GAUGE_READ";
const GIVEN: &str = "EVENT_MN_MOORHEAD_DRY_KIT_GIVEN";

fn point(v: &Value) -> (i16, i16) {
    (
        v[0].as_i64().expect("x") as i16,
        v[1].as_i64().expect("y") as i16,
    )
}

fn flag(c: &VisibleShellController, name: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("native Moorhead flag")
}

fn quantity(c: &VisibleShellController, r: &CrystalRuntime) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items["FULL_HEAL"])
}

fn read_marker(c: &mut VisibleShellController, r: &CrystalRuntime) {
    if c.snapshot().expect("before marker walk").overworld.map_name != CITY {
        travel(c, r, CITY);
    }
    walk(c, r, (13, 4));
    let text = talk(c, (12, 4));
    assert!(text.contains("high-water"));
    assert!(flag(c, READ));
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with Moorhead"]
fn moorhead_controller_all_people_rooms_returns_and_both_prairie_lanes() {
    let (root, r, _) = load();
    let plan: Value = serde_json::from_str(
        &std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../planning/minnesota/towns/MnMoorhead.json"),
        )
        .expect("Moorhead source"),
    )
    .expect("native source");
    let mut c = fixture(&root, &r, CITY, (30, 12));
    let mut partner = r
        .data()
        .create_pokemon("VOYAGEAR", 50, Dv::default())
        .expect("legal Surf fixture");
    partner.moves = vec![LearnedMove {
        name: "SURF".into(),
        current_pp: r.data().moves["SURF"].pp,
        pp_ups: 0,
    }];
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.badges.johto[3] = true;
    state
        .flags
        .set_engine_flag("ENGINE_FOGBADGE", true)
        .expect("consistent initial Surf badge");
    mark_runtime_snapshot_dirty(&mut c.shell);
    // Native input begins here; the Surf party is present throughout the tour.
    for actor in plan["residents"].as_array().expect("town residents") {
        walk(&mut c, &r, point(&actor["approach"]));
        assert!(!talk(&mut c, point(&actor["tile"])).is_empty());
    }
    for placard in plan["signs"].as_array().expect("public placards") {
        walk(&mut c, &r, point(&placard["approach"]));
        assert!(!talk(&mut c, point(&placard["tile"])).is_empty());
    }
    for door in plan["doors"].as_array().expect("two real facades") {
        let name = door["target"].as_str().expect("room");
        let room = plan["interiors"]
            .as_array()
            .expect("rooms")
            .iter()
            .find(|v| v["id"] == name)
            .expect("paired room");
        walk(&mut c, &r, point(&door["tile"]));
        assert_eq!(c.snapshot().expect("native door").overworld.map_name, name);
        for actor in room["residents"].as_array().expect("residents") {
            if actor["service"] == "pokemart" {
                continue;
            }
            walk(&mut c, &r, point(&actor["approach"]));
            assert!(
                !talk(
                    &mut c,
                    point(actor.get("counter_tile").unwrap_or(&actor["tile"]))
                )
                .is_empty()
            );
        }
        for exit in room["exits"].as_array().expect("paired carpets") {
            walk(&mut c, &r, point(&exit["tile"]));
            let s = c.snapshot().expect("exact town return").overworld;
            assert_eq!(s.map_name, CITY);
            assert_eq!((s.tile.x, s.tile.y), point(&door["return_tile"]));
            walk(&mut c, &r, point(&door["tile"]));
            assert_eq!(
                c.snapshot().expect("passive reentry").overworld.map_name,
                name
            );
        }
        travel(&mut c, &r, CITY);
    }
    for y in 2..22 {
        walk(&mut c, &r, (11, y));
        if r.data().maps[CITY]
            .scripts
            .contains_key("MnMoorheadRiverAccess")
            && matches!(y, 12 | 13)
        {
            // The habitat successor opens exactly this two-tile brick bank.
            // Its separate regression casts, mounts and tests the river bounds.
            for _ in 0..4 {
                press(&mut c, GameButton::Left);
                if c.snapshot().expect("brick bank step").overworld.tile.x == 10 {
                    break;
                }
            }
            let bank = c.snapshot().expect("signed bank approach").overworld;
            assert_eq!((bank.tile.x, bank.tile.y), (10, y));
            assert_eq!(bank.mode, MovementMode::Normal);
            walk(&mut c, &r, (11, y));
            continue;
        }
        press(&mut c, GameButton::Left);
        press(&mut c, GameButton::A);
        let s = c.snapshot().expect("actual levee barrier").overworld;
        assert_eq!((s.tile.x, s.tile.y), (11, y));
        assert_eq!(
            s.mode,
            MovementMode::Normal,
            "no launch through the native levee"
        );
    }
    for lane in [12, 13] {
        walk(&mut c, &r, (31, lane));
        for _ in 0..8 {
            press(&mut c, GameButton::Right);
            if c.snapshot().expect("outbound seam").overworld.map_name == "MnR16C00R00" {
                break;
            }
        }
        let s = c.snapshot().expect("actual prairie arrival").overworld;
        assert_eq!(s.map_name, "MnR16C00R00");
        assert_eq!((s.tile.x, s.tile.y), (0, lane + 12));
        for _ in 0..8 {
            press(&mut c, GameButton::Left);
            if c.snapshot().expect("return seam").overworld.map_name == CITY {
                break;
            }
        }
        let s = c.snapshot().expect("exact reciprocal lane").overworld;
        assert_eq!(s.map_name, CITY);
        assert_eq!((s.tile.x, s.tile.y), (31, lane));
    }
    let before = c.snapshot().expect("before Continue").overworld;
    c = continue_save(root, r.clone(), &mut c);
    let after = c.snapshot().expect("continued town").overworld;
    assert_eq!(
        (after.map_name, after.tile, after.facing, after.mode),
        (before.map_name, before.tile, before.facing, before.mode)
    );
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with Moorhead"]
fn moorhead_controller_free_care_pc_poison_recovery_and_priced_stock() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, CLINIC, (3, 6));
    let mut partner = r
        .data()
        .create_pokemon("GILLPUP", 20, Dv::default())
        .expect("care fixture");
    partner.hp = 10;
    partner.status = Some("POISON".into());
    for m in &mut partner.moves {
        m.current_pp = 0;
    }
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.money = 0;
    mark_runtime_snapshot_dirty(&mut c.shell);
    walk(&mut c, &r, (3, 3));
    talk(&mut c, (3, 2));
    let s = c.snapshot().expect("native free care");
    let p = &s.party.slots[0].pokemon;
    assert_eq!(p.hp, p.max_hp);
    assert!(p.status.is_none());
    assert!(
        p.moves
            .iter()
            .all(|m| m.current_pp == r.data().moves[&m.name].pp)
    );
    assert_eq!(c.shell.shell.session().state().money, 0);
    walk(&mut c, &r, (9, 2));
    press(&mut c, GameButton::Up);
    for _ in 0..96 {
        press(&mut c, GameButton::A);
        if c.shell.pc_hub_cursor.is_some() {
            break;
        }
    }
    assert!(c.shell.pc_hub_cursor.is_some());
    let at = c.snapshot().expect("PC owner").overworld.tile;
    press(&mut c, GameButton::Down);
    assert_eq!(c.snapshot().expect("PC blocks walking").overworld.tile, at);
    for _ in 0..96 {
        if c.shell.pc_hub_cursor.is_none() && !c.has_pending_script_work() {
            break;
        }
        press(&mut c, GameButton::B);
    }
    c = continue_save(root.clone(), r.clone(), &mut c);
    travel(&mut c, &r, CITY);
    // Start a separate declared poison scenario from the actual reached,
    // registered-clinic save. The original healthy checkpoint is unchanged.
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = super::authored_route_fixture::reached_save_path(&format!(
        "moorhead-earned-clinic-poison-scenario-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    c.save(&path)
        .expect("reached native clinic registration checkpoint");
    c = VisibleShellController::load_save(root.clone(), r.clone(), path.clone(), None)
        .expect("fresh local poison scenario");
    super::authored_route_fixture::cleanup_reached_save(&path);
    // Initial damage and money are fixtures; all later recovery uses input.
    let state = c.shell.shell.session_mut().state_mut();
    let battle_result = state.battle_result;
    let p = state.storage.party.pokemon[0].as_mut().expect("partner");
    p.hp = 1;
    p.status = Some("POISON".into());
    state.sync_party_from_storage();
    state.money = 101;
    mark_runtime_snapshot_dirty(&mut c.shell);
    for _ in 0..128 {
        if c.shell.field_notice.is_some()
            || !c.shell.battle_messages.is_empty()
            || c.shell.visible_blackout_phase.is_some()
            || c.snapshot()
                .expect("whiteout input owner")
                .ui
                .text_window_open
        {
            press(&mut c, GameButton::A);
        } else {
            for b in [
                GameButton::Left,
                GameButton::Left,
                GameButton::Right,
                GameButton::Right,
            ] {
                press(&mut c, b);
            }
        }
        if c.snapshot().expect("recovery").overworld.map_name == CLINIC {
            break;
        }
    }
    let s = c.snapshot().expect("registered native recovery");
    assert_eq!(s.overworld.map_name, CLINIC);
    assert_eq!((s.overworld.tile.x, s.overworld.tile.y), (3, 6));
    assert_eq!(s.party.slots[0].pokemon.hp, s.party.slots[0].pokemon.max_hp);
    assert_eq!(c.shell.shell.session().state().money, 50);
    assert_eq!(c.shell.shell.session().state().battle_result, battle_result);
    let mut shop = fixture(&root, &r, "MnMoorheadMart", (3, 6));
    shop.shell.shell.session_mut().state_mut().money = 10_000;
    mark_runtime_snapshot_dirty(&mut shop.shell);
    walk(&mut shop, &r, (3, 3));
    press(&mut shop, GameButton::Left);
    press(&mut shop, GameButton::A);
    for _ in 0..64 {
        if shop.shell.shop_top_cursor.is_some()
            && shop.shell.shop_welcome_seen
            && shop.shell.shop_notice.is_none()
        {
            break;
        }
        press(&mut shop, GameButton::A);
    }
    assert_eq!(
        shop.snapshot()
            .expect("stock")
            .pending_shop
            .expect("native shop")
            .inventory,
        r.data().marts.0["MART_MN_MOORHEAD"]
    );
    let item = &r.data().items["GREAT_BALL"];
    let before = shop.shell.shell.session().state().bag.quantity(item);
    for _ in 0..3 {
        press(&mut shop, GameButton::A);
    }
    assert_eq!(shop.shell.shell.session().state().money, 10_000);
    press(&mut shop, GameButton::A);
    assert_eq!(
        shop.shell.shell.session().state().money,
        10_000 - u32::from(item.price)
    );
    assert_eq!(
        shop.shell.shell.session().state().bag.quantity(item),
        before + 1
    );
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with Moorhead"]
fn moorhead_controller_reads_real_marker_then_remembers_notebook_gift_after_continue() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, CLINIC, (2, 4));
    let before = quantity(&c, &r);
    assert!(!flag(&c, READ) && !flag(&c, GIVEN));
    assert!(talk(&mut c, (2, 3)).contains("levee"));
    assert_eq!(quantity(&c, &r), before);
    assert!(!flag(&c, GIVEN));
    read_marker(&mut c, &r);
    c = continue_save(root.clone(), r.clone(), &mut c);
    travel(&mut c, &r, CLINIC);
    walk(&mut c, &r, (2, 4));
    let text = talk(&mut c, (2, 3));
    assert!(text.contains("names"));
    assert_eq!(quantity(&c, &r), before + 1);
    assert!(flag(&c, READ) && flag(&c, GIVEN));
    c = continue_save(root, r.clone(), &mut c);
    assert!(talk(&mut c, (2, 3)).contains("scorecard"));
    assert_eq!(quantity(&c, &r), before + 1);
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with Moorhead"]
fn moorhead_controller_full_pocket_keeps_kit_then_native_toss_retry_and_continue() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, CITY, (30, 12));
    c.shell.shell.session_mut().state_mut().bag.items.clear();
    c.shell
        .shell
        .add_bag_item("POTION", 1)
        .expect("one tossable fixture item");
    while c.shell.shell.session().state().bag.items.len()
        < crystal_core::models::bag::ITEM_POCKET_CAPACITY
    {
        c.shell
            .shell
            .add_bag_item("BERRY", 99)
            .expect("full physical pocket fixture");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    read_marker(&mut c, &r);
    travel(&mut c, &r, CLINIC);
    walk(&mut c, &r, (2, 4));
    assert!(talk(&mut c, (2, 3)).to_lowercase().contains("full"));
    assert!(flag(&c, READ));
    assert!(!flag(&c, GIVEN));
    assert_eq!(quantity(&c, &r), 0);
    c = continue_save(root.clone(), r.clone(), &mut c);
    assert!(!flag(&c, GIVEN));
    super::detroit_lakes::toss_potion(&mut c);
    talk(&mut c, (2, 3));
    assert!(flag(&c, GIVEN));
    assert_eq!(quantity(&c, &r), 1);
    c = continue_save(root, r.clone(), &mut c);
    talk(&mut c, (2, 3));
    assert_eq!(quantity(&c, &r), 1);
}
