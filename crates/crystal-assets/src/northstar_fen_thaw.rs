//! Join the green Fen to the snowy Northwoods along their shared footpath.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::{CompiledGamePack, authored_town_publication, native_tile_png};

const SOURCE: &str =
    include_str!("../../../planning/minnesota/revisions/fen-clearings-v01/thaw.json");
const PINE: [u8; 6] = [0x1e, 0x1f, 0x2e, 0x2f, 0x3e, 0x3f];
type Inks = [[u8; 3]; 3];

#[derive(Deserialize)]
struct Edge {
    map: String,
    tileset: String,
    crowns: Vec<Crown>,
    track: Vec<Track>,
    terrain: Vec<Terrain>,
}

#[derive(Deserialize)]
struct Crown {
    first: u8,
    foregrounds5: Option<Inks>,
}

#[derive(Deserialize)]
struct Track {
    slot: u8,
    palette: u8,
    #[serde(flatten)]
    marks: Marks,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Marks {
    Source(u8),
    Rows([String; 8]),
    MeltSpan([u8; 8]),
}

#[derive(Deserialize)]
struct Terrain {
    name: String,
    from_block: u8,
    tiles: [u8; 16],
    at: Vec<[usize; 2]>,
}

struct Glyph {
    slot: u8,
    palette: u8,
    ink: [u8; 16],
    snow_background: bool,
    foregrounds: Option<Inks>,
}

fn source_glyph(johto: &[u8], index: u8) -> Result<[u8; 16]> {
    let start = usize::from(index) * 16;
    Ok(johto
        .get(start..start + 16)
        .context("native Johto glyph")?
        .try_into()?)
}

// Keep the precise road gravel at native color 2. The two ground colors show
// the wet track passing into retained snow without moving a single pebble.
fn melt_ink(johto: &[u8], widths: [u8; 8]) -> Result<[u8; 16]> {
    let road = source_glyph(johto, 0x06)?;
    let mut ink = [0; 16];
    for (y, width) in widths.into_iter().enumerate() {
        ensure!(
            width <= 8 && road[2 * y + 1] == 0,
            "unexpected native gravel or thaw width"
        );
        let bare = (0xff00u16 >> width) as u8;
        ink[2 * y] = bare & !road[2 * y];
        ink[2 * y + 1] = road[2 * y];
    }
    Ok(ink)
}

fn track_ink(rows: [String; 8]) -> Result<[u8; 16]> {
    let mut bytes = [0; 16];
    for (y, row) in rows.iter().enumerate() {
        ensure!(
            row.len() == 8 && row.bytes().all(|c| matches!(c, b'0'..=b'3')),
            "thaw verge needs eight pixels of four shades"
        );
        for (x, c) in row.bytes().enumerate() {
            let index = c - b'0';
            bytes[y * 2] |= (index & 1) << (7 - x);
            bytes[y * 2 + 1] |= (index >> 1) << (7 - x);
        }
    }
    Ok(bytes)
}

fn read_table<T: serde::de::DeserializeOwned + Default>(
    pack: &CompiledGamePack,
    path: &str,
) -> Result<T> {
    Ok(pack
        .runtime_files
        .get(path)
        .map(|b| serde_json::from_slice(b))
        .transpose()?
        .unwrap_or_default())
}

fn install(pack: &mut CompiledGamePack, edge: Edge, johto: &[u8]) -> Result<()> {
    let bank = &edge.tileset;
    let mut room = pack
        .data
        .maps
        .get(&edge.map)
        .context("Fen seam room")?
        .clone();
    ensure!(
        room.attributes.tileset_name == *bank,
        "unexpected terrain at {}",
        edge.map
    );
    let file = |suffix: &str| format!("data/tilesets/{bank}{suffix}");
    let foreground_file = file("_foregrounds.json");
    let background_file = file("_native_backgrounds.json");
    let mut foregrounds: BTreeMap<u8, Inks> = read_table(pack, &foreground_file)?;
    let mut backgrounds: BTreeMap<u8, u8> = read_table(pack, &background_file)?;
    let mut animated = BTreeSet::from([0x03, 0x14]);
    for path in [
        file("_animations.json"),
        "data/tilesets/johto_animations.json".to_owned(),
    ] {
        let clips: BTreeMap<u8, serde_json::Value> = read_table(pack, &path)?;
        animated.extend(clips.into_keys());
    }

    let mut glyphs = Vec::new();
    for crown in edge.crowns {
        ensure!(
            crown
                .foregrounds5
                .as_ref()
                .is_none_or(|inks| inks.iter().flatten().all(|&c| c < 32)),
            "pine palette must fit native five-bit channels"
        );
        let foregrounds = crown
            .foregrounds5
            .map(|colors| colors.map(|rgb| rgb.map(|c| (c << 3) | (c >> 2))));
        for (i, src) in PINE.iter().enumerate() {
            glyphs.push(Glyph {
                slot: crown.first + u8::try_from(i)?,
                palette: 2,
                ink: source_glyph(johto, *src)?,
                foregrounds,
                snow_background: false,
            });
        }
    }
    for track in edge.track {
        let (ink, snow_background) = match track.marks {
            Marks::Source(src) => (source_glyph(johto, src)?, false),
            Marks::Rows(rows) => (track_ink(rows)?, false),
            Marks::MeltSpan(rows) => (melt_ink(johto, rows)?, true),
        };
        // Native Johto's daylight track color and neutral gravel. Northwoods'
        // existing snow palette provides color zero and lights the inks at night.
        let road_inks = [[222, 255, 222], [173, 173, 173], [107, 107, 107]];
        glyphs.push(Glyph {
            slot: track.slot,
            palette: track.palette,
            ink,
            snow_background,
            foregrounds: snow_background.then_some(road_inks),
        });
    }

    let sheet_path = format!("gfx/tilesets/{bank}.2bpp");
    let mut graphics = pack
        .runtime_files
        .get(&sheet_path)
        .context("Fen seam graphics")?
        .clone();
    let mut atlas = pack
        .runtime_files
        .get(&file("_metatiles.bin"))
        .context("Fen seam metatiles")?
        .clone();
    let definition = pack.data.tilesets.get_mut(bank).context("seam tileset")?;
    ensure!(
        graphics.len() == 4096 && atlas.len().is_multiple_of(16),
        "expected two native glyph pages"
    );
    let mut occupied: BTreeSet<u8> = atlas.iter().copied().chain(animated).collect();
    for g in glyphs {
        let begin = usize::from(g.slot) * 16;
        ensure!(
            g.slot < 128
                && g.palette < 8
                && occupied.insert(g.slot)
                && graphics[begin..begin + 16].iter().all(|&b| b == 0)
                && !foregrounds.contains_key(&g.slot)
                && !backgrounds.contains_key(&g.slot),
            "occupied native seam glyph {:#x} in {bank}",
            g.slot
        );
        graphics[begin..begin + 16].copy_from_slice(&g.ink);
        definition.palette_map[usize::from(g.slot)] = g.palette;
        // Palette zero follows the shared native clock; wet ground continues through the seam.
        if !g.snow_background {
            backgrounds.insert(g.slot, 0);
        }
        if let Some(colors) = g.foregrounds {
            foregrounds.insert(g.slot, colors);
        }
    }

    for terrain in edge.terrain {
        ensure!(
            terrain.tiles.iter().all(|id| occupied.contains(id)),
            "unknown native glyph in {}",
            terrain.name
        );
        let id = u8::try_from(atlas.len() / 16).context("full Fen seam metatile bank")?;
        atlas.extend(terrain.tiles);
        let roles = definition
            .collision
            .get(&format!("{:02x}", terrain.from_block))
            .context("native seam permissions")?
            .clone();
        ensure!(
            definition
                .collision
                .insert(format!("{id:02x}"), roles)
                .is_none(),
            "occupied seam metatile"
        );
        for [x, y] in terrain.at {
            ensure!(
                x < usize::from(room.attributes.width) && y < usize::from(room.attributes.height),
                "seam outside map"
            );
            let slot = &mut room.blocks[y * usize::from(room.attributes.width) + x];
            ensure!(
                *slot == u16::from(terrain.from_block),
                "{}: changed seam at {x},{y}",
                edge.map
            );
            *slot = u16::from(id);
        }
    }
    pack.runtime_files
        .insert(file(".json"), serde_json::to_vec(&definition.collision)?);
    pack.runtime_files.insert(
        file("_palette_map.json"),
        serde_json::to_vec(&definition.palette_map)?,
    );
    pack.runtime_files.insert(file("_metatiles.bin"), atlas);
    pack.runtime_files
        .insert(background_file, serde_json::to_vec(&backgrounds)?);
    if !foregrounds.is_empty() {
        pack.runtime_files
            .insert(foreground_file, serde_json::to_vec(&foregrounds)?);
    }
    pack.runtime_files.insert(
        format!("gfx/tilesets/{bank}.png"),
        native_tile_png::from_2bpp(&graphics)?,
    );
    pack.runtime_files.insert(sheet_path, graphics);
    authored_town_publication::replace(pack, room)
}

/// Preserve the trail's passage and interactions while its pines gradually take on frost.
/// Returns an error if the native donor art, room or available bank slots have changed.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let johto = pack
        .runtime_files
        .get("gfx/tilesets/johto.2bpp")
        .context("source pine art")?
        .clone();
    for edge in serde_json::from_str::<[Edge; 2]>(SOURCE)? {
        install(pack, edge, &johto)?;
    }
    Ok(())
}
