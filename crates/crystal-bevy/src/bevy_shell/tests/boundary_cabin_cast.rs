//! Local unpaid hospitality fixtures; care and both returns use native input.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::authored_route_fixture::{cleanup_reached_save, reached_save_path};
use super::boundary_waters::assert_no_expedition;
use super::interior_life::fixture;
use super::northstar_ice_lair::walk;
use super::r10_gallery::{continue_save, settle};
use super::winona::{at, load, press, travel};

const ROOM: &str = "MnBoundaryRootBeerCabin";
const VOLUNTEER: &str = "MN_INTERIOR_CABIN_VOLUNTEER";
const EXTERIOR: &str = "BWS_ROOT_BEER_LADYRootBeerService";

fn setup() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, ROOM, (5, 4));
    {
        let state = c.shell.shell.session_mut().state_mut();
        let mut partner = runtime
            .data()
            .create_pokemon("POUCHID", 20, Dv::default())
            .expect("legal native cabin patient");
        partner.hp -= 3;
        partner
            .moves
            .first_mut()
            .expect("native learned move")
            .current_pp = 0;
        state.storage.party.pokemon[0] = Some(partner);
        state.sync_party_from_storage();
        state.money = 1000;
        assert!(
            state
                .bag
                .add_item(&runtime.data().items["ROOT_BEER"], 2)
                .expect("declared carried bottles")
        );
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64).expect("settled local fixture");
    // No paid admission, fee, party, position, progression or RNG edits below.
    (root, runtime, c)
}

fn cast(runtime: &CrystalRuntime) {
    let room = &runtime.data().maps[ROOM];
    assert_eq!(
        room.objects
            .iter()
            .map(|o| (o.script.as_str(), o.sprite.as_str(), o.x, o.y))
            .collect::<Vec<_>>(),
        vec![
            (VOLUNTEER, "SPRITE_TEACHER", 5, 3),
            ("MN_INTERIOR_CABIN_PADDLER", "SPRITE_FISHER", 7, 5),
        ]
    );
    assert!(
        room.scripts
            .keys()
            .all(|label| !label.starts_with("MN_INTERIOR_DOROTHY"))
    );
    let dorothy = runtime
        .data()
        .maps
        .values()
        .flat_map(|m| m.objects.iter().map(move |o| (m, o)))
        .filter(|(_, o)| o.script == EXTERIOR)
        .collect::<Vec<_>>();
    assert_eq!(dorothy.len(), 1);
    assert_ne!(dorothy[0].0.id, ROOM);
    assert_eq!(dorothy[0].1.sprite, "SPRITE_GRANNY");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the cabin volunteer revision"]
fn boundary_cabin_cast_controller_distinct_volunteer_care_held_continue_and_repeat() {
    let (root, runtime, mut c) = setup();
    cast(&runtime);
    assert_no_expedition(&c);
    let bag = c.shell.shell.session().state().bag.clone();
    let prior_spawn = c
        .shell
        .shell
        .session()
        .state()
        .last_spawn_map_constant
        .clone();
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    c.wait_frames(128).expect("first care page");
    let before = c.snapshot().expect("volunteer dialogue");
    assert_eq!(
        before.ui.text.as_ref().expect("native text").label,
        format!("{VOLUNTEER}InteriorText")
    );
    assert!(before.party.slots[0].pokemon.hp < before.party.slots[0].pokemon.max_hp);
    press(&mut c, GameButton::Down);
    assert_eq!(
        c.snapshot().expect("dialogue owns movement").overworld.tile,
        before.overworld.tile
    );
    let held = reached_save_path("boundary-cabin-volunteer-held.crystalsave");
    c.save(&held).expect("actual held dialogue save");
    c = VisibleShellController::load_save(root.clone(), runtime.clone(), held.clone(), None)
        .expect("same-pack held Continue");
    cleanup_reached_save(&held);
    assert_eq!(
        c.snapshot()
            .expect("held volunteer continuation")
            .ui
            .text
            .as_ref()
            .expect("text")
            .label,
        format!("{VOLUNTEER}InteriorText")
    );
    assert!(
        c.snapshot()
            .expect("care still waits for acknowledgment")
            .party
            .slots[0]
            .pokemon
            .hp
            < before.party.slots[0].pokemon.max_hp
    );
    settle(&mut c, "acknowledged volunteer care");
    let state = c.shell.shell.session().state();
    let partner = state.storage.party.pokemon[0]
        .as_ref()
        .expect("cared-for partner");
    assert_eq!(partner.hp, partner.max_hp);
    assert!(
        partner
            .moves
            .iter()
            .all(|m| m.current_pp == runtime.data().moves[&m.name].pp)
    );
    assert_eq!(state.money, 1000);
    assert_eq!(state.bag, bag);
    assert_eq!(state.last_spawn_map_constant, prior_spawn);
    assert_no_expedition(&c);
    let healed = reached_save_path("boundary-cabin-volunteer-healed.crystalsave");
    c.save(&healed).expect("actual completed care checkpoint");
    cleanup_reached_save(&healed);
    press(&mut c, GameButton::A);
    settle(&mut c, "repeat free care");
    assert_eq!(c.shell.shell.session().state().money, 1000);
    assert_eq!(c.shell.shell.session().state().bag, bag);
    assert_no_expedition(&c);
    c = continue_save(root, runtime, &mut c);
    assert_no_expedition(&c);
    assert_eq!(c.shell.shell.session().state().bag, bag);
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the cabin volunteer revision"]
fn boundary_cabin_cast_controller_two_returns_reentry_and_continue() {
    let (root, runtime, mut c) = setup();
    cast(&runtime);
    let room = &runtime.data().maps[ROOM];
    for lane in [5, 6] {
        let exit = room
            .events
            .warps
            .iter()
            .find(|w| w.x == lane && w.y == 7)
            .expect("paired cabin carpet");
        let outside_name = &runtime.data().runtime_map_metadata[&exit.target_map_constant].name;
        let outside = &runtime.data().maps[outside_name];
        let landing = outside
            .events
            .warps
            .iter()
            .find(|w| i16::try_from(w.index).expect("warp index") == exit.target_warp_id)
            .expect("passive exterior arrival");
        let point = (
            i16::try_from(landing.x).expect("return x"),
            i16::try_from(landing.y).expect("return y"),
        );
        walk(&mut c, &runtime, (i16::try_from(lane).expect("lane"), 6));
        travel(&mut c, GameButton::Down, outside_name, point);
        assert_no_expedition(&c);
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        at(&mut c, outside_name, point);
        let entry = outside
            .events
            .warps
            .iter()
            .find(|w| w.target_map_constant == "MN_BOUNDARY_ROOT_BEER_CABIN")
            .expect("real cabin door");
        let x = i16::try_from(entry.x).expect("door x");
        let y = i16::try_from(entry.y).expect("door y");
        walk(&mut c, &runtime, (x, y + 1));
        travel(&mut c, GameButton::Up, ROOM, (5, 6));
        cast(&runtime);
        assert_no_expedition(&c);
    }
}
