//! Local empty-party/position/RNG setup, then only production controller input.
//! This proves the optional room contract, not earned Border Pines progression.
use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleFieldItemPhase, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, RouteGrid, RouteSpec, indoor_walk, neighbors};
use super::r10_gallery::{
    completed_visible_page, continue_save, flags, map, press, remember_dialogue, settle, tile,
};

const ROOM: &str = "MnR17Cave1F";
const OUTSIDE: &str = "MnR17C00R00";
const EVENT: &str = "EVENT_MN_R17_ITEM_04";
fn source() -> Value {
    let path=std::env::var_os("CRYSTAL_R17_TOR_SOURCE").map(PathBuf::from).unwrap_or_else(||
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/renders/caves/r17-candidate/f590d69e806a4a9e/MnR17Cave1F.candidate.json"));
    serde_json::from_str(&std::fs::read_to_string(path).expect("frozen R17 tor source"))
        .expect("room source")
}
fn point(value: &Value) -> Point {
    (
        value[0].as_i64().expect("x") as i16,
        value[1].as_i64().expect("y") as i16,
    )
}
fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_R17_TOR_CONTROLLER_PACK")
                    .expect("actual revised R17 native pack"),
            );
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("strict revised native pack"),
            )
            .expect("native runtime");
            (root, runtime)
        })
        .clone()
}
fn fixture() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        r.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: r.title_new_game_spawn_identifier().expect("retained HOME"),
            map_name: OUTSIDE.into(),
            tile_x: 10,
            tile_y: 44,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local passive mouth fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("initial local trainer name");
    c.shell.shell.session_mut().state_mut().random_state = Default::default();
    *c.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    // Initial setup ends here: empty native party, local position and RNG only.
    // No later party, inventory, badge, flag, money or position writes.
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert!(
        c.snapshot()
            .expect("honest empty geometry party")
            .party
            .slots
            .is_empty()
    );
    assert_eq!(map(&mut c), OUTSIDE);
    assert_eq!(tile(&mut c), (10, 44));
    assert!(!collected(&c));
    let s = source();
    let m = &r.data().maps[ROOM];
    let blocks = s["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .flat_map(|row| row.as_str().expect("row").split_whitespace())
        .map(|v| u16::from_str_radix(v, 16).expect("native block"))
        .collect::<Vec<_>>();
    assert_eq!(
        m.blocks, blocks,
        "actual pack must match frozen fitted tongue source"
    );
    assert_eq!([m.attributes.width, m.attributes.height], [7, 8]);
    assert_eq!(m.attributes.time_of_day.as_deref(), Some("nite"));
    assert_eq!(m.attributes.palette.as_deref(), Some("PALETTE_NITE"));
    assert_eq!(m.objects.len(), 1, "original Revive only, no invented NPC");
    (root, r, c)
}
fn collected(c: &VisibleShellController) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(EVENT)
        .expect("native original item flag")
}
fn quantity(c: &VisibleShellController, r: &CrystalRuntime) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items["REVIVE"])
}
fn kind(r: &CrystalRuntime, p: Point) -> &str {
    let m = &r.data().maps[ROOM];
    let b = m.blocks[p.1 as usize / 2 * 7 + p.0 as usize / 2];
    &r.data().tilesets["cave"].collision[&format!("{b:02x}")]
        [p.1 as usize % 2 * 2 + p.0 as usize % 2]
}
fn direction(a: Point, b: Point) -> GameButton {
    neighbors(a)
        .into_iter()
        .find_map(|(p, input)| (p == b).then_some(input))
        .expect("one adjacent step")
}
fn step(c: &mut VisibleShellController, input: GameButton, to: Point) {
    for _ in 0..8 {
        if tile(c) == to {
            return;
        }
        press(c, input);
    }
    assert_eq!(tile(c), to);
}
fn blocked(c: &mut VisibleShellController, input: GameButton) {
    let at = tile(c);
    for _ in 0..4 {
        press(c, input);
        assert_eq!(tile(c), at);
    }
}
fn outside(r: &CrystalRuntime) -> RouteGrid {
    RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix: "MnR17",
            columns: 1,
            rows: 3,
            tileset: "mn_border_pines",
            start: (10, 44),
            doors: BTreeMap::from([((10, 43), ROOM), ((45, 49), "MnR17RangerHut")]),
        },
    )
}
fn enter(c: &mut VisibleShellController, r: &CrystalRuntime) {
    outside(r).walk(c, (10, 43));
    assert_eq!(map(c), ROOM);
    assert_eq!(tile(c), (9, 13));
    indoor_walk(c, r, (9, 12));
    assert_eq!(tile(c), (9, 12));
}
fn leave(c: &mut VisibleShellController, r: &CrystalRuntime) {
    indoor_walk(c, r, (9, 12));
    indoor_walk(c, r, (9, 13));
    assert_eq!(map(c), OUTSIDE);
    assert_eq!(tile(c), (10, 44));
    c.wait_frames(60)
        .expect("original passive Floor stays stable");
    assert_eq!(map(c), OUTSIDE);
    assert_eq!(tile(c), (10, 44));
    let g = outside(r);
    g.walk(c, (10, 45));
    g.walk(c, (10, 44));
    assert_eq!(map(c), OUTSIDE);
    assert_eq!(tile(c), (10, 44));
}
fn collect(c: &mut VisibleShellController, r: &CrystalRuntime) {
    indoor_walk(c, r, (5, 6));
    press(c, GameButton::Up);
    assert_eq!(tile(c), (5, 6), "original uncollected item occupies5,5");
    // Production press settles the automatic Found printer and60-frame fanfare
    // internally before returning at the first PocketText A-wait. Retained
    // notice data proves its native item payload; only Pocket pages are claimed
    // as actually observed here. Do not alter the shared production clock.
    c.press(GameButton::A).expect("native Revive interaction");
    assert!(collected(c));
    assert_eq!(quantity(c, r), 1);
    let at = tile(c);
    c.press(GameButton::Down)
        .expect("native item movement lock");
    assert_eq!(tile(c), at, "native successful-item notice owns movement");
    let name = r.data().items["REVIVE"].name.replace('_', " ");
    let notice = c
        .shell
        .visible_field_item_notice
        .as_ref()
        .expect("native retained item notice");
    assert_eq!(notice.sound_trigger_text, format!("CHRIS found\n{name}!"));
    assert_eq!(notice.phase, VisibleFieldItemPhase::PocketText);
    assert!(matches!(
        notice.presentation,
        super::super::VisibleFieldItemPresentation::ItemBall
    ));
    let mut pocket = BTreeMap::new();
    let mut diagnostics = VecDeque::new();
    for _ in 0..1280 {
        remember_dialogue(c, &mut diagnostics);
        let phase = c
            .shell
            .visible_field_item_notice
            .as_ref()
            .map(|n| n.phase.clone());
        if let Some((index, page)) = completed_visible_page(c) {
            match phase {
                Some(VisibleFieldItemPhase::PocketText) => {
                    pocket.insert(index, page);
                    let at = tile(c);
                    press(c, GameButton::Down);
                    assert_eq!(
                        tile(c),
                        at,
                        "fully printed pocket page retains movement ownership"
                    );
                    press(c, GameButton::A);
                }
                other => panic!("unexpected complete native item phase{other:?}: {diagnostics:#?}"),
            }
        }
        if c.shell.field_notice.is_none()
            && c.shell.visible_field_item_notice.is_none()
            && !c.has_pending_script_work()
        {
            break;
        }
        c.wait_frames(1).expect("one native printer/fanfare frame");
    }
    assert_eq!(
        pocket.get(&0),
        Some(&format!("CHRIS put the\n{name} in")),
        "{diagnostics:#?}"
    );
    assert_eq!(
        pocket.get(&1),
        Some(&format!("{name} in\nthe ITEM POCKET.")),
        "{diagnostics:#?}"
    );
    settle(c, "after actual Revive found/fanfare/two pocket pages");
}

#[test]
#[ignore = "requires actual revised R17 tor pack and frozen f590 source; empty-party local fixture only"]
fn r17_tor_cave_controller_bent_route_directional_roof_revive_once_and_continue() {
    let (root, r, mut c) = fixture();
    let initial_flags = flags(&c);
    let initial_money = c.shell.shell.session().state().money;
    assert_eq!(quantity(&c, &r), 0);
    enter(&mut c, &r);
    let s = source();
    let probes = &s["review_contract"]["controller_probes"];
    let route = probes["ordinary_route_core"]
        .as_array()
        .expect("short fitted bend")
        .iter()
        .map(point)
        .collect::<Vec<_>>();
    assert_eq!(route.len() - 1, 10);
    assert_eq!(tile(&mut c), route[0]);
    for &p in route.iter().skip(1) {
        assert_eq!(kind(&r, p), "FLOOR");
        let at = tile(&mut c);
        step(&mut c, direction(at, p), p);
    }
    // Native UP_WALL is an enterable directional roof edge, not solid terrain.
    indoor_walk(&mut c, &r, (5, 3));
    for p in [(5, 2), (6, 2)] {
        assert_eq!(kind(&r, p), "UP_WALL");
        let at = tile(&mut c);
        step(&mut c, direction(at, p), p);
    }
    assert_eq!(kind(&r, (6, 3)), "FLOOR");
    step(&mut c, GameButton::Down, (6, 3));
    indoor_walk(&mut c, &r, (3, 5));
    assert_eq!(kind(&r, (3, 4)), "UP_WALL");
    step(&mut c, GameButton::Up, (3, 4));
    blocked(&mut c, GameButton::Up);
    step(&mut c, GameButton::Down, (3, 5));
    collect(&mut c, &r);
    assert_eq!(quantity(&c, &r), 1);
    let mut expected_flags = initial_flags;
    expected_flags.0.insert(EVENT.into());
    assert_eq!(flags(&c), expected_flags);
    leave(&mut c, &r);
    enter(&mut c, &r);
    indoor_walk(&mut c, &r, (5, 6));
    step(&mut c, GameButton::Up, (5, 5));
    assert_eq!(
        tile(&mut c),
        (5, 5),
        "collected native object tile is now walkable"
    );
    press(&mut c, GameButton::A);
    settle(&mut c, "repeat collected Revive tile");
    assert_eq!(quantity(&c, &r), 1);
    assert_eq!(flags(&c), expected_flags);
    c = continue_save(root, r.clone(), &mut c);
    assert_eq!(tile(&mut c), (5, 5));
    assert!(collected(&c));
    assert_eq!(quantity(&c, &r), 1);
    assert_eq!(flags(&c), expected_flags);
    assert_eq!(c.shell.shell.session().state().money, initial_money);
    step(&mut c, GameButton::Down, (5, 6));
    leave(&mut c, &r);
}
