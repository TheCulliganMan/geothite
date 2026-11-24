//! Finish cave ceilings and cliff faces while preserving native movement.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::northstar_cave_void_art;

const PARENT: &str = "be0f0dd31e1e18e373112c4aa14ab54eab3e8aae94f67f8f9b2bab6f99249677";
const MANIFEST: &str = "northstar-cave-wall-finish-v3";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoomEdges {
    map: String,
    edits: Vec<EdgeEdit>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EdgeEdit {
    at: [u16; 2],
    from: String,
    to: String,
}

/// Replaces the noisy cave ceiling fill with quiet native art and applies
/// individually authored cliff edits. Movement permissions agree in every
/// quadrant, including the off-map border and all retained script states.
///
/// # Errors
/// Rejects a stale parent, missing graphics, a moved terrain edit or any change
/// in the native collision of a tile used by the replacement.
pub fn build_northstar_cave_wall_finish(
    parent: &CompiledGamePack,
    sources: &[&str],
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(parent)?;
    ensure!(
        parent.identity()?.content_hash == PARENT,
        "expected the selected cave replacement pack"
    );
    let mut edges: BTreeMap<String, Vec<EdgeEdit>> = BTreeMap::new();
    for source in sources {
        for room in serde_json::from_str::<Vec<RoomEdges>>(source)? {
            edges.entry(room.map).or_default().extend(room.edits);
        }
    }
    let mut result = parent.clone();
    let quiet = northstar_cave_void_art::install(&mut result)?;
    let mut finished = 0;
    for (name, old) in &parent.data.maps {
        let tileset = &old.attributes.tileset_name;
        let cave = old.attributes.environment.as_deref() == Some("CAVE");
        let iron = tileset.starts_with("mn_iron_")
            && old.attributes.environment.as_deref() == Some("DUNGEON");
        if !name.starts_with("Mn") || !(cave || iron) {
            continue;
        }
        let replacement = u16::from(*quiet.get(tileset).context("quiet native cave ceiling")?);
        let background = if tileset == "ice_path" { 0x3b } else { 0x2e };
        let collision = &result.data.tilesets[tileset].collision;
        let same_collision = |before, after| {
            let a = collision.get(&format!("{before:02x}"));
            let b = collision.get(&format!("{after:02x}"));
            a.is_some() && a == b
        };
        ensure!(
            same_collision(background, replacement),
            "cave background changes movement in {name}"
        );
        ensure!(
            same_collision(u16::from(old.attributes.border_block), replacement),
            "cave border changes movement in {name}"
        );
        let mut room = old.clone();
        let width = room.attributes.width;
        let height = room.attributes.height;
        let index = |x, y| usize::from(y) * usize::from(width) + usize::from(x);
        let cues: std::collections::BTreeSet<_> = room
            .events
            .bg_events
            .iter()
            .map(|e| index(e.x / 2, e.y / 2))
            .chain(
                room.script_block_changes
                    .iter()
                    .map(|e| index(e.x / 2, e.y / 2)),
            )
            .collect();
        for (at, block) in room.blocks.iter_mut().enumerate() {
            if *block == background && !cues.contains(&at) {
                *block = replacement;
            }
        }
        for edit in edges.remove(name).unwrap_or_default() {
            let [x, y] = edit.at;
            ensure!(
                x < width && y < height && !cues.contains(&index(x, y)),
                "edge edit touches a cue or lies outside {name} at {x},{y}"
            );
            let from = u16::from_str_radix(&edit.from, 16)?;
            let to = u16::from_str_radix(&edit.to, 16)?;
            let block = &mut room.blocks[index(x, y)];
            ensure!(
                *block == from && same_collision(from, to),
                "edge collision or source drift in {name} at {x},{y}"
            );
            *block = to;
        }
        room.attributes.border_block = u8::try_from(replacement)?;
        crate::northstar_cave_overhaul::validate_native_art(&result, &room)?;
        let blocks = room
            .blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let label = room
            .attributes
            .blocks_label
            .as_ref()
            .context("cave block label")?;
        result
            .data
            .map_blocks
            .insert(label.clone(), crate::encode_base64_bytes(&blocks));
        result
            .data
            .map_attributes
            .insert(name.clone(), room.attributes.clone());
        result.data.maps.insert(name.clone(), room);
        finished += 1;
    }
    ensure!(
        finished == 19 && edges.is_empty(),
        "wall finish must cover the selected 19 interiors"
    );
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
