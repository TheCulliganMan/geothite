//! The two Fen pools share a shoreline vocabulary with the original Johto lakes.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, authored_town_publication, native_tile_png};

const ROOM: &str = "MnLayeredFen";
const WIDTH: usize = 20;
const HEIGHT: usize = 13;
const GROUND: &str =
    include_str!("../../../planning/minnesota/revisions/trail-battles-v01/fen-ground.txt");
const WATER: &str =
    include_str!("../../../planning/minnesota/revisions/trail-battles-v01/fen-water.txt");
const ATLAS: &str = "data/tilesets/johto_metatiles.bin";
const SHEET: &str = "gfx/tilesets/johto.2bpp";

// Johto has upper water corners and a sunlit low rear rim (0x2c). Reflect only
// the silhouettes of the corners to join that rim to the existing side banks.
const SOUTH_CORNERS: [(u8, u8); 4] = [(0x74, 0x5c), (0x75, 0x5d), (0x76, 0x4d), (0x77, 0x4b)];

struct Pools<'a>(Vec<&'a [u8]>);

impl Pools<'_> {
    fn wet(&self, x: i32, y: i32) -> bool {
        y.try_into()
            .ok()
            .and_then(|i: usize| self.0.get(i))
            .and_then(|row| row.get(usize::try_from(x).ok()?))
            == Some(&b'~')
    }

    fn corner(&self, x: i32, y: i32, dx: i32, dy: i32) -> u8 {
        let west = dx < 0;
        let north = dy < 0;
        let q = usize::from(!north) * 2 + usize::from(!west);
        match (self.wet(x + dx, y), self.wet(x, y + dy)) {
            (false, false) => [0x5c, 0x5d, 0x74, 0x75][q],
            (false, true) => {
                if west {
                    0x3d
                } else {
                    0x3b
                }
            }
            (true, false) => {
                if north {
                    0x4c
                } else {
                    0x2c
                }
            }
            (true, true) if !self.wet(x + dx, y + dy) => [0x4d, 0x4b, 0x76, 0x77][q],
            (true, true) => 0x14, // Native animated open-water tile.
        }
    }
}

fn install_corners(pack: &mut CompiledGamePack, original_atlas: &[u8]) -> Result<()> {
    let palette = &mut pack
        .data
        .tilesets
        .get_mut("johto")
        .context("Johto palette")?
        .palette_map;
    let sheet = pack.runtime_files.get_mut(SHEET).context(SHEET)?;
    ensure!(sheet.len() == 256 * 16, "unexpected Johto tile pages");
    for (destination, source) in SOUTH_CORNERS {
        let (from, to) = (usize::from(source) * 16, usize::from(destination) * 16);
        ensure!(
            !original_atlas.contains(&destination) && sheet[to..to + 16].iter().all(|&b| b == 0),
            "Johto Fen corner overlaps existing art"
        );
        for y in 0..8 {
            sheet.copy_within(from + (7 - y) * 2..from + (8 - y) * 2, to + y * 2);
        }
        palette[usize::from(destination)] = palette[usize::from(source)];
    }
    let png = native_tile_png::from_2bpp(sheet)?;
    pack.runtime_files
        .insert("gfx/tilesets/johto.png".into(), png);
    Ok(())
}

pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(ROOM).context(ROOM)?.clone();
    ensure!(
        room.attributes.tileset_name == "johto"
            && (
                usize::from(room.attributes.width),
                usize::from(room.attributes.height)
            ) == (WIDTH, HEIGHT)
            && [
                room.blocks[6 * WIDTH + 6],
                room.blocks[9 * WIDTH + 6],
                room.blocks[6 * WIDTH + 15]
            ] == [0x8a, 0x8b, 0x8c],
        "unexpected previous Fen layout"
    );
    let pools = Pools(WATER.lines().map(str::as_bytes).collect());
    ensure!(
        pools.0.len() == 2 * HEIGHT
            && pools
                .0
                .iter()
                .all(|row| row.len() == 2 * WIDTH && row.iter().all(|&v| matches!(v, b'.' | b'~'))),
        "wrong Fen water map"
    );
    let ground: Vec<Vec<u8>> = GROUND
        .lines()
        .map(|line| {
            line.split_whitespace()
                .map(|id| u8::from_str_radix(id, 16))
                .collect()
        })
        .collect::<Result<_, _>>()?;
    ensure!(
        ground.len() == HEIGHT && ground.iter().all(|row| row.len() == WIDTH),
        "wrong Fen ground map"
    );

    let mut art = pack.runtime_files.get(ATLAS).context(ATLAS)?.clone();
    install_corners(pack, &art)?;
    let bank = pack
        .data
        .tilesets
        .get_mut("johto")
        .context("Johto terrain")?;
    let mut blocks = BTreeMap::new();
    let (native_blocks, remainder) = art.as_chunks::<16>();
    ensure!(remainder.is_empty(), "incomplete Johto metatile");
    for (id, &tiles) in native_blocks.iter().enumerate() {
        let collision = bank
            .collision
            .get(&format!("{id:02x}"))
            .context("Johto walking permissions")?;
        blocks
            .entry((tiles, collision.clone()))
            .or_insert(u16::try_from(id)?);
    }
    for (y, row) in ground.into_iter().enumerate() {
        for (x, dry) in row.into_iter().enumerate() {
            let start = usize::from(dry) * 16;
            let mut tiles: [u8; 16] = art
                .get(start..start + 16)
                .context("Fen ground block")?
                .try_into()?;
            let mut collision = bank
                .collision
                .get(&format!("{dry:02x}"))
                .context("Fen ground collision")?
                .clone();
            for (q, role) in collision.iter_mut().enumerate() {
                let u = i32::try_from(x * 2 + q % 2)?;
                let v = i32::try_from(y * 2 + q / 2)?;
                if !pools.wet(u, v) {
                    continue;
                }
                *role = "WATER".to_owned();
                let first = 8 * (q / 2) + 2 * (q % 2);
                for (i, (dx, dy)) in [(-1, -1), (1, -1), (-1, 1), (1, 1)].into_iter().enumerate() {
                    tiles[first + i / 2 * 4 + i % 2] = pools.corner(u, v, dx, dy);
                }
            }
            let next = u8::try_from(art.len() / 16).context("Johto has no free metatiles")?;
            let id = blocks
                .entry((tiles, collision))
                .or_insert_with_key(|(tiles, roles)| {
                    art.extend_from_slice(tiles);
                    bank.collision.insert(format!("{next:02x}"), roles.clone());
                    u16::from(next)
                });
            room.blocks[y * WIDTH + x] = *id;
        }
    }
    pack.runtime_files.insert(ATLAS.into(), art);
    pack.runtime_files.insert(
        "data/tilesets/johto.json".into(),
        serde_json::to_vec(&bank.collision)?,
    );
    pack.runtime_files.insert(
        "data/tilesets/johto_palette_map.json".into(),
        serde_json::to_vec(&bank.palette_map)?,
    );
    authored_town_publication::replace(pack, room)
}
