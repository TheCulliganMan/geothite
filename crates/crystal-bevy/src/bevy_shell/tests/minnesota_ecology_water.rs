//! Native Surf and clock-dependent rod captures in the curated regional biomes.
use crystal_core::input::GameButton;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::super::super::{checkpoint, encounter_and_capture, fixture_at_hour, verify_capture};
use super::{completed_local_trainers, hour, source, tile};
use crate::bevy_shell::tests::r10_gallery::continue_save;
use crate::bevy_shell::tests::rochester::flee;
use crate::bevy_shell::tests::rochester_earned::try_recruit_with_ball;
use crate::bevy_shell::tests::winona::press;
use crate::bevy_shell::{VisibleFishingPhase, VisibleShellController, mark_runtime_snapshot_dirty};

fn direction(from: [i16; 2], to: [i16; 2]) -> GameButton {
    match (to[0] - from[0], to[1] - from[1]) {
        (1, 0) => GameButton::Right,
        (-1, 0) => GameButton::Left,
        (0, 1) => GameButton::Down,
        (0, -1) => GameButton::Up,
        _ => panic!("adjacent native bank required"),
    }
}

pub(super) fn mount(c: &mut VisibleShellController, from: [i16; 2], to: [i16; 2]) {
    assert_eq!(
        c.snapshot().expect("shore mode").overworld.mode,
        MovementMode::Normal
    );
    press(c, direction(from, to));
    for _ in 0..128 {
        if c.snapshot().expect("Surf prompt").overworld.mode == MovementMode::Surf {
            c.wait_frames(60).expect("native mounting presentation");
            let s = c.snapshot().expect("native water arrival");
            assert_eq!([s.overworld.tile.x, s.overworld.tile.y], to);
            return;
        }
        press(c, GameButton::A);
    }
    panic!("native Surf did not mount from {from:?} to {to:?}");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK and CRYSTAL_ECOLOGY_SOURCE with curated Surf banks"]
fn ecology_all58_local_surf_banks_mount_capture_and_continue() {
    let plan = source();
    let mut count = 0;
    for site in plan["surf"].as_array().expect("authored Surf sites") {
        if site["shore"].is_null() {
            continue;
        }
        count += 1;
        let map = site["map"].as_str().expect("Surf map");
        let shore = tile(&site["shore"]);
        let a = tile(&site["sample"]);
        let b = tile(&site["loop"]);
        let target = site["replay"]["species"]
            .as_str()
            .expect("local water resident");
        let time = site["replay"]["time"].as_str().expect("native period");
        let (root, runtime, mut c) = fixture_at_hour(map, shore, true, false, hour(time));
        completed_local_trainers(&mut c, &runtime, map);
        // Setup ends here. Surf, movement, encounters and captures use input.
        checkpoint(&mut c, &format!("ecology-{map}-surf-start"));
        mount(&mut c, shore, a);
        encounter_and_capture(&mut c, &runtime, target, a, b);
        verify_capture(&c, &runtime, target);
        c = continue_save(root, runtime.clone(), &mut c);
        verify_capture(&c, &runtime, target);
        assert_eq!(
            c.snapshot().expect("continued Surf").overworld.mode,
            MovementMode::Surf
        );
        checkpoint(&mut c, &format!("ecology-{map}-surf-caught"));
        eprintln!("verified native Surf {map} {time}: {target}");
    }
    assert_eq!(count, 58, "all declared local banks must be replayed");
}

fn cast_until_caught(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    target: &str,
    shore: [i16; 2],
) {
    for cast in 0..512 {
        c.press(GameButton::Select).expect("registered native rod");
        assert!(
            c.shell.visible_fishing_animation.is_some(),
            "actual casting animation"
        );
        for _ in 0..256 {
            if c.shell
                .visible_fishing_animation
                .as_ref()
                .is_some_and(|a| a.phase == VisibleFishingPhase::AwaitText)
            {
                break;
            }
            c.wait_frames(1).expect("native fishing clock");
        }
        for _ in 0..128 {
            let s = c.snapshot().expect("native fishing notice");
            assert_eq!([s.overworld.tile.x, s.overworld.tile.y], shore);
            if s.battle.is_some()
                || (c.shell.visible_fishing_animation.is_none()
                    && c.shell.field_notice.is_none()
                    && !c.has_pending_script_work())
            {
                break;
            }
            press(c, GameButton::A);
        }
        if let Some(battle) = c.snapshot().expect("native bite result").battle {
            if battle.enemy_pokemon.species.id == target {
                assert!(try_recruit_with_ball(c, runtime, "MASTER_BALL"));
                return;
            }
            flee(c);
        }
        c.wait_frames(17 + cast % 11).expect("ordinary bank idle");
    }
    panic!("native rod did not select {target} at {shore:?}");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK and CRYSTAL_ECOLOGY_SOURCE with private biome rod groups"]
fn ecology_all6_rod_biomes_good_super_day_night_capture_and_continue() {
    let plan = source();
    let mut banks = 0;
    for site in plan["fishing"].as_array().expect("authored rod sites") {
        if site["bank"].is_null() {
            continue;
        }
        banks += 1;
        let map = site["map"].as_str().expect("rod map");
        let biome = site["biome"].as_str().expect("rod biome");
        let bank = tile(&site["bank"]["shore"]);
        let water = tile(&site["bank"]["water"]);
        for (rod, field) in [("GOOD_ROD", "timed_good"), ("SUPER_ROD", "timed_super")] {
            for (index, time) in [(0, "day"), (1, "night")] {
                let target = plan["rod_recipes"][biome][field][index]
                    .as_str()
                    .expect("resolved clock resident");
                let (root, runtime, mut c) = fixture_at_hour(map, bank, false, true, hour(time));
                completed_local_trainers(&mut c, &runtime, map);
                if rod == "SUPER_ROD" {
                    let state = c.shell.shell.session_mut().state_mut();
                    assert!(
                        state
                            .bag
                            .add_item(&runtime.data().items[rod], 1)
                            .expect("declared prior Super Rod")
                    );
                    state.registered_key_item = Some(rod.into());
                    mark_runtime_snapshot_dirty(&mut c.shell);
                }
                assert!(
                    runtime.data().maps[map]
                        .attributes
                        .fishing_group
                        .as_ref()
                        .expect("private rod group")
                        .starts_with("FISHGROUP_MN_ECO_")
                );
                // No state, inventory, time or RNG writes after this setup.
                press(&mut c, direction(bank, water));
                cast_until_caught(&mut c, &runtime, target, bank);
                verify_capture(&c, &runtime, target);
                c = continue_save(root, runtime.clone(), &mut c);
                verify_capture(&c, &runtime, target);
                assert_eq!(
                    c.shell
                        .shell
                        .session()
                        .state()
                        .bag
                        .quantity(&runtime.data().items[rod]),
                    1,
                    "rods remain reusable"
                );
                checkpoint(&mut c, &format!("ecology-{biome}-{rod}-{time}-caught"));
                eprintln!("verified native {rod} {biome} {time}: {target}");
            }
        }
    }
    assert_eq!(
        banks, 6,
        "all six private rod recipes need native clock coverage"
    );
}
