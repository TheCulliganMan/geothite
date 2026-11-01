//! Explicit transition from the Rochester story pack to all eight reviewed gyms.
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::grand_marais_modpack::authoring;
use crate::gym_design::GymDesign;
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "minnesota-reviewed-eight-gyms-refresh-v1";
const BASE: &str = "7a136c478074fc03544d89770eb7cae4e9dc83d4e2c8f0fbec6383055ed297e9";
const DONOR: &str = "b94a53f5dbe3cf0737658a890025a46eb7747350eea574df3e8a1b54f96734bb";
const ROOMS: [&str; 8] = [
    "160c6bb9b03aa6baabfb4717d270a6866b2b6b49ebf1627c205921274f076f87",
    "d9914b17de2d131573778234e5f8e6c5c247e787d06e9452f628e7253685a850",
    "dbaf62641726b5a9f45a3d61ca6c1a6498024810d89be74518dfd29ff6a7269c",
    "95215c67f1fa16dc7baaf13cc480da9272e7f6689152ea216a7fb3c737378eb4",
    "4c0aae779e7fd5ed2dac92dce0a911b8dfc3ff7abee6443ec734d41ab15ba08d",
    "c85cb1967b29948462758774b24ae892816c631bae22a816fa83644eb2e96b32",
    "629bcb254a5b1eb2b8b8263dd7cd629ce5ce850d9b8f82cd34660ab10a34f289",
    "1e16fc8a72cff43af098f8f630e2ef9cece5df1c877f74e8c25a1098c0cc1f68",
];
const REPLACED: [u8; 6] = [2, 4, 5, 6, 7, 8];

/// Imports the captured reviewed gym generation into the Rochester story input.
///
/// Rochester and Minneapolis already contain their current rooms and remain
/// exact, including Rochester's city-specific delivery refusal. Six obsolete
/// gyms receive the reviewed modules/art/rosters, preserving their exterior
/// destinations and group/map identities. The review hall and authored city
/// name references are reconciled. Existing event numbers, unrelated trainers,
/// story scenes, source art/audio, HOME and recovery remain stable; installation
/// grants no progress. Old saves inside replaced gyms have no migration promise.
///
/// # Errors
/// Returns an error for unsupported input/donor/source generations, shared owned
/// assets or labels, invalid trainer/event allocation, unsafe bindings, missing
/// canonical Cut behavior or failed native runtime verification. Inputs remain
/// unchanged on failure.
pub fn refresh_installed_minnesota_gyms(
    base: &CompiledGamePack,
    reviewed: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_compiled_game_pack_for_runtime(reviewed)?;
    ensure!(
        base.identity.content_hash == BASE
            && reviewed.identity.content_hash == DONOR
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "exact Rochester story and reviewed eight-room generations required"
    );
    let design: GymDesign = serde_json::from_str(source)?;
    design.validate()?;
    ensure!(
        design.gyms.len() == ROOMS.len(),
        "complete eight-room canonical source required"
    );
    let raw: Value = serde_json::from_str(source)?;
    for (index, gym) in design.gyms.iter().enumerate() {
        ensure!(
            usize::from(gym.slot) == index + 1
                && format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&raw["gyms"][index])?)
                ) == ROOMS[index],
            "reviewed canonical source differs for slot{}",
            gym.slot
        );
    }
    let mut pack = base.clone();
    crate::gym_refresh_roster::import(&mut pack, base, reviewed, &REPLACED)?;
    let mut names = Vec::new();
    for gym in &design.gyms {
        let old = base.data.maps.get(&gym.id).context("installed owned gym")?;
        let donor = reviewed
            .data
            .maps
            .get(&gym.id)
            .context("reviewed owned gym")?;
        if !REPLACED.contains(&gym.slot) {
            validate_retained(base, reviewed, old, donor, gym.slot)?;
            continue;
        }
        ensure!(
            old.attributes.tileset_name == gym.tileset()
                && old.events.warps.len() >= 3
                && old.events.warps[..3].iter().all(|w| w.target_map_constant
                    == old.events.warps[0].target_map_constant
                    && w.target_warp_id == old.events.warps[0].target_warp_id)
                && !base
                    .data
                    .maps
                    .values()
                    .any(|m| m.id != gym.id && m.attributes.tileset_name == gym.tileset()),
            "owned gym art/bindings are shared or inconsistent"
        );
        let old_name = &base.data.trainers.trainers[&gym.battle_id()].name;
        names.push((old_name.clone(), gym.leader.clone()));
        let tileset = gym.tileset();
        pack.data.tilesets.insert(
            tileset.clone(),
            reviewed
                .data
                .tilesets
                .get(&tileset)
                .context("reviewed tileset")?
                .clone(),
        );
        for path in art_files(&tileset) {
            ensure!(
                base.runtime_files.contains_key(&path),
                "missing owned old art {path}"
            );
            pack.runtime_files.insert(
                path.clone(),
                reviewed
                    .runtime_files
                    .get(&path)
                    .context("reviewed owned art")?
                    .clone(),
            );
        }
        ensure!(
            !base
                .data
                .field_moves
                .whirlpool
                .replacements
                .contains_key(&tileset),
            "owned gym has unrelated whirlpool behavior"
        );
        if let Some(replacements) = reviewed.data.field_moves.cut.replacements.get(&tileset) {
            pack.data
                .field_moves
                .cut
                .replacements
                .insert(tileset, replacements.clone());
        } else {
            ensure!(
                !base
                    .data
                    .field_moves
                    .cut
                    .replacements
                    .contains_key(&tileset),
                "unexpected owned Cut behavior"
            );
        }
        let mut module = donor.clone();
        for (warp, previous) in module
            .events
            .warps
            .iter_mut()
            .take(3)
            .zip(&old.events.warps[..3])
        {
            warp.target_map_constant = previous.target_map_constant.clone();
            warp.target_map = previous.target_map.clone();
            warp.target_warp_id = previous.target_warp_id;
        }
        if old.events.warps[0].target_map_constant != "MN_GYM_REVIEW_HALL" {
            let town = base
                .data
                .maps
                .values()
                .find(|m| {
                    m.attributes.map_constant.as_deref()
                        == Some(&old.events.warps[0].target_map_constant)
                })
                .context("bound gym city")?;
            module.attributes.location = town.attributes.location.clone();
            pack.data.pokegear_landmarks.map_to_landmark.insert(
                gym.id.clone(),
                town.attributes.location.clone().context("city landmark")?,
            );
        }
        crate::gym_scripts::finish(&pack, &mut module)?;
        crate::authored_town_publication::replace(&mut pack, module)?;
    }
    let hall = reviewed
        .data
        .maps
        .get("MnGymReviewHall")
        .context("reviewed hall")?
        .clone();
    crate::authored_town_publication::replace(&mut pack, hall)?;
    crate::gym_refresh_names::reconcile(&mut pack, &names)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.report.maps = pack.data.maps.len();
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

fn validate_retained(
    base: &CompiledGamePack,
    donor: &CompiledGamePack,
    old: &MapModule,
    reviewed: &MapModule,
    slot: u8,
) -> Result<()> {
    let mut expected = reviewed.clone();
    for (warp, binding) in expected
        .events
        .warps
        .iter_mut()
        .take(3)
        .zip(&old.events.warps[..3])
    {
        warp.target_map_constant = binding.target_map_constant.clone();
        warp.target_map = binding.target_map.clone();
        warp.target_warp_id = binding.target_warp_id;
    }
    expected.attributes.location = old.attributes.location.clone();
    if slot == 1 {
        authoring::text(
            &mut expected,
            "MnRochesterGymRequirementsText",
            &["Visit the clinic.", "Take your parcel."],
        );
    }
    crate::gym_scripts::finish(base, &mut expected)?;
    ensure!(expected == *old, "retained canonical Gym{slot} differs");
    let tileset = &old.attributes.tileset_name;
    ensure!(
        base.data.tilesets.get(tileset) == donor.data.tilesets.get(tileset),
        "retained gym art definition differs"
    );
    for path in art_files(tileset) {
        ensure!(
            base.runtime_files.get(&path) == donor.runtime_files.get(&path),
            "retained gym art differs: {path}"
        );
    }
    for (id, trainer) in &base.data.trainers.trainers {
        if crate::gym_refresh_roster::owned(id, slot) {
            ensure!(
                donor.data.trainers.trainers.get(id) == Some(trainer),
                "retained trainer differs: {id}"
            );
        }
    }
    Ok(())
}

fn art_files(tileset: &str) -> [String; 5] {
    [
        format!("data/tilesets/{tileset}.json"),
        format!("data/tilesets/{tileset}_metatiles.bin"),
        format!("data/tilesets/{tileset}_palette_map.json"),
        format!("gfx/tilesets/{tileset}.2bpp"),
        format!("gfx/tilesets/{tileset}.png"),
    ]
}
