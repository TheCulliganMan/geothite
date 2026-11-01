//! Declared local partners; switch participation and earned XP use native input.
use crystal_core::input::GameButton;
use crystal_runtime::RuntimeBattleKind;

use crate::bevy_shell::tests::r10_gallery::continue_save;
use crate::bevy_shell::tests::rochester_earned::train_wild;
use crate::bevy_shell::tests::winona::{load, press};
use crate::bevy_shell::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};

#[test]
#[ignore = "requires current Minnesota pack; declared local party, native switch/XP/Continue only"]
fn lake_native_switch_training_keeps_participant_alive_and_awards_xp_through_continue() {
    let (root, runtime) = load();
    let targets = super::grass(&runtime);
    assert_eq!(targets[0].1 / 48, targets[1].1 / 48);
    let map = format!("MnR06C00R{:02}", targets[0].1 / 48);
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: map,
            tile_x: targets[0].0,
            tile_y: targets[0].1 % 48,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("local field access fixture");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("local trainer name");
    {
        let state = controller.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.pokemon.iter().all(Option::is_none));
        // These levels isolate switch participation from encounter difficulty.
        // They do not establish balance or earned access to the Lake chapter.
        for (species, level) in [("VELVAWN", 45), ("BOREALYNX", 60)] {
            let mut partner = runtime
                .data()
                .create_pokemon(species, level, Default::default())
                .expect("active Minnesota partner");
            partner.item = None;
            assert!(state.storage.party.add_pokemon(partner));
        }
        for room in runtime
            .data()
            .maps
            .values()
            .filter(|room| room.id.starts_with("MnR06"))
        {
            for trainer in room.trainer_scripts.values() {
                state
                    .flags
                    .set_event_flag(&trainer.event_flag, true)
                    .expect("declared previously defeated road trainers");
            }
        }
        state.sync_party_from_storage();
    }
    mark_runtime_snapshot_dirty(&mut controller.shell);
    let before = controller
        .snapshot()
        .expect("declared two-partner fixture")
        .party;
    assert_eq!(before.slots.len(), 2);
    let mut encountered = false;
    for step in 0..512 {
        let snapshot = controller.snapshot().expect("actual grass movement");
        if let Some(battle) = snapshot.battle {
            assert!(matches!(battle.kind, RuntimeBattleKind::Wild { .. }));
            encountered = true;
            break;
        }
        let at = (snapshot.overworld.tile.x, snapshot.overworld.tile.y);
        let first = (targets[0].0, targets[0].1 % 48);
        let second = (targets[1].0, targets[1].1 % 48);
        assert!(
            at == first || at == second,
            "remain on the declared grass pair"
        );
        let target = if at == first { second } else { first };
        let button = if at.0 < target.0 {
            GameButton::Right
        } else if at.0 > target.0 {
            GameButton::Left
        } else if at.1 < target.1 {
            GameButton::Down
        } else {
            GameButton::Up
        };
        press(&mut controller, button);
        controller
            .wait_frames(17 + step % 11)
            .expect("ordinary field idle");
    }
    assert!(encountered, "actual authored grass encounter required");
    assert!(
        train_wild(&mut controller, &runtime),
        "actual native wild victory required"
    );
    let after = controller
        .snapshot()
        .expect("native participation rewards")
        .party;
    assert!(after.slots[0].pokemon.hp > 0, "training lead must survive");
    assert!(after.slots[0].pokemon.experience > before.slots[0].pokemon.experience);
    assert!(
        after.slots[1].pokemon.moves.iter().any(|learned| {
            before.slots[1]
                .pokemon
                .moves
                .iter()
                .any(|old| old.name == learned.name && learned.current_pp < old.current_pp)
        }),
        "stronger partner must actually switch in and spend PP"
    );
    controller = continue_save(root, runtime, &mut controller);
    assert_eq!(
        controller
            .snapshot()
            .expect("continued earned participation")
            .party,
        after
    );
}
