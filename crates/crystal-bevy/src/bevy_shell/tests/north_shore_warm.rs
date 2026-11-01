//! Legal initial Floor/Mew/HM/badge fixtures, followed only by production input.
//! Warm-coast regressions do not establish earned campaign or the new GM port.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::world::map::Direction;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::super::north_shore::climb_added_cascade;
use super::super::r10_gallery::flags;
use super::grand_marais_harbor::use_max_repel;
use super::{
    Point, VisibleShellController, at, collision, fixture_with_items_on_runtime,
    mark_runtime_snapshot_dirty, mount, settle, step,
};

const WARM: &str = "mn_north_shore";
const CONTRACT: &str = include_str!(
    "../../../../../planning/minnesota/revisions/north-shore-ely-geography-v01/r11-warm/source-v01/native.contract.json"
);

fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_NORTH_SHORE_WARM_CONTROLLER_PACK")
                    .expect("actual completed warm North Shore pack"),
            )
            .canonicalize()
            .expect("completed external warm coast pack");
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let r = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("verified composed warm pack"),
            )
            .expect("native warm coast runtime");
            let maps = r
                .data()
                .maps
                .iter()
                .filter(|(name, _)| name.starts_with("MnR11C"))
                .collect::<Vec<_>>();
            assert_eq!(maps.len(), 8, "all eight actual R11 chunks");
            for (_, m) in maps {
                assert_eq!(m.attributes.tileset_name, WARM);
                assert!(
                    m.blocks
                        .iter()
                        .all(|b| r.data().tilesets[WARM].collision[&format!("{b:02x}")]
                            .iter()
                            .all(|kind| kind != "ICE")),
                    "no active North Shore sliding ice"
                );
            }
            (root, r)
        })
        .clone()
}

fn resolve(point: Point) -> (String, Point) {
    (
        format!("MnR11C{:02}R{:02}", point.0 / 64, point.1 / 48),
        (point.0 % 64, point.1 % 48),
    )
}

fn global(c: &mut VisibleShellController) -> Point {
    let (name, local) = at(c);
    let (_, chunk) = name.split_once('C').expect("remain on actual North Shore");
    let (column, row) = chunk.split_once('R').expect("native R11 chunk origin");
    (
        column.parse::<i16>().expect("column") * 64 + local.0,
        row.parse::<i16>().expect("row") * 48 + local.1,
    )
}

fn kind(r: &CrystalRuntime, point: Point) -> String {
    let (name, local) = resolve(point);
    let m = &r.data().maps[&name];
    let b = m.blocks[usize::from(local.1 as u16 / 2) * usize::from(m.attributes.width)
        + usize::from(local.0 as u16 / 2)];
    r.data().tilesets[WARM].collision[&format!("{b:02x}")][(local.1 % 2 * 2 + local.0 % 2) as usize]
        .clone()
}

fn fixture(
    point: Point,
    rising: bool,
    waterfall: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (name, local) = resolve(point);
    let (root, r, mut c) = fixture_with_items_on_runtime(
        load(),
        &name,
        local,
        false,
        rising,
        waterfall,
        &[("MAX_REPEL", 2)],
    );
    assert_eq!(
        collision(&r, &mut c, local),
        "FLOOR",
        "legal initial Floor fixture"
    );
    use_max_repel(&mut c, &r);
    (root, r, c)
}

fn direction(from: Point, to: Point) -> GameButton {
    match (to.0 - from.0, to.1 - from.1) {
        (0, -1) => GameButton::Up,
        (0, 1) => GameButton::Down,
        (-1, 0) => GameButton::Left,
        (1, 0) => GameButton::Right,
        other => panic!("nonadjacent authored warm input: {from:?}->{to:?}: {other:?}"),
    }
}

fn travel(c: &mut VisibleShellController, target: Point) {
    let key = direction(global(c), target);
    let (map, local) = resolve(target);
    step(c, key, &map, local, "authored warm coast input");
    assert!(
        c.snapshot().expect("native warm travel").battle.is_none(),
        "avoid original trainer sight"
    );
}

fn foot_line(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    target: Point,
    visited: &mut BTreeSet<Point>,
) {
    let mut p = global(c);
    assert!(p.0 == target.0 || p.1 == target.1);
    while p != target {
        p.0 += (target.0 - p.0).signum();
        p.1 += (target.1 - p.1).signum();
        assert!(matches!(kind(r, p).as_str(), "FLOOR" | "TALL_GRASS"));
        travel(c, p);
        c.wait_frames(96)
            .expect("ordinary released-input stop, beyond old forced-slide duration");
        assert_eq!(
            global(c),
            p,
            "one press must stop on each former ice quadrant"
        );
        assert_eq!(
            c.snapshot().expect("ordinary warm Foot").overworld.mode,
            MovementMode::Normal
        );
        visited.insert(p);
    }
}

fn blocked(c: &mut VisibleShellController, key: GameButton, mode: MovementMode) {
    let point = global(c);
    for _ in 0..2 {
        c.press(key).expect("native permanent barrier input");
        c.wait_frames(16).expect("blocked edge presentation");
        settle(c);
        assert_eq!(global(c), point);
        assert_eq!(
            c.snapshot().expect("blocked traversal").overworld.mode,
            mode
        );
    }
}

fn capture(c: &mut VisibleShellController, label: &str) -> PathBuf {
    let dir = std::env::var_os("CRYSTAL_NORTH_SHORE_WARM_SAVE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("geothite-north-shore-warm"));
    std::fs::create_dir_all(&dir).expect("retained warm-coast saves");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("save clock")
        .as_nanos();
    let path = dir.join(format!(
        "{label}-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    assert!(!path.exists(), "retain earlier native save epochs");
    c.save(&path).expect("production warm coast Save");
    path
}

fn continue_game(
    root: &AssetRoot,
    r: &CrystalRuntime,
    c: &mut VisibleShellController,
    label: &str,
) {
    let before = c.snapshot().expect("warm Continue preimage");
    let original = flags(c);
    let path = capture(c, label);
    *c = VisibleShellController::load_save(root.clone(), r.clone(), path, None)
        .expect("native warm Continue");
    let after = c.snapshot().expect("continued warm coast");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.mode, after.overworld.mode);
    assert_eq!(before.overworld.facing, after.overworld.facing);
    assert_eq!(original, flags(c));
}

#[test]
#[ignore = "requires CRYSTAL_NORTH_SHORE_WARM_CONTROLLER_PACK; explicit local fixtures"]
fn north_shore_warm_controller_all52_former_ice_quadrants_stop_after_each_foot_input() {
    assert_eq!(
        format!("{:x}", Sha256::digest(CONTRACT.as_bytes())),
        "ad3d97771d1bd0c2e4bcba253a3dba7e060ee7996f8e723527796644fdff0976"
    );
    let contract: Value = serde_json::from_str(CONTRACT).expect("frozen warm contract");
    let mut former = BTreeSet::new();
    for row in contract["modules"].as_array().expect("native chunks") {
        for cell in row["cells"].as_array().expect("exact restoration cells") {
            if cell["before"] != 0x85 {
                continue;
            }
            let x = cell["source_metatile"][0].as_i64().expect("global x") as i16 * 2;
            let y = cell["source_metatile"][1].as_i64().expect("global y") as i16 * 2;
            for dx in 0..2 {
                for dy in 0..2 {
                    former.insert((x + dx, y + dy));
                }
            }
        }
    }
    assert_eq!(former.len(), 52);
    let courses: &[(&str, Point, &[Point])] = &[
        ("warm-northern-lane", (40, 8), &[(40, 1), (41, 1), (41, 8)]),
        (
            "warm-upper-bank",
            (34, 24),
            &[
                (34, 17),
                (35, 17),
                (35, 24),
                (36, 24),
                (36, 17),
                (37, 17),
                (37, 24),
            ],
        ),
        (
            "warm-shore-cove",
            (43, 30),
            &[
                (48, 30),
                (48, 31),
                (43, 31),
                (44, 31),
                (44, 32),
                (48, 32),
                (48, 33),
                (44, 33),
                (44, 34),
            ],
        ),
    ];
    let mut visited = BTreeSet::new();
    for (label, start, turns) in courses {
        let (root, r, mut c) = fixture(*start, false, false);
        let before = c.snapshot().expect("warm Foot preimage");
        let original = flags(&c);
        for target in *turns {
            foot_line(&mut c, &r, *target, &mut visited);
        }
        continue_game(&root, &r, &mut c, label);
        assert_eq!(
            before.party,
            c.snapshot().expect("ordinary Foot preserves partner").party
        );
        assert_eq!(
            before.bag,
            c.snapshot().expect("ordinary Foot preserves bag").bag
        );
        assert_eq!(original, flags(&c));
    }
    assert!(
        former.is_subset(&visited),
        "every formerly sliding quadrant was entered and stopped on through real inputs"
    );
}

#[test]
#[ignore = "requires CRYSTAL_NORTH_SHORE_WARM_CONTROLLER_PACK; explicit local fixture"]
fn north_shore_warm_controller_original_green_grass_access_and_continue() {
    let (root, r, mut c) = fixture((44, 30), false, false);
    let original = flags(&c);
    let before = c.snapshot().expect("grass preimage");
    let mut visited = BTreeSet::new();
    for target in [(44, 29), (46, 29)] {
        foot_line(&mut c, &r, target, &mut visited);
    }
    assert_eq!(kind(&r, (46, 29)), "TALL_GRASS");
    let (map, local) = resolve((46, 29));
    let m = &r.data().maps[&map];
    assert_eq!(
        m.blocks[usize::from(local.1 as u16 / 2) * usize::from(m.attributes.width)
            + usize::from(local.0 as u16 / 2)],
        3
    );
    continue_game(&root, &r, &mut c, "warm-restored-green-grass");
    for target in [(47, 29), (44, 29), (44, 30)] {
        foot_line(&mut c, &r, target, &mut visited);
    }
    assert!((44..=47).all(|x| visited.contains(&(x, 29))));
    assert_eq!(
        before.party,
        c.snapshot().expect("native grass partner unchanged").party
    );
    assert_eq!(
        before.bag,
        c.snapshot().expect("native grass bag unchanged").bag
    );
    assert_eq!(original, flags(&c));
    capture(&mut c, "warm-grass-return");
}

#[test]
#[ignore = "requires CRYSTAL_NORTH_SHORE_WARM_CONTROLLER_PACK; explicit local fixtures"]
fn north_shore_warm_controller_stairs_both_directions_and_palisade_one_way_return() {
    let lanes: &[(&str, &[Point], Point, GameButton)] = &[
        (
            "warm-palisade-stair",
            &[(26, 48), (25, 48), (24, 48), (24, 49), (23, 49)],
            (25, 48),
            GameButton::Down,
        ),
        (
            "warm-birch-stair",
            &[(37, 89), (37, 90), (37, 91), (37, 92)],
            (37, 91),
            GameButton::Left,
        ),
        (
            "warm-lester-stair",
            &[(40, 170), (39, 170), (38, 170), (37, 170)],
            (38, 170),
            GameButton::Down,
        ),
    ];
    for (label, path, side, key) in lanes {
        let (root, r, mut c) = fixture(path[0], false, false);
        let original = flags(&c);
        let mut visited = BTreeSet::new();
        for p in path.iter().skip(1) {
            foot_line(&mut c, &r, *p, &mut visited);
        }
        continue_game(&root, &r, &mut c, label);
        for p in path.iter().rev().skip(1) {
            foot_line(&mut c, &r, *p, &mut visited);
        }
        for p in path.iter().skip(1) {
            foot_line(&mut c, &r, *p, &mut visited);
            if p == side {
                blocked(&mut c, *key, MovementMode::Normal);
            }
        }
        assert_eq!(original, flags(&c));
    }
    let (root, r, mut c) = fixture((23, 49), false, false);
    let original = flags(&c);
    let mut visited = BTreeSet::new();
    for p in [(23, 51), (18, 51)] {
        foot_line(&mut c, &r, p, &mut visited);
    }
    assert_eq!(kind(&r, (18, 52)), "HOP_DOWN");
    travel(&mut c, (18, 52));
    for _ in 0..8 {
        if global(&mut c) == (18, 54) {
            break;
        }
        c.press(GameButton::Down)
            .expect("native one-way Palisade hop");
        c.wait_frames(16).expect("native hop presentation");
        settle(&mut c);
        assert_ne!(global(&mut c), (18, 53), "forbidden ledge reverse quadrant");
    }
    assert_eq!(global(&mut c), (18, 54));
    blocked(&mut c, GameButton::Up, MovementMode::Normal);
    continue_game(&root, &r, &mut c, "warm-palisade-ledge-landing");
    let mut return_steps = 0;
    for p in [
        (28, 54),
        (28, 49),
        (26, 49),
        (26, 48),
        (24, 48),
        (24, 51),
        (18, 51),
    ] {
        let before = global(&mut c);
        return_steps += (p.0 - before.0).abs() + (p.1 - before.1).abs();
        foot_line(&mut c, &r, p, &mut visited);
    }
    assert_eq!(
        return_steps, 29,
        "preserved useful return through actual stair/ridge lanes"
    );
    assert_eq!(original, flags(&c));
    capture(&mut c, "warm-palisade-return-upper");
}

fn cascade_base(c: &mut VisibleShellController) {
    let (map, local) = resolve((21, 27));
    mount(c, GameButton::Left, &map, local);
    for p in [(20, 27), (19, 27), (18, 27), (18, 26)] {
        travel(c, p);
    }
    if c.snapshot().expect("cascade facing").overworld.facing != Direction::Up {
        c.press(GameButton::Up).expect("face native uphill cascade");
        settle(c);
    }
    assert_eq!(global(c), (18, 26));
}

#[test]
#[ignore = "requires CRYSTAL_NORTH_SHORE_WARM_CONTROLLER_PACK; local HM/badge fixtures"]
fn north_shore_warm_controller_waterfall_move_badge_denials_and_qualified_return() {
    for (rising, known) in [(true, false), (false, true)] {
        let (_, r, mut c) = fixture((22, 27), rising, known);
        cascade_base(&mut c);
        assert_eq!(kind(&r, (18, 25)), "WATERFALL");
        let before = c.snapshot().expect("Waterfall refusal preimage");
        let expected = before
            .presentation
            .asm_text_for_label(".HugeWaterfallText@Script_CantDoWaterfall")
            .expect("native refusal text")
            .to_owned();
        c.press(GameButton::A)
            .expect("unqualified native uphill use");
        assert_eq!(c.shell.field_notice.as_deref(), Some(expected.as_str()));
        c.press(GameButton::Down)
            .expect("refusal notice owns movement");
        assert_eq!(global(&mut c), (18, 26));
        assert!(!c.shell.flygon_field_actions.contains("waterfall"));
        assert_eq!(
            before.party,
            c.snapshot().expect("denial partner preserved").party
        );
        settle(&mut c);
        capture(
            &mut c,
            if rising {
                "warm-falls-move-denial"
            } else {
                "warm-falls-badge-denial"
            },
        );
    }
    let (root, r, mut c) = fixture((22, 27), true, true);
    let original = flags(&c);
    cascade_base(&mut c);
    climb_added_cascade(&mut c, 18, 26, 19);
    assert_eq!(global(&mut c), (18, 19));
    assert_eq!(kind(&r, (18, 19)), "WATER");
    assert!(c.shell.flygon_field_actions.contains("waterfall"));
    continue_game(&root, &r, &mut c, "warm-baptism-falls-summit");
    for _ in 0..64 {
        if global(&mut c) == (18, 26) {
            break;
        }
        c.press(GameButton::Down)
            .expect("retained native downward cascade current");
        c.wait_frames(16).expect("downward current clock");
        settle(&mut c);
        assert_eq!(global(&mut c).0, 18);
    }
    assert_eq!(global(&mut c), (18, 26));
    for p in [(18, 27), (19, 27), (20, 27), (21, 27), (22, 27)] {
        travel(&mut c, p);
    }
    assert_eq!(
        c.snapshot().expect("native dry bank return").overworld.mode,
        MovementMode::Normal
    );
    assert_eq!(original, flags(&c));
    continue_game(&root, &r, &mut c, "warm-baptism-falls-bank-return");
}

#[test]
#[ignore = "requires CRYSTAL_NORTH_SHORE_WARM_CONTROLLER_PACK; local Surf/Fog fixture"]
fn north_shore_warm_controller_superior_buoy_blocks_surf_and_legal_bank_return() {
    let (root, r, mut c) = fixture((49, 188), false, false);
    let original = flags(&c);
    let (map, local) = resolve((49, 187));
    mount(&mut c, GameButton::Up, &map, local);
    for x in 50..=58 {
        travel(&mut c, (x, 187));
    }
    for y in 188..=189 {
        travel(&mut c, (58, y));
    }
    assert_eq!(kind(&r, (58, 190)), "BUOY");
    blocked(&mut c, GameButton::Down, MovementMode::Surf);
    continue_game(&root, &r, &mut c, "warm-superior-southern-visible-buoy");
    for y in (187..189).rev() {
        travel(&mut c, (58, y));
    }
    for x in (49..58).rev() {
        travel(&mut c, (x, 187));
    }
    travel(&mut c, (49, 188));
    assert_eq!(
        c.snapshot()
            .expect("actual Superior dismount")
            .overworld
            .mode,
        MovementMode::Normal
    );
    assert_eq!(original, flags(&c));
    continue_game(&root, &r, &mut c, "warm-superior-ordinary-bank-return");
}

#[test]
#[ignore = "requires completed coastal-coves pack; Floor/Repel/prior-defeated R11 trainer fixtures"]
fn north_shore_coves_controller_walks_three_grass_capes_and_continues_at_water_views() {
    let courses: &[(&str, Point, &[Point])] = &[
        ("split-rock-cape", (40, 66), &[(58, 66), (58, 68)]),
        ("sheltered-middle-cape", (34, 136), &[(60, 136)]),
        ("southern-pine-cape", (48, 178), &[(64, 178)]),
    ];
    for (label, start, turns) in courses {
        let (root, r, mut c) = fixture(*start, false, false);
        // Reed's existing south-facing one-tile sight at (43,65) crosses
        // this course at (43,66): this is an ordinary native Trainer battle,
        // not a wild/static encounter. This scoped terrain fixture represents
        // a return visit after the route's trainers were already defeated.
        // It does not claim earned battle or regional progression.
        let reed = &r.data().maps["MnR11C00R01"].trainer_scripts["MN_R11_T05"];
        assert_eq!(reed.battle_type, "BATTLETYPE_TRAINER");
        assert_eq!(reed.trainer_id, "MN_R11_T05");
        assert_eq!(reed.event_flag, "EVENT_MN_R11_TRAINER_05");
        let defeated = r
            .data()
            .maps
            .iter()
            .filter(|(name, _)| name.starts_with("MnR11C"))
            .flat_map(|(_, module)| {
                module
                    .trainer_scripts
                    .values()
                    .map(|trainer| trainer.event_flag.clone())
            })
            .collect::<BTreeSet<_>>();
        assert!(!defeated.is_empty(), "actual native route trainer fixture");
        for flag in defeated {
            c.shell
                .shell
                .session_mut()
                .state_mut()
                .flags
                .set_event_flag(&flag, true)
                .expect("explicit initial prior-defeated coast trainer fixture");
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        // All fixture writes end here; traversal, Save and Continue below use
        // production controller inputs and must preserve these starting flags.
        let original = flags(&c);
        let before = c.snapshot().expect("legal cape fixture");
        let mut visited = BTreeSet::new();
        let mut stops = vec![*start];
        for target in *turns {
            foot_line(&mut c, &r, *target, &mut visited);
            stops.push(*target);
        }
        assert!(
            visited.iter().any(|&p| kind(&r, p) == "TALL_GRASS"),
            "{label}: native grass saddle was actually crossed"
        );
        let tip = global(&mut c);
        assert!(
            (-5..=5).any(|dx| (-4..=4).any(|dy| kind(&r, (tip.0 + dx, tip.1 + dy)) == "WATER")),
            "{label}: actual Superior must appear in the native player camera"
        );
        continue_game(&root, &r, &mut c, label);
        for target in stops.iter().rev().skip(1) {
            foot_line(&mut c, &r, *target, &mut visited);
        }
        assert_eq!(global(&mut c), *start);
        assert_eq!(original, flags(&c), "coastal spur changes no story gates");
        let after = c.snapshot().expect("cape return");
        assert_eq!(before.party, after.party);
        assert_eq!(before.bag, after.bag);
    }
}
