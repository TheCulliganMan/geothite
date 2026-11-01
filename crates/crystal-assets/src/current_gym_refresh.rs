//! Rebuild the eight selected gyms inside the current story and city branches.
use anyhow::{Result, ensure};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::gym_design::GymDesign;

const MANIFEST: &str = "minnesota-current-eight-gyms-v1";
const PARENTS: [&str; 2] = [
    "e96cd37fd94ca71f39d72a4a7924fe613ddb1cecf64f12f5a5985095a5fea7d4",
    "23b3ca0c19fbc6c7f687645540aa9bdd12becfbdb7b693804d6f3b125a41ed41",
];
const SOURCES: [&str; 8] = [
    "6945fa19fd82c4afca1d04dc88fc5fca0aa61b5559e86bb0540dc3520e87e818",
    "5c446a797c820e6aea2e3565d22a902e530fe0acfce503b7f37b13386330a758",
    "d1306fa4f5a67af4cf876695d4e00fa408a54d9a886f264b90aeb577a7c02b54",
    "515e6534b759b1bc780994e4ac4229140980be3c9a967e75144aaecfc7ac3d22",
    "64d0d8809461f930cfd936d08435761c623404dbc47f55fe425f4231480649ce",
    "fc505d66f5fad956399b0c6d67a51a6953c1170bea97a097ea3f82ab7d508ff5",
    "1ff8351fc9ee98d25622747f8db03d3b55bd771d95fc3fdfcd7fcb532cf91c8c",
    "035ebb18c6eeb315d372369165baf0e62beb7c6bd00d6a16cdf76cb0d8f6b6a9",
];

/// Installs all eight current gym revisions into a supported regional parent.
///
/// Supports the frozen 554-map Ely story and 660-map compact Twin Cities packs.
/// Recompiles selected source rows, art and native scripts while retaining each
/// city's indexed doors, room identities and original trainer rosters/numbers.
/// Existing flags retain their numbers; new gym flags are allocated afterward.
/// Every other map, story scene, service, recovery record and audio stays intact.
/// Installation grants no progress. Existing saves inside revised rooms need
/// separate position/block-override migration review before switching packs.
///
/// # Errors
/// Returns an error for unsupported or repeated installation, another source
/// generation, shared private art, incompatible owned Cut/warp data, unsafe
/// room geometry or invalid compiled payloads. Inputs remain unchanged on error.
pub fn refresh_current_minnesota_gyms(
    base: &CompiledGamePack,
    selected_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        PARENTS.contains(&base.identity.content_hash.as_str())
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "supported Ely story or compact Twin Cities parent required"
    );
    let raw: Value = serde_json::from_str(selected_json)?;
    let design: GymDesign = serde_json::from_value(raw.clone())?;
    design.validate()?;
    for (index, _) in design.gyms.iter().enumerate() {
        ensure!(
            format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&raw["gyms"][index])?)
            ) == SOURCES[index],
            "current reviewed gym{} source differs",
            index + 1
        );
    }
    let mut result = base.clone();
    for gym in &design.gyms {
        let crate::gym_room_revision::RebuiltGym {
            mut pack,
            mut room,
            symbols,
        } = crate::gym_room_revision::rebuild(&result, gym)?;
        if gym.slot == 1 {
            authoring::text(
                &mut room,
                "MnRochesterGymRequirementsText",
                &["Deliver the parcel", "in ROCHESTER first."],
            );
            crate::gym_scripts::finish(&pack, &mut room)?;
        }
        for coordinate in &mut room.events.coord_events {
            if coordinate.scene_id.is_empty() {
                coordinate.scene_id = "-1".into();
            }
        }
        crate::gym_publication::validate(&pack, &room, gym, &symbols)?;
        crate::authored_town_publication::replace(&mut pack, room)?;
        result = pack;
    }
    result.report.manifests.push(MANIFEST.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}
