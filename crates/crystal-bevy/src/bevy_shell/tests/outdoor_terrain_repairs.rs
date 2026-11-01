//! Explicit local review fixtures. Coordinates/Repel are initial setup only.

use std::path::PathBuf;

use crystal_core::input::GameButton;
use serde_json::Value;

use super::{Setup, assert_at, fixture, load, move_to, press};

fn point(value: &Value) -> (i16, i16) {
    (
        i16::try_from(value[0].as_i64().expect("fixture x")).expect("x range"),
        i16::try_from(value[1].as_i64().expect("fixture y")).expect("y range"),
    )
}

#[test]
#[ignore = "requires the external boulder-setbacks pack and its runtime authoring plan"]
fn outdoor_setback_boulders_leave_walkable_rims_and_solid_cliff_faces() {
    let (root, runtime) = load();
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_OUTDOOR_BOULDER_PLAN").expect("reviewed boulder setback plan"),
    );
    let plan: Value = serde_json::from_slice(&std::fs::read(path).expect("boulder plan"))
        .expect("boulder plan JSON");
    let moves = plan["boulder_moves"].as_array().expect("explicit setbacks");
    assert_eq!(moves.len(), 10, "all reviewed straight-front boulders");
    for movement in moves {
        let map = movement["map"].as_str().expect("boulder map");
        let (mx, my) = point(&movement["from"]);
        let left = (mx * 2, my * 2);
        let right = (left.0 + 1, left.1);
        let mut controller = fixture(&root, &runtime, Setup::at(map, left));
        let before = controller.snapshot().expect("initial cliff rim");
        for at in [left, right] {
            if at == right {
                move_to(&mut controller, GameButton::Right, right);
            }
            for direction in [GameButton::Up, GameButton::Down] {
                for _ in 0..3 {
                    press(&mut controller, direction);
                }
                assert_at(&mut controller, map, at);
                let snapshot = controller
                    .snapshot()
                    .expect("retaining wall denies crossing");
                assert!(!snapshot.ui.text_window_open);
                assert!(matches!(
                    controller
                        .shell
                        .shell
                        .snapshot()
                        .expect("runtime phase snapshot")
                        .phase,
                    crystal_runtime::RuntimeShellPhase::Overworld
                ));
            }
        }
        move_to(&mut controller, GameButton::Left, left);
        let after = controller
            .snapshot()
            .expect("crossed rim in both directions");
        assert_eq!(before.party, after.party);
        assert_eq!(before.bag, after.bag);
    }
}

#[test]
#[ignore = "requires external repaired pack and CRYSTAL_OUTDOOR_STAIR_PHYSICS runtime receipt"]
fn outdoor_repaired_stairs_climb_descend_and_reject_side_exits() {
    let (root, runtime) = load();
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_OUTDOOR_STAIR_PHYSICS")
            .expect("external static flight inventory"),
    );
    let receipt: Value = serde_json::from_slice(&std::fs::read(path).expect("flight inventory"))
        .expect("flight JSON");
    let mut count = 0;
    for flight in receipt["flights"].as_array().expect("flight list") {
        let map = flight["map"].as_str().expect("flight map");
        // Duluth has its own complete courses. The four unchanged northern
        // ramps and R06's Cut-gated foot have distinct traversal contracts.
        if map == "MnDuluth"
            || map == "MnR06C00R00"
            || flight["kind"] == "retained native north-facing ramp"
        {
            continue;
        }
        let upper = point(&flight["upper"]);
        let step = point(&flight["step"]);
        let lower = point(&flight["lower"]);
        let mut c = fixture(&root, &runtime, Setup::at(map, upper));
        let state = c.shell.shell.session_mut().state_mut();
        state.repel_steps_remaining = 200;
        state.active_repel_item = Some("MAX_REPEL".into());
        super::mark_runtime_snapshot_dirty(&mut c.shell);
        let before = c.snapshot().expect("initial upper landing");
        move_to(&mut c, GameButton::Down, step);
        for direction in [GameButton::Left, GameButton::Right] {
            for _ in 0..3 {
                press(&mut c, direction);
            }
            assert_at(&mut c, map, step);
        }
        move_to(&mut c, GameButton::Down, lower);
        move_to(&mut c, GameButton::Up, upper);
        let after = c.snapshot().expect("returned upper landing");
        assert_eq!(before.party, after.party, "{map} stair preserves partners");
        assert_eq!(before.bag, after.bag, "{map} stair preserves inventory");
        count += 1;
    }
    assert_eq!(
        count, 22,
        "all repaired non-Duluth flights except the explicit Cut gate"
    );
}

#[test]
#[ignore = "requires external repaired Minnesota pack and the runtime cliff-joins authoring plan"]
fn outdoor_cliffs_deny_hops_while_low_banks_retain_them() {
    let (root, runtime) = load();
    let path = std::env::var_os("CRYSTAL_OUTDOOR_CLIFF_PLAN")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                "../../planning/minnesota/revisions/outdoor-stair-landings-v01/cliff-joins.json",
            )
        });
    let plan: Value = serde_json::from_slice(&std::fs::read(path).expect("explicit cliff plan"))
        .expect("cliff JSON");
    let mut cells = 0;
    for change in plan["changes"].as_array().expect("explicit cliff changes") {
        if change["after"] != "72" || !matches!(change["before"].as_str(), Some("56" | "57")) {
            continue;
        }
        let map = change["map"].as_str().expect("cliff map");
        let (mx, my) = point(&change["metatile"]);
        let module = &runtime.data().maps[map];
        let collision = &runtime.data().tilesets[&module.attributes.tileset_name].collision;
        for x in [mx * 2, mx * 2 + 1] {
            let y = my * 2;
            let mut c = fixture(&root, &runtime, Setup::at(map, (x, y)));
            for _ in 0..3 {
                press(&mut c, GameButton::Down);
            }
            assert_at(&mut c, map, (x, y));
            assert!(matches!(
                c.shell
                    .shell
                    .snapshot()
                    .expect("runtime phase snapshot")
                    .phase,
                crystal_runtime::RuntimeShellPhase::Overworld
            ));
            assert!(
                !c.snapshot()
                    .expect("blocked cliff input")
                    .ui
                    .text_window_open
            );
            let index = usize::try_from((y + 2) / 2).expect("lower row")
                * usize::from(module.attributes.width)
                + usize::try_from(x / 2).expect("lower column");
            let quadrant = usize::try_from((y + 2) % 2 * 2 + x % 2).expect("lower quadrant");
            let kind = &collision[&format!("{:02x}", module.blocks[index])][quadrant];
            if matches!(kind.as_str(), "FLOOR" | "TALL_GRASS") {
                let mut lower = fixture(&root, &runtime, Setup::at(map, (x, y + 2)));
                for _ in 0..3 {
                    press(&mut lower, GameButton::Up);
                }
                assert_at(&mut lower, map, (x, y + 2));
                assert!(matches!(
                    lower
                        .shell
                        .shell
                        .snapshot()
                        .expect("runtime phase snapshot")
                        .phase,
                    crystal_runtime::RuntimeShellPhase::Overworld
                ));
                assert!(
                    !lower
                        .snapshot()
                        .expect("blocked reverse cliff input")
                        .ui
                        .text_window_open
                );
            }
        }
        cells += 1;
    }
    assert_eq!(cells, 33, "every authored tall-cliff hop closure");
    for (map, y, lanes) in [
        ("MnR19C00R00", 32, vec![32, 33, 34, 35]),
        ("MnR03C00R00", 20, vec![10, 11, 12, 13, 14, 15]),
    ] {
        for x in lanes {
            let mut c = fixture(&root, &runtime, Setup::at(map, (x, y)));
            let before = c.snapshot().expect("low-bank crest");
            move_to(&mut c, GameButton::Down, (x, y + 2));
            for _ in 0..3 {
                press(&mut c, GameButton::Up);
            }
            assert_at(&mut c, map, (x, y + 2));
            let after = c.snapshot().expect("reverse hop denied");
            assert_eq!(before.party, after.party);
            assert_eq!(before.bag, after.bag);
        }
    }
}

#[test]
#[ignore = "requires external repaired Minnesota outdoor pack"]
fn outdoor_pictured_boulders_leave_a_clear_rim_and_solid_cliff() {
    let (root, runtime) = load();
    let map = "MnBoundarySafariC01R00";
    for (y, lanes) in [(18, [26, 27, 30, 31]), (26, [16, 17, 20, 21])] {
        let mut c = fixture(&root, &runtime, Setup::at(map, (lanes[0], y)));
        move_to(&mut c, GameButton::Right, (lanes[3], y));
        move_to(&mut c, GameButton::Left, (lanes[0], y));
        for x in lanes {
            let mut c = fixture(&root, &runtime, Setup::at(map, (x, y)));
            for direction in [GameButton::Up, GameButton::Down] {
                for _ in 0..3 {
                    press(&mut c, direction);
                }
                assert_at(&mut c, map, (x, y));
            }
        }
    }
}

#[test]
#[ignore = "requires external repaired Minnesota outdoor pack"]
fn outdoor_r06_stair_keeps_native_cut_permissions_and_usable_return() {
    use crystal_core::models::{Dv, LearnedMove};

    let (root, runtime) = load();
    let map = "MnR06C00R00";
    for (knows_cut, hive) in [(false, true), (true, false), (true, true)] {
        let mut setup = Setup::at(map, (29, 30));
        setup.badges[1] = hive;
        let mut c = fixture(&root, &runtime, setup);
        let mut carrier = runtime
            .data()
            .create_pokemon("VOYAGEAR", 45, Dv::default())
            .expect("Minnesota Cut carrier");
        assert!(
            carrier
                .species
                .tmhm_learnset
                .iter()
                .any(|name| name == "CUT")
        );
        if knows_cut {
            carrier.moves[0] = LearnedMove {
                name: "CUT".into(),
                current_pp: runtime.data().moves["CUT"].pp,
                pp_ups: 0,
            };
        }
        let state = c.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(carrier));
        state.sync_party_from_storage();
        super::mark_runtime_snapshot_dirty(&mut c.shell);
        let before = c.snapshot().expect("initial Cut permission fixture");
        move_to(&mut c, GameButton::Down, (29, 31));
        for _ in 0..3 {
            press(&mut c, GameButton::Down);
        }
        assert_at(&mut c, map, (29, 31));
        assert_eq!(
            c.shell.shell.session().overworld().map.metatile_at(14, 16),
            Some(0x5b)
        );
        press(&mut c, GameButton::A);
        for _ in 0..96 {
            if c.shell.shell.session().overworld().map.metatile_at(14, 16) == Some(0x3c) {
                break;
            }
            press(&mut c, GameButton::A);
        }
        if !knows_cut || !hive {
            assert_eq!(
                c.shell.shell.session().overworld().map.metatile_at(14, 16),
                Some(0x5b),
                "native Cut denies missing prerequisites"
            );
            assert_at(&mut c, map, (29, 31));
            continue;
        }
        assert_eq!(
            c.shell.shell.session().overworld().map.metatile_at(14, 16),
            Some(0x3c),
            "native Cut opens the real stair foot"
        );
        for _ in 0..96 {
            let snapshot = c.snapshot().expect("Cut presentation");
            if !snapshot.ui.text_window_open
                && c.shell.field_notice.is_none()
                && !c.has_pending_script_work()
            {
                break;
            }
            press(&mut c, GameButton::A);
        }
        c.wait_frames(60).expect("Cut presentation released");
        move_to(&mut c, GameButton::Down, (29, 33));
        move_to(&mut c, GameButton::Up, (29, 30));
        let after = c.snapshot().expect("climbed back after native Cut");
        assert_eq!(
            before.party, after.party,
            "field Cut does not spend PP or HP"
        );
        assert_eq!(before.bag, after.bag);
    }
}
