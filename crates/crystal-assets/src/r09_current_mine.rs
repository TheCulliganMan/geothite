//! Compose reviewed mine terrain while retaining the current regional Q06 story.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::authored_cave_revision;
use crate::authored_route_design::Interior;
use crate::authored_route_modpack;
use crate::r09_mine_revision;

const MINE: &str = "MnR09Cave1F";
const TRAINER: &str = "MN_LODE_FOREMAN";
const PARENT: &str = "4d826107eede5bd551d45987dbd107460a474e152b6a08707974047372a07168";
const PARENT_FILE: &str = "a69aa145e6ea870b459d79859958806170b56036af4f85e0c1e007629a2f67d6";
const PREVIOUS_SOURCE: &str = "7e0ead4bdd6aae39029618db5e9bfe504a32b127c9739a2c93233134bbe3f76c";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    schema_version: u16,
    id: String,
    parent_sha256: String,
    parent_identity: String,
    module_sha256: String,
    trainer_sha256: String,
    previous_source_sha256: String,
    revised_source_sha256: String,
}

/// Revises the current regional mine without reinstalling its historical story.
///
/// Requires the exact reviewed R10-parent identity, original source, cached mine,
/// foreman team and complete uncached reconstruction. Only the mine's blocks,
/// solid rock border, their mirrors and derived identity change. Current faction
/// dialogue, regional team and moves, prerequisites, cache acknowledgment and
/// all unrelated maps/programs/species/art/audio remain exact. The historical
/// source party is retained as source provenance, never installed over the
/// current regional party. This does not establish earned access or visual
/// acceptance; inspect and exercise the written candidate separately.
///
/// # Errors
/// Rejects an unreviewed contract, different or inconsistent parent, source or
/// native preimage drift, incomplete split payloads, shared block ownership,
/// unsafe cave geometry, blocked story approaches or unrelated output changes.
/// Borrowed inputs are unchanged on success and failure.
pub fn build_current_r09_mine_pack(
    base: &CompiledGamePack,
    previous_source: &str,
    revised_source: &str,
    contract_json: &str,
    reviewed_sha256: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(contract_json.as_bytes()) == reviewed_sha256,
        "reviewed current-mine contract differs"
    );
    let plan: Contract = serde_json::from_str(contract_json)?;
    ensure!(
        plan.schema_version == 1
            && plan.id == "CURRENT_REGIONAL_R09_V1"
            && plan.parent_sha256 == PARENT_FILE
            && plan.parent_identity == PARENT
            && base.identity.content_hash == PARENT
            && base.data.maps.len() == 691,
        "exact current R10 parent required"
    );
    ensure!(
        plan.previous_source_sha256 == PREVIOUS_SOURCE
            && hash(previous_source.as_bytes()) == PREVIOUS_SOURCE
            && hash(revised_source.as_bytes()) == plan.revised_source_sha256,
        "current-mine source preimage differs"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        crate::derive_compiled_game_pack_identity_from_manifest(
            base.format_version,
            &base.data,
            &base.audio_manifest,
            &base.runtime_files,
            &base.report,
        )? == base.identity,
        "current-mine parent identity is inconsistent"
    );
    let mut old_raw: Value = serde_json::from_str(previous_source)?;
    let mut new_raw: Value = serde_json::from_str(revised_source)?;
    let old: Interior = serde_json::from_value(old_raw.clone())?;
    let new: Interior = serde_json::from_value(new_raw.clone())?;
    let approach_source = new_raw.clone();
    for raw in [&mut old_raw, &mut new_raw] {
        let fields = raw.as_object_mut().context("current-mine room source")?;
        for key in [
            "rows",
            "border_block",
            "purpose",
            "candidate_status",
            "review_contract",
        ] {
            fields.remove(key);
        }
    }
    ensure!(old_raw == new_raw, "current-mine retained source drift");
    let original = base.data.maps.get(MINE).context("current regional mine")?;
    let team = base
        .data
        .trainers
        .trainers
        .get(TRAINER)
        .context("current foreman")?;
    ensure!(
        hash_json(original)? == plan.module_sha256
            && hash_json(team)? == plan.trainer_sha256
            && team
                .party
                .iter()
                .map(|p| (p.species.as_str(), p.level))
                .collect::<Vec<_>>()
                == [("BANDRAKE", 35), ("HELIKNOCK", 35), ("MESABLAST", 36)],
        "current regional mine/program/team preimage differs"
    );
    let attributes = &original.attributes;
    ensure!(
        old.id == MINE
            && new.id == MINE
            && attributes.tileset_name == "cave"
            && [attributes.width, attributes.height] == [12, 11]
            && attributes.time_of_day.as_deref() == Some("nite")
            && attributes.palette.as_deref() == Some("PALETTE_NITE")
            && base.data.map_attributes.get(MINE) == Some(attributes)
            && original.blocks == authored_cave_revision::blocks(&old)?,
        "current mine geometry or attribute preimage differs"
    );
    let blocks_label = attributes.blocks_label.as_deref().context("mine blocks")?;
    let scripts_label = attributes
        .map_scripts_label
        .as_deref()
        .context("mine scripts")?;
    let events_label = attributes
        .map_events_label
        .as_deref()
        .context("mine events")?;
    ensure!(
        base.data.maps.iter().all(|(name, module)| name == MINE
            || module.attributes.blocks_label.as_deref() != Some(blocks_label))
            && crate::decode_base64_bytes(&base.data.map_blocks[blocks_label])?
                .into_iter()
                .map(u16::from)
                .eq(original.blocks.iter().copied()),
        "mine blocks are shared or disagree with their mirror"
    );
    r09_mine_revision::certify_uncached(base, original, scripts_label, events_label)?;
    let mut revised = original.clone();
    revised.blocks = authored_cave_revision::blocks(&new)?;
    revised.attributes.border_block =
        authored_cave_revision::revised_border(base, original, &old, &new)?;
    ensure!(
        revised.blocks != original.blocks,
        "current mine terrain is unchanged"
    );
    authored_route_modpack::validate_room(base, &revised, &new)?;
    authored_cave_revision::validate_terrain(base, &revised, &new)?;
    r09_mine_revision::validate_approaches(base, &revised, &approach_source)?;
    let bytes = revised
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut result = base.clone();
    result
        .data
        .map_blocks
        .insert(blocks_label.into(), crate::encode_base64_bytes(&bytes));
    result
        .data
        .map_attributes
        .insert(MINE.into(), revised.attributes.clone());
    result.data.maps.insert(MINE.into(), revised.clone());
    r09_mine_revision::certify_uncached(&result, &revised, scripts_label, events_label)?;
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    let mut normalized = result.clone();
    normalized.identity = base.identity.clone();
    normalized.data.maps.insert(MINE.into(), original.clone());
    normalized
        .data
        .map_attributes
        .insert(MINE.into(), attributes.clone());
    normalized.data.map_blocks.insert(
        blocks_label.into(),
        base.data.map_blocks[blocks_label].clone(),
    );
    ensure!(
        normalized == *base,
        "mine revision changes unrelated content or current story"
    );
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    crate::validate_compact_superior_geography(&result)?;
    Ok(result)
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hash_json(value: &impl serde::Serialize) -> Result<String> {
    Ok(hash(&serde_json::to_vec(&serde_json::to_value(value)?)?))
}
