//! Native Park railing and continuous bank recipes; original art is unchanged.
use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, CompiledTilesetExtension};

pub(crate) fn build_twin_cities_art_pack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let files = base.runtime_files();
    let mut definition = base
        .data()
        .tilesets
        .get("park")
        .context("Park source")?
        .clone();
    let mut metatiles = files
        .get("data/tilesets/park_metatiles.bin")
        .context("Park metatiles")?
        .clone();
    ensure!(metatiles.len() == 64 * 16, "Park art extent changed");
    ensure!(
        !files.contains_key("gfx/tilesets/park.pal"),
        "Park source requires a dedicated palette import"
    );
    let original = &files["data/tilesets/park_metatiles.bin"];
    for (block, row) in [(0x19, 2), (0x19, 3), (0x01, 2), (0x01, 3)] {
        metatiles.extend_from_slice(&original[block * 16 + row * 4..block * 16 + row * 4 + 4]);
    }
    let source = definition.collision.get("19").context("Park lower fence")?;
    ensure!(
        source == &["FLOOR", "FLOOR", "WALL", "WALL"],
        "Park fence collision changed"
    );
    let collision = [2, 3, 0, 1]
        .into_iter()
        .map(|index| source[index].clone())
        .collect();
    definition.collision.insert("40".into(), collision);
    for (id, block) in [("41", 0x08), ("42", 0x09)] {
        // Repeat only the straight middle rows of each fountain corner. The
        // original four-row corner would leave a cyan cap every metatile.
        let source = &files["data/tilesets/park_metatiles.bin"][block * 16..(block + 1) * 16];
        for row in [2, 2, 2, 2] {
            metatiles.extend_from_slice(&source[row * 4..row * 4 + 4]);
        }
        let collision = definition
            .collision
            .get(&format!("{block:02x}"))
            .context("Park fountain bank")?;
        ensure!(
            collision.iter().all(|tile| tile == "WALL"),
            "Park retaining bank collision changed"
        );
        definition.collision.insert(id.into(), collision.clone());
    }
    for (block, row) in [(0x01, 0), (0x01, 1), (0x19, 2), (0x19, 3)] {
        metatiles.extend_from_slice(&original[block * 16 + row * 4..block * 16 + row * 4 + 4]);
    }
    definition
        .collision
        .insert("43".into(), definition.collision["19"].clone());
    base.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: "minnesota-twin-cities-art".into(),
        tileset_id: "mn_river_park".into(),
        behavior_source_tileset_id: Some("park".into()),
        definition,
        metatiles,
        tile_graphics_2bpp: files
            .get("gfx/tilesets/park.2bpp")
            .context("Park pixels")?
            .clone(),
        tile_graphics_png: files
            .get("gfx/tilesets/park.png")
            .context("Park preview pixels")?
            .clone(),
    })
}
