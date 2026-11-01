//! Declared local starting party; every later door, road, conversation and return uses input.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::authored_route_fixture::indoor_walk;
use super::grand_rapids::frontage;
use super::interior_life::fixture;
use super::winona::{at, finish, load, press, travel};

const CITY: &str = "MnGrandRapids";

fn setup() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    assert_eq!(frontage(&r, "MnGrandRapidsPokecenter1F"), (9, 22));
    assert_eq!(frontage(&r, "MnGrandRapidsMart"), (29, 30));
    assert_eq!(frontage(&r, "MnGrandRapidsSurveyDepot"), (21, 24));
    let mut c = fixture(&root, &r, CITY, (6, 24));
    let partner = r
        .data()
        .create_pokemon("CINDORE", 50, Default::default())
        .expect("declared original local partner");
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    // No state writes after the complete initial fixture.
    (root, r, c)
}

fn checkpoint(c: &mut VisibleShellController, name: &str) -> PathBuf {
    let folder = PathBuf::from(
        std::env::var_os("CRYSTAL_DEPOT_QUAY_REVIEW_DIR")
            .expect("fresh ignored depot/quay review directory"),
    );
    std::fs::create_dir_all(&folder).expect("retained review saves");
    let path = folder.join(format!("{name}.crystalsave"));
    assert!(!path.exists(), "retain every reached checkpoint");
    c.save(&path).expect("actual played town checkpoint");
    path
}

fn continued(c: &mut VisibleShellController, root: &AssetRoot, r: &CrystalRuntime, name: &str) {
    let before = c.snapshot().expect("before Continue");
    let state = c.shell.shell.session().state().clone();
    *c = VisibleShellController::load_save(root.clone(), r.clone(), checkpoint(c, name), None)
        .expect("native depot/quay Continue");
    c.set_runtime_journal_enabled(false);
    let after = c.snapshot().expect("after Continue");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(state.flags, c.shell.shell.session().state().flags);
    assert_eq!(state.badges, c.shell.shell.session().state().badges);
    assert_eq!(state.money, c.shell.shell.session().state().money);
}

#[test]
#[ignore = "requires current depot/quay candidate and fresh CRYSTAL_DEPOT_QUAY_REVIEW_DIR"]
fn depot_quay_controller_three_doors_six_carpets_three_posts_and_continue() {
    let (root, r, mut c) = setup();
    let town = r
        .data()
        .assemble_map_module_from_compiled_payloads(CITY)
        .expect("installed native split-payload town");
    assert_eq!(town.events.warps.len(), 6);
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
            warp.target_map_constant == "MN_GRAND_RAPIDS"
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
    assert_eq!(carpets, 6);
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
    indoor_walk(&mut c, &r, (17, 24));
    press(&mut c, GameButton::Up);
    at(&mut c, CITY, (17, 24));
    press(&mut c, GameButton::A);
    assert!(
        c.snapshot()
            .expect("worker's original story")
            .ui
            .text_window_open
    );
    finish(&mut c);
    checkpoint(&mut c, "worker-and-care-frontage");
}

#[test]
#[ignore = "requires current depot/quay candidate and fresh CRYSTAL_DEPOT_QUAY_REVIEW_DIR"]
fn depot_quay_controller_clear_street_and_pier_lanes_native_fisher_decline_and_continue() {
    let (root, r, mut c) = setup();
    for y in [24, 25] {
        indoor_walk(&mut c, &r, (4, y));
        for x in 5..=43 {
            travel(&mut c, GameButton::Right, CITY, (x, y));
        }
        for x in (4..43).rev() {
            travel(&mut c, GameButton::Left, CITY, (x, y));
        }
    }
    for x in [25, 26] {
        indoor_walk(&mut c, &r, (x, 34));
        travel(&mut c, GameButton::Down, CITY, (x, 38));
        press(&mut c, GameButton::Down);
        at(&mut c, CITY, (x, 38));
        checkpoint(&mut c, &format!("clear-pier-lane-{x}"));
        travel(&mut c, GameButton::Up, CITY, (x, 34));
    }
    indoor_walk(&mut c, &r, (23, 32));
    let state = c.shell.shell.session().state().clone();
    press(&mut c, GameButton::Up);
    at(&mut c, CITY, (23, 32));
    press(&mut c, GameButton::A);
    super::northstar_ice_lair::choice(&mut c, false);
    finish(&mut c);
    assert!(
        c.snapshot()
            .expect("declined optional fisher")
            .battle
            .is_none()
    );
    assert_eq!(state.flags, c.shell.shell.session().state().flags);
    assert_eq!(
        state.storage.party,
        c.shell.shell.session().state().storage.party
    );
    assert_eq!(state.bag, c.shell.shell.session().state().bag);
    assert_eq!(state.money, c.shell.shell.session().state().money);
    continued(&mut c, &root, &r, "angler-off-the-pier");
    indoor_walk(&mut c, &r, (6, 24));
    assert_eq!(c.shell.shell.session().state().badges.johto, [false; 8]);
    press(&mut c, GameButton::Start);
    assert!(c.snapshot().expect("production Start").ui.menu.is_some());
    press(&mut c, GameButton::B);
}
