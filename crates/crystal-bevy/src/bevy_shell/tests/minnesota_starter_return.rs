//! Declared League/party fixtures; gift choices and delivery use native input.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;

use super::super::mark_runtime_snapshot_dirty;
use super::interior_life::fixture;
use super::minnesota_fossils::talk;
use super::r10_gallery::continue_save;
use super::winona::load;

const CLAIMED: &str = "EVENT_MN_KINDLIT_RESEARCH_RECEIVED";

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Kindlit research partner"]
fn starter_return_controller_league_guard_full_party_and_once_after_continue() {
    let (root, runtime) = load();
    for (champion, party_count) in [(false, 1), (true, 6), (true, 1)] {
        let mut c = fixture(&root, &runtime, "MnWinonaFieldStation", (3, 9));
        {
            let state = c.shell.shell.session_mut().state_mut();
            state
                .flags
                .set_event_flag("EVENT_MN_LEAGUE_CHAMPION", champion)
                .expect("declared League fixture");
            for flag in [
                "EVENT_MN_STARTER_RECEIVED",
                "EVENT_MN_WINONA_BALLS_RECEIVED",
            ] {
                state
                    .flags
                    .set_event_flag(flag, true)
                    .expect("completed opening fixture");
            }
            state
                .flags
                .set_engine_flag("ENGINE_POKEDEX", true)
                .expect("research fixture");
            for index in 0..party_count {
                state.storage.party.pokemon[index] = Some(
                    runtime
                        .data()
                        .create_pokemon("POUCHID", 10, Dv::default())
                        .expect("non-Fire starter fixture"),
                );
            }
            state.sync_party_from_storage();
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        // No state, inventory, party, flags, position or RNG writes below.
        if champion {
            talk(&mut c, GameButton::Left, &[false]);
            assert!(
                !c.shell
                    .shell
                    .session()
                    .state()
                    .flags
                    .is_event_flag_set(CLAIMED)
                    .expect("declined research partner")
            );
        }
        talk(
            &mut c,
            GameButton::Left,
            if champion { &[true] } else { &[] },
        );
        let delivered = champion && party_count < 6;
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(CLAIMED)
                .expect("native claim"),
            delivered
        );
        let snapshot = c.snapshot().expect("native research outcome");
        assert_eq!(
            snapshot.party.slots.len(),
            party_count + usize::from(delivered)
        );
        assert_eq!(
            snapshot.party.slots[0].pokemon.species.id, "POUCHID",
            "research must not replace the chosen partner"
        );
        if delivered {
            let partner = &snapshot.party.slots[party_count].pokemon;
            assert_eq!(partner.species.id, "KINDLIT");
            assert_eq!(partner.level, 5);
            assert!(
                c.shell
                    .shell
                    .session()
                    .state()
                    .pokedex
                    .has_caught("KINDLIT")
            );
        }
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .pokedex
                .has_caught("KINDLIT"),
            delivered
        );
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(CLAIMED)
                .expect("continued claim"),
            delivered
        );
        if delivered || !champion {
            talk(&mut c, GameButton::Left, &[]);
            assert_eq!(
                c.snapshot().expect("no duplicate").party.slots.len(),
                party_count + usize::from(delivered)
            );
        } else {
            talk(&mut c, GameButton::Left, &[true]);
            assert!(
                !c.shell
                    .shell
                    .session()
                    .state()
                    .flags
                    .is_event_flag_set(CLAIMED)
                    .expect("full party still retains the offer")
            );
        }
    }
}
