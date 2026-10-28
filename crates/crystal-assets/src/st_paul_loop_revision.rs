//! A four-room pad circuit with a direct onward route and a short return.
use anyhow::{Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::gym_design::GymDesign;

const MANIFEST: &str = "minnesota-st-paul-pad-loop-v1";

/// Reconnects St. Paul's four chambers into a continuous reciprocal pad loop.
///
/// Preserves all battles, actors, badge permission, reward, room dimensions and
/// the three indexed city doors. Internal pad coordinates change, while their
/// original record numbers and reciprocal destinations survive. Other content
/// and source audio stay unchanged. Installation grants no progress; existing
/// in-room saves require migration review.
///
/// # Errors
/// Returns an error for another parent, changed battle/binding contracts,
/// invalid or unpaired pads, a bypassed trainer gate, duplicate installation or
/// invalid compiled payloads.
pub fn revise_st_paul_loop(
    base: &CompiledGamePack,
    historical_json: &str,
    candidate_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash
            == "d3ffc94b129fee3e3e8d604a07f1c44b9401d64908ea244b4610c9ebeb2c6d31"
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "current Minneapolis staged-circuit parent required"
    );
    let mut historical: Value = serde_json::from_str(historical_json)?;
    let candidate: Value = serde_json::from_str(candidate_json)?;
    ensure!(
        candidate["gyms"].as_array().is_some_and(|g| g.len() == 1),
        "one St. Paul room required"
    );
    let old = &historical["gyms"][3];
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
        "gates",
        "walkthrough",
    ] {
        ensure!(
            old[field] == new[field],
            "fourth-gym contract changed: {field}"
        );
    }
    ensure!(
        new["trainers"].as_array().is_some_and(|t| t.len() == 2),
        "both original trainers required"
    );
    for index in 0..2 {
        for field in [
            "id",
            "name",
            "trainer_class",
            "sprite",
            "party",
            "at",
            "optional",
            "requires",
        ] {
            ensure!(
                old["trainers"][index][field] == new["trainers"][index][field],
                "practice contract changed: {field}"
            );
        }
    }
    historical["gyms"][3] = new.clone();
    let design: GymDesign = serde_json::from_value(historical)?;
    design.validate()?;
    let gym = &design.gyms[3];
    ensure!(
        gym.id == "MnStPaulGym" && gym.lifts.len() == 4,
        "four reciprocal pad pairs required"
    );
    for (lift, (id, ends)) in gym.lifts.iter().zip([
        ("WEST_ROOM", [[1, 11], [1, 3]]),
        ("CROSS_ROOM", [[5, 7], [9, 11]]),
        ("EAST_RETURN", [[11, 17], [9, 7]]),
        ("MAIN_LIFT", [[5, 15], [11, 7]]),
    ]) {
        ensure!(
            lift.id == id && lift.ends_core == ends,
            "each chamber must connect to its next room and return"
        );
    }
    let crate::gym_room_revision::RebuiltGym {
        mut pack,
        room,
        symbols,
    } = crate::gym_room_revision::rebuild(base, gym)?;
    crate::gym_publication::validate(&pack, &room, gym, &symbols)?;
    // Native geometry must still seal the leader with either practice win missing.
    let occupied = room.objects.iter().map(|o| [o.x, o.y]).collect();
    for flags in [
        vec![],
        vec!["EVENT_MN_GYM_4_TRAINER_1_DEFEATED"],
        vec!["EVENT_MN_GYM_4_TRAINER_2_DEFEATED"],
        vec![
            "EVENT_MN_GYM_4_TRAINER_1_DEFEATED",
            "EVENT_MN_GYM_4_TRAINER_2_DEFEATED",
        ],
    ] {
        let mut phase = room.clone();
        for gate in &gym.gates {
            if gate.requires.iter().all(|r| flags.contains(&r.as_str())) {
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
            gym.trainers.iter().all(|t| adjacent(t.at)),
            "both practice rooms must be reachable in either order"
        );
        ensure!(
            adjacent(gym.leader_at) == (flags.len() == 2),
            "both wins must seal/open the actual leader approach"
        );
        ensure!(
            gym.exit_tiles_core
                .iter()
                .all(|[x, y]| y.checked_sub(1).is_some_and(|y| reached.contains(&[*x, y]))),
            "both carpets must remain reachable"
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
