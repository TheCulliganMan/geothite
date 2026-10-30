//! Prior cargo access is explicit; every new floor/action uses native input.
#[path = "northstar_ice_headquarters_current.rs"]
mod current;
#[path = "northstar_ice_headquarters_ports.rs"]
mod ports;
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::RuntimeBattleKind;
use serde_json::Value;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::northstar_ice_lair::{choice, event, page, path, save, talk};
use super::winona::{at, finish, load, press, travel};

const KEY: &str = "EVENT_MN_ICE_HQ_CARD_RECEIVED";
const POWER: &str = "EVENT_MN_ICE_HOLDING_POWER_OFF";
const RESCUED: &str = "EVENT_MN_ICE_CAPTIVE_RESCUED";

fn source() -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../planning/minnesota/lairs/ice-headquarters/network.json"),
        )
        .expect("authored floor graph"),
    )
    .expect("network JSON")
}

fn point(value: &Value) -> (i16, i16) {
    (
        value[0].as_i64().expect("x") as i16,
        value[1].as_i64().expect("y") as i16,
    )
}

fn encounter(c: &mut VisibleShellController, r: &crystal_runtime::CrystalRuntime) {
    for _ in 0..64 {
        let snapshot = c.snapshot().expect("native sight encounter");
        if let Some(battle) = snapshot.battle {
            let RuntimeBattleKind::Trainer { trainer_id, .. } = battle.kind else {
                panic!("unexpected wild battle")
            };
            assert!(trainer_id.starts_with("MN_ICE_HQ_"));
            assert!(super::rochester_earned::try_battle(
                c,
                r,
                &trainer_id,
                &format!("EVENT_{trainer_id}_DEFEATED")
            ));
            finish(c);
            return;
        }
        if !snapshot.ui.text_window_open && !c.has_pending_script_work() {
            return;
        }
        press(c, GameButton::A);
    }
    panic!("sight trainer did not release input");
}

fn walk(c: &mut VisibleShellController, r: &crystal_runtime::CrystalRuntime, to: (i16, i16)) {
    let name = c.snapshot().expect("walking start").overworld.map_name;
    for _ in 0..512 {
        let before = c.snapshot().expect("live tile").overworld.tile;
        if (before.x, before.y) == to {
            at(c, &name, to);
            return;
        }
        let button = *path(c, r, to)
            .unwrap_or_else(|| panic!("{name}: no live route to {to:?}"))
            .first()
            .expect("remaining step");
        for _ in 0..8 {
            press(c, button);
            encounter(c, r);
            if c.snapshot().expect("actual step").overworld.tile != before {
                break;
            }
        }
        assert_ne!(
            c.snapshot().expect("walked tile").overworld.tile,
            before,
            "blocked native step"
        );
        assert_eq!(
            c.snapshot().expect("same floor").overworld.map_name,
            name,
            "unplanned pad crossing"
        );
    }
    panic!("bounded route did not finish");
}

fn cross(
    c: &mut VisibleShellController,
    r: &crystal_runtime::CrystalRuntime,
    n: &Value,
    id: &str,
    reverse: bool,
) {
    let link = n["links"]
        .as_array()
        .expect("links")
        .iter()
        .find(|v| v["id"] == id)
        .expect("paired link");
    let (from, to) = if reverse {
        (&link["b"], &link["a"])
    } else {
        (&link["a"], &link["b"])
    };
    assert_eq!(
        c.snapshot().expect("port floor").overworld.map_name,
        from["map"].as_str().expect("map")
    );
    let (x, y) = point(&from["active"]);
    walk(c, r, (x, y + 1));
    travel(
        c,
        GameButton::Up,
        to["map"].as_str().expect("destination"),
        point(&to["arrival"]),
    );
    encounter(c, r);
}

fn door(
    c: &mut VisibleShellController,
    r: &crystal_runtime::CrystalRuntime,
    n: &Value,
    id: &str,
    east: bool,
) {
    let d = n["doors"]
        .as_array()
        .expect("doors")
        .iter()
        .find(|v| v["id"] == id)
        .expect("door");
    let (x, y) = point(&d["metatile"]);
    let (tile, facing) = if id == "RETURN" {
        ((x * 2, y * 2 - 1), GameButton::Down)
    } else if east {
        ((x * 2 + 2, y * 2), GameButton::Left)
    } else {
        ((x * 2 - 1, y * 2), GameButton::Right)
    };
    walk(c, r, tile);
    talk(c, facing);
    finish(c);
    assert!(event(c, d["flag"].as_str().expect("door result")));
}

#[test]
#[ignore = "requires the cross-floor regional headquarters and isolated saves"]
fn ice_headquarters_cross_floor_card_eight_pads_six_doors_optional_rooms_and_rescue() {
    headquarters_course(None);
}

#[test]
#[ignore = "requires regional headquarters; intended-level full headquarters course"]
fn ice_headquarters_intended_level_three_starters_complete_the_gauntlet() {
    for starter in ["SLIPPERINA", "RIMEPELT", "PORTABEAU"] {
        headquarters_course(Some(starter));
    }
}

fn headquarters_course(starter: Option<&str>) {
    let (root, r) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        r.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: r.title_new_game_spawn_identifier().expect("HOME"),
            map_name: if starter.is_some() {
                "MnMinneapolisDispatchOffice"
            } else {
                "MnIceOperations"
            }
            .into(),
            tile_x: if starter.is_some() { 10 } else { 4 },
            tile_y: if starter.is_some() { 14 } else { 19 },
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("prior-cargo fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    let state = c.shell.shell.session_mut().state_mut();
    let party = if let Some(starter) = starter {
        vec![
            (starter, 23),
            ("GILLPUP", 22),
            ("CINDORE", 22),
            ("NOTCHBILL", 22),
        ]
    } else {
        vec![
            ("BOREALYNX", 45),
            ("MUDMENDER", 43),
            ("GOOSNIDE", 42),
            ("PORTABEAU", 42),
        ]
    };
    for (species, level) in party {
        assert!(
            state.storage.party.add_pokemon(
                r.data()
                    .create_pokemon(species, level, Dv::from_non_hp(8, 9, 8, 9))
                    .expect("regional geometry partner")
            )
        );
    }
    state.sync_party_from_storage();
    assert!(
        state
            .bag
            .add_item(&r.data().items["SUPER_POTION"], 4)
            .expect("geometry supplies")
    );
    state.badges.johto = [true, true, false, false, false, false, false, false];
    for f in &crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[..2] {
        state.flags.set_engine_flag(f, true).expect("prior badges");
    }
    if starter.is_none() {
        for f in ["EVENT_MN_REED_JOINED_ICE", "EVENT_MN_ICE_CARGO_FEED_OFF"] {
            state
                .flags
                .set_event_flag(f, true)
                .expect("prior acted cargo chapter");
        }
    }
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64).expect("settled fixture");
    let _ = play_chapter(c, &root, &r, starter.is_some(), starter.is_some());
}

pub(super) fn play_chapter(
    mut c: VisibleShellController,
    root: &crystal_assets::AssetRoot,
    r: &crystal_runtime::CrystalRuntime,
    with_cargo: bool,
    normal: bool,
) -> VisibleShellController {
    let n = source();
    // No state writes beyond setup. Intended-level variants use ordinary DVs,
    // four shop potions and the real optional nurse. Their complete office-to-
    // rescue route earns recruitment and cargo access; party/badges are setup.
    assert!(!event(&c, KEY) && !event(&c, POWER) && !event(&c, RESCUED));
    if with_cargo {
        super::northstar_ice_lair::join(&mut c, r, root);
        walk(&mut c, r, (5, 3));
        travel(&mut c, GameButton::Up, "MnIceCargoBasement", (4, 19));
        walk(&mut c, r, (23, 18));
        talk(&mut c, GameButton::Up);
        choice(&mut c, true);
        finish(&mut c);
        assert!(event(&c, "EVENT_MN_ICE_CARGO_FEED_OFF"));
        cross(&mut c, r, &n, "cargo_stair", false);
    }
    walk(&mut c, r, (15, 14));
    talk(&mut c, GameButton::Right);
    page(&mut c, "MnIceHQDoorOPERATIONSLockedText");
    finish(&mut c);
    assert!(path(&mut c, r, (26, 19)).is_none(), "relay side is sealed");
    // A genuine native sight encounter, not a manually granted result.
    walk(&mut c, r, (11, 12));
    assert!(event(&c, "EVENT_MN_ICE_HQ_OPERATIONS_GUARD_DEFEATED"));
    cross(&mut c, r, &n, "C", false);
    walk(&mut c, r, (25, 18));
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    cross(&mut c, r, &n, "D", false);
    assert!(
        path(&mut c, r, (24, 5)).is_none(),
        "public stair hall cannot reach Ada's room"
    );
    cross(&mut c, r, &n, "records_stair", true);
    cross(&mut c, r, &n, "records_stair", false);
    cross(&mut c, r, &n, "workshop_stair", false);
    cross(&mut c, r, &n, "A", false);
    cross(&mut c, r, &n, "B", false);
    walk(&mut c, r, (24, 5));
    talk(&mut c, GameButton::Right);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, KEY));
    c = save(&mut c, root, r, "hq-card-declined");
    talk(&mut c, GameButton::Right);
    choice(&mut c, true);
    finish(&mut c);
    assert!(event(&c, KEY));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["CARD_KEY"]),
        1
    );
    c = save(&mut c, root, r, "hq-card-held");
    talk(&mut c, GameButton::Right);
    page(&mut c, "MnIceHQCardAfterText");
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["CARD_KEY"]),
        1
    );
    cross(&mut c, r, &n, "H", false);
    assert!(
        path(&mut c, r, (4, 19)).is_none(),
        "pad returns into a separate earlier-floor pocket"
    );
    door(&mut c, r, &n, "RETURN", false);
    c = save(&mut c, root, r, "hq-return-open");
    assert!(path(&mut c, r, (4, 19)).is_some());
    door(&mut c, r, &n, "OPERATIONS", false);
    cross(&mut c, r, &n, "records_stair", false);
    door(&mut c, r, &n, "REST", false);
    walk(&mut c, r, (24, 17));
    talk(&mut c, GameButton::Right);
    choice(&mut c, true);
    finish(&mut c);
    assert!(
        c.snapshot()
            .expect("native recovery")
            .party
            .slots
            .iter()
            .all(|s| s.pokemon.hp == s.pokemon.max_hp)
    );
    cross(&mut c, r, &n, "E", false);
    walk(&mut c, r, (24, 5));
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_ICE_HQ_TESS_HELPED") && !event(&c, RESCUED));
    door(&mut c, r, &n, "CELLS", true);
    door(&mut c, r, &n, "ANNEX", false);
    cross(&mut c, r, &n, "F", false);
    walk(&mut c, r, (24, 7));
    let before = c
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items["SUPER_POTION"]);
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["SUPER_POTION"]),
        before + 2
    );
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["SUPER_POTION"]),
        before + 2
    );
    door(&mut c, r, &n, "WORKSHOP", true);
    cross(&mut c, r, &n, "F", true);
    cross(&mut c, r, &n, "G", true);
    c = save(&mut c, root, r, "hq-all-doors-and-eight-pads");
    for d in n["doors"].as_array().expect("six doors") {
        assert!(event(&c, d["flag"].as_str().expect("flag")));
    }
    if normal {
        // The optional nurse is now reachable through the opened floor graph.
        // Use that actual service before the rival as well as the final boss.
        recover_at_records(&mut c, r, &n);
    }
    cross(&mut c, r, &n, "relay_stair", false);
    walk(&mut c, r, (21, 5));
    talk(&mut c, GameButton::Up);
    assert!(super::rochester_earned::try_battle(
        &mut c,
        r,
        "MN_REED_RELAY",
        "EVENT_MN_REED_RELAY_DEFEATED"
    ));
    finish(&mut c);
    assert!(!event(&c, POWER));
    walk(&mut c, r, (20, 3));
    talk(&mut c, GameButton::Up);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, POWER));
    talk(&mut c, GameButton::Up);
    choice(&mut c, true);
    finish(&mut c);
    assert!(event(&c, POWER) && !event(&c, RESCUED));
    c = save(&mut c, root, r, "hq-holding-power-off");
    finish_from_relay(c, root, r, &n, normal)
}

pub(super) fn resume_from_relay(
    c: VisibleShellController,
    root: &crystal_assets::AssetRoot,
    r: &crystal_runtime::CrystalRuntime,
) -> VisibleShellController {
    assert!(event(&c, KEY) && event(&c, POWER) && !event(&c, RESCUED));
    assert!(event(&c, "EVENT_MN_REED_RELAY_DEFEATED"));
    finish_from_relay(c, root, r, &source(), true)
}

fn finish_from_relay(
    mut c: VisibleShellController,
    root: &crystal_assets::AssetRoot,
    r: &crystal_runtime::CrystalRuntime,
    n: &Value,
    normal: bool,
) -> VisibleShellController {
    assert!(event(&c, POWER) && !event(&c, RESCUED));
    at(&mut c, "MnIceRelayFloor", (20, 3));
    cross(&mut c, r, n, "relay_stair", true);
    if normal {
        // Reed can exhaust a normal party. The unlocked nurse is a real return
        // destination before the enforcer, reached with ordinary walking.
        recover_at_records(&mut c, r, n);
    }
    cross(&mut c, r, n, "cargo_stair", true);
    walk(&mut c, r, (4, 18));
    travel(
        &mut c,
        GameButton::Right,
        "MnMinneapolisDispatchOffice",
        (5, 4),
    );
    if normal {
        // The added guards consume carried medicine. Restock through the city
        // shop with the money actually earned in this course before the boss.
        let (office_outside, office_inside) = ports::city_port(r, "MnMinneapolisDispatchOffice");
        let (mart_outside, mart_inside) = ports::city_port(r, "MnMinneapolisMart");
        walk(&mut c, r, (10, 14));
        travel(&mut c, GameButton::Down, "MnMinneapolis", office_outside);
        let item = &r.data().items["SUPER_POTION"];
        let state = c.shell.shell.session().state();
        let quantity = u32::from(5_u16.saturating_sub(state.bag.quantity(item)))
            .min(state.money / u32::from(item.price)) as u16;
        if quantity > 0 {
            super::authored_route_fixture::indoor_walk(&mut c, r, mart_outside);
            travel(&mut c, GameButton::Up, "MnMinneapolisMart", mart_inside);
            super::authored_route_fixture::indoor_walk(&mut c, r, (3, 3));
            talk(&mut c, GameButton::Left);
            super::rochester_earned::checkout(&mut c, r, "SUPER_POTION", quantity);
            super::authored_route_fixture::indoor_walk(&mut c, r, (6, 6));
            travel(&mut c, GameButton::Down, "MnMinneapolis", mart_outside);
        }
        super::authored_route_fixture::indoor_walk(&mut c, r, office_outside);
        travel(
            &mut c,
            GameButton::Up,
            "MnMinneapolisDispatchOffice",
            office_inside,
        );
    }
    walk(&mut c, r, (9, 9));
    talk(&mut c, GameButton::Left);
    assert!(super::rochester_earned::try_battle(
        &mut c,
        r,
        "MN_ICE_ENFORCER",
        "EVENT_MN_ICE_ENFORCER_DEFEATED"
    ));
    finish(&mut c);
    walk(&mut c, r, (11, 6));
    talk(&mut c, GameButton::Right);
    choice(&mut c, true);
    finish(&mut c);
    assert!(event(&c, RESCUED) && !event(&c, "EVENT_MN_DISPATCH_NOTE"));
    c = save(&mut c, root, r, "hq-ellis-released");
    c
}

fn recover_at_records(
    c: &mut VisibleShellController,
    r: &crystal_runtime::CrystalRuntime,
    n: &Value,
) {
    cross(c, r, n, "records_stair", false);
    walk(c, r, (24, 17));
    talk(c, GameButton::Right);
    choice(c, true);
    finish(c);
    assert!(
        c.snapshot()
            .expect("native nurse recovery")
            .party
            .slots
            .iter()
            .all(|slot| slot.pokemon.hp == slot.pokemon.max_hp)
    );
    cross(c, r, n, "records_stair", true);
}
