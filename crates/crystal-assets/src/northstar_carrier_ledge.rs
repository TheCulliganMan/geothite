//! Restore the one-way downhill return from the Route 10 carrier bench.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_town_publication;

const TILESET: &str = "mn_iron_descent_thaw";
const SHELF: usize = 0xa0;
const NATIVE_LEDGE: usize = 0x56;
const CARRIER_LEDGE: u16 = 0xd7;

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut road = pack
        .data
        .maps
        .get("MnR10C00R00")
        .context("Route 10 west")?
        .clone();
    ensure!(
        road.attributes.tileset_name == TILESET
            && road.attributes.width == 32
            && road.attributes.height == 20,
        "unexpected carrier bench geometry"
    );
    let definition = pack.data.tilesets.get_mut(TILESET).context("thaw atlas")?;
    let ledge = definition
        .collision
        .get("56")
        .context("native down ledge")?;
    ensure!(
        ledge == &["HOP_DOWN", "HOP_DOWN", "WALL", "WALL"]
            && definition
                .collision
                .get("a0")
                .is_some_and(|shelf| { shelf == &["FLOOR", "FLOOR", "WALL", "WALL"] }),
        "unexpected cliff or ledge permissions"
    );
    let native_collision = ledge.clone();
    let layout = pack
        .runtime_files
        .get_mut("data/tilesets/mn_iron_descent_thaw_metatiles.bin")
        .context("native thaw block art")?;
    ensure!(
        layout.len() == usize::from(CARRIER_LEDGE) * 16 && !definition.collision.contains_key("d7"),
        "carrier ledge block already occupied"
    );

    // Continue the existing tan bench across the upper three tile rows; the
    // original low ledge lip on the fourth row marks the two jumpable faces.
    let mut art = [0; 16];
    art[..8].copy_from_slice(&layout[SHELF * 16..SHELF * 16 + 8]);
    art[8..12].copy_from_slice(&layout[SHELF * 16..SHELF * 16 + 4]);
    art[12..].copy_from_slice(&layout[NATIVE_LEDGE * 16 + 12..NATIVE_LEDGE * 16 + 16]);
    layout.extend(art);
    definition
        .collision
        .insert("d7".to_owned(), native_collision);
    pack.runtime_files.insert(
        "data/tilesets/mn_iron_descent_thaw.json".into(),
        serde_json::to_vec(&definition.collision)?,
    );
    for x in [12, 13] {
        let i = 10 * usize::from(road.attributes.width) + x;
        ensure!(usize::from(road.blocks[i]) == SHELF, "carrier lip moved");
        road.blocks[i] = CARRIER_LEDGE;
    }
    authored_town_publication::replace(pack, road)
}
