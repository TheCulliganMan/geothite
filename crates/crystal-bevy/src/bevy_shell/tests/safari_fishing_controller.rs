//! Registered native rod input reaches the same trainer-backed Safari battle.

use crystal_core::input::GameButton;
use crystal_core::state::BattleMemory;
use crystal_core::systems::safari::{SafariEncounterSource, SafariPhase, SafariResolution};
use crystal_core::world::map::TilePosition;

use super::{
    active, at, button, collision, continue_save, controller_fixture, mark_runtime_snapshot_dirty,
    path, press, save, walk,
};

#[test]
#[ignore = "requires enabled CRYSTAL_INTERIOR_PACK Safari review config"]
fn safari_registered_rod_plays_native_cast_and_delivers_capture_without_combat() {
    let (root, runtime, mut c) = controller_fixture();
    let state = c.shell.shell.session_mut().state_mut();
    assert!(
        state
            .bag
            .add_item(&runtime.data().items["GOOD_ROD"], 1)
            .expect("prior earned rod fixture")
    );
    state.registered_key_item = Some("GOOD_ROD".into());
    mark_runtime_snapshot_dirty(&mut c.shell);
    // All explicit setup ends here; actual ranger and rod input follow.
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::A);
    // The lower court chunk has no reachable casting bank. Follow its actual
    // north walking connection into the lake/portage chunk, without teleporting.
    let exit = path(&mut c, &runtime, |p| p == (12, 0));
    walk(&mut c, &exit, true);
    if matches!(active(&mut c).phase, SafariPhase::Encounter(_)) {
        super::run(&mut c);
    }
    press(&mut c, GameButton::Up);
    assert_eq!(
        c.snapshot()
            .expect("native north connection")
            .overworld
            .map_name,
        "MnBoundarySafariC00R01"
    );
    if matches!(active(&mut c).phase, SafariPhase::Encounter(_)) {
        super::run(&mut c);
    }
    let scene = c.snapshot().expect("preserve entry");
    let map = &runtime.data().maps[&scene.overworld.map_name];
    let water_at = |p: (i16, i16)| {
        [
            (p.0 - 1, p.1),
            (p.0 + 1, p.1),
            (p.0, p.1 - 1),
            (p.0, p.1 + 1),
        ]
        .into_iter()
        .find(|q| collision(&runtime, map, *q) == Some("WATER"))
    };
    let points = path(&mut c, &runtime, |p| {
        matches!(collision(&runtime, map, p), Some("FLOOR" | "TALL_GRASS")) && water_at(p).is_some()
    });
    walk(&mut c, &points, true);
    if matches!(active(&mut c).phase, SafariPhase::Encounter(_)) {
        super::run(&mut c);
    }
    let bank = at(&mut c);
    let water = water_at(bank).expect("real water contact");
    press(&mut c, button(bank, water));
    assert_eq!(
        at(&mut c),
        bank,
        "facing the water must retain the actual bank"
    );
    let steps = active(&mut c).steps_remaining;
    assert_eq!(
        active(&mut c).phase,
        SafariPhase::Exploring,
        "shore approach must finish its grass encounter before casting"
    );
    for cast_number in 0..30 {
        c.press(GameButton::Select)
            .expect("registered native Good Rod");
        assert!(c.shell.visible_fishing_animation.is_some());
        assert!(
            !super::super::super::safari_visible::battle_active(&c.shell),
            "cast owns the field before the battle screen"
        );
        let committed = c.shell.shell.session().clone();
        for _ in 0..130 {
            if c.shell
                .visible_fishing_animation
                .as_ref()
                .is_some_and(|a| a.phase == super::super::super::VisibleFishingPhase::AwaitText)
            {
                break;
            }
            c.press(GameButton::Up)
                .expect("casting clock owns movement");
        }
        assert_eq!(
            c.shell
                .visible_fishing_animation
                .as_ref()
                .expect("native bite text")
                .phase,
            super::super::super::VisibleFishingPhase::AwaitText
        );
        assert_eq!(
            c.shell.shell.session(),
            &committed,
            "cast presentation cannot spend steps or change authoritative RNG"
        );
        // Native text needs reveal/advance acknowledgement, rather than one A
        // being assumed to dismiss every text speed and prompt-arrow state.
        for _ in 0..64 {
            if c.shell.visible_fishing_animation.is_none() {
                break;
            }
            press(&mut c, GameButton::A);
        }
        assert!(c.shell.visible_fishing_animation.is_none());
        assert_eq!(at(&mut c), bank);
        assert_eq!(active(&mut c).steps_remaining, steps);
        let SafariPhase::Encounter(wild) = active(&mut c).phase else {
            continue;
        };
        assert!(super::super::super::safari_visible::battle_active(&c.shell));
        assert_eq!(wild.origin_tile, TilePosition::new(bank.0, bank.1));
        let SafariEncounterSource::Fishing {
            water_tile, rod, ..
        } = &wild.source
        else {
            panic!("native rod source");
        };
        assert_eq!(*water_tile, TilePosition::new(water.0, water.1));
        assert_eq!(rod, "GOOD_ROD");
        let owned = active(&mut c);
        c.press(GameButton::Select)
            .expect("capture ignores registered rod shortcut");
        assert_eq!(active(&mut c), owned);
        assert!(c.shell.visible_fishing_animation.is_none());
        assert_eq!(
            c.shell.shell.session().state().battle,
            BattleMemory::Inactive
        );
        save(&mut c, &format!("rod-wildlife-{cast_number}"));
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::Left);
        press(&mut c, GameButton::A);
        let pending = active(&mut c);
        let SafariPhase::Presenting { result, .. } = &pending.phase else {
            panic!("real throw");
        };
        if let SafariResolution::Caught(stored) = &result.resolution {
            assert_eq!(stored.pokemon.species.id, wild.pokemon.species.id);
            assert_eq!(
                stored
                    .pokemon
                    .caught_data
                    .as_ref()
                    .expect("caught area")
                    .location,
                wild.caught_location
            );
            let storage = c.shell.shell.session().state().storage.clone();
            save(&mut c, "rod-caught-result");
            c = continue_save(root.clone(), runtime.clone(), &mut c);
            assert_eq!(active(&mut c), pending);
            assert_eq!(c.shell.shell.session().state().storage, storage);
            super::finish_catch_without_nickname(&mut c);
            assert_eq!(active(&mut c).phase, SafariPhase::Exploring);
            assert_eq!(c.shell.shell.session().state().storage, storage);
            save(&mut c, "rod-caught-shore");
            return;
        }
        press(&mut c, GameButton::A);
        if matches!(active(&mut c).phase, SafariPhase::Encounter(_)) {
            super::run(&mut c);
        }
    }
    panic!("the thirty-ball native rod itinerary produced no capture");
}
