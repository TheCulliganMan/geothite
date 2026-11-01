//! Readable directional signs on the existing native forced-wind course.
use anyhow::{Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::gym_design::GymDesign;

const MANIFEST: &str = "minnesota-st-cloud-arrow-wind-v1";

/// Marks each wind tile with a native chevron matching its forced direction.
///
/// Preserves the complete course, trainer battles/numbers, actor coordinates,
/// gate requirements, badge permission, reward and indexed city doors. Only
/// directional markings and the guide/trainer hints change. Other content and
/// source audio stay unchanged. Installation grants no progress.
///
/// # Errors
/// Returns an error for another parent, changed gameplay contracts, missing
/// markings or native source pixels, duplicate installation or invalid compiled
/// payloads. Existing saves require review before migration to a new pack.
pub fn revise_st_cloud_arrows(
    base: &CompiledGamePack,
    historical_json: &str,
    candidate_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash
            == "3983e786b3509fc0e2beddf5c4c03ac6a9809eabe93c57a037bbe90a1bd0a9eb"
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "current St. Paul pad-loop parent required"
    );
    let mut historical: Value = serde_json::from_str(historical_json)?;
    let candidate: Value = serde_json::from_str(candidate_json)?;
    ensure!(
        candidate["gyms"].as_array().is_some_and(|g| g.len() == 1),
        "one St. Cloud room required"
    );
    let new = &candidate["gyms"][0];
    let mut expected = historical["gyms"][4].clone();
    expected["wind_arrows"] = true.into();
    expected["guide"] = new["guide"].clone();
    for index in 0..2 {
        expected["trainers"][index]["after_lines"] = new["trainers"][index]["after_lines"].clone();
    }
    ensure!(
        expected == *new,
        "only native wind markings and directional hints may change"
    );
    historical["gyms"][4] = new.clone();
    let design: GymDesign = serde_json::from_value(historical)?;
    design.validate()?;
    let gym = &design.gyms[4];
    ensure!(
        gym.id == "MnStCloudGym" && gym.wind_arrows,
        "St. Cloud directional markings required"
    );
    let crate::gym_room_revision::RebuiltGym {
        mut pack,
        room,
        symbols,
    } = crate::gym_room_revision::rebuild(base, gym)?;
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
