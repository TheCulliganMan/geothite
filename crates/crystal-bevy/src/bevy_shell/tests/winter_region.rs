//! Local legal Mew80/Surf/badge/RNG fixtures, then production input only.
//! These cases certify the optional winter branch, not earned regional access.
use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::super::super::{VisibleFieldItemPhase, mark_runtime_snapshot_dirty};
use super::super::r10_gallery::{completed_visible_page, continue_save, flags, remember_dialogue};
use super::super::rochester_earned::try_battle;
use super::grand_marais_harbor::use_max_repel;
use super::{
    Point, VisibleShellController, assert_at, at, collision, fixture_with_items_on_runtime, settle,
    step,
};

const FIELD: &str = "MnRainyRiverIcefields";
const HUT: &str = "MnRainyRiverWarmingHut";
const R20: &str = "MnR20C00R00";
const CACHE: &str = "EVENT_MN_ICE_NEVERMELTICE";

fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_WINTER_REGION_CONTROLLER_PACK")
                    .expect("actual written winter-region pack"),
            )
            .canonicalize()
            .expect("external winter pack path");
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let r = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("verified winter pack"),
            )
            .expect("native winter runtime");
            let field = &r.data().maps[FIELD];
            assert_eq!([field.attributes.width, field.attributes.height], [32, 24]);
            let blocks = field.blocks.iter().map(|b| *b as u8).collect::<Vec<_>>();
            let expected = if r.data().maps[FIELD]
                .blocks
                .iter()
                .filter(|&&b| b == 0x3d)
                .count()
                == 14
            {
                "5794d029f45ab670c32d7ea008dfbd485b8d1b9e22cf83e33481e0bba08944a0"
            } else {
                "718fcae45a4ade9bef073c5577c6e40ba1aa28a4ba6e1441b3abac55fc1ec057"
            };
            assert_eq!(format!("{:x}", Sha256::digest(blocks)), expected);
            (root, r)
        })
        .clone()
}

fn fixture(
    map: &str,
    point: Point,
    injured: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, mut c) =
        fixture_with_items_on_runtime(load(), map, point, false, false, false, &[("MAX_REPEL", 2)]);
    // Extend only the explicit initial fixture. No gameplay input has occurred.
    if injured {
        let state = c.shell.shell.session_mut().state_mut();
        let p = state.storage.party.pokemon[0]
            .as_mut()
            .expect("fixture Mew");
        p.hp = p.max_hp - 30;
        p.status = Some("PARALYSIS".into());
        p.moves[0].current_pp = 1;
        state.sync_party_from_storage();
        mark_runtime_snapshot_dirty(&mut c.shell);
    }
    // All subsequent changes, including Repel consumption, are native inputs.
    (root, r, c)
}

fn capture_save(c: &mut VisibleShellController, label: &str) {
    if let Some(directory) = std::env::var_os("CRYSTAL_WINTER_REGION_SAVE_DIR") {
        let directory = PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("ignored native winter capture saves");
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("capture clock")
            .as_nanos();
        let path = directory.join(format!(
            "winter-{label}-{}-{nonce}.crystalsave",
            std::process::id()
        ));
        assert!(!path.exists(), "preserve every earlier native capture save");
        c.save(path)
            .expect("production save at actual reached winter stop");
    }
}

fn plan() -> Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../planning/minnesota/regions/ice/rainy-river-controller-plan.json"
    )))
    .expect("explicit frozen actor/sight-aware winter input courses")
}

fn point(value: &Value) -> Point {
    (
        i16::try_from(value[0].as_i64().expect("core x")).expect("core x range"),
        i16::try_from(value[1].as_i64().expect("core y")).expect("core y range"),
    )
}

fn button(label: &str) -> GameButton {
    match label {
        "up" => GameButton::Up,
        "right" => GameButton::Right,
        "down" => GameButton::Down,
        "left" => GameButton::Left,
        other => panic!("unknown explicit source input {other}"),
    }
}

fn opposite(button: GameButton) -> GameButton {
    match button {
        GameButton::Up => GameButton::Down,
        GameButton::Right => GameButton::Left,
        GameButton::Down => GameButton::Up,
        GameButton::Left => GameButton::Right,
        other => panic!("not a cardinal winter input {other:?}"),
    }
}

fn event(c: &VisibleShellController, name: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("native winter flag")
}

fn quantity(c: &VisibleShellController, r: &CrystalRuntime, item: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items[item])
}

fn keep_repel(c: &mut VisibleShellController, r: &CrystalRuntime) {
    if c.shell.shell.session().state().repel_steps_remaining == 0 {
        use_max_repel(c, r);
    }
}

fn walk_course(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    key: &str,
    reverse: bool,
) -> usize {
    let plan = plan();
    let edges = plan["courses"][key]
        .as_array()
        .expect("named source course");
    let edges = if reverse {
        edges.iter().rev().collect::<Vec<_>>()
    } else {
        edges.iter().collect()
    };
    let before_flags = flags(c);
    let mut grass = 0;
    for (index, edge) in edges.into_iter().enumerate() {
        assert_eq!(
            edge["traversed"]
                .as_array()
                .expect("ordinary traversal")
                .len(),
            1
        );
        let (from, target, input) = if reverse {
            (
                point(&edge["to_core"]),
                point(&edge["from_core"]),
                opposite(button(edge["button"].as_str().expect("button"))),
            )
        } else {
            (
                point(&edge["from_core"]),
                point(&edge["to_core"]),
                button(edge["button"].as_str().expect("button")),
            )
        };
        assert_at(c, FIELD, from, key);
        let kind = collision(r, c, target);
        assert!(
            matches!(kind.as_str(), "FLOOR" | "TALL_GRASS"),
            "{key} step{index} {target:?}: {kind}"
        );
        grass += usize::from(kind == "TALL_GRASS");
        keep_repel(c, r);
        step(c, input, FIELD, target, &format!("{key} step{index}"));
        assert!(c.snapshot().expect("ordinary snow course").battle.is_none());
        assert_eq!(
            flags(c),
            before_flags,
            "ordinary course changes no story flags"
        );
    }
    grass
}

fn slide(c: &mut VisibleShellController, r: &CrystalRuntime, key: &str) {
    let plan = plan();
    let edge = &plan["courses"][key][0];
    let from = point(&edge["from_core"]);
    let stop = point(&edge["to_core"]);
    let forward = button(edge["button"].as_str().expect("slide input"));
    let course = edge["traversed"]
        .as_array()
        .expect("all traversed ice/stop tiles");
    assert_at(c, FIELD, from, key);
    for p in &course[..course.len() - 1] {
        assert_eq!(
            collision(r, c, point(p)),
            "ICE",
            "{key}: native ICE, not snow"
        );
    }
    let before_flags = flags(c);
    let mut observed = Vec::new();
    for _ in 0..8 {
        c.press(forward).expect("actual entry into ice");
        c.wait_frames(1).expect("observe a live native ice frame");
        let actual = at(c);
        observed.push(actual.clone());
        if actual.1 != from {
            break;
        }
    }
    let entered = at(c).1;
    assert_ne!(entered, from, "{key}: {observed:?}");
    assert!(
        course.iter().any(|p| point(p) == entered),
        "{key}: {observed:?}"
    );
    if entered != stop {
        assert_eq!(collision(r, c, entered), "ICE");
        c.press(opposite(forward))
            .expect("opposing input during real forced ice travel");
    } else {
        assert!(
            course.len() <= 3,
            "a long slide must expose an intermediate frame: {observed:?}"
        );
    }
    for _ in 0..128 {
        if at(c).1 == stop {
            break;
        }
        c.wait_frames(8)
            .expect("empty frames continue native ice direction");
        let actual = at(c);
        observed.push(actual.clone());
        assert_eq!(actual.0, FIELD);
        assert!(
            course.iter().any(|p| point(p) == actual.1),
            "{key}: {observed:?}"
        );
    }
    assert_at(c, FIELD, stop, &format!("{key}: {observed:?}"));
    c.wait_frames(80)
        .expect("native stop stays stationary without input");
    assert_at(c, FIELD, stop, key);
    assert_eq!(flags(c), before_flags);
}

fn shelter_heal(c: &mut VisibleShellController, r: &CrystalRuntime) {
    assert_at(c, HUT, (5, 6), "real passive arrival/service approach");
    let before = c.snapshot().expect("injured team before shelter").party;
    let mut expected = before.clone();
    for slot in &mut expected.slots {
        let p = &mut slot.pokemon;
        assert!(p.hp < p.max_hp && p.status.is_some());
        assert!(p.moves.iter().all(|m| m.current_pp == 1));
        p.hp = p.max_hp;
        p.status = None;
        for learned in &mut p.moves {
            learned.current_pp = r.data().moves[&learned.name].pp;
        }
    }
    c.press(GameButton::Up).expect("face actual shelter keeper");
    c.press(GameButton::A)
        .expect("start native HealParty program");
    let mut printed = BTreeMap::new();
    let mut history = VecDeque::new();
    let mut healed = false;
    for _ in 0..64 {
        c.wait_frames(240).expect("print actual keeper page");
        remember_dialogue(c, &mut history);
        let s = c.snapshot().expect("native HealParty continuation");
        if s.party == expected && printed.len() == 2 {
            healed = true;
            break;
        }
        assert_eq!(
            s.party, before,
            "healing waits for actual text acknowledgement: {history:#?}"
        );
        if let Some((index, page)) = completed_visible_page(c) {
            printed.insert(index, page);
            let position = at(c);
            c.press(GameButton::Down)
                .expect("keeper dialogue owns movement");
            assert_eq!(at(c), position);
        }
        c.press(GameButton::A)
            .expect("acknowledge completed keeper page");
    }
    assert_eq!(
        printed.get(&0).map(String::as_str),
        Some("Shelter's open.\nWarm up your team."),
        "{history:#?}"
    );
    assert_eq!(
        printed.get(&1).map(String::as_str),
        Some("No border report\nis handled here."),
        "{history:#?}"
    );
    assert!(
        healed,
        "actual native HealParty must restore HP/status/PP: {history:#?}"
    );
    settle(c);
    assert_eq!(c.snapshot().expect("healed partner").party, expected);
}

fn collect_cache(c: &mut VisibleShellController, r: &CrystalRuntime) {
    assert_at(c, FIELD, (20, 11), "actual cache approach");
    assert_eq!(quantity(c, r, "NEVERMELTICE"), 0);
    c.press(GameButton::Right)
        .expect("face occupied NeverMeltIce tile");
    assert_at(c, FIELD, (20, 11), "item still occupies tile21,11");
    c.press(GameButton::A).expect("native one-time item ball");
    assert!(event(c, CACHE));
    assert_eq!(quantity(c, r, "NEVERMELTICE"), 1);
    let name = r.data().items["NEVERMELTICE"].name.replace('_', " ");
    let notice = c
        .shell
        .visible_field_item_notice
        .as_ref()
        .expect("retained native item notice");
    assert_eq!(notice.sound_trigger_text, format!("CHRIS found\n{name}!"));
    let expected_pocket = notice.pocket_text.clone();
    let mut printed = BTreeMap::new();
    let mut history = VecDeque::new();
    for _ in 0..1280 {
        remember_dialogue(c, &mut history);
        if let Some((index, page)) = completed_visible_page(c) {
            assert_eq!(
                c.shell
                    .visible_field_item_notice
                    .as_ref()
                    .expect("item printer owns page")
                    .phase,
                VisibleFieldItemPhase::PocketText
            );
            printed.insert(index, page);
            let position = at(c);
            c.press(GameButton::Down)
                .expect("fully printed pocket page owns movement");
            assert_eq!(at(c), position);
            c.press(GameButton::A)
                .expect("acknowledge fully printed pocket page");
        }
        if c.shell.visible_field_item_notice.is_none()
            && c.shell.field_notice.is_none()
            && !c.has_pending_script_work()
        {
            break;
        }
        c.wait_frames(1).expect("native found/fanfare/pocket clock");
    }
    assert_eq!(
        printed.len(),
        2,
        "both actual PocketText pages: {history:#?}"
    );
    assert_eq!(printed[&0], format!("CHRIS put the\n{name} in"));
    assert_eq!(printed[&1], format!("{name} in\nthe ITEM POCKET."));
    assert!(expected_pocket.contains(&name));
    settle(c);
    assert!(c.shell.visible_field_item_notice.is_none());
}

#[test]
#[ignore = "requires actual written winter-region pack; local legal Mew80/Surf fixture only"]
fn winter_region_controller_both_seam_lanes_long_slides_grass_shelter_and_continue() {
    let (root, r, mut c) = fixture(R20, (2, 44), true);
    let initial_flags = flags(&c);
    use_max_repel(&mut c, &r);
    for (input, map, target) in [
        (GameButton::Left, R20, (1, 44)),
        (GameButton::Left, R20, (0, 44)),
        (GameButton::Left, FIELD, (63, 44)),
        (GameButton::Down, FIELD, (63, 45)),
        (GameButton::Right, R20, (0, 45)),
        (GameButton::Right, R20, (1, 45)),
        (GameButton::Left, R20, (0, 45)),
        (GameButton::Left, FIELD, (63, 45)),
        (GameButton::Up, FIELD, (63, 44)),
        (GameButton::Right, R20, (0, 44)),
        (GameButton::Left, FIELD, (63, 44)),
        (GameButton::Left, FIELD, (62, 44)),
        (GameButton::Left, FIELD, (61, 44)),
        (GameButton::Left, FIELD, (60, 44)),
    ] {
        step(&mut c, input, map, target, "actual reciprocal R20 seam");
    }
    walk_course(&mut c, &r, "entry_to_east_stop", false);
    slide(&mut c, &r, "lower_west");
    capture_save(&mut c, "lower-seventeen-stop");
    slide(&mut c, &r, "lower_east");
    slide(&mut c, &r, "lower_west");
    assert_eq!(walk_course(&mut c, &r, "west_stop_to_hut", false), 6);
    step(
        &mut c,
        GameButton::Up,
        HUT,
        (5, 6),
        "true facade door to passive Floor arrival",
    );
    shelter_heal(&mut c, &r);
    capture_save(&mut c, "hut-after-heal");
    assert_eq!(
        flags(&c),
        initial_flags,
        "shelter awards no border/quest flags"
    );
    c = continue_save(root, r.clone(), &mut c);
    // Native directional carpets do not warp on the landing step. Continue
    // preserves the passive arrival; walk onto the carpet before exiting it.
    assert_at(&mut c, HUT, (5, 6), "Continue inside the real shelter");
    assert_eq!(collision(&r, &mut c, (5, 7)), "WARP_CARPET_DOWN");
    step(
        &mut c,
        GameButton::Down,
        HUT,
        (5, 7),
        "native carpet landing precedes its directional warp",
    );
    step(
        &mut c,
        GameButton::Down,
        FIELD,
        (9, 28),
        "native carpet returns to exterior passive2",
    );
    c.wait_frames(80)
        .expect("safe passive exterior return does not rewarp");
    assert_at(&mut c, FIELD, (9, 28), "stable hut return");
    assert_eq!(walk_course(&mut c, &r, "west_stop_to_hut", true), 6);
    assert_eq!(flags(&c), initial_flags);
    assert_eq!(quantity(&c, &r, "MAX_REPEL"), 1);
    assert!(c.shell.shell.session().state().repel_steps_remaining > 0);
}

#[test]
#[ignore = "requires actual written winter-region pack and source ICE/cache courses"]
fn winter_region_controller_cache_requires_ice_once_wall_stop_turn_and_continue() {
    let (root, r, mut c) = fixture(FIELD, (19, 7), false);
    use_max_repel(&mut c, &r);
    let before_flags = flags(&c);
    slide(&mut c, &r, "cache_down");
    walk_course(&mut c, &r, "island_to_cache", false);
    collect_cache(&mut c, &r);
    capture_save(&mut c, "cache-after-pocket-pages");
    step(
        &mut c,
        GameButton::Right,
        FIELD,
        (21, 11),
        "collected cache no longer occupies tile",
    );
    step(
        &mut c,
        GameButton::Left,
        FIELD,
        (20, 11),
        "return to cache approach",
    );
    walk_course(&mut c, &r, "cache_to_island_wall_launch", false);
    slide(&mut c, &r, "island_wall_stop");
    assert_eq!(collision(&r, &mut c, (12, 10)), "ICE");
    capture_save(&mut c, "stationary-ice-wall-stop");
    c = continue_save(root, r.clone(), &mut c);
    assert_at(
        &mut c,
        FIELD,
        (12, 10),
        "Continue preserves real stationary ice stop",
    );
    slide(&mut c, &r, "island_wall_return");
    walk_course(&mut c, &r, "island_to_exit", false);
    slide(&mut c, &r, "cache_up");
    slide(&mut c, &r, "cache_down");
    walk_course(&mut c, &r, "island_to_cache", false);
    c.press(GameButton::Right)
        .expect("approach already collected cache");
    settle(&mut c);
    c.press(GameButton::A).expect("no second cache award");
    settle(&mut c);
    assert_eq!(quantity(&c, &r, "NEVERMELTICE"), 1);
    let mut expected_flags = before_flags;
    expected_flags.0.insert(CACHE.into());
    assert_eq!(flags(&c), expected_flags);
}

fn trainer(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    id: &str,
    flag: &str,
    face: GameButton,
) {
    assert!(!event(c, flag));
    let money = c.shell.shell.session().state().money;
    let pp = c.snapshot().expect("actual initial battle PP").party.slots[0]
        .pokemon
        .moves[0]
        .current_pp;
    c.press(face).expect("face real optional native trainer");
    c.press(GameButton::A)
        .expect("native optional trainer interaction");
    assert!(
        try_battle(c, r, id, flag),
        "actual trainer victory required"
    );
    assert!(event(c, flag));
    assert!(c.shell.shell.session().state().money > money);
    assert!(
        c.snapshot().expect("real post-battle PP").party.slots[0]
            .pokemon
            .moves[0]
            .current_pp
            < pp
    );
    let after = c.snapshot().expect("one-time trainer outcome");
    let money = c.shell.shell.session().state().money;
    c.press(GameButton::A)
        .expect("already defeated trainer talks without rebattle");
    settle(c);
    assert_eq!(
        c.snapshot().expect("no second trainer reward").party,
        after.party
    );
    assert_eq!(c.shell.shell.session().state().money, money);
}

#[test]
#[ignore = "requires actual written winter-region pack; actual legal native winter trainers"]
fn winter_region_controller_long_snow_return_two_native_battles_pp_money_once_and_continue() {
    let (root, r, mut c) = fixture(FIELD, (37, 36), false);
    use_max_repel(&mut c, &r);
    assert_eq!(
        plan()["courses"]["ordinary_snow_return"]
            .as_array()
            .expect("return")
            .len(),
        139
    );
    walk_course(&mut c, &r, "ordinary_snow_return", false);
    walk_course(&mut c, &r, "east_stop_to_skier", false);
    trainer(
        &mut c,
        &r,
        "MN_ICE_SKIER_ELSA",
        "EVENT_MN_ICE_SKIER_ELSA",
        GameButton::Right,
    );
    walk_course(&mut c, &r, "skier_to_boarder", false);
    trainer(
        &mut c,
        &r,
        "MN_ICE_BOARDER_NILS",
        "EVENT_MN_ICE_BOARDER_NILS",
        GameButton::Down,
    );
    capture_save(&mut c, "after-both-native-trainers");
    c = continue_save(root, r.clone(), &mut c);
    assert!(event(&c, "EVENT_MN_ICE_SKIER_ELSA"));
    assert!(event(&c, "EVENT_MN_ICE_BOARDER_NILS"));
    assert!(!event(&c, CACHE));
}

#[path = "rainy_icefields_grass.rs"]
mod rainy_icefields_grass;
