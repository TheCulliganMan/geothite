//! Explicit legal Floor/party/badge fixtures, then production inputs only.
//! These local park cases do not prove earned regional campaign progression.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;
use crystal_core::world::session::ConnectionTrigger;
use crystal_runtime::CrystalRuntime;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::super::r10_gallery::{completed_visible_page, flags};
use super::grand_marais_harbor::use_max_repel;
use super::{
    Point, VisibleShellController, assert_at, at, collision, fixture_with_items, settle, step,
};

const PARK: &str = "MnR04C00R00";
const ITEM_FLAG: &str = "EVENT_MN_R04_ITEM_02";
const CONTRACT: &str = "7c55b8813a53655daaa8dba88507e7a2f4d7a8e4b4d3c8ee2f90f7c1973a7cfb";

#[derive(Deserialize)]
struct Input {
    button: String,
    from: [i16; 2],
    to: [i16; 2],
}

#[derive(Deserialize)]
struct Course {
    id: String,
    from_core: [i16; 2],
    to_core: [i16; 2],
    ordinary_steps: usize,
    grass_steps: usize,
    inputs: Vec<Input>,
}

#[derive(Deserialize)]
struct InputPlan {
    schema_version: u32,
    map: String,
    pack_sha256: String,
    candidate_contract_sha256: String,
    courses: Vec<Course>,
}

fn plan() -> &'static InputPlan {
    static PLAN: OnceLock<InputPlan> = OnceLock::new();
    PLAN.get_or_init(|| {
        let path = std::env::var_os("CRYSTAL_PARK_POND_INPUT_PLAN")
            .expect("published-pack-bound park input plan");
        let plan: InputPlan = serde_json::from_slice(&std::fs::read(path).expect("park plan"))
            .expect("bound park input plan schema");
        let pack = std::env::var_os("CRYSTAL_MINNEAPOLIS_FALLS_CONTROLLER_PACK")
            .expect("actual published park pack");
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(std::fs::read(pack).expect("native pack"))
            ),
            plan.pack_sha256,
            "input plan must bind the actual written pack"
        );
        assert_eq!(plan.schema_version, 1);
        assert_eq!(plan.map, PARK);
        assert_eq!(plan.candidate_contract_sha256, CONTRACT);
        plan
    })
}

fn fixture(point: Point) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    plan();
    let (root, r, mut c) =
        fixture_with_items(PARK, point, false, false, false, &[("MAX_REPEL", 2)]);
    assert_eq!(r.data().maps.len(), 683);
    assert_eq!(collision(&r, &mut c, point), "FLOOR");
    use_max_repel(&mut c, &r);
    (root, r, c)
}

fn button(value: &str) -> GameButton {
    match value {
        "Up" => GameButton::Up,
        "Down" => GameButton::Down,
        "Left" => GameButton::Left,
        "Right" => GameButton::Right,
        other => panic!("unknown authored park button: {other}"),
    }
}

fn adjacent_button(from: Point, to: Point) -> GameButton {
    match (to.0 - from.0, to.1 - from.1) {
        (0, -1) => GameButton::Up,
        (0, 1) => GameButton::Down,
        (-1, 0) => GameButton::Left,
        (1, 0) => GameButton::Right,
        other => panic!("nonadjacent park input: {from:?} -> {to:?}: {other:?}"),
    }
}

fn walk(c: &mut VisibleShellController, point: Point) {
    let before = at(c).1;
    step(
        c,
        adjacent_button(before, point),
        PARK,
        point,
        "authored park Foot input",
    );
    let s = c.snapshot().expect("native park walking");
    assert_eq!(s.overworld.mode, MovementMode::Normal);
    assert!(
        s.battle.is_none(),
        "authored route avoids existing trainer sight"
    );
}

fn line(c: &mut VisibleShellController, target: Point) {
    let mut point = at(c).1;
    assert!(point.0 == target.0 || point.1 == target.1);
    while point != target {
        point.0 += (target.0 - point.0).signum();
        point.1 += (target.1 - point.1).signum();
        walk(c, point);
    }
}

fn course(c: &mut VisibleShellController, r: &CrystalRuntime, id: &str, reverse: bool) -> usize {
    let course = plan()
        .courses
        .iter()
        .find(|v| v.id == id)
        .expect("frozen park course");
    assert_eq!(course.inputs.len(), course.ordinary_steps);
    let (start, end) = if reverse {
        (course.to_core, course.from_core)
    } else {
        (course.from_core, course.to_core)
    };
    assert_at(c, PARK, start.into(), id);
    let mut grass = 0;
    for step_index in 0..course.inputs.len() {
        let index = if reverse {
            course.inputs.len() - 1 - step_index
        } else {
            step_index
        };
        let input = &course.inputs[index];
        // Check the source button as well as the exact endpoint schema.
        assert_eq!(
            button(&input.button),
            adjacent_button(input.from.into(), input.to.into())
        );
        let (from, to) = if reverse {
            (input.to, input.from)
        } else {
            (input.from, input.to)
        };
        assert_at(c, PARK, from.into(), id);
        let tile = collision(r, c, to.into());
        assert!(matches!(tile.as_str(), "FLOOR" | "TALL_GRASS"));
        grass += usize::from(tile == "TALL_GRASS");
        walk(c, to.into());
        if to == [40, 20] || to == [46, 20] {
            retained_save(c, &format!("{id}-east-bend-{}-{}", to[0], to[1]));
        }
    }
    assert_eq!(
        grass, course.grass_steps,
        "{id}: actual native grass entries"
    );
    assert_at(c, PARK, end.into(), id);
    grass
}

fn retained_save(c: &mut VisibleShellController, label: &str) -> PathBuf {
    let directory = std::env::var_os("CRYSTAL_PARK_POND_SAVE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("geothite-park-pond"));
    std::fs::create_dir_all(&directory).expect("retained park saves");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("save clock")
        .as_nanos();
    let path = directory.join(format!(
        "{label}-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    assert!(!path.exists(), "preserve every previous park save");
    c.save(&path).expect("production park Save");
    path
}

fn continue_game(
    root: &AssetRoot,
    r: &CrystalRuntime,
    c: &mut VisibleShellController,
    label: &str,
) {
    let before = c.snapshot().expect("park Continue preimage");
    let before_flags = flags(c);
    let path = retained_save(c, label);
    *c = VisibleShellController::load_save(root.clone(), r.clone(), path, None)
        .expect("native park Continue on the same pack identity");
    let after = c.snapshot().expect("continued park");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.mode, after.overworld.mode);
    assert_eq!(before.overworld.facing, after.overworld.facing);
    assert_eq!(before_flags, flags(c));
}

fn blocked(c: &mut VisibleShellController, direction: GameButton, point: Point) {
    for _ in 0..2 {
        c.press(direction).expect("native bank or gold rail input");
        c.wait_frames(16).expect("blocking-edge presentation");
        let s = c.snapshot().expect("actual blocked input");
        assert!(!s.ui.text_window_open && s.ui.menu.is_none());
        assert!(c.shell.field_notice.is_none() && !c.has_pending_script_work());
        assert_at(c, PARK, point, "native bank or gold rail blocks movement");
        assert_eq!(s.overworld.mode, MovementMode::Normal);
    }
}

fn town_return(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    direction: &str,
    key: GameButton,
) {
    let point = at(c).1;
    let map = &r.data().maps[PARK];
    let connection = map
        .attributes
        .connections
        .iter()
        .find(|v| v.direction == direction)
        .expect("existing park town connection")
        .clone();
    let target = if direction == "west" {
        "MnMinneapolis"
    } else {
        "MnStPaul"
    };
    assert_eq!(connection.target_map, target);
    assert_eq!(
        connection.offset,
        if direction == "west" { -9 } else { -15 }
    );
    let outside = if direction == "west" {
        (-1, point.1)
    } else {
        (
            i16::try_from(map.attributes.width * 2).expect("park core width"),
            point.1,
        )
    };
    let destination = r
        .data()
        .resolve_connection_transition(&ConnectionTrigger {
            map_name: PARK.into(),
            tile: TilePosition::new(outside.0, outside.1),
            connection,
        })
        .expect("native connection arrival derived from actual attributes")
        .destination;
    let arrival = (destination.tile.x, destination.tile.y);
    step(c, key, target, arrival, "actual park-to-town seam");
    assert_eq!(
        c.snapshot().expect("town walking").overworld.mode,
        MovementMode::Normal
    );
    retained_save(c, &format!("park-town-{direction}-arrival"));
    let back = if direction == "west" {
        GameButton::Right
    } else {
        GameButton::Left
    };
    step(c, back, PARK, point, "actual town-to-park paired return");
}

fn printed_dialogue(c: &mut VisibleShellController) -> String {
    let mut pages = BTreeMap::new();
    for _ in 0..64 {
        c.wait_frames(240).expect("complete native park text page");
        if let Some((index, page)) = completed_visible_page(c) {
            pages.insert(index, page);
        }
        let s = c.snapshot().expect("actual native park dialogue");
        assert!(s.battle.is_none() && s.ui.menu.is_none());
        if !s.ui.text_window_open
            && s.ui.pending_text_wait.is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
            && c.shell.visible_field_item_notice.is_none()
        {
            return pages.into_values().collect::<Vec<_>>().join("\n");
        }
        c.press(GameButton::A)
            .expect("acknowledge native park page");
        c.wait_frames(16).expect("park page continuation");
    }
    panic!("park text did not close at {:?}: {pages:?}", at(c));
}

#[test]
#[ignore = "requires the published park pack and bound CRYSTAL_PARK_POND_INPUT_PLAN"]
fn park_pond_ordinary_paths_pay_four_grass_and_cross_both_town_seams() {
    let (root, r, mut c) = fixture((0, 18));
    town_return(&mut c, &r, "west", GameButton::Left);
    assert_eq!(course(&mut c, &r, "both-ports-west-east", false), 4);
    town_return(&mut c, &r, "east", GameButton::Right);
    continue_game(&root, &r, &mut c, "park-east-edge");
    assert_eq!(course(&mut c, &r, "both-ports-east-west", false), 4);
    town_return(&mut c, &r, "west", GameButton::Left);
    continue_game(&root, &r, &mut c, "park-west-edge-return");
}

#[test]
#[ignore = "requires the published park pack and bound CRYSTAL_PARK_POND_INPUT_PLAN"]
fn park_pond_bridge_two_walk_rows_block_gold_rails_keep_bank_bays() {
    let (root, r, mut c) = fixture((24, 18));
    for y in [18, 19] {
        line(&mut c, (24, y));
        line(&mut c, (28, y));
        let (key, rail) = if y == 18 {
            (GameButton::Up, (28, 17))
        } else {
            (GameButton::Down, (28, 20))
        };
        assert_eq!(collision(&r, &mut c, rail), "WALL");
        blocked(&mut c, key, (28, y));
        continue_game(&root, &r, &mut c, &format!("park-bridge-row-{y}"));
        line(&mut c, (33, y));
        line(&mut c, (24, y));
    }
    for x in [24, 25, 32, 33] {
        line(&mut c, (x, 19));
        for y in 17..=20 {
            assert_eq!(collision(&r, &mut c, (x, y)), "FLOOR");
        }
        line(&mut c, (x, 17));
        line(&mut c, (x, 20));
        retained_save(&mut c, &format!("park-bank-passing-bay-{x}"));
        line(&mut c, (x, 19));
    }
}

#[test]
#[ignore = "requires the published park pack and bound CRYSTAL_PARK_POND_INPUT_PLAN"]
fn park_pond_bridge_sign_prints_both_lines_and_owns_movement() {
    let (root, r, mut c) = fixture((48, 18));
    course(&mut c, &r, "bridge-sign-from-east", false);
    blocked(&mut c, GameButton::Up, (36, 15));
    assert_eq!(
        c.snapshot().expect("sign facing").overworld.facing,
        Direction::Up
    );
    retained_save(&mut c, "park-bridge-sign-facing-up");
    let before = c.snapshot().expect("before actual sign");
    let before_flags = flags(&c);
    assert_eq!(collision(&r, &mut c, (36, 16)), "FLOOR");
    c.press(GameButton::A)
        .expect("read actual park bridge sign");
    c.wait_frames(240).expect("print park bridge sign");
    assert!(c.snapshot().expect("open sign").ui.text_window_open);
    c.press(GameButton::Down)
        .expect("movement attempt while sign owns input");
    c.wait_frames(16).expect("sign input ownership");
    assert_at(
        &mut c,
        PARK,
        (36, 15),
        "sign text prevents a legal downward step",
    );
    let text = printed_dialogue(&mut c);
    assert!(
        text.contains("PARK POND BRIDGE"),
        "actual sign text: {text}"
    );
    assert!(
        text.contains("Keep to the deck."),
        "actual sign text: {text}"
    );
    let after = c.snapshot().expect("sign finished");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before_flags, flags(&c));
    continue_game(&root, &r, &mut c, "park-sign-read");
}

#[test]
#[ignore = "requires the published park pack and bound CRYSTAL_PARK_POND_INPUT_PLAN"]
fn park_pond_closed_ends_reject_surf_without_false_prompts() {
    let (root, r, mut c) = fixture((6, 18));
    for (id, point) in [
        ("north-end-view-from-west", (23, 10)),
        ("south-end-lookout-from-west", (22, 24)),
    ] {
        course(&mut c, &r, id, false);
        assert_eq!(collision(&r, &mut c, (point.0 + 1, point.1)), "WALL");
        blocked(&mut c, GameButton::Right, point);
        let before = c.snapshot().expect("closed pond bank");
        let before_flags = flags(&c);
        let actions = c.shell.flygon_field_actions.clone();
        c.press(GameButton::A)
            .expect("native Surf interaction attempt against closed bank");
        for frames in [16, 240] {
            c.wait_frames(frames)
                .expect("native bank refusal presentation");
            let after = c.snapshot().expect("no false Surf prompt");
            assert!(!after.ui.text_window_open && after.ui.menu.is_none());
            assert!(after.ui.pending_text_wait.is_none());
            assert!(c.shell.field_notice.is_none() && !c.has_pending_script_work());
            assert_eq!(after.overworld.mode, MovementMode::Normal);
            assert_at(&mut c, PARK, point, "closed pond bank cannot launch Surf");
            assert_eq!(before.party, after.party);
            assert_eq!(before.bag, after.bag);
            assert_eq!(before_flags, flags(&c));
            assert_eq!(actions, c.shell.flygon_field_actions);
        }
        continue_game(&root, &r, &mut c, id);
        course(&mut c, &r, id, true);
    }
}

fn collected(c: &VisibleShellController) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(ITEM_FLAG)
        .expect("existing park item flag")
}

#[test]
#[ignore = "requires the published park pack and bound CRYSTAL_PARK_POND_INPUT_PLAN"]
fn park_pond_cache_paid_grass_pickup_is_once_only_and_continues() {
    let (root, r, mut c) = fixture((42, 27));
    assert!(!collected(&c));
    assert_eq!(course(&mut c, &r, "picnic-cache-paid-approach", false), 4);
    blocked(&mut c, GameButton::Right, (48, 27));
    retained_save(&mut c, "park-cache-approach");
    let before = c.snapshot().expect("uncollected native park cache");
    let before_flags = flags(&c);
    let mut expected_bag = c.shell.shell.session().state().bag.clone();
    assert!(
        expected_bag
            .add_item(&r.data().items["GREAT_BALL"], 1)
            .expect("expected one Great Ball")
    );
    c.press(GameButton::A)
        .expect("collect existing native park Great Ball");
    let text = printed_dialogue(&mut c);
    let item_name = r.data().items["GREAT_BALL"].name.replace('_', " ");
    assert!(
        text.to_uppercase().contains(&item_name.to_uppercase()),
        "actual cache text: {text}"
    );
    assert!(collected(&c));
    assert_eq!(c.shell.shell.session().state().bag, expected_bag);
    assert_eq!(
        before.party,
        c.snapshot().expect("cache party preserved").party
    );
    let after_flags = flags(&c);
    let mut expected_flags = before_flags;
    expected_flags.0.insert(ITEM_FLAG.into());
    assert_eq!(
        expected_flags, after_flags,
        "only the existing item flag changes"
    );
    continue_game(&root, &r, &mut c, "park-cache-collected");
    c.press(GameButton::A)
        .expect("repeat native cache interaction");
    settle(&mut c);
    assert_eq!(c.shell.shell.session().state().bag, expected_bag);
    assert_eq!(after_flags, flags(&c));
    walk(&mut c, (49, 27));
    walk(&mut c, (48, 27));
    assert_eq!(course(&mut c, &r, "picnic-cache-paid-approach", true), 4);
    // The old trainer's sight tile (42,25) is avoided through the clear row26.
    walk(&mut c, (42, 26));
    line(&mut c, (40, 26));
    line(&mut c, (40, 19));
    assert_at(
        &mut c,
        PARK,
        (40, 19),
        "cache returns to the ordinary bridge route",
    );
    continue_game(&root, &r, &mut c, "park-cache-return-ordinary-route");
    assert!(collected(&c));
    assert_eq!(c.shell.shell.session().state().bag, expected_bag);
}
