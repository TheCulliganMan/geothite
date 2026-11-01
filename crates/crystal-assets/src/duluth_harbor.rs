//! Open Duluth's inner harbor and add two native floating fishing decks.

use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;
use crate::grand_marais_modpack::authoring;

const CITY: &str = "MnDuluth";
const MANIFEST: &str = "minnesota-duluth-harbor-platforms-v1";
const PLATFORMS: [[usize; 2]; 2] = [[26, 36], [34, 39]];
const DECK: [[u16; 3]; 3] = [[0x40, 0x41, 0x42], [0x01, 0x01, 0x46], [0x48, 0x49, 0x4a]];

const SHORE_ROCKS: [[usize; 2]; 38] = [
    [61, 10],
    [59, 13],
    [59, 14],
    [60, 20],
    [60, 23],
    [60, 26],
    [60, 30],
    [48, 23],
    [48, 26],
    [49, 30],
    [55, 33],
    [31, 28],
    [15, 31],
    [14, 33],
    [13, 35],
    [2, 34],
    [2, 35],
    [2, 38],
    [3, 41],
    [5, 43],
    [6, 43],
    [9, 43],
    [12, 42],
    [39, 34],
    [40, 38],
    [42, 39],
    [45, 41],
    [46, 41],
    [45, 34],
    [47, 35],
    [51, 37],
    [52, 37],
    [57, 38],
    [59, 40],
    [60, 40],
    [61, 42],
    [54, 43],
    [55, 44],
];

/// Clears the remaining interior buoy corner without changing the map boundary.
///
/// Requires the complete current layout as a preimage and changes only native
/// buoy38 at metatile16,33 to ordinary water35. Retains all other terrain,
/// events, native art and audio. Borrows the parent and writes no files.
///
/// # Errors
/// Rejects stale layouts, changed art, unexpected buoy collision, unrelated
/// terrain changes, repeated publication or failed runtime verification.
pub fn clear_duluth_inner_buoys(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let original = base.data.maps.get(CITY).context("installed Duluth")?;
    let before: Layout = serde_json::from_str(before_json)?;
    let after: Layout = serde_json::from_str(after_json)?;
    let old_art: serde_json::Value = serde_json::from_str(before_json)?;
    let new_art: serde_json::Value = serde_json::from_str(after_json)?;
    ensure!(
        original.attributes.width == 64
            && original.attributes.height == 46
            && before.id == CITY
            && after.id == CITY
            && before.dimensions_metatiles == [64, 46]
            && after.dimensions_metatiles == [64, 46]
            && before.compiled_tileset == original.attributes.tileset_name
            && after.compiled_tileset == original.attributes.tileset_name
            && before.compile()? == original.blocks,
        "complete current Duluth source required"
    );
    ensure!(
        ["tileset", "derived_blocks", "source_pixels_modified"]
            .iter()
            .all(|key| old_art.get(key) == new_art.get(key))
            && new_art["source_pixels_modified"] == false,
        "retain existing native shore art"
    );
    let mut city = original.clone();
    ensure!(
        city.blocks[33 * 64 + 16] == 0x38,
        "interior buoy preimage differs"
    );
    let collision = &base.data.tilesets[&original.attributes.tileset_name].collision;
    ensure!(
        collision["38"].iter().any(|kind| kind == "BUOY")
            && collision["35"].iter().all(|kind| kind == "WATER"),
        "native buoy/water collision differs"
    );
    city.blocks[33 * 64 + 16] = 0x35;
    ensure!(
        city.blocks == after.compile()?,
        "only the interior buoy corner may change"
    );
    let mut result = base.clone();
    crate::authored_town_publication::replace(&mut result, city)?;
    result
        .report
        .manifests
        .push("minnesota-duluth-interior-buoy-clearance-v1".into());
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

/// Adds native boulder clusters on the grassy shore of the opened harbor.
///
/// Rocks occupy only the specified land cells next to water. All harbor water,
/// piers, floating decks, perimeter buoys, events, art and audio remain exact.
/// Existing reachable walking surfaces outside those cells remain reachable.
/// Borrows the parent and writes no files.
///
/// # Errors
/// Rejects stale complete layouts, changed native art, non-shore rock sites,
/// occupied sites, disconnected walking terrain or failed pack verification.
pub fn revise_duluth_shoreline(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let before: Layout = serde_json::from_str(before_json)?;
    let after: Layout = serde_json::from_str(after_json)?;
    let before_art: serde_json::Value = serde_json::from_str(before_json)?;
    let after_art: serde_json::Value = serde_json::from_str(after_json)?;
    let original = base.data.maps.get(CITY).context("installed Duluth")?;
    ensure!(
        original.attributes.width == 64
            && original.attributes.height == 46
            && original.attributes.tileset_name == "mn_duluth_expanded"
            && before.id == CITY
            && after.id == CITY
            && before.dimensions_metatiles == [64, 46]
            && after.dimensions_metatiles == [64, 46]
            && before.compiled_tileset == original.attributes.tileset_name
            && after.compiled_tileset == original.attributes.tileset_name
            && before.compile()? == original.blocks,
        "current complete Duluth source preimage required"
    );
    ensure!(
        ["tileset", "derived_blocks", "source_pixels_modified"]
            .iter()
            .all(|key| before_art.get(key) == after_art.get(key))
            && after_art["source_pixels_modified"] == false,
        "shore rocks must use unchanged native art"
    );
    let mut city = original.clone();
    for [x, y] in SHORE_ROCKS {
        ensure!(
            original.blocks[y * 64 + x] == 0x02
                && [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
                    .into_iter()
                    .any(|(nx, ny)| original.blocks[ny * 64 + nx] == 0x35)
                && !city
                    .objects
                    .iter()
                    .any(|actor| usize::from(actor.x / 2) == x && usize::from(actor.y / 2) == y),
            "shore rocks need unoccupied grassy land beside water at {x},{y}"
        );
        city.blocks[y * 64 + x] = 0x0a;
    }
    ensure!(
        after.compile()? == city.blocks,
        "only contracted shore rocks may change"
    );
    let old_walk = crate::authored_town_geometry::reachable(base, original, [64, 42], false)?;
    let mut result = base.clone();
    crate::authored_town_publication::replace(&mut result, city)?;
    let city = &result.data.maps[CITY];
    let new_walk = crate::authored_town_geometry::reachable(&result, city, [64, 42], false)?;
    ensure!(
        old_walk.iter().all(|point| {
            SHORE_ROCKS.contains(&[usize::from(point[0] / 2), usize::from(point[1] / 2)])
                || new_walk.contains(point)
        }),
        "shore rocks cannot disconnect existing walking terrain"
    );
    result
        .report
        .manifests
        .push("minnesota-duluth-shore-rocks-v1".into());
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

/// Removes raised water-edge art and the isolated buoy, retaining the perimeter.
///
/// Two railed decks have open western boarding faces and ordinary fishermen
/// with native paged dialogue. Existing actors, scripts, rooms, native art,
/// encounters and audio are retained. Does not write files or migrate saves.
///
/// # Errors
/// Rejects stale terrain, changed extents/art, occupied platform water, missing
/// native fishermen, altered unrelated terrain, or failed pack verification.
pub fn revise_duluth_harbor(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        !base.report.manifests.iter().any(|id| id == MANIFEST),
        "harbor revision already installed"
    );
    let before: Layout = serde_json::from_str(before_json)?;
    let after: Layout = serde_json::from_str(after_json)?;
    let old_source: serde_json::Value = serde_json::from_str(before_json)?;
    let new_source: serde_json::Value = serde_json::from_str(after_json)?;
    ensure!(
        new_source["source_pixels_modified"] == false
            && old_source["derived_blocks"] == new_source["derived_blocks"],
        "retain every existing native art recipe"
    );
    let original = base.data.maps.get(CITY).context("installed Duluth")?;
    ensure!(
        original.attributes.width == 64
            && original.attributes.height == 46
            && original.attributes.tileset_name == "mn_duluth_expanded"
            && before.id == CITY
            && after.id == CITY
            && before.dimensions_metatiles == [64, 46]
            && after.dimensions_metatiles == [64, 46]
            && before.compiled_tileset == original.attributes.tileset_name
            && after.compiled_tileset == original.attributes.tileset_name
            && before.compile()? == original.blocks,
        "current complete Duluth source preimage required"
    );
    let mut expected = original.blocks.clone();
    let collision = &base.data.tilesets[&original.attributes.tileset_name].collision;
    for y in 8..45 {
        for x in 1..63 {
            let block = &mut expected[y * 64 + x];
            if matches!(*block, 0x54 | 0x55 | 0x58 | 0x59 | 0x76 | 0x79 | 0x7a) {
                ensure!(
                    collision[&format!("{block:02x}")]
                        .iter()
                        .all(|kind| kind == "WATER"),
                    "seawall art must retain ordinary water collision"
                );
                *block = 0x35;
            }
        }
    }
    ensure!(
        expected[33 * 64 + 47] == 0x3a,
        "isolated buoy preimage drift"
    );
    expected[33 * 64 + 47] = 0x35;
    for [x, y] in PLATFORMS {
        for (dy, row) in DECK.iter().enumerate() {
            for (dx, block) in row.iter().enumerate() {
                let cell = &mut expected[(y + dy) * 64 + x + dx];
                ensure!(*cell == 0x35, "floating decks require clear harbor water");
                *cell = *block;
            }
        }
    }
    let mut city = original.clone();
    city.blocks = after.compile()?;
    ensure!(
        city.blocks == expected,
        "only contracted harbor terrain may change"
    );
    let fisher = original
        .objects
        .iter()
        .find(|object| object.object_identifier.as_deref() == Some("MN_DULUTH_PIER_FISHER"))
        .context("existing native pier fisherman")?;
    for (index, [x, y]) in PLATFORMS.into_iter().enumerate() {
        let script = format!("MnDuluthFloatingFisher{}", index + 1);
        let label = format!("{script}Text");
        ensure!(
            !city.scripts.contains_key(&script),
            "new fisher script occupied"
        );
        city.scripts.insert(
            script.clone(),
            json!([authoring::command("jumptextfaceplayer", &[&label])]),
        );
        authoring::text(
            &mut city,
            &label,
            if index == 0 {
                &[
                    "This deck rises",
                    "with the waves.",
                    "The fish gather",
                    "in its shade.",
                ]
            } else {
                &[
                    "Quiet water out",
                    "past the piers.",
                    "I could fish here",
                    "all afternoon!",
                ]
            },
        );
        let mut actor = fisher.clone();
        actor.x = u16::try_from(x * 2 + 3)?;
        actor.y = u16::try_from(y * 2 + 3)?;
        actor.script = script;
        actor.object_identifier = Some(format!("MN_DULUTH_FLOATING_FISHER_{}", index + 1));
        actor.event_flag = "-1".into();
        actor.move_range_x = 0;
        actor.move_range_y = 0;
        city.objects.push(actor);
    }
    // Rebuild dialogue lookups; existing mutation and battle tables stay exact.
    city.script_text_commands = crate::parse_script_text_commands(CITY, &city.scripts)?;
    city.script_text_bodies = crate::parse_script_text_bodies(CITY, &city.scripts)?;
    authoring::refresh_event_section(&mut city);
    let mut result = base.clone();
    crate::authored_town_publication::replace(&mut result, city)?;
    let city = &result.data.maps[CITY];
    let water = crate::authored_town_geometry::reachable(&result, city, [51, 75], true)?;
    for [x, y] in PLATFORMS {
        let boarding = [u16::try_from(x * 2)?, u16::try_from(y * 2 + 3)?];
        let approach = [boarding[0] + 2, boarding[1]];
        ensure!(
            water.contains(&boarding) && water.contains(&approach),
            "harbor must reach both deck boarding faces and fisherman approaches"
        );
        let deck = crate::authored_town_geometry::reachable(&result, city, boarding, false)?;
        ensure!(
            deck.contains(&approach),
            "fisherman must leave a clear deck approach"
        );
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
    crate::validate_compact_superior_geography(&result)?;
    Ok(result)
}
