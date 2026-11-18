//! Assemble reviewed 16-pixel quadrants into native map blocks without replacing old art.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, MapModule};

pub(super) type Placement = (u16, u16, u16, [u16; 4]);

fn positions(q: usize) -> [usize; 4] {
    let first = (q / 2) * 8 + (q % 2) * 2;
    [first, first + 1, first + 4, first + 5]
}

pub(super) fn install(
    pack: &mut CompiledGamePack,
    room: &mut MapModule,
    edits: &[Placement],
) -> Result<()> {
    let name = &room.attributes.tileset_name;
    let atlas = format!("data/tilesets/{name}_metatiles.bin");
    let layout = pack
        .runtime_files
        .get_mut(&atlas)
        .context("native mosaic")?;
    let collisions = &mut pack
        .data
        .tilesets
        .get_mut(name)
        .context("maze tileset")?
        .collision;
    for &(x, y, old, donors) in edits {
        let index = usize::from(y) * usize::from(room.attributes.width) + usize::from(x);
        ensure!(
            x < room.attributes.width
                && room.blocks.get(index) == Some(&old)
                && !room
                    .script_block_changes
                    .iter()
                    .any(|c| [c.x / 2, c.y / 2] == [x, y]),
            "terrain preimage in {} at {x},{y}",
            room.id
        );
        let mut art = [0_u8; 16];
        let mut collision = Vec::with_capacity(4);
        for (q, source) in donors.into_iter().enumerate() {
            let id = source / 4;
            let sq = usize::from(source % 4);
            let before = &collisions[&format!("{old:02x}")][q];
            let after = &collisions
                .get(&format!("{id:02x}"))
                .context("donor collision")?[sq];
            ensure!(
                before == after || before == "WALL",
                "displaced former traversal at {}, {x},{y}/{q}: {before} to {after}",
                room.id
            );
            collision.push(after.clone());
            for (to, from) in positions(q).into_iter().zip(positions(sq)) {
                art[to] = *layout
                    .get(usize::from(id) * 16 + from)
                    .context("donor pixels")?;
            }
        }
        let existing = layout
            .as_chunks::<16>()
            .0
            .iter()
            .enumerate()
            .find(|(n, b)| **b == art && collisions.get(&format!("{n:02x}")) == Some(&collision));
        let block = if let Some((n, _)) = existing {
            n
        } else {
            let n = layout.len() / 16;
            ensure!(
                layout.len() % 16 == 0 && n < 256 && !collisions.contains_key(&format!("{n:02x}")),
                "maze atlas full or misaligned"
            );
            layout.extend(art);
            collisions.insert(format!("{n:02x}"), collision);
            n
        };
        room.blocks[index] = u16::try_from(block)?;
    }
    pack.runtime_files.insert(
        format!("data/tilesets/{name}.json"),
        serde_json::to_vec(&collisions)?,
    );
    Ok(())
}

// Transplant the established Gunflint snow sapling. Its four glyphs have
// authored foreground colors, which are part of its native appearance.
pub(super) fn snow_sapling(pack: &mut CompiledGamePack, road: &MapModule) -> Result<()> {
    let name = &road.attributes.tileset_name;
    let source = "mn_winter_gunflint";
    let atlas = format!("data/tilesets/{name}_metatiles.bin");
    let fg_path = format!("data/tilesets/{name}_foregrounds.json");
    let gfx_path = format!("gfx/tilesets/{name}.2bpp");
    let source_layout = &pack.runtime_files[&format!("data/tilesets/{source}_metatiles.bin")];
    let source_gfx = &pack.runtime_files[&format!("gfx/tilesets/{source}.2bpp")];
    let colors =
        |bytes: &[u8]| -> Result<BTreeMap<u8, [[u8; 3]; 3]>> { Ok(serde_json::from_slice(bytes)?) };
    let source_colors =
        colors(&pack.runtime_files[&format!("data/tilesets/{source}_foregrounds.json")])?;
    let mut colors = colors(&pack.runtime_files[&fg_path])?;
    let mut layout = pack.runtime_files[&atlas].clone();
    let mut gfx = pack.runtime_files[&gfx_path].clone();
    let source_pal = &pack.data.tilesets[source].palette_map;
    let mut pal = pack.data.tilesets[name].palette_map.clone();
    let addr = |g: u8, p: u8| {
        if p & 8 == 0 {
            usize::from(g)
        } else {
            usize::from(g & 127) + 128
        }
    };
    ensure!(
        gfx.len() == 4096 && source_gfx.len() == 4096 && pal.len() == 256,
        "unexpected winter glyph bank"
    );
    let mut used = [false; 256];
    for &g in &layout {
        used[usize::from(g)] = true;
        used[addr(g, pal[usize::from(g)])] = true;
    }
    let gate = road.blocks[5 * usize::from(road.attributes.width) + 10];
    let def = &pack.data.tilesets[name].collision;
    ensure!(
        gate >= 0xd8 && def[&format!("{gate:02x}")][0] == "CUT_TREE",
        "snow sapling quadrant"
    );
    // The native carrier lip sits on the lower eight pixels of its 32px block.
    // Both crown takeoffs occupy the bottom quadrants here, above the old cliff.
    let edge = usize::from(road.blocks[2 * usize::from(road.attributes.width) + 12]);
    ensure!(
        edge >= 0xd8
            && def[&format!("{edge:02x}")][2..] == ["HOP_DOWN", "HOP_DOWN"]
            && road
                .blocks
                .iter()
                .filter(|&&id| usize::from(id) == edge)
                .count()
                == 1,
        "expected the isolated two-tile crown lip"
    );
    for x in 0..4 {
        layout[edge * 16 + 12 + x] = layout[0xd7 * 16 + 12 + x];
    }
    let cleared = layout.len() / 16;
    ensure!(
        cleared < 256 && !def.contains_key(&format!("{cleared:02x}")),
        "no room for cleared snow"
    );
    let mut clear_block: [u8; 16] =
        layout[usize::from(gate) * 16..(usize::from(gate) + 1) * 16].try_into()?;
    for (to, from) in positions(0).into_iter().zip(positions(2)) {
        let g = source_layout[15 * 16 + from];
        let i = used
            .iter()
            .position(|&u| !u)
            .context("no free sapling glyph")?;
        used[i] = true;
        // Use the lower bank selector for low logical IDs, and canonical high
        // bank addressing for high logical IDs.
        let color = source_pal[usize::from(g)];
        let physical = addr(g, color);
        gfx[i * 16..i * 16 + 16].copy_from_slice(&source_gfx[physical * 16..physical * 16 + 16]);
        pal[i] = (color & 7) | if i >= 128 { 8 } else { 0 };
        let id = u8::try_from(i)?;
        colors.insert(id, *source_colors.get(&g).context("native sapling shades")?);
        layout[usize::from(gate) * 16 + to] = id;
        clear_block[to] = layout[0x86 * 16 + to];
    }
    layout.extend(clear_block);
    let def = pack
        .data
        .tilesets
        .get_mut(name)
        .context("iron descent tileset")?;
    def.palette_map = pal;
    let mut clear_collision = def.collision[&format!("{gate:02x}")].clone();
    clear_collision[0] = "FLOOR".into();
    def.collision
        .insert(format!("{cleared:02x}"), clear_collision);
    let mut rule = pack.data.field_moves.cut.replacements["johto"][&0x67].clone();
    rule.replacement_block_id = u16::try_from(cleared)?;
    ensure!(
        rule.variant == "tree"
            && pack
                .data
                .field_moves
                .cut
                .replacements
                .get_mut(name)
                .context("iron trail Cut registry")?
                .insert(gate, rule)
                .is_none(),
        "occupied cut rule"
    );
    pack.runtime_files.insert(
        format!("gfx/tilesets/{name}.png"),
        crate::native_tile_png::from_2bpp(&gfx)?,
    );
    pack.runtime_files.insert(gfx_path, gfx);
    pack.runtime_files
        .insert(fg_path, serde_json::to_vec(&colors)?);
    pack.runtime_files.insert(atlas, layout);
    pack.runtime_files.insert(
        format!("data/tilesets/{name}_palette_map.json"),
        serde_json::to_vec(&def.palette_map)?,
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{name}.json"),
        serde_json::to_vec(&def.collision)?,
    );
    Ok(())
}
