//! Guarded walking seams after explicit legal local party/flag/position setup.
//! These inputs do not earn eight badges, the harbor signal or Q07 restoration.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::world::map::Direction;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, neighbors, opposite};

const CAMP: &str = "MnBoundaryWaters";
const ROOM: &str = "MnBoundaryWatersSurveyShelter";
const EVENTS: [&str; 3] = [
    "EVENT_MN_LIGHTHOUSE_SIGNAL",
    "EVENT_MN_SURVEY_CHIEF_DEFEATED",
    "EVENT_MN_SURVEY_RESTORED",
];

struct Passage {
    route: &'static str,
    outward: GameButton,
    inner: [Point; 2],
    bodies: [Point; 2],
    outer: [Point; 2],
    camp_edge: [Point; 2],
    route_edge: [Point; 2],
    ids: [&'static str; 2],
    root: &'static str,
}

const PASSAGES: [Passage; 3] = [
    Passage {
        route: "MnR12C00R00",
        outward: GameButton::Right,
        inner: [(60, 24), (60, 25)],
        bodies: [(61, 24), (61, 25)],
        outer: [(62, 24), (62, 25)],
        camp_edge: [(63, 24), (63, 25)],
        route_edge: [(0, 28), (0, 29)],
        ids: [
            "MN_BOUNDARY_GUNFLINT_GUARD_NORTH",
            "MN_BOUNDARY_GUNFLINT_GUARD_SOUTH",
        ],
        root: "MnBoundaryGunflintCheck",
    },
    Passage {
        route: "MnR13C00R00",
        outward: GameButton::Down,
        inner: [(32, 42), (33, 42)],
        bodies: [(32, 43), (33, 43)],
        outer: [(32, 44), (33, 44)],
        camp_edge: [(32, 47), (33, 47)],
        route_edge: [(32, 0), (33, 0)],
        ids: [
            "MN_BOUNDARY_LEAGUE_GUARD_WEST",
            "MN_BOUNDARY_LEAGUE_GUARD_EAST",
        ],
        root: "MnBoundaryLeagueCheck",
    },
    Passage {
        route: "MnR20C01R00",
        outward: GameButton::Left,
        inner: [(3, 24), (3, 25)],
        bodies: [(2, 24), (2, 25)],
        outer: [(1, 24), (1, 25)],
        camp_edge: [(0, 24), (0, 25)],
        route_edge: [(63, 36), (63, 37)],
        ids: [
            "MN_BOUNDARY_VOYAGEURS_GUARD_NORTH",
            "MN_BOUNDARY_VOYAGEURS_GUARD_SOUTH",
        ],
        root: "MnBoundaryVoyageursCheck",
    },
];

fn load() -> (AssetRoot, CrystalRuntime) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_BOUNDARY_BASECAMP_CONNECTED_TEST_PACK")
            .expect("actual published connected471-map pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path)
            .expect("verified actual guarded-connection pack"),
    )
    .expect("native connected camp runtime");
    let connections = &runtime.data().maps[CAMP].attributes.connections;
    assert_eq!(connections.len(), 4);
    for target in [
        "MnR12C00R00",
        "MnR13C00R00",
        "MnR20C01R00",
        "MnPreservePineEntry",
    ] {
        assert!(
            connections
                .iter()
                .any(|connection| connection.target_map == target),
            "current basecamp connection missing: {target}"
        );
    }
    for id in PASSAGES.iter().flat_map(|p| p.ids) {
        assert!(
            runtime.data().maps[CAMP]
                .objects
                .iter()
                .any(|o| o.object_identifier.as_deref() == Some(id)),
            "pack must contain the actual visible checkpoint, not a bare town"
        );
    }
    (root, runtime)
}

#[derive(Clone, Copy)]
struct Setup {
    map: &'static str,
    at: Point,
    badges: [bool; 8],
    signal: bool,
    restored: bool,
    injured: bool,
}

fn fixture(root: &AssetRoot, runtime: &CrystalRuntime, setup: Setup) -> VisibleShellController {
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("original HOME"),
            map_name: setup.map.into(),
            tile_x: setup.at.0,
            tile_y: setup.at.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local connection-review placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture trainer name");
    let mut pokemon = runtime
        .data()
        .create_pokemon("MEW", 80, Dv::default())
        .expect("legal native review partner");
    pokemon.moves = vec![LearnedMove {
        name: "SURF".into(),
        current_pp: runtime.data().moves["SURF"].pp,
        pp_ups: 0,
    }];
    if setup.injured {
        pokemon.hp -= 60;
        pokemon.status = Some("POISON".into());
        pokemon.moves[0].current_pp = 0;
    }
    let state = controller.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(pokemon));
    state.sync_party_from_storage();
    state.money = 0;
    state.badges.johto = setup.badges;
    for (name, value) in EVENTS
        .into_iter()
        .zip([setup.signal, setup.restored, setup.restored])
    {
        state
            .flags
            .set_event_flag(name, value)
            .expect("explicit upstream/Q07 fixture only");
    }
    state.random_state = Default::default();
    *controller.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    // Every write above is initial setup. There are no later state, clock,
    // divider, badge, visibility, story-event, position or HP/PP injections.
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_at(&mut controller, setup.map, setup.at);
    assert_no_safari(&controller);
    controller
}

fn progress(controller: &VisibleShellController) -> [bool; 3] {
    EVENTS.map(|name| {
        controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set(name)
            .expect("registered story event")
    })
}

fn assert_no_safari(controller: &VisibleShellController) {
    let state = controller.shell.shell.session().state();
    assert_eq!(
        state.flags.is_engine_flag_set("ENGINE_SAFARI_ZONE"),
        Ok(false)
    );
    assert!(!state.script_runtime.memory.contains_key("wSafariBalls"));
    assert!(!state.script_runtime.memory.contains_key("wSafariSteps"));
    assert_eq!(state.money, 0);
}

fn location(controller: &mut VisibleShellController) -> (String, Point) {
    let snapshot = controller.snapshot().expect("actual production location");
    (
        snapshot.overworld.map_name,
        (snapshot.overworld.tile.x, snapshot.overworld.tile.y),
    )
}

fn assert_at(controller: &mut VisibleShellController, map: &str, at: Point) {
    assert_eq!(location(controller), (map.into(), at));
}

fn press(controller: &mut VisibleShellController, button: GameButton) {
    controller
        .press(button)
        .expect("production checkpoint input");
    controller
        .wait_frames(16)
        .expect("native presentation frames");
    assert_eq!(controller.shell.last_error, None);
}

fn settle(controller: &mut VisibleShellController) -> BTreeSet<String> {
    let mut labels = BTreeSet::new();
    for _ in 0..192 {
        let snapshot = controller.snapshot().expect("native checkpoint ownership");
        assert!(
            snapshot.battle.is_none(),
            "no battle expected on these checked ordinary Floor cells"
        );
        assert!(
            snapshot.ui.pending_yes_no.is_none(),
            "no invented admission prompt"
        );
        if let Some(text) = &snapshot.ui.text {
            labels.insert(text.label.clone());
        }
        if snapshot.phase == RuntimeShellPhase::Overworld
            && !snapshot.ui.text_window_open
            && !controller.has_pending_script_work()
            && controller.shell.field_notice.is_none()
            && controller.shell.battle_messages.is_empty()
        {
            return labels;
        }
        press(controller, GameButton::A);
    }
    panic!(
        "checkpoint did not release input: {:?}",
        controller.snapshot().expect("diagnosis")
    );
}

fn advance(controller: &mut VisibleShellController, button: GameButton, map: &str, target: Point) {
    for _ in 0..8 {
        if location(controller) == (map.into(), target) {
            return;
        }
        press(controller, button);
        let labels = settle(controller);
        assert!(
            !labels.iter().any(|label| label.contains("BadgesText")
                || label.contains("SignalText")
                || label.contains("RestoredText")),
            "unexpected refusal on a qualified/ordinary step: {labels:?}"
        );
    }
    assert_at(controller, map, target);
}

fn step(point: Point, button: GameButton) -> Point {
    let (dx, dy) = match button {
        GameButton::Up => (0, -1),
        GameButton::Down => (0, 1),
        GameButton::Left => (-1, 0),
        GameButton::Right => (1, 0),
        _ => panic!("walking direction"),
    };
    (point.0 + dx, point.1 + dy)
}

fn visible_pair(controller: &VisibleShellController, p: &Passage, visible: bool) {
    let session = controller.shell.shell.session().overworld();
    assert_eq!(session.map.name, CAMP);
    for id in p.ids {
        let actor = session
            .objects
            .iter()
            .find(|o| o.object_identifier.as_deref() == Some(id))
            .expect("actual native checkpoint actor");
        assert_eq!(actor.event_flag, "-1");
        assert_eq!(session.is_object_visible(actor), visible, "{id}");
    }
}

fn not_stranded(controller: &mut VisibleShellController) {
    let (map, at) = location(controller);
    if map != CAMP {
        return;
    }
    assert!(
        !PASSAGES
            .iter()
            .flat_map(|p| p.bodies)
            .any(|body| body == at),
        "trainer cannot settle on a checkpoint body"
    );
}

/// Fixture-only ordinary Floor path, excluding actual currently visible bodies.
/// It does not decide script, hidden-object, movement or connection behavior.
fn walk(controller: &mut VisibleShellController, runtime: &CrystalRuntime, to: Point) {
    let (name, from) = location(controller);
    let module = &runtime.data().maps[&name];
    let session = controller.shell.shell.session().overworld();
    let blocked: BTreeSet<_> = session
        .objects
        .iter()
        .filter(|o| session.is_object_visible(o))
        .map(|o| (o.x as i16, o.y as i16))
        .collect();
    let mut floor = BTreeSet::new();
    let collision = &runtime.data().tilesets[&module.attributes.tileset_name].collision;
    for y in 0..module.attributes.height * 2 {
        for x in 0..module.attributes.width * 2 {
            let block = module.blocks
                [usize::from(y / 2) * usize::from(module.attributes.width) + usize::from(x / 2)];
            if matches!(
                collision[&format!("{block:02x}")][usize::from(y % 2 * 2 + x % 2)].as_str(),
                "FLOOR" | "LADDER" | "WARP_CARPET_DOWN"
            ) {
                floor.insert((x as i16, y as i16));
            }
        }
    }
    if name == CAMP {
        // A sibling inner-plane cell can legitimately run a refusal and push
        // the trainer away from a precomputed ordinary path. Treat checked
        // passages as authored interaction hazards for the care tour; direct
        // checkpoint inputs below still exercise every plane and both lanes.
        for at in PASSAGES.iter().flat_map(|passage| passage.inner) {
            if at != from && at != to {
                floor.remove(&at);
            }
        }
    }
    let mut parents = BTreeMap::from([(from, (from, GameButton::A))]);
    let mut queue = VecDeque::from([from]);
    while let Some(at) = queue.pop_front() {
        if at == to {
            break;
        }
        for (next, button) in neighbors(at) {
            if floor.contains(&next) && !blocked.contains(&next) && !parents.contains_key(&next) {
                parents.insert(next, (at, button));
                queue.push_back(next);
            }
        }
    }
    assert!(
        parents.contains_key(&to),
        "no ordinary visible-occupancy path {name}:{from:?}→{to:?}"
    );
    let mut path = Vec::new();
    let mut at = to;
    while at != from {
        let (prior, button) = parents[&at];
        path.push((at, button));
        at = prior;
    }
    path.reverse();
    for (target, button) in path {
        for _ in 0..8 {
            if location(controller) == (name.clone(), target) {
                break;
            }
            press(controller, button);
            settle(controller);
            let (now, _) = location(controller);
            if now != name {
                assert_eq!(
                    name, ROOM,
                    "only real room carpets may leave this walking planner"
                );
                return;
            }
        }
        assert_at(controller, &name, target);
    }
}

fn enter_room(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    walk(controller, runtime, (43, 20));
    advance(controller, GameButton::Up, ROOM, (8, 9));
    press(controller, GameButton::A);
    settle(controller);
    assert_at(controller, ROOM, (8, 9));
}

fn return_from_room(
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    carpet: Point,
) {
    walk(controller, runtime, (carpet.0, 10));
    advance(controller, GameButton::Down, CAMP, (43, 20));
    press(controller, GameButton::A);
    settle(controller);
    assert_at(controller, CAMP, (43, 20));
}

fn continue_save(
    controller: &mut VisibleShellController,
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    stage: &str,
) -> VisibleShellController {
    let before = controller.snapshot().expect("before native save");
    let flags = progress(controller);
    let badges = controller.shell.shell.session().state().badges.clone();
    let visibility: Vec<_> = if before.overworld.map_name == CAMP {
        PASSAGES
            .iter()
            .map(|p| {
                p.ids.map(|id| {
                    let session = controller.shell.shell.session().overworld();
                    let actor = session
                        .objects
                        .iter()
                        .find(|o| o.object_identifier.as_deref() == Some(id))
                        .expect("guard");
                    session.is_object_visible(actor)
                })
            })
            .collect()
    } else {
        Vec::new()
    };
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unique save name")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-connected-camp-{stage}-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    controller
        .save(&path)
        .expect("production connected checkpoint save");
    let mut loaded =
        VisibleShellController::load_save(root.clone(), runtime.clone(), path.clone(), None)
            .expect("Continue actual connected camp");
    let after = loaded.snapshot().expect("continued guard state");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.mode, after.overworld.mode);
    assert_eq!(progress(&loaded), flags);
    assert_eq!(loaded.shell.shell.session().state().badges, badges);
    if !visibility.is_empty() {
        for (p, expected) in PASSAGES.iter().zip(visibility) {
            let session = loaded.shell.shell.session().overworld();
            for (id, visible) in p.ids.into_iter().zip(expected) {
                let actor = session
                    .objects
                    .iter()
                    .find(|o| o.object_identifier.as_deref() == Some(id))
                    .expect("continued guard");
                assert_eq!(
                    session.is_object_visible(actor),
                    visible,
                    "Continue must preserve actual current-map visibility"
                );
            }
        }
    }
    assert_no_safari(&loaded);
    not_stranded(&mut loaded);
    if let Some(directory) = std::env::var_os("CRYSTAL_BOUNDARY_BASECAMP_CONNECTIONS_SAVE_DIR") {
        let directory = PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("owned save directory");
        loaded
            .save(directory.join(format!(
                "{stage}-{}-{nonce}.crystalsave",
                std::process::id()
            )))
            .expect("retain unique actual checkpoint save");
    }
    std::fs::remove_file(path).expect("remove temporary save only, never renders");
    loaded
}

#[test]
#[ignore = "requires the actual published connected Boundary Waters pack"]
fn boundary_basecamp_connections_controller_twelve_walk_seams_reentry_and_continue() {
    let (root, runtime) = load();
    let mut crossings = 0;
    for p in &PASSAGES {
        for lane in 0..2 {
            let inward = opposite(p.outward);
            let start = step(p.inner[lane], inward);
            let mut controller = fixture(
                &root,
                &runtime,
                Setup {
                    map: CAMP,
                    at: start,
                    badges: [true; 8],
                    signal: true,
                    restored: true,
                    injured: false,
                },
            );
            visible_pair(&controller, p, true);
            advance(&mut controller, p.outward, CAMP, p.inner[lane]);
            visible_pair(&controller, p, false);
            advance(&mut controller, p.outward, CAMP, p.bodies[lane]);
            advance(&mut controller, p.outward, CAMP, p.outer[lane]);
            // Outward movement enters the outer plane facing away from care, so
            // the inward-only source root must not escort the trainer backward.
            assert_eq!(
                controller.shell.shell.session().overworld().player.facing,
                button_direction(p.outward)
            );
            advance(&mut controller, p.outward, CAMP, p.camp_edge[lane]);
            advance(&mut controller, p.outward, p.route, p.route_edge[lane]);
            crossings += 1;
            advance(&mut controller, inward, CAMP, p.camp_edge[lane]);
            crossings += 1;
            advance(&mut controller, inward, CAMP, p.inner[lane]);
            visible_pair(&controller, p, false);
            not_stranded(&mut controller);
            assert_eq!(progress(&controller), [true; 3]);
            assert_no_safari(&controller);
            controller = continue_save(&mut controller, &root, &runtime, "qualified-inner");
            enter_room(&mut controller, &runtime);
            return_from_room(
                &mut controller,
                &runtime,
                if lane == 0 { (8, 11) } else { (9, 11) },
            );
            walk(&mut controller, &runtime, start);
            advance(&mut controller, p.outward, CAMP, p.inner[lane]);
            visible_pair(&controller, p, false);
            assert_eq!(progress(&controller), [true; 3]);
            not_stranded(&mut controller);
        }
    }
    assert_eq!(crossings, 12);
}

fn button_direction(button: GameButton) -> Direction {
    match button {
        GameButton::Up => Direction::Up,
        GameButton::Down => Direction::Down,
        GameButton::Left => Direction::Left,
        GameButton::Right => Direction::Right,
        _ => panic!("direction"),
    }
}

#[test]
#[ignore = "requires the actual published connected Boundary Waters pack"]
fn boundary_basecamp_connections_controller_every_badge_signal_and_restoration_refusal() {
    let (root, runtime) = load();
    for p in &PASSAGES {
        for missing in 0..if p.root == "MnBoundaryLeagueCheck" {
            10
        } else {
            9
        } {
            let lane = missing % 2;
            let start = step(p.inner[lane], opposite(p.outward));
            let mut badges = [true; 8];
            if missing < 8 {
                badges[missing] = false;
            }
            let signal = missing != 8;
            let restored = missing != 9;
            let mut controller = fixture(
                &root,
                &runtime,
                Setup {
                    map: CAMP,
                    at: start,
                    badges,
                    signal,
                    restored,
                    injured: false,
                },
            );
            let initial = controller.snapshot().expect("refusal initial resources");
            let flags = progress(&controller);
            let suffix = if missing < 8 {
                "BadgesText"
            } else if missing == 8 {
                "SignalText"
            } else {
                "RestoredText"
            };
            let expected = format!("{}{suffix}", p.root);
            let mut labels = BTreeSet::new();
            for _ in 0..8 {
                press(&mut controller, p.outward);
                labels.extend(settle(&mut controller));
                if labels.contains(&expected) {
                    break;
                }
            }
            assert!(
                labels.contains(&expected),
                "missing actual refusal {expected}: {labels:?}"
            );
            assert_at(&mut controller, CAMP, start);
            visible_pair(&controller, p, true);
            let after = controller.snapshot().expect("refused departure");
            assert_eq!(after.party, initial.party);
            assert_eq!(after.bag, initial.bag);
            assert_eq!(progress(&controller), flags);
            assert_eq!(
                controller.shell.shell.session().state().badges.johto,
                badges
            );
            assert_no_safari(&controller);
            not_stranded(&mut controller);
        }
    }
}

#[test]
#[ignore = "requires the actual published connected Boundary Waters pack"]
fn boundary_basecamp_connections_controller_unqualified_inbound_care_dock_heal_and_continue() {
    let (root, runtime) = load();
    for p in &PASSAGES {
        for lane in 0..2 {
            let mut controller = fixture(
                &root,
                &runtime,
                Setup {
                    map: p.route,
                    at: p.route_edge[lane],
                    badges: [false; 8],
                    signal: false,
                    restored: false,
                    injured: true,
                },
            );
            let initial = controller.snapshot().expect("injured inbound resources");
            let inward = opposite(p.outward);
            advance(&mut controller, inward, CAMP, p.camp_edge[lane]);
            // Move toward the outer plane; its real native source escorts across
            // the cleared pair. No hidden-object or position writes are allowed.
            for _ in 0..16 {
                if location(&mut controller) == (CAMP.into(), p.inner[lane]) {
                    break;
                }
                press(&mut controller, inward);
                settle(&mut controller);
            }
            assert_at(&mut controller, CAMP, p.inner[lane]);
            visible_pair(&controller, p, true);
            not_stranded(&mut controller);
            walk(&mut controller, &runtime, (13, 14));
            assert_at(&mut controller, CAMP, (13, 14));
            enter_room(&mut controller, &runtime);
            walk(&mut controller, &runtime, (4, 7));
            press(&mut controller, GameButton::Left);
            assert_at(&mut controller, ROOM, (4, 7));
            press(&mut controller, GameButton::A);
            assert!(settle(&mut controller).contains("MN_BOUNDARY_SURVEY_MEDICText"));
            let healed = controller.snapshot().expect("actual native free treatment");
            let partner = &healed.party.slots[0].pokemon;
            assert_eq!(partner.hp, partner.max_hp);
            assert_eq!(partner.status, None);
            assert_eq!(partner.moves[0].current_pp, runtime.data().moves["SURF"].pp);
            assert_eq!(healed.bag, initial.bag);
            return_from_room(
                &mut controller,
                &runtime,
                if lane == 0 { (8, 11) } else { (9, 11) },
            );
            assert_eq!(progress(&controller), [false; 3]);
            assert_eq!(
                controller.shell.shell.session().state().badges.johto,
                [false; 8]
            );
            assert_no_safari(&controller);
            controller = continue_save(&mut controller, &root, &runtime, "unqualified-free-care");
            let start = step(p.inner[lane], inward);
            walk(&mut controller, &runtime, start);
            let expected = format!("{}BadgesText", p.root);
            let mut labels = BTreeSet::new();
            for _ in 0..8 {
                press(&mut controller, p.outward);
                labels.extend(settle(&mut controller));
                if labels.contains(&expected) {
                    break;
                }
            }
            assert!(labels.contains(&expected));
            assert_at(&mut controller, CAMP, start);
            visible_pair(&controller, p, true);
            assert_eq!(progress(&controller), [false; 3]);
            not_stranded(&mut controller);
        }
    }
}

#[test]
#[ignore = "requires the actual published connected Boundary Waters pack"]
fn boundary_basecamp_connections_controller_transverse_outer_arrivals_escort_to_free_care() {
    let (root, runtime) = load();
    for p in &PASSAGES {
        for lane in 0..2 {
            // Start on the opposite outer lane as an explicit local review setup,
            // then physically walk sideways onto this plane. The safe passage
            // escorts transverse arrivals inward too: leaving them here could
            // strand a trainer when a guard blocks the next inward step.
            // Outward/inward arrivals are covered by the seam/care fixtures.
            let from = p.outer[1 - lane];
            let target = p.outer[lane];
            let direction = neighbors(from)
                .into_iter()
                .find(|(at, _)| *at == target)
                .expect("adjacent outer lanes")
                .1;
            let mut controller = fixture(
                &root,
                &runtime,
                Setup {
                    map: CAMP,
                    at: from,
                    badges: [false; 8],
                    signal: false,
                    restored: false,
                    injured: false,
                },
            );
            for _ in 0..8 {
                if location(&mut controller) == (CAMP.into(), p.inner[lane]) {
                    break;
                }
                press(&mut controller, direction);
                settle(&mut controller);
            }
            assert_at(&mut controller, CAMP, p.inner[lane]);
            assert_eq!(
                controller.shell.shell.session().overworld().player.facing,
                button_direction(opposite(p.outward)),
                "native escort ends facing inward; transverse is the actual initiating input"
            );
            visible_pair(&controller, p, true);
            assert_eq!(progress(&controller), [false; 3]);
            not_stranded(&mut controller);
            controller = continue_save(&mut controller, &root, &runtime, "transverse-inward-care");
            let inward = opposite(p.outward);
            let start = step(p.inner[lane], inward);
            walk(&mut controller, &runtime, start);
            let expected = format!("{}BadgesText", p.root);
            let mut labels = BTreeSet::new();
            for _ in 0..8 {
                press(&mut controller, p.outward);
                labels.extend(settle(&mut controller));
                if labels.contains(&expected) {
                    break;
                }
            }
            assert!(labels.contains(&expected));
            assert_at(&mut controller, CAMP, start);
            visible_pair(&controller, p, true);
            not_stranded(&mut controller);
            assert_no_safari(&controller);
        }
    }
}

fn collision(runtime: &CrystalRuntime, at: Point) -> Option<&str> {
    let map = &runtime.data().maps[CAMP];
    let (x, y) = (u16::try_from(at.0).ok()?, u16::try_from(at.1).ok()?);
    if x >= map.attributes.width * 2 || y >= map.attributes.height * 2 {
        return None;
    }
    let block =
        map.blocks[usize::from(y / 2) * usize::from(map.attributes.width) + usize::from(x / 2)];
    runtime.data().tilesets[&map.attributes.tileset_name]
        .collision
        .get(&format!("{block:02x}"))
        .map(|quadrants| quadrants[usize::from(y % 2 * 2 + x % 2)].as_str())
}

fn water_paths(runtime: &CrystalRuntime, from: Point) -> BTreeMap<Point, (Point, GameButton)> {
    let mut parents = BTreeMap::from([(from, (from, GameButton::A))]);
    let mut queue = VecDeque::from([from]);
    while let Some(at) = queue.pop_front() {
        for (next, button) in neighbors(at) {
            // Ordinary WATER only. This fixture planner never treats rocks,
            // directional buoys, land or arbitrary collision as Surf Floor.
            if collision(runtime, next) == Some("WATER") && !parents.contains_key(&next) {
                parents.insert(next, (at, button));
                queue.push_back(next);
            }
        }
    }
    parents
}

fn paddle(controller: &mut VisibleShellController, runtime: &CrystalRuntime, to: Point) {
    let (map, from) = location(controller);
    assert_eq!(map, CAMP);
    let parents = water_paths(runtime, from);
    assert!(parents.contains_key(&to), "ordinary native water path");
    let mut path = Vec::new();
    let mut at = to;
    while at != from {
        let (prior, button) = parents[&at];
        path.push((at, button));
        at = prior;
    }
    path.reverse();
    for (target, button) in path {
        advance(controller, button, CAMP, target);
        assert_eq!(
            controller
                .snapshot()
                .expect("actual paddling mode")
                .overworld
                .mode,
            MovementMode::Surf
        );
    }
}

#[test]
#[ignore = "requires the actual published connected Boundary Waters pack"]
fn boundary_basecamp_connections_controller_public_surf_banks_cannot_open_signal_checkpoints() {
    let (root, runtime) = load();
    let mut controller = fixture(
        &root,
        &runtime,
        Setup {
            map: CAMP,
            at: (13, 14),
            badges: [true; 8],
            signal: false,
            restored: false,
            injured: false,
        },
    );
    let initial = controller.snapshot().expect("public lake resources");
    assert_eq!(collision(&runtime, (13, 14)), Some("FLOOR"));
    assert_eq!(collision(&runtime, (13, 13)), Some("WATER"));
    press(&mut controller, GameButton::Up);
    assert_at(&mut controller, CAMP, (13, 14));
    press(&mut controller, GameButton::A);
    for _ in 0..128 {
        if controller
            .snapshot()
            .expect("actual native Surf prompt")
            .overworld
            .mode
            == MovementMode::Surf
        {
            break;
        }
        press(&mut controller, GameButton::A);
    }
    controller
        .wait_frames(60)
        .expect("native Surf source animation");
    settle(&mut controller);
    assert_at(&mut controller, CAMP, (13, 13));
    assert_eq!(
        controller
            .snapshot()
            .expect("actual public Surf")
            .overworld
            .mode,
        MovementMode::Surf
    );
    let reachable = water_paths(&runtime, (13, 13));
    let native = &runtime;
    for p in &PASSAGES {
        let distance = |at: Point| at.0.abs_diff(p.inner[0].0) + at.1.abs_diff(p.inner[0].1);
        let (at, direction) = reachable
            .keys()
            .flat_map(|at| {
                neighbors(*at)
                    .into_iter()
                    .filter_map(move |(next, button)| {
                        (collision(native, next) == Some("WALL") && distance(next) < distance(*at))
                            .then_some((*at, button))
                    })
            })
            .min_by_key(|(at, _)| distance(*at))
            .expect("actual closed forest bank toward checkpoint");
        paddle(&mut controller, &runtime, at);
        for _ in 0..4 {
            press(&mut controller, direction);
            settle(&mut controller);
            assert_at(&mut controller, CAMP, at);
            assert_eq!(
                controller
                    .snapshot()
                    .expect("native closed shore")
                    .overworld
                    .mode,
                MovementMode::Surf
            );
        }
        visible_pair(&controller, p, true);
        assert_eq!(progress(&controller), [false; 3]);
    }
    paddle(&mut controller, &runtime, (13, 13));
    advance(&mut controller, GameButton::Down, CAMP, (13, 14));
    assert_eq!(
        controller
            .snapshot()
            .expect("real native dismount")
            .overworld
            .mode,
        MovementMode::Normal
    );
    for p in &PASSAGES {
        let start = step(p.inner[0], opposite(p.outward));
        walk(&mut controller, &runtime, start);
        let expected = format!("{}SignalText", p.root);
        let mut labels = BTreeSet::new();
        for _ in 0..8 {
            press(&mut controller, p.outward);
            labels.extend(settle(&mut controller));
            if labels.contains(&expected) {
                break;
            }
        }
        assert!(
            labels.contains(&expected),
            "legal Surf cannot bypass required harbor signal"
        );
        assert_at(&mut controller, CAMP, start);
        visible_pair(&controller, p, true);
    }
    assert_eq!(
        controller.snapshot().expect("lake trip bag").bag,
        initial.bag
    );
    assert_eq!(progress(&controller), [false; 3]);
    assert_no_safari(&controller);
    let _continued = continue_save(&mut controller, &root, &runtime, "public-surf-closed-gates");
}
