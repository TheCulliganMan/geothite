//! Distinct native gold gates on Duluth's silver-stop ice course.
use anyhow::{Result, ensure};
use serde_json::{Value, json};

use crate::CompiledGamePack;
use crate::gym_design::GymDesign;

const MANIFEST: &str = "minnesota-duluth-gold-gates-v1";

/// Makes trainer-operated gates distinguishable from permanent ice stops.
///
/// Preserves both slide courses, every collision quadrant, actors, battles,
/// trainer numbers, gate conditions, badge permission, rewards and city doors.
/// Only four native gate recipes and their hints change. Other content and
/// audio remain intact. Installation grants no progress; existing saves need
/// migration review before switching packs.
///
/// # Errors
/// Returns an error for another parent, altered course or battle contracts,
/// missing gate distinctions, duplicate installation or invalid native payloads.
pub fn revise_duluth_gates(
    base: &CompiledGamePack,
    historical_json: &str,
    candidate_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash
            == "95c4cd685949c3334c7691b48e8088ace5212a9659727ce32a2713259291072c"
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "current Bemidji garden-return parent required"
    );
    let mut historical: Value = serde_json::from_str(historical_json)?;
    let candidate: Value = serde_json::from_str(candidate_json)?;
    ensure!(
        candidate["gyms"].as_array().is_some_and(|g| g.len() == 1),
        "one Duluth course required"
    );
    let new = &candidate["gyms"][0];
    let mut expected = historical["gyms"][7].clone();
    for symbol in ["X", "Y", "Z", "V"] {
        expected["art"][symbol] = json!(["underground", "0b"]);
    }
    expected["guide"] = new["guide"].clone();
    for index in 0..3 {
        expected["trainers"][index]["after_lines"] = new["trainers"][index]["after_lines"].clone();
    }
    ensure!(
        expected == *new,
        "only four gold gates and explanatory hints may change"
    );
    historical["gyms"][7] = new.clone();
    let design: GymDesign = serde_json::from_value(historical)?;
    design.validate()?;
    let gym = &design.gyms[7];
    let crate::gym_room_revision::RebuiltGym {
        mut pack,
        room,
        symbols,
    } = crate::gym_room_revision::rebuild(base, gym)?;
    ensure!(
        pack.data.tilesets[&gym.tileset()].collision
            == base.data.tilesets[&gym.tileset()].collision,
        "gold gates must retain all native collision quadrants"
    );
    crate::gym_publication::validate(&pack, &room, gym, &symbols)?;
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
