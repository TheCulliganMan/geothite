//! Rear-edge Glacier Line depots with native railway blocks and paired doors.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::grand_marais_modpack::authoring;
use crate::native_tile_png;
use crate::{CompiledGamePack, CompiledTilesetExtension, MapModule};

const SOURCE: &str =
    include_str!("../../../planning/minnesota/transit/glacier-rear-rail-v01/stations.json");

#[derive(Deserialize)]
struct Layout {
    parent_content_hash: String,
    cities: Vec<City>,
}

#[derive(Deserialize)]
struct City {
    city: String,
    station: String,
    source_tileset: String,
    moves: Vec<Anchor>,
    rail_end: Option<RailEnd>,
    rows: Vec<String>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RailEnd {
    West,
    East,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum AnchorKind {
    Warp,
    Sign,
    Actor,
}

#[derive(Deserialize)]
struct Anchor {
    kind: AnchorKind,
    id: String,
    before: [u16; 2],
    after: [u16; 2],
}

fn move_anchors(map: &mut MapModule, city: &City) -> Result<()> {
    for anchor in &city.moves {
        let (x, y) = match anchor.kind {
            AnchorKind::Warp => {
                let index = anchor.id.parse::<u16>()?;
                let warp = map
                    .events
                    .warps
                    .iter_mut()
                    .find(|warp| warp.index == index)
                    .context("station door or return warp")?;
                (&mut warp.x, &mut warp.y)
            }
            AnchorKind::Sign => {
                let sign = map
                    .events
                    .bg_events
                    .iter_mut()
                    .find(|sign| sign.script == anchor.id)
                    .context("station sign")?;
                (&mut sign.x, &mut sign.y)
            }
            AnchorKind::Actor => {
                let actor = map
                    .objects
                    .iter_mut()
                    .find(|actor| actor.script == anchor.id)
                    .context("station guide")?;
                (&mut actor.x, &mut actor.y)
            }
        };
        ensure!(
            [*x, *y] == anchor.before,
            "stale anchor in {}: {}",
            city.city,
            anchor.id
        );
        [*x, *y] = anchor.after;
    }
    Ok(())
}

// Work within the original graphics extent so banked tiles keep their addresses.
fn rail_art(
    pack: &CompiledGamePack,
    source: &str,
    alias: &str,
    end: RailEnd,
) -> Result<CompiledTilesetExtension> {
    let files = pack.runtime_files();
    let mut definition = pack.data().tilesets[source].clone();
    let mut blocks = files[&format!("data/tilesets/{source}_metatiles.bin")].clone();
    let mut graphics = files[&format!("gfx/tilesets/{source}.2bpp")].clone();
    // Kanto fills its original single bank. Keep that bank's addresses intact
    // and extend the sheet before assigning any new glyphs.
    if source == "kanto" {
        ensure!(
            blocks
                .iter()
                .all(|&tile| tile < 96 && definition.palette_map[usize::from(tile)] & 8 == 0),
            "Kanto source must use its original unbanked glyphs"
        );
        graphics.resize(224 * 16, 0);
    }
    let half = graphics.len() / 32;
    let physical = |tile: usize, palette: u8| {
        if palette & 8 == 0 {
            tile
        } else {
            (tile & 0x7f) + half
        }
    };
    let used: BTreeSet<_> = blocks
        .iter()
        .map(|&tile| physical(usize::from(tile), definition.palette_map[usize::from(tile)]))
        .collect();
    // Select both an unused logical address and an unused physical glyph.
    // Keep low engine animation addresses out of the allocation.
    let mut free = (0x80..definition.palette_map.len())
        .filter(|&glyph| (glyph & 0x7f) < half)
        .filter(|&glyph| !blocks.contains(&(glyph as u8)))
        .map(|glyph| (glyph, physical(glyph, 8)))
        .filter(|&(_, slot)| !used.contains(&slot))
        .collect::<Vec<_>>()
        .into_iter();
    let native = &files["gfx/tilesets/johto_modern.2bpp"];
    let modern = &pack.data().tilesets["johto_modern"];
    let rail = &files["data/tilesets/johto_modern_metatiles.bin"][0x7e * 16..0x7f * 16];
    let mut track = [0; 16];
    for row in 0..4 {
        let glyph = usize::from(rail[row * 4]);
        let pixels = &native[glyph * 16..(glyph + 1) * 16];
        let palette = modern.palette_map[glyph];
        let slot = if source == "mn_outdoor_modern_stairs" {
            glyph
        } else {
            let (glyph, slot) = free
                .next()
                .with_context(|| format!("unused native rail glyph in {source}"))?;
            graphics[slot * 16..(slot + 1) * 16].copy_from_slice(pixels);
            definition.palette_map[glyph] = palette | 8;
            glyph
        };
        track[row * 4..(row + 1) * 4].fill(u8::try_from(slot)?);
    }
    if source != "mn_outdoor_modern_stairs" {
        let id = blocks.len() / 16;
        blocks.extend_from_slice(&track);
        definition
            .collision
            .insert(format!("{id:02x}"), modern.collision["7e"].clone());
    }
    // A dark timber buffer spans both rails. The pixels beyond it are clear.
    let edge = match end {
        RailEnd::West => 0,
        RailEnd::East => 3,
    };
    let (keep, timber) = match end {
        RailEnd::West => (0x0f, 0x18),
        RailEnd::East => (0xf0, 0x18),
    };
    for row in 0..3 {
        let old = usize::from(track[row * 4]);
        let (glyph, slot) = free
            .next()
            .with_context(|| format!("unused buffer glyph in {source}"))?;
        let old_slot = physical(old, definition.palette_map[old]);
        let mut pixels = [0; 16];
        pixels.copy_from_slice(&graphics[old_slot * 16..(old_slot + 1) * 16]);
        for (y, planes) in pixels.as_chunks_mut::<2>().0.iter_mut().enumerate() {
            let beam = if (6..=22).contains(&(row * 8 + y)) {
                timber
            } else {
                0
            };
            for plane in planes {
                *plane = (*plane & keep) | beam;
            }
        }
        graphics[slot * 16..(slot + 1) * 16].copy_from_slice(&pixels);
        definition.palette_map[glyph] = definition.palette_map[old] | 8;
        track[row * 4 + edge] = u8::try_from(glyph)?;
    }
    let id = blocks.len() / 16;
    blocks.extend_from_slice(&track);
    definition
        .collision
        .insert(format!("{id:02x}"), modern.collision["7e"].clone());

    Ok(CompiledTilesetExtension {
        manifest_id: format!("glacier-rear-rail-{source}"),
        tileset_id: alias.into(),
        behavior_source_tileset_id: Some(source.into()),
        definition,
        metatiles: blocks,
        tile_graphics_png: native_tile_png::from_2bpp(&graphics)?,
        tile_graphics_2bpp: graphics,
    })
}

/// Relocate the five Glacier Line approaches on the verified red-canoe parent.
///
/// Returns an error for any other parent, stale anchor, or invalid map/art.
pub fn revise_glacier_rear_rail(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let layout: Layout = serde_json::from_str(SOURCE)?;
    ensure!(
        base.identity.content_hash == layout.parent_content_hash,
        "expected the selected red-canoe parent"
    );
    let mut pack = base.clone();
    for city in layout.cities {
        let mut map = base.data().maps[&city.city].clone();
        ensure!(
            map.attributes.tileset_name == city.source_tileset,
            "station tileset drift"
        );
        let hall = &base.data().maps[&city.station];
        ensure!(
            hall.events.warps.iter().any(|warp| {
                Some(&warp.target_map_constant) == map.attributes.map_constant.as_ref()
            }),
            "station return must use its existing outdoor warp index"
        );
        let width = usize::from(map.attributes.width);
        ensure!(
            city.rows.len() == usize::from(map.attributes.height),
            "station map height drift"
        );
        map.blocks.clear();
        for row in &city.rows {
            let tiles = row
                .split_ascii_whitespace()
                .map(|value| u16::from_str_radix(value, 16))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            ensure!(tiles.len() == width, "station map width drift");
            map.blocks.extend(tiles);
        }
        if let Some(end) = city.rail_end {
            let alias = format!("{}_glacier_rail", city.source_tileset);
            pack =
                pack.with_tileset_extension(rail_art(&pack, &city.source_tileset, &alias, end)?)?;
            for path in [
                "gfx/tilesets/{}.pal",
                "data/tilesets/{}_animations.json",
                "data/tilesets/{}_foregrounds.json",
                "data/tilesets/{}_native_backgrounds.json",
            ] {
                if let Some(bytes) = base
                    .runtime_files()
                    .get(&path.replace("{}", &city.source_tileset))
                {
                    pack.runtime_files
                        .insert(path.replace("{}", &alias), bytes.clone());
                }
            }
            map.attributes.tileset_name = alias;
        }
        move_anchors(&mut map, &city)?;
        if city.city == "MnStPaul" {
            authoring::text(
                &mut map,
                "MN_ST_PAUL_SIGN_WAYFINDINGText",
                &[
                    "West: CITY CROSS",
                    "North: CAPITOL",
                    "Northwest:",
                    "UNION DEPOT",
                ],
            );
            authoring::text(
                &mut map,
                "MN_ST_PAUL_CIVIC_GUIDEText",
                &["CAPITOL HALL.", "For UNION DEPOT,", "head northwest."],
            );
        }
        if city.city == "MnDetroitLakes" {
            authoring::text(
                &mut map,
                "MN_DETROIT_STATION_STEWARDText",
                &["GLACIER LINE.", "Tracks run behind", "the station hall."],
            );
        }
        if matches!(city.city.as_str(), "MnStPaul" | "MnDetroitLakes") {
            map.script_text_commands = crate::parse_script_text_commands(&map.id, &map.scripts)?;
            map.script_text_bodies = crate::parse_script_text_bodies(&map.id, &map.scripts)?;
        }
        authoring::refresh_event_section(&mut map);
        crate::authored_town_publication::replace(&mut pack, map)?;
        // The next extension verifies its input, including the previous city.
        pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )?;
    }
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
