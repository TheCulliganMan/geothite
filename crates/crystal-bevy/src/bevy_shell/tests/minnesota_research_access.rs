//! Declared gate/capacity cases plus an earned first-delivery return.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;

use super::authored_route_fixture::indoor_walk;
use super::interior_life::fixture;
use super::minnesota_fossils::talk;
use super::r10_gallery::continue_save;
use super::winona::{at, load, press, travel};
use crate::bevy_shell::{VisibleShellController, mark_runtime_snapshot_dirty};

const DELIVERED: &str = "EVENT_MN_DELIVERED_ROCHESTER_PARCEL";
fn event(c: &VisibleShellController, flag: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(flag)
        .expect("research flag")
}
fn quantity(c: &VisibleShellController, item: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&c.shell.shell.runtime().data().items[item])
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with early research access"]
fn early_research_starters_all_choices_decline_full_party_and_continue() {
    let (root, runtime) = load();
    for (chosen, initial) in [
        ("POUCHID", "CHIKORITA"),
        ("KINDLIT", "CYNDAQUIL"),
        ("BIRCHIT", "TOTODILE"),
    ] {
        for (delivered, count) in [(false, 1), (true, 6), (true, 1)] {
            let mut c = fixture(&root, &runtime, "MnWinonaFieldStation", (3, 9));
            let state = c.shell.shell.session_mut().state_mut();
            for name in [
                "EVENT_MN_STARTER_RECEIVED",
                "EVENT_MN_WINONA_BALLS_RECEIVED",
                &format!("EVENT_MN_WINONA_{initial}"),
            ] {
                state
                    .flags
                    .set_event_flag(name, true)
                    .expect("declared opening choice");
            }
            state
                .flags
                .set_event_flag(DELIVERED, delivered)
                .expect("declared first parcel");
            state
                .flags
                .set_engine_flag("ENGINE_POKEDEX", true)
                .expect("research access");
            for _ in 0..count {
                assert!(
                    state.storage.party.add_pokemon(
                        runtime
                            .data()
                            .create_pokemon(chosen, 5, Dv::default())
                            .expect("legal initial starter")
                    )
                );
            }
            state.sync_party_from_storage();
            let money = state.money;
            mark_runtime_snapshot_dirty(&mut c.shell);
            // No progress, position, party, resources or RNG writes after setup.
            if delivered {
                talk(&mut c, GameButton::Left, &[false]);
                assert_eq!(c.snapshot().expect("refused gift").party.slots.len(), count);
            }
            let alternatives = ["POUCHID", "KINDLIT", "BIRCHIT"]
                .into_iter()
                .filter(|s| *s != chosen)
                .collect::<Vec<_>>();
            for (index, species) in alternatives.iter().enumerate() {
                talk(
                    &mut c,
                    GameButton::Left,
                    if delivered { &[true] } else { &[] },
                );
                let accepted = delivered && count < 6;
                assert_eq!(
                    event(&c, &format!("EVENT_MN_{species}_RESEARCH_RECEIVED")),
                    accepted
                );
                if accepted {
                    let s = c.snapshot().expect("real early gift");
                    assert_eq!(s.party.slots[count + index].pokemon.species.id, *species);
                    assert_eq!(s.party.slots[count + index].pokemon.level, 5);
                    assert!(c.shell.shell.session().state().pokedex.has_caught(species));
                }
                c = continue_save(root.clone(), runtime.clone(), &mut c);
                if !accepted {
                    break;
                }
            }
            assert_eq!(
                c.shell.shell.session().state().money,
                money,
                "all starter partners are free"
            );
            assert!(!event(&c, "EVENT_MN_LEAGUE_CHAMPION"));
            if delivered && count < 6 {
                talk(&mut c, GameButton::Left, &[]);
                assert_eq!(c.snapshot().expect("once-only gifts").party.slots.len(), 3);
                assert!(
                    !event(&c, &format!("EVENT_MN_{chosen}_RESEARCH_RECEIVED")),
                    "original choice is not granted again"
                );
            }
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with early research access"]
fn early_research_starters_earned_first_delivery_returns_before_any_badge() {
    let (root, runtime, mut c) = super::winona::opening(1, "CYNDAQUIL");
    indoor_walk(&mut c, &runtime, (6, 6));
    travel(&mut c, GameButton::Down, "MnWinona", (7, 8));
    indoor_walk(&mut c, &runtime, (4, 18));
    travel(&mut c, GameButton::Left, "MnR01C00R00", (63, 24));
    super::rochester::trail(&mut c, &runtime, (0, 24));
    travel(&mut c, GameButton::Left, "MnRochester", (39, 18));
    indoor_walk(&mut c, &runtime, (15, 8));
    travel(&mut c, GameButton::Up, "MnRochesterPokecenter1F", (10, 13));
    indoor_walk(&mut c, &runtime, (11, 9));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    assert!(
        !event(&c, DELIVERED),
        "the first visible page cannot grant delivery"
    );
    super::winona::finish(&mut c);
    assert!(event(&c, DELIVERED));
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .badges
            .johto
            .iter()
            .any(|b| *b)
    );
    indoor_walk(&mut c, &runtime, (10, 14));
    travel(&mut c, GameButton::Down, "MnRochester", (15, 8));
    indoor_walk(&mut c, &runtime, (38, 18));
    travel(&mut c, GameButton::Right, "MnR01C00R00", (0, 24));
    super::rochester::trail(&mut c, &runtime, (63, 24));
    travel(&mut c, GameButton::Right, "MnWinona", (0, 18));
    travel(&mut c, GameButton::Right, "MnWinona", (4, 18));
    indoor_walk(&mut c, &runtime, (18, 20));
    travel(&mut c, GameButton::Up, "MnWinonaFieldStation", (4, 10));
    indoor_walk(&mut c, &runtime, (3, 9));
    let money = c.shell.shell.session().state().money;
    talk(&mut c, GameButton::Left, &[true]);
    talk(&mut c, GameButton::Left, &[true]);
    let mut species = c
        .snapshot()
        .expect("earned early team")
        .party
        .slots
        .iter()
        .map(|p| p.pokemon.species.id.clone())
        .collect::<Vec<_>>();
    species.sort();
    assert_eq!(species, ["BIRCHIT", "KINDLIT", "POUCHID"]);
    assert_eq!(c.shell.shell.session().state().money, money);
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .badges
            .johto
            .iter()
            .any(|b| *b)
    );
    c = continue_save(root, runtime, &mut c);
    talk(&mut c, GameButton::Left, &[]);
    assert_eq!(
        c.snapshot()
            .expect("continued earned trio")
            .party
            .slots
            .len(),
        3
    );
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with early research access and Rochester lab"]
fn early_research_fossils_both_choices_walk_to_lab_restore_both_without_champion() {
    let (root, runtime) = load();
    for lens in [true, false] {
        let mut c = fixture(&root, &runtime, "MnR01C00R00", (40, 21));
        let state = c.shell.shell.session_mut().state_mut();
        state.badges.johto[0] = true;
        state
            .flags
            .set_engine_flag("ENGINE_ZEPHYRBADGE", true)
            .expect("declared Care Badge");
        let mut partner = runtime
            .data()
            .create_pokemon("POUCHID", 10, Dv::default())
            .expect("legal escort");
        partner.item = Some("SMOKE_BALL".into());
        assert!(state.storage.party.add_pokemon(partner));
        state.sync_party_from_storage();
        mark_runtime_snapshot_dirty(&mut c.shell);
        // Care Badge and escort are declared; all fossils and gifts are earned below.
        talk(&mut c, GameButton::Left, &[true, lens, true]);
        let (first_item, first, second_item, second) = if lens {
            ("LENS_FOSSIL", "TRILUME", "SPIRE_FOSSIL", "SIPHLET")
        } else {
            ("SPIRE_FOSSIL", "SIPHLET", "LENS_FOSSIL", "TRILUME")
        };
        assert_eq!(quantity(&c, first_item), 1);
        talk(&mut c, GameButton::Left, &[]);
        assert_eq!(
            quantity(&c, second_item),
            0,
            "restore the first fossil before claiming the other"
        );
        super::rochester::trail(&mut c, &runtime, (0, 24));
        travel(&mut c, GameButton::Left, "MnRochester", (39, 18));
        indoor_walk(&mut c, &runtime, (23, 10));
        travel(&mut c, GameButton::Up, "MnRochesterFossilLab", (5, 8));
        indoor_walk(&mut c, &runtime, (7, 7));
        talk(&mut c, GameButton::Up, &[]);
        assert!(!event(&c, "EVENT_MN_FOSSIL_EXCHANGED"));
        indoor_walk(&mut c, &runtime, (3, 5));
        talk(&mut c, GameButton::Up, &[true]);
        assert_eq!(quantity(&c, first_item), 0);
        assert!(event(&c, &format!("EVENT_MN_{first}_RESTORED")));
        indoor_walk(&mut c, &runtime, (7, 7));
        talk(&mut c, GameButton::Up, &[false]);
        assert!(!event(&c, "EVENT_MN_FOSSIL_EXCHANGED"));
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        let money = c.shell.shell.session().state().money;
        talk(&mut c, GameButton::Up, &[true]);
        assert!(event(&c, "EVENT_MN_FOSSIL_EXCHANGED"));
        assert_eq!(quantity(&c, second_item), 1);
        assert_eq!(c.shell.shell.session().state().money, money);
        talk(&mut c, GameButton::Up, &[]);
        assert_eq!(
            quantity(&c, second_item),
            1,
            "shared claim prevents duplicate fossils"
        );
        indoor_walk(&mut c, &runtime, (3, 5));
        talk(&mut c, GameButton::Up, &[true]);
        assert_eq!(quantity(&c, second_item), 0);
        assert!(event(&c, &format!("EVENT_MN_{second}_RESTORED")));
        assert!(!event(&c, "EVENT_MN_LEAGUE_CHAMPION"));
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        let state = c.shell.shell.session().state();
        assert!(state.pokedex.has_caught(first) && state.pokedex.has_caught(second));
        assert_eq!(state.storage.party.filled_slots(), 3);
        indoor_walk(&mut c, &runtime, (7, 7));
        talk(&mut c, GameButton::Up, &[]);
        assert_eq!(quantity(&c, first_item) + quantity(&c, second_item), 0);
        at(&mut c, "MnRochesterFossilLab", (7, 7));
        press(&mut c, GameButton::Start);
        press(&mut c, GameButton::B);
    }
}
