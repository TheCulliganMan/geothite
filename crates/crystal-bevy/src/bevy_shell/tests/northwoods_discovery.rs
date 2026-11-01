//! Declared local native party; every quest/seam action and Continue is production input.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, MAX_BOX_MONS, MAX_PC_BOXES, PcBox};
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::authored_route_fixture::{cleanup_reached_save, reached_save_path};
use super::interior_life::fixture;
use super::northstar_ice_lair::{choice, event, walk};
use super::r10_gallery::settle;
use super::winona::{at, load, press, travel};

const MAP: &str = "MnTallTaleTrail";
const OUTSIDE: &str = "MnBemidjiNorthwoods";
const GATE: &str = "MnTallTaleForestGate";

fn flag(name: &str) -> String {
    format!("EVENT_MN_TALL_TALE_{name}")
}
fn setup(map: &str, tile: (i16, i16)) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    setup_partner(map, tile, "POUCHID", 60)
}
fn setup_partner(
    map: &str,
    tile: (i16, i16),
    species: &str,
    level: u8,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    setup_storage(map, tile, species, level, InitialStorage::PartySpace)
}
#[derive(Clone, Copy)]
enum InitialStorage {
    PartySpace,
    PartyFull,
    AllFull,
    Recovery,
}

fn setup_storage(
    map: &str,
    tile: (i16, i16),
    species: &str,
    level: u8,
    storage: InitialStorage,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, map, tile);
    {
        let state = c.shell.shell.session_mut().state_mut();
        assert_eq!(
            state.storage.party.pokemon.iter().flatten().count(),
            0,
            "declared empty initial party"
        );
        let mut partner = runtime
            .data()
            .create_pokemon(species, level, Dv::default())
            .expect("native fixture partner");
        partner.item = Some("SMOKE_BALL".into());
        assert!(state.storage.party.add_pokemon(partner));
        if matches!(storage, InitialStorage::PartyFull | InitialStorage::AllFull) {
            for slot in 1..6 {
                state.storage.party.pokemon[slot] = Some(
                    runtime
                        .data()
                        .create_pokemon("POUCHID", 60, Dv::default())
                        .expect("declared full native party"),
                );
            }
            state.current_pc_box = 3;
            while state.storage.pc_boxes.len() <= state.current_pc_box {
                state
                    .storage
                    .pc_boxes
                    .push(PcBox::new(state.storage.pc_boxes.len()));
            }
        }
        if matches!(storage, InitialStorage::AllFull) {
            while state.storage.pc_boxes.len() < MAX_PC_BOXES {
                state
                    .storage
                    .pc_boxes
                    .push(PcBox::new(state.storage.pc_boxes.len()));
            }
            for pc_box in &mut state.storage.pc_boxes {
                for _ in 0..MAX_BOX_MONS {
                    let partner = runtime
                        .data()
                        .create_pokemon("POUCHID", 60, Dv::default())
                        .expect("declared full native storage");
                    assert!(pc_box.add_pokemon(partner));
                }
            }
        }
        state.sync_party_from_storage();
        state.money = 10000;
        if matches!(storage, InitialStorage::Recovery) {
            state.badges.johto = [true; 8];
            for badge in &BADGE_ENGINE_FLAGS[..8] {
                state
                    .flags
                    .set_engine_flag(badge, true)
                    .expect("declared earned badge fixture");
            }
        } else {
            assert!(
                state
                    .bag
                    .add_item(&runtime.data().items["MASTER_BALL"], 1)
                    .expect("declared capture resource")
            );
        }
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64).expect("initial native presentation");
    // No party, bag, position, event, HP, PP or RNG writes beyond setup.
    (root, runtime, c)
}
fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    name: &str,
) -> VisibleShellController {
    let path = reached_save_path(name);
    c.save(&path).expect("supported production save");
    let loaded = VisibleShellController::load_save(root.clone(), r.clone(), path.clone(), None)
        .expect("same-pack Continue");
    cleanup_reached_save(&path);
    loaded
}
fn open(c: &mut VisibleShellController, r: &CrystalRuntime, to: (i16, i16), facing: GameButton) {
    walk(c, r, to);
    press(c, facing);
    press(c, GameButton::A);
}
fn read(c: &mut VisibleShellController, r: &CrystalRuntime, to: (i16, i16), facing: GameButton) {
    open(c, r, to, facing);
    settle(c, "acknowledged northwoods page");
}
fn visible(c: &mut VisibleShellController, species: &str) -> bool {
    let actor = format!("MN_TALE_{species}");
    c.snapshot()
        .expect("native actors")
        .visible_objects
        .iter()
        .any(|o| o.object_identifier.as_deref() == Some(actor.as_str()))
}
fn discover(c: &mut VisibleShellController, root: &AssetRoot, r: &CrystalRuntime, stage: &str) {
    read(c, r, (10, 35), GameButton::Up);
    for (to, facing) in [
        ((22, 29), GameButton::Right),
        ((16, 15), GameButton::Right),
        ((29, 21), GameButton::Up),
    ] {
        read(c, r, to, facing);
        if to == (16, 15) {
            assert!(!visible(c, "BABEHEMOTH"));
        }
    }
    assert!(!event(c, &flag("OX_REVEALED")));
    walk(c, r, (28, 17));
    press(c, GameButton::Up);
    at(c, MAP, (28, 17)); // Native timber collision seals the only camp throat.

    read(c, r, (10, 35), GameButton::Up);
    assert!(event(c, &flag("CAMP_DIRECTIONS")));
    open(c, r, (28, 17), GameButton::Up);
    choice(c, false);
    settle(c, "refused limb movement");
    assert!(!event(c, &flag("CAMP_PATH_ACKNOWLEDGED")));
    press(c, GameButton::Up);
    at(c, MAP, (28, 17));

    open(c, r, (28, 17), GameButton::Up);
    choice(c, true);
    settle(c, "acknowledged optional path");
    assert!(event(c, &flag("CAMP_PATH_ACKNOWLEDGED")));
    travel(c, GameButton::Up, MAP, (28, 16));
    *c = continued(
        c,
        root,
        r,
        &format!("northwoods-{stage}-cleared-throat.crystalsave"),
    );
    travel(c, GameButton::Up, MAP, (28, 15));

    walk(c, r, (28, 9));
    assert!(!visible(c, "BUNJANT"));
    read(c, r, (28, 9), GameButton::Up);
    assert!(event(c, &flag("LOG_ACCOUNT_READ")));
    read(c, r, (10, 35), GameButton::Up);
    assert!(event(c, &flag("OX_REVEALED")) && event(c, &flag("CAMP_KEEPER_REVEALED")));
}
fn encounter(c: &mut VisibleShellController, r: &CrystalRuntime, species: &str) {
    let party: Vec<_> = c
        .shell
        .shell
        .session()
        .state()
        .storage
        .party
        .pokemon
        .iter()
        .flatten()
        .cloned()
        .collect();
    let expected_levels: Vec<_> = r.data().wild_challenge.map_or_else(
        || vec![60],
        |rules| {
            (0..=rules.level_spread * 2)
                .map(|variation| rules.encounter_level(&party, 60, variation))
                .collect()
        },
    );
    let (to, facing) = if species == "BABEHEMOTH" {
        ((19, 13), GameButton::Left)
    } else {
        ((30, 9), GameButton::Up)
    };
    open(c, r, to, facing);
    choice(c, true);
    for _ in 0..128 {
        if let Some(b) = c.snapshot().expect("native encounter").battle {
            assert_eq!(b.enemy_pokemon.species.id, species);
            assert!(
                expected_levels.contains(&b.enemy_pokemon.level),
                "native encounter must retain the authored floor and active party scaling"
            );
            return;
        }
        c.wait_frames(64).expect("battle transition");
    }
    panic!("native {species} encounter did not start");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the northwoods discovery review"]
fn northwoods_discovery_controller_both_seam_lanes_and_continue() {
    for lane in [4, 5] {
        let (root, r, mut c) = setup(OUTSIDE, (13, 12));
        let before = c.shell.shell.session().state().clone();
        travel(&mut c, GameButton::Up, GATE, (1, 4));
        walk(&mut c, &r, (8, lane));
        travel(&mut c, GameButton::Right, MAP, (3, 34));
        c = continued(
            &mut c,
            &root,
            &r,
            &format!("northwoods-gate-{lane}.crystalsave"),
        );
        at(&mut c, MAP, (3, 34));
        assert!(!visible(&mut c, "BABEHEMOTH") && !visible(&mut c, "BUNJANT"));
        travel(&mut c, GameButton::Up, GATE, (8, 4));
        walk(&mut c, &r, (1, lane));
        travel(&mut c, GameButton::Left, OUTSIDE, (13, 12));
        let after = c.shell.shell.session().state();
        assert!(
            after.flags.event_flags == before.flags.event_flags,
            "seam changed story events"
        );
        // Native map entry records several cleared engine bits explicitly.
        // Missing and false represent the same value in EventFlagMemory.
        for name in after
            .flags
            .engine_flags
            .keys()
            .chain(before.flags.engine_flags.keys())
        {
            assert_eq!(
                after
                    .flags
                    .is_engine_flag_set(name)
                    .expect("native engine flag"),
                before
                    .flags
                    .is_engine_flag_set(name)
                    .expect("prior native engine flag"),
                "seam changed engine flag {name}"
            );
        }
        assert_eq!(after.bag, before.bag);
        assert_eq!(after.money, before.money);
        let connections = &r.data().maps[OUTSIDE].attributes.connections;
        assert!(
            connections.iter().any(|a| a.direction == "north"
                && a.target_map == "MnR17C00R02"
                && a.offset == -12)
        );
        assert!(
            connections
                .iter()
                .any(|a| a.direction == "south" && a.target_map == "MnBemidji" && a.offset == -8)
        );
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the northwoods discovery review"]
fn northwoods_discovery_controller_held_clue_acknowledgment_and_continue() {
    let (root, r, mut c) = setup(MAP, (3, 34));
    open(&mut c, &r, (22, 29), GameButton::Right);
    c.wait_frames(128).expect("held clue page");
    assert!(!event(&c, &flag("CULVERT_READ")));
    let tile = c.snapshot().expect("held clue").overworld.tile;
    press(&mut c, GameButton::Down);
    assert_eq!(
        c.snapshot().expect("dialogue ownership").overworld.tile,
        tile
    );
    c = continued(&mut c, &root, &r, "northwoods-held-clue.crystalsave");
    assert!(!event(&c, &flag("CULVERT_READ")));
    settle(&mut c, "finish held observation");
    assert!(event(&c, &flag("CULVERT_READ")));
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the northwoods discovery review"]
fn northwoods_discovery_controller_earned_local_report_capture_and_other_retry() {
    for (first, second) in [("BABEHEMOTH", "BUNJANT"), ("BUNJANT", "BABEHEMOTH")] {
        let (root, r, mut c) = setup(MAP, (3, 34));
        discover(&mut c, &root, &r, first);
        c = continued(
            &mut c,
            &root,
            &r,
            &format!("northwoods-{first}-quest-revealed.crystalsave"),
        );
        encounter(&mut c, &r, first);
        capture(&mut c, first);
        assert!(event(&c, &flag(&format!("{first}_CAUGHT"))));
        assert!(!event(&c, &flag(&format!("{second}_CAUGHT"))));
        assert!(c.shell.shell.session().state().pokedex.has_caught(first));
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items["MASTER_BALL"]),
            0
        );
        encounter(&mut c, &r, second);
        super::rochester::flee(&mut c);
        settle(&mut c, "other creature RUN");
        assert!(!visible(&mut c, second));
        c = continued(
            &mut c,
            &root,
            &r,
            &format!("northwoods-{first}-run-hidden.crystalsave"),
        );
        assert!(!visible(&mut c, second));
        leave_trail(&mut c, &r);
        enter_trail(&mut c, &r);
        walk(
            &mut c,
            &r,
            if first == "BABEHEMOTH" {
                (19, 13)
            } else {
                (30, 9)
            },
        );
        assert!(!visible(&mut c, first));
        walk(
            &mut c,
            &r,
            if second == "BABEHEMOTH" {
                (19, 13)
            } else {
                (30, 9)
            },
        );
        assert!(visible(&mut c, second));
        c = continued(
            &mut c,
            &root,
            &r,
            &format!("northwoods-{first}-retry-revealed.crystalsave"),
        );
        assert!(visible(&mut c, second));
        assert!(
            event(&c, &flag(&format!("{first}_CAUGHT")))
                && !event(&c, &flag(&format!("{second}_CAUGHT")))
        );
    }
}

fn capture(c: &mut VisibleShellController, species: &str) {
    for _ in 0..256 {
        let s = c.snapshot().expect("real capture menus");
        if event(c, &flag(&format!("{species}_CAUGHT"))) && s.battle.is_none() {
            settle(c, "native caught continuation");
            return;
        }
        if let Some(name) = &c.shell.pending_name_choice {
            if name.nickname_pages.len() > 1 {
                press(c, GameButton::A);
            } else {
                press(c, GameButton::Down);
                press(c, GameButton::A);
            }
            continue;
        }
        let button = if c.shell.pokedex_menu_open
            || (s.ui.menu.is_none() && s.ui.text.is_some())
            || c.shell.field_pack_action_cursor.is_some()
            || c.shell.ball_cursor.is_some()
        {
            GameButton::A
        } else if c.shell.bag_cursor.is_some() {
            GameButton::Right
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            match cursor.option_index {
                0 | 1 => GameButton::Down,
                3 => GameButton::Left,
                _ => GameButton::A,
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    panic!("native {species} capture did not complete");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with northwoods v02 Headbutt tables"]
fn northwoods_tree_controller_native_headbutt_encounter_run_and_continue() {
    let (root, r, mut c) = setup_partner(MAP, (3, 34), "DENSY", 13);
    assert!(
        c.snapshot().expect("native level13 learnset").party.slots[0]
            .pokemon
            .moves
            .iter()
            .any(|movement| movement.name == "HEADBUTT")
    );
    let bag = c.shell.shell.session().state().bag.clone();
    let money = c.shell.shell.session().state().money;
    walk(&mut c, &r, (14, 19));
    press(&mut c, GameButton::Up);
    at(&mut c, MAP, (14, 19));
    // Genuine native misses advance normally; never rewrite the ID or RNG.
    for _ in 0..1024 {
        if c.snapshot()
            .expect("native Headbutt activity")
            .battle
            .is_some()
        {
            break;
        }
        let button = if c
            .shell
            .yes_no_cursor
            .as_ref()
            .is_some_and(|cursor| cursor.option_index != 0)
        {
            GameButton::Up
        } else {
            GameButton::A
        };
        press(&mut c, button);
    }
    let snapshot = c.snapshot().expect("actual tree battle");
    let battle = snapshot
        .battle
        .expect("native Headbutt must eventually find a resident");
    assert_eq!(battle.battle_type, "BATTLETYPE_TREE");
    let source_level = c
        .shell
        .shell
        .session()
        .state()
        .script_runtime
        .memory
        .get("wCurPartyLevel")
        .expect("native field encounter source level")
        .parse::<u8>()
        .expect("source level");
    let table = &r.data().field_encounters[MAP].tables["headbutt"];
    assert!(
        table
            .common
            .iter()
            .chain(&table.rare)
            .any(|slot| slot.species == battle.enemy_pokemon.species.id
                && slot.level == source_level),
        "native tree encounter must retain an actual authored resident and source level"
    );
    super::rochester::flee(&mut c);
    settle(&mut c, "native tree RUN continuation");
    assert_eq!(c.shell.shell.session().state().bag, bag);
    assert_eq!(c.shell.shell.session().state().money, money);
    assert!(!event(&c, &flag("CULVERT_READ")) && !event(&c, &flag("OX_REVEALED")));
    c = continued(&mut c, &root, &r, "northwoods-native-tree-run.crystalsave");
    at(&mut c, MAP, (14, 19));
    assert!(!event(&c, &flag("CAMP_PATH_ACKNOWLEDGED")));
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with corrected northwoods discovery"]
fn northwoods_discovery_controller_full_party_selected_box_delivery_and_continue() {
    let (root, r, mut c) = setup_storage(MAP, (3, 34), "POUCHID", 60, InitialStorage::PartyFull);
    let before = c.shell.shell.session().state().storage.clone();
    discover(&mut c, &root, &r, "full-party");
    encounter(&mut c, &r, "BUNJANT");
    capture(&mut c, "BUNJANT");
    let state = c.shell.shell.session().state();
    assert_eq!(state.current_pc_box, 3);
    assert_eq!(state.storage.pc_boxes.len(), before.pc_boxes.len());
    assert!(
        state.storage.party == before.party,
        "capture must preserve the full party"
    );
    for (index, original) in before.pc_boxes.iter().enumerate() {
        if index != 3 {
            assert!(
                &state.storage.pc_boxes[index] == original,
                "unselected box changed"
            );
        }
    }
    let box4 = &state.storage.pc_boxes[3];
    assert_eq!(box4.pokemon.iter().flatten().count(), 1);
    assert_eq!(
        box4.pokemon[0]
            .as_ref()
            .expect("native selected-box delivery")
            .species
            .id,
        "BUNJANT"
    );
    assert_eq!(state.bag.quantity(&r.data().items["MASTER_BALL"]), 0);
    assert!(state.pokedex.has_caught("BUNJANT"));
    let stored = state.storage.clone();
    c = continued(&mut c, &root, &r, "northwoods-boxed-bunjant.crystalsave");
    assert!(c.shell.shell.session().state().storage == stored);
    leave_trail(&mut c, &r);
    enter_trail(&mut c, &r);
    walk(&mut c, &r, (30, 9));
    assert!(!visible(&mut c, "BUNJANT"));
    travel(&mut c, GameButton::Up, MAP, (30, 8));
    assert!(
        c.shell.shell.session().state().storage == stored,
        "physical reentry must not duplicate capture"
    );
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with corrected northwoods discovery"]
fn northwoods_discovery_controller_knockout_hidden_continue_and_real_retry() {
    // A full declared counter party prevents the campaign helper's optional
    // single-partner recruitment branch; the result must come from combat.
    let (root, r, mut c) = setup_storage(MAP, (3, 34), "BOREALYNX", 80, InitialStorage::PartyFull);
    discover(&mut c, &root, &r, "knockout");
    encounter(&mut c, &r, "BUNJANT");
    assert!(super::rochester_earned::try_battle(&mut c, &r, "WILD", ""));
    settle(&mut c, "actual optional keeper knockout");
    assert!(!event(&c, &flag("BUNJANT_CAUGHT")) && !visible(&mut c, "BUNJANT"));
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .pokedex
            .has_caught("BUNJANT")
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["MASTER_BALL"]),
        1
    );
    c = continued(&mut c, &root, &r, "northwoods-knockout-hidden.crystalsave");
    assert!(!visible(&mut c, "BUNJANT"));
    leave_trail(&mut c, &r);
    enter_trail(&mut c, &r);
    encounter(&mut c, &r, "BUNJANT");
    super::rochester::flee(&mut c);
    settle(&mut c, "native retry battle RUN");
    assert!(!event(&c, &flag("BUNJANT_CAUGHT")));
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with northwoods discovery"]
fn northwoods_discovery_controller_full_storage_refusal_preserves_retry() {
    let (root, r, mut c) = setup_storage(MAP, (3, 34), "POUCHID", 60, InitialStorage::AllFull);
    let initial_pc_boxes = c.shell.shell.session().state().storage.pc_boxes.clone();
    discover(&mut c, &root, &r, "full-storage");
    encounter(&mut c, &r, "BUNJANT");
    // Walking can legitimately change party friendship. Isolate the capacity
    // refusal from the native journey and battle introduction before it.
    let original_storage = c.shell.shell.session().state().storage.clone();
    let original_bag = c.shell.shell.session().state().bag.clone();
    assert!(
        original_storage.pc_boxes == initial_pc_boxes,
        "the clue journey must preserve every PC creature"
    );
    let mut refused = false;
    for _ in 0..128 {
        let snapshot = c.snapshot().expect("native full-box capture attempt");
        assert!(
            snapshot.battle.is_some(),
            "full storage must retain the battle"
        );
        if snapshot
            .ui
            .text
            .as_ref()
            .and_then(|text| text.asm_text.as_deref())
            .is_some_and(|text| text.contains("BOX") && text.contains("full"))
        {
            refused = true;
            break;
        }
        let button = if (snapshot.ui.menu.is_none() && snapshot.ui.text.is_some())
            || c.shell.field_pack_action_cursor.is_some()
            || c.shell.ball_cursor.is_some()
        {
            GameButton::A
        } else if c.shell.bag_cursor.is_some() {
            GameButton::Right
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            match cursor.option_index {
                0 | 1 => GameButton::Down,
                3 => GameButton::Left,
                _ => GameButton::A,
            }
        } else {
            GameButton::A
        };
        press(&mut c, button);
    }
    assert!(refused, "native capture must expose the full-box refusal");
    let state = c.shell.shell.session().state();
    assert!(
        state.storage == original_storage,
        "refusal must preserve every stored creature"
    );
    assert!(
        state.bag == original_bag,
        "refusal must retain the Master Ball"
    );
    assert!(!state.pokedex.has_caught("BUNJANT"));
    assert!(!event(&c, &flag("BUNJANT_CAUGHT")));
    // Dismiss the refusal and back out of PACK through production inputs.
    for _ in 0..16 {
        if c.shell.battle_action_cursor.is_some()
            && c.shell.bag_cursor.is_none()
            && c.shell.ball_cursor.is_none()
            && c.shell.field_pack_action_cursor.is_none()
        {
            break;
        }
        press(&mut c, GameButton::B);
    }
    super::rochester::flee(&mut c);
    settle(&mut c, "native full-storage RUN");
    c = continued(
        &mut c,
        &root,
        &r,
        "northwoods-full-storage-refused.crystalsave",
    );
    assert!(!visible(&mut c, "BUNJANT"));
    leave_trail(&mut c, &r);
    enter_trail(&mut c, &r);
    encounter(&mut c, &r, "BUNJANT");
    super::rochester::flee(&mut c);
    settle(&mut c, "native full-storage encounter retry");
    let state = c.shell.shell.session().state();
    assert!(
        state.storage.pc_boxes == original_storage.pc_boxes && state.bag == original_bag,
        "refusal, Continue and physical reentry must preserve every PC creature and ball"
    );
    assert_eq!(
        state
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .map(|p| (&p.species.id, p.level, p.experience))
            .collect::<Vec<_>>(),
        original_storage
            .party
            .pokemon
            .iter()
            .flatten()
            .map(|p| (&p.species.id, p.level, p.experience))
            .collect::<Vec<_>>(),
        "walking may change friendship but must not add or replace a carried creature"
    );
    assert!(!event(&c, &flag("BUNJANT_CAUGHT")));
}

// Both the city departure and corridor have encounter grass. Replan from live
// collision and actor positions after every actual step, retaining native RUN.
fn walk_northwoods(c: &mut VisibleShellController, r: &CrystalRuntime, target: (i16, i16)) {
    use std::collections::{BTreeMap, BTreeSet, VecDeque};
    let name = c.snapshot().expect("woodland departure").overworld.map_name;
    assert!(matches!(name.as_str(), "MnBemidji" | OUTSIDE));
    for _ in 0..512 {
        let snapshot = c.snapshot().expect("live woodland walk");
        assert_eq!(snapshot.overworld.map_name, name);
        let start = (snapshot.overworld.tile.x, snapshot.overworld.tile.y);
        if start == target {
            return;
        }
        let occupied = snapshot
            .visible_objects
            .iter()
            .map(|object| {
                let at = c
                    .shell
                    .shell
                    .session()
                    .overworld()
                    .object_runtime_tile_by_id(
                        object
                            .object_identifier
                            .as_deref()
                            .expect("native actor identity"),
                    )
                    .expect("native actor position");
                (at.x, at.y)
            })
            .collect::<BTreeSet<_>>();
        let live = &c.shell.shell.session().overworld().map;
        let module = &r.data().maps[&name];
        let mut previous = BTreeMap::from([(start, (start, GameButton::A))]);
        let mut queue = VecDeque::from([start]);
        while let Some((x, y)) = queue.pop_front() {
            if (x, y) == target {
                break;
            }
            for (next, button) in [
                ((x - 1, y), GameButton::Left),
                ((x + 1, y), GameButton::Right),
                ((x, y - 1), GameButton::Up),
                ((x, y + 1), GameButton::Down),
            ] {
                let (xx, yy) = next;
                if xx < 0
                    || yy < 0
                    || xx >= live.width as i16 * 2
                    || yy >= live.height as i16 * 2
                    || previous.contains_key(&next)
                    || occupied.contains(&next)
                {
                    continue;
                }
                let block =
                    live.metatile_ids[yy as usize / 2 * live.width as usize + xx as usize / 2];
                let kind = &r.data().tilesets[&module.attributes.tileset_name].collision
                    [&format!("{block:02x}")][yy as usize % 2 * 2 + xx as usize % 2];
                if !matches!(kind.as_str(), "FLOOR" | "TALL_GRASS") {
                    continue;
                }
                previous.insert(next, ((x, y), button));
                queue.push_back(next);
            }
        }
        if !previous.contains_key(&target) {
            c.wait_frames(24).expect("passing city actor");
            continue;
        }
        let mut cursor = target;
        while previous[&cursor].0 != start {
            cursor = previous[&cursor].0;
        }
        let button = previous[&cursor].1;
        let before = snapshot.overworld.tile;
        for _ in 0..8 {
            press(c, button);
            if c.snapshot()
                .expect("ordinary woodland encounter")
                .battle
                .is_some()
            {
                super::rochester::flee(c);
            }
            if c.snapshot().expect("actual woodland step").overworld.tile != before {
                break;
            }
        }
    }
    panic!("bounded live woodland path {name} -> {target:?}");
}

fn enter_trail(c: &mut VisibleShellController, r: &CrystalRuntime) {
    at(c, OUTSIDE, (13, 12));
    travel(c, GameButton::Up, GATE, (1, 4));
    walk(c, r, (8, 4));
    travel(c, GameButton::Right, MAP, (3, 34));
}

fn leave_trail(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (3, 34));
    travel(c, GameButton::Up, GATE, (8, 4));
    walk(c, r, (1, 4));
    travel(c, GameButton::Left, OUTSIDE, (13, 12));
}

fn return_from_bemidji_clinic(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (10, 14));
    travel(c, GameButton::Down, "MnBemidji", (31, 10));
    // The two north checkpoint officers occupy both lanes. Reach the public
    // approach and let the qualified native coordinate script carry us through;
    // a collision-only planner must not try to route around their bodies.
    walk_northwoods(c, r, (24, 4));
    travel(c, GameButton::Up, OUTSIDE, (8, 27));
    walk_northwoods(c, r, (13, 12));
    enter_trail(c, r);
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with original-art northwoods discovery"]
fn northwoods_discovery_controller_blackout_nurse_continue_and_physical_return() {
    // Only the prior badges and weak native partner are declared. No prior
    // clinic or quest flags are supplied; nursing and both journeys are inputs.
    let (root, r, mut c) = setup_storage(
        "MnBemidjiPokecenter1F",
        (7, 5),
        "CINDORE",
        1,
        InitialStorage::Recovery,
    );
    read(&mut c, &r, (7, 5), GameButton::Up);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .last_spawn_map_constant
            .as_deref(),
        Some("MN_BEMIDJI_POKECENTER_1F")
    );
    return_from_bemidji_clinic(&mut c, &r);
    discover(&mut c, &root, &r, "blackout");
    let before = c.shell.shell.session().state().clone();
    encounter(&mut c, &r, "BABEHEMOTH");
    assert!(!super::rochester_earned::try_battle(&mut c, &r, "WILD", ""));
    settle(&mut c, "actual northwoods blackout");
    at(&mut c, "MnBemidjiPokecenter1F", (10, 13));
    let state = c.shell.shell.session().state();
    assert_eq!(state.battle_result & 0x3f, 1);
    assert_eq!(state.money, before.money / 2);
    assert!(state.bag == before.bag && state.badges == before.badges);
    assert!(!state.pokedex.has_caught("BABEHEMOTH"));
    assert!(!event(&c, &flag("BABEHEMOTH_CAUGHT")));
    assert!(event(&c, &flag("CAMP_PATH_ACKNOWLEDGED")) && event(&c, &flag("OX_REVEALED")));
    assert!(
        state
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .all(|p| p.hp == p.max_hp)
    );
    c = continued(&mut c, &root, &r, "northwoods-blackout-clinic.crystalsave");
    at(&mut c, "MnBemidjiPokecenter1F", (10, 13));
    press(&mut c, GameButton::Start);
    assert!(
        c.snapshot()
            .expect("recovered Start menu")
            .ui
            .menu
            .is_some()
    );
    press(&mut c, GameButton::B);
    let money = c.shell.shell.session().state().money;
    return_from_bemidji_clinic(&mut c, &r);
    encounter(&mut c, &r, "BABEHEMOTH");
    super::rochester::flee(&mut c);
    settle(&mut c, "actual post-blackout encounter retry");
    assert!(!event(&c, &flag("BABEHEMOTH_CAUGHT")));
    assert_eq!(c.shell.shell.session().state().money, money);
}

#[test]
#[ignore = "requires the northwoods review with the buoy-free pool"]
fn northwoods_discovery_controller_buoy_free_pool_walk_and_continue() {
    let (root, r, mut c) = setup(MAP, (3, 34));
    let room = &r.data().maps[MAP];
    for block in &room.blocks {
        assert!(
            r.data().tilesets[&room.attributes.tileset_name].collision[&format!("{block:02x}")]
                .iter()
                .all(|kind| kind != "BUOY"),
            "buoy collision remains in the pool"
        );
    }
    walk(&mut c, &r, (11, 11));
    at(&mut c, MAP, (11, 11));
    c = continued(&mut c, &root, &r, "northwoods-pool-no-buoys.crystalsave");
    at(&mut c, MAP, (11, 11));
    assert!(!event(&c, &flag("CAMP_PATH_ACKNOWLEDGED")));
    leave_trail(&mut c, &r);
    enter_trail(&mut c, &r);
    assert!(!event(&c, &flag("CAMP_PATH_ACKNOWLEDGED")));
}
