//! Declared cleared chapter; knockout and blackout use real battle input.
#[path = "verdigrim_return_trip.rs"]
mod return_trip;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::CrystalRuntime;

use super::super::interior_life::fixture;
use super::super::northstar_ice_lair::{choice, event, walk};
use super::super::r10_gallery::{continue_save, settle};
use super::super::winona::{load, press, travel};
use super::{CONTAINED, RESTORED, VisibleShellController, mark_runtime_snapshot_dirty};

const CAUGHT: &str = "EVENT_MN_VERDIGRIM_CAUGHT";
const WOLF: &str = "MN_VERDIGRIM_INTAKE";

fn setup(
    species: &str,
    level: u8,
    fly_return: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, "MnBoundaryIntake", (4, 18));
    {
        let state = c.shell.shell.session_mut().state_mut();
        for &flag in super::CLEARED {
            state
                .flags
                .set_event_flag(flag, true)
                .expect("declared cleared chapter");
        }
        let mut partner = runtime
            .data()
            .create_pokemon(species, level, Dv::default())
            .expect("declared native recovery partner");
        partner.item = Some("SMOKE_BALL".into());
        state.storage.party.pokemon[0] = Some(partner);
        if fly_return {
            let mut bird = runtime
                .data()
                .create_pokemon("LOOBIT", 1, Dv::default())
                .expect("declared native prior Fly user");
            assert!(bird.species.tmhm_learnset.iter().any(|name| name == "FLY"));
            bird.moves.truncate(3);
            bird.moves.push(LearnedMove {
                name: "FLY".into(),
                current_pp: runtime.data().moves["FLY"].pp,
                pp_ups: 0,
            });
            bird.item = Some("SMOKE_BALL".into());
            state.storage.party.pokemon[1] = Some(bird);
            for (i, &flag) in crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS
                .iter()
                .take(8)
                .enumerate()
            {
                state.badges.johto[i] = true;
                state
                    .flags
                    .set_engine_flag(flag, true)
                    .expect("declared prior badges");
            }
            state
                .flags
                .set_engine_flag("ENGINE_FLYPOINT_MN_BOUNDARY_WATERS", true)
                .expect("declared earlier camp visit");
        } else {
            let mut teammate = runtime
                .data()
                .create_pokemon("FURROWBANK", 60, Dv::default())
                .expect("declared legal Ground-type teammate");
            teammate.item = Some("SMOKE_BALL".into());
            state.storage.party.pokemon[1] = Some(teammate);
        }
        state.sync_party_from_storage();
        state.last_spawn_map_constant = Some("MN_ROCHESTER_POKECENTER_1F".into());
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert!(
        c.snapshot()
            .expect("no capture supplies")
            .bag
            .balls
            .iter()
            .all(|b| b.quantity == 0)
    );
    (root, runtime, c)
}

fn visit(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    walk(c, runtime, (4, 18));
    travel(
        c,
        GameButton::Right,
        "MnBoundaryWatersSurveyShelter",
        (12, 7),
    );
    walk(c, runtime, (13, 7));
    travel(c, GameButton::Down, "MnBoundaryIntake", (4, 19));
    walk(c, runtime, (24, 6));
    assert!(
        c.snapshot()
            .expect("uncaught encounter on new visit")
            .visible_objects
            .iter()
            .any(|o| o.object_identifier.as_deref() == Some(WOLF))
    );
}

fn encounter(c: &mut VisibleShellController) {
    press(c, GameButton::Right);
    press(c, GameButton::A);
    choice(c, true);
    for _ in 0..128 {
        if c.snapshot().expect("optional encounter").battle.is_some() {
            break;
        }
        c.wait_frames(64).expect("native battle transition");
    }
    let battle = c
        .snapshot()
        .expect("actual encounter")
        .battle
        .expect("wild battle");
    assert_eq!(battle.enemy_pokemon.species.id, "VERDIGRIM");
    assert_eq!(battle.enemy_pokemon.level, 60);
}

fn uncaught(c: &VisibleShellController) {
    assert!(!event(c, CAUGHT));
    assert!(event(c, CONTAINED) && event(c, RESTORED));
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .pokedex
            .has_caught("VERDIGRIM")
    );
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Verdigrim intake scene"]
fn verdigrim_controller_knockout_retreat_continue_and_actual_retry() {
    let (root, runtime, mut c) = setup("BOREALYNX", 60, false);
    // Two legal level60 partners are a fixture, not earned progression.
    // No state, party, bag, flags, position, battle or RNG writes below.
    visit(&mut c, &runtime);
    encounter(&mut c);
    assert!(super::super::rochester_earned::try_battle(
        &mut c, &runtime, "WILD", ""
    ));
    assert_eq!(c.shell.shell.session().state().battle_result & 0x3f, 0);
    settle(&mut c, "actual knockout retreat");
    uncaught(&c);
    assert!(
        c.snapshot()
            .expect("KO hides current visit")
            .visible_objects
            .iter()
            .all(|o| o.object_identifier.as_deref() != Some(WOLF))
    );
    c = continue_save(root, runtime.clone(), &mut c);
    uncaught(&c);
    assert!(
        c.snapshot()
            .expect("continued KO visit")
            .visible_objects
            .iter()
            .all(|o| o.object_identifier.as_deref() != Some(WOLF))
    );
    visit(&mut c, &runtime);
    encounter(&mut c);
    super::super::rochester::flee(&mut c);
    settle(&mut c, "real retry returned to field");
    uncaught(&c);
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Verdigrim intake scene"]
fn verdigrim_controller_blackout_preserves_containment_and_uncaught_continue() {
    let (root, runtime, mut c) = setup("POUCHID", 1, true);
    // Two weak legal partners, prior clinic, badges and learned Fly are setup.
    // No state, party, bag, flags, position, battle or RNG writes below.
    visit(&mut c, &runtime);
    encounter(&mut c);
    assert!(!super::super::rochester_earned::try_battle(
        &mut c, &runtime, "WILD", ""
    ));
    assert_eq!(c.shell.shell.session().state().battle_result & 0x3f, 1);
    settle(&mut c, "native blackout recovery");
    uncaught(&c);
    let snapshot = c.snapshot().expect("recovered at prior clinic");
    assert_eq!(snapshot.overworld.map_name, "MnRochesterPokecenter1F");
    assert!(snapshot.battle.is_none());
    assert!(
        snapshot
            .party
            .slots
            .iter()
            .all(|slot| slot.pokemon.hp == slot.pokemon.max_hp)
    );
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    uncaught(&c);
    assert_eq!(
        c.snapshot().expect("continued recovery").overworld.map_name,
        "MnRochesterPokecenter1F"
    );
    press(&mut c, GameButton::Start);
    assert!(
        c.snapshot()
            .expect("input after recovery")
            .ui
            .menu
            .is_some()
    );
    press(&mut c, GameButton::B);
    return_trip::to_intake(&mut c, &runtime);
    uncaught(&c);
    encounter(&mut c);
    super::super::rochester::flee(&mut c);
    settle(&mut c, "native retry after blackout and return journey");
    uncaught(&c);
    c = continue_save(root, runtime, &mut c);
    uncaught(&c);
}
