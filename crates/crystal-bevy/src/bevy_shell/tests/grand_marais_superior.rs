//! Explicit local party/badge/signal/Floor fixtures, then production input only.
//! These checks do not establish earned regional progression.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

use super::grand_marais_harbor::use_max_repel;
use super::{
    Point, VisibleShellController, assert_at, at, fixture_with_items_on_runtime,
    mark_runtime_snapshot_dirty, mount, settle, step,
};

const TOWN: &str = "MnGrandMarais";
const COAST: &str = "MnR11C01R00";
const INLAND: &str = "MnMaraisCedarApproach";
const BADGES: [&str; 8] = [
    "ENGINE_ZEPHYRBADGE",
    "ENGINE_HIVEBADGE",
    "ENGINE_PLAINBADGE",
    "ENGINE_FOGBADGE",
    "ENGINE_STORMBADGE",
    "ENGINE_MINERALBADGE",
    "ENGINE_GLACIERBADGE",
    "ENGINE_RISINGBADGE",
];

fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_MARAIS_SUPERIOR_CONTROLLER_PACK")
                    .expect("explicit final composed coast/inland pack"),
            )
            .canonicalize()
            .expect("completed pack");
            let root = AssetRoot::new(path.parent().expect("pack directory"));
            let r = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("strict composed pack"),
            )
            .expect("native composed runtime");
            let town = &r.data().maps[TOWN];
            let coast = &r.data().maps[COAST].attributes;
            let inland = &r.data().maps[INLAND].attributes;
            let inland_offset = match (coast.width, coast.height, inland.width, inland.height) {
                (24, 24, 32, 19) => 19,
                (4, 24, 20, 12) => 4,
                profile => panic!("unreviewed Superior geometry profile: {profile:?}"),
            };
            assert_eq!(town.attributes.tileset_name, "mn_marais_shore");
            assert!(
                town.attributes
                    .connections
                    .iter()
                    .any(|c| c.direction == "west" && c.target_map == COAST && c.offset == 0)
            );
            assert!(
                town.attributes
                    .connections
                    .iter()
                    .any(|c| c.direction == "north"
                        && c.target_map == INLAND
                        && c.offset == -inland_offset)
            );
            assert!(
                r.data().maps[COAST]
                    .attributes
                    .connections
                    .iter()
                    .any(|c| c.direction == "east" && c.target_map == TOWN && c.offset == 0)
            );
            assert!(
                r.data().maps[INLAND]
                    .attributes
                    .connections
                    .iter()
                    .any(|c| c.direction == "south"
                        && c.target_map == TOWN
                        && c.offset == inland_offset)
            );
            (root, r)
        })
        .clone()
}

fn fixture(
    map: &str,
    point: Point,
    all_badges: bool,
    signal: bool,
    hurt: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, mut c) =
        fixture_with_items_on_runtime(load(), map, point, false, false, false, &[("MAX_REPEL", 5)]);
    let state = c.shell.shell.session_mut().state_mut();
    for (i, flag) in BADGES.iter().enumerate() {
        // Fog remains available for legal Surf in the missing-Zephyr fixture.
        let enabled = all_badges || i == 3;
        state.badges.johto[i] = enabled;
        state
            .flags
            .set_engine_flag(flag, enabled)
            .expect("consistent initial badge fixture");
    }
    state
        .flags
        .set_event_flag("EVENT_MN_LIGHTHOUSE_SIGNAL", signal)
        .expect("initial Duluth signal");
    state
        .flags
        .set_event_flag("EVENT_MN_GRAND_MARAIS_SIGNAL", true)
        .expect("separate local lamp does not substitute");
    if hurt {
        let p = state.storage.party.pokemon[0]
            .as_mut()
            .expect("initial Mew");
        p.hp = p.max_hp.saturating_sub(20);
    }
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert_at(&mut c, map, point, "explicit initial Floor fixture");
    // No state/party/mode/flags/position/RNG writes below this boundary.
    (root, r, c)
}

fn save(c: &mut VisibleShellController, label: &str) -> PathBuf {
    let dir = PathBuf::from(
        std::env::var_os("CRYSTAL_MARAIS_SUPERIOR_SAVE_DIR")
            .expect("caller-named ignored reached-save directory"),
    );
    std::fs::create_dir_all(&dir).expect("save directory");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = dir.join(format!(
        "marais-{label}-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    assert!(!path.exists());
    c.save(&path).expect("production reached save");
    path
}

fn button(a: Point, b: Point) -> GameButton {
    match (b.0 - a.0, b.1 - a.1) {
        (1, 0) => GameButton::Right,
        (-1, 0) => GameButton::Left,
        (0, 1) => GameButton::Down,
        (0, -1) => GameButton::Up,
        other => panic!("nonadjacent input {other:?}"),
    }
}

fn kind(r: &CrystalRuntime, map: &str, p: Point) -> String {
    let m = &r.data().maps[map];
    let x = usize::try_from(p.0).expect("nonnegative x");
    let y = usize::try_from(p.1).expect("nonnegative y");
    let b = m.blocks[y / 2 * usize::from(m.attributes.width) + x / 2];
    r.data().tilesets[&m.attributes.tileset_name].collision[&format!("{b:02x}")][y % 2 * 2 + x % 2]
        .clone()
}

fn path(r: &CrystalRuntime, c: &mut VisibleShellController, target: Point, water: bool) {
    terrain_path(r, c, target, water, false);
}

fn terrain_path(
    r: &CrystalRuntime,
    c: &mut VisibleShellController,
    target: Point,
    water: bool,
    include_grass: bool,
) -> usize {
    let (map, start) = at(c);
    let m = &r.data().maps[&map];
    let occupied = m
        .objects
        .iter()
        .map(|o| {
            (
                i16::try_from(o.x).expect("actor x"),
                i16::try_from(o.y).expect("actor y"),
            )
        })
        .collect::<BTreeSet<_>>();
    let mut queue = VecDeque::from([start]);
    let mut previous = BTreeMap::from([(start, None)]);
    while let Some(p) = queue.pop_front() {
        if p == target {
            break;
        }
        for n in [
            (p.0 - 1, p.1),
            (p.0 + 1, p.1),
            (p.0, p.1 - 1),
            (p.0, p.1 + 1),
        ] {
            if n.0 < 0
                || n.1 < 0
                || n.0 >= i16::try_from(m.attributes.width * 2).expect("width")
                || n.1 >= i16::try_from(m.attributes.height * 2).expect("height")
                || occupied.contains(&n)
                || previous.contains_key(&n)
            {
                continue;
            }
            let k = kind(r, &map, n);
            let passable = if water {
                k == "WATER"
            } else {
                k == "FLOOR" || (include_grass && k == "TALL_GRASS")
            };
            if !passable {
                continue;
            }
            // Avoid incidental warps/events. A named target may intentionally
            // trigger the room's authored welcome scene through real input.
            if m.events.warps.iter().any(|w| {
                (
                    i16::try_from(w.x).expect("warp x"),
                    i16::try_from(w.y).expect("warp y"),
                ) == n
            }) || (n != target
                && m.events.coord_events.iter().any(|e| {
                    (
                        i16::try_from(e.x).expect("event x"),
                        i16::try_from(e.y).expect("event y"),
                    ) == n
                }))
            {
                continue;
            }
            previous.insert(n, Some(p));
            queue.push_back(n);
        }
    }
    assert!(
        previous.contains_key(&target),
        "ordinary {map} route {start:?}->{target:?}"
    );
    let mut course = vec![];
    let mut end = target;
    while let Some(p) = previous[&end] {
        course.push((p, end));
        end = p;
    }
    let mut grass_steps = 0;
    for (from, to) in course.into_iter().rev() {
        grass_steps += usize::from(kind(r, &map, to) == "TALL_GRASS");
        step(
            c,
            button(from, to),
            &map,
            to,
            "body-aware ordinary input course",
        );
    }
    grass_steps
}

fn refused(c: &mut VisibleShellController, input: GameButton, point: Point, water: bool) {
    let before = c.snapshot().expect("checkpoint baseline");
    let mut found = false;
    for _ in 0..4 {
        c.press(input).expect("outgoing checkpoint input");
        c.wait_frames(128).expect("print refusal");
        let s = c.snapshot().expect("checkpoint text");
        if s.ui.text_window_open {
            let page =
                s.ui.text
                    .as_ref()
                    .and_then(|p| p.asm_text.as_ref())
                    .expect("authored refusal page");
            assert!(
                page.contains("BADGES") || page.contains("beacon"),
                "actual badge/signal explanation: {page}"
            );
            let held = at(c);
            c.press(input).expect("movement rejected during dialogue");
            assert_eq!(at(c), held, "dialogue owns movement");
            found = true;
            break;
        }
    }
    assert!(found, "actual checkpoint refusal must be visible");
    settle(c);
    assert_at(c, TOWN, point, "safe one-step denial returns inside");
    let after = c.snapshot().expect("checkpoint after dialogue");
    assert_eq!(before.progression.badges, after.progression.badges);
    assert_eq!(
        before.progression.active_engine_flags,
        after.progression.active_engine_flags
    );
    assert_eq!(
        before.progression.active_event_flags,
        after.progression.active_event_flags
    );
    assert_eq!(before.party, after.party, "refusal preserves the partner");
    assert_eq!(before.bag, after.bag, "refusal consumes no item");
    assert_eq!(
        after.overworld.mode,
        if water {
            MovementMode::Surf
        } else {
            MovementMode::Normal
        }
    );
}

fn enter(c: &mut VisibleShellController, input: GameButton, map: &str) {
    for _ in 0..6 {
        if at(c).0 == map {
            return;
        }
        c.press(input).expect("real room/seam input");
        c.wait_frames(16).expect("warp presentation");
        settle(c);
    }
    assert_eq!(at(c).0, map, "actual intended map transition");
}

fn nurse(r: &CrystalRuntime, c: &mut VisibleShellController) {
    path(r, c, (15, 10), false);
    enter(c, GameButton::Up, "MnGrandMaraisPokecenter");
    path(r, c, (3, 3), false);
    c.press(GameButton::Up).expect("face nurse across counter");
    c.press(GameButton::A)
        .expect("interact with installed nurse");
    settle(c);
    let p = c.shell.shell.session().state().storage.party.pokemon[0]
        .as_ref()
        .expect("partner");
    assert_eq!(
        p.hp, p.max_hp,
        "actual nurse restores initially injured partner"
    );
    assert_at(
        c,
        "MnGrandMaraisPokecenter",
        (3, 3),
        "native care completed",
    );
    save(c, "free-inward-care");
}

fn settle_original_field_station(c: &mut VisibleShellController) {
    for _ in 0..256 {
        if c.shell.pending_gift_pokemon_nickname.is_some() {
            c.press(GameButton::B)
                .expect("decline original gift nickname through production input");
            c.wait_frames(16).expect("gift script continuation");
            continue;
        }
        let snapshot = c.snapshot().expect("original field station scene");
        assert!(
            snapshot.battle.is_none() && snapshot.ui.menu.is_none(),
            "unexpected room surface"
        );
        if snapshot.phase == RuntimeShellPhase::Overworld
            && !snapshot.ui.text_window_open
            && snapshot.ui.pending_text_wait.is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
        {
            return;
        }
        c.wait_frames(120).expect("print original room page");
        c.press(GameButton::A).expect("advance original room page");
        c.wait_frames(16).expect("room script continuation");
    }
    panic!("original field station scene did not settle");
}

#[test]
#[ignore = "requires final composed native geography pack and caller save directory"]
fn grand_marais_superior_controller_closed_foot_checks_badges_and_duluth_signal() {
    for (badges, signal) in [(false, true), (true, false)] {
        for (point, input) in [
            ((4, 8), GameButton::Left),
            ((4, 9), GameButton::Left),
            ((18, 4), GameButton::Up),
            ((19, 4), GameButton::Up),
        ] {
            let (_, _, mut c) = fixture(TOWN, point, badges, signal, false);
            refused(&mut c, input, point, false);
            save(&mut c, "closed-foot");
        }
    }
}

#[test]
#[ignore = "requires final composed native geography pack and caller save directory"]
fn grand_marais_superior_controller_open_west_north_and_both_free_inward_care() {
    let (_, runtime) = load();
    let coast_x =
        i16::try_from(runtime.data().maps[COAST].attributes.width).expect("coast width") * 2 - 1;
    let north = runtime.data().maps[TOWN]
        .attributes
        .connections
        .iter()
        .find(|edge| edge.direction == "north")
        .expect("reviewed town north seam");
    let inland_x = 18 - i16::try_from(north.offset).expect("guarded native north offset") * 2;
    let inland_y =
        i16::try_from(runtime.data().maps[INLAND].attributes.height).expect("inland height") * 2
            - 1;
    for (point, input, edge, neighbor, target, back, inside) in [
        (
            (4, 8),
            GameButton::Left,
            (0, 8),
            COAST,
            (coast_x, 8),
            GameButton::Right,
            (3, 8),
        ),
        (
            (18, 4),
            GameButton::Up,
            (18, 0),
            INLAND,
            (inland_x, inland_y),
            GameButton::Down,
            (18, 3),
        ),
        (
            (4, 9),
            GameButton::Left,
            (0, 9),
            COAST,
            (coast_x, 9),
            GameButton::Right,
            (3, 9),
        ),
        (
            (19, 4),
            GameButton::Up,
            (19, 0),
            INLAND,
            (inland_x + 1, inland_y),
            GameButton::Down,
            (19, 3),
        ),
    ] {
        let (_, _, mut c) = fixture(TOWN, point, true, true, false);
        let mut p = point;
        while p != edge {
            let n = if input == GameButton::Left {
                (p.0 - 1, p.1)
            } else {
                (p.0, p.1 - 1)
            };
            step(&mut c, input, TOWN, n, "open officer lane");
            p = n;
        }
        step(
            &mut c,
            input,
            neighbor,
            target,
            "outward real connected seam",
        );
        step(&mut c, back, TOWN, edge, "inward real connected seam");
        step(
            &mut c,
            back,
            TOWN,
            inside,
            "inward native two-step officer wrapper",
        );
        save(&mut c, "open-seam");
    }
    for (neighbor, point, input, edge, inside) in [
        (COAST, (coast_x, 8), GameButton::Right, (0, 8), (3, 8)),
        (
            INLAND,
            (inland_x, inland_y),
            GameButton::Down,
            (18, 0),
            (18, 3),
        ),
        (COAST, (coast_x, 9), GameButton::Right, (0, 9), (3, 9)),
        (
            INLAND,
            (inland_x + 1, inland_y),
            GameButton::Down,
            (19, 0),
            (19, 3),
        ),
    ] {
        let (_, r, mut c) = fixture(neighbor, point, false, false, true);
        step(
            &mut c,
            input,
            TOWN,
            edge,
            "unqualified incoming physical seam",
        );
        step(
            &mut c,
            input,
            TOWN,
            inside,
            "free inward hides guards and reconciles behind",
        );
        nurse(&r, &mut c);
        assert!(
            !c.snapshot()
                .expect("care flags")
                .progression
                .active_event_flags
                .contains("EVENT_MN_LIGHTHOUSE_SIGNAL")
        );
    }
}

#[test]
#[ignore = "requires final composed native geography pack and caller save directory"]
fn grand_marais_superior_controller_all_six_original_room_pairs_return() {
    let (_, r, mut c) = fixture(TOWN, (15, 11), true, true, false);
    for (door, room) in [
        ((6, 7), "MnGrandMaraisFieldStation"),
        ((15, 9), "MnGrandMaraisPokecenter"),
        ((21, 9), "MnGrandMaraisMart"),
        ((9, 15), "MnGrandMaraisArtistHouse"),
        ((27, 15), "MnGrandMaraisFisherHouse"),
        ((15, 27), "MnGrandMaraisLighthouse"),
    ] {
        path(&r, &mut c, (door.0, door.1 + 1), false);
        enter(&mut c, GameButton::Up, room);
        let exit = &r.data().maps[room].events.warps[0];
        let x = i16::try_from(exit.x).expect("room exit x");
        let y = i16::try_from(exit.y).expect("room exit y");
        if room == "MnGrandMaraisFieldStation" {
            // An initial Up may only turn after the entry warp. Observe the
            // actual scene flag rather than assuming that input moved a tile.
            for _ in 0..8 {
                if c.snapshot()
                    .expect("welcome progress")
                    .progression
                    .active_event_flags
                    .contains("EVENT_MN_GRAND_MARAIS_STARTER")
                {
                    break;
                }
                c.press(GameButton::Up)
                    .expect("walk into original welcome trigger");
                c.wait_frames(16).expect("native welcome begins");
                settle_original_field_station(&mut c);
            }
            assert!(
                c.snapshot()
                    .expect("completed original welcome")
                    .progression
                    .active_event_flags
                    .contains("EVENT_MN_GRAND_MARAIS_STARTER"),
                "original scene must actually execute"
            );
            path(&r, &mut c, (x, y - 1), false);
        } else {
            path(&r, &mut c, (x, y - 1), false);
        }
        enter(&mut c, GameButton::Down, TOWN);
        assert_at(
            &mut c,
            TOWN,
            (door.0, door.1 + 1),
            "original paired return and passive outdoor step",
        );
        save(&mut c, room);
    }
}

fn blocked(c: &mut VisibleShellController, input: GameButton, point: Point) {
    for _ in 0..3 {
        c.press(input).expect("real boundary attempt");
        c.wait_frames(16).expect("boundary presentation");
        settle(c);
        assert_at(c, TOWN, point, "visible native buoy blocks unmatched edge");
        assert_eq!(
            c.snapshot().expect("boundary mode").overworld.mode,
            MovementMode::Surf
        );
    }
}

#[test]
#[ignore = "requires final composed native geography pack and caller save directory"]
fn grand_marais_superior_controller_surfs_closed_gate_all_rows_open_seam_and_boundaries() {
    let (_, r, mut c) = fixture(TOWN, (4, 20), false, false, false);
    let coast_x =
        i16::try_from(r.data().maps[COAST].attributes.width).expect("installed coast width") * 2
            - 1;
    use_max_repel(&mut c, &r);
    mount(&mut c, GameButton::Left, TOWN, (3, 20));
    path(&r, &mut c, (3, 16), true);
    for y in 16..37 {
        if y > 16 {
            step(
                &mut c,
                GameButton::Down,
                TOWN,
                (3, y),
                "inside water checkpoint column",
            );
        }
        refused(&mut c, GameButton::Left, (3, y), true);
    }
    blocked(&mut c, GameButton::Down, (3, 36));
    save(&mut c, "closed-surf-bottom-corner");
    let (root, r, mut c) = fixture(TOWN, (4, 20), true, true, false);
    use_max_repel(&mut c, &r);
    mount(&mut c, GameButton::Left, TOWN, (3, 20));
    for x in (0..3).rev() {
        step(
            &mut c,
            GameButton::Left,
            TOWN,
            (x, 20),
            "qualified Surf passes old west gate",
        );
    }
    step(
        &mut c,
        GameButton::Left,
        COAST,
        (coast_x, 20),
        "same-Superior outward seam",
    );
    step(
        &mut c,
        GameButton::Right,
        TOWN,
        (0, 20),
        "same-Superior inward seam",
    );
    step(
        &mut c,
        GameButton::Right,
        TOWN,
        (3, 20),
        "native inward Surf1->2->3",
    );
    assert_eq!(
        c.snapshot().expect("inward Surf").overworld.mode,
        MovementMode::Surf
    );
    let before = c.snapshot().expect("reached Surf state");
    let saved = save(&mut c, "surf-continue");
    c = VisibleShellController::load_save(root, r.clone(), saved, None)
        .expect("actual same-pack Continue");
    let after = c.snapshot().expect("continued Surf state");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.mode, after.overworld.mode);
    assert_eq!(before.overworld.facing, after.overworld.facing);
    assert_eq!(before.progression.badges, after.progression.badges);
    assert_eq!(
        before.progression.active_engine_flags,
        after.progression.active_engine_flags
    );
    assert_eq!(
        before.progression.active_event_flags,
        after.progression.active_event_flags
    );
    assert_eq!(
        before.progression.repel_steps_remaining,
        after.progression.repel_steps_remaining
    );
    assert_eq!(before.party, after.party);
    // Traverse every physical water lane through production input. The town
    // checkpoint has an extra denial row36; only16..35 are connected seams.
    for y in 16..36 {
        path(&r, &mut c, (3, y), true);
        for x in (0..3).rev() {
            step(&mut c, GameButton::Left, TOWN, (x, y), "open water lane");
        }
        step(
            &mut c,
            GameButton::Left,
            COAST,
            (coast_x, y),
            "each Superior outgoing lane",
        );
        step(
            &mut c,
            GameButton::Right,
            TOWN,
            (0, y),
            "each Superior incoming lane",
        );
        step(
            &mut c,
            GameButton::Right,
            TOWN,
            (3, y),
            "each native inward Surf wrapper",
        );
        assert_eq!(
            c.snapshot().expect("seam mode").overworld.mode,
            MovementMode::Surf
        );
    }
    path(&r, &mut c, (3, 36), true);
    for x in 3..39 {
        if x > 3 {
            step(
                &mut c,
                GameButton::Right,
                TOWN,
                (x, 36),
                "south inner harbor lane",
            );
        }
        blocked(&mut c, GameButton::Down, (x, 36));
    }
    for y in (16..36).rev() {
        step(
            &mut c,
            GameButton::Up,
            TOWN,
            (38, y),
            "east inner reef lane",
        );
        blocked(&mut c, GameButton::Right, (38, y));
    }
    save(&mut c, "east-and-south-reefs");
    // A separate unqualified exterior fixture mounts Surf through real input,
    // crosses the actual lake, then uses free inward care without adding flags.
    let (_, r, mut c) = fixture(COAST, (3, 13), false, false, true);
    use_max_repel(&mut c, &r);
    mount(&mut c, GameButton::Down, COAST, (3, 14));
    path(&r, &mut c, (coast_x, 20), true);
    step(
        &mut c,
        GameButton::Right,
        TOWN,
        (0, 20),
        "unqualified real incoming lake seam",
    );
    step(
        &mut c,
        GameButton::Right,
        TOWN,
        (3, 20),
        "unqualified native Surf inward1->2->3",
    );
    assert_eq!(
        c.snapshot().expect("free inward mode").overworld.mode,
        MovementMode::Surf
    );
    save(&mut c, "unqualified-surf-inward");
    step(
        &mut c,
        GameButton::Right,
        TOWN,
        (4, 20),
        "native harbor dismount for care",
    );
    nurse(&r, &mut c);
}

#[test]
#[ignore = "requires the selected warm Cedar approach pack and reached-save directory"]
fn grand_marais_superior_controller_existing_cedar_grass_wild_run_and_dry_ledge_return() {
    use super::super::authored_route_fixture::escape_wild;

    let (root, r, mut c) = fixture(INLAND, (0, 14), true, true, false);
    assert_eq!(
        r.data().maps[INLAND].attributes.tileset_name,
        "mn_north_shore"
    );
    assert!(terrain_path(&r, &mut c, (17, 14), false, true) >= 3);
    assert!(terrain_path(&r, &mut c, (0, 14), false, true) >= 3);
    terrain_path(&r, &mut c, (12, 10), false, true);
    assert_eq!(kind(&r, INLAND, (12, 10)), "TALL_GRASS");
    assert_eq!(kind(&r, INLAND, (13, 10)), "TALL_GRASS");
    let mut encountered = false;
    for index in 0..256 {
        let (input, target) = if index % 2 == 0 {
            (GameButton::Right, (13, 10))
        } else {
            (GameButton::Left, (12, 10))
        };
        for _ in 0..8 {
            c.press(input).expect("actual existing Cedar habitat step");
            c.wait_frames(16).expect("native grass presentation");
            if c.snapshot()
                .expect("actual encounter state")
                .battle
                .is_some()
            {
                escape_wild(&mut c);
                encountered = true;
            }
            if at(&mut c) == (INLAND.to_owned(), target) {
                break;
            }
        }
        assert_at(&mut c, INLAND, target, "walked existing Cedar grass");
        if encountered {
            break;
        }
    }
    assert!(
        encountered,
        "existing Cedar grass must start a native wild battle"
    );
    let before = c.snapshot().expect("reached grass resources");
    let reached = save(&mut c, "existing-cedar-grass");
    let mut loaded = VisibleShellController::load_save(root, r, reached, None)
        .expect("Continue actual existing-grass journey");
    let after = loaded.snapshot().expect("continued existing grass");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.progression.badges, after.progression.badges);
    assert_eq!(
        before.progression.active_event_flags,
        after.progression.active_event_flags
    );
    assert_eq!(
        before.progression.active_engine_flags,
        after.progression.active_engine_flags
    );
    if let Some(path) = std::env::var_os("CRYSTAL_CEDAR_GRASS_REACHED_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("retain actual grass save");
    }

    // Independent local placement proves the preserved one-way return. Party,
    // badges and initial positions above are setup, not earned regional access.
    let (_, r, mut c) = fixture(INLAND, (27, 20), true, true, false);
    assert!(terrain_path(&r, &mut c, (27, 17), false, true) >= 4);
    step(
        &mut c,
        GameButton::Down,
        INLAND,
        (27, 18),
        "ordinary ledge lip",
    );
    step(
        &mut c,
        GameButton::Down,
        INLAND,
        (27, 20),
        "native dry descent",
    );
    assert_eq!(kind(&r, INLAND, (27, 20)), "FLOOR");
    for _ in 0..3 {
        c.press(GameButton::Up)
            .expect("native reverse ledge denial");
        c.wait_frames(16).expect("settled reverse attempt");
        settle(&mut c);
        assert_at(
            &mut c,
            INLAND,
            (27, 20),
            "Cedar reverse ledge remains blocked",
        );
    }
}

#[path = "bemidji_northwoods_grass.rs"]
mod bemidji_northwoods_grass;

#[path = "detroit_approach_grass.rs"]
mod detroit_approach_grass;

#[path = "ely_shore_pass_grass.rs"]
mod ely_shore_pass_grass;

#[path = "mille_lacs_grass.rs"]
mod mille_lacs_grass;

#[path = "itasca_grass.rs"]
mod itasca_grass;

#[path = "preserve_pine_grass.rs"]
mod preserve_pine_grass;

#[path = "winona_mouth_review.rs"]
mod winona_mouth_review;

#[path = "lower_woods_grass.rs"]
mod lower_woods_grass;

#[path = "upstream_woods_grass.rs"]
mod upstream_woods_grass;

#[path = "confluence_grass.rs"]
mod confluence_grass;
