//! Current installed authored teams; optional collection-scaling stays separate.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::state::{BattleMemory, OverworldMemory};
use crystal_core::world::map::Direction;

use super::{VisibleShellController, fixture, load, mark_runtime_snapshot_dirty};

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with current fixed-level Minnesota trainers"]
fn current_authored_trainers_talk_team_turn_and_continue() {
    let (root, runtime, _) = load();
    assert!(!runtime.data().minnesota_trainer_scaling);
    for id in [
        "MN_R01_T01",
        "MN_R09_T01",
        "MN_R17_T04",
        "MN_EXTRA_MPLS_CYCLE_01",
    ] {
        let (map_name, map, actor) = runtime
            .data()
            .maps
            .iter()
            .filter(|(name, _)| name.starts_with("Mn"))
            .find_map(|(name, map)| {
                map.objects
                    .iter()
                    .find(|actor| actor.script == id)
                    .map(|actor| (name, map, actor))
            })
            .expect("trainer must have an actual installed regional actor");
        let (x, y) = (actor.x as i16, actor.y as i16);
        let width = map.attributes.width as i16 * 2;
        let height = map.attributes.height as i16 * 2;
        let (approach, facing) = [
            ((x, y + 1), Direction::Up),
            ((x - 1, y), Direction::Right),
            ((x + 1, y), Direction::Left),
            ((x, y - 1), Direction::Down),
        ]
        .into_iter()
        .find(|(point, _)| {
            let (px, py) = *point;
            if !(0..width).contains(&px) || !(0..height).contains(&py) {
                return false;
            }
            let block = map.blocks[(py / 2 * (width / 2) + px / 2) as usize];
            let kind = &runtime.data().tilesets[&map.attributes.tileset_name].collision
                [&format!("{block:02x}")][(py % 2 * 2 + px % 2) as usize];
            kind == "FLOOR"
                && !map
                    .objects
                    .iter()
                    .any(|o| (o.x as i16, o.y as i16) == *point)
                && !map
                    .events
                    .warps
                    .iter()
                    .any(|e| (e.x as i16, e.y as i16) == *point)
                && !map
                    .events
                    .coord_events
                    .iter()
                    .any(|e| (e.x as i16, e.y as i16) == *point)
        })
        .expect("safe approach to the current installed actor");
        let mut c = fixture(&root, &runtime, map_name, approach);
        {
            // Declared inspection party and placement; all later state changes
            // come from native conversation, battle inputs and Save/Continue.
            let (state, overworld) = c.shell.shell.session_mut().state_and_overworld_mut();
            state.storage.party.pokemon[0] = Some(
                runtime
                    .data()
                    .create_pokemon("BOREALYNX", 70, Dv::default())
                    .expect("active Minnesota inspection partner"),
            );
            state.sync_party_from_storage();
            overworld.set_player_facing(facing);
            state.overworld = OverworldMemory::from_snapshot(&overworld.snapshot());
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        c.press(GameButton::A).expect("talk to installed trainer");
        for _ in 0..200 {
            let snapshot = c.snapshot().expect("native trainer introduction");
            if snapshot.battle.is_some() {
                break;
            }
            if snapshot.ui.pending_yes_no.is_some()
                && c.text_view_selection_indices(&snapshot).1 != 0
            {
                c.press(GameButton::Up).expect("accept optional battle");
            }
            c.press(GameButton::A)
                .expect("advance trainer introduction");
        }
        let authored = runtime.data().trainers.get(id).expect("authored team");
        let state = c.shell.shell.session().state();
        let BattleMemory::Trainer { enemy_party, .. } = &state.battle else {
            panic!("native trainer battle required: {id}");
        };
        assert_eq!(enemy_party.len(), authored.party.len());
        for (actual, expected) in enemy_party.iter().zip(&authored.party) {
            assert_eq!(actual.species.id, expected.species, "{id}");
            assert_eq!(actual.level, expected.level, "{id}");
        }
        let before_save = state.battle.clone();
        let path = std::env::temp_dir().join(format!(
            "mn-current-trainer-{}-{id}.crystalsave",
            std::process::id()
        ));
        c.save(&path).expect("native battle Save");
        c = VisibleShellController::load_save(root.clone(), runtime.clone(), path.clone(), None)
            .expect("same-pack production Continue into trainer battle");
        c.set_runtime_journal_enabled(false);
        assert_eq!(c.shell.shell.session().state().battle, before_save);
        for _ in 0..200 {
            if c.shell.battle_action_cursor.is_some() {
                break;
            }
            c.press(GameButton::A).expect("settle continued battle");
        }
        let before = c.snapshot().expect("before native attack").party.slots[0]
            .pokemon
            .moves
            .clone();
        c.press(GameButton::A).expect("FIGHT");
        c.press(GameButton::A).expect("selected native move");
        let after = c.snapshot().expect("actual move result");
        assert!(
            after.party.slots[0]
                .pokemon
                .moves
                .iter()
                .zip(before)
                .any(|(new, old)| new.current_pp < old.current_pp),
            "native PP mutation: {id}"
        );
        assert!(!c.shell.battle_messages.is_empty());
        std::fs::remove_file(&path).expect("remove temporary inspection save");
        let backup = crystal_core::save::save_backup_path(&path);
        if backup.exists() {
            std::fs::remove_file(backup).expect("remove temporary backup");
        }
    }
}
