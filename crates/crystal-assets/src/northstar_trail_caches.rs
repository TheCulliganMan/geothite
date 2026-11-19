//! Native terrain clues for the North Shore, Itasca and Tamarack hidden items.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::authored_route_design::RouteDesign;
use crate::authored_town_geometry::kind;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::refresh_event_section;
use crate::{CompiledGamePack, native_tile_png, northstar_regional_design};

const ART: &[u8] =
    include_bytes!("../../../planning/minnesota/revisions/trail-landmarks-v01/cache-art.json");
const ROOMS: &[u8] =
    include_bytes!("../../../planning/minnesota/revisions/trail-landmarks-v01/caches.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Landmark {
    name: String,
    pub(super) tileset: String,
    pub(super) block: u8,
    pub(super) collision_from: u8,
    pub(super) background: u8,
    pub(super) foreground: Option<u8>,
    rows: [String; 32],
}

impl Landmark {
    pub(super) fn validate_rows(&self) -> Result<()> {
        ensure!(
            self.rows.iter().all(|row| row.len() == 32
                && row.bytes().all(|pixel| matches!(pixel, b'.' | b'0'..=b'3'))),
            "invalid indexed art for {}",
            self.name
        );
        Ok(())
    }

    /// Composites one 8×8 tile over its original two-bit pixel planes.
    pub(super) fn painted_glyph(&self, index: usize, mut pixels: [u8; 16]) -> Option<[u8; 16]> {
        let x = index % 4 * 8;
        let y = index / 4 * 8;
        let rows = &self.rows[y..y + 8];
        if rows
            .iter()
            .all(|row| row.as_bytes()[x..x + 8] == *b"........")
        {
            return None;
        }
        for (dy, row) in rows.iter().enumerate() {
            for (dx, pixel) in row.as_bytes()[x..x + 8].iter().copied().enumerate() {
                if pixel == b'.' {
                    continue;
                }
                let mask = 1 << (7 - dx);
                for (plane, byte) in pixels[2 * dy..2 * dy + 2].iter_mut().enumerate() {
                    *byte &= !mask;
                    if ((pixel - b'0') >> plane) & 1 != 0 {
                        *byte |= mask;
                    }
                }
            }
        }
        Some(pixels)
    }
}

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    install_landmark_art(pack, ART)?;
    northstar_regional_design::apply_rooms(pack, ROOMS)
}

/// Aligns the prairie boards and publishes the Birch Walker clue and Cedar Ridge cairn.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    const CLUE: &str = "MnDiscoveryR08ClueText";
    align_prairie_boards(pack)?;
    let mut grove = pack
        .data
        .maps
        .get("MnR08C00R00")
        .context("Route 8 Birch Walker map")?
        .clone();
    let story: serde_json::Value =
        serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))?;
    let pages: Vec<AuthoredDialoguePage> =
        serde_json::from_value(story["story_flow_revision"]["dialogue"][&grove.id][CLUE].clone())
            .context("canonical Birch Walker pages")?;
    let updated = authored_dialogue::compile(&pages)?;
    let prior = grove.scripts.get_mut(CLUE).context("Birch Walker clue")?;
    ensure!(*prior != updated, "expected the original flower-bed clue");
    *prior = updated;
    grove.script_text_bodies = crate::parse_script_text_bodies(&grove.id, &grove.scripts)?;
    authored_town_publication::replace(pack, grove)?;

    let ridge = pack.data.maps.get("MnR13C00R00").context("Cedar Ridge")?;
    ensure!(
        ridge.attributes.tileset_name == "mn_outdoor_winter_stairs"
            && ridge.events.bg_events.iter().any(|event| {
                event.script == "MN_HIDDEN_R13"
                    && event.event_type == "BGEVENT_ITEM"
                    && [event.x, event.y] == [38, 32]
            }),
        "expected the hidden Hyper Potion at the Cedar Ridge trail marker"
    );
    install_landmark_art_at(
        pack,
        include_bytes!(
            "../../../planning/minnesota/revisions/trail-discoveries-v01/cedar-cairn.json"
        ),
        0xc0,
    )?;
    northstar_regional_design::apply_rooms(
        pack,
        br#"[{"map":"MnR13C00R00","blocks":[{"at":[19,16],"from":136,"to":142}]}]"#,
    )
}

fn align_prairie_boards(pack: &mut CompiledGamePack) -> Result<()> {
    let route: RouteDesign = serde_json::from_str(include_str!(
        "../../../planning/minnesota/routes/MN_R02.json"
    ))?;
    let mut road = pack
        .data
        .maps
        .get("MnR02C00R00")
        .context("Prairie Farm Road")?
        .clone();
    ensure!(route.signs.len() == 5, "expected five prairie noticeboards");
    for sign in route.signs {
        let [x, y] = sign.tile;
        ensure!(
            kind(pack, &road, sign.tile)? == "WALL"
                && !road
                    .events
                    .bg_events
                    .iter()
                    .any(|e| [e.x, e.y] == sign.tile),
            "occupied or walkable noticeboard at {x},{y}"
        );
        let read = road
            .events
            .bg_events
            .iter_mut()
            .find(|e| e.script == sign.id)
            .with_context(|| format!("missing {}", sign.id))?;
        ensure!(
            read.event_type == "BGEVENT_READ" && read.y == y && read.x.checked_add(1) == Some(x),
            "{} is no longer on the left post",
            sign.id
        );
        read.x = x;
    }
    refresh_event_section(&mut road);
    let events = road
        .attributes
        .map_events_label
        .as_ref()
        .context("prairie map event label")?;
    if let Some(raw) = road.scripts.get_mut(events) {
        *raw = road
            .map_event_section_commands
            .iter()
            .map(|row| serde_json::json!({"command":row.command,"args":row.args}))
            .collect();
    }
    authored_town_publication::replace(pack, road)
}

pub(super) fn install_landmark_art(pack: &mut CompiledGamePack, source: &[u8]) -> Result<()> {
    install_landmark_art_at(pack, source, 0xc4)
}

/// Installs indexed terrain artwork into free glyph positions in each named atlas.
pub(super) fn install_landmark_art_at(
    pack: &mut CompiledGamePack,
    source: &[u8],
    first_glyph: u8,
) -> Result<()> {
    let designs: Vec<Landmark> = serde_json::from_slice(source)?;
    for sequence in designs.chunk_by(|a, b| a.tileset == b.tileset) {
        install_art(pack, &sequence[0].tileset, sequence, first_glyph)?;
    }
    Ok(())
}

pub(super) fn install_art(
    pack: &mut CompiledGamePack,
    tileset: &str,
    designs: &[Landmark],
    first_glyph: u8,
) -> Result<()> {
    let graphic_path = format!("gfx/tilesets/{tileset}.2bpp");
    let block_path = format!("data/tilesets/{tileset}_metatiles.bin");
    let shade_path = format!("data/tilesets/{tileset}_foregrounds.json");
    let mut blocks = pack
        .runtime_files
        .get(&block_path)
        .context("landmark metatiles")?
        .clone();
    let graphics = pack
        .runtime_files
        .get_mut(&graphic_path)
        .context("landmark art sheet")?;
    let tiles = graphics.len() / 16;
    ensure!(
        tiles == 192 || tiles == 256,
        "unsupported outdoor tile banks"
    );
    let definition = pack
        .data
        .tilesets
        .get_mut(tileset)
        .context("outdoor tileset")?;
    let addr = |glyph: u8, palette: u8| {
        if palette & 8 == 0 {
            usize::from(glyph)
        } else {
            usize::from(glyph & 127) + tiles / 2
        }
    };
    // Each stamp uses spare glyphs in the second bank, never changing the
    // physical bytes referenced by another block (including bank aliases).
    let mut occupied = vec![false; tiles];
    for &glyph in &blocks {
        let palette = definition
            .palette_map
            .get(usize::from(glyph))
            .context("native glyph palette")?;
        let physical = addr(glyph, *palette);
        *occupied
            .get_mut(physical)
            .context("native glyph outside art sheet")? = true;
    }
    let mut next = first_glyph;
    let mut palettes = Vec::new();
    for design in designs {
        ensure!(
            blocks.len() == usize::from(design.block) * 16,
            "unexpected block for {}",
            design.name
        );
        design.validate_rows()?;
        let start = usize::from(design.background) * 16;
        let mut block: [u8; 16] = blocks
            .get(start..start + 16)
            .context("landmark background")?
            .try_into()?;
        for (index, glyph) in block.iter_mut().enumerate() {
            let source = *glyph;
            let old = addr(source, definition.palette_map[usize::from(source)]) * 16;
            let original = graphics[old..old + 16].try_into()?;
            let Some(painted) = design.painted_glyph(index, original) else {
                continue;
            };
            let destination = addr(next, 8);
            ensure!(
                destination < tiles && !occupied[destination] && !blocks.contains(&next),
                "landmark would overwrite active art on {tileset}"
            );
            graphics[destination * 16..destination * 16 + 16].copy_from_slice(&painted);
            let donor = design.foreground.unwrap_or(source);
            definition.palette_map[usize::from(next)] =
                8 | (definition.palette_map[usize::from(donor)] & 7);
            palettes.push((next, donor));
            occupied[destination] = true;
            *glyph = next;
            next = next
                .checked_add(1)
                .context("trail art exhausted native glyphs")?;
        }
        blocks.extend(block);
        let collision = definition
            .collision
            .get(&format!("{:02x}", design.collision_from))
            .context("landmark collision source")?
            .clone();
        ensure!(
            definition
                .collision
                .insert(format!("{:02x}", design.block), collision)
                .is_none(),
            "occupied trail metatile"
        );
    }
    let png = native_tile_png::from_2bpp(graphics)?;
    pack.runtime_files
        .insert(format!("gfx/tilesets/{tileset}.png"), png);
    pack.runtime_files.insert(block_path, blocks);
    pack.runtime_files.insert(
        format!("data/tilesets/{tileset}.json"),
        serde_json::to_vec(&definition.collision)?,
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{tileset}_palette_map.json"),
        serde_json::to_vec(&definition.palette_map)?,
    );
    if let Some(bytes) = pack.runtime_files.get_mut(&shade_path) {
        let mut shades: BTreeMap<u8, [[u8; 3]; 3]> = serde_json::from_slice(bytes)?;
        for (destination, donor) in palettes {
            if let Some(colors) = shades.get(&donor).copied() {
                shades.insert(destination, colors);
            } else {
                shades.remove(&destination);
            }
        }
        *bytes = serde_json::to_vec(&shades)?;
    }
    Ok(())
}
