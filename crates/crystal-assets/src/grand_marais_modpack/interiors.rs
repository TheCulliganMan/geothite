//! Authored room geometry shares the town's editable design and existing scripts.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use super::authoring::{blank, blocks};
use super::design::Interior;
use crate::{CompiledGamePack, MapModule};

pub(super) fn apply(
    base: &CompiledGamePack,
    designs: &[Interior],
    modules: &mut [MapModule],
) -> Result<()> {
    let mut remaining = BTreeSet::from([
        "MnGrandMaraisFieldStation",
        "MnGrandMaraisArtistHouse",
        "MnGrandMaraisFisherHouse",
        "MnGrandMaraisLighthouse",
    ]);
    for design in designs {
        ensure!(
            remaining.remove(design.map.as_str()),
            "unexpected or duplicate authored interior {}",
            design.map
        );
        ensure!(
            design.width > 0
                && design.height > 0
                && design.rows.len() == usize::from(design.height)
                && design
                    .rows
                    .iter()
                    .all(|row| { row.split_whitespace().count() == usize::from(design.width) }),
            "invalid interior dimensions for {}",
            design.map
        );
        let module = modules
            .iter_mut()
            .find(|module| module.id == design.map)
            .context("authored interior module")?;
        let constant = module
            .attributes
            .map_constant
            .as_deref()
            .context("interior map constant")?;
        module.attributes = blank(base, &design.template, &module.id, constant)?.attributes;
        blocks(module, design.width, design.height, &design.rows.join(" "))?;
        ensure!(
            design.exits.len() == 2 && module.events.warps.len() == 2,
            "{} needs both doorway lanes",
            design.map
        );
        for (warp, tile) in module.events.warps.iter_mut().zip(&design.exits) {
            [warp.x, warp.y] = *tile;
        }
        for tile in &design.exits {
            ensure!(
                collision_at(base, module, *tile)? == "WARP_CARPET_DOWN",
                "{} exit {tile:?} is not on its visible carpet",
                design.map
            );
        }
        for object in &module.objects {
            ensure!(
                collision_at(base, module, [object.x, object.y])? == "FLOOR",
                "{} resident stands on furniture",
                design.map
            );
        }
    }
    ensure!(
        remaining.is_empty(),
        "missing authored rooms: {remaining:?}"
    );
    Ok(())
}

fn collision_at<'a>(
    base: &'a CompiledGamePack,
    module: &MapModule,
    [x, y]: [u16; 2],
) -> Result<&'a str> {
    let width = usize::from(module.attributes.width);
    ensure!(
        usize::from(x) < width * 2 && usize::from(y) < usize::from(module.attributes.height) * 2,
        "interior event outside {}",
        module.id
    );
    let block = module.blocks[usize::from(y / 2) * width + usize::from(x / 2)];
    let tileset = base
        .data
        .tilesets
        .get(&module.attributes.tileset_name)
        .context("interior tileset")?;
    let quadrants = tileset
        .collision
        .get(&format!("{block:02x}"))
        .context("interior collision block")?;
    Ok(quadrants
        .get(usize::from(y % 2) * 2 + usize::from(x % 2))
        .context("interior collision quadrant")?
        .as_str())
}
