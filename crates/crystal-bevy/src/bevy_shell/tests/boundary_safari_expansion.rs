//! Production input for the expanded preserve's closed terrain/hospitality
//! review. Beyond-gate placements are labeled fixtures, not paid admission.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

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
    Point, RouteGrid, RouteSpec, indoor_walk, neighbors, opposite, position,
};
use super::boundary_waters::{
    answer, assert_no_expedition, blocked_water, checkout_one, close_service,
    decline_rest_and_shop, drink_gift, drink_quantity, gift_flag, mount_surf, open_shop, press,
    prompt, step,
};

const CABIN: &str = "MnBoundaryRootBeerCabin";
const SERVICE: &str = "BWS_ROOT_BEER_LADYRootBeerService";
fn layout() -> Value {
    let path=std::env::var_os("CRYSTAL_BOUNDARY_SAFARI_EXPANSION_LAYOUT").map(PathBuf::from)
        .unwrap_or_else(||PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/revisions/northwoods-expansion/after/boundary-waters-safari.layout.json"));
    let v: Value = serde_json::from_str(
        &std::fs::read_to_string(path).expect("matching frozen Safari terrain source"),
    )
    .expect("source layout");
    assert_eq!(v["dimensions_metatiles"], serde_json::json!([64, 56]));
    v
}
fn point(v: &Value) -> Point {
    (
        v[0].as_i64().expect("core x") as i16,
        v[1].as_i64().expect("core y") as i16,
    )
}
fn map(at: Point) -> String {
    format!("MnBoundarySafariC{:02}R{:02}", at.0 / 64, at.1 / 48)
}
fn direction(from: Point, to: Point) -> GameButton {
    neighbors(from)
        .into_iter()
        .find_map(|(p, b)| (p == to).then_some(b))
        .expect("adjacent input")
}
fn load() -> (AssetRoot, CrystalRuntime) {
    // Only immutable native data is reused; controllers and sessions stay fresh.
    // The pack environment path is captured once for this test process.
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_BOUNDARY_SAFARI_EXPANSION_CONTROLLER_PACK")
                    .expect("actual published expanded preserve pack"),
            );
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("strict published pack"),
            )
            .expect("native runtime");
            (root, runtime)
        })
        .clone()
}
fn fixture(at: Point, injured: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("preserved HOME"),
            map_name: map(at),
            tile_x: at.0 % 64,
            tile_y: at.1 % 48,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local beyond-gate review fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture name");
    let mut pokemon = runtime
        .data()
        .create_pokemon("MEW", 60, Dv::default())
        .expect("legal native HM-compatible partner");
    pokemon.moves = vec![LearnedMove {
        name: "SURF".into(),
        current_pp: runtime.data().moves["SURF"].pp,
        pp_ups: 0,
    }];
    if injured {
        pokemon.hp = pokemon.max_hp - 100;
        pokemon.status = Some("POISON".into());
        pokemon.moves[0].current_pp = 0;
    }
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(pokemon));
    state.sync_party_from_storage();
    state.money = 1000;
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
                source_script: "ExpandedPreserveLocalFixture".into(),
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
    // No session permit, Safari counter, success flag, later position or party
    // state is injected. Every mutation after this line is production input.
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert_eq!(position(&mut c), at);
    assert_no_expedition(&c);
    (root, runtime, c)
}
fn grid(runtime: &CrystalRuntime, start: Point, l: &Value) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnBoundarySafari",
            columns: 2,
            rows: 3,
            tileset: "mn_boundary_safari",
            start,
            doors: BTreeMap::from([(point(&l["root_beer_lady"]["door"]["tile"]), CABIN)]),
        },
    )
}
fn block(runtime: &CrystalRuntime, at: Point) -> u16 {
    let m = &runtime.data().maps[&map(at)];
    m.blocks[(at.1 as usize % 48) / 2 * usize::from(m.attributes.width) + (at.0 as usize % 64) / 2]
}
fn kind(runtime: &CrystalRuntime, at: Point) -> &str {
    &runtime.data().tilesets["mn_boundary_safari"].collision[&format!("{:02x}", block(runtime, at))]
        [(at.1 % 2 * 2 + at.0 % 2) as usize]
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
fn save_continue(
    root: AssetRoot,
    runtime: CrystalRuntime,
    c: &mut VisibleShellController,
) -> VisibleShellController {
    let before = c.snapshot().expect("actual pre-save state");
    let before_flags = flags(c);
    let money = c.shell.shell.session().state().money;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let retained =
        std::env::var_os("CRYSTAL_BOUNDARY_SAFARI_EXPANSION_SAVE_DIR").map(PathBuf::from);
    let directory = retained.clone().unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&directory).expect("native checkpoint directory");
    let path = directory.join(format!(
        "geothite-expanded-preserve-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    assert!(!path.exists(), "preserve every prior reached save");
    c.save(&path).expect("native save");
    let mut loaded = VisibleShellController::load_save(root, runtime, path.clone(), None)
        .expect("production Continue");
    let after = loaded.snapshot().expect("continued native state");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(flags(&loaded), before_flags);
    assert_eq!(loaded.shell.shell.session().state().money, money);
    assert_no_expedition(&loaded);
    if retained.is_some() {
        eprintln!("retained expanded preserve save: {}", path.display());
    } else {
        std::fs::remove_file(path).expect("remove temporary save");
    }
    loaded
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
    step(c, button, from);
    for _ in 0..8 {
        if position(c) == landing {
            break;
        }
        press(c, button);
        assert_ne!(position(c), forbidden);
    }
    assert_eq!(position(c), landing);
}
fn barrier(c: &mut VisibleShellController, button: GameButton) {
    let at = position(c);
    for _ in 0..4 {
        press(c, button);
        assert_eq!(position(c), at);
    }
}

#[test]
#[ignore = "requires completed expanded preserve pack and matching frozen layout"]
fn expanded_safari_controller_closed_new_gate_preserves_resources_and_blocks_admission() {
    let l = layout();
    let start = point(&l["entry_contract"]["arrival_tile"]);
    let (_, runtime, mut c) = fixture(start, false);
    let g = grid(&runtime, start, &l);
    let before = c.snapshot().expect("closed gate resources");
    let before_flags = flags(&c);
    let gates = l["entry_gate"]["gate_tiles"]
        .as_array()
        .expect("paired throat");
    let lower = gates
        .iter()
        .map(point)
        .map(|p| p.1)
        .max()
        .expect("guard row");
    let xs = gates
        .iter()
        .map(point)
        .map(|p| p.0)
        .collect::<BTreeSet<_>>();
    for x in xs {
        let guard = (x, lower);
        assert!(g.talk(&mut c, guard).contains("EXPEDITIONS CLOSED"));
        g.walk(&mut c, (x, lower + 1));
        barrier(&mut c, GameButton::Up);
    }
    let ranger = point(&l["retire_anchors"][0]["tile"]);
    assert!(g.talk(&mut c, ranger).contains("EXPEDITIONS CLOSED"));
    let after = c.snapshot().expect("honest closed admission");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(flags(&c), before_flags);
    assert_eq!(c.shell.shell.session().state().money, 1000);
    assert_no_expedition(&c);
    assert!(
        !g.floor.contains(&point(&l["entry_gate"]["inside_tile"])),
        "closed guard objects must cut the real arrival court from preserve land"
    );
}

#[test]
#[ignore = "requires completed expanded preserve pack and matching frozen layout"]
fn expanded_safari_controller_relocated_cabin_has_two_safe_returns_and_continue() {
    let l = layout();
    let door = &l["root_beer_lady"]["door"];
    let arrival = point(&door["arrival_tile"]);
    let (root, runtime, mut c) = fixture(arrival, false);
    let g = grid(&runtime, arrival, &l);
    assert_eq!(
        runtime.data().runtime_map_metadata["MN_BOUNDARY_ROOT_BEER_CABIN"].map_id,
        5
    );
    let outside = &runtime.data().maps[&map(arrival)];
    let dorothy = point(&l["landmark_actors"][0]["tile"]);
    assert_eq!(
        outside
            .objects
            .iter()
            .filter(|o| o.script == SERVICE)
            .count(),
        1
    );
    assert!(outside.objects.iter().any(
        |o| o.script == SERVICE && (o.x as i16, o.y as i16) == (dorothy.0 % 64, dorothy.1 % 48)
    ));
    assert!(
        runtime.data().maps[CABIN].objects.is_empty(),
        "no duplicate indoor Dorothy"
    );
    for x in [5, 6] {
        g.walk(&mut c, point(&door["tile"]));
        let entered = c.snapshot().expect("native paired entry");
        assert_eq!(entered.overworld.map_name, CABIN);
        assert_eq!((entered.overworld.tile.x, entered.overworld.tile.y), (5, 6));
        indoor_walk(&mut c, &runtime, (6, 6));
        indoor_walk(&mut c, &runtime, (5, 6));
        assert_eq!(
            c.snapshot()
                .expect("passive cabin Floor")
                .overworld
                .map_name,
            CABIN
        );
        if x == 5 {
            c = save_continue(root.clone(), runtime.clone(), &mut c);
        }
        indoor_walk(&mut c, &runtime, (x, 6));
        indoor_walk(&mut c, &runtime, (x, 7));
        for _ in 0..4 {
            if c.snapshot().expect("native exit").overworld.map_name != CABIN {
                break;
            }
            press(&mut c, GameButton::Down);
        }
        assert_eq!(position(&mut c), arrival);
        g.walk(&mut c, (arrival.0, arrival.1 + 1));
        g.walk(&mut c, arrival);
        assert_eq!(position(&mut c), arrival);
        assert_no_expedition(&c);
    }
}

fn dorothy(c: &mut VisibleShellController, l: &Value) {
    let actor = &l["landmark_actors"][0];
    let approach = point(&actor["approach"]);
    assert_eq!(position(c), approach);
    press(c, direction(approach, point(&actor["tile"])));
    assert_eq!(position(c), approach);
    press(c, GameButton::A);
}
#[test]
#[ignore = "requires completed expanded preserve pack and matching frozen layout"]
fn expanded_safari_controller_actual_relocated_root_beer_gift_drink_rest_shop_and_continue() {
    let l = layout();
    let approach = point(&l["landmark_actors"][0]["approach"]);
    let (root, runtime, mut c) = fixture(approach, true);
    let initial_hp = c.snapshot().expect("explicit injured partner").party.slots[0]
        .pokemon
        .hp;
    dorothy(&mut c, &l);
    prompt(&mut c, SERVICE, 5);
    assert!(!gift_flag(&c));
    answer(&mut c, true);
    prompt(&mut c, &format!("{SERVICE}Choices"), 1);
    assert!(gift_flag(&c));
    assert_eq!(drink_quantity(&c, &runtime), 1);
    assert_eq!(c.shell.shell.session().state().money, 1000);
    assert_eq!(
        c.snapshot().expect("gift is not healing").party.slots[0]
            .pokemon
            .hp,
        initial_hp
    );
    decline_rest_and_shop(&mut c);
    drink_gift(&mut c);
    assert_eq!(drink_quantity(&c, &runtime), 0);
    let partner = &c.snapshot().expect("real medicine").party.slots[0].pokemon;
    assert_eq!(partner.hp, initial_hp + 60);
    assert_eq!(partner.moves[0].current_pp, 0);
    dorothy(&mut c, &l);
    prompt(&mut c, &format!("{SERVICE}Choices"), 1);
    answer(&mut c, true);
    prompt(&mut c, &format!("{SERVICE}Shop"), 1);
    let state = c.shell.shell.session().state();
    let partner = state.storage.party.pokemon[0]
        .as_ref()
        .expect("rested partner");
    assert_eq!(partner.hp, partner.max_hp);
    assert_eq!(partner.status, None);
    assert_eq!(partner.moves[0].current_pp, runtime.data().moves["SURF"].pp);
    answer(&mut c, true);
    open_shop(&mut c);
    checkout_one(&mut c);
    assert_eq!(drink_quantity(&c, &runtime), 1);
    assert_eq!(c.shell.shell.session().state().money, 700);
    close_service(&mut c);
    c = save_continue(root, runtime.clone(), &mut c);
    dorothy(&mut c, &l);
    decline_rest_and_shop(&mut c);
    assert_eq!(
        drink_quantity(&c, &runtime),
        1,
        "saved gift flag must prevent duplicate free bottle"
    );
    assert_eq!(c.shell.shell.session().state().money, 700);
    assert_no_expedition(&c);
}

#[test]
#[ignore = "requires completed expanded preserve pack and matching frozen layout"]
fn expanded_safari_controller_five_real_stairs_ledges_and_six_grassy_portages() {
    let l = layout();
    let (_, runtime, _) = fixture(point(&l["entry_contract"]["arrival_tile"]), false);
    let mut stairs = Vec::new();
    for row in 0..3 {
        for col in 0..2 {
            let m = &runtime.data().maps[&format!("MnBoundarySafariC{col:02}R{row:02}")];
            for (index, b) in m.blocks.iter().enumerate() {
                if *b == 0x84 {
                    stairs.push((
                        col * 64 + (index % usize::from(m.attributes.width)) as i16 * 2,
                        row * 48 + (index / usize::from(m.attributes.width)) as i16 * 2,
                    ));
                }
            }
        }
    }
    assert_eq!(stairs.len(), 5, "five exact native cave12 climbs");
    for (x, y) in stairs {
        let start = (x + 1, y - 1);
        let (_, runtime, mut c) = fixture(start, false);
        let g = grid(&runtime, start, &l);
        for at in [(x + 1, y), (x + 1, y + 1), (x + 1, y + 2), (x + 1, y + 1)] {
            assert_eq!(kind(&runtime, at), "FLOOR");
            g.walk(&mut c, at);
            assert_eq!(
                c.snapshot()
                    .expect("ordinary rock stairs")
                    .overworld
                    .map_name,
                map(at)
            );
        }
        barrier(&mut c, GameButton::Left);
        g.walk(&mut c, (x + 1, y));
        g.walk(&mut c, start);
        assert_no_expedition(&c);
    }
    let ledges = l["one_way_ledges"].as_array().expect("five native ledges");
    assert_eq!(ledges.len(), 5);
    for ledge in ledges {
        let from = point(&ledge["hop_from"]);
        let landing = point(&ledge["landing"]);
        let upper = point(&ledge["upper_approach"]);
        let (_, runtime, mut c) = fixture(upper, false);
        let g = grid(&runtime, upper, &l);
        let button = match ledge["direction"].as_str().expect("direction") {
            "south" => GameButton::Down,
            "north" => GameButton::Up,
            "west" => GameButton::Left,
            "east" => GameButton::Right,
            _ => panic!("cardinal native ledge"),
        };
        native_hop(
            &mut c,
            upper,
            from,
            landing,
            point(&ledge["reverse_blocked"]),
            button,
        );
        barrier(&mut c, opposite(button));
        assert!(
            g.path(landing, upper).len()
                >= ledge["minimum_return_steps"]
                    .as_u64()
                    .expect("meaningful return") as usize
        );
        g.walk(&mut c, upper);
        native_hop(
            &mut c,
            upper,
            from,
            landing,
            point(&ledge["reverse_blocked"]),
            button,
        );
        assert_no_expedition(&c);
    }
    let mut crossed_walking_seams = BTreeSet::new();
    for p in l["portages"].as_array().expect("six named grassy portages") {
        let west = point(&p["west_land"]);
        let ww = point(&p["west_water"]);
        let east = point(&p["east_land"]);
        let ew = point(&p["east_water"]);
        let (_, runtime, mut c) = fixture(west, false);
        mount_surf(&mut c, direction(west, ww), ww);
        step(&mut c, direction(ww, west), west);
        assert_eq!(
            c.snapshot()
                .expect("native portage dismount")
                .overworld
                .mode,
            MovementMode::Normal
        );
        let g = grid(&runtime, west, &l);
        let mut grass = 0;
        for button in g.path(west, east) {
            let at = position(&mut c);
            let next = neighbors(at)
                .into_iter()
                .find_map(|(p, b)| (b == button).then_some(p))
                .expect("next land step");
            let old = map(at);
            step(&mut c, button, next);
            grass += usize::from(kind(&runtime, next) == "TALL_GRASS");
            if old != map(next) {
                crossed_walking_seams.insert((old, map(next)));
            }
        }
        assert!(
            grass
                >= p["minimum_grass_steps"]
                    .as_u64()
                    .expect("required grassy saddle") as usize,
            "{} bypassed grass",
            p["id"]
        );
        mount_surf(&mut c, direction(east, ew), ew);
        step(&mut c, direction(ew, east), east);
        assert_eq!(
            c.snapshot().expect("second landing").overworld.mode,
            MovementMode::Normal
        );
        g.walk(&mut c, west);
        assert_no_expedition(&c);
    }
    for pair in l["controller_probes"]["walking_seam_pairs"]
        .as_array()
        .expect("explicit EW/NS walking seam probes")
    {
        let a = point(&pair[0]);
        let b = point(&pair[1]);
        let (_, runtime, mut c) = fixture(a, false);
        let g = grid(&runtime, a, &l);
        assert_ne!(map(a), map(b));
        for at in [b, a, b, a] {
            assert_eq!(kind(&runtime, at), "FLOOR");
            g.walk(&mut c, at);
            assert_eq!(
                c.snapshot()
                    .expect("reciprocal native walking seam")
                    .overworld
                    .map_name,
                map(at)
            );
        }
        assert_no_expedition(&c);
    }
    assert!(
        !crossed_walking_seams.is_empty(),
        "real portage walking must cross at least one native chunk seam"
    );
}

fn paddle(
    runtime: &CrystalRuntime,
    c: &mut VisibleShellController,
    to: Point,
    visited: &mut BTreeSet<String>,
) {
    let from = position(c);
    let mut previous = BTreeMap::from([(from, from)]);
    let mut queue = VecDeque::from([from]);
    while let Some(at) = queue.pop_front() {
        if at == to {
            break;
        }
        for (next, _) in neighbors(at) {
            if next.0 < 0
                || next.0 >= 128
                || next.1 < 0
                || next.1 >= 112
                || previous.contains_key(&next)
                || kind(runtime, next) != "WATER"
            {
                continue;
            }
            previous.insert(next, at);
            queue.push_back(next);
        }
    }
    assert!(
        previous.contains_key(&to),
        "ordinary connected-lake water path {from:?}->{to:?}"
    );
    let mut path = Vec::new();
    let mut at = to;
    while at != from {
        path.push(at);
        at = previous[&at];
    }
    visited.insert(map(from));
    for at in path.into_iter().rev() {
        let previous = position(c);
        step(c, direction(previous, at), at);
        let s = c.snapshot().expect("native six-chunk paddle");
        assert_eq!(s.overworld.mode, MovementMode::Surf);
        assert_eq!(s.overworld.map_name, map(at));
        visited.insert(s.overworld.map_name);
    }
}
#[test]
#[ignore = "requires completed expanded preserve pack and matching frozen layout"]
fn expanded_safari_controller_surfs_six_chunks_and_rejects_real_closed_water_boundaries() {
    let l = layout();
    let probes = &l["controller_probes"];
    let bank = point(&probes["sea_bank"]);
    let launch = point(&probes["sea_launch"]);
    let (root, runtime, mut c) = fixture(bank, false);
    mount_surf(&mut c, direction(bank, launch), launch);
    let mut visited = BTreeSet::new();
    for at in probes["sea_waypoints"]
        .as_array()
        .expect("six-chunk lake itinerary")
    {
        paddle(&runtime, &mut c, point(at), &mut visited);
    }
    assert_eq!(
        visited.len(),
        6,
        "native Surf must cross bothcolumns and allthreerows"
    );
    for probe in probes["sea_boundary_probes"]
        .as_array()
        .expect("real bank/rock boundary probes")
    {
        let at = point(&probe["at"]);
        let target = point(&probe["target"]);
        paddle(&runtime, &mut c, at, &mut visited);
        assert!(matches!(kind(&runtime, target), "BUOY" | "WALL"));
        blocked_water(&mut c, direction(at, target));
    }
    let perimeter = (0..128)
        .flat_map(|x| [(x, 0), (x, 111)])
        .chain((0..112).flat_map(|y| [(0, y), (127, y)]))
        .collect::<BTreeSet<_>>();
    assert_eq!(perimeter.len(), 476);
    let source = &runtime.data().maps["MnBoundarySafariC00R02"];
    let pine = &runtime.data().maps["MnPreservePineEntry"];
    assert!(
        source
            .attributes
            .connections
            .iter()
            .any(|c| c.direction == "south"
                && c.target_map == "MnPreservePineEntry"
                && c.offset == -1)
    );
    assert!(
        pine.attributes
            .connections
            .iter()
            .any(|c| c.direction == "north"
                && c.target_map == "MnBoundarySafariC00R02"
                && c.offset == 1)
    );
    for at in perimeter {
        if matches!(at, (14 | 15, 111)) {
            assert_eq!(kind(&runtime, at), "FLOOR", "declared public walking port");
        } else {
            assert!(
                matches!(kind(&runtime, at), "WALL" | "BUOY"),
                "unclosed native outer edge {at:?}"
            );
        }
    }
    paddle(&runtime, &mut c, launch, &mut visited);
    step(&mut c, direction(launch, bank), bank);
    assert_eq!(
        c.snapshot().expect("safe main landing").overworld.mode,
        MovementMode::Normal
    );
    assert_no_expedition(&c);
    let _continued = save_continue(root, runtime, &mut c);
}

#[path = "boundary_safari_grass_review.rs"]
mod boundary_safari_grass_review;
