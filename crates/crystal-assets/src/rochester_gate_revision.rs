//! Replace the rejected first-gym maze with two visibly switched lanes.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, ensure};
use serde_json::Value;

use crate::grand_marais_modpack::authoring;
use crate::gym_design::{Gym, GymDesign};
use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnRochesterGym";
const MANIFEST: &str = "minnesota-rochester-sliding-gates-v1";

/// Publishes the smaller two-switch Rochester course over its reviewed parent.
///
/// Original trainer parties/numbers, badge, reward, parcel check, city warp
/// destinations, other maps and audio remain intact. Only this room's private
/// terrain art, scripts, dimensions and local event coordinates change.
/// Installation grants no progress. Old in-room saves need position migration.
///
/// # Errors
/// Returns an error for an unsupported parent, changed battle contract, shared
/// owned art, unsafe geometry, repeated installation or invalid native payloads.
/// Inputs remain unchanged on failure.
pub fn revise_rochester_gates(
    base: &CompiledGamePack,
    historical_json: &str,
    candidate_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash
            == "471572f0929249c5c924a35e4b260bfafda96c53d31ceeb24cf67edf43e3cdef"
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "the reviewed Rochester maze parent is required"
    );
    let mut historical: Value = serde_json::from_str(historical_json)?;
    let candidate: Value = serde_json::from_str(candidate_json)?;
    ensure!(
        candidate["gyms"].as_array().is_some_and(|g| g.len() == 1),
        "one complete Rochester revision required"
    );
    let old = &historical["gyms"][0];
    let new = &candidate["gyms"][0];
    for field in [
        "slot",
        "id",
        "constant",
        "city",
        "leader",
        "trainer_class",
        "leader_sprite",
        "badge_flag",
        "requires",
        "reward",
        "party",
    ] {
        ensure!(
            new[field] == old[field],
            "first-gym contract changed: {field}"
        );
    }
    ensure!(
        new["trainers"].as_array().is_some_and(|t| t.len() == 2),
        "two original trainers required"
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
                new["trainers"][index][field] == old["trainers"][index][field],
                "practice battle contract changed: {field}"
            );
        }
    }
    historical["gyms"][0] = new.clone();
    let design: GymDesign = serde_json::from_value(historical)?;
    design.validate()?;
    let gym = &design.gyms[0];
    ensure!(
        gym.id == MAP && gym.routers.len() == 2,
        "two-switch first gym required"
    );
    let crate::gym_room_revision::RebuiltGym {
        mut pack,
        mut room,
        symbols,
    } = crate::gym_room_revision::rebuild(base, gym)?;
    authoring::text(
        &mut room,
        "MnRochesterGymRequirementsText",
        &["Deliver the parcel", "in ROCHESTER first."],
    );
    crate::gym_scripts::finish(&pack, &mut room)?;
    crate::gym_publication::validate(&pack, &room, gym, &symbols)?;
    validate_lane_choices(&pack, &room, gym, &symbols)?;
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

fn validate_lane_choices(
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
    for lane in [None, Some("LEFT"), Some("RIGHT")] {
        for upper_open in [false, true] {
            let mut flags = gym
                .trainers
                .iter()
                .map(|t| format!("EVENT_{}_DEFEATED", t.id))
                .collect::<BTreeSet<_>>();
            if let Some(lane) = lane {
                flags.insert(format!("EVENT_MN_GYM_1_LOWER_{lane}"));
            }
            if upper_open {
                flags.insert("EVENT_MN_GYM_1_UPPER_OPEN".into());
            }
            let mut state = room.clone();
            for gate in &gym.gates {
                if gate.requires.iter().all(|r| flags.contains(r)) {
                    let index = usize::from(gate.at[1]) * usize::from(room.attributes.width)
                        + usize::from(gate.at[0]);
                    state.blocks[index] = symbols[&gate.open_symbol];
                }
            }
            let reached = crate::gym_lifts::reachable(pack, &state, gym, &occupied)?;
            ensure!(
                adjacent(&reached, gym.routers[0].at),
                "lower switch must always be reachable"
            );
            ensure!(
                adjacent(&reached, gym.trainers[1].at) == (lane == Some("LEFT"))
                    && adjacent(&reached, gym.routers[1].at) == (lane == Some("LEFT")),
                "only the left lane may reach Elvik and the upper switch"
            );
            ensure!(
                adjacent(&reached, gym.leader_at) == (lane == Some("RIGHT") && upper_open),
                "the court requires an open upper gate and a return to the right lane"
            );
            ensure!(
                gym.exit_tiles_core
                    .iter()
                    .all(|[x, y]| y.checked_sub(1).is_some_and(|y| reached.contains(&[*x, y]))),
                "every lane configuration needs both ordinary returns"
            );
        }
    }
    Ok(())
}
