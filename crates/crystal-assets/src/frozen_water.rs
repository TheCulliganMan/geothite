//! Slippery northern lakes with original Crystal Ice Path rock boundaries.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, MapModule};

type Foregrounds = BTreeMap<u8, [[u8; 3]; 3]>;

#[derive(Deserialize)]
struct Plan {
    schema_version: u8,
    base_content_hash: String,
    rock_source: RockSource,
    art: Vec<Art>,
    maps: Vec<Map>,
}
#[derive(Deserialize)]
struct RockSource {
    tileset: String,
    block: usize,
    palette: String,
    index_map: [u8; 4],
}
#[derive(Deserialize)]
struct Art {
    tileset: String,
    expected_blocks_sha256: String,
    recipes: Vec<Recipe>,
}
#[derive(Deserialize)]
struct Recipe {
    id: u16,
    source_block: usize,
    quadrants: [Option<String>; 4],
}
#[derive(Deserialize)]
struct Map {
    id: String,
    old_tileset: String,
    tileset: String,
    expected_blocks_sha256: String,
    changes: Vec<Cell>,
}
#[derive(Deserialize)]
struct Cell {
    at: [u16; 2],
    before: u16,
    after: u16,
}

/// Freezes lakes on the already snowy inland routes and Rainy River icefields.
/// The complete warm North Shore remains unchanged.
///
/// Imports original Ice Path rock pixels and indoor colors into unused private
/// winter glyph slots. Existing glyphs and collision remain exact. New mixed
/// metatiles replace water with native sliding ICE, buoys/frozen waterfalls with
/// solid rocks, and the existing walls beside ice with the same cave rocks.
/// Floor approaches, trainers, fruit trees, events and connections are retained.
/// Edge ice requires a real in-range map connection; all other edges are solid.
///
/// # Errors
/// Rejects stale inputs, occupied glyphs/recipes, unsupported terrain changes,
/// altered native rock art, unsafe ice edges or an invalid resulting pack.
pub fn build_frozen_water(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::river_content_lineage::validate(base)?;
    let plan: Plan = serde_json::from_str(source)?;
    ensure!(
        plan.schema_version == 1 && plan.base_content_hash == base.identity()?.content_hash,
        "frozen-water parent differs"
    );
    ensure!(
        plan.rock_source.tileset == "ice_path"
            && plan.rock_source.block == 0x19
            && plan.rock_source.palette == "indoor"
            && plan.rock_source.index_map == [1, 2, 3, 3]
            && plan.art.len() == 3
            && plan.maps.len() == 7,
        "reviewed native winter extent required"
    );
    let mut pack = base.clone();
    for art in &plan.art {
        append_art(base, &mut pack, art)?;
    }
    let mut names = BTreeSet::new();
    for row in &plan.maps {
        ensure!(
            names.insert(&row.id)
                && matches!(
                    row.id.as_str(),
                    "MnR12C00R00"
                        | "MnR12C00R01"
                        | "MnR13C00R00"
                        | "MnR13C00R01"
                        | "MnR17C00R00"
                        | "MnR17C00R01"
                        | "MnRainyRiverIcefields"
                )
                && matches!(
                    row.old_tileset.as_str(),
                    "mn_winter_outdoor" | "mn_winter_gunflint" | "mn_winter_north_shore"
                )
                && row.tileset == row.old_tileset,
            "duplicate regional map"
        );
        let old = base.data.maps.get(&row.id).context("frozen map")?;
        ensure!(
            old.attributes.tileset_name == row.old_tileset
                && hash(&block_bytes(&old.blocks)?) == row.expected_blocks_sha256,
            "frozen map preimage differs: {}",
            row.id
        );
        let mut map = old.clone();
        map.attributes.tileset_name = row.tileset.clone();
        map.attributes.border_block = u8::try_from(
            plan.art
                .iter()
                .find(|art| art.tileset == row.tileset)
                .and_then(|art| {
                    art.recipes.iter().find(|recipe| {
                        recipe
                            .quadrants
                            .iter()
                            .all(|q| q.as_deref() == Some("rock"))
                    })
                })
                .context("solid native cave-rock border recipe")?
                .id,
        )?;
        let mut seen = BTreeSet::new();
        for cell in &row.changes {
            let [x, y] = cell.at;
            ensure!(
                x < map.attributes.width && y < map.attributes.height && seen.insert(cell.at),
                "invalid frozen cell"
            );
            let index = usize::from(y) * usize::from(map.attributes.width) + usize::from(x);
            ensure!(
                map.blocks[index] == cell.before,
                "frozen cell preimage differs"
            );
            map.blocks[index] = cell.after;
        }
        for y in 0..map.attributes.height * 2 {
            for x in 0..map.attributes.width * 2 {
                if crate::authored_town_geometry::kind(base, old, [x, y])? == "WALL"
                    && crate::authored_town_geometry::kind(&pack, &map, [x, y])? == "FLOOR"
                {
                    ensure!(
                        row.id == "MnR17C00R01" && matches!([x, y], [22, 19] | [23, 19]),
                        "unreviewed rock-rim opening"
                    );
                }
            }
        }
        crate::authored_town_publication::replace(&mut pack, map)?;
    }
    for name in names {
        closed_ice_edges(&pack, &pack.data.maps[name])?;
    }
    ensure!(
        pack.data.fruit_trees == base.data.fruit_trees
            && pack.data.trainers == base.data.trainers
            && pack.compiled_audio == base.compiled_audio
            && pack.audio_manifest == base.audio_manifest,
        "frozen terrain changed unrelated activities/audio"
    );
    pack.report
        .manifests
        .push("minnesota-frozen-water-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::river_content_lineage::validate(&pack)?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn block_bytes(blocks: &[u16]) -> Result<Vec<u8>> {
    blocks
        .iter()
        .copied()
        .map(|v| u8::try_from(v).map_err(Into::into))
        .collect()
}
fn physical(glyph: u8, attribute: u8, count: usize) -> usize {
    if attribute & 8 == 0 {
        usize::from(glyph)
    } else {
        usize::from(glyph & 127) + count / 2
    }
}

fn indoor_colors(base: &CompiledGamePack) -> Result<Vec<[[u8; 3]; 4]>> {
    let text = std::str::from_utf8(&base.runtime_files["gfx/tilesets/bg_tiles.pal"])?;
    let mut indoor = false;
    let mut colors = Vec::new();
    for line in text.lines().map(str::trim) {
        if let Some(section) = line.strip_prefix(';') {
            indoor = section.trim() == "indoor";
        }
        if !indoor {
            continue;
        }
        if let Some(values) = line.strip_prefix("RGB") {
            let values = values
                .split(';')
                .next()
                .context("native palette line")?
                .split(',')
                .map(|v| v.trim().parse::<u8>())
                .collect::<std::result::Result<Vec<_>, _>>()?;
            ensure!(
                values.len().is_multiple_of(3) && values.iter().all(|v| *v <= 31),
                "native palette range"
            );
            for rgb in values.as_chunks::<3>().0 {
                colors.push(rgb.map(|v| (v << 3) | (v >> 2)));
            }
        }
    }
    ensure!(colors.len() == 32, "native indoor palette extent");
    Ok(colors.as_chunks::<4>().0.to_vec())
}

fn append_art(base: &CompiledGamePack, output: &mut CompiledGamePack, art: &Art) -> Result<()> {
    let name = &art.tileset;
    ensure!(
        matches!(
            name.as_str(),
            "mn_winter_outdoor" | "mn_winter_gunflint" | "mn_winter_north_shore"
        ),
        "private winter art required"
    );
    let meta_path = format!("data/tilesets/{name}_metatiles.bin");
    let gfx_path = format!("gfx/tilesets/{name}.2bpp");
    let fg_path = format!("data/tilesets/{name}_foregrounds.json");
    let old_blocks = &base.runtime_files[&meta_path];
    let old_graphics = &base.runtime_files[&gfx_path];
    ensure!(
        hash(old_blocks) == art.expected_blocks_sha256 && old_graphics.len() == 4096,
        "winter art preimage differs"
    );
    let mut definition = base.data.tilesets[name].clone();
    let mut blocks = old_blocks.clone();
    let mut graphics = old_graphics.clone();
    let mut foregrounds: Foregrounds = serde_json::from_slice(&base.runtime_files[&fg_path])?;
    let used = old_blocks.iter().copied().collect::<BTreeSet<_>>();
    let used_physical = used
        .iter()
        .map(|&g| physical(g, definition.palette_map[usize::from(g)], 256))
        .collect::<BTreeSet<_>>();
    let mut free = (96_u8..=255)
        .filter(|g| {
            !used.contains(g)
                && !used_physical.contains(&usize::from(*g))
                && !foregrounds.contains_key(g)
        })
        .collect::<VecDeque<_>>();
    let mut shared = BTreeMap::new();
    let donor = &base.data.tilesets["ice_path"];
    ensure!(
        donor.collision["19"].iter().all(|k| k == "WALL"),
        "native cave rock collision differs"
    );
    let donor_blocks = &base.runtime_files["data/tilesets/ice_path_metatiles.bin"];
    let donor_graphics = &base.runtime_files["gfx/tilesets/ice_path.2bpp"];
    let colors = indoor_colors(base)?;
    let snow_block = if name == "mn_winter_north_shore" {
        0x83
    } else {
        0x3c
    };
    let snow_white = foregrounds
        .get(&old_blocks[snow_block * 16])
        .context("winter snow foreground")?[0];
    let mut rock = Vec::new();
    for &g in &donor_blocks[0x19 * 16..0x1a * 16] {
        let attribute = donor.palette_map[usize::from(g)];
        let slot = physical(g, attribute, donor_graphics.len() / 16);
        let mut pixels: [u8; 16] = donor_graphics[slot * 16..(slot + 1) * 16].try_into()?;
        // Private foregrounds own three opaque colors. Put source zero into
        // snowy white instead of inheriting the outdoor lawn background.
        for row in 0..8 {
            let low = pixels[row * 2];
            let high = pixels[row * 2 + 1];
            pixels[row * 2] = 0;
            pixels[row * 2 + 1] = 0;
            for column in 0..8 {
                let bit = 7 - column;
                let index = ((low >> bit) & 1) | (((high >> bit) & 1) << 1);
                let mapped = [1, 2, 3, 3][usize::from(index)];
                pixels[row * 2] |= (mapped & 1) << bit;
                pixels[row * 2 + 1] |= (mapped >> 1) << bit;
            }
        }
        let palette = colors[usize::from(attribute & 7)];
        let foreground = [snow_white, palette[1], palette[3]];
        let glyph = if let Some(&g) = shared.get(&(pixels, foreground)) {
            g
        } else {
            let new = free.pop_front().context("winter rock glyph capacity")?;
            graphics[usize::from(new) * 16..(usize::from(new) + 1) * 16].copy_from_slice(&pixels);
            definition.palette_map[usize::from(new)] = if new < 128 { 3 } else { 11 };
            foregrounds.insert(new, foreground);
            shared.insert((pixels, foreground), new);
            new
        };
        rock.push(glyph);
    }
    let ice = if name == "mn_winter_north_shore" {
        0x85
    } else {
        0x3e
    };
    let snow = snow_block;
    ensure!(
        definition.collision[&format!("{ice:02x}")]
            .iter()
            .all(|k| k == "ICE"),
        "native sliding ice differs"
    );
    for recipe in &art.recipes {
        ensure!(
            usize::from(recipe.id) == blocks.len() / 16
                && recipe.id < 256
                && recipe.source_block * 16 < old_blocks.len(),
            "frozen recipe address differs"
        );
        let mut pixels =
            old_blocks[recipe.source_block * 16..(recipe.source_block + 1) * 16].to_vec();
        let mut collision = definition.collision[&format!("{:02x}", recipe.source_block)].clone();
        for (q, material) in recipe.quadrants.iter().enumerate() {
            let Some(material) = material else { continue };
            let donor_pixels = match material.as_str() {
                "ice" => {
                    ensure!(
                        matches!(collision[q].as_str(), "WATER" | "WATER_21"),
                        "only water may freeze"
                    );
                    collision[q] = "ICE".into();
                    &old_blocks[ice * 16..(ice + 1) * 16]
                }
                "snow" => {
                    ensure!(collision[q] == "FLOOR", "only Floor may receive snow");
                    &old_blocks[snow * 16..(snow + 1) * 16]
                }
                "ramp" => {
                    ensure!(
                        collision[q] == "WALL",
                        "snow entry needs a reviewed old rock rim"
                    );
                    collision[q] = "FLOOR".into();
                    &old_blocks[snow * 16..(snow + 1) * 16]
                }
                "rock" => {
                    ensure!(
                        matches!(
                            collision[q].as_str(),
                            "WALL" | "BUOY" | "WATERFALL" | "WATER" | "WATER_21" | "ICE"
                        ),
                        "rock obstructs an original path"
                    );
                    collision[q] = "WALL".into();
                    rock.as_slice()
                }
                _ => anyhow::bail!("unknown frozen material"),
            };
            for dy in 0..2 {
                for dx in 0..2 {
                    let i = (q / 2 * 2 + dy) * 4 + q % 2 * 2 + dx;
                    pixels[i] = donor_pixels[i];
                }
            }
        }
        blocks.extend(pixels);
        ensure!(
            definition
                .collision
                .insert(format!("{:02x}", recipe.id), collision)
                .is_none(),
            "occupied frozen collision"
        );
    }
    for &g in &used {
        let slot = physical(g, definition.palette_map[usize::from(g)], 256);
        ensure!(
            definition.palette_map[usize::from(g)]
                == base.data.tilesets[name].palette_map[usize::from(g)]
                && graphics[slot * 16..(slot + 1) * 16] == old_graphics[slot * 16..(slot + 1) * 16],
            "existing winter glyph altered"
        );
    }
    output.runtime_files.insert(meta_path, blocks);
    output.runtime_files.insert(gfx_path, graphics.clone());
    output.runtime_files.insert(
        format!("gfx/tilesets/{name}.png"),
        crate::native_tile_png::from_2bpp(&graphics)?,
    );
    output
        .runtime_files
        .insert(fg_path, serde_json::to_vec(&foregrounds)?);
    output.runtime_files.insert(
        format!("data/tilesets/{name}.json"),
        serde_json::to_vec(&definition.collision)?,
    );
    output.runtime_files.insert(
        format!("data/tilesets/{name}_palette_map.json"),
        serde_json::to_vec(&definition.palette_map)?,
    );
    output.data.tilesets.insert(name.clone(), definition);
    Ok(())
}

fn closed_ice_edges(pack: &CompiledGamePack, map: &MapModule) -> Result<()> {
    let a = &map.attributes;
    for y in 0..a.height * 2 {
        for x in 0..a.width * 2 {
            if crate::authored_town_geometry::kind(pack, map, [x, y])? != "ICE" {
                continue;
            }
            for direction in ["north", "south", "west", "east"] {
                if !match direction {
                    "north" => y == 0,
                    "south" => y == a.height * 2 - 1,
                    "west" => x == 0,
                    _ => x == a.width * 2 - 1,
                } {
                    continue;
                }
                let edge = a
                    .connections
                    .iter()
                    .find(|e| e.direction == direction)
                    .context("ice reaches an unconnected edge")?;
                let b = &pack.data.maps[&edge.target_map].attributes;
                let coordinate = if matches!(direction, "north" | "south") {
                    i32::from(x)
                } else {
                    i32::from(y)
                } - i32::from(edge.offset) * 2;
                let extent = if matches!(direction, "north" | "south") {
                    b.width * 2
                } else {
                    b.height * 2
                };
                ensure!(
                    coordinate >= 0 && coordinate < i32::from(extent),
                    "ice reaches an out-of-range connection"
                );
            }
        }
    }
    Ok(())
}
