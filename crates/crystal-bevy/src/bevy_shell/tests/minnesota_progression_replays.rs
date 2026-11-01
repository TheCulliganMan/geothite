//! Declared stage-appropriate parties; all subsequent encounters use input.
use super::{completed_local_trainers, source, tile};
use crate::bevy_shell::VisibleShellController;
use crate::bevy_shell::tests::minnesota_habitats::{checkpoint, fixture_with_partner};
use crate::bevy_shell::tests::r10_gallery::continue_save;
use crate::bevy_shell::tests::winona::press;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

fn catch_in_band(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    a: [i16; 2],
    b: [i16; 2],
    low: u8,
    high: u8,
) -> String {
    for step in 0..512 {
        let s = c.snapshot().expect("native stage encounter");
        if let Some(battle) = s.battle {
            let enemy = &battle.enemy_pokemon;
            assert!(
                (low..=high).contains(&enemy.level),
                "{} level{} outside stage band {low}..{high}",
                enemy.species.id,
                enemy.level
            );
            let species = enemy.species.id.clone();
            assert!(
                crate::bevy_shell::tests::rochester_earned::try_recruit_with_ball(
                    c,
                    r,
                    "MASTER_BALL"
                ),
                "native capture"
            );
            return species;
        }
        let at = [s.overworld.tile.x, s.overworld.tile.y];
        assert!(at == a || at == b, "stage loop left reviewed grass");
        let next = if at == a { b } else { a };
        press(
            c,
            match (next[0] - at[0], next[1] - at[1]) {
                (1, 0) => GameButton::Right,
                (-1, 0) => GameButton::Left,
                (0, 1) => GameButton::Down,
                (0, -1) => GameButton::Up,
                _ => panic!("adjacent grass loop"),
            },
        );
        c.wait_frames(17 + step % 11).expect("normal field frames");
    }
    panic!("no natural encounter at stage sample");
}

#[test]
#[ignore = "requires the progression pack and CRYSTAL_ECOLOGY_SOURCE; local stage fixtures do not prove earned arrival"]
fn progression_native_opening_through_league_uses_appropriate_levels_and_continue() {
    let plan = source();
    for (map, partner, level, low, high) in [
        ("MnR01C00R00", "POUCHID", 5, 2, 7),
        ("MnR02C00R00", "POUCHID", 12, 8, 14),
        ("MnR03C00R00", "SLIPPERINA", 18, 12, 20),
        ("MnR04C00R00", "SLIPPERINA", 23, 17, 25),
        ("MnR05C00R00", "SLIPPERINA", 27, 21, 29),
        ("MnR06C00R00", "SLIPPERINA", 32, 25, 34),
        ("MnR07C00R00", "VOYAGEAR", 36, 29, 38),
        ("MnR08C00R00", "VOYAGEAR", 38, 31, 40),
        ("MnR09C00R00", "VOYAGEAR", 41, 34, 43),
        ("MnR10C00R00", "VOYAGEAR", 43, 36, 45),
        ("MnGrandMarais", "VOYAGEAR", 46, 39, 48),
        ("MnR12C00R00", "VOYAGEAR", 47, 41, 49),
        ("MnR13C00R00", "VOYAGEAR", 50, 43, 52),
    ] {
        let site = plan["grass"]
            .as_array()
            .expect("reviewed surfaces")
            .iter()
            .find(|s| s["map"] == map);
        let (a, b) = match map {
            "MnR09C00R00" => ([50, 18], [51, 18]),
            "MnR10C00R00" => ([34, 8], [35, 8]),
            _ => {
                let site = site.expect("reviewed stage surface");
                (tile(&site["sample"]), tile(&site["loop"]))
            }
        };
        let (root, r, mut c) = fixture_with_partner(map, a, false, false, 12, partner, level);
        completed_local_trainers(&mut c, &r, map);
        // Setup ends here. No party, progress, time, position or RNG writes.
        let species = catch_in_band(&mut c, &r, a, b, low, high);
        super::super::verify_capture(&c, &r, &species);
        c = continue_save(root, r.clone(), &mut c);
        super::super::verify_capture(&c, &r, &species);
        checkpoint(&mut c, &format!("progression-{map}-caught"));
        eprintln!("native progression {map}: {species}, band{low}..{high}, partner{level}");
    }
}

#[test]
#[ignore = "requires the progression pack; declared four-badge side trip and capture supplies"]
fn progression_native_fair_food_families_fit_first_visit_and_continue() {
    for (zone, a) in [
        ("growers", [42, 6]),
        ("cookie-side", [44, 12]),
        ("midway-meadow", [36, 36]),
        ("cheese", [74, 46]),
        ("orchard-east", [76, 32]),
        ("smokehouse", [54, 36]),
    ] {
        let (root, r, mut c) =
            fixture_with_partner("MnStateFairgrounds", a, false, false, 12, "SLIPPERINA", 27);
        completed_local_trainers(&mut c, &r, "MnStateFairgrounds");
        assert_eq!(
            r.data().wild_encounters["MnStateFairgrounds"]
                .zone_at(a[0], a[1])
                .expect("native food grass")
                .id,
            zone
        );
        // All subsequent changes use production input.
        let species = catch_in_band(&mut c, &r, a, [a[0] + 1, a[1]], 21, 29);
        super::super::verify_capture(&c, &r, &species);
        c = continue_save(root, r.clone(), &mut c);
        super::super::verify_capture(&c, &r, &species);
        eprintln!("native first-visit fair {zone}: {species}");
    }
}

#[test]
#[ignore = "requires the progression pack and CRYSTAL_ECOLOGY_SOURCE with native Surf banks"]
fn progression_native_surf_return_and_late_shore_levels_and_continue() {
    let plan = source();
    for (map, partner, level, low, high) in [
        ("MnWinona", "BIRCHIT", 25, 20, 27),
        ("MnR06C00R00", "BIRCHIT", 32, 25, 34),
        ("MnGrandMarais", "VOYAGEAR", 46, 39, 48),
    ] {
        let site = plan["surf"]
            .as_array()
            .expect("reviewed banks")
            .iter()
            .find(|s| s["map"] == map);
        let (a, b, shore) = if map == "MnWinona" {
            ([26, 8], [27, 8], [25, 8])
        } else {
            let site = site.expect("reviewed Surf stage site");
            (
                tile(&site["sample"]),
                tile(&site["loop"]),
                tile(&site["shore"]),
            )
        };
        let (root, r, mut c) = fixture_with_partner(map, shore, true, false, 12, partner, level);
        completed_local_trainers(&mut c, &r, map);
        // Native shore prompt, mounting, swimming and capture follow setup.
        super::water::mount(&mut c, shore, a);
        let species = catch_in_band(&mut c, &r, a, b, low, high);
        super::super::verify_capture(&c, &r, &species);
        c = continue_save(root, r.clone(), &mut c);
        super::super::verify_capture(&c, &r, &species);
        eprintln!("native Surf progression {map}: {species}, band{low}..{high}");
    }
}
