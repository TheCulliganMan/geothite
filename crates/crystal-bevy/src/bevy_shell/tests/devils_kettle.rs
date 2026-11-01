//! Declared prior chapter state; all travel and encounter actions use input.
#[path = "devils_kettle_recovery.rs"]
mod recovery;
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, PcBox};
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life::fixture;
use super::northstar_ice_lair::{choice, event, walk};
use super::r10_gallery::{continue_save, settle};
use super::winona::{at, load, press, travel};

const MAP: &str = "MnDevilsKettle";
const OUTSIDE: &str = "MnMaraisCedarApproach";
const ACTOR: &str = "MN_DEVILS_KETTLURK";
const CAUGHT: &str = "EVENT_MN_DEVILS_KETTLE_CAUGHT";

enum FixtureParty {
    Surveyor,
    Full,
    Counter,
    Weak,
}

fn setup(released: bool, restored: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    setup_storage(released, restored, false)
}

fn setup_storage(
    released: bool,
    restored: bool,
    full_party: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    setup_party(
        released,
        restored,
        if full_party {
            FixtureParty::Full
        } else {
            FixtureParty::Surveyor
        },
    )
}

fn setup_party(
    released: bool,
    restored: bool,
    party: FixtureParty,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, OUTSIDE, (14, 9));
    {
        let state = c.shell.shell.session_mut().state_mut();
        for (flag, value) in [
            ("EVENT_MN_KETTLURK_RELEASED", released),
            ("EVENT_MN_SURVEY_RESTORED", restored),
        ] {
            state
                .flags
                .set_event_flag(flag, value)
                .expect("declared prior chapter");
        }
        let partners: &[(&str, u8)] = match party {
            FixtureParty::Surveyor | FixtureParty::Full => &[("POUCHID", 60)],
            FixtureParty::Counter => &[("BEETYNAMO", 60), ("TILTAWISP", 60)],
            FixtureParty::Weak => &[("CINDORE", 1)],
        };
        for (slot, &(species, level)) in partners.iter().enumerate() {
            let mut partner = runtime
                .data()
                .create_pokemon(species, level, Dv::default())
                .expect("declared legal native partner");
            partner.item = Some("SMOKE_BALL".into());
            state.storage.party.pokemon[slot] = Some(partner);
        }
        if matches!(party, FixtureParty::Weak) {
            state.last_spawn_map_constant = Some("MN_GRAND_MARAIS_POKECENTER".into());
            // Restored-survey access already implies the late-game coastal
            // prerequisites. Declare them before input, including the return
            // through Grand Marais's actual north guard after a blackout.
            for (index, flag) in [
                "ENGINE_ZEPHYRBADGE",
                "ENGINE_HIVEBADGE",
                "ENGINE_PLAINBADGE",
                "ENGINE_FOGBADGE",
                "ENGINE_STORMBADGE",
                "ENGINE_MINERALBADGE",
                "ENGINE_GLACIERBADGE",
                "ENGINE_RISINGBADGE",
            ]
            .iter()
            .enumerate()
            {
                state.badges.johto[index] = true;
                state
                    .flags
                    .set_engine_flag(flag, true)
                    .expect("declared prior eight-badge access");
            }
            state
                .flags
                .set_event_flag("EVENT_MN_LIGHTHOUSE_SIGNAL", true)
                .expect("declared prior lighthouse signal");
        }
        if matches!(party, FixtureParty::Full) {
            for slot in 1..6 {
                state.storage.party.pokemon[slot] = Some(
                    runtime
                        .data()
                        .create_pokemon("POUCHID", 60, Dv::default())
                        .expect("declared full native party"),
                );
            }
            state.current_pc_box = 3;
            while state.storage.pc_boxes.len() <= state.current_pc_box {
                state
                    .storage
                    .pc_boxes
                    .push(PcBox::new(state.storage.pc_boxes.len()));
            }
        }
        state.sync_party_from_storage();
        if matches!(party, FixtureParty::Surveyor | FixtureParty::Full) {
            assert!(
                state
                    .bag
                    .add_item(&runtime.data().items["MASTER_BALL"], 1)
                    .expect("declared capture item")
            );
        }
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    (root, runtime, c)
}

fn enter(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    walk(c, runtime, (14, 8));
    travel(c, GameButton::Left, MAP, (30, 32));
    // Ascend past the exit landing before planning the walk into the valley.
    // A floor-only shortest path otherwise walks left over that active exit.
    travel(c, GameButton::Up, MAP, (30, 30));
    walk(c, runtime, (21, 12));
}

fn visible(c: &mut VisibleShellController) -> bool {
    c.snapshot()
        .expect("falls presentation")
        .visible_objects
        .iter()
        .any(|o| o.object_identifier.as_deref() == Some(ACTOR))
}

fn leave(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    walk(c, runtime, (30, 31));
    travel(c, GameButton::Down, MAP, (30, 32));
    travel(c, GameButton::Left, OUTSIDE, (14, 9));
}

fn ask(c: &mut VisibleShellController, yes: bool) {
    press(c, GameButton::Left);
    press(c, GameButton::A);
    choice(c, yes);
}

fn battle(c: &mut VisibleShellController) {
    ask(c, true);
    for _ in 0..128 {
        if let Some(battle) = c.snapshot().expect("native battle transition").battle {
            assert_eq!(battle.enemy_pokemon.species.id, "KETTLURK");
            assert_eq!(battle.enemy_pokemon.level, 60);
            return;
        }
        c.wait_frames(64).expect("battle transition");
    }
    panic!("native Kettlurk battle did not begin");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Devil's Kettle branch"]
fn devils_kettle_controller_stairs_both_story_requirements_and_continue() {
    for (released, restored) in [(false, false), (true, false), (false, true), (true, true)] {
        let (root, runtime, mut c) = setup(released, restored);
        // No state, party, bag, position, flags or RNG writes after setup.
        enter(&mut c, &runtime);
        assert_eq!(visible(&mut c), released && restored);
        assert!(!event(&c, CAUGHT));
        c = continue_save(root, runtime.clone(), &mut c);
        assert_eq!(visible(&mut c), released && restored);
        leave(&mut c, &runtime);
        at(&mut c, OUTSIDE, (14, 9));
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Devil's Kettle branch"]
fn devils_kettle_controller_refusal_run_hidden_continue_and_physical_retry() {
    let (root, runtime, mut c) = setup(true, true);
    enter(&mut c, &runtime);
    let bag = c.shell.shell.session().state().bag.clone();
    ask(&mut c, false);
    settle(&mut c, "refused optional approach");
    assert!(visible(&mut c));
    assert_eq!(c.shell.shell.session().state().bag, bag);
    battle(&mut c);
    super::rochester::flee(&mut c);
    settle(&mut c, "native RUN retreat");
    assert!(!event(&c, CAUGHT) && !visible(&mut c));
    c = continue_save(root, runtime.clone(), &mut c);
    assert!(!visible(&mut c));
    leave(&mut c, &runtime);
    enter(&mut c, &runtime);
    assert!(visible(&mut c));
    battle(&mut c);
    super::rochester::flee(&mut c);
    settle(&mut c, "native retry complete");
    assert!(!event(&c, CAUGHT));
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Devil's Kettle branch"]
fn devils_kettle_controller_capture_dex_continue_and_permanent_removal() {
    let (root, runtime, mut c) = setup(true, true);
    enter(&mut c, &runtime);
    let path = super::authored_route_fixture::reached_save_path(
        "devils-kettle-reached-before-capture.crystalsave",
    );
    c.save(&path).expect("actual walked falls checkpoint");
    super::authored_route_fixture::cleanup_reached_save(&path);
    battle(&mut c);
    capture(&mut c);
    assert!(event(&c, CAUGHT) && !visible(&mut c));
    let state = c.shell.shell.session().state();
    assert!(state.pokedex.has_caught("KETTLURK"));
    assert_eq!(state.bag.quantity(&runtime.data().items["MASTER_BALL"]), 0);
    assert!(
        state
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .any(|p| p.species.id == "KETTLURK")
    );
    c = continue_save(root, runtime.clone(), &mut c);
    assert!(event(&c, CAUGHT) && !visible(&mut c));
    leave(&mut c, &runtime);
    enter(&mut c, &runtime);
    assert!(event(&c, CAUGHT) && !visible(&mut c));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .filter(|p| p.species.id == "KETTLURK")
            .count(),
        1
    );
}

fn capture(c: &mut VisibleShellController) {
    for _ in 0..256 {
        let snapshot = c.snapshot().expect("native capture menus");
        if event(c, CAUGHT) && snapshot.battle.is_none() {
            settle(c, "capture continuation");
            return;
        }
        if let Some(choice) = &c.shell.pending_name_choice {
            if choice.nickname_pages.len() > 1 {
                press(c, GameButton::A);
            } else {
                press(c, GameButton::Down);
                press(c, GameButton::A);
            }
            continue;
        }
        let button = if c.shell.pokedex_menu_open
            || (snapshot.ui.menu.is_none() && snapshot.ui.text.is_some())
            || c.shell.field_pack_action_cursor.is_some()
            || c.shell.ball_cursor.is_some()
        {
            GameButton::A
        } else if c.shell.bag_cursor.is_some() {
            GameButton::Right
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            match cursor.option_index {
                0 | 1 => GameButton::Down,
                3 => GameButton::Left,
                _ => GameButton::A,
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    panic!("native Kettlurk capture did not finish");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Devil's Kettle branch"]
fn devils_kettle_controller_full_party_selected_pc_capture_and_reentry() {
    let (root, runtime, mut c) = setup_storage(true, true, true);
    // Six native partners, the fourth PC box and one ball are declared setup.
    // No storage, party, bag, position, flag, battle or RNG writes below.
    let before = c.shell.shell.session().state().storage.clone();
    enter(&mut c, &runtime);
    battle(&mut c);
    capture(&mut c);
    assert!(event(&c, CAUGHT) && !visible(&mut c));
    let state = c.shell.shell.session().state();
    assert_eq!(state.current_pc_box, 3);
    assert_eq!(state.storage.party, before.party);
    for (index, original) in before.pc_boxes.iter().enumerate() {
        if index != 3 {
            assert_eq!(&state.storage.pc_boxes[index], original);
        }
    }
    let boxed = state.storage.pc_boxes[3].pokemon[0]
        .as_ref()
        .expect("captured in selected PC box");
    assert_eq!(boxed.species.id, "KETTLURK");
    assert_eq!(
        state.storage.pc_boxes[3].pokemon.iter().flatten().count(),
        1
    );
    assert_eq!(state.bag.quantity(&runtime.data().items["MASTER_BALL"]), 0);
    assert!(state.pokedex.has_caught("KETTLURK"));
    let captured = state.storage.clone();
    c = continue_save(root, runtime.clone(), &mut c);
    assert!(event(&c, CAUGHT) && !visible(&mut c));
    assert_eq!(c.shell.shell.session().state().storage, captured);
    leave(&mut c, &runtime);
    enter(&mut c, &runtime);
    assert!(event(&c, CAUGHT) && !visible(&mut c));
    assert_eq!(c.shell.shell.session().state().storage, captured);
}
