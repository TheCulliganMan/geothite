//! Declared chapter access; exposure and containment use production input.
#[path = "verdigrim_encounter.rs"]
mod encounter;
#[path = "verdigrim_recovery.rs"]
mod recovery;
#[path = "verdigrim_water_animation.rs"]
mod water_animation;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life::fixture;
use super::minnesota_fossils::talk;
use super::northstar_ice_lair::{event, page, walk};
use super::r10_gallery::{continue_save, settle};
use super::winona::{at, load, press, travel};

const EXPOSED: &str = "EVENT_MN_VERDIGRIM_EXPOSED";
const GREEN: &str = "EVENT_MN_VERDIGRIM_GREEN_WATER";
const COUNTER: &str = "EVENT_MN_VERDIGRIM_COUNTER_CURRENT";
const DRILL: &str = "EVENT_MN_VERDIGRIM_DRILL_STOPPED";
const OUTLET: &str = "EVENT_MN_VERDIGRIM_OUTLET_CLOSED";
const BYPASS: &str = "EVENT_MN_VERDIGRIM_CLEAN_BYPASS";
const CONTAINED: &str = "EVENT_MN_VERDIGRIM_CONTAINED";
const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";
const CLEARED: &[&str] = &[
    EXPOSED,
    GREEN,
    COUNTER,
    DRILL,
    OUTLET,
    BYPASS,
    CONTAINED,
    RESTORED,
    "EVENT_MN_VERDIGRIM_FOREMAN_FLED",
    "EVENT_MN_VERDIGRIM_PINNED",
    "EVENT_MN_VERDIGRIM_RETREATED",
    "EVENT_MN_ARC_LINK_SENT",
    "EVENT_MN_REED_BOUNDARY_DEFEATED",
    "EVENT_MN_INTAKE_FIRST_SEEN",
    "EVENT_MN_KETTLURK_RELEASED",
];

fn setup(won: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, "MnBoundaryIntake", (18, 18));
    {
        let state = c.shell.shell.session_mut().state_mut();
        state
            .flags
            .set_event_flag("EVENT_MN_ARC_LINK_SENT", true)
            .expect("declared prior public link");
        state
            .flags
            .set_event_flag("EVENT_MN_REED_BOUNDARY_DEFEATED", won)
            .expect("declared rival result");
        let mut partner = runtime
            .data()
            .create_pokemon("POUCHID", 60, Dv::default())
            .expect("legal native scene partner");
        partner.item = Some("SMOKE_BALL".into());
        state.storage.party.pokemon[0] = Some(partner);
        state.sync_party_from_storage();
        assert!(
            state
                .bag
                .add_item(&runtime.data().items["MASTER_BALL"], 1)
                .expect("declared optional-encounter capture item")
        );
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    (root, runtime, c)
}

fn enter(c: &mut VisibleShellController) {
    for _ in 0..8 {
        press(c, GameButton::Up);
        if c.snapshot()
            .expect("native intake arrival")
            .ui
            .text_window_open
        {
            return;
        }
    }
}

fn water(c: &VisibleShellController) -> Vec<u16> {
    let map = &c.shell.shell.session().overworld().map;
    (10..15).map(|x| map.metatile_ids[6 * 16 + x]).collect()
}

fn held_continue(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    c: &mut VisibleShellController,
    name: &str,
) -> VisibleShellController {
    let path = super::authored_route_fixture::reached_save_path(name);
    let before = c.snapshot().expect("held dialogue");
    let blocks = c.shell.shell.session().overworld().map.metatile_ids.clone();
    c.save(&path).expect("native mid-dialogue autosave");
    let mut after =
        VisibleShellController::load_save(root.clone(), runtime.clone(), path.clone(), None)
            .expect("same-pack held Continue");
    let snapshot = after.snapshot().expect("continued dialogue");
    assert_eq!(
        before.ui.text.as_ref().map(|t| &t.label),
        snapshot.ui.text.as_ref().map(|t| &t.label)
    );
    assert_eq!(before.overworld.tile, snapshot.overworld.tile);
    assert_eq!(
        blocks,
        after.shell.shell.session().overworld().map.metatile_ids
    );
    super::authored_route_fixture::cleanup_reached_save(&path);
    after
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Verdigrim intake scene"]
fn verdigrim_controller_exposure_cry_green_water_dialogue_and_continue() {
    let (root, runtime, mut c) = setup(false);
    // No state/position/party/inventory/RNG writes after declared setup.
    enter(&mut c);
    page(&mut c, "MnVerdigrimExposedText");
    assert!(event(&c, EXPOSED));
    assert!(!event(&c, GREEN) && !event(&c, COUNTER) && !event(&c, RESTORED));
    assert!(
        c.shell
            .last_audio_events
            .iter()
            .any(|event| event.contains("CRY_MN_319"))
    );
    let position = c.snapshot().expect("exposure").overworld.tile;
    press(&mut c, GameButton::Down);
    assert_eq!(
        c.snapshot().expect("text input owner").overworld.tile,
        position
    );
    page(&mut c, "MnVerdigrimGreenText");
    assert!(event(&c, GREEN));
    let green = water(&c);
    assert!(green.iter().all(|id| *id == green[0] && *id != 0x40));
    assert_eq!(
        runtime.data().tilesets["mn_verdigrim_intake"].collision[&format!("{:02x}", green[0])],
        ["WATER"; 4]
    );
    c = held_continue(&root, &runtime, &mut c, "verdigrim-held-green.crystalsave");
    assert_eq!(water(&c), green);
    settle(&mut c, "acknowledged exposure");
    at(&mut c, "MnBoundaryIntake", (19, 11));
    assert!(event(&c, COUNTER));
    assert!(!event(&c, DRILL) && !event(&c, CONTAINED) && !event(&c, RESTORED));
    walk(&mut c, &runtime, (17, 12));
    talk(&mut c, GameButton::Up, &[]);
    assert!(
        !event(&c, DRILL),
        "unbeaten rival must retain control guard"
    );
    c = continue_save(root, runtime, &mut c);
    assert_eq!(water(&c), green);
    assert!(!event(&c, RESTORED));
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Verdigrim intake scene"]
fn verdigrim_controller_three_controls_refusal_order_and_acknowledged_crew() {
    let (root, runtime, mut c) = setup(true);
    enter(&mut c);
    settle(&mut c, "complete exposure");
    let green = water(&c)[0];
    walk(&mut c, &runtime, (21, 20));
    talk(&mut c, GameButton::Up, &[]);
    assert!(!event(&c, OUTLET));
    walk(&mut c, &runtime, (17, 12));
    talk(&mut c, GameButton::Up, &[false]);
    assert!(!event(&c, DRILL));
    talk(&mut c, GameButton::Up, &[true]);
    assert!(event(&c, DRILL));
    assert!(!event(&c, OUTLET) && !event(&c, BYPASS) && !event(&c, RESTORED));
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    walk(&mut c, &runtime, (13, 12));
    talk(&mut c, GameButton::Up, &[]);
    assert!(!event(&c, BYPASS));
    walk(&mut c, &runtime, (21, 20));
    talk(&mut c, GameButton::Up, &[false]);
    assert!(!event(&c, OUTLET));
    talk(&mut c, GameButton::Up, &[true]);
    assert!(event(&c, OUTLET));
    assert_eq!(water(&c), vec![green; 5]);
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    walk(&mut c, &runtime, (13, 12));
    talk(&mut c, GameButton::Up, &[false]);
    assert!(!event(&c, BYPASS));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    super::northstar_ice_lair::choice(&mut c, true);
    page(&mut c, "MnVerdigrimBypassDoneText");
    assert!(event(&c, BYPASS));
    assert!(!event(&c, CONTAINED) && !event(&c, RESTORED));
    assert_eq!(water(&c), vec![0x40, 0x40, 0x40, green, green]);
    c = held_continue(&root, &runtime, &mut c, "verdigrim-held-bypass.crystalsave");
    page(&mut c, "MnArcIntakeDoneText");
    assert!(
        !event(&c, CONTAINED) && !event(&c, RESTORED),
        "crew text must be acknowledged"
    );
    settle(&mut c, "crew clearance");
    assert!(event(&c, CONTAINED) && event(&c, RESTORED));
    assert!(!event(&c, "EVENT_MN_VERDIGRIM_CAUGHT"));
    assert_eq!(water(&c), vec![0x40, 0x40, 0x40, green, green]);
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    assert!(event(&c, CONTAINED) && event(&c, RESTORED));
    assert_eq!(water(&c), vec![0x40, 0x40, 0x40, green, green]);
    walk(&mut c, &runtime, (24, 6));
    talk(&mut c, GameButton::Right, &[false]);
    assert!(!event(&c, "EVENT_MN_VERDIGRIM_CAUGHT"));
    assert!(c.snapshot().expect("declined encounter").battle.is_none());
    press(&mut c, GameButton::A);
    super::northstar_ice_lair::choice(&mut c, true);
    for _ in 0..128 {
        if c.snapshot().expect("optional encounter").battle.is_some() {
            break;
        }
        c.wait_frames(64).expect("native encounter transition");
    }
    super::rochester::flee(&mut c);
    settle(&mut c, "uncaught Verdigrim withdraws");
    assert!(!event(&c, "EVENT_MN_VERDIGRIM_CAUGHT"));
    assert!(event(&c, CONTAINED) && event(&c, RESTORED));
    assert!(
        c.snapshot()
            .expect("RUN hides this visit")
            .visible_objects
            .iter()
            .all(|object| object.object_identifier.as_deref() != Some("MN_VERDIGRIM_INTAKE"))
    );
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    assert!(
        c.snapshot()
            .expect("Continue preserves this visit")
            .visible_objects
            .iter()
            .all(|object| object.object_identifier.as_deref() != Some("MN_VERDIGRIM_INTAKE"))
    );
    walk(&mut c, &runtime, (4, 18));
    travel(
        &mut c,
        GameButton::Right,
        "MnBoundaryWatersSurveyShelter",
        (12, 7),
    );
    walk(&mut c, &runtime, (13, 7));
    travel(&mut c, GameButton::Down, "MnBoundaryIntake", (4, 19));
    assert!(event(&c, CONTAINED) && event(&c, RESTORED));
    assert_eq!(water(&c), vec![0x40, 0x40, 0x40, green, green]);
    assert!(
        c.snapshot()
            .expect("uncaught creature returns on reentry")
            .visible_objects
            .iter()
            .any(|object| object.object_identifier.as_deref() == Some("MN_VERDIGRIM_INTAKE"))
    );
    walk(&mut c, &runtime, (24, 6));
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    super::northstar_ice_lair::choice(&mut c, true);
    for _ in 0..128 {
        if c.snapshot().expect("returned encounter").battle.is_some() {
            break;
        }
        c.wait_frames(64).expect("native encounter transition");
    }
    assert!(super::rochester_earned::try_recruit_with_ball(
        &mut c,
        &runtime,
        "MASTER_BALL"
    ));
    settle(&mut c, "actual Verdigrim capture");
    assert!(event(&c, "EVENT_MN_VERDIGRIM_CAUGHT"));
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .pokedex
            .has_caught("VERDIGRIM")
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["MASTER_BALL"]),
        0
    );
    assert!(
        c.snapshot()
            .expect("captured party")
            .party
            .slots
            .iter()
            .any(|slot| slot.pokemon.species.id == "VERDIGRIM")
    );
    c = continue_save(root, runtime, &mut c);
    assert!(event(&c, "EVENT_MN_VERDIGRIM_CAUGHT"));
    assert!(
        c.snapshot()
            .expect("captured actor stays hidden")
            .visible_objects
            .iter()
            .all(|object| object.object_identifier.as_deref() != Some("MN_VERDIGRIM_INTAKE"))
    );
}
