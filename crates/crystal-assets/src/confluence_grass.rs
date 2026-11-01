//! Grass on Reed's landing approach and a short east-bank southward return.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::authored_route_layout::Layout;
use crate::authored_route_surface::walking_map_distances;
use crate::{CompiledGamePack, MapModule};

const PREFIX: &str = "MnMississippiConfluence";
const MANIFEST: &str = "minnesota-confluence-grass-v1";
const PROFILES: [(&str, usize, usize, usize, usize); 4] = [
    ("MnMississippiConfluenceC00R00", 0, 4, 6, 8),
    ("MnMississippiConfluenceC01R00", 6, 0, 32, 12),
    ("MnMississippiConfluenceC02R00", 38, 0, 32, 12),
    ("MnMississippiConfluenceC03R00", 70, 0, 4, 12),
];
const PATCHES: [(usize, usize, usize, u16, u16); 10] = [
    (1, 6, 2, 2, 3),
    (1, 7, 1, 1, 3),
    (2, 19, 3, 2, 5),
    (2, 20, 3, 2, 0x56),
    (2, 21, 3, 2, 5),
    (2, 20, 1, 2, 3),
    (2, 20, 2, 2, 3),
    (2, 19, 2, 2, 3),
    (2, 21, 2, 1, 3),
    (2, 22, 2, 2, 3),
];

/// Revises ten original Floor cells across two installed Confluence banks.
///
/// Certifies the complete 74×12 source, including the offset six-wide city bend
/// and clipped St Paul mouth. Only the two bank block arrays, encoded mirrors,
/// manifest append and identity change. All shores, events, actors, encounters,
/// source art and unrelated game content remain intact. The dry bank return has
/// both legal southward hops and a useful ordinary ascent. No file is written.
///
/// # Errors
/// Rejects source/profile/preimage drift, repeated application, shared labels,
/// affected anchors, lost walking access, invalid native collision or unsafe
/// ascent/return geometry. The parent remains unchanged on failure.
pub fn revise_confluence_grass(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        !base.report.manifests.iter().any(|v| v == MANIFEST),
        "Confluence grass already installed"
    );
    let mut a: Value = serde_json::from_str(before_json)?;
    let mut b: Value = serde_json::from_str(after_json)?;
    for header in [&mut a, &mut b] {
        header
            .as_object_mut()
            .context("source object")?
            .remove("layers");
    }
    ensure!(
        a == b && a["shore_fitting"] == "explicit_source_edges",
        "only explicit source layers may change"
    );
    let before: Layout = serde_json::from_str(before_json)?;
    let after: Layout = serde_json::from_str(after_json)?;
    for layout in [&before, &after] {
        ensure!(
            layout.schema_version == 2
                && layout.id == PREFIX
                && layout.dimensions_metatiles == [74, 12]
                && layout.compiled_tileset == "johto",
            "complete current Confluence source required"
        );
    }
    let mut expected = vec![5; 74 * 12];
    for (name, ox, oy, width, height) in PROFILES {
        let m = base.data.maps.get(name).context("current Confluence map")?;
        ensure!(
            usize::from(m.attributes.width) == width
                && usize::from(m.attributes.height) == height
                && m.attributes.tileset_name == "johto"
                && m.attributes.border_block == 5,
            "Confluence current clipped profile differs"
        );
        let links: [(&str, &str, i32); 2] = match name {
            "MnMississippiConfluenceC00R00" => [
                ("west", "MnR03C00R00", 0),
                ("east", "MnMississippiConfluenceC01R00", -4),
            ],
            "MnMississippiConfluenceC01R00" => [
                ("west", "MnMississippiConfluenceC00R00", 4),
                ("east", "MnMississippiConfluenceC02R00", 0),
            ],
            "MnMississippiConfluenceC02R00" => [
                ("west", "MnMississippiConfluenceC01R00", 0),
                ("east", "MnMississippiConfluenceC03R00", 0),
            ],
            _ => [
                ("west", "MnMississippiConfluenceC02R00", 0),
                ("north", "MnStPaul", -36),
            ],
        };
        ensure!(
            m.attributes.connections.len() == 2
                && links.iter().all(|(direction, target, offset)| m
                    .attributes
                    .connections
                    .iter()
                    .any(|c| c.direction == *direction
                        && c.target_map == *target
                        && c.offset == *offset)),
            "Confluence reciprocal seam or city mouth changed"
        );
        for y in 0..height {
            for x in 0..width {
                expected[(oy + y) * 74 + ox + x] = m.blocks[y * width + x];
            }
        }
    }
    ensure!(
        before.compile()? == expected,
        "full current source preimage differs"
    );
    let definition = &base.data.tilesets["johto"];
    for (block, collision) in [
        (1, ["FLOOR"; 4]),
        (2, ["FLOOR"; 4]),
        (3, ["TALL_GRASS"; 4]),
        (5, ["WALL"; 4]),
        (0x56, ["HOP_DOWN", "HOP_DOWN", "WALL", "WALL"]),
    ] {
        ensure!(
            definition
                .collision
                .get(&format!("{block:02x}"))
                .is_some_and(|v| v.iter().map(String::as_str).eq(collision)),
            "native bank collision drift"
        );
    }
    let mut output = base.clone();
    for index in [1, 2] {
        let (name, ox, oy, width, _) = PROFILES[index];
        let old = base.data.maps.get(name).context("owned bank")?;
        ensure!(
            old.script_block_changes.is_empty(),
            "unreviewed bank block program"
        );
        ensure!(
            base.data
                .wild_encounters
                .get(name)
                .is_some_and(|w| w.grass.is_some()),
            "native bank grass encounters required"
        );
        let label = old
            .attributes
            .blocks_label
            .as_ref()
            .context("native blocks label")?;
        ensure!(
            base.data
                .maps
                .iter()
                .all(|(n, m)| n == name || m.attributes.blocks_label.as_ref() != Some(label)),
            "shared bank block label"
        );
        ensure!(
            crate::decode_base64_bytes(
                base.data
                    .map_blocks
                    .get(label)
                    .context("native block mirror")?
            )?
            .into_iter()
            .map(u16::from)
            .eq(old.blocks.iter().copied()),
            "native block mirror drift"
        );
        let protected = old
            .objects
            .iter()
            .map(|v| (usize::from(v.x / 2), usize::from(v.y / 2)))
            .chain(
                old.events
                    .warps
                    .iter()
                    .map(|v| (usize::from(v.x / 2), usize::from(v.y / 2))),
            )
            .chain(
                old.events
                    .bg_events
                    .iter()
                    .map(|v| (usize::from(v.x / 2), usize::from(v.y / 2))),
            )
            .chain(
                old.events
                    .coord_events
                    .iter()
                    .map(|v| (usize::from(v.x / 2), usize::from(v.y / 2))),
            )
            .collect::<BTreeSet<_>>();
        let mut revised = old.clone();
        for (owner, x, y, pre, post) in PATCHES {
            if owner != index {
                continue;
            }
            ensure!(
                !protected.contains(&(x, y)) && revised.blocks[y * width + x] == pre,
                "bank preimage or actor/event anchor differs at {name}:{x},{y}"
            );
            revised.blocks[y * width + x] = post;
            expected[(oy + y) * 74 + ox + x] = post;
        }
        validate_access(base, old, &revised, index)?;
        let bytes = revised
            .blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        output.data.maps.insert(name.into(), revised);
        output
            .data
            .map_blocks
            .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    }
    ensure!(
        after.compile()? == expected,
        "only ten reviewed bank cells may change"
    );
    output.report.manifests.push(MANIFEST.into());
    output.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        output.format_version,
        &output.data,
        &output.audio_manifest,
        &output.runtime_files,
        &output.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    crate::validate_compact_superior_geography(&output)?;
    Ok(output)
}

fn validate_access(
    pack: &CompiledGamePack,
    old: &MapModule,
    revised: &MapModule,
    index: usize,
) -> Result<()> {
    let occupied = old
        .objects
        .iter()
        .map(|v| [v.x, v.y])
        .collect::<BTreeSet<_>>();
    let starts: &[[u16; 2]] = if index == 1 {
        &[[13, 4], [14, 3]]
    } else {
        &[[40, 8], [41, 8], [42, 11], [40, 5]]
    };
    for &start in starts {
        let before = walking_map_distances(pack, old, start, &occupied)?;
        let after = walking_map_distances(pack, revised, start, &occupied)?;
        ensure!(
            before.keys().all(|at| after.contains_key(at)
                || (index == 2 && matches!(*at, [38 | 39 | 42 | 43, 6 | 7] | [40 | 41, 7]))),
            "bank loses undeclared original walking access"
        );
    }
    if index == 2 {
        for (x, distance) in [(40, 11), (41, 9)] {
            let down = walking_map_distances(pack, revised, [x, 5], &occupied)?;
            ensure!(down.get(&[x, 8]) == Some(&3), "native dry bank hop differs");
            let mut no_lip = occupied.clone();
            no_lip.extend([[40, 6], [41, 6]]);
            let up = walking_map_distances(pack, revised, [x, 8], &no_lip)?;
            ensure!(
                up.get(&[x, 5]) == Some(&distance),
                "ordinary bank return differs"
            );
        }
    }
    Ok(())
}
