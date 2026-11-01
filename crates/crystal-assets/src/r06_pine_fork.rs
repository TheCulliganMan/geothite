//! Add a native downhill choice to the Pine Country lakeside fork.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, MapModule};

const SOURCE: &str = "28a41698794d63140e17326b2f0c52e2e1bbc137e6fd6733278ae68089581c48";
const PARENT: &str = "408b2944618db7e6ba1497e29c6e7f82b9d6f5e93eeb3c65f5995aeaddea5d60";
const MARKER: &str = "minnesota-r06-pine-fork-v1";

#[derive(Deserialize)]
struct Proposal {
    schema_version: u8,
    route: String,
    dimensions_metatiles: [u16; 2],
    parent_content_hash: String,
    source_pixels_modified: bool,
    maps: Vec<Chunk>,
    cliff_repairs: Vec<CliffRepair>,
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

#[derive(Deserialize)]
struct CliffRepair {
    map: String,
    metatile: [usize; 2],
    before: String,
    after: String,
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn blocks(rows: &[String], [width, height]: [u16; 2]) -> Result<Vec<u16>> {
    ensure!(
        rows.len() == usize::from(height),
        "complete terrace rows required"
    );
    let mut blocks = Vec::with_capacity(usize::from(width) * usize::from(height));
    for row in rows {
        let row = row
            .split_whitespace()
            .map(|word| u8::from_str_radix(word, 16).map(u16::from))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(
            row.len() == usize::from(width),
            "complete terrace columns required"
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
        .context("terrace blocks label")?;
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
        .context("existing terrace block payload")? = crate::encode_base64_bytes(&bytes);
    pack.data.maps.insert(module.id.clone(), module);
    Ok(())
}

/// Publishes the Pine Country fork's native drop, grassy returns and lookout.
///
/// Changes 39 northern-chunk metatiles and their encoded mirror. The complete
/// route is bound to its reviewed parent. Both original cabins, the Cut gate,
/// stairs, fishing pier, bridges, external roads, actors, programs, regional
/// fauna, source art, audio and all other maps stay exact. Returns an unselected
/// candidate without migrating saved progress.
///
/// # Errors
/// Rejects source or parent drift, incomplete native preimages, shared blocks,
/// mirror drift, edits outside the owned terrain, unsafe ledge geometry and
/// unrelated mutation. Borrowed inputs remain unchanged on failure.
pub fn revise_r06_pine_fork(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    ensure!(
        hash(source.as_bytes()) == SOURCE,
        "reviewed terrace source differs"
    );
    let plan: Proposal = serde_json::from_str(source)?;
    ensure!(
        plan.schema_version == 1
            && plan.route == "MN_R06"
            && plan.dimensions_metatiles == [32, 44]
            && !plan.source_pixels_modified
            && plan.parent_content_hash == PARENT
            && base.identity.content_hash == PARENT
            && plan.maps.len() == 2
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "current complete Pine Country parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let mut child = base.clone();
    for (row, record) in plan.maps.iter().enumerate() {
        let tileset = if row == 1 {
            "mn_headwaters"
        } else {
            "mn_outdoor_headwaters_stairs"
        };
        ensure!(
            record.id == format!("MnR06C00R{row:02}")
                && record.tileset == tileset
                && record.dimensions_metatiles == [32, if row == 1 { 20 } else { 24 }],
            "complete native Pine Country chunk required"
        );
        let original = base
            .data
            .maps
            .get(&record.id)
            .context("installed terrace chunk")?;
        ensure!(
            hash(&serde_json::to_vec(&serde_json::to_value(original)?)?)
                == record.native_module_sha256
                && original.blocks == blocks(&record.before_rows, record.dimensions_metatiles)?
                && original.attributes.tileset_name == record.tileset
                && [original.attributes.width, original.attributes.height]
                    == record.dimensions_metatiles,
            "complete terrace preimage differs"
        );
        let label = original
            .attributes
            .blocks_label
            .as_ref()
            .context("terrace blocks")?;
        ensure!(
            base.data
                .maps
                .values()
                .filter(|m| m.attributes.blocks_label.as_ref() == Some(label))
                .count()
                == 1,
            "shared terrace blocks label"
        );
        let bytes = original
            .blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(
            base.data.map_blocks.get(label) == Some(&crate::encode_base64_bytes(&bytes)),
            "terrace encoded preimage differs"
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
            record.changed_metatiles == [39, 0][row] && changed.len() == record.changed_metatiles,
            "terrace terrain delta differs"
        );
        for repair in plan
            .cliff_repairs
            .iter()
            .filter(|repair| repair.map == record.id)
        {
            let [x, y] = repair.metatile;
            ensure!(
                x < 32 && y < usize::from(record.dimensions_metatiles[1]),
                "cliff repair outside chunk"
            );
            let i = y * 32 + x;
            ensure!(
                original.blocks[i] == u16::from_str_radix(&repair.before, 16)?
                    && revised.blocks[i] == u16::from_str_radix(&repair.after, 16)?,
                "cliff repair preimage differs"
            );
        }
        ensure!(
            changed.iter().all(|&i| {
                let owned_cliff = plan
                    .cliff_repairs
                    .iter()
                    .any(|repair| repair.map == record.id && repair.metatile == [i % 32, i / 32]);
                owned_cliff
                    || (row == 0
                        && i % 32 == 21
                        && i / 32 == 13
                        && original.blocks[i] == 1
                        && revised.blocks[i] == 0x57)
                    || ((9..=27).contains(&(i % 32))
                        && (10..=16).contains(&(i / 32))
                        && matches!(original.blocks[i], 1..=4 | 0x71)
                        && matches!(revised.blocks[i], 1..=5))
            }),
            "protected terrace geometry changed"
        );
        if !changed.is_empty() {
            replace_blocks(&mut child, revised)?;
        }
    }
    let module = &child.data.maps["MnR06C00R00"];
    for x in [42, 43] {
        for (y, expected) in [(25, "FLOOR"), (26, "HOP_DOWN"), (27, "WALL"), (28, "FLOOR")] {
            ensure!(
                crate::authored_town_geometry::kind(&child, module, [x, y])? == expected,
                "unsafe Pine Country drop"
            );
        }
    }
    for x in [41, 44] {
        ensure!(
            crate::authored_town_geometry::kind(&child, module, [x, 27])? == "WALL",
            "open Pine Country ledge flank"
        );
    }
    ensure!(
        module.blocks[16 * 32 + 14] == 0x5b,
        "original Cut gate changed"
    );
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
    replace_blocks(&mut residual, base.data.maps["MnR06C00R00"].clone())?;
    residual.identity = base.identity.clone();
    residual.report = base.report.clone();
    ensure!(residual == *base, "unrelated Pine Country content changed");
    Ok(child)
}
