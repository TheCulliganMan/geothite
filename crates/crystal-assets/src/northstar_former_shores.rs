//! Shell-bearing beach ridges and an eroded spillway above the Strandrak basin.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::authored_route_surface::{reachable_map_with_surf, walking_map_distances};
use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, MapModule, authored_town_publication, native_tile_png};

const MAP: &str = "MnFormerShores";
const TILESET: &str = "mn_prairie_headwaters";
const DESIGN: &[u8] =
    include_bytes!("../../../planning/minnesota/discoveries/former-shores-v01/strandlines.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Strandlines {
    glyphs: Vec<Glyph>,
    placements: Vec<Placement>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Glyph {
    symbol: char,
    palette: u8,
    rows: [String; 8],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Placement {
    at: [u16; 2],
    from: u16,
    footing: u16,
    cells: [String; 4],
    swale: Option<String>,
}

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let design: Strandlines = serde_json::from_slice(DESIGN)?;
    let mut room = pack.data.maps.get(MAP).context("former shores")?.clone();
    ensure!(
        room.attributes.tileset_name == TILESET
            && room.attributes.width == 18
            && room.attributes.height == 14,
        "expected the released former-shores survey"
    );
    let occupied: BTreeSet<_> = room
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .chain(room.events.warps.iter().map(|warp| [warp.x, warp.y]))
        .collect();
    let north = [14, 0];
    let old_reachable = walking_map_distances(pack, &room, north, &occupied)?;
    ensure!(
        survey_walk(pack, &room, &occupied)? == [18, 18, 16, 36, 37],
        "unexpected parent survey paths"
    );
    install_terrain(pack, &mut room, &design)?;
    let reachable = walking_map_distances(pack, &room, north, &occupied)?;
    ensure!(
        old_reachable.keys().all(|p| reachable.contains_key(p))
            && reachable.len() == old_reachable.len() + 8,
        "the survey retains every old stopping square and opens eight new dry bank squares"
    );
    ensure!(
        survey_walk(pack, &room, &occupied)? == [6, 10, 14, 22, 23],
        "survey spillway or an observation is disconnected"
    );
    let by_water = reachable_map_with_surf(pack, &room, north, &occupied)?;
    let wets: Vec<_> = by_water
        .iter()
        .filter(|p| !reachable.contains_key(*p))
        .collect();
    ensure!(
        wets.len() == 22
            && wets
                .iter()
                .all(|&&p| (28..=33).contains(&p[0]) && (4..=11).contains(&p[1]))
            && kind(pack, &room, [30, 8])? == "FLOOR"
            && kind(pack, &room, [31, 8])? == "WATER"
            && room.attributes.fishing_group.as_deref() == Some("FISHGROUP_MN_QWILFISH"),
        "the swale must retain its fishable west bank and enclosed water"
    );
    authored_town_publication::replace(pack, room)
}

fn survey_walk(
    pack: &CompiledGamePack,
    room: &MapModule,
    occupied: &BTreeSet<[u16; 2]>,
) -> Result<[u32; 5]> {
    let surveyor = [17, 6];
    let high = [16, 9];
    let middle = [20, 15];
    let low = [12, 21];
    let mouth = [18, 22];
    let distance = |a, b| {
        walking_map_distances(pack, room, a, occupied)?
            .get(&b)
            .copied()
            .context("inaccessible survey stop")
    };
    Ok([
        distance(surveyor, high)?,
        distance(high, middle)?,
        distance(middle, low)?,
        distance(low, surveyor)?,
        distance(surveyor, mouth)?,
    ])
}

fn install_terrain(
    pack: &mut CompiledGamePack,
    room: &mut MapModule,
    design: &Strandlines,
) -> Result<()> {
    let layout_path = format!("data/tilesets/{TILESET}_metatiles.bin");
    let gfx_path = format!("gfx/tilesets/{TILESET}.2bpp");
    let mut layout = pack
        .runtime_files
        .get(&layout_path)
        .context("prairie metatiles")?
        .clone();
    let graphics = pack
        .runtime_files
        .get_mut(&gfx_path)
        .context("prairie graphics")?;
    let tileset = pack
        .data
        .tilesets
        .get_mut(TILESET)
        .context("prairie tileset")?;
    ensure!(
        graphics.len() == 192 * 16
            && tileset.palette_map.len() == 224
            && layout.len().is_multiple_of(16),
        "unexpected native prairie atlas"
    );
    let mut cells = BTreeMap::from([
        ('_', 5_u8),
        (':', 44_u8),
        ('=', 76_u8),
        ('[', 43_u8),
        (']', 45_u8),
        ('(', 75_u8),
        (')', 77_u8),
        ('g', 60_u8),
        ('0', 30_u8),
        ('1', 31_u8),
        ('2', 62_u8),
        ('3', 63_u8),
        ('*', 3_u8),
    ]);
    // A 192-glyph GBC sheet maps upper-bank IDs c0..df to physical slots a0..bf.
    let physical = |id: u8, palette: u8| {
        if palette & 8 == 0 {
            usize::from(id)
        } else {
            usize::from(id & 0x7f) + 96
        }
    };
    let used: BTreeSet<_> = layout
        .iter()
        .map(|&id| physical(id, tileset.palette_map[usize::from(id)]))
        .collect();
    for (index, glyph) in design.glyphs.iter().enumerate() {
        let id = u8::try_from(0xc0 + index)?;
        let slot = physical(id, 8);
        ensure!(
            !layout.contains(&id)
                && !used.contains(&slot)
                && glyph.palette < 8
                && cells.insert(glyph.symbol, id).is_none(),
            "occupied prairie glyph {id:02x}"
        );
        let dest = graphics
            .get_mut(16 * slot..16 * (slot + 1))
            .context("prairie character bank full")?;
        dest.fill(0);
        for (y, row) in glyph.rows.iter().enumerate() {
            ensure!(row.len() == 8, "shore glyph row width");
            for (x, level) in row.bytes().enumerate() {
                ensure!((b'0'..=b'3').contains(&level), "invalid shore pigment");
                let shade = level - b'0';
                dest[2 * y] |= (shade & 1) << (7 - x);
                dest[2 * y + 1] |= (shade >> 1) << (7 - x);
            }
        }
        tileset.palette_map[usize::from(id)] = 8 | glyph.palette;
    }
    let mut positions = BTreeSet::new();
    for edit in &design.placements {
        let [x, y] = edit.at;
        ensure!(
            x < room.attributes.width && y < room.attributes.height && positions.insert(edit.at),
            "invalid or duplicate former shore position"
        );
        let i = usize::from(y) * usize::from(room.attributes.width) + usize::from(x);
        ensure!(
            room.blocks[i] == edit.from
                && (edit.from == edit.footing || (edit.from == 0x70 && edit.footing == 0x02)),
            "expected the original native footing at {x},{y}"
        );
        let start = 16 * usize::from(edit.from);
        let mut art: [u8; 16] = layout
            .get(start..start + 16)
            .context("original prairie metatile")?
            .try_into()?;
        for (row, source) in edit.cells.iter().enumerate() {
            ensure!(source.len() == 4, "shore metatile row width");
            for (col, symbol) in source.chars().enumerate() {
                if symbol != '.' {
                    art[4 * row + col] = *cells.get(&symbol).context("unknown shore relief")?;
                }
            }
        }
        let mut collision = tileset
            .collision
            .get(&format!("{:02x}", edit.footing))
            .context("strandline footing")?
            .clone();
        if let Some(swale) = &edit.swale {
            ensure!(
                edit.from == 5 && swale.len() == 4 && collision.iter().all(|c| c == "WALL"),
                "swale must open only former woodland"
            );
            for (cell, token) in collision.iter_mut().zip(swale.bytes()) {
                match token {
                    b'.' => {}
                    b'f' => *cell = "FLOOR".into(),
                    b'w' => *cell = "WATER".into(),
                    _ => anyhow::bail!("unknown swale footing"),
                }
            }
        }
        let existing = layout
            .as_chunks::<16>()
            .0
            .iter()
            .enumerate()
            .find(|(id, block)| {
                **block == art && tileset.collision.get(&format!("{id:02x}")) == Some(&collision)
            });
        let id = if let Some((id, _)) = existing {
            id
        } else {
            let id = layout.len() / 16;
            ensure!(
                id < 256
                    && tileset
                        .collision
                        .insert(format!("{id:02x}"), collision)
                        .is_none(),
                "occupied shore metatile"
            );
            layout.extend(art);
            id
        };
        room.blocks[i] = u16::try_from(id)?;
    }
    let png = native_tile_png::from_2bpp(graphics)?;
    pack.runtime_files
        .insert(format!("gfx/tilesets/{TILESET}.png"), png);
    pack.runtime_files.insert(layout_path, layout);
    pack.runtime_files.insert(
        format!("data/tilesets/{TILESET}.json"),
        serde_json::to_vec(&tileset.collision)?,
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{TILESET}_palette_map.json"),
        serde_json::to_vec(&tileset.palette_map)?,
    );
    Ok(())
}
