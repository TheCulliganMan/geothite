//! Native gray fence around Ellis, with a separately opening rescue gate.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, CompiledTilesetExtension, MapModule, native_tile_png};

pub(crate) const TILESET: &str = "mn_northstar_holding";
pub(crate) const CELLS: [(usize, usize, u16); 5] = [
    (6, 1, 0x40),
    (6, 2, 0x40),
    (6, 4, 0x42),
    (7, 4, 0x41),
    (8, 4, 0x41),
];

pub(crate) fn install(pack: &mut CompiledGamePack, room: &mut MapModule) -> Result<()> {
    ensure!(
        room.id == "MnMinneapolisDispatchOffice"
            && room.attributes.width == 10
            && room.attributes.height == 8
            && room.blocks.len() == 80
            && room.blocks[36] == 0x38,
        "holding room and separate native gate required"
    );
    if room.attributes.tileset_name == TILESET {
        ensure!(
            pack.data.tilesets.contains_key(TILESET)
                && CELLS
                    .iter()
                    .all(|&(x, y, id)| room.blocks[y * 10 + x] == id),
            "installed gray holding fence changed"
        );
        return Ok(());
    }
    ensure!(
        room.attributes.tileset_name == "underground"
            && CELLS
                .iter()
                .all(|&(x, y, _)| room.blocks[y * 10 + x] == 0x38),
        "original solid holding barrier required"
    );
    let mut definition = pack.data.tilesets["underground"].clone();
    let source = pack
        .runtime_files()
        .get("data/tilesets/underground_metatiles.bin")
        .context("native holding room metatiles")?;
    ensure!(source.len() == 64 * 16, "native holding art extent changed");
    let mut graphics = vec![0; 192 * 16];
    definition.palette_map.fill(0);
    let mut glyphs = BTreeMap::new();
    let mut blocks = source
        .iter()
        .map(|&glyph| {
            copy_glyph(
                pack,
                "underground",
                glyph,
                &mut glyphs,
                &mut graphics,
                &mut definition.palette_map,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let fence = pack
        .runtime_files()
        .get("data/tilesets/johto_modern_metatiles.bin")
        .context("native gray metal fence")?;
    let floor: [u8; 16] = blocks[5 * 16..6 * 16]
        .try_into()
        .expect("native metatiles contain sixteen glyphs");
    let mut side = floor;
    for row in 0..4 {
        side[row * 4 + 1] = copy_glyph(
            pack,
            "johto_modern",
            fence[0x46 * 16 + row * 4 + 3],
            &mut glyphs,
            &mut graphics,
            &mut definition.palette_map,
        )?;
    }
    let mut bottom = floor;
    for row in 0..2 {
        for column in 0..4 {
            bottom[(row + 2) * 4 + column] = copy_glyph(
                pack,
                "johto_modern",
                fence[0x41 * 16 + row * 4 + column],
                &mut glyphs,
                &mut graphics,
                &mut definition.palette_map,
            )?;
        }
    }
    let mut corner = bottom;
    for row in 0..2 {
        corner[row * 4 + 1] = side[row * 4 + 1];
    }
    // The old gate keeps its block identity: existing release/return scripts
    // still replace 0x38 with native floor 0x05 after acknowledgment.
    blocks[0x38 * 16..0x39 * 16].copy_from_slice(&side);
    blocks.extend(side);
    blocks.extend(bottom);
    blocks.extend(corner);
    for (id, collision) in [
        ("38", ["WALL", "FLOOR", "WALL", "FLOOR"]),
        ("40", ["WALL", "FLOOR", "WALL", "FLOOR"]),
        ("41", ["FLOOR", "FLOOR", "WALL", "WALL"]),
        ("42", ["WALL", "FLOOR", "WALL", "WALL"]),
    ] {
        definition
            .collision
            .insert(id.into(), collision.map(str::to_owned).to_vec());
    }
    let extension = CompiledTilesetExtension {
        manifest_id: "northstar-gray-holding-fence-v1".into(),
        tileset_id: TILESET.into(),
        behavior_source_tileset_id: Some("underground".into()),
        definition,
        metatiles: blocks,
        tile_graphics_png: native_tile_png::from_2bpp(&graphics)?,
        tile_graphics_2bpp: graphics,
    };
    crate::validate_compiled_tileset_extension(&extension)?;
    crate::insert_compiled_tileset_extension(pack, extension)?;
    room.attributes.tileset_name = TILESET.into();
    for (x, y, id) in CELLS {
        room.blocks[y * 10 + x] = id;
    }
    Ok(())
}

fn copy_glyph(
    pack: &CompiledGamePack,
    source: &'static str,
    glyph: u8,
    glyphs: &mut BTreeMap<(&'static str, u8), u8>,
    graphics: &mut [u8],
    palettes: &mut [u8],
) -> Result<u8> {
    if let Some(&id) = glyphs.get(&(source, glyph)) {
        return Ok(id);
    }
    let index = glyphs.len();
    ensure!(index < 192, "holding fence exceeds native glyph capacity");
    let attribute = pack.data.tilesets[source].palette_map[usize::from(glyph)];
    let pixels = pack
        .runtime_files()
        .get(&format!("gfx/tilesets/{source}.2bpp"))
        .context("native fence source pixels")?;
    let physical = if attribute & 8 == 0 {
        usize::from(glyph)
    } else {
        usize::from(glyph & 0x7f) + pixels.len() / 32
    };
    graphics[index * 16..(index + 1) * 16].copy_from_slice(
        pixels
            .get(physical * 16..(physical + 1) * 16)
            .context("native source glyph")?,
    );
    let (logical, bank) = if index < 96 {
        (index, 0)
    } else {
        ((index - 96) | 0x80, 8)
    };
    palettes[logical] = (attribute & 7) | bank;
    let logical = u8::try_from(logical)?;
    glyphs.insert((source, glyph), logical);
    Ok(logical)
}

#[cfg(test)]
pub(crate) fn assert_enclosure(pack: &CompiledGamePack) {
    use std::collections::{BTreeSet, VecDeque};

    let room = &pack.data.maps["MnMinneapolisDispatchOffice"];
    for open in [false, true] {
        let mut blocks = room.blocks.clone();
        if open {
            blocks[36] = 5;
        }
        let mut visited = BTreeSet::from([[11_u16, 6_u16]]);
        let mut queue = VecDeque::from([[11_u16, 6_u16]]);
        while let Some([x, y]) = queue.pop_front() {
            for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
                let (Ok(x), Ok(y)) = (
                    u16::try_from(i32::from(x) + dx),
                    u16::try_from(i32::from(y) + dy),
                ) else {
                    continue;
                };
                if x >= 20 || y >= 16 || visited.contains(&[x, y]) {
                    continue;
                }
                let collision =
                    crate::authored_route_modpack::collision(pack, &blocks, 10, [x, y], TILESET)
                        .expect("private fence native collision");
                if matches!(collision, "FLOOR" | "WARP_CARPET_DOWN" | "LADDER") {
                    visited.insert([x, y]);
                    queue.push_back([x, y]);
                }
            }
        }
        assert_eq!(
            visited.contains(&[15, 5]),
            open,
            "the gate alone controls entry to Ellis"
        );
    }
}
