//! Declared local setups; all town travel, care, purchases and Continue use input.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::authored_route_fixture::indoor_walk;
use super::interior_life::fixture;
use super::winona::{at, finish, load, press, travel};

const CITY: &str = "MnRochester";

fn review_dir() -> PathBuf {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_ROCHESTER_CARE_GREEN_REVIEW_DIR")
            .expect("fresh ignored Rochester evidence directory"),
    );
    std::fs::create_dir_all(&path).expect("retained Rochester saves");
    path
}

fn checkpoint(c: &mut VisibleShellController, label: &str) -> PathBuf {
    let path = review_dir().join(format!("{label}.crystalsave"));
    assert!(!path.exists(), "retain every reached checkpoint");
    c.save(&path).expect("actual Rochester checkpoint");
    path
}

fn continued(c: &mut VisibleShellController, root: &AssetRoot, r: &CrystalRuntime, label: &str) {
    let before = c.snapshot().expect("before Continue");
    let state = c.shell.shell.session().state().clone();
    *c = VisibleShellController::load_save(root.clone(), r.clone(), checkpoint(c, label), None)
        .expect("native Rochester Continue");
    c.set_runtime_journal_enabled(false);
    let after = c.snapshot().expect("after Continue");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    let after_state = c.shell.shell.session().state();
    assert_eq!(state.flags, after_state.flags);
    assert_eq!(state.badges, after_state.badges);
    assert_eq!(state.money, after_state.money);
}

fn checked() -> (AssetRoot, CrystalRuntime) {
    let (root, r) = load();
    assert_eq!(super::rochester::gym_front(&r), (26, 18));
    let town = r
        .data()
        .assemble_map_module_from_compiled_payloads(CITY)
        .expect("actual native split-payload town");
    assert_eq!((town.attributes.width, town.attributes.height), (20, 16));
    assert_eq!(town.events.warps.len(), 12);
    assert_eq!(town.events.bg_events.len(), 5);
    (root, r)
}

#[test]
#[ignore = "requires current Rochester care-green pack and fresh review directory"]
fn rochester_care_green_six_doors_every_carpet_five_signs_and_continue() {
    let (root, r) = checked();
    let town = r
        .data()
        .assemble_map_module_from_compiled_payloads(CITY)
        .expect("native town");
    let mut c = fixture(&root, &r, CITY, (39, 18));
    // Empty-party initial placement only. No state writes after this boundary.
    let mut carpets = 0;
    let (door_pairs, remainder) = town.events.warps.as_chunks::<2>();
    assert!(remainder.is_empty(), "every door has a passive return pair");
    for pair in door_pairs {
        let door = &pair[0];
        let outside = (
            i16::try_from(pair[1].x).expect("x"),
            i16::try_from(pair[1].y).expect("y"),
        );
        let room = r
            .data()
            .maps
            .values()
            .find(|m| m.attributes.map_constant.as_deref() == Some(&door.target_map_constant))
            .expect("retained real room");
        let arrival = room
            .events
            .warps
            .iter()
            .find(|warp| i16::try_from(warp.index).ok() == Some(door.target_warp_id))
            .expect("native passive arrival");
        let inside = (
            i16::try_from(arrival.x).expect("x"),
            i16::try_from(arrival.y).expect("y"),
        );
        for exit in room.events.warps.iter().filter(|warp| {
            warp.target_map_constant == "MN_ROCHESTER"
                && i16::try_from(warp.index).ok() != Some(door.target_warp_id)
        }) {
            indoor_walk(&mut c, &r, outside);
            travel(&mut c, GameButton::Up, &room.id, inside);
            at(&mut c, &room.id, inside);
            indoor_walk(
                &mut c,
                &r,
                (
                    i16::try_from(exit.x).expect("x"),
                    i16::try_from(exit.y).expect("y"),
                ),
            );
            if c.snapshot().expect("carpet").overworld.map_name != CITY {
                travel(&mut c, GameButton::Down, CITY, outside);
            }
            at(&mut c, CITY, outside);
            continued(
                &mut c,
                &root,
                &r,
                &format!("{}-carpet-{}", room.id, exit.index),
            );
            carpets += 1;
        }
    }
    assert_eq!(carpets, 12);
    for event in &town.events.bg_events {
        let sign = (
            i16::try_from(event.x).expect("x"),
            i16::try_from(event.y).expect("y"),
        );
        let stand = (sign.0 + 1, sign.1);
        indoor_walk(&mut c, &r, stand);
        press(&mut c, GameButton::Left);
        at(&mut c, CITY, stand);
        press(&mut c, GameButton::A);
        let snapshot = c.snapshot().expect("actual sign text");
        assert_eq!(town.scripts[&event.script][0]["command"], "jumptext");
        let expected = town.scripts[&event.script][0]["args"][0]
            .as_str()
            .expect("authored native sign target");
        assert_eq!(
            snapshot.ui.text.expect("visible native reading").label,
            expected
        );
        press(&mut c, GameButton::Down);
        at(&mut c, CITY, stand);
        finish(&mut c);
    }
    indoor_walk(&mut c, &r, (19, 12));
    press(&mut c, GameButton::Up); // The actual gardener blocks this step.
    at(&mut c, CITY, (19, 12));
    checkpoint(&mut c, "clinic-garden");
    press(&mut c, GameButton::Start);
    assert!(c.snapshot().expect("native Start").ui.menu.is_some());
    press(&mut c, GameButton::B);
}

#[test]
#[ignore = "requires current Rochester care-green pack and fresh review directory"]
fn rochester_care_green_both_roads_free_inward_refusal_and_badge_exit() {
    let (root, r) = checked();
    let prairie_edge = i16::try_from(r.data().maps["MnR02C00R00"].attributes.width * 2 - 1)
        .expect("actual prairie route edge");
    for lane in 0..2 {
        let mut c = fixture(&root, &r, CITY, (39, 18 + lane));
        travel(&mut c, GameButton::Right, "MnR01C00R00", (0, 24 + lane));
        continued(&mut c, &root, &r, &format!("first-route-lane-{lane}"));
        travel(&mut c, GameButton::Left, CITY, (39, 18 + lane));
        let mut inbound = fixture(&root, &r, "MnR02C00R00", (prairie_edge, 14 + lane));
        travel(&mut inbound, GameButton::Right, CITY, (0, 18 + lane));
        travel(&mut inbound, GameButton::Right, CITY, (4, 18 + lane));
        assert_eq!(
            inbound.shell.shell.session().state().badges.johto,
            [false; 8]
        );
        for _ in 0..8 {
            if inbound
                .snapshot()
                .expect("visible Care refusal")
                .ui
                .text_window_open
            {
                break;
            }
            press(&mut inbound, GameButton::Left);
            assert_eq!(
                inbound
                    .snapshot()
                    .expect("guarded departure")
                    .overworld
                    .map_name,
                CITY
            );
        }
        assert!(
            inbound
                .snapshot()
                .expect("missing Care refusal")
                .ui
                .text_window_open
        );
        let before = inbound.snapshot().expect("owned refusal").overworld.tile;
        press(&mut inbound, GameButton::Left);
        assert_eq!(
            inbound.snapshot().expect("text owns input").overworld.tile,
            before
        );
        finish(&mut inbound);
        at(&mut inbound, CITY, (4, 18 + lane));
        continued(
            &mut inbound,
            &root,
            &r,
            &format!("free-inbound-refusal-{lane}"),
        );
        let mut qualified = fixture(&root, &r, CITY, (4, 18 + lane));
        let state = qualified.shell.shell.session_mut().state_mut();
        state.badges.johto[0] = true;
        state
            .flags
            .set_engine_flag("ENGINE_ZEPHYRBADGE", true)
            .expect("declared consistent Care fixture");
        mark_runtime_snapshot_dirty(&mut qualified.shell);
        // No state writes after initial qualification. Native scripts remove wardens.
        travel(
            &mut qualified,
            GameButton::Left,
            "MnR02C00R00",
            (prairie_edge, 14 + lane),
        );
        travel(&mut qualified, GameButton::Right, CITY, (0, 18 + lane));
        travel(&mut qualified, GameButton::Right, CITY, (4, 18 + lane));
        continued(
            &mut qualified,
            &root,
            &r,
            &format!("qualified-road-return-{lane}"),
        );
    }
}

#[test]
#[ignore = "requires current Rochester care-green pack and fresh review directory"]
fn rochester_care_green_native_clinic_pc_priced_shop_and_continue() {
    let (root, r) = checked();
    let mut c = fixture(&root, &r, CITY, (39, 18));
    for (species, level) in [("POUCHID", 20), ("CINDORE", 12)] {
        let mut partner = r
            .data()
            .create_pokemon(species, level, Dv::default())
            .expect("declared native review partner");
        partner.hp = partner.max_hp.saturating_sub(5).max(1);
        partner.status = Some("PARALYSIS".into());
        for m in &mut partner.moves {
            m.current_pp = 0;
        }
        assert!(
            c.shell
                .shell
                .session_mut()
                .state_mut()
                .storage
                .party
                .add_pokemon(partner)
        );
    }
    let state = c.shell.shell.session_mut().state_mut();
    state.sync_party_from_storage();
    state.money = 5000;
    mark_runtime_snapshot_dirty(&mut c.shell);
    // Party/HP/PP/status/money are initial fixtures only; all services use input.
    indoor_walk(&mut c, &r, (15, 8));
    travel(&mut c, GameButton::Up, "MnRochesterPokecenter1F", (10, 13));
    super::rochester_earned::heal(&mut c, &r);
    indoor_walk(&mut c, &r, (13, 4));
    press(&mut c, GameButton::Up);
    assert_eq!(
        c.shell
            .shell
            .current_overworld_interaction_checked()
            .expect("PC interaction")
            .map(|i| i.script),
        Some("MnRochesterPokecenter1FPC".into())
    );
    press(&mut c, GameButton::A);
    for _ in 0..32 {
        if c.shell.pc_hub_cursor.is_some() {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(c.shell.pc_hub_cursor.is_some());
    press(&mut c, GameButton::Right);
    at(&mut c, "MnRochesterPokecenter1F", (13, 4));
    for _ in 0..16 {
        if c.shell.pc_hub_cursor.is_none() {
            break;
        }
        press(&mut c, GameButton::B);
    }
    finish(&mut c);
    assert!(c.shell.pc_hub_cursor.is_none());
    continued(&mut c, &root, &r, "native-care-pc");
    super::rochester_earned::clinic_exit(&mut c, &r);
    let before = c.shell.shell.session().state().money;
    let quantity = c
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items["POTION"]);
    super::rochester_earned::supplies(&mut c, &r, 3);
    assert_eq!(
        c.shell.shell.session().state().money,
        before - 3 * u32::from(r.data().items["POTION"].price)
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["POTION"]),
        quantity + 3
    );
    continued(&mut c, &root, &r, "native-care-shop-return");
}
