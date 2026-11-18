//! Lower a short stretch of the native rock face to mark an outdoor one-way return.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_town_publication;

const NATIVE_LEDGE: usize = 0x56;

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    lower_cliff_lip(
        pack,
        "MnR10C00R00",
        "mn_iron_descent_thaw",
        0xa0,
        0xd7,
        &[[12, 10], [13, 10]],
    )
}

pub(super) fn install_fossil_bluff(pack: &mut CompiledGamePack) -> Result<()> {
    lower_cliff_lip(
        pack,
        "MnR01C00R00",
        "mn_outdoor_johto_stairs",
        0x72,
        0x81,
        &[[12, 8]],
    )
}

fn lower_cliff_lip(
    pack: &mut CompiledGamePack,
    map: &str,
    tileset: &str,
    shelf: u8,
    lip: u8,
    placements: &[[u16; 2]],
) -> Result<()> {
    let mut road = pack.data.maps.get(map).context("cliff road")?.clone();
    ensure!(
        road.attributes.tileset_name == tileset
            && road.attributes.width == 32
            && road.attributes.height == 20,
        "unexpected cliff geometry in {map}"
    );
    let definition = pack
        .data
        .tilesets
        .get_mut(tileset)
        .context("road cliff atlas")?;
    let ledge = definition
        .collision
        .get("56")
        .context("native down ledge")?;
    ensure!(
        ledge == &["HOP_DOWN", "HOP_DOWN", "WALL", "WALL"]
            && definition
                .collision
                .get(&format!("{shelf:02x}"))
                .is_some_and(|shelf| { shelf == &["FLOOR", "FLOOR", "WALL", "WALL"] }),
        "unexpected cliff or ledge permissions"
    );
    let native_collision = ledge.clone();
    let layout = pack
        .runtime_files
        .get_mut(&format!("data/tilesets/{tileset}_metatiles.bin"))
        .context("native cliff block art")?;
    ensure!(
        layout.len() == usize::from(lip) * 16
            && !definition.collision.contains_key(&format!("{lip:02x}")),
        "cliff lip block already occupied"
    );

    // Continue the existing tan bench across the upper three tile rows; the
    // original low ledge lip on the fourth row marks the two jumpable faces.
    let shelf_index = usize::from(shelf);
    let mut art = [0; 16];
    art[..8].copy_from_slice(&layout[shelf_index * 16..shelf_index * 16 + 8]);
    art[8..12].copy_from_slice(&layout[shelf_index * 16..shelf_index * 16 + 4]);
    art[12..].copy_from_slice(&layout[NATIVE_LEDGE * 16 + 12..NATIVE_LEDGE * 16 + 16]);
    layout.extend(art);
    definition
        .collision
        .insert(format!("{lip:02x}"), native_collision);
    pack.runtime_files.insert(
        format!("data/tilesets/{tileset}.json"),
        serde_json::to_vec(&definition.collision)?,
    );
    for &[x, y] in placements {
        let i = usize::from(y) * usize::from(road.attributes.width) + usize::from(x);
        ensure!(
            road.blocks.get(i) == Some(&u16::from(shelf)),
            "cliff shelf moved in {map}"
        );
        road.blocks[i] = u16::from(lip);
    }
    authored_town_publication::replace(pack, road)
}
