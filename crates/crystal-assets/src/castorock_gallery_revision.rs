//! Terrain-only revision of the installed, scripted Castorock discovery.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::authored_town_geometry;
use crate::{CompiledGamePack, MapModule};

const ROOM: &str = "MnR03Cave1F";
const BOULDER: &str = "MN_R03_JAW_BOULDER";
const CREATURE: &str = "MN_R03_JAW_CASTOROCK";

#[derive(Deserialize)]
struct Site {
    id: String,
    map: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    rows: Vec<String>,
    entrance_preserved: [u16; 2],
    initial_arrival: [u16; 2],
    boulder: [u16; 2],
    boulder_approach: [u16; 2],
    boulder_pushes_to_junction: u16,
    creature: [u16; 2],
    creature_approach: [u16; 2],
    clues: [[u16; 2]; 3],
    clue_approaches: [[u16; 2]; 3],
}

/// Reshapes the current incisor gallery while retaining every native program.
///
/// The two site documents may differ only in their complete native block rows.
/// The original blocks, their uniquely owned split payload, dimensions, forced
/// palette, actor anchors and observation markers must match the installed game.
/// The Strength throat must still seal the gallery even with Surf. Opening it
/// must connect every dry floor, observation, actor and original mouth, including
/// both the eastward two-push position and the reverse return buffer. Art,
/// audio, scripts, encounters, flags, warps and all unrelated maps remain exact.
/// This operation does not migrate saves or prove earned access or visual quality.
///
/// # Errors
/// Rejects stale sources, changed nonterrain fields, unsupported native geometry,
/// open borders, disconnected floor, a bypassed Strength gate or invalid packs.
/// Borrowed inputs remain unchanged on failure.
pub fn revise_castorock_gallery(
    base: &CompiledGamePack,
    previous_site: &str,
    revised_site: &str,
) -> Result<CompiledGamePack> {
    let mut old_value: Value = serde_json::from_str(previous_site)?;
    let mut new_value: Value = serde_json::from_str(revised_site)?;
    let old: Site = serde_json::from_value(old_value.clone())?;
    let new: Site = serde_json::from_value(new_value.clone())?;
    for value in [&mut old_value, &mut new_value] {
        value
            .as_object_mut()
            .context("gallery site object")?
            .remove("rows");
    }
    ensure!(
        old_value == new_value,
        "gallery revision changes retained source fields"
    );
    ensure!(
        old.id == "minnesota-castorock-incisor-gallery-v1"
            && old.map == ROOM
            && old.tileset == "cave"
            && old.dimensions_metatiles == [14, 8]
            && old.entrance_preserved == [7, 9]
            && old.initial_arrival == [7, 8]
            && old.boulder == [13, 6]
            && old.boulder_approach == [12, 6]
            && old.boulder_pushes_to_junction == 2
            && old.creature == [21, 11]
            && old.creature_approach == [21, 10]
            && old.clues == [[2, 5], [20, 4], [23, 13]]
            && old.clue_approaches == [[3, 5], [19, 4], [23, 12]],
        "requires the original installed Castorock discovery"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let before = blocks(&old)?;
    let after = blocks(&new)?;
    ensure!(before != after, "gallery terrain is unchanged");
    let original = base
        .data
        .maps
        .get(ROOM)
        .context("installed discovery cave")?;
    let attributes = &original.attributes;
    ensure!(
        original.blocks == before
            && [attributes.width, attributes.height] == [14, 8]
            && attributes.tileset_name == "cave"
            && attributes.environment.as_deref() == Some("CAVE")
            && attributes.time_of_day.as_deref() == Some("nite")
            && attributes.connections.is_empty()
            && original.script_block_changes.is_empty()
            && base.data.map_attributes.get(ROOM) == Some(attributes),
        "native gallery preimage differs"
    );
    let label = attributes
        .blocks_label
        .as_deref()
        .context("gallery blocks label")?;
    ensure!(
        base.data.maps.iter().all(
            |(name, map)| name == ROOM || map.attributes.blocks_label.as_deref() != Some(label)
        ) && crate::decode_base64_bytes(
            base.data
                .map_blocks
                .get(label)
                .context("gallery blocks payload")?
        )?
        .into_iter()
        .map(u16::from)
        .eq(before),
        "gallery block payload disagrees or is shared"
    );
    for (identifier, at, movement) in [
        (BOULDER, old.boulder, "SPRITEMOVEDATA_STRENGTH_BOULDER"),
        (CREATURE, old.creature, "SPRITEMOVEDATA_STANDING_DOWN"),
    ] {
        let actor = original
            .objects
            .iter()
            .find(|o| o.object_identifier.as_deref() == Some(identifier))
            .context("retained discovery actor")?;
        ensure!(
            [actor.x, actor.y] == at && actor.spritemovedata == movement,
            "discovery actor source differs"
        );
    }
    ensure!(
        original.events.warps.len() == 1
            && [original.events.warps[0].x, original.events.warps[0].y] == old.entrance_preserved
            && original
                .events
                .bg_events
                .iter()
                .map(|e| [e.x, e.y])
                .collect::<BTreeSet<_>>()
                == old.clues.into_iter().collect(),
        "discovery event anchors differ"
    );
    let mut revised = original.clone();
    revised.blocks = after;
    validate_geometry(base, &revised, &new)?;
    let bytes = revised
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut result = base.clone();
    result.data.maps.insert(ROOM.into(), revised);
    result
        .data
        .map_blocks
        .insert(label.into(), crate::encode_base64_bytes(&bytes));
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

fn blocks(site: &Site) -> Result<Vec<u16>> {
    ensure!(site.rows.len() == 8, "eight complete cave rows required");
    let mut blocks = Vec::new();
    for row in &site.rows {
        let values = row
            .split_whitespace()
            .map(|word| u16::from_str_radix(word, 16))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(
            values.len() == 14 && values.iter().all(|&v| v < 0x40),
            "native cave row extent"
        );
        blocks.extend(values);
    }
    Ok(blocks)
}

fn validate_geometry(base: &CompiledGamePack, room: &MapModule, site: &Site) -> Result<()> {
    let kind = |at| authored_town_geometry::kind(base, room, at);
    for y in 0..16 {
        for x in 0..28 {
            let token = kind([x, y])?;
            ensure!(
                matches!(token, "FLOOR" | "WALL" | "UP_WALL" | "WATER" | "LADDER"),
                "unsupported discovery terrain at {x},{y}: {token}"
            );
            if x == 0 || x == 27 || y == 0 || y == 15 {
                ensure!(token == "WALL", "open discovery border at {x},{y}");
            }
            if token == "LADDER" {
                ensure!([x, y] == site.entrance_preserved, "unpaired cave ladder");
            }
        }
    }
    ensure!(
        kind(site.entrance_preserved)? == "LADDER",
        "original mouth must remain a ladder"
    );
    let reserved = [
        site.initial_arrival,
        site.boulder_approach,
        site.creature_approach,
    ]
    .into_iter()
    .chain(site.clue_approaches)
    .chain(room.objects.iter().map(|o| [o.x, o.y]));
    for at in reserved {
        ensure!(
            kind(at)? == "FLOOR",
            "retained native approach or body blocked at {at:?}"
        );
    }
    for at in site.clues {
        ensure!(kind(at)? == "WALL", "observation marker must remain stone");
    }
    let closed = authored_town_geometry::reachable(base, room, site.initial_arrival, true)?;
    ensure!(
        closed.contains(&site.boulder_approach)
            && !closed.contains(&site.creature_approach)
            && !closed.contains(&site.clue_approaches[1])
            && !closed.contains(&site.clue_approaches[2]),
        "Strength gate has an ordinary or Surf bypass"
    );
    // The two forward pushes need both their destination and the player's
    // ordinary walking buffer. Reverse pushes retain the original west escape.
    for x in 10..=15 {
        ensure!(
            kind([x, 6])? == "FLOOR",
            "Strength push/return buffer blocked"
        );
    }
    for boulder_x in [10, 15] {
        let mut opened = room.clone();
        let actor = opened
            .objects
            .iter_mut()
            .find(|o| o.object_identifier.as_deref() == Some(BOULDER))
            .context("retained Strength boulder")?;
        actor.x = boulder_x;
        let start = if boulder_x == 15 {
            [14, 6]
        } else {
            site.creature_approach
        };
        let seen = authored_town_geometry::reachable(base, &opened, start, false)?;
        ensure!(
            seen.contains(&site.initial_arrival)
                && seen.contains(&site.creature_approach)
                && site.clue_approaches.iter().all(|at| seen.contains(at)),
            "pushed boulder blocks a discovery or the original exit"
        );
    }
    let mut opened = room.clone();
    opened
        .objects
        .retain(|o| o.object_identifier.as_deref() != Some(BOULDER));
    let seen = authored_town_geometry::reachable(base, &opened, site.initial_arrival, false)?;
    let bodies = opened
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    for y in 0..16 {
        for x in 0..28 {
            if kind([x, y])? == "FLOOR" && !bodies.contains(&[x, y]) {
                ensure!(
                    seen.contains(&[x, y]),
                    "unreachable dry discovery floor at {x},{y}"
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::revise_castorock_gallery;

    #[test]
    #[ignore = "requires CRYSTAL_CASTOROCK_REVISION_PACK with installed original gallery"]
    fn castorock_gallery_revision_preserves_programs_and_rejects_broken_geometry() {
        let path = std::env::var_os("CRYSTAL_CASTOROCK_REVISION_PACK")
            .expect("set CRYSTAL_CASTOROCK_REVISION_PACK to the completed parent");
        let base = crate::read_verified_compiled_game_pack(Path::new(&path))
            .expect("verified installed gallery parent");
        let source_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/caves/current-r03-gallery-v02");
        let old = std::fs::read_to_string(source_dir.join("site-before.json"))
            .expect("original runtime-loaded site fixture");
        let new = std::fs::read_to_string(source_dir.join("site-v02.json"))
            .expect("revised runtime-loaded site fixture");
        let revised = revise_castorock_gallery(&base, &old, &new)
            .expect("reviewed dry gallery with closed Strength gate");
        assert_ne!(revised.identity, base.identity);
        let mut normalized = revised;
        normalized
            .data
            .maps
            .insert(super::ROOM.into(), base.data.maps[super::ROOM].clone());
        let label = base.data.maps[super::ROOM]
            .attributes
            .blocks_label
            .as_ref()
            .expect("owned gallery block payload");
        normalized
            .data
            .map_blocks
            .insert(label.clone(), base.data.map_blocks[label].clone());
        normalized.identity.clone_from(&base.identity);
        assert_eq!(
            normalized, base,
            "only native terrain/payload/identity may change"
        );

        let mut changed: serde_json::Value = serde_json::from_str(&new).expect("site JSON");
        changed["level"] = 25.into();
        let error = revise_castorock_gallery(&base, &old, &changed.to_string())
            .expect_err("terrain authoring must not change encounter level");
        assert!(error.to_string().contains("retained source fields"));

        for (x, y, block, expected_error) in [
            (0, 0, "02", "open discovery border"),
            (6, 3, "02", "Strength gate"),
            (7, 3, "2d", "Strength push/return buffer"),
            (10, 2, "02", "observation marker"),
            (1, 6, "02", "unreachable dry discovery floor"),
        ] {
            let mut changed: serde_json::Value = serde_json::from_str(&new).expect("site JSON");
            let mut row = changed["rows"][y]
                .as_str()
                .expect("native row")
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            row[x] = block.into();
            changed["rows"][y] = row.join(" ").into();
            let error = revise_castorock_gallery(&base, &old, &changed.to_string())
                .expect_err("unsafe authored gallery must be rejected");
            assert!(
                error.to_string().contains(expected_error),
                "{x},{y}: {error:#}"
            );
        }
        let error = revise_castorock_gallery(&base, &new, &old)
            .expect_err("the revised source is not the native preimage");
        assert!(error.to_string().contains("preimage differs"));
    }
}
