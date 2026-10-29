//! Local access/party fixtures; every later road, ledge, Surf and reward uses input.
use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life::{fixture, load, talk};
use super::minneapolis_falls::{mount, step};
use super::r10_gallery::{continue_save, flags, map, press, settle, tile};

const PARK: &str = "MnMilleLacs";
const ROUTE: &str = "MnR18C01R00";
const REWARD: &str = "EVENT_MN_MILLE_LACS_ROOST";
type Point = (i16, i16);

fn save(c: &mut VisibleShellController, label: &str) {
    if let Some(root) = std::env::var_os("CRYSTAL_MILLE_LACS_PARK_SAVES") {
        let root = PathBuf::from(root);
        std::fs::create_dir_all(&root).expect("retained park checkpoints");
        let path = root.join(format!("{label}.crystalsave"));
        assert!(!path.exists(), "preserve previous reached park saves");
        c.save(path).expect("actual reached park save");
    }
}

fn walk(c: &mut VisibleShellController, r: &CrystalRuntime, to: Point) -> usize {
    assert_eq!(map(c), PARK);
    let start = tile(c);
    let m = &r.data().maps[PARK];
    let collision = &r.data().tilesets["mn_headwaters"].collision;
    let mut parents = BTreeMap::from([(start, None)]);
    let mut todo = VecDeque::from([start]);
    while let Some((x, y)) = todo.pop_front() {
        for (at, button) in [
            ((x - 1, y), GameButton::Left),
            ((x + 1, y), GameButton::Right),
            ((x, y - 1), GameButton::Up),
            ((x, y + 1), GameButton::Down),
        ] {
            if at.0 < 0 || at.1 < 0 || at.0 >= 48 || at.1 >= 40 || parents.contains_key(&at) {
                continue;
            }
            if m.objects.iter().any(|o| {
                (o.x as i16, o.y as i16) == at
                    && (o.event_flag == "-1"
                        || !c
                            .shell
                            .shell
                            .session()
                            .state()
                            .flags
                            .is_event_flag_set(&o.event_flag)
                            .expect("native park object event"))
            }) {
                continue;
            }
            let b = m.blocks[at.1 as usize / 2 * 24 + at.0 as usize / 2];
            if matches!(
                collision[&format!("{b:02x}")][at.1 as usize % 2 * 2 + at.0 as usize % 2].as_str(),
                "FLOOR" | "TALL_GRASS"
            ) {
                parents.insert(at, Some(((x, y), button)));
                todo.push_back(at);
            }
        }
    }
    assert!(
        parents.contains_key(&to),
        "ordinary park path {start:?} -> {to:?}"
    );
    let mut course = Vec::new();
    let mut at = to;
    while let Some((prior, button)) = parents[&at] {
        course.push((at, button));
        at = prior;
    }
    let length = course.len();
    for (target, button) in course.into_iter().rev() {
        step(c, button, PARK, target, "actual ordinary park path");
    }
    length
}

fn blocked(c: &mut VisibleShellController, button: GameButton) {
    let at = tile(c);
    for _ in 0..3 {
        press(c, button);
        assert_eq!(tile(c), at, "native park barrier");
        assert_eq!(map(c), PARK);
    }
}

fn partner(c: &mut VisibleShellController, r: &CrystalRuntime) {
    assert!(
        r.data().pokemon["GILLPUP"]
            .tmhm_learnset
            .iter()
            .any(|m| m == "SURF")
    );
    let mut p = r
        .data()
        .create_pokemon("GILLPUP", 60, Dv::default())
        .expect("regional legal Surf partner");
    p.moves = vec![LearnedMove {
        name: "SURF".into(),
        current_pp: r.data().moves["SURF"].pp,
        pp_ups: 0,
    }];
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(p));
    for id in ["BIRCHIT", "POUCHID"] {
        assert!(
            state.storage.party.add_pokemon(
                r.data()
                    .create_pokemon(id, 30, Dv::default())
                    .expect("regional reserve fixture")
            )
        );
    }
    state.sync_party_from_storage();
    state.badges.johto[3] = true;
    state
        .flags
        .set_engine_flag("ENGINE_FOGBADGE", true)
        .expect("initial Surf badge fixture");
    mark_runtime_snapshot_dirty(&mut c.shell);
}

fn run_from_wild(c: &mut VisibleShellController) {
    for _ in 0..1024 {
        let s = c.snapshot().expect("native wild input owner");
        let button = if let Some(battle) = s.battle {
            assert!(matches!(
                battle.kind,
                crystal_runtime::RuntimeBattleKind::Wild { .. }
            ));
            if !c.shell.battle_messages.is_empty() || (s.ui.menu.is_none() && s.ui.text.is_some()) {
                GameButton::A
            } else if let Some(cursor) = &c.shell.battle_action_cursor {
                match cursor.option_index {
                    0 | 1 => GameButton::Down,
                    2 => GameButton::Right,
                    3 => GameButton::A,
                    _ => panic!("native action cursor"),
                }
            } else {
                GameButton::A
            }
        } else if !s.ui.text_window_open
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
            && c.shell.battle_messages.is_empty()
        {
            return;
        } else {
            GameButton::A
        };
        // The encounter intro can precede its action cursor. Wait for native
        // printing/presentation after every input, including battle inputs.
        press(c, button);
    }
    panic!("native wild encounter did not release input");
}

#[test]
#[ignore = "requires the current native Mille Lacs park via CRYSTAL_INTERIOR_PACK"]
fn mille_lacs_park_controller_roads_residents_grass_ledge_dock_and_continue() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, ROUTE, (15, 10));
    let original = flags(&c);
    step(
        &mut c,
        GameButton::Right,
        PARK,
        (0, 12),
        "real R18 road enters the park",
    );
    walk(&mut c, &r, (4, 13));
    save(&mut c, "entry");
    for (approach, actor, label) in [
        ((6, 13), (7, 13), "MN_ML_PARK_RANGERText"),
        ((16, 16), (17, 16), "MN_ML_PARK_BIRDERText"),
        ((26, 23), (27, 23), "MN_ML_PARK_ANGLERText"),
        ((12, 30), (11, 30), "MN_ML_PARK_PICNICText"),
        ((39, 10), (39, 9), "MN_ML_PARK_BOATERText"),
        ((2, 11), (2, 10), "MN_ML_PARK_ENTRYText"),
        ((24, 7), (24, 6), "MN_ML_PARK_WINDText"),
    ] {
        walk(&mut c, &r, approach);
        assert!(
            talk(&mut c, actor).contains(label),
            "original authored park conversation"
        );
    }
    for x in [16, 17] {
        walk(&mut c, &r, (x, 17));
        if x == 16 {
            save(&mut c, "ledge-lip");
        }
        for _ in 0..8 {
            if tile(&mut c) == (x, 20) {
                break;
            }
            press(&mut c, GameButton::Down);
            assert!(
                [(x, 17), (x, 18), (x, 20)].contains(&tile(&mut c)),
                "native hop may stand on its lip but never on its wall course"
            );
        }
        assert_eq!(tile(&mut c), (x, 20), "native short lookout return");
        if x == 16 {
            save(&mut c, "ledge-landing");
        }
        blocked(&mut c, GameButton::Up);
        assert!(
            walk(&mut c, &r, (x, 17)) >= 9,
            "ordinary uphill route remains useful"
        );
    }
    for (point, direction) in [
        ((24, 21), GameButton::Up),
        ((24, 24), GameButton::Down),
        ((27, 22), GameButton::Right),
    ] {
        walk(&mut c, &r, point);
        blocked(&mut c, direction);
    }
    for (label, point) in [
        ("lookout", (16, 16)),
        ("dock", (24, 23)),
        ("picnic", (12, 30)),
        ("launch", (19, 29)),
        ("north-shore", (38, 10)),
    ] {
        walk(&mut c, &r, point);
        save(&mut c, label);
    }
    assert_eq!(
        flags(&c),
        original,
        "park sightseeing grants no quest or rod result"
    );
    c = continue_save(root.clone(), r.clone(), &mut c);
    assert_eq!(flags(&c), original);
    walk(&mut c, &r, (0, 13));
    step(
        &mut c,
        GameButton::Left,
        ROUTE,
        (15, 11),
        "other real road lane returns",
    );
    step(
        &mut c,
        GameButton::Right,
        PARK,
        (0, 13),
        "other lane reenters",
    );
    walk(&mut c, &r, (0, 12));
    step(
        &mut c,
        GameButton::Left,
        ROUTE,
        (15, 10),
        "original road lane returns",
    );
    assert_eq!(flags(&c), original);
}

#[test]
#[ignore = "requires the current native park; initial legal regional Surf/badge fixture"]
fn mille_lacs_park_controller_surfs_to_roost_pickup_once_and_continues() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, PARK, (19, 29));
    partner(&mut c, &r);
    let original = flags(&c);
    let item = &r.data().items["PP_UP"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(item), 0);
    save(&mut c, "surf-launch");
    mount(&mut c, GameButton::Right, PARK, (20, 29));
    for (to, button) in [
        ((20, 32), GameButton::Down),
        ((38, 32), GameButton::Right),
        ((38, 29), GameButton::Up),
        ((37, 29), GameButton::Left),
    ] {
        while tile(&mut c) != to {
            let at = tile(&mut c);
            let next = match button {
                GameButton::Right => (at.0 + 1, at.1),
                GameButton::Down => (at.0, at.1 + 1),
                GameButton::Up => (at.0, at.1 - 1),
                GameButton::Left => (at.0 - 1, at.1),
                _ => unreachable!("lake direction"),
            };
            step(&mut c, button, PARK, next, "real lake course");
        }
    }
    assert_eq!(
        c.snapshot().expect("actual dismount").overworld.mode,
        MovementMode::Normal
    );
    save(&mut c, "roost");
    press(&mut c, GameButton::Left);
    assert_eq!(
        tile(&mut c),
        (37, 29),
        "uncollected roost item blocks its tile"
    );
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::Down);
    assert_eq!(tile(&mut c), (37, 29), "native item notice owns input");
    settle(&mut c, "roost PP Up notice");
    assert_eq!(c.shell.shell.session().state().bag.quantity(item), 1);
    let mut expected = original;
    expected.0.insert(REWARD.into());
    assert_eq!(flags(&c), expected);
    step(
        &mut c,
        GameButton::Left,
        PARK,
        (36, 29),
        "collected roost floor opens",
    );
    c = continue_save(root, r.clone(), &mut c);
    assert_eq!(tile(&mut c), (36, 29));
    assert_eq!(c.shell.shell.session().state().bag.quantity(item), 1);
    assert_eq!(flags(&c), expected);
    press(&mut c, GameButton::A);
    settle(&mut c, "old roost item cannot repeat");
    assert_eq!(c.shell.shell.session().state().bag.quantity(item), 1);
    save(&mut c, "roost-collected");
    step(
        &mut c,
        GameButton::Right,
        PARK,
        (37, 29),
        "return to roost landing",
    );
    mount(&mut c, GameButton::Right, PARK, (38, 29));
    for (to, button) in [
        ((38, 32), GameButton::Down),
        ((20, 32), GameButton::Left),
        ((20, 29), GameButton::Up),
        ((19, 29), GameButton::Left),
    ] {
        while tile(&mut c) != to {
            let at = tile(&mut c);
            let next = match button {
                GameButton::Right => (at.0 + 1, at.1),
                GameButton::Down => (at.0, at.1 + 1),
                GameButton::Up => (at.0, at.1 - 1),
                GameButton::Left => (at.0 - 1, at.1),
                _ => unreachable!("lake direction"),
            };
            step(&mut c, button, PARK, next, "native return paddle");
        }
    }
    assert_eq!(
        c.snapshot().expect("returned to launch").overworld.mode,
        MovementMode::Normal
    );
    assert_eq!(flags(&c), expected);
    walk(&mut c, &r, (4, 13));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["GOOD_ROD"]),
        0
    );
}

#[test]
#[ignore = "requires the native park; legal regional party fixture and real grass encounter"]
fn mille_lacs_park_controller_grass_has_regional_wild_battle_and_native_run() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, PARK, (13, 16));
    partner(&mut c, &r);
    let original = flags(&c);
    let table = r.data().wild_encounters[PARK]
        .grass
        .as_ref()
        .expect("park grass table");
    let mut observed = None;
    for _ in 0..1024 {
        if let Some(battle) = c.snapshot().expect("native grass check").battle {
            assert!(matches!(
                battle.kind,
                crystal_runtime::RuntimeBattleKind::Wild { .. }
            ));
            assert!(
                table
                    .morning
                    .iter()
                    .chain(&table.day)
                    .chain(&table.night)
                    .any(|slot| slot.species == battle.enemy_pokemon.species.id)
            );
            observed = Some(battle.enemy_pokemon.species.id);
            break;
        }
        let button = if tile(&mut c).0 == 13 {
            GameButton::Right
        } else {
            GameButton::Left
        };
        press(&mut c, button);
    }
    assert!(
        observed.is_some(),
        "actual encounter required, not just a painted grass patch"
    );
    run_from_wild(&mut c);
    if tile(&mut c).0 == 13 {
        step(
            &mut c,
            GameButton::Right,
            PARK,
            (14, 16),
            "native RUN then grass step",
        );
    }
    step(
        &mut c,
        GameButton::Left,
        PARK,
        (13, 16),
        "native RUN and ordinary return",
    );
    assert!(c.snapshot().expect("RUN released input").battle.is_none());
    assert_eq!(flags(&c), original);
    save(&mut c, "grass-after-run");
    eprintln!(
        "Observed native regional park encounter: {}",
        observed.expect("observed wild")
    );
}
