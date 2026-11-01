//! Four-tile coastal decks with solid rails and preserved town checkpoints.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::{CompiledGamePack, CoordEvent, MapModule};

const TILESET: &str = "mn_north_shore";
const MARKER: &str = "minnesota-north-shore-passing-width-v1";
const CROSSINGS: [(usize, usize, usize); 7] = [
    (40, 40, 2),
    (40, 76, 2),
    (44, 96, 4),
    (40, 110, 2),
    (34, 122, 2),
    (40, 162, 2),
    (46, 186, 2),
];

#[derive(Deserialize)]
struct Terrain {
    dimensions_metatiles: [u16; 2],
    tileset: String,
    rows: Vec<String>,
}

impl Terrain {
    fn blocks(&self) -> Result<Vec<u16>> {
        ensure!(
            self.rows.len() == usize::from(self.dimensions_metatiles[1]),
            "complete height required"
        );
        let mut blocks = Vec::new();
        for row in &self.rows {
            let words = row.split_whitespace().collect::<Vec<_>>();
            ensure!(
                words.len() == usize::from(self.dimensions_metatiles[0]),
                "complete width required"
            );
            for word in words {
                blocks.push(u16::from_str_radix(word, 16).context("native coast block")?);
            }
        }
        Ok(blocks)
    }
}

#[derive(Deserialize)]
struct Coordinates {
    before: Vec<CoordEvent>,
    after: Vec<CoordEvent>,
}

fn names() -> Vec<String> {
    let mut names = (0..2)
        .flat_map(|column| (0..4).map(move |row| format!("MnR11C{column:02}R{row:02}")))
        .collect::<Vec<_>>();
    names.extend(["MnDuluth".into(), "MnGrandMarais".into()]);
    names
}

fn put_route(maps: &mut BTreeMap<String, MapModule>, x: usize, y: usize, block: u16) -> Result<()> {
    let name = format!("MnR11C{:02}R{:02}", x / 64, y / 48);
    let map = maps.get_mut(&name).context("coastal chunk")?;
    let index = y % 48 / 2 * usize::from(map.attributes.width) + x % 64 / 2;
    *map.blocks.get_mut(index).context("bounded coastal cell")? = block;
    Ok(())
}

fn widen(maps: &mut BTreeMap<String, MapModule>) -> Result<()> {
    for (x, y, length) in CROSSINGS {
        for yy in (y..y + length).step_by(2) {
            for (xx, block) in [(x - 2, 0x85), (x, 1), (x + 2, 0x86)] {
                put_route(maps, xx, yy, block)?;
            }
        }
        for yy in [y - 2, y + length] {
            for xx in (x - 2..x + 4).step_by(2) {
                put_route(maps, xx, yy, 1)?;
            }
        }
    }
    // Shift the lighthouse crossing west, retaining visible native water
    // margins on both sides rather than placing its rails against dry grass.
    for (y, block) in [(96, 0x54), (98, 0x58)] {
        put_route(maps, 40, y, block)?;
    }
    for (name, cells) in [
        (
            "MnDuluth",
            vec![(32, 0), (33, 0), (32, 1), (33, 1), (32, 2), (33, 2)],
        ),
        ("MnGrandMarais", vec![(0, 5), (1, 5)]),
    ] {
        let map = maps.get_mut(name).context("coastal town")?;
        for (x, y) in cells {
            map.blocks[y * usize::from(map.attributes.width) + x] = 1;
        }
    }
    for y in [188, 190] {
        for x in [46, 48] {
            put_route(maps, x, y, 1)?;
        }
    }
    for x in [66, 68, 70] {
        put_route(maps, x, 10, 1)?;
    }
    Ok(())
}

fn extend_gate(map: &mut MapModule) -> Result<()> {
    let (lanes, checks) = match map.id.as_str() {
        "MnDuluth" => ([66, 67], ["MnDuluthShoreCheck", "MnDuluthShoreCheckInward"]),
        "MnGrandMarais" => (
            [10, 11],
            ["MnCohesionMaraisWest", "MnCohesionMaraisWestInward"],
        ),
        _ => anyhow::bail!("only the two coastal town checkpoints may expand"),
    };
    for lane in lanes {
        for script in checks {
            let mut event = map
                .events
                .coord_events
                .iter()
                .find(|event| event.script_name == script)
                .context("original coastal checkpoint")?
                .clone();
            if map.id == "MnDuluth" {
                event.x = lane;
            } else {
                event.y = lane;
            }
            ensure!(
                !map.events
                    .coord_events
                    .iter()
                    .any(|old| old.x == event.x && old.y == event.y),
                "new gate lane occupied"
            );
            map.events.coord_events.push(event);
        }
    }
    crate::grand_marais_modpack::authoring::refresh_event_section(map);
    Ok(())
}

/// Widens seven native decks and both town mouths without opening story gates.
///
/// Appends two rail caps assembled exclusively from existing native glyphs.
/// Each deck has four FLOOR lanes and two WALL edge lanes. Original programs,
/// objects, connections, warps, audio, encounters and all other maps are retained.
/// Existing saved terrain overrides are not migrated.
///
/// # Errors
/// Rejects stale full-map sources, changed art extents, any terrain outside the
/// reviewed footprints, missing new checkpoint events, repeat publication, or
/// invalid resulting content.
pub fn rebuild_north_shore_passing_width(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
    coordinates_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        !base.report.manifests.iter().any(|id| id == MARKER),
        "passing width already installed"
    );
    let before: BTreeMap<String, Terrain> = serde_json::from_str(before_json)?;
    let after: BTreeMap<String, Terrain> = serde_json::from_str(after_json)?;
    let coordinates: BTreeMap<String, Coordinates> = serde_json::from_str(coordinates_json)?;
    ensure!(
        before.len() == 10 && after.len() == 10 && coordinates.len() == 2,
        "exact ten-map terrain and two checkpoint sources required"
    );
    let mut maps = BTreeMap::new();
    for name in names() {
        let original = base.data.maps.get(&name).context("installed coastal map")?;
        let old = before.get(&name).context("complete coast preimage")?;
        let new = after.get(&name).context("complete coast candidate")?;
        ensure!(
            old.dimensions_metatiles == [original.attributes.width, original.attributes.height]
                && new.dimensions_metatiles == old.dimensions_metatiles
                && old.tileset == original.attributes.tileset_name
                && new.tileset == old.tileset
                && old.blocks()? == original.blocks,
            "stale coast preimage: {name}"
        );
        maps.insert(name, original.clone());
    }
    widen(&mut maps)?;
    for (name, map) in &mut maps {
        ensure!(
            after[name].blocks()? == map.blocks,
            "only reviewed bridge, apron and town-mouth cells may change: {name}"
        );
        if matches!(name.as_str(), "MnDuluth" | "MnGrandMarais") {
            let gates = coordinates.get(name).context("coastal checkpoint source")?;
            ensure!(
                gates.before == map.events.coord_events,
                "stale checkpoint preimage"
            );
            extend_gate(map)?;
            ensure!(
                gates.after == map.events.coord_events,
                "every new lane must retain the original gate"
            );
        }
    }
    let definition = base
        .data
        .tilesets
        .get(TILESET)
        .context("installed coastal tileset")?;
    ensure!(
        definition
            .collision
            .get("01")
            .is_some_and(|c| c.iter().all(|k| k == "FLOOR"))
            && definition
                .collision
                .get("84")
                .is_some_and(|c| c.iter().all(|k| k == "LADDER"))
            && !definition.collision.contains_key("85")
            && !definition.collision.contains_key("86"),
        "native rail/floor extent differs"
    );
    let path = format!("data/tilesets/{TILESET}_metatiles.bin");
    let original_art = base.runtime_files.get(&path).context("coastal art")?;
    ensure!(
        original_art.len() == 0x85 * 16,
        "installed narrow bridge art required"
    );
    let mut art = original_art.clone();
    for rail_column in [0, 3] {
        for row in 0..4 {
            for column in 0..4 {
                let glyph = if column == rail_column {
                    original_art[0x44 * 16 + row * 4]
                } else {
                    original_art[16 + row * 4 + column]
                };
                art.push(glyph);
            }
        }
    }
    let mut updated = definition.clone();
    for (id, collision) in [
        ("85", ["WALL", "FLOOR", "WALL", "FLOOR"]),
        ("86", ["FLOOR", "WALL", "FLOOR", "WALL"]),
    ] {
        updated
            .collision
            .insert(id.into(), collision.map(str::to_owned).to_vec());
    }
    let mut pack = base.clone();
    for (name, map) in maps {
        if map != base.data.maps[&name] {
            crate::authored_town_publication::replace(&mut pack, map)?;
        }
    }
    pack.runtime_files.insert(path, art);
    pack.runtime_files.insert(
        format!("data/tilesets/{TILESET}.json"),
        serde_json::to_vec(&updated.collision)?,
    );
    pack.data.tilesets.insert(TILESET.into(), updated);
    pack.report.manifests.push(MARKER.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}

#[cfg(test)]
#[path = "north_shore_passing_width_tests.rs"]
mod tests;
