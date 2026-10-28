//! Copy exact existing metatile glyph references into opt-in bridge permissions.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::authored_river_connections_design::BridgeInventory;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn install(
    base: &CompiledGamePack,
    pack: &mut CompiledGamePack,
    maps: &mut BTreeMap<String, MapModule>,
    inventory: &BridgeInventory,
) -> Result<()> {
    let mut occupied = BTreeSet::new();
    let mut recipes = BTreeMap::<(String, String, u16), u16>::new();
    let mut changed_tilesets = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for bridge in &inventory.bridges {
        ensure!(ids.insert(&bridge.id), "duplicate bridge identity");
        let original = base
            .data
            .maps
            .get(&bridge.map)
            .context("bridge original map")?;
        ensure!(
            crate::authored_river_connections::module_hash(original)?
                == bridge.native_module_sha256
                && original.attributes.tileset_name == bridge.tileset
                && bridge.existing_foot_collision_must_remain_exact,
            "{} original bridge preimage differs",
            bridge.id
        );
        ensure!(
            matches!(bridge.axis.as_str(), "north_south" | "east_west"),
            "bridge water axis"
        );
        let [x0, y0, x1, y1] = bridge.metatile_bounds_exclusive;
        ensure!(
            x0 < x1
                && y0 < y1
                && x1 <= original.attributes.width
                && y1 <= original.attributes.height,
            "bridge aperture bounds"
        );
        ensure!(
            bridge.metatiles.len() == usize::from(x1 - x0) * usize::from(y1 - y0)
                && bridge.water_mouths_core.len() == 2
                && bridge.aperture_width_core > 0,
            "complete bounded bridge rectangle required"
        );
        let prefix = format!("data/tilesets/{}", bridge.tileset);
        let file = format!("{prefix}_metatiles.bin");
        for cell in &bridge.metatiles {
            let [x, y] = cell.metatile;
            ensure!(
                x0 <= x && x < x1 && y0 <= y && y < y1,
                "bridge cell outside rectangle"
            );
            ensure!(
                occupied.insert((bridge.map.clone(), x, y)),
                "overlapping bridge apertures"
            );
            let index = usize::from(y) * usize::from(original.attributes.width) + usize::from(x);
            ensure!(
                original.blocks[index] == cell.block_decimal
                    && u16::from_str_radix(&cell.block_hex, 16)? == cell.block_decimal,
                "stale native bridge cell"
            );
            let definition = &base.data.tilesets[&bridge.tileset];
            ensure!(
                definition.collision.get(&cell.block_hex) == Some(&cell.foot_collision)
                    && cell.foot_collision.len() == 4
                    && cell.proposed_surf_underpass_collision == ["WATER"; 4],
                "exact native bridge quadrant inventory required"
            );
            let key = (
                bridge.tileset.clone(),
                bridge.axis.clone(),
                cell.block_decimal,
            );
            let block = if let Some(&block) = recipes.get(&key) {
                block
            } else {
                let original_art = base
                    .runtime_files
                    .get(&file)
                    .context("native bridge metatiles")?;
                let offset = usize::from(cell.block_decimal) * 16;
                let pixels = original_art
                    .get(offset..offset + 16)
                    .context("native source bridge block")?;
                let art = pack
                    .runtime_files
                    .get_mut(&file)
                    .context("bridge target metatiles")?;
                ensure!(art.len() % 16 == 0, "native block alignment");
                let block = u16::try_from(art.len() / 16)?;
                ensure!(block <= 255, "bridge append exceeds native block range");
                art.extend_from_slice(pixels);
                let mut collision = Vec::new();
                for old in &cell.foot_collision {
                    let token = match (bridge.axis.as_str(), old.as_str()) {
                        ("north_south", "FLOOR") => "BRIDGE_NS_DECK",
                        ("north_south", "WALL") => "BRIDGE_NS_RAIL",
                        ("east_west", "FLOOR") => "BRIDGE_EW_DECK",
                        ("east_west", "WALL") => "BRIDGE_EW_RAIL",
                        _ => anyhow::bail!("bridge cannot reinterpret native {old}"),
                    };
                    collision.push(token.into());
                }
                ensure!(
                    pack.data
                        .tilesets
                        .get_mut(&bridge.tileset)
                        .context("bridge definition")?
                        .collision
                        .insert(format!("{block:02x}"), collision)
                        .is_none(),
                    "bridge block ID occupied"
                );
                recipes.insert(key, block);
                changed_tilesets.insert(bridge.tileset.clone());
                block
            };
            let map = maps.get_mut(&bridge.map).context("bridge working map")?;
            ensure!(
                map.blocks[index] == cell.block_decimal,
                "terrain revision overlaps bridge aperture"
            );
            map.blocks[index] = block;
        }
    }
    for name in changed_tilesets {
        let file = format!("data/tilesets/{name}_metatiles.bin");
        ensure!(
            pack.runtime_files
                .get(&file)
                .context("appended bridge metatiles")?
                .starts_with(
                    base.runtime_files
                        .get(&file)
                        .context("original bridge metatiles")?
                ),
            "original native bridge metatiles changed"
        );
        pack.runtime_files.insert(
            format!("data/tilesets/{name}.json"),
            serde_json::to_vec_pretty(&pack.data.tilesets[&name].collision)?,
        );
        // No glyph, PNG, palette or animation bytes are changed or copied.
        for suffix in [".2bpp", ".png", ".pal"] {
            let path = format!("gfx/tilesets/{name}{suffix}");
            ensure!(
                pack.runtime_files.get(&path) == base.runtime_files.get(&path),
                "source bridge graphics changed"
            );
        }
    }
    Ok(())
}
