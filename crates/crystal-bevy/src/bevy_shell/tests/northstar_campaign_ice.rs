//! Earn the Minneapolis chapter with the actual earlier campaign party.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::authored_route_fixture::indoor_walk;
use super::super::minneapolis::release_and_read;
use super::super::native_city_ports::city_port;
use super::super::rochester::flee;
use super::super::rochester_earned::{checkout, try_battle};
use super::super::winona::{at as location, continue_in_place, finish, load, press, travel};
use super::northstar_campaign::{learn_tm, prairie};
use super::river_cities::try_road_with;
use super::{VisibleShellController, event};

const WON: &str = "EVENT_MN_ICE_ENFORCER_DEFEATED";
const RESCUED: &str = "EVENT_MN_ICE_CAPTIVE_RESCUED";
const NOTE: &str = "EVENT_MN_DISPATCH_NOTE";

fn arrival() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let Some(path) = std::env::var_os("CRYSTAL_NORTHSTAR_ICE_SAVE") else {
        return prairie();
    };
    // Only a real exported Minneapolis arrival or clinic recovery is accepted.
    // With no override the entire course starts at Winona New Game.
    let (root, r) = load();
    let mut c =
        VisibleShellController::load_save(root.clone(), r.clone(), PathBuf::from(path), None)
            .expect("actual earned arrival with the same pack identity");
    c.set_runtime_journal_enabled(false);
    match c
        .snapshot()
        .expect("actual Minneapolis checkpoint")
        .overworld
        .map_name
        .as_str()
    {
        "MnMinneapolis" => location(&mut c, "MnMinneapolis", (40, 62)),
        "MnMinneapolisPokecenter1F" => location(&mut c, "MnMinneapolisPokecenter1F", (10, 13)),
        road if road.starts_with("MnR03") => return_to_city(&mut c, &r),
        _ => panic!("expected an earned Minneapolis arrival or native blackout recovery"),
    }
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, false, false, false, false, false, false]
    );
    for flag in [
        "EVENT_MN_GYM_1_DEFEATED",
        "EVENT_MN_GYM_2_DEFEATED",
        "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
    ] {
        assert!(event(&c, flag));
    }
    for flag in [WON, RESCUED, NOTE] {
        assert!(!event(&c, flag));
    }
    (root, r, c)
}

pub(in crate::bevy_shell::tests) fn care(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let (door, arrival) = city_port(r, "MnMinneapolis", "MnMinneapolisPokecenter1F");
    if c.snapshot().expect("city care").overworld.map_name == "MnMinneapolis" {
        indoor_walk(c, r, door);
        travel(c, GameButton::Up, "MnMinneapolisPokecenter1F", arrival);
    }
    indoor_walk(c, r, (7, 5));
    press(c, GameButton::Up);
    press(c, GameButton::A);
    finish(c);
    assert!(
        c.snapshot()
            .expect("free care")
            .party
            .slots
            .iter()
            .all(|s| {
                s.pokemon.hp == s.pokemon.max_hp
                    && s.pokemon.status.is_none()
                    && s.pokemon
                        .moves
                        .iter()
                        .all(|m| m.current_pp == r.data().moves[&m.name].pp)
            })
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .last_spawn_map_constant
            .as_deref(),
        Some("MN_MINNEAPOLIS_POKECENTER_1F")
    );
    indoor_walk(c, r, (10, 14));
    travel(c, GameButton::Down, "MnMinneapolis", door);
}

pub(super) fn supplies(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let item = &r.data().items["SUPER_POTION"];
    let state = c.shell.shell.session().state();
    let quantity = (state.money / u32::from(item.price))
        .min(u32::from(3_u16.saturating_sub(state.bag.quantity(item)))) as u16;
    if quantity == 0 {
        return;
    }
    let (door, arrival) = city_port(r, "MnMinneapolis", "MnMinneapolisMart");
    indoor_walk(c, r, door);
    travel(c, GameButton::Up, "MnMinneapolisMart", arrival);
    indoor_walk(c, r, (3, 3));
    press(c, GameButton::Left);
    press(c, GameButton::A);
    checkout(c, r, "SUPER_POTION", quantity);
    indoor_walk(c, r, (6, 6));
    travel(c, GameButton::Down, "MnMinneapolis", door);
}

fn return_to_city(c: &mut VisibleShellController, r: &CrystalRuntime) {
    if c.snapshot()
        .expect("road return")
        .overworld
        .map_name
        .starts_with("MnR03")
        && try_road_with(c, r, (20, 0), flee)
    {
        super::northstar_campaign::enter_minneapolis(c, r);
    } else {
        location(c, "MnMinneapolisPokecenter1F", (10, 13));
    }
}

fn road_trainers(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    for (tile, facing, id, flag) in [
        (
            (36, 49),
            GameButton::Right,
            "MN_R03_T01",
            "EVENT_MN_R03_TRAINER_01",
        ),
        (
            (14, 17),
            GameButton::Right,
            "MN_R03_T02",
            "EVENT_MN_R03_TRAINER_02",
        ),
        (
            (15, 30),
            GameButton::Up,
            "MN_R03_T03",
            "EVENT_MN_R03_TRAINER_03",
        ),
        (
            (36, 37),
            GameButton::Right,
            "MN_R03_T04",
            "EVENT_MN_R03_TRAINER_04",
        ),
    ] {
        for attempt in 0..4 {
            if event(c, flag) {
                break;
            }
            care(c, r);
            indoor_walk(c, r, (40, 62));
            travel(c, GameButton::Down, "MnR03C00R00", (20, 0));
            if try_road_with(c, r, tile, flee) {
                press(c, facing);
                press(c, GameButton::A);
                let _ = try_battle(c, r, id, flag);
            }
            return_to_city(c, r);
            continue_in_place(c, root, r, &format!("campaign-{id}-{attempt}"));
        }
        assert!(event(c, flag));
    }
}

pub(super) fn train(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    root: &AssetRoot,
    level: u8,
) {
    for outing in 0..96 {
        if c.snapshot().expect("earned level").party.slots[0]
            .pokemon
            .level
            >= level
        {
            return;
        }
        care(c, r);
        indoor_walk(c, r, (40, 62));
        travel(c, GameButton::Down, "MnR03C00R00", (20, 0));
        let mut fought = false;
        for step in 0..64 {
            let target = if step % 2 == 0 { (16, 8) } else { (21, 8) };
            let arrived = try_road_with(c, r, target, |c| {
                let s = c.snapshot().expect("wild training");
                let b = s.battle.as_ref().expect("actual encounter");
                if !fought
                    && b.enemy_pokemon.level
                        <= s.party
                            .slots
                            .iter()
                            .map(|p| p.pokemon.level)
                            .max()
                            .expect("carried training partner")
                            + 1
                {
                    let _ = try_battle(c, r, "WILD", "");
                    fought = true;
                } else {
                    flee(c);
                }
            });
            if fought || !arrived {
                break;
            }
        }
        return_to_city(c, r);
        continue_in_place(
            c,
            root,
            r,
            &format!("campaign-ice-training-{level}-{outing}"),
        );
    }
    assert!(
        c.snapshot().expect("bounded native training").party.slots[0]
            .pokemon
            .level
            >= level
    );
}

#[test]
#[ignore = "live-RNG New Game campaign; optional CRYSTAL_NORTHSTAR_ICE_SAVE is an actual earned Minneapolis arrival"]
fn northstar_campaign_earned_party_defeats_ice_and_releases_dispatcher() {
    super::northstar_campaign_review::require_unboosted();
    let _ = ice();
}

pub(super) fn ice() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    if let Some(path) = std::env::var_os("CRYSTAL_NORTHSTAR_RELAY_SAVE") {
        let (root, r) = load();
        let mut c =
            VisibleShellController::load_save(root.clone(), r.clone(), PathBuf::from(path), None)
                .expect("genuine same-pack earned relay checkpoint");
        c.set_runtime_journal_enabled(false);
        assert_eq!(
            c.shell.shell.session().state().badges.johto,
            [true, true, false, false, false, false, false, false]
        );
        for flag in [
            "EVENT_MN_GYM_1_DEFEATED",
            "EVENT_MN_GYM_2_DEFEATED",
            "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
            "EVENT_MN_REED_RELAY_DEFEATED",
        ] {
            assert!(event(&c, flag), "earned relay prerequisite {flag}");
        }
        // No party, item, position, flags or RNG rewrites after Continue.
        c = super::super::northstar_ice_headquarters::resume_from_relay(c, &root, &r);
        c = super::northstar_arc_campaign::ellis_home(c, &root, &r);
        return (root, r, c);
    }
    let (root, r, mut c) = arrival();
    super::northstar_campaign_review::boost(&mut c, &r);
    care(&mut c, &r);
    let forgotten = if c.snapshot().expect("earned move choice").party.slots[0]
        .pokemon
        .moves
        .iter()
        .any(|m| m.name == "LEER")
    {
        "LEER"
    } else {
        "QUICK_ATTACK"
    };
    if c.snapshot()
        .expect("legal earned TM compatibility")
        .party
        .slots[0]
        .pokemon
        .species
        .tmhm_learnset
        .iter()
        .any(|m| m == "MUD_SLAP")
    {
        learn_tm(&mut c, &r, "TM_MUD_SLAP", "MUD_SLAP", forgotten);
    }
    assert!(
        c.snapshot().expect("preserved first gym move").party.slots[0]
            .pokemon
            .moves
            .iter()
            .any(|m| m.name == "HEADBUTT")
    );
    let (home_door, home_arrival) = city_port(&r, "MnMinneapolis", "MnMinneapolisDispatcherHouse");
    indoor_walk(&mut c, &r, home_door);
    travel(
        &mut c,
        GameButton::Up,
        "MnMinneapolisDispatcherHouse",
        home_arrival,
    );
    indoor_walk(&mut c, &r, (6, 4));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    indoor_walk(&mut c, &r, (5, 6));
    travel(&mut c, GameButton::Down, "MnMinneapolis", home_door);
    road_trainers(&mut c, &r, &root);
    train(&mut c, &r, &root, 18);
    if r.data().maps.contains_key("MnBoundaryIntake") {
        super::northstar_arc_campaign::recruit_party(&mut c, &r, &root);
        super::northstar_arc_campaign::prepare_cut_carrier(&mut c, &r, &root);
        train(&mut c, &r, &root, 23);
        care(&mut c, &r);
        supplies(&mut c, &r);
        let (door, arrival) = city_port(&r, "MnMinneapolis", "MnMinneapolisDispatchOffice");
        indoor_walk(&mut c, &r, door);
        travel(
            &mut c,
            GameButton::Up,
            "MnMinneapolisDispatchOffice",
            arrival,
        );
        c = super::super::northstar_ice_headquarters::play_chapter(c, &root, &r, true, true);
        c = super::northstar_arc_campaign::ellis_home(c, &root, &r);
        return (root, r, c);
    }
    for attempt in 0..4 {
        care(&mut c, &r);
        supplies(&mut c, &r);
        let (door, arrival) = city_port(&r, "MnMinneapolis", "MnMinneapolisDispatchOffice");
        indoor_walk(&mut c, &r, door);
        travel(
            &mut c,
            GameButton::Up,
            "MnMinneapolisDispatchOffice",
            arrival,
        );
        indoor_walk(&mut c, &r, (9, 9));
        let money = c.shell.shell.session().state().money;
        press(&mut c, GameButton::Left);
        press(&mut c, GameButton::A);
        if try_battle(&mut c, &r, "MN_ICE_ENFORCER", WON) {
            assert!(c.shell.shell.session().state().money > money);
            break;
        }
        location(&mut c, "MnMinneapolisPokecenter1F", (10, 13));
        for flag in [WON, RESCUED, NOTE] {
            assert!(!event(&c, flag));
        }
        continue_in_place(&mut c, &root, &r, &format!("campaign-ice-loss-{attempt}"));
        if attempt < 3 {
            let level = c.snapshot().expect("earned preparation level").party.slots[0]
                .pokemon
                .level;
            train(&mut c, &r, &root, level.saturating_add(1).max(19 + attempt));
        }
    }
    assert!(event(&c, WON));
    assert!(
        c.snapshot()
            .expect("earned Mud-Slap used in the fight")
            .party
            .slots[0]
            .pokemon
            .moves
            .iter()
            .any(|m| m.name == "MUD_SLAP" && m.current_pp < r.data().moves["MUD_SLAP"].pp)
    );
    assert!(!event(&c, RESCUED) && !event(&c, NOTE));
    c = release_and_read(&root, &r, c);
    for flag in [WON, RESCUED, NOTE] {
        assert!(event(&c, flag));
    }
    continue_in_place(&mut c, &root, &r, "campaign-earned-ice-dispatcher-home");
    // The chapter cannot supply the next badge or later investigation results.
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, false, false, false, false, false, false]
    );
    assert!(!event(&c, "EVENT_MN_COURIER_DEFEATED"));
    (root, r, c)
}
