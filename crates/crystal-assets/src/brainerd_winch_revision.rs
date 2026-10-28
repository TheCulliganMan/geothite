//! A physical two-position dock control replaces the trainer's routing menu.
use std::collections::BTreeSet;

use anyhow::{Result, ensure};
use serde_json::{Value, json};

use crate::CompiledGamePack;
use crate::gym_design::GymDesign;

const MANIFEST: &str = "minnesota-brainerd-dock-winch-v1";

/// Publishes a visible dock switch that alternates the center and left piers.
///
/// All battles, actor positions, bridge conditions, badge permission, reward,
/// room extent and indexed city doors remain intact. A solid blue control
/// replaces the right entrance statue; the first trainer no longer offers a
/// route-selection menu. Other content and source audio stay unchanged.
/// Installation grants no progress; existing saves require migration review.
///
/// # Errors
/// Returns an error for another parent, changed gameplay contracts, an absent
/// or inaccessible physical control, invalid native geometry, repeated
/// installation or invalid compiled payloads.
pub fn revise_brainerd_winch(
    base: &CompiledGamePack,
    historical_json: &str,
    candidate_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash
            == "1aa55a37a6d67cb907ff438422e90ec6d6fb0a9f658a85b3101c8eb6a9a73189"
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "current St. Cloud arrow-lane parent required"
    );
    let mut historical: Value = serde_json::from_str(historical_json)?;
    let candidate: Value = serde_json::from_str(candidate_json)?;
    ensure!(
        candidate["gyms"].as_array().is_some_and(|g| g.len() == 1),
        "one Brainerd room required"
    );
    let new = &candidate["gyms"][0];
    let mut expected = historical["gyms"][5].clone();
    expected["rows"][6] = "lL.Sr".into();
    expected["art"]["S"] = json!(["elite_four_room", "37"]);
    expected["routers"][0]["operator"] = Value::Null;
    expected["routers"][0]["toggle"] = true.into();
    expected["routers"][0]["at"] = json!([6, 13]);
    expected["guide"] = new["guide"].clone();
    for index in 0..2 {
        expected["trainers"][index]["after_lines"] = new["trainers"][index]["after_lines"].clone();
    }
    ensure!(
        expected == *new,
        "only the visible two-position control and its hints may change"
    );
    historical["gyms"][5] = new.clone();
    let design: GymDesign = serde_json::from_value(historical)?;
    design.validate()?;
    let gym = &design.gyms[5];
    ensure!(
        gym.id == "MnBrainerdGym" && gym.routers.len() == 1,
        "one Brainerd dock control required"
    );
    let crate::gym_room_revision::RebuiltGym {
        mut pack,
        room,
        symbols,
    } = crate::gym_room_revision::rebuild(base, gym)?;
    crate::gym_publication::validate(&pack, &room, gym, &symbols)?;
    // In every meaningful mode, the control must be reachable from the foyer,
    // Bravel requires the center position, and the court requires both wins.
    let occupied = room.objects.iter().map(|o| [o.x, o.y]).collect();
    for (first, second, mode) in [
        (false, false, None),
        (true, false, None),
        (true, false, Some(0)),
        (true, false, Some(1)),
        (true, true, Some(0)),
        (true, true, Some(1)),
    ] {
        let mut flags = BTreeSet::new();
        if first {
            flags.insert("EVENT_MN_GYM_6_TRAINER_1_DEFEATED".to_owned());
        }
        if second {
            flags.insert("EVENT_MN_GYM_6_TRAINER_2_DEFEATED".to_owned());
        }
        if let Some(index) = mode {
            flags.insert(gym.routers[0].modes[index].event.clone());
        }
        let mut phase = room.clone();
        for gate in &gym.gates {
            if gate.requires.iter().all(|r| flags.contains(r)) {
                phase.blocks[usize::from(gate.at[1]) * usize::from(room.attributes.width)
                    + usize::from(gate.at[0])] = symbols[&gate.open_symbol];
            }
        }
        let reached = crate::gym_lifts::reachable(&pack, &phase, gym, &occupied)?;
        let adjacent = |at: [u16; 2]| {
            reached
                .iter()
                .any(|p| p[0].abs_diff(at[0]) + p[1].abs_diff(at[1]) == 1)
        };
        ensure!(
            adjacent(gym.routers[0].at) && adjacent(gym.trainers[0].at),
            "the dock control and first trainer must remain reachable"
        );
        ensure!(
            adjacent(gym.trainers[1].at)
                == (first && (mode == Some(0) || second && mode == Some(1))),
            "the pier position must control the second trainer approach"
        );
        ensure!(
            adjacent(gym.leader_at) == (first && second && mode == Some(1)),
            "both wins and the left pier must gate Sivren"
        );
        ensure!(
            gym.exit_tiles_core
                .iter()
                .all(|[x, y]| y.checked_sub(1).is_some_and(|y| reached.contains(&[*x, y]))),
            "both ordinary exits must remain reachable"
        );
    }
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
