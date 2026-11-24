//! Publish hand-authored cave layouts on the selected wall-finish pack.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, MapModule};

const PARENT: &str = "e9ec9900f13336921922a8b61fea912b3a6da0290164cc3f9c73c367b3503f2a";
const MANIFEST: &str = "northstar-cave-detail-v4";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Layout {
    map: String,
    rows: Vec<String>,
}

/// Compiles a complete set of authored Minnesota cave floor plans. Keeps the
/// selected pack's native events, warp positions, encounters and script state.
///
/// # Errors
/// Rejects another parent, missing layouts, invalid native terrain, and edits
/// to a scripted block, entrance, actor or cave boundary that invalidate its
/// authored terrain contract.
pub fn build_northstar_cave_detail(
    parent: &CompiledGamePack,
    sources: &[&str],
    art: &[&str],
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(parent)?;
    ensure!(
        parent.identity()?.content_hash == PARENT,
        "expected the selected cave wall-finish pack"
    );
    let mut remaining: BTreeSet<_> = parent
        .data
        .maps
        .iter()
        .filter_map(|(name, room)| {
            let attributes = &room.attributes;
            let cave = attributes.environment.as_deref() == Some("CAVE");
            let mine = attributes.environment.as_deref() == Some("DUNGEON")
                && attributes.tileset_name.starts_with("mn_iron_");
            (name.starts_with("Mn") && (cave || mine)).then_some(name.as_str())
        })
        .collect();
    let allowed_tilesets = remaining
        .iter()
        .map(|&name| parent.data.maps[name].attributes.tileset_name.as_str())
        .collect();
    let mut result = parent.clone();
    crate::northstar_cave_detail_art::install(&mut result, art, &allowed_tilesets)?;
    for source in sources {
        for layout in serde_json::from_str::<Vec<Layout>>(source)? {
            ensure!(
                remaining.remove(layout.map.as_str()),
                "unexpected or repeated cave layout {}",
                layout.map
            );
            install_room(&mut result, &layout)
                .with_context(|| format!("installing cave layout {}", layout.map))?;
        }
    }
    ensure!(remaining.is_empty(), "missing cave layouts: {remaining:?}");
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

fn install_room(pack: &mut CompiledGamePack, layout: &Layout) -> Result<()> {
    let original = &pack.data.maps[&layout.map];
    let [width, height] = [original.attributes.width, original.attributes.height];
    ensure!(layout.rows.len() == usize::from(height), "terrain height");
    let mut changed = original.clone();
    let mut bytes = Vec::with_capacity(original.blocks.len());
    for row in &layout.rows {
        let start = bytes.len();
        for block in row.split_whitespace() {
            bytes.push(u8::from_str_radix(block, 16)?);
        }
        ensure!(bytes.len() - start == usize::from(width), "terrain width");
    }
    changed.blocks = bytes.iter().copied().map(u16::from).collect();
    ensure!(changed.blocks != original.blocks, "no authored changes");
    crate::northstar_cave_overhaul::validate_native_art(pack, &changed)?;
    validate_anchors(pack, original, &changed)?;
    let label = original
        .attributes
        .blocks_label
        .as_ref()
        .context("block label")?;
    pack.data
        .map_blocks
        .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    pack.data.maps.insert(layout.map.clone(), changed);
    Ok(())
}

fn validate_anchors(
    pack: &CompiledGamePack,
    original: &MapModule,
    changed: &MapModule,
) -> Result<()> {
    let width = original.attributes.width;
    let height = original.attributes.height;
    let block_index =
        |[x, y]: [u16; 2]| usize::from(y / 2) * usize::from(width) + usize::from(x / 2);
    let retained_blocks = original
        .script_block_changes
        .iter()
        .map(|edit| [edit.x, edit.y]);
    for at in retained_blocks {
        let index = block_index(at);
        ensure!(
            original.blocks.get(index) == changed.blocks.get(index),
            "changed terrain written by a script at {at:?}"
        );
    }
    let foreground_anchors = original
        .events
        .warps
        .iter()
        .map(|warp| [warp.x, warp.y])
        .chain(
            original
                .events
                .bg_events
                .iter()
                .map(|event| [event.x, event.y]),
        )
        .chain(
            original
                .events
                .coord_events
                .iter()
                .map(|event| [event.x, event.y]),
        )
        .chain(original.objects.iter().map(|object| [object.x, object.y]));
    let horizontal = (0..width * 2).flat_map(|x| [[x, 0], [x, height * 2 - 1]]);
    let vertical = (0..height * 2).flat_map(|y| [[0, y], [width * 2 - 1, y]]);
    for at in foreground_anchors.chain(horizontal).chain(vertical) {
        ensure!(
            kind(pack, original, at)? == kind(pack, changed, at)?,
            "changed anchor or perimeter collision at {at:?}"
        );
    }
    Ok(())
}
