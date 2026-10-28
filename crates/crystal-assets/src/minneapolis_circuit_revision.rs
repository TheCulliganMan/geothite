//! A trainer between the two Electric locks, with a clue for each switch.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, ensure};
use serde_json::Value;

use crate::gym_design::{Gym, GymDesign};
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "minnesota-minneapolis-staged-circuit-v1";

/// Publishes a staged adjacent-switch circuit over the two-gym revision parent.
///
/// Existing battles, trainer numbers, badge permission, reward and indexed city
/// doors remain intact. Jossa moves into the chamber opened by the first lock.
/// Other maps and source audio stay unchanged. Installation grants no progress;
/// old in-room saves need review before migration.
///
/// # Errors
/// Returns an error for another parent, altered battle/binding contracts,
/// skipped trainer requirements, invisible/bypassable barriers, unsafe native
/// geometry, duplicate installation or invalid compiled payloads.
pub fn revise_minneapolis_circuit(
    base: &CompiledGamePack,
    historical_json: &str,
    candidate_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash
            == "04992dc854f2219bf330948aecbd8ba7b96a0bbee211c5faa4dd2043dd27982b"
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "current Mankato terrace parent required"
    );
    let mut historical: Value = serde_json::from_str(historical_json)?;
    let candidate: Value = serde_json::from_str(candidate_json)?;
    ensure!(
        candidate["gyms"].as_array().is_some_and(|g| g.len() == 1),
        "one Minneapolis revision required"
    );
    let old = &historical["gyms"][2];
    let new = &candidate["gyms"][0];
    for field in [
        "slot",
        "id",
        "constant",
        "city",
        "leader",
        "trainer_class",
        "leader_sprite",
        "leader_at",
        "badge_flag",
        "requires",
        "reward",
        "party",
        "dimensions_metatiles",
        "arrival_core",
        "exit_tiles_core",
        "guide_at",
        "leader_court_locked",
    ] {
        ensure!(
            old[field] == new[field],
            "third-gym contract changed: {field}"
        );
    }
    ensure!(
        new["trainers"].as_array().is_some_and(|t| t.len() == 2),
        "two original practice trainers required"
    );
    for index in 0..2 {
        for field in [
            "id",
            "name",
            "trainer_class",
            "sprite",
            "party",
            "optional",
            "requires",
        ] {
            ensure!(
                old["trainers"][index][field] == new["trainers"][index][field],
                "practice battle contract changed: {field}"
            );
        }
    }
    for field in ["sequence", "events", "requires"] {
        ensure!(
            old["switch_puzzle"][field] == new["switch_puzzle"][field],
            "original adjacent pair and second-stage requirements must survive: {field}"
        );
    }
    historical["gyms"][2] = new.clone();
    let design: GymDesign = serde_json::from_value(historical)?;
    design.validate()?;
    let gym = &design.gyms[2];
    ensure!(
        gym.id == "MnMinneapolisGym"
            && gym.trainers[0].at == [1, 13]
            && gym.trainers[1].at == [4, 6]
            && gym.barriers == BTreeSet::from(['A', 'B'])
            && gym.gates.len() == 2
            && gym.gates[0].at == [2, 4]
            && gym.gates[1].at == [2, 2],
        "a chamber between two visible charged barriers is required"
    );
    let puzzle = gym
        .switch_puzzle
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("adjacent switches required"))?;
    ensure!(
        puzzle.first_requires == ["EVENT_MN_GYM_3_TRAINER_1_DEFEATED"]
            && puzzle.controls.len() == 6
            && puzzle
                .controls
                .iter()
                .map(|c| c.at)
                .collect::<BTreeSet<_>>()
                == BTreeSet::from([[3, 11], [5, 11], [7, 11], [3, 13], [5, 13], [7, 13]]),
        "six accessible cans and a Lenvi-powered first circuit required"
    );
    let crate::gym_room_revision::RebuiltGym {
        mut pack,
        room,
        symbols,
    } = crate::gym_room_revision::rebuild(base, gym)?;
    crate::gym_publication::validate(&pack, &room, gym, &symbols)?;
    validate_stages(&pack, &room, gym, &symbols)?;
    crate::authored_town_publication::replace(&mut pack, room)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn validate_stages(
    pack: &CompiledGamePack,
    room: &MapModule,
    gym: &Gym,
    symbols: &BTreeMap<char, u16>,
) -> Result<()> {
    let occupied = room.objects.iter().map(|o| [o.x, o.y]).collect();
    let adjacent = |reached: &BTreeSet<[u16; 2]>, at: [u16; 2]| {
        reached
            .iter()
            .any(|p| p[0].abs_diff(at[0]) + p[1].abs_diff(at[1]) == 1)
    };
    for (lenvi, first, jossa, second) in [
        (false, false, false, false),
        (true, false, false, false),
        (true, true, false, false),
        (true, true, true, false),
        (true, false, true, false),
        (true, true, true, true),
    ] {
        let mut flags = BTreeSet::new();
        for (set, flag) in [
            (lenvi, "EVENT_MN_GYM_3_TRAINER_1_DEFEATED"),
            (first, "EVENT_MN_GYM_3_FIRST_SWITCH_ON"),
            (jossa, "EVENT_MN_GYM_3_TRAINER_2_DEFEATED"),
            (second, "EVENT_MN_GYM_3_SECOND_SWITCH_ON"),
        ] {
            if set {
                flags.insert(flag.to_owned());
            }
        }
        let mut state = room.clone();
        for gate in &gym.gates {
            if gate.requires.iter().all(|r| flags.contains(r)) {
                state.blocks[usize::from(gate.at[1]) * usize::from(room.attributes.width)
                    + usize::from(gate.at[0])] = symbols[&gate.open_symbol];
            }
        }
        let reached = crate::gym_lifts::reachable(pack, &state, gym, &occupied)?;
        ensure!(
            adjacent(&reached, gym.trainers[0].at),
            "first trainer must remain reachable"
        );
        ensure!(
            adjacent(&reached, gym.trainers[1].at) == (lenvi && first),
            "Jossa must require the first win and first switch"
        );
        ensure!(
            adjacent(&reached, gym.leader_at) == (lenvi && first && jossa && second),
            "Avrin must require both trainer/switch stages"
        );
        ensure!(
            gym.switch_puzzle
                .as_ref()
                .is_some_and(|p| p.controls.iter().all(|c| adjacent(&reached, c.at))),
            "all six cans must remain accessible after reset"
        );
        ensure!(
            gym.exit_tiles_core
                .iter()
                .all(|[x, y]| y.checked_sub(1).is_some_and(|y| reached.contains(&[*x, y]))),
            "all stages need both ordinary exits"
        );
    }
    Ok(())
}
