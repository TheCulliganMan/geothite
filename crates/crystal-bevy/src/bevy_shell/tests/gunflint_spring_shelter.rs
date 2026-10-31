//! Local setup only; rest, climbs, cache, exterior returns and Continue use input.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;
use serde_json::{Value, json};

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life;
use super::northstar_ice_lair::{choice, event, page, talk, walk};
use super::r10_gallery::press;
use super::winona::{at, finish, load, travel};

const ROOM: &str = "MnR12Cave1F";

fn output(name: &str) -> PathBuf {
    let p = PathBuf::from(
        std::env::var_os("CRYSTAL_GUNFLINT_SPRING_REVIEW_DIR").expect("fresh spring review output"),
    )
    .join(name);
    assert!(!p.exists(), "retain every spring review");
    std::fs::create_dir_all(&p).expect("spring output");
    p
}

fn fixture(
    root: &AssetRoot,
    r: &CrystalRuntime,
    map: &str,
    at: (i16, i16),
    injured: bool,
) -> VisibleShellController {
    let mut c = interior_life::fixture(root, r, map, at);
    let state = c.shell.shell.session_mut().state_mut();
    for (i, species) in ["CINDORE", "FORTIGARD"].into_iter().enumerate() {
        let mut p = r
            .data()
            .create_pokemon(species, 30, Default::default())
            .expect("original active regional partner");
        assert!(p.moves.iter().all(|m| m.name != "SURF"));
        if injured {
            p.hp = if i == 0 { 1 } else { 0 };
            p.status = if i == 0 { Some("POISON".into()) } else { None };
            for m in &mut p.moves {
                m.current_pp = 0;
            }
        }
        p.item = Some("SMOKE_BALL".into());
        assert!(state.storage.party.add_pokemon(p));
    }
    state.sync_party_from_storage();
    state.money = 0;
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    c
}

fn health(c: &VisibleShellController) -> Vec<Value> {
    c.shell.shell.session().state().storage.party.pokemon.iter().flatten()
        .map(|p|json!({"hp":p.hp,"status":p.status,"pp":p.moves.iter().map(|m|m.current_pp).collect::<Vec<_>>() })).collect()
}

fn fully_rested(c: &VisibleShellController, r: &CrystalRuntime) {
    for p in c
        .shell
        .shell
        .session()
        .state()
        .storage
        .party
        .pokemon
        .iter()
        .flatten()
    {
        assert_eq!(p.hp, p.max_hp, "rest must revive and restore every partner");
        assert!(p.status.is_none(), "native rest cures status");
        for m in &p.moves {
            assert_eq!(
                m.current_pp,
                r.data().moves[&m.name].pp,
                "native PP restoration"
            );
        }
    }
    assert_eq!(
        c.shell.shell.session().state().money,
        0,
        "camp rest stays free"
    );
}

fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    path: PathBuf,
) -> VisibleShellController {
    assert!(
        !c.has_pending_script_work(),
        "Continue only from released scenes"
    );
    assert!(!path.exists());
    c.save(&path).expect("actual reached checkpoint");
    let mut next = VisibleShellController::load_save(root.clone(), r.clone(), path, None)
        .expect("production Continue");
    next.set_runtime_journal_enabled(false);
    next
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK and fresh CRYSTAL_GUNFLINT_SPRING_REVIEW_DIR; explicit initial injured local party"]
fn gunflint_spring_controller_rest_refusal_acknowledgement_all_party_resources_and_continue() {
    let out = output("rest");
    let (root, r) = load();
    let mut c = fixture(&root, &r, ROOM, (6, 5), true);
    c.save(out.join("initial-injured-camp.crystalsave"))
        .expect("declared injured local party before input");
    let before = health(&c);
    assert!(!event(&c, "EVENT_MN_R12_CAVE_ETHER"));
    talk(&mut c, GameButton::Left);
    choice(&mut c, false);
    finish(&mut c);
    assert_eq!(
        health(&c),
        before,
        "declining must preserve HP, status and PP"
    );
    c = continued(&mut c, &root, &r, out.join("rest-declined.crystalsave"));
    assert_eq!(health(&c), before);
    talk(&mut c, GameButton::Left);
    choice(&mut c, true);
    page(&mut c, "MnGunflintRestAcceptedText");
    assert_eq!(health(&c), before, "unacknowledged rest must not heal");
    press(&mut c, GameButton::Down);
    at(&mut c, ROOM, (6, 5));
    c.save(out.join("rest-before-acknowledgement.crystalsave"))
        .expect("actual pending rest frame for rendering only");
    finish(&mut c);
    fully_rested(&c, &r);
    assert!(
        !event(&c, "EVENT_MN_R12_CAVE_ETHER"),
        "rest cannot grant the cache"
    );
    c = continued(&mut c, &root, &r, out.join("rest-acknowledged.crystalsave"));
    fully_rested(&c, &r);
    talk(&mut c, GameButton::Left);
    choice(&mut c, true);
    finish(&mut c);
    fully_rested(&c, &r);
    press(&mut c, GameButton::Start);
    assert!(c.snapshot().expect("production Start").ui.menu.is_some());
    press(&mut c, GameButton::B);
    c.save(out.join("repeat-rest-and-start.crystalsave"))
        .expect("repeat free service checkpoint");
}

#[test]
#[ignore = "requires current pack and fresh spring output; explicit healthy party outside the actual compact cave mouth"]
fn gunflint_spring_controller_both_dry_paths_native_stairs_cache_and_real_exterior_return() {
    let out = output("course");
    let (root, r) = load();
    let mut c = fixture(&root, &r, "MnR12C00R00", (40, 16), false);
    c.save(out.join("initial-real-exterior.crystalsave"))
        .expect("declared exterior start");
    travel(&mut c, GameButton::Up, ROOM, (17, 23));
    travel(&mut c, GameButton::Up, ROOM, (17, 22));
    for at in [
        (13, 18),
        (8, 17),
        (8, 12),
        (10, 10),
        (13, 10),
        (17, 10),
        (20, 14),
        (20, 18),
        (13, 18),
    ] {
        walk(&mut c, &r, at);
    }
    walk(&mut c, &r, (6, 10));
    for y in (5..=9).rev() {
        travel(&mut c, GameButton::Up, ROOM, (6, y));
    }
    for y in 6..=10 {
        travel(&mut c, GameButton::Down, ROOM, (6, y));
    }
    walk(&mut c, &r, (26, 10));
    for y in (5..=9).rev() {
        travel(&mut c, GameButton::Up, ROOM, (26, y));
    }
    let initial = c
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items["MAX_ETHER"]);
    assert!(!event(&c, "EVENT_MN_R12_CAVE_ETHER"));
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["MAX_ETHER"]),
        initial + 1
    );
    assert!(event(&c, "EVENT_MN_R12_CAVE_ETHER"));
    c = continued(
        &mut c,
        &root,
        &r,
        out.join("actual-cache-collected.crystalsave"),
    );
    assert!(event(&c, "EVENT_MN_R12_CAVE_ETHER"));
    for y in 6..=10 {
        travel(&mut c, GameButton::Down, ROOM, (26, y));
    }
    travel(&mut c, GameButton::Up, ROOM, (26, 9));
    for _ in 0..4 {
        press(&mut c, GameButton::Right);
        at(&mut c, ROOM, (26, 9));
    }
    for y in 10..=11 {
        travel(&mut c, GameButton::Down, ROOM, (26, y));
    }
    walk(&mut c, &r, (20, 16));
    c.save(out.join("actual-spring-bank.crystalsave"))
        .expect("reached spring bank");
    walk(&mut c, &r, (17, 22));
    for _ in 0..3 {
        press(&mut c, GameButton::Down);
        if c.snapshot().expect("cave exit").overworld.map_name != ROOM {
            break;
        }
    }
    // The production controller settles the native cave arrival onto the
    // existing compact contract's safe tile; the raw warp remains at40,15.
    at(&mut c, "MnR12C00R00", (40, 16));
    travel(&mut c, GameButton::Down, "MnR12C00R00", (40, 17));
    travel(&mut c, GameButton::Up, "MnR12C00R00", (40, 16));
    c = continued(
        &mut c,
        &root,
        &r,
        out.join("actual-safe-exterior-return.crystalsave"),
    );
    at(&mut c, "MnR12C00R00", (40, 16));
    travel(&mut c, GameButton::Up, ROOM, (17, 23));
    travel(&mut c, GameButton::Up, ROOM, (17, 22));
    walk(&mut c, &r, (26, 5));
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["MAX_ETHER"]),
        initial + 1,
        "cache remains once-only after exit and Continue"
    );
}
