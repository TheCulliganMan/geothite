//! Placement and party are fixtures; talk, battle input and save use production paths.
use std::fs;
use std::path::Path;

use crystal_core::input::GameButton;
use crystal_core::models::{Dv, PcBox};
use crystal_core::state::{BattleMemory, OverworldMemory};
use crystal_core::world::map::Direction;
use serde_json::Value;

use super::super::{
    VisibleShellController, mark_runtime_snapshot_dirty, visible_field_dialog_text,
};
use super::interior_life::{fixture, load};

#[path = "minnesota_trainers_current.rs"]
mod current;

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the published Minnesota trainer expansion and external placement JSON"]
fn minnesota_trainers_controller_talk_scaling_evolution_turn_and_continue() {
    let (root, runtime, _) = load();
    assert!(runtime.data().minnesota_trainer_scaling);
    let placement_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/trainers/history/expansion-v1.json");
    let source: Value = serde_json::from_str(
        &fs::read_to_string(placement_path).expect("external trainer expansion placement fixture"),
    )
    .expect("valid trainer expansion placement JSON");
    for id in [
        "MN_EXTRA_R01_01",
        "MN_EXTRA_R09_01",
        "MN_EXTRA_R17_01",
        "MN_EXTRA_MINNEAPOLIS_01",
    ] {
        let row = source["trainers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == id)
            .unwrap();
        let map = row["map"].as_str().unwrap();
        let approach = (
            row["approach"][0].as_i64().unwrap() as i16,
            row["approach"][1].as_i64().unwrap() as i16,
        );
        let mut c = fixture(&root, &runtime, map, approach);
        let facing = match row["facing"].as_str().unwrap() {
            "west" => Direction::Right,
            "east" => Direction::Left,
            "north" => Direction::Down,
            "south" => Direction::Up,
            _ => unreachable!(),
        };
        {
            let (state, overworld) = c.shell.shell.session_mut().state_and_overworld_mut();
            for (index, (species, level)) in [("BOREALYNX", 59), ("POUCHID", 20), ("BIRCHIT", 20)]
                .into_iter()
                .enumerate()
            {
                state.storage.party.pokemon[index] = Some(
                    runtime
                        .data()
                        .create_pokemon(species, level, Dv::from_non_hp(15, 15, 15, 15))
                        .unwrap(),
                );
            }
            state.storage.pc_boxes = (0..14).map(PcBox::new).collect();
            for (box_index, species) in [(0, "CYPRIELLE"), (13, "VOYAGEAR")] {
                assert!(
                    state.storage.pc_boxes[box_index].add_pokemon(
                        runtime
                            .data()
                            .create_pokemon(species, 60, Dv::default())
                            .unwrap()
                    )
                );
            }
            state.sync_party_from_storage();
            overworld.set_player_facing(facing);
            state.overworld = OverworldMemory::from_snapshot(&overworld.snapshot());
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        c.press(GameButton::A).unwrap();
        let before = c.snapshot().unwrap();
        assert!(
            visible_field_dialog_text(&before, &c.shell).is_some(),
            "trainer must introduce the battle: {id}"
        );
        c.press(GameButton::Down).unwrap();
        assert_eq!(
            c.snapshot().unwrap().overworld.tile,
            before.overworld.tile,
            "dialogue owns movement"
        );
        for _ in 0..200 {
            if c.snapshot().unwrap().battle.is_some() {
                break;
            }
            c.press(GameButton::A).unwrap();
        }
        let state = c.shell.shell.session().state();
        let trainer = runtime.data().trainers.get(id).unwrap();
        let BattleMemory::Trainer {
            enemy_party: enemy, ..
        } = &state.battle
        else {
            panic!("production trainer battle must start: {id}");
        };
        assert_eq!(
            enemy.iter().map(|mon| mon.level).max(),
            Some(60),
            "PC collection outranks the active team's level-59 ace"
        );
        assert!(
            enemy
                .iter()
                .zip(&trainer.party)
                .any(|(mon, authored)| mon.species.id != authored.species),
            "training must evolve the team: {id}"
        );
        let path = std::env::temp_dir().join(format!(
            "minnesota-trainers-{}-{id}.crystalsave",
            std::process::id()
        ));
        c.save(&path).unwrap();
        let saved = runtime.load_save(&path).unwrap();
        assert_eq!(
            saved.battle,
            c.shell.shell.session().state().battle,
            "evolved team survives save/load"
        );
        for _ in 0..200 {
            let s = c.snapshot().unwrap();
            if s.ui.menu.as_ref().is_some_and(|menu| {
                menu.layout
                    .vertical_menus
                    .iter()
                    .any(|menu| menu.options.iter().any(|entry| entry == ">FIGHT"))
            }) {
                break;
            }
            c.press(GameButton::A).unwrap();
        }
        c.press(GameButton::A).unwrap();
        let before = c.snapshot().unwrap().party.slots[0].pokemon.moves.clone();
        c.press(GameButton::A).unwrap();
        let after = c.snapshot().unwrap();
        assert!(
            after.party.slots[0]
                .pokemon
                .moves
                .iter()
                .zip(before)
                .any(|(new, old)| new.current_pp < old.current_pp),
            "production battle executes the selected move: {id}"
        );
        assert!(!c.shell.battle_messages.is_empty());
        std::fs::remove_file(&path).unwrap();
        let _ = std::fs::remove_file(crystal_core::save::save_backup_path(&path));
    }
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the published Minneapolis cyclists"]
fn minneapolis_cyclists_controller_talk_turn_and_continue() {
    let (root, runtime, _) = load();
    for id in [
        "MN_EXTRA_MPLS_CYCLE_01",
        "MN_EXTRA_MPLS_CYCLE_13",
        "MN_EXTRA_MPLS_CYCLE_18",
    ] {
        let map = "MnMinneapolis";
        let city = &runtime.data().maps[map];
        let rider = city
            .objects
            .iter()
            .find(|object| object.script == id)
            .expect("installed cycling opponent");
        let x = rider.x as i16;
        let y = rider.y as i16;
        let (approach, facing) = [
            ((x, y + 1), Direction::Up),
            ((x - 1, y), Direction::Right),
            ((x + 1, y), Direction::Left),
            ((x, y - 1), Direction::Down),
        ]
        .into_iter()
        .find(|(point, _)| {
            let (px, py) = *point;
            if !(0..80).contains(&px) || !(0..64).contains(&py) {
                return false;
            }
            let block = city.blocks[(py / 2 * 40 + px / 2) as usize];
            runtime.data().tilesets[&city.attributes.tileset_name].collision
                [&format!("{block:02x}")][(py % 2 * 2 + px % 2) as usize]
                == "FLOOR"
                && !city
                    .objects
                    .iter()
                    .any(|o| (o.x as i16, o.y as i16) == *point)
                && !city
                    .events
                    .warps
                    .iter()
                    .any(|e| (e.x as i16, e.y as i16) == *point)
                && !city
                    .events
                    .coord_events
                    .iter()
                    .any(|e| (e.x as i16, e.y as i16) == *point)
        })
        .expect("safe approach to the actual installed rider");
        let mut c = fixture(&root, &runtime, map, approach);
        {
            let (state, overworld) = c.shell.shell.session_mut().state_and_overworld_mut();
            for (index, (species, level)) in [("BOREALYNX", 59), ("POUCHID", 20), ("BIRCHIT", 20)]
                .into_iter()
                .enumerate()
            {
                state.storage.party.pokemon[index] = Some(
                    runtime
                        .data()
                        .create_pokemon(species, level, Dv::from_non_hp(15, 15, 15, 15))
                        .unwrap(),
                );
            }
            state.storage.pc_boxes = (0..14).map(PcBox::new).collect();
            for (box_index, species) in [(0, "CYPRIELLE"), (13, "VOYAGEAR")] {
                assert!(
                    state.storage.pc_boxes[box_index].add_pokemon(
                        runtime
                            .data()
                            .create_pokemon(species, 60, Dv::default())
                            .unwrap()
                    )
                );
            }
            state.sync_party_from_storage();
            overworld.set_player_facing(facing);
            state.overworld = OverworldMemory::from_snapshot(&overworld.snapshot());
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        c.press(GameButton::A).unwrap();
        let before = c.snapshot().unwrap();
        assert!(
            visible_field_dialog_text(&before, &c.shell).is_some(),
            "trainer must introduce the battle: {id}"
        );
        c.press(GameButton::Down).unwrap();
        assert_eq!(
            c.snapshot().unwrap().overworld.tile,
            before.overworld.tile,
            "dialogue owns movement"
        );
        for _ in 0..200 {
            let snapshot = c.snapshot().unwrap();
            if snapshot.battle.is_some() {
                break;
            }
            if snapshot.ui.pending_yes_no.is_some()
                && c.text_view_selection_indices(&snapshot).1 != 0
            {
                // The movement-ownership check above may select NO. Accept
                // this optional battle explicitly rather than relying on the
                // prompt's cursor after directional input.
                c.press(GameButton::Up).unwrap();
            }
            c.press(GameButton::A).unwrap();
        }
        let state = c.shell.shell.session().state();
        let BattleMemory::Trainer {
            enemy_party: enemy, ..
        } = &state.battle
        else {
            panic!("production trainer battle must start: {id}");
        };
        let expected = if runtime.data().minnesota_trainer_scaling {
            60
        } else {
            runtime
                .data()
                .trainers
                .get(id)
                .expect("cyclist party")
                .party
                .iter()
                .map(|mon| mon.level)
                .max()
                .expect("nonempty cyclist party")
        };
        assert_eq!(enemy.iter().map(|mon| mon.level).max(), Some(expected));
        let path = std::env::temp_dir().join(format!(
            "minnesota-trainers-{}-{id}.crystalsave",
            std::process::id()
        ));
        c.save(&path).unwrap();
        let saved = runtime.load_save(&path).unwrap();
        assert_eq!(
            saved.battle,
            c.shell.shell.session().state().battle,
            "evolved team survives save/load"
        );
        c = VisibleShellController::load_save(root.clone(), runtime.clone(), path.clone(), None)
            .expect("production Continue into the cyclist battle");
        c.set_runtime_journal_enabled(false);
        for _ in 0..200 {
            let s = c.snapshot().unwrap();
            if s.ui.menu.as_ref().is_some_and(|menu| {
                menu.layout
                    .vertical_menus
                    .iter()
                    .any(|menu| menu.options.iter().any(|entry| entry == ">FIGHT"))
            }) {
                break;
            }
            c.press(GameButton::A).unwrap();
        }
        c.press(GameButton::A).unwrap();
        let before = c.snapshot().unwrap().party.slots[0].pokemon.moves.clone();
        c.press(GameButton::A).unwrap();
        let after = c.snapshot().unwrap();
        assert!(
            after.party.slots[0]
                .pokemon
                .moves
                .iter()
                .zip(before)
                .any(|(new, old)| new.current_pp < old.current_pp),
            "production battle executes the selected move: {id}"
        );
        assert!(!c.shell.battle_messages.is_empty());
        std::fs::remove_file(&path).unwrap();
        let _ = std::fs::remove_file(crystal_core::save::save_backup_path(&path));
    }
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the published Minneapolis cyclists"]
fn minneapolis_cyclist_victory_runs_the_authored_return_script() {
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, "MnMinneapolis", (48, 21));
    {
        let (state, overworld) = c.shell.shell.session_mut().state_and_overworld_mut();
        for (index, species) in ["BOREALYNX", "VOYAGEAR", "CYPRIELLE"]
            .into_iter()
            .enumerate()
        {
            state.storage.party.pokemon[index] = Some(
                runtime
                    .data()
                    .create_pokemon(species, 80, Dv::from_non_hp(15, 15, 15, 15))
                    .expect("active Minnesota battle fixture"),
            );
        }
        state.sync_party_from_storage();
        overworld.set_player_facing(Direction::Up);
        state.overworld = OverworldMemory::from_snapshot(&overworld.snapshot());
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    let original_tile = c
        .snapshot()
        .expect("cyclist challenge position")
        .overworld
        .tile;
    c.press(GameButton::A)
        .expect("challenge the actual cyclist");
    assert!(super::rochester_earned::try_battle(
        &mut c,
        &runtime,
        "MN_EXTRA_MPLS_CYCLE_01",
        "EVENT_MN_EXTRA_MPLS_CYCLE_01",
    ));
    for _ in 0..64 {
        c.wait_frames(32).expect("battle-return presentation");
        let snapshot = c.snapshot().expect("return to the city");
        assert_eq!(c.shell.last_error, None);
        assert_eq!(snapshot.overworld.map_name, "MnMinneapolis");
        assert_eq!(snapshot.overworld.tile, original_tile);
        if snapshot.battle.is_none()
            && !snapshot.ui.text_window_open
            && snapshot.ui.menu.is_none()
            && !c.has_pending_script_work()
        {
            c.press(GameButton::Start)
                .expect("overworld input after victory");
            assert!(
                c.snapshot()
                    .expect("production Start menu")
                    .ui
                    .menu
                    .is_some()
            );
            return;
        }
        c.press(GameButton::A)
            .expect("finish the authored battle return");
    }
    panic!("cyclist victory did not release overworld input");
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the published Minneapolis cyclists"]
fn minneapolis_cyclists_controller_patrol_stays_on_authored_lane() {
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, "MnMinneapolis", (48, 21));
    let mut visited = std::collections::BTreeSet::new();
    for _ in 0..128 {
        c.wait_frames(8)
            .expect("native autonomous bicycle movement");
        let tile = c
            .shell
            .shell
            .session()
            .overworld()
            .object_runtime_tile_by_id("MN_EXTRA_MPLS_CYCLE_01")
            .expect("loaded bicycle rider");
        assert!((48..=49).contains(&tile.x));
        assert!((20..=35).contains(&tile.y));
        assert!(tile.x == 48 || tile.x == 49 || tile.y == 20 || tile.y == 35);
        visited.insert((tile.x, tile.y));
    }
    assert!(visited.len() > 1, "bicycle rider must actually move");
    assert!(c.snapshot().expect("no sight battle").battle.is_none());
    if let Some(directory) = std::env::var_os("CRYSTAL_CYCLISTS_RENDER_DIR") {
        let mut app = super::menu_render_test_app(c.shell);
        app.update();
        let world = app.world_mut();
        let images = world
            .remove_resource::<bevy::prelude::Assets<bevy::prelude::Image>>()
            .expect("native CPU art");
        let canvas = super::pokemon_overworld_context_canvas(world, &images);
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("native review directory");
        canvas
            .save(directory.join("cyclists-native.png"))
            .expect("native cyclist view");
    }
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the published Minneapolis cyclists"]
fn minneapolis_cyclists_controller_decline_keeps_battle_closed() {
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, "MnMinneapolis", (48, 21));
    // Explicit initial facing; subsequent interaction uses production input.
    c.shell
        .shell
        .session_mut()
        .overworld_mut()
        .set_player_facing(Direction::Up);
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.press(GameButton::A).expect("talk to cyclist");
    for _ in 0..32 {
        if c.snapshot().expect("prompt").ui.pending_yes_no.is_some() {
            break;
        }
        c.press(GameButton::A).expect("show optional battle prompt");
    }
    assert!(c.snapshot().expect("yes/no").ui.pending_yes_no.is_some());
    c.press(GameButton::Down).expect("select No");
    c.press(GameButton::A).expect("decline battle");
    c.wait_frames(32).expect("resume city riding");
    assert!(c.snapshot().expect("declined").battle.is_none());
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_EXTRA_MPLS_CYCLE_01")
            .unwrap_or(false)
    );
}
