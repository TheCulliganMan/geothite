//! Local initialized badges/position/party are explicit fixtures, not earned travel.
//! All later checkpoint, walking, trainer, menu and persistence actions use the
//! production controller. Closed preserve admission is never bypassed or claimed.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};
use serde_json::Value;

use super::leech_shore::play_with_move;
use super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};

type Point = (i16, i16);
fn plan() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(||{let path=std::env::var_os("CRYSTAL_STATE_COHESION_CONTRACT").map(PathBuf::from).unwrap_or_else(||PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/connections/state-cohesion-v01/source-v01/integration.contract.json"));serde_json::from_str(&std::fs::read_to_string(path).expect("same frozen contract as native publication")).expect("cohesion JSON")})
}
fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let p = std::env::var_os("CRYSTAL_STATE_COHESION_CONTROLLER_PACK")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../content-packs/minnesota-state-cohesion-review.crystalpack")
                })
                .canonicalize()
                .expect("actual completed681-map cohesion pack");
            let root = AssetRoot::new(p.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(&p).expect("strict actual cohesion pack"),
            )
            .expect("native cohesion runtime");
            assert_eq!(runtime.data().maps.len(), 681);
            (root, runtime)
        })
        .clone()
}
fn point(value: &Value) -> Point {
    (
        i16::try_from(value[0].as_i64().expect("core x")).expect("i16 x"),
        i16::try_from(value[1].as_i64().expect("core y")).expect("i16 y"),
    )
}
fn button(side: &str) -> GameButton {
    match side {
        "north" => GameButton::Up,
        "south" => GameButton::Down,
        "west" => GameButton::Left,
        "east" => GameButton::Right,
        _ => panic!("authored cardinal side"),
    }
}
fn opposite(b: GameButton) -> GameButton {
    match b {
        GameButton::Up => GameButton::Down,
        GameButton::Down => GameButton::Up,
        GameButton::Left => GameButton::Right,
        GameButton::Right => GameButton::Left,
        _ => panic!("cardinal input"),
    }
}
fn shifted((x, y): Point, b: GameButton) -> Point {
    match b {
        GameButton::Up => (x, y - 1),
        GameButton::Down => (x, y + 1),
        GameButton::Left => (x - 1, y),
        GameButton::Right => (x + 1, y),
        _ => panic!("cardinal input"),
    }
}
fn at(c: &mut VisibleShellController) -> (String, Point) {
    let s = c.snapshot().expect("authoritative position");
    (
        s.overworld.map_name,
        (s.overworld.tile.x, s.overworld.tile.y),
    )
}
fn press(c: &mut VisibleShellController, b: GameButton) {
    c.press(b).expect("production input");
    c.wait_frames(20).expect("native presentation");
    assert!(c.shell.last_error.is_none());
}
fn floor(r: &CrystalRuntime, map: &str, (x, y): Point) -> String {
    let m = &r.data().maps[map];
    assert!(
        x >= 0
            && y >= 0
            && x < i16::try_from(m.attributes.width * 2).expect("width")
            && y < i16::try_from(m.attributes.height * 2).expect("height")
    );
    let x = usize::try_from(x).expect("x");
    let y = usize::try_from(y).expect("y");
    let b = m.blocks[y / 2 * usize::from(m.attributes.width) + x / 2];
    r.data().tilesets[&m.attributes.tileset_name].collision[&format!("{b:02x}")][y % 2 * 2 + x % 2]
        .clone()
}
fn fixture(
    map: &str,
    position: Point,
    flags: &BTreeSet<&str>,
    signal: bool,
    populated: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    assert_eq!(floor(&r, map, position), "FLOOR");
    assert!(
        !r.data().maps[map].objects.iter().any(|o| (
            i16::try_from(o.x).expect("x"),
            i16::try_from(o.y).expect("y")
        ) == position),
        "initial tile unoccupied"
    );
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        r.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: r.title_new_game_spawn_identifier().expect("retained HOME"),
            map_name: map.into(),
            tile_x: position.0,
            tile_y: position.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local review fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture name");
    let state = c.shell.shell.session_mut().state_mut();
    state.badges.johto = std::array::from_fn(|i| flags.contains(BADGE_ENGINE_FLAGS[i]));
    for flag in &BADGE_ENGINE_FLAGS[..8] {
        state
            .flags
            .set_engine_flag(flag, flags.contains(flag))
            .expect("explicit consistent prior badge");
    }
    state
        .flags
        .set_event_flag("EVENT_MN_LIGHTHOUSE_SIGNAL", signal)
        .expect("explicit prior beacon fixture");
    state.storage.party.pokemon.fill(None);
    if populated {
        let mut p = r
            .data()
            .create_pokemon("MEW", 80, Dv::default())
            .expect("legal initial native partner");
        p.moves = vec![LearnedMove {
            name: "PSYCHIC_M".into(),
            current_pp: r.data().moves["PSYCHIC_M"].pp,
            pp_ups: 0,
        }];
        assert!(state.storage.party.add_pokemon(p));
    }
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    assert_eq!(at(&mut c), (map.to_owned(), position));
    (root, r, c)
}
fn semantic_flags(c: &VisibleShellController) -> (BTreeSet<String>, BTreeSet<String>) {
    let flags = &c.shell.shell.session().state().flags;
    (
        flags
            .event_flags
            .iter()
            .filter(|(_, v)| **v)
            .map(|(k, _)| k.clone())
            .collect(),
        flags
            .engine_flags
            .iter()
            .filter(|(_, v)| **v)
            .map(|(k, _)| k.clone())
            .collect(),
    )
}
fn travel(c: &mut VisibleShellController, b: GameButton, map: &str, target: Point) {
    for _ in 0..16 {
        if at(c) == (map.to_owned(), target) {
            return;
        }
        press(c, b);
    }
    assert_eq!(
        at(c),
        (map.to_owned(), target),
        "turn-first production travel"
    );
}
fn full_text(c: &mut VisibleShellController) -> String {
    let mut all = String::new();
    for _ in 0..64 {
        c.wait_frames(128).expect("fully printed native page");
        let s = c.snapshot().expect("native text");
        if s.phase == RuntimeShellPhase::Overworld
            && !s.ui.text_window_open
            && !c.has_pending_script_work()
        {
            return all;
        }
        assert!(
            s.ui.pending_yes_no.is_none(),
            "checkpoint has no hidden choice"
        );
        if let Some(text) = s.ui.text.and_then(|t| t.asm_text) {
            all.push_str(&text);
            all.push(' ');
        }
        press(c, GameButton::A);
    }
    panic!("native text did not release input")
}
fn continue_game(
    mut c: VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    label: &str,
) -> VisibleShellController {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "cohesion-{label}-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    let before = c.snapshot().expect("before Continue");
    let flags = semantic_flags(&c);
    let money = c.shell.shell.session().state().money;
    let badges = c.shell.shell.session().state().badges.clone();
    c.save(&path).expect("production save");
    let mut loaded = VisibleShellController::load_save(root.clone(), r.clone(), path.clone(), None)
        .expect("real Continue");
    loaded.set_runtime_journal_enabled(false);
    let after = loaded.snapshot().expect("continued snapshot");
    assert_eq!(after.overworld.map_name, before.overworld.map_name);
    assert_eq!(after.overworld.tile, before.overworld.tile);
    assert_eq!(after.party, before.party);
    assert_eq!(after.bag, before.bag);
    assert_eq!(loaded.shell.shell.session().state().money, money);
    assert_eq!(loaded.shell.shell.session().state().badges, badges);
    assert_eq!(
        semantic_flags(&loaded),
        flags,
        "complete true flag sets; absent and explicit-false storage are equivalent"
    );
    std::fs::remove_file(path).expect("remove only successful own save");
    loaded
}
fn link_for(g: &Value) -> (&'static str, Point, Point) {
    for link in plan()["new_reciprocal_links"].as_array().expect("links") {
        if link["from_map"] == g["map"] && link["direction"] == g["side"] {
            return (
                link["to_map"].as_str().expect("neighbor"),
                point(&link["from_core"][0]),
                point(&link["to_core"][0]),
            );
        }
        if link["to_map"] == g["map"]
            && opposite(button(link["direction"].as_str().expect("side")))
                == button(g["side"].as_str().expect("side"))
        {
            return (
                link["from_map"].as_str().expect("neighbor"),
                point(&link["to_core"][0]),
                point(&link["from_core"][0]),
            );
        }
    }
    panic!("new checkpoint must bind a declared physical seam")
}
fn denial(c: &mut VisibleShellController, g: &Value, lane: usize, evidence: bool) {
    let map = g["map"].as_str().expect("map");
    let out = button(g["side"].as_str().expect("side"));
    let inside = point(&g["inside"][lane]);
    let start = shifted(inside, opposite(out));
    let before = c.snapshot().expect("before denial");
    let flags = semantic_flags(c);
    let money = c.shell.shell.session().state().money;
    for _ in 0..8 {
        if c.snapshot().expect("checkpoint input").ui.text.is_some() {
            break;
        }
        press(c, out);
    }
    c.wait_frames(128).expect("fully printed refusal");
    assert_eq!(at(c), (map.to_owned(), inside));
    let s = c.snapshot().expect("refusal");
    assert_eq!(
        s.ui.text.as_ref().expect("native refusal").label,
        format!(
            "{}{}Text",
            g["id"].as_str().expect("root"),
            if evidence { "Evidence" } else { "Badges" }
        )
    );
    press(c, GameButton::Start);
    assert_eq!(at(c), (map.to_owned(), inside));
    assert!(c.snapshot().expect("text owns input").ui.menu.is_none());
    let all = full_text(c);
    let visible = all.split_whitespace().collect::<Vec<_>>().join(" ");
    for line in g["denial_text"].as_array().expect("lines") {
        assert!(
            visible.contains(line.as_str().expect("line")),
            "all refusal pages fully printed: {visible}"
        );
    }
    assert_eq!(
        at(c),
        (map.to_owned(), start),
        "native one-step inward refusal"
    );
    let after = c.snapshot().expect("after refusal");
    assert_eq!(after.party, before.party);
    assert_eq!(after.bag, before.bag);
    assert_eq!(c.shell.shell.session().state().money, money);
    assert_eq!(semantic_flags(c), flags);
}

#[test]
#[ignore = "requires actual published681-map state cohesion pack; explicit local empty-party fixtures"]
fn state_cohesion_controller_all_six_guards_deny_each_missing_requirement() {
    for g in plan()["guard_proposals"].as_array().expect("six guards") {
        let required = g["flags"]
            .as_array()
            .expect("badge prerequisites")
            .iter()
            .map(|v| v.as_str().expect("native flag"))
            .collect::<Vec<_>>();
        for missing in &required {
            let flags = required
                .iter()
                .copied()
                .filter(|f| f != missing)
                .collect::<BTreeSet<_>>();
            for lane in 0..2 {
                let out = button(g["side"].as_str().expect("side"));
                let start = shifted(point(&g["inside"][lane]), opposite(out));
                let (_root, _r, mut c) =
                    fixture(g["map"].as_str().expect("map"), start, &flags, true, false);
                denial(&mut c, g, lane, false);
            }
        }
        if g.get("event")
            .and_then(Value::as_str)
            .is_some_and(|e| !e.is_empty())
        {
            let flags = required.into_iter().collect::<BTreeSet<_>>();
            for lane in 0..2 {
                let start = shifted(
                    point(&g["inside"][lane]),
                    opposite(button(g["side"].as_str().expect("side"))),
                );
                let (_root, _r, mut c) =
                    fixture(g["map"].as_str().expect("map"), start, &flags, false, false);
                denial(&mut c, g, lane, true);
            }
        }
    }
}

#[test]
#[ignore = "requires actual published681-map state cohesion pack; explicit local badge fixtures"]
fn state_cohesion_controller_all_six_guards_qualified_exit_reverse_free_and_continue() {
    for g in plan()["guard_proposals"].as_array().expect("six guards") {
        let map = g["map"].as_str().expect("map");
        let out = button(g["side"].as_str().expect("side"));
        let flags = g["flags"]
            .as_array()
            .expect("required badges")
            .iter()
            .map(|v| v.as_str().expect("flag"))
            .collect::<BTreeSet<_>>();
        let (neighbor, _edge, neighbor0) = link_for(g);
        for lane in 0..2 {
            let neighbor_edge = if matches!(out, GameButton::Up | GameButton::Down) {
                (
                    neighbor0.0 + i16::try_from(lane).expect("lane"),
                    neighbor0.1,
                )
            } else {
                (
                    neighbor0.0,
                    neighbor0.1 + i16::try_from(lane).expect("lane"),
                )
            };
            let start = shifted(point(&g["inside"][lane]), opposite(out));
            let (root, r, mut c) = fixture(map, start, &flags, true, false);
            let before = semantic_flags(&c);
            travel(&mut c, out, neighbor, neighbor_edge);
            assert_eq!(semantic_flags(&c), before, "exit grants no story progress");
            if lane == 0 {
                c = continue_game(c, &root, &r, g["id"].as_str().expect("root"));
                assert_eq!(at(&mut c), (neighbor.to_owned(), neighbor_edge));
            }
            let (_root, _r, mut inbound) =
                fixture(neighbor, neighbor_edge, &BTreeSet::new(), false, false);
            let no_progress = semantic_flags(&inbound);
            travel(&mut inbound, opposite(out), map, point(&g["inside"][lane]));
            assert_eq!(
                semantic_flags(&inbound),
                no_progress,
                "unqualified inbound is free"
            );
            travel(&mut inbound, opposite(out), map, start);
            denial(&mut inbound, g, lane, false);
        }
    }
}

#[test]
#[ignore = "requires actual published681-map state cohesion pack; fixtures prove local physical seams only"]
fn state_cohesion_controller_all13_native_pairs_and_four_ordinary_woodland_courses() {
    let all = BADGE_ENGINE_FLAGS[..8]
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    for link in plan()["new_reciprocal_links"].as_array().expect("links") {
        for lane in 0..2 {
            for reverse in [false, true] {
                let (map, target, from, to, b) = if reverse {
                    (
                        link["to_map"].as_str().expect("map"),
                        link["from_map"].as_str().expect("target"),
                        point(&link["to_core"][lane]),
                        point(&link["from_core"][lane]),
                        opposite(button(link["direction"].as_str().expect("side"))),
                    )
                } else {
                    (
                        link["from_map"].as_str().expect("map"),
                        link["to_map"].as_str().expect("target"),
                        point(&link["from_core"][lane]),
                        point(&link["to_core"][lane]),
                        button(link["direction"].as_str().expect("side")),
                    )
                };
                let (_root, _r, mut c) = fixture(map, from, &all, true, false);
                travel(&mut c, b, target, to);
            }
        }
    }
    let path=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/connections/state-cohesion-v01/source-v01/controller-input-plan.json");
    let courses: Value = serde_json::from_str(
        &std::fs::read_to_string(path).expect("frozen ordinary input courses"),
    )
    .expect("course JSON");
    for course in courses["courses"].as_array().expect("four courses") {
        let map = course["map"].as_str().expect("map");
        let points = course["core_waypoints"]
            .as_array()
            .expect("explicit points")
            .iter()
            .map(point)
            .collect::<Vec<_>>();
        let (_root, _r, mut c) = fixture(map, points[0], &all, true, false);
        for path in [points.clone(), points.iter().rev().copied().collect()] {
            for pair in path.windows(2) {
                let b = match (pair[1].0 - pair[0].0, pair[1].1 - pair[0].1) {
                    (1, 0) => GameButton::Right,
                    (-1, 0) => GameButton::Left,
                    (0, 1) => GameButton::Down,
                    (0, -1) => GameButton::Up,
                    _ => panic!("ordinary adjacent course"),
                };
                travel(&mut c, b, map, pair[1]);
                assert!(
                    c.snapshot()
                        .expect("empty party native eligibility")
                        .battle
                        .is_none()
                );
            }
        }
    }
}

#[test]
#[ignore = "requires actual published681-map state cohesion pack; prior legal partner/badges are fixture setup"]
fn state_cohesion_controller_four_optional_trainers_real_pp_money_once_and_continue() {
    let flags = BADGE_ENGINE_FLAGS[..8]
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    for source in plan()["new_maps"].as_array().expect("maps") {
        let dossier = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(source["dossier_file"].as_str().expect("dossier path"));
        let dossier: Value =
            serde_json::from_str(&std::fs::read_to_string(dossier).expect("source dossier"))
                .expect("dossier JSON");
        let map = source["id"].as_str().expect("map");
        for actor in dossier["actors"].as_array().expect("optional trainers") {
            let target = point(&actor["tile"]);
            let (_root, r) = load();
            let approach = [
                GameButton::Up,
                GameButton::Down,
                GameButton::Left,
                GameButton::Right,
            ]
            .into_iter()
            .map(|b| (shifted(target, opposite(b)), b))
            .find(|(p, _)| {
                floor(&r, map, *p) == "FLOOR"
                    && !r.data().maps[map].objects.iter().any(|o| {
                        (
                            i16::try_from(o.x).expect("x"),
                            i16::try_from(o.y).expect("y"),
                        ) == *p
                    })
            })
            .expect("legal adjacent trainer approach");
            let (root, r, mut c) = fixture(map, approach.0, &flags, true, true);
            let before = c.snapshot().expect("before optional battle");
            let money = c.shell.shell.session().state().money;
            let flag = actor["event_flag"].as_str().expect("defeat flag");
            assert!(
                !c.shell
                    .shell
                    .session()
                    .state()
                    .flags
                    .is_event_flag_set(flag)
                    .expect("initial flag")
            );
            press(&mut c, approach.1);
            assert_eq!(
                at(&mut c),
                (map.to_owned(), approach.0),
                "native actor collision"
            );
            press(&mut c, GameButton::A);
            play_with_move(&mut c, "PSYCHIC_M");
            full_text(&mut c);
            let after = c.snapshot().expect("actual battle result");
            assert!(after.battle.is_none());
            assert!(
                after.party.slots[0].pokemon.moves[0].current_pp
                    < before.party.slots[0].pokemon.moves[0].current_pp
            );
            assert!(c.shell.shell.session().state().money > money);
            assert!(
                c.shell
                    .shell
                    .session()
                    .state()
                    .flags
                    .is_event_flag_set(flag)
                    .expect("won flag")
            );
            c = continue_game(c, &root, &r, actor["id"].as_str().expect("trainer id"));
            let paid = c.shell.shell.session().state().money;
            let pp = c.snapshot().expect("continued PP").party.slots[0]
                .pokemon
                .moves[0]
                .current_pp;
            press(&mut c, approach.1);
            press(&mut c, GameButton::A);
            assert!(full_text(&mut c).contains("Safe travels!"));
            assert!(c.snapshot().expect("once").battle.is_none());
            assert_eq!(c.shell.shell.session().state().money, paid);
            assert_eq!(
                c.snapshot().expect("no repeated battle PP").party.slots[0]
                    .pokemon
                    .moves[0]
                    .current_pp,
                pp
            );
        }
    }
}

#[test]
#[ignore = "requires actual c481 cohesion pack; explicit local empty-party edge fixture"]
fn state_cohesion_controller_r03_trim_preserves_original_super_potion_and_blocks_east_edge() {
    const MAP: &str = "MnR03C00R01";
    const SCRIPT: &str = "MN_R03_ITEM_01";
    const EVENT: &str = "EVENT_MN_R03_ITEM_01";
    const ITEM: &str = "SUPER_POTION";
    // The actual exterior reward is Super Potion. Great Ball is the separate
    // cave ITEM_02; never change native content to match an old planning label.
    let (_root, r, mut c) = fixture(MAP, (40, 7), &BTreeSet::new(), false, false);
    let map = &r.data().maps[MAP];
    assert_eq!([map.attributes.width, map.attributes.height], [21, 8]);
    assert!(
        map.events.warps.is_empty(),
        "no edge warp may hide a boundary error"
    );
    assert_eq!(floor(&r, MAP, (40, 7)), "FLOOR");
    assert_eq!(floor(&r, MAP, (41, 7)), "FLOOR");
    let object = map
        .objects
        .iter()
        .find(|o| [o.x, o.y] == [41, 7])
        .expect("original boundary item object");
    assert_eq!(object.script, SCRIPT);
    assert_eq!(object.event_flag, EVENT);
    let pickup = map
        .script_field_pickups
        .iter()
        .find(|p| p.source_script == SCRIPT)
        .expect("original typed exterior pickup");
    assert_eq!(pickup.item_id.as_deref(), Some(ITEM));
    assert_eq!(pickup.quantity, 1);
    assert_eq!(pickup.event_flag.as_deref(), Some(EVENT));
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set(EVENT)
            .expect("original item flag")
    );
    let quantity = c
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items[ITEM]);
    let before_flags = semantic_flags(&c);
    let before = c.snapshot().expect("before native reward");
    press(&mut c, GameButton::Right);
    assert_eq!(
        at(&mut c),
        (MAP.into(), (40, 7)),
        "original item still owns its tile"
    );
    press(&mut c, GameButton::A);
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set(EVENT)
            .expect("collected native flag")
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items[ITEM]),
        quantity + 1
    );
    let mut pages = std::collections::BTreeMap::new();
    for _ in 0..32 {
        c.wait_frames(240).expect("fully print native reward page");
        if let Some((index, text)) = super::super::r10_gallery::completed_visible_page(&mut c) {
            pages.insert(index, text);
        }
        if c.shell.field_notice.is_none()
            && c.shell.visible_field_item_notice.is_none()
            && !c.has_pending_script_work()
        {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(
        c.shell.field_notice.is_none()
            && c.shell.visible_field_item_notice.is_none()
            && !c.has_pending_script_work(),
        "reward must release production input"
    );
    let name = r.data().items[ITEM].name.replace('_', " ");
    assert_eq!(r.data().items[ITEM].pocket, "ITEM");
    assert_eq!(pages.get(&0), Some(&format!("CHRIS put the\n{name} in")));
    assert_eq!(pages.get(&1), Some(&format!("{name} in\nthe ITEM POCKET.")));
    let mut expected_flags = before_flags;
    expected_flags.0.insert(EVENT.into());
    assert_eq!(
        semantic_flags(&c),
        expected_flags,
        "only the original item flag changes"
    );
    assert_eq!(c.snapshot().expect("retained party").party, before.party);
    travel(&mut c, GameButton::Right, MAP, (41, 7));
    for _ in 0..3 {
        press(&mut c, GameButton::Right);
        assert_eq!(
            at(&mut c),
            (MAP.into(), (41, 7)),
            "trimmed east edge rejects off-map travel"
        );
        let s = c.snapshot().expect("ordinary boundary rejection");
        assert_eq!(s.phase, RuntimeShellPhase::Overworld);
        assert!(s.ui.text.is_none() && s.ui.menu.is_none() && s.battle.is_none());
    }
    if let Some(folder) = std::env::var_os("CRYSTAL_STATE_COHESION_CAPTURE_SAVES_DIR") {
        let folder = PathBuf::from(folder);
        std::fs::create_dir_all(&folder).expect("optional private capture-save folder");
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = folder.join(format!(
            "MnR03C00R01-41-7-Right-{}-{nonce}.crystalsave",
            std::process::id()
        ));
        assert!(!path.exists(), "retain every earlier capture save");
        c.save(&path)
            .expect("production edge-camera save, no later state edits");
    }
    travel(&mut c, GameButton::Left, MAP, (40, 7));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items[ITEM]),
        quantity + 1
    );
    assert_eq!(semantic_flags(&c), expected_flags);
}
