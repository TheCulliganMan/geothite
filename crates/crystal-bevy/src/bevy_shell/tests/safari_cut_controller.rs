//! Paid approach, native Cut permissions, habitat access and temporary-tree reset.

use std::collections::{BTreeSet, VecDeque};

use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::safari::SafariPhase;
use crystal_core::systems::time::GameDate;
use crystal_runtime::CrystalRuntime;

use super::{
    LakeGrid, Point, VisibleShellController, active, admit, button, continue_save,
    controller_fixture, global, launch, press, release_wild, save, step, travel,
};

const APPROACH: Point = (10, 30);
const WATER: Point = (10, 29);
const TREE: Point = (10, 31);
const TARGET: Point = (9, 35);

fn tree(c: &VisibleShellController) -> u16 {
    c.shell
        .shell
        .session()
        .overworld()
        .map
        .metatile_at(5, 15)
        .expect("current native thicket metatile")
}

fn live_grid(c: &mut VisibleShellController, runtime: &CrystalRuntime) -> LakeGrid {
    assert_eq!(
        c.snapshot()
            .expect("thicket current map")
            .overworld
            .map_name,
        "MnBoundarySafariC00R00"
    );
    let mut grid = LakeGrid::new(runtime);
    let collision = &runtime.data().tilesets["mn_boundary_safari"].collision;
    let quadrants = &collision[&format!("{:02x}", tree(c))];
    for (index, kind) in quadrants.iter().enumerate() {
        grid.cells.insert(
            (10 + (index % 2) as i16, 30 + (index / 2) as i16),
            kind.clone(),
        );
    }
    grid
}

// Conservative graph: allow all ordinary walking/Surf transitions and native
// down-hop candidates. Denying access even in this superset rules out a dry,
// shore or ledge bypass. Successful traversal still requires real input below.
fn reached(grid: &LakeGrid, start: Point) -> BTreeSet<Point> {
    let mut seen = BTreeSet::from([start]);
    let mut pending = VecDeque::from([start]);
    while let Some((x, y)) = pending.pop_front() {
        let mut next = vec![(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];
        if grid.kind((x, y)) == "HOP_DOWN" {
            next.push((x, y + 2));
        }
        for point in next {
            if !grid.blocked.contains(&point)
                && matches!(
                    grid.kind(point),
                    "FLOOR" | "TALL_GRASS" | "WATER" | "HOP_DOWN"
                )
                && seen.insert(point)
            {
                pending.push_back(point);
            }
        }
    }
    seen
}

fn close(c: &mut VisibleShellController) {
    for _ in 0..96 {
        let snapshot = c.snapshot().expect("native Cut script settling");
        if !snapshot.ui.text_window_open
            && c.shell.field_notice.is_none()
            && !c.has_pending_script_work()
        {
            break;
        }
        press(c, GameButton::A);
    }
    c.wait_frames(60).expect("native Cut animation settling");
    assert!(
        !c.snapshot()
            .expect("Cut field released")
            .ui
            .text_window_open
    );
    assert!(!c.has_pending_script_work());
}

fn cut(c: &mut VisibleShellController, facing: GameButton) {
    release_wild(c);
    let at = global(c);
    let before = active(c);
    let time_of_day = c
        .snapshot()
        .expect("Cut clock before input")
        .progression
        .time
        .time_of_day;
    press(c, facing);
    assert_eq!(global(c), at, "uncut tree blocks the actual approach");
    for _ in 0..96 {
        if tree(c) == 0x3f {
            close(c);
            assert_eq!(active(c), before, "Cut itself spends no Safari resources");
            assert_eq!(
                c.snapshot()
                    .expect("Cut clock after input")
                    .progression
                    .time
                    .time_of_day,
                time_of_day
            );
            return;
        }
        press(c, GameButton::A);
    }
    panic!("native63→3f Cut failed at the paid thicket");
}

#[test]
#[ignore = "requires enabled current-parent CRYSTAL_INTERIOR_PACK Safari review"]
fn safari_paid_cut_gate_denies_permissions_opens_habitat_and_resets_safely() {
    for (name, knows_cut, hive) in [
        ("cut-no-move", false, true),
        ("cut-no-badge", true, false),
        ("cut-available", true, true),
    ] {
        let (root, runtime, mut c) = controller_fixture();
        let mut carrier = runtime
            .data()
            .create_pokemon("VOYAGEAR", 40, Dv::default())
            .expect("regional native Cut carrier");
        assert!(carrier.species.tmhm_learnset.iter().any(|m| m == "CUT"));
        if knows_cut {
            carrier.moves[0] = LearnedMove {
                name: "CUT".into(),
                current_pp: runtime.data().moves["CUT"].pp,
                pp_ups: 0,
            };
        }
        // All explicit party/permission setup ends before the first input.
        let state = c.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(carrier));
        state.badges.johto[1] = hive;
        state
            .flags
            .set_engine_flag("ENGINE_HIVEBADGE", hive)
            .expect("coherent native Cut permission fixture");
        state.sync_party_from_storage();
        // The location fixture otherwise starts with an unset midnight clock.
        // Establish noon before any input for matched native before/after views.
        c.shell
            .shell
            .update_clock_from_datetime(GameDate::new(2000, 1, 1), 12, 0, 0)
            .expect("explicit initial daytime review clock");
        super::super::mark_runtime_snapshot_dirty(&mut c.shell);
        let initial_party = c.snapshot().expect("initial regional team").party;
        let initial_bag = c.shell.shell.session().state().bag.clone();
        let grid = LakeGrid::new(&runtime);
        assert_eq!(grid.kind(TREE), "CUT_TREE");
        assert!(reached(&grid, (3, 89)).contains(&APPROACH));
        assert!(!reached(&grid, (3, 89)).contains(&TARGET));
        admit(&mut c);
        travel(&mut c, &grid, (3, 90), false);
        launch(&mut c, (3, 89));
        travel(&mut c, &grid, (30, 21), true);
        step(&mut c, (29, 21));
        assert!(travel(&mut c, &grid, (22, 21), false) >= 4);
        launch(&mut c, (21, 21));
        travel(&mut c, &grid, WATER, true);
        step(&mut c, APPROACH);
        assert_eq!(tree(&c), 0x63);
        save(&mut c, &format!("{name}-closed"));
        let before = active(&mut c);
        if !knows_cut || !hive {
            press(&mut c, GameButton::Down);
            press(&mut c, GameButton::A);
            c.wait_frames(240).expect("native Cut denial prints");
            let denied = c.snapshot().expect("Cut denial owns input");
            let text = denied
                .ui
                .text
                .as_ref()
                .and_then(|text| text.asm_text.as_ref())
                .expect("original native Cut denial text");
            assert!(text.to_ascii_uppercase().contains("CUT"));
            assert!(denied.ui.pending_yes_no.is_none());
            press(&mut c, GameButton::Down);
            assert_eq!(global(&mut c), APPROACH, "denial owns movement");
            save(&mut c, &format!("{name}-denied"));
            close(&mut c);
            assert_eq!(tree(&c), 0x63);
            assert_eq!(active(&mut c), before);
            assert!(!reached(&live_grid(&mut c, &runtime), WATER).contains(&TARGET));
        } else {
            press(&mut c, GameButton::Down);
            for _ in 0..96 {
                if c.shell.yes_no_cursor.is_some() {
                    break;
                }
                assert_eq!(tree(&c), 0x63, "no Cut without confirmation");
                press(&mut c, GameButton::A);
            }
            assert!(c.shell.yes_no_cursor.is_some());
            c.wait_frames(240).expect("complete Cut question");
            let question = c.snapshot().expect("renderer-neutral Cut question");
            let menu = question.ui.menu.as_ref().expect("visible Cut options");
            assert_eq!(menu.menu_id, "field:move-confirm");
            assert_eq!(menu.layout.vertical_menus[0].options, [">YES", " NO"]);
            assert!(
                question
                    .ui
                    .text
                    .as_ref()
                    .and_then(|t| t.asm_text.as_ref())
                    .is_some_and(|t| t.contains("CUT"))
            );
            save(&mut c, "cut-confirmation");
            press(&mut c, GameButton::Down);
            assert_eq!(
                c.shell.yes_no_cursor.as_ref().expect("Cut NO").option_index,
                1
            );
            assert_eq!(
                c.snapshot()
                    .expect("actual NO projection")
                    .ui
                    .menu
                    .expect("Cut options")
                    .layout
                    .vertical_menus[0]
                    .options,
                [" YES", ">NO"]
            );
            press(&mut c, GameButton::A);
            close(&mut c);
            assert_eq!(tree(&c), 0x63, "native NO preserves the gate");
            assert_eq!(active(&mut c), before);
            assert!(
                c.snapshot()
                    .expect("cancelled Cut menu released")
                    .ui
                    .menu
                    .is_none()
            );
            save(&mut c, "cut-cancelled");
            cut(&mut c, GameButton::Down);
            let collision = &runtime.data().tilesets["mn_boundary_safari"].collision;
            for quadrant in [0, 1, 3] {
                assert_eq!(collision["63"][quadrant], collision["3f"][quadrant]);
            }
            let opened = live_grid(&mut c, &runtime);
            assert!(reached(&opened, WATER).contains(&TARGET));
            save(&mut c, "cut-opened");
            assert!(travel(&mut c, &opened, TARGET, false) > 0);
            assert_eq!(active(&mut c).phase, SafariPhase::Exploring);
            let mut found_wildlife = false;
            for _ in 0..64 {
                let next = if global(&mut c) == TARGET {
                    (9, 36)
                } else {
                    TARGET
                };
                step(&mut c, next);
                if let SafariPhase::Encounter(wild) = active(&mut c).phase {
                    assert_eq!(wild.habitat_id, "PINE_THICKET");
                    assert_eq!(wild.origin_map_name, "MnBoundarySafariC00R00");
                    save(&mut c, "cut-thicket-wildlife");
                    release_wild(&mut c);
                    found_wildlife = true;
                    break;
                }
            }
            assert!(
                found_wildlife,
                "actual thicket grass must produce native Safari wildlife"
            );
            travel(&mut c, &opened, TARGET, false);
            save(&mut c, "cut-thicket-inside");
            c = continue_save(root.clone(), runtime.clone(), &mut c);
            assert_eq!(global(&mut c), TARGET);
            assert_eq!(tree(&c), 0x63, "native TILES reload restores vegetation");
            let restored = live_grid(&mut c, &runtime);
            travel(&mut c, &restored, (10, 32), false);
            cut(&mut c, GameButton::Up);
            let reopened = live_grid(&mut c, &runtime);
            travel(&mut c, &reopened, APPROACH, false);
            save(&mut c, "cut-inside-continue-escaped");
        }
        launch(&mut c, WATER);
        travel(&mut c, &grid, (21, 21), true);
        step(&mut c, (22, 21));
        assert!(travel(&mut c, &grid, (29, 21), false) >= 4);
        launch(&mut c, (30, 21));
        if knows_cut && hive {
            // Cross an actual native connection and return, rather than
            // resetting or changing the current map through a test hook.
            travel(&mut c, &grid, (22, 47), true);
            step(&mut c, (22, 48));
            step(&mut c, (22, 47));
            assert_eq!(tree(&c), 0x63, "ordinary map return also restores the gate");
            save(&mut c, "cut-map-return-restored");
        }
        travel(&mut c, &grid, (3, 89), true);
        step(&mut c, (3, 90));
        travel(&mut c, &grid, (10, 104), false);
        press(&mut c, button((10, 104), (10, 103)));
        press(&mut c, GameButton::A);
        assert!(
            c.snapshot()
                .expect("real ranger retirement")
                .safari_prompt
                .is_some()
        );
        let returning = active(&mut c);
        assert_eq!(returning.balls_remaining, 30);
        println!(
            "{name}: {} travel commits, 30 balls before real retirement",
            500 - returning.steps_remaining
        );
        save(&mut c, &format!("{name}-return-offer"));
        press(&mut c, GameButton::A);
        let returned = c.snapshot().expect("thicket trip returned");
        assert!(returned.safari.is_none());
        assert_eq!(returned.overworld.map_name, "MnBoundaryWaters");
        assert_eq!(returned.trainer.money, 1000);
        let mut expected = initial_party;
        for (expected, actual) in expected.slots.iter_mut().zip(&returned.party.slots) {
            expected.pokemon.happiness = actual.pokemon.happiness;
        }
        assert_eq!(
            returned.party, expected,
            "Cut/Surf preserve PP, HP and status"
        );
        assert_eq!(c.shell.shell.session().state().bag, initial_bag);
        save(&mut c, &format!("{name}-retired"));
    }
}
