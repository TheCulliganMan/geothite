//! Current R19 native tree capture; old headland fixture coordinates moved.
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};

use super::super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::super::r10_gallery::continue_save;
use super::super::rochester::flee;
use super::super::rochester_earned::try_recruit_with_ball;
use super::super::winona::{load, press};
use super::{checkpoint, verify_capture};

#[test]
#[ignore = "requires current Minnesota pack; declared local Headbutt partner and capture ball"]
fn habitats_burrbud_current_r19_tree_capture_and_continue() {
    replay_tree_capture("BURRBUD", None);
}

#[test]
#[ignore = "requires curated R19 forest Headbutt table"]
fn ecology_forest_woodpecker_night_capture_and_continue() {
    replay_tree_capture("NOTCHBILL", Some(22));
}

fn replay_tree_capture(target: &str, hour: Option<u8>) {
    let (root, runtime) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnR19C00R00".into(),
            tile_x: 34,
            tile_y: 20,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("declared current tree shoulder");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("name");
    {
        let mut pokemon = runtime
            .data()
            .create_pokemon("NOTCHBILL", 60, Dv::default())
            .expect("Minnesota Headbutt partner");
        assert!(
            pokemon
                .species
                .tmhm_learnset
                .iter()
                .any(|name| name == "HEADBUTT")
        );
        pokemon.moves = vec![LearnedMove {
            name: "HEADBUTT".into(),
            current_pp: runtime.data().moves["HEADBUTT"].pp,
            pp_ups: 0,
        }];
        pokemon.item = Some("SMOKE_BALL".into());
        let state = controller.shell.shell.session_mut().state_mut();
        state.storage.party = Default::default();
        assert!(state.storage.party.add_pokemon(pokemon));
        state.sync_party_from_storage();
        if let Some(hour) = hour {
            state.time.update_server_datetime(
                crystal_core::systems::time::GameDate {
                    year: 2000,
                    month: 1,
                    day: 3,
                },
                hour,
                0,
                0,
            );
        }
        assert!(
            state
                .bag
                .add_item(&runtime.data().items["MASTER_BALL"], 1)
                .expect("declared capture ball")
        );
    }
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    // No player-ID, RNG, divider, position, inventory or party writes follow.
    let room = &runtime.data().maps["MnR19C00R00"];
    let block = room.blocks[10 * room.attributes.width as usize + 16];
    assert_eq!(
        runtime.data().tilesets[&room.attributes.tileset_name].collision[&format!("{block:02x}")]
            [1],
        "HEADBUTT_TREE"
    );
    press(&mut controller, GameButton::Left);
    let prefix = if target == "BURRBUD" {
        "burrbud-current-tree"
    } else {
        "ecology-woodpecker-night-tree"
    };
    checkpoint(&mut controller, &format!("{prefix}-start"));
    for _ in 0..16384 {
        let snapshot = controller.snapshot().expect("native tree selection");
        assert_eq!(
            (snapshot.overworld.tile.x, snapshot.overworld.tile.y),
            (34, 20)
        );
        if let Some(battle) = snapshot.battle {
            assert_eq!(battle.battle_type, "BATTLETYPE_TREE");
            if battle.enemy_pokemon.species.id == target {
                assert!(try_recruit_with_ball(
                    &mut controller,
                    &runtime,
                    "MASTER_BALL"
                ));
                verify_capture(&controller, &runtime, target);
                controller = continue_save(root, runtime.clone(), &mut controller);
                verify_capture(&controller, &runtime, target);
                checkpoint(&mut controller, &format!("{prefix}-caught"));
                return;
            }
            flee(&mut controller);
            continue;
        }
        let button = if controller
            .shell
            .yes_no_cursor
            .as_ref()
            .is_some_and(|cursor| cursor.option_index != 0)
        {
            GameButton::Up
        } else {
            GameButton::A
        };
        press(&mut controller, button);
    }
    panic!("current R19 Headbutt tree did not select {target} through native rolls");
}
