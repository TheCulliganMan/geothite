//! Local placement/party are fixtures; conversations, errands and care use input.
#[path = "glacier_line_controller.rs"]
mod glacier_line;
#[path = "interior_regional_flavor.rs"]
mod regional_flavor;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

use crystal_assets::{AssetRoot, MapModule, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};

type Point = (i16, i16);

pub(super) fn load() -> (AssetRoot, CrystalRuntime, Value) {
    let path =
        PathBuf::from(std::env::var_os("CRYSTAL_INTERIOR_PACK").expect("published interiors"));
    let root = AssetRoot::new(path.parent().expect("pack directory"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified interior pack"),
    )
    .expect("runtime");
    let source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/interior-life/stories.json"),
    )
    .expect("room contract");
    (
        root,
        runtime,
        serde_json::from_str(&source).expect("authored rooms"),
    )
}

pub(super) fn fixture(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    map: &str,
    at: Point,
) -> VisibleShellController {
    assert_eq!(
        collision(runtime, &runtime.data().maps[map], at),
        Some("FLOOR"),
        "explicit interior fixture must start on native floor in {map}"
    );
    let mut c = VisibleShellController {
        shell: initialize_bevy_runtime_shell(
            root.clone(),
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
                map_name: map.into(),
                tile_x: at.0,
                tile_y: at.1,
            },
            BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()),
                ..Default::default()
            },
        )
        .expect("explicit room fixture"),
    };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    c.set_runtime_journal_enabled(false);
    c
}

fn press(c: &mut VisibleShellController, button: GameButton) {
    c.press(button).expect("production interior input");
    c.wait_frames(16).expect("settle visible input");
}

fn at(c: &mut VisibleShellController) -> Point {
    let s = c.snapshot().expect("room position");
    (s.overworld.tile.x, s.overworld.tile.y)
}

fn face(from: Point, to: Point) -> GameButton {
    match (to.0 - from.0, to.1 - from.1) {
        (1, 0) => GameButton::Right,
        (-1, 0) => GameButton::Left,
        (0, 1) => GameButton::Down,
        (0, -1) => GameButton::Up,
        other => panic!("nonadjacent conversation {other:?}"),
    }
}

pub(super) fn talk(c: &mut VisibleShellController, actor: Point) -> String {
    let start = at(c);
    press(c, face(start, actor));
    assert_eq!(at(c), start, "resident body blocks the facing input");
    press(c, GameButton::A);
    assert!(
        c.snapshot()
            .expect("opened conversation")
            .ui
            .text_window_open
    );
    press(c, GameButton::Right);
    assert_eq!(at(c), start, "dialogue owns movement");
    let mut pages = String::new();
    for _ in 0..200 {
        let s = c.snapshot().expect("visible dialogue page");
        if let Some(t) = s.ui.text {
            pages.push_str(&t.label);
            pages.push_str(&t.asm_text.unwrap_or_default());
            pages.push('\n');
        }
        if !s.ui.text_window_open
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
            && c.shell.pc_notice.is_none()
            && c.shell.special_boundary.is_none()
            && c.shell.battle_messages.is_empty()
        {
            c.wait_frames(16).expect("release closing A");
            return pages;
        }
        press(c, GameButton::A);
    }
    panic!("conversation did not release input: {pages}");
}

pub(super) fn collision<'a>(r: &'a CrystalRuntime, m: &MapModule, p: Point) -> Option<&'a str> {
    let (x, y) = (u16::try_from(p.0).ok()?, u16::try_from(p.1).ok()?);
    if x >= m.attributes.width * 2 || y >= m.attributes.height * 2 {
        return None;
    }
    let b = m.blocks[usize::from(y / 2) * usize::from(m.attributes.width) + usize::from(x / 2)];
    Some(
        r.data().tilesets[&m.attributes.tileset_name].collision[&format!("{b:02x}")]
            [usize::from(y % 2 * 2 + x % 2)]
        .as_str(),
    )
}

pub(super) fn walk(c: &mut VisibleShellController, r: &CrystalRuntime, target: Point) {
    let snapshot = c.snapshot().expect("walk origin");
    let name = snapshot.overworld.map_name;
    let m = &r.data().maps[&name];
    let start = at(c);
    let mut prev = BTreeMap::new();
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(p) = queue.pop_front() {
        if p == target {
            break;
        }
        for q in [
            (p.0 - 1, p.1),
            (p.0 + 1, p.1),
            (p.0, p.1 - 1),
            (p.0, p.1 + 1),
        ] {
            if seen.contains(&q) || m.objects.iter().any(|o| (o.x as i16, o.y as i16) == q) {
                continue;
            }
            let Some(k) = collision(r, m, q) else {
                continue;
            };
            if !matches!(
                k,
                "FLOOR"
                    | "DOOR"
                    | "LADDER"
                    | "STAIRCASE"
                    | "WARP_CARPET_DOWN"
                    | "BRIDGE_NS_DECK"
                    | "BRIDGE_EW_DECK"
            ) {
                continue;
            }
            if q != target && matches!(k, "DOOR" | "LADDER" | "STAIRCASE" | "WARP_CARPET_DOWN") {
                continue;
            }
            seen.insert(q);
            prev.insert(q, p);
            queue.push_back(q);
        }
    }
    assert!(
        seen.contains(&target),
        "{name}: no path {start:?} to {target:?}"
    );
    let mut route = vec![];
    let mut cursor = target;
    while cursor != start {
        let previous = prev[&cursor];
        route.push((previous, cursor));
        cursor = previous;
    }
    let last_button = route.first().map(|(from, to)| face(*from, *to));
    for (from, to) in route.into_iter().rev() {
        for _ in 0..8 {
            press(c, face(from, to));
            if c.snapshot().expect("walking map").overworld.map_name != name {
                return;
            }
            if at(c) == to {
                break;
            }
        }
        assert_eq!(at(c), to, "{name} native step");
    }
    // A native exit carpet owns the following outward step, not merely the
    // approach onto its painted tile. Keep that final step on the same input path.
    let outward = match collision(r, m, target) {
        Some("WARP_CARPET_DOWN") => Some(GameButton::Down),
        Some("DOOR") => Some(GameButton::Up),
        Some("LADDER" | "STAIRCASE") => last_button,
        _ => None,
    };
    if let Some(button) = outward {
        for _ in 0..4 {
            press(c, button);
            if c.snapshot().expect("outward warp").overworld.map_name != name {
                return;
            }
        }
    }
}

pub(super) fn target_name(r: &CrystalRuntime, name: &str) -> String {
    if r.data().maps.contains_key(name) {
        return name.into();
    }
    r.data()
        .runtime_map_metadata
        .get(name)
        .expect("native warp destination")
        .name
        .clone()
}

pub(super) fn travel(c: &mut VisibleShellController, r: &CrystalRuntime, target: &str) {
    let start = c.snapshot().expect("journey origin").overworld.map_name;
    let mut prev = BTreeMap::new();
    let mut seen = BTreeSet::from([start.clone()]);
    let mut queue = VecDeque::from([start.clone()]);
    while let Some(name) = queue.pop_front() {
        if name == target {
            break;
        }
        let m = &r.data().maps[&name];
        for w in &m.events.warps {
            let destination = target_name(r, &w.target_map);
            if seen.contains(&destination)
                || !matches!(
                    collision(r, m, (w.x as i16, w.y as i16)),
                    Some("DOOR" | "LADDER" | "STAIRCASE" | "WARP_CARPET_DOWN")
                )
            {
                continue;
            }
            seen.insert(destination.clone());
            prev.insert(
                destination.clone(),
                (name.clone(), (w.x as i16, w.y as i16)),
            );
            queue.push_back(destination);
        }
    }
    assert!(
        seen.contains(target),
        "no building journey {start} to {target}"
    );
    let mut route = vec![];
    let mut cursor = target.to_owned();
    while cursor != start {
        let (name, tile) = &prev[&cursor];
        route.push((cursor.clone(), *tile));
        cursor = name.clone();
    }
    for (name, tile) in route.into_iter().rev() {
        walk(c, r, tile);
        assert_eq!(
            c.snapshot()
                .expect("reciprocal room warp")
                .overworld
                .map_name,
            name
        );
    }
}

pub(super) fn resident(r: &CrystalRuntime, name: &str, script: &str) -> Point {
    let o = r.data().maps[name]
        .objects
        .iter()
        .find(|o| o.script == script)
        .expect("authored resident");
    (o.x as i16, o.y as i16)
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK"]
fn interior_life_all_new_residents_talk_without_blocking_room_returns() {
    let (root, r, plan) = load();
    let relocated: Value = std::env::var_os("CRYSTAL_INTERIOR_ACTOR_RELOCATIONS")
        .map(|path| {
            serde_json::from_str(
                &std::fs::read_to_string(path).expect("explicit resident relocation source"),
            )
            .expect("resident relocation contract")
        })
        .unwrap_or(Value::Null);
    let mut count = 0;
    for room in plan["rooms"].as_array().expect("rooms") {
        let name = room["map"].as_str().expect("room name");
        for conversation in room["conversations"].as_array().into_iter().flatten() {
            let Some(a) = conversation.get("actor") else {
                continue;
            };
            let mut pos = (
                a["approach"][0].as_i64().expect("x") as i16,
                a["approach"][1].as_i64().expect("y") as i16,
            );
            if let Some(move_) = relocated["actor_relocations"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|move_| move_["map"] == name && move_["script"] == conversation["script"])
            {
                assert_eq!(
                    move_["before"], a["tile"],
                    "preserved original actor source"
                );
                pos = (
                    move_["approach"][0].as_i64().expect("relocated approach x") as i16,
                    move_["approach"][1].as_i64().expect("relocated approach y") as i16,
                );
                let actual = resident(&r, name, conversation["script"].as_str().expect("resident"));
                assert_eq!(
                    (
                        move_["tile"][0].as_i64().expect("resident x") as i16,
                        move_["tile"][1].as_i64().expect("resident y") as i16
                    ),
                    actual,
                    "actual native resident relocation"
                );
            }
            let mut c = fixture(&root, &r, name, pos);
            let pages = talk(
                &mut c,
                resident(&r, name, conversation["script"].as_str().expect("script")),
            );
            assert!(!pages.is_empty(), "visible local story {name}");
            let exterior = r.data().maps[name]
                .events
                .warps
                .iter()
                .find(|w| {
                    w.target_map != name
                        && matches!(
                            collision(&r, &r.data().maps[name], (w.x as i16, w.y as i16)),
                            Some("DOOR" | "LADDER" | "STAIRCASE" | "WARP_CARPET_DOWN")
                        )
                })
                .expect("room exit");
            walk(&mut c, &r, (exterior.x as i16, exterior.y as i16));
            assert_eq!(
                c.snapshot().expect("room return").overworld.map_name,
                target_name(&r, &exterior.target_map)
            );
            count += 1;
        }
    }
    assert_eq!(count, 117, "every explicitly placed new resident");
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK"]
fn interior_life_full_gift_pocket_preserves_unclaimed_reward_after_continue() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, "MnDuluthHillGardenHouse", (8, 4));
    // Native inventory supports several stacks of the same item. One 99-stack
    // is not a full pocket; fill every physical slot in this explicit fixture.
    while c.shell.shell.session().state().bag.items.len()
        < crystal_core::models::bag::ITEM_POCKET_CAPACITY
    {
        c.shell
            .shell
            .add_bag_item("BERRY", 99)
            .expect("explicit full gift pocket");
    }
    let initial = c
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items["BERRY"]);
    mark_runtime_snapshot_dirty(&mut c.shell);
    let pages = talk(&mut c, (8, 5));
    assert!(
        pages.to_lowercase().contains("full"),
        "visible bag-full feedback: {pages}"
    );
    let item = &r.data().items["BERRY"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(item), initial);
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_INTERIOR_WINDOWSILL_BERRY")
            .expect("gift completion")
    );
    let mut c = super::r10_gallery::continue_save(root.clone(), r.clone(), &mut c);
    talk(&mut c, (8, 5));
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_INTERIOR_WINDOWSILL_BERRY")
            .expect("gift still waits")
    );
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK"]
fn interior_life_weather_and_labor_errands_walk_reward_save_and_deny_repeats() {
    let (root, r, _) = load();
    for (reward_room, script, witnesses, item) in [
        (
            "MnDuluthWeatherStudy2F",
            "MN_DULUTH_X_WEATHERSTUDY_F2_HOST",
            [
                ("MnDuluthLookoutLodge2F", "MN_DULUTH_X_LOOKOUTLODGE_F2_HOST"),
                ("MnDuluthReadingHall2F", "MN_DULUTH_X_READINGHALL_F2_HOST"),
            ],
            "MINT_BERRY",
        ),
        (
            "MnDuluthUnionHall2F",
            "MN_DULUTH_X_UNIONHALL_F2_HOST",
            [
                ("MnDuluthUnionHall1F", "MN_DULUTH_X_UNIONHALL_F1_HOST"),
                ("MnDuluthOreWarehouse2F", "MN_DULUTH_X_OREWAREHOUSE_F2_HOST"),
            ],
            "ETHER",
        ),
    ] {
        let mut c = fixture(&root, &r, reward_room, (5, 6));
        let actor = resident(&r, reward_room, script);
        let initial = c
            .shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items[item]);
        talk(&mut c, actor);
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items[item]),
            initial,
            "no early award"
        );
        for (room, witness) in witnesses {
            travel(&mut c, &r, room);
            let npc = resident(&r, room, witness);
            walk(&mut c, &r, (npc.0, npc.1 + 1));
            talk(&mut c, npc);
        }
        travel(&mut c, &r, reward_room);
        walk(&mut c, &r, (5, 6));
        talk(&mut c, actor);
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items[item]),
            initial + 1,
            "real native item reward"
        );
        let mut c = super::r10_gallery::continue_save(root.clone(), r.clone(), &mut c);
        talk(&mut c, actor);
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items[item]),
            initial + 1,
            "no repeat reward after Continue"
        );
    }
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK"]
fn interior_life_dorothy_restores_hp_pp_and_status_for_free() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, "MnBoundaryRootBeerCabin", (5, 4));
    let mut pokemon = r
        .data()
        .create_pokemon("CHIKORITA", 20, Dv::default())
        .expect("legal partner");
    pokemon.hp = 1;
    pokemon.status = Some("POISON".into());
    for m in &mut pokemon.moves {
        m.current_pp = 0;
    }
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(pokemon));
    state.sync_party_from_storage();
    state.money = 0;
    mark_runtime_snapshot_dirty(&mut c.shell);
    talk(&mut c, (5, 3));
    let s = c.snapshot().expect("rested partners");
    let p = &s.party.slots[0].pokemon;
    assert_eq!(p.hp, p.max_hp);
    assert!(p.status.is_none());
    assert!(p.moves.iter().all(|m| m.current_pp > 0));
    assert_eq!(c.shell.shell.session().state().money, 0);
    talk(&mut c, (5, 3)); // A second visit stays free and does not open a fee/menu.
    walk(&mut c, &r, (5, 7));
    assert_eq!(
        c.snapshot().expect("island return").overworld.map_name,
        "MnBoundarySafariC01R01"
    );
}
