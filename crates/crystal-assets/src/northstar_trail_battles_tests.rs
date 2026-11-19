use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_core::battle::start::TrainerBattleStartStatus;
use crystal_core::models::Dv;
use crystal_core::state::GameState;

use crate::northstar_regional_roster::RegionalRoster;

#[test]
#[ignore = "requires an external archive in MN_TRAIL_BATTLES_PACK"]
fn gym_prizes_and_city_parties_survive_trainer_scaling() -> Result<()> {
    let path = PathBuf::from(std::env::var_os("MN_TRAIL_BATTLES_PACK").context("campaign pack")?);
    let mut pack = crate::read_verified_compiled_game_pack(path)?;
    // Verify the authored regional rules through the normal battle entry point.
    check_gyms_and_cities(pack.data())?;
    // The optional scaling pack uses this same production entry point.
    if !pack.data.minnesota_trainer_scaling {
        pack.data.minnesota_trainer_scaling = true;
        check_gyms_and_cities(pack.data())?;
    }
    Ok(())
}

fn check_gyms_and_cities(data: &crate::GameDataSet) -> Result<()> {
    let roster = RegionalRoster::current()?;
    let mut state = GameState::default();
    let mut gyms = 0;
    for gym in data
        .maps
        .values()
        .filter(|m| m.id.starts_with("Mn") && m.id.ends_with("Gym"))
    {
        let Some(battle) = gym.scripted_trainer_battles.iter().find(|b| {
            b.request.trainer_id.starts_with("MN_GYM_") && b.source_script.ends_with("GymLeader")
        }) else {
            continue;
        };
        let trainer = &data.trainers.trainers[&battle.request.trainer_id];
        let reward = format!("{}Reward", battle.source_script);
        let prize = gym
            .script_item_grants
            .iter()
            .filter(|g| g.source_script == reward)
            .find_map(|g| data.items[&g.item_id].tmhm_move.as_deref())
            .context("badge prize")?;
        let ace = trainer
            .party
            .iter()
            .map(|p| p.level)
            .max()
            .context("gym team")?;
        for level in ace..=100 {
            state.storage.party.pokemon[0] =
                Some(data.create_pokemon("POUCHID", level, Dv::default())?);
            let TrainerBattleStartStatus::Started(start) =
                data.trainer_battle_start(&state, battle.request.clone())?
            else {
                anyhow::bail!("gym battle unavailable")
            };
            ensure!(
                start
                    .enemy_party
                    .iter()
                    .any(|p| p.moves.iter().any(|m| m.name == prize)),
                "{} at {level} lost {prize}",
                gym.id
            );
            ensure!(
                start
                    .enemy_party
                    .iter()
                    .all(|p| roster.allowed.contains(&p.species.id)
                        && p.moves.len() <= 4
                        && p.moves
                            .iter()
                            .all(|m| m.current_pp == data.moves[&m.name].pp)),
                "unexpected scaled gym party"
            );
        }
        gyms += 1;
    }
    ensure!(gyms == 8);
    for (city, count) in [("MnMinneapolis", 18), ("MnStateFairgrounds", 35)] {
        let room = &data.maps[city];
        let trainers: Vec<_> = room
            .objects
            .iter()
            .filter_map(|o| room.trainer_scripts.get(&o.script))
            .map(|s| &data.trainers.trainers[&s.trainer_id])
            .collect();
        ensure!(trainers.len() == count);
        let cap = if data.minnesota_trainer_scaling {
            100
        } else {
            1
        };
        for level in 1..=cap {
            let mut teams = BTreeSet::new();
            for t in &trainers {
                let ace = t.party.iter().map(|p| p.level).max().context("city team")?;
                let team = data.rematch_trainer_at_ace(t, level.max(ace))?;
                ensure!(
                    team.party
                        .iter()
                        .all(|p| roster.allowed.contains(&p.species)),
                    "inactive {city} partner"
                );
                let fingerprint: Vec<_> = team
                    .party
                    .into_iter()
                    .map(|p| {
                        (
                            p.species,
                            p.level,
                            p.moves.into_iter().map(|m| m.name).collect::<Vec<_>>(),
                        )
                    })
                    .collect();
                ensure!(
                    teams.insert(fingerprint),
                    "{city}: duplicate battle at {level}: {}",
                    t.name
                );
            }
        }
    }
    Ok(())
}
