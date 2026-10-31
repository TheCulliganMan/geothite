//! Native evolution with move replacement must register both Dex states.
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::state::OverworldMemory;
use crystal_core::world::map::Direction;
use crystal_runtime::RuntimeBattleKind;

use super::super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::super::interior_life::collision;
use super::super::winona::{load, press};

#[test]
#[ignore = "requires a current Minnesota pack; declared near-level fixture, then native battle/learning/Continue"]
fn earned_evolution_native_prairondo_move_replacement_records_caught_dex_and_continues() {
    let (root, runtime) = load();
    let id = "MN_R01_T01";
    let (name, map, actor) = runtime
        .data()
        .maps
        .iter()
        .find_map(|(name, map)| {
            map.objects
                .iter()
                .find(|actor| actor.script == id)
                .map(|actor| (name, map, actor))
        })
        .expect("installed early Minnesota trainer");
    let (x, y) = (actor.x as i16, actor.y as i16);
    let (point, facing) = [
        ((x, y + 1), Direction::Up),
        ((x - 1, y), Direction::Right),
        ((x + 1, y), Direction::Left),
        ((x, y - 1), Direction::Down),
    ]
    .into_iter()
    .find(|(point, _)| {
        collision(&runtime, map, *point) == Some("FLOOR")
            && !map
                .objects
                .iter()
                .any(|actor| (actor.x as i16, actor.y as i16) == *point)
            && !map
                .events
                .warps
                .iter()
                .any(|warp| (warp.x as i16, warp.y as i16) == *point)
            && !map
                .events
                .coord_events
                .iter()
                .any(|event| (event.x as i16, event.y as i16) == *point)
    })
    .expect("unoccupied safe trainer approach");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: name.clone(),
            tile_x: point.0,
            tile_y: point.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("declared learning fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    {
        // Legal near-level experience and the carried moves that exposed the
        // coverage gap are explicit setup, not an earned campaign checkpoint.
        let mut pokemon = runtime
            .data()
            .create_pokemon("BOOMRUFF", 37, Dv::default())
            .expect("regional partner");
        pokemon.experience = runtime
            .data()
            .create_pokemon("BOOMRUFF", 38, Dv::default())
            .expect("native next-level threshold")
            .experience
            - 1;
        pokemon.moves = ["BODY_SLAM", "WING_ATTACK", "MUD_SLAP", "FOCUS_ENERGY"]
            .into_iter()
            .map(|name| LearnedMove {
                name: name.into(),
                current_pp: runtime.data().moves[name].pp,
                pp_ups: 0,
            })
            .collect();
        let (state, overworld) = c.shell.shell.session_mut().state_and_overworld_mut();
        state.storage.party = Default::default();
        state.pokedex.record_caught_pokemon(&pokemon);
        assert!(state.storage.party.add_pokemon(pokemon));
        state.sync_party_from_storage();
        overworld.set_player_facing(facing);
        state.overworld = OverworldMemory::from_snapshot(&overworld.snapshot());
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    // No party, experience, move, HP/PP, progression or RNG writes after setup.
    for _ in 0..128 {
        if c.snapshot().expect("trainer introduction").battle.is_some() {
            break;
        }
        press(&mut c, GameButton::A);
    }
    let battle = c
        .snapshot()
        .expect("native battle")
        .battle
        .expect("trainer battle started");
    let RuntimeBattleKind::Trainer {
        trainer_id,
        event_flag,
        ..
    } = battle.kind
    else {
        panic!("authored trainer battle required");
    };
    assert_eq!(trainer_id, id);
    assert!(super::try_battle(&mut c, &runtime, id, &event_flag));
    let before = c.snapshot().expect("native learned coverage").party;
    let pokemon = &before.slots[0].pokemon;
    assert_eq!(pokemon.level, 38);
    assert_eq!(pokemon.species.id, "PRAIRONDO");
    assert!(
        pokemon
            .moves
            .iter()
            .any(|learned| learned.name == "DIG" && learned.current_pp > 0)
    );
    assert!(
        !pokemon
            .moves
            .iter()
            .any(|learned| learned.name == "MUD_SLAP")
    );
    for retained in ["BODY_SLAM", "WING_ATTACK", "FOCUS_ENERGY"] {
        assert!(pokemon.moves.iter().any(|learned| learned.name == retained));
    }
    assert!(
        pokemon
            .moves
            .iter()
            .any(|learned| learned.current_pp < runtime.data().moves[&learned.name].pp)
    );
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .pokedex
            .has_caught("BOOMRUFF")
    );
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .pokedex
            .has_caught("PRAIRONDO"),
        "accepted native evolution must record caught: pending_learn={:?}, cancellations={}, messages={}, evolution_trace={:?}",
        c.shell
            .shell
            .session()
            .state()
            .pending_move_learn
            .as_ref()
            .map(|pending| &pending.learned_move.name),
        c.shell.battle_evolution_cancellations.len(),
        c.shell.battle_messages.len(),
        c.shell
            .last_audio_events
            .iter()
            .filter(|line| line.starts_with("evolution exit "))
            .collect::<Vec<_>>(),
    );
    c = super::super::r10_gallery::continue_save(root, runtime, &mut c);
    assert_eq!(
        c.snapshot().expect("learned move after Continue").party,
        before
    );
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .pokedex
            .has_caught("BOOMRUFF")
    );
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .pokedex
            .has_caught("PRAIRONDO")
    );
}
