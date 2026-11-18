//! Native artwork for the Gunflint cut-tree clearing and its cave threshold.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    install_fishing_terrace(pack)?;
    install_grotto_stairs(pack)?;
    install_snow_tree(pack)
}

// Taper the fishing terrace into the same grass path that reaches the cabin.
fn install_fishing_terrace(pack: &mut CompiledGamePack) -> Result<()> {
    let layout = pack
        .runtime_files
        .get_mut("data/tilesets/kanto_metatiles.bin")
        .context("native lakeshore art")?;
    let definition = pack
        .data
        .tilesets
        .get_mut("kanto")
        .context("native lakeshore tileset")?;
    ensure!(layout.len() == 0x83 * 16, "unexpected shore atlas extent");
    for (id, quadrants) in [
        (0x83, [0x55, 0x55, 0x55, 0x31]),
        (0x84, [0x55, 0x31, 0x31, 0x31]),
    ] {
        let mut tiles = [0; 16];
        for (i, glyph) in tiles.iter_mut().enumerate() {
            let quadrant = (i / 8) * 2 + (i % 4) / 2;
            *glyph = layout[quadrants[quadrant] * 16 + i];
        }
        layout.extend(tiles);
        ensure!(
            definition
                .collision
                .insert(format!("{id:02x}"), vec![String::from("FLOOR"); 4])
                .is_none(),
            "occupied shore art"
        );
    }
    pack.runtime_files.insert(
        "data/tilesets/kanto.json".into(),
        serde_json::to_vec(&definition.collision)?,
    );
    Ok(())
}

// Use the original cuttable tree silhouette on snow. The standing evergreen
// stays on the right even after Cut clears the smaller tree on its left.
fn install_snow_tree(pack: &mut CompiledGamePack) -> Result<()> {
    let name = "mn_winter_gunflint";
    let path = format!("data/tilesets/{name}_metatiles.bin");
    let mut layout = pack
        .runtime_files
        .get(&path)
        .context("Gunflint winter layout")?
        .clone();
    let mut standing: [u8; 16] = layout
        .get(0x44 * 16..0x45 * 16)
        .context("snow tree pair")?
        .try_into()?;
    let mut cleared = standing;
    let snow: [u8; 16] = layout[0x3c * 16..0x3d * 16].try_into()?;
    let definition = pack
        .data
        .tilesets
        .get_mut(name)
        .context("Gunflint winter tileset")?;
    ensure!(
        definition.collision["0f"] == ["FLOOR", "FLOOR", "CUT_TREE", "HEADBUTT_TREE"]
            && definition.collision["17"] == ["FLOOR", "FLOOR", "FLOOR", "HEADBUTT_TREE"],
        "native Cut preimage"
    );
    let gfx_path = format!("gfx/tilesets/{name}.2bpp");
    let fg_path = format!("data/tilesets/{name}_foregrounds.json");
    let mut fg: BTreeMap<u8, [[u8; 3]; 3]> =
        serde_json::from_slice(pack.runtime_files.get(&fg_path).context("winter shades")?)?;
    let pine = *fg.get(&250).context("native evergreen shades")?;
    let graphics = pack
        .runtime_files
        .get_mut(&gfx_path)
        .context("winter graphics")?;
    ensure!(
        graphics.len() == 256 * 16,
        "expected two equal winter graphics banks"
    );
    for (tile, dest) in [8, 9, 12, 13].into_iter().zip(0x54_u8..=0x57) {
        let src = layout[0x0f * 16 + tile];
        let physical = |g: u8| {
            if definition.palette_map[usize::from(g)] & 8 == 0 {
                usize::from(g)
            } else {
                usize::from(g & 127) + 128
            }
        };
        ensure!(
            layout.iter().all(|&g| physical(g) != usize::from(dest)) && !layout.contains(&dest),
            "winter glyph slot already used"
        );
        let start = physical(src) * 16;
        graphics.copy_within(start..start + 16, usize::from(dest) * 16);
        definition.palette_map[usize::from(dest)] = definition.palette_map[250] & 7;
        fg.insert(dest, pine);
        standing[tile] = dest;
        cleared[tile] = snow[tile];
    }
    layout[0x0f * 16..0x10 * 16].copy_from_slice(&standing);
    layout[0x17 * 16..0x18 * 16].copy_from_slice(&cleared);
    let png = crate::native_tile_png::from_2bpp(graphics)?;
    pack.runtime_files.insert(path, layout);
    pack.runtime_files
        .insert(format!("gfx/tilesets/{name}.png"), png);
    pack.runtime_files.insert(fg_path, serde_json::to_vec(&fg)?);
    pack.runtime_files.insert(
        format!("data/tilesets/{name}_palette_map.json"),
        serde_json::to_vec(&definition.palette_map)?,
    );
    Ok(())
}

// The return warp occupies just the bottom-right player tile. Copy the native
// staircase there, while keeping the adjacent bottom-left square plain floor.
fn install_grotto_stairs(pack: &mut CompiledGamePack) -> Result<()> {
    let layout = pack
        .runtime_files
        .get_mut("data/tilesets/cave_metatiles.bin")
        .context("cave art")?;
    ensure!(layout.len() == 0x40 * 16, "unexpected cave atlas extent");
    let mut stairs: [u8; 16] = layout[0x12 * 16..0x13 * 16].try_into()?;
    for i in [8, 9, 12, 13] {
        stairs[i] = layout[0x02 * 16 + i];
    }
    layout.extend(stairs);
    let definition = pack
        .data
        .tilesets
        .get_mut("cave")
        .context("native cave tileset")?;
    let threshold = definition
        .collision
        .get("24")
        .context("grotto threshold")?
        .clone();
    ensure!(
        threshold == ["FLOOR", "FLOOR", "FLOOR", "WARP_CARPET_DOWN"]
            && definition
                .collision
                .insert("40".into(), threshold)
                .is_none(),
        "grotto threshold collision preimage"
    );
    pack.runtime_files.insert(
        "data/tilesets/cave.json".into(),
        serde_json::to_vec(&definition.collision)?,
    );
    Ok(())
}
