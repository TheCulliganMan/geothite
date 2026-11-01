//! Explicit local partners/rod/badge fixtures; all subsequent play uses native input.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleFishingPhase, VisibleShellController, mark_runtime_snapshot_dirty};
use super::authored_route_fixture::indoor_walk;
use super::interior_life::fixture;
use super::winona::{at, finish, load, press, travel};

const CITY: &str = "MnBrainerd";

fn frontage(r: &CrystalRuntime, room: &str) -> (i16, i16) {
    super::native_city_ports::city_port(r, CITY, room).0
}

fn setup(injured: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    assert_eq!(frontage(&r, "MnBrainerdPokecenter1F"), (9, 22));
    assert_eq!(frontage(&r, "MnBrainerdMart"), (19, 22));
    assert_eq!(frontage(&r, "MnBrainerdGym"), (12, 36));
    assert_eq!(frontage(&r, "MnBrainerdOutfitter"), (51, 24));
    let mut c = fixture(&root, &r, CITY, (9, 22));
    let mut partner = r
        .data()
        .create_pokemon("CINDORE", 25, Default::default())
        .expect("original Minnesota town companion");
    partner.item = Some("SMOKE_BALL".into());
    if injured {
        partner.hp = partner.max_hp.saturating_sub(20).max(1);
        partner.status = Some("POISON".into());
        for learned in &mut partner.moves {
            learned.current_pp = 0;
        }
    }
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    assert!(
        state
            .bag
            .add_item(&r.data().items["OLD_ROD"], 1)
            .expect("declared local rod")
    );
    state.registered_key_item = Some("OLD_ROD".into());
    mark_runtime_snapshot_dirty(&mut c.shell);
    // Local setup ends here. No later state, position, party, flag or RNG writes.
    (root, r, c)
}

fn checkpoint(c: &mut VisibleShellController, name: &str) -> PathBuf {
    let folder = PathBuf::from(
        std::env::var_os("CRYSTAL_LAKESIDE_LOOP_REVIEW_DIR")
            .expect("fresh ignored lakeside output"),
    );
    std::fs::create_dir_all(&folder).expect("native lakeside review saves");
    let path = folder.join(format!("{name}.crystalsave"));
    assert!(!path.exists(), "retain every played checkpoint");
    c.save(&path).expect("save actual lakeside progress");
    path
}

fn continued(c: &mut VisibleShellController, root: &AssetRoot, r: &CrystalRuntime, name: &str) {
    let before = c.snapshot().expect("before lakeside Continue");
    let state = c.shell.shell.session().state().clone();
    *c = VisibleShellController::load_save(root.clone(), r.clone(), checkpoint(c, name), None)
        .expect("native lakeside Continue");
    c.set_runtime_journal_enabled(false);
    let after = c.snapshot().expect("continued lakeside state");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.mode, after.overworld.mode);
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(state.flags, c.shell.shell.session().state().flags);
    assert_eq!(state.badges, c.shell.shell.session().state().badges);
    assert_eq!(state.money, c.shell.shell.session().state().money);
}

#[test]
#[ignore = "requires lakeside candidate and fresh CRYSTAL_LAKESIDE_LOOP_REVIEW_DIR"]
fn lakeside_loop_controller_five_doors_ten_carpets_seven_posts_and_continue() {
    let (root, r, mut c) = setup(false);
    let town = r
        .data()
        .assemble_map_module_from_compiled_payloads(CITY)
        .expect("installed native split-payload town");
    assert_eq!(town.events.warps.len(), 10);
    let (pairs, rest) = town.events.warps.as_chunks::<2>();
    assert!(rest.is_empty());
    let mut carpets = 0;
    for pair in pairs {
        let room = r
            .data()
            .maps
            .values()
            .find(|m| m.attributes.map_constant.as_deref() == Some(&pair[0].target_map_constant))
            .expect("unchanged actual room");
        let arrival = room
            .events
            .warps
            .iter()
            .find(|warp| i16::try_from(warp.index).ok() == Some(pair[0].target_warp_id))
            .expect("native passive room arrival");
        let inside = (
            i16::try_from(arrival.x).expect("x"),
            i16::try_from(arrival.y).expect("y"),
        );
        let outside = frontage(&r, &room.id);
        for exit in room.events.warps.iter().filter(|warp| {
            warp.target_map_constant == "MN_BRAINERD"
                && i16::try_from(warp.index).ok() != Some(pair[0].target_warp_id)
        }) {
            indoor_walk(&mut c, &r, outside);
            travel(&mut c, GameButton::Up, &room.id, inside);
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
    assert_eq!(carpets, 10);
    for event in &town.events.bg_events {
        let stand = (
            i16::try_from(event.x + 1).expect("x"),
            i16::try_from(event.y).expect("y"),
        );
        indoor_walk(&mut c, &r, stand);
        press(&mut c, GameButton::Left);
        at(&mut c, CITY, stand);
        press(&mut c, GameButton::A);
        let snapshot = c.snapshot().expect("actual sign reading");
        assert_eq!(town.scripts[&event.script][0]["command"], "jumptext");
        assert_eq!(
            snapshot.ui.text.expect("native text").label,
            town.scripts[&event.script][0]["args"][0]
                .as_str()
                .expect("authored sign target")
        );
        press(&mut c, GameButton::Down);
        at(&mut c, CITY, stand);
        finish(&mut c);
    }
    assert_eq!(town.events.bg_events.len(), 7);
    checkpoint(&mut c, "five-door-town");
}

#[test]
#[ignore = "requires lakeside candidate and fresh CRYSTAL_LAKESIDE_LOOP_REVIEW_DIR"]
fn lakeside_loop_controller_both_road_lanes_rail_pier_fishing_and_continue() {
    let (root, r, mut c) = setup(false);
    for y in [24, 25] {
        for (left, right) in [(8, 31), (38, 59)] {
            indoor_walk(&mut c, &r, (left, y));
            for x in left + 1..=right {
                travel(&mut c, GameButton::Right, CITY, (x, y));
            }
            for x in (left..right).rev() {
                travel(&mut c, GameButton::Left, CITY, (x, y));
            }
        }
    }
    // The relocated span closes the forest margin, not the through road.
    for x in 38..48 {
        indoor_walk(&mut c, &r, (x, 26));
        press(&mut c, GameButton::Down);
        at(&mut c, CITY, (x, 26));
    }
    checkpoint(&mut c, "clear-two-lane-street");
    // Both pier columns remain usable; the fisher has a clear passing column.
    indoor_walk(&mut c, &r, (47, 20));
    for y in (13..20).rev() {
        travel(&mut c, GameButton::Up, CITY, (47, y));
    }
    travel(&mut c, GameButton::Right, CITY, (48, 13));
    for y in [14, 15] {
        travel(&mut c, GameButton::Down, CITY, (48, y));
    }
    press(&mut c, GameButton::Down);
    at(&mut c, CITY, (48, 15));
    press(&mut c, GameButton::A);
    assert!(c.snapshot().expect("native fisher pages").ui.text.is_some());
    finish(&mut c);
    travel(&mut c, GameButton::Left, CITY, (47, 15));
    for y in 16..=20 {
        travel(&mut c, GameButton::Down, CITY, (47, y));
    }
    indoor_walk(&mut c, &r, (44, 20));
    let before = c.snapshot().expect("dry lake bank");
    assert_eq!(before.overworld.mode, MovementMode::Normal);
    assert_eq!(c.shell.shell.session().state().badges.johto, [false; 8]);
    assert!(before.party.slots.iter().all(|p| {
        p.pokemon
            .moves
            .iter()
            .all(|m| !matches!(m.name.as_str(), "CUT" | "SURF" | "WATERFALL"))
    }));
    press(&mut c, GameButton::Up);
    at(&mut c, CITY, (44, 20));
    checkpoint(&mut c, "dry-rod-bank");
    let mut bite = false;
    for _ in 0..64 {
        press(&mut c, GameButton::Select);
        assert!(
            c.shell.visible_fishing_animation.is_some(),
            "native rod animation"
        );
        for _ in 0..512 {
            c.wait_frames(1).expect("native casting clock");
            if c.shell
                .visible_fishing_animation
                .as_ref()
                .is_some_and(|a| a.phase == VisibleFishingPhase::AwaitText)
            {
                break;
            }
        }
        for _ in 0..128 {
            let s = c.snapshot().expect("native rod result");
            if s.battle.is_some()
                || (c.shell.visible_fishing_animation.is_none()
                    && c.shell.field_notice.is_none()
                    && !c.has_pending_script_work())
            {
                break;
            }
            press(&mut c, GameButton::A);
        }
        if let Some(battle) = c.snapshot().expect("native lake fish").battle {
            assert!(matches!(
                battle.enemy_pokemon.species.id.as_str(),
                "SCUTLET" | "CLINKLAW"
            ));
            checkpoint(&mut c, "actual-lake-fish");
            super::rochester::flee(&mut c);
            bite = true;
            break;
        }
        finish(&mut c);
    }
    assert!(bite, "ordinary dry-bank casting must produce a native bite");
    at(&mut c, CITY, (44, 20));
    assert_eq!(c.snapshot().expect("after RUN").party, before.party);
    assert_eq!(c.shell.shell.session().state().badges.johto, [false; 8]);
    continued(&mut c, &root, &r, "dry-rod-bank-continue");
    indoor_walk(&mut c, &r, (32, 28));
    for y in 29..=45 {
        travel(&mut c, GameButton::Down, CITY, (32, y));
    }
    travel(&mut c, GameButton::Right, CITY, (33, 45));
    for y in (28..45).rev() {
        travel(&mut c, GameButton::Up, CITY, (33, y));
    }
    checkpoint(&mut c, "south-boulevard-return");
}

#[test]
#[ignore = "requires lakeside candidate and fresh CRYSTAL_LAKESIDE_LOOP_REVIEW_DIR"]
fn lakeside_loop_controller_native_care_relocated_supply_gift_repeat_and_continue() {
    let (root, r, mut c) = setup(true);
    travel(&mut c, GameButton::Up, "MnBrainerdPokecenter1F", (10, 13));
    indoor_walk(&mut c, &r, (7, 5));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    let held = c.snapshot().expect("native nurse text");
    assert!(held.ui.text.is_some());
    press(&mut c, GameButton::Down);
    at(&mut c, "MnBrainerdPokecenter1F", (7, 5));
    finish(&mut c);
    let healed = c.snapshot().expect("native healed companion").party.slots[0]
        .pokemon
        .clone();
    assert_eq!(healed.hp, healed.max_hp);
    assert_eq!(healed.status, None);
    assert!(!healed.moves.is_empty());
    assert!(
        healed
            .moves
            .iter()
            .all(|m| m.current_pp == r.data().moves[&m.name].pp)
    );
    indoor_walk(&mut c, &r, (10, 15));
    if c.snapshot().expect("clinic carpet").overworld.map_name != CITY {
        travel(&mut c, GameButton::Down, CITY, (9, 22));
    }
    indoor_walk(&mut c, &r, (51, 24));
    checkpoint(&mut c, "outfitter-frontage");
    travel(&mut c, GameButton::Up, "MnBrainerdOutfitter", (6, 5));
    indoor_walk(&mut c, &r, (3, 4));
    let before = c
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items["SUPER_REPEL"]);
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["SUPER_REPEL"]),
        before + 1
    );
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_BRAINERD_CAMP_REPEL")
            .expect("actual gift flag")
    );
    continued(&mut c, &root, &r, "supply-gift-continue");
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["SUPER_REPEL"]),
        before + 1
    );
    indoor_walk(&mut c, &r, (7, 7));
    if c.snapshot().expect("supply carpet").overworld.map_name != CITY {
        travel(&mut c, GameButton::Down, CITY, (51, 24));
    }
    at(&mut c, CITY, (51, 24));
    checkpoint(&mut c, "supply-gift-town-return");
    press(&mut c, GameButton::Start);
    assert!(
        c.snapshot()
            .expect("Start after actual town return")
            .ui
            .menu
            .is_some()
    );
}
