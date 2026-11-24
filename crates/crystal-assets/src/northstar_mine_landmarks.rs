//! Follow the working haul rails, vent housings, and discharge line through the Iron Range mine.
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde::Deserialize;
use serde_json::json;

use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Junction {
    map: String,
    fixtures: BTreeMap<String, Vec<[u16; 3]>>,
    // Metatile x, y, original block, and the native (block, quadrant) of each quarter.
    bank: Vec<(u16, u16, u16, [[u16; 2]; 4])>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    palette: u8,
    rows: [String; 16],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Approach {
    map: String,
    plates: Vec<ApproachPlate>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ApproachPlate {
    tile: [u16; 3],
    kind: String,
    shaft: Option<[u16; 2]>,
    lines: Vec<String>,
}

#[derive(Clone, PartialEq)]
struct Block {
    glyphs: [u8; 16],
    collision: [String; 4],
}

fn positions(quadrant: usize) -> [usize; 4] {
    let start = quadrant / 2 * 8 + quadrant % 2 * 2;
    [start, start + 1, start + 4, start + 5]
}

// All existing mine banks have two native 128-glyph pages. Allocate only low-bank
// slots that no old glyph addresses. Identical ink and palette share one slot.
fn store_glyph(
    ink: &[u8; 16],
    palette: u8,
    graphics: &mut [u8],
    palettes: &mut [u8],
    used: &mut [bool; 128],
) -> Result<u8> {
    for glyph in 0x41_u8..0x80 {
        let i = usize::from(glyph);
        if used[i] && palettes[i] == palette && &graphics[i * 16..(i + 1) * 16] == ink {
            return Ok(glyph);
        }
    }
    let glyph = (0x41_u8..0x80)
        .find(|&g| !used[usize::from(g)])
        .context("no free native mine glyph")?;
    let i = usize::from(glyph);
    used[i] = true;
    palettes[i] = palette;
    graphics[i * 16..(i + 1) * 16].copy_from_slice(ink);
    Ok(glyph)
}

fn overlay(
    block: &mut Block,
    quadrant: usize,
    art: &Fixture,
    graphics: &mut [u8],
    palettes: &mut [u8],
    used: &mut [bool; 128],
) -> Result<()> {
    ensure!(art.palette < 8, "invalid mine palette");
    for (subtile, at) in positions(quadrant).into_iter().enumerate() {
        let glyph = block.glyphs[at];
        let slot = if palettes[usize::from(glyph)] & 8 == 0 {
            usize::from(glyph)
        } else {
            usize::from(glyph & 127) + 128
        };
        let mut ink: [u8; 16] = graphics[slot * 16..(slot + 1) * 16].try_into()?;
        for y in 0..8 {
            let row = art.rows[subtile / 2 * 8 + y].as_bytes();
            ensure!(row.len() == 16, "mine pixel row width");
            for x in 0..8 {
                let tone = row[subtile % 2 * 8 + x];
                if tone == b'.' {
                    continue;
                }
                ensure!((b'0'..=b'3').contains(&tone), "invalid mine ink");
                let mask = 1 << (7 - x);
                for plane in 0..2 {
                    let bits = &mut ink[2 * y + plane];
                    *bits &= !mask;
                    if (tone - b'0') & (1 << plane) != 0 {
                        *bits |= mask;
                    }
                }
            }
        }
        block.glyphs[at] = store_glyph(&ink, art.palette, graphics, palettes, used)?;
    }
    Ok(())
}

fn coordinate(room: &MapModule, x: u16, y: u16, expected: u16) -> Result<usize> {
    let a = &room.attributes;
    ensure!(x < a.width && y < a.height, "outside {}", room.id);
    let i = usize::from(y) * usize::from(a.width) + usize::from(x);
    ensure!(
        room.blocks.get(i) == Some(&expected)
            && !room
                .script_block_changes
                .iter()
                .any(|change| change.x / 2 == x && change.y / 2 == y),
        "unexpected or scripted mine terrain at {}, {x}, {y}",
        room.id
    );
    Ok(i)
}

fn revise(
    pack: &mut CompiledGamePack,
    spec: &Junction,
    art: &BTreeMap<String, Fixture>,
    plates: &[ApproachPlate],
) -> Result<()> {
    let mut room = pack.data.maps.get(&spec.map).context("mine room")?.clone();
    let name = &room.attributes.tileset_name;
    ensure!(
        name.starts_with("mn_iron_") && name.ends_with("_rock"),
        "private mine tileset required"
    );
    let def = pack.data.tilesets.get_mut(name).context("mine tileset")?;
    let layout_path = format!("data/tilesets/{name}_metatiles.bin");
    let gfx_path = format!("gfx/tilesets/{name}.2bpp");
    let mut layout = pack
        .runtime_files
        .get(&layout_path)
        .context("mine layout")?
        .clone();
    let graphics = pack
        .runtime_files
        .get_mut(&gfx_path)
        .context("mine graphics")?;
    ensure!(
        graphics.len() == 256 * 16
            && def.palette_map.len() >= 128
            && layout.len().is_multiple_of(16),
        "unexpected mine bank size"
    );
    let mut used = [false; 128];
    for &glyph in &layout {
        if glyph < 128 {
            // A logical low glyph could address the high bank via a palette flag.
            // Its own palette entry must still never be reassigned.
            used[usize::from(glyph)] = true;
        }
    }
    let original = |id: u16| -> Result<Block> {
        let start = usize::from(id) * 16;
        Ok(Block {
            glyphs: layout
                .get(start..start + 16)
                .context("mine art block")?
                .try_into()?,
            collision: def
                .collision
                .get(&format!("{id:02x}"))
                .context("mine collision block")?
                .clone()
                .try_into()
                .ok()
                .context("mine collision quadrants")?,
        })
    };
    let mut edits: BTreeMap<usize, Block> = BTreeMap::new();
    for &(x, y, base, donors) in &spec.bank {
        let i = coordinate(&room, x, y, base)?;
        let mut block = original(base)?;
        for (q, [donor, from]) in donors.into_iter().enumerate() {
            ensure!(from < 4, "invalid bank quadrant");
            let source = original(donor)?;
            let after = &source.collision[usize::from(from)];
            ensure!(
                block.collision[q] == *after || (block.collision[q] == "WALL" && after == "WATER"),
                "drain would displace existing mine travel"
            );
            block.collision[q].clone_from(after);
            for (dst, src) in positions(q).into_iter().zip(positions(usize::from(from))) {
                block.glyphs[dst] = source.glyphs[src];
            }
        }
        ensure!(edits.insert(i, block).is_none(), "duplicate bank placement");
    }
    let mut occupied = BTreeSet::new();
    let gravel = original(0x0d)?;
    for (kind, placements) in &spec.fixtures {
        for &[x, y, base] in placements {
            let i = coordinate(&room, x / 2, y / 2, base)?;
            ensure!(occupied.insert([x, y]), "duplicate mine fixture");
            let block = match edits.entry(i) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(original(base)?),
            };
            let q = usize::from(y % 2 * 2 + x % 2);
            let required = match kind.as_str() {
                "up" | "down" => {
                    ensure!(
                        room.events.warps.iter().any(|w| w.x == x && w.y == y),
                        "unpaired shaft"
                    );
                    for at in positions(q) {
                        block.glyphs[at] = gravel.glyphs[at];
                    }
                    "LADDER"
                }
                "rail" | "sump" | "vent" | "gallery" => "FLOOR",
                "pump" | "pipe" | "fan" => "WALL",
                _ => anyhow::bail!("unknown mine fixture {kind}"),
            };
            ensure!(
                block.collision[q] == required,
                "misplaced {kind} at {}, {x}, {y}",
                room.id
            );
            overlay(
                block,
                q,
                art.get(kind).context("mine fixture pixels")?,
                graphics,
                &mut def.palette_map,
                &mut used,
            )?;
        }
    }
    for (i, block) in edits {
        let existing = layout
            .as_chunks::<16>()
            .0
            .iter()
            .enumerate()
            .find(|(id, glyphs)| {
                **glyphs == block.glyphs
                    && def
                        .collision
                        .get(&format!("{id:02x}"))
                        .is_some_and(|c| c == &block.collision)
            });
        let id = match existing {
            Some((id, _)) => id,
            None => {
                let id = layout.len() / 16;
                ensure!(
                    id < 256
                        && def
                            .collision
                            .insert(format!("{id:02x}"), block.collision.to_vec())
                            .is_none(),
                    "native mine atlas full"
                );
                layout.extend(block.glyphs);
                id
            }
        };
        room.blocks[i] = u16::try_from(id)?;
    }
    let png = crate::native_tile_png::from_2bpp(graphics)?;
    pack.runtime_files
        .insert(format!("gfx/tilesets/{name}.png"), png);
    pack.runtime_files.insert(layout_path, layout);
    pack.runtime_files.insert(
        format!("data/tilesets/{name}.json"),
        serde_json::to_vec(&def.collision)?,
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{name}_palette_map.json"),
        serde_json::to_vec(&def.palette_map)?,
    );
    if !plates.is_empty() {
        add_approach_signs(pack, &mut room, plates)?;
    }
    authored_town_publication::replace(pack, room)
}

fn add_approach_signs(
    pack: &CompiledGamePack,
    room: &mut MapModule,
    plates: &[ApproachPlate],
) -> Result<()> {
    let mut scripts = BTreeMap::new();
    for plate in plates {
        let [x, y, _] = plate.tile;
        let target = match plate.kind.as_str() {
            "sump" => "MN_IRON_SUMP",
            "vent" => "MN_IRON_VENTILATION",
            "gallery" => "MN_IRON_PUMP_GALLERY",
            kind => anyhow::bail!("unknown mine plate {kind}"),
        };
        if let Some([sx, sy]) = plate.shaft {
            ensure!(
                x.abs_diff(sx) + y.abs_diff(sy) == 1
                    && room.events.warps.iter().any(|warp| {
                        (warp.x, warp.y) == (sx, sy) && warp.target_map_constant == target
                    }),
                "incorrect ladder destination for {} plate at {x}, {y}",
                room.id
            );
        }
        ensure!(
            !room.events.warps.iter().any(|e| (e.x, e.y) == (x, y))
                && !room
                    .events
                    .coord_events
                    .iter()
                    .any(|e| (e.x, e.y) == (x, y))
                && !room.events.bg_events.iter().any(|e| (e.x, e.y) == (x, y))
                && !room.objects.iter().any(|e| (e.x, e.y) == (x, y)),
            "occupied mine plate position in {} at {x}, {y}",
            room.id
        );
        let script = format!("{}ApproachPlate{x}_{y}", room.id);
        let label = format!("{script}Text");
        ensure!(
            !pack.data.map_scripts.contains_key(&script)
                && !pack.data.map_scripts.contains_key(&label),
            "existing mine plate script {script}"
        );
        let lines: Vec<_> = plate.lines.iter().map(String::as_str).collect();
        authoring::text(room, &label, &lines);
        let body = room.scripts.remove(&label).context("mine plate text")?;
        scripts.insert(label.clone(), body);
        scripts.insert(
            script.clone(),
            json!([authoring::command("jumptext", &[&label])]),
        );
        room.events.bg_events.push(BackgroundEvent {
            x,
            y,
            event_type: "BGEVENT_READ".into(),
            script,
        });
    }
    room.script_text_commands
        .extend(crate::parse_script_text_commands(&room.id, &scripts)?);
    room.script_text_bodies
        .extend(crate::parse_script_text_bodies(&room.id, &scripts)?);
    room.scripts.extend(scripts);
    authoring::refresh_event_section(room);
    Ok(())
}

// Compare the closed and open crosscut using production movement rules. In
// either state every previous dry stopping square must remain, and Surf from
// the east may add only the fifteen tiles inside the rock-lined drainage cut.
fn verify_discharge(
    pack: &CompiledGamePack,
    original: &MapModule,
    revised: &MapModule,
) -> Result<()> {
    use crate::northstar_cave_geometry::resting_tiles;
    let water: BTreeSet<_> = (16..=20)
        .flat_map(|y| (31..=33).map(move |x| [x, y]))
        .collect();
    let mut before = original.clone();
    let mut after = revised.clone();
    let shutter = 8 * usize::from(original.attributes.width) + 9;
    ensure!(
        before.blocks[shutter] == 0x41 && after.blocks[shutter] == 0x41,
        "haulage crosscut"
    );
    for open in [false, true] {
        if open {
            // Both versions use the same four-quadrant floor when HAUL is off.
            before.blocks[shutter] = 0x0d;
            after.blocks[shutter] = 0x0d;
        }
        for (entry, east) in [([4, 23], false), ([28, 23], true)] {
            ensure!(
                resting_tiles(pack, &before, entry, false)?
                    == resting_tiles(pack, &after, entry, false)?,
                "dry haulage reachability changed"
            );
            let mut compatible = resting_tiles(pack, &before, entry, true)?;
            if east || open {
                compatible.extend(&water);
            }
            ensure!(
                compatible == resting_tiles(pack, &after, entry, true)?,
                "the drain must return to the same working and respect pressure closure"
            );
        }
    }
    Ok(())
}

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let rooms: Vec<Junction> = serde_json::from_str(include_str!(
        "../../../planning/minnesota/caves/iron-mine-junctions-v01/junctions.json"
    ))?;
    let art = serde_json::from_str(include_str!(
        "../../../planning/minnesota/caves/iron-mine-junctions-v01/fixtures.json"
    ))?;
    let before = pack
        .data
        .maps
        .get("MnIronHaulage")
        .context("previous haulage working")?
        .clone();
    for room in &rooms {
        revise(pack, room, &art, &[])?;
    }
    verify_discharge(pack, &before, &pack.data.maps["MnIronHaulage"])
}

/// Builds an isolated Iron Range wayfinding preview from either released Fair
/// Presentation edition. The preview uses its own save identity.
///
/// # Errors
/// Rejects unexpected parent content, conflicting terrain or events, exhausted
/// native art banks, or a result that fails runtime pack validation.
pub fn revise_northstar_iron_wayfinding(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        matches!(
            base.identity()?.content_hash.as_str(),
            "d09a77f306578627e661a0811d2bacc12d22552e5e84041e0f002f6635db46c2"
                | "d03e25c72b2852c6c87e31924a7c2467f35f51469a7fc7e327370f7a84676a3e"
        ),
        "expected a released Fair Presentation edition"
    );
    let approaches: Vec<Approach> = serde_json::from_str(include_str!(
        "../../../planning/minnesota/caves/iron-mine-junctions-v01/approach-plates.json"
    ))?;
    let art = serde_json::from_str(include_str!(
        "../../../planning/minnesota/caves/iron-mine-junctions-v01/approach-art.json"
    ))?;
    let mut pack = base.clone();
    for approach in approaches {
        let mut spec = Junction {
            map: approach.map,
            fixtures: BTreeMap::new(),
            bank: Vec::new(),
        };
        for plate in &approach.plates {
            spec.fixtures
                .entry(plate.kind.clone())
                .or_default()
                .push(plate.tile);
        }
        revise(&mut pack, &spec, &art, &approach.plates)?;
    }
    // The published migration policy is addressed to the Fair Presentation ID.
    pack.runtime_files
        .remove("data/northstar-save-compatibility.json");
    pack.report
        .manifests
        .push("northstar-iron-wayfinding-preview-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
