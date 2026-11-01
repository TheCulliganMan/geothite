//! Declared specimen/badge fixtures; real streets, doors, text and gifts use input.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;

use super::{event, quantity, talk};
use crate::bevy_shell::VisibleShellController;
use crate::bevy_shell::mark_runtime_snapshot_dirty;
use crate::bevy_shell::tests::authored_route_fixture::indoor_walk;
use crate::bevy_shell::tests::interior_life::{fixture, talk as read_text};
use crate::bevy_shell::tests::r10_gallery::continue_save;
use crate::bevy_shell::tests::winona::{at, load, press};

fn checkpoint(controller: &mut VisibleShellController, name: &str) {
    if let Some(directory) = std::env::var_os("CRYSTAL_MAP_RETAIN_SAVES_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("lab evidence directory");
        let file = directory.join(format!("fossil-lab-{name}.crystalsave"));
        assert!(!file.exists(), "fresh checkpoint required");
        controller.save(file).expect("native lab checkpoint");
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the actual Rochester fossil lab"]
fn fossil_lab_controller_streets_sign_bench_restoration_both_returns_and_continue() {
    let (root, runtime) = load();
    assert!(runtime.data().maps.contains_key("MnRochesterFossilLab"));
    for (species, item, exit) in [
        ("TRILUME", "LENS_FOSSIL", (4, 9)),
        ("SIPHLET", "SPIRE_FOSSIL", (5, 9)),
    ] {
        let mut controller = fixture(&root, &runtime, "MnRochester", (24, 12));
        {
            let state = controller.shell.shell.session_mut().state_mut();
            state.badges.johto[0] = true;
            state
                .flags
                .set_engine_flag("ENGINE_ZEPHYRBADGE", true)
                .expect("declared Care Badge");
            assert!(
                state
                    .bag
                    .add_item(&runtime.data().items[item], 1)
                    .expect("declared chosen specimen")
            );
            assert!(
                state.storage.party.add_pokemon(
                    runtime
                        .data()
                        .create_pokemon("POUCHID", 10, Dv::default())
                        .expect("Minnesota partner")
                )
            );
            state.sync_party_from_storage();
        }
        mark_runtime_snapshot_dirty(&mut controller.shell);
        // No party, flag, inventory, position or RNG writes after this setup.
        checkpoint(&mut controller, &format!("{species}-road"));
        indoor_walk(&mut controller, &runtime, (26, 11));
        let sign = read_text(&mut controller, (26, 10));
        assert!(sign.contains("FOSSIL LAB"), "actual exterior sign");
        indoor_walk(&mut controller, &runtime, (23, 10));
        for _ in 0..4 {
            press(&mut controller, GameButton::Up);
            if controller
                .snapshot()
                .expect("lab entrance")
                .overworld
                .map_name
                == "MnRochesterFossilLab"
            {
                break;
            }
        }
        at(&mut controller, "MnRochesterFossilLab", (5, 8));
        checkpoint(&mut controller, &format!("{species}-arrival"));
        indoor_walk(&mut controller, &runtime, (6, 4));
        let bench = read_text(&mut controller, (6, 3));
        assert!(
            bench.contains("TRILUME") && bench.contains("SIPHLET"),
            "readable native specimen bench"
        );
        indoor_walk(&mut controller, &runtime, (3, 5));
        talk(&mut controller, GameButton::Up, &[true]);
        assert_eq!(
            quantity(&controller, item),
            0,
            "successful restoration consumes the specimen"
        );
        assert!(event(&controller, &format!("EVENT_MN_{species}_RESTORED")));
        let state = controller.shell.shell.session().state();
        assert!(state.pokedex.has_caught(species));
        let gift = state.storage.party.pokemon[1]
            .as_ref()
            .expect("native gift partner");
        assert_eq!(gift.species.id, species);
        assert_eq!(gift.level, 10);
        controller = continue_save(root.clone(), runtime.clone(), &mut controller);
        talk(&mut controller, GameButton::Up, &[]);
        assert_eq!(
            controller
                .snapshot()
                .expect("once-only gift")
                .party
                .slots
                .len(),
            2
        );
        checkpoint(&mut controller, &format!("{species}-restored"));
        indoor_walk(&mut controller, &runtime, exit);
        // The native carpet takes the outward step after reaching its tile.
        for _ in 0..4 {
            press(&mut controller, GameButton::Down);
            if controller.snapshot().expect("lab exit").overworld.map_name == "MnRochester" {
                break;
            }
        }
        at(&mut controller, "MnRochester", (23, 10));
        for _ in 0..4 {
            press(&mut controller, GameButton::Up);
            if controller
                .snapshot()
                .expect("lab reentry")
                .overworld
                .map_name
                == "MnRochesterFossilLab"
            {
                break;
            }
        }
        at(&mut controller, "MnRochesterFossilLab", (5, 8));
        controller = continue_save(root.clone(), runtime.clone(), &mut controller);
        at(&mut controller, "MnRochesterFossilLab", (5, 8));
        checkpoint(&mut controller, &format!("{species}-return"));
    }
}
