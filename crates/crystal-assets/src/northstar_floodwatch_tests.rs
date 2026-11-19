//! Native stepping, sight dispatch and fixture battle setup for the three prairie runners.
use std::collections::BTreeSet;

use anyhow::{Context, Result, bail, ensure};
use crystal_core::battle::start::{
    TrainerBattleStartStatus, advance_active_trainer_battle,
    claim_active_trainer_battle_reward_index,
};
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::random::ReplayDivider;
use crystal_core::state::{BattleMemory, GameState};
use crystal_core::world::map::{Direction, TilePosition};

use crate::RuntimeMapTrainerInteractionOutcome;
use crate::read_verified_compiled_game_pack;

#[test]
#[ignore = "requires a compiled Floodwatch edition in MN_FLOODWATCH_PACK"]
fn floodwatch_trainer_sights_start_the_authored_parties_and_stop_after_victory() -> Result<()> {
    let path = std::env::var_os("MN_FLOODWATCH_PACK").context("MN_FLOODWATCH_PACK")?;
    let pack = read_verified_compiled_game_pack(path)?;
    let data = pack.data();
    let mothlight = data.maps.contains_key("MnMothlightLodge");
    let lyle = if mothlight {
        &["JIGLET", "CACHEEK", "RONDIVE"][..]
    } else {
        &["JIGLET", "CACHEEK"]
    };
    let june = if mothlight {
        &["ZAPSAP", "TOTLIT", "RUMBLEHIDE"][..]
    } else {
        &["ZAPSAP", "TOTLIT"]
    };
    let cases = [
        (
            "MnR16C00R00",
            (11, 16),
            Direction::Right,
            GameButton::Right,
            (12, 16),
            "MN_R16_T01",
            "LYLE",
            "EVENT_MN_R16_TRAINER_01",
            lyle,
        ),
        (
            "MnR16C00R00",
            (39, 38),
            Direction::Down,
            GameButton::Down,
            (39, 39),
            "MN_R16_T04",
            "JUNE",
            "EVENT_MN_R16_TRAINER_04",
            june,
        ),
        (
            "MnR16C01R00",
            (26, 17),
            Direction::Right,
            GameButton::Right,
            (27, 17),
            "MN_R16_T09",
            "FINCH",
            "EVENT_MN_R16_TRAINER_09",
            &["TAPDUCK", "VANECHICK", "BOOMRUFF"],
        ),
    ];

    let mut state = GameState::reset_wram_for_new_game();
    state.storage.party.pokemon[0] = Some(data.create_pokemon("PIKELING", 25, Dv::default())?);
    state.sync_party_from_storage();
    for (room, (x, y), direction, button, (sx, sy), id, name, win, lineup) in cases {
        let mut session = data.overworld_session(room, TilePosition::new(x, y), 0)?;
        session.player.facing = direction;
        ensure!(
            data.check_trainer_sight_after_step(&state, &session)?
                .is_none(),
            "{name} saw past the lane"
        );
        let step = data.apply_overworld_input(
            &mut state,
            &mut session,
            [button],
            &BTreeSet::new(),
            &mut ReplayDivider::new([1; 64]),
        )?;
        let sight = step
            .trainer_sight
            .with_context(|| format!("{name} missed the player"))?;
        ensure!(
            session.player.tile == TilePosition::new(sx, sy) && sight.script == id,
            "expected {name} at ({sx},{sy}), got {sight:?}"
        );
        let status =
            data.resolve_map_trainer_interaction(&mut state, room, room, &sight.script, 0, false)?;
        let RuntimeMapTrainerInteractionOutcome::BattleStarted(TrainerBattleStartStatus::Started(
            start,
        )) = status
        else {
            bail!("{name} did not start the authored encounter")
        };
        ensure!(
            start.trainer_name == name && start.trainer_id == id && start.event_flag == win,
            "{name} lost its native battle binding"
        );
        ensure!(
            start
                .enemy_party
                .iter()
                .map(|p| p.species.id.as_str())
                .eq(lineup.iter().copied()),
            "{name} materialized the wrong party"
        );
        ensure!(
            start
                .enemy_party
                .iter()
                .map(|p| p.level)
                .eq([31, 31, 30].into_iter().take(lineup.len())),
            "{name} materialized the wrong levels"
        );
        let authored = data.trainers.get(id).context("original trainer record")?;
        for (enemy, slot) in start.enemy_party.iter().zip(&authored.party) {
            if !slot.moves.is_empty() {
                ensure!(enemy.moves == slot.moves, "{name} lost its authored moves");
            }
            ensure!(!enemy.moves.is_empty(), "{name} has no battle moves");
        }
        // Each fixture knockout uses the same per-slot reward index and next-opponent
        // transition as a resolved battle. This test exercises sight and the victory receipt.
        for (slot, opponent) in start.enemy_party.iter().enumerate() {
            let BattleMemory::Trainer { enemy_pokemon, .. } = &mut state.battle else {
                bail!("{name}: no active opponent to finish")
            };
            ensure!(
                *enemy_pokemon == *opponent,
                "{name}: wrong active opponent at slot {slot}"
            );
            enemy_pokemon.hp = 0;
            ensure!(
                claim_active_trainer_battle_reward_index(&mut state)? == slot,
                "{name}: reward slot"
            );
            let advancement = advance_active_trainer_battle(&mut state)?;
            ensure!(
                advancement.trainer_defeated == (slot + 1 == lineup.len()),
                "{name}: premature victory"
            );
        }
        data.complete_scripted_trainer_battle(
            &mut state,
            room,
            room,
            id,
            0,
            true,
            false,
            &mut ReplayDivider::new([1; 64]),
        )?;
        ensure!(
            state.flags.is_event_flag_set(win)?
                && data
                    .check_trainer_sight_after_step(&state, &session)?
                    .is_none(),
            "{name} must record victory and stop challenging on sight"
        );
    }
    ensure!(
        !state
            .flags
            .is_event_flag_set("EVENT_MN_PRAIRIE_CIRCUIT_REWARDED")?,
        "a battle granted the helper's prize"
    );
    Ok(())
}
