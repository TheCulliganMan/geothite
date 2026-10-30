//! One inspected town contract per process; every action after setup is native input.
//! Contracts and pack prerequisites are read at runtime, never embedded in builds.
use super::*;
use crystal_core::systems::script_flags::{BADGE_ENGINE_FLAGS, reconcile_saved_badge_flags};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

fn review() -> &'static (AssetRoot, CrystalRuntime, Value) {
    static REVIEW: OnceLock<(AssetRoot, CrystalRuntime, Value)> = OnceLock::new();
    REVIEW.get_or_init(|| {
        let path = std::env::var_os("CRYSTAL_TOWN_EXTERIOR_CONTRACT")
            .expect("set the inspected single-town contract path");
        let contract: Value = serde_json::from_str(
            &std::fs::read_to_string(path).expect("external single-town contract"),
        )
        .expect("town review JSON");
        for key in ["roads", "gates", "ledges", "doors", "fences"] {
            assert_eq!(
                contract[key]
                    .as_array()
                    .expect("explicit review scope")
                    .len() as u64,
                contract["counts"][key]
                    .as_u64()
                    .expect("inspected scope count")
            );
        }
        let path = repo()
            .join(contract["pack"].as_str().expect("frozen current pack"))
            .canonicalize()
            .expect("completed pack");
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(std::fs::read(&path).expect("frozen pack bytes"))
            ),
            contract["pack_sha256"]
                .as_str()
                .expect("inspected pack SHA256")
        );
        let root = AssetRoot::new(path.parent().expect("pack parent"));
        let runtime = CrystalRuntime::from_loaded_compiled_pack(
            &root,
            read_loaded_verified_compiled_game_pack(path).expect("verified current pack"),
        )
        .expect("native town runtime");
        assert!(
            runtime
                .data()
                .maps
                .contains_key(contract["map"].as_str().expect("town"))
        );
        for flag in contract["arrival_discovery_flags"]
            .as_array()
            .expect("inspected arrival callbacks")
        {
            let flag = flag.as_str().expect("native Fly flag");
            assert!(flag.starts_with("ENGINE_FLYPOINT_"));
            let destination = runtime
                .data()
                .fly_destinations
                .get(flag)
                .expect("native Fly destination");
            let spawn = &runtime.data().runtime_spawn_points
                [&destination.destination_spawn_identifier.to_string()];
            assert_eq!(spawn.map_name, contract["map"].as_str().unwrap());
        }
        (root, runtime, contract)
    })
}
fn point(v: &Value) -> Point {
    (
        i16::try_from(v[0].as_i64().expect("core x")).unwrap(),
        i16::try_from(v[1].as_i64().expect("core y")).unwrap(),
    )
}
fn button(v: &str) -> GameButton {
    match v {
        "up" => GameButton::Up,
        "down" => GameButton::Down,
        "left" => GameButton::Left,
        "right" => GameButton::Right,
        _ => panic!("cardinal contract input"),
    }
}
fn opposite(b: GameButton) -> GameButton {
    match b {
        GameButton::Up => GameButton::Down,
        GameButton::Down => GameButton::Up,
        GameButton::Left => GameButton::Right,
        GameButton::Right => GameButton::Left,
        _ => panic!("cardinal road"),
    }
}
fn start(
    map: &str,
    position: Point,
    qualified: bool,
    omitted: Option<&str>,
) -> VisibleShellController {
    let (root, runtime, plan) = review();
    let mut c = fixture(root, runtime, map, position);
    // Local QA partner/qualifications only. No later position, party, flag,
    // badge, menu, terrain, movement mode or clock is repaired.
    let mut partner = runtime
        .data()
        .create_pokemon("MEW", 60, Dv::default())
        .expect("declared legal local Surf fixture");
    partner.moves = vec![LearnedMove {
        name: "SURF".into(),
        current_pp: runtime.data().moves["SURF"].pp,
        pp_ups: 0,
    }];
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    let count = if qualified {
        plan["badge_count"].as_u64().expect("prior badge count")
    } else {
        0
    };
    assert!(count <= 8);
    state.badges.johto = std::array::from_fn(|i| (i as u64) < count);
    if let Some(flag) = omitted {
        if let Some(i) = BADGE_ENGINE_FLAGS.iter().position(|f| *f == flag) {
            assert!(
                state.badges.johto[i],
                "omitted badge must otherwise qualify"
            );
            state.badges.johto[i] = false;
        }
    }
    reconcile_saved_badge_flags(state);
    if qualified {
        for flag in plan["event_flags"]
            .as_array()
            .expect("declared prior events")
        {
            let flag = flag.as_str().expect("event flag");
            state
                .flags
                .set_event_flag(flag, Some(flag) != omitted)
                .expect("initial event fixture");
        }
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    c
}
fn expect_at(c: &mut VisibleShellController, name: &str, target: Point) {
    assert_eq!((map(c), at(c)), (name.to_owned(), target));
}
fn escape_if_wild(c: &mut VisibleShellController) {
    if let Some(battle) = c.snapshot().expect("actual lane battle").battle {
        assert!(
            matches!(battle.kind, crystal_runtime::RuntimeBattleKind::Wild { .. }),
            "a trainer/story battle requires its own authored review"
        );
        super::super::authored_route_fixture::escape_wild(c);
    }
}
fn travel(c: &mut VisibleShellController, direction: GameButton, name: &str, target: Point) {
    for _ in 0..32 {
        press(c, direction);
        escape_if_wild(c);
        if map(c) == name && at(c) == target {
            return;
        }
        assert!(
            !c.snapshot().unwrap().ui.text_window_open,
            "qualified road must be open"
        );
    }
    expect_at(c, name, target);
}

#[test]
#[ignore = "requires one inspected CRYSTAL_TOWN_EXTERIOR_CONTRACT and its exact completed pack"]
fn native_town_exterior_public_paths_qualified_roads_and_water_boundary() {
    let (_, runtime, plan) = review();
    let town = plan["map"].as_str().unwrap();
    let source: Value = serde_json::from_str(
        &std::fs::read_to_string(
            repo().join(
                plan["source_dossier"]
                    .as_str()
                    .expect("authored town dossier"),
            ),
        )
        .expect("current source"),
    )
    .unwrap();
    assert_eq!(source["id"], plan["map"]);
    for key in ["residents", "signs"] {
        assert_eq!(
            source[key].as_array().unwrap().len() as u64,
            plan["counts"][key]
                .as_u64()
                .expect("inspected public scope count")
        );
    }
    let mut c = start(town, point(&plan["start"]), true, None);
    for actor in source["residents"].as_array().unwrap() {
        let (x, y) = point(&actor["tile"]);
        let (approach, facing) = match actor["facing"].as_str().unwrap() {
            "north" => ((x, y - 1), GameButton::Down),
            "south" => ((x, y + 1), GameButton::Up),
            "west" => ((x - 1, y), GameButton::Right),
            "east" => ((x + 1, y), GameButton::Left),
            _ => panic!("cardinal actor"),
        };
        talk_from(&mut c, runtime, approach, facing);
        assert!(c.snapshot().unwrap().ui.text_window_open || c.has_pending_script_work());
        finish(&mut c);
    }
    for sign in source["signs"].as_array().unwrap() {
        let (x, y) = point(&sign["tile"]);
        talk_from(&mut c, runtime, (x, y + 1), GameButton::Up);
        assert!(c.snapshot().unwrap().ui.text_window_open || c.has_pending_script_work());
        finish(&mut c);
    }
    let flags = semantic_flags(&c);
    for road in plan["roads"].as_array().unwrap() {
        let inside = point(&road["inside"]);
        let outward = button(road["out"].as_str().unwrap());
        walk(&mut c, runtime, inside);
        travel(
            &mut c,
            outward,
            road["neighbor"].as_str().unwrap(),
            point(&road["there"]),
        );
        travel(&mut c, opposite(outward), town, inside);
    }
    assert_eq!(
        semantic_flags(&c),
        flags,
        "roads grant no new story qualification"
    );
    for fence in plan["fences"].as_array().unwrap() {
        let stand = point(&fence["stand"]);
        walk(&mut c, runtime, stand);
        for _ in 0..4 {
            press(&mut c, button(fence["blocked"].as_str().unwrap()));
        }
        expect_at(&mut c, town, stand);
        assert_eq!(
            c.snapshot().unwrap().overworld.mode,
            crystal_core::world::movement::MovementMode::Normal
        );
    }
    if let Some(surf) = plan.get("surf").filter(|v| !v.is_null()) {
        walk(&mut c, runtime, point(&surf["bank"]));
        press(&mut c, button(surf["out"].as_str().unwrap()));
        for _ in 0..96 {
            escape_if_wild(&mut c);
            let s = c.snapshot().unwrap();
            if s.overworld.mode == crystal_core::world::movement::MovementMode::Surf
                && at(&mut c) == point(&surf["water"])
                && !s.ui.text_window_open
                && !c.has_pending_script_work()
                && c.shell.field_notice.is_none()
            {
                break;
            }
            press(&mut c, GameButton::A);
        }
        expect_at(&mut c, town, point(&surf["water"]));
        assert_eq!(
            c.snapshot().unwrap().overworld.mode,
            crystal_core::world::movement::MovementMode::Surf
        );
        for step in surf["steps"].as_array().unwrap() {
            travel(
                &mut c,
                button(step["input"].as_str().unwrap()),
                town,
                point(&step["at"]),
            );
        }
        assert_eq!(
            c.snapshot().unwrap().overworld.mode,
            crystal_core::world::movement::MovementMode::Normal
        );
    }
    walk(&mut c, runtime, point(&plan["start"]));
    let _loaded = continue_save(&mut c, &review().0, runtime);
}

#[test]
#[ignore = "requires one inspected town contract; omitted prerequisites are initial fixtures"]
fn native_town_exterior_each_gate_requirement_denial_and_free_inbound() {
    let (root, runtime, plan) = review();
    let town = plan["map"].as_str().unwrap();
    for gate in plan["gates"].as_array().unwrap() {
        let inside = point(&gate["inside"]);
        let outward = button(gate["out"].as_str().unwrap());
        for required in gate["required_flags"].as_array().unwrap() {
            let mut c = start(town, inside, true, Some(required.as_str().unwrap()));
            let before = semantic_flags(&c);
            for _ in 0..8 {
                press(&mut c, outward);
                if c.snapshot().unwrap().ui.text_window_open {
                    break;
                }
            }
            let s = c.snapshot().unwrap();
            let label = &s.ui.text.as_ref().expect("actual guard refusal").label;
            assert!(
                label.starts_with(gate["script_root"].as_str().unwrap()),
                "{label}"
            );
            finish(&mut c);
            expect_at(&mut c, town, inside);
            assert_eq!(semantic_flags(&c), before);
        }
        let mut inbound = start(
            gate["neighbor"].as_str().unwrap(),
            point(&gate["there"]),
            false,
            None,
        );
        let before = semantic_flags(&inbound);
        travel(&mut inbound, opposite(outward), town, inside);
        let mut expected = before;
        for flag in plan["arrival_discovery_flags"].as_array().unwrap() {
            expected.1.insert(flag.as_str().unwrap().to_owned());
        }
        assert_eq!(
            semantic_flags(&inbound),
            expected,
            "inbound may register only the inspected native arrival discovery; no guard qualification is granted"
        );
        let _loaded = continue_save(&mut inbound, root, runtime);
    }
}

#[test]
#[ignore = "requires inspected native ledge coordinates and ordinary return waypoints"]
fn native_town_exterior_ledge_hops_reverse_denial_and_ordinary_returns() {
    let (root, runtime, plan) = review();
    let town = plan["map"].as_str().unwrap();
    for ledge in plan["ledges"].as_array().unwrap() {
        let mut c = start(town, point(&ledge["upper"]), false, None);
        let direction = button(ledge["out"].as_str().unwrap());
        let upper = point(&ledge["upper"]);
        let lip = point(&ledge["lip"]);
        let landing = point(&ledge["landing"]);
        let mut saw_lip = false;
        for _ in 0..8 {
            c.press(direction).expect("native ledge input");
            let p = at(&mut c);
            assert!(
                [upper, lip, landing].contains(&p),
                "unexpected native hop {p:?}"
            );
            saw_lip |= p == lip;
            if p == landing {
                break;
            }
        }
        assert!(saw_lip, "actual native ledge face must be traversed");
        expect_at(&mut c, town, landing);
        for _ in 0..4 {
            press(&mut c, opposite(direction));
            expect_at(&mut c, town, landing);
        }
        for via in ledge["return_via"].as_array().unwrap() {
            walk(&mut c, runtime, point(via));
        }
        walk(&mut c, runtime, upper);
        let _loaded = continue_save(&mut c, root, runtime);
    }
}

#[test]
#[ignore = "requires inspected current room arrivals and both native return carpets"]
fn native_town_exterior_all_facades_both_returns_and_continue() {
    let (root, runtime, plan) = review();
    let town = plan["map"].as_str().unwrap();
    for door in plan["doors"].as_array().unwrap() {
        let outside = point(&door["outside"]);
        let mut c = start(town, outside, false, None);
        for carpet in door["carpets"].as_array().unwrap() {
            let room = door["room"].as_str().unwrap();
            travel(&mut c, GameButton::Up, room, point(&door["arrival"]));
            c.wait_frames(120).expect("passive native arrival");
            expect_at(&mut c, room, point(&door["arrival"]));
            let carpet = point(carpet);
            walk(&mut c, runtime, (carpet.0, carpet.1 - 1));
            travel(&mut c, GameButton::Down, town, outside);
            c.wait_frames(120).expect("safe exterior return");
            expect_at(&mut c, town, outside);
        }
        let _loaded = continue_save(&mut c, root, runtime);
    }
}
