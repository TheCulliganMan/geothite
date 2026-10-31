//! Declared local setup; all subsequent encounters and captures use native input.
#[path = "minnesota_ecology_replays.rs"]
mod replays;

use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::systems::time::GameDate;

use super::{checkpoint, verify_capture};
use crate::bevy_shell::tests::winona::press;

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the harvest zone and native Sweet Scent"]
fn ecology_native_sweet_scent_finds_harvest_zone_oatlet_and_continues() {
    let (root, runtime) = crate::bevy_shell::tests::winona::load();
    let mut controller =
        crate::bevy_shell::tests::interior_life::fixture(&root, &runtime, "MnR02C00R00", (40, 8));
    let data = &runtime.data().wild_encounters["MnR02C00R00"];
    assert!(
        !data
            .grass
            .as_ref()
            .expect("base grass")
            .day
            .iter()
            .any(|slot| slot.species == "OATLET"),
        "the base table cannot satisfy this zone regression"
    );
    assert!(
        data.zone_at(40, 7)
            .expect("harvest field")
            .grass
            .day
            .iter()
            .any(|slot| slot.species == "OATLET")
    );
    let mut partner = runtime
        .data()
        .create_pokemon("POUCHID", 10, Dv::default())
        .expect("natural Sweet Scent learner");
    partner.item = Some("SMOKE_BALL".into());
    let pp = partner
        .moves
        .iter()
        .find(|m| m.name == "SWEET_SCENT")
        .expect("learned at level 9")
        .current_pp;
    let state = controller.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    assert!(
        state
            .bag
            .add_item(&runtime.data().items["MASTER_BALL"], 1)
            .expect("declared capture supply")
    );
    state.time.update_server_datetime(
        GameDate {
            year: 2000,
            month: 1,
            day: 3,
        },
        12,
        0,
        0,
    );
    state
        .flags
        .set_engine_flag("STATUSFLAGS_NO_WILD_ENCOUNTERS_F", false)
        .expect("ordinary encounters");
    controller.set_runtime_journal_enabled(false);
    crate::bevy_shell::mark_runtime_snapshot_dirty(&mut controller.shell);
    // Setup ends here. No later position, party, flag, time, bag or RNG writes.
    for _ in 0..4 {
        if controller
            .snapshot()
            .expect("field approach")
            .overworld
            .tile
            .y
            == 7
        {
            break;
        }
        press(&mut controller, GameButton::Up);
    }
    crate::bevy_shell::tests::winona::at(&mut controller, "MnR02C00R00", (40, 7));
    if controller.snapshot().expect("grass entry").battle.is_some() {
        crate::bevy_shell::tests::rochester::flee(&mut controller);
    }
    checkpoint(&mut controller, "oatlet-scent-start");
    let mut caught = false;
    for _ in 0..32 {
        crate::bevy_shell::tests::minnesota_field_use::use_move(
            &mut controller,
            crate::bevy_shell::PartyFieldMove::SweetScent,
        );
        for _ in 0..256 {
            if controller
                .snapshot()
                .expect("native scent")
                .battle
                .is_some()
            {
                break;
            }
            press(&mut controller, GameButton::A);
        }
        let battle = controller
            .snapshot()
            .expect("native scent result")
            .battle
            .expect("a grass-zone encounter");
        let target = battle.enemy_pokemon.species.id == "OATLET";
        assert!(
            battle.commands.can_run,
            "Sweet Scent is an ordinary wild encounter"
        );
        if target {
            checkpoint(&mut controller, "oatlet-scent-battle");
            let before = controller.shell.shell.session().state().battle.clone();
            let path = super::super::authored_route_fixture::reached_save_path(&format!(
                "geothite-oatlet-scent-battle-{}.crystalsave",
                std::process::id()
            ));
            controller.save(&path).expect("native battle Save");
            controller = crate::bevy_shell::VisibleShellController::load_save(
                root.clone(),
                runtime.clone(),
                path.clone(),
                None,
            )
            .expect("same-pack Continue during Sweet Scent battle");
            controller.set_runtime_journal_enabled(false);
            assert_eq!(controller.shell.shell.session().state().battle, before);
            super::super::authored_route_fixture::cleanup_reached_save(&path);
            assert_eq!(
                controller
                    .snapshot()
                    .expect("continued scent battle")
                    .battle
                    .expect("retained wild battle")
                    .enemy_pokemon
                    .species
                    .id,
                "OATLET"
            );
            assert!(
                crate::bevy_shell::tests::rochester_earned::try_recruit_with_ball(
                    &mut controller,
                    &runtime,
                    "MASTER_BALL",
                )
            );
            caught = true;
        } else {
            let hp = controller
                .snapshot()
                .expect("before native escape")
                .party
                .slots[0]
                .pokemon
                .hp;
            crate::bevy_shell::tests::rochester::flee(&mut controller);
            assert_eq!(
                controller
                    .snapshot()
                    .expect("Smoke Ball escape")
                    .party
                    .slots[0]
                    .pokemon
                    .hp,
                hp
            );
        }
        crate::bevy_shell::tests::winona::at(&mut controller, "MnR02C00R00", (40, 7));
        let state = controller.shell.shell.session().state();
        assert_eq!(
            state.storage.party.pokemon[0]
                .as_ref()
                .expect("retained partner")
                .moves
                .iter()
                .find(|m| m.name == "SWEET_SCENT")
                .expect("retained move")
                .current_pp,
            pp
        );
        if caught {
            break;
        }
    }
    assert!(caught, "Sweet Scent must select the authored harvest zone");
    verify_capture(&controller, &runtime, "OATLET");
    controller = crate::bevy_shell::tests::r10_gallery::continue_save(
        root,
        runtime.clone(),
        &mut controller,
    );
    verify_capture(&controller, &runtime, "OATLET");
    checkpoint(&mut controller, "oatlet-scent-caught");
}
