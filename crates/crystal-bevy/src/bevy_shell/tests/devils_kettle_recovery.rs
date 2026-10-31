//! Legal local battle fixtures; outcomes and recovery use production input.
use crystal_core::input::GameButton;

use super::super::minnesota_gyms::northstar_campaign_finale::walk_cedar;
use super::super::northstar_ice_lair::{event, walk};
use super::super::r10_gallery::{continue_save, settle};
use super::super::winona::{press, travel};
use super::{CAUGHT, FixtureParty, battle, enter, leave, setup_party, visible};

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Devil's Kettle branch"]
fn devils_kettle_controller_knockout_hidden_continue_and_actual_retry() {
    let (root, runtime, mut c) = setup_party(true, true, FixtureParty::Counter);
    // Two level60 native counters are setup; no state or RNG writes below.
    enter(&mut c, &runtime);
    battle(&mut c);
    assert!(super::super::rochester_earned::try_battle(
        &mut c, &runtime, "WILD", ""
    ));
    assert_eq!(c.shell.shell.session().state().battle_result & 0x3f, 0);
    settle(&mut c, "real Kettlurk knockout");
    assert!(!event(&c, CAUGHT) && !visible(&mut c));
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .pokedex
            .has_caught("KETTLURK")
    );
    c = continue_save(root, runtime.clone(), &mut c);
    assert!(!event(&c, CAUGHT) && !visible(&mut c));
    leave(&mut c, &runtime);
    enter(&mut c, &runtime);
    assert!(visible(&mut c));
    battle(&mut c);
    super::super::rochester::flee(&mut c);
    settle(&mut c, "actual uncaught retry");
    assert!(!event(&c, CAUGHT));
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Devil's Kettle branch"]
fn devils_kettle_controller_blackout_prior_clinic_healing_and_continue() {
    let (root, runtime, mut c) = setup_party(true, true, FixtureParty::Weak);
    // Level1 Cindore, prior clinic, eight badges and the lighthouse signal
    // are declared setup. The complete recovery return below uses real gates.
    // No state, HP, party, flags, position or RNG writes after setup.
    enter(&mut c, &runtime);
    battle(&mut c);
    assert!(!super::super::rochester_earned::try_battle(
        &mut c, &runtime, "WILD", ""
    ));
    assert_eq!(c.shell.shell.session().state().battle_result & 0x3f, 1);
    settle(&mut c, "native blackout recovery");
    assert!(!event(&c, CAUGHT));
    assert!(event(&c, "EVENT_MN_SURVEY_RESTORED") && event(&c, "EVENT_MN_KETTLURK_RELEASED"));
    let snapshot = c.snapshot().expect("actual prior-clinic recovery");
    assert_eq!(snapshot.overworld.map_name, "MnGrandMaraisPokecenter");
    assert!(snapshot.battle.is_none());
    assert!(
        snapshot
            .party
            .slots
            .iter()
            .all(|slot| slot.pokemon.hp == slot.pokemon.max_hp)
    );
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .pokedex
            .has_caught("KETTLURK")
    );
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    assert!(!event(&c, CAUGHT));
    assert_eq!(
        c.snapshot().expect("recovery Continue").overworld.map_name,
        "MnGrandMaraisPokecenter"
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
    let money = c.shell.shell.session().state().money;
    for _ in 0..8 {
        if c.snapshot().expect("clinic exit").overworld.map_name == "MnGrandMarais" {
            break;
        }
        press(&mut c, GameButton::Down);
    }
    assert_eq!(
        c.snapshot()
            .expect("physical clinic exit")
            .overworld
            .map_name,
        "MnGrandMarais"
    );
    walk(&mut c, &runtime, (18, 4));
    travel(&mut c, GameButton::Up, "MnMaraisCedarApproach", (26, 23));
    walk_cedar(&mut c, &runtime, (25, 14));
    travel(&mut c, GameButton::Left, "MnGrandMaraisShoreGate", (8, 4));
    walk(&mut c, &runtime, (1, 4));
    travel(&mut c, GameButton::Left, "MnMaraisCedarApproach", (18, 14));
    walk_cedar(&mut c, &runtime, (14, 9));
    c = continue_save(root, runtime.clone(), &mut c);
    enter(&mut c, &runtime);
    assert!(
        visible(&mut c),
        "uncaught Kettlurk returns after real clinic journey"
    );
    battle(&mut c);
    super::super::rochester::flee(&mut c);
    settle(&mut c, "actual post-blackout return and retry");
    assert!(!event(&c, CAUGHT));
    assert_eq!(c.shell.shell.session().state().money, money);
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with Grand Marais clinic recovery"]
fn devils_kettle_controller_actual_nurse_registers_recovery_and_heals() {
    use super::super::super::mark_runtime_snapshot_dirty;
    use super::super::interior_life::fixture;
    use super::super::northstar_ice_lair::choice;
    use super::super::winona::load;
    use crystal_core::models::Dv;

    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, "MnGrandMaraisPokecenter", (3, 3));
    {
        let state = c.shell.shell.session_mut().state_mut();
        let mut partner = runtime
            .data()
            .create_pokemon("CINDORE", 5, Dv::default())
            .expect("declared native clinic patient");
        partner.hp -= 1;
        partner.moves[0].current_pp = 0;
        state.storage.party.pokemon[0] = Some(partner);
        state.sync_party_from_storage();
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    // Healing and registration below are earned through the nurse dialogue.
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    choice(&mut c, true);
    settle(&mut c, "actual native nurse care");
    let state = c.shell.shell.session().state();
    assert_eq!(
        state.last_spawn_map_constant.as_deref(),
        Some("MN_GRAND_MARAIS_POKECENTER")
    );
    let partner = state.storage.party.pokemon[0]
        .as_ref()
        .expect("healed native partner");
    assert_eq!(partner.hp, partner.max_hp);
    assert_eq!(
        partner.moves[0].current_pp,
        runtime.data().moves[&partner.moves[0].name].pp
    );
    c = continue_save(root, runtime, &mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .last_spawn_map_constant
            .as_deref(),
        Some("MN_GRAND_MARAIS_POKECENTER")
    );
}
