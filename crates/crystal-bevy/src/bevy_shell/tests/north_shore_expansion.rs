//! Expanded-coast production input. Party/badges/position/RNG below are explicit
//! local fixtures, not earned regional progress. All later mutations use press.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::script_flags::{ScriptFlagCommand, apply_script_flag_mutation};
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{
    Point, RouteGrid, RouteSpec, finish_text, indoor_walk, neighbors, opposite, position,
};
use super::north_shore::{climb_added_cascade, mount_surf, settle_water, water_step};

fn source() -> Value {
    let path = std::env::var_os("CRYSTAL_NORTH_SHORE_EXPANSION_DOSSIER")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../planning/minnesota/revisions/northwoods-expansion/after/MN_R11.json")
        });
    serde_json::from_str(
        &std::fs::read_to_string(path).expect("same frozen dossier as native publication"),
    )
    .expect("frozen expanded coast")
}
fn point(v: &Value) -> Point {
    (
        v[0].as_i64().expect("core x") as i16,
        v[1].as_i64().expect("core y") as i16,
    )
}
fn map(at: Point) -> String {
    format!("MnR11C{:02}R{:02}", at.0 / 64, at.1 / 48)
}
fn load() -> (AssetRoot, CrystalRuntime) {
    // Immutable pack data and its environment path are captured once per process.
    // Every call still creates a separate gameplay session in fixture().
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_NORTH_SHORE_EXPANSION_CONTROLLER_PACK")
                    .expect("actual published expanded North Shore pack"),
            );
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("verified actual pack"),
            )
            .expect("native runtime");
            (root, runtime)
        })
        .clone()
}
fn fixture(at: Point, populated: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("retained HOME"),
            map_name: map(at),
            tile_x: at.0 % 64,
            tile_y: at.1 % 48,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local geometry fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture name");
    if populated {
        let mut pokemon = runtime
            .data()
            .create_pokemon("MEW", 80, Dv::default())
            .expect("legal native HM-compatible fixture");
        pokemon.moves = ["SURF", "WATERFALL"]
            .into_iter()
            .map(|name| LearnedMove {
                name: name.into(),
                current_pp: runtime.data().moves[name].pp,
                pp_ups: 0,
            })
            .collect();
        let state = c.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(pokemon));
        state.sync_party_from_storage();
        for (index, flag) in [
            "ENGINE_ZEPHYRBADGE",
            "ENGINE_HIVEBADGE",
            "ENGINE_PLAINBADGE",
            "ENGINE_FOGBADGE",
            "ENGINE_STORMBADGE",
            "ENGINE_MINERALBADGE",
            "ENGINE_GLACIERBADGE",
            "ENGINE_RISINGBADGE",
        ]
        .into_iter()
        .enumerate()
        {
            apply_script_flag_mutation(
                state,
                ScriptFlagCommand {
                    command: "setflag".into(),
                    flag_id: flag.into(),
                    source_script: "ExpandedCoastLocalFixture".into(),
                    command_index: index,
                },
            )
            .expect("consistent explicit badge setup");
        }
        state.random_state = Default::default();
        *c.shell.shell.session_mut().divider_mut_for_tests() =
            crystal_core::random::RuntimeDividerSource::replay(
                (0_u32..1048576)
                    .map(|n| ((n * 73 + 19) % 256) as u8)
                    .collect::<Vec<_>>(),
            );
    }
    // No state, position, badge, party or RNG writes after this setup boundary.
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert_eq!(position(&mut c), at);
    (root, runtime, c)
}
fn grid(runtime: &CrystalRuntime, start: Point) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnR11",
            columns: 2,
            rows: 4,
            tileset: "mn_north_shore",
            start,
            doors: BTreeMap::from([((51, 99), "MnR11SplitRockLighthouse")]),
        },
    )
}
fn block(runtime: &CrystalRuntime, at: Point) -> u16 {
    let m = &runtime.data().maps[&map(at)];
    m.blocks[usize::from((at.1 % 48) as u16 / 2) * usize::from(m.attributes.width)
        + usize::from((at.0 % 64) as u16 / 2)]
}
fn kind(runtime: &CrystalRuntime, at: Point) -> &str {
    &runtime.data().tilesets["mn_north_shore"].collision[&format!("{:02x}", block(runtime, at))]
        [(at.1 % 2 * 2 + at.0 % 2) as usize]
}
fn direction(from: Point, to: Point) -> GameButton {
    neighbors(from)
        .into_iter()
        .find_map(|(p, b)| (p == to).then_some(b))
        .expect("adjacent controller step")
}
fn barrier(c: &mut VisibleShellController, button: GameButton) {
    let at = position(c);
    for _ in 0..4 {
        c.press(button).expect("native blocked input");
        settle_water(c);
        assert_eq!(position(c), at);
    }
}
fn native_hop(
    c: &mut VisibleShellController,
    upper: Point,
    from: Point,
    landing: Point,
    forbidden: Point,
    button: GameButton,
) {
    assert_eq!(position(c), upper);
    assert_eq!(direction(upper, from), button);
    for _ in 0..8 {
        if position(c) == from {
            break;
        }
        c.press(button)
            .expect("step onto actual native hop quadrant");
        settle_water(c);
        assert_ne!(position(c), forbidden);
    }
    assert_eq!(position(c), from);
    for _ in 0..8 {
        if position(c) == landing {
            break;
        }
        c.press(button).expect("real native ledge hop");
        settle_water(c);
        assert_ne!(position(c), forbidden);
    }
    assert_eq!(position(c), landing);
}
fn flags(c: &VisibleShellController) -> (BTreeSet<String>, BTreeSet<String>) {
    let f = &c.shell.shell.session().state().flags;
    (
        f.active_event_flags().cloned().collect(),
        f.engine_flags
            .iter()
            .filter_map(|(k, v)| v.then_some(k.clone()))
            .collect(),
    )
}

#[test]
#[ignore = "requires completed expanded North Shore pack and matching frozen dossier"]
fn expanded_north_shore_controller_walks_stairs_three_ledges_grass_and_keeper_returns() {
    let d = source();
    let start = point(&d["endpoints"][1]["landing_metatile"]);
    let start = (start.0 * 2, start.1 * 2);
    let (_, runtime, mut c) = fixture(start, false);
    let g = grid(&runtime, start);
    assert!(
        c.snapshot()
            .expect("empty party geometry fixture")
            .party
            .slots
            .is_empty()
    );
    for e in d["endpoints"].as_array().expect("ports") {
        let p = point(&e["gate_metatile"]);
        g.walk(&mut c, (p.0 * 2, p.1 * 2));
    }
    let mut native_stairs = BTreeSet::new();
    for row in 0..4 {
        for col in 0..2 {
            let m = &runtime.data().maps[&format!("MnR11C{col:02}R{row:02}")];
            for (index, b) in m.blocks.iter().enumerate() {
                if matches!(*b, 0x81 | 0x82) {
                    native_stairs.insert((
                        col * 64 + (index % usize::from(m.attributes.width)) as i16 * 2,
                        row * 48 + (index / usize::from(m.attributes.width)) as i16 * 2,
                    ));
                }
            }
        }
    }
    let lanes = d["expansion_contract"]["controller_probes"]["stair_lanes"]
        .as_array()
        .expect("authored native 12/30 stair lanes");
    assert_eq!(lanes.len(), 3);
    let mut traversed_stairs = BTreeSet::new();
    for lane in lanes {
        let expected_block = u16::from_str_radix(lane["block"].as_str().expect("stair art"), 16)
            .expect("native derived stair block");
        assert!(matches!(expected_block, 0x81 | 0x82));
        let path = std::iter::once(point(&lane["before"]))
            .chain(
                lane["steps"]
                    .as_array()
                    .expect("ordinary lane")
                    .iter()
                    .map(point),
            )
            .chain(std::iter::once(point(&lane["after"])))
            .collect::<Vec<_>>();
        for (index, &at) in path.iter().enumerate() {
            assert_eq!(kind(&runtime, at), "FLOOR", "{} {at:?}", lane["id"]);
            if index > 0 {
                assert_eq!(
                    g.path(path[index - 1], at).len(),
                    1,
                    "adjacent ordinary stair step"
                );
            }
            if block(&runtime, at) == expected_block {
                traversed_stairs.insert((at.0 / 2 * 2, at.1 / 2 * 2));
            }
            g.walk(&mut c, at);
            assert_eq!(
                c.snapshot()
                    .expect("ordinary stone stairs")
                    .overworld
                    .map_name,
                map(at)
            );
        }
        let side = &lane["blocked_side"];
        let from = point(&side["from_core"]);
        let target = point(&side["target_core"]);
        assert_eq!(kind(&runtime, target), "WALL");
        g.walk(&mut c, from);
        barrier(&mut c, direction(from, target));
        g.walk(&mut c, *path.last().expect("stair far landing"));
        for &at in path.iter().rev().skip(1) {
            g.walk(&mut c, at);
        }
    }
    assert_eq!(
        traversed_stairs, native_stairs,
        "every installed stair crossed both ways"
    );
    for ledge in d["one_way_ledges"].as_array().expect("three ledges") {
        let from = point(&ledge["hop_from"]);
        let landing = point(&ledge["landing"]);
        let forbidden = point(&ledge["reverse_blocked"]);
        let upper = point(&ledge["upper_approach"]);
        g.walk(&mut c, upper);
        let button = match ledge["direction"].as_str().expect("native ledge direction") {
            "south" => GameButton::Down,
            "north" => GameButton::Up,
            "east" => GameButton::Right,
            "west" => GameButton::Left,
            _ => panic!("cardinal ledge"),
        };
        native_hop(&mut c, upper, from, landing, forbidden, button);
        barrier(&mut c, opposite(button));
        let route = g.path(landing, upper);
        assert!(
            route.len()
                >= ledge["minimum_return_steps"]
                    .as_u64()
                    .expect("nontrivial return") as usize
        );
        g.walk(&mut c, upper);
        native_hop(&mut c, upper, from, landing, forbidden, button);
    }
    for grass in d["required_grass_crossings"]
        .as_array()
        .expect("both required habitats")
    {
        let b = &grass["bounds_core"];
        for y in b[1].as_i64().expect("y0")..b[3].as_i64().expect("y1") {
            let at = (b[0].as_i64().expect("x") as i16, y as i16);
            assert_eq!(kind(&runtime, at), "TALL_GRASS");
            g.walk(&mut c, at);
        }
    }
    for river in d["river_catchments"].as_array().expect("authored falls") {
        g.walk(&mut c, point(&river["viewpoint_core"]));
        for at in river["cascade_blocks"].as_array().expect("real falls") {
            let p = point(at);
            assert_eq!(block(&runtime, (p.0 * 2, p.1 * 2)), 0x80);
            assert_eq!(kind(&runtime, (p.0 * 2, p.1 * 2)), "WATERFALL");
        }
    }
    for x in [4, 5] {
        g.walk(&mut c, (51, 99));
        assert_eq!(
            c.snapshot()
                .expect("real lighthouse arrival")
                .overworld
                .map_name,
            "MnR11SplitRockLighthouse"
        );
        let before = flags(&c);
        indoor_walk(&mut c, &runtime, (7, 4));
        c.press(GameButton::Up).expect("face keeper");
        c.press(GameButton::A).expect("keeper interaction");
        assert!(c.snapshot().expect("keeper text").ui.text.is_some());
        finish_text(&mut c);
        assert_eq!(flags(&c), before);
        indoor_walk(&mut c, &runtime, (x, 8));
        indoor_walk(&mut c, &runtime, (x, 9));
        for _ in 0..4 {
            if c.snapshot().expect("exit").overworld.map_name != "MnR11SplitRockLighthouse" {
                break;
            }
            c.press(GameButton::Down).expect("native carpet");
        }
        assert_eq!(position(&mut c), (51, 100));
        g.walk(&mut c, (51, 101));
        g.walk(&mut c, (51, 100));
        assert_eq!(position(&mut c), (51, 100));
    }
}

fn water_path(runtime: &CrystalRuntime, from: Point, to: Point) -> Vec<Point> {
    let mut queue = VecDeque::from([from]);
    let mut previous = BTreeMap::from([(from, from)]);
    while let Some(p) = queue.pop_front() {
        if p == to {
            break;
        }
        for (q, _) in neighbors(p) {
            if q.0 < 0
                || q.1 < 0
                || q.0 >= 112
                || q.1 >= 192
                || previous.contains_key(&q)
                || kind(runtime, q) != "WATER"
            {
                continue;
            }
            previous.insert(q, p);
            queue.push_back(q);
        }
    }
    assert!(
        previous.contains_key(&to),
        "ordinary-water route {from:?}->{to:?}"
    );
    let mut result = Vec::new();
    let mut at = to;
    while at != from {
        result.push(at);
        at = previous[&at];
    }
    result.reverse();
    result
}
fn surf_to(
    runtime: &CrystalRuntime,
    c: &mut VisibleShellController,
    to: Point,
    visited: &mut BTreeSet<String>,
) {
    for at in water_path(runtime, position(c), to) {
        let before = position(c);
        water_step(c, direction(before, at), at);
        assert_eq!(
            c.snapshot()
                .expect("actual expanded sea movement")
                .overworld
                .map_name,
            map(at)
        );
        visited.insert(map(at));
    }
}

#[test]
#[ignore = "requires completed expanded North Shore pack and matching frozen dossier"]
fn expanded_north_shore_controller_surfs_eight_chunks_blocks_outer_edges_and_continues() {
    let d = source();
    let probes = &d["expansion_contract"]["controller_probes"];
    let bank = point(&probes["sea_bank"]);
    let launch = point(&probes["sea_launch"]);
    let (root, runtime, mut c) = fixture(bank, true);
    mount_surf(&mut c, direction(bank, launch), launch);
    let mut visited = BTreeSet::from([map(launch)]);
    for waypoint in probes["sea_waypoints"]
        .as_array()
        .expect("frozen eight-chunk sea itinerary")
    {
        surf_to(&runtime, &mut c, point(waypoint), &mut visited);
    }
    assert_eq!(
        visited.len(),
        8,
        "real Surf must visit all eight exterior chunks"
    );
    for probe in probes["sea_boundary_probes"]
        .as_array()
        .expect("outer visible water closure")
    {
        let at = point(&probe["at"]);
        let target = point(&probe["target"]);
        surf_to(&runtime, &mut c, at, &mut visited);
        assert_eq!(kind(&runtime, target), "BUOY");
        barrier(&mut c, direction(at, target));
        assert_eq!(
            c.snapshot().expect("closed Surf edge").overworld.mode,
            MovementMode::Surf
        );
    }
    surf_to(&runtime, &mut c, launch, &mut visited);
    water_step(&mut c, direction(launch, bank), bank);
    assert_eq!(
        c.snapshot().expect("safe dismount").overworld.mode,
        MovementMode::Normal
    );
    let before = c.snapshot().expect("completed sea circuit");
    let before_flags = flags(&c);
    let money = c.shell.shell.session().state().money;
    let path = std::env::temp_dir().join(format!(
        "geothite-expanded-coast-{}.crystalsave",
        std::process::id()
    ));
    c.save(&path).expect("production save");
    let mut loaded = VisibleShellController::load_save(root, runtime, path.clone(), None)
        .expect("native Continue");
    let after = loaded.snapshot().expect("continued sea circuit");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(flags(&loaded), before_flags);
    assert_eq!(loaded.shell.shell.session().state().money, money);
    std::fs::remove_file(path).expect("remove temporary save");
}

#[test]
#[ignore = "requires completed expanded North Shore pack and matching frozen dossier"]
fn expanded_north_shore_controller_climbs_authored_cascades_through_native_waterfall() {
    let d = source();
    let mut tested = 0;
    for river in d["river_catchments"]
        .as_array()
        .expect("five authored catchments")
    {
        let stages = river["climb_stages"]
            .as_array()
            .expect("native stage contracts");
        let bank = point(&river["base_surf_bank_core"]);
        let launch = point(&river["base_surf_launch_core"]);
        let (_, runtime, mut c) = fixture(bank, true);
        mount_surf(&mut c, direction(bank, launch), launch);
        let mut visited = BTreeSet::new();
        for stage in stages {
            let base = point(&stage["base_water"]);
            let top = point(&stage["upper_water"]);
            assert_eq!(base.0, top.0);
            surf_to(&runtime, &mut c, base, &mut visited);
            climb_added_cascade(&mut c, base.0, base.1, top.1);
            assert_eq!(kind(&runtime, top), "WATER");
            tested += 1;
        }
        let upper = point(&river["upper_surf_launch_core"]);
        let dry = point(&river["upper_surf_bank_core"]);
        surf_to(&runtime, &mut c, upper, &mut visited);
        water_step(&mut c, direction(upper, dry), dry);
        assert_eq!(
            c.snapshot()
                .expect("actual upper shore landing")
                .overworld
                .mode,
            MovementMode::Normal
        );
    }
    assert!(
        tested >= 7,
        "all declared old/new waterfall stages must execute"
    );
}
