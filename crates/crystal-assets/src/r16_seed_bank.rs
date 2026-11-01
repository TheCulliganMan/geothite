//! Fit a short native prairie drop beside the existing seed grower.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, MapModule};

const SOURCE: &str = "2c0cb252df3534b42595afc6c547ad2fa72a867067727bb769b14148dba53bac";
const PARENT: &str = "6a9deafeeb226577012ec95864778686c0d047fccdccdbde9fcea986682bbbad";
const MARKER: &str = "minnesota-r16-seed-bank-v1";

#[derive(Deserialize)]
struct Proposal {
    schema_version: u8,
    route: String,
    dimensions_metatiles: [u16; 2],
    parent_content_hash: String,
    source_pixels_modified: bool,
    maps: Vec<Chunk>,
}

#[derive(Deserialize)]
struct Chunk {
    id: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    native_module_sha256: String,
    before_rows: Vec<String>,
    rows: Vec<String>,
    changed_metatiles: usize,
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn blocks(rows: &[String], [width, height]: [u16; 2]) -> Result<Vec<u16>> {
    ensure!(
        rows.len() == usize::from(height),
        "complete prairie rows required"
    );
    let mut blocks = Vec::with_capacity(usize::from(width) * usize::from(height));
    for row in rows {
        let row = row
            .split_whitespace()
            .map(|word| u8::from_str_radix(word, 16).map(u16::from))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(
            row.len() == usize::from(width),
            "complete prairie columns required"
        );
        blocks.extend(row);
    }
    Ok(blocks)
}

fn replace_blocks(pack: &mut CompiledGamePack, module: MapModule) -> Result<()> {
    let label = module
        .attributes
        .blocks_label
        .as_ref()
        .context("prairie blocks label")?;
    let bytes = module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    *pack
        .data
        .map_blocks
        .get_mut(label)
        .context("existing prairie block payload")? = crate::encode_base64_bytes(&bytes);
    pack.data.maps.insert(module.id.clone(), module);
    Ok(())
}

/// Publishes the reviewed seed-bank hop and yellow-grass return on R16.
///
/// Changes fifteen native metatiles and their encoded mirror in the western
/// chunk. Both chunks are preimage-bound. All water, regional seams, actors,
/// scripts, fishing tables, trainer parties, art, audio and attributes stay exact.
/// Returns an unselected candidate; existing saves are not migrated.
///
/// # Errors
/// Rejects source, parent, chunk or mirror drift, shared block labels, changed
/// shores, incidental Cut/ice, missing hop geometry or unrelated pack mutation.
/// Borrowed inputs remain unchanged on failure.
pub fn revise_r16_seed_bank(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    ensure!(
        hash(source.as_bytes()) == SOURCE,
        "reviewed prairie source differs"
    );
    let plan: Proposal = serde_json::from_str(source)?;
    ensure!(
        plan.schema_version == 1
            && plan.route == "MN_R16"
            && plan.dimensions_metatiles == [48, 24]
            && !plan.source_pixels_modified
            && plan.parent_content_hash == PARENT
            && base.identity.content_hash == PARENT
            && plan.maps.len() == 2
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "current complete prairie parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let mut child = base.clone();
    for (index, record) in plan.maps.iter().enumerate() {
        ensure!(
            record.id == format!("MnR16C{index:02}R00")
                && record.tileset == "mn_prairie_headwaters"
                && record.dimensions_metatiles == [if index == 0 { 32 } else { 16 }, 24],
            "complete native prairie chunk required"
        );
        let original = base
            .data
            .maps
            .get(&record.id)
            .context("installed prairie chunk")?;
        ensure!(
            hash(&serde_json::to_vec(&serde_json::to_value(original)?)?)
                == record.native_module_sha256
                && original.blocks == blocks(&record.before_rows, record.dimensions_metatiles)?
                && original.attributes.tileset_name == record.tileset
                && [original.attributes.width, original.attributes.height]
                    == record.dimensions_metatiles,
            "complete prairie preimage differs"
        );
        let label = original
            .attributes
            .blocks_label
            .as_ref()
            .context("prairie blocks")?;
        ensure!(
            base.data
                .maps
                .values()
                .filter(|m| m.attributes.blocks_label.as_ref() == Some(label))
                .count()
                == 1,
            "shared prairie blocks label"
        );
        let old_bytes = original
            .blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(
            base.data.map_blocks.get(label) == Some(&crate::encode_base64_bytes(&old_bytes)),
            "prairie encoded preimage differs"
        );
        let mut revised = original.clone();
        revised.blocks = blocks(&record.rows, record.dimensions_metatiles)?;
        let changed = original
            .blocks
            .iter()
            .zip(&revised.blocks)
            .enumerate()
            .filter_map(|(i, (a, b))| (a != b).then_some(i))
            .collect::<Vec<_>>();
        ensure!(
            record.changed_metatiles == if index == 0 { 15 } else { 0 }
                && changed.len() == record.changed_metatiles,
            "prairie terrain delta differs"
        );
        ensure!(
            changed
                .iter()
                .all(|i| (18..=25).contains(&(i % 32)) && (11..=16).contains(&(i / 32))),
            "protected prairie terrain changed"
        );
        if index == 0 {
            for x in [42, 43] {
                ensure!(
                    kind(base, &revised, [x, 25])? == "FLOOR"
                        && kind(base, &revised, [x, 26])? == "HOP_DOWN"
                        && kind(base, &revised, [x, 27])? == "WALL"
                        && kind(base, &revised, [x, 28])? == "FLOOR",
                    "unsafe seed-bank hop"
                );
            }
            for x in [41, 44] {
                ensure!(
                    kind(base, &revised, [x, 27])? == "HEADBUTT_TREE",
                    "open seed-bank side"
                );
            }
            replace_blocks(&mut child, revised)?;
        }
    }
    child.report.manifests.push(MARKER.into());
    child.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        child.format_version,
        &child.data,
        &child.audio_manifest,
        &child.runtime_files,
        &child.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&child)?;
    crate::validate_compact_superior_geography(&child)?;
    let mut residual = child.clone();
    replace_blocks(&mut residual, base.data.maps["MnR16C00R00"].clone())?;
    residual.identity = base.identity.clone();
    residual.report = base.report.clone();
    ensure!(residual == *base, "unrelated prairie content changed");
    Ok(child)
}
