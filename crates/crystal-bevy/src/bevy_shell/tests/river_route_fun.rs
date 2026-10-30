//! Explicit local party/badge fixtures, then native input for every bank activity.
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::world::movement::MovementMode;
use serde_json::Value;

use super::super::rochester_earned::try_battle;
use super::grand_marais_harbor::use_max_repel;
use super::{assert_at, at, fixture_with_items, mark_runtime_snapshot_dirty, mount, settle, step};

fn plan() -> Value {
    let path = std::env::var_os("CRYSTAL_RIVER_FUN_PROOF")
        .map(PathBuf::from)
        .expect("written river proof required");
    let proof: Value = serde_json::from_slice(&std::fs::read(path).expect("static bank proof"))
        .expect("bank cases");
    let pack = std::env::var_os("CRYSTAL_MINNEAPOLIS_FALLS_CONTROLLER_PACK")
        .expect("published river pack");
    use sha2::{Digest, Sha256};
    assert_eq!(
        proof["pack_sha256"].as_str().expect("written pack hash"),
        format!(
            "{:x}",
            Sha256::digest(std::fs::read(pack).expect("published pack bytes"))
        )
    );
    proof
}

fn button(from: (i16, i16), to: (i16, i16)) -> GameButton {
    match (to.0 - from.0, to.1 - from.1) {
        (1, 0) => GameButton::Right,
        (-1, 0) => GameButton::Left,
        (0, 1) => GameButton::Down,
        (0, -1) => GameButton::Up,
        other => panic!("nonadjacent bank course {other:?}"),
    }
}
fn point(v: &Value) -> (i16, i16) {
    (
        v[0].as_i64().expect("x") as i16,
        v[1].as_i64().expect("y") as i16,
    )
}
fn global(v: &Value) -> (i16, i16) {
    let p = point(&v["point"]);
    let name = v["map"].as_str().expect("map");
    let col = name[name.len() - 5..name.len() - 3]
        .parse::<i16>()
        .expect("column");
    let row = name[name.len() - 2..].parse::<i16>().expect("row");
    (p.0 + col * 64, p.1 + row * 48)
}

fn run_reach(reach: &str) {
    let plan = plan();
    for case in plan["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .filter(|c| c["map"].as_str().expect("map").contains(reach))
    {
        let id = case["id"].as_str().expect("activity");
        let path = case["path"].as_array().expect("bank Foot course");
        let first = &path[0];
        let map = first["map"].as_str().expect("landing map");
        let landing = point(&first["point"]);
        let (root, r, mut c) =
            fixture_with_items(map, landing, false, false, false, &[("MAX_REPEL", 3)]);
        // Current regional wilds scale to the whole party. These two legal
        // initial partners keep a genuinely used Repel below the strong lead,
        // including the injured medic fixture; no field RNG or HP is rewritten.
        if std::env::var_os("CRYSTAL_RIVER_FUN_THREE_MEMBER_FIXTURE").is_some() {
            for species in ["VOYAGEAR", "DOZEBEAR"] {
                let partner = r
                    .data()
                    .create_pokemon(species, 40, crystal_core::models::Dv::default())
                    .expect("initial regional partner fixture");
                let state = c.shell.shell.session_mut().state_mut();
                assert!(state.storage.party.add_pokemon(partner));
                state.sync_party_from_storage();
            }
            mark_runtime_snapshot_dirty(&mut c.shell);
        }
        let healer = case["kind"] == "healer";
        if healer {
            // Explicit injured, low-PP initial fixture; setup ends before any input.
            let state = c.shell.shell.session_mut().state_mut();
            let p = state.storage.party.pokemon[0]
                .as_mut()
                .expect("fixture partner");
            p.hp = 1;
            p.moves[0].current_pp = 1;
            state.sync_party_from_storage();
            mark_runtime_snapshot_dirty(&mut c.shell);
        }
        use_max_repel(&mut c, &r);
        let water = &case["water"];
        let water_map = water["map"].as_str().expect("launch map");
        let water_point = point(&water["point"]);
        mount(
            &mut c,
            button(global(first), global(water)),
            water_map,
            water_point,
        );
        step(
            &mut c,
            button(global(water), global(first)),
            map,
            landing,
            "native grassy bank dismount",
        );
        assert_eq!(
            c.snapshot().expect("bank mode").overworld.mode,
            MovementMode::Normal
        );
        for pair in path.windows(2) {
            let target = &pair[1];
            step(
                &mut c,
                button(global(&pair[0]), global(target)),
                target["map"].as_str().expect("bank map"),
                point(&target["point"]),
                id,
            );
        }
        let approach = at(&mut c).1;
        let target = point(&case["tile"]);
        c.press(button(approach, target)).expect("face activity");
        c.wait_frames(16).expect("face presentation");
        assert_at(
            &mut c,
            case["map"].as_str().expect("activity map"),
            approach,
            "body blocks forward step",
        );
        let directory = std::env::var_os("CRYSTAL_RIVER_FUN_SAVES")
            .map(PathBuf::from)
            .expect("retained saves directory");
        std::fs::create_dir_all(&directory).expect("save directory");
        let save = directory.join(format!("{id}-approach.crystalsave"));
        assert!(!save.exists(), "retain previous save epochs");
        c.save(&save).expect("native reached approach save");
        let before = c.snapshot().expect("before river activity");
        let money =
            c.shell.shell.session().state().money + c.shell.shell.session().state().moms_money;
        c.press(GameButton::A).expect("interact with bank activity");
        if case["kind"] == "trainer" {
            let flag = case["defeated_flag"].as_str().expect("trainer flag");
            assert!(
                try_battle(&mut c, &r, id, flag),
                "native bank trainer victory: {id}"
            );
            settle(&mut c);
            let after = c.snapshot().expect("victory");
            assert!(
                after.party.slots[0].pokemon.moves[0].current_pp
                    < before.party.slots[0].pokemon.moves[0].current_pp,
                "selected Surf PP changed"
            );
            assert!(
                after.party.slots[0].pokemon.experience > before.party.slots[0].pokemon.experience
            );
            assert!(
                c.shell.shell.session().state().money + c.shell.shell.session().state().moms_money
                    > money
            );
        } else {
            settle(&mut c);
            if healer {
                let p = &c.snapshot().expect("native field care").party.slots[0].pokemon;
                assert_eq!(p.hp, p.max_hp);
                assert_eq!(p.moves[0].current_pp, r.data().moves["SURF"].pp);
            } else {
                let item = &r.data().items[case["item"].as_str().expect("supply")];
                assert_eq!(c.shell.shell.session().state().bag.quantity(item), 1);
            }
        }
        let won = c.snapshot().expect("completed bank activity");
        let money = c.shell.shell.session().state().money;
        let mom = c.shell.shell.session().state().moms_money;
        let save = directory.join(format!("{id}-completed.crystalsave"));
        c.save(&save).expect("native activity save");
        c = super::VisibleShellController::load_save(root.clone(), r.clone(), save, None)
            .expect("native Continue");
        assert_eq!(c.snapshot().expect("continued bank").party, won.party);
        c.press(GameButton::A).expect("repeat native interaction");
        settle(&mut c);
        let again = c.snapshot().expect("once-only rewards");
        assert!(again.battle.is_none());
        assert_eq!(again.bag, won.bag);
        assert_eq!(c.shell.shell.session().state().money, money);
        assert_eq!(c.shell.shell.session().state().moms_money, mom);
        eprintln!("river activity passed: {id}");
    }
}

#[test]
#[ignore = "requires published river fun pack and written reachability proof"]
fn river_fun_lower_twenty_trainers_grassy_banks_rewards_and_care() {
    run_reach("LowerWoods");
}
#[test]
#[ignore = "requires published river fun pack and written reachability proof"]
fn river_fun_upper_twenty_trainers_grassy_banks_rewards_and_care() {
    run_reach("UpstreamWoods");
}

#[test]
#[ignore = "requires published river fun pack and written reachability proof"]
fn river_fun_new_grass_runs_native_wild_encounters() {
    let plan = plan();
    let case = plan["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|c| c["kind"] == "item")
        .expect("grass reward course");
    let path = case["path"].as_array().expect("Foot path");
    let first = &path[0];
    let (root, r, mut c) = fixture_with_items(
        first["map"].as_str().expect("map"),
        point(&first["point"]),
        false,
        false,
        false,
        &[],
    );
    let mut previous = first;
    let mut grass_step = None;
    for target in path.iter().skip(1) {
        for _ in 0..8 {
            if at(&mut c)
                == (
                    target["map"].as_str().expect("map").to_owned(),
                    point(&target["point"]),
                )
            {
                break;
            }
            c.press(button(global(previous), global(target)))
                .expect("native grass walk");
            c.wait_frames(16).expect("grass step presentation");
            if let Some(battle) = c.snapshot().expect("grass encounter").battle {
                assert!(matches!(
                    battle.kind,
                    crystal_runtime::RuntimeBattleKind::Wild { .. }
                ));
                finish_wild_course(root, r, &mut c, path);
                return;
            }
        }
        if super::collision(&r, &mut c, point(&target["point"])) == "TALL_GRASS" {
            grass_step = Some((previous, target));
        }
        previous = target;
    }
    let (a, b) = grass_step.expect("supply approach crosses grass");
    // Return along the authored Foot course to this adjacent pair, then pace it.
    for target in path.iter().rev().skip(1) {
        if at(&mut c)
            == (
                b["map"].as_str().expect("map").to_owned(),
                point(&b["point"]),
            )
        {
            break;
        }
        step(
            &mut c,
            button(global(previous), global(target)),
            target["map"].as_str().expect("map"),
            point(&target["point"]),
            "grass return",
        );
        previous = target;
    }
    for i in 0..256 {
        let (from, to) = if i % 2 == 0 { (b, a) } else { (a, b) };
        for _ in 0..8 {
            if at(&mut c)
                == (
                    to["map"].as_str().expect("map").to_owned(),
                    point(&to["point"]),
                )
            {
                break;
            }
            c.press(button(global(from), global(to)))
                .expect("native habitat input");
            c.wait_frames(16).expect("native wild encounter clock");
            if let Some(battle) = c.snapshot().expect("native habitat encounter").battle {
                assert!(matches!(
                    battle.kind,
                    crystal_runtime::RuntimeBattleKind::Wild { .. }
                ));
                finish_wild_course(root, r, &mut c, path);
                return;
            }
        }
    }
    panic!("new tall grass did not produce a native wild encounter");
}

// Finish the actual wild encounter and walk the reached path back to its real
// dry landing before Save/Continue. Historical callers may use a fresh temp
// save; current reviews name their retained production-client checkpoint.
fn finish_wild_course(
    root: super::AssetRoot,
    r: super::CrystalRuntime,
    c: &mut super::VisibleShellController,
    course: &[Value],
) {
    super::super::authored_route_fixture::escape_wild(c);
    settle(c);
    let reached = at(c);
    let index = course
        .iter()
        .position(|v| {
            (
                v["map"].as_str().expect("course map").to_owned(),
                point(&v["point"]),
            ) == reached
        })
        .expect("actual reached grass step on native course");
    for i in (1..=index).rev() {
        step(
            c,
            button(global(&course[i]), global(&course[i - 1])),
            course[i - 1]["map"].as_str().expect("return map"),
            point(&course[i - 1]["point"]),
            "ordinary wild return to dry landing",
        );
    }
    assert_eq!(super::collision(&r, c, point(&course[0]["point"])), "FLOOR");
    let before = c.snapshot().expect("wild resolved at actual landing");
    let path = std::env::var_os("CRYSTAL_RIVER_FUN_REACHED_SAVE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::temp_dir().join(format!(
                "river-grass-wild-{}-{}.crystalsave",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("clock")
                    .as_nanos()
            ))
        });
    assert!(!path.exists(), "retain previous wild checkpoints");
    c.save(&path).expect("native reached landing save");
    let mut loaded = super::VisibleShellController::load_save(root, r, path, None)
        .expect("native wild Continue");
    let after = loaded.snapshot().expect("continued landing");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.mode, after.overworld.mode);
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
}
